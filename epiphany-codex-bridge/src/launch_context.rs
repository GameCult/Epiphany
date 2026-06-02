use chrono::SecondsFormat;
use chrono::Utc;
use epiphany_core::EpiphanyMemoryContextPacket;
use epiphany_core::EpiphanyPromptContextInput;
use epiphany_core::EpiphanyThreadState;
use epiphany_core::query_epiphany_local_verse_context;
use epiphany_core::render_epiphany_prompt_context;
use epiphany_core::seed_epiphany_local_verse_context;
use std::path::Path;
use std::path::PathBuf;

pub const EPIPHANY_LOCAL_VERSE_RUNTIME_ID: &str = "epiphany-local";

pub fn local_verse_store_path(runtime_store_path: &Path) -> PathBuf {
    runtime_store_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("local-verse.ccmp")
}

pub fn role_launch_context_focus(state: &EpiphanyThreadState, role_label: &str) -> String {
    let objective = state
        .objective
        .as_deref()
        .map(str::trim)
        .filter(|objective| !objective.is_empty())
        .unwrap_or("Epiphany worker launch");
    format!("Launch `{role_label}` worker for: {objective}")
}

pub fn reorient_launch_context_focus(state: &EpiphanyThreadState, next_action: &str) -> String {
    let objective = state
        .objective
        .as_deref()
        .map(str::trim)
        .filter(|objective| !objective.is_empty())
        .unwrap_or("Epiphany reorientation");
    format!("Launch reorientation worker for: {objective}. Next action: {next_action}")
}

pub fn render_launch_dynamic_prompt_context(
    runtime_store_path: &Path,
    state: &EpiphanyThreadState,
    focus: String,
) -> Result<String, String> {
    let local_verse_store = local_verse_store_path(runtime_store_path);
    seed_epiphany_local_verse_context(
        &local_verse_store,
        EPIPHANY_LOCAL_VERSE_RUNTIME_ID,
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    )
    .map_err(|error| {
        format!(
            "failed to seed local Verse context store {}: {error}",
            local_verse_store.display()
        )
    })?;
    let local_verse =
        query_epiphany_local_verse_context(&local_verse_store, EPIPHANY_LOCAL_VERSE_RUNTIME_ID)
            .map_err(|error| {
                format!(
                    "failed to query local Verse context store {}: {error}",
                    local_verse_store.display()
                )
            })?;
    let memory_context = EpiphanyMemoryContextPacket {
        id: format!("memctx-launch-state-rev-{}", state.revision),
        query_id: "bridge-launch-local-verse-only".to_string(),
        warnings: vec![
            "Semantic memory graph context is not connected to this bridge launch path yet; this packet carries local Verse context only.".to_string(),
        ],
        ..Default::default()
    };
    Ok(render_epiphany_prompt_context(
        &EpiphanyPromptContextInput {
            focus,
            local_verse,
            memory_context,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use uuid::Uuid;

    #[test]
    fn launch_context_renders_local_verse_packet() -> anyhow::Result<()> {
        let temp =
            std::env::temp_dir().join(format!("epiphany-bridge-launch-context-{}", Uuid::new_v4()));
        fs::create_dir_all(&temp)?;
        let runtime_store = temp.join("runtime-spine.msgpack");
        let state = EpiphanyThreadState {
            revision: 7,
            objective: Some("Test launch context.".to_string()),
            ..Default::default()
        };

        let rendered = render_launch_dynamic_prompt_context(
            &runtime_store,
            &state,
            role_launch_context_focus(&state, "modeling"),
        )
        .map_err(anyhow::Error::msg)?;

        assert!(rendered.contains("<epiphany_dynamic_context>"));
        assert!(rendered.contains("Test launch context."));
        assert!(rendered.contains("Odin"));
        assert!(rendered.contains("Yggdrasil"));
        assert!(rendered.contains("local Verse context only"));
        assert!(local_verse_store_path(&runtime_store).exists());
        fs::remove_dir_all(&temp)?;
        Ok(())
    }
}
