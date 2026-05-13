use super::*;
use crate::outgoing_message::OutgoingEnvelope;
use crate::outgoing_message::OutgoingMessage;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use codex_app_server_protocol::ServerRequestPayload;
use codex_app_server_protocol::ToolRequestUserInputParams;
use codex_config::SessionThreadConfig;
use codex_config::StaticThreadConfigLoader;
use codex_config::ThreadConfigSource;
use codex_core::config_loader::CloudRequirementsLoader;
use codex_core::config_loader::LoaderOverrides;
use codex_model_provider_info::ModelProviderInfo;
use codex_model_provider_info::WireApi;
use codex_protocol::ThreadId;
use codex_protocol::openai_models::ReasoningEffort;
use codex_protocol::protocol::AskForApproval;
use codex_protocol::protocol::SandboxPolicy;
use codex_protocol::protocol::SessionSource;
use codex_protocol::protocol::SubAgentSource;
use codex_thread_store::StoredThread;
use codex_utils_absolute_path::test_support::PathBufExt;
use codex_utils_absolute_path::test_support::test_path_buf;
use pretty_assertions::assert_eq;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;

fn core_token_usage_info(
    cumulative_tokens: i64,
    current_context_tokens: i64,
    model_context_window: Option<i64>,
    model_auto_compact_token_limit: Option<i64>,
) -> CoreTokenUsageInfo {
    CoreTokenUsageInfo {
        total_token_usage: codex_protocol::protocol::TokenUsage {
            total_tokens: cumulative_tokens,
            ..Default::default()
        },
        last_token_usage: codex_protocol::protocol::TokenUsage {
            total_tokens: current_context_tokens,
            ..Default::default()
        },
        model_context_window,
        model_auto_compact_token_limit,
    }
}

#[test]
fn validate_dynamic_tools_rejects_unsupported_input_schema() {
    let tools = vec![ApiDynamicToolSpec {
        namespace: None,
        name: "my_tool".to_string(),
        description: "test".to_string(),
        input_schema: json!({"type": "null"}),
        defer_loading: false,
    }];
    let err = validate_dynamic_tools(&tools).expect_err("invalid schema");
    assert!(err.contains("my_tool"), "unexpected error: {err}");
}

#[test]
fn validate_dynamic_tools_accepts_sanitizable_input_schema() {
    let tools = vec![ApiDynamicToolSpec {
        namespace: None,
        name: "my_tool".to_string(),
        description: "test".to_string(),
        // Missing `type` is common; core sanitizes these to a supported schema.
        input_schema: json!({"properties": {}}),
        defer_loading: false,
    }];
    validate_dynamic_tools(&tools).expect("valid schema");
}

#[test]
fn validate_dynamic_tools_accepts_nullable_field_schema() {
    let tools = vec![ApiDynamicToolSpec {
        namespace: None,
        name: "my_tool".to_string(),
        description: "test".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {"type": ["string", "null"]}
            },
            "required": ["query"],
            "additionalProperties": false
        }),
        defer_loading: false,
    }];
    validate_dynamic_tools(&tools).expect("valid schema");
}

#[test]
fn validate_dynamic_tools_accepts_same_name_in_different_namespaces() {
    let tools = vec![
        ApiDynamicToolSpec {
            namespace: Some("codex_app".to_string()),
            name: "my_tool".to_string(),
            description: "test".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            defer_loading: true,
        },
        ApiDynamicToolSpec {
            namespace: Some("other_app".to_string()),
            name: "my_tool".to_string(),
            description: "test".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            defer_loading: true,
        },
    ];
    validate_dynamic_tools(&tools).expect("valid schema");
}

#[test]
fn validate_dynamic_tools_rejects_duplicate_name_in_same_namespace() {
    let tools = vec![
        ApiDynamicToolSpec {
            namespace: Some("codex_app".to_string()),
            name: "my_tool".to_string(),
            description: "test".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            defer_loading: true,
        },
        ApiDynamicToolSpec {
            namespace: Some("codex_app".to_string()),
            name: "my_tool".to_string(),
            description: "test".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            defer_loading: true,
        },
    ];
    let err = validate_dynamic_tools(&tools).expect_err("duplicate name");
    assert!(err.contains("codex_app"), "unexpected error: {err}");
    assert!(err.contains("my_tool"), "unexpected error: {err}");
}

#[test]
fn validate_dynamic_tools_rejects_empty_namespace() {
    let tools = vec![ApiDynamicToolSpec {
        namespace: Some("".to_string()),
        name: "my_tool".to_string(),
        description: "test".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }),
        defer_loading: false,
    }];
    let err = validate_dynamic_tools(&tools).expect_err("empty namespace");
    assert!(err.contains("my_tool"), "unexpected error: {err}");
    assert!(err.contains("namespace"), "unexpected error: {err}");
}

#[test]
fn validate_dynamic_tools_rejects_reserved_namespace() {
    let tools = vec![ApiDynamicToolSpec {
        namespace: Some("mcp__server__".to_string()),
        name: "my_tool".to_string(),
        description: "test".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }),
        defer_loading: false,
    }];
    let err = validate_dynamic_tools(&tools).expect_err("reserved namespace");
    assert!(err.contains("my_tool"), "unexpected error: {err}");
    assert!(err.contains("reserved"), "unexpected error: {err}");
}

#[test]
fn summary_from_stored_thread_preserves_millisecond_precision() {
    let created_at =
        DateTime::parse_from_rfc3339("2025-01-02T03:04:05.678Z").expect("valid timestamp");
    let updated_at =
        DateTime::parse_from_rfc3339("2025-01-02T03:04:06.789Z").expect("valid timestamp");
    let thread_id =
        ThreadId::from_string("00000000-0000-0000-0000-000000000123").expect("valid thread");
    let stored_thread = StoredThread {
        thread_id,
        rollout_path: Some(PathBuf::from("/tmp/thread.jsonl")),
        forked_from_id: None,
        preview: "preview".to_string(),
        name: None,
        model_provider: "openai".to_string(),
        model: None,
        reasoning_effort: None,
        created_at: created_at.with_timezone(&Utc),
        updated_at: updated_at.with_timezone(&Utc),
        archived_at: None,
        cwd: PathBuf::from("/tmp"),
        cli_version: "0.0.0".to_string(),
        source: SessionSource::Cli,
        agent_nickname: None,
        agent_role: None,
        agent_path: None,
        git_info: None,
        approval_mode: AskForApproval::OnRequest,
        sandbox_policy: SandboxPolicy::new_read_only_policy(),
        token_usage: None,
        first_user_message: Some("first user message".to_string()),
        history: None,
    };

    let summary =
        summary_from_stored_thread(stored_thread, "fallback").expect("summary should exist");

    assert_eq!(
        summary.timestamp.as_deref(),
        Some("2025-01-02T03:04:05.678Z")
    );
    assert_eq!(
        summary.updated_at.as_deref(),
        Some("2025-01-02T03:04:06.789Z")
    );
}

#[test]
fn thread_response_permission_profile_omits_external_sandbox() {
    let cwd = test_path_buf("/tmp").abs();
    let profile = codex_protocol::models::PermissionProfile::from_legacy_sandbox_policy(
        &SandboxPolicy::DangerFullAccess,
        cwd.as_path(),
    );

    assert_eq!(
        thread_response_permission_profile(
            &SandboxPolicy::ExternalSandbox {
                network_access: codex_protocol::protocol::NetworkAccess::Restricted,
            },
            profile.clone(),
        ),
        None
    );
    assert_eq!(
        thread_response_permission_profile(&SandboxPolicy::DangerFullAccess, profile.clone()),
        Some(profile.into())
    );
}

#[test]
fn requested_permissions_trust_project_uses_permission_profile_intent() {
    let cwd = test_path_buf("/tmp/project").abs();
    let full_access_profile = codex_protocol::models::PermissionProfile::from_legacy_sandbox_policy(
        &SandboxPolicy::DangerFullAccess,
        cwd.as_path(),
    );
    let workspace_write_profile =
        codex_protocol::models::PermissionProfile::from_legacy_sandbox_policy(
            &SandboxPolicy::new_workspace_write_policy(),
            cwd.as_path(),
        );
    let read_only_profile = codex_protocol::models::PermissionProfile::from_legacy_sandbox_policy(
        &SandboxPolicy::new_read_only_policy(),
        cwd.as_path(),
    );

    assert!(requested_permissions_trust_project(
        &ConfigOverrides {
            permission_profile: Some(full_access_profile),
            ..Default::default()
        },
        cwd.as_path()
    ));
    assert!(requested_permissions_trust_project(
        &ConfigOverrides {
            permission_profile: Some(workspace_write_profile),
            ..Default::default()
        },
        cwd.as_path()
    ));
    assert!(!requested_permissions_trust_project(
        &ConfigOverrides {
            permission_profile: Some(read_only_profile),
            ..Default::default()
        },
        cwd.as_path()
    ));
}

#[test]
fn config_load_error_marks_cloud_requirements_failures_for_relogin() {
    let err = std::io::Error::other(CloudRequirementsLoadError::new(
        CloudRequirementsLoadErrorCode::Auth,
        Some(401),
        "Your authentication session could not be refreshed automatically. Please log out and sign in again.",
    ));

    let error = config_load_error(&err);

    assert_eq!(
        error.data,
        Some(json!({
            "reason": "cloudRequirements",
            "errorCode": "Auth",
            "action": "relogin",
            "statusCode": 401,
            "detail": "Your authentication session could not be refreshed automatically. Please log out and sign in again.",
        }))
    );
    assert!(
        error.message.contains("failed to load configuration"),
        "unexpected error message: {}",
        error.message
    );
}

#[test]
fn config_load_error_leaves_non_cloud_requirements_failures_unmarked() {
    let err = std::io::Error::other("required MCP servers failed to initialize");

    let error = config_load_error(&err);

    assert_eq!(error.data, None);
    assert!(
        error.message.contains("failed to load configuration"),
        "unexpected error message: {}",
        error.message
    );
}

#[test]
fn config_load_error_marks_non_auth_cloud_requirements_failures_without_relogin() {
    let err = std::io::Error::other(CloudRequirementsLoadError::new(
        CloudRequirementsLoadErrorCode::RequestFailed,
        /*status_code*/ None,
        "failed to load your workspace-managed config",
    ));

    let error = config_load_error(&err);

    assert_eq!(
        error.data,
        Some(json!({
            "reason": "cloudRequirements",
            "errorCode": "RequestFailed",
            "detail": "failed to load your workspace-managed config",
        }))
    );
}

#[tokio::test]
async fn derive_config_from_params_uses_session_thread_config_model_provider() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let session_provider = ModelProviderInfo {
        name: "session".to_string(),
        base_url: Some("http://127.0.0.1:8061/api/codex".to_string()),
        env_key: None,
        env_key_instructions: None,
        experimental_bearer_token: None,
        auth: None,
        aws: None,
        wire_api: WireApi::Responses,
        query_params: None,
        http_headers: None,
        env_http_headers: None,
        request_max_retries: None,
        stream_max_retries: None,
        stream_idle_timeout_ms: None,
        websocket_connect_timeout_ms: None,
        requires_openai_auth: false,
        supports_websockets: true,
    };
    let config_manager = ConfigManager::new(
        temp_dir.path().to_path_buf(),
        Vec::new(),
        LoaderOverrides::default(),
        CloudRequirementsLoader::default(),
        Arg0DispatchPaths::default(),
        Arc::new(StaticThreadConfigLoader::new(vec![
            ThreadConfigSource::Session(SessionThreadConfig {
                model_provider: Some("session".to_string()),
                model_providers: HashMap::from([("session".to_string(), session_provider.clone())]),
                features: BTreeMap::from([("plugins".to_string(), false)]),
            }),
        ])),
    );
    let config = config_manager
        .load_with_overrides(
            Some(HashMap::from([
                ("model_provider".to_string(), json!("request")),
                ("features.plugins".to_string(), json!(true)),
                (
                    "model_providers.session".to_string(),
                    json!({
                        "name": "request",
                        "base_url": "http://127.0.0.1:9999/api/codex",
                        "wire_api": "responses",
                    }),
                ),
            ])),
            ConfigOverrides::default(),
        )
        .await?;

    assert_eq!(config.model_provider_id, "session");
    assert_eq!(config.model_provider, session_provider);
    assert!(!config.features.enabled(Feature::Plugins));
    Ok(())
}

#[test]
fn collect_resume_override_mismatches_includes_service_tier() {
    let cwd = test_path_buf("/tmp").abs();
    let request = ThreadResumeParams {
        thread_id: "thread-1".to_string(),
        history: None,
        path: None,
        model: None,
        model_provider: None,
        service_tier: Some(Some(codex_protocol::config_types::ServiceTier::Fast)),
        cwd: None,
        approval_policy: None,
        approvals_reviewer: None,
        sandbox: None,
        permission_profile: None,
        config: None,
        base_instructions: None,
        developer_instructions: None,
        personality: None,
        persist_extended_history: false,
    };
    let config_snapshot = ThreadConfigSnapshot {
        model: "gpt-5".to_string(),
        model_provider_id: "openai".to_string(),
        service_tier: Some(codex_protocol::config_types::ServiceTier::Flex),
        approval_policy: codex_protocol::protocol::AskForApproval::OnRequest,
        approvals_reviewer: codex_protocol::config_types::ApprovalsReviewer::User,
        sandbox_policy: codex_protocol::protocol::SandboxPolicy::DangerFullAccess,
        permission_profile: codex_protocol::models::PermissionProfile::from_legacy_sandbox_policy(
            &codex_protocol::protocol::SandboxPolicy::DangerFullAccess,
            cwd.as_path(),
        ),
        cwd,
        ephemeral: false,
        reasoning_effort: None,
        personality: None,
        session_source: SessionSource::Cli,
    };

    assert_eq!(
        collect_resume_override_mismatches(&request, &config_snapshot),
        vec!["service_tier requested=Some(Fast) active=Some(Flex)".to_string()]
    );
}

fn test_thread_metadata(
    model: Option<&str>,
    reasoning_effort: Option<ReasoningEffort>,
) -> Result<ThreadMetadata> {
    let thread_id = ThreadId::from_string("3f941c35-29b3-493b-b0a4-e25800d9aeb0")?;
    let mut builder = ThreadMetadataBuilder::new(
        thread_id,
        PathBuf::from("/tmp/rollout.jsonl"),
        Utc::now(),
        codex_protocol::protocol::SessionSource::default(),
    );
    builder.model_provider = Some("mock_provider".to_string());
    let mut metadata = builder.build("mock_provider");
    metadata.model = model.map(ToString::to_string);
    metadata.reasoning_effort = reasoning_effort;
    Ok(metadata)
}

#[test]
fn summary_from_thread_metadata_formats_protocol_timestamps_as_seconds() -> Result<()> {
    let mut metadata = test_thread_metadata(/*model*/ None, /*reasoning_effort*/ None)?;
    metadata.created_at =
        DateTime::parse_from_rfc3339("2025-09-05T16:53:11.123Z")?.with_timezone(&Utc);
    metadata.updated_at =
        DateTime::parse_from_rfc3339("2025-09-05T16:53:12.456Z")?.with_timezone(&Utc);

    let summary = summary_from_thread_metadata(&metadata);

    assert_eq!(summary.timestamp, Some("2025-09-05T16:53:11Z".to_string()));
    assert_eq!(summary.updated_at, Some("2025-09-05T16:53:12Z".to_string()));
    Ok(())
}

#[test]
fn merge_persisted_resume_metadata_prefers_persisted_model_and_reasoning_effort() -> Result<()> {
    let mut request_overrides = None;
    let mut typesafe_overrides = ConfigOverrides::default();
    let persisted_metadata =
        test_thread_metadata(Some("gpt-5.1-codex-max"), Some(ReasoningEffort::High))?;

    merge_persisted_resume_metadata(
        &mut request_overrides,
        &mut typesafe_overrides,
        &persisted_metadata,
    );

    assert_eq!(
        typesafe_overrides.model,
        Some("gpt-5.1-codex-max".to_string())
    );
    assert_eq!(
        request_overrides,
        Some(HashMap::from([(
            "model_reasoning_effort".to_string(),
            serde_json::Value::String("high".to_string()),
        )]))
    );
    Ok(())
}

#[test]
fn merge_persisted_resume_metadata_preserves_explicit_overrides() -> Result<()> {
    let mut request_overrides = Some(HashMap::from([(
        "model_reasoning_effort".to_string(),
        serde_json::Value::String("low".to_string()),
    )]));
    let mut typesafe_overrides = ConfigOverrides {
        model: Some("gpt-5.2-codex".to_string()),
        ..Default::default()
    };
    let persisted_metadata =
        test_thread_metadata(Some("gpt-5.1-codex-max"), Some(ReasoningEffort::High))?;

    merge_persisted_resume_metadata(
        &mut request_overrides,
        &mut typesafe_overrides,
        &persisted_metadata,
    );

    assert_eq!(typesafe_overrides.model, Some("gpt-5.2-codex".to_string()));
    assert_eq!(
        request_overrides,
        Some(HashMap::from([(
            "model_reasoning_effort".to_string(),
            serde_json::Value::String("low".to_string()),
        )]))
    );
    Ok(())
}

#[test]
fn merge_persisted_resume_metadata_skips_persisted_values_when_model_overridden() -> Result<()> {
    let mut request_overrides = Some(HashMap::from([(
        "model".to_string(),
        serde_json::Value::String("gpt-5.2-codex".to_string()),
    )]));
    let mut typesafe_overrides = ConfigOverrides::default();
    let persisted_metadata =
        test_thread_metadata(Some("gpt-5.1-codex-max"), Some(ReasoningEffort::High))?;

    merge_persisted_resume_metadata(
        &mut request_overrides,
        &mut typesafe_overrides,
        &persisted_metadata,
    );

    assert_eq!(typesafe_overrides.model, None);
    assert_eq!(
        request_overrides,
        Some(HashMap::from([(
            "model".to_string(),
            serde_json::Value::String("gpt-5.2-codex".to_string()),
        )]))
    );
    Ok(())
}

#[test]
fn merge_persisted_resume_metadata_skips_persisted_values_when_provider_overridden() -> Result<()> {
    let mut request_overrides = None;
    let mut typesafe_overrides = ConfigOverrides {
        model_provider: Some("oss".to_string()),
        ..Default::default()
    };
    let persisted_metadata =
        test_thread_metadata(Some("gpt-5.1-codex-max"), Some(ReasoningEffort::High))?;

    merge_persisted_resume_metadata(
        &mut request_overrides,
        &mut typesafe_overrides,
        &persisted_metadata,
    );

    assert_eq!(typesafe_overrides.model, None);
    assert_eq!(typesafe_overrides.model_provider, Some("oss".to_string()));
    assert_eq!(request_overrides, None);
    Ok(())
}

#[test]
fn merge_persisted_resume_metadata_skips_persisted_values_when_reasoning_effort_overridden()
-> Result<()> {
    let mut request_overrides = Some(HashMap::from([(
        "model_reasoning_effort".to_string(),
        serde_json::Value::String("low".to_string()),
    )]));
    let mut typesafe_overrides = ConfigOverrides::default();
    let persisted_metadata =
        test_thread_metadata(Some("gpt-5.1-codex-max"), Some(ReasoningEffort::High))?;

    merge_persisted_resume_metadata(
        &mut request_overrides,
        &mut typesafe_overrides,
        &persisted_metadata,
    );

    assert_eq!(typesafe_overrides.model, None);
    assert_eq!(
        request_overrides,
        Some(HashMap::from([(
            "model_reasoning_effort".to_string(),
            serde_json::Value::String("low".to_string()),
        )]))
    );
    Ok(())
}

#[test]
fn merge_persisted_resume_metadata_skips_missing_values() -> Result<()> {
    let mut request_overrides = None;
    let mut typesafe_overrides = ConfigOverrides::default();
    let persisted_metadata =
        test_thread_metadata(/*model*/ None, /*reasoning_effort*/ None)?;

    merge_persisted_resume_metadata(
        &mut request_overrides,
        &mut typesafe_overrides,
        &persisted_metadata,
    );

    assert_eq!(typesafe_overrides.model, None);
    assert_eq!(request_overrides, None);
    Ok(())
}

