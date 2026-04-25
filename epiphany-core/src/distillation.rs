use anyhow::Result;
use anyhow::anyhow;
use codex_protocol::protocol::EpiphanyCodeRef;
use codex_protocol::protocol::EpiphanyEvidenceRecord;
use codex_protocol::protocol::EpiphanyObservation;
use sha1::Digest;
use sha1::Sha1;

const SUMMARY_LIMIT: usize = 320;
const TEXT_FINGERPRINT_LIMIT: usize = 4096;

#[derive(Debug, Clone, Default)]
pub struct EpiphanyDistillInput {
    pub source_kind: String,
    pub status: String,
    pub text: String,
    pub subject: Option<String>,
    pub evidence_kind: Option<String>,
    pub code_refs: Vec<EpiphanyCodeRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyDistillProposal {
    pub observation: EpiphanyObservation,
    pub evidence: EpiphanyEvidenceRecord,
}

pub fn distill_observation(input: EpiphanyDistillInput) -> Result<EpiphanyDistillProposal> {
    let source_kind = normalize_required("sourceKind", &input.source_kind)?;
    let status = normalize_required("status", &input.status)?;
    let raw_text = input.text.as_str();
    let text = normalize_required("text", &input.text)?;
    let subject = input
        .subject
        .as_deref()
        .map(normalize_text)
        .filter(|value| !value.is_empty());
    let evidence_kind = input
        .evidence_kind
        .as_deref()
        .map(normalize_text)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default_evidence_kind(&source_kind).to_string());
    let summary = summarize(&source_kind, subject.as_deref(), raw_text, &text);
    let fingerprint = fingerprint(&source_kind, &status, subject.as_deref(), &text);
    let evidence_id = format!("ev-{fingerprint}");
    let observation = EpiphanyObservation {
        id: format!("obs-{fingerprint}"),
        summary: summary.clone(),
        source_kind,
        status: status.clone(),
        code_refs: input.code_refs.clone(),
        evidence_ids: vec![evidence_id.clone()],
    };
    let evidence = EpiphanyEvidenceRecord {
        id: evidence_id,
        kind: evidence_kind,
        status,
        summary,
        code_refs: input.code_refs,
    };

    Ok(EpiphanyDistillProposal {
        observation,
        evidence,
    })
}

fn normalize_required(field: &str, value: &str) -> Result<String> {
    let value = normalize_text(value);
    if value.is_empty() {
        Err(anyhow!("{field} must not be empty"))
    } else {
        Ok(value)
    }
}

fn normalize_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn summarize(
    source_kind: &str,
    subject: Option<&str>,
    raw_text: &str,
    normalized_text: &str,
) -> String {
    let distilled = if should_distill_source_output(source_kind) {
        summarize_source_output(raw_text).unwrap_or_else(|| normalized_text.to_string())
    } else {
        normalized_text.to_string()
    };
    let summary = match subject {
        Some(subject) if !subject.is_empty() => format!("{subject}: {distilled}"),
        _ => distilled,
    };
    truncate_chars(&summary, SUMMARY_LIMIT)
}

fn should_distill_source_output(source_kind: &str) -> bool {
    let lower = source_kind.to_ascii_lowercase();
    lower.contains("tool")
        || lower.contains("command")
        || lower.contains("shell")
        || lower.contains("model")
        || lower.contains("assistant")
}

fn summarize_source_output(text: &str) -> Option<String> {
    let lines = normalized_lines(text);
    if lines.is_empty() {
        return None;
    }

    let mut high_signal = Vec::new();
    let mut warnings = Vec::new();
    for line in &lines {
        if is_high_signal_output_line(line) {
            high_signal.push(line.clone());
        } else if is_warning_output_line(line) {
            warnings.push(line.clone());
        }
    }

    let mut selected = if high_signal.is_empty() {
        warnings
    } else {
        high_signal
    };
    selected.truncate(3);

    if selected.is_empty() {
        selected.extend(lines.into_iter().take(2));
    }

    Some(selected.join(" | "))
}

fn normalized_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(normalize_text)
        .filter(|line| !line.is_empty())
        .collect()
}

fn is_high_signal_output_line(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.contains("exit code")
        || lower.contains("test result")
        || lower.contains("finished")
        || lower.contains("passed")
        || lower.contains("failed")
        || lower.contains("error")
}

fn is_warning_output_line(line: &str) -> bool {
    line.to_ascii_lowercase().contains("warning")
}

fn fingerprint(source_kind: &str, status: &str, subject: Option<&str>, text: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(source_kind.as_bytes());
    hasher.update(b"\0");
    hasher.update(status.as_bytes());
    hasher.update(b"\0");
    hasher.update(subject.unwrap_or_default().as_bytes());
    hasher.update(b"\0");
    hasher.update(truncate_chars(text, TEXT_FINGERPRINT_LIMIT).as_bytes());
    let digest = hasher.finalize();
    format!("{digest:x}").chars().take(12).collect()
}

