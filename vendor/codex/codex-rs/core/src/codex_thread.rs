use crate::agent::AgentStatus;
use crate::config::ConstraintResult;
use crate::epiphany_promotion::EpiphanyStateReplacementValidationInput;
use crate::epiphany_promotion::validate_state_replacement_patch;
use crate::epiphany_retrieval;
use crate::epiphany_retrieval::EpiphanyRetrieveQuery;
use crate::epiphany_retrieval::EpiphanyRetrieveResponse;
use crate::file_watcher::WatchRegistration;
use crate::session::Codex;
use crate::session::SessionSettingsUpdate;
use crate::session::SteerInputError;
use codex_features::Feature;
use codex_protocol::config_types::ApprovalsReviewer;
use codex_protocol::config_types::CollaborationMode;
use codex_protocol::config_types::Personality;
use codex_protocol::config_types::ReasoningSummary;
use codex_protocol::config_types::ServiceTier;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::error::CodexErr;
use codex_protocol::error::Result as CodexResult;
use codex_protocol::mcp::CallToolResult;
use codex_protocol::models::ContentItem;
use codex_protocol::models::PermissionProfile;
use codex_protocol::models::ResponseInputItem;
use codex_protocol::models::ResponseItem;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::EpiphanyAcceptanceReceipt;
use codex_protocol::protocol::EpiphanyChurnState;
use codex_protocol::protocol::EpiphanyEvidenceRecord;
use codex_protocol::protocol::EpiphanyGraphCheckpoint;
use codex_protocol::protocol::EpiphanyGraphFrontier;
use codex_protocol::protocol::EpiphanyGraphs;
use codex_protocol::protocol::EpiphanyInvariant;
use codex_protocol::protocol::EpiphanyInvestigationCheckpoint;
use codex_protocol::protocol::EpiphanyJobBinding;
use codex_protocol::protocol::EpiphanyJobKind;
use codex_protocol::protocol::EpiphanyModeState;
use codex_protocol::protocol::EpiphanyObservation;
use codex_protocol::protocol::EpiphanyPlanningState;
use codex_protocol::protocol::EpiphanyRetrievalState;
use codex_protocol::protocol::EpiphanyRuntimeLink;
use codex_protocol::protocol::EpiphanyScratchPad;
use codex_protocol::protocol::EpiphanyStateItem;
use codex_protocol::protocol::EpiphanySubgoal;
use codex_protocol::protocol::EpiphanyThreadState;
use codex_protocol::protocol::Event;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::RolloutItem;
use codex_protocol::protocol::SandboxPolicy;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::Submission;
use codex_protocol::protocol::ThreadMemoryMode;
use codex_protocol::protocol::TokenUsage;
use codex_protocol::protocol::TokenUsageInfo;
use codex_protocol::protocol::W3cTraceContext;
use codex_protocol::user_input::UserInput;
use codex_utils_absolute_path::AbsolutePathBuf;
use epiphany_core::EpiphanyWorkerLaunchDocument;
use epiphany_core::RuntimeSpineInitOptions;
use epiphany_core::RuntimeSpineJobOptions;
use epiphany_core::RuntimeSpineSessionOptions;
use epiphany_core::create_runtime_job;
use epiphany_core::ensure_runtime_session;
use epiphany_core::initialize_runtime_spine;
use rmcp::model::ReadResourceRequestParams;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::sync::Mutex;
use tokio::sync::watch;

use codex_rollout::state_db::StateDbHandle;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ThreadConfigSnapshot {
    pub model: String,
    pub model_provider_id: String,
    pub service_tier: Option<ServiceTier>,
    pub approval_policy: AskForApproval,
    pub approvals_reviewer: ApprovalsReviewer,
    pub sandbox_policy: SandboxPolicy,
    pub permission_profile: PermissionProfile,
    pub cwd: AbsolutePathBuf,
    pub ephemeral: bool,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub personality: Option<Personality>,
    pub session_source: SessionSource,
}

/// Turn context overrides that app-server validates before starting a turn.
#[derive(Clone, Default)]
pub struct CodexThreadTurnContextOverrides {
    pub cwd: Option<PathBuf>,
    pub approval_policy: Option<AskForApproval>,
    pub approvals_reviewer: Option<ApprovalsReviewer>,
    pub sandbox_policy: Option<SandboxPolicy>,
    pub permission_profile: Option<PermissionProfile>,
    pub windows_sandbox_level: Option<WindowsSandboxLevel>,
    pub model: Option<String>,
    pub effort: Option<Option<ReasoningEffort>>,
    pub summary: Option<ReasoningSummary>,
    pub service_tier: Option<Option<ServiceTier>>,
    pub collaboration_mode: Option<CollaborationMode>,
    pub personality: Option<Personality>,
}

pub struct CodexThread {
    pub(crate) codex: Codex,
    rollout_path: Option<PathBuf>,
    out_of_band_elicitation_count: Mutex<u64>,
    _watch_registration: WatchRegistration,
}

#[derive(Debug, Clone, Default)]
pub struct EpiphanyStateUpdate {
    pub expected_revision: Option<u64>,
    pub objective: Option<String>,
    pub active_subgoal_id: Option<String>,
    pub subgoals: Option<Vec<EpiphanySubgoal>>,
    pub invariants: Option<Vec<EpiphanyInvariant>>,
    pub graphs: Option<EpiphanyGraphs>,
    pub graph_frontier: Option<EpiphanyGraphFrontier>,
    pub graph_checkpoint: Option<EpiphanyGraphCheckpoint>,
    pub scratch: Option<EpiphanyScratchPad>,
    pub investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
    pub job_bindings: Option<Vec<EpiphanyJobBinding>>,
    pub acceptance_receipts: Vec<EpiphanyAcceptanceReceipt>,
    pub runtime_links: Vec<EpiphanyRuntimeLink>,
    pub observations: Vec<EpiphanyObservation>,
    pub evidence: Vec<EpiphanyEvidenceRecord>,
    pub churn: Option<EpiphanyChurnState>,
    pub mode: Option<EpiphanyModeState>,
    pub planning: Option<EpiphanyPlanningState>,
}

#[derive(Debug, Clone)]
pub struct EpiphanyJobLaunchRequest {
    pub expected_revision: Option<u64>,
    pub binding_id: String,
    pub kind: EpiphanyJobKind,
    pub scope: String,
    pub owner_role: String,
    pub authority_scope: String,
    pub linked_subgoal_ids: Vec<String>,
    pub linked_graph_node_ids: Vec<String>,
    pub instruction: String,
    pub launch_document: EpiphanyWorkerLaunchDocument,
    pub output_contract_id: String,
    pub max_runtime_seconds: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct EpiphanyJobLaunchResult {
    pub epiphany_state: EpiphanyThreadState,
    pub binding_id: String,
    pub launcher_job_id: String,
    pub backend_job_id: String,
}

#[derive(Debug, Clone)]
pub struct EpiphanyJobInterruptRequest {
    pub expected_revision: Option<u64>,
    pub binding_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EpiphanyJobInterruptResult {
    pub epiphany_state: EpiphanyThreadState,
    pub binding_id: String,
    pub cancel_requested: bool,
    pub interrupted_thread_ids: Vec<String>,
}

impl EpiphanyStateUpdate {
    fn is_empty(&self) -> bool {
        self.objective.is_none()
            && self.active_subgoal_id.is_none()
            && self.subgoals.is_none()
            && self.invariants.is_none()
            && self.graphs.is_none()
            && self.graph_frontier.is_none()
            && self.graph_checkpoint.is_none()
            && self.scratch.is_none()
            && self.investigation_checkpoint.is_none()
            && self.job_bindings.is_none()
            && self.acceptance_receipts.is_empty()
            && self.runtime_links.is_empty()
            && self.observations.is_empty()
            && self.evidence.is_empty()
            && self.churn.is_none()
            && self.mode.is_none()
            && self.planning.is_none()
    }
}

/// Conduit for the bidirectional stream of messages that compose a thread
/// (formerly called a conversation) in Codex.
impl CodexThread {
    pub(crate) fn new(
        codex: Codex,
        rollout_path: Option<PathBuf>,
        watch_registration: WatchRegistration,
    ) -> Self {
        Self {
            codex,
            rollout_path,
            out_of_band_elicitation_count: Mutex::new(0),
            _watch_registration: watch_registration,
        }
    }

    pub async fn submit(&self, op: Op) -> CodexResult<String> {
        self.codex.submit(op).await
    }

