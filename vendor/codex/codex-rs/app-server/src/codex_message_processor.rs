mod auth_routes;
mod catalog_routes;
mod command_routes;
mod epiphany_automation;
mod epiphany_mutation_routes;
mod epiphany_read_routes;
mod mcp_routes;
mod realtime_routes;
mod review_routes;
mod thread_admin_routes;
mod thread_read_routes;

use self::auth_routes::ActiveLogin;
pub(crate) use self::epiphany_automation::maybe_run_epiphany_coordinator_automation_for_turn_boundary;
pub(crate) use self::epiphany_automation::maybe_run_epiphany_pre_compaction_checkpoint_intervention_for_token_count;
use crate::bespoke_event_handling::apply_bespoke_event_handling;
use crate::bespoke_event_handling::maybe_emit_hook_prompt_item_completed;
use crate::command_exec::CommandExecManager;
use crate::command_exec::StartCommandExecParams;
use crate::config_manager::ConfigManager;
use crate::error_code::INPUT_TOO_LARGE_ERROR_CODE;
use crate::error_code::INTERNAL_ERROR_CODE;
use crate::error_code::INVALID_PARAMS_ERROR_CODE;
use crate::error_code::INVALID_REQUEST_ERROR_CODE;
use crate::fuzzy_file_search::FuzzyFileSearchSession;
use crate::fuzzy_file_search::run_fuzzy_file_search;
use crate::fuzzy_file_search::start_fuzzy_file_search_session;
use crate::models::supported_models;
use crate::outgoing_message::ConnectionId;
use crate::outgoing_message::ConnectionRequestId;
use crate::outgoing_message::OutgoingMessageSender;
use crate::outgoing_message::RequestContext;
use crate::outgoing_message::ThreadScopedOutgoingMessageSender;
use crate::thread_status::ThreadWatchManager;
use crate::thread_status::resolve_thread_status;
use chrono::DateTime;
use chrono::Duration as ChronoDuration;
use chrono::SecondsFormat;
use chrono::Utc;
use codex_analytics::AnalyticsEventsClient;
use codex_analytics::AnalyticsJsonRpcError;
use codex_analytics::InputError;
use codex_analytics::TurnSteerRequestError;
use codex_app_server_protocol::Account;
use codex_app_server_protocol::AccountLoginCompletedNotification;
use codex_app_server_protocol::AccountUpdatedNotification;
use codex_app_server_protocol::AddCreditsNudgeCreditType;
use codex_app_server_protocol::AddCreditsNudgeEmailStatus;
use codex_app_server_protocol::AskForApproval;
use codex_app_server_protocol::AuthMode;
use codex_app_server_protocol::AuthMode as CoreAuthMode;
use codex_app_server_protocol::CancelLoginAccountParams;
use codex_app_server_protocol::CancelLoginAccountResponse;
use codex_app_server_protocol::CancelLoginAccountStatus;
use codex_app_server_protocol::ClientRequest;
use codex_app_server_protocol::ClientResponse;
use codex_app_server_protocol::CodexErrorInfo;
use codex_app_server_protocol::CollaborationModeListParams;
use codex_app_server_protocol::CollaborationModeListResponse;
use codex_app_server_protocol::CommandExecParams;
use codex_app_server_protocol::CommandExecResizeParams;
use codex_app_server_protocol::CommandExecTerminateParams;
use codex_app_server_protocol::CommandExecWriteParams;
use codex_app_server_protocol::ConversationGitInfo;
use codex_app_server_protocol::ConversationSummary;
use codex_app_server_protocol::DynamicToolSpec as ApiDynamicToolSpec;
use codex_app_server_protocol::ExperimentalFeature as ApiExperimentalFeature;
use codex_app_server_protocol::ExperimentalFeatureListParams;
use codex_app_server_protocol::ExperimentalFeatureListResponse;
use codex_app_server_protocol::ExperimentalFeatureStage as ApiExperimentalFeatureStage;
use codex_app_server_protocol::FeedbackUploadParams;
use codex_app_server_protocol::FeedbackUploadResponse;
use codex_app_server_protocol::FuzzyFileSearchParams;
use codex_app_server_protocol::FuzzyFileSearchResponse;
use codex_app_server_protocol::FuzzyFileSearchSessionStartParams;
use codex_app_server_protocol::FuzzyFileSearchSessionStartResponse;
use codex_app_server_protocol::FuzzyFileSearchSessionStopParams;
use codex_app_server_protocol::FuzzyFileSearchSessionStopResponse;
use codex_app_server_protocol::FuzzyFileSearchSessionUpdateParams;
use codex_app_server_protocol::FuzzyFileSearchSessionUpdateResponse;
use codex_app_server_protocol::GetAccountParams;
use codex_app_server_protocol::GetAccountRateLimitsResponse;
use codex_app_server_protocol::GetAccountResponse;
use codex_app_server_protocol::GetAuthStatusParams;
use codex_app_server_protocol::GetAuthStatusResponse;
use codex_app_server_protocol::GetConversationSummaryParams;
use codex_app_server_protocol::GetConversationSummaryResponse;
use codex_app_server_protocol::GitDiffToRemoteResponse;
use codex_app_server_protocol::GitInfo as ApiGitInfo;
use codex_app_server_protocol::JSONRPCErrorError;
use codex_app_server_protocol::ListMcpServerStatusParams;
use codex_app_server_protocol::ListMcpServerStatusResponse;
use codex_app_server_protocol::LoginAccountParams;
use codex_app_server_protocol::LoginAccountResponse;
use codex_app_server_protocol::LoginApiKeyParams;
use codex_app_server_protocol::LogoutAccountResponse;
use codex_app_server_protocol::McpResourceReadParams;
use codex_app_server_protocol::McpResourceReadResponse;
use codex_app_server_protocol::McpServerOauthLoginCompletedNotification;
use codex_app_server_protocol::McpServerOauthLoginParams;
use codex_app_server_protocol::McpServerOauthLoginResponse;
use codex_app_server_protocol::McpServerRefreshResponse;
use codex_app_server_protocol::McpServerStatus;
use codex_app_server_protocol::McpServerStatusDetail;
use codex_app_server_protocol::McpServerToolCallParams;
use codex_app_server_protocol::McpServerToolCallResponse;
use codex_app_server_protocol::MemoryResetResponse;
use codex_app_server_protocol::MockExperimentalMethodParams;
use codex_app_server_protocol::MockExperimentalMethodResponse;
use codex_app_server_protocol::ModelListParams;
use codex_app_server_protocol::ModelListResponse;
use codex_app_server_protocol::PermissionProfile as ApiPermissionProfile;
use codex_app_server_protocol::RequestId;
use codex_app_server_protocol::ReviewDelivery as ApiReviewDelivery;
use codex_app_server_protocol::ReviewStartParams;
use codex_app_server_protocol::ReviewStartResponse;
use codex_app_server_protocol::ReviewTarget as ApiReviewTarget;
use codex_app_server_protocol::SandboxMode;
use codex_app_server_protocol::SendAddCreditsNudgeEmailParams;
use codex_app_server_protocol::SendAddCreditsNudgeEmailResponse;
use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::ServerRequestResolvedNotification;
use codex_app_server_protocol::SortDirection;
use codex_app_server_protocol::Thread;
use codex_app_server_protocol::ThreadApproveGuardianDeniedActionParams;
use codex_app_server_protocol::ThreadApproveGuardianDeniedActionResponse;
use codex_app_server_protocol::ThreadArchiveParams;
use codex_app_server_protocol::ThreadArchiveResponse;
use codex_app_server_protocol::ThreadArchivedNotification;
use codex_app_server_protocol::ThreadBackgroundTerminalsCleanParams;
use codex_app_server_protocol::ThreadBackgroundTerminalsCleanResponse;
use codex_app_server_protocol::ThreadClosedNotification;
use codex_app_server_protocol::ThreadCompactStartParams;
use codex_app_server_protocol::ThreadCompactStartResponse;
use codex_app_server_protocol::ThreadDecrementElicitationParams;
use codex_app_server_protocol::ThreadDecrementElicitationResponse;
use codex_app_server_protocol::ThreadForkParams;
use codex_app_server_protocol::ThreadForkResponse;
use codex_app_server_protocol::ThreadIncrementElicitationParams;
use codex_app_server_protocol::ThreadIncrementElicitationResponse;
use codex_app_server_protocol::ThreadInjectItemsParams;
use codex_app_server_protocol::ThreadInjectItemsResponse;
use codex_app_server_protocol::ThreadItem;
use codex_app_server_protocol::ThreadListCwdFilter;
use codex_app_server_protocol::ThreadListParams;
use codex_app_server_protocol::ThreadListResponse;
use codex_app_server_protocol::ThreadLoadedListParams;
use codex_app_server_protocol::ThreadLoadedListResponse;
use codex_app_server_protocol::ThreadMemoryModeSetParams;
use codex_app_server_protocol::ThreadMemoryModeSetResponse;
use codex_app_server_protocol::ThreadMetadataGitInfoUpdateParams;
use codex_app_server_protocol::ThreadMetadataUpdateParams;
use codex_app_server_protocol::ThreadMetadataUpdateResponse;
use codex_app_server_protocol::ThreadNameUpdatedNotification;
use codex_app_server_protocol::ThreadReadParams;
use codex_app_server_protocol::ThreadReadResponse;
use codex_app_server_protocol::ThreadRealtimeAppendAudioParams;
use codex_app_server_protocol::ThreadRealtimeAppendAudioResponse;
use codex_app_server_protocol::ThreadRealtimeAppendTextParams;
use codex_app_server_protocol::ThreadRealtimeAppendTextResponse;
use codex_app_server_protocol::ThreadRealtimeListVoicesParams;
use codex_app_server_protocol::ThreadRealtimeListVoicesResponse;
use codex_app_server_protocol::ThreadRealtimeStartParams;
use codex_app_server_protocol::ThreadRealtimeStartResponse;
use codex_app_server_protocol::ThreadRealtimeStartTransport;
use codex_app_server_protocol::ThreadRealtimeStopParams;
use codex_app_server_protocol::ThreadRealtimeStopResponse;
use codex_app_server_protocol::ThreadResumeParams;
use codex_app_server_protocol::ThreadResumeResponse;
use codex_app_server_protocol::ThreadRollbackParams;
use codex_app_server_protocol::ThreadSetNameParams;
use codex_app_server_protocol::ThreadSetNameResponse;
use codex_app_server_protocol::ThreadShellCommandParams;
use codex_app_server_protocol::ThreadShellCommandResponse;
use codex_app_server_protocol::ThreadSortKey;
use codex_app_server_protocol::ThreadSourceKind;
use codex_app_server_protocol::ThreadStartParams;
use codex_app_server_protocol::ThreadStartResponse;
use codex_app_server_protocol::ThreadStartedNotification;
use codex_app_server_protocol::ThreadStatus;
use codex_app_server_protocol::ThreadTurnsListParams;
use codex_app_server_protocol::ThreadTurnsListResponse;
use codex_app_server_protocol::ThreadUnarchiveParams;
use codex_app_server_protocol::ThreadUnarchiveResponse;
use codex_app_server_protocol::ThreadUnarchivedNotification;
use codex_app_server_protocol::ThreadUnsubscribeParams;
use codex_app_server_protocol::ThreadUnsubscribeResponse;
use codex_app_server_protocol::ThreadUnsubscribeStatus;
use codex_app_server_protocol::Turn;
use codex_app_server_protocol::TurnError;
use codex_app_server_protocol::TurnInterruptParams;
use codex_app_server_protocol::TurnInterruptResponse;
use codex_app_server_protocol::TurnStartParams;
use codex_app_server_protocol::TurnStartResponse;
use codex_app_server_protocol::TurnStatus;
use codex_app_server_protocol::TurnSteerParams;
use codex_app_server_protocol::TurnSteerResponse;
use codex_app_server_protocol::UserInput as V2UserInput;
use codex_app_server_protocol::WindowsSandboxSetupCompletedNotification;
use codex_app_server_protocol::WindowsSandboxSetupMode;
use codex_app_server_protocol::WindowsSandboxSetupStartParams;
use codex_app_server_protocol::WindowsSandboxSetupStartResponse;
use codex_app_server_protocol::build_turns_from_rollout_items;
use codex_arg0::Arg0DispatchPaths;
use codex_backend_client::AddCreditsNudgeCreditType as BackendAddCreditsNudgeCreditType;
use codex_backend_client::Client as BackendClient;
use codex_config::types::McpServerTransportConfig;
use codex_core::CodexThread;
use codex_core::CodexThreadTurnContextOverrides;
use codex_core::ForkSnapshot;
use codex_core::NewThread;
use codex_core::RolloutRecorder;
use codex_core::SessionMeta;
use codex_core::SteerInputError;
use codex_core::ThreadConfigSnapshot;
use codex_core::ThreadManager;
use codex_core::clear_memory_roots_contents;
use codex_core::config::Config;
use codex_core::config::ConfigOverrides;
use codex_core::config::NetworkProxyAuditMetadata;
use codex_core::config_loader::CloudRequirementsLoadError;
use codex_core::config_loader::CloudRequirementsLoadErrorCode;
use codex_core::config_loader::project_trust_key;
use codex_core::exec::ExecCapturePolicy;
use codex_core::exec::ExecExpiration;
use codex_core::exec::ExecParams;
use codex_core::exec_env::create_env;
use codex_core::find_archived_thread_path_by_id_str;
use codex_core::find_thread_name_by_id;
use codex_core::find_thread_names_by_ids;
use codex_core::find_thread_path_by_id_str;
use codex_core::path_utils;
use codex_core::read_head_for_summary;
use codex_core::read_session_meta_line;
use codex_core::sandboxing::SandboxPermissions;
use codex_core::windows_sandbox::WindowsSandboxLevelExt;
use codex_core::windows_sandbox::WindowsSandboxSetupMode as CoreWindowsSandboxSetupMode;
use codex_core::windows_sandbox::WindowsSandboxSetupRequest;
use codex_exec_server::LOCAL_FS;
use codex_features::FEATURES;
use codex_features::Feature;
use codex_features::Stage;
use codex_feedback::CodexFeedback;
use codex_feedback::FeedbackUploadOptions;
use codex_git_utils::git_diff_to_remote;
use codex_git_utils::resolve_root_git_project_for_trust;
use codex_login::AuthManager;
use codex_login::CLIENT_ID;
use codex_login::CodexAuth;
use codex_login::ServerOptions as LoginServerOptions;
use codex_login::ShutdownHandle;
use codex_login::auth::login_with_chatgpt_auth_tokens;
use codex_login::complete_device_code_login;
use codex_login::login_with_api_key;
use codex_login::request_device_code;
use codex_login::run_login_server;
use codex_mcp::McpRuntimeEnvironment;
use codex_mcp::McpServerStatusSnapshot;
use codex_mcp::McpSnapshotDetail;
use codex_mcp::collect_mcp_server_status_snapshot_with_detail;
use codex_mcp::discover_supported_scopes;
use codex_mcp::effective_mcp_servers;
use codex_mcp::read_mcp_resource as read_mcp_resource_without_thread;
use codex_mcp::resolve_oauth_scopes;
use codex_models_manager::collaboration_mode_presets::CollaborationModesConfig;
use codex_protocol::ThreadId;
use codex_protocol::config_types::CollaborationMode;
use codex_protocol::config_types::ForcedLoginMethod;
use codex_protocol::config_types::Personality;
use codex_protocol::config_types::TrustLevel;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::dynamic_tools::DynamicToolSpec as CoreDynamicToolSpec;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::items::TurnItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::AgentStatus;
use codex_protocol::protocol::ConversationAudioParams;
use codex_protocol::protocol::ConversationStartParams;
use codex_protocol::protocol::ConversationStartTransport;
use codex_protocol::protocol::ConversationTextParams;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::GitInfo as CoreGitInfo;
use codex_protocol::protocol::InitialHistory;
use codex_protocol::protocol::McpAuthStatus as CoreMcpAuthStatus;
use codex_protocol::protocol::McpServerRefreshConfig;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::RateLimitSnapshot as CoreRateLimitSnapshot;
use codex_protocol::protocol::RealtimeVoicesList;
use codex_protocol::protocol::ReviewDelivery as CoreReviewDelivery;
use codex_protocol::protocol::ReviewRequest;
use codex_protocol::protocol::ReviewTarget as CoreReviewTarget;
use codex_protocol::protocol::RolloutItem;
use codex_protocol::protocol::SessionConfiguredEvent;
use codex_protocol::protocol::SessionMetaLine;
use codex_protocol::protocol::TurnEnvironmentSelection;
use codex_protocol::protocol::USER_MESSAGE_BEGIN;
use codex_protocol::protocol::W3cTraceContext;
use codex_protocol::user_input::MAX_USER_INPUT_TEXT_CHARS;
use codex_protocol::user_input::UserInput as CoreInputItem;
use codex_rmcp_client::perform_oauth_login_return_url;
use codex_rollout::state_db::StateDbHandle;
use codex_rollout::state_db::get_state_db;
use codex_rollout::state_db::reconcile_rollout;
use codex_state::StateRuntime;
use codex_state::ThreadMetadata;
use codex_state::ThreadMetadataBuilder;
use codex_state::log_db::LogDbLayer;
use codex_thread_store::ArchiveThreadParams as StoreArchiveThreadParams;
use codex_thread_store::ListThreadsParams as StoreListThreadsParams;
use codex_thread_store::LocalThreadStore;
use codex_thread_store::ReadThreadParams as StoreReadThreadParams;
use codex_thread_store::RemoteThreadStore;
use codex_thread_store::SortDirection as StoreSortDirection;
use codex_thread_store::StoredThread;
use codex_thread_store::ThreadMetadataPatch as StoreThreadMetadataPatch;
use codex_thread_store::ThreadSortKey as StoreThreadSortKey;
use codex_thread_store::ThreadStore;
use codex_thread_store::ThreadStoreError;
use codex_thread_store::UpdateThreadMetadataParams as StoreUpdateThreadMetadataParams;
use codex_utils_absolute_path::AbsolutePathBuf;
use codex_utils_pty::DEFAULT_OUTPUT_BYTES_CAP;
use epiphany_codex_bridge::invalidation::EpiphanyInvalidationManager;
use epiphany_codex_bridge::state::live_thread_epiphany_state;
use epiphany_codex_bridge::state::load_epiphany_state_from_rollout_path;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Error as IoError;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use toml::Value as TomlValue;
use tracing::Instrument;
use tracing::error;
use tracing::info;
use tracing::warn;
use uuid::Uuid;

mod token_usage_replay;
mod utility_routes;

use crate::filters::compute_source_filters;
use crate::filters::source_kind_matches;
use crate::thread_state::ThreadListenerCommand;
use crate::thread_state::ThreadState;
use crate::thread_state::ThreadStateManager;
use token_usage_replay::latest_token_usage_info_from_rollout_path;
use token_usage_replay::latest_token_usage_turn_id_for_thread_path;
use token_usage_replay::latest_token_usage_turn_id_from_rollout_items;
use token_usage_replay::latest_token_usage_turn_id_from_rollout_path;
use token_usage_replay::send_thread_token_usage_update_to_connection;

const THREAD_LIST_DEFAULT_LIMIT: usize = 25;
const THREAD_LIST_MAX_LIMIT: usize = 100;
const THREAD_TURNS_DEFAULT_LIMIT: usize = 25;
const THREAD_TURNS_MAX_LIMIT: usize = 100;

struct ThreadListFilters {
    model_providers: Option<Vec<String>>,
    source_kinds: Option<Vec<ThreadSourceKind>>,
    archived: bool,
    cwd_filters: Option<Vec<PathBuf>>,
    search_term: Option<String>,
    use_state_db_only: bool,
}

const THREAD_UNLOADING_DELAY: Duration = Duration::from_secs(30 * 60);

enum ThreadShutdownResult {
    Complete,
    SubmitFailed,
    TimedOut,
}

enum ThreadReadViewError {
    InvalidRequest(String),
    Internal(String),
}

