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
use codex_protocol::ThreadId;
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
use codex_protocol::protocol::EpiphanyChurnState;
use codex_protocol::protocol::EpiphanyEvidenceRecord;
use codex_protocol::protocol::EpiphanyGraphCheckpoint;
use codex_protocol::protocol::EpiphanyGraphFrontier;
use codex_protocol::protocol::EpiphanyGraphs;
use codex_protocol::protocol::EpiphanyInvariant;
use codex_protocol::protocol::EpiphanyInvestigationCheckpoint;
use codex_protocol::protocol::EpiphanyJobBackendKind;
use codex_protocol::protocol::EpiphanyJobBinding;
use codex_protocol::protocol::EpiphanyJobKind;
use codex_protocol::protocol::EpiphanyModeState;
use codex_protocol::protocol::EpiphanyObservation;
use codex_protocol::protocol::EpiphanyRetrievalState;
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
use rmcp::model::ReadResourceRequestParams;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;
use tokio::sync::Mutex;
use tokio::sync::watch;

use codex_rollout::state_db::StateDbHandle;
use serde_json::Value;
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
    pub observations: Vec<EpiphanyObservation>,
    pub evidence: Vec<EpiphanyEvidenceRecord>,
    pub churn: Option<EpiphanyChurnState>,
    pub mode: Option<EpiphanyModeState>,
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
    pub input_json: Value,
    pub output_schema_json: Option<Value>,
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
            && self.observations.is_empty()
            && self.evidence.is_empty()
            && self.churn.is_none()
            && self.mode.is_none()
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

    pub async fn epiphany_state_runtime(&self) -> Option<StateDbHandle> {
        if let Some(state_db) = self.state_db() {
            return Some(state_db);
        }

        let codex_home = self.codex.session.codex_home().await;
        let config = self.codex.thread_config_snapshot().await;
        codex_state::StateRuntime::init(codex_home.to_path_buf(), config.model_provider_id)
            .await
            .ok()
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

        let session = self.codex.session.clone();
        let state_db = self.epiphany_state_runtime().await.ok_or_else(|| {
            CodexErr::InvalidRequest(
                "sqlite state db is unavailable for Epiphany job launch".to_string(),
            )
        })?;
        let launch_turn = session.new_default_turn().await;
        let launch_options = crate::tools::handlers::agent_jobs::build_runner_options(
            &session,
            &launch_turn,
            Some(1),
        )
        .await
        .map_err(|err| {
            CodexErr::InvalidRequest(format!(
                "failed to prepare Epiphany agent job runner: {err}"
            ))
        })?;

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
        validate_epiphany_job_launch_target(&current_state, &state_db, &request).await?;

        let launcher_job_id = format!("epiphany-launch-{}", Uuid::new_v4());
        let backend_job_id = Uuid::new_v4().to_string();
        let replacement_binding = build_epiphany_job_launch_binding(
            &request,
            launcher_job_id.as_str(),
            backend_job_id.as_str(),
        );
        let next_job_bindings = replace_or_append_epiphany_job_binding(
            current_state.job_bindings.clone(),
            replacement_binding,
        );

        let validation_errors = epiphany_state_update_validation_errors(
            &current_state,
            &EpiphanyStateUpdate {
                job_bindings: Some(next_job_bindings.clone()),
                ..Default::default()
            },
        );
        if !validation_errors.is_empty() {
            return Err(CodexErr::InvalidRequest(format!(
                "invalid Epiphany job launch patch: {}",
                validation_errors.join("; ")
            )));
        }

        let output_csv_path = epiphany_agent_job_output_csv_path(
            self.codex.session.codex_home().await.as_path(),
            backend_job_id.as_str(),
        );
        let row_object = request.input_json.as_object().cloned().ok_or_else(|| {
            CodexErr::InvalidRequest(
                "epiphany job launch input_json must be a JSON object".to_string(),
            )
        })?;
        let input_headers = sorted_json_object_keys(&row_object);
        let job_items = vec![codex_state::AgentJobItemCreateParams {
            item_id: request.binding_id.clone(),
            row_index: 0,
            source_id: Some(request.binding_id.clone()),
            row_json: Value::Object(row_object),
        }];

        state_db
            .create_agent_job(
                &codex_state::AgentJobCreateParams {
                    id: backend_job_id.clone(),
                    name: epiphany_agent_job_name(request.binding_id.as_str()),
                    instruction: request.instruction.clone(),
                    auto_export: false,
                    max_runtime_seconds: request.max_runtime_seconds,
                    output_schema_json: request.output_schema_json.clone(),
                    input_headers,
                    input_csv_path: output_csv_path.display().to_string(),
                    output_csv_path: output_csv_path.display().to_string(),
                },
                job_items.as_slice(),
            )
            .await
            .map_err(|err| {
                CodexErr::Fatal(format!("failed to create Epiphany backend job: {err}"))
            })?;

        let epiphany_state = match self
            .epiphany_update_state(EpiphanyStateUpdate {
                expected_revision: request.expected_revision,
                job_bindings: Some(next_job_bindings),
                ..Default::default()
            })
            .await
        {
            Ok(epiphany_state) => epiphany_state,
            Err(err) => {
                let _ = state_db
                    .mark_agent_job_cancelled(
                        backend_job_id.as_str(),
                        "Epiphany launch state update failed before the binding could persist.",
                    )
                    .await;
                return Err(err);
            }
        };

        let runner_session = session.clone();
        let runner_state_db = state_db.clone();
        let runner_job_id = backend_job_id.clone();
        tokio::spawn(async move {
            if let Err(err) = runner_state_db
                .mark_agent_job_running(runner_job_id.as_str())
                .await
            {
                let error_message =
                    format!("failed to transition Epiphany backend job to running: {err}");
                let _ = runner_state_db
                    .mark_agent_job_failed(runner_job_id.as_str(), error_message.as_str())
                    .await;
                return;
            }

            if let Err(err) = crate::tools::handlers::agent_jobs::run_agent_job_loop(
                runner_session,
                launch_turn,
                runner_state_db.clone(),
                runner_job_id.clone(),
                launch_options,
            )
            .await
            {
                let error_message = format!("Epiphany backend job runner failed: {err}");
                let _ = runner_state_db
                    .mark_agent_job_failed(runner_job_id.as_str(), error_message.as_str())
                    .await;
            }
        });

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

        let state_db = self.epiphany_state_runtime().await.ok_or_else(|| {
            CodexErr::InvalidRequest(
                "sqlite state db is unavailable for Epiphany job interrupt".to_string(),
            )
        })?;
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
        let binding = current_state.job_bindings[binding_index].clone();
        let agent_job_id = binding_agent_jobs_job_id_core(&binding).ok_or_else(|| {
            CodexErr::InvalidRequest(format!(
                "epiphany job binding {:?} is not currently backed by the agent_jobs runtime",
                request.binding_id
            ))
        })?;

        let mut interrupted_thread_ids = Vec::new();
        let running_items = state_db
            .list_agent_job_items(
                agent_job_id,
                Some(codex_state::AgentJobItemStatus::Running),
                Some(64),
            )
            .await
            .map_err(|err| {
                CodexErr::Fatal(format!(
                    "failed to inspect running Epiphany backend job items: {err}"
                ))
            })?;
        for running_item in running_items {
            let Some(assigned_thread_id) = running_item.assigned_thread_id else {
                continue;
            };
            let Ok(thread_id) = ThreadId::from_string(assigned_thread_id.as_str()) else {
                continue;
            };
            match self
                .codex
                .session
                .services
                .agent_control
                .shutdown_live_agent(thread_id)
                .await
            {
                Ok(_) | Err(CodexErr::ThreadNotFound(_)) | Err(CodexErr::InternalAgentDied) => {
                    interrupted_thread_ids.push(assigned_thread_id);
                }
                Err(err) => {
                    return Err(CodexErr::Fatal(format!(
                        "failed to interrupt Epiphany worker thread {assigned_thread_id}: {err}"
                    )));
                }
            }
        }

        let cancel_requested = state_db
            .mark_agent_job_cancelled(
                agent_job_id,
                request.reason.as_deref().unwrap_or(
                    "Epiphany launch authority requested an interrupt for the bound backend job.",
                ),
            )
            .await
            .map_err(|err| {
                CodexErr::Fatal(format!("failed to cancel Epiphany backend job: {err}"))
            })?;

        let next_job_bindings = clear_epiphany_job_binding_backend(
            current_state.job_bindings.clone(),
            binding_index,
            "No active runtime backend is currently bound; launch explicitly to resume specialist work.",
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
            "epiphany job launch currently supports only specialist jobs on the agent_jobs backend"
                .to_string(),
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
    if !request.input_json.is_object() {
        return Err(CodexErr::InvalidRequest(
            "epiphany job launch input_json must be a JSON object".to_string(),
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

async fn validate_epiphany_job_launch_target(
    state: &EpiphanyThreadState,
    state_db: &StateDbHandle,
    request: &EpiphanyJobLaunchRequest,
) -> CodexResult<()> {
    let Some(existing_binding) = state
        .job_bindings
        .iter()
        .find(|binding| binding.id == request.binding_id)
    else {
        return Ok(());
    };
    let Some(agent_job_id) = binding_agent_jobs_job_id_core(existing_binding) else {
        return Ok(());
    };
    let existing_job = state_db.get_agent_job(agent_job_id).await.map_err(|err| {
        CodexErr::Fatal(format!(
            "failed to inspect existing Epiphany backend job {:?}: {err}",
            agent_job_id
        ))
    })?;
    if existing_job.as_ref().is_some_and(|job| {
        matches!(
            job.status,
            codex_state::AgentJobStatus::Pending | codex_state::AgentJobStatus::Running
        )
    }) {
        let progress = state_db
            .get_agent_job_progress(agent_job_id)
            .await
            .map_err(|err| {
                CodexErr::Fatal(format!(
                    "failed to inspect existing Epiphany backend job progress {:?}: {err}",
                    agent_job_id
                ))
            })?;
        if progress.pending_items == 0 && progress.running_items == 0 {
            if progress.failed_items > 0 {
                state_db
                    .mark_agent_job_failed(
                        agent_job_id,
                        "Epiphany launch recovered a stale active backend job with no active items.",
                    )
                    .await
                    .map_err(|err| {
                        CodexErr::Fatal(format!(
                            "failed to mark stale Epiphany backend job {:?} failed: {err}",
                            agent_job_id
                        ))
                    })?;
            } else {
                state_db
                    .mark_agent_job_completed(agent_job_id)
                    .await
                    .map_err(|err| {
                        CodexErr::Fatal(format!(
                            "failed to mark stale Epiphany backend job {:?} completed: {err}",
                            agent_job_id
                        ))
                    })?;
            }
            return Ok(());
        }
        return Err(CodexErr::InvalidRequest(format!(
            "epiphany job binding {:?} is already bound to active backend job {:?}; interrupt it before launching a replacement",
            request.binding_id, agent_job_id
        )));
    }
    Ok(())
}

fn build_epiphany_job_launch_binding(
    request: &EpiphanyJobLaunchRequest,
    launcher_job_id: &str,
    backend_job_id: &str,
) -> EpiphanyJobBinding {
    EpiphanyJobBinding {
        id: request.binding_id.clone(),
        kind: request.kind,
        scope: request.scope.clone(),
        owner_role: request.owner_role.clone(),
        launcher_job_id: Some(launcher_job_id.to_string()),
        authority_scope: Some(request.authority_scope.clone()),
        backend_kind: Some(EpiphanyJobBackendKind::AgentJobs),
        backend_job_id: Some(backend_job_id.to_string()),
        runtime_agent_job_id: Some(backend_job_id.to_string()),
        linked_subgoal_ids: request.linked_subgoal_ids.clone(),
        linked_graph_node_ids: request.linked_graph_node_ids.clone(),
        progress_note: Some(
            "Explicitly launched through the Epiphany authority surface onto the agent_jobs backend."
                .to_string(),
        ),
        blocking_reason: None,
    }
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
    binding.launcher_job_id = None;
    binding.backend_kind = None;
    binding.backend_job_id = None;
    binding.runtime_agent_job_id = None;
    binding.progress_note = None;
    binding.blocking_reason = Some(blocking_reason.to_string());
    bindings
}

fn binding_backend_kind_core(binding: &EpiphanyJobBinding) -> Option<EpiphanyJobBackendKind> {
    binding.backend_kind.or_else(|| {
        if binding.runtime_agent_job_id.is_some() {
            Some(EpiphanyJobBackendKind::AgentJobs)
        } else {
            None
        }
    })
}

fn binding_backend_job_id_core(binding: &EpiphanyJobBinding) -> Option<&str> {
    binding
        .backend_job_id
        .as_deref()
        .or(binding.runtime_agent_job_id.as_deref())
}

fn binding_agent_jobs_job_id_core(binding: &EpiphanyJobBinding) -> Option<&str> {
    match binding_backend_kind_core(binding) {
        Some(EpiphanyJobBackendKind::AgentJobs) => binding_backend_job_id_core(binding),
        None => None,
    }
}

fn epiphany_agent_job_name(binding_id: &str) -> String {
    let suffix = Uuid::new_v4().simple().to_string();
    let short_suffix = &suffix[..8];
    format!("epiphany-{binding_id}-{short_suffix}")
}

fn epiphany_agent_job_output_csv_path(codex_home: &std::path::Path, job_id: &str) -> PathBuf {
    codex_home
        .join("epiphany-agent-jobs")
        .join(format!("{job_id}.csv"))
}

fn sorted_json_object_keys(row_object: &serde_json::Map<String, Value>) -> Vec<String> {
    let mut keys = row_object.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    keys
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

    errors.extend(epiphany_state_replacement_validation_errors(state, update));
    errors
}

fn validate_epiphany_job_bindings(job_bindings: &[EpiphanyJobBinding]) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen_ids = HashSet::<&str>::new();

    for binding in job_bindings {
        require_nonempty_update(&binding.id, "job_binding.id", &mut errors);
        require_nonempty_update(&binding.scope, "job_binding.scope", &mut errors);
        require_nonempty_update(&binding.owner_role, "job_binding.owner_role", &mut errors);
        if let Some(launcher_job_id) = binding.launcher_job_id.as_deref() {
            require_nonempty_update(launcher_job_id, "job_binding.launcher_job_id", &mut errors);
        }
        if let Some(authority_scope) = binding.authority_scope.as_deref() {
            require_nonempty_update(authority_scope, "job_binding.authority_scope", &mut errors);
        }
        if let Some(backend_job_id) = binding.backend_job_id.as_deref() {
            require_nonempty_update(backend_job_id, "job_binding.backend_job_id", &mut errors);
        }

        if let Some(runtime_agent_job_id) = binding.runtime_agent_job_id.as_deref() {
            require_nonempty_update(
                runtime_agent_job_id,
                "job_binding.runtime_agent_job_id",
                &mut errors,
            );
        }
        if let Some(progress_note) = binding.progress_note.as_deref() {
            require_nonempty_update(progress_note, "job_binding.progress_note", &mut errors);
        }
        if let Some(blocking_reason) = binding.blocking_reason.as_deref() {
            require_nonempty_update(blocking_reason, "job_binding.blocking_reason", &mut errors);
        }
        match (binding.backend_kind, binding.backend_job_id.as_deref()) {
            (Some(_), None) => errors.push(format!(
                "job binding {:?} sets backend_kind without backend_job_id",
                binding.id
            )),
            (None, Some(_)) => errors.push(format!(
                "job binding {:?} sets backend_job_id without backend_kind",
                binding.id
            )),
            _ => {}
        }
        if let (
            Some(EpiphanyJobBackendKind::AgentJobs),
            Some(backend_job_id),
            Some(runtime_agent_job_id),
        ) = (
            binding.backend_kind,
            binding.backend_job_id.as_deref(),
            binding.runtime_agent_job_id.as_deref(),
        ) && backend_job_id != runtime_agent_job_id
        {
            errors.push(format!(
                "job binding {:?} sets mismatched agent_jobs ids across backend_job_id and runtime_agent_job_id",
                binding.id
            ));
        }
        if !binding.id.is_empty() && !seen_ids.insert(binding.id.as_str()) {
            errors.push(format!("duplicate job binding id {:?}", binding.id));
        }
    }

    errors
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
    if let Some(churn) = update.churn {
        state.churn = Some(churn);
    }
    if let Some(mode) = update.mode {
        state.mode = Some(mode);
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
    use codex_protocol::protocol::EpiphanyGraph;
    use codex_protocol::protocol::EpiphanyGraphNode;
    use codex_protocol::protocol::EpiphanyJobKind;

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
            launcher_job_id: Some(format!("launcher-{id}")),
            authority_scope: Some("epiphany.specialist".to_string()),
            backend_kind: Some(EpiphanyJobBackendKind::AgentJobs),
            backend_job_id: Some(format!("agent-job-{id}")),
            runtime_agent_job_id: Some(format!("agent-job-{id}")),
            linked_subgoal_ids: vec!["phase-6".to_string()],
            linked_graph_node_ids: vec!["job-surface".to_string()],
            progress_note: Some("Bound to a real runtime agent job.".to_string()),
            blocking_reason: None,
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
            state.job_bindings[0].launcher_job_id.as_deref(),
            Some("launcher-new")
        );
        assert_eq!(
            state.job_bindings[0].authority_scope.as_deref(),
            Some("epiphany.specialist")
        );
        assert_eq!(
            state.job_bindings[0].runtime_agent_job_id.as_deref(),
            Some("agent-job-new")
        );
        assert_eq!(state.last_updated_turn_id.as_deref(), Some("turn-jobs"));
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
                    launcher_job_id: Some(String::new()),
                    authority_scope: Some(String::new()),
                    backend_kind: Some(EpiphanyJobBackendKind::AgentJobs),
                    backend_job_id: Some(String::new()),
                    runtime_agent_job_id: Some(String::new()),
                    linked_subgoal_ids: Vec::new(),
                    linked_graph_node_ids: Vec::new(),
                    progress_note: Some(String::new()),
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
                .any(|error| error.contains("job_binding.runtime_agent_job_id must not be empty"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("job_binding.launcher_job_id must not be empty"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("job_binding.authority_scope must not be empty"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("job_binding.backend_job_id must not be empty"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("job_binding.progress_note must not be empty"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("job_binding.blocking_reason must not be empty"))
        );
    }

    #[test]
    fn validate_epiphany_state_update_rejects_incomplete_or_conflicting_job_backend_bindings() {
        let update = EpiphanyStateUpdate {
            job_bindings: Some(vec![
                EpiphanyJobBinding {
                    backend_kind: Some(EpiphanyJobBackendKind::AgentJobs),
                    backend_job_id: None,
                    ..job_binding("missing-backend-job-id")
                },
                EpiphanyJobBinding {
                    backend_kind: None,
                    backend_job_id: Some("agent-job-orphan".to_string()),
                    runtime_agent_job_id: None,
                    ..job_binding("orphan-backend-job-id")
                },
                EpiphanyJobBinding {
                    backend_kind: Some(EpiphanyJobBackendKind::AgentJobs),
                    backend_job_id: Some("agent-job-backend".to_string()),
                    runtime_agent_job_id: Some("agent-job-legacy".to_string()),
                    ..job_binding("mismatched-ids")
                },
            ]),
            ..Default::default()
        };

        let errors =
            epiphany_state_update_validation_errors(&EpiphanyThreadState::default(), &update);

        assert!(errors.iter().any(|error| {
            error.contains("backend_kind without backend_job_id")
                && error.contains("missing-backend-job-id")
        }));
        assert!(errors.iter().any(|error| {
            error.contains("backend_job_id without backend_kind")
                && error.contains("orphan-backend-job-id")
        }));
        assert!(errors.iter().any(|error| {
            error.contains("mismatched agent_jobs ids") && error.contains("mismatched-ids")
        }));
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
