use std::path::Path;

use super::read_rollout_items_from_rollout;
use codex_core::latest_epiphany_state_from_rollout_items;
use codex_protocol::protocol::EpiphanyThreadState;

pub(super) async fn load_epiphany_state_from_rollout_path(
    rollout_path: &Path,
) -> std::result::Result<Option<EpiphanyThreadState>, String> {
    let items = read_rollout_items_from_rollout(rollout_path)
        .await
        .map_err(|err| {
            format!(
                "failed to load rollout `{}` for Epiphany state: {err}",
                rollout_path.display()
            )
        })?;
    Ok(latest_epiphany_state_from_rollout_items(&items))
}
