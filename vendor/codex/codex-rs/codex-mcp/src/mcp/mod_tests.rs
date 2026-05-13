use super::*;
use codex_config::Constrained;
use codex_protocol::protocol::AskForApproval;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::path::PathBuf;

fn test_mcp_config(codex_home: PathBuf) -> McpConfig {
    McpConfig {
        chatgpt_base_url: "https://chatgpt.com".to_string(),
        codex_home,
        mcp_oauth_credentials_store_mode: OAuthCredentialsStoreMode::default(),
        mcp_oauth_callback_port: None,
        mcp_oauth_callback_url: None,
        skill_mcp_dependency_install_enabled: true,
        approval_policy: Constrained::allow_any(AskForApproval::OnFailure),
        codex_linux_sandbox_exe: None,
        use_legacy_landlock: false,
        configured_mcp_servers: HashMap::new(),
    }
}

fn make_tool(name: &str) -> Tool {
    Tool {
        name: name.to_string(),
        title: None,
        description: None,
        input_schema: serde_json::json!({"type": "object", "properties": {}}),
        output_schema: None,
        annotations: None,
        icons: None,
        meta: None,
    }
}

#[test]
fn split_qualified_tool_name_returns_server_and_tool() {
    assert_eq!(
        split_qualified_tool_name("mcp__alpha__do_thing"),
        Some(("alpha".to_string(), "do_thing".to_string()))
    );
}

#[test]
fn qualified_mcp_tool_name_prefix_sanitizes_server_names_without_lowercasing() {
    assert_eq!(
        qualified_mcp_tool_name_prefix("Some-Server"),
        "mcp__Some_Server__".to_string()
    );
}

#[test]
fn split_qualified_tool_name_rejects_invalid_names() {
    assert_eq!(split_qualified_tool_name("other__alpha__do_thing"), None);
    assert_eq!(split_qualified_tool_name("mcp__alpha__"), None);
}

#[test]
fn group_tools_by_server_strips_prefix_and_groups() {
    let mut tools = HashMap::new();
    tools.insert("mcp__alpha__do_thing".to_string(), make_tool("do_thing"));
    tools.insert(
        "mcp__alpha__nested__op".to_string(),
        make_tool("nested__op"),
    );
    tools.insert("mcp__beta__do_other".to_string(), make_tool("do_other"));

    let mut expected_alpha = HashMap::new();
    expected_alpha.insert("do_thing".to_string(), make_tool("do_thing"));
    expected_alpha.insert("nested__op".to_string(), make_tool("nested__op"));

    let mut expected_beta = HashMap::new();
    expected_beta.insert("do_other".to_string(), make_tool("do_other"));

    let mut expected = HashMap::new();
    expected.insert("alpha".to_string(), expected_alpha);
    expected.insert("beta".to_string(), expected_beta);

    assert_eq!(group_tools_by_server(&tools), expected);
}

#[tokio::test]
async fn effective_mcp_servers_preserve_user_servers() {
    let codex_home = tempfile::tempdir().expect("tempdir");
    let mut config = test_mcp_config(codex_home.path().to_path_buf());

    config.configured_mcp_servers.insert(
        "sample".to_string(),
        McpServerConfig {
            transport: McpServerTransportConfig::StreamableHttp {
                url: "https://user.example/mcp".to_string(),
                bearer_token_env_var: None,
                http_headers: None,
                env_http_headers: None,
            },
            experimental_environment: None,
            enabled: true,
            required: false,
            supports_parallel_tool_calls: false,
            disabled_reason: None,
            startup_timeout_sec: None,
            tool_timeout_sec: None,
            default_tools_approval_mode: None,
            enabled_tools: None,
            disabled_tools: None,
            scopes: None,
            oauth_resource: None,
            tools: HashMap::new(),
        },
    );
    config.configured_mcp_servers.insert(
        "docs".to_string(),
        McpServerConfig {
            transport: McpServerTransportConfig::StreamableHttp {
                url: "https://docs.example/mcp".to_string(),
                bearer_token_env_var: None,
                http_headers: None,
                env_http_headers: None,
            },
            experimental_environment: None,
            enabled: true,
            required: false,
            supports_parallel_tool_calls: false,
            disabled_reason: None,
            startup_timeout_sec: None,
            tool_timeout_sec: None,
            default_tools_approval_mode: None,
            enabled_tools: None,
            disabled_tools: None,
            scopes: None,
            oauth_resource: None,
            tools: HashMap::new(),
        },
    );

    let effective = effective_mcp_servers(&config, None);

    let sample = effective.get("sample").expect("user server should exist");
    let docs = effective
        .get("docs")
        .expect("configured server should exist");
    assert_eq!(effective.len(), 2);

    match &sample.transport {
        McpServerTransportConfig::StreamableHttp { url, .. } => {
            assert_eq!(url, "https://user.example/mcp");
        }
        other => panic!("expected streamable http transport, got {other:?}"),
    }
    match &docs.transport {
        McpServerTransportConfig::StreamableHttp { url, .. } => {
            assert_eq!(url, "https://docs.example/mcp");
        }
        other => panic!("expected streamable http transport, got {other:?}"),
    }
}