    pub async fn shutdown_and_wait(&self) -> CodexResult<()> {
        self.codex.shutdown_and_wait().await
    }

    #[doc(hidden)]
    pub async fn ensure_rollout_materialized(&self) {
        self.codex.session.ensure_rollout_materialized().await;
    }

    #[doc(hidden)]
    pub async fn flush_rollout(&self) -> std::io::Result<()> {
        self.codex.session.flush_rollout().await
    }

    pub async fn submit_with_trace(
        &self,
        op: Op,
        trace: Option<W3cTraceContext>,
    ) -> CodexResult<String> {
        self.codex.submit_with_trace(op, trace).await
    }

    /// Persist whether this thread is eligible for future memory generation.
    pub async fn set_thread_memory_mode(&self, mode: ThreadMemoryMode) -> anyhow::Result<()> {
        self.codex.set_thread_memory_mode(mode).await
    }

    pub async fn steer_input(
        &self,
        input: Vec<UserInput>,
        expected_turn_id: Option<&str>,
        responsesapi_client_metadata: Option<HashMap<String, String>>,
    ) -> Result<String, SteerInputError> {
        self.codex
            .steer_input(input, expected_turn_id, responsesapi_client_metadata)
            .await
    }

    pub async fn set_app_server_client_info(
        &self,
        app_server_client_name: Option<String>,
        app_server_client_version: Option<String>,
    ) -> ConstraintResult<()> {
        self.codex
            .set_app_server_client_info(app_server_client_name, app_server_client_version)
            .await
    }

    /// Validate persistent turn context overrides without committing them.
    pub async fn validate_turn_context_overrides(
        &self,
        overrides: CodexThreadTurnContextOverrides,
    ) -> ConstraintResult<()> {
        let CodexThreadTurnContextOverrides {
            cwd,
            approval_policy,
            approvals_reviewer,
            sandbox_policy,
            permission_profile,
            windows_sandbox_level,
            model,
            effort,
            summary,
            service_tier,
            collaboration_mode,
            personality,
        } = overrides;
        let collaboration_mode = if let Some(collaboration_mode) = collaboration_mode {
            collaboration_mode
        } else {
            self.codex
                .session
                .collaboration_mode()
                .await
                .with_updates(model, effort, /*developer_instructions*/ None)
        };

        let updates = SessionSettingsUpdate {
            cwd,
            approval_policy,
            approvals_reviewer,
            sandbox_policy,
            permission_profile,
            windows_sandbox_level,
            collaboration_mode: Some(collaboration_mode),
            reasoning_summary: summary,
            service_tier,
            personality,
            ..Default::default()
        };
        self.codex.session.validate_settings(&updates).await
    }

    /// Use sparingly: this is intended to be removed soon.
    pub async fn submit_with_id(&self, sub: Submission) -> CodexResult<()> {
        self.codex.submit_with_id(sub).await
    }

    pub async fn next_event(&self) -> CodexResult<Event> {
        self.codex.next_event().await
    }

    pub async fn agent_status(&self) -> AgentStatus {
        self.codex.agent_status().await
    }

    pub(crate) fn subscribe_status(&self) -> watch::Receiver<AgentStatus> {
        self.codex.agent_status.clone()
    }

    pub(crate) async fn total_token_usage(&self) -> Option<TokenUsage> {
        self.codex.session.total_token_usage().await
    }

    /// Returns the complete token usage snapshot currently cached for this thread.
    ///
    /// This accessor is intentionally narrower than direct session access: it lets
    /// app-server lifecycle paths replay restored usage after resume or fork without
    /// exposing broader session mutation authority. A caller that only reads
    /// `total_token_usage` would drop last-turn usage and make the v2
    /// `thread/tokenUsage/updated` payload incomplete.
    pub async fn token_usage_info(&self) -> Option<TokenUsageInfo> {
        self.codex.session.token_usage_info().await
    }

    /// Records a user-role session-prefix message without creating a new user turn boundary.
    pub(crate) async fn inject_user_message_without_turn(&self, message: String) {
        let message = ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText { text: message }],
            end_turn: None,
            phase: None,
        };
        let pending_item = match pending_message_input_item(&message) {
            Ok(pending_item) => pending_item,
            Err(err) => {
                debug_assert!(false, "session-prefix message append should succeed: {err}");
                return;
            }
        };
        if self
            .codex
            .session
            .inject_response_items(vec![pending_item])
            .await
            .is_err()
        {
            let turn_context = self.codex.session.new_default_turn().await;
            self.codex
                .session
                .record_conversation_items(turn_context.as_ref(), &[message])
                .await;
        }
    }

    /// Append a prebuilt message to the thread history without treating it as a user turn.
    ///
    /// If the thread already has an active turn, the message is queued as pending input for that
    /// turn. Otherwise it is queued at session scope and a regular turn is started so the agent
    /// can consume that pending input through the normal turn pipeline.
    #[cfg(test)]
    pub(crate) async fn append_message(&self, message: ResponseItem) -> CodexResult<String> {
        let submission_id = uuid::Uuid::new_v4().to_string();
        let pending_item = pending_message_input_item(&message)?;
        if let Err(items) = self
            .codex
            .session
            .inject_response_items(vec![pending_item])
            .await
        {
            self.codex
                .session
                .queue_response_items_for_next_turn(items)
                .await;
            self.codex.session.maybe_start_turn_for_pending_work().await;
        }

        Ok(submission_id)
    }

    /// Append raw Responses API items to the thread's model-visible history.
    pub async fn inject_response_items(&self, items: Vec<ResponseItem>) -> CodexResult<()> {
        if items.is_empty() {
            return Err(CodexErr::InvalidRequest(
                "items must not be empty".to_string(),
            ));
        }

        let turn_context = self.codex.session.new_default_turn().await;
        if self.codex.session.reference_context_item().await.is_none() {
            self.codex
                .session
                .record_context_updates_and_set_reference_context_item(turn_context.as_ref())
                .await;
        }
        self.codex
            .session
            .record_conversation_items(turn_context.as_ref(), &items)
            .await;
        self.codex.session.flush_rollout().await?;
        Ok(())
    }

    pub fn rollout_path(&self) -> Option<PathBuf> {
        self.rollout_path.clone()
    }

    pub fn state_db(&self) -> Option<StateDbHandle> {
        self.codex.state_db()
    }

    pub async fn config_snapshot(&self) -> ThreadConfigSnapshot {
        self.codex.thread_config_snapshot().await
    }

    pub async fn epiphany_state(&self) -> Option<EpiphanyThreadState> {
        self.codex.session.epiphany_state().await
    }

    pub async fn epiphany_update_state(
        &self,
        update: EpiphanyStateUpdate,
    ) -> CodexResult<EpiphanyThreadState> {
        if update.is_empty() {
            return Err(CodexErr::InvalidRequest(
                "epiphany update patch must contain at least one mutation".to_string(),
            ));
        }

        let reference_turn_id = self
            .codex
            .session
            .reference_context_item()
            .await
            .and_then(|item| item.turn_id);
        let mut next_state = self
            .codex
            .session
            .epiphany_state()
            .await
            .unwrap_or_default();
        if let Some(expected_revision) = update.expected_revision
            && next_state.revision != expected_revision
        {
            return Err(CodexErr::InvalidRequest(format!(
                "epiphany state revision mismatch: expected {expected_revision}, found {}",
                next_state.revision
            )));
        }

        let validation_errors = epiphany_state_update_validation_errors(&next_state, &update);
        if !validation_errors.is_empty() {
            return Err(CodexErr::InvalidRequest(format!(
                "invalid epiphany update patch: {}",
                validation_errors.join("; ")
            )));
        }

        apply_epiphany_state_update(&mut next_state, update, reference_turn_id.clone());
        self.codex
            .session
            .set_epiphany_state(Some(next_state.clone()))
            .await;
        self.codex
            .session
            .persist_rollout_items(&[RolloutItem::EpiphanyState(EpiphanyStateItem {
                turn_id: reference_turn_id,
                state: next_state.clone(),
            })])
            .await;
        self.codex.session.flush_rollout().await?;
        Ok(next_state)
    }