#[test]
fn extract_conversation_summary_prefers_plain_user_messages() -> Result<()> {
    let conversation_id = ThreadId::from_string("3f941c35-29b3-493b-b0a4-e25800d9aeb0")?;
    let timestamp = Some("2025-09-05T16:53:11.850Z".to_string());
    let path = PathBuf::from("rollout.jsonl");

    let head = vec![
        json!({
            "id": conversation_id.to_string(),
            "timestamp": timestamp,
            "cwd": "/",
            "originator": "codex",
            "cli_version": "0.0.0",
            "model_provider": "test-provider"
        }),
        json!({
            "type": "message",
            "role": "user",
            "content": [{
                "type": "input_text",
                "text": "# AGENTS.md instructions for project\n\n<INSTRUCTIONS>\n<AGENTS.md contents>\n</INSTRUCTIONS>".to_string(),
            }],
        }),
        json!({
            "type": "message",
            "role": "user",
            "content": [{
                "type": "input_text",
                "text": format!("<prior context> {USER_MESSAGE_BEGIN}Count to 5"),
            }],
        }),
    ];

    let session_meta = serde_json::from_value::<SessionMeta>(head[0].clone())?;

    let summary = extract_conversation_summary(
        path.clone(),
        &head,
        &session_meta,
        /*git*/ None,
        "test-provider",
        timestamp.clone(),
    )
    .expect("summary");

    let expected = ConversationSummary {
        conversation_id,
        timestamp: timestamp.clone(),
        updated_at: timestamp,
        path,
        preview: "Count to 5".to_string(),
        model_provider: "test-provider".to_string(),
        cwd: PathBuf::from("/"),
        cli_version: "0.0.0".to_string(),
        source: SessionSource::VSCode,
        git_info: None,
    };

    assert_eq!(summary, expected);
    Ok(())
}

#[tokio::test]
async fn read_summary_from_rollout_returns_empty_preview_when_no_user_message() -> Result<()> {
    use codex_protocol::protocol::RolloutItem;
    use codex_protocol::protocol::RolloutLine;
    use codex_protocol::protocol::SessionMetaLine;
    use std::fs;
    use std::fs::FileTimes;

    let temp_dir = TempDir::new()?;
    let path = temp_dir.path().join("rollout.jsonl");

    let conversation_id = ThreadId::from_string("bfd12a78-5900-467b-9bc5-d3d35df08191")?;
    let timestamp = "2025-09-05T16:53:11.850Z".to_string();

    let session_meta = SessionMeta {
        id: conversation_id,
        timestamp: timestamp.clone(),
        model_provider: None,
        ..SessionMeta::default()
    };

    let line = RolloutLine {
        timestamp: timestamp.clone(),
        item: RolloutItem::SessionMeta(SessionMetaLine {
            meta: session_meta.clone(),
            git: None,
        }),
    };

    fs::write(&path, format!("{}\n", serde_json::to_string(&line)?))?;
    let parsed = chrono::DateTime::parse_from_rfc3339(&timestamp)?.with_timezone(&Utc);
    let times = FileTimes::new().set_modified(parsed.into());
    std::fs::OpenOptions::new()
        .append(true)
        .open(&path)?
        .set_times(times)?;

    let summary = read_summary_from_rollout(path.as_path(), "fallback").await?;

    let expected = ConversationSummary {
        conversation_id,
        timestamp: Some(timestamp.clone()),
        updated_at: Some(timestamp),
        path: path.clone(),
        preview: String::new(),
        model_provider: "fallback".to_string(),
        cwd: PathBuf::new(),
        cli_version: String::new(),
        source: SessionSource::VSCode,
        git_info: None,
    };

    assert_eq!(summary, expected);
    Ok(())
}

#[tokio::test]
async fn read_summary_from_rollout_preserves_agent_nickname() -> Result<()> {
    use codex_protocol::protocol::RolloutItem;
    use codex_protocol::protocol::RolloutLine;
    use codex_protocol::protocol::SessionMetaLine;
    use std::fs;

    let temp_dir = TempDir::new()?;
    let path = temp_dir.path().join("rollout.jsonl");

    let conversation_id = ThreadId::from_string("bfd12a78-5900-467b-9bc5-d3d35df08191")?;
    let parent_thread_id = ThreadId::from_string("ad7f0408-99b8-4f6e-a46f-bd0eec433370")?;
    let timestamp = "2025-09-05T16:53:11.850Z".to_string();

    let session_meta = SessionMeta {
        id: conversation_id,
        timestamp: timestamp.clone(),
        source: SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
            parent_thread_id,
            depth: 1,
            agent_path: None,
            agent_nickname: None,
            agent_role: None,
        }),
        agent_nickname: Some("atlas".to_string()),
        agent_role: Some("explorer".to_string()),
        model_provider: Some("test-provider".to_string()),
        ..SessionMeta::default()
    };

    let line = RolloutLine {
        timestamp,
        item: RolloutItem::SessionMeta(SessionMetaLine {
            meta: session_meta,
            git: None,
        }),
    };
    fs::write(&path, format!("{}\n", serde_json::to_string(&line)?))?;

    let summary = read_summary_from_rollout(path.as_path(), "fallback").await?;
    let fallback_cwd = AbsolutePathBuf::from_absolute_path("/")?;
    let thread = summary_to_thread(summary, &fallback_cwd);

    assert_eq!(thread.agent_nickname, Some("atlas".to_string()));
    assert_eq!(thread.agent_role, Some("explorer".to_string()));
    Ok(())
}

#[tokio::test]
async fn read_summary_from_rollout_preserves_forked_from_id() -> Result<()> {
    use codex_protocol::protocol::RolloutItem;
    use codex_protocol::protocol::RolloutLine;
    use codex_protocol::protocol::SessionMetaLine;
    use std::fs;

    let temp_dir = TempDir::new()?;
    let path = temp_dir.path().join("rollout.jsonl");

    let conversation_id = ThreadId::from_string("bfd12a78-5900-467b-9bc5-d3d35df08191")?;
    let forked_from_id = ThreadId::from_string("ad7f0408-99b8-4f6e-a46f-bd0eec433370")?;
    let timestamp = "2025-09-05T16:53:11.850Z".to_string();

    let session_meta = SessionMeta {
        id: conversation_id,
        forked_from_id: Some(forked_from_id),
        timestamp: timestamp.clone(),
        model_provider: Some("test-provider".to_string()),
        ..SessionMeta::default()
    };

    let line = RolloutLine {
        timestamp,
        item: RolloutItem::SessionMeta(SessionMetaLine {
            meta: session_meta,
            git: None,
        }),
    };
    fs::write(&path, format!("{}\n", serde_json::to_string(&line)?))?;

    assert_eq!(
        forked_from_id_from_rollout(path.as_path()).await,
        Some(forked_from_id.to_string())
    );
    Ok(())
}

#[tokio::test]
async fn load_epiphany_state_from_rollout_path_reads_latest_snapshot() -> Result<()> {
    use codex_protocol::protocol::EpiphanyStateItem;
    use codex_protocol::protocol::EpiphanyThreadState;
    use codex_protocol::protocol::EventMsg;
    use codex_protocol::protocol::RolloutItem;
    use codex_protocol::protocol::RolloutLine;
    use codex_protocol::protocol::SessionMetaLine;
    use codex_protocol::protocol::TurnCompleteEvent;
    use codex_protocol::protocol::TurnStartedEvent;
    use codex_protocol::protocol::UserMessageEvent;
    use std::fs;

    let temp_dir = TempDir::new()?;
    let path = temp_dir.path().join("rollout.jsonl");

    let conversation_id = ThreadId::from_string("bfd12a78-5900-467b-9bc5-d3d35df08191")?;
    let timestamp = "2025-09-05T16:53:11.850Z".to_string();
    let epiphany_state = EpiphanyThreadState {
        revision: 7,
        objective: Some("Expose thread state to clients".to_string()),
        active_subgoal_id: Some("phase-3".to_string()),
        last_updated_turn_id: Some("turn-2".to_string()),
        ..Default::default()
    };

    let rollout_lines = vec![
        RolloutLine {
            timestamp: timestamp.clone(),
            item: RolloutItem::SessionMeta(SessionMetaLine {
                meta: SessionMeta {
                    id: conversation_id,
                    timestamp: timestamp.clone(),
                    model_provider: Some("test-provider".to_string()),
                    ..SessionMeta::default()
                },
                git: None,
            }),
        },
        RolloutLine {
            timestamp: timestamp.clone(),
            item: RolloutItem::EventMsg(EventMsg::TurnStarted(TurnStartedEvent {
                turn_id: "turn-2".to_string(),
                started_at: None,
                model_context_window: None,
                collaboration_mode_kind: Default::default(),
            })),
        },
        RolloutLine {
            timestamp: timestamp.clone(),
            item: RolloutItem::EventMsg(EventMsg::UserMessage(UserMessageEvent {
                message: "load the scene".to_string(),
                images: None,
                text_elements: Vec::new(),
                local_images: Vec::new(),
            })),
        },
        RolloutLine {
            timestamp: timestamp.clone(),
            item: RolloutItem::EpiphanyState(EpiphanyStateItem {
                turn_id: Some("turn-2".to_string()),
                state: epiphany_state.clone(),
            }),
        },
        RolloutLine {
            timestamp,
            item: RolloutItem::EventMsg(EventMsg::TurnComplete(TurnCompleteEvent {
                turn_id: "turn-2".to_string(),
                last_agent_message: None,
                completed_at: None,
                duration_ms: None,
                time_to_first_token_ms: None,
            })),
        },
    ];

    let encoded = rollout_lines
        .iter()
        .map(serde_json::to_string)
        .collect::<std::result::Result<Vec<_>, _>>()?
        .join("\n");
    fs::write(&path, format!("{encoded}\n"))?;

    let loaded = load_epiphany_state_from_rollout_path(path.as_path())
        .await
        .map_err(anyhow::Error::msg)?;
    assert_eq!(loaded, Some(epiphany_state));
    Ok(())
}

#[test]
fn map_epiphany_retrieve_response_preserves_summary_and_results() -> Result<()> {
    let response = map_epiphany_retrieve_response(codex_core::EpiphanyRetrieveResponse {
        query: "checkpoint frontier".to_string(),
        index_summary: codex_protocol::protocol::EpiphanyRetrievalState {
            workspace_root: test_path_buf("/repo"),
            index_revision: Some("query-time-bm25-v1".to_string()),
            status: codex_protocol::protocol::EpiphanyRetrievalStatus::Ready,
            semantic_available: true,
            last_indexed_at_unix_seconds: Some(1_744_500_000),
            indexed_file_count: Some(12),
            indexed_chunk_count: Some(34),
            shards: vec![codex_protocol::protocol::EpiphanyRetrievalShardSummary {
                shard_id: "workspace".to_string(),
                path_prefix: PathBuf::from("."),
                indexed_file_count: Some(12),
                indexed_chunk_count: Some(34),
                status: codex_protocol::protocol::EpiphanyRetrievalStatus::Ready,
                exact_available: true,
                semantic_available: true,
            }],
            dirty_paths: vec![PathBuf::from("src/session/mod.rs")],
        },
        results: vec![codex_core::EpiphanyRetrieveResult {
            kind: codex_core::EpiphanyRetrieveResultKind::SemanticChunk,
            path: PathBuf::from("notes/design.md"),
            score: 2.5,
            line_start: Some(3),
            line_end: Some(9),
            excerpt: Some("checkpoint frontier".to_string()),
        }],
    })?;

    assert_eq!(response.query, "checkpoint frontier");
    assert_eq!(
        response.index_summary.workspace_root,
        test_path_buf("/repo").abs()
    );
    assert_eq!(response.index_summary.indexed_chunk_count, Some(34));
    assert_eq!(response.results.len(), 1);
    assert_eq!(
        response.results[0].kind,
        ThreadEpiphanyRetrieveResultKind::SemanticChunk
    );
    assert_eq!(response.results[0].path, PathBuf::from("notes/design.md"));
    Ok(())
}

#[test]
fn map_epiphany_retrieve_index_summary_preserves_ready_qdrant_summary() -> Result<()> {
    let summary =
        map_epiphany_retrieve_index_summary(codex_protocol::protocol::EpiphanyRetrievalState {
            workspace_root: test_path_buf("/repo"),
            index_revision: Some("qdrant-ollama-v1:qwen3-embedding:0.6b".to_string()),
            status: codex_protocol::protocol::EpiphanyRetrievalStatus::Ready,
            semantic_available: true,
            last_indexed_at_unix_seconds: Some(1_744_500_100),
            indexed_file_count: Some(12),
            indexed_chunk_count: Some(34),
            shards: vec![codex_protocol::protocol::EpiphanyRetrievalShardSummary {
                shard_id: "workspace".to_string(),
                path_prefix: PathBuf::from("."),
                indexed_file_count: Some(12),
                indexed_chunk_count: Some(34),
                status: codex_protocol::protocol::EpiphanyRetrievalStatus::Ready,
                exact_available: true,
                semantic_available: true,
            }],
            dirty_paths: Vec::new(),
        })?;

    assert_eq!(
        summary.index_revision.as_deref(),
        Some("qdrant-ollama-v1:qwen3-embedding:0.6b")
    );
    assert_eq!(summary.indexed_file_count, Some(12));
    assert!(summary.dirty_paths.is_empty());
    Ok(())
}

#[test]
fn map_epiphany_scene_projects_client_reflection_without_mutation_shape() {
    let scene = map_epiphany_scene(
        Some(&codex_protocol::protocol::EpiphanyThreadState {
            revision: 5,
            objective: Some("Expose typed state without making a second brain".to_string()),
            active_subgoal_id: Some("phase-6".to_string()),
            subgoals: vec![
                codex_protocol::protocol::EpiphanySubgoal {
                    id: "phase-5".to_string(),
                    title: "Finish control plane".to_string(),
                    status: "done".to_string(),
                    summary: None,
                },
                codex_protocol::protocol::EpiphanySubgoal {
                    id: "phase-6".to_string(),
                    title: "Reflect typed state".to_string(),
                    status: "active".to_string(),
                    summary: Some("Thin client scene".to_string()),
                },
            ],
            invariants: vec![
                codex_protocol::protocol::EpiphanyInvariant {
                    id: "inv-1".to_string(),
                    description: "No GUI source of truth".to_string(),
                    status: "ok".to_string(),
                    rationale: None,
                },
                codex_protocol::protocol::EpiphanyInvariant {
                    id: "inv-2".to_string(),
                    description: "No hidden mutation".to_string(),
                    status: "ok".to_string(),
                    rationale: None,
                },
            ],
            graphs: codex_protocol::protocol::EpiphanyGraphs {
                architecture: codex_protocol::protocol::EpiphanyGraph {
                    nodes: vec![codex_protocol::protocol::EpiphanyGraphNode {
                        id: "scene".to_string(),
                        title: "Scene projection".to_string(),
                        purpose: "Reflect the typed state".to_string(),
                        ..Default::default()
                    }],
                    edges: Vec::new(),
                },
                dataflow: codex_protocol::protocol::EpiphanyGraph {
                    nodes: vec![codex_protocol::protocol::EpiphanyGraphNode {
                        id: "state".to_string(),
                        title: "State".to_string(),
                        purpose: "Authoritative input".to_string(),
                        ..Default::default()
                    }],
                    edges: Vec::new(),
                },
                links: vec![codex_protocol::protocol::EpiphanyGraphLink {
                    dataflow_node_id: "state".to_string(),
                    architecture_node_id: "scene".to_string(),
                    relationship: Some("derived".to_string()),
                    code_refs: Vec::new(),
                }],
            },
            graph_frontier: Some(codex_protocol::protocol::EpiphanyGraphFrontier {
                active_node_ids: vec!["scene".to_string()],
                active_edge_ids: Vec::new(),
                open_question_ids: vec!["q-1".to_string()],
                open_gap_ids: Vec::new(),
                dirty_paths: vec![PathBuf::from("app-server/src/codex_message_processor.rs")],
            }),
            graph_checkpoint: Some(codex_protocol::protocol::EpiphanyGraphCheckpoint {
                checkpoint_id: "ck-5".to_string(),
                graph_revision: 5,
                summary: Some("Phase 6 start".to_string()),
                frontier_node_ids: Vec::new(),
                open_question_ids: Vec::new(),
                open_gap_ids: Vec::new(),
            }),
            retrieval: Some(codex_protocol::protocol::EpiphanyRetrievalState {
                workspace_root: test_path_buf("/repo"),
                status: codex_protocol::protocol::EpiphanyRetrievalStatus::Ready,
                semantic_available: true,
                indexed_file_count: Some(2),
                indexed_chunk_count: Some(8),
                shards: vec![codex_protocol::protocol::EpiphanyRetrievalShardSummary {
                    shard_id: "workspace".to_string(),
                    path_prefix: PathBuf::from("."),
                    status: codex_protocol::protocol::EpiphanyRetrievalStatus::Ready,
                    exact_available: true,
                    semantic_available: true,
                    ..Default::default()
                }],
                ..Default::default()
            }),
            investigation_checkpoint: Some(
                codex_protocol::protocol::EpiphanyInvestigationCheckpoint {
                    checkpoint_id: "ix-5".to_string(),
                    kind: "slice_planning".to_string(),
                    disposition:
                        codex_protocol::protocol::EpiphanyInvestigationDisposition::ResumeReady,
                    focus: "Keep the durable planning packet visible.".to_string(),
                    summary: Some("Checkpointed the next bounded slice.".to_string()),
                    next_action: Some("Patch the typed state writer next.".to_string()),
                    captured_at_turn_id: Some("turn-5".to_string()),
                    open_questions: vec!["Should scene and context both surface this?".to_string()],
                    evidence_ids: vec!["ev-scene".to_string()],
                    code_refs: vec![codex_protocol::protocol::EpiphanyCodeRef {
                        path: test_path_buf("/repo/app-server/src/codex_message_processor.rs"),
                        start_line: Some(10692),
                        end_line: Some(10811),
                        symbol: Some("map_epiphany_scene".to_string()),
                        note: None,
                    }],
                },
            ),
            observations: vec![codex_protocol::protocol::EpiphanyObservation {
                id: "obs-scene".to_string(),
                summary: "Scene is derived".to_string(),
                source_kind: "test".to_string(),
                status: "ok".to_string(),
                code_refs: Vec::new(),
                evidence_ids: vec!["ev-scene".to_string()],
            }],
            recent_evidence: vec![codex_protocol::protocol::EpiphanyEvidenceRecord {
                id: "ev-scene".to_string(),
                kind: "test".to_string(),
                status: "ok".to_string(),
                summary: "Projection test".to_string(),
                code_refs: Vec::new(),
            }],
            churn: Some(codex_protocol::protocol::EpiphanyChurnState {
                understanding_status: "ready".to_string(),
                diff_pressure: "low".to_string(),
                graph_freshness: Some("fresh".to_string()),
                warning: None,
                unexplained_writes: Some(0),
            }),
            ..Default::default()
        }),
        true,
    );

    assert_eq!(scene.state_status, ThreadEpiphanySceneStateStatus::Ready);
    assert_eq!(scene.source, ThreadEpiphanySceneSource::Live);
    assert_eq!(scene.revision, Some(5));
    assert_eq!(
        scene
            .active_subgoal
            .as_ref()
            .map(|subgoal| subgoal.id.as_str()),
        Some("phase-6")
    );
    assert_eq!(
        scene.invariant_status_counts,
        vec![ThreadEpiphanySceneStatusCount {
            status: "ok".to_string(),
            count: 2,
        }]
    );
    assert_eq!(scene.graph.architecture_node_count, 1);
    assert_eq!(scene.graph.dataflow_node_count, 1);
    assert_eq!(scene.graph.link_count, 1);
    assert_eq!(scene.graph.checkpoint_id.as_deref(), Some("ck-5"));
    assert_eq!(
        scene.retrieval.as_ref().and_then(|r| r.indexed_chunk_count),
        Some(8)
    );
    assert_eq!(
        scene
            .investigation_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_id.as_str()),
        Some("ix-5")
    );
    assert_eq!(scene.observations.total_count, 1);
    assert_eq!(scene.evidence.latest[0].id, "ev-scene");
    assert_eq!(
        scene.available_actions,
        vec![
            ThreadEpiphanySceneAction::Index,
            ThreadEpiphanySceneAction::Retrieve,
            ThreadEpiphanySceneAction::Distill,
            ThreadEpiphanySceneAction::Context,
            ThreadEpiphanySceneAction::Planning,
            ThreadEpiphanySceneAction::GraphQuery,
            ThreadEpiphanySceneAction::Jobs,
            ThreadEpiphanySceneAction::Roles,
            ThreadEpiphanySceneAction::Coordinator,
            ThreadEpiphanySceneAction::RoleLaunch,
            ThreadEpiphanySceneAction::RoleResult,
            ThreadEpiphanySceneAction::RoleAccept,
            ThreadEpiphanySceneAction::JobLaunch,
            ThreadEpiphanySceneAction::Freshness,
            ThreadEpiphanySceneAction::Pressure,
            ThreadEpiphanySceneAction::Reorient,
            ThreadEpiphanySceneAction::Crrc,
            ThreadEpiphanySceneAction::ReorientLaunch,
            ThreadEpiphanySceneAction::Update,
            ThreadEpiphanySceneAction::JobInterrupt,
            ThreadEpiphanySceneAction::Propose,
            ThreadEpiphanySceneAction::Promote,
        ]
    );
}

