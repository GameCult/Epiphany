use std::collections::HashMap;
use std::sync::Arc;

use codex_features::Feature;
use codex_features::Features;
use codex_mcp::ToolInfo;
use codex_models_manager::manager::ModelsManager;
use codex_protocol::config_types::WebSearchMode;
use codex_protocol::config_types::WindowsSandboxLevel;
use codex_protocol::protocol::SandboxPolicy;
use codex_protocol::protocol::SessionSource;
use codex_tools::ToolsConfig;
use codex_tools::ToolsConfigParams;
use pretty_assertions::assert_eq;
use rmcp::model::JsonObject;
use rmcp::model::Tool;

use super::*;
use crate::config::test_config;

fn make_mcp_tool(server_name: &str, tool_name: &str) -> ToolInfo {
    ToolInfo {
        server_name: server_name.to_string(),
        callable_name: tool_name.to_string(),
        callable_namespace: format!("mcp__{server_name}__"),
        server_instructions: None,
        tool: Tool {
            name: tool_name.to_string().into(),
            title: None,
            description: Some(format!("Test tool: {tool_name}").into()),
            input_schema: Arc::new(JsonObject::default()),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        },
        connector_id: None,
        connector_name: None,
        plugin_display_names: Vec::new(),
        connector_description: None,
    }
}

fn numbered_mcp_tools(count: usize) -> HashMap<String, ToolInfo> {
    (0..count)
        .map(|index| {
            let tool_name = format!("tool_{index}");
            (
                format!("mcp__rmcp__{tool_name}"),
                make_mcp_tool("rmcp", &tool_name),
            )
        })
        .collect()
}

async fn tools_config_for_mcp_tool_exposure(search_tool: bool) -> ToolsConfig {
    let config = test_config().await;
    let model_info = ModelsManager::construct_model_info_offline_for_tests(
        "gpt-5.4",
        &config.to_models_manager_config(),
    );
    let features = Features::with_defaults();
    let available_models = Vec::new();
    let mut tools_config = ToolsConfig::new(&ToolsConfigParams {
        model_info: &model_info,
        available_models: &available_models,
        features: &features,
        image_generation_tool_auth_allowed: true,
        web_search_mode: Some(WebSearchMode::Cached),
        session_source: SessionSource::Cli,
        sandbox_policy: &SandboxPolicy::DangerFullAccess,
        windows_sandbox_level: WindowsSandboxLevel::Disabled,
    });
    tools_config.search_tool = search_tool;
    tools_config
}

#[tokio::test]
async fn directly_exposes_small_mcp_tool_sets() {
    let config = test_config().await;
    let tools_config = tools_config_for_mcp_tool_exposure(/*search_tool*/ true).await;
    let mcp_tools = numbered_mcp_tools(DIRECT_MCP_TOOL_EXPOSURE_THRESHOLD - 1);

    let exposure = build_mcp_tool_exposure(&mcp_tools, &config, &tools_config);

    let mut direct_tool_names: Vec<_> = exposure.direct_tools.keys().cloned().collect();
    direct_tool_names.sort();
    let mut expected_tool_names: Vec<_> = mcp_tools.keys().cloned().collect();
    expected_tool_names.sort();
    assert_eq!(direct_tool_names, expected_tool_names);
    assert!(exposure.deferred_tools.is_none());
}

#[tokio::test]
async fn searches_large_mcp_tool_sets() {
    let config = test_config().await;
    let tools_config = tools_config_for_mcp_tool_exposure(/*search_tool*/ true).await;
    let mcp_tools = numbered_mcp_tools(DIRECT_MCP_TOOL_EXPOSURE_THRESHOLD);

    let exposure = build_mcp_tool_exposure(&mcp_tools, &config, &tools_config);

    assert!(exposure.direct_tools.is_empty());
    let deferred_tools = exposure
        .deferred_tools
        .as_ref()
        .expect("large tool sets should be discoverable through tool_search");
    let mut deferred_tool_names: Vec<_> = deferred_tools.keys().cloned().collect();
    deferred_tool_names.sort();
    let mut expected_tool_names: Vec<_> = mcp_tools.keys().cloned().collect();
    expected_tool_names.sort();
    assert_eq!(deferred_tool_names, expected_tool_names);
}

#[tokio::test]
async fn always_defer_feature_defers_all_mcp_tools_uniformly() {
    let mut config = test_config().await;
    config
        .features
        .enable(Feature::ToolSearchAlwaysDeferMcpTools)
        .expect("test config should allow feature update");
    let tools_config = tools_config_for_mcp_tool_exposure(/*search_tool*/ true).await;
    let mcp_tools = HashMap::from([
        ("mcp__rmcp__tool".to_string(), make_mcp_tool("rmcp", "tool")),
        (
            "mcp__custom__calendar_create_event".to_string(),
            make_mcp_tool("custom", "calendar_create_event"),
        ),
    ]);

    let exposure = build_mcp_tool_exposure(&mcp_tools, &config, &tools_config);

    assert!(exposure.direct_tools.is_empty());
    let deferred_tools = exposure
        .deferred_tools
        .as_ref()
        .expect("MCP tools should be discoverable through tool_search");
    assert_eq!(deferred_tools.len(), 2);
    assert!(deferred_tools.contains_key("mcp__rmcp__tool"));
    assert!(deferred_tools.contains_key("mcp__custom__calendar_create_event"));
}
