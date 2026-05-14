use std::path::PathBuf;
use std::sync::Arc;
use std::sync::RwLock;

use base64::Engine;
use chrono::DateTime;
use chrono::Utc;
use codex_client::build_reqwest_client_with_custom_ca;
use codex_client::with_chatgpt_cloudflare_cookie_store;
use reqwest::StatusCode;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;

pub const OPENAI_API_KEY_ENV_VAR: &str = "OPENAI_API_KEY";
pub const CODEX_API_KEY_ENV_VAR: &str = "CODEX_API_KEY";
pub const REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR: &str = "CODEX_REFRESH_TOKEN_URL_OVERRIDE";
const REFRESH_TOKEN_URL: &str = "https://auth.openai.com/oauth/token";
const CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
const TOKEN_REFRESH_INTERVAL_MINUTES: i64 = 8;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthCredentialsStoreMode {
    #[default]
    File,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    ApiKey,
    Chatgpt,
    #[serde(rename = "chatgptAuthTokens")]
    ChatgptAuthTokens,
    #[serde(rename = "agentIdentity")]
    AgentIdentity,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AccountPlanType {
    #[default]
    Free,
    Go,
    Plus,
    Pro,
    ProLite,
    Team,
    #[serde(rename = "self_serve_business_usage_based")]
    SelfServeBusinessUsageBased,
    Business,
    #[serde(rename = "enterprise_cbp_usage_based")]
    EnterpriseCbpUsageBased,
    Enterprise,
    Edu,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ChatGptPlanType {
    Known(KnownChatGptPlan),
    Unknown(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KnownChatGptPlan {
    Free,
    Go,
    Plus,
    Pro,
    ProLite,
    Team,
    #[serde(rename = "self_serve_business_usage_based")]
    SelfServeBusinessUsageBased,
    Business,
    #[serde(rename = "enterprise_cbp_usage_based")]
    EnterpriseCbpUsageBased,
    #[serde(alias = "hc")]
    Enterprise,
    Edu,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
pub struct TokenData {
    #[serde(
        deserialize_with = "deserialize_id_token",
        serialize_with = "serialize_id_token"
    )]
    pub id_token: IdTokenInfo,
    pub access_token: String,
    pub refresh_token: String,
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct IdTokenInfo {
    pub email: Option<String>,
    pub chatgpt_plan_type: Option<ChatGptPlanType>,
    pub chatgpt_user_id: Option<String>,
    pub chatgpt_account_id: Option<String>,
    pub chatgpt_account_is_fedramp: bool,
    pub raw_jwt: String,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct AuthDotJson {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<AuthMode>,
    #[serde(rename = "OPENAI_API_KEY")]
    pub openai_api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tokens: Option<TokenData>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_refresh: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_identity: Option<Value>,
}

#[derive(Debug, Clone)]
pub enum CodexAuth {
    ApiKey(ApiKeyAuth),
    Chatgpt(ChatgptAuth),
    ChatgptAuthTokens(ChatgptAuthTokens),
    AgentIdentity(AgentIdentityAuth),
}

#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    api_key: String,
}

#[derive(Debug, Clone)]
pub struct ChatgptAuth {
    auth: AuthDotJson,
    codex_home: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ChatgptAuthTokens {
    auth: AuthDotJson,
}

#[derive(Debug, Clone)]
pub struct AgentIdentityAuth {
    account_id: Option<String>,
    plan_type: Option<AccountPlanType>,
    chatgpt_account_is_fedramp: bool,
}

impl CodexAuth {
    pub fn from_api_key(api_key: impl Into<String>) -> Self {
        Self::ApiKey(ApiKeyAuth {
            api_key: api_key.into(),
        })
    }

    pub fn is_api_key_auth(&self) -> bool {
        matches!(self, Self::ApiKey(_))
    }

    pub fn is_chatgpt_auth(&self) -> bool {
        matches!(self, Self::Chatgpt(_) | Self::ChatgptAuthTokens(_))
    }

    pub fn auth_mode(&self) -> AuthMode {
        match self {
            Self::ApiKey(_) => AuthMode::ApiKey,
            Self::Chatgpt(_) => AuthMode::Chatgpt,
            Self::ChatgptAuthTokens(_) => AuthMode::ChatgptAuthTokens,
            Self::AgentIdentity(_) => AuthMode::AgentIdentity,
        }
    }

    pub fn uses_codex_backend(&self) -> bool {
        matches!(
            self,
            Self::Chatgpt(_) | Self::ChatgptAuthTokens(_) | Self::AgentIdentity(_)
        )
    }

    pub fn get_token(&self) -> std::io::Result<String> {
        match self {
            Self::ApiKey(auth) => Ok(auth.api_key.clone()),
            Self::Chatgpt(auth) => auth.access_token(),
            Self::ChatgptAuthTokens(auth) => auth.access_token(),
            Self::AgentIdentity(_) => Err(std::io::Error::other(
                "agent identity auth is not implemented in Epiphany auth spine",
            )),
        }
    }

    pub fn get_account_id(&self) -> Option<String> {
        match self {
            Self::ApiKey(_) => None,
            Self::Chatgpt(auth) => auth.account_id(),
            Self::ChatgptAuthTokens(auth) => auth.account_id(),
            Self::AgentIdentity(auth) => auth.account_id.clone(),
        }
    }

    pub fn account_plan_type(&self) -> Option<AccountPlanType> {
        match self {
            Self::ApiKey(_) => None,
            Self::Chatgpt(auth) => auth.account_plan_type(),
            Self::ChatgptAuthTokens(auth) => auth.account_plan_type(),
            Self::AgentIdentity(auth) => auth.plan_type,
        }
    }

    pub fn is_fedramp_account(&self) -> bool {
        match self {
            Self::ApiKey(_) => false,
            Self::Chatgpt(auth) => auth.is_fedramp_account(),
            Self::ChatgptAuthTokens(auth) => auth.is_fedramp_account(),
            Self::AgentIdentity(auth) => auth.chatgpt_account_is_fedramp,
        }
    }
}

impl ChatgptAuth {
    fn access_token(&self) -> std::io::Result<String> {
        let tokens = self.tokens()?;
        let should_refresh = match self.auth.last_refresh {
            Some(last_refresh) => {
                Utc::now().signed_duration_since(last_refresh).num_minutes()
                    >= TOKEN_REFRESH_INTERVAL_MINUTES
            }
            None => true,
        };
        if should_refresh && !tokens.refresh_token.is_empty() {
            let previous_access_token = tokens.access_token.clone();
            return refresh_and_persist_tokens(
                self.codex_home.clone(),
                tokens.refresh_token.clone(),
            )
            .map(|auth| {
                auth.tokens
                    .map(|tokens| tokens.access_token)
                    .unwrap_or(previous_access_token)
            });
        }
        Ok(tokens.access_token.clone())
    }

    fn tokens(&self) -> std::io::Result<&TokenData> {
        self.auth
            .tokens
            .as_ref()
            .ok_or_else(|| std::io::Error::other("ChatGPT token data is not available"))
    }

    fn account_id(&self) -> Option<String> {
        token_account_id(self.auth.tokens.as_ref())
    }

    fn account_plan_type(&self) -> Option<AccountPlanType> {
        token_account_plan_type(self.auth.tokens.as_ref())
    }

    fn is_fedramp_account(&self) -> bool {
        token_is_fedramp(self.auth.tokens.as_ref())
    }
}

impl ChatgptAuthTokens {
    fn access_token(&self) -> std::io::Result<String> {
        self.auth
            .tokens
            .as_ref()
            .map(|tokens| tokens.access_token.clone())
            .ok_or_else(|| std::io::Error::other("ChatGPT token data is not available"))
    }

    fn account_id(&self) -> Option<String> {
        token_account_id(self.auth.tokens.as_ref())
    }

    fn account_plan_type(&self) -> Option<AccountPlanType> {
        token_account_plan_type(self.auth.tokens.as_ref())
    }

    fn is_fedramp_account(&self) -> bool {
        token_is_fedramp(self.auth.tokens.as_ref())
    }
}

#[derive(Debug)]
pub struct AuthManager {
    codex_home: PathBuf,
    enable_codex_api_key_env: bool,
    cached_auth: RwLock<Option<CodexAuth>>,
}

impl AuthManager {
    pub fn shared(
        codex_home: PathBuf,
        enable_codex_api_key_env: bool,
        _auth_credentials_store_mode: AuthCredentialsStoreMode,
        _chatgpt_base_url: Option<String>,
    ) -> Arc<Self> {
        Arc::new(Self::new(codex_home, enable_codex_api_key_env))
    }

    pub fn new(codex_home: PathBuf, enable_codex_api_key_env: bool) -> Self {
        let cached_auth = load_auth(&codex_home, enable_codex_api_key_env)
            .ok()
            .flatten();
        Self {
            codex_home,
            enable_codex_api_key_env,
            cached_auth: RwLock::new(cached_auth),
        }
    }

    pub async fn auth(&self) -> Option<CodexAuth> {
        if self.enable_codex_api_key_env
            && let Some(api_key) = read_codex_api_key_from_env()
        {
            return Some(CodexAuth::from_api_key(api_key));
        }
        if let Ok(guard) = self.cached_auth.read()
            && let Some(auth) = guard.clone()
        {
            return Some(auth);
        }
        let auth = load_auth(&self.codex_home, self.enable_codex_api_key_env)
            .ok()
            .flatten();
        if let Ok(mut guard) = self.cached_auth.write() {
            *guard = auth.clone();
        }
        auth
    }

    pub fn auth_mode(&self) -> Option<AuthMode> {
        if self.enable_codex_api_key_env && read_codex_api_key_from_env().is_some() {
            return Some(AuthMode::ApiKey);
        }
        self.cached_auth
            .read()
            .ok()
            .and_then(|guard| guard.as_ref().map(CodexAuth::auth_mode))
    }

    pub fn codex_api_key_env_enabled(&self) -> bool {
        self.enable_codex_api_key_env
    }
}

pub fn read_openai_api_key_from_env() -> Option<String> {
    read_trimmed_env(OPENAI_API_KEY_ENV_VAR)
}

pub fn read_codex_api_key_from_env() -> Option<String> {
    read_trimmed_env(CODEX_API_KEY_ENV_VAR)
}

fn read_trimmed_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn load_auth(
    codex_home: &PathBuf,
    enable_codex_api_key_env: bool,
) -> std::io::Result<Option<CodexAuth>> {
    if enable_codex_api_key_env && let Some(api_key) = read_codex_api_key_from_env() {
        return Ok(Some(CodexAuth::from_api_key(api_key)));
    }
    if let Some(api_key) = read_openai_api_key_from_env() {
        return Ok(Some(CodexAuth::from_api_key(api_key)));
    }
    let auth_path = codex_home.join("auth.json");
    let contents = match std::fs::read_to_string(&auth_path) {
        Ok(contents) => contents,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err),
    };
    let auth_dot_json: AuthDotJson = serde_json::from_str(&contents)?;
    build_auth(codex_home.clone(), auth_dot_json).map(Some)
}

fn build_auth(codex_home: PathBuf, auth_dot_json: AuthDotJson) -> std::io::Result<CodexAuth> {
    if let Some(api_key) = auth_dot_json.openai_api_key.as_ref() {
        return Ok(CodexAuth::from_api_key(api_key.clone()));
    }
    match auth_dot_json.auth_mode.unwrap_or(AuthMode::Chatgpt) {
        AuthMode::ApiKey => auth_dot_json
            .openai_api_key
            .map(CodexAuth::from_api_key)
            .ok_or_else(|| std::io::Error::other("OpenAI API key is not available")),
        AuthMode::Chatgpt => Ok(CodexAuth::Chatgpt(ChatgptAuth {
            auth: auth_dot_json,
            codex_home,
        })),
        AuthMode::ChatgptAuthTokens => Ok(CodexAuth::ChatgptAuthTokens(ChatgptAuthTokens {
            auth: auth_dot_json,
        })),
        AuthMode::AgentIdentity => Ok(CodexAuth::AgentIdentity(AgentIdentityAuth {
            account_id: None,
            plan_type: None,
            chatgpt_account_is_fedramp: false,
        })),
    }
}

fn token_account_id(tokens: Option<&TokenData>) -> Option<String> {
    tokens.and_then(|tokens| {
        tokens
            .account_id
            .clone()
            .or_else(|| tokens.id_token.chatgpt_account_id.clone())
    })
}

fn token_account_plan_type(tokens: Option<&TokenData>) -> Option<AccountPlanType> {
    let plan = tokens.and_then(|tokens| tokens.id_token.chatgpt_plan_type.as_ref())?;
    match plan {
        ChatGptPlanType::Known(KnownChatGptPlan::Free) => Some(AccountPlanType::Free),
        ChatGptPlanType::Known(KnownChatGptPlan::Go) => Some(AccountPlanType::Go),
        ChatGptPlanType::Known(KnownChatGptPlan::Plus) => Some(AccountPlanType::Plus),
        ChatGptPlanType::Known(KnownChatGptPlan::Pro) => Some(AccountPlanType::Pro),
        ChatGptPlanType::Known(KnownChatGptPlan::ProLite) => Some(AccountPlanType::ProLite),
        ChatGptPlanType::Known(KnownChatGptPlan::Team) => Some(AccountPlanType::Team),
        ChatGptPlanType::Known(KnownChatGptPlan::SelfServeBusinessUsageBased) => {
            Some(AccountPlanType::SelfServeBusinessUsageBased)
        }
        ChatGptPlanType::Known(KnownChatGptPlan::Business) => Some(AccountPlanType::Business),
        ChatGptPlanType::Known(KnownChatGptPlan::EnterpriseCbpUsageBased) => {
            Some(AccountPlanType::EnterpriseCbpUsageBased)
        }
        ChatGptPlanType::Known(KnownChatGptPlan::Enterprise) => Some(AccountPlanType::Enterprise),
        ChatGptPlanType::Known(KnownChatGptPlan::Edu) => Some(AccountPlanType::Edu),
        ChatGptPlanType::Unknown(_) => Some(AccountPlanType::Unknown),
    }
}

fn token_is_fedramp(tokens: Option<&TokenData>) -> bool {
    tokens
        .map(|tokens| tokens.id_token.chatgpt_account_is_fedramp)
        .unwrap_or(false)
}

fn refresh_and_persist_tokens(
    codex_home: PathBuf,
    refresh_token: String,
) -> std::io::Result<AuthDotJson> {
    let response = request_chatgpt_token_refresh_blocking(refresh_token)?;
    let auth_path = codex_home.join("auth.json");
    let contents = std::fs::read_to_string(&auth_path)?;
    let mut auth_dot_json: AuthDotJson = serde_json::from_str(&contents)?;
    let tokens = auth_dot_json.tokens.get_or_insert_with(TokenData::default);
    if let Some(id_token) = response.id_token {
        tokens.id_token = parse_chatgpt_jwt_claims(&id_token).map_err(std::io::Error::other)?;
    }
    if let Some(access_token) = response.access_token {
        tokens.access_token = access_token;
    }
    if let Some(refresh_token) = response.refresh_token {
        tokens.refresh_token = refresh_token;
    }
    auth_dot_json.last_refresh = Some(Utc::now());
    std::fs::write(&auth_path, serde_json::to_string_pretty(&auth_dot_json)?)?;
    Ok(auth_dot_json)
}

fn request_chatgpt_token_refresh_blocking(
    refresh_token: String,
) -> std::io::Result<RefreshResponse> {
    let refresh_request = RefreshRequest {
        client_id: CLIENT_ID,
        grant_type: "refresh_token",
        refresh_token,
    };
    let endpoint = std::env::var(REFRESH_TOKEN_URL_OVERRIDE_ENV_VAR)
        .unwrap_or_else(|_| REFRESH_TOKEN_URL.to_string());
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!(
            "epiphany-openai-auth-spine/{}",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .map_err(std::io::Error::other)?;
    let response = client
        .post(endpoint)
        .header("Content-Type", "application/json")
        .json(&refresh_request)
        .send()
        .map_err(std::io::Error::other)?;
    let status = response.status();
    if status.is_success() {
        return response
            .json::<RefreshResponse>()
            .map_err(std::io::Error::other);
    }
    let body = response.text().unwrap_or_default();
    if status == StatusCode::UNAUTHORIZED {
        return Err(std::io::Error::other(classify_refresh_token_failure(&body)));
    }
    Err(std::io::Error::other(format!(
        "failed to refresh ChatGPT token: {status}: {}",
        try_parse_error_message(&body)
    )))
}

#[derive(Serialize)]
struct RefreshRequest {
    client_id: &'static str,
    grant_type: &'static str,
    refresh_token: String,
}

#[derive(Deserialize, Clone)]
struct RefreshResponse {
    id_token: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{message}")]
pub struct RefreshTokenFailedError {
    pub reason: RefreshTokenFailedReason,
    pub message: String,
}

impl RefreshTokenFailedError {
    pub fn new(reason: RefreshTokenFailedReason, message: impl Into<String>) -> Self {
        Self {
            reason,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshTokenFailedReason {
    Expired,
    Exhausted,
    Revoked,
    Other,
}

fn classify_refresh_token_failure(body: &str) -> RefreshTokenFailedError {
    let code = extract_refresh_token_error_code(body);
    let reason = match code.as_deref().map(str::to_ascii_lowercase).as_deref() {
        Some("refresh_token_expired") => RefreshTokenFailedReason::Expired,
        Some("refresh_token_reused") => RefreshTokenFailedReason::Exhausted,
        Some("refresh_token_invalidated") => RefreshTokenFailedReason::Revoked,
        _ => RefreshTokenFailedReason::Other,
    };
    RefreshTokenFailedError::new(reason, "ChatGPT refresh token could not be refreshed")
}

fn extract_refresh_token_error_code(body: &str) -> Option<String> {
    let Value::Object(map) = serde_json::from_str::<Value>(body).ok()? else {
        return None;
    };
    if let Some(error_value) = map.get("error") {
        match error_value {
            Value::Object(obj) => {
                if let Some(code) = obj.get("code").and_then(Value::as_str) {
                    return Some(code.to_string());
                }
            }
            Value::String(code) => return Some(code.to_string()),
            _ => {}
        }
    }
    map.get("code").and_then(Value::as_str).map(str::to_string)
}

fn try_parse_error_message(body: &str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| value.get("error").cloned())
        .and_then(|error| match error {
            Value::String(message) => Some(message),
            Value::Object(mut map) => map
                .remove("message")
                .and_then(|message| message.as_str().map(ToString::to_string)),
            _ => None,
        })
        .unwrap_or_else(|| body.to_string())
}

#[derive(Deserialize)]
struct IdClaims {
    #[serde(default)]
    email: Option<String>,
    #[serde(rename = "https://api.openai.com/profile", default)]
    profile: Option<ProfileClaims>,
    #[serde(rename = "https://api.openai.com/auth", default)]
    auth: Option<AuthClaims>,
}

#[derive(Deserialize)]
struct ProfileClaims {
    #[serde(default)]
    email: Option<String>,
}

#[derive(Deserialize)]
struct AuthClaims {
    #[serde(default)]
    chatgpt_plan_type: Option<ChatGptPlanType>,
    #[serde(default)]
    chatgpt_user_id: Option<String>,
    #[serde(default)]
    user_id: Option<String>,
    #[serde(default)]
    chatgpt_account_id: Option<String>,
    #[serde(default)]
    chatgpt_account_is_fedramp: bool,
}

#[derive(Debug, Error)]
pub enum IdTokenInfoError {
    #[error("invalid ID token format")]
    InvalidFormat,
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

fn decode_jwt_payload<T: DeserializeOwned>(jwt: &str) -> Result<T, IdTokenInfoError> {
    let mut parts = jwt.split('.');
    let (_header_b64, payload_b64, _sig_b64) = match (parts.next(), parts.next(), parts.next()) {
        (Some(h), Some(p), Some(s)) if !h.is_empty() && !p.is_empty() && !s.is_empty() => (h, p, s),
        _ => return Err(IdTokenInfoError::InvalidFormat),
    };
    let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload_b64)?;
    Ok(serde_json::from_slice(&payload_bytes)?)
}

pub fn parse_chatgpt_jwt_claims(jwt: &str) -> Result<IdTokenInfo, IdTokenInfoError> {
    let claims: IdClaims = decode_jwt_payload(jwt)?;
    let email = claims
        .email
        .or_else(|| claims.profile.and_then(|profile| profile.email));
    match claims.auth {
        Some(auth) => Ok(IdTokenInfo {
            email,
            raw_jwt: jwt.to_string(),
            chatgpt_plan_type: auth.chatgpt_plan_type,
            chatgpt_user_id: auth.chatgpt_user_id.or(auth.user_id),
            chatgpt_account_id: auth.chatgpt_account_id,
            chatgpt_account_is_fedramp: auth.chatgpt_account_is_fedramp,
        }),
        None => Ok(IdTokenInfo {
            email,
            raw_jwt: jwt.to_string(),
            chatgpt_plan_type: None,
            chatgpt_user_id: None,
            chatgpt_account_id: None,
            chatgpt_account_is_fedramp: false,
        }),
    }
}

fn deserialize_id_token<'de, D>(deserializer: D) -> Result<IdTokenInfo, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    parse_chatgpt_jwt_claims(&value).map_err(serde::de::Error::custom)
}

fn serialize_id_token<S>(id_token: &IdTokenInfo, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&id_token.raw_jwt)
}

pub mod default_client {
    use super::*;

    pub fn build_reqwest_client() -> reqwest::Client {
        try_build_reqwest_client().unwrap_or_else(|error| {
            tracing::warn!(error = %error, "failed to build Epiphany OpenAI auth client");
            with_chatgpt_cloudflare_cookie_store(reqwest::Client::builder())
                .build()
                .unwrap_or_else(|_| reqwest::Client::new())
        })
    }

    fn try_build_reqwest_client()
    -> Result<reqwest::Client, codex_client::BuildCustomCaTransportError> {
        let mut headers = HeaderMap::new();
        headers.insert("originator", HeaderValue::from_static("epiphany"));
        let builder = reqwest::Client::builder()
            .user_agent(format!(
                "epiphany-openai-auth-spine/{}",
                env!("CARGO_PKG_VERSION")
            ))
            .default_headers(headers);
        build_reqwest_client_with_custom_ca(with_chatgpt_cloudflare_cookie_store(builder))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_auth_exposes_openai_token() {
        let auth = CodexAuth::from_api_key("sk-test");
        assert!(auth.is_api_key_auth());
        assert_eq!(auth.get_token().unwrap(), "sk-test");
        assert!(!auth.uses_codex_backend());
    }

    #[test]
    fn parses_minimal_api_key_auth_json() {
        let auth: AuthDotJson =
            serde_json::from_str(r#"{"auth_mode":"apikey","OPENAI_API_KEY":"sk-test"}"#).unwrap();
        let built = build_auth(PathBuf::from("."), auth).unwrap();
        assert_eq!(built.auth_mode(), AuthMode::ApiKey);
    }
}
