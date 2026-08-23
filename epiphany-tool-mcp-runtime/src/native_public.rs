use anyhow::{Context, Result, anyhow};
use epiphany_core::ImmutableGithubSource;
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
    let source = ImmutableGithubSource::from_components(
        required_string(arguments, "owner")?,
        required_string(arguments, "repository")?,
        required_string(arguments, "revision")?,
        required_string(arguments, "path")?,
    )?;
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
    let url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}",
        source.owner(),
        source.repository_name(),
        source.revision(),
        source.path()
    );
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
        "repository": source.repository_ref(),
        "revision": source.revision(),
        "path": source.path(),
        "sourceRef": source.to_string(),
        "contentSha256": content_sha256,
        "evidenceReceiptId": evidence_receipt_id,
        "byteCount": bytes.len(),
        "content": content,
    }))
}

fn required_string<'a>(value: &'a Value, name: &str) -> Result<&'a str> {
    value
        .get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("missing required string argument {name:?}"))
}