    pub async fn epiphany_launch_job(
        &self,
        request: EpiphanyJobLaunchRequest,
    ) -> CodexResult<EpiphanyJobLaunchResult> {
        validate_epiphany_job_launch_request(&request)?;

        let current_state = self
            .codex
            .session
            .epiphany_state()
            .await
            .unwrap_or_default();
        if let Some(expected_revision) = request.expected_revision
            && current_state.revision != expected_revision
        {
            return Err(CodexErr::InvalidRequest(format!(
                "epiphany state revision mismatch: expected {expected_revision}, found {}",
                current_state.revision
            )));
        }
        validate_epiphany_job_launch_target(&current_state, &request)?;

        let launcher_job_id = format!("epiphany-heartbeat-launch-{}", Uuid::new_v4());
        let backend_job_id = Uuid::new_v4().to_string();
        let runtime_store = self.epiphany_runtime_spine_store_path().await;
        open_epiphany_runtime_spine_job(
            runtime_store.as_path(),
            &current_state,
            &request,
            backend_job_id.as_str(),
        )?;
        let replacement_binding = build_epiphany_job_launch_binding(&request);
        let replacement_runtime_link =
            build_epiphany_runtime_link(&request, backend_job_id.as_str());
        let next_job_bindings = replace_or_append_epiphany_job_binding(
            current_state.job_bindings.clone(),
            replacement_binding,
        );

        let validation_errors = epiphany_state_update_validation_errors(
            &current_state,
            &EpiphanyStateUpdate {
                job_bindings: Some(next_job_bindings.clone()),
                runtime_links: vec![replacement_runtime_link.clone()],
                ..Default::default()
            },
        );
        if !validation_errors.is_empty() {
            return Err(CodexErr::InvalidRequest(format!(
                "invalid Epiphany job launch patch: {}",
                validation_errors.join("; ")
            )));
        }

        let epiphany_state = self
            .epiphany_update_state(EpiphanyStateUpdate {
                expected_revision: request.expected_revision,
                job_bindings: Some(next_job_bindings),
                runtime_links: vec![replacement_runtime_link],
                ..Default::default()
            })
            .await?;

        Ok(EpiphanyJobLaunchResult {
            epiphany_state,
            binding_id: request.binding_id,
            launcher_job_id,
            backend_job_id,
        })
    }

    pub async fn epiphany_interrupt_job(
        &self,
        request: EpiphanyJobInterruptRequest,
    ) -> CodexResult<EpiphanyJobInterruptResult> {
        if request.binding_id.trim().is_empty() {
            return Err(CodexErr::InvalidRequest(
                "epiphany job interrupt binding_id must be non-empty".to_string(),
            ));
        }

        let current_state = self
            .codex
            .session
            .epiphany_state()
            .await
            .unwrap_or_default();
        if let Some(expected_revision) = request.expected_revision
            && current_state.revision != expected_revision
        {
            return Err(CodexErr::InvalidRequest(format!(
                "epiphany state revision mismatch: expected {expected_revision}, found {}",
                current_state.revision
            )));
        }

        let Some(binding_index) = current_state
            .job_bindings
            .iter()
            .position(|binding| binding.id == request.binding_id)
        else {
            return Err(CodexErr::InvalidRequest(format!(
                "epiphany job binding {:?} was not found",
                request.binding_id
            )));
        };
        let interrupted_thread_ids = Vec::new();
        let cancel_requested = false;

        let next_job_bindings = clear_epiphany_job_binding_backend(
            current_state.job_bindings.clone(),
            binding_index,
            "No active heartbeat turn is currently bound; launch explicitly to resume specialist work.",
        );
        let epiphany_state = self
            .epiphany_update_state(EpiphanyStateUpdate {
                expected_revision: request.expected_revision,
                job_bindings: Some(next_job_bindings),
                ..Default::default()
            })
            .await?;

        Ok(EpiphanyJobInterruptResult {
            epiphany_state,
            binding_id: request.binding_id,
            cancel_requested,
            interrupted_thread_ids,
        })
    }

    pub async fn epiphany_retrieval_state(&self) -> EpiphanyRetrievalState {
        let config = self.codex.thread_config_snapshot().await;
        let workspace_root = config.cwd.to_path_buf();
        let codex_home = self.codex.session.codex_home().await;
        let fallback_workspace_root = workspace_root.clone();
        let fallback_codex_home = codex_home.clone();
        tokio::task::spawn_blocking(move || {
            epiphany_retrieval::retrieval_state_for_workspace(&workspace_root, codex_home.as_path())
        })
        .await
        .unwrap_or_else(|_| {
            epiphany_retrieval::retrieval_state_for_workspace(
                &fallback_workspace_root,
                fallback_codex_home.as_path(),
            )
        })
    }

    pub async fn epiphany_runtime_spine_store_path(&self) -> PathBuf {
        self.config_snapshot()
            .await
            .cwd
            .join("state")
            .join("runtime-spine.msgpack")
            .to_path_buf()
    }

    pub async fn epiphany_index(
        &self,
        force_full_rebuild: bool,
    ) -> anyhow::Result<EpiphanyRetrievalState> {
        let config = self.codex.thread_config_snapshot().await;
        let workspace_root = config.cwd.to_path_buf();
        let codex_home = self.codex.session.codex_home().await;
        tokio::task::spawn_blocking(move || {
            epiphany_retrieval::index_workspace(
                &workspace_root,
                codex_home.as_path(),
                force_full_rebuild,
            )
        })
        .await
        .map_err(|err| anyhow::anyhow!("epiphany index worker failed: {err}"))?
    }

    pub async fn epiphany_retrieve(
        &self,
        query: EpiphanyRetrieveQuery,
    ) -> anyhow::Result<EpiphanyRetrieveResponse> {
        let config = self.codex.thread_config_snapshot().await;
        let workspace_root = config.cwd.to_path_buf();
        let codex_home = self.codex.session.codex_home().await;
        tokio::task::spawn_blocking(move || {
            epiphany_retrieval::retrieve_workspace(&workspace_root, codex_home.as_path(), query)
        })
        .await
        .map_err(|err| anyhow::anyhow!("epiphany retrieval worker failed: {err}"))?
    }

    pub async fn read_mcp_resource(
        &self,
        server: &str,
        uri: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let result = self
            .codex
            .session
            .read_resource(
                server,
                ReadResourceRequestParams {
                    meta: None,
                    uri: uri.to_string(),
                },
            )
            .await?;

        Ok(serde_json::to_value(result)?)
    }

    pub async fn call_mcp_tool(
        &self,
        server: &str,
        tool: &str,
        arguments: Option<serde_json::Value>,
        meta: Option<serde_json::Value>,
    ) -> anyhow::Result<CallToolResult> {
        self.codex
            .session
            .call_tool(server, tool, arguments, meta)
            .await
    }

    pub fn enabled(&self, feature: Feature) -> bool {
        self.codex.enabled(feature)
    }

    pub async fn increment_out_of_band_elicitation_count(&self) -> CodexResult<u64> {
        let mut guard = self.out_of_band_elicitation_count.lock().await;
        let was_zero = *guard == 0;
        *guard = guard.checked_add(1).ok_or_else(|| {
            CodexErr::Fatal("out-of-band elicitation count overflowed".to_string())
        })?;

        if was_zero {
            self.codex
                .session
                .set_out_of_band_elicitation_pause_state(/*paused*/ true);
        }

        Ok(*guard)
    }

    pub async fn decrement_out_of_band_elicitation_count(&self) -> CodexResult<u64> {
        let mut guard = self.out_of_band_elicitation_count.lock().await;
        if *guard == 0 {
            return Err(CodexErr::InvalidRequest(
                "out-of-band elicitation count is already zero".to_string(),
            ));
        }

        *guard -= 1;
        let now_zero = *guard == 0;
        if now_zero {
            self.codex
                .session
                .set_out_of_band_elicitation_pause_state(/*paused*/ false);
        }

        Ok(*guard)
    }
}

