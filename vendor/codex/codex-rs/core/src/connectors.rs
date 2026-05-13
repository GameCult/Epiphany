pub use codex_app_server_protocol::AppBranding;
pub use codex_app_server_protocol::AppInfo;
pub use codex_app_server_protocol::AppMetadata;
use codex_config::types::AppToolApproval;
use codex_mcp::ToolInfo;
use rmcp::model::ToolAnnotations;

use crate::config::Config;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AppToolPolicy {
    pub enabled: bool,
    pub approval: AppToolApproval,
}

impl Default for AppToolPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            approval: AppToolApproval::Auto,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AccessibleConnectorsStatus {
    pub connectors: Vec<AppInfo>,
    pub codex_apps_ready: bool,
}

pub async fn list_accessible_connectors_from_mcp_tools(
    _config: &Config,
) -> anyhow::Result<Vec<AppInfo>> {
    Ok(Vec::new())
}

pub async fn list_cached_accessible_connectors_from_mcp_tools(
    _config: &Config,
) -> Option<Vec<AppInfo>> {
    Some(Vec::new())
}

pub async fn list_accessible_connectors_from_mcp_tools_with_options(
    _config: &Config,
    _force_refetch: bool,
) -> anyhow::Result<Vec<AppInfo>> {
    Ok(Vec::new())
}

pub async fn list_accessible_connectors_from_mcp_tools_with_options_and_status(
    _config: &Config,
    _force_refetch: bool,
) -> anyhow::Result<AccessibleConnectorsStatus> {
    Ok(AccessibleConnectorsStatus {
        connectors: Vec::new(),
        codex_apps_ready: true,
    })
}

pub async fn list_accessible_connectors_from_mcp_tools_with_environment_manager(
    _config: &Config,
    _force_refetch: bool,
    _environment_manager: &codex_exec_server::EnvironmentManager,
) -> anyhow::Result<AccessibleConnectorsStatus> {
    Ok(AccessibleConnectorsStatus {
        connectors: Vec::new(),
        codex_apps_ready: true,
    })
}

pub fn accessible_connectors_from_mcp_tools(
    _mcp_tools: &std::collections::HashMap<String, ToolInfo>,
) -> Vec<AppInfo> {
    Vec::new()
}

pub fn with_app_enabled_state(connectors: Vec<AppInfo>, _config: &Config) -> Vec<AppInfo> {
    connectors
}

pub(crate) fn app_tool_policy(
    _config: &Config,
    _connector_id: Option<&str>,
    _tool_name: &str,
    _tool_title: Option<&str>,
    _annotations: Option<&ToolAnnotations>,
) -> AppToolPolicy {
    AppToolPolicy::default()
}

pub(crate) fn codex_app_tool_is_enabled(_config: &Config, _tool_info: &ToolInfo) -> bool {
    true
}
