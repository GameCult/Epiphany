use anyhow::{Context, Result, anyhow};
use epiphany_tool_adapter::EpiphanyToolInvocationIntent;
use reqwest::redirect::Policy;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::time::Duration;

const MAX_PUBLIC_SOURCE_BYTES: usize = 64 * 1024;

pub async fn execute_epiphany_public(intent: &EpiphanyToolInvocationIntent) -> Result<Value> {
    let arguments: Value =
        serde_json::from_str(&intent.arguments_json).context("arguments_json is not valid JSON")?;
    if !arguments.is_object() {
        return Err(anyhow!("epiphany_public arguments must be an object"));
    }
    match intent.tool_name.as_str() {
        "github_file" => github_file(intent, &arguments).await,
        other => Err(anyhow!("unknown epiphany_public tool {other:?}")),
    }
}

async fn github_file(intent: &EpiphanyToolInvocationIntent, arguments: &Value) -> Result<Value> {
    let owner = repository_component(arguments, "owner")?;
    let repository = repository_component(arguments, "repository")?;
    let revision = required_string(arguments, "revision")?;
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(anyhow!(
            "github_file revision must be an immutable 40-hex commit id"
        ));
    }
    let path = required_string(arguments, "path")?;
    validate_repository_path(path)?;
    let maximum = arguments
        .get("maxBytes")
        .map(|value| {
            value
                .as_u64()
                .ok_or_else(|| anyhow!("github_file maxBytes must be unsigned"))
        })
        .transpose()?
        .unwrap_or(32_768)
        .clamp(512, MAX_PUBLIC_SOURCE_BYTES as u64) as usize;
    let revision = revision.to_ascii_lowercase();
    let url = format!("https://raw.githubusercontent.com/{owner}/{repository}/{revision}/{path}");
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .no_proxy()
        .timeout(Duration::from_secs(15))
        .user_agent("epiphany-public-source/0")
        .build()
        .context("building bounded public-source client")?;
    let mut response = client
        .get(&url)
        .send()
        .await
        .context("reading immutable GitHub source")?;
    if !response.status().is_success() {
        return Err(anyhow!(
            "immutable GitHub source returned HTTP {}",
            response.status()
        ));
    }
    if response
        .content_length()
        .is_some_and(|length| length > maximum as u64)
    {
        return Err(anyhow!("immutable GitHub source exceeds maxBytes"));
    }
    let mut bytes = Vec::with_capacity(maximum.min(8_192));
    while let Some(chunk) = response
        .chunk()
        .await
        .context("reading immutable GitHub source body")?
    {
        if bytes.len() + chunk.len() > maximum {
            return Err(anyhow!("immutable GitHub source exceeds maxBytes"));
        }
        bytes.extend_from_slice(&chunk);
    }
    let content = String::from_utf8(bytes.clone())
        .map_err(|_| anyhow!("immutable GitHub source is not UTF-8 text"))?;
    let content_sha256 = format!("{:x}", Sha256::digest(&bytes));
    let evidence_receipt_id = format!("eyes-source-{}", intent.intent_id);
    Ok(json!({
        "provider": "github",
        "repository": format!("github://{owner}/{repository}"),
        "revision": revision,
        "path": path,
        "sourceRef": format!("github://{owner}/{repository}@{revision}/{path}"),
        "contentSha256": content_sha256,
        "evidenceReceiptId": evidence_receipt_id,
        "byteCount": bytes.len(),
        "content": content,
    }))
}

fn repository_component<'a>(arguments: &'a Value, name: &str) -> Result<&'a str> {
    let value = required_string(arguments, name)?;
    if value.len() > 100
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(anyhow!("github_file {name} is invalid"));
    }
    Ok(value)
}

fn validate_repository_path(path: &str) -> Result<()> {
    if path.len() > 512
        || path.starts_with('/')
        || path.split('/').any(|segment| {
            segment.is_empty()
                || matches!(segment, "." | "..")
                || !segment
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        })
    {
        return Err(anyhow!("github_file path is invalid"));
    }
    Ok(())
}

fn required_string<'a>(value: &'a Value, name: &str) -> Result<&'a str> {
    value
        .get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("missing required string argument {name:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_source_identity_requires_immutable_github_shape() {
        let valid = json!({
            "owner": "openai",
            "repository": "openai-openapi",
            "revision": "0123456789abcdef0123456789abcdef01234567",
            "path": "openapi.yaml"
        });
        assert_eq!(repository_component(&valid, "owner").unwrap(), "openai");
        assert!(validate_repository_path("docs/source_file.rs").is_ok());
        assert!(validate_repository_path("../secret").is_err());
        assert!(validate_repository_path("path with spaces").is_err());
    }

    #[tokio::test]
    #[ignore = "live immutable GitHub source proof"]
    async fn reads_exact_public_epiphany_source_with_digest_provenance() -> Result<()> {
        let intent = EpiphanyToolInvocationIntent::new(
            "live-public-source",
            epiphany_tool_adapter::EPIPHANY_TOOL_RUNTIME_ADAPTER_ID,
            "epiphany_public",
            "github_file",
            r#"{"owner":"GameCult","repository":"Epiphany","revision":"43dc865e0f332b82de4b95292d5c600f8b901706","path":"README.md","maxBytes":65536}"#,
            "live-test",
            "Prove exact public source access.",
            "2026-08-11T00:00:00Z",
        );
        let result = execute_epiphany_public(&intent).await?;
        assert_eq!(
            result["sourceRef"],
            "github://GameCult/Epiphany@43dc865e0f332b82de4b95292d5c600f8b901706/README.md"
        );
        assert_eq!(result["contentSha256"].as_str().map(str::len), Some(64));
        assert!(result["byteCount"].as_u64().is_some_and(|count| count > 0));
        assert!(
            result["content"]
                .as_str()
                .is_some_and(|content| content.contains("Epiphany"))
        );
        println!(
            "sourceRef={} contentSha256={} byteCount={}",
            result["sourceRef"], result["contentSha256"], result["byteCount"]
        );
        Ok(())
    }
}
