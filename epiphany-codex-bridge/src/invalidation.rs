use codex_utils_absolute_path::AbsolutePathBuf;
use notify::RecursiveMode;
use notify::Watcher;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tracing::warn;

use crate::reorient::EpiphanyFreshnessWatcherSnapshot;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpiphanyInvalidationSnapshot {
    pub available: bool,
    pub workspace_root: Option<AbsolutePathBuf>,
    pub observed_at_unix_seconds: Option<i64>,
    pub changed_paths: Vec<PathBuf>,
}

pub fn epiphany_freshness_watcher_snapshot(
    snapshot: &EpiphanyInvalidationSnapshot,
) -> EpiphanyFreshnessWatcherSnapshot<'_> {
    EpiphanyFreshnessWatcherSnapshot {
        available: snapshot.available,
        workspace_root: snapshot.workspace_root.as_ref().map(|path| path.as_path()),
        observed_at_unix_seconds: snapshot.observed_at_unix_seconds,
        changed_paths: snapshot.changed_paths.as_slice(),
    }
}

#[derive(Clone)]
pub struct EpiphanyInvalidationManager {
    state: Arc<AsyncMutex<HashMap<String, WatchEntry>>>,
}

struct WatchEntry {
    workspace_root: AbsolutePathBuf,
    latest: Arc<AsyncMutex<LatestInvalidation>>,
    terminate_tx: oneshot::Sender<oneshot::Sender<()>>,
    _watcher: notify::RecommendedWatcher,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct LatestInvalidation {
    observed_at_unix_seconds: Option<i64>,
    changed_paths: Vec<PathBuf>,
}

impl EpiphanyInvalidationManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }

    pub async fn ensure_thread_watch(&self, thread_id: &str, workspace_root: &AbsolutePathBuf) {
        let existing_entry = {
            let mut state = self.state.lock().await;
            match state.get(thread_id) {
                Some(entry) if &entry.workspace_root == workspace_root => return,
                Some(_) => state.remove(thread_id),
                None => None,
            }
        };

        if let Some(entry) = existing_entry {
            stop_watch_entry(entry).await;
        }

        let latest = Arc::new(AsyncMutex::new(LatestInvalidation::default()));
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<Vec<PathBuf>>();
        let mut watcher =
            match notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
                match result {
                    Ok(event) => {
                        let _ = event_tx.send(event.paths);
                    }
                    Err(err) => warn!("epiphany invalidation watcher event failed: {err}"),
                }
            }) {
                Ok(watcher) => watcher,
                Err(err) => {
                    warn!("epiphany invalidation manager could not create watcher: {err}");
                    return;
                }
            };
        if let Err(err) = watcher.watch(workspace_root.as_path(), RecursiveMode::Recursive) {
            warn!(
                "epiphany invalidation manager could not watch `{}`: {err}",
                workspace_root.display()
            );
            return;
        }
        let (terminate_tx, terminate_rx) = oneshot::channel();
        let workspace_root_path = workspace_root.to_path_buf();

        let entry = WatchEntry {
            workspace_root: workspace_root.clone(),
            latest: Arc::clone(&latest),
            terminate_tx,
            _watcher: watcher,
        };
        self.state.lock().await.insert(thread_id.to_string(), entry);

        tokio::spawn(async move {
            let mut terminate_rx = terminate_rx;
            let mut done_tx = None;

            loop {
                let paths = tokio::select! {
                    biased;
                    result = &mut terminate_rx => {
                        done_tx = result.ok();
                        break;
                    }
                    paths = event_rx.recv() => match paths {
                        Some(paths) => paths,
                        None => break,
                    },
                };

                let observed_at_unix_seconds = unix_now();
                let mut latest = latest.lock().await;
                latest.observed_at_unix_seconds = Some(observed_at_unix_seconds);
                latest.changed_paths =
                    normalize_changed_paths(paths, workspace_root_path.as_path());
            }

            if let Some(done_tx) = done_tx {
                let _ = done_tx.send(());
            }
        });
    }

    pub async fn snapshot(&self, thread_id: &str) -> EpiphanyInvalidationSnapshot {
        let (workspace_root, latest) = {
            let state = self.state.lock().await;
            match state.get(thread_id) {
                Some(entry) => (
                    Some(entry.workspace_root.clone()),
                    Some(Arc::clone(&entry.latest)),
                ),
                None => (None, None),
            }
        };

        let Some(latest) = latest else {
            return EpiphanyInvalidationSnapshot {
                available: false,
                workspace_root: None,
                observed_at_unix_seconds: None,
                changed_paths: Vec::new(),
            };
        };

        let latest = latest.lock().await.clone();
        EpiphanyInvalidationSnapshot {
            available: true,
            workspace_root,
            observed_at_unix_seconds: latest.observed_at_unix_seconds,
            changed_paths: latest.changed_paths,
        }
    }

    pub async fn remove_thread(&self, thread_id: &str) {
        let entry = self.state.lock().await.remove(thread_id);
        if let Some(entry) = entry {
            stop_watch_entry(entry).await;
        }
    }
}

async fn stop_watch_entry(entry: WatchEntry) {
    let (done_tx, done_rx) = oneshot::channel();
    let _ = entry.terminate_tx.send(done_tx);
    let _ = done_rx.await;
}

fn unix_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

fn normalize_changed_paths(paths: Vec<PathBuf>, workspace_root: &std::path::Path) -> Vec<PathBuf> {
    let mut normalized = paths
        .into_iter()
        .filter_map(|path| match path.strip_prefix(workspace_root) {
            Ok(relative) if relative.as_os_str().is_empty() => None,
            Ok(relative) => Some(relative.to_path_buf()),
            Err(_) => Some(path),
        })
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}
