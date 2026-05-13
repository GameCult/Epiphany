use std::collections::HashMap;

use crate::config::Config;
use codex_config::McpServerConfig;
use codex_login::CodexAuth;
use codex_mcp::configured_mcp_servers;
use codex_mcp::effective_mcp_servers;

#[derive(Clone)]
pub struct McpManager;

impl McpManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn configured_servers(&self, config: &Config) -> HashMap<String, McpServerConfig> {
        let mcp_config = config.to_mcp_config();
        configured_mcp_servers(&mcp_config)
    }

    pub async fn effective_servers(
        &self,
        config: &Config,
        auth: Option<&CodexAuth>,
    ) -> HashMap<String, McpServerConfig> {
        let mcp_config = config.to_mcp_config();
        effective_mcp_servers(&mcp_config, auth)
    }
}
