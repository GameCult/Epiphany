use chrono::SecondsFormat;
use chrono::Utc;
use epiphany_core::EpiphanyMemoryContextPacket;
use epiphany_core::EpiphanyMemoryContextQuery;
use epiphany_core::EpiphanyMemoryProfile;
use epiphany_core::EpiphanyPromptContextInput;
use epiphany_core::EpiphanyThreadState;
use epiphany_core::load_memory_graph_snapshot;
use epiphany_core::memory_graph_from_epiphany_graphs;
use epiphany_core::plan_memory_graph_context_cut;
use epiphany_core::query_epiphany_local_verse_context;
use epiphany_core::render_epiphany_prompt_context;
use epiphany_core::seed_epiphany_local_verse_context;
use epiphany_core::write_memory_graph_snapshot;
use std::path::Path;
use std::path::PathBuf;

pub const EPIPHANY_LOCAL_VERSE_RUNTIME_ID: &str = "epiphany-local";

pub fn local_verse_store_path(runtime_store_path: &Path) -> PathBuf {
    sibling_state_store_path(runtime_store_path, "local-verse.ccmp")
}

pub fn memory_graph_store_path(runtime_store_path: &Path) -> PathBuf {
    sibling_state_store_path(runtime_store_path, "memory-graph.msgpack")
}

fn sibling_state_store_path(runtime_store_path: &Path, filename: &str) -> PathBuf {
    runtime_store_path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(filename)
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
    let memory_context =
        launch_memory_context(runtime_store_path, state, focus.as_str()).map_err(|error| {
            format!(
                "failed to build launch memory context beside {}: {error}",
                runtime_store_path.display()
            )
        })?;
    Ok(render_epiphany_prompt_context(
        &EpiphanyPromptContextInput {
            focus,
            local_verse,
            memory_context,
        },
    ))
}

fn launch_memory_context(
    runtime_store_path: &Path,
    state: &EpiphanyThreadState,
    focus: &str,
) -> Result<EpiphanyMemoryContextPacket, String> {
    let memory_graph_store = memory_graph_store_path(runtime_store_path);
    let (snapshot, refreshed_from_state) = match load_memory_graph_snapshot(&memory_graph_store)
        .map_err(|error| {
            format!(
                "failed to load memory graph store {}: {error}",
                memory_graph_store.display()
            )
        })? {
        Some(snapshot) => (snapshot, false),
        None => {
            let snapshot = memory_graph_from_epiphany_graphs(
                format!("bridge-launch-state-rev-{}", state.revision),
                &state.graphs,
            );
            write_memory_graph_snapshot(&memory_graph_store, &snapshot).map_err(|error| {
                format!(
                    "failed to write memory graph store {} from thread state: {error}",
                    memory_graph_store.display()
                )
            })?;
            (snapshot, true)
        }
    };

    let mut packet = plan_memory_graph_context_cut(
        &snapshot,
        &EpiphanyMemoryContextQuery {
            id: format!("bridge-launch-query-state-rev-{}", state.revision),
            profile: Some(EpiphanyMemoryProfile::RepoArchitecture),
            text: Some(focus.to_string()),
            budget: Some(5),
            ..Default::default()
        },
    );
    if refreshed_from_state {
        packet.warnings.push(format!(
            "Memory graph store was refreshed from current thread-state repo graph at {}.",
            memory_graph_store.display()
        ));
    }
    if packet.nodes.is_empty() && packet.summaries.is_empty() {
        packet.warnings.push(
            "Memory graph context is empty for this launch focus; the accepted repo graph may be thin or stale.".to_string(),
        );
    }
    Ok(packet)
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
        assert!(rendered.contains("Memory graph"));
        assert!(local_verse_store_path(&runtime_store).exists());
        assert!(memory_graph_store_path(&runtime_store).exists());
        fs::remove_dir_all(&temp)?;
        Ok(())
    }
}
