//! Epiphany's named boundary to the retained Codex-compatible auth organ.
//!
//! This crate intentionally does not clone Codex login behavior. Subscription auth,
//! token refresh, keyring/file storage, originator headers, and model-provider auth
//! identity remain owned by vendored `codex-login` so Epiphany stays a modified
//! Codex-derived backend instead of a clean-room impersonator.

pub use codex_login::AuthConfig;
pub use codex_login::AuthCredentialsStoreMode;
pub use codex_login::AuthDotJson;
pub use codex_login::AuthEnvTelemetry;
pub use codex_login::AuthManager;
pub use codex_login::AuthManagerConfig;
pub use codex_login::AuthMode;
pub use codex_login::CLIENT_ID;
pub use codex_login::CODEX_API_KEY_ENV_VAR;
pub use codex_login::CodexAuth;
pub use codex_login::DeviceCode;
pub use codex_login::ExternalAuth;
pub use codex_login::ExternalAuthChatgptMetadata;
pub use codex_login::ExternalAuthRefreshContext;
pub use codex_login::ExternalAuthRefreshReason;
pub use codex_login::ExternalAuthTokens;
pub use codex_login::LoginServer;
pub use codex_login::OPENAI_API_KEY_ENV_VAR;
pub use codex_login::REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR;
pub use codex_login::REVOKE_TOKEN_URL_OVERRIDE_ENV_VAR;
pub use codex_login::RefreshTokenError;
pub use codex_login::ServerOptions;
pub use codex_login::ShutdownHandle;
pub use codex_login::TokenData;
pub use codex_login::UnauthorizedRecovery;
pub use codex_login::collect_auth_env_telemetry;
pub use codex_login::complete_device_code_login;
pub use codex_login::default_client;
pub use codex_login::enforce_login_restrictions;
pub use codex_login::load_auth_dot_json;
pub use codex_login::login_with_api_key;
pub use codex_login::logout;
pub use codex_login::logout_with_revoke;
pub use codex_login::read_openai_api_key_from_env;
pub use codex_login::request_device_code;
pub use codex_login::run_device_code_login;
pub use codex_login::run_login_server;
pub use codex_login::save_auth;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_auth_exposes_openai_token_through_codex_login() {
        let auth = CodexAuth::from_api_key("sk-test");
        assert!(auth.is_api_key_auth());
        assert_eq!(auth.get_token().unwrap(), "sk-test");
        assert!(!auth.uses_codex_backend());
    }

    #[test]
    fn parses_minimal_codex_auth_json_with_vendored_shape() {
        let auth: AuthDotJson =
            serde_json::from_str(r#"{"auth_mode":"apikey","OPENAI_API_KEY":"sk-test"}"#).unwrap();
        assert_eq!(auth.auth_mode, Some(AuthMode::ApiKey));
        assert_eq!(auth.openai_api_key.as_deref(), Some("sk-test"));
    }

    #[test]
    fn parses_codex_auth_store_modes() {
        assert_eq!(
            serde_json::from_str::<AuthCredentialsStoreMode>(r#""auto""#).unwrap(),
            AuthCredentialsStoreMode::Auto
        );
        assert_eq!(
            serde_json::from_str::<AuthCredentialsStoreMode>(r#""keyring""#).unwrap(),
            AuthCredentialsStoreMode::Keyring
        );
    }
}
