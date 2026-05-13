pub use codex_app_server_protocol::AppBranding;
pub use codex_app_server_protocol::AppInfo;
pub use codex_app_server_protocol::AppMetadata;
use codex_mcp::ToolInfo;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct AccessibleConnectorsStatus {
    pub connectors: Vec<AppInfo>,
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

pub async fn list_all_connectors_with_options(
    _config: &Config,
    _force_refetch: bool,
) -> anyhow::Result<Vec<AppInfo>> {
    Ok(Vec::new())
}

pub fn merge_connectors_with_accessible(
    _all_connectors: Vec<AppInfo>,
    accessible_connectors: Vec<AppInfo>,
    _all_connectors_loaded: bool,
) -> Vec<AppInfo> {
    accessible_connectors
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
    })
}

pub async fn list_accessible_connectors_from_mcp_tools_with_environment_manager(
    _config: &Config,
    _force_refetch: bool,
    _environment_manager: &codex_exec_server::EnvironmentManager,
) -> anyhow::Result<AccessibleConnectorsStatus> {
    Ok(AccessibleConnectorsStatus {
        connectors: Vec::new(),
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