/// Handles JSON-RPC messages for Codex threads (and legacy conversation APIs).
pub(crate) struct CodexMessageProcessor {
    auth_manager: Arc<AuthManager>,
    thread_manager: Arc<ThreadManager>,
    outgoing: Arc<OutgoingMessageSender>,
    analytics_events_client: AnalyticsEventsClient,
    arg0_paths: Arg0DispatchPaths,
    config: Arc<Config>,
    thread_store: Arc<dyn ThreadStore>,
    config_manager: ConfigManager,
    active_login: Arc<Mutex<Option<ActiveLogin>>>,
    pending_thread_unloads: Arc<Mutex<HashSet<ThreadId>>>,
    thread_state_manager: ThreadStateManager,
    thread_watch_manager: ThreadWatchManager,
    epiphany_invalidation_manager: EpiphanyInvalidationManager,
    command_exec_manager: CommandExecManager,
    pending_fuzzy_searches: Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>,
    fuzzy_search_sessions: Arc<Mutex<HashMap<String, FuzzyFileSearchSession>>>,
    background_tasks: TaskTracker,
    feedback: CodexFeedback,
    log_db: Option<LogDbLayer>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum ApiVersion {
    #[allow(dead_code)]
    V1,
    #[default]
    V2,
}

#[derive(Clone)]
struct ListenerTaskContext {
    thread_manager: Arc<ThreadManager>,
    thread_state_manager: ThreadStateManager,
    outgoing: Arc<OutgoingMessageSender>,
    pending_thread_unloads: Arc<Mutex<HashSet<ThreadId>>>,
    analytics_events_client: AnalyticsEventsClient,
    general_analytics_enabled: bool,
    thread_watch_manager: ThreadWatchManager,
    epiphany_invalidation_manager: EpiphanyInvalidationManager,
    fallback_model_provider: String,
    codex_home: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EnsureConversationListenerResult {
    Attached,
    ConnectionClosed,
}

struct UnloadingState {
    delay: Duration,
    has_subscribers_rx: watch::Receiver<bool>,
    has_subscribers: (bool, Instant),
    thread_status_rx: watch::Receiver<ThreadStatus>,
    is_active: (bool, Instant),
}

impl UnloadingState {
    async fn new(
        listener_task_context: &ListenerTaskContext,
        thread_id: ThreadId,
        delay: Duration,
    ) -> Option<Self> {
        let has_subscribers_rx = listener_task_context
            .thread_state_manager
            .subscribe_to_has_connections(thread_id)
            .await?;
        let thread_status_rx = listener_task_context
            .thread_watch_manager
            .subscribe(thread_id)
            .await?;
        let has_subscribers = (*has_subscribers_rx.borrow(), Instant::now());
        let is_active = (
            matches!(*thread_status_rx.borrow(), ThreadStatus::Active { .. }),
            Instant::now(),
        );
        Some(Self {
            delay,
            has_subscribers_rx,
            thread_status_rx,
            has_subscribers,
            is_active,
        })
    }

    fn unloading_target(&self) -> Option<Instant> {
        match (self.has_subscribers, self.is_active) {
            ((false, has_no_subscribers_since), (false, is_inactive_since)) => {
                Some(std::cmp::max(has_no_subscribers_since, is_inactive_since) + self.delay)
            }
            _ => None,
        }
    }

    fn sync_receiver_values(&mut self) {
        let has_subscribers = *self.has_subscribers_rx.borrow();
        if self.has_subscribers.0 != has_subscribers {
            self.has_subscribers = (has_subscribers, Instant::now());
        }

        let is_active = matches!(*self.thread_status_rx.borrow(), ThreadStatus::Active { .. });
        if self.is_active.0 != is_active {
            self.is_active = (is_active, Instant::now());
        }
    }

    fn should_unload_now(&mut self) -> bool {
        self.sync_receiver_values();
        self.unloading_target()
            .is_some_and(|target| target <= Instant::now())
    }

    fn note_thread_activity_observed(&mut self) {
        if !self.is_active.0 {
            self.is_active = (false, Instant::now());
        }
    }

    async fn wait_for_unloading_trigger(&mut self) -> bool {
        loop {
            self.sync_receiver_values();
            let unloading_target = self.unloading_target();
            if let Some(target) = unloading_target
                && target <= Instant::now()
            {
                return true;
            }
            let unloading_sleep = async {
                if let Some(target) = unloading_target {
                    tokio::time::sleep_until(target.into()).await;
                } else {
                    futures::future::pending::<()>().await;
                }
            };
            tokio::select! {
                _ = unloading_sleep => return true,
                changed = self.has_subscribers_rx.changed() => {
                    if changed.is_err() {
                        return false;
                    }
                    self.sync_receiver_values();
                },
                changed = self.thread_status_rx.changed() => {
                    if changed.is_err() {
                        return false;
                    }
                    self.sync_receiver_values();
                },
            }
        }
    }
}

pub(crate) struct CodexMessageProcessorArgs {
    pub(crate) auth_manager: Arc<AuthManager>,
    pub(crate) thread_manager: Arc<ThreadManager>,
    pub(crate) outgoing: Arc<OutgoingMessageSender>,
    pub(crate) analytics_events_client: AnalyticsEventsClient,
    pub(crate) arg0_paths: Arg0DispatchPaths,
    /// Startup config used as the process baseline. Fresh effective config loads
    /// go through `config_manager`.
    pub(crate) config: Arc<Config>,
    pub(crate) config_manager: ConfigManager,
    pub(crate) feedback: CodexFeedback,
    pub(crate) log_db: Option<LogDbLayer>,
}

fn configured_thread_store(config: &Config) -> Arc<dyn ThreadStore> {
    match config.experimental_thread_store_endpoint.as_deref() {
        Some(endpoint) => Arc::new(RemoteThreadStore::new(endpoint)),
        None => Arc::new(LocalThreadStore::new(
            codex_rollout::RolloutConfig::from_view(config),
        )),
    }
}

impl CodexMessageProcessor {
    async fn instruction_sources_from_config(config: &Config) -> Vec<AbsolutePathBuf> {
        codex_core::AgentsMdManager::new(config)
            .instruction_sources(LOCAL_FS.as_ref())
            .await
    }

    fn current_account_updated_notification(&self) -> AccountUpdatedNotification {
        let auth = self.auth_manager.auth_cached();
        AccountUpdatedNotification {
            auth_mode: auth.as_ref().map(CodexAuth::api_auth_mode),
            plan_type: auth.as_ref().and_then(CodexAuth::account_plan_type),
        }
    }

    fn track_error_response(
        &self,
        request_id: &ConnectionRequestId,
        error: &JSONRPCErrorError,
        error_type: Option<AnalyticsJsonRpcError>,
    ) {
        if self.config.features.enabled(Feature::GeneralAnalytics) {
            self.analytics_events_client.track_error_response(
                request_id.connection_id.0,
                request_id.request_id.clone(),
                error.clone(),
                error_type,
            );
        }
    }

    async fn load_thread(
        &self,
        thread_id: &str,
    ) -> Result<(ThreadId, Arc<CodexThread>), JSONRPCErrorError> {
        // Resolve the core conversation handle from a v2 thread id string.
        let thread_id = ThreadId::from_string(thread_id).map_err(|err| JSONRPCErrorError {
            code: INVALID_REQUEST_ERROR_CODE,
            message: format!("invalid thread id: {err}"),
            data: None,
        })?;

        let thread = self
            .thread_manager
            .get_thread(thread_id)
            .await
            .map_err(|_| JSONRPCErrorError {
                code: INVALID_REQUEST_ERROR_CODE,
                message: format!("thread not found: {thread_id}"),
                data: None,
            })?;

        Ok((thread_id, thread))
    }
    pub fn new(args: CodexMessageProcessorArgs) -> Self {
        let CodexMessageProcessorArgs {
            auth_manager,
            thread_manager,
            outgoing,
            analytics_events_client,
            arg0_paths,
            config,
            config_manager,
            feedback,
            log_db,
        } = args;
        Self {
            auth_manager,
            thread_manager,
            outgoing: outgoing.clone(),
            analytics_events_client,
            arg0_paths,
            thread_store: configured_thread_store(&config),
            config,
            config_manager,
            active_login: Arc::new(Mutex::new(None)),
            pending_thread_unloads: Arc::new(Mutex::new(HashSet::new())),
            thread_state_manager: ThreadStateManager::new(),
            thread_watch_manager: ThreadWatchManager::new_with_outgoing(outgoing),
            epiphany_invalidation_manager: EpiphanyInvalidationManager::new(),
            command_exec_manager: CommandExecManager::default(),
            pending_fuzzy_searches: Arc::new(Mutex::new(HashMap::new())),
            fuzzy_search_sessions: Arc::new(Mutex::new(HashMap::new())),
            background_tasks: TaskTracker::new(),
            feedback,
            log_db,
        }
    }

    async fn load_latest_config(
        &self,
        fallback_cwd: Option<PathBuf>,
    ) -> Result<Config, JSONRPCErrorError> {
        self.config_manager
            .load_latest_config(fallback_cwd)
            .await
            .map_err(|err| JSONRPCErrorError {
                code: INTERNAL_ERROR_CODE,
                message: format!("failed to reload config: {err}"),
                data: None,
            })
    }

    /// If a client sends `developer_instructions: null` during a mode switch,
    /// use the built-in instructions for that mode.
    fn normalize_turn_start_collaboration_mode(
        &self,
        mut collaboration_mode: CollaborationMode,
        collaboration_modes_config: CollaborationModesConfig,
    ) -> CollaborationMode {
        if collaboration_mode.settings.developer_instructions.is_none()
            && let Some(instructions) = self
                .thread_manager
                .get_models_manager()
                .list_collaboration_modes_for_config(collaboration_modes_config)
                .into_iter()
                .find(|preset| preset.mode == Some(collaboration_mode.mode))
                .and_then(|preset| preset.developer_instructions.flatten())
                .filter(|instructions| !instructions.is_empty())
        {
            collaboration_mode.settings.developer_instructions = Some(instructions);
        }

        collaboration_mode
    }

    fn review_request_from_target(
        target: ApiReviewTarget,
    ) -> Result<(ReviewRequest, String), JSONRPCErrorError> {
        fn invalid_request(message: String) -> JSONRPCErrorError {
            JSONRPCErrorError {
                code: INVALID_REQUEST_ERROR_CODE,
                message,
                data: None,
            }
        }

        let cleaned_target = match target {
            ApiReviewTarget::UncommittedChanges => ApiReviewTarget::UncommittedChanges,
            ApiReviewTarget::BaseBranch { branch } => {
                let branch = branch.trim().to_string();
                if branch.is_empty() {
                    return Err(invalid_request("branch must not be empty".to_string()));
                }
                ApiReviewTarget::BaseBranch { branch }
            }
            ApiReviewTarget::Commit { sha, title } => {
                let sha = sha.trim().to_string();
                if sha.is_empty() {
                    return Err(invalid_request("sha must not be empty".to_string()));
                }
                let title = title
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty());
                ApiReviewTarget::Commit { sha, title }
            }
            ApiReviewTarget::Custom { instructions } => {
                let trimmed = instructions.trim().to_string();
                if trimmed.is_empty() {
                    return Err(invalid_request(
                        "instructions must not be empty".to_string(),
                    ));
                }
                ApiReviewTarget::Custom {
                    instructions: trimmed,
                }
            }
        };

        let core_target = match cleaned_target {
            ApiReviewTarget::UncommittedChanges => CoreReviewTarget::UncommittedChanges,
            ApiReviewTarget::BaseBranch { branch } => CoreReviewTarget::BaseBranch { branch },
            ApiReviewTarget::Commit { sha, title } => CoreReviewTarget::Commit { sha, title },
            ApiReviewTarget::Custom { instructions } => CoreReviewTarget::Custom { instructions },
        };

        let hint = codex_core::review_prompts::user_facing_hint(&core_target);
        let review_request = ReviewRequest {
            target: core_target,
            user_facing_hint: Some(hint.clone()),
        };

        Ok((review_request, hint))
    }

