use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Deserializer};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::time::Duration;

fn default_enabled() -> bool {
    true
}

fn default_startup_timeout() -> u64 {
    10
}

fn default_tool_timeout() -> u64 {
    60
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct McpRuntimeConfig {
    #[serde(default)]
    pub mcp_servers: BTreeMap<String, McpServerConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpServerConfig {
    pub transport: McpTransportConfig,
    pub enabled: bool,
    pub startup_timeout_sec: u64,
    pub tool_timeout_sec: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpTransportConfig {
    Stdio {
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        cwd: Option<PathBuf>,
    },
    Http {
        url: String,
        bearer_token_env_var: Option<String>,
        http_headers: HashMap<String, String>,
        env_http_headers: HashMap<String, String>,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMcpServerConfig {
    command: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: HashMap<String, String>,
    cwd: Option<PathBuf>,
    url: Option<String>,
    bearer_token_env_var: Option<String>,
    #[serde(default)]
    http_headers: HashMap<String, String>,
    #[serde(default)]
    env_http_headers: HashMap<String, String>,
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(default = "default_startup_timeout")]
    startup_timeout_sec: u64,
    #[serde(default = "default_tool_timeout")]
    tool_timeout_sec: u64,
}

impl<'de> Deserialize<'de> for McpServerConfig {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let raw = RawMcpServerConfig::deserialize(deserializer)?;
        let transport = match (raw.command, raw.url) {
            (Some(command), None)
                if raw.bearer_token_env_var.is_none()
                    && raw.http_headers.is_empty()
                    && raw.env_http_headers.is_empty() =>
            {
                McpTransportConfig::Stdio {
                    command,
                    args: raw.args,
                    env: raw.env,
                    cwd: raw.cwd,
                }
            }
            (None, Some(url)) if raw.args.is_empty() && raw.env.is_empty() && raw.cwd.is_none() => {
                McpTransportConfig::Http {
                    url,
                    bearer_token_env_var: raw.bearer_token_env_var,
                    http_headers: raw.http_headers,
                    env_http_headers: raw.env_http_headers,
                }
            }
            _ => {
                return Err(serde::de::Error::custom(
                    "MCP server must configure exactly one coherent stdio or HTTP transport",
                ));
            }
        };
        Ok(Self {
            transport,
            enabled: raw.enabled,
            startup_timeout_sec: raw.startup_timeout_sec,
            tool_timeout_sec: raw.tool_timeout_sec,
        })
    }
}

impl McpRuntimeConfig {
    pub fn from_path(path: &Path) -> Result<Self> {
        let raw = std::fs::read_to_string(path)
            .with_context(|| format!("reading MCP config {}", path.display()))?;
        Self::from_toml(&raw)
    }

    pub fn from_toml(raw: &str) -> Result<Self> {
        let config: Self = toml::from_str(raw).context("decoding MCP config")?;
        config.validate()?;
        Ok(config)
    }

    pub fn server(&self, name: &str) -> Result<&McpServerConfig> {
        let server = self
            .mcp_servers
            .get(name)
            .ok_or_else(|| anyhow!("MCP server {name:?} is not configured"))?;
        if !server.enabled {
            return Err(anyhow!("MCP server {name:?} is disabled"));
        }
        Ok(server)
    }

    fn validate(&self) -> Result<()> {
        for (name, server) in &self.mcp_servers {
            if name.trim().is_empty()
                || server.startup_timeout_sec == 0
                || server.tool_timeout_sec == 0
            {
                return Err(anyhow!("MCP server name and timeouts must be nonzero"));
            }
            match &server.transport {
                McpTransportConfig::Stdio { command, cwd, .. } => {
                    if !Path::new(command).is_absolute() {
                        return Err(anyhow!(
                            "MCP stdio server {name:?} command must be absolute"
                        ));
                    }
                    if cwd.as_ref().is_some_and(|path| !path.is_absolute()) {
                        return Err(anyhow!("MCP stdio server {name:?} cwd must be absolute"));
                    }
                }
                McpTransportConfig::Http { url, .. } => validate_http_url(name, url)?,
            }
        }
        Ok(())
    }
}

fn validate_http_url(name: &str, raw: &str) -> Result<()> {
    let url = reqwest::Url::parse(raw)
        .with_context(|| format!("MCP HTTP server {name:?} URL is invalid"))?;
    if !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
        return Err(anyhow!(
            "MCP HTTP server {name:?} URL may not contain credentials or a fragment"
        ));
    }
    let loopback_http =
        url.scheme() == "http" && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
    if url.scheme() != "https" && !loopback_http {
        return Err(anyhow!(
            "MCP HTTP server {name:?} must use HTTPS or loopback HTTP"
        ));
    }
    Ok(())
}

impl McpServerConfig {
    pub fn startup_timeout(&self) -> Duration {
        Duration::from_secs(self.startup_timeout_sec)
    }

    pub fn tool_timeout(&self) -> Duration {
        Duration::from_secs(self.tool_timeout_sec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_only_the_owned_stdio_and_http_subset() -> Result<()> {
        let config = McpRuntimeConfig::from_toml(
            r#"
[mcp_servers.local]
command = "C:/mcp/server.exe"
args = ["--stdio"]
startup_timeout_sec = 2

[mcp_servers.remote]
url = "https://example.test/mcp"
bearer_token_env_var = "MCP_TOKEN"
tool_timeout_sec = 4
"#,
        )?;
        assert!(matches!(
            config.server("local")?.transport,
            McpTransportConfig::Stdio { .. }
        ));
        assert!(matches!(
            config.server("remote")?.transport,
            McpTransportConfig::Http { .. }
        ));
        assert!(
            McpRuntimeConfig::from_toml("[mcp_servers.bad]\nurl='http://example.test/mcp'")
                .is_err()
        );
        assert!(
            McpRuntimeConfig::from_toml(
                "[mcp_servers.bad]\nurl='http://localhost.evil.test:8080/mcp'"
            )
            .is_err()
        );
        assert!(
            McpRuntimeConfig::from_toml("[mcp_servers.bad]\nurl='https://secret@example.test/mcp'")
                .is_err()
        );
        assert!(
            McpRuntimeConfig::from_toml("[mcp_servers.bad]\ncommand='x'\nurl='https://x.test'")
                .is_err()
        );
        assert!(
            McpRuntimeConfig::from_toml("[mcp_servers.bad]\ncommand='relative-server'").is_err()
        );
        assert!(
            McpRuntimeConfig::from_toml(
                "[mcp_servers.bad]\ncommand='C:/server.exe'\ncwd='relative'"
            )
            .is_err()
        );
        assert!(
            McpRuntimeConfig::from_toml(
                "[mcp_servers.bad]\ncommand='C:/server.exe'\ninherit_env=true"
            )
            .is_err()
        );
        Ok(())
    }
}