#[test]
fn map_epiphany_scene_handles_missing_state_without_write_actions() {
    let scene = map_epiphany_scene(None, false);

    assert_eq!(scene.state_status, ThreadEpiphanySceneStateStatus::Missing);
    assert_eq!(scene.source, ThreadEpiphanySceneSource::Stored);
    assert!(scene.available_actions.is_empty());
    assert_eq!(scene.graph, ThreadEpiphanySceneGraph::default());
}

#[test]
fn map_epiphany_freshness_can_reflect_live_retrieval_without_state() {
    let retrieval = EpiphanyRetrievalState {
        workspace_root: PathBuf::from("/workspace"),
        status: EpiphanyRetrievalStatus::Ready,
        semantic_available: true,
        ..Default::default()
    };

    let (state_revision, retrieval, graph, watcher) =
        map_epiphany_freshness(None, Some(&retrieval), None);

    assert_eq!(state_revision, None);
    assert_eq!(
        retrieval.status,
        ThreadEpiphanyRetrievalFreshnessStatus::Ready
    );
    assert_eq!(retrieval.semantic_available, Some(true));
    assert_eq!(graph.status, ThreadEpiphanyGraphFreshnessStatus::Missing);
    assert_eq!(graph.dirty_path_count, 0);
    assert_eq!(
        watcher.status,
        ThreadEpiphanyInvalidationStatus::Unavailable
    );
}