fn validate_epiphany_job_launch_request(request: &EpiphanyJobLaunchRequest) -> CodexResult<()> {
    if request.binding_id.trim().is_empty() {
        return Err(CodexErr::InvalidRequest(
            "epiphany job launch binding_id must be non-empty".to_string(),
        ));
    }
    if matches!(
        request.binding_id.as_str(),
        "retrieval-index" | "graph-remap" | "verification"
    ) {
        return Err(CodexErr::InvalidRequest(format!(
            "epiphany job launch binding_id {:?} is reserved for a derived built-in slot",
            request.binding_id
        )));
    }
    if request.kind != EpiphanyJobKind::Specialist {
        return Err(CodexErr::InvalidRequest(
            "epiphany job launch currently supports only specialist heartbeat turns".to_string(),
        ));
    }
    if request.scope.trim().is_empty() {
        return Err(CodexErr::InvalidRequest(
            "epiphany job launch scope must be non-empty".to_string(),
        ));
    }
    if request.owner_role.trim().is_empty() {
        return Err(CodexErr::InvalidRequest(
            "epiphany job launch owner_role must be non-empty".to_string(),
        ));
    }
    if request.authority_scope.trim().is_empty() {
        return Err(CodexErr::InvalidRequest(
            "epiphany job launch authority_scope must be non-empty".to_string(),
        ));
    }
    if request.instruction.trim().is_empty() {
        return Err(CodexErr::InvalidRequest(
            "epiphany job launch instruction must be non-empty".to_string(),
        ));
    }
    if request.launch_document.thread_id().trim().is_empty() {
        return Err(CodexErr::InvalidRequest(
            "epiphany job launch document must include a thread id".to_string(),
        ));
    }
    if request.output_contract_id.trim().is_empty() {
        return Err(CodexErr::InvalidRequest(
            "epiphany job launch output_contract_id must be non-empty".to_string(),
        ));
    }
    if request.output_contract_id != request.launch_document.output_contract_id() {
        return Err(CodexErr::InvalidRequest(
            "epiphany job launch output_contract_id must match the typed launch document"
                .to_string(),
        ));
    }
    if let Some(max_runtime_seconds) = request.max_runtime_seconds
        && max_runtime_seconds == 0
    {
        return Err(CodexErr::InvalidRequest(
            "epiphany job launch max_runtime_seconds must be >= 1".to_string(),
        ));
    }
    Ok(())
}

fn validate_epiphany_job_launch_target(
    state: &EpiphanyThreadState,
    request: &EpiphanyJobLaunchRequest,
) -> CodexResult<()> {
    let existing_binding = state
        .job_bindings
        .iter()
        .find(|binding| binding.id == request.binding_id);
    let existing_runtime_link = state.runtime_links.iter().find(|link| {
        link.binding_id == request.binding_id && !link.runtime_job_id.trim().is_empty()
    });
    if existing_runtime_link.is_some()
        && existing_binding.is_none_or(|binding| binding.blocking_reason.is_none())
    {
        return Err(CodexErr::InvalidRequest(format!(
            "epiphany job binding {:?} is already bound to an active heartbeat turn; interrupt it before launching a replacement",
            request.binding_id
        )));
    }
    Ok(())
}

fn build_epiphany_job_launch_binding(request: &EpiphanyJobLaunchRequest) -> EpiphanyJobBinding {
    EpiphanyJobBinding {
        id: request.binding_id.clone(),
        kind: request.kind,
        scope: request.scope.clone(),
        owner_role: request.owner_role.clone(),
        authority_scope: Some(request.authority_scope.clone()),
        linked_subgoal_ids: request.linked_subgoal_ids.clone(),
        linked_graph_node_ids: request.linked_graph_node_ids.clone(),
        blocking_reason: None,
    }
}

fn build_epiphany_runtime_link(
    request: &EpiphanyJobLaunchRequest,
    runtime_job_id: &str,
) -> EpiphanyRuntimeLink {
    EpiphanyRuntimeLink {
        id: format!("runtime-link-{}-{runtime_job_id}", request.binding_id),
        binding_id: request.binding_id.clone(),
        surface: "jobLaunch".to_string(),
        role_id: request.owner_role.clone(),
        authority_scope: request.authority_scope.clone(),
        runtime_job_id: runtime_job_id.to_string(),
        runtime_result_id: None,
        linked_subgoal_ids: request.linked_subgoal_ids.clone(),
        linked_graph_node_ids: request.linked_graph_node_ids.clone(),
    }
}

fn open_epiphany_runtime_spine_job(
    store_path: &std::path::Path,
    state: &EpiphanyThreadState,
    request: &EpiphanyJobLaunchRequest,
    backend_job_id: &str,
) -> CodexResult<()> {
    let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    initialize_runtime_spine(
        store_path,
        RuntimeSpineInitOptions {
            runtime_id: "epiphany-local".to_string(),
            display_name: "Epiphany Local".to_string(),
            created_at: now.clone(),
        },
    )
    .map_err(|err| {
        CodexErr::Fatal(format!(
            "failed to initialize Epiphany runtime spine {}: {err}",
            store_path.display()
        ))
    })?;
    ensure_runtime_session(
        store_path,
        RuntimeSpineSessionOptions {
            session_id: "epiphany-main".to_string(),
            objective: state
                .objective
                .clone()
                .filter(|objective| !objective.trim().is_empty())
                .unwrap_or_else(|| "Epiphany heartbeat activation".to_string()),
            created_at: now.clone(),
            coordinator_note: "App-server launch opened this typed runtime session.".to_string(),
        },
    )
    .map_err(|err| {
        CodexErr::Fatal(format!(
            "failed to ensure Epiphany runtime session in {}: {err}",
            store_path.display()
        ))
    })?;
    create_runtime_job(
        store_path,
        RuntimeSpineJobOptions {
            job_id: backend_job_id.to_string(),
            session_id: "epiphany-main".to_string(),
            role: request.owner_role.clone(),
            created_at: now,
            summary: format!(
                "Heartbeat activation queued for binding {} with authority {}.",
                request.binding_id, request.authority_scope
            ),
            artifact_refs: Vec::new(),
        },
    )
    .map_err(|err| {
        CodexErr::Fatal(format!(
            "failed to open Epiphany runtime job {:?} in {}: {err}",
            backend_job_id,
            store_path.display()
        ))
    })?;
    Ok(())
}

fn replace_or_append_epiphany_job_binding(
    mut bindings: Vec<EpiphanyJobBinding>,
    replacement: EpiphanyJobBinding,
) -> Vec<EpiphanyJobBinding> {
    if let Some(existing) = bindings
        .iter_mut()
        .find(|binding| binding.id == replacement.id)
    {
        *existing = replacement;
        return bindings;
    }
    bindings.push(replacement);
    bindings
}

fn clear_epiphany_job_binding_backend(
    mut bindings: Vec<EpiphanyJobBinding>,
    binding_index: usize,
    blocking_reason: &str,
) -> Vec<EpiphanyJobBinding> {
    let binding = &mut bindings[binding_index];
    binding.blocking_reason = Some(blocking_reason.to_string());
    bindings
}

fn epiphany_state_update_validation_errors(
    state: &EpiphanyThreadState,
    update: &EpiphanyStateUpdate,
) -> Vec<String> {
    let mut errors = Vec::new();
    let mut known_evidence_ids: HashSet<&str> = state
        .recent_evidence
        .iter()
        .filter_map(|evidence| nonempty_id(&evidence.id))
        .collect();
    let existing_evidence_ids = known_evidence_ids.clone();
    let existing_observation_ids: HashSet<&str> = state
        .observations
        .iter()
        .filter_map(|observation| nonempty_id(&observation.id))
        .collect();

    let mut patch_evidence_ids = HashSet::new();
    for evidence in &update.evidence {
        require_nonempty_update(&evidence.id, "patch.evidence.id", &mut errors);
        require_nonempty_update(&evidence.kind, "patch.evidence.kind", &mut errors);
        require_nonempty_update(&evidence.status, "patch.evidence.status", &mut errors);
        require_nonempty_update(&evidence.summary, "patch.evidence.summary", &mut errors);
        if !evidence.id.is_empty() && !patch_evidence_ids.insert(evidence.id.as_str()) {
            errors.push(format!("duplicate evidence id {:?}", evidence.id));
        }
        if existing_evidence_ids.contains(evidence.id.as_str()) {
            errors.push(format!(
                "evidence id {:?} already exists in Epiphany state",
                evidence.id
            ));
        }
        if let Some(id) = nonempty_id(&evidence.id) {
            known_evidence_ids.insert(id);
        }
    }

    let mut patch_observation_ids = HashSet::new();
    for observation in &update.observations {
        require_nonempty_update(&observation.id, "patch.observations.id", &mut errors);
        require_nonempty_update(
            &observation.summary,
            "patch.observations.summary",
            &mut errors,
        );
        require_nonempty_update(
            &observation.source_kind,
            "patch.observations.source_kind",
            &mut errors,
        );
        require_nonempty_update(
            &observation.status,
            "patch.observations.status",
            &mut errors,
        );
        if !observation.id.is_empty() && !patch_observation_ids.insert(observation.id.as_str()) {
            errors.push(format!("duplicate observation id {:?}", observation.id));
        }
        if existing_observation_ids.contains(observation.id.as_str()) {
            errors.push(format!(
                "observation id {:?} already exists in Epiphany state",
                observation.id
            ));
        }
        if observation.evidence_ids.is_empty() {
            errors.push(format!(
                "observation {:?} must cite at least one evidence id",
                observation.id
            ));
        }
        for evidence_id in &observation.evidence_ids {
            if !known_evidence_ids.contains(evidence_id.as_str()) {
                errors.push(format!(
                    "observation {:?} cites missing evidence id {:?}",
                    observation.id, evidence_id
                ));
            }
        }
    }

    if let Some(checkpoint) = update.investigation_checkpoint.as_ref() {
        for evidence_id in &checkpoint.evidence_ids {
            if !known_evidence_ids.contains(evidence_id.as_str()) {
                errors.push(format!(
                    "investigation checkpoint cites missing evidence id {:?}",
                    evidence_id
                ));
            }
        }
    }

    if let Some(job_bindings) = update.job_bindings.as_ref() {
        errors.extend(validate_epiphany_job_bindings(job_bindings));
    }
    if !update.acceptance_receipts.is_empty() {
        errors.extend(validate_epiphany_acceptance_receipts(
            &state.acceptance_receipts,
            &update.acceptance_receipts,
            &known_evidence_ids,
        ));
    }
    if !update.runtime_links.is_empty() {
        errors.extend(validate_epiphany_runtime_links(
            &state.runtime_links,
            &update.runtime_links,
        ));
    }
    if let Some(planning) = update.planning.as_ref() {
        errors.extend(validate_epiphany_planning_state(planning));
    }

    errors.extend(epiphany_state_replacement_validation_errors(state, update));
    errors
}

