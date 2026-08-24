use anyhow::{Context, Result, anyhow};
use epiphany_tool_adapter::EpiphanyToolInvocationIntent;
use serde_json::{Value, json};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

pub fn execute_epiphany_source(
    intent: &EpiphanyToolInvocationIntent,
    cwd: &Path,
) -> Result<Value> {
    let arguments: Value =
        serde_json::from_str(&intent.arguments_json).context("arguments_json is not valid JSON")?;
    if !arguments.is_object() {
        return Err(anyhow!("epiphany_source arguments must be an object"));
    }
    match intent.tool_name.as_str() {
        "read_file" => read_file(cwd, &arguments),
        "directory_inventory" => directory_inventory(cwd, &arguments),
        "git_show" => git_show(cwd, &arguments),
        other => Err(anyhow!("unknown epiphany_source tool {other:?}")),
    }
}

fn directory_inventory(cwd: &Path, arguments: &Value) -> Result<Value> {
    let requested = required_string(arguments, "path")?;
    let maximum_depth = optional_u64(arguments, "maxDepth")?
        .unwrap_or(3)
        .clamp(0, 8) as usize;
    let maximum_entries = optional_u64(arguments, "maxEntries")?
        .unwrap_or(1_024)
        .clamp(1, 4_096) as usize;
    let maximum_samples = optional_u64(arguments, "maxSamples")?
        .unwrap_or(40)
        .clamp(1, 100) as usize;
    let root = confined_path(cwd, requested)?;
    if !root.is_dir() {
        return Err(anyhow!("directory inventory path is not a directory"));
    }

    let mut pending = vec![(root.clone(), 0usize)];
    let mut entries = Vec::new();
    let mut entry_limit_hit = false;
    let mut depth_limit_hit = false;
    while let Some((directory, depth)) = pending.pop() {
        let mut children = fs::read_dir(&directory)
            .with_context(|| format!("reading directory {}", directory.display()))?
            .collect::<std::io::Result<Vec<_>>>()?;
        children.sort_by_key(|entry| entry.file_name());
        for child in children {
            if entries.len() == maximum_entries {
                entry_limit_hit = true;
                break;
            }
            let path = child.path();
            let metadata = fs::symlink_metadata(&path)
                .with_context(|| format!("reading metadata for {}", path.display()))?;
            let kind = if metadata.file_type().is_symlink() {
                "symlink"
            } else if metadata.is_dir() {
                "directory"
            } else if metadata.is_file() {
                "file"
            } else {
                "other"
            };
            entries.push((path.clone(), kind, metadata.len()));
            if kind == "directory" && depth < maximum_depth {
                pending.push((path, depth + 1));
            } else if kind == "directory"
                && fs::read_dir(&path)
                    .with_context(|| format!("testing directory depth at {}", path.display()))?
                    .next()
                    .transpose()?
                    .is_some()
            {
                depth_limit_hit = true;
            }
        }
        if entry_limit_hit {
            break;
        }
        pending.sort_by(|left, right| right.0.cmp(&left.0));
    }
    entries.sort_by(|left, right| left.0.cmp(&right.0));

    let file_count = entries.iter().filter(|entry| entry.1 == "file").count();
    let directory_count = entries
        .iter()
        .filter(|entry| entry.1 == "directory")
        .count();
    let symlink_count = entries.iter().filter(|entry| entry.1 == "symlink").count();
    let total_file_bytes = entries
        .iter()
        .filter(|entry| entry.1 == "file")
        .map(|entry| entry.2)
        .sum::<u64>();
    let sample = entries
        .iter()
        .take(maximum_samples)
        .map(|(path, kind, bytes)| {
            json!({
                "path": path.strip_prefix(&root).unwrap_or(path).display().to_string(),
                "kind": kind,
                "bytes": if *kind == "file" { Some(*bytes) } else { None },
            })
        })
        .collect::<Vec<_>>();
    let complete = !entry_limit_hit && !depth_limit_hit;
    Ok(json!({
        "path": root.display().to_string(),
        "maxDepth": maximum_depth,
        "maxEntries": maximum_entries,
        "complete": complete,
        "entryLimitHit": entry_limit_hit,
        "depthLimitHit": depth_limit_hit,
        "entryCount": entries.len(),
        "fileCount": file_count,
        "directoryCount": directory_count,
        "symlinkCount": symlink_count,
        "totalFileBytes": total_file_bytes,
        "sample": sample,
    }))
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
        )?;
        assert_eq!(value["content"], "2: two\n3: three");
        assert!(
            execute_epiphany_source(
                &intent("read_file", r#"{"path":"../escape"}"#),
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

    #[test]
    fn inventories_workspace_directory_with_deterministic_totals() -> Result<()> {
        let dir = tempfile::tempdir()?;
        fs::create_dir(dir.path().join("artifacts"))?;
        fs::create_dir(dir.path().join("artifacts/pulse-2"))?;
        fs::create_dir(dir.path().join("artifacts/pulse-1"))?;
        fs::write(dir.path().join("artifacts/pulse-1/a.txt"), b"awake")?;
        fs::write(dir.path().join("artifacts/pulse-2/b.txt"), b"machine")?;
        let value = execute_epiphany_source(
            &intent(
                "directory_inventory",
                r#"{"path":"artifacts","maxDepth":2,"maxEntries":10,"maxSamples":10}"#,
            ),
            dir.path(),
        )?;
        assert_eq!(value["complete"], true);
        assert_eq!(value["entryCount"], 4);
        assert_eq!(value["directoryCount"], 2);
        assert_eq!(value["fileCount"], 2);
        assert_eq!(value["totalFileBytes"], 12);
        assert_eq!(value["sample"][0]["path"], "pulse-1");
        assert!(
            execute_epiphany_source(
                &intent("directory_inventory", r#"{"path":"../escape"}"#),
                dir.path(),
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn directory_inventory_reports_truncation_instead_of_partial_truth() -> Result<()> {
        let dir = tempfile::tempdir()?;
        fs::create_dir(dir.path().join("artifacts"))?;
        for name in ["a", "b", "c"] {
            fs::write(dir.path().join("artifacts").join(name), name)?;
        }
        let value = execute_epiphany_source(
            &intent(
                "directory_inventory",
                r#"{"path":"artifacts","maxEntries":2}"#,
            ),
            dir.path(),
        )?;
        assert_eq!(value["complete"], false);
        assert_eq!(value["entryCount"], 2);

        fs::create_dir(dir.path().join("nested"))?;
        fs::create_dir(dir.path().join("nested/child"))?;
        fs::write(dir.path().join("nested/child/body"), b"hidden by depth")?;
        let depth_limited = execute_epiphany_source(
            &intent(
                "directory_inventory",
                r#"{"path":"nested","maxDepth":0,"maxEntries":10}"#,
            ),
            dir.path(),
        )?;
        assert_eq!(depth_limited["complete"], false);
        assert_eq!(depth_limited["depthLimitHit"], true);
        Ok(())
    }
}
