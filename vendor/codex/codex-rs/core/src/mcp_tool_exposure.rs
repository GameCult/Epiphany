use std::collections::HashMap;

use codex_features::Feature;
use codex_mcp::ToolInfo as McpToolInfo;
use codex_tools::ToolsConfig;

use crate::config::Config;

pub(crate) const DIRECT_MCP_TOOL_EXPOSURE_THRESHOLD: usize = 100;

pub(crate) struct McpToolExposure {
    pub(crate) direct_tools: HashMap<String, McpToolInfo>,
    pub(crate) deferred_tools: Option<HashMap<String, McpToolInfo>>,
}

pub(crate) fn build_mcp_tool_exposure(
    all_mcp_tools: &HashMap<String, McpToolInfo>,
    config: &Config,
    tools_config: &ToolsConfig,
) -> McpToolExposure {
    let deferred_tools = all_mcp_tools.clone();
    let should_defer = tools_config.search_tool
        && (config
            .features
            .enabled(Feature::ToolSearchAlwaysDeferMcpTools)
            || deferred_tools.len() >= DIRECT_MCP_TOOL_EXPOSURE_THRESHOLD);

    if !should_defer {
        return McpToolExposure {
            direct_tools: deferred_tools,
            deferred_tools: None,
        };
    }

    McpToolExposure {
        direct_tools: HashMap::new(),
        deferred_tools: (!deferred_tools.is_empty()).then_some(deferred_tools),
    }
}

#[cfg(test)]
#[path = "mcp_tool_exposure_test.rs"]
mod tests;
