mod auth_routes;
mod catalog_routes;
mod command_routes;
mod epiphany_mutation_routes;
mod epiphany_read_routes;
mod epiphany_thread_host;
#[cfg(test)]
mod epiphany_compatibility_architecture;
mod listener_lifecycle;
mod legacy_epiphany_rollout;
mod mcp_routes;
mod realtime_routes;
mod review_routes;
mod running_thread_resume_routes;
mod thread_admin_routes;
mod thread_archive_routes;
mod thread_config;
mod thread_fork_routes;
mod thread_metadata_routes;
mod thread_projection;
mod thread_read_routes;
mod thread_resume_routes;
mod thread_start_routes;
mod thread_turn_projection;
mod thread_turn_routes;
mod turn_routes;

use self::auth_routes::ActiveLogin;
use self::epiphany_thread_host::live_thread_epiphany_state;
use self::epiphany_thread_host::load_epiphany_state_from_rollout_path;
pub(super) use self::listener_lifecycle::{EnsureConversationListenerResult, ListenerTaskContext};
pub(crate) use self::running_thread_resume_routes::handle_pending_thread_resume_request;
pub(crate) use self::thread_config::{config_load_error, validate_dynamic_tools};
pub(crate) use self::thread_projection::*;
pub(super) use self::thread_read_routes::ThreadReadViewError;
pub(crate) use self::thread_turn_projection::*;
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
use codex_thread_store::ReadThreadByRolloutPathParams as StoreReadThreadByRolloutPathParams;
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

    async fn send_invalid_request_error(&self, request_id: ConnectionRequestId, message: String) {
        let error = JSONRPCErrorError {
            code: INVALID_REQUEST_ERROR_CODE,
            message,
            data: None,
        };
        self.outgoing.send_error(request_id, error).await;
    }

    async fn send_internal_error(&self, request_id: ConnectionRequestId, message: String) {
        let error = JSONRPCErrorError {
            code: INTERNAL_ERROR_CODE,
            message,
            data: None,
        };
        self.outgoing.send_error(request_id, error).await;
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
