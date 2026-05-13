use codex_app_server_protocol::AuthMode;
use codex_login::AuthManager;
use codex_login::CodexAuth;
use epiphany_openai_adapter::EpiphanyOpenAiAdapterStatus;
use epiphany_openai_adapter::EpiphanyOpenAiAuthMode;

pub const CODEX_SPINE_ADAPTER_ID: &str = "codex-openai-subscription-spine";

pub async fn status_from_auth_manager(
    auth_manager: &AuthManager,
    default_model: Option<String>,
    supports_websockets: bool,
) -> EpiphanyOpenAiAdapterStatus {
    let auth = auth_manager.auth().await;
    let auth_mode = auth_mode_from_manager(auth_manager, auth.as_ref());
    let account_id = auth.as_ref().and_then(CodexAuth::get_account_id);
    let plan_type = auth
        .as_ref()
        .and_then(CodexAuth::account_plan_type)
        .map(|plan| format!("{plan:?}"));

    EpiphanyOpenAiAdapterStatus {
        adapter_id: CODEX_SPINE_ADAPTER_ID.to_string(),
        auth_mode,
        account_id,
        plan_type,
        default_model,
        supports_websockets,
        codex_transport_attached: true,
    }
}

pub fn status_from_codex_auth(
    auth: Option<&CodexAuth>,
    default_model: Option<String>,
    supports_websockets: bool,
) -> EpiphanyOpenAiAdapterStatus {
    EpiphanyOpenAiAdapterStatus {
        adapter_id: CODEX_SPINE_ADAPTER_ID.to_string(),
        auth_mode: auth_mode_from_codex_auth(auth),
        account_id: auth.and_then(CodexAuth::get_account_id),
        plan_type: auth
            .and_then(CodexAuth::account_plan_type)
            .map(|plan| format!("{plan:?}")),
        default_model,
        supports_websockets,
        codex_transport_attached: true,
    }
}

fn auth_mode_from_manager(
    auth_manager: &AuthManager,
    auth: Option<&CodexAuth>,
) -> EpiphanyOpenAiAuthMode {
    match auth_manager.auth_mode() {
        Some(AuthMode::ApiKey) => EpiphanyOpenAiAuthMode::ApiKey,
        Some(AuthMode::Chatgpt) | Some(AuthMode::ChatgptAuthTokens) => {
            EpiphanyOpenAiAuthMode::ChatGptSubscription
        }
        Some(AuthMode::AgentIdentity) => EpiphanyOpenAiAuthMode::ExternalBearer,
        None => auth_mode_from_codex_auth(auth),
    }
}

fn auth_mode_from_codex_auth(auth: Option<&CodexAuth>) -> EpiphanyOpenAiAuthMode {
    let Some(auth) = auth else {
        return EpiphanyOpenAiAuthMode::Unknown;
    };
    if auth.is_api_key_auth() {
        EpiphanyOpenAiAuthMode::ApiKey
    } else if auth.is_chatgpt_auth() {
        EpiphanyOpenAiAuthMode::ChatGptSubscription
    } else {
        EpiphanyOpenAiAuthMode::ExternalBearer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_auth_maps_to_typed_adapter_status() {
        let auth = CodexAuth::from_api_key("test-key");
        let status = status_from_codex_auth(Some(&auth), Some("gpt-5.4".to_string()), true);

        assert_eq!(status.adapter_id, CODEX_SPINE_ADAPTER_ID);
        assert_eq!(status.auth_mode, EpiphanyOpenAiAuthMode::ApiKey);
        assert_eq!(status.default_model.as_deref(), Some("gpt-5.4"));
        assert!(status.supports_websockets);
        assert!(status.codex_transport_attached);
    }

    #[test]
    fn missing_auth_maps_to_unknown_status() {
        let status = status_from_codex_auth(None, None, false);

        assert_eq!(status.auth_mode, EpiphanyOpenAiAuthMode::Unknown);
        assert_eq!(status.account_id, None);
        assert!(!status.supports_websockets);
    }
}