#[test]
fn map_epiphany_freshness_marks_graph_and_retrieval_stale() {
    let state = EpiphanyThreadState {
        revision: 7,
        graphs: codex_protocol::protocol::EpiphanyGraphs {
            architecture: codex_protocol::protocol::EpiphanyGraph {
                nodes: vec![codex_protocol::protocol::EpiphanyGraphNode {
                    id: "state-spine".to_string(),
                    title: "State spine".to_string(),
                    purpose: "Typed state".to_string(),
                    code_refs: vec![codex_protocol::protocol::EpiphanyCodeRef {
                        path: test_path_buf("/workspace/src/router.rs"),
                        start_line: None,
                        end_line: None,
                        symbol: None,
                        note: None,
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            },
            ..Default::default()
        },
        graph_frontier: Some(codex_protocol::protocol::EpiphanyGraphFrontier {
            active_node_ids: vec!["state-spine".to_string()],
            dirty_paths: vec![PathBuf::from("src/router.rs")],
            open_question_ids: vec!["q-stale".to_string()],
            ..Default::default()
        }),
        graph_checkpoint: Some(codex_protocol::protocol::EpiphanyGraphCheckpoint {
            checkpoint_id: "ck-7".to_string(),
            graph_revision: 7,
            ..Default::default()
        }),
        churn: Some(codex_protocol::protocol::EpiphanyChurnState {
            understanding_status: "ready".to_string(),
            diff_pressure: "low".to_string(),
            graph_freshness: Some("stale".to_string()),
            warning: None,
            unexplained_writes: Some(0),
        }),
        ..Default::default()
    };
    let retrieval = EpiphanyRetrievalState {
        workspace_root: PathBuf::from("/workspace"),
        status: EpiphanyRetrievalStatus::Stale,
        semantic_available: true,
        indexed_file_count: Some(12),
        indexed_chunk_count: Some(48),
        dirty_paths: vec![PathBuf::from("src/router.rs")],
        ..Default::default()
    };
    let watcher_snapshot = EpiphanyInvalidationSnapshot {
        available: true,
        workspace_root: Some(test_path_buf("/workspace").abs()),
        observed_at_unix_seconds: Some(1_744_600_000),
        changed_paths: vec![PathBuf::from("src/router.rs")],
    };

    let (state_revision, retrieval, graph, watcher) =
        map_epiphany_freshness(Some(&state), Some(&retrieval), Some(&watcher_snapshot));

    assert_eq!(state_revision, Some(7));
    assert_eq!(
        retrieval.status,
        ThreadEpiphanyRetrievalFreshnessStatus::Stale
    );
    assert_eq!(retrieval.dirty_paths, vec![PathBuf::from("src/router.rs")]);
    assert_eq!(graph.status, ThreadEpiphanyGraphFreshnessStatus::Stale);
    assert_eq!(graph.checkpoint_id.as_deref(), Some("ck-7"));
    assert_eq!(graph.dirty_path_count, 1);
    assert_eq!(graph.open_question_count, 1);
    assert_eq!(graph.open_gap_count, 0);
    assert_eq!(watcher.status, ThreadEpiphanyInvalidationStatus::Changed);
    assert_eq!(
        watcher.watched_root,
        Some(test_path_buf("/workspace").abs().to_path_buf())
    );
    assert_eq!(watcher.changed_paths, vec![PathBuf::from("src/router.rs")]);
    assert_eq!(watcher.graph_node_ids, vec!["state-spine".to_string()]);
    assert_eq!(
        watcher.active_frontier_node_ids,
        vec!["state-spine".to_string()]
    );
}

#[test]
fn map_epiphany_freshness_reports_clean_watcher_when_no_changes_were_seen() {
    let watcher_snapshot = EpiphanyInvalidationSnapshot {
        available: true,
        workspace_root: Some(test_path_buf("/workspace").abs()),
        observed_at_unix_seconds: None,
        changed_paths: Vec::new(),
    };

    let (_, _, _, watcher) = map_epiphany_freshness(None, None, Some(&watcher_snapshot));

    assert_eq!(watcher.status, ThreadEpiphanyInvalidationStatus::Clean);
    assert_eq!(watcher.changed_path_count, 0);
    assert_eq!(
        watcher.watched_root,
        Some(test_path_buf("/workspace").abs().to_path_buf())
    );
}

#[test]
fn map_epiphany_freshness_reports_unmatched_changed_paths_without_inventing_nodes() {
    let state = EpiphanyThreadState {
        graphs: codex_protocol::protocol::EpiphanyGraphs {
            architecture: codex_protocol::protocol::EpiphanyGraph {
                nodes: vec![codex_protocol::protocol::EpiphanyGraphNode {
                    id: "scene".to_string(),
                    title: "Scene".to_string(),
                    purpose: "Reflection".to_string(),
                    code_refs: vec![codex_protocol::protocol::EpiphanyCodeRef {
                        path: test_path_buf("/workspace/src/scene.rs"),
                        start_line: None,
                        end_line: None,
                        symbol: None,
                        note: None,
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };
    let watcher_snapshot = EpiphanyInvalidationSnapshot {
        available: true,
        workspace_root: Some(test_path_buf("/workspace").abs()),
        observed_at_unix_seconds: Some(1_744_600_001),
        changed_paths: vec![PathBuf::from("src/other.rs")],
    };

    let (_, _, _, watcher) = map_epiphany_freshness(Some(&state), None, Some(&watcher_snapshot));

    assert_eq!(watcher.status, ThreadEpiphanyInvalidationStatus::Changed);
    assert!(watcher.graph_node_ids.is_empty());
    assert!(watcher.active_frontier_node_ids.is_empty());
}

#[test]
fn pre_compaction_checkpoint_intervention_uses_compaction_prep_threshold() {
    let elevated = map_epiphany_pressure(Some(&core_token_usage_info(1_000, 79, None, Some(100))));
    let high = map_epiphany_pressure(Some(&core_token_usage_info(1_000, 80, None, Some(100))));

    assert!(!should_run_epiphany_pre_compaction_checkpoint_intervention(
        &elevated
    ));
    assert!(should_run_epiphany_pre_compaction_checkpoint_intervention(
        &high
    ));

    let prompt = render_epiphany_pre_compaction_checkpoint_intervention(&high);
    assert!(prompt.contains("pre-compaction checkpoint intervention"));
    assert!(prompt.contains("bank the active working context"));
    assert!(prompt.contains("do not continue implementation"));
}

#[test]
fn map_epiphany_reorient_regathers_without_state() {
    let pressure = map_epiphany_pressure(None);
    let retrieval = ThreadEpiphanyRetrievalFreshness {
        status: ThreadEpiphanyRetrievalFreshnessStatus::Missing,
        semantic_available: None,
        last_indexed_at_unix_seconds: None,
        indexed_file_count: None,
        indexed_chunk_count: None,
        dirty_paths: Vec::new(),
        note: "missing".to_string(),
    };
    let graph = ThreadEpiphanyGraphFreshness {
        status: ThreadEpiphanyGraphFreshnessStatus::Missing,
        graph_freshness: None,
        checkpoint_id: None,
        dirty_path_count: 0,
        dirty_paths: Vec::new(),
        open_question_count: 0,
        open_gap_count: 0,
        note: "missing".to_string(),
    };
    let watcher = ThreadEpiphanyInvalidationInput {
        status: ThreadEpiphanyInvalidationStatus::Unavailable,
        watched_root: None,
        observed_at_unix_seconds: None,
        changed_path_count: 0,
        changed_paths: Vec::new(),
        graph_node_ids: Vec::new(),
        active_frontier_node_ids: Vec::new(),
        note: "missing".to_string(),
    };

    let (state_status, decision) =
        map_epiphany_reorient(None, &pressure, &retrieval, &graph, &watcher);

    assert_eq!(state_status, ThreadEpiphanyReorientStateStatus::Missing);
    assert_eq!(decision.action, ThreadEpiphanyReorientAction::Regather);
    assert_eq!(
        decision.checkpoint_status,
        ThreadEpiphanyReorientCheckpointStatus::Missing
    );
    assert_eq!(
        decision.reasons,
        vec![
            ThreadEpiphanyReorientReason::MissingState,
            ThreadEpiphanyReorientReason::MissingCheckpoint,
        ]
    );
    assert!(decision.note.contains("No Epiphany state survived"));
}

#[test]
fn map_epiphany_reorient_resumes_when_checkpoint_is_still_aligned() {
    let state = codex_protocol::protocol::EpiphanyThreadState {
        revision: 7,
        investigation_checkpoint: Some(codex_protocol::protocol::EpiphanyInvestigationCheckpoint {
            checkpoint_id: "ix-resume".to_string(),
            kind: "source_gathering".to_string(),
            disposition: codex_protocol::protocol::EpiphanyInvestigationDisposition::ResumeReady,
            focus: "Keep the seam visible.".to_string(),
            next_action: Some("Resume from the durable checkpoint.".to_string()),
            code_refs: vec![codex_protocol::protocol::EpiphanyCodeRef {
                path: PathBuf::from("src/lib.rs"),
                start_line: Some(1),
                end_line: Some(10),
                symbol: Some("resume_target".to_string()),
                note: None,
            }],
            ..Default::default()
        }),
        ..Default::default()
    };
    let pressure = map_epiphany_pressure(Some(&core_token_usage_info(
        1_000,
        40,
        Some(200),
        Some(100),
    )));
    let retrieval = ThreadEpiphanyRetrievalFreshness {
        status: ThreadEpiphanyRetrievalFreshnessStatus::Ready,
        semantic_available: Some(true),
        last_indexed_at_unix_seconds: Some(1_744_500_000),
        indexed_file_count: Some(12),
        indexed_chunk_count: Some(34),
        dirty_paths: Vec::new(),
        note: "ready".to_string(),
    };
    let graph = ThreadEpiphanyGraphFreshness {
        status: ThreadEpiphanyGraphFreshnessStatus::Ready,
        graph_freshness: Some("fresh".to_string()),
        checkpoint_id: Some("ck-1".to_string()),
        dirty_path_count: 0,
        dirty_paths: Vec::new(),
        open_question_count: 0,
        open_gap_count: 0,
        note: "ready".to_string(),
    };
    let watcher = ThreadEpiphanyInvalidationInput {
        status: ThreadEpiphanyInvalidationStatus::Clean,
        watched_root: Some(test_path_buf("/workspace")),
        observed_at_unix_seconds: Some(1_744_600_000),
        changed_path_count: 0,
        changed_paths: Vec::new(),
        graph_node_ids: Vec::new(),
        active_frontier_node_ids: Vec::new(),
        note: "clean".to_string(),
    };

    let (state_status, decision) =
        map_epiphany_reorient(Some(&state), &pressure, &retrieval, &graph, &watcher);

    assert_eq!(state_status, ThreadEpiphanyReorientStateStatus::Ready);
    assert_eq!(decision.action, ThreadEpiphanyReorientAction::Resume);
    assert_eq!(
        decision.checkpoint_status,
        ThreadEpiphanyReorientCheckpointStatus::ResumeReady
    );
    assert_eq!(
        decision.reasons,
        vec![ThreadEpiphanyReorientReason::CheckpointReady]
    );
    assert_eq!(decision.checkpoint_id.as_deref(), Some("ix-resume"));
    assert_eq!(
        decision.next_action,
        "Resume from the durable checkpoint.".to_string()
    );
    assert!(decision.checkpoint_dirty_paths.is_empty());
    assert!(decision.checkpoint_changed_paths.is_empty());
}

#[test]
fn map_epiphany_reorient_regathers_when_checkpoint_paths_or_frontier_shift() {
    let state = codex_protocol::protocol::EpiphanyThreadState {
        revision: 9,
        investigation_checkpoint: Some(codex_protocol::protocol::EpiphanyInvestigationCheckpoint {
            checkpoint_id: "ix-shifted".to_string(),
            kind: "slice_planning".to_string(),
            disposition: codex_protocol::protocol::EpiphanyInvestigationDisposition::ResumeReady,
            focus: "Map the live seam.".to_string(),
            next_action: Some("Re-gather the touched source before editing.".to_string()),
            code_refs: vec![codex_protocol::protocol::EpiphanyCodeRef {
                path: PathBuf::from("src/lib.rs"),
                start_line: Some(1),
                end_line: Some(20),
                symbol: Some("shifted_target".to_string()),
                note: None,
            }],
            ..Default::default()
        }),
        ..Default::default()
    };
    let pressure = map_epiphany_pressure(None);
    let retrieval = ThreadEpiphanyRetrievalFreshness {
        status: ThreadEpiphanyRetrievalFreshnessStatus::Ready,
        semantic_available: Some(true),
        last_indexed_at_unix_seconds: Some(1_744_500_000),
        indexed_file_count: Some(12),
        indexed_chunk_count: Some(34),
        dirty_paths: Vec::new(),
        note: "ready".to_string(),
    };
    let graph = ThreadEpiphanyGraphFreshness {
        status: ThreadEpiphanyGraphFreshnessStatus::Ready,
        graph_freshness: Some("fresh".to_string()),
        checkpoint_id: Some("ck-1".to_string()),
        dirty_path_count: 0,
        dirty_paths: Vec::new(),
        open_question_count: 0,
        open_gap_count: 0,
        note: "ready".to_string(),
    };
    let watcher = ThreadEpiphanyInvalidationInput {
        status: ThreadEpiphanyInvalidationStatus::Changed,
        watched_root: Some(test_path_buf("/workspace")),
        observed_at_unix_seconds: Some(1_744_600_001),
        changed_path_count: 1,
        changed_paths: vec![PathBuf::from("src/lib.rs")],
        graph_node_ids: vec!["shifted-target".to_string()],
        active_frontier_node_ids: vec!["shifted-target".to_string()],
        note: "changed".to_string(),
    };

    let (state_status, decision) =
        map_epiphany_reorient(Some(&state), &pressure, &retrieval, &graph, &watcher);

    assert_eq!(state_status, ThreadEpiphanyReorientStateStatus::Ready);
    assert_eq!(decision.action, ThreadEpiphanyReorientAction::Regather);
    assert_eq!(
        decision.checkpoint_status,
        ThreadEpiphanyReorientCheckpointStatus::ResumeReady
    );
    assert_eq!(
        decision.reasons,
        vec![
            ThreadEpiphanyReorientReason::CheckpointPathsChanged,
            ThreadEpiphanyReorientReason::FrontierChanged,
        ]
    );
    assert_eq!(
        decision.checkpoint_changed_paths,
        vec![PathBuf::from("src/lib.rs")]
    );
    assert_eq!(
        decision.active_frontier_node_ids,
        vec!["shifted-target".to_string()]
    );
    assert!(decision.note.contains("Re-gather before editing"));
}

#[test]
fn map_epiphany_scene_latest_records_preserve_newest_first_order() {
    let state = codex_protocol::protocol::EpiphanyThreadState {
        observations: (1..=6)
            .rev()
            .map(|index| codex_protocol::protocol::EpiphanyObservation {
                id: format!("obs-{index}"),
                summary: format!("Observation {index}"),
                source_kind: "test".to_string(),
                status: "ok".to_string(),
                evidence_ids: vec![format!("ev-{index}")],
                ..Default::default()
            })
            .collect(),
        recent_evidence: (1..=6)
            .rev()
            .map(|index| codex_protocol::protocol::EpiphanyEvidenceRecord {
                id: format!("ev-{index}"),
                kind: "test".to_string(),
                status: "ok".to_string(),
                summary: format!("Evidence {index}"),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };

    let scene = map_epiphany_scene(Some(&state), true);
    let observation_ids: Vec<_> = scene
        .observations
        .latest
        .iter()
        .map(|record| record.id.as_str())
        .collect();
    let evidence_ids: Vec<_> = scene
        .evidence
        .latest
        .iter()
        .map(|record| record.id.as_str())
        .collect();

    assert_eq!(
        observation_ids,
        vec!["obs-6", "obs-5", "obs-4", "obs-3", "obs-2"]
    );
    assert_eq!(evidence_ids, vec!["ev-6", "ev-5", "ev-4", "ev-3", "ev-2"]);
}

#[test]
fn map_epiphany_scene_exposes_reorient_result_when_binding_exists() {
    let state = codex_protocol::protocol::EpiphanyThreadState {
        job_bindings: vec![codex_protocol::protocol::EpiphanyJobBinding {
            id: EPIPHANY_REORIENT_LAUNCH_BINDING_ID.to_string(),
            kind: codex_protocol::protocol::EpiphanyJobKind::Specialist,
            scope: "reorient-guided checkpoint resume".to_string(),
            owner_role: EPIPHANY_REORIENT_OWNER_ROLE.to_string(),
            authority_scope: Some("epiphany.reorient.resume".to_string()),
            linked_subgoal_ids: Vec::new(),
            linked_graph_node_ids: Vec::new(),
            blocking_reason: None,
        }],
        ..Default::default()
    };

    let scene = map_epiphany_scene(Some(&state), true);

    assert!(
        scene
            .available_actions
            .contains(&ThreadEpiphanySceneAction::ReorientResult),
        "scene should advertise the reorient result read-back surface"
    );
}

#[test]
fn map_epiphany_reorient_result_finding_projects_structured_output() {
    let raw_result = serde_json::json!({
        "mode": "resume",
        "summary": "Checkpoint still matches the source seam.",
        "nextSafeMove": "Continue with the bounded read-back slice.",
        "checkpointStillValid": true,
        "filesInspected": ["src/a.rs", "src/b.rs"],
        "frontierNodeIds": ["node-1"],
        "evidenceIds": ["ev-1"],
        "artifactRefs": ["artifact:reorient"],
        "runtimeResultId": "result-1",
        "runtimeJobId": "job-1",
        "extra": {"left": "intact"}
    });

    let finding =
        map_epiphany_reorient_finding(raw_result.clone(), Some("job warning".to_string()), None);

    assert_eq!(finding.mode.as_deref(), Some("resume"));
    assert_eq!(
        finding.summary.as_deref(),
        Some("Checkpoint still matches the source seam.")
    );
    assert_eq!(
        finding.next_safe_move.as_deref(),
        Some("Continue with the bounded read-back slice.")
    );
    assert_eq!(finding.checkpoint_still_valid, Some(true));
    assert_eq!(finding.files_inspected, vec!["src/a.rs", "src/b.rs"]);
    assert_eq!(finding.frontier_node_ids, vec!["node-1"]);
    assert_eq!(finding.evidence_ids, vec!["ev-1"]);
    assert_eq!(finding.artifact_refs, vec!["artifact:reorient"]);
    assert_eq!(finding.runtime_result_id.as_deref(), Some("result-1"));
    assert_eq!(finding.runtime_job_id.as_deref(), Some("job-1"));
    assert_eq!(finding.job_error.as_deref(), Some("job warning"));
}

#[test]
fn map_epiphany_crrc_recommendation_continues_clean_checkpoint() {
    let pressure = map_epiphany_pressure(None);
    let decision = ThreadEpiphanyReorientDecision {
        action: ThreadEpiphanyReorientAction::Resume,
        checkpoint_status: ThreadEpiphanyReorientCheckpointStatus::ResumeReady,
        checkpoint_id: Some("ix-clean".to_string()),
        pressure_level: pressure.level,
        retrieval_status: ThreadEpiphanyRetrievalFreshnessStatus::Ready,
        graph_status: ThreadEpiphanyGraphFreshnessStatus::Ready,
        watcher_status: ThreadEpiphanyInvalidationStatus::Clean,
        reasons: vec![ThreadEpiphanyReorientReason::CheckpointReady],
        checkpoint_dirty_paths: Vec::new(),
        checkpoint_changed_paths: Vec::new(),
        active_frontier_node_ids: Vec::new(),
        next_action: "Continue the bounded task.".to_string(),
        note: "ready".to_string(),
    };

    let recommendation = map_epiphany_crrc_recommendation(
        true,
        ThreadEpiphanyReorientStateStatus::Ready,
        &pressure,
        &decision,
        ThreadEpiphanyReorientResultStatus::MissingBinding,
        true,
        false,
        false,
    );

    assert_eq!(recommendation.action, ThreadEpiphanyCrrcAction::Continue);
    assert_eq!(
        recommendation.recommended_scene_action,
        Some(ThreadEpiphanySceneAction::Reorient)
    );
}

#[test]
fn map_epiphany_crrc_recommendation_launches_on_regather_verdict() {
    let pressure = map_epiphany_pressure(None);
    let decision = ThreadEpiphanyReorientDecision {
        action: ThreadEpiphanyReorientAction::Regather,
        checkpoint_status: ThreadEpiphanyReorientCheckpointStatus::ResumeReady,
        checkpoint_id: Some("ix-drifted".to_string()),
        pressure_level: pressure.level,
        retrieval_status: ThreadEpiphanyRetrievalFreshnessStatus::Ready,
        graph_status: ThreadEpiphanyGraphFreshnessStatus::Stale,
        watcher_status: ThreadEpiphanyInvalidationStatus::Changed,
        reasons: vec![ThreadEpiphanyReorientReason::CheckpointPathsChanged],
        checkpoint_dirty_paths: Vec::new(),
        checkpoint_changed_paths: vec![PathBuf::from("src/lib.rs")],
        active_frontier_node_ids: vec!["node-a".to_string()],
        next_action: "Regather.".to_string(),
        note: "drifted".to_string(),
    };

    let recommendation = map_epiphany_crrc_recommendation(
        true,
        ThreadEpiphanyReorientStateStatus::Ready,
        &pressure,
        &decision,
        ThreadEpiphanyReorientResultStatus::MissingBinding,
        true,
        false,
        false,
    );

    assert_eq!(
        recommendation.action,
        ThreadEpiphanyCrrcAction::LaunchReorientWorker
    );
    assert_eq!(
        recommendation.recommended_scene_action,
        Some(ThreadEpiphanySceneAction::ReorientLaunch)
    );
}

#[test]
fn map_epiphany_crrc_recommendation_accepts_completed_finding() {
    let pressure = map_epiphany_pressure(None);
    let decision = ThreadEpiphanyReorientDecision {
        action: ThreadEpiphanyReorientAction::Regather,
        checkpoint_status: ThreadEpiphanyReorientCheckpointStatus::RegatherRequired,
        checkpoint_id: Some("ix-result".to_string()),
        pressure_level: pressure.level,
        retrieval_status: ThreadEpiphanyRetrievalFreshnessStatus::Ready,
        graph_status: ThreadEpiphanyGraphFreshnessStatus::Ready,
        watcher_status: ThreadEpiphanyInvalidationStatus::Clean,
        reasons: vec![ThreadEpiphanyReorientReason::CheckpointRequestedRegather],
        checkpoint_dirty_paths: Vec::new(),
        checkpoint_changed_paths: Vec::new(),
        active_frontier_node_ids: Vec::new(),
        next_action: "Review result.".to_string(),
        note: "result".to_string(),
    };

    let recommendation = map_epiphany_crrc_recommendation(
        true,
        ThreadEpiphanyReorientStateStatus::Ready,
        &pressure,
        &decision,
        ThreadEpiphanyReorientResultStatus::Completed,
        true,
        true,
        false,
    );

    assert_eq!(
        recommendation.action,
        ThreadEpiphanyCrrcAction::AcceptReorientResult
    );
    assert_eq!(
        recommendation.recommended_scene_action,
        Some(ThreadEpiphanySceneAction::ReorientAccept)
    );
}

#[test]
fn map_epiphany_crrc_recommendation_relaunches_stale_accepted_regather() {
    let pressure = map_epiphany_pressure(None);
    let decision = ThreadEpiphanyReorientDecision {
        action: ThreadEpiphanyReorientAction::Regather,
        checkpoint_status: ThreadEpiphanyReorientCheckpointStatus::RegatherRequired,
        checkpoint_id: Some("ix-accepted".to_string()),
        pressure_level: pressure.level,
        retrieval_status: ThreadEpiphanyRetrievalFreshnessStatus::Ready,
        graph_status: ThreadEpiphanyGraphFreshnessStatus::Ready,
        watcher_status: ThreadEpiphanyInvalidationStatus::Clean,
        reasons: vec![ThreadEpiphanyReorientReason::CheckpointRequestedRegather],
        checkpoint_dirty_paths: Vec::new(),
        checkpoint_changed_paths: Vec::new(),
        active_frontier_node_ids: Vec::new(),
        next_action: "Regather from the accepted result.".to_string(),
        note: "accepted".to_string(),
    };

    let recommendation = map_epiphany_crrc_recommendation(
        true,
        ThreadEpiphanyReorientStateStatus::Ready,
        &pressure,
        &decision,
        ThreadEpiphanyReorientResultStatus::Completed,
        true,
        true,
        true,
    );

    assert_eq!(
        recommendation.action,
        ThreadEpiphanyCrrcAction::LaunchReorientWorker
    );
    assert_eq!(
        recommendation.recommended_scene_action,
        Some(ThreadEpiphanySceneAction::ReorientLaunch)
    );

    let unloaded = map_epiphany_crrc_recommendation(
        false,
        ThreadEpiphanyReorientStateStatus::Ready,
        &pressure,
        &decision,
        ThreadEpiphanyReorientResultStatus::Completed,
        true,
        true,
        true,
    );
    assert_eq!(unloaded.action, ThreadEpiphanyCrrcAction::RegatherManually);
    assert_eq!(
        unloaded.recommended_scene_action,
        Some(ThreadEpiphanySceneAction::Reorient)
    );
}

#[test]
fn map_epiphany_crrc_recommendation_continues_after_accepted_resume_finding() {
    let pressure = map_epiphany_pressure(None);
    let decision = ThreadEpiphanyReorientDecision {
        action: ThreadEpiphanyReorientAction::Resume,
        checkpoint_status: ThreadEpiphanyReorientCheckpointStatus::ResumeReady,
        checkpoint_id: Some("ix-accepted-resume".to_string()),
        pressure_level: pressure.level,
        retrieval_status: ThreadEpiphanyRetrievalFreshnessStatus::Ready,
        graph_status: ThreadEpiphanyGraphFreshnessStatus::Ready,
        watcher_status: ThreadEpiphanyInvalidationStatus::Clean,
        reasons: Vec::new(),
        checkpoint_dirty_paths: Vec::new(),
        checkpoint_changed_paths: Vec::new(),
        active_frontier_node_ids: Vec::new(),
        next_action: "Continue from the accepted result.".to_string(),
        note: "accepted resume".to_string(),
    };

    let recommendation = map_epiphany_crrc_recommendation(
        true,
        ThreadEpiphanyReorientStateStatus::Ready,
        &pressure,
        &decision,
        ThreadEpiphanyReorientResultStatus::Completed,
        true,
        true,
        true,
    );

    assert_eq!(recommendation.action, ThreadEpiphanyCrrcAction::Continue);
    assert_eq!(
        recommendation.recommended_scene_action,
        Some(ThreadEpiphanySceneAction::Reorient)
    );
}

#[test]
fn map_epiphany_roles_projects_mvp_ownership_lanes() {
    let state = codex_protocol::protocol::EpiphanyThreadState {
        investigation_checkpoint: Some(EpiphanyInvestigationCheckpoint {
            checkpoint_id: "ix-roles".to_string(),
            kind: "slice_planning".to_string(),
            disposition: codex_protocol::protocol::EpiphanyInvestigationDisposition::ResumeReady,
            focus: "Keep roles explicit.".to_string(),
            next_action: Some("Continue from the bounded checkpoint.".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let pressure = map_epiphany_pressure(None);
    let decision = ThreadEpiphanyReorientDecision {
        action: ThreadEpiphanyReorientAction::Resume,
        checkpoint_status: ThreadEpiphanyReorientCheckpointStatus::ResumeReady,
        checkpoint_id: Some("ix-roles".to_string()),
        pressure_level: pressure.level,
        retrieval_status: ThreadEpiphanyRetrievalFreshnessStatus::Ready,
        graph_status: ThreadEpiphanyGraphFreshnessStatus::Ready,
        watcher_status: ThreadEpiphanyInvalidationStatus::Clean,
        reasons: vec![ThreadEpiphanyReorientReason::CheckpointReady],
        checkpoint_dirty_paths: Vec::new(),
        checkpoint_changed_paths: Vec::new(),
        active_frontier_node_ids: Vec::new(),
        next_action: "Continue from the bounded checkpoint.".to_string(),
        note: "ready".to_string(),
    };
    let recommendation = ThreadEpiphanyCrrcRecommendation {
        action: ThreadEpiphanyCrrcAction::Continue,
        recommended_scene_action: Some(ThreadEpiphanySceneAction::Reorient),
        reason: "Pressure is tolerable.".to_string(),
    };
    let jobs = vec![ThreadEpiphanyJob {
        id: "verification".to_string(),
        kind: ThreadEpiphanyJobKind::Verification,
        scope: "invariant verification".to_string(),
        owner_role: "epiphany-verifier".to_string(),
        launcher_job_id: None,
        authority_scope: None,
        backend_job_id: None,
        status: ThreadEpiphanyJobStatus::Needed,
        items_processed: None,
        items_total: None,
        progress_note: Some("Evidence needs review.".to_string()),
        last_checkpoint_at_unix_seconds: None,
        blocking_reason: None,
        active_thread_ids: Vec::new(),
        linked_subgoal_ids: Vec::new(),
        linked_graph_node_ids: Vec::new(),
    }];

    let roles = map_epiphany_roles(
        Some(&state),
        &jobs,
        &decision,
        &pressure,
        &recommendation,
        ThreadEpiphanyReorientResultStatus::MissingBinding,
        None,
    );

    let role_ids = roles.iter().map(|role| role.id).collect::<Vec<_>>();
    assert_eq!(
        role_ids,
        vec![
            ThreadEpiphanyRoleId::Implementation,
            ThreadEpiphanyRoleId::Imagination,
            ThreadEpiphanyRoleId::Modeling,
            ThreadEpiphanyRoleId::Verification,
            ThreadEpiphanyRoleId::Reorientation,
        ]
    );
    assert_eq!(roles[0].status, ThreadEpiphanyRoleStatus::Ready);
    assert_eq!(roles[1].status, ThreadEpiphanyRoleStatus::Needed);
    assert_eq!(roles[2].status, ThreadEpiphanyRoleStatus::Ready);
    assert_eq!(roles[3].status, ThreadEpiphanyRoleStatus::Needed);
    assert_eq!(roles[3].jobs[0].owner_role, "epiphany-verifier");
    assert_eq!(roles[4].status, ThreadEpiphanyRoleStatus::Ready);
}

fn base_coordinator_roles() -> Vec<ThreadEpiphanyRoleLane> {
    vec![
        coordinator_role(
            ThreadEpiphanyRoleId::Implementation,
            ThreadEpiphanyRoleStatus::Ready,
        ),
        coordinator_role(
            ThreadEpiphanyRoleId::Imagination,
            ThreadEpiphanyRoleStatus::Ready,
        ),
        coordinator_role(
            ThreadEpiphanyRoleId::Modeling,
            ThreadEpiphanyRoleStatus::Ready,
        ),
        coordinator_role(
            ThreadEpiphanyRoleId::Verification,
            ThreadEpiphanyRoleStatus::Ready,
        ),
        coordinator_role(
            ThreadEpiphanyRoleId::Reorientation,
            ThreadEpiphanyRoleStatus::Ready,
        ),
    ]
}

fn coordinator_role(
    id: ThreadEpiphanyRoleId,
    status: ThreadEpiphanyRoleStatus,
) -> ThreadEpiphanyRoleLane {
    ThreadEpiphanyRoleLane {
        id,
        title: format!("{id:?}"),
        owner_role: format!("{id:?}"),
        status,
        note: "test lane".to_string(),
        jobs: Vec::new(),
        authority_scopes: Vec::new(),
        recommended_action: None,
    }
}

fn coordinator_signals(
    crrc_action: ThreadEpiphanyCrrcAction,
    reorient_result_status: ThreadEpiphanyReorientResultStatus,
    modeling_result_status: ThreadEpiphanyRoleResultStatus,
    verification_result_status: ThreadEpiphanyRoleResultStatus,
) -> ThreadEpiphanyCoordinatorSignals {
    ThreadEpiphanyCoordinatorSignals {
        pressure_level: ThreadEpiphanyPressureLevel::Low,
        should_prepare_compaction: false,
        reorient_action: ThreadEpiphanyReorientAction::Resume,
        crrc_action,
        modeling_result_status,
        verification_result_status,
        reorient_result_status,
    }
}

#[derive(Clone, Copy)]
struct CoordinatorTestFlags {
    modeling_result_accepted: bool,
    modeling_result_reviewable: bool,
    modeling_result_accepted_after_verification: bool,
    implementation_evidence_after_verification: bool,
    verification_result_cites_implementation_evidence: bool,
    verification_result_covers_current_modeling: bool,
    verification_result_accepted: bool,
    verification_result_allows_implementation: bool,
    verification_result_needs_evidence: bool,
    reorient_finding_accepted: bool,
}

impl Default for CoordinatorTestFlags {
    fn default() -> Self {
        Self {
            modeling_result_accepted: false,
            modeling_result_reviewable: false,
            modeling_result_accepted_after_verification: false,
            implementation_evidence_after_verification: false,
            verification_result_cites_implementation_evidence: false,
            verification_result_covers_current_modeling: true,
            verification_result_accepted: false,
            verification_result_allows_implementation: false,
            verification_result_needs_evidence: false,
            reorient_finding_accepted: false,
        }
    }
}

fn coordinator_decision(
    state_status: ThreadEpiphanyReorientStateStatus,
    checkpoint_present: bool,
    pressure: &ThreadEpiphanyPressure,
    recommendation: &ThreadEpiphanyCrrcRecommendation,
    roles: &[ThreadEpiphanyRoleLane],
    signals: &ThreadEpiphanyCoordinatorSignals,
    flags: CoordinatorTestFlags,
) -> EpiphanyCoordinatorDecision {
    map_epiphany_coordinator(
        state_status,
        checkpoint_present,
        pressure,
        recommendation,
        roles,
        signals,
        flags.modeling_result_accepted,
        flags.modeling_result_reviewable,
        flags.modeling_result_accepted_after_verification,
        flags.implementation_evidence_after_verification,
        flags.verification_result_cites_implementation_evidence,
        flags.verification_result_covers_current_modeling,
        flags.verification_result_accepted,
        flags.verification_result_allows_implementation,
        flags.verification_result_needs_evidence,
        flags.reorient_finding_accepted,
    )
}

#[test]
fn map_epiphany_coordinator_prepares_missing_checkpoint() {
    let pressure = map_epiphany_pressure(None);
    let recommendation = ThreadEpiphanyCrrcRecommendation {
        action: ThreadEpiphanyCrrcAction::PrepareCheckpoint,
        recommended_scene_action: Some(ThreadEpiphanySceneAction::Update),
        reason: "checkpoint missing".to_string(),
    };
    let roles = base_coordinator_roles();
    let signals = coordinator_signals(
        ThreadEpiphanyCrrcAction::PrepareCheckpoint,
        ThreadEpiphanyReorientResultStatus::MissingBinding,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
    );

    let decision = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        false,
        &pressure,
        &recommendation,
        &roles,
        &signals,
        CoordinatorTestFlags::default(),
    );

    assert_eq!(
        decision.action,
        ThreadEpiphanyCoordinatorAction::PrepareCheckpoint
    );
    assert!(!decision.can_auto_run);
}

#[test]
fn map_epiphany_coordinator_compacts_at_pressure_threshold() {
    let pressure = map_epiphany_pressure(Some(&core_token_usage_info(
        1_000,
        80,
        Some(200),
        Some(100),
    )));
    let recommendation = ThreadEpiphanyCrrcRecommendation {
        action: ThreadEpiphanyCrrcAction::Continue,
        recommended_scene_action: Some(ThreadEpiphanySceneAction::Reorient),
        reason: "continue".to_string(),
    };
    let roles = base_coordinator_roles();
    let signals = coordinator_signals(
        ThreadEpiphanyCrrcAction::Continue,
        ThreadEpiphanyReorientResultStatus::MissingBinding,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
    );

    let decision = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &signals,
        CoordinatorTestFlags::default(),
    );

    assert_eq!(
        decision.action,
        ThreadEpiphanyCoordinatorAction::CompactRehydrateReorient
    );
    assert!(decision.can_auto_run);
    assert_eq!(
        map_epiphany_coordinator_automation_action(&decision),
        EpiphanyCoordinatorAutomationAction::CompactRehydrateReorient
    );
}

#[test]
fn map_epiphany_coordinator_does_not_recompact_after_accepted_resume_reorient() {
    let pressure = map_epiphany_pressure(Some(&core_token_usage_info(
        1_000,
        80,
        Some(200),
        Some(100),
    )));
    let recommendation = ThreadEpiphanyCrrcRecommendation {
        action: ThreadEpiphanyCrrcAction::Continue,
        recommended_scene_action: Some(ThreadEpiphanySceneAction::Reorient),
        reason: "accepted resume finding".to_string(),
    };
    let signals = coordinator_signals(
        ThreadEpiphanyCrrcAction::Continue,
        ThreadEpiphanyReorientResultStatus::Completed,
        ThreadEpiphanyRoleResultStatus::BackendMissing,
        ThreadEpiphanyRoleResultStatus::BackendMissing,
    );

    let decision = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &[],
        &signals,
        CoordinatorTestFlags {
            reorient_finding_accepted: true,
            ..Default::default()
        },
    );

    assert_eq!(
        decision.action,
        ThreadEpiphanyCoordinatorAction::ContinueImplementation
    );
}

#[test]
fn select_epiphany_coordinator_automation_forces_checkpoint_compaction_handoff() {
    let decision = EpiphanyCoordinatorDecision {
        action: ThreadEpiphanyCoordinatorAction::ContinueImplementation,
        target_role: Some(ThreadEpiphanyRoleId::Implementation),
        recommended_scene_action: None,
        requires_review: false,
        can_auto_run: false,
        reason: "ordinary coordinator would continue".to_string(),
    };

    assert_eq!(
        map_epiphany_coordinator_automation_action(&decision),
        EpiphanyCoordinatorAutomationAction::None
    );
    assert_eq!(
        select_epiphany_coordinator_automation_action(&decision, true),
        EpiphanyCoordinatorAutomationAction::CompactRehydrateReorient
    );
}

#[test]
fn map_epiphany_coordinator_launches_reorient_worker_from_crrc() {
    let pressure = map_epiphany_pressure(None);
    let recommendation = ThreadEpiphanyCrrcRecommendation {
        action: ThreadEpiphanyCrrcAction::LaunchReorientWorker,
        recommended_scene_action: Some(ThreadEpiphanySceneAction::ReorientLaunch),
        reason: "regather".to_string(),
    };
    let roles = base_coordinator_roles();
    let signals = coordinator_signals(
        ThreadEpiphanyCrrcAction::LaunchReorientWorker,
        ThreadEpiphanyReorientResultStatus::MissingBinding,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
    );

    let decision = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &signals,
        CoordinatorTestFlags::default(),
    );

    assert_eq!(
        decision.action,
        ThreadEpiphanyCoordinatorAction::LaunchReorientWorker
    );
    assert_eq!(
        decision.target_role,
        Some(ThreadEpiphanyRoleId::Reorientation)
    );
    assert!(decision.can_auto_run);
    assert_eq!(
        map_epiphany_coordinator_automation_action(&decision),
        EpiphanyCoordinatorAutomationAction::LaunchReorientWorker
    );
}

#[test]
fn map_epiphany_coordinator_reviews_reorient_result() {
    let pressure = map_epiphany_pressure(None);
    let recommendation = ThreadEpiphanyCrrcRecommendation {
        action: ThreadEpiphanyCrrcAction::AcceptReorientResult,
        recommended_scene_action: Some(ThreadEpiphanySceneAction::ReorientAccept),
        reason: "review".to_string(),
    };
    let roles = base_coordinator_roles();
    let signals = coordinator_signals(
        ThreadEpiphanyCrrcAction::AcceptReorientResult,
        ThreadEpiphanyReorientResultStatus::Completed,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
    );

    let decision = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &signals,
        CoordinatorTestFlags::default(),
    );

    assert_eq!(
        decision.action,
        ThreadEpiphanyCoordinatorAction::ReviewReorientResult
    );
    assert!(decision.requires_review);
    assert_eq!(
        map_epiphany_coordinator_automation_action(&decision),
        EpiphanyCoordinatorAutomationAction::None
    );
}

#[test]
fn map_epiphany_coordinator_uses_fixed_lanes_before_manual_regather() {
    let pressure = map_epiphany_pressure(None);
    let recommendation = ThreadEpiphanyCrrcRecommendation {
        action: ThreadEpiphanyCrrcAction::RegatherManually,
        recommended_scene_action: Some(ThreadEpiphanySceneAction::Reorient),
        reason: "checkpoint requested regather".to_string(),
    };
    let roles = base_coordinator_roles();
    let missing = coordinator_signals(
        ThreadEpiphanyCrrcAction::RegatherManually,
        ThreadEpiphanyReorientResultStatus::Completed,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
    );

    let launch_modeling = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &missing,
        CoordinatorTestFlags {
            reorient_finding_accepted: true,
            ..Default::default()
        },
    );
    assert_eq!(
        launch_modeling.action,
        ThreadEpiphanyCoordinatorAction::LaunchModeling
    );

    let modeling_done = coordinator_signals(
        ThreadEpiphanyCrrcAction::RegatherManually,
        ThreadEpiphanyReorientResultStatus::Completed,
        ThreadEpiphanyRoleResultStatus::Completed,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
    );
    let review_modeling = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &modeling_done,
        CoordinatorTestFlags {
            modeling_result_reviewable: true,
            reorient_finding_accepted: true,
            ..Default::default()
        },
    );
    assert_eq!(
        review_modeling.action,
        ThreadEpiphanyCoordinatorAction::ReviewModelingResult
    );

    let mut blocked_roles = base_coordinator_roles();
    blocked_roles[0].status = ThreadEpiphanyRoleStatus::Blocked;
    let verifier_done = coordinator_signals(
        ThreadEpiphanyCrrcAction::RegatherManually,
        ThreadEpiphanyReorientResultStatus::Completed,
        ThreadEpiphanyRoleResultStatus::Completed,
        ThreadEpiphanyRoleResultStatus::Completed,
    );
    let blocked_regather = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &blocked_roles,
        &verifier_done,
        CoordinatorTestFlags {
            modeling_result_accepted: true,
            modeling_result_accepted_after_verification: true,
            verification_result_accepted: true,
            verification_result_needs_evidence: true,
            reorient_finding_accepted: true,
            ..Default::default()
        },
    );
    assert_eq!(
        blocked_regather.action,
        ThreadEpiphanyCoordinatorAction::RegatherManually
    );
}

#[test]
fn map_epiphany_coordinator_runs_modeling_then_verification_then_continue() {
    let pressure = map_epiphany_pressure(None);
    let recommendation = ThreadEpiphanyCrrcRecommendation {
        action: ThreadEpiphanyCrrcAction::Continue,
        recommended_scene_action: Some(ThreadEpiphanySceneAction::Reorient),
        reason: "continue".to_string(),
    };
    let roles = base_coordinator_roles();
    let missing = coordinator_signals(
        ThreadEpiphanyCrrcAction::Continue,
        ThreadEpiphanyReorientResultStatus::MissingBinding,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
    );

    let launch_modeling = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &missing,
        CoordinatorTestFlags::default(),
    );
    assert_eq!(
        launch_modeling.action,
        ThreadEpiphanyCoordinatorAction::LaunchModeling
    );
    assert_eq!(
        map_epiphany_coordinator_automation_action(&launch_modeling),
        EpiphanyCoordinatorAutomationAction::None
    );

    let modeling_backend_unavailable = coordinator_signals(
        ThreadEpiphanyCrrcAction::Continue,
        ThreadEpiphanyReorientResultStatus::MissingBinding,
        ThreadEpiphanyRoleResultStatus::BackendUnavailable,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
    );
    let relaunch_modeling = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &modeling_backend_unavailable,
        CoordinatorTestFlags::default(),
    );
    assert_eq!(
        relaunch_modeling.action,
        ThreadEpiphanyCoordinatorAction::LaunchModeling
    );

    let modeling_done = coordinator_signals(
        ThreadEpiphanyCrrcAction::Continue,
        ThreadEpiphanyReorientResultStatus::MissingBinding,
        ThreadEpiphanyRoleResultStatus::Completed,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
    );
    let review_modeling = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &modeling_done,
        CoordinatorTestFlags {
            modeling_result_reviewable: true,
            ..Default::default()
        },
    );
    assert_eq!(
        review_modeling.action,
        ThreadEpiphanyCoordinatorAction::ReviewModelingResult
    );
    assert!(review_modeling.requires_review);

    let modeling_running_with_old_verification = coordinator_signals(
        ThreadEpiphanyCrrcAction::Continue,
        ThreadEpiphanyReorientResultStatus::MissingBinding,
        ThreadEpiphanyRoleResultStatus::Running,
        ThreadEpiphanyRoleResultStatus::Completed,
    );
    let wait_for_modeling = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &modeling_running_with_old_verification,
        CoordinatorTestFlags::default(),
    );
    assert_eq!(
        wait_for_modeling.action,
        ThreadEpiphanyCoordinatorAction::ReviewModelingResult
    );
    assert!(!wait_for_modeling.requires_review);
    assert!(wait_for_modeling.reason.contains("stale verification"));

    let relaunch_unreviewable_modeling = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &modeling_done,
        CoordinatorTestFlags::default(),
    );
    assert_eq!(
        relaunch_unreviewable_modeling.action,
        ThreadEpiphanyCoordinatorAction::LaunchModeling
    );
    assert!(relaunch_unreviewable_modeling.reason.contains("statePatch"));

    let launch_verification = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &modeling_done,
        CoordinatorTestFlags {
            modeling_result_accepted: true,
            modeling_result_reviewable: true,
            ..Default::default()
        },
    );
    assert_eq!(
        launch_verification.action,
        ThreadEpiphanyCoordinatorAction::LaunchVerification
    );
    assert_eq!(
        map_epiphany_coordinator_automation_action(&launch_verification),
        EpiphanyCoordinatorAutomationAction::None
    );

    let verification_done = coordinator_signals(
        ThreadEpiphanyCrrcAction::Continue,
        ThreadEpiphanyReorientResultStatus::MissingBinding,
        ThreadEpiphanyRoleResultStatus::Completed,
        ThreadEpiphanyRoleResultStatus::Completed,
    );
    let stale_verification = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &verification_done,
        CoordinatorTestFlags {
            modeling_result_accepted: true,
            modeling_result_reviewable: true,
            verification_result_covers_current_modeling: false,
            ..Default::default()
        },
    );
    assert_eq!(
        stale_verification.action,
        ThreadEpiphanyCoordinatorAction::LaunchVerification
    );
    assert!(
        stale_verification
            .reason
            .contains("currently accepted modeling")
    );

    let review_verification = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &verification_done,
        CoordinatorTestFlags {
            modeling_result_accepted: true,
            modeling_result_reviewable: true,
            ..Default::default()
        },
    );
    assert_eq!(
        review_verification.action,
        ThreadEpiphanyCoordinatorAction::ReviewVerificationResult
    );
    assert!(review_verification.requires_review);

    let accepted_non_pass_verification = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &verification_done,
        CoordinatorTestFlags {
            modeling_result_accepted: true,
            modeling_result_reviewable: true,
            verification_result_accepted: true,
            ..Default::default()
        },
    );
    assert_eq!(
        accepted_non_pass_verification.action,
        ThreadEpiphanyCoordinatorAction::LaunchModeling
    );
    assert_eq!(
        accepted_non_pass_verification.target_role,
        Some(ThreadEpiphanyRoleId::Modeling)
    );

    let accepted_needs_evidence_verification = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &verification_done,
        CoordinatorTestFlags {
            modeling_result_accepted: true,
            modeling_result_reviewable: true,
            verification_result_accepted: true,
            verification_result_needs_evidence: true,
            ..Default::default()
        },
    );
    assert_eq!(
        accepted_needs_evidence_verification.action,
        ThreadEpiphanyCoordinatorAction::ContinueImplementation
    );
    assert_eq!(
        accepted_needs_evidence_verification.target_role,
        Some(ThreadEpiphanyRoleId::Implementation)
    );
    assert!(
        accepted_needs_evidence_verification
            .reason
            .contains("implementation evidence")
    );

    let accepted_failed_implementation_verification = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &verification_done,
        CoordinatorTestFlags {
            modeling_result_accepted: true,
            modeling_result_reviewable: true,
            verification_result_accepted: true,
            verification_result_cites_implementation_evidence: true,
            ..Default::default()
        },
    );
    assert_eq!(
        accepted_failed_implementation_verification.action,
        ThreadEpiphanyCoordinatorAction::ContinueImplementation
    );
    assert_eq!(
        accepted_failed_implementation_verification.target_role,
        Some(ThreadEpiphanyRoleId::Implementation)
    );
    assert!(
        accepted_failed_implementation_verification
            .reason
            .contains("bounded repair")
    );

    let implementation_after_verification = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &verification_done,
        CoordinatorTestFlags {
            modeling_result_accepted: true,
            modeling_result_reviewable: true,
            implementation_evidence_after_verification: true,
            verification_result_accepted: true,
            verification_result_needs_evidence: true,
            ..Default::default()
        },
    );
    assert_eq!(
        implementation_after_verification.action,
        ThreadEpiphanyCoordinatorAction::LaunchVerification
    );
    assert_eq!(
        implementation_after_verification.target_role,
        Some(ThreadEpiphanyRoleId::Verification)
    );
    assert!(
        implementation_after_verification
            .reason
            .contains("Implementation evidence")
    );

    let accepted_verification = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &roles,
        &verification_done,
        CoordinatorTestFlags {
            modeling_result_accepted: true,
            modeling_result_reviewable: true,
            verification_result_accepted: true,
            verification_result_allows_implementation: true,
            ..Default::default()
        },
    );
    assert_eq!(
        accepted_verification.action,
        ThreadEpiphanyCoordinatorAction::ContinueImplementation
    );
    assert!(!accepted_verification.requires_review);

    let reviewed = coordinator_signals(
        ThreadEpiphanyCrrcAction::Continue,
        ThreadEpiphanyReorientResultStatus::MissingBinding,
        ThreadEpiphanyRoleResultStatus::BackendMissing,
        ThreadEpiphanyRoleResultStatus::BackendMissing,
    );
    let continue_implementation = coordinator_decision(
        ThreadEpiphanyReorientStateStatus::Ready,
        true,
        &pressure,
        &recommendation,
        &[],
        &reviewed,
        CoordinatorTestFlags::default(),
    );
    assert_eq!(
        continue_implementation.action,
        ThreadEpiphanyCoordinatorAction::ContinueImplementation
    );
}