fn truncate_chars(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }

    let mut truncated = value
        .chars()
        .take(limit.saturating_sub(3))
        .collect::<String>();
    truncated.push_str("...");
    truncated
}

fn default_evidence_kind(source_kind: &str) -> &'static str {
    let lower = source_kind.to_ascii_lowercase();
    if lower.contains("test") || lower.contains("smoke") || lower.contains("verification") {
        "verification"
    } else if lower.contains("tool") || lower.contains("command") || lower.contains("shell") {
        "tool-output"
    } else if lower.contains("model") || lower.contains("assistant") {
        "model-output"
    } else {
        "observation"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn distill_observation_returns_patch_ready_records() {
        let proposal = distill_observation(EpiphanyDistillInput {
            source_kind: "smoke".to_string(),
            status: "ok".to_string(),
            subject: Some("thread/epiphany/update".to_string()),
            text: "  live smoke returned revision 1  ".to_string(),
            code_refs: vec![EpiphanyCodeRef {
                path: PathBuf::from("src/lib.rs"),
                start_line: Some(1),
                end_line: Some(2),
                symbol: Some("demo".to_string()),
                note: None,
            }],
            ..Default::default()
        })
        .expect("distill observation");

        assert!(proposal.observation.id.starts_with("obs-"));
        assert!(proposal.evidence.id.starts_with("ev-"));
        assert_eq!(
            proposal.observation.evidence_ids,
            vec![proposal.evidence.id.clone()]
        );
        assert_eq!(proposal.observation.source_kind, "smoke");
        assert_eq!(proposal.evidence.kind, "verification");
        assert_eq!(
            proposal.observation.summary,
            "thread/epiphany/update: live smoke returned revision 1"
        );
        assert_eq!(proposal.evidence.code_refs.len(), 1);
    }

    #[test]
    fn distill_observation_rejects_empty_text() {
        let err = distill_observation(EpiphanyDistillInput {
            source_kind: "tool".to_string(),
            status: "ok".to_string(),
            text: " \n\t ".to_string(),
            ..Default::default()
        })
        .expect_err("empty text should fail");

        assert!(err.to_string().contains("text must not be empty"));
    }

    #[test]
    fn distill_observation_summarizes_noisy_tool_output() {
        let proposal = distill_observation(EpiphanyDistillInput {
            source_kind: "shell-tool".to_string(),
            status: "ok".to_string(),
            subject: Some("cargo test".to_string()),
            text: r#"
                Compiling epiphany-core v0.1.0
                running 36 tests
                lots of harmless output
                test result: ok. 36 passed; 0 failed; 0 ignored
                Finished `test` profile
            "#
            .to_string(),
            ..Default::default()
        })
        .expect("distill noisy tool output");

        assert_eq!(proposal.evidence.kind, "tool-output");
        assert_eq!(
            proposal.observation.summary,
            "cargo test: test result: ok. 36 passed; 0 failed; 0 ignored | Finished `test` profile"
        );
    }

    #[test]
    fn distill_observation_prioritizes_results_over_generic_warnings() {
        let proposal = distill_observation(EpiphanyDistillInput {
            source_kind: "shell-tool".to_string(),
            status: "ok".to_string(),
            subject: Some("cargo test".to_string()),
            text: r#"
                warning: unused variable in unrelated fixture
                running 3 tests
                test result: ok. 3 passed; 0 failed; 0 ignored; finished in 0.02s
                Finished `test` profile
            "#
            .to_string(),
            ..Default::default()
        })
        .expect("distill noisy tool output");

        assert_eq!(
            proposal.observation.summary,
            "cargo test: test result: ok. 3 passed; 0 failed; 0 ignored; finished in 0.02s | Finished `test` profile"
        );
    }

    #[test]
    fn distill_observation_keeps_warning_when_it_is_the_only_signal() {
        let proposal = distill_observation(EpiphanyDistillInput {
            source_kind: "shell-tool".to_string(),
            status: "ok".to_string(),
            subject: Some("cargo check".to_string()),
            text: r#"
                compiling fixture
                warning: suspicious configuration
                generated intermediate files
            "#
            .to_string(),
            ..Default::default()
        })
        .expect("distill warning-only tool output");

        assert_eq!(
            proposal.observation.summary,
            "cargo check: warning: suspicious configuration"
        );
    }

    #[test]
    fn distill_observation_types_model_output_as_model_evidence() {
        let proposal = distill_observation(EpiphanyDistillInput {
            source_kind: "model-output".to_string(),
            status: "accepted".to_string(),
            text: "The candidate map delta is coherent.".to_string(),
            ..Default::default()
        })
        .expect("distill model output");

        assert_eq!(proposal.evidence.kind, "model-output");
        assert_eq!(
            proposal.observation.summary,
            "The candidate map delta is coherent."
        );
    }
}
