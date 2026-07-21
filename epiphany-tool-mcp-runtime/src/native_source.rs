use anyhow::{Context, Result, anyhow};
use epiphany_tool_adapter::EpiphanyToolInvocationIntent;
use serde_json::{Value, json};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

pub fn execute_epiphany_source(
    intent: &EpiphanyToolInvocationIntent,
    store: &Path,
    cwd: &Path,
) -> Result<Value> {
    let arguments: Value =
        serde_json::from_str(&intent.arguments_json).context("arguments_json is not valid JSON")?;
    if !arguments.is_object() {
        return Err(anyhow!("epiphany_source arguments must be an object"));
    }
    match intent.tool_name.as_str() {
        "read_file" => read_file(cwd, &arguments),
        "git_show" => git_show(cwd, &arguments),
        "read_hands_receipt" => read_hands_receipt(store, &arguments),
        other => Err(anyhow!("unknown epiphany_source tool {other:?}")),
    }
}

fn read_file(cwd: &Path, arguments: &Value) -> Result<Value> {
    let requested = required_string(arguments, "path")?;
    let start = optional_u64(arguments, "startLine")?.unwrap_or(1).max(1) as usize;
    let maximum = optional_u64(arguments, "maxLines")?
        .unwrap_or(120)
        .clamp(1, 240) as usize;
    let path = confined_path(cwd, requested)?;
    let reader =
        BufReader::new(File::open(&path).with_context(|| format!("reading {}", path.display()))?);
    let mut content = Vec::new();
    let mut count = 0usize;
    for line in reader.lines() {
        let line = line.with_context(|| format!("reading {}", path.display()))?;
        count += 1;
        if count >= start && content.len() < maximum {
            content.push(format!("{count}: {}", truncate_chars(&line, 8_192)));
        }
    }
    Ok(
        json!({"path":path.display().to_string(),"startLine":start,"maxLines":maximum,"lineCount":count,"content":content.join("\n")}),
    )
}

fn git_show(cwd: &Path, arguments: &Value) -> Result<Value> {
    let revision = required_string(arguments, "revision")?;
    let maximum = optional_u64(arguments, "maxBytes")?
        .unwrap_or(16_000)
        .clamp(512, 24_000) as usize;
    let mut command = Command::new("git");
    command
        .current_dir(cwd)
        .args([
            "show",
            "--stat",
            "--patch",
            "--format=medium",
            revision,
            "--",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(paths) = arguments.get("paths").and_then(Value::as_array) {
        for path in paths.iter().map(|value| {
            value
                .as_str()
                .ok_or_else(|| anyhow!("git_show paths must be strings"))
        }) {
            command.arg(path?);
        }
    }
    let mut child = command.spawn().context("starting git show")?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("git stdout unavailable"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("git stderr unavailable"))?;
    let out = thread::spawn(move || read_retained(stdout, maximum));
    let err = thread::spawn(move || read_retained(stderr, 4_000));
    let status = child.wait().context("waiting for git show")?;
    let stdout = out
        .join()
        .map_err(|_| anyhow!("git stdout reader panicked"))??;
    let stderr = err
        .join()
        .map_err(|_| anyhow!("git stderr reader panicked"))??;
    Ok(
        json!({"revision":revision,"status":status.code(),"success":status.success(),"stdout":stdout,"stderr":stderr}),
    )
}

fn read_retained(mut input: impl Read, limit: usize) -> Result<String> {
    let mut retained = Vec::with_capacity(limit.min(8_192));
    let mut buffer = [0u8; 8_192];
    let mut truncated = false;
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let available = limit.saturating_sub(retained.len());
        retained.extend_from_slice(&buffer[..count.min(available)]);
        truncated |= count > available;
    }
    let mut text = String::from_utf8_lossy(&retained).into_owned();
    if truncated {
        text.push_str("\n...<truncated>");
    }
    Ok(text)
}

fn read_hands_receipt(store: &Path, arguments: &Value) -> Result<Value> {
    let id = required_string(arguments, "receiptId")?;
    match required_string(arguments, "kind")? {
        "patch" => {
            let r = epiphany_core::runtime_hands_patch_receipt(store, id)?
                .ok_or_else(|| anyhow!("Hands patch receipt {id:?} not found"))?;
            serde_json::to_value(r).context("encoding Hands patch receipt")
        }
        "command" => {
            let r = epiphany_core::runtime_hands_command_receipt(store, id)?
                .ok_or_else(|| anyhow!("Hands command receipt {id:?} not found"))?;
            serde_json::to_value(r).context("encoding Hands command receipt")
        }
        "commit" => {
            let r = epiphany_core::runtime_hands_commit_receipt(store, id)?
                .ok_or_else(|| anyhow!("Hands commit receipt {id:?} not found"))?;
            serde_json::to_value(r).context("encoding Hands commit receipt")
        }
        other => Err(anyhow!("unsupported Hands receipt kind {other:?}")),
    }
}

fn confined_path(cwd: &Path, requested: &str) -> Result<PathBuf> {
    let root = cwd
        .canonicalize()
        .with_context(|| format!("canonicalizing cwd {}", cwd.display()))?;
    let requested = PathBuf::from(requested);
    let candidate = (if requested.is_absolute() {
        requested
    } else {
        root.join(requested)
    })
    .canonicalize()
    .context("canonicalizing requested path")?;
    if !candidate.starts_with(&root) {
        return Err(anyhow!("read path escapes workspace"));
    }
    Ok(candidate)
}

fn required_string<'a>(value: &'a Value, name: &str) -> Result<&'a str> {
    value
        .get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow!("missing required string argument {name:?}"))
}
fn optional_u64(value: &Value, name: &str) -> Result<Option<u64>> {
    match value.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_u64()
            .map(Some)
            .ok_or_else(|| anyhow!("argument {name:?} must be unsigned")),
    }
}
fn truncate_chars(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        value.into()
    } else {
        value.chars().take(limit).collect::<String>() + "...<truncated>"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_tool_adapter::EPIPHANY_TOOL_RUNTIME_ADAPTER_ID;
    use std::fs;

    fn intent(tool: &str, arguments: &str) -> EpiphanyToolInvocationIntent {
        EpiphanyToolInvocationIntent::new(
            "i",
            EPIPHANY_TOOL_RUNTIME_ADAPTER_ID,
            "epiphany_source",
            tool,
            arguments,
            "test",
            "test",
            "now",
        )
    }

    #[test]
    fn reads_only_bounded_workspace_slice() -> Result<()> {
        let dir = tempfile::tempdir()?;
        fs::write(dir.path().join("body.txt"), "one\ntwo\nthree\nfour\n")?;
        let value = execute_epiphany_source(
            &intent(
                "read_file",
                r#"{"path":"body.txt","startLine":2,"maxLines":2}"#,
            ),
            dir.path(),
            dir.path(),
        )?;
        assert_eq!(value["content"], "2: two\n3: three");
        assert!(
            execute_epiphany_source(
                &intent("read_file", r#"{"path":"../escape"}"#),
                dir.path(),
                dir.path()
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn retained_reader_discards_excess_bytes() -> Result<()> {
        assert_eq!(read_retained(&b"abcdef"[..], 3)?, "abc\n...<truncated>");
        Ok(())
    }
}