#[test]
fn epiphany_specialist_prompt_config_parses() {
    let prompts = epiphany_specialist_prompt_config();
    assert!(
        prompts
            .shared
            .persistent_memory
            .contains("Ghostlight-derived memory")
    );
    assert!(
        prompts
            .shared
            .persistent_memory
            .contains("First law of the lane")
    );
    assert!(
        prompts
            .roles
            .imagination
            .contains("Imagination of the machine")
    );
    assert!(
        prompts
            .roles
            .imagination
            .contains("Imagination improves itself")
    );
    assert!(prompts.roles.modeling.contains("Body of the machine"));
    assert!(prompts.roles.modeling.contains("The Body improves itself"));
    assert!(prompts.roles.verification.contains("Soul of the machine"));
    assert!(
        prompts
            .roles
            .verification
            .contains("The Soul improves itself")
    );
    assert!(prompts.roles.research.contains("Eyes of the machine"));
    assert!(prompts.roles.face.contains("Epiphany Face"));
    assert!(prompts.roles.face.contains("#aquarium"));
    assert!(
        prompts
            .roles
            .research
            .contains("The Eyes improve themselves")
    );
    assert!(
        prompts
            .implementation
            .continue_template
            .contains("Hands of the machine")
    );
    assert!(
        prompts
            .implementation
            .continue_template
            .contains("The Hands improve themselves")
    );
    assert!(prompts.reorientation.resume.contains("Life across sleep"));
    assert!(
        prompts
            .reorientation
            .resume
            .contains("Life improves itself")
    );
    assert!(
        prompts
            .reorientation
            .regather
            .contains("Life returning after rupture")
    );
    assert!(
        prompts
            .crrc
            .pre_compaction_checkpoint_intervention
            .contains("pre-compaction checkpoint intervention")
    );

    let note = render_epiphany_coordinator_note(
        ThreadEpiphanyCrrcAction::Continue,
        ThreadEpiphanyPressureLevel::Unknown,
        ThreadEpiphanyRoleResultStatus::Completed,
        ThreadEpiphanyRoleResultStatus::MissingBinding,
        ThreadEpiphanyReorientResultStatus::MissingBinding,
        ThreadEpiphanyCoordinatorAction::LaunchVerification,
    );
    assert!(note.contains("read-only Self"));
    assert!(note.contains("Epiphany Persistent Memory"));
    assert!(note.contains("Self improves itself"));
    assert!(note.contains("Imagination/planning"));
    assert!(note.contains("Eyes/research"));
    assert!(note.contains("Hands/implementation"));
    assert!(note.contains("LaunchVerification"));
    assert!(!note.contains("{coordinator_action}"));
}

#[test]
fn build_epiphany_role_launch_request_uses_fixed_mvp_templates() {
    let state = codex_protocol::protocol::EpiphanyThreadState {
        revision: 7,
        objective: Some("Keep the MVP slice reviewable.".to_string()),
        active_subgoal_id: Some("sg-1".to_string()),
        subgoals: vec![codex_protocol::protocol::EpiphanySubgoal {
            id: "sg-1".to_string(),
            title: "Role launch".to_string(),
            status: "active".to_string(),
            summary: None,
        }],
        graph_frontier: Some(codex_protocol::protocol::EpiphanyGraphFrontier {
            active_node_ids: vec!["node-1".to_string()],
            ..Default::default()
        }),
        ..Default::default()
    };

    let imagination = build_epiphany_role_launch_request(
        "thr_123",
        ThreadEpiphanyRoleId::Imagination,
        Some(7),
        Some(90),
        &state,
    )
    .expect("imagination should have a fixed launch template");
    assert_eq!(imagination.binding_id, EPIPHANY_IMAGINATION_ROLE_BINDING_ID);
    assert_eq!(imagination.owner_role, EPIPHANY_IMAGINATION_OWNER_ROLE);
    assert_eq!(imagination.authority_scope, "epiphany.role.imagination");
    let EpiphanyWorkerLaunchDocument::Role(imagination_document) = &imagination.launch_document
    else {
        panic!("imagination should build a role launch document");
    };
    assert_eq!(imagination_document.role_id, "imagination");
    assert!(imagination_document.planning.is_some());
    assert!(
        imagination
            .instruction
            .contains("Epiphany Persistent Memory")
    );
    assert!(
        imagination
            .instruction
            .contains("Imagination of the machine")
    );
    assert!(
        imagination
            .instruction
            .contains("reviewable `thread/epiphany/update` patch")
    );
    assert_eq!(
        imagination.output_contract_id,
        "epiphany.worker.role_result.v0"
    );
    let imagination_schema = epiphany_role_launch_output_schema(ThreadEpiphanyRoleId::Imagination);
    assert!(imagination_schema["properties"].get("selfPatch").is_some());
    assert!(imagination_schema["properties"].get("statePatch").is_some());
    assert_eq!(
        imagination_schema["properties"]["statePatch"]["required"][0],
        "planning"
    );
    assert_eq!(
        imagination_schema["properties"]["statePatch"]["properties"]["planning"]["properties"]["objective_drafts"]
            ["minItems"],
        1
    );

    let modeling = build_epiphany_role_launch_request(
        "thr_123",
        ThreadEpiphanyRoleId::Modeling,
        Some(7),
        Some(90),
        &state,
    )
    .expect("modeling should have a fixed launch template");
    assert_eq!(modeling.binding_id, EPIPHANY_MODELING_ROLE_BINDING_ID);
    assert_eq!(modeling.owner_role, EPIPHANY_MODELING_OWNER_ROLE);
    assert_eq!(modeling.authority_scope, "epiphany.role.modeling");
    assert_eq!(modeling.linked_subgoal_ids, vec!["sg-1".to_string()]);
    assert_eq!(modeling.linked_graph_node_ids, vec!["node-1".to_string()]);
    let EpiphanyWorkerLaunchDocument::Role(modeling_document) = &modeling.launch_document else {
        panic!("modeling should build a role launch document");
    };
    assert_eq!(modeling_document.role_id, "modeling");
    assert!(modeling_document.graphs.is_some());
    assert!(modeling_document.recent_observations.is_empty());
    assert!(modeling.instruction.contains("Epiphany Persistent Memory"));
    assert!(modeling.instruction.contains("Body of the machine"));
    assert!(modeling.instruction.contains("coherent anatomy"));
    assert!(
        modeling
            .instruction
            .contains("`statePatch` is part of the modeling job")
    );
    assert!(modeling.instruction.contains("optional `selfPatch`"));
    assert_eq!(
        modeling.output_contract_id,
        "epiphany.worker.role_result.v0"
    );
    let modeling_schema = epiphany_role_launch_output_schema(ThreadEpiphanyRoleId::Modeling);
    assert!(modeling_schema["properties"].get("openQuestions").is_some());
    assert!(modeling_schema["properties"].get("selfPatch").is_some());
    assert!(modeling_schema["properties"].get("statePatch").is_some());
    assert_eq!(
        modeling_schema["properties"]["statePatch"]["anyOf"][0]["required"][0],
        "graphs"
    );
    assert_eq!(
        modeling_schema["properties"]["statePatch"]["properties"]["investigationCheckpoint"]["properties"]
            ["disposition"]["enum"][0],
        "resume_ready"
    );
    let modeling_required = modeling_schema["required"]
        .as_array()
        .expect("modeling schema required fields")
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert!(modeling_required.contains(&"statePatch"));
    assert!(modeling_required.contains(&"verdict"));

    let verification = build_epiphany_role_launch_request(
        "thr_123",
        ThreadEpiphanyRoleId::Verification,
        Some(7),
        Some(90),
        &state,
    )
    .expect("verification should have a fixed launch template");
    assert_eq!(
        verification.binding_id,
        EPIPHANY_VERIFICATION_ROLE_BINDING_ID
    );
    assert_eq!(verification.owner_role, EPIPHANY_VERIFICATION_OWNER_ROLE);
    assert_eq!(verification.authority_scope, "epiphany.role.verification");
    let EpiphanyWorkerLaunchDocument::Role(verification_document) = &verification.launch_document
    else {
        panic!("verification should build a role launch document");
    };
    assert_eq!(verification_document.role_id, "verification");
    assert!(
        verification
            .instruction
            .contains("Epiphany Persistent Memory")
    );
    assert!(verification.instruction.contains("Soul of the machine"));
    assert!(verification.instruction.contains("falsify"));
    let verification_schema =
        epiphany_role_launch_output_schema(ThreadEpiphanyRoleId::Verification);
    assert!(
        verification_schema["properties"]
            .get("evidenceGaps")
            .is_some()
    );
    assert!(verification_schema["properties"].get("risks").is_some());
    assert!(verification_schema["properties"].get("selfPatch").is_some());
    assert!(
        verification_schema["properties"]
            .get("statePatch")
            .is_none()
    );
}

