use super::*;

impl CodexMessageProcessor {
    pub(crate) async fn instruction_sources_from_config(config: &Config) -> Vec<AbsolutePathBuf> {
        codex_core::AgentsMdManager::new(config)
            .instruction_sources(LOCAL_FS.as_ref())
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_thread_config_overrides(
        &self,
        model: Option<String>,
        model_provider: Option<String>,
        service_tier: Option<Option<codex_protocol::config_types::ServiceTier>>,
        cwd: Option<String>,
        approval_policy: Option<codex_app_server_protocol::AskForApproval>,
        approvals_reviewer: Option<codex_app_server_protocol::ApprovalsReviewer>,
        sandbox: Option<SandboxMode>,
        permission_profile: Option<ApiPermissionProfile>,
        base_instructions: Option<String>,
        developer_instructions: Option<String>,
        personality: Option<Personality>,
    ) -> ConfigOverrides {
        ConfigOverrides {
            model,
            model_provider,
            service_tier,
            cwd: cwd.map(PathBuf::from),
            approval_policy: approval_policy
                .map(codex_app_server_protocol::AskForApproval::to_core),
            approvals_reviewer: approvals_reviewer
                .map(codex_app_server_protocol::ApprovalsReviewer::to_core),
            sandbox_mode: sandbox.map(SandboxMode::to_core),
            permission_profile: permission_profile.map(Into::into),
            codex_linux_sandbox_exe: self.arg0_paths.codex_linux_sandbox_exe.clone(),
            main_execve_wrapper_exe: self.arg0_paths.main_execve_wrapper_exe.clone(),
            base_instructions,
            developer_instructions,
            personality,
            ..Default::default()
        }
    }
}

fn cloud_requirements_load_error(err: &std::io::Error) -> Option<&CloudRequirementsLoadError> {
    let mut current: Option<&(dyn std::error::Error + 'static)> = err
        .get_ref()
        .map(|source| source as &(dyn std::error::Error + 'static));
    while let Some(source) = current {
        if let Some(cloud_error) = source.downcast_ref::<CloudRequirementsLoadError>() {
            return Some(cloud_error);
        }
        current = source.source();
    }
    None
}

pub(crate) fn config_load_error(err: &std::io::Error) -> JSONRPCErrorError {
    let data = cloud_requirements_load_error(err).map(|cloud_error| {
        let mut data = serde_json::json!({
            "reason": "cloudRequirements",
            "errorCode": format!("{:?}", cloud_error.code()),
            "detail": cloud_error.to_string(),
        });
        if let Some(status_code) = cloud_error.status_code() {
            data["statusCode"] = serde_json::json!(status_code);
        }
        if cloud_error.code() == CloudRequirementsLoadErrorCode::Auth {
            data["action"] = serde_json::json!("relogin");
        }
        data
    });

    JSONRPCErrorError {
        code: INVALID_REQUEST_ERROR_CODE,
        message: format!("failed to load configuration: {err}"),
        data,
    }
}

pub(crate) fn validate_dynamic_tools(tools: &[ApiDynamicToolSpec]) -> Result<(), String> {
    let mut seen = HashSet::new();
    for tool in tools {
        let name = tool.name.trim();
        if name.is_empty() {
            return Err("dynamic tool name must not be empty".to_string());
        }
        if name != tool.name {
            return Err(format!(
                "dynamic tool name has leading/trailing whitespace: {}",
                tool.name
            ));
        }
        if name == "mcp" || name.starts_with("mcp__") {
            return Err(format!("dynamic tool name is reserved: {name}"));
        }
        let namespace = tool.namespace.as_deref().map(str::trim);
        if let Some(namespace) = namespace {
            if namespace.is_empty() {
                return Err(format!(
                    "dynamic tool namespace must not be empty for {name}"
                ));
            }
            if Some(namespace) != tool.namespace.as_deref() {
                return Err(format!(
                    "dynamic tool namespace has leading/trailing whitespace for {name}: {namespace}",
                ));
            }
            if namespace == "mcp" || namespace.starts_with("mcp__") {
                return Err(format!(
                    "dynamic tool namespace is reserved for {name}: {namespace}"
                ));
            }
        }
        if !seen.insert((namespace, name)) {
            if let Some(namespace) = namespace {
                return Err(format!(
                    "duplicate dynamic tool name in namespace {namespace}: {name}"
                ));
            }
            return Err(format!("duplicate dynamic tool name: {name}"));
        }
        if tool.defer_loading && namespace.is_none() {
            return Err(format!(
                "deferred dynamic tool must include a namespace: {name}"
            ));
        }

        if let Err(err) = codex_tools::parse_tool_input_schema(&tool.input_schema) {
            return Err(format!(
                "dynamic tool input schema is not supported for {name}: {err}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tool(namespace: Option<&str>, name: &str, defer_loading: bool) -> ApiDynamicToolSpec {
        ApiDynamicToolSpec {
            namespace: namespace.map(str::to_string),
            name: name.to_string(),
            description: "test tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }),
            defer_loading,
        }
    }

    #[test]
    fn dynamic_tool_validation_accepts_namespaced_deferred_tool() {
        let tools = vec![tool(Some("analysis"), "inspect_state", true)];

        assert_eq!(validate_dynamic_tools(&tools), Ok(()));
    }

    #[test]
    fn dynamic_tool_validation_rejects_duplicate_authority_in_same_namespace() {
        let tools = vec![
            tool(Some("analysis"), "inspect_state", false),
            tool(Some("analysis"), "inspect_state", false),
        ];

        let error = validate_dynamic_tools(&tools).expect_err("duplicate should fail");
        assert!(error.contains("duplicate dynamic tool name in namespace analysis"));
    }

    #[test]
    fn dynamic_tool_validation_rejects_mcp_namespace_impersonation() {
        let tools = vec![tool(Some("mcp__demo"), "inspect_state", false)];

        let error = validate_dynamic_tools(&tools).expect_err("reserved namespace should fail");
        assert!(error.contains("reserved"));
    }

    #[test]
    fn dynamic_tool_validation_rejects_hidden_tool_without_namespace() {
        let tools = vec![tool(None, "inspect_state", true)];

        let error = validate_dynamic_tools(&tools).expect_err("deferred tool should fail");
        assert!(error.contains("namespace"));
    }
}