    pub async fn process_request(
        &self,
        connection_id: ConnectionId,
        request: ClientRequest,
        app_server_client_name: Option<String>,
        app_server_client_version: Option<String>,
        request_context: RequestContext,
    ) {
        let to_connection_request_id = |request_id| ConnectionRequestId {
            connection_id,
            request_id,
        };

        match request {
            ClientRequest::Initialize { .. } => {
                panic!("Initialize should be handled in MessageProcessor");
            }
            // === v2 Thread/Turn APIs ===
            ClientRequest::ThreadStart { request_id, params } => {
                self.thread_start(
                    to_connection_request_id(request_id),
                    params,
                    app_server_client_name.clone(),
                    app_server_client_version.clone(),
                    request_context,
                )
                .await;
            }
            ClientRequest::ThreadUnsubscribe { request_id, params } => {
                self.thread_unsubscribe(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadResume { request_id, params } => {
                self.thread_resume(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadFork { request_id, params } => {
                self.thread_fork(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadArchive { request_id, params } => {
                self.thread_archive(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadIncrementElicitation { request_id, params } => {
                self.thread_increment_elicitation(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadDecrementElicitation { request_id, params } => {
                self.thread_decrement_elicitation(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadSetName { request_id, params } => {
                self.thread_set_name(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadMetadataUpdate { request_id, params } => {
                self.thread_metadata_update(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadMemoryModeSet { request_id, params } => {
                self.thread_memory_mode_set(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::MemoryReset { request_id, params } => {
                self.memory_reset(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadUnarchive { request_id, params } => {
                self.thread_unarchive(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadCompactStart { request_id, params } => {
                self.thread_compact_start(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadBackgroundTerminalsClean { request_id, params } => {
                self.thread_background_terminals_clean(
                    to_connection_request_id(request_id),
                    params,
                )
                .await;
            }
            ClientRequest::ThreadRollback { request_id, params } => {
                self.thread_rollback(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadList { request_id, params } => {
                self.thread_list(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadLoadedList { request_id, params } => {
                self.thread_loaded_list(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadRead { request_id, params } => {
                self.thread_read(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyView { request_id, params } => {
                self.thread_epiphany_view(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyRoleLaunch { request_id, params } => {
                self.thread_epiphany_role_launch(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyRoleResult { request_id, params } => {
                self.thread_epiphany_role_result(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyRoleAccept { request_id, params } => {
                self.thread_epiphany_role_accept(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyFreshness { request_id, params } => {
                self.thread_epiphany_freshness(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyContext { request_id, params } => {
                self.thread_epiphany_context(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyGraphQuery { request_id, params } => {
                self.thread_epiphany_graph_query(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyReorientLaunch { request_id, params } => {
                self.thread_epiphany_reorient_launch(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyReorientResult { request_id, params } => {
                self.thread_epiphany_reorient_result(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyReorientAccept { request_id, params } => {
                self.thread_epiphany_reorient_accept(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyIndex { request_id, params } => {
                self.thread_epiphany_index(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyDistill { request_id, params } => {
                self.thread_epiphany_distill(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyPropose { request_id, params } => {
                self.thread_epiphany_propose(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyPromote { request_id, params } => {
                self.thread_epiphany_promote(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyJobLaunch { request_id, params } => {
                self.thread_epiphany_job_launch(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyJobInterrupt { request_id, params } => {
                self.thread_epiphany_job_interrupt(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyUpdate { request_id, params } => {
                self.thread_epiphany_update(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyRetrieve { request_id, params } => {
                self.thread_epiphany_retrieve(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadTurnsList { request_id, params } => {
                self.thread_turns_list(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadShellCommand { request_id, params } => {
                self.thread_shell_command(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadApproveGuardianDeniedAction { request_id, params } => {
                self.thread_approve_guardian_denied_action(
                    to_connection_request_id(request_id),
                    params,
                )
                .await;
            }
            ClientRequest::TurnStart { request_id, params } => {
                self.turn_start(
                    to_connection_request_id(request_id),
                    params,
                    app_server_client_name.clone(),
                    app_server_client_version.clone(),
                )
                .await;
            }
            ClientRequest::ThreadInjectItems { request_id, params } => {
                self.thread_inject_items(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::TurnSteer { request_id, params } => {
                self.turn_steer(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::TurnInterrupt { request_id, params } => {
                self.turn_interrupt(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadRealtimeStart { request_id, params } => {
                self.thread_realtime_start(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadRealtimeAppendAudio { request_id, params } => {
                self.thread_realtime_append_audio(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadRealtimeAppendText { request_id, params } => {
                self.thread_realtime_append_text(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadRealtimeStop { request_id, params } => {
                self.thread_realtime_stop(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadRealtimeListVoices { request_id, params } => {
                self.thread_realtime_list_voices(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ReviewStart { request_id, params } => {
                self.review_start(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::GetConversationSummary { request_id, params } => {
                self.get_thread_summary(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ModelList { request_id, params } => {
                let outgoing = self.outgoing.clone();
                let thread_manager = self.thread_manager.clone();
                let request_id = to_connection_request_id(request_id);

                tokio::spawn(async move {
                    Self::list_models(outgoing, thread_manager, request_id, params).await;
                });
            }
            ClientRequest::ExperimentalFeatureList { request_id, params } => {
                self.experimental_feature_list(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::CollaborationModeList { request_id, params } => {
                let outgoing = self.outgoing.clone();
                let thread_manager = self.thread_manager.clone();
                let request_id = to_connection_request_id(request_id);

                tokio::spawn(async move {
                    Self::list_collaboration_modes(outgoing, thread_manager, request_id, params)
                        .await;
                });
            }
            ClientRequest::MockExperimentalMethod { request_id, params } => {
                self.mock_experimental_method(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::McpServerOauthLogin { request_id, params } => {
                self.mcp_server_oauth_login(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::McpServerRefresh { request_id, params } => {
                self.mcp_server_refresh(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::McpServerStatusList { request_id, params } => {
                self.list_mcp_server_status(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::McpResourceRead { request_id, params } => {
                self.read_mcp_resource(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::McpServerToolCall { request_id, params } => {
                self.call_mcp_server_tool(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::WindowsSandboxSetupStart { request_id, params } => {
                self.windows_sandbox_setup_start(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::LoginAccount { request_id, params } => {
                self.login_v2(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::LogoutAccount {
                request_id,
                params: _,
            } => {
                self.logout_v2(to_connection_request_id(request_id)).await;
            }
            ClientRequest::CancelLoginAccount { request_id, params } => {
                self.cancel_login_v2(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::GetAccount { request_id, params } => {
                self.get_account(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::GitDiffToRemote { request_id, params } => {
                self.git_diff_to_origin(to_connection_request_id(request_id), params.cwd)
                    .await;
            }
            ClientRequest::GetAuthStatus { request_id, params } => {
                self.get_auth_status(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::FuzzyFileSearch { request_id, params } => {
                self.fuzzy_file_search(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::FuzzyFileSearchSessionStart { request_id, params } => {
                self.fuzzy_file_search_session_start(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::FuzzyFileSearchSessionUpdate { request_id, params } => {
                self.fuzzy_file_search_session_update(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::FuzzyFileSearchSessionStop { request_id, params } => {
                self.fuzzy_file_search_session_stop(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::OneOffCommandExec { request_id, params } => {
                self.exec_one_off_command(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::CommandExecWrite { request_id, params } => {
                self.command_exec_write(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::CommandExecResize { request_id, params } => {
                self.command_exec_resize(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::CommandExecTerminate { request_id, params } => {
                self.command_exec_terminate(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::DeviceKeyCreate { .. }
            | ClientRequest::DeviceKeyPublic { .. }
            | ClientRequest::DeviceKeySign { .. } => {
                warn!("Device key request reached CodexMessageProcessor unexpectedly");
            }
            ClientRequest::ConfigRead { .. }
            | ClientRequest::ConfigValueWrite { .. }
            | ClientRequest::ConfigBatchWrite { .. }
            | ClientRequest::ExperimentalFeatureEnablementSet { .. } => {
                warn!("Config request reached CodexMessageProcessor unexpectedly");
            }
            ClientRequest::FsReadFile { .. }
            | ClientRequest::FsWriteFile { .. }
            | ClientRequest::FsCreateDirectory { .. }
            | ClientRequest::FsGetMetadata { .. }
            | ClientRequest::FsReadDirectory { .. }
            | ClientRequest::FsRemove { .. }
            | ClientRequest::FsCopy { .. }
            | ClientRequest::FsWatch { .. }
            | ClientRequest::FsUnwatch { .. } => {
                warn!("Filesystem request reached CodexMessageProcessor unexpectedly");
            }
            ClientRequest::ConfigRequirementsRead { .. } => {
                warn!("ConfigRequirementsRead request reached CodexMessageProcessor unexpectedly");
            }
            ClientRequest::GetAccountRateLimits {
                request_id,
                params: _,
            } => {
                self.get_account_rate_limits(to_connection_request_id(request_id))
                    .await;
            }
            ClientRequest::SendAddCreditsNudgeEmail { request_id, params } => {
                self.send_add_credits_nudge_email(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::FeedbackUpload { request_id, params } => {
                self.upload_feedback(to_connection_request_id(request_id), params)
                    .await;
            }
        }
    }

    async fn thread_start(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadStartParams,
        app_server_client_name: Option<String>,
        app_server_client_version: Option<String>,
        request_context: RequestContext,
    ) {
        let ThreadStartParams {
            model,
            model_provider,
            service_tier,
            cwd,
            approval_policy,
            approvals_reviewer,
            sandbox,
            permission_profile,
            config,
            service_name,
            base_instructions,
            developer_instructions,
            dynamic_tools,
            mock_experimental_field: _mock_experimental_field,
            experimental_raw_events,
            personality,
            ephemeral,
            session_start_source,
            persist_extended_history,
        } = params;
        if sandbox.is_some() && permission_profile.is_some() {
            self.send_invalid_request_error(
                request_id,
                "`permissionProfile` cannot be combined with `sandbox`".to_string(),
            )
            .await;
            return;
        }
        let mut typesafe_overrides = self.build_thread_config_overrides(
            model,
            model_provider,
            service_tier,
            cwd,
            approval_policy,
            approvals_reviewer,
            sandbox,
            permission_profile,
            base_instructions,
            developer_instructions,
            personality,
        );
        typesafe_overrides.ephemeral = ephemeral;
        let listener_task_context = ListenerTaskContext {
            thread_manager: Arc::clone(&self.thread_manager),
            thread_state_manager: self.thread_state_manager.clone(),
            outgoing: Arc::clone(&self.outgoing),
            pending_thread_unloads: Arc::clone(&self.pending_thread_unloads),
            analytics_events_client: self.analytics_events_client.clone(),
            general_analytics_enabled: self.config.features.enabled(Feature::GeneralAnalytics),
            thread_watch_manager: self.thread_watch_manager.clone(),
            epiphany_invalidation_manager: self.epiphany_invalidation_manager.clone(),
            fallback_model_provider: self.config.model_provider_id.clone(),
            codex_home: self.config.codex_home.to_path_buf(),
        };
        let request_trace = request_context.request_trace();
        let config_manager = self.config_manager.clone();
        let thread_start_task = async move {
            Self::thread_start_task(
                listener_task_context,
                config_manager,
                request_id,
                app_server_client_name,
                app_server_client_version,
                config,
                typesafe_overrides,
                dynamic_tools,
                session_start_source,
                persist_extended_history,
                service_name,
                experimental_raw_events,
                request_trace,
            )
            .await;
        };
        self.background_tasks
            .spawn(thread_start_task.instrument(request_context.span()));
    }

    pub(crate) async fn drain_background_tasks(&self) {
        self.background_tasks.close();
        if tokio::time::timeout(Duration::from_secs(10), self.background_tasks.wait())
            .await
            .is_err()
        {
            warn!("timed out waiting for background tasks to shut down; proceeding");
        }
    }

    pub(crate) async fn cancel_active_login(&self) {
        let mut guard = self.active_login.lock().await;
        if let Some(active_login) = guard.take() {
            drop(active_login);
        }
    }

    pub(crate) async fn clear_all_thread_listeners(&self) {
        self.thread_state_manager.clear_all_listeners().await;
    }

    pub(crate) async fn shutdown_threads(&self) {
        let report = self
            .thread_manager
            .shutdown_all_threads_bounded(Duration::from_secs(10))
            .await;
        for thread_id in report.submit_failed {
            warn!("failed to submit Shutdown to thread {thread_id}");
        }
        for thread_id in report.timed_out {
            warn!("timed out waiting for thread {thread_id} to shut down");
        }
    }

    async fn request_trace_context(
        &self,
        request_id: &ConnectionRequestId,
    ) -> Option<codex_protocol::protocol::W3cTraceContext> {
        self.outgoing.request_trace_context(request_id).await
    }

    async fn submit_core_op(
        &self,
        request_id: &ConnectionRequestId,
        thread: &CodexThread,
        op: Op,
    ) -> CodexResult<String> {
        thread
            .submit_with_trace(op, self.request_trace_context(request_id).await)
            .await
    }

    #[allow(clippy::too_many_arguments)]
    async fn thread_start_task(
        listener_task_context: ListenerTaskContext,
        config_manager: ConfigManager,
        request_id: ConnectionRequestId,
        app_server_client_name: Option<String>,
        app_server_client_version: Option<String>,
        config_overrides: Option<HashMap<String, serde_json::Value>>,
        typesafe_overrides: ConfigOverrides,
        dynamic_tools: Option<Vec<ApiDynamicToolSpec>>,
        session_start_source: Option<codex_app_server_protocol::ThreadStartSource>,
        persist_extended_history: bool,
        service_name: Option<String>,
        experimental_raw_events: bool,
        request_trace: Option<W3cTraceContext>,
    ) {
        let requested_cwd = typesafe_overrides.cwd.clone();
        let mut config = match config_manager
            .load_with_overrides(config_overrides.clone(), typesafe_overrides.clone())
            .await
        {
            Ok(config) => config,
            Err(err) => {
                let error = config_load_error(&err);
                listener_task_context
                    .outgoing
                    .send_error(request_id, error)
                    .await;
                return;
            }
        };

        // The user may have requested WorkspaceWrite or DangerFullAccess via
        // the command line, though in the process of deriving the Config, it
        // could be downgraded to ReadOnly (perhaps there is no sandbox
        // available on Windows or the enterprise config disallows it). The cwd
        // should still be considered "trusted" in this case.
        let requested_permissions_trust_project =
            requested_permissions_trust_project(&typesafe_overrides, config.cwd.as_path());

        if requested_cwd.is_some()
            && !config.active_project.is_trusted()
            && (requested_permissions_trust_project
                || matches!(
                    config.permissions.sandbox_policy.get(),
                    codex_protocol::protocol::SandboxPolicy::WorkspaceWrite { .. }
                        | codex_protocol::protocol::SandboxPolicy::DangerFullAccess
                        | codex_protocol::protocol::SandboxPolicy::ExternalSandbox { .. }
                ))
        {
            let trust_target = resolve_root_git_project_for_trust(LOCAL_FS.as_ref(), &config.cwd)
                .await
                .unwrap_or_else(|| config.cwd.clone());
            let current_cli_overrides = config_manager.current_cli_overrides();
            let cli_overrides_with_trust;
            let cli_overrides_for_reload = if let Err(err) =
                codex_core::config::set_project_trust_level(
                    &listener_task_context.codex_home,
                    trust_target.as_path(),
                    TrustLevel::Trusted,
                ) {
                warn!(
                    "failed to persist trusted project state for {}; continuing with in-memory trust for this thread: {err}",
                    trust_target.display()
                );
                let mut project = toml::map::Map::new();
                project.insert(
                    "trust_level".to_string(),
                    TomlValue::String("trusted".to_string()),
                );
                let mut projects = toml::map::Map::new();
                projects.insert(
                    project_trust_key(trust_target.as_path()),
                    TomlValue::Table(project),
                );
                cli_overrides_with_trust = current_cli_overrides
                    .iter()
                    .cloned()
                    .chain(std::iter::once((
                        "projects".to_string(),
                        TomlValue::Table(projects),
                    )))
                    .collect::<Vec<_>>();
                cli_overrides_with_trust.as_slice()
            } else {
                current_cli_overrides.as_slice()
            };

            config = match config_manager
                .load_with_cli_overrides(
                    cli_overrides_for_reload,
                    config_overrides,
                    typesafe_overrides,
                    /*fallback_cwd*/ None,
                )
                .await
            {
                Ok(config) => config,
                Err(err) => {
                    let error = config_load_error(&err);
                    listener_task_context
                        .outgoing
                        .send_error(request_id, error)
                        .await;
                    return;
                }
            };
        }

        let instruction_sources = Self::instruction_sources_from_config(&config).await;
        let dynamic_tools = dynamic_tools.unwrap_or_default();
        let core_dynamic_tools = if dynamic_tools.is_empty() {
            Vec::new()
        } else {
            if let Err(message) = validate_dynamic_tools(&dynamic_tools) {
                let error = JSONRPCErrorError {
                    code: INVALID_REQUEST_ERROR_CODE,
                    message,
                    data: None,
                };
                listener_task_context
                    .outgoing
                    .send_error(request_id, error)
                    .await;
                return;
            }
            dynamic_tools
                .into_iter()
                .map(|tool| CoreDynamicToolSpec {
                    namespace: tool.namespace,
                    name: tool.name,
                    description: tool.description,
                    input_schema: tool.input_schema,
                    defer_loading: tool.defer_loading,
                })
                .collect()
        };
        let core_dynamic_tool_count = core_dynamic_tools.len();

        match listener_task_context
            .thread_manager
            .start_thread_with_tools_and_service_name(
                config,
                match session_start_source
                    .unwrap_or(codex_app_server_protocol::ThreadStartSource::Startup)
                {
                    codex_app_server_protocol::ThreadStartSource::Startup => InitialHistory::New,
                    codex_app_server_protocol::ThreadStartSource::Clear => InitialHistory::Cleared,
                },
                core_dynamic_tools,
                persist_extended_history,
                service_name,
                request_trace,
            )
            .instrument(tracing::info_span!(
                "app_server.thread_start.create_thread",
                otel.name = "app_server.thread_start.create_thread",
                thread_start.dynamic_tool_count = core_dynamic_tool_count,
                thread_start.persist_extended_history = persist_extended_history,
            ))
            .await
        {
            Ok(new_conv) => {
                let NewThread {
                    thread_id,
                    thread: codex_thread,
                    session_configured,
                    ..
                } = new_conv;
                if let Err(error) = Self::set_app_server_client_info(
                    codex_thread.as_ref(),
                    app_server_client_name,
                    app_server_client_version,
                )
                .await
                {
                    listener_task_context
                        .outgoing
                        .send_error(request_id, error)
                        .await;
                    return;
                }
                let config_snapshot = codex_thread
                    .config_snapshot()
                    .instrument(tracing::info_span!(
                        "app_server.thread_start.config_snapshot",
                        otel.name = "app_server.thread_start.config_snapshot",
                    ))
                    .await;
                let mut thread = build_thread_from_snapshot(
                    thread_id,
                    &config_snapshot,
                    session_configured.rollout_path.clone(),
                );
                thread.epiphany_state = codex_thread
                    .epiphany_state()
                    .instrument(tracing::info_span!(
                        "app_server.thread_start.epiphany_state",
                        otel.name = "app_server.thread_start.epiphany_state",
                    ))
                    .await;

                // Auto-attach a thread listener when starting a thread.
                Self::log_listener_attach_result(
                    Self::ensure_conversation_listener_task(
                        listener_task_context.clone(),
                        thread_id,
                        request_id.connection_id,
                        experimental_raw_events,
                        ApiVersion::V2,
                    )
                    .instrument(tracing::info_span!(
                        "app_server.thread_start.attach_listener",
                        otel.name = "app_server.thread_start.attach_listener",
                        thread_start.experimental_raw_events = experimental_raw_events,
                    ))
                    .await,
                    thread_id,
                    request_id.connection_id,
                    "thread",
                );

                listener_task_context
                    .thread_watch_manager
                    .upsert_thread_silently(thread.clone())
                    .instrument(tracing::info_span!(
                        "app_server.thread_start.upsert_thread",
                        otel.name = "app_server.thread_start.upsert_thread",
                    ))
                    .await;

                thread.status = resolve_thread_status(
                    listener_task_context
                        .thread_watch_manager
                        .loaded_status_for_thread(&thread.id)
                        .instrument(tracing::info_span!(
                            "app_server.thread_start.resolve_status",
                            otel.name = "app_server.thread_start.resolve_status",
                        ))
                        .await,
                    /*has_in_progress_turn*/ false,
                );

                let permission_profile = thread_response_permission_profile(
                    &config_snapshot.sandbox_policy,
                    config_snapshot.permission_profile,
                );

                let response = ThreadStartResponse {
                    thread: thread.clone(),
                    model: config_snapshot.model,
                    model_provider: config_snapshot.model_provider_id,
                    service_tier: config_snapshot.service_tier,
                    cwd: config_snapshot.cwd,
                    instruction_sources,
                    approval_policy: config_snapshot.approval_policy.into(),
                    approvals_reviewer: config_snapshot.approvals_reviewer.into(),
                    sandbox: config_snapshot.sandbox_policy.into(),
                    permission_profile,
                    reasoning_effort: config_snapshot.reasoning_effort,
                };
                if listener_task_context.general_analytics_enabled {
                    listener_task_context
                        .analytics_events_client
                        .track_response(
                            request_id.connection_id.0,
                            ClientResponse::ThreadStart {
                                request_id: request_id.request_id.clone(),
                                response: response.clone(),
                            },
                        );
                }

                listener_task_context
                    .outgoing
                    .send_response(request_id, response)
                    .instrument(tracing::info_span!(
                        "app_server.thread_start.send_response",
                        otel.name = "app_server.thread_start.send_response",
                    ))
                    .await;

                let notif = ThreadStartedNotification { thread };
                listener_task_context
                    .outgoing
                    .send_server_notification(ServerNotification::ThreadStarted(notif))
                    .instrument(tracing::info_span!(
                        "app_server.thread_start.notify_started",
                        otel.name = "app_server.thread_start.notify_started",
                    ))
                    .await;
            }
            Err(err) => {
                let error = JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message: format!("error creating thread: {err}"),
                    data: None,
                };
                listener_task_context
                    .outgoing
                    .send_error(request_id, error)
                    .await;
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_thread_config_overrides(
        &self,
        model: Option<String>,
        model_provider: Option<String>,
        service_tier: Option<Option<codex_protocol::config_types::ServiceTier>>,
        cwd: Option<String>,
        approval_policy: Option<codex_app_server_protocol::AskForApproval>,
        approvals_reviewer: Option<codex_app_server_protocol::ApprovalsReviewer>,
        sandbox: Option<SandboxMode>,
        permission_profile: Option<ApiPermissionProfile>,
        base_instructions: Option<String>,
        developer_instructions: Option<String>,
        personality: Option<Personality>,
    ) -> ConfigOverrides {
        ConfigOverrides {
            model,
            model_provider,
            service_tier,
            cwd: cwd.map(PathBuf::from),
            approval_policy: approval_policy
                .map(codex_app_server_protocol::AskForApproval::to_core),
            approvals_reviewer: approvals_reviewer
                .map(codex_app_server_protocol::ApprovalsReviewer::to_core),
            sandbox_mode: sandbox.map(SandboxMode::to_core),
            permission_profile: permission_profile.map(Into::into),
            codex_linux_sandbox_exe: self.arg0_paths.codex_linux_sandbox_exe.clone(),
            main_execve_wrapper_exe: self.arg0_paths.main_execve_wrapper_exe.clone(),
            base_instructions,
            developer_instructions,
            personality,
            ..Default::default()
        }
    }

    pub(crate) fn thread_created_receiver(&self) -> broadcast::Receiver<ThreadId> {
        self.thread_manager.subscribe_thread_created()
    }

    pub(crate) async fn connection_initialized(&self, connection_id: ConnectionId) {
        self.thread_state_manager
            .connection_initialized(connection_id)
            .await;
    }

    pub(crate) async fn connection_closed(&self, connection_id: ConnectionId) {
        self.command_exec_manager
            .connection_closed(connection_id)
            .await;
        let thread_ids = self
            .thread_state_manager
            .remove_connection(connection_id)
            .await;

        for thread_id in thread_ids {
            if self.thread_manager.get_thread(thread_id).await.is_err() {
                // Reconcile stale app-server bookkeeping when the thread has already been
                // removed from the core manager.
                self.finalize_thread_teardown(thread_id).await;
            }
        }
    }

    pub(crate) fn subscribe_running_assistant_turn_count(&self) -> watch::Receiver<usize> {
        self.thread_watch_manager.subscribe_running_turn_count()
    }

    /// Best-effort: ensure initialized connections are subscribed to this thread.
    pub(crate) async fn try_attach_thread_listener(
        &self,
        thread_id: ThreadId,
        connection_ids: Vec<ConnectionId>,
    ) {
        if let Ok(thread) = self.thread_manager.get_thread(thread_id).await {
            let config_snapshot = thread.config_snapshot().await;
            let loaded_thread =
                build_thread_from_snapshot(thread_id, &config_snapshot, thread.rollout_path());
            self.thread_watch_manager.upsert_thread(loaded_thread).await;
        }

        for connection_id in connection_ids {
            Self::log_listener_attach_result(
                self.ensure_conversation_listener(
                    thread_id,
                    connection_id,
                    /*raw_events_enabled*/ false,
                    ApiVersion::V2,
                )
                .await,
                thread_id,
                connection_id,
                "thread",
            );
        }
    }

    async fn thread_resume(&self, request_id: ConnectionRequestId, params: ThreadResumeParams) {
        if let Ok(thread_id) = ThreadId::from_string(&params.thread_id)
            && self
                .pending_thread_unloads
                .lock()
                .await
                .contains(&thread_id)
        {
            self.send_invalid_request_error(
                request_id,
                format!(
                    "thread {thread_id} is closing; retry thread/resume after the thread is closed"
                ),
            )
            .await;
            return;
        }

        if params.sandbox.is_some() && params.permission_profile.is_some() {
            self.send_invalid_request_error(
                request_id,
                "`permissionProfile` cannot be combined with `sandbox`".to_string(),
            )
            .await;
            return;
        }

        if self
            .resume_running_thread(request_id.clone(), &params)
            .await
        {
            return;
        }

        let ThreadResumeParams {
            thread_id,
            history,
            path,
            model,
            model_provider,
            service_tier,
            cwd,
            approval_policy,
            approvals_reviewer,
            sandbox,
            permission_profile,
            config: mut request_overrides,
            base_instructions,
            developer_instructions,
            personality,
            persist_extended_history,
        } = params;

        let thread_history = if let Some(history) = history {
            let Some(thread_history) = self
                .resume_thread_from_history(request_id.clone(), history.as_slice())
                .await
            else {
                return;
            };
            thread_history
        } else {
            let Some(thread_history) = self
                .resume_thread_from_rollout(request_id.clone(), &thread_id, path.as_ref())
                .await
            else {
                return;
            };
            thread_history
        };

        let history_cwd = thread_history.session_cwd();
        let mut typesafe_overrides = self.build_thread_config_overrides(
            model,
            model_provider,
            service_tier,
            cwd,
            approval_policy,
            approvals_reviewer,
            sandbox,
            permission_profile,
            base_instructions,
            developer_instructions,
            personality,
        );
        let persisted_resume_metadata = self
            .load_and_apply_persisted_resume_metadata(
                &thread_history,
                &mut request_overrides,
                &mut typesafe_overrides,
            )
            .await;

        // Derive a Config using the same logic as new conversation, honoring overrides if provided.
        let config = match self
            .config_manager
            .load_for_cwd(request_overrides, typesafe_overrides, history_cwd)
            .await
        {
            Ok(config) => config,
            Err(err) => {
                let error = config_load_error(&err);
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        let fallback_model_provider = config.model_provider_id.clone();
        let instruction_sources = Self::instruction_sources_from_config(&config).await;
        let response_history = thread_history.clone();

        match self
            .thread_manager
            .resume_thread_with_history(
                config,
                thread_history,
                self.auth_manager.clone(),
                persist_extended_history,
                self.request_trace_context(&request_id).await,
            )
            .await
        {
            Ok(NewThread {
                thread_id,
                thread: codex_thread,
                session_configured,
                ..
            }) => {
                let SessionConfiguredEvent { rollout_path, .. } = session_configured;
                let Some(rollout_path) = rollout_path else {
                    self.send_internal_error(
                        request_id,
                        format!("rollout path missing for thread {thread_id}"),
                    )
                    .await;
                    return;
                };
                // Auto-attach a thread listener when resuming a thread.
                Self::log_listener_attach_result(
                    self.ensure_conversation_listener(
                        thread_id,
                        request_id.connection_id,
                        /*raw_events_enabled*/ false,
                        ApiVersion::V2,
                    )
                    .await,
                    thread_id,
                    request_id.connection_id,
                    "thread",
                );

                let mut thread = match self
                    .load_thread_from_resume_source_or_send_internal(
                        thread_id,
                        codex_thread.as_ref(),
                        &response_history,
                        rollout_path.as_path(),
                        fallback_model_provider.as_str(),
                        persisted_resume_metadata.as_ref(),
                    )
                    .await
                {
                    Ok(thread) => thread,
                    Err(message) => {
                        self.send_internal_error(request_id, message).await;
                        return;
                    }
                };

                self.thread_watch_manager
                    .upsert_thread(thread.clone())
                    .await;

                let thread_status = self
                    .thread_watch_manager
                    .loaded_status_for_thread(&thread.id)
                    .await;

                set_thread_status_and_interrupt_stale_turns(
                    &mut thread,
                    thread_status,
                    /*has_live_in_progress_turn*/ false,
                );
                let permission_profile = thread_response_permission_profile(
                    &session_configured.sandbox_policy,
                    codex_thread.config_snapshot().await.permission_profile,
                );

                let response = ThreadResumeResponse {
                    thread,
                    model: session_configured.model,
                    model_provider: session_configured.model_provider_id,
                    service_tier: session_configured.service_tier,
                    cwd: session_configured.cwd,
                    instruction_sources,
                    approval_policy: session_configured.approval_policy.into(),
                    approvals_reviewer: session_configured.approvals_reviewer.into(),
                    sandbox: session_configured.sandbox_policy.into(),
                    permission_profile,
                    reasoning_effort: session_configured.reasoning_effort,
                };
                if self.config.features.enabled(Feature::GeneralAnalytics) {
                    self.analytics_events_client.track_response(
                        request_id.connection_id.0,
                        ClientResponse::ThreadResume {
                            request_id: request_id.request_id.clone(),
                            response: response.clone(),
                        },
                    );
                }

                let connection_id = request_id.connection_id;
                let token_usage_thread = response.thread.clone();
                let token_usage_turn_id = latest_token_usage_turn_id_from_rollout_items(
                    &response_history.get_rollout_items(),
                    &token_usage_thread,
                );
                self.outgoing.send_response(request_id, response).await;
                // The client needs restored usage before it starts another turn.
                // Sending after the response preserves JSON-RPC request ordering while
                // still filling the status line before the next turn lifecycle begins.
                send_thread_token_usage_update_to_connection(
                    &self.outgoing,
                    connection_id,
                    thread_id,
                    &token_usage_thread,
                    codex_thread.as_ref(),
                    token_usage_turn_id,
                )
                .await;
            }
            Err(err) => {
                let error = JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message: format!("error resuming thread: {err}"),
                    data: None,
                };
                self.outgoing.send_error(request_id, error).await;
            }
        }
    }

    async fn load_and_apply_persisted_resume_metadata(
        &self,
        thread_history: &InitialHistory,
        request_overrides: &mut Option<HashMap<String, serde_json::Value>>,
        typesafe_overrides: &mut ConfigOverrides,
    ) -> Option<ThreadMetadata> {
        let InitialHistory::Resumed(resumed_history) = thread_history else {
            return None;
        };
        let state_db_ctx = get_state_db(&self.config).await?;
        let persisted_metadata = state_db_ctx
            .get_thread(resumed_history.conversation_id)
            .await
            .ok()
            .flatten()?;
        merge_persisted_resume_metadata(request_overrides, typesafe_overrides, &persisted_metadata);
        Some(persisted_metadata)
    }

    async fn resume_running_thread(
        &self,
        request_id: ConnectionRequestId,
        params: &ThreadResumeParams,
    ) -> bool {
        if let Ok(existing_thread_id) = ThreadId::from_string(&params.thread_id)
            && let Ok(existing_thread) = self.thread_manager.get_thread(existing_thread_id).await
        {
            if params.history.is_some() {
                self.send_invalid_request_error(
                    request_id,
                    format!(
                        "cannot resume thread {existing_thread_id} with history while it is already running"
                    ),
                )
                .await;
                return true;
            }

            let rollout_path = if let Some(path) = existing_thread.rollout_path() {
                if path.exists() {
                    path
                } else {
                    match find_thread_path_by_id_str(
                        &self.config.codex_home,
                        &existing_thread_id.to_string(),
                    )
                    .await
                    {
                        Ok(Some(path)) => path,
                        Ok(None) => {
                            self.send_invalid_request_error(
                                request_id,
                                format!("no rollout found for thread id {existing_thread_id}"),
                            )
                            .await;
                            return true;
                        }
                        Err(err) => {
                            self.send_invalid_request_error(
                                request_id,
                                format!("failed to locate thread id {existing_thread_id}: {err}"),
                            )
                            .await;
                            return true;
                        }
                    }
                }
            } else {
                match find_thread_path_by_id_str(
                    &self.config.codex_home,
                    &existing_thread_id.to_string(),
                )
                .await
                {
                    Ok(Some(path)) => path,
                    Ok(None) => {
                        self.send_invalid_request_error(
                            request_id,
                            format!("no rollout found for thread id {existing_thread_id}"),
                        )
                        .await;
                        return true;
                    }
                    Err(err) => {
                        self.send_invalid_request_error(
                            request_id,
                            format!("failed to locate thread id {existing_thread_id}: {err}"),
                        )
                        .await;
                        return true;
                    }
                }
            };

            if let Some(requested_path) = params.path.as_ref()
                && requested_path != &rollout_path
            {
                self.send_invalid_request_error(
                    request_id,
                    format!(
                        "cannot resume running thread {existing_thread_id} with mismatched path: requested `{}`, active `{}`",
                        requested_path.display(),
                        rollout_path.display()
                    ),
                )
                .await;
                return true;
            }

            let thread_state = self
                .thread_state_manager
                .thread_state(existing_thread_id)
                .await;
            if let Err(error) = self
                .ensure_listener_task_running(
                    existing_thread_id,
                    existing_thread.clone(),
                    thread_state.clone(),
                    ApiVersion::V2,
                )
                .await
            {
                self.outgoing.send_error(request_id, error).await;
                return true;
            }

            let config_snapshot = existing_thread.config_snapshot().await;
            let mismatch_details = collect_resume_override_mismatches(params, &config_snapshot);
            if !mismatch_details.is_empty() {
                tracing::warn!(
                    "thread/resume overrides ignored for running thread {}: {}",
                    existing_thread_id,
                    mismatch_details.join("; ")
                );
            }
            let mut config_for_instruction_sources = self.config.as_ref().clone();
            config_for_instruction_sources.cwd = config_snapshot.cwd.clone();
            let instruction_sources =
                Self::instruction_sources_from_config(&config_for_instruction_sources).await;
            let thread_summary = match load_thread_summary_for_rollout(
                &self.config,
                existing_thread_id,
                rollout_path.as_path(),
                config_snapshot.model_provider_id.as_str(),
                /*persisted_metadata*/ None,
            )
            .await
            {
                Ok(thread) => thread,
                Err(message) => {
                    self.send_internal_error(request_id, message).await;
                    return true;
                }
            };

            let listener_command_tx = {
                let thread_state = thread_state.lock().await;
                thread_state.listener_command_tx()
            };
            let Some(listener_command_tx) = listener_command_tx else {
                let err = JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message: format!(
                        "failed to enqueue running thread resume for thread {existing_thread_id}: thread listener is not running"
                    ),
                    data: None,
                };
                self.outgoing.send_error(request_id, err).await;
                return true;
            };

            let command = crate::thread_state::ThreadListenerCommand::SendThreadResumeResponse(
                Box::new(crate::thread_state::PendingThreadResumeRequest {
                    request_id: request_id.clone(),
                    rollout_path: rollout_path.clone(),
                    config_snapshot,
                    instruction_sources,
                    thread_summary,
                }),
            );
            if listener_command_tx.send(command).is_err() {
                let err = JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message: format!(
                        "failed to enqueue running thread resume for thread {existing_thread_id}: thread listener command channel is closed"
                    ),
                    data: None,
                };
                self.outgoing.send_error(request_id, err).await;
            }
            return true;
        }
        false
    }

    async fn resume_thread_from_history(
        &self,
        request_id: ConnectionRequestId,
        history: &[ResponseItem],
    ) -> Option<InitialHistory> {
        if history.is_empty() {
            self.send_invalid_request_error(request_id, "history must not be empty".to_string())
                .await;
            return None;
        }
        Some(InitialHistory::Forked(
            history
                .iter()
                .cloned()
                .map(RolloutItem::ResponseItem)
                .collect(),
        ))
    }

    async fn resume_thread_from_rollout(
        &self,
        request_id: ConnectionRequestId,
        thread_id: &str,
        path: Option<&PathBuf>,
    ) -> Option<InitialHistory> {
        let rollout_path = if let Some(path) = path {
            path.clone()
        } else {
            let existing_thread_id = match ThreadId::from_string(thread_id) {
                Ok(id) => id,
                Err(err) => {
                    let error = JSONRPCErrorError {
                        code: INVALID_REQUEST_ERROR_CODE,
                        message: format!("invalid thread id: {err}"),
                        data: None,
                    };
                    self.outgoing.send_error(request_id, error).await;
                    return None;
                }
            };

            match find_thread_path_by_id_str(
                &self.config.codex_home,
                &existing_thread_id.to_string(),
            )
            .await
            {
                Ok(Some(path)) => path,
                Ok(None) => {
                    self.send_invalid_request_error(
                        request_id,
                        format!("no rollout found for thread id {existing_thread_id}"),
                    )
                    .await;
                    return None;
                }
                Err(err) => {
                    self.send_invalid_request_error(
                        request_id,
                        format!("failed to locate thread id {existing_thread_id}: {err}"),
                    )
                    .await;
                    return None;
                }
            }
        };

        match RolloutRecorder::get_rollout_history(&rollout_path).await {
            Ok(initial_history) => Some(initial_history),
            Err(err) => {
                self.send_invalid_request_error(
                    request_id,
                    format!("failed to load rollout `{}`: {err}", rollout_path.display()),
                )
                .await;
                None
            }
        }
    }

    async fn load_thread_from_resume_source_or_send_internal(
        &self,
        thread_id: ThreadId,
        live_thread: &CodexThread,
        thread_history: &InitialHistory,
        rollout_path: &Path,
        fallback_provider: &str,
        persisted_resume_metadata: Option<&ThreadMetadata>,
    ) -> std::result::Result<Thread, String> {
        let thread = match thread_history {
            InitialHistory::Resumed(resumed) => {
                load_thread_summary_for_rollout(
                    &self.config,
                    resumed.conversation_id,
                    resumed.rollout_path.as_path(),
                    fallback_provider,
                    persisted_resume_metadata,
                )
                .await
            }
            InitialHistory::Forked(items) => {
                let config_snapshot = live_thread.config_snapshot().await;
                let mut thread = build_thread_from_snapshot(
                    thread_id,
                    &config_snapshot,
                    Some(rollout_path.into()),
                );
                thread.preview = preview_from_rollout_items(items);
                Ok(thread)
            }
            InitialHistory::New | InitialHistory::Cleared => Err(format!(
                "failed to build resume response for thread {thread_id}: initial history missing"
            )),
        };
        let mut thread = thread?;
        thread.id = thread_id.to_string();
        thread.path = Some(rollout_path.to_path_buf());
        let history_items = thread_history.get_rollout_items();
        populate_thread_turns(
            &mut thread,
            ThreadTurnSource::HistoryItems(&history_items),
            /*active_turn*/ None,
        )
        .await?;
        self.attach_thread_name(thread_id, &mut thread).await;
        thread.epiphany_state = live_thread_epiphany_state(live_thread).await;
        Ok(thread)
    }

    async fn attach_thread_name(&self, thread_id: ThreadId, thread: &mut Thread) {
        if let Some(title) = title_from_state_db(&self.config, thread_id).await {
            set_thread_name_from_title(thread, title);
        }
    }

    async fn thread_fork(&self, request_id: ConnectionRequestId, params: ThreadForkParams) {
        let ThreadForkParams {
            thread_id,
            path,
            model,
            model_provider,
            service_tier,
            cwd,
            approval_policy,
            approvals_reviewer,
            sandbox,
            permission_profile,
            config: cli_overrides,
            base_instructions,
            developer_instructions,
            ephemeral,
            persist_extended_history,
        } = params;
        if sandbox.is_some() && permission_profile.is_some() {
            self.send_invalid_request_error(
                request_id,
                "`permissionProfile` cannot be combined with `sandbox`".to_string(),
            )
            .await;
            return;
        }

        let (rollout_path, source_thread_id) = if let Some(path) = path {
            (path, None)
        } else {
            let existing_thread_id = match ThreadId::from_string(&thread_id) {
                Ok(id) => id,
                Err(err) => {
                    self.send_invalid_request_error(
                        request_id,
                        format!("invalid thread id: {err}"),
                    )
                    .await;
                    return;
                }
            };

            match find_thread_path_by_id_str(
                &self.config.codex_home,
                &existing_thread_id.to_string(),
            )
            .await
            {
                Ok(Some(p)) => (p, Some(existing_thread_id)),
                Ok(None) => {
                    self.send_invalid_request_error(
                        request_id,
                        format!("no rollout found for thread id {existing_thread_id}"),
                    )
                    .await;
                    return;
                }
                Err(err) => {
                    self.send_invalid_request_error(
                        request_id,
                        format!("failed to locate thread id {existing_thread_id}: {err}"),
                    )
                    .await;
                    return;
                }
            }
        };

        let history_cwd =
            read_history_cwd_from_state_db(&self.config, source_thread_id, rollout_path.as_path())
                .await;

        // Persist Windows sandbox mode.
        let mut cli_overrides = cli_overrides.unwrap_or_default();
        if cfg!(windows) {
            match WindowsSandboxLevel::from_config(&self.config) {
                WindowsSandboxLevel::Elevated => {
                    cli_overrides
                        .insert("windows.sandbox".to_string(), serde_json::json!("elevated"));
                }
                WindowsSandboxLevel::RestrictedToken => {
                    cli_overrides.insert(
                        "windows.sandbox".to_string(),
                        serde_json::json!("unelevated"),
                    );
                }
                WindowsSandboxLevel::Disabled => {}
            }
        }
        let request_overrides = if cli_overrides.is_empty() {
            None
        } else {
            Some(cli_overrides)
        };
        let mut typesafe_overrides = self.build_thread_config_overrides(
            model,
            model_provider,
            service_tier,
            cwd,
            approval_policy,
            approvals_reviewer,
            sandbox,
            permission_profile,
            base_instructions,
            developer_instructions,
            /*personality*/ None,
        );
        typesafe_overrides.ephemeral = ephemeral.then_some(true);
        // Derive a Config using the same logic as new conversation, honoring overrides if provided.
        let config = match self
            .config_manager
            .load_for_cwd(request_overrides, typesafe_overrides, history_cwd)
            .await
        {
            Ok(config) => config,
            Err(err) => {
                self.outgoing
                    .send_error(request_id, config_load_error(&err))
                    .await;
                return;
            }
        };

        let fallback_model_provider = config.model_provider_id.clone();
        let instruction_sources = Self::instruction_sources_from_config(&config).await;

        let NewThread {
            thread_id,
            thread: forked_thread,
            session_configured,
            ..
        } = match self
            .thread_manager
            .fork_thread(
                ForkSnapshot::Interrupted,
                config,
                rollout_path.clone(),
                persist_extended_history,
                self.request_trace_context(&request_id).await,
            )
            .await
        {
            Ok(thread) => thread,
            Err(err) => {
                match err {
                    CodexErr::Io(_) | CodexErr::Json(_) => {
                        self.send_invalid_request_error(
                            request_id,
                            format!("failed to load rollout `{}`: {err}", rollout_path.display()),
                        )
                        .await;
                    }
                    CodexErr::InvalidRequest(message) => {
                        self.send_invalid_request_error(request_id, message).await;
                    }
                    _ => {
                        self.send_internal_error(
                            request_id,
                            format!("error forking thread: {err}"),
                        )
                        .await;
                    }
                }
                return;
            }
        };

        // Auto-attach a conversation listener when forking a thread.
        Self::log_listener_attach_result(
            self.ensure_conversation_listener(
                thread_id,
                request_id.connection_id,
                /*raw_events_enabled*/ false,
                ApiVersion::V2,
            )
            .await,
            thread_id,
            request_id.connection_id,
            "thread",
        );

        // Persistent forks materialize their own rollout immediately. Ephemeral forks stay
        // pathless, so they rebuild their visible history from the copied source rollout instead.
        let mut thread = if let Some(fork_rollout_path) = session_configured.rollout_path.as_ref() {
            match read_summary_from_rollout(
                fork_rollout_path.as_path(),
                fallback_model_provider.as_str(),
            )
            .await
            {
                Ok(summary) => {
                    let mut thread = summary_to_thread(summary, &self.config.cwd);
                    thread.forked_from_id =
                        forked_from_id_from_rollout(fork_rollout_path.as_path()).await;
                    thread
                }
                Err(err) => {
                    self.send_internal_error(
                        request_id,
                        format!(
                            "failed to load rollout `{}` for thread {thread_id}: {err}",
                            fork_rollout_path.display()
                        ),
                    )
                    .await;
                    return;
                }
            }
        } else {
            let config_snapshot = forked_thread.config_snapshot().await;
            // forked thread names do not inherit the source thread name
            let mut thread =
                build_thread_from_snapshot(thread_id, &config_snapshot, /*path*/ None);
            let history_items = match read_rollout_items_from_rollout(rollout_path.as_path()).await
            {
                Ok(items) => items,
                Err(err) => {
                    self.send_internal_error(
                        request_id,
                        format!(
                            "failed to load source rollout `{}` for thread {thread_id}: {err}",
                            rollout_path.display()
                        ),
                    )
                    .await;
                    return;
                }
            };
            thread.preview = preview_from_rollout_items(&history_items);
            thread.forked_from_id = source_thread_id
                .or_else(|| {
                    history_items.iter().find_map(|item| match item {
                        RolloutItem::SessionMeta(meta_line) => Some(meta_line.meta.id),
                        _ => None,
                    })
                })
                .map(|id| id.to_string());
            if let Err(message) = populate_thread_turns(
                &mut thread,
                ThreadTurnSource::HistoryItems(&history_items),
                /*active_turn*/ None,
            )
            .await
            {
                self.send_internal_error(request_id, message).await;
                return;
            }
            thread
        };

        if let Some(fork_rollout_path) = session_configured.rollout_path.as_ref()
            && let Err(message) = populate_thread_turns(
                &mut thread,
                ThreadTurnSource::RolloutPath(fork_rollout_path.as_path()),
                /*active_turn*/ None,
            )
            .await
        {
            self.send_internal_error(request_id, message).await;
            return;
        }

        thread.epiphany_state = live_thread_epiphany_state(forked_thread.as_ref()).await;

        self.thread_watch_manager
            .upsert_thread_silently(thread.clone())
            .await;

        thread.status = resolve_thread_status(
            self.thread_watch_manager
                .loaded_status_for_thread(&thread.id)
                .await,
            /*has_in_progress_turn*/ false,
        );
        let permission_profile = thread_response_permission_profile(
            &session_configured.sandbox_policy,
            forked_thread.config_snapshot().await.permission_profile,
        );

        let response = ThreadForkResponse {
            thread: thread.clone(),
            model: session_configured.model,
            model_provider: session_configured.model_provider_id,
            service_tier: session_configured.service_tier,
            cwd: session_configured.cwd,
            instruction_sources,
            approval_policy: session_configured.approval_policy.into(),
            approvals_reviewer: session_configured.approvals_reviewer.into(),
            sandbox: session_configured.sandbox_policy.into(),
            permission_profile,
            reasoning_effort: session_configured.reasoning_effort,
        };
        if self.config.features.enabled(Feature::GeneralAnalytics) {
            self.analytics_events_client.track_response(
                request_id.connection_id.0,
                ClientResponse::ThreadFork {
                    request_id: request_id.request_id.clone(),
                    response: response.clone(),
                },
            );
        }

        let connection_id = request_id.connection_id;
        let token_usage_thread = response.thread.clone();
        let token_usage_turn_id = if let Some(turn_id) =
            latest_token_usage_turn_id_for_thread_path(&token_usage_thread).await
        {
            Some(turn_id)
        } else {
            latest_token_usage_turn_id_from_rollout_path(
                rollout_path.as_path(),
                &token_usage_thread,
            )
            .await
        };
        self.outgoing.send_response(request_id, response).await;
        // Mirror the resume contract for forks: the new thread is usable as soon
        // as the response arrives, so restored usage must follow immediately.
        send_thread_token_usage_update_to_connection(
            &self.outgoing,
            connection_id,
            thread_id,
            &token_usage_thread,
            forked_thread.as_ref(),
            token_usage_turn_id,
        )
        .await;

        let notif = ThreadStartedNotification { thread };
        self.outgoing
            .send_server_notification(ServerNotification::ThreadStarted(notif))
            .await;
    }

    async fn get_thread_summary(
        &self,
        request_id: ConnectionRequestId,
        params: GetConversationSummaryParams,
    ) {
        let fallback_provider = self.config.model_provider_id.as_str();
        let read_result = match params {
            GetConversationSummaryParams::ThreadId { conversation_id } => self
                .thread_store
                .read_thread(StoreReadThreadParams {
                    thread_id: conversation_id,
                    include_archived: true,
                    include_history: false,
                })
                .await
                .map_err(|err| conversation_summary_thread_id_read_error(conversation_id, err)),
            GetConversationSummaryParams::RolloutPath { rollout_path } => {
                let Some(local_thread_store) = self
                    .thread_store
                    .as_any()
                    .downcast_ref::<LocalThreadStore>()
                else {
                    let error = JSONRPCErrorError {
                        code: INVALID_REQUEST_ERROR_CODE,
                        message:
                            "rollout path queries are only supported with the local thread store"
                                .to_string(),
                        data: None,
                    };
                    return self.outgoing.send_error(request_id, error).await;
                };

                local_thread_store
                    .read_thread_by_rollout_path(
                        rollout_path.clone(),
                        /*include_archived*/ true,
                        /*include_history*/ false,
                    )
                    .await
                    .map_err(|err| conversation_summary_rollout_path_read_error(&rollout_path, err))
            }
        };

        match read_result {
            Ok(stored_thread) => {
                let Some(summary) = summary_from_stored_thread(stored_thread, fallback_provider)
                else {
                    let error = JSONRPCErrorError {
                        code: INTERNAL_ERROR_CODE,
                        message:
                            "failed to load conversation summary: thread is missing rollout path"
                                .to_string(),
                        data: None,
                    };
                    self.outgoing.send_error(request_id, error).await;
                    return;
                };
                let response = GetConversationSummaryResponse { summary };
                self.outgoing.send_response(request_id, response).await;
            }
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
            }
        }
    }

    async fn list_threads_common(
        &self,
        requested_page_size: usize,
        cursor: Option<String>,
        sort_key: StoreThreadSortKey,
        sort_direction: SortDirection,
        filters: ThreadListFilters,
    ) -> Result<(Vec<ConversationSummary>, Option<String>), JSONRPCErrorError> {
        let ThreadListFilters {
            model_providers,
            source_kinds,
            archived,
            cwd_filters,
            search_term,
            use_state_db_only,
        } = filters;
        let mut cursor_obj = cursor;
        let mut last_cursor = cursor_obj.clone();
        let mut remaining = requested_page_size;
        let mut items = Vec::with_capacity(requested_page_size);
        let mut next_cursor: Option<String> = None;

        let model_provider_filter = match model_providers {
            Some(providers) => {
                if providers.is_empty() {
                    None
                } else {
                    Some(providers)
                }
            }
            None => Some(vec![self.config.model_provider_id.clone()]),
        };
        let fallback_provider = self.config.model_provider_id.clone();
        let (allowed_sources_vec, source_kind_filter) = compute_source_filters(source_kinds);
        let allowed_sources = allowed_sources_vec.as_slice();
        let store_sort_direction = match sort_direction {
            SortDirection::Asc => StoreSortDirection::Asc,
            SortDirection::Desc => StoreSortDirection::Desc,
        };

        while remaining > 0 {
            let page_size = remaining.min(THREAD_LIST_MAX_LIMIT);
            let page = self
                .thread_store
                .list_threads(StoreListThreadsParams {
                    page_size,
                    cursor: cursor_obj.clone(),
                    sort_key,
                    sort_direction: store_sort_direction,
                    allowed_sources: allowed_sources.to_vec(),
                    model_providers: model_provider_filter.clone(),
                    cwd_filters: cwd_filters.clone(),
                    archived,
                    search_term: search_term.clone(),
                    use_state_db_only,
                })
                .await
                .map_err(thread_store_list_error)?;

            let mut filtered = Vec::with_capacity(page.items.len());
            for it in page.items {
                let Some(summary) = summary_from_stored_thread(it, fallback_provider.as_str())
                else {
                    continue;
                };
                if source_kind_filter
                    .as_ref()
                    .is_none_or(|filter| source_kind_matches(&summary.source, filter))
                    && cwd_filters.as_ref().is_none_or(|expected_cwds| {
                        expected_cwds.iter().any(|expected_cwd| {
                            path_utils::paths_match_after_normalization(&summary.cwd, expected_cwd)
                        })
                    })
                {
                    filtered.push(summary);
                    if filtered.len() >= remaining {
                        break;
                    }
                }
            }
            items.extend(filtered);
            remaining = requested_page_size.saturating_sub(items.len());

            next_cursor = page.next_cursor;
            if remaining == 0 {
                break;
            }

            let Some(cursor_val) = next_cursor.clone() else {
                break;
            };
            // Break if our pagination would reuse the same cursor again; this avoids
            // an infinite loop when filtering drops everything on the page.
            if last_cursor.as_ref() == Some(&cursor_val) {
                next_cursor = None;
                break;
            }
            last_cursor = Some(cursor_val.clone());
            cursor_obj = Some(cursor_val);
        }

        Ok((items, next_cursor))
    }

    async fn send_invalid_request_error(&self, request_id: ConnectionRequestId, message: String) {
        let error = JSONRPCErrorError {
            code: INVALID_REQUEST_ERROR_CODE,
            message,
            data: None,
        };
        self.outgoing.send_error(request_id, error).await;
    }

    fn input_too_large_error(actual_chars: usize) -> JSONRPCErrorError {
        JSONRPCErrorError {
            code: INVALID_PARAMS_ERROR_CODE,
            message: format!(
                "Input exceeds the maximum length of {MAX_USER_INPUT_TEXT_CHARS} characters."
            ),
            data: Some(serde_json::json!({
                "input_error_code": INPUT_TOO_LARGE_ERROR_CODE,
                "max_chars": MAX_USER_INPUT_TEXT_CHARS,
                "actual_chars": actual_chars,
            })),
        }
    }

    fn validate_v2_input_limit(items: &[V2UserInput]) -> Result<(), JSONRPCErrorError> {
        let actual_chars: usize = items.iter().map(V2UserInput::text_char_count).sum();
        if actual_chars > MAX_USER_INPUT_TEXT_CHARS {
            return Err(Self::input_too_large_error(actual_chars));
        }
        Ok(())
    }

    async fn send_internal_error(&self, request_id: ConnectionRequestId, message: String) {
        let error = JSONRPCErrorError {
            code: INTERNAL_ERROR_CODE,
            message,
            data: None,
        };
        self.outgoing.send_error(request_id, error).await;
    }

    async fn wait_for_thread_shutdown(thread: &Arc<CodexThread>) -> ThreadShutdownResult {
        match tokio::time::timeout(Duration::from_secs(10), thread.shutdown_and_wait()).await {
            Ok(Ok(())) => ThreadShutdownResult::Complete,
            Ok(Err(_)) => ThreadShutdownResult::SubmitFailed,
            Err(_) => ThreadShutdownResult::TimedOut,
        }
    }

    async fn finalize_thread_teardown(&self, thread_id: ThreadId) {
        self.pending_thread_unloads.lock().await.remove(&thread_id);
        self.outgoing
            .cancel_requests_for_thread(thread_id, /*error*/ None)
            .await;
        self.thread_state_manager
            .remove_thread_state(thread_id)
            .await;
        self.epiphany_invalidation_manager
            .remove_thread(&thread_id.to_string())
            .await;
        self.thread_watch_manager
            .remove_thread(&thread_id.to_string())
            .await;
    }

    async fn unload_thread_without_subscribers(
        thread_manager: Arc<ThreadManager>,
        outgoing: Arc<OutgoingMessageSender>,
        pending_thread_unloads: Arc<Mutex<HashSet<ThreadId>>>,
        thread_state_manager: ThreadStateManager,
        thread_watch_manager: ThreadWatchManager,
        epiphany_invalidation_manager: EpiphanyInvalidationManager,
        thread_id: ThreadId,
        thread: Arc<CodexThread>,
    ) {
        info!("thread {thread_id} has no subscribers and is idle; shutting down");

        // Any pending app-server -> client requests for this thread can no longer be
        // answered; cancel their callbacks before shutdown/unload.
        outgoing
            .cancel_requests_for_thread(thread_id, /*error*/ None)
            .await;
        thread_state_manager.remove_thread_state(thread_id).await;
        epiphany_invalidation_manager
            .remove_thread(&thread_id.to_string())
            .await;

        tokio::spawn(async move {
            match Self::wait_for_thread_shutdown(&thread).await {
                ThreadShutdownResult::Complete => {
                    if thread_manager.remove_thread(&thread_id).await.is_none() {
                        info!("thread {thread_id} was already removed before teardown finalized");
                        epiphany_invalidation_manager
                            .remove_thread(&thread_id.to_string())
                            .await;
                        thread_watch_manager
                            .remove_thread(&thread_id.to_string())
                            .await;
                        pending_thread_unloads.lock().await.remove(&thread_id);
                        return;
                    }
                    epiphany_invalidation_manager
                        .remove_thread(&thread_id.to_string())
                        .await;
                    thread_watch_manager
                        .remove_thread(&thread_id.to_string())
                        .await;
                    let notification = ThreadClosedNotification {
                        thread_id: thread_id.to_string(),
                    };
                    outgoing
                        .send_server_notification(ServerNotification::ThreadClosed(notification))
                        .await;
                    pending_thread_unloads.lock().await.remove(&thread_id);
                }
                ThreadShutdownResult::SubmitFailed => {
                    pending_thread_unloads.lock().await.remove(&thread_id);
                    warn!("failed to submit Shutdown to thread {thread_id}");
                }
                ThreadShutdownResult::TimedOut => {
                    pending_thread_unloads.lock().await.remove(&thread_id);
                    warn!("thread {thread_id} shutdown timed out; leaving thread loaded");
                }
            }
        });
    }

    async fn thread_unsubscribe(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadUnsubscribeParams,
    ) {
        let thread_id = match ThreadId::from_string(&params.thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        if self.thread_manager.get_thread(thread_id).await.is_err() {
            // Reconcile stale app-server bookkeeping when the thread has already been
            // removed from the core manager. This keeps loaded-status/subscription state
            // consistent with the source of truth before reporting NotLoaded.
            self.finalize_thread_teardown(thread_id).await;
            self.outgoing
                .send_response(
                    request_id,
                    ThreadUnsubscribeResponse {
                        status: ThreadUnsubscribeStatus::NotLoaded,
                    },
                )
                .await;
            return;
        };

        let was_subscribed = self
            .thread_state_manager
            .unsubscribe_connection_from_thread(thread_id, request_id.connection_id)
            .await;

        let status = if was_subscribed {
            ThreadUnsubscribeStatus::Unsubscribed
        } else {
            ThreadUnsubscribeStatus::NotSubscribed
        };
        self.outgoing
            .send_response(request_id, ThreadUnsubscribeResponse { status })
            .await;
    }

    async fn prepare_thread_for_archive(&self, thread_id: ThreadId) {
        // If the thread is active, request shutdown and wait briefly.
        let removed_conversation = self.thread_manager.remove_thread(&thread_id).await;
        if let Some(conversation) = removed_conversation {
            info!("thread {thread_id} was active; shutting down");
            match Self::wait_for_thread_shutdown(&conversation).await {
                ThreadShutdownResult::Complete => {}
                ThreadShutdownResult::SubmitFailed => {
                    error!(
                        "failed to submit Shutdown to thread {thread_id}; proceeding with archive"
                    );
                }
                ThreadShutdownResult::TimedOut => {
                    warn!("thread {thread_id} shutdown timed out; proceeding with archive");
                }
            }
        }
        self.finalize_thread_teardown(thread_id).await;
    }

    async fn turn_start(
        &self,
        request_id: ConnectionRequestId,
        params: TurnStartParams,
        app_server_client_name: Option<String>,
        app_server_client_version: Option<String>,
    ) {
        if let Err(error) = Self::validate_v2_input_limit(&params.input) {
            self.track_error_response(
                &request_id,
                &error,
                Some(AnalyticsJsonRpcError::Input(InputError::TooLarge)),
            );
            self.outgoing.send_error(request_id, error).await;
            return;
        }
        let (_, thread) = match self.load_thread(&params.thread_id).await {
            Ok(v) => v,
            Err(error) => {
                self.track_error_response(&request_id, &error, /*error_type*/ None);
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };
        if let Err(error) = Self::set_app_server_client_info(
            thread.as_ref(),
            app_server_client_name,
            app_server_client_version,
        )
        .await
        {
            self.track_error_response(&request_id, &error, /*error_type*/ None);
            self.outgoing.send_error(request_id, error).await;
            return;
        }

        let collaboration_modes_config = CollaborationModesConfig {
            default_mode_request_user_input: thread.enabled(Feature::DefaultModeRequestUserInput),
        };
        let collaboration_mode = params.collaboration_mode.map(|mode| {
            self.normalize_turn_start_collaboration_mode(mode, collaboration_modes_config)
        });
        let environments = params.environments.map(|environments| {
            environments
                .into_iter()
                .map(|environment| TurnEnvironmentSelection {
                    environment_id: environment.environment_id,
                    cwd: environment.cwd,
                })
                .collect()
        });

        // Map v2 input items to core input items.
        let mapped_items: Vec<CoreInputItem> = params
            .input
            .into_iter()
            .map(V2UserInput::into_core)
            .collect();

        let has_any_overrides = params.cwd.is_some()
            || params.approval_policy.is_some()
            || params.approvals_reviewer.is_some()
            || params.sandbox_policy.is_some()
            || params.permission_profile.is_some()
            || params.model.is_some()
            || params.service_tier.is_some()
            || params.effort.is_some()
            || params.summary.is_some()
            || collaboration_mode.is_some()
            || params.personality.is_some();

        if params.sandbox_policy.is_some() && params.permission_profile.is_some() {
            self.send_invalid_request_error(
                request_id,
                "`permissionProfile` cannot be combined with `sandboxPolicy`".to_string(),
            )
            .await;
            return;
        }

        let cwd = params.cwd;
        let approval_policy = params.approval_policy.map(AskForApproval::to_core);
        let approvals_reviewer = params
            .approvals_reviewer
            .map(codex_app_server_protocol::ApprovalsReviewer::to_core);
        let sandbox_policy = params.sandbox_policy.map(|p| p.to_core());
        let permission_profile = params.permission_profile.map(Into::into);
        let model = params.model;
        let effort = params.effort.map(Some);
        let summary = params.summary;
        let service_tier = params.service_tier;
        let personality = params.personality;

        // If any overrides are provided, validate them synchronously so the
        // request can fail before accepting user input. The actual update is
        // still queued together with the input below to preserve submission order.
        if has_any_overrides {
            let result = thread
                .validate_turn_context_overrides(CodexThreadTurnContextOverrides {
                    cwd: cwd.clone(),
                    approval_policy,
                    approvals_reviewer,
                    sandbox_policy: sandbox_policy.clone(),
                    permission_profile: permission_profile.clone(),
                    windows_sandbox_level: None,
                    model: model.clone(),
                    effort,
                    summary,
                    service_tier,
                    collaboration_mode: collaboration_mode.clone(),
                    personality,
                })
                .await;
            if let Err(err) = result {
                self.send_invalid_request_error(
                    request_id,
                    format!("invalid turn context override: {err}"),
                )
                .await;
                return;
            }
        }

        // Start the turn by submitting the user input. Return its submission id as turn_id.
        let turn_op = if has_any_overrides {
            Op::UserInputWithTurnContext {
                items: mapped_items,
                environments,
                final_output_json_schema: params.output_schema,
                responsesapi_client_metadata: params.responsesapi_client_metadata,
                cwd,
                approval_policy,
                approvals_reviewer,
                sandbox_policy,
                permission_profile,
                windows_sandbox_level: None,
                model,
                effort,
                summary,
                service_tier,
                collaboration_mode,
                personality,
            }
        } else {
            Op::UserInput {
                items: mapped_items,
                environments,
                final_output_json_schema: params.output_schema,
                responsesapi_client_metadata: params.responsesapi_client_metadata,
            }
        };
        let turn_id = self
            .submit_core_op(&request_id, thread.as_ref(), turn_op)
            .await;

        match turn_id {
            Ok(turn_id) => {
                self.outgoing
                    .record_request_turn_id(&request_id, &turn_id)
                    .await;
                let turn = Turn {
                    id: turn_id.clone(),
                    items: vec![],
                    error: None,
                    status: TurnStatus::InProgress,
                    started_at: None,
                    completed_at: None,
                    duration_ms: None,
                };

                let response = TurnStartResponse { turn };
                if self.config.features.enabled(Feature::GeneralAnalytics) {
                    self.analytics_events_client.track_response(
                        request_id.connection_id.0,
                        ClientResponse::TurnStart {
                            request_id: request_id.request_id.clone(),
                            response: response.clone(),
                        },
                    );
                }
                self.outgoing.send_response(request_id, response).await;
            }
            Err(err) => {
                let error = JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message: format!("failed to start turn: {err}"),
                    data: None,
                };
                self.track_error_response(&request_id, &error, /*error_type*/ None);
                self.outgoing.send_error(request_id, error).await;
            }
        }
    }

    async fn thread_inject_items(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadInjectItemsParams,
    ) {
        let (_, thread) = match self.load_thread(&params.thread_id).await {
            Ok(value) => value,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        let items = match params
            .items
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                serde_json::from_value::<ResponseItem>(value)
                    .map_err(|err| format!("items[{index}] is not a valid response item: {err}"))
            })
            .collect::<std::result::Result<Vec<_>, _>>()
        {
            Ok(items) => items,
            Err(message) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
        };

        match thread.inject_response_items(items).await {
            Ok(()) => {
                self.outgoing
                    .send_response(request_id, ThreadInjectItemsResponse {})
                    .await;
            }
            Err(CodexErr::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to inject response items: {err}"),
                )
                .await;
            }
        }
    }

    async fn set_app_server_client_info(
        thread: &CodexThread,
        app_server_client_name: Option<String>,
        app_server_client_version: Option<String>,
    ) -> Result<(), JSONRPCErrorError> {
        thread
            .set_app_server_client_info(app_server_client_name, app_server_client_version)
            .await
            .map_err(|err| JSONRPCErrorError {
                code: INTERNAL_ERROR_CODE,
                message: format!("failed to set app server client info: {err}"),
                data: None,
            })
    }

    async fn turn_steer(&self, request_id: ConnectionRequestId, params: TurnSteerParams) {
        let (_, thread) = match self.load_thread(&params.thread_id).await {
            Ok(v) => v,
            Err(error) => {
                self.track_error_response(&request_id, &error, /*error_type*/ None);
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        if params.expected_turn_id.is_empty() {
            self.send_invalid_request_error(
                request_id,
                "expectedTurnId must not be empty".to_string(),
            )
            .await;
            return;
        }
        self.outgoing
            .record_request_turn_id(&request_id, &params.expected_turn_id)
            .await;
        if let Err(error) = Self::validate_v2_input_limit(&params.input) {
            self.track_error_response(
                &request_id,
                &error,
                Some(AnalyticsJsonRpcError::Input(InputError::TooLarge)),
            );
            self.outgoing.send_error(request_id, error).await;
            return;
        }

        let mapped_items: Vec<CoreInputItem> = params
            .input
            .into_iter()
            .map(V2UserInput::into_core)
            .collect();

        match thread
            .steer_input(
                mapped_items,
                Some(&params.expected_turn_id),
                params.responsesapi_client_metadata,
            )
            .await
        {
            Ok(turn_id) => {
                let response = TurnSteerResponse { turn_id };
                if self.config.features.enabled(Feature::GeneralAnalytics) {
                    self.analytics_events_client.track_response(
                        request_id.connection_id.0,
                        ClientResponse::TurnSteer {
                            request_id: request_id.request_id.clone(),
                            response: response.clone(),
                        },
                    );
                }
                self.outgoing.send_response(request_id, response).await;
            }
            Err(err) => {
                let (code, message, data, error_type) = match err {
                    SteerInputError::NoActiveTurn(_) => (
                        INVALID_REQUEST_ERROR_CODE,
                        "no active turn to steer".to_string(),
                        None,
                        Some(AnalyticsJsonRpcError::TurnSteer(
                            TurnSteerRequestError::NoActiveTurn,
                        )),
                    ),
                    SteerInputError::ExpectedTurnMismatch { expected, actual } => (
                        INVALID_REQUEST_ERROR_CODE,
                        format!("expected active turn id `{expected}` but found `{actual}`"),
                        None,
                        Some(AnalyticsJsonRpcError::TurnSteer(
                            TurnSteerRequestError::ExpectedTurnMismatch,
                        )),
                    ),
                    SteerInputError::ActiveTurnNotSteerable { turn_kind } => {
                        let (message, turn_steer_error) = match turn_kind {
                            codex_protocol::protocol::NonSteerableTurnKind::Review => (
                                "cannot steer a review turn".to_string(),
                                TurnSteerRequestError::NonSteerableReview,
                            ),
                            codex_protocol::protocol::NonSteerableTurnKind::Compact => (
                                "cannot steer a compact turn".to_string(),
                                TurnSteerRequestError::NonSteerableCompact,
                            ),
                        };
                        let error = TurnError {
                            message: message.clone(),
                            codex_error_info: Some(CodexErrorInfo::ActiveTurnNotSteerable {
                                turn_kind: turn_kind.into(),
                            }),
                            additional_details: None,
                        };
                        let data = match serde_json::to_value(error) {
                            Ok(data) => Some(data),
                            Err(error) => {
                                tracing::error!(
                                    ?error,
                                    "failed to serialize active-turn-not-steerable turn error"
                                );
                                None
                            }
                        };
                        (
                            INVALID_REQUEST_ERROR_CODE,
                            message,
                            data,
                            Some(AnalyticsJsonRpcError::TurnSteer(turn_steer_error)),
                        )
                    }
                    SteerInputError::EmptyInput => (
                        INVALID_REQUEST_ERROR_CODE,
                        "input must not be empty".to_string(),
                        None,
                        Some(AnalyticsJsonRpcError::Input(InputError::Empty)),
                    ),
                };
                let error = JSONRPCErrorError {
                    code,
                    message,
                    data,
                };
                self.track_error_response(&request_id, &error, error_type);
                self.outgoing.send_error(request_id, error).await;
            }
        }
    }

    async fn turn_interrupt(&self, request_id: ConnectionRequestId, params: TurnInterruptParams) {
        let TurnInterruptParams { thread_id, turn_id } = params;
        let is_startup_interrupt = turn_id.is_empty();
        if !is_startup_interrupt {
            self.outgoing
                .record_request_turn_id(&request_id, &turn_id)
                .await;
        }

        let (thread_uuid, thread) = match self.load_thread(&thread_id).await {
            Ok(v) => v,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        // Record turn interrupts so we can reply when TurnAborted arrives. Startup
        // interrupts do not have a turn and are acknowledged after submission.
        if !is_startup_interrupt {
            let thread_state = self.thread_state_manager.thread_state(thread_uuid).await;
            let mut thread_state = thread_state.lock().await;
            thread_state
                .pending_interrupts
                .push((request_id.clone(), ApiVersion::V2));
        }

        // Submit the interrupt. Turn interrupts respond upon TurnAborted; startup
        // interrupts respond here because startup cancellation has no turn event.
        let submit_result = self
            .submit_core_op(&request_id, thread.as_ref(), Op::Interrupt)
            .await;
        match submit_result {
            Ok(_) if is_startup_interrupt => {
                self.outgoing
                    .send_response(request_id, TurnInterruptResponse {})
                    .await;
            }
            Ok(_) => {}
            Err(err) => {
                if !is_startup_interrupt {
                    let thread_state = self.thread_state_manager.thread_state(thread_uuid).await;
                    let mut thread_state = thread_state.lock().await;
                    thread_state
                        .pending_interrupts
                        .retain(|(pending_request_id, _)| pending_request_id != &request_id);
                }
                let interrupt_target = if is_startup_interrupt {
                    "startup"
                } else {
                    "turn"
                };
                self.send_internal_error(
                    request_id,
                    format!("failed to interrupt {interrupt_target}: {err}"),
                )
                .await;
            }
        }
    }

    async fn ensure_conversation_listener(
        &self,
        conversation_id: ThreadId,
        connection_id: ConnectionId,
        raw_events_enabled: bool,
        api_version: ApiVersion,
    ) -> Result<EnsureConversationListenerResult, JSONRPCErrorError> {
        Self::ensure_conversation_listener_task(
            ListenerTaskContext {
                thread_manager: Arc::clone(&self.thread_manager),
                thread_state_manager: self.thread_state_manager.clone(),
                outgoing: Arc::clone(&self.outgoing),
                pending_thread_unloads: Arc::clone(&self.pending_thread_unloads),
                analytics_events_client: self.analytics_events_client.clone(),
                general_analytics_enabled: self.config.features.enabled(Feature::GeneralAnalytics),
                thread_watch_manager: self.thread_watch_manager.clone(),
                epiphany_invalidation_manager: self.epiphany_invalidation_manager.clone(),
                fallback_model_provider: self.config.model_provider_id.clone(),
                codex_home: self.config.codex_home.to_path_buf(),
            },
            conversation_id,
            connection_id,
            raw_events_enabled,
            api_version,
        )
        .await
    }

    #[expect(
        clippy::await_holding_invalid_type,
        reason = "listener subscription must be serialized against pending thread unloads"
    )]
    async fn ensure_conversation_listener_task(
        listener_task_context: ListenerTaskContext,
        conversation_id: ThreadId,
        connection_id: ConnectionId,
        raw_events_enabled: bool,
        api_version: ApiVersion,
    ) -> Result<EnsureConversationListenerResult, JSONRPCErrorError> {
        let conversation = match listener_task_context
            .thread_manager
            .get_thread(conversation_id)
            .await
        {
            Ok(conv) => conv,
            Err(_) => {
                return Err(JSONRPCErrorError {
                    code: INVALID_REQUEST_ERROR_CODE,
                    message: format!("thread not found: {conversation_id}"),
                    data: None,
                });
            }
        };
        let thread_state = {
            let pending_thread_unloads = listener_task_context.pending_thread_unloads.lock().await;
            if pending_thread_unloads.contains(&conversation_id) {
                return Err(JSONRPCErrorError {
                    code: INVALID_REQUEST_ERROR_CODE,
                    message: format!(
                        "thread {conversation_id} is closing; retry after the thread is closed"
                    ),
                    data: None,
                });
            }
            let Some(thread_state) = listener_task_context
                .thread_state_manager
                .try_ensure_connection_subscribed(
                    conversation_id,
                    connection_id,
                    raw_events_enabled,
                )
                .await
            else {
                return Ok(EnsureConversationListenerResult::ConnectionClosed);
            };
            thread_state
        };
        if let Err(error) = Self::ensure_listener_task_running_task(
            listener_task_context.clone(),
            conversation_id,
            conversation,
            thread_state,
            api_version,
        )
        .await
        {
            let _ = listener_task_context
                .thread_state_manager
                .unsubscribe_connection_from_thread(conversation_id, connection_id)
                .await;
            return Err(error);
        }
        Ok(EnsureConversationListenerResult::Attached)
    }

    fn log_listener_attach_result(
        result: Result<EnsureConversationListenerResult, JSONRPCErrorError>,
        thread_id: ThreadId,
        connection_id: ConnectionId,
        thread_kind: &'static str,
    ) {
        match result {
            Ok(EnsureConversationListenerResult::Attached) => {}
            Ok(EnsureConversationListenerResult::ConnectionClosed) => {
                tracing::debug!(
                    thread_id = %thread_id,
                    connection_id = ?connection_id,
                    "skipping auto-attach for closed connection"
                );
            }
            Err(err) => {
                tracing::warn!(
                    "failed to attach listener for {thread_kind} {thread_id}: {message}",
                    message = err.message
                );
            }
        }
    }

    async fn ensure_listener_task_running(
        &self,
        conversation_id: ThreadId,
        conversation: Arc<CodexThread>,
        thread_state: Arc<Mutex<ThreadState>>,
        api_version: ApiVersion,
    ) -> Result<(), JSONRPCErrorError> {
        Self::ensure_listener_task_running_task(
            ListenerTaskContext {
                thread_manager: Arc::clone(&self.thread_manager),
                thread_state_manager: self.thread_state_manager.clone(),
                outgoing: Arc::clone(&self.outgoing),
                pending_thread_unloads: Arc::clone(&self.pending_thread_unloads),
                analytics_events_client: self.analytics_events_client.clone(),
                general_analytics_enabled: self.config.features.enabled(Feature::GeneralAnalytics),
                thread_watch_manager: self.thread_watch_manager.clone(),
                epiphany_invalidation_manager: self.epiphany_invalidation_manager.clone(),
                fallback_model_provider: self.config.model_provider_id.clone(),
                codex_home: self.config.codex_home.to_path_buf(),
            },
            conversation_id,
            conversation,
            thread_state,
            api_version,
        )
        .await
    }

    async fn ensure_listener_task_running_task(
        listener_task_context: ListenerTaskContext,
        conversation_id: ThreadId,
        conversation: Arc<CodexThread>,
        thread_state: Arc<Mutex<ThreadState>>,
        api_version: ApiVersion,
    ) -> Result<(), JSONRPCErrorError> {
        let (cancel_tx, mut cancel_rx) = oneshot::channel();
        let Some(mut unloading_state) = UnloadingState::new(
            &listener_task_context,
            conversation_id,
            THREAD_UNLOADING_DELAY,
        )
        .await
        else {
            return Err(JSONRPCErrorError {
                code: INVALID_REQUEST_ERROR_CODE,
                message: format!(
                    "thread {conversation_id} is closing; retry after the thread is closed"
                ),
                data: None,
            });
        };
        let (mut listener_command_rx, listener_generation) = {
            let mut thread_state = thread_state.lock().await;
            if thread_state.listener_matches(&conversation) {
                return Ok(());
            }
            thread_state.set_listener(cancel_tx, &conversation)
        };
        let ListenerTaskContext {
            outgoing,
            thread_manager,
            thread_state_manager,
            pending_thread_unloads,
            analytics_events_client: _,
            general_analytics_enabled: _,
            thread_watch_manager,
            epiphany_invalidation_manager,
            fallback_model_provider,
            codex_home,
        } = listener_task_context;
        let outgoing_for_task = Arc::clone(&outgoing);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = &mut cancel_rx => {
                        // Listener was superseded or the thread is being torn down.
                        break;
                    }
                    listener_command = listener_command_rx.recv() => {
                        let Some(listener_command) = listener_command else {
                            break;
                        };
                        handle_thread_listener_command(
                            conversation_id,
                            &conversation,
                            codex_home.as_path(),
                            &thread_state_manager,
                            &thread_state,
                            &thread_watch_manager,
                            &outgoing_for_task,
                            &pending_thread_unloads,
                            listener_command,
                        )
                        .await;
                    }
                    event = conversation.next_event() => {
                        let event = match event {
                            Ok(event) => event,
                            Err(err) => {
                                tracing::warn!("thread.next_event() failed with: {err}");
                                break;
                            }
                        };

                        // Track the event before emitting any typed
                        // translations so thread-local state such as raw event
                        // opt-in stays synchronized with the conversation.
                        let raw_events_enabled = {
                            let mut thread_state = thread_state.lock().await;
                            thread_state.track_current_turn_event(&event.msg);
                            thread_state.experimental_raw_events
                        };
                        let subscribed_connection_ids = thread_state_manager
                            .subscribed_connection_ids(conversation_id)
                            .await;
                        let thread_outgoing = ThreadScopedOutgoingMessageSender::new(
                            outgoing_for_task.clone(),
                            subscribed_connection_ids,
                            conversation_id,
                        );

                        if let EventMsg::RawResponseItem(raw_response_item_event) = &event.msg
                            && !raw_events_enabled
                        {
                            maybe_emit_hook_prompt_item_completed(
                                api_version,
                                conversation_id,
                                &event.id,
                                &raw_response_item_event.item,
                                &thread_outgoing,
                            )
                            .await;
                            continue;
                        }

                        apply_bespoke_event_handling(
                            event.clone(),
                            conversation_id,
                            conversation.clone(),
                            thread_manager.clone(),
                            listener_task_context
                                .general_analytics_enabled
                                .then(|| listener_task_context.analytics_events_client.clone()),
                            thread_outgoing,
                            thread_state.clone(),
                            thread_watch_manager.clone(),
                            epiphany_invalidation_manager.clone(),
                            api_version,
                            fallback_model_provider.clone(),
                            codex_home.as_path(),
                        )
                        .await;
                    }
                    unloading_watchers_open = unloading_state.wait_for_unloading_trigger() => {
                        if !unloading_watchers_open {
                            break;
                        }
                        if !unloading_state.should_unload_now() {
                            continue;
                        }
                        if matches!(conversation.agent_status().await, AgentStatus::Running) {
                            unloading_state.note_thread_activity_observed();
                            continue;
                        }
                        {
                            let mut pending_thread_unloads = pending_thread_unloads.lock().await;
                            if pending_thread_unloads.contains(&conversation_id) {
                                continue;
                            }
                            if !unloading_state.should_unload_now() {
                                continue;
                            }
                            pending_thread_unloads.insert(conversation_id);
                        }
                        Self::unload_thread_without_subscribers(
                            thread_manager.clone(),
                            outgoing_for_task.clone(),
                            pending_thread_unloads.clone(),
                            thread_state_manager.clone(),
                            thread_watch_manager.clone(),
                            epiphany_invalidation_manager.clone(),
                            conversation_id,
                            conversation.clone(),
                        )
                        .await;
                        break;
                    }
                }
            }

            let mut thread_state = thread_state.lock().await;
            if thread_state.listener_generation == listener_generation {
                thread_state.clear_listener();
            }
        });
        Ok(())
    }
    async fn resolve_rollout_path(
        &self,
        conversation_id: ThreadId,
        state_db_ctx: Option<&StateDbHandle>,
    ) -> Option<PathBuf> {
        if let Ok(conversation) = self.thread_manager.get_thread(conversation_id).await
            && let Some(rollout_path) = conversation.rollout_path()
        {
            return Some(rollout_path);
        }

        let state_db_ctx = state_db_ctx?;
        state_db_ctx
            .find_rollout_path_by_id(conversation_id, /*archived_only*/ None)
            .await
            .unwrap_or_else(|err| {
                warn!("failed to resolve rollout path for thread_id={conversation_id}: {err}");
                None
            })
    }
}

fn normalize_thread_list_cwd_filters(
    cwd: Option<ThreadListCwdFilter>,
) -> Result<Option<Vec<PathBuf>>, JSONRPCErrorError> {
    let Some(cwd) = cwd else {
        return Ok(None);
    };

    let cwds = match cwd {
        ThreadListCwdFilter::One(cwd) => vec![cwd],
        ThreadListCwdFilter::Many(cwds) => cwds,
    };
    let mut normalized_cwds = Vec::with_capacity(cwds.len());
    for cwd in cwds {
        let cwd = AbsolutePathBuf::relative_to_current_dir(cwd.as_str())
            .map(AbsolutePathBuf::into_path_buf)
            .map_err(|err| JSONRPCErrorError {
                code: INVALID_PARAMS_ERROR_CODE,
                message: format!("invalid thread/list cwd filter `{cwd}`: {err}"),
                data: None,
            })?;
        normalized_cwds.push(cwd);
    }

    Ok(Some(normalized_cwds))
}

#[cfg(test)]
mod thread_list_cwd_filter_tests {
    use super::normalize_thread_list_cwd_filters;
    use codex_app_server_protocol::ThreadListCwdFilter;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn normalize_thread_list_cwd_filter_preserves_absolute_paths() {
        let cwd = if cfg!(windows) {
            String::from(r"C:\srv\repo-b")
        } else {
            String::from("/srv/repo-b")
        };

        assert_eq!(
            normalize_thread_list_cwd_filters(Some(ThreadListCwdFilter::One(cwd.clone())))
                .expect("cwd filter should parse"),
            Some(vec![PathBuf::from(cwd)])
        );
    }

    #[test]
    fn normalize_thread_list_cwd_filter_resolves_relative_paths_against_server_cwd()
    -> std::io::Result<()> {
        let expected = AbsolutePathBuf::relative_to_current_dir("repo-b")?.to_path_buf();

        assert_eq!(
            normalize_thread_list_cwd_filters(Some(ThreadListCwdFilter::Many(vec![String::from(
                "repo-b"
            ),])))
            .expect("cwd filter should parse"),
            Some(vec![expected])
        );
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
async fn handle_thread_listener_command(
    conversation_id: ThreadId,
    conversation: &Arc<CodexThread>,
    codex_home: &Path,
    thread_state_manager: &ThreadStateManager,
    thread_state: &Arc<Mutex<ThreadState>>,
    thread_watch_manager: &ThreadWatchManager,
    outgoing: &Arc<OutgoingMessageSender>,
    pending_thread_unloads: &Arc<Mutex<HashSet<ThreadId>>>,
    listener_command: ThreadListenerCommand,
) {
    match listener_command {
        ThreadListenerCommand::SendThreadResumeResponse(resume_request) => {
            handle_pending_thread_resume_request(
                conversation_id,
                conversation,
                codex_home,
                thread_state_manager,
                thread_state,
                thread_watch_manager,
                outgoing,
                pending_thread_unloads,
                *resume_request,
            )
            .await;
        }
        ThreadListenerCommand::ResolveServerRequest {
            request_id,
            completion_tx,
        } => {
            resolve_pending_server_request(
                conversation_id,
                thread_state_manager,
                outgoing,
                request_id,
            )
            .await;
            let _ = completion_tx.send(());
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[expect(
    clippy::await_holding_invalid_type,
    reason = "running-thread resume subscription must be serialized against pending unloads"
)]
async fn handle_pending_thread_resume_request(
    conversation_id: ThreadId,
    conversation: &Arc<CodexThread>,
    _codex_home: &Path,
    thread_state_manager: &ThreadStateManager,
    thread_state: &Arc<Mutex<ThreadState>>,
    thread_watch_manager: &ThreadWatchManager,
    outgoing: &Arc<OutgoingMessageSender>,
    pending_thread_unloads: &Arc<Mutex<HashSet<ThreadId>>>,
    pending: crate::thread_state::PendingThreadResumeRequest,
) {
    let active_turn = {
        let state = thread_state.lock().await;
        state.active_turn_snapshot()
    };
    tracing::debug!(
        thread_id = %conversation_id,
        request_id = ?pending.request_id,
        active_turn_present = active_turn.is_some(),
        active_turn_id = ?active_turn.as_ref().map(|turn| turn.id.as_str()),
        active_turn_status = ?active_turn.as_ref().map(|turn| &turn.status),
        "composing running thread resume response"
    );
    let has_live_in_progress_turn =
        matches!(conversation.agent_status().await, AgentStatus::Running)
            || active_turn
                .as_ref()
                .is_some_and(|turn| matches!(turn.status, TurnStatus::InProgress));

    let request_id = pending.request_id;
    let connection_id = request_id.connection_id;
    let mut thread = pending.thread_summary;
    if let Err(message) = populate_thread_turns(
        &mut thread,
        ThreadTurnSource::RolloutPath(pending.rollout_path.as_path()),
        active_turn.as_ref(),
    )
    .await
    {
        outgoing
            .send_error(
                request_id,
                JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message,
                    data: None,
                },
            )
            .await;
        return;
    }

    let thread_status = thread_watch_manager
        .loaded_status_for_thread(&thread.id)
        .await;

    set_thread_status_and_interrupt_stale_turns(
        &mut thread,
        thread_status,
        has_live_in_progress_turn,
    );

    {
        let pending_thread_unloads = pending_thread_unloads.lock().await;
        if pending_thread_unloads.contains(&conversation_id) {
            drop(pending_thread_unloads);
            outgoing
                .send_error(
                    request_id,
                    JSONRPCErrorError {
                        code: INVALID_REQUEST_ERROR_CODE,
                        message: format!(
                            "thread {conversation_id} is closing; retry thread/resume after the thread is closed"
                        ),
                        data: None,
                    },
                )
                .await;
            return;
        }
        if !thread_state_manager
            .try_add_connection_to_thread(conversation_id, connection_id)
            .await
        {
            tracing::debug!(
                thread_id = %conversation_id,
                connection_id = ?connection_id,
                "skipping running thread resume for closed connection"
            );
            return;
        }
    }

    let ThreadConfigSnapshot {
        model,
        model_provider_id,
        service_tier,
        approval_policy,
        approvals_reviewer,
        sandbox_policy,
        permission_profile,
        cwd,
        reasoning_effort,
        ..
    } = pending.config_snapshot;
    let instruction_sources = pending.instruction_sources;
    let permission_profile =
        thread_response_permission_profile(&sandbox_policy, permission_profile);

    let response = ThreadResumeResponse {
        thread,
        model,
        model_provider: model_provider_id,
        service_tier,
        cwd,
        instruction_sources,
        approval_policy: approval_policy.into(),
        approvals_reviewer: approvals_reviewer.into(),
        sandbox: sandbox_policy.into(),
        permission_profile,
        reasoning_effort,
    };
    let token_usage_thread = response.thread.clone();
    let token_usage_turn_id = latest_token_usage_turn_id_from_rollout_path(
        pending.rollout_path.as_path(),
        &token_usage_thread,
    )
    .await;
    outgoing.send_response(request_id, response).await;
    // Rejoining a loaded thread has the same UI contract as a cold resume, but
    // uses the live conversation state instead of reconstructing a new session.
    send_thread_token_usage_update_to_connection(
        outgoing,
        connection_id,
        conversation_id,
        &token_usage_thread,
        conversation.as_ref(),
        token_usage_turn_id,
    )
    .await;
    outgoing
        .replay_requests_to_connection_for_thread(connection_id, conversation_id)
        .await;
}

enum ThreadTurnSource<'a> {
    RolloutPath(&'a Path),
    HistoryItems(&'a [RolloutItem]),
}

async fn populate_thread_turns(
    thread: &mut Thread,
    turn_source: ThreadTurnSource<'_>,
    active_turn: Option<&Turn>,
) -> std::result::Result<(), String> {
    let mut turns = match turn_source {
        ThreadTurnSource::RolloutPath(rollout_path) => {
            read_rollout_items_from_rollout(rollout_path)
                .await
                .map(|items| build_turns_from_rollout_items(&items))
                .map_err(|err| {
                    format!(
                        "failed to load rollout `{}` for thread {}: {err}",
                        rollout_path.display(),
                        thread.id
                    )
                })?
        }
        ThreadTurnSource::HistoryItems(items) => build_turns_from_rollout_items(items),
    };
    if let Some(active_turn) = active_turn {
        merge_turn_history_with_active_turn(&mut turns, active_turn.clone());
    }
    thread.turns = turns;
    Ok(())
}

async fn resolve_pending_server_request(
    conversation_id: ThreadId,
    thread_state_manager: &ThreadStateManager,
    outgoing: &Arc<OutgoingMessageSender>,
    request_id: RequestId,
) {
    let thread_id = conversation_id.to_string();
    let subscribed_connection_ids = thread_state_manager
        .subscribed_connection_ids(conversation_id)
        .await;
    let outgoing = ThreadScopedOutgoingMessageSender::new(
        outgoing.clone(),
        subscribed_connection_ids,
        conversation_id,
    );
    outgoing
        .send_server_notification(ServerNotification::ServerRequestResolved(
            ServerRequestResolvedNotification {
                thread_id,
                request_id,
            },
        ))
        .await;
}

fn merge_turn_history_with_active_turn(turns: &mut Vec<Turn>, active_turn: Turn) {
    turns.retain(|turn| turn.id != active_turn.id);
    turns.push(active_turn);
}

fn set_thread_status_and_interrupt_stale_turns(
    thread: &mut Thread,
    loaded_status: ThreadStatus,
    has_live_in_progress_turn: bool,
) {
    let status = resolve_thread_status(loaded_status, has_live_in_progress_turn);
    if !matches!(status, ThreadStatus::Active { .. }) {
        for turn in &mut thread.turns {
            if matches!(turn.status, TurnStatus::InProgress) {
                turn.status = TurnStatus::Interrupted;
            }
        }
    }
    thread.status = status;
}

fn collect_resume_override_mismatches(
    request: &ThreadResumeParams,
    config_snapshot: &ThreadConfigSnapshot,
) -> Vec<String> {
    let mut mismatch_details = Vec::new();

    if let Some(requested_model) = request.model.as_deref()
        && requested_model != config_snapshot.model
    {
        mismatch_details.push(format!(
            "model requested={requested_model} active={}",
            config_snapshot.model
        ));
    }
    if let Some(requested_provider) = request.model_provider.as_deref()
        && requested_provider != config_snapshot.model_provider_id
    {
        mismatch_details.push(format!(
            "model_provider requested={requested_provider} active={}",
            config_snapshot.model_provider_id
        ));
    }
    if let Some(requested_service_tier) = request.service_tier.as_ref()
        && requested_service_tier != &config_snapshot.service_tier
    {
        mismatch_details.push(format!(
            "service_tier requested={requested_service_tier:?} active={:?}",
            config_snapshot.service_tier
        ));
    }
    if let Some(requested_cwd) = request.cwd.as_deref() {
        let requested_cwd_path = std::path::PathBuf::from(requested_cwd);
        if requested_cwd_path != config_snapshot.cwd.as_path() {
            mismatch_details.push(format!(
                "cwd requested={} active={}",
                requested_cwd_path.display(),
                config_snapshot.cwd.display()
            ));
        }
    }
    if let Some(requested_approval) = request.approval_policy.as_ref() {
        let active_approval: AskForApproval = config_snapshot.approval_policy.into();
        if requested_approval != &active_approval {
            mismatch_details.push(format!(
                "approval_policy requested={requested_approval:?} active={active_approval:?}"
            ));
        }
    }
    if let Some(requested_review_policy) = request.approvals_reviewer.as_ref() {
        let active_review_policy: codex_app_server_protocol::ApprovalsReviewer =
            config_snapshot.approvals_reviewer.into();
        if requested_review_policy != &active_review_policy {
            mismatch_details.push(format!(
                "approvals_reviewer requested={requested_review_policy:?} active={active_review_policy:?}"
            ));
        }
    }
    if let Some(requested_sandbox) = request.sandbox.as_ref() {
        let sandbox_matches = matches!(
            (requested_sandbox, &config_snapshot.sandbox_policy),
            (
                SandboxMode::ReadOnly,
                codex_protocol::protocol::SandboxPolicy::ReadOnly { .. }
            ) | (
                SandboxMode::WorkspaceWrite,
                codex_protocol::protocol::SandboxPolicy::WorkspaceWrite { .. }
            ) | (
                SandboxMode::DangerFullAccess,
                codex_protocol::protocol::SandboxPolicy::DangerFullAccess
            ) | (
                SandboxMode::DangerFullAccess,
                codex_protocol::protocol::SandboxPolicy::ExternalSandbox { .. }
            )
        );
        if !sandbox_matches {
            mismatch_details.push(format!(
                "sandbox requested={requested_sandbox:?} active={:?}",
                config_snapshot.sandbox_policy
            ));
        }
    }
    if let Some(requested_permission_profile) = request.permission_profile.as_ref() {
        let requested_permission_profile =
            codex_protocol::models::PermissionProfile::from(requested_permission_profile.clone());
        if requested_permission_profile != config_snapshot.permission_profile {
            mismatch_details.push(format!(
                "permission_profile requested={requested_permission_profile:?} active={:?}",
                config_snapshot.permission_profile
            ));
        }
    }
    if let Some(requested_personality) = request.personality.as_ref()
        && config_snapshot.personality.as_ref() != Some(requested_personality)
    {
        mismatch_details.push(format!(
            "personality requested={requested_personality:?} active={:?}",
            config_snapshot.personality
        ));
    }

    if request.config.is_some() {
        mismatch_details
            .push("config overrides were provided and ignored while running".to_string());
    }
    if request.base_instructions.is_some() {
        mismatch_details
            .push("baseInstructions override was provided and ignored while running".to_string());
    }
    if request.developer_instructions.is_some() {
        mismatch_details.push(
            "developerInstructions override was provided and ignored while running".to_string(),
        );
    }
    if request.persist_extended_history {
        mismatch_details.push(
            "persistExtendedHistory override was provided and ignored while running".to_string(),
        );
    }

    mismatch_details
}

fn merge_persisted_resume_metadata(
    request_overrides: &mut Option<HashMap<String, serde_json::Value>>,
    typesafe_overrides: &mut ConfigOverrides,
    persisted_metadata: &ThreadMetadata,
) {
    if has_model_resume_override(request_overrides.as_ref(), typesafe_overrides) {
        return;
    }

    typesafe_overrides.model = persisted_metadata.model.clone();

    if let Some(reasoning_effort) = persisted_metadata.reasoning_effort {
        request_overrides.get_or_insert_with(HashMap::new).insert(
            "model_reasoning_effort".to_string(),
            serde_json::Value::String(reasoning_effort.to_string()),
        );
    }
}

fn has_model_resume_override(
    request_overrides: Option<&HashMap<String, serde_json::Value>>,
    typesafe_overrides: &ConfigOverrides,
) -> bool {
    typesafe_overrides.model.is_some()
        || typesafe_overrides.model_provider.is_some()
        || request_overrides.is_some_and(|overrides| overrides.contains_key("model"))
        || request_overrides
            .is_some_and(|overrides| overrides.contains_key("model_reasoning_effort"))
}

fn cloud_requirements_load_error(err: &std::io::Error) -> Option<&CloudRequirementsLoadError> {
    let mut current: Option<&(dyn std::error::Error + 'static)> = err
        .get_ref()
        .map(|source| source as &(dyn std::error::Error + 'static));
    while let Some(source) = current {
        if let Some(cloud_error) = source.downcast_ref::<CloudRequirementsLoadError>() {
            return Some(cloud_error);
        }
        current = source.source();
    }
    None
}

fn config_load_error(err: &std::io::Error) -> JSONRPCErrorError {
    let data = cloud_requirements_load_error(err).map(|cloud_error| {
        let mut data = serde_json::json!({
            "reason": "cloudRequirements",
            "errorCode": format!("{:?}", cloud_error.code()),
            "detail": cloud_error.to_string(),
        });
        if let Some(status_code) = cloud_error.status_code() {
            data["statusCode"] = serde_json::json!(status_code);
        }
        if cloud_error.code() == CloudRequirementsLoadErrorCode::Auth {
            data["action"] = serde_json::json!("relogin");
        }
        data
    });

    JSONRPCErrorError {
        code: INVALID_REQUEST_ERROR_CODE,
        message: format!("failed to load configuration: {err}"),
        data,
    }
}

fn validate_dynamic_tools(tools: &[ApiDynamicToolSpec]) -> Result<(), String> {
    let mut seen = HashSet::new();
    for tool in tools {
        let name = tool.name.trim();
        if name.is_empty() {
            return Err("dynamic tool name must not be empty".to_string());
        }
        if name != tool.name {
            return Err(format!(
                "dynamic tool name has leading/trailing whitespace: {}",
                tool.name
            ));
        }
        if name == "mcp" || name.starts_with("mcp__") {
            return Err(format!("dynamic tool name is reserved: {name}"));
        }
        let namespace = tool.namespace.as_deref().map(str::trim);
        if let Some(namespace) = namespace {
            if namespace.is_empty() {
                return Err(format!(
                    "dynamic tool namespace must not be empty for {name}"
                ));
            }
            if Some(namespace) != tool.namespace.as_deref() {
                return Err(format!(
                    "dynamic tool namespace has leading/trailing whitespace for {name}: {namespace}",
                ));
            }
            if namespace == "mcp" || namespace.starts_with("mcp__") {
                return Err(format!(
                    "dynamic tool namespace is reserved for {name}: {namespace}"
                ));
            }
        }
        if !seen.insert((namespace, name)) {
            if let Some(namespace) = namespace {
                return Err(format!(
                    "duplicate dynamic tool name in namespace {namespace}: {name}"
                ));
            }
            return Err(format!("duplicate dynamic tool name: {name}"));
        }
        if tool.defer_loading && namespace.is_none() {
            return Err(format!(
                "deferred dynamic tool must include a namespace: {name}"
            ));
        }

        if let Err(err) = codex_tools::parse_tool_input_schema(&tool.input_schema) {
            return Err(format!(
                "dynamic tool input schema is not supported for {name}: {err}"
            ));
        }
    }
    Ok(())
}

async fn read_history_cwd_from_state_db(
    config: &Config,
    thread_id: Option<ThreadId>,
    rollout_path: &Path,
) -> Option<PathBuf> {
    if let Some(state_db_ctx) = get_state_db(config).await
        && let Some(thread_id) = thread_id
        && let Ok(Some(metadata)) = state_db_ctx.get_thread(thread_id).await
    {
        return Some(metadata.cwd);
    }

    match read_session_meta_line(rollout_path).await {
        Ok(meta_line) => Some(meta_line.meta.cwd),
        Err(err) => {
            let rollout_path = rollout_path.display();
            warn!("failed to read session metadata from rollout {rollout_path}: {err}");
            None
        }
    }
}

async fn read_summary_from_state_db_by_thread_id(
    config: &Config,
    thread_id: ThreadId,
) -> Option<ConversationSummary> {
    let state_db_ctx = open_state_db_for_direct_thread_lookup(config).await;
    read_summary_from_state_db_context_by_thread_id(state_db_ctx.as_ref(), thread_id).await
}

async fn read_summary_from_state_db_context_by_thread_id(
    state_db_ctx: Option<&StateDbHandle>,
    thread_id: ThreadId,
) -> Option<ConversationSummary> {
    let state_db_ctx = state_db_ctx?;

    let metadata = match state_db_ctx.get_thread(thread_id).await {
        Ok(Some(metadata)) => metadata,
        Ok(None) | Err(_) => return None,
    };
    Some(summary_from_thread_metadata(&metadata))
}

async fn title_from_state_db(config: &Config, thread_id: ThreadId) -> Option<String> {
    if let Some(state_db_ctx) = open_state_db_for_direct_thread_lookup(config).await
        && let Some(metadata) = state_db_ctx.get_thread(thread_id).await.ok().flatten()
        && let Some(title) = distinct_title(&metadata)
    {
        return Some(title);
    }
    find_thread_name_by_id(&config.codex_home, &thread_id)
        .await
        .ok()
        .flatten()
}

async fn thread_titles_by_ids(
    config: &Config,
    thread_ids: &HashSet<ThreadId>,
) -> HashMap<ThreadId, String> {
    let mut names = HashMap::with_capacity(thread_ids.len());
    if let Some(state_db_ctx) = open_state_db_for_direct_thread_lookup(config).await {
        for &thread_id in thread_ids {
            let Ok(Some(metadata)) = state_db_ctx.get_thread(thread_id).await else {
                continue;
            };
            if let Some(title) = distinct_title(&metadata) {
                names.insert(thread_id, title);
            }
        }
    }
    if names.len() < thread_ids.len()
        && let Ok(legacy_names) = find_thread_names_by_ids(&config.codex_home, thread_ids).await
    {
        for (thread_id, title) in legacy_names {
            names.entry(thread_id).or_insert(title);
        }
    }
    names
}

async fn open_state_db_for_direct_thread_lookup(config: &Config) -> Option<StateDbHandle> {
    StateRuntime::init(config.sqlite_home.clone(), config.model_provider_id.clone())
        .await
        .ok()
}

fn non_empty_title(metadata: &ThreadMetadata) -> Option<String> {
    let title = metadata.title.trim();
    (!title.is_empty()).then(|| title.to_string())
}

fn distinct_title(metadata: &ThreadMetadata) -> Option<String> {
    let title = non_empty_title(metadata)?;
    if metadata.first_user_message.as_deref().map(str::trim) == Some(title.as_str()) {
        None
    } else {
        Some(title)
    }
}

fn set_thread_name_from_title(thread: &mut Thread, title: String) {
    if title.trim().is_empty() || thread.preview.trim() == title.trim() {
        return;
    }
    thread.name = Some(title);
}

fn thread_store_list_error(err: ThreadStoreError) -> JSONRPCErrorError {
    match err {
        ThreadStoreError::InvalidRequest { message } => JSONRPCErrorError {
            code: INVALID_REQUEST_ERROR_CODE,
            message,
            data: None,
        },
        err => JSONRPCErrorError {
            code: INTERNAL_ERROR_CODE,
            message: format!("failed to list threads: {err}"),
            data: None,
        },
    }
}

fn conversation_summary_thread_id_read_error(
    conversation_id: ThreadId,
    err: ThreadStoreError,
) -> JSONRPCErrorError {
    let no_rollout_message = format!("no rollout found for thread id {conversation_id}");
    match err {
        ThreadStoreError::InvalidRequest { message } if message == no_rollout_message => {
            conversation_summary_not_found_error(conversation_id)
        }
        ThreadStoreError::ThreadNotFound { thread_id } if thread_id == conversation_id => {
            conversation_summary_not_found_error(conversation_id)
        }
        ThreadStoreError::InvalidRequest { message } => JSONRPCErrorError {
            code: INVALID_REQUEST_ERROR_CODE,
            message,
            data: None,
        },
        err => JSONRPCErrorError {
            code: INTERNAL_ERROR_CODE,
            message: format!("failed to load conversation summary for {conversation_id}: {err}"),
            data: None,
        },
    }
}

fn conversation_summary_not_found_error(conversation_id: ThreadId) -> JSONRPCErrorError {
    JSONRPCErrorError {
        code: INVALID_REQUEST_ERROR_CODE,
        message: format!("no rollout found for conversation id {conversation_id}"),
        data: None,
    }
}

fn conversation_summary_rollout_path_read_error(
    path: &Path,
    err: ThreadStoreError,
) -> JSONRPCErrorError {
    match err {
        ThreadStoreError::InvalidRequest { message } => JSONRPCErrorError {
            code: INVALID_REQUEST_ERROR_CODE,
            message,
            data: None,
        },
        err => JSONRPCErrorError {
            code: INTERNAL_ERROR_CODE,
            message: format!(
                "failed to load conversation summary from {}: {}",
                path.display(),
                err
            ),
            data: None,
        },
    }
}

fn thread_store_write_error(operation: &str, err: ThreadStoreError) -> JSONRPCErrorError {
    match err {
        ThreadStoreError::ThreadNotFound { thread_id } => JSONRPCErrorError {
            code: INVALID_REQUEST_ERROR_CODE,
            message: format!("thread not found: {thread_id}"),
            data: None,
        },
        ThreadStoreError::InvalidRequest { message } => JSONRPCErrorError {
            code: INVALID_REQUEST_ERROR_CODE,
            message,
            data: None,
        },
        err => JSONRPCErrorError {
            code: INTERNAL_ERROR_CODE,
            message: format!("failed to {operation}: {err}"),
            data: None,
        },
    }
}

fn thread_from_stored_thread(
    thread: StoredThread,
    fallback_provider: &str,
    fallback_cwd: &AbsolutePathBuf,
) -> (Thread, Option<codex_thread_store::StoredThreadHistory>) {
    let path = thread.rollout_path;
    let git_info = thread.git_info.map(|info| ApiGitInfo {
        sha: info.commit_hash.map(|sha| sha.0),
        branch: info.branch,
        origin_url: info.repository_url,
    });
    let cwd = AbsolutePathBuf::relative_to_current_dir(path_utils::normalize_for_native_workdir(
        thread.cwd,
    ))
    .unwrap_or_else(|err| {
        warn!("failed to normalize thread cwd while reading stored thread: {err}");
        fallback_cwd.clone()
    });
    let source = with_thread_spawn_agent_metadata(
        thread.source,
        thread.agent_nickname.clone(),
        thread.agent_role.clone(),
    );
    let history = thread.history;
    let thread = Thread {
        id: thread.thread_id.to_string(),
        forked_from_id: thread.forked_from_id.map(|id| id.to_string()),
        preview: thread.first_user_message.unwrap_or(thread.preview),
        ephemeral: false,
        model_provider: if thread.model_provider.is_empty() {
            fallback_provider.to_string()
        } else {
            thread.model_provider
        },
        created_at: thread.created_at.timestamp(),
        updated_at: thread.updated_at.timestamp(),
        status: ThreadStatus::NotLoaded,
        path,
        cwd,
        cli_version: thread.cli_version,
        agent_nickname: source.get_nickname(),
        agent_role: source.get_agent_role(),
        source: source.into(),
        git_info,
        name: thread.name,
        epiphany_state: None,
        turns: Vec::new(),
    };
    (thread, history)
}

fn thread_store_archive_error(operation: &str, err: ThreadStoreError) -> JSONRPCErrorError {
    match err {
        ThreadStoreError::InvalidRequest { message } => JSONRPCErrorError {
            code: INVALID_REQUEST_ERROR_CODE,
            message,
            data: None,
        },
        err => JSONRPCErrorError {
            code: INTERNAL_ERROR_CODE,
            message: format!("failed to {operation} thread: {err}"),
            data: None,
        },
    }
}

fn summary_from_stored_thread(
    thread: StoredThread,
    fallback_provider: &str,
) -> Option<ConversationSummary> {
    let path = thread.rollout_path?;
    let source = with_thread_spawn_agent_metadata(
        thread.source,
        thread.agent_nickname.clone(),
        thread.agent_role.clone(),
    );
    let git_info = thread.git_info.map(|git| ConversationGitInfo {
        sha: git.commit_hash.map(|sha| sha.0),
        branch: git.branch,
        origin_url: git.repository_url,
    });
    Some(ConversationSummary {
        conversation_id: thread.thread_id,
        path,
        preview: thread.first_user_message.unwrap_or(thread.preview),
        // Preserve millisecond precision from the thread store so thread/list cursors
        // round-trip the same ordering key used by pagination queries.
        timestamp: Some(
            thread
                .created_at
                .to_rfc3339_opts(SecondsFormat::Millis, true),
        ),
        updated_at: Some(
            thread
                .updated_at
                .to_rfc3339_opts(SecondsFormat::Millis, true),
        ),
        model_provider: if thread.model_provider.is_empty() {
            fallback_provider.to_string()
        } else {
            thread.model_provider
        },
        cwd: thread.cwd,
        cli_version: thread.cli_version,
        source,
        git_info,
    })
}

#[allow(clippy::too_many_arguments)]
fn summary_from_state_db_metadata(
    conversation_id: ThreadId,
    path: PathBuf,
    first_user_message: Option<String>,
    timestamp: String,
    updated_at: String,
    model_provider: String,
    cwd: PathBuf,
    cli_version: String,
    source: String,
    agent_nickname: Option<String>,
    agent_role: Option<String>,
    git_sha: Option<String>,
    git_branch: Option<String>,
    git_origin_url: Option<String>,
) -> ConversationSummary {
    let preview = first_user_message.unwrap_or_default();
    let source = serde_json::from_str(&source)
        .or_else(|_| serde_json::from_value(serde_json::Value::String(source.clone())))
        .unwrap_or(codex_protocol::protocol::SessionSource::Unknown);
    let source = with_thread_spawn_agent_metadata(source, agent_nickname, agent_role);
    let git_info = if git_sha.is_none() && git_branch.is_none() && git_origin_url.is_none() {
        None
    } else {
        Some(ConversationGitInfo {
            sha: git_sha,
            branch: git_branch,
            origin_url: git_origin_url,
        })
    };
    ConversationSummary {
        conversation_id,
        path,
        preview,
        timestamp: Some(timestamp),
        updated_at: Some(updated_at),
        model_provider,
        cwd,
        cli_version,
        source,
        git_info,
    }
}

fn summary_from_thread_metadata(metadata: &ThreadMetadata) -> ConversationSummary {
    summary_from_state_db_metadata(
        metadata.id,
        metadata.rollout_path.clone(),
        metadata.first_user_message.clone(),
        metadata
            .created_at
            .to_rfc3339_opts(SecondsFormat::Secs, true),
        metadata
            .updated_at
            .to_rfc3339_opts(SecondsFormat::Secs, true),
        metadata.model_provider.clone(),
        metadata.cwd.clone(),
        metadata.cli_version.clone(),
        metadata.source.clone(),
        metadata.agent_nickname.clone(),
        metadata.agent_role.clone(),
        metadata.git_sha.clone(),
        metadata.git_branch.clone(),
        metadata.git_origin_url.clone(),
    )
}

pub(crate) async fn read_summary_from_rollout(
    path: &Path,
    fallback_provider: &str,
) -> std::io::Result<ConversationSummary> {
    let head = read_head_for_summary(path).await?;

    let Some(first) = head.first() else {
        return Err(IoError::other(format!(
            "rollout at {} is empty",
            path.display()
        )));
    };

    let session_meta_line =
        serde_json::from_value::<SessionMetaLine>(first.clone()).map_err(|_| {
            IoError::other(format!(
                "rollout at {} does not start with session metadata",
                path.display()
            ))
        })?;
    let SessionMetaLine {
        meta: session_meta,
        git,
    } = session_meta_line;
    let mut session_meta = session_meta;
    session_meta.source = with_thread_spawn_agent_metadata(
        session_meta.source.clone(),
        session_meta.agent_nickname.clone(),
        session_meta.agent_role.clone(),
    );

    let created_at = if session_meta.timestamp.is_empty() {
        None
    } else {
        Some(session_meta.timestamp.as_str())
    };
    let updated_at = read_updated_at(path, created_at).await;
    if let Some(summary) = extract_conversation_summary(
        path.to_path_buf(),
        &head,
        &session_meta,
        git.as_ref(),
        fallback_provider,
        updated_at.clone(),
    ) {
        return Ok(summary);
    }

    let timestamp = if session_meta.timestamp.is_empty() {
        None
    } else {
        Some(session_meta.timestamp.clone())
    };
    let model_provider = session_meta
        .model_provider
        .clone()
        .unwrap_or_else(|| fallback_provider.to_string());
    let git_info = git.as_ref().map(map_git_info);
    let updated_at = updated_at.or_else(|| timestamp.clone());

    Ok(ConversationSummary {
        conversation_id: session_meta.id,
        timestamp,
        updated_at,
        path: path.to_path_buf(),
        preview: String::new(),
        model_provider,
        cwd: session_meta.cwd,
        cli_version: session_meta.cli_version,
        source: session_meta.source,
        git_info,
    })
}

pub(crate) async fn read_rollout_items_from_rollout(
    path: &Path,
) -> std::io::Result<Vec<RolloutItem>> {
    let items = match RolloutRecorder::get_rollout_history(path).await? {
        InitialHistory::New | InitialHistory::Cleared => Vec::new(),
        InitialHistory::Forked(items) => items,
        InitialHistory::Resumed(resumed) => resumed.history,
    };

    Ok(items)
}

fn extract_conversation_summary(
    path: PathBuf,
    head: &[serde_json::Value],
    session_meta: &SessionMeta,
    git: Option<&CoreGitInfo>,
    fallback_provider: &str,
    updated_at: Option<String>,
) -> Option<ConversationSummary> {
    let preview = head
        .iter()
        .filter_map(|value| serde_json::from_value::<ResponseItem>(value.clone()).ok())
        .find_map(|item| match codex_core::parse_turn_item(&item) {
            Some(TurnItem::UserMessage(user)) => Some(user.message()),
            _ => None,
        })?;

    let preview = match preview.find(USER_MESSAGE_BEGIN) {
        Some(idx) => preview[idx + USER_MESSAGE_BEGIN.len()..].trim(),
        None => preview.as_str(),
    };

    let timestamp = if session_meta.timestamp.is_empty() {
        None
    } else {
        Some(session_meta.timestamp.clone())
    };
    let conversation_id = session_meta.id;
    let model_provider = session_meta
        .model_provider
        .clone()
        .unwrap_or_else(|| fallback_provider.to_string());
    let git_info = git.map(map_git_info);
    let updated_at = updated_at.or_else(|| timestamp.clone());

    Some(ConversationSummary {
        conversation_id,
        timestamp,
        updated_at,
        path,
        preview: preview.to_string(),
        model_provider,
        cwd: session_meta.cwd.clone(),
        cli_version: session_meta.cli_version.clone(),
        source: session_meta.source.clone(),
        git_info,
    })
}

fn map_git_info(git_info: &CoreGitInfo) -> ConversationGitInfo {
    ConversationGitInfo {
        sha: git_info.commit_hash.as_ref().map(|sha| sha.0.clone()),
        branch: git_info.branch.clone(),
        origin_url: git_info.repository_url.clone(),
    }
}

async fn load_thread_summary_for_rollout(
    config: &Config,
    thread_id: ThreadId,
    rollout_path: &Path,
    fallback_provider: &str,
    persisted_metadata: Option<&ThreadMetadata>,
) -> std::result::Result<Thread, String> {
    let mut thread = read_summary_from_rollout(rollout_path, fallback_provider)
        .await
        .map(|summary| summary_to_thread(summary, &config.cwd))
        .map_err(|err| {
            format!(
                "failed to load rollout `{}` for thread {thread_id}: {err}",
                rollout_path.display()
            )
        })?;
    thread.forked_from_id = forked_from_id_from_rollout(rollout_path).await;
    if let Some(persisted_metadata) = persisted_metadata {
        merge_mutable_thread_metadata(
            &mut thread,
            summary_to_thread(
                summary_from_thread_metadata(persisted_metadata),
                &config.cwd,
            ),
        );
    } else if let Some(summary) = read_summary_from_state_db_by_thread_id(config, thread_id).await {
        merge_mutable_thread_metadata(&mut thread, summary_to_thread(summary, &config.cwd));
    }
    let title = if let Some(metadata) = persisted_metadata {
        non_empty_title(metadata)
    } else {
        title_from_state_db(config, thread_id).await
    };
    if let Some(title) = title {
        set_thread_name_from_title(&mut thread, title);
    }
    Ok(thread)
}

async fn forked_from_id_from_rollout(path: &Path) -> Option<String> {
    read_session_meta_line(path)
        .await
        .ok()
        .and_then(|meta_line| meta_line.meta.forked_from_id)
        .map(|thread_id| thread_id.to_string())
}

fn merge_mutable_thread_metadata(thread: &mut Thread, persisted_thread: Thread) {
    thread.git_info = persisted_thread.git_info;
}

fn preview_from_rollout_items(items: &[RolloutItem]) -> String {
    items
        .iter()
        .find_map(|item| match item {
            RolloutItem::ResponseItem(item) => match codex_core::parse_turn_item(item) {
                Some(codex_protocol::items::TurnItem::UserMessage(user)) => Some(user.message()),
                _ => None,
            },
            _ => None,
        })
        .map(|preview| match preview.find(USER_MESSAGE_BEGIN) {
            Some(idx) => preview[idx + USER_MESSAGE_BEGIN.len()..].trim().to_string(),
            None => preview,
        })
        .unwrap_or_default()
}

fn with_thread_spawn_agent_metadata(
    source: codex_protocol::protocol::SessionSource,
    agent_nickname: Option<String>,
    agent_role: Option<String>,
) -> codex_protocol::protocol::SessionSource {
    if agent_nickname.is_none() && agent_role.is_none() {
        return source;
    }

    match source {
        codex_protocol::protocol::SessionSource::SubAgent(
            codex_protocol::protocol::SubAgentSource::ThreadSpawn {
                parent_thread_id,
                depth,
                agent_path,
                agent_nickname: existing_agent_nickname,
                agent_role: existing_agent_role,
            },
        ) => codex_protocol::protocol::SessionSource::SubAgent(
            codex_protocol::protocol::SubAgentSource::ThreadSpawn {
                parent_thread_id,
                depth,
                agent_path,
                agent_nickname: agent_nickname.or(existing_agent_nickname),
                agent_role: agent_role.or(existing_agent_role),
            },
        ),
        _ => source,
    }
}

fn thread_response_permission_profile(
    sandbox_policy: &codex_protocol::protocol::SandboxPolicy,
    permission_profile: codex_protocol::models::PermissionProfile,
) -> Option<codex_app_server_protocol::PermissionProfile> {
    match sandbox_policy {
        codex_protocol::protocol::SandboxPolicy::DangerFullAccess
        | codex_protocol::protocol::SandboxPolicy::ReadOnly { .. }
        | codex_protocol::protocol::SandboxPolicy::WorkspaceWrite { .. } => {
            Some(permission_profile.into())
        }
        codex_protocol::protocol::SandboxPolicy::ExternalSandbox { .. } => None,
    }
}

fn requested_permissions_trust_project(overrides: &ConfigOverrides, cwd: &Path) -> bool {
    if matches!(
        overrides.sandbox_mode,
        Some(
            codex_protocol::config_types::SandboxMode::WorkspaceWrite
                | codex_protocol::config_types::SandboxMode::DangerFullAccess
        )
    ) {
        return true;
    }

    overrides
        .permission_profile
        .as_ref()
        .is_some_and(|profile| {
            profile
                .to_legacy_sandbox_policy(cwd)
                .is_ok_and(|sandbox_policy| {
                    matches!(
                        sandbox_policy,
                        codex_protocol::protocol::SandboxPolicy::WorkspaceWrite { .. }
                            | codex_protocol::protocol::SandboxPolicy::DangerFullAccess
                            | codex_protocol::protocol::SandboxPolicy::ExternalSandbox { .. }
                    )
                })
        })
}

fn parse_datetime(timestamp: Option<&str>) -> Option<DateTime<Utc>> {
    timestamp.and_then(|ts| {
        chrono::DateTime::parse_from_rfc3339(ts)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc))
    })
}

async fn read_updated_at(path: &Path, created_at: Option<&str>) -> Option<String> {
    let updated_at = tokio::fs::metadata(path)
        .await
        .ok()
        .and_then(|meta| meta.modified().ok())
        .map(|modified| {
            let updated_at: DateTime<Utc> = modified.into();
            updated_at.to_rfc3339_opts(SecondsFormat::Millis, true)
        });
    updated_at.or_else(|| created_at.map(str::to_string))
}

fn build_thread_from_snapshot(
    thread_id: ThreadId,
    config_snapshot: &ThreadConfigSnapshot,
    path: Option<PathBuf>,
) -> Thread {
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    Thread {
        id: thread_id.to_string(),
        forked_from_id: None,
        preview: String::new(),
        ephemeral: config_snapshot.ephemeral,
        model_provider: config_snapshot.model_provider_id.clone(),
        created_at: now,
        updated_at: now,
        status: ThreadStatus::NotLoaded,
        path,
        cwd: config_snapshot.cwd.clone(),
        cli_version: env!("CARGO_PKG_VERSION").to_string(),
        agent_nickname: config_snapshot.session_source.get_nickname(),
        agent_role: config_snapshot.session_source.get_agent_role(),
        source: config_snapshot.session_source.clone().into(),
        git_info: None,
        name: None,
        epiphany_state: None,
        turns: Vec::new(),
    }
}

pub(crate) fn summary_to_thread(
    summary: ConversationSummary,
    fallback_cwd: &AbsolutePathBuf,
) -> Thread {
    let ConversationSummary {
        conversation_id,
        path,
        preview,
        timestamp,
        updated_at,
        model_provider,
        cwd,
        cli_version,
        source,
        git_info,
    } = summary;

    let created_at = parse_datetime(timestamp.as_deref());
    let updated_at = parse_datetime(updated_at.as_deref()).or(created_at);
    let git_info = git_info.map(|info| ApiGitInfo {
        sha: info.sha,
        branch: info.branch,
        origin_url: info.origin_url,
    });
    let cwd =
        AbsolutePathBuf::relative_to_current_dir(path_utils::normalize_for_native_workdir(cwd))
            .unwrap_or_else(|err| {
                warn!(
                    path = %path.display(),
                    "failed to normalize thread cwd while summarizing thread: {err}"
                );
                fallback_cwd.clone()
            });

    Thread {
        id: conversation_id.to_string(),
        forked_from_id: None,
        preview,
        ephemeral: false,
        model_provider,
        created_at: created_at.map(|dt| dt.timestamp()).unwrap_or(0),
        updated_at: updated_at.map(|dt| dt.timestamp()).unwrap_or(0),
        status: ThreadStatus::NotLoaded,
        path: Some(path),
        cwd,
        cli_version,
        agent_nickname: source.get_nickname(),
        agent_role: source.get_agent_role(),
        source: source.into(),
        git_info,
        name: None,
        epiphany_state: None,
        turns: Vec::new(),
    }
}

fn thread_backwards_cursor_for_sort_key(
    summary: &ConversationSummary,
    sort_key: StoreThreadSortKey,
    sort_direction: SortDirection,
) -> Option<String> {
    let timestamp = match sort_key {
        StoreThreadSortKey::CreatedAt => summary.timestamp.as_deref(),
        StoreThreadSortKey::UpdatedAt => summary
            .updated_at
            .as_deref()
            .or(summary.timestamp.as_deref()),
    };
    let timestamp = parse_datetime(timestamp)?;
    // The state DB stores unique millisecond timestamps. Offset the reverse cursor by one
    // millisecond so the opposite-direction query includes the page anchor.
    let timestamp = match sort_direction {
        SortDirection::Asc => timestamp.checked_add_signed(ChronoDuration::milliseconds(1))?,
        SortDirection::Desc => timestamp.checked_sub_signed(ChronoDuration::milliseconds(1))?,
    };
    Some(timestamp.to_rfc3339_opts(SecondsFormat::Millis, true))
}

struct ThreadTurnsPage {
    turns: Vec<Turn>,
    next_cursor: Option<String>,
    backwards_cursor: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThreadTurnsCursor {
    turn_id: String,
    include_anchor: bool,
}

fn paginate_thread_turns(
    turns: Vec<Turn>,
    cursor: Option<&str>,
    limit: Option<u32>,
    sort_direction: SortDirection,
) -> Result<ThreadTurnsPage, JSONRPCErrorError> {
    if turns.is_empty() {
        return Ok(ThreadTurnsPage {
            turns: Vec::new(),
            next_cursor: None,
            backwards_cursor: None,
        });
    }

    let anchor = cursor.map(parse_thread_turns_cursor).transpose()?;
    let page_size = limit
        .map(|value| value as usize)
        .unwrap_or(THREAD_TURNS_DEFAULT_LIMIT)
        .clamp(1, THREAD_TURNS_MAX_LIMIT);

    let anchor_index = anchor
        .as_ref()
        .and_then(|anchor| turns.iter().position(|turn| turn.id == anchor.turn_id));
    if anchor.is_some() && anchor_index.is_none() {
        return Err(JSONRPCErrorError {
            code: INVALID_REQUEST_ERROR_CODE,
            message: "invalid cursor: anchor turn is no longer present".to_string(),
            data: None,
        });
    }

    let mut keyed_turns: Vec<_> = turns.into_iter().enumerate().collect();
    match sort_direction {
        SortDirection::Asc => {
            if let (Some(anchor), Some(anchor_index)) = (anchor.as_ref(), anchor_index) {
                keyed_turns.retain(|(index, _)| {
                    if anchor.include_anchor {
                        *index >= anchor_index
                    } else {
                        *index > anchor_index
                    }
                });
            }
        }
        SortDirection::Desc => {
            keyed_turns.reverse();
            if let (Some(anchor), Some(anchor_index)) = (anchor.as_ref(), anchor_index) {
                keyed_turns.retain(|(index, _)| {
                    if anchor.include_anchor {
                        *index <= anchor_index
                    } else {
                        *index < anchor_index
                    }
                });
            }
        }
    }

    let more_turns_available = keyed_turns.len() > page_size;
    keyed_turns.truncate(page_size);
    let backwards_cursor = keyed_turns
        .first()
        .map(|(_, turn)| serialize_thread_turns_cursor(&turn.id, /*include_anchor*/ true))
        .transpose()?;
    let next_cursor = if more_turns_available {
        keyed_turns
            .last()
            .map(|(_, turn)| serialize_thread_turns_cursor(&turn.id, /*include_anchor*/ false))
            .transpose()?
    } else {
        None
    };
    let turns = keyed_turns.into_iter().map(|(_, turn)| turn).collect();

    Ok(ThreadTurnsPage {
        turns,
        next_cursor,
        backwards_cursor,
    })
}

fn serialize_thread_turns_cursor(
    turn_id: &str,
    include_anchor: bool,
) -> Result<String, JSONRPCErrorError> {
    serde_json::to_string(&ThreadTurnsCursor {
        turn_id: turn_id.to_string(),
        include_anchor,
    })
    .map_err(|err| JSONRPCErrorError {
        code: INTERNAL_ERROR_CODE,
        message: format!("failed to serialize cursor: {err}"),
        data: None,
    })
}

fn parse_thread_turns_cursor(cursor: &str) -> Result<ThreadTurnsCursor, JSONRPCErrorError> {
    serde_json::from_str(cursor).map_err(|_| JSONRPCErrorError {
        code: INVALID_REQUEST_ERROR_CODE,
        message: format!("invalid cursor: {cursor}"),
        data: None,
    })
}

fn reconstruct_thread_turns_from_rollout_items(
    items: &[RolloutItem],
    loaded_status: ThreadStatus,
    has_live_in_progress_turn: bool,
) -> Vec<Turn> {
    let mut turns = build_turns_from_rollout_items(items);
    normalize_thread_turns_status(&mut turns, loaded_status, has_live_in_progress_turn);
    turns
}

fn normalize_thread_turns_status(
    turns: &mut [Turn],
    loaded_status: ThreadStatus,
    has_live_in_progress_turn: bool,
) {
    let status = resolve_thread_status(loaded_status, has_live_in_progress_turn);
    if matches!(status, ThreadStatus::Active { .. }) {
        return;
    }
    for turn in turns {
        if matches!(turn.status, TurnStatus::InProgress) {
            turn.status = TurnStatus::Interrupted;
        }
    }
}