#[test]
fn build_epiphany_reorient_launch_request_uses_life_template() {
    let state = codex_protocol::protocol::EpiphanyThreadState {
        revision: 7,
        scratch: Some(codex_protocol::protocol::EpiphanyScratchPad {
            summary: Some("Carry the current seam across compaction.".to_string()),
            ..Default::default()
        }),
        graph_frontier: Some(codex_protocol::protocol::EpiphanyGraphFrontier {
            active_node_ids: vec!["node-1".to_string()],
            ..Default::default()
        }),
        ..Default::default()
    };
    let checkpoint = codex_protocol::protocol::EpiphanyInvestigationCheckpoint {
        checkpoint_id: "ix-1".to_string(),
        kind: "source_gathering".to_string(),
        disposition: codex_protocol::protocol::EpiphanyInvestigationDisposition::ResumeReady,
        focus: "Resume the bounded specialist prompt pass.".to_string(),
        summary: Some("The prompt seam was banked before compaction.".to_string()),
        ..Default::default()
    };
    let decision = ThreadEpiphanyReorientDecision {
        action: ThreadEpiphanyReorientAction::Resume,
        checkpoint_status: ThreadEpiphanyReorientCheckpointStatus::ResumeReady,
        checkpoint_id: Some("ix-1".to_string()),
        pressure_level: ThreadEpiphanyPressureLevel::Low,
        retrieval_status: ThreadEpiphanyRetrievalFreshnessStatus::Ready,
        graph_status: ThreadEpiphanyGraphFreshnessStatus::Ready,
        watcher_status: ThreadEpiphanyInvalidationStatus::Clean,
        reasons: vec![ThreadEpiphanyReorientReason::CheckpointReady],
        checkpoint_dirty_paths: Vec::new(),
        checkpoint_changed_paths: Vec::new(),
        active_frontier_node_ids: vec!["node-1".to_string()],
        next_action: "Continue after source recheck.".to_string(),
        note: "Checkpoint is warm enough to resume.".to_string(),
    };

    let request = build_epiphany_reorient_launch_request(
        "thr_123",
        Some(7),
        Some(90),
        &state,
        &checkpoint,
        &decision,
    );

    assert_eq!(request.binding_id, EPIPHANY_REORIENT_LAUNCH_BINDING_ID);
    assert_eq!(request.owner_role, EPIPHANY_REORIENT_OWNER_ROLE);
    assert_eq!(request.authority_scope, "epiphany.reorient.resume");
    let EpiphanyWorkerLaunchDocument::Reorient(reorient_document) = &request.launch_document else {
        panic!("reorient should build a reorient launch document");
    };
    assert_eq!(
        reorient_document
            .scratch
            .as_ref()
            .and_then(|scratch| scratch.summary.as_deref()),
        Some("Carry the current seam across compaction.")
    );
    assert!(reorient_document.graphs.is_some());
    assert!(reorient_document.recent_evidence.is_empty());
    assert!(reorient_document.recent_observations.is_empty());
    assert!(request.instruction.contains("Epiphany Persistent Memory"));
    assert!(request.instruction.contains("Life across sleep"));
    assert!(request.instruction.contains("checkpoint as the ember"));
    assert_eq!(
        request.output_contract_id,
        "epiphany.worker.reorient_result.v0"
    );
    let schema = epiphany_reorient_launch_output_schema();
    assert!(schema["properties"].get("openQuestions").is_some());
    assert!(schema["properties"].get("continuityRisks").is_some());

    let regather_instruction =
        build_epiphany_reorient_launch_instruction(ThreadEpiphanyReorientAction::Regather);
    assert!(regather_instruction.contains("Life returning after rupture"));
}

#[test]
fn build_epiphany_role_launch_request_rejects_untemplated_roles() {
    let state = codex_protocol::protocol::EpiphanyThreadState::default();

    assert!(
        build_epiphany_role_launch_request(
            "thr_123",
            ThreadEpiphanyRoleId::Implementation,
            None,
            None,
            &state,
        )
        .unwrap_err()
        .contains("main coding agent")
    );
    assert!(
        build_epiphany_role_launch_request(
            "thr_123",
            ThreadEpiphanyRoleId::Reorientation,
            None,
            None,
            &state,
        )
        .unwrap_err()
        .contains("reorientLaunch")
    );
}

#[test]
fn map_epiphany_role_finding_projects_structured_output() {
    let raw_result = serde_json::json!({
        "verdict": "pass",
        "summary": "The evidence covers the bounded slice.",
        "nextSafeMove": "Promote after human review.",
        "filesInspected": ["src/lib.rs"],
        "frontierNodeIds": ["node-1"],
        "evidenceIds": ["ev-1"],
        "artifactRefs": ["artifact:role"],
        "runtimeResultId": "result-1",
        "runtimeJobId": "job-1"
    });

    let finding = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Verification,
        raw_result.clone(),
        None,
        None,
    );

    assert_eq!(finding.role_id, ThreadEpiphanyRoleId::Verification);
    assert_eq!(finding.verdict.as_deref(), Some("pass"));
    assert_eq!(
        finding.next_safe_move.as_deref(),
        Some("Promote after human review.")
    );
    assert_eq!(finding.files_inspected, vec!["src/lib.rs"]);
    assert_eq!(finding.frontier_node_ids, vec!["node-1"]);
    assert_eq!(finding.evidence_ids, vec!["ev-1"]);
    assert_eq!(finding.artifact_refs, vec!["artifact:role"]);
    assert_eq!(finding.runtime_result_id.as_deref(), Some("result-1"));
    assert_eq!(finding.runtime_job_id.as_deref(), Some("job-1"));
}

#[test]
fn map_epiphany_role_finding_reviews_acceptable_self_patch() {
    let finding = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Modeling,
        serde_json::json!({
            "roleId": "modeling",
            "verdict": "checkpoint-ready",
            "summary": "The Body learned a durable modeling lesson.",
            "nextSafeMove": "Review the lane memory request.",
            "statePatch": {
                "scratch": {
                    "summary": "Source-grounded modeling checkpoint.",
                    "next_probe": "Run verification."
                }
            },
            "selfPatch": {
                "agentId": "epiphany.body",
                "reason": "The Body should remember graph growth must stay source-grounded and bounded.",
                "semanticMemories": [
                    {
                        "memoryId": "mem-body-source-grounded-growth",
                        "summary": "Grow graph and checkpoint state only when source evidence makes the anatomy harder to misread.",
                        "salience": 0.82,
                        "confidence": 0.9
                    }
                ]
            }
        }),
        None,
        None,
    );

    let review = finding.self_persistence.as_ref().unwrap();
    assert_eq!(
        review.status,
        ThreadEpiphanyRoleSelfPersistenceStatus::Accepted
    );
    assert_eq!(review.target_agent_id.as_deref(), Some("epiphany.body"));
    assert_eq!(
        review.target_path.as_deref(),
        Some("state/agents/body.agent-state.json")
    );
    assert!(review.reasons.is_empty());
}

#[test]
fn map_epiphany_role_finding_rejects_bad_self_patch() {
    let finding = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Verification,
        serde_json::json!({
            "roleId": "verification",
            "verdict": "needs-evidence",
            "summary": "The Soul saw a bad persistence request.",
            "nextSafeMove": "Ask the lane to reshape the memory request.",
            "selfPatch": {
                "agentId": "epiphany.body",
                "reason": "Too broad.",
                "graphs": {},
                "semanticMemories": [
                    {
                        "memoryId": "mem-soul-bad-project-truth",
                        "summary": "Project graph state belongs in memory now.",
                        "salience": 0.7,
                        "confidence": 0.4
                    }
                ]
            }
        }),
        None,
        None,
    );

    let review = finding.self_persistence.as_ref().unwrap();
    assert_eq!(
        review.status,
        ThreadEpiphanyRoleSelfPersistenceStatus::Rejected
    );
    assert_eq!(review.target_agent_id.as_deref(), Some("epiphany.soul"));
    assert!(
        review
            .reasons
            .iter()
            .any(|reason| reason.contains("expected \"epiphany.soul\""))
    );
    assert!(
        review
            .reasons
            .iter()
            .any(|reason| reason.contains("project truth"))
    );
}

#[test]
fn map_epiphany_role_finding_marks_modeling_without_patch_unreviewable() {
    let missing_patch = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Modeling,
        serde_json::json!({
            "roleId": "modeling",
            "verdict": "checkpoint-ready",
            "summary": "Source was inspected, but nothing durable was banked.",
            "nextSafeMove": "Relaunch modeling with a state patch."
        }),
        None,
        None,
    );

    assert!(missing_patch.state_patch.is_none());
    assert!(
        missing_patch
            .item_error
            .as_deref()
            .is_some_and(|error| error.contains("missing required statePatch"))
    );
    assert!(!epiphany_modeling_finding_has_reviewable_state_patch(
        &missing_patch
    ));

    let reviewable = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Modeling,
        serde_json::json!({
            "roleId": "modeling",
            "verdict": "checkpoint-ready",
            "summary": "The checkpoint is banked in scratch.",
            "nextSafeMove": "Review the modeling patch.",
            "statePatch": {
                "scratch": {
                    "summary": "Source-grounded modeling checkpoint.",
                    "hypothesis": "The current graph remains coherent.",
                    "next_probe": "Run verification."
                }
            }
        }),
        None,
        None,
    );

    assert!(reviewable.item_error.is_none());
    assert!(epiphany_modeling_finding_has_reviewable_state_patch(
        &reviewable
    ));

    let invalid_checkpoint_enum = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Modeling,
        serde_json::json!({
            "roleId": "modeling",
            "verdict": "checkpoint-ready",
            "summary": "The checkpoint enum used the verdict spelling.",
            "nextSafeMove": "Use the typed checkpoint enum.",
            "statePatch": {
                "investigationCheckpoint": {
                    "checkpoint_id": "ix-1",
                    "kind": "source_gathering",
                    "disposition": "checkpoint_ready",
                    "focus": "Model the seam."
                }
            }
        }),
        None,
        None,
    );
    assert!(
        invalid_checkpoint_enum
            .item_error
            .as_deref()
            .is_some_and(|error| error.contains("checkpoint_ready"))
    );
    assert!(!epiphany_modeling_finding_has_reviewable_state_patch(
        &invalid_checkpoint_enum
    ));
}

#[test]
fn map_epiphany_role_finding_marks_imagination_without_planning_patch_unreviewable() {
    let missing_patch = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Imagination,
        serde_json::json!({
            "roleId": "imagination",
            "verdict": "draft-ready",
            "summary": "A plan was described but no durable planning patch was returned.",
            "nextSafeMove": "Relaunch imagination with a state patch."
        }),
        None,
        None,
    );

    assert!(missing_patch.state_patch.is_none());
    assert!(
        missing_patch
            .item_error
            .as_deref()
            .is_some_and(|error| error.contains("missing required statePatch"))
    );
    assert!(!epiphany_imagination_finding_has_reviewable_state_patch(
        &missing_patch
    ));

    let reviewable = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Imagination,
        serde_json::json!({
            "roleId": "imagination",
            "verdict": "draft-ready",
            "summary": "Planning material was synthesized into a draft objective.",
            "nextSafeMove": "Review and optionally adopt the draft.",
            "statePatch": {
                "planning": {
                    "captures": [
                        {
                            "id": "capture-imagination-test",
                            "title": "Planning source",
                            "confidence": "medium",
                            "status": "triaged",
                            "source": {"kind": "chat", "uri": "codex://threads/test"}
                        }
                    ],
                    "backlog_items": [
                        {
                            "id": "backlog-imagination-test",
                            "title": "Synthesized planning seam",
                            "kind": "feature",
                            "summary": "Prove Imagination can bank a reviewable objective draft.",
                            "status": "ready",
                            "horizon": "near",
                            "priority": {"value": "medium", "rationale": "Smoke coverage"},
                            "confidence": "medium",
                            "product_area": "planning",
                            "source_refs": [{"kind": "chat", "uri": "codex://threads/test"}]
                        }
                    ],
                    "objective_drafts": [
                        {
                            "id": "draft-imagination-test",
                            "title": "Review Imagination patch",
                            "summary": "Accept a planning-only role patch after review.",
                            "source_item_ids": ["backlog-imagination-test"],
                            "scope": {
                                "includes": ["planning"],
                                "excludes": ["objective adoption", "implementation"]
                            },
                            "acceptance_criteria": ["A planning patch can be accepted without adopting the draft."],
                            "evidence_required": ["role finding"],
                            "lane_plan": {"imagination": "synthesize", "soul": "review"},
                            "risks": ["accidental adoption"],
                            "review_gates": ["human adoption review"],
                            "status": "draft"
                        }
                    ]
                }
            }
        }),
        None,
        None,
    );

    assert!(reviewable.item_error.is_none());
    assert!(epiphany_imagination_finding_has_reviewable_state_patch(
        &reviewable
    ));

    let missing_review_gate = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Imagination,
        serde_json::json!({
            "roleId": "imagination",
            "verdict": "draft-ready",
            "summary": "The draft forgot its gate.",
            "nextSafeMove": "Add review gates before accepting.",
            "statePatch": {
                "planning": {
                    "objective_drafts": [
                        {
                            "id": "draft-no-gate",
                            "title": "Ungated draft",
                            "summary": "This must not be accepted.",
                            "scope": {},
                            "acceptance_criteria": ["It has criteria."],
                            "lane_plan": {},
                            "status": "draft"
                        }
                    ]
                }
            }
        }),
        None,
        None,
    );
    assert!(
        missing_review_gate
            .item_error
            .as_deref()
            .is_some_and(|error| error.contains("review gates"))
    );
    assert!(!epiphany_imagination_finding_has_reviewable_state_patch(
        &missing_review_gate
    ));
}

