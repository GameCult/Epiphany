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
use codex_protocol::protocol::EpiphanyChurnState;
use codex_protocol::protocol::EpiphanyEvidenceRecord;
use codex_protocol::protocol::EpiphanyGraphCheckpoint;
use codex_protocol::protocol::EpiphanyGraphFrontier;
use codex_protocol::protocol::EpiphanyGraphs;
use codex_protocol::protocol::EpiphanyInvariant;
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
    pub observations: Vec<EpiphanyObservation>,
    pub evidence: Vec<EpiphanyEvidenceRecord>,
    pub churn: Option<EpiphanyChurnState>,
    pub mode: Option<EpiphanyModeState>,
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

    errors.extend(epiphany_state_replacement_validation_errors(state, update));
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
