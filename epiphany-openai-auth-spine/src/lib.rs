//! Epiphany's named boundary to the retained Codex-compatible auth organ.
//!
//! This crate intentionally does not clone Codex login behavior. Subscription auth,
//! token refresh, keyring/file storage, originator headers, and model-provider auth
//! identity remain owned by vendored `codex-login` so Epiphany stays a modified
//! Codex-derived backend instead of a clean-room impersonator.
//!
//! Keep this export surface deliberately smaller than `codex-login`. The vendored
//! crate remains intact for clean upstream merges; this bridge exposes only what
//! Epiphany's subscription transport currently consumes.

pub use codex_login::AuthCredentialsStoreMode;
pub use codex_login::AuthManager;
pub use codex_login::AuthMode;
pub use codex_login::CodexAuth;
pub use codex_login::default_client::build_reqwest_client;

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