fn validate_epiphany_runtime_links(
    existing: &[EpiphanyRuntimeLink],
    links: &[EpiphanyRuntimeLink],
) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen_ids = existing
        .iter()
        .map(|link| link.id.as_str())
        .collect::<HashSet<_>>();
    for link in links {
        require_nonempty_update(&link.id, "runtime_link.id", &mut errors);
        require_nonempty_update(&link.binding_id, "runtime_link.binding_id", &mut errors);
        require_nonempty_update(&link.surface, "runtime_link.surface", &mut errors);
        require_nonempty_update(&link.role_id, "runtime_link.role_id", &mut errors);
        require_nonempty_update(
            &link.authority_scope,
            "runtime_link.authority_scope",
            &mut errors,
        );
        require_nonempty_update(
            &link.runtime_job_id,
            "runtime_link.runtime_job_id",
            &mut errors,
        );
        if !seen_ids.insert(link.id.as_str()) {
            errors.push(format!("runtime link id {:?} is duplicated", link.id));
        }
    }
    errors
}

fn validate_epiphany_acceptance_receipts(
    existing: &[EpiphanyAcceptanceReceipt],
    receipts: &[EpiphanyAcceptanceReceipt],
    known_evidence_ids: &HashSet<&str>,
) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen_ids = existing
        .iter()
        .map(|receipt| receipt.id.as_str())
        .collect::<HashSet<_>>();
    let mut seen_result_ids = existing
        .iter()
        .map(|receipt| receipt.result_id.as_str())
        .collect::<HashSet<_>>();

    for receipt in receipts {
        require_nonempty_update(&receipt.id, "acceptance_receipt.id", &mut errors);
        require_nonempty_update(
            &receipt.result_id,
            "acceptance_receipt.result_id",
            &mut errors,
        );
        require_nonempty_update(&receipt.job_id, "acceptance_receipt.job_id", &mut errors);
        require_nonempty_update(
            &receipt.binding_id,
            "acceptance_receipt.binding_id",
            &mut errors,
        );
        require_nonempty_update(&receipt.surface, "acceptance_receipt.surface", &mut errors);
        require_nonempty_update(&receipt.role_id, "acceptance_receipt.role_id", &mut errors);
        require_nonempty_update(&receipt.status, "acceptance_receipt.status", &mut errors);
        require_nonempty_update(
            &receipt.accepted_at,
            "acceptance_receipt.accepted_at",
            &mut errors,
        );
        if !seen_ids.insert(receipt.id.as_str()) {
            errors.push(format!(
                "acceptance receipt id {:?} is duplicated",
                receipt.id
            ));
        }
        if !seen_result_ids.insert(receipt.result_id.as_str()) {
            errors.push(format!(
                "runtime result {:?} already has an acceptance receipt",
                receipt.result_id
            ));
        }
        if let Some(evidence_id) = receipt.accepted_evidence_id.as_deref()
            && !known_evidence_ids.contains(evidence_id)
        {
            errors.push(format!(
                "acceptance receipt {:?} cites missing evidence id {:?}",
                receipt.id, evidence_id
            ));
        }
    }

    errors
}

fn validate_epiphany_job_bindings(job_bindings: &[EpiphanyJobBinding]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen_ids = HashSet::<&str>::new();

    for binding in job_bindings {
        require_nonempty_update(&binding.id, "job_binding.id", &mut errors);
        require_nonempty_update(&binding.scope, "job_binding.scope", &mut errors);
        require_nonempty_update(&binding.owner_role, "job_binding.owner_role", &mut errors);
        if let Some(authority_scope) = binding.authority_scope.as_deref() {
            require_nonempty_update(authority_scope, "job_binding.authority_scope", &mut errors);
        }
        if let Some(blocking_reason) = binding.blocking_reason.as_deref() {
            require_nonempty_update(blocking_reason, "job_binding.blocking_reason", &mut errors);
        }
        if !binding.id.is_empty() && !seen_ids.insert(binding.id.as_str()) {
            errors.push(format!("duplicate job binding id {:?}", binding.id));
        }
    }

    errors
}

fn validate_epiphany_planning_state(planning: &EpiphanyPlanningState) -> Vec<String> {
    let mut errors = Vec::new();
    let mut capture_ids = HashSet::<&str>::new();
    for capture in &planning.captures {
        require_nonempty_update(&capture.id, "planning.captures.id", &mut errors);
        require_nonempty_update(&capture.title, "planning.captures.title", &mut errors);
        require_nonempty_update(
            &capture.confidence,
            "planning.captures.confidence",
            &mut errors,
        );
        require_nonempty_update(&capture.status, "planning.captures.status", &mut errors);
        validate_epiphany_planning_source_ref(
            &capture.source,
            "planning.captures.source",
            &mut errors,
        );
        if !capture.id.is_empty() && !capture_ids.insert(capture.id.as_str()) {
            errors.push(format!("duplicate planning capture id {:?}", capture.id));
        }
    }

    let mut backlog_ids = HashSet::<&str>::new();
    for item in &planning.backlog_items {
        require_nonempty_update(&item.id, "planning.backlog_items.id", &mut errors);
        require_nonempty_update(&item.title, "planning.backlog_items.title", &mut errors);
        require_nonempty_update(&item.kind, "planning.backlog_items.kind", &mut errors);
        require_nonempty_update(&item.summary, "planning.backlog_items.summary", &mut errors);
        require_nonempty_update(&item.status, "planning.backlog_items.status", &mut errors);
        require_nonempty_update(&item.horizon, "planning.backlog_items.horizon", &mut errors);
        require_nonempty_update(
            &item.priority.value,
            "planning.backlog_items.priority.value",
            &mut errors,
        );
        require_nonempty_update(
            &item.priority.rationale,
            "planning.backlog_items.priority.rationale",
            &mut errors,
        );
        require_nonempty_update(
            &item.confidence,
            "planning.backlog_items.confidence",
            &mut errors,
        );
        require_nonempty_update(
            &item.product_area,
            "planning.backlog_items.product_area",
            &mut errors,
        );
        for (index, source_ref) in item.source_refs.iter().enumerate() {
            validate_epiphany_planning_source_ref(
                source_ref,
                &format!("planning.backlog_items.source_refs[{index}]"),
                &mut errors,
            );
        }
        if !item.id.is_empty() && !backlog_ids.insert(item.id.as_str()) {
            errors.push(format!("duplicate planning backlog item id {:?}", item.id));
        }
    }

    let mut stream_ids = HashSet::<&str>::new();
    for stream in &planning.roadmap_streams {
        require_nonempty_update(&stream.id, "planning.roadmap_streams.id", &mut errors);
        require_nonempty_update(&stream.title, "planning.roadmap_streams.title", &mut errors);
        require_nonempty_update(
            &stream.purpose,
            "planning.roadmap_streams.purpose",
            &mut errors,
        );
        require_nonempty_update(
            &stream.status,
            "planning.roadmap_streams.status",
            &mut errors,
        );
        for item_id in &stream.item_ids {
            if !backlog_ids.contains(item_id.as_str()) {
                errors.push(format!(
                    "roadmap stream {:?} references missing backlog item {:?}",
                    stream.id, item_id
                ));
            }
        }
        if let Some(near_term_focus) = stream.near_term_focus.as_deref()
            && !near_term_focus.trim().is_empty()
            && !backlog_ids.contains(near_term_focus)
        {
            errors.push(format!(
                "roadmap stream {:?} has missing near_term_focus {:?}",
                stream.id, near_term_focus
            ));
        }
        if !stream.id.is_empty() && !stream_ids.insert(stream.id.as_str()) {
            errors.push(format!(
                "duplicate planning roadmap stream id {:?}",
                stream.id
            ));
        }
    }

    let mut objective_draft_ids = HashSet::<&str>::new();
    for draft in &planning.objective_drafts {
        require_nonempty_update(&draft.id, "planning.objective_drafts.id", &mut errors);
        require_nonempty_update(&draft.title, "planning.objective_drafts.title", &mut errors);
        require_nonempty_update(
            &draft.summary,
            "planning.objective_drafts.summary",
            &mut errors,
        );
        require_nonempty_update(
            &draft.status,
            "planning.objective_drafts.status",
            &mut errors,
        );
        if draft.acceptance_criteria.is_empty() {
            errors.push(format!(
                "objective draft {:?} must include at least one acceptance criterion",
                draft.id
            ));
        }
        for item_id in &draft.source_item_ids {
            if !backlog_ids.contains(item_id.as_str()) {
                errors.push(format!(
                    "objective draft {:?} references missing source backlog item {:?}",
                    draft.id, item_id
                ));
            }
        }
        if !draft.id.is_empty() && !objective_draft_ids.insert(draft.id.as_str()) {
            errors.push(format!("duplicate objective draft id {:?}", draft.id));
        }
    }

    errors
}

