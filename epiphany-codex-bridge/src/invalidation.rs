use codex_core::file_watcher::FileWatcher;
use codex_core::file_watcher::FileWatcherSubscriber;
use codex_core::file_watcher::ThrottledWatchReceiver;
use codex_core::file_watcher::WatchPath;
use codex_core::file_watcher::WatchRegistration;
use codex_utils_absolute_path::AbsolutePathBuf;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::oneshot;
use tracing::warn;

use crate::reorient::EpiphanyFreshnessWatcherSnapshot;

const INVALIDATION_WATCH_DEBOUNCE: Duration = Duration::from_millis(200);

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
    available: bool,
    file_watcher: Arc<FileWatcher>,
    state: Arc<AsyncMutex<HashMap<String, WatchEntry>>>,
}

struct WatchEntry {
    workspace_root: AbsolutePathBuf,
    latest: Arc<AsyncMutex<LatestInvalidation>>,
    terminate_tx: oneshot::Sender<oneshot::Sender<()>>,
    _subscriber: FileWatcherSubscriber,
    _registration: WatchRegistration,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct LatestInvalidation {
    observed_at_unix_seconds: Option<i64>,
    changed_paths: Vec<PathBuf>,
}

impl EpiphanyInvalidationManager {
    pub fn new() -> Self {
        match FileWatcher::new() {
            Ok(file_watcher) => Self {
                available: true,
                file_watcher: Arc::new(file_watcher),
                state: Arc::new(AsyncMutex::new(HashMap::new())),
            },
            Err(err) => {
                warn!("epiphany invalidation manager falling back to noop core watcher: {err}");
                Self {
                    available: false,
                    file_watcher: Arc::new(FileWatcher::noop()),
                    state: Arc::new(AsyncMutex::new(HashMap::new())),
                }
            }
        }
    }

    pub async fn ensure_thread_watch(&self, thread_id: &str, workspace_root: &AbsolutePathBuf) {
        if !self.available {
            return;
        }

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

        let (subscriber, rx) = self.file_watcher.add_subscriber();
        let registration = subscriber.register_paths(vec![WatchPath {
            path: workspace_root.to_path_buf(),
            recursive: true,
        }]);
        let latest = Arc::new(AsyncMutex::new(LatestInvalidation::default()));
        let (terminate_tx, terminate_rx) = oneshot::channel();
        let workspace_root_path = workspace_root.to_path_buf();

        let entry = WatchEntry {
            workspace_root: workspace_root.clone(),
            latest: Arc::clone(&latest),
            terminate_tx,
            _subscriber: subscriber,
            _registration: registration,
        };
        self.state.lock().await.insert(thread_id.to_string(), entry);

        tokio::spawn(async move {
            let mut rx = ThrottledWatchReceiver::new(rx, INVALIDATION_WATCH_DEBOUNCE);
            let mut terminate_rx = terminate_rx;
            let mut done_tx = None;

            loop {
                let event = tokio::select! {
                    biased;
                    result = &mut terminate_rx => {
                        done_tx = result.ok();
                        break;
                    }
                    event = rx.recv() => match event {
                        Some(event) => event,
                        None => break,
                    },
                };

                let observed_at_unix_seconds = unix_now();
                let mut latest = latest.lock().await;
                latest.observed_at_unix_seconds = Some(observed_at_unix_seconds);
                latest.changed_paths =
                    normalize_changed_paths(event.paths, workspace_root_path.as_path());
            }

            if let Some(done_tx) = done_tx {
                let _ = done_tx.send(());
            }
        });
    }

    pub async fn snapshot(&self, thread_id: &str) -> EpiphanyInvalidationSnapshot {
        if !self.available {
            return EpiphanyInvalidationSnapshot {
                available: false,
                workspace_root: None,
                observed_at_unix_seconds: None,
                changed_paths: Vec::new(),
            };
        }

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