#[test]
fn role_finding_accepted_after_uses_newest_first_evidence_order() {
    let modeling = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Modeling,
        serde_json::json!({
            "roleId": "modeling",
            "verdict": "checkpoint-update-needed",
            "summary": "Newer modeling checkpoint.",
            "nextSafeMove": "Verify the new model.",
            "runtimeResultId": "runtime-result-modeling-new",
            "runtimeJobId": "runtime-job-modeling-new",
            "statePatch": {
                "scratch": {
                    "summary": "New model.",
                    "next_probe": "Verify."
                }
            }
        }),
        None,
        None,
    );
    let verification = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Verification,
        serde_json::json!({
            "roleId": "verification",
            "verdict": "needs-evidence",
            "summary": "Older verification finding.",
            "nextSafeMove": "Strengthen modeling.",
            "runtimeResultId": "runtime-result-verification-old",
            "runtimeJobId": "runtime-job-verification-old"
        }),
        None,
        None,
    );
    let state = EpiphanyThreadState {
        acceptance_receipts: vec![
            EpiphanyAcceptanceReceipt {
                id: "accept-modeling-new".to_string(),
                result_id: "runtime-result-modeling-new".to_string(),
                job_id: "runtime-job-modeling-new".to_string(),
                binding_id: "modeling".to_string(),
                surface: "roleAccept".to_string(),
                role_id: "modeling".to_string(),
                status: "accepted".to_string(),
                accepted_at: "2026-05-12T00:00:01Z".to_string(),
                accepted_observation_id: Some("obs-modeling-new".to_string()),
                accepted_evidence_id: Some("ev-modeling-new".to_string()),
                summary: Some("Newer modeling checkpoint.".to_string()),
            },
            EpiphanyAcceptanceReceipt {
                id: "accept-verification-old".to_string(),
                result_id: "runtime-result-verification-old".to_string(),
                job_id: "runtime-job-verification-old".to_string(),
                binding_id: "verification".to_string(),
                surface: "roleAccept".to_string(),
                role_id: "verification".to_string(),
                status: "accepted".to_string(),
                accepted_at: "2026-05-12T00:00:00Z".to_string(),
                accepted_observation_id: Some("obs-verification-old".to_string()),
                accepted_evidence_id: Some("ev-verification-old".to_string()),
                summary: Some("Older verification finding.".to_string()),
            },
        ],
        recent_evidence: vec![
            EpiphanyEvidenceRecord {
                id: "ev-modeling-new".to_string(),
                kind: "modeling_result".to_string(),
                status: "accepted".to_string(),
                summary: role_finding_summary(&modeling),
                ..Default::default()
            },
            EpiphanyEvidenceRecord {
                id: "ev-verification-old".to_string(),
                kind: "verification_result".to_string(),
                status: "accepted".to_string(),
                summary: role_finding_summary(&verification),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    assert!(role_finding_accepted_after(
        &state,
        Some(&modeling),
        Some(&verification)
    ));
    assert!(!role_finding_accepted_after(
        &state,
        Some(&verification),
        Some(&modeling)
    ));
}

#[test]
fn role_finding_acceptance_prefers_runtime_result_receipt() {
    let modeling = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Modeling,
        serde_json::json!({
            "roleId": "modeling",
            "verdict": "checkpoint-ready",
            "summary": "Runtime receipt owns identity.",
            "nextSafeMove": "Verify from the receipt.",
            "runtimeResultId": "runtime-result-1",
            "runtimeJobId": "runtime-job-1",
            "statePatch": {
                "scratch": {
                    "summary": "Receipt-backed model.",
                    "next_probe": "Verify."
                }
            }
        }),
        None,
        None,
    );
    let state = EpiphanyThreadState {
        acceptance_receipts: vec![EpiphanyAcceptanceReceipt {
            id: "accept-modeling-runtime-result-1".to_string(),
            result_id: "runtime-result-1".to_string(),
            job_id: "runtime-job-1".to_string(),
            binding_id: "modeling".to_string(),
            surface: "roleAccept".to_string(),
            role_id: "modeling".to_string(),
            status: "accepted".to_string(),
            accepted_at: "2026-05-12T00:00:00Z".to_string(),
            accepted_observation_id: Some("obs-modeling".to_string()),
            accepted_evidence_id: Some("ev-modeling".to_string()),
            summary: Some("Summary text can drift without owning identity.".to_string()),
        }],
        recent_evidence: vec![EpiphanyEvidenceRecord {
            id: "ev-modeling".to_string(),
            kind: "modeling_result".to_string(),
            status: "accepted".to_string(),
            summary: "Different summary; receipt should still match.".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    assert!(epiphany_role_finding_already_accepted(&state, &modeling));
    assert_eq!(
        epiphany_role_finding_accepted_evidence_id(&state, &modeling).as_deref(),
        Some("ev-modeling")
    );
}

#[test]
fn verification_coverage_accepts_modeling_source_evidence_ids() {
    let modeling = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Modeling,
        serde_json::json!({
            "roleId": "modeling",
            "verdict": "checkpoint-update-needed",
            "summary": "Modeling found the source seam.",
            "nextSafeMove": "Verify the modeled seam.",
            "evidenceIds": ["ev-model-source"],
            "runtimeResultId": "runtime-result-model-source",
            "runtimeJobId": "runtime-job-model-source",
            "statePatch": {
                "scratch": {
                    "summary": "Modeling checkpoint.",
                    "next_probe": "Verify."
                }
            }
        }),
        None,
        None,
    );
    let verification = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Verification,
        serde_json::json!({
            "roleId": "verification",
            "verdict": "needs-evidence",
            "summary": "Verification covered the modeler's source evidence.",
            "nextSafeMove": "Gather implementation evidence.",
            "evidenceIds": ["ev-model-source"]
        }),
        None,
        None,
    );
    let stale_verification = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Verification,
        serde_json::json!({
            "roleId": "verification",
            "verdict": "needs-evidence",
            "summary": "Verification covered unrelated evidence.",
            "nextSafeMove": "Gather implementation evidence.",
            "evidenceIds": ["ev-other"]
        }),
        None,
        None,
    );
    let state = EpiphanyThreadState {
        acceptance_receipts: vec![EpiphanyAcceptanceReceipt {
            id: "accept-modeling-source".to_string(),
            result_id: "runtime-result-model-source".to_string(),
            job_id: "runtime-job-model-source".to_string(),
            binding_id: "modeling".to_string(),
            surface: "roleAccept".to_string(),
            role_id: "modeling".to_string(),
            status: "accepted".to_string(),
            accepted_at: "2026-05-12T00:00:00Z".to_string(),
            accepted_observation_id: Some("obs-modeling-accepted".to_string()),
            accepted_evidence_id: Some("ev-modeling-accepted".to_string()),
            summary: Some("Modeling found the source seam.".to_string()),
        }],
        recent_evidence: vec![EpiphanyEvidenceRecord {
            id: "ev-modeling-accepted".to_string(),
            kind: "modeling_result".to_string(),
            status: "accepted".to_string(),
            summary: role_finding_summary(&modeling),
            ..Default::default()
        }],
        ..Default::default()
    };

    assert!(epiphany_verification_finding_covers_current_modeling(
        &state,
        true,
        Some(&modeling),
        Some(&verification)
    ));
    assert!(epiphany_verification_finding_covers_current_modeling(
        &state,
        true,
        Some(&modeling),
        Some(&map_epiphany_role_finding(
            ThreadEpiphanyRoleId::Verification,
            serde_json::json!({
                "roleId": "verification",
                "verdict": "needs-evidence",
                "summary": "Verification covered the accepted wrapper evidence.",
                "nextSafeMove": "Gather implementation evidence.",
                "evidenceIds": ["ev-modeling-accepted"]
            }),
            None,
            None,
        ))
    ));
    assert!(!epiphany_verification_finding_covers_current_modeling(
        &state,
        true,
        Some(&modeling),
        Some(&stale_verification)
    ));
}

#[test]
fn implementation_evidence_after_verification_uses_latest_audit() {
    let verification = map_epiphany_role_finding(
        ThreadEpiphanyRoleId::Verification,
        serde_json::json!({
            "roleId": "verification",
            "verdict": "needs-evidence",
            "summary": "Need implementation evidence.",
            "nextSafeMove": "Gather evidence."
        }),
        None,
        None,
    );
    let mut state = EpiphanyThreadState {
        recent_evidence: vec![
            EpiphanyEvidenceRecord {
                id: "ev-implementation-blocked".to_string(),
                kind: "implementation-audit".to_string(),
                status: "blocked".to_string(),
                summary: "No new diff.".to_string(),
                ..Default::default()
            },
            EpiphanyEvidenceRecord {
                id: "ev-implementation-ok".to_string(),
                kind: "implementation-audit".to_string(),
                status: "ok".to_string(),
                summary: "Older diff.".to_string(),
                ..Default::default()
            },
            EpiphanyEvidenceRecord {
                id: "ev-verification".to_string(),
                kind: "verification_result".to_string(),
                status: "accepted".to_string(),
                summary: role_finding_summary(&verification),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    assert!(!implementation_evidence_after_role_finding(
        &state,
        Some(&verification)
    ));
    state.recent_evidence[0].status = "ok".to_string();
    assert!(implementation_evidence_after_role_finding(
        &state,
        Some(&verification)
    ));
}

#[test]
fn reorient_finding_acceptance_builds_scratch_and_checkpoint_update() {
    let finding = ThreadEpiphanyReorientFinding {
        mode: Some("regather".to_string()),
        summary: Some("The old checkpoint no longer matches source.".to_string()),
        next_safe_move: Some("Re-read src/lib.rs before editing.".to_string()),
        checkpoint_still_valid: Some(false),
        files_inspected: vec!["src/lib.rs".to_string()],
        frontier_node_ids: vec!["node-1".to_string()],
        evidence_ids: Vec::new(),
        artifact_refs: Vec::new(),
        runtime_result_id: None,
        runtime_job_id: None,
        job_error: None,
        item_error: None,
    };

    let scratch = reorient_finding_scratch("reorient-worker", &finding);
    assert_eq!(
        scratch.summary.as_deref(),
        Some("The old checkpoint no longer matches source.")
    );
    assert_eq!(
        scratch.next_probe.as_deref(),
        Some("Re-read src/lib.rs before editing.")
    );
    assert!(
        scratch
            .hypothesis
            .as_deref()
            .is_some_and(|text| text.contains("checkpoint validity is invalid"))
    );

    let checkpoint = EpiphanyInvestigationCheckpoint {
        checkpoint_id: "ix-1".to_string(),
        kind: "source_gathering".to_string(),
        disposition: EpiphanyInvestigationDisposition::ResumeReady,
        focus: "Old focus".to_string(),
        ..Default::default()
    };
    let code_refs = reorient_finding_code_refs(&finding);
    let updated = reorient_finding_investigation_checkpoint(
        &checkpoint,
        "ev-reorient-1",
        &code_refs,
        &finding,
    );

    assert_eq!(
        updated.disposition,
        EpiphanyInvestigationDisposition::ResumeReady
    );
    assert_eq!(updated.evidence_ids, vec!["ev-reorient-1"]);
    assert_eq!(updated.code_refs[0].path, PathBuf::from("src/lib.rs"));
    assert_eq!(
        updated.next_action.as_deref(),
        Some("Re-read src/lib.rs before editing.")
    );
}

#[test]
fn map_epiphany_context_projects_targeted_shard_without_mutation() {
    let state = codex_protocol::protocol::EpiphanyThreadState {
        revision: 7,
        graphs: codex_protocol::protocol::EpiphanyGraphs {
            architecture: codex_protocol::protocol::EpiphanyGraph {
                nodes: vec![
                    codex_protocol::protocol::EpiphanyGraphNode {
                        id: "active-node".to_string(),
                        title: "Active node".to_string(),
                        purpose: "Current frontier focus".to_string(),
                        ..Default::default()
                    },
                    codex_protocol::protocol::EpiphanyGraphNode {
                        id: "manual-node".to_string(),
                        title: "Manual node".to_string(),
                        purpose: "Explicit context selector".to_string(),
                        ..Default::default()
                    },
                    codex_protocol::protocol::EpiphanyGraphNode {
                        id: "ignored-node".to_string(),
                        title: "Ignored node".to_string(),
                        purpose: "Outside the requested shard".to_string(),
                        ..Default::default()
                    },
                ],
                edges: vec![
                    codex_protocol::protocol::EpiphanyGraphEdge {
                        id: Some("edge-active".to_string()),
                        source_id: "active-node".to_string(),
                        target_id: "ignored-node".to_string(),
                        kind: "frontier".to_string(),
                        ..Default::default()
                    },
                    codex_protocol::protocol::EpiphanyGraphEdge {
                        id: Some("edge-manual".to_string()),
                        source_id: "manual-node".to_string(),
                        target_id: "active-node".to_string(),
                        kind: "selected".to_string(),
                        ..Default::default()
                    },
                ],
            },
            dataflow: codex_protocol::protocol::EpiphanyGraph {
                nodes: vec![codex_protocol::protocol::EpiphanyGraphNode {
                    id: "data-node".to_string(),
                    title: "Data node".to_string(),
                    purpose: "Linked dataflow context".to_string(),
                    ..Default::default()
                }],
                edges: Vec::new(),
            },
            links: vec![codex_protocol::protocol::EpiphanyGraphLink {
                dataflow_node_id: "data-node".to_string(),
                architecture_node_id: "active-node".to_string(),
                relationship: Some("supports".to_string()),
                code_refs: Vec::new(),
            }],
        },
        graph_frontier: Some(codex_protocol::protocol::EpiphanyGraphFrontier {
            active_node_ids: vec!["active-node".to_string()],
            active_edge_ids: vec!["edge-active".to_string()],
            ..Default::default()
        }),
        graph_checkpoint: Some(codex_protocol::protocol::EpiphanyGraphCheckpoint {
            checkpoint_id: "ck-context".to_string(),
            graph_revision: 7,
            summary: Some("Context shard checkpoint".to_string()),
            ..Default::default()
        }),
        investigation_checkpoint: Some(codex_protocol::protocol::EpiphanyInvestigationCheckpoint {
            checkpoint_id: "ix-context".to_string(),
            kind: "source_gathering".to_string(),
            focus: "Trace the exact reflection seam.".to_string(),
            next_action: Some("Re-gather source before editing if this goes stale.".to_string()),
            evidence_ids: vec!["ev-linked".to_string()],
            ..Default::default()
        }),
        observations: vec![codex_protocol::protocol::EpiphanyObservation {
            id: "obs-1".to_string(),
            summary: "Observation carries linked evidence.".to_string(),
            source_kind: "test".to_string(),
            status: "ok".to_string(),
            evidence_ids: vec!["ev-linked".to_string()],
            ..Default::default()
        }],
        recent_evidence: vec![
            codex_protocol::protocol::EpiphanyEvidenceRecord {
                id: "ev-linked".to_string(),
                kind: "test".to_string(),
                status: "ok".to_string(),
                summary: "Linked from observation.".to_string(),
                ..Default::default()
            },
            codex_protocol::protocol::EpiphanyEvidenceRecord {
                id: "ev-extra".to_string(),
                kind: "review".to_string(),
                status: "ok".to_string(),
                summary: "Requested directly.".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    let params = ThreadEpiphanyContextParams {
        thread_id: "thr_123".to_string(),
        graph_node_ids: vec!["manual-node".to_string(), "missing-node".to_string()],
        graph_edge_ids: vec!["edge-manual".to_string(), "missing-edge".to_string()],
        observation_ids: vec!["obs-1".to_string(), "obs-missing".to_string()],
        evidence_ids: vec!["ev-extra".to_string(), "ev-missing".to_string()],
        include_active_frontier: None,
        include_linked_evidence: None,
    };

    let (state_status, state_revision, context, missing) =
        map_epiphany_context(Some(&state), &params);

    assert_eq!(state_status, ThreadEpiphanyContextStateStatus::Ready);
    assert_eq!(state_revision, Some(7));
    assert_eq!(
        context
            .graph
            .architecture_nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<Vec<_>>(),
        vec!["active-node", "manual-node"]
    );
    assert_eq!(
        context
            .graph
            .architecture_edges
            .iter()
            .filter_map(|edge| edge.id.as_deref())
            .collect::<Vec<_>>(),
        vec!["edge-active", "edge-manual"]
    );
    assert_eq!(context.graph.links.len(), 1);
    assert_eq!(
        context
            .frontier
            .as_ref()
            .map(|frontier| frontier.active_node_ids.as_slice()),
        Some(&["active-node".to_string()][..])
    );
    assert_eq!(
        context
            .checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_id.as_str()),
        Some("ck-context")
    );
    assert_eq!(
        context
            .investigation_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.checkpoint_id.as_str()),
        Some("ix-context")
    );
    assert_eq!(
        context
            .observations
            .iter()
            .map(|observation| observation.id.as_str())
            .collect::<Vec<_>>(),
        vec!["obs-1"]
    );
    assert_eq!(
        context
            .evidence
            .iter()
            .map(|evidence| evidence.id.as_str())
            .collect::<Vec<_>>(),
        vec!["ev-linked", "ev-extra"]
    );
    assert_eq!(missing.graph_node_ids, vec!["missing-node".to_string()]);
    assert_eq!(missing.graph_edge_ids, vec!["missing-edge".to_string()]);
    assert_eq!(missing.observation_ids, vec!["obs-missing".to_string()]);
    assert_eq!(missing.evidence_ids, vec!["ev-missing".to_string()]);
}

#[test]
fn map_epiphany_context_reports_missing_state_without_inventing_context() {
    let params = ThreadEpiphanyContextParams {
        thread_id: "thr_123".to_string(),
        graph_node_ids: vec!["node-1".to_string()],
        graph_edge_ids: Vec::new(),
        observation_ids: vec!["obs-1".to_string()],
        evidence_ids: Vec::new(),
        include_active_frontier: None,
        include_linked_evidence: None,
    };

    let (state_status, state_revision, context, missing) = map_epiphany_context(None, &params);

    assert_eq!(state_status, ThreadEpiphanyContextStateStatus::Missing);
    assert_eq!(state_revision, None);
    assert_eq!(context, ThreadEpiphanyContext::default());
    assert_eq!(missing.graph_node_ids, vec!["node-1".to_string()]);
    assert_eq!(missing.observation_ids, vec!["obs-1".to_string()]);
}

#[test]
fn map_epiphany_planning_projects_counts_without_mutation() {
    let github_source = codex_protocol::protocol::EpiphanyPlanningSourceRef {
        kind: "github_issue".to_string(),
        provider: Some("github".to_string()),
        repo: Some("GameCult/Epiphany".to_string()),
        issue_number: Some(7),
        ..Default::default()
    };
    let state = codex_protocol::protocol::EpiphanyThreadState {
        revision: 12,
        objective: Some("Current adopted objective".to_string()),
        planning: codex_protocol::protocol::EpiphanyPlanningState {
            captures: vec![
                codex_protocol::protocol::EpiphanyPlanningCapture {
                    id: "capture-gh-7".to_string(),
                    title: "GitHub issue import".to_string(),
                    confidence: "medium".to_string(),
                    status: "new".to_string(),
                    source: github_source.clone(),
                    ..Default::default()
                },
                codex_protocol::protocol::EpiphanyPlanningCapture {
                    id: "capture-chat-1".to_string(),
                    title: "Human planning note".to_string(),
                    confidence: "low".to_string(),
                    status: "triaged".to_string(),
                    source: codex_protocol::protocol::EpiphanyPlanningSourceRef {
                        kind: "chat".to_string(),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ],
            backlog_items: vec![codex_protocol::protocol::EpiphanyBacklogItem {
                id: "backlog-planning-view".to_string(),
                title: "Build planning view".to_string(),
                kind: "feature".to_string(),
                summary: "Expose planning state to the GUI.".to_string(),
                status: "ready".to_string(),
                horizon: "now".to_string(),
                priority: codex_protocol::protocol::EpiphanyPlanningPriority {
                    value: "p1".to_string(),
                    rationale: "Needed before planning can be operated.".to_string(),
                    ..Default::default()
                },
                confidence: "high".to_string(),
                product_area: "gui".to_string(),
                source_refs: vec![github_source],
                ..Default::default()
            }],
            roadmap_streams: vec![codex_protocol::protocol::EpiphanyRoadmapStream {
                id: "stream-gui".to_string(),
                title: "GUI Operator Surface".to_string(),
                purpose: "Let the human inspect and steer Epiphany.".to_string(),
                status: "active".to_string(),
                item_ids: vec!["backlog-planning-view".to_string()],
                ..Default::default()
            }],
            objective_drafts: vec![codex_protocol::protocol::EpiphanyObjectiveDraft {
                id: "objdraft-planning-view".to_string(),
                title: "Build planning view slice".to_string(),
                summary: "Render typed planning state in the GUI.".to_string(),
                source_item_ids: vec!["backlog-planning-view".to_string()],
                acceptance_criteria: vec!["Planning counts render.".to_string()],
                status: "draft".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        },
        ..Default::default()
    };

    let (state_status, state_revision, planning, summary) = map_epiphany_planning(Some(&state));

    assert_eq!(state_status, ThreadEpiphanyContextStateStatus::Ready);
    assert_eq!(state_revision, Some(12));
    assert_eq!(planning.captures.len(), 2);
    assert_eq!(summary.capture_count, 2);
    assert_eq!(summary.pending_capture_count, 1);
    assert_eq!(summary.github_issue_capture_count, 1);
    assert_eq!(summary.backlog_item_count, 1);
    assert_eq!(summary.ready_backlog_item_count, 1);
    assert_eq!(summary.roadmap_stream_count, 1);
    assert_eq!(summary.objective_draft_count, 1);
    assert_eq!(summary.draft_objective_count, 1);
    assert_eq!(
        summary.active_objective.as_deref(),
        Some("Current adopted objective")
    );
    assert!(
        summary
            .note
            .contains("planning state only until a human explicitly adopts")
    );
}

#[test]
fn map_epiphany_planning_reports_missing_state_without_inventing_backlog() {
    let (state_status, state_revision, planning, summary) = map_epiphany_planning(None);

    assert_eq!(state_status, ThreadEpiphanyContextStateStatus::Missing);
    assert_eq!(state_revision, None);
    assert!(planning.is_empty());
    assert_eq!(summary.capture_count, 0);
    assert_eq!(summary.backlog_item_count, 0);
    assert_eq!(summary.active_objective, None);
}

#[test]
fn map_epiphany_graph_query_returns_frontier_neighborhood() {
    let state = codex_protocol::protocol::EpiphanyThreadState {
        revision: 3,
        graphs: codex_protocol::protocol::EpiphanyGraphs {
            architecture: codex_protocol::protocol::EpiphanyGraph {
                nodes: vec![
                    codex_protocol::protocol::EpiphanyGraphNode {
                        id: "operator-console".to_string(),
                        title: "Operator console".to_string(),
                        purpose: "Expose bounded controls.".to_string(),
                        code_refs: vec![codex_protocol::protocol::EpiphanyCodeRef {
                            path: PathBuf::from("apps/epiphany-gui/src/App.tsx"),
                            symbol: Some("App".to_string()),
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                    codex_protocol::protocol::EpiphanyGraphNode {
                        id: "action-bridge".to_string(),
                        title: "Action bridge".to_string(),
                        purpose: "Call app-server APIs.".to_string(),
                        ..Default::default()
                    },
                ],
                edges: vec![codex_protocol::protocol::EpiphanyGraphEdge {
                    id: Some("edge-console-bridge".to_string()),
                    source_id: "operator-console".to_string(),
                    target_id: "action-bridge".to_string(),
                    kind: "invokes".to_string(),
                    ..Default::default()
                }],
            },
            dataflow: codex_protocol::protocol::EpiphanyGraph {
                nodes: vec![codex_protocol::protocol::EpiphanyGraphNode {
                    id: "operator-action-flow".to_string(),
                    title: "Operator action flow".to_string(),
                    purpose: "Button to Tauri to Python to app-server.".to_string(),
                    ..Default::default()
                }],
                edges: Vec::new(),
            },
            links: vec![codex_protocol::protocol::EpiphanyGraphLink {
                dataflow_node_id: "operator-action-flow".to_string(),
                architecture_node_id: "operator-console".to_string(),
                relationship: Some("starts-at".to_string()),
                code_refs: Vec::new(),
            }],
        },
        graph_frontier: Some(codex_protocol::protocol::EpiphanyGraphFrontier {
            active_node_ids: vec!["operator-console".to_string()],
            ..Default::default()
        }),
        ..Default::default()
    };

    let query = ThreadEpiphanyGraphQuery {
        kind: ThreadEpiphanyGraphQueryKind::FrontierNeighborhood,
        node_ids: Vec::new(),
        edge_ids: Vec::new(),
        paths: Vec::new(),
        symbols: Vec::new(),
        edge_kinds: Vec::new(),
        direction: Some(ThreadEpiphanyGraphQueryDirection::Outgoing),
        depth: Some(1),
        include_links: None,
    };

    let (state_status, state_revision, graph, frontier, _checkpoint, matched, missing) =
        map_epiphany_graph_query(Some(&state), &query);

    assert_eq!(state_status, ThreadEpiphanyContextStateStatus::Ready);
    assert_eq!(state_revision, Some(3));
    assert_eq!(
        graph
            .architecture_nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<Vec<_>>(),
        vec!["operator-console", "action-bridge"]
    );
    assert_eq!(
        graph
            .dataflow_nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<Vec<_>>(),
        vec!["operator-action-flow"]
    );
    assert_eq!(
        graph
            .architecture_edges
            .iter()
            .filter_map(|edge| edge.id.as_deref())
            .collect::<Vec<_>>(),
        vec!["edge-console-bridge"]
    );
    assert_eq!(graph.links.len(), 1);
    assert_eq!(
        frontier
            .as_ref()
            .map(|frontier| frontier.active_node_ids.as_slice()),
        Some(&["operator-console".to_string()][..])
    );
    assert_eq!(matched.edge_ids, vec!["edge-console-bridge".to_string()]);
    assert!(missing.node_ids.is_empty());
    assert!(missing.edge_ids.is_empty());
}

#[test]
fn map_epiphany_jobs_reflects_current_progress_without_scheduling_work() {
    let state = codex_protocol::protocol::EpiphanyThreadState {
        active_subgoal_id: Some("phase-6".to_string()),
        invariants: vec![
            codex_protocol::protocol::EpiphanyInvariant {
                id: "inv-ready".to_string(),
                description: "State reads are read-only".to_string(),
                status: "ok".to_string(),
                rationale: None,
            },
            codex_protocol::protocol::EpiphanyInvariant {
                id: "inv-review".to_string(),
                description: "Job state needs review".to_string(),
                status: "needs_review".to_string(),
                rationale: None,
            },
        ],
        graph_frontier: Some(codex_protocol::protocol::EpiphanyGraphFrontier {
            active_node_ids: vec!["retrieval".to_string()],
            active_edge_ids: Vec::new(),
            open_question_ids: vec!["q-jobs".to_string()],
            open_gap_ids: Vec::new(),
            dirty_paths: vec![PathBuf::from("notes/jobs.md")],
        }),
        retrieval: Some(codex_protocol::protocol::EpiphanyRetrievalState {
            workspace_root: test_path_buf("/repo"),
            status: codex_protocol::protocol::EpiphanyRetrievalStatus::Stale,
            indexed_file_count: Some(12),
            indexed_chunk_count: Some(34),
            last_indexed_at_unix_seconds: Some(1_744_500_100),
            dirty_paths: vec![PathBuf::from("src/lib.rs")],
            ..Default::default()
        }),
        churn: Some(codex_protocol::protocol::EpiphanyChurnState {
            graph_freshness: Some("stale".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let jobs = map_epiphany_jobs(Some(&state), None);

    assert_eq!(jobs.len(), 3);
    assert_eq!(jobs[0].id, "retrieval-index");
    assert_eq!(jobs[0].kind, ThreadEpiphanyJobKind::Indexing);
    assert_eq!(jobs[0].status, ThreadEpiphanyJobStatus::Needed);
    assert_eq!(jobs[0].items_processed, Some(12));
    assert_eq!(jobs[0].last_checkpoint_at_unix_seconds, Some(1_744_500_100));
    assert_eq!(jobs[0].linked_subgoal_ids, vec!["phase-6".to_string()]);
    assert_eq!(jobs[0].linked_graph_node_ids, vec!["retrieval".to_string()]);

    assert_eq!(jobs[1].id, "graph-remap");
    assert_eq!(jobs[1].kind, ThreadEpiphanyJobKind::Remap);
    assert_eq!(jobs[1].status, ThreadEpiphanyJobStatus::Needed);
    assert_eq!(jobs[1].linked_graph_node_ids, vec!["retrieval".to_string()]);

    assert_eq!(jobs[2].id, "verification");
    assert_eq!(jobs[2].kind, ThreadEpiphanyJobKind::Verification);
    assert_eq!(jobs[2].status, ThreadEpiphanyJobStatus::Needed);
    assert_eq!(jobs[2].items_processed, Some(1));
    assert_eq!(jobs[2].items_total, Some(2));

    assert_eq!(jobs.len(), 3);
    assert!(!jobs.iter().any(|job| job.id == "specialist-work"));
}

#[test]
fn map_epiphany_jobs_can_reflect_retrieval_without_epiphany_state() {
    let retrieval = codex_protocol::protocol::EpiphanyRetrievalState {
        workspace_root: test_path_buf("/repo"),
        status: codex_protocol::protocol::EpiphanyRetrievalStatus::Ready,
        indexed_file_count: Some(7),
        ..Default::default()
    };

    let jobs = map_epiphany_jobs(None, Some(&retrieval));

    assert_eq!(jobs[0].status, ThreadEpiphanyJobStatus::Idle);
    assert_eq!(jobs[0].items_processed, Some(7));
    assert_eq!(jobs[1].status, ThreadEpiphanyJobStatus::Blocked);
    assert_eq!(jobs[2].status, ThreadEpiphanyJobStatus::Blocked);
}

#[test]
fn map_epiphany_jobs_projects_heartbeat_binding_as_pending() {
    let state = codex_protocol::protocol::EpiphanyThreadState {
        job_bindings: vec![codex_protocol::protocol::EpiphanyJobBinding {
            id: "modeling-checkpoint-worker".to_string(),
            kind: codex_protocol::protocol::EpiphanyJobKind::Specialist,
            scope: "role-scoped modeling/checkpoint maintenance".to_string(),
            owner_role: "epiphany-modeler".to_string(),
            authority_scope: Some("epiphany.role.modeling".to_string()),
            linked_subgoal_ids: vec!["phase-6".to_string()],
            linked_graph_node_ids: vec!["runtime-spine".to_string()],
            blocking_reason: None,
        }],
        runtime_links: vec![codex_protocol::protocol::EpiphanyRuntimeLink {
            id: "runtime-link-modeling-checkpoint-worker-heartbeat-turn-1".to_string(),
            binding_id: "modeling-checkpoint-worker".to_string(),
            surface: "roleLaunch".to_string(),
            role_id: "epiphany-modeler".to_string(),
            authority_scope: "epiphany.role.modeling".to_string(),
            runtime_job_id: "heartbeat-turn-1".to_string(),
            runtime_result_id: None,
            linked_subgoal_ids: vec!["phase-6".to_string()],
            linked_graph_node_ids: vec!["runtime-spine".to_string()],
        }],
        ..Default::default()
    };

    let jobs = map_epiphany_jobs(Some(&state), None);
    let specialist = jobs
        .iter()
        .find(|job| job.id == "modeling-checkpoint-worker")
        .expect("heartbeat specialist slot should exist");

    assert_eq!(specialist.status, ThreadEpiphanyJobStatus::Pending);
    assert_eq!(
        specialist.backend_job_id.as_deref(),
        Some("heartbeat-turn-1")
    );
    assert_eq!(specialist.blocking_reason, None);
    assert!(
        specialist
            .progress_note
            .as_deref()
            .is_some_and(|note| note.contains("heartbeat activation"))
    );
}

#[test]
fn load_epiphany_role_result_reads_heartbeat_runtime_spine_result() {
    let temp = tempfile::tempdir().expect("tempdir");
    let store = temp.path().join("runtime.msgpack");
    epiphany_core::initialize_runtime_spine(
        &store,
        epiphany_core::RuntimeSpineInitOptions {
            runtime_id: "epiphany-test".to_string(),
            display_name: "Epiphany Test".to_string(),
            created_at: "2026-05-06T00:00:00Z".to_string(),
        },
    )
    .expect("runtime identity");
    epiphany_core::create_runtime_session(
        &store,
        epiphany_core::RuntimeSpineSessionOptions {
            session_id: "epiphany-main".to_string(),
            objective: "Read typed runtime results.".to_string(),
            created_at: "2026-05-06T00:01:00Z".to_string(),
            coordinator_note: "test".to_string(),
        },
    )
    .expect("runtime session");
    epiphany_core::create_runtime_job(
        &store,
        epiphany_core::RuntimeSpineJobOptions {
            job_id: "heartbeat-job-1".to_string(),
            session_id: "epiphany-main".to_string(),
            role: "epiphany-modeler".to_string(),
            created_at: "2026-05-06T00:02:00Z".to_string(),
            summary: "queued".to_string(),
            artifact_refs: Vec::new(),
        },
    )
    .expect("runtime job");
    epiphany_core::complete_runtime_job(
        &store,
        epiphany_core::RuntimeSpineJobResultOptions {
            result_id: "heartbeat-result-1".to_string(),
            job_id: "heartbeat-job-1".to_string(),
            completed_at: "2026-05-06T00:03:00Z".to_string(),
            verdict: "pass".to_string(),
            summary: "Runtime model ready.".to_string(),
            next_safe_move: "Launch verification.".to_string(),
            evidence_refs: vec!["evidence:model".to_string()],
            artifact_refs: vec!["artifact:model".to_string()],
        },
    )
    .expect("runtime result");

    let (status, finding, note) = load_epiphany_role_result_from_runtime_spine_job(
        "heartbeat-job-1",
        Some(store.as_path()),
        ThreadEpiphanyRoleId::Modeling,
    );

    assert_eq!(status, ThreadEpiphanyRoleResultStatus::Completed);
    let finding = finding.expect("runtime finding");
    assert_eq!(finding.verdict.as_deref(), Some("pass"));
    assert_eq!(finding.summary.as_deref(), Some("Runtime model ready."));
    assert_eq!(finding.evidence_ids, vec!["evidence:model".to_string()]);
    assert!(note.contains("Modeling role specialist completed"));
}

#[test]
fn map_epiphany_jobs_blocks_binding_after_interrupt_clears_backend() {
    let state = codex_protocol::protocol::EpiphanyThreadState {
            job_bindings: vec![codex_protocol::protocol::EpiphanyJobBinding {
                id: "specialist-work".to_string(),
                kind: codex_protocol::protocol::EpiphanyJobKind::Specialist,
                scope: "role-scoped specialist work".to_string(),
                owner_role: "epiphany-harness".to_string(),
                authority_scope: Some("epiphany.specialist".to_string()),
                linked_subgoal_ids: vec!["phase-6".to_string()],
                linked_graph_node_ids: vec!["job-control".to_string()],
                blocking_reason: Some(
                    "No active runtime backend is currently bound; launch explicitly to resume specialist work."
                        .to_string(),
                ),
            }],
            ..Default::default()
        };

    let jobs = map_epiphany_jobs(Some(&state), None);
    let specialist = jobs
        .iter()
        .find(|job| job.id == "specialist-work")
        .expect("specialist slot should exist");

    assert_eq!(specialist.status, ThreadEpiphanyJobStatus::Blocked);
    assert_eq!(
        specialist.authority_scope.as_deref(),
        Some("epiphany.specialist")
    );
    assert_eq!(specialist.launcher_job_id, None);
    assert_eq!(specialist.backend_job_id, None);
    assert_eq!(specialist.linked_subgoal_ids, vec!["phase-6".to_string()]);
    assert_eq!(
        specialist.linked_graph_node_ids,
        vec!["job-control".to_string()]
    );
    assert!(
        specialist
            .blocking_reason
            .as_deref()
            .is_some_and(|reason| reason.contains("launch explicitly"))
    );
}

#[test]
fn epiphany_update_patch_changed_fields_reports_patch_surface() {
    let fields = epiphany_update_patch_changed_fields(&ThreadEpiphanyUpdatePatch {
        objective: Some("Keep the event readable".to_string()),
        graphs: Some(Default::default()),
        graph_frontier: Some(Default::default()),
        investigation_checkpoint: Some(Default::default()),
        job_bindings: Some(Vec::new()),
        observations: vec![Default::default()],
        evidence: vec![Default::default()],
        churn: Some(Default::default()),
        ..Default::default()
    });

    assert_eq!(
        fields,
        vec![
            ThreadEpiphanyStateUpdatedField::Objective,
            ThreadEpiphanyStateUpdatedField::Graphs,
            ThreadEpiphanyStateUpdatedField::GraphFrontier,
            ThreadEpiphanyStateUpdatedField::InvestigationCheckpoint,
            ThreadEpiphanyStateUpdatedField::JobBindings,
            ThreadEpiphanyStateUpdatedField::Observations,
            ThreadEpiphanyStateUpdatedField::Evidence,
            ThreadEpiphanyStateUpdatedField::Churn,
        ]
    );
}

#[test]
fn epiphany_promote_changed_fields_reports_appended_verifier_evidence() {
    let fields = epiphany_promote_changed_fields(&ThreadEpiphanyUpdatePatch {
        observations: vec![Default::default()],
        ..Default::default()
    });

    assert_eq!(
        fields,
        vec![
            ThreadEpiphanyStateUpdatedField::Observations,
            ThreadEpiphanyStateUpdatedField::Evidence,
        ]
    );
}

#[tokio::test]
async fn aborting_pending_request_clears_pending_state() -> Result<()> {
    let thread_id = ThreadId::from_string("bfd12a78-5900-467b-9bc5-d3d35df08191")?;
    let connection_id = ConnectionId(7);

    let (outgoing_tx, mut outgoing_rx) = tokio::sync::mpsc::channel(8);
    let outgoing = Arc::new(OutgoingMessageSender::new(outgoing_tx));
    let thread_outgoing =
        ThreadScopedOutgoingMessageSender::new(outgoing.clone(), vec![connection_id], thread_id);

    let (request_id, client_request_rx) = thread_outgoing
        .send_request(ServerRequestPayload::ToolRequestUserInput(
            ToolRequestUserInputParams {
                thread_id: thread_id.to_string(),
                turn_id: "turn-1".to_string(),
                item_id: "call-1".to_string(),
                questions: vec![],
            },
        ))
        .await;
    thread_outgoing.abort_pending_server_requests().await;

    let request_message = outgoing_rx.recv().await.expect("request should be sent");
    let OutgoingEnvelope::ToConnection {
        connection_id: request_connection_id,
        message:
            OutgoingMessage::Request(ServerRequest::ToolRequestUserInput {
                request_id: sent_request_id,
                ..
            }),
        ..
    } = request_message
    else {
        panic!("expected tool request to be sent to the subscribed connection");
    };
    assert_eq!(request_connection_id, connection_id);
    assert_eq!(sent_request_id, request_id);

    let response = client_request_rx
        .await
        .expect("callback should be resolved");
    let error = response.expect_err("request should be aborted during cleanup");
    assert_eq!(
        error.message,
        "client request resolved because the turn state was changed"
    );
    assert_eq!(error.data, Some(json!({ "reason": "turnTransition" })));
    assert!(
        outgoing
            .pending_requests_for_thread(thread_id)
            .await
            .is_empty()
    );
    assert!(outgoing_rx.try_recv().is_err());
    Ok(())
}

#[test]
fn summary_from_state_db_metadata_preserves_agent_nickname() -> Result<()> {
    let conversation_id = ThreadId::from_string("bfd12a78-5900-467b-9bc5-d3d35df08191")?;
    let source = serde_json::to_string(&SessionSource::SubAgent(SubAgentSource::ThreadSpawn {
        parent_thread_id: ThreadId::from_string("ad7f0408-99b8-4f6e-a46f-bd0eec433370")?,
        depth: 1,
        agent_path: None,
        agent_nickname: None,
        agent_role: None,
    }))?;

    let summary = summary_from_state_db_metadata(
        conversation_id,
        PathBuf::from("/tmp/rollout.jsonl"),
        Some("hi".to_string()),
        "2025-09-05T16:53:11Z".to_string(),
        "2025-09-05T16:53:12Z".to_string(),
        "test-provider".to_string(),
        PathBuf::from("/"),
        "0.0.0".to_string(),
        source,
        Some("atlas".to_string()),
        Some("explorer".to_string()),
        /*git_sha*/ None,
        /*git_branch*/ None,
        /*git_origin_url*/ None,
    );

    let fallback_cwd = AbsolutePathBuf::from_absolute_path("/")?;
    let thread = summary_to_thread(summary, &fallback_cwd);

    assert_eq!(thread.agent_nickname, Some("atlas".to_string()));
    assert_eq!(thread.agent_role, Some("explorer".to_string()));
    Ok(())
}

#[tokio::test]
async fn removing_thread_state_clears_listener_and_active_turn_history() -> Result<()> {
    let manager = ThreadStateManager::new();
    let thread_id = ThreadId::from_string("ad7f0408-99b8-4f6e-a46f-bd0eec433370")?;
    let connection = ConnectionId(1);
    let (cancel_tx, cancel_rx) = oneshot::channel();

    manager.connection_initialized(connection).await;
    manager
        .try_ensure_connection_subscribed(
            thread_id, connection, /*experimental_raw_events*/ false,
        )
        .await
        .expect("connection should be live");
    {
        let state = manager.thread_state(thread_id).await;
        let mut state = state.lock().await;
        state.cancel_tx = Some(cancel_tx);
        state.track_current_turn_event(&EventMsg::TurnStarted(
            codex_protocol::protocol::TurnStartedEvent {
                turn_id: "turn-1".to_string(),
                started_at: None,
                model_context_window: None,
                collaboration_mode_kind: Default::default(),
            },
        ));
    }

    manager.remove_thread_state(thread_id).await;
    assert_eq!(cancel_rx.await, Ok(()));

    let state = manager.thread_state(thread_id).await;
    let subscribed_connection_ids = manager.subscribed_connection_ids(thread_id).await;
    assert!(subscribed_connection_ids.is_empty());
    let state = state.lock().await;
    assert!(state.cancel_tx.is_none());
    assert!(state.active_turn_snapshot().is_none());
    Ok(())
}

#[tokio::test]
async fn removing_auto_attached_connection_preserves_listener_for_other_connections() -> Result<()>
{
    let manager = ThreadStateManager::new();
    let thread_id = ThreadId::from_string("ad7f0408-99b8-4f6e-a46f-bd0eec433370")?;
    let connection_a = ConnectionId(1);
    let connection_b = ConnectionId(2);
    let (cancel_tx, mut cancel_rx) = oneshot::channel();

    manager.connection_initialized(connection_a).await;
    manager.connection_initialized(connection_b).await;
    manager
        .try_ensure_connection_subscribed(
            thread_id,
            connection_a,
            /*experimental_raw_events*/ false,
        )
        .await
        .expect("connection_a should be live");
    manager
        .try_ensure_connection_subscribed(
            thread_id,
            connection_b,
            /*experimental_raw_events*/ false,
        )
        .await
        .expect("connection_b should be live");
    {
        let state = manager.thread_state(thread_id).await;
        state.lock().await.cancel_tx = Some(cancel_tx);
    }

    let threads_to_unload = manager.remove_connection(connection_a).await;
    assert_eq!(threads_to_unload, Vec::<ThreadId>::new());
    assert!(
        tokio::time::timeout(Duration::from_millis(20), &mut cancel_rx)
            .await
            .is_err()
    );

    assert_eq!(
        manager.subscribed_connection_ids(thread_id).await,
        vec![connection_b]
    );
    Ok(())
}

#[tokio::test]
async fn adding_connection_to_thread_updates_has_connections_watcher() -> Result<()> {
    let manager = ThreadStateManager::new();
    let thread_id = ThreadId::from_string("ad7f0408-99b8-4f6e-a46f-bd0eec433370")?;
    let connection_a = ConnectionId(1);
    let connection_b = ConnectionId(2);

    manager.connection_initialized(connection_a).await;
    manager.connection_initialized(connection_b).await;
    manager
        .try_ensure_connection_subscribed(
            thread_id,
            connection_a,
            /*experimental_raw_events*/ false,
        )
        .await
        .expect("connection_a should be live");
    let mut has_connections = manager
        .subscribe_to_has_connections(thread_id)
        .await
        .expect("thread should have a has-connections watcher");
    assert!(*has_connections.borrow());

    assert!(
        manager
            .unsubscribe_connection_from_thread(thread_id, connection_a)
            .await
    );
    tokio::time::timeout(Duration::from_secs(1), has_connections.changed())
        .await
        .expect("timed out waiting for no-subscriber update")
        .expect("has-connections watcher should remain open");
    assert!(!*has_connections.borrow());

    assert!(
        manager
            .try_add_connection_to_thread(thread_id, connection_b)
            .await
    );
    tokio::time::timeout(Duration::from_secs(1), has_connections.changed())
        .await
        .expect("timed out waiting for subscriber update")
        .expect("has-connections watcher should remain open");
    assert!(*has_connections.borrow());
    Ok(())
}

#[tokio::test]
async fn closed_connection_cannot_be_reintroduced_by_auto_subscribe() -> Result<()> {
    let manager = ThreadStateManager::new();
    let thread_id = ThreadId::from_string("ad7f0408-99b8-4f6e-a46f-bd0eec433370")?;
    let connection = ConnectionId(1);

    manager.connection_initialized(connection).await;
    let threads_to_unload = manager.remove_connection(connection).await;
    assert_eq!(threads_to_unload, Vec::<ThreadId>::new());

    assert!(
        manager
            .try_ensure_connection_subscribed(
                thread_id, connection, /*experimental_raw_events*/ false
            )
            .await
            .is_none()
    );
    assert!(!manager.has_subscribers(thread_id).await);
    Ok(())
}