fn validate_epiphany_planning_source_ref(
    source_ref: &codex_protocol::protocol::EpiphanyPlanningSourceRef,
    label: &str,
    errors: &mut Vec<String>,
) {
    require_nonempty_update(&source_ref.kind, &format!("{label}.kind"), errors);
    if source_ref.kind == "github_issue" {
        match source_ref.repo.as_deref() {
            Some(repo) => require_nonempty_update(repo, &format!("{label}.repo"), errors),
            None => errors.push(format!("{label}.repo is required for github_issue sources")),
        }
        if source_ref.issue_number.is_none() {
            errors.push(format!(
                "{label}.issue_number is required for github_issue sources"
            ));
        }
    }
}

fn epiphany_state_replacement_validation_errors(
    state: &EpiphanyThreadState,
    update: &EpiphanyStateUpdate,
) -> Vec<String> {
    let validates_subgoal_target = update.subgoals.is_some() || update.active_subgoal_id.is_some();
    let validates_graph_target = update.graphs.is_some()
        || update.graph_frontier.is_some()
        || update.graph_checkpoint.is_some();
    let mut known_evidence_ids: HashSet<&str> = state
        .recent_evidence
        .iter()
        .filter_map(|evidence| nonempty_id(&evidence.id))
        .collect();
    for evidence in &update.evidence {
        if let Some(id) = nonempty_id(&evidence.id) {
            known_evidence_ids.insert(id);
        }
    }

    validate_state_replacement_patch(EpiphanyStateReplacementValidationInput {
        active_subgoal_id: update.active_subgoal_id.as_deref(),
        subgoals: if validates_subgoal_target {
            update
                .subgoals
                .as_deref()
                .or(Some(state.subgoals.as_slice()))
        } else {
            None
        },
        invariants: update.invariants.as_deref(),
        graphs: if validates_graph_target {
            update.graphs.as_ref().or(Some(&state.graphs))
        } else {
            None
        },
        graph_frontier: update.graph_frontier.as_ref(),
        graph_checkpoint: update.graph_checkpoint.as_ref(),
        investigation_checkpoint: update.investigation_checkpoint.as_ref(),
        available_evidence_ids: Some(&known_evidence_ids),
        churn: update.churn.as_ref(),
    })
}

fn nonempty_id(id: &str) -> Option<&str> {
    if id.is_empty() { None } else { Some(id) }
}

fn require_nonempty_update(value: &str, label: &str, errors: &mut Vec<String>) {
    if value.trim().is_empty() {
        errors.push(format!("{label} must not be empty"));
    }
}

fn apply_epiphany_state_update(
    state: &mut EpiphanyThreadState,
    update: EpiphanyStateUpdate,
    reference_turn_id: Option<String>,
) {
    if let Some(objective) = update.objective {
        state.objective = Some(objective);
    }
    if let Some(active_subgoal_id) = update.active_subgoal_id {
        state.active_subgoal_id = Some(active_subgoal_id);
    }
    if let Some(subgoals) = update.subgoals {
        state.subgoals = subgoals;
    }
    if let Some(invariants) = update.invariants {
        state.invariants = invariants;
    }
    if let Some(graphs) = update.graphs {
        state.graphs = graphs;
    }
    if let Some(graph_frontier) = update.graph_frontier {
        state.graph_frontier = Some(graph_frontier);
    }
    if let Some(graph_checkpoint) = update.graph_checkpoint {
        state.graph_checkpoint = Some(graph_checkpoint);
    }
    if let Some(scratch) = update.scratch {
        state.scratch = Some(scratch);
    }
    if let Some(checkpoint) = update.investigation_checkpoint {
        state.investigation_checkpoint = Some(checkpoint);
    }
    if let Some(job_bindings) = update.job_bindings {
        state.job_bindings = job_bindings;
    }
    prepend_recent(&mut state.acceptance_receipts, update.acceptance_receipts);
    prepend_recent(&mut state.runtime_links, update.runtime_links);
    if let Some(churn) = update.churn {
        state.churn = Some(churn);
    }
    if let Some(mode) = update.mode {
        state.mode = Some(mode);
    }
    if let Some(planning) = update.planning {
        state.planning = planning;
    }

    prepend_recent(&mut state.observations, update.observations);
    prepend_recent(&mut state.recent_evidence, update.evidence);
    state.revision = state.revision.saturating_add(1);
    state.last_updated_turn_id = reference_turn_id;
}

fn prepend_recent<T>(items: &mut Vec<T>, mut new_items: Vec<T>) {
    if new_items.is_empty() {
        return;
    }
    new_items.append(items);
    *items = new_items;
}

#[cfg(test)]
mod epiphany_update_tests {
    use super::*;
    use codex_protocol::protocol::EpiphanyBacklogItem;
    use codex_protocol::protocol::EpiphanyGraph;
    use codex_protocol::protocol::EpiphanyGraphNode;
    use codex_protocol::protocol::EpiphanyJobKind;
    use codex_protocol::protocol::EpiphanyPlanningCapture;
    use codex_protocol::protocol::EpiphanyPlanningPriority;
    use codex_protocol::protocol::EpiphanyPlanningSourceRef;

    fn evidence(id: &str) -> EpiphanyEvidenceRecord {
        EpiphanyEvidenceRecord {
            id: id.to_string(),
            kind: "verification".to_string(),
            status: "ok".to_string(),
            summary: "Evidence summary".to_string(),
            code_refs: Vec::new(),
        }
    }

    fn observation(id: &str, evidence_ids: Vec<&str>) -> EpiphanyObservation {
        EpiphanyObservation {
            id: id.to_string(),
            summary: "Observation summary".to_string(),
            source_kind: "smoke".to_string(),
            status: "ok".to_string(),
            code_refs: Vec::new(),
            evidence_ids: evidence_ids.into_iter().map(str::to_string).collect(),
        }
    }

    fn job_binding(id: &str) -> EpiphanyJobBinding {
        EpiphanyJobBinding {
            id: id.to_string(),
            kind: EpiphanyJobKind::Specialist,
            scope: "role-scoped specialist work".to_string(),
            owner_role: "epiphany-harness".to_string(),
            authority_scope: Some("epiphany.specialist".to_string()),
            linked_subgoal_ids: vec!["phase-6".to_string()],
            linked_graph_node_ids: vec!["job-surface".to_string()],
            blocking_reason: None,
        }
    }

    fn acceptance_receipt(
        id: &str,
        result_id: &str,
        evidence_id: &str,
    ) -> EpiphanyAcceptanceReceipt {
        EpiphanyAcceptanceReceipt {
            id: id.to_string(),
            result_id: result_id.to_string(),
            job_id: "runtime-job-1".to_string(),
            binding_id: "modeling".to_string(),
            surface: "roleAccept".to_string(),
            role_id: "modeling".to_string(),
            status: "accepted".to_string(),
            accepted_at: "2026-05-12T00:00:00Z".to_string(),
            accepted_observation_id: Some("obs-modeling".to_string()),
            accepted_evidence_id: Some(evidence_id.to_string()),
            summary: Some("Accepted modeling result.".to_string()),
        }
    }

    #[test]
    fn apply_epiphany_state_update_replaces_typed_fields_and_prepends_evidence() {
        let mut state = EpiphanyThreadState {
            revision: 3,
            recent_evidence: vec![EpiphanyEvidenceRecord {
                id: "old-evidence".to_string(),
                kind: "research".to_string(),
                status: "ok".to_string(),
                summary: "Older finding".to_string(),
                code_refs: Vec::new(),
            }],
            ..Default::default()
        };

        apply_epiphany_state_update(
            &mut state,
            EpiphanyStateUpdate {
                objective: Some("Keep the map honest".to_string()),
                investigation_checkpoint: Some(EpiphanyInvestigationCheckpoint {
                    checkpoint_id: "ix-1".to_string(),
                    kind: "slice_planning".to_string(),
                    focus: "Keep the durable packet small and explicit.".to_string(),
                    next_action: Some(
                        "Resume from the packet instead of the ghost transcript.".to_string(),
                    ),
                    ..Default::default()
                }),
                evidence: vec![EpiphanyEvidenceRecord {
                    id: "new-evidence".to_string(),
                    kind: "verification".to_string(),
                    status: "ok".to_string(),
                    summary: "New finding".to_string(),
                    code_refs: Vec::new(),
                }],
                churn: Some(EpiphanyChurnState {
                    understanding_status: "grounded".to_string(),
                    diff_pressure: "low".to_string(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            Some("turn-1".to_string()),
        );

        assert_eq!(state.revision, 4);
        assert_eq!(state.objective.as_deref(), Some("Keep the map honest"));
        assert_eq!(state.last_updated_turn_id.as_deref(), Some("turn-1"));
        assert_eq!(
            state
                .investigation_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.checkpoint_id.as_str()),
            Some("ix-1")
        );
        assert_eq!(state.recent_evidence[0].id, "new-evidence");
        assert_eq!(state.recent_evidence[1].id, "old-evidence");
        assert_eq!(
            state
                .churn
                .as_ref()
                .map(|churn| churn.diff_pressure.as_str()),
            Some("low")
        );
    }

    #[test]
    fn apply_epiphany_state_update_replaces_job_bindings() {
        let mut state = EpiphanyThreadState {
            revision: 2,
            job_bindings: vec![job_binding("old")],
            ..Default::default()
        };

        apply_epiphany_state_update(
            &mut state,
            EpiphanyStateUpdate {
                job_bindings: Some(vec![job_binding("new")]),
                ..Default::default()
            },
            Some("turn-jobs".to_string()),
        );

        assert_eq!(state.revision, 3);
        assert_eq!(state.job_bindings.len(), 1);
        assert_eq!(state.job_bindings[0].id, "new");
        assert_eq!(
            state.job_bindings[0].authority_scope.as_deref(),
            Some("epiphany.specialist")
        );
        assert_eq!(state.last_updated_turn_id.as_deref(), Some("turn-jobs"));
    }

    #[test]
    fn apply_epiphany_state_update_prepends_acceptance_receipts() {
        let mut state = EpiphanyThreadState {
            revision: 2,
            recent_evidence: vec![evidence("ev-new")],
            acceptance_receipts: vec![acceptance_receipt("accept-old", "result-old", "ev-old")],
            ..Default::default()
        };

        apply_epiphany_state_update(
            &mut state,
            EpiphanyStateUpdate {
                acceptance_receipts: vec![acceptance_receipt("accept-new", "result-new", "ev-new")],
                ..Default::default()
            },
            Some("turn-accept".to_string()),
        );

        assert_eq!(state.revision, 3);
        assert_eq!(state.acceptance_receipts[0].id, "accept-new");
        assert_eq!(state.acceptance_receipts[1].id, "accept-old");
    }

    #[test]
    fn validate_epiphany_state_update_accepts_planning_state() {
        let update = EpiphanyStateUpdate {
            planning: Some(EpiphanyPlanningState {
                captures: vec![EpiphanyPlanningCapture {
                    id: "capture-github-42".to_string(),
                    title: "Import issue backlog".to_string(),
                    confidence: "medium".to_string(),
                    status: "new".to_string(),
                    source: EpiphanyPlanningSourceRef {
                        kind: "github_issue".to_string(),
                        provider: Some("github".to_string()),
                        repo: Some("GameCult/Epiphany".to_string()),
                        issue_number: Some(42),
                        ..Default::default()
                    },
                    ..Default::default()
                }],
                backlog_items: vec![EpiphanyBacklogItem {
                    id: "backlog-planning-api".to_string(),
                    title: "Expose planning projection".to_string(),
                    kind: "feature".to_string(),
                    summary: "Make planning state queryable by the GUI.".to_string(),
                    status: "ready".to_string(),
                    horizon: "now".to_string(),
                    priority: EpiphanyPlanningPriority {
                        value: "p1".to_string(),
                        rationale: "Unblocks planning operations.".to_string(),
                        ..Default::default()
                    },
                    confidence: "high".to_string(),
                    product_area: "gui".to_string(),
                    lane_hints: vec!["imagination".to_string()],
                    ..Default::default()
                }],
                roadmap_streams: vec![codex_protocol::protocol::EpiphanyRoadmapStream {
                    id: "stream-gui".to_string(),
                    title: "GUI Operator Surface".to_string(),
                    purpose: "Let the human inspect and steer planning.".to_string(),
                    status: "active".to_string(),
                    item_ids: vec!["backlog-planning-api".to_string()],
                    ..Default::default()
                }],
                objective_drafts: vec![codex_protocol::protocol::EpiphanyObjectiveDraft {
                    id: "objdraft-planning-api".to_string(),
                    title: "Build planning API slice".to_string(),
                    summary: "Land typed planning state and read-only projection.".to_string(),
                    source_item_ids: vec!["backlog-planning-api".to_string()],
                    acceptance_criteria: vec!["Projection returns planning counts.".to_string()],
                    status: "draft".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };

        assert!(
            epiphany_state_update_validation_errors(&EpiphanyThreadState::default(), &update)
                .is_empty()
        );
    }

    #[test]
    fn validate_epiphany_state_update_rejects_invalid_planning_state() {
        let update = EpiphanyStateUpdate {
            planning: Some(EpiphanyPlanningState {
                captures: vec![EpiphanyPlanningCapture {
                    id: "capture-bad-github".to_string(),
                    title: "Missing repo issue".to_string(),
                    confidence: "medium".to_string(),
                    status: "new".to_string(),
                    source: EpiphanyPlanningSourceRef {
                        kind: "github_issue".to_string(),
                        issue_number: None,
                        ..Default::default()
                    },
                    ..Default::default()
                }],
                backlog_items: vec![EpiphanyBacklogItem {
                    id: "backlog-1".to_string(),
                    title: "Backlog without priority rationale".to_string(),
                    kind: "feature".to_string(),
                    summary: "Invalid on purpose.".to_string(),
                    status: "ready".to_string(),
                    horizon: "now".to_string(),
                    priority: EpiphanyPlanningPriority {
                        value: "p1".to_string(),
                        rationale: String::new(),
                        ..Default::default()
                    },
                    confidence: "high".to_string(),
                    product_area: "gui".to_string(),
                    ..Default::default()
                }],
                roadmap_streams: vec![codex_protocol::protocol::EpiphanyRoadmapStream {
                    id: "stream-gui".to_string(),
                    title: "GUI Operator Surface".to_string(),
                    purpose: "Let the human inspect and steer planning.".to_string(),
                    status: "active".to_string(),
                    item_ids: vec!["missing-backlog".to_string()],
                    ..Default::default()
                }],
                objective_drafts: vec![codex_protocol::protocol::EpiphanyObjectiveDraft {
                    id: "objdraft-empty".to_string(),
                    title: "Empty acceptance draft".to_string(),
                    summary: "Invalid on purpose.".to_string(),
                    source_item_ids: vec!["missing-backlog".to_string()],
                    status: "draft".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            }),
            ..Default::default()
        };

        let errors =
            epiphany_state_update_validation_errors(&EpiphanyThreadState::default(), &update);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("repo is required for github_issue"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("priority.rationale"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("references missing backlog item"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("must include at least one acceptance criterion"))
        );
    }

    #[test]
    fn validate_epiphany_state_update_accepts_observation_linked_to_existing_evidence() {
        let state = EpiphanyThreadState {
            recent_evidence: vec![evidence("ev-existing")],
            ..Default::default()
        };
        let update = EpiphanyStateUpdate {
            observations: vec![observation("obs-new", vec!["ev-existing"])],
            ..Default::default()
        };

        assert!(epiphany_state_update_validation_errors(&state, &update).is_empty());
    }

    #[test]
    fn validate_epiphany_state_update_rejects_observation_with_missing_evidence() {
        let update = EpiphanyStateUpdate {
            observations: vec![observation("obs-new", vec!["ev-missing"])],
            ..Default::default()
        };

        let errors =
            epiphany_state_update_validation_errors(&EpiphanyThreadState::default(), &update);

        assert!(
            errors
                .iter()
                .any(|error| error.contains("cites missing evidence id"))
        );
    }

    #[test]
    fn validate_epiphany_state_update_rejects_empty_and_duplicate_append_ids() {
        let state = EpiphanyThreadState {
            observations: vec![observation("obs-existing", vec!["ev-existing"])],
            recent_evidence: vec![evidence("ev-existing")],
            ..Default::default()
        };
        let update = EpiphanyStateUpdate {
            observations: vec![
                observation("obs-existing", vec!["ev-new"]),
                observation("obs-dup", vec!["ev-new"]),
                observation("obs-dup", vec!["ev-new"]),
                EpiphanyObservation {
                    id: String::new(),
                    ..observation("unused", vec!["ev-new"])
                },
            ],
            evidence: vec![
                evidence("ev-existing"),
                evidence("ev-new"),
                evidence("ev-new"),
                EpiphanyEvidenceRecord {
                    id: String::new(),
                    ..evidence("unused")
                },
            ],
            ..Default::default()
        };

        let errors = epiphany_state_update_validation_errors(&state, &update);

        assert!(
            errors
                .iter()
                .any(|error| error.contains("evidence id \"ev-existing\" already exists"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("duplicate evidence id \"ev-new\""))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("observation id \"obs-existing\" already exists"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("duplicate observation id \"obs-dup\""))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("patch.evidence.id must not be empty"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("patch.observations.id must not be empty"))
        );
    }

    #[test]
    fn validate_epiphany_state_update_rejects_duplicate_acceptance_result() {
        let state = EpiphanyThreadState {
            acceptance_receipts: vec![acceptance_receipt(
                "accept-existing",
                "result-existing",
                "ev-existing",
            )],
            recent_evidence: vec![evidence("ev-existing"), evidence("ev-new")],
            ..Default::default()
        };
        let update = EpiphanyStateUpdate {
            acceptance_receipts: vec![
                acceptance_receipt("accept-new", "result-existing", "ev-new"),
                acceptance_receipt("accept-new", "result-new", "ev-new"),
            ],
            ..Default::default()
        };

        let errors = epiphany_state_update_validation_errors(&state, &update);

        assert!(
            errors
                .iter()
                .any(|error| error.contains("runtime result \"result-existing\" already has"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("acceptance receipt id \"accept-new\" is duplicated"))
        );
    }

    #[test]
    fn validate_epiphany_state_update_rejects_structurally_invalid_replacements() {
        let state = EpiphanyThreadState {
            graphs: EpiphanyGraphs {
                architecture: EpiphanyGraph {
                    nodes: vec![EpiphanyGraphNode {
                        id: "state".to_string(),
                        title: "State".to_string(),
                        purpose: "Carry the explicit map".to_string(),
                        ..Default::default()
                    }],
                    edges: Vec::new(),
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let update = EpiphanyStateUpdate {
            graph_frontier: Some(EpiphanyGraphFrontier {
                active_node_ids: vec!["missing".to_string()],
                ..Default::default()
            }),
            ..Default::default()
        };

        let errors = epiphany_state_update_validation_errors(&state, &update);

        assert!(
            errors
                .iter()
                .any(|error| error.contains("graph frontier references missing node"))
        );
    }

    #[test]
    fn validate_epiphany_state_update_rejects_investigation_checkpoint_with_missing_evidence() {
        let update = EpiphanyStateUpdate {
            investigation_checkpoint: Some(EpiphanyInvestigationCheckpoint {
                checkpoint_id: "ix-missing".to_string(),
                kind: "source_gathering".to_string(),
                focus: "Trace the compaction seam.".to_string(),
                next_action: Some("Re-gather source before implementation.".to_string()),
                evidence_ids: vec!["ev-missing".to_string()],
                ..Default::default()
            }),
            ..Default::default()
        };

        let errors =
            epiphany_state_update_validation_errors(&EpiphanyThreadState::default(), &update);

        assert!(
            errors
                .iter()
                .any(|error| error.contains("investigation checkpoint cites missing evidence id"))
        );
    }

    #[test]
    fn validate_epiphany_state_update_rejects_invalid_job_bindings() {
        let update = EpiphanyStateUpdate {
            job_bindings: Some(vec![
                job_binding("dup"),
                job_binding("dup"),
                EpiphanyJobBinding {
                    id: String::new(),
                    kind: EpiphanyJobKind::Verification,
                    scope: String::new(),
                    owner_role: String::new(),
                    authority_scope: Some(String::new()),
                    linked_subgoal_ids: Vec::new(),
                    linked_graph_node_ids: Vec::new(),
                    blocking_reason: Some(String::new()),
                },
            ]),
            ..Default::default()
        };

        let errors =
            epiphany_state_update_validation_errors(&EpiphanyThreadState::default(), &update);

        assert!(
            errors
                .iter()
                .any(|error| error.contains("duplicate job binding id"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("job_binding.id must not be empty"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("job_binding.scope must not be empty"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("job_binding.owner_role must not be empty"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("job_binding.authority_scope must not be empty"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("job_binding.blocking_reason must not be empty"))
        );
    }

    #[test]
    fn build_epiphany_job_launch_binding_leaves_lifecycle_to_runtime_links() {
        let request = EpiphanyJobLaunchRequest {
            expected_revision: Some(7),
            binding_id: "modeling-checkpoint-worker".to_string(),
            kind: EpiphanyJobKind::Specialist,
            scope: "role-scoped modeling/checkpoint maintenance".to_string(),
            owner_role: "epiphany-modeler".to_string(),
            authority_scope: "epiphany.role.modeling".to_string(),
            linked_subgoal_ids: vec!["phase-6".to_string()],
            linked_graph_node_ids: vec!["runtime-spine".to_string()],
            instruction: "Model the target before implementation.".to_string(),
            launch_document: EpiphanyWorkerLaunchDocument::Role(
                epiphany_core::EpiphanyRoleWorkerLaunchDocument {
                    thread_id: "thread-1".to_string(),
                    role_id: "modeling".to_string(),
                    state_revision: 7,
                    objective: Some("keep state typed".to_string()),
                    active_subgoal_id: None,
                    active_subgoals: Vec::new(),
                    active_graph_node_ids: vec!["runtime-spine".to_string()],
                    investigation_checkpoint: None,
                    scratch: None,
                    invariants: Vec::new(),
                    graphs: None,
                    recent_evidence: Vec::new(),
                    recent_observations: Vec::new(),
                    graph_frontier: None,
                    graph_checkpoint: None,
                    planning: None,
                    churn: None,
                },
            ),
            output_contract_id: "epiphany.worker.role_result.v0".to_string(),
            max_runtime_seconds: Some(60),
        };

        let binding = build_epiphany_job_launch_binding(&request);
        let runtime_link = build_epiphany_runtime_link(&request, "turn-1");

        assert_eq!(
            binding.authority_scope.as_deref(),
            Some("epiphany.role.modeling")
        );
        assert_eq!(
            runtime_link.id,
            "runtime-link-modeling-checkpoint-worker-turn-1"
        );
        assert_eq!(runtime_link.runtime_job_id, "turn-1");
        assert_eq!(runtime_link.runtime_result_id, None);
        assert_eq!(runtime_link.role_id, "epiphany-modeler");
    }
}

fn pending_message_input_item(message: &ResponseItem) -> CodexResult<ResponseInputItem> {
    match message {
        ResponseItem::Message { role, content, .. } => Ok(ResponseInputItem::Message {
            role: role.clone(),
            content: content.clone(),
        }),
        _ => Err(CodexErr::InvalidRequest(
            "append_message only supports ResponseItem::Message".to_string(),
        )),
    }
}
