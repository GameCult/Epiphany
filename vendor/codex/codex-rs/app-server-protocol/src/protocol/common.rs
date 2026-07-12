use std::path::Path;

use crate::JSONRPCNotification;
use crate::JSONRPCRequest;
use crate::RequestId;
use crate::export::GeneratedSchema;
use crate::export::write_json_schema;
use crate::protocol::v1;
use crate::protocol::v2;
use codex_experimental_api_macros::ExperimentalApi;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use strum_macros::Display;
use ts_rs::TS;

/// Authentication mode for OpenAI-backed providers.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Display, JsonSchema, TS)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    /// OpenAI API key provided by the caller and stored by Codex.
    ApiKey,
    /// ChatGPT OAuth managed by Codex (tokens persisted and refreshed by Codex).
    Chatgpt,
    /// [UNSTABLE] FOR OPENAI INTERNAL USE ONLY - DO NOT USE.
    ///
    /// ChatGPT auth tokens are supplied by an external host app and are only
    /// stored in memory. Token refresh must be handled by the external host app.
    #[serde(rename = "chatgptAuthTokens")]
    #[ts(rename = "chatgptAuthTokens")]
    #[strum(serialize = "chatgptAuthTokens")]
    ChatgptAuthTokens,
    /// Programmatic Codex auth backed by a registered Agent Identity.
    #[serde(rename = "agentIdentity")]
    #[ts(rename = "agentIdentity")]
    #[strum(serialize = "agentIdentity")]
    AgentIdentity,
}

macro_rules! experimental_reason_expr {
    // If a request variant is explicitly marked experimental, that reason wins.
    (variant $variant:ident, #[experimental($reason:expr)] $params:ident $(, $inspect_params:tt)?) => {
        Some($reason)
    };
    // `inspect_params: true` is used when a method is mostly stable but needs
    // field-level gating from its params type (for example, ThreadStart).
    (variant $variant:ident, $params:ident, true) => {
        crate::experimental_api::ExperimentalApi::experimental_reason($params)
    };
    (variant $variant:ident, $params:ident $(, $inspect_params:tt)?) => {
        None
    };
}

macro_rules! experimental_method_entry {
    (#[experimental($reason:expr)] => $wire:literal) => {
        $wire
    };
    (#[experimental($reason:expr)]) => {
        $reason
    };
    ($($tt:tt)*) => {
        ""
    };
}

macro_rules! experimental_type_entry {
    (#[experimental($reason:expr)] $ty:ty) => {
        stringify!($ty)
    };
    ($ty:ty) => {
        ""
    };
}

/// Generates an `enum ClientRequest` where each variant is a request that the
/// client can send to the server. Each variant has associated `params` and
/// `response` types. Also generates a `export_client_responses()` function to
/// export all response types to TypeScript.
macro_rules! client_request_definitions {
    (
        $(
            $(#[experimental($reason:expr)])?
            $(#[doc = $variant_doc:literal])*
            $variant:ident $(=> $wire:literal)? {
                params: $(#[$params_meta:meta])* $params:ty,
                $(inspect_params: $inspect_params:tt,)?
                response: $response:ty,
            }
        ),* $(,)?
    ) => {
        /// Request from the client to the server.
        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS)]
        #[serde(tag = "method", rename_all = "camelCase")]
        pub enum ClientRequest {
            $(
                $(#[doc = $variant_doc])*
                $(#[serde(rename = $wire)] #[ts(rename = $wire)])?
                $variant {
                    #[serde(rename = "id")]
                    request_id: RequestId,
                    $(#[$params_meta])*
                    params: $params,
                },
            )*
        }

        impl ClientRequest {
            pub fn id(&self) -> &RequestId {
                match self {
                    $(Self::$variant { request_id, .. } => request_id,)*
                }
            }

            pub fn method(&self) -> String {
                serde_json::to_value(self)
                    .ok()
                    .and_then(|value| {
                        value
                            .get("method")
                            .and_then(serde_json::Value::as_str)
                            .map(str::to_owned)
                    })
                    .unwrap_or_else(|| "<unknown>".to_string())
            }
        }

        /// Typed response from the server to the client.
        #[derive(Serialize, Deserialize, Debug, Clone)]
        #[serde(tag = "method", rename_all = "camelCase")]
        pub enum ClientResponse {
            $(
                $(#[doc = $variant_doc])*
                $(#[serde(rename = $wire)])?
                $variant {
                    #[serde(rename = "id")]
                    request_id: RequestId,
                    response: $response,
                },
            )*
        }

        impl ClientResponse {
            pub fn id(&self) -> &RequestId {
                match self {
                    $(Self::$variant { request_id, .. } => request_id,)*
                }
            }

            pub fn method(&self) -> String {
                serde_json::to_value(self)
                    .ok()
                    .and_then(|value| {
                        value
                            .get("method")
                            .and_then(serde_json::Value::as_str)
                            .map(str::to_owned)
                    })
                    .unwrap_or_else(|| "<unknown>".to_string())
            }
        }

        impl crate::experimental_api::ExperimentalApi for ClientRequest {
            fn experimental_reason(&self) -> Option<&'static str> {
                match self {
                    $(
                        Self::$variant { params: _params, .. } => {
                            experimental_reason_expr!(
                                variant $variant,
                                $(#[experimental($reason)])?
                                _params
                                $(, $inspect_params)?
                            )
                        }
                    )*
                }
            }
        }

        pub(crate) const EXPERIMENTAL_CLIENT_METHODS: &[&str] = &[
            $(
                experimental_method_entry!($(#[experimental($reason)])? $(=> $wire)?),
            )*
        ];
        pub(crate) const EXPERIMENTAL_CLIENT_METHOD_PARAM_TYPES: &[&str] = &[
            $(
                experimental_type_entry!($(#[experimental($reason)])? $params),
            )*
        ];
        pub(crate) const EXPERIMENTAL_CLIENT_METHOD_RESPONSE_TYPES: &[&str] = &[
            $(
                experimental_type_entry!($(#[experimental($reason)])? $response),
            )*
        ];

        pub fn export_client_responses(
            out_dir: &::std::path::Path,
        ) -> ::std::result::Result<(), ::ts_rs::ExportError> {
            $(
                <$response as ::ts_rs::TS>::export_all_to(out_dir)?;
            )*
            Ok(())
        }

        pub(crate) fn visit_client_response_types(v: &mut impl ::ts_rs::TypeVisitor) {
            $(
                v.visit::<$response>();
            )*
        }

        #[allow(clippy::vec_init_then_push)]
        pub fn export_client_response_schemas(
            out_dir: &::std::path::Path,
        ) -> ::anyhow::Result<Vec<GeneratedSchema>> {
            let mut schemas = Vec::new();
            $(
                schemas.push(write_json_schema::<$response>(out_dir, stringify!($response))?);
            )*
            Ok(schemas)
        }

        #[allow(clippy::vec_init_then_push)]
        pub fn export_client_param_schemas(
            out_dir: &::std::path::Path,
        ) -> ::anyhow::Result<Vec<GeneratedSchema>> {
            let mut schemas = Vec::new();
            $(
                schemas.push(write_json_schema::<$params>(out_dir, stringify!($params))?);
            )*
            Ok(schemas)
        }
    };
}

client_request_definitions! {
    Initialize {
        params: v1::InitializeParams,
        response: v1::InitializeResponse,
    },

    /// NEW APIs
    // Thread lifecycle
    // Uses `inspect_params` because only some fields are experimental.
    ThreadStart => "thread/start" {
        params: v2::ThreadStartParams,
        inspect_params: true,
        response: v2::ThreadStartResponse,
    },
    ThreadResume => "thread/resume" {
        params: v2::ThreadResumeParams,
        inspect_params: true,
        response: v2::ThreadResumeResponse,
    },
    ThreadFork => "thread/fork" {
        params: v2::ThreadForkParams,
        inspect_params: true,
        response: v2::ThreadForkResponse,
    },
    ThreadArchive => "thread/archive" {
        params: v2::ThreadArchiveParams,
        response: v2::ThreadArchiveResponse,
    },
    ThreadUnsubscribe => "thread/unsubscribe" {
        params: v2::ThreadUnsubscribeParams,
        response: v2::ThreadUnsubscribeResponse,
    },
    #[experimental("thread/increment_elicitation")]
    /// Increment the thread-local out-of-band elicitation counter.
    ///
    /// This is used by external helpers to pause timeout accounting while a user
    /// approval or other elicitation is pending outside the app-server request flow.
    ThreadIncrementElicitation => "thread/increment_elicitation" {
        params: v2::ThreadIncrementElicitationParams,
        response: v2::ThreadIncrementElicitationResponse,
    },
    #[experimental("thread/decrement_elicitation")]
    /// Decrement the thread-local out-of-band elicitation counter.
    ///
    /// When the count reaches zero, timeout accounting resumes for the thread.
    ThreadDecrementElicitation => "thread/decrement_elicitation" {
        params: v2::ThreadDecrementElicitationParams,
        response: v2::ThreadDecrementElicitationResponse,
    },
    ThreadSetName => "thread/name/set" {
        params: v2::ThreadSetNameParams,
        response: v2::ThreadSetNameResponse,
    },
    ThreadMetadataUpdate => "thread/metadata/update" {
        params: v2::ThreadMetadataUpdateParams,
        response: v2::ThreadMetadataUpdateResponse,
    },
    #[experimental("thread/memoryMode/set")]
    ThreadMemoryModeSet => "thread/memoryMode/set" {
        params: v2::ThreadMemoryModeSetParams,
        response: v2::ThreadMemoryModeSetResponse,
    },
    #[experimental("memory/reset")]
    MemoryReset => "memory/reset" {
        params: #[ts(type = "undefined")] #[serde(skip_serializing_if = "Option::is_none")] Option<()>,
        response: v2::MemoryResetResponse,
    },
    ThreadUnarchive => "thread/unarchive" {
        params: v2::ThreadUnarchiveParams,
        response: v2::ThreadUnarchiveResponse,
    },
    ThreadCompactStart => "thread/compact/start" {
        params: v2::ThreadCompactStartParams,
        response: v2::ThreadCompactStartResponse,
    },
    ThreadShellCommand => "thread/shellCommand" {
        params: v2::ThreadShellCommandParams,
        response: v2::ThreadShellCommandResponse,
    },
    ThreadApproveGuardianDeniedAction => "thread/approveGuardianDeniedAction" {
        params: v2::ThreadApproveGuardianDeniedActionParams,
        response: v2::ThreadApproveGuardianDeniedActionResponse,
    },
    #[experimental("thread/backgroundTerminals/clean")]
    ThreadBackgroundTerminalsClean => "thread/backgroundTerminals/clean" {
        params: v2::ThreadBackgroundTerminalsCleanParams,
        response: v2::ThreadBackgroundTerminalsCleanResponse,
    },
    ThreadRollback => "thread/rollback" {
        params: v2::ThreadRollbackParams,
        response: v2::ThreadRollbackResponse,
    },
    ThreadList => "thread/list" {
        params: v2::ThreadListParams,
        response: v2::ThreadListResponse,
    },
    ThreadLoadedList => "thread/loaded/list" {
        params: v2::ThreadLoadedListParams,
        response: v2::ThreadLoadedListResponse,
    },
    ThreadRead => "thread/read" {
        params: v2::ThreadReadParams,
        response: v2::ThreadReadResponse,
    },
    #[experimental("thread/epiphany/view")]
    ThreadEpiphanyView => "thread/epiphany/view" {
        params: v2::ThreadEpiphanyViewParams,
        response: v2::ThreadEpiphanyViewResponse,
    },
    #[experimental("thread/epiphany/roleResult")]
    ThreadEpiphanyRoleResult => "thread/epiphany/roleResult" {
        params: v2::ThreadEpiphanyRoleResultParams,
        response: v2::ThreadEpiphanyRoleResultResponse,
    },
    #[experimental("thread/epiphany/freshness")]
    ThreadEpiphanyFreshness => "thread/epiphany/freshness" {
        params: v2::ThreadEpiphanyFreshnessParams,
        response: v2::ThreadEpiphanyFreshnessResponse,
    },
    #[experimental("thread/epiphany/context")]
    ThreadEpiphanyContext => "thread/epiphany/context" {
        params: v2::ThreadEpiphanyContextParams,
        response: v2::ThreadEpiphanyContextResponse,
    },
    #[experimental("thread/epiphany/graphQuery")]
    ThreadEpiphanyGraphQuery => "thread/epiphany/graphQuery" {
        params: v2::ThreadEpiphanyGraphQueryParams,
        response: v2::ThreadEpiphanyGraphQueryResponse,
    },
    #[experimental("thread/epiphany/reorientResult")]
    ThreadEpiphanyReorientResult => "thread/epiphany/reorientResult" {
        params: v2::ThreadEpiphanyReorientResultParams,
        response: v2::ThreadEpiphanyReorientResultResponse,
    },
    #[experimental("thread/epiphany/jobInterrupt")]
    ThreadEpiphanyJobInterrupt => "thread/epiphany/jobInterrupt" {
        params: v2::ThreadEpiphanyJobInterruptParams,
        response: v2::ThreadEpiphanyJobInterruptResponse,
    },
    #[experimental("thread/epiphany/update")]
    ThreadEpiphanyUpdate => "thread/epiphany/update" {
        params: v2::ThreadEpiphanyUpdateParams,
        response: v2::ThreadEpiphanyUpdateResponse,
    },
    ThreadTurnsList => "thread/turns/list" {
        params: v2::ThreadTurnsListParams,
        response: v2::ThreadTurnsListResponse,
    },
    /// Append raw Responses API items to the thread history without starting a user turn.
    ThreadInjectItems => "thread/inject_items" {
        params: v2::ThreadInjectItemsParams,
        response: v2::ThreadInjectItemsResponse,
    },
    DeviceKeyCreate => "device/key/create" {
        params: v2::DeviceKeyCreateParams,
        response: v2::DeviceKeyCreateResponse,
    },
    DeviceKeyPublic => "device/key/public" {
        params: v2::DeviceKeyPublicParams,
        response: v2::DeviceKeyPublicResponse,
    },
    DeviceKeySign => "device/key/sign" {
        params: v2::DeviceKeySignParams,
        response: v2::DeviceKeySignResponse,
    },
    FsReadFile => "fs/readFile" {
        params: v2::FsReadFileParams,
        response: v2::FsReadFileResponse,
    },
    FsWriteFile => "fs/writeFile" {
        params: v2::FsWriteFileParams,
        response: v2::FsWriteFileResponse,
    },
    FsCreateDirectory => "fs/createDirectory" {
        params: v2::FsCreateDirectoryParams,
        response: v2::FsCreateDirectoryResponse,
    },
    FsGetMetadata => "fs/getMetadata" {
        params: v2::FsGetMetadataParams,
        response: v2::FsGetMetadataResponse,
    },
    FsReadDirectory => "fs/readDirectory" {
        params: v2::FsReadDirectoryParams,
        response: v2::FsReadDirectoryResponse,
    },
    FsRemove => "fs/remove" {
        params: v2::FsRemoveParams,
        response: v2::FsRemoveResponse,
    },
    FsCopy => "fs/copy" {
        params: v2::FsCopyParams,
        response: v2::FsCopyResponse,
    },
    FsWatch => "fs/watch" {
        params: v2::FsWatchParams,
        response: v2::FsWatchResponse,
    },
    FsUnwatch => "fs/unwatch" {
        params: v2::FsUnwatchParams,
        response: v2::FsUnwatchResponse,
    },
    TurnStart => "turn/start" {
        params: v2::TurnStartParams,
        inspect_params: true,
        response: v2::TurnStartResponse,
    },
    TurnSteer => "turn/steer" {
        params: v2::TurnSteerParams,
        inspect_params: true,
        response: v2::TurnSteerResponse,
    },
    TurnInterrupt => "turn/interrupt" {
        params: v2::TurnInterruptParams,
        response: v2::TurnInterruptResponse,
    },
    #[experimental("thread/realtime/start")]
    ThreadRealtimeStart => "thread/realtime/start" {
        params: v2::ThreadRealtimeStartParams,
        response: v2::ThreadRealtimeStartResponse,
    },
    #[experimental("thread/realtime/appendAudio")]
    ThreadRealtimeAppendAudio => "thread/realtime/appendAudio" {
        params: v2::ThreadRealtimeAppendAudioParams,
        response: v2::ThreadRealtimeAppendAudioResponse,
    },
    #[experimental("thread/realtime/appendText")]
    ThreadRealtimeAppendText => "thread/realtime/appendText" {
        params: v2::ThreadRealtimeAppendTextParams,
        response: v2::ThreadRealtimeAppendTextResponse,
    },
    #[experimental("thread/realtime/stop")]
    ThreadRealtimeStop => "thread/realtime/stop" {
        params: v2::ThreadRealtimeStopParams,
        response: v2::ThreadRealtimeStopResponse,
    },
    #[experimental("thread/realtime/listVoices")]
    ThreadRealtimeListVoices => "thread/realtime/listVoices" {
        params: v2::ThreadRealtimeListVoicesParams,
        response: v2::ThreadRealtimeListVoicesResponse,
    },
    ReviewStart => "review/start" {
        params: v2::ReviewStartParams,
        response: v2::ReviewStartResponse,
    },

    ModelList => "model/list" {
        params: v2::ModelListParams,
        response: v2::ModelListResponse,
    },
    ExperimentalFeatureList => "experimentalFeature/list" {
        params: v2::ExperimentalFeatureListParams,
        response: v2::ExperimentalFeatureListResponse,
    },
    ExperimentalFeatureEnablementSet => "experimentalFeature/enablement/set" {
        params: v2::ExperimentalFeatureEnablementSetParams,
        response: v2::ExperimentalFeatureEnablementSetResponse,
    },
    #[experimental("collaborationMode/list")]
    /// Lists collaboration mode presets.
    CollaborationModeList => "collaborationMode/list" {
        params: v2::CollaborationModeListParams,
        response: v2::CollaborationModeListResponse,
    },
    #[experimental("mock/experimentalMethod")]
    /// Test-only method used to validate experimental gating.
    MockExperimentalMethod => "mock/experimentalMethod" {
        params: v2::MockExperimentalMethodParams,
        response: v2::MockExperimentalMethodResponse,
    },

    McpServerOauthLogin => "mcpServer/oauth/login" {
        params: v2::McpServerOauthLoginParams,
        response: v2::McpServerOauthLoginResponse,
    },

    McpServerRefresh => "config/mcpServer/reload" {
        params: #[ts(type = "undefined")] #[serde(skip_serializing_if = "Option::is_none")] Option<()>,
        response: v2::McpServerRefreshResponse,
    },

    McpServerStatusList => "mcpServerStatus/list" {
        params: v2::ListMcpServerStatusParams,
        response: v2::ListMcpServerStatusResponse,
    },

    McpResourceRead => "mcpServer/resource/read" {
        params: v2::McpResourceReadParams,
        response: v2::McpResourceReadResponse,
    },

    McpServerToolCall => "mcpServer/tool/call" {
        params: v2::McpServerToolCallParams,
        response: v2::McpServerToolCallResponse,
    },

    WindowsSandboxSetupStart => "windowsSandbox/setupStart" {
        params: v2::WindowsSandboxSetupStartParams,
        response: v2::WindowsSandboxSetupStartResponse,
    },

    LoginAccount => "account/login/start" {
        params: v2::LoginAccountParams,
        inspect_params: true,
        response: v2::LoginAccountResponse,
    },

    CancelLoginAccount => "account/login/cancel" {
        params: v2::CancelLoginAccountParams,
        response: v2::CancelLoginAccountResponse,
    },

    LogoutAccount => "account/logout" {
        params: #[ts(type = "undefined")] #[serde(skip_serializing_if = "Option::is_none")] Option<()>,
        response: v2::LogoutAccountResponse,
    },

    GetAccountRateLimits => "account/rateLimits/read" {
        params: #[ts(type = "undefined")] #[serde(skip_serializing_if = "Option::is_none")] Option<()>,
        response: v2::GetAccountRateLimitsResponse,
    },

    SendAddCreditsNudgeEmail => "account/sendAddCreditsNudgeEmail" {
        params: v2::SendAddCreditsNudgeEmailParams,
        response: v2::SendAddCreditsNudgeEmailResponse,
    },

    FeedbackUpload => "feedback/upload" {
        params: v2::FeedbackUploadParams,
        response: v2::FeedbackUploadResponse,
    },

    /// Execute a standalone command (argv vector) under the server's sandbox.
    OneOffCommandExec => "command/exec" {
        params: v2::CommandExecParams,
        response: v2::CommandExecResponse,
    },
    /// Write stdin bytes to a running `command/exec` session or close stdin.
    CommandExecWrite => "command/exec/write" {
        params: v2::CommandExecWriteParams,
        response: v2::CommandExecWriteResponse,
    },
    /// Terminate a running `command/exec` session by client-supplied `processId`.
    CommandExecTerminate => "command/exec/terminate" {
        params: v2::CommandExecTerminateParams,
        response: v2::CommandExecTerminateResponse,
    },
    /// Resize a running PTY-backed `command/exec` session by client-supplied `processId`.
    CommandExecResize => "command/exec/resize" {
        params: v2::CommandExecResizeParams,
        response: v2::CommandExecResizeResponse,
    },

    ConfigRead => "config/read" {
        params: v2::ConfigReadParams,
        response: v2::ConfigReadResponse,
    },
    ConfigValueWrite => "config/value/write" {
        params: v2::ConfigValueWriteParams,
        response: v2::ConfigWriteResponse,
    },
    ConfigBatchWrite => "config/batchWrite" {
        params: v2::ConfigBatchWriteParams,
        response: v2::ConfigWriteResponse,
    },

    ConfigRequirementsRead => "configRequirements/read" {
        params: #[ts(type = "undefined")] #[serde(skip_serializing_if = "Option::is_none")] Option<()>,
        response: v2::ConfigRequirementsReadResponse,
    },

    GetAccount => "account/read" {
        params: v2::GetAccountParams,
        response: v2::GetAccountResponse,
    },

    /// DEPRECATED APIs below
    GetConversationSummary {
        params: v1::GetConversationSummaryParams,
        response: v1::GetConversationSummaryResponse,
    },
    GitDiffToRemote {
        params: v1::GitDiffToRemoteParams,
        response: v1::GitDiffToRemoteResponse,
    },
    /// DEPRECATED in favor of GetAccount
    GetAuthStatus {
        params: v1::GetAuthStatusParams,
        response: v1::GetAuthStatusResponse,
    },
    FuzzyFileSearch {
        params: FuzzyFileSearchParams,
        response: FuzzyFileSearchResponse,
    },
    #[experimental("fuzzyFileSearch/sessionStart")]
    FuzzyFileSearchSessionStart => "fuzzyFileSearch/sessionStart" {
        params: FuzzyFileSearchSessionStartParams,
        response: FuzzyFileSearchSessionStartResponse,
    },
    #[experimental("fuzzyFileSearch/sessionUpdate")]
    FuzzyFileSearchSessionUpdate => "fuzzyFileSearch/sessionUpdate" {
        params: FuzzyFileSearchSessionUpdateParams,
        response: FuzzyFileSearchSessionUpdateResponse,
    },
    #[experimental("fuzzyFileSearch/sessionStop")]
    FuzzyFileSearchSessionStop => "fuzzyFileSearch/sessionStop" {
        params: FuzzyFileSearchSessionStopParams,
        response: FuzzyFileSearchSessionStopResponse,
    },
}

/// Generates an `enum ServerRequest` where each variant is a request that the
/// server can send to the client along with the corresponding params and
/// response types. It also generates helper types used by the app/server
/// infrastructure (payload enum, request constructor, and export helpers).
macro_rules! server_request_definitions {
    (
        $(
            $(#[$variant_meta:meta])*
            $variant:ident $(=> $wire:literal)? {
                params: $params:ty,
                response: $response:ty,
            }
        ),* $(,)?
    ) => {
        /// Request initiated from the server and sent to the client.
        #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS)]
        #[allow(clippy::large_enum_variant)]
        #[serde(tag = "method", rename_all = "camelCase")]
        pub enum ServerRequest {
            $(
                $(#[$variant_meta])*
                $(#[serde(rename = $wire)] #[ts(rename = $wire)])?
                $variant {
                    #[serde(rename = "id")]
                    request_id: RequestId,
                    params: $params,
                },
            )*
        }

        impl ServerRequest {
            pub fn id(&self) -> &RequestId {
                match self {
                    $(Self::$variant { request_id, .. } => request_id,)*
                }
            }
        }

        /// Typed response from the client to the server.
        #[derive(Serialize, Deserialize, Debug, Clone)]
        #[serde(tag = "method", rename_all = "camelCase")]
        pub enum ServerResponse {
            $(
                $(#[$variant_meta])*
                $(#[serde(rename = $wire)])?
                $variant {
                    #[serde(rename = "id")]
                    request_id: RequestId,
                    response: $response,
                },
            )*
        }

        impl ServerResponse {
            pub fn id(&self) -> &RequestId {
                match self {
                    $(Self::$variant { request_id, .. } => request_id,)*
                }
            }

            pub fn method(&self) -> String {
                serde_json::to_value(self)
                    .ok()
                    .and_then(|value| {
                        value
                            .get("method")
                            .and_then(serde_json::Value::as_str)
                            .map(str::to_owned)
                    })
                    .unwrap_or_else(|| "<unknown>".to_string())
            }
        }

        #[derive(Debug, Clone, PartialEq, JsonSchema)]
        #[allow(clippy::large_enum_variant)]
        pub enum ServerRequestPayload {
            $( $variant($params), )*
        }

        impl ServerRequestPayload {
            pub fn request_with_id(self, request_id: RequestId) -> ServerRequest {
                match self {
                    $(Self::$variant(params) => ServerRequest::$variant { request_id, params },)*
                }
            }
        }

        pub fn export_server_responses(
            out_dir: &::std::path::Path,
        ) -> ::std::result::Result<(), ::ts_rs::ExportError> {
            $(
                <$response as ::ts_rs::TS>::export_all_to(out_dir)?;
            )*
            Ok(())
        }

        pub(crate) fn visit_server_response_types(v: &mut impl ::ts_rs::TypeVisitor) {
            $(
                v.visit::<$response>();
            )*
        }

        #[allow(clippy::vec_init_then_push)]
        pub fn export_server_response_schemas(
            out_dir: &Path,
        ) -> ::anyhow::Result<Vec<GeneratedSchema>> {
            let mut schemas = Vec::new();
            $(
                schemas.push(crate::export::write_json_schema::<$response>(
                    out_dir,
                    concat!(stringify!($variant), "Response"),
                )?);
            )*
            Ok(schemas)
        }

        #[allow(clippy::vec_init_then_push)]
        pub fn export_server_param_schemas(
            out_dir: &Path,
        ) -> ::anyhow::Result<Vec<GeneratedSchema>> {
            let mut schemas = Vec::new();
            $(
                schemas.push(crate::export::write_json_schema::<$params>(
                    out_dir,
                    concat!(stringify!($variant), "Params"),
                )?);
            )*
            Ok(schemas)
        }
    };
}

/// Generates `ServerNotification` enum and helpers, including a JSON Schema
/// exporter for each notification.
macro_rules! server_notification_definitions {
    (
        $(
            $(#[$variant_meta:meta])*
            $variant:ident $(=> $wire:literal)? ( $payload:ty )
        ),* $(,)?
    ) => {
        /// Notification sent from the server to the client.
        #[derive(
            Serialize,
            Deserialize,
            Debug,
            Clone,
            JsonSchema,
            TS,
            Display,
            ExperimentalApi,
        )]
        #[allow(clippy::large_enum_variant)]
        #[serde(tag = "method", content = "params", rename_all = "camelCase")]
        #[strum(serialize_all = "camelCase")]
        pub enum ServerNotification {
            $(
                $(#[$variant_meta])*
                $(#[serde(rename = $wire)] #[ts(rename = $wire)] #[strum(serialize = $wire)])?
                $variant($payload),
            )*
        }

        impl ServerNotification {
            pub fn to_params(self) -> Result<serde_json::Value, serde_json::Error> {
                match self {
                    $(Self::$variant(params) => serde_json::to_value(params),)*
                }
            }
        }

        impl TryFrom<JSONRPCNotification> for ServerNotification {
            type Error = serde_json::Error;

            fn try_from(value: JSONRPCNotification) -> Result<Self, serde_json::Error> {
                serde_json::from_value(serde_json::to_value(value)?)
            }
        }

        #[allow(clippy::vec_init_then_push)]
        pub fn export_server_notification_schemas(
            out_dir: &::std::path::Path,
        ) -> ::anyhow::Result<Vec<GeneratedSchema>> {
            let mut schemas = Vec::new();
            $(schemas.push(crate::export::write_json_schema::<$payload>(out_dir, stringify!($payload))?);)*
            Ok(schemas)
        }
    };
}
/// Notifications sent from the client to the server.
macro_rules! client_notification_definitions {
    (
        $(
            $(#[$variant_meta:meta])*
            $variant:ident $( ( $payload:ty ) )?
        ),* $(,)?
    ) => {
        #[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, TS, Display)]
        #[serde(tag = "method", content = "params", rename_all = "camelCase")]
        #[strum(serialize_all = "camelCase")]
        pub enum ClientNotification {
            $(
                $(#[$variant_meta])*
                $variant $( ( $payload ) )?,
            )*
        }

        pub fn export_client_notification_schemas(
            _out_dir: &::std::path::Path,
        ) -> ::anyhow::Result<Vec<GeneratedSchema>> {
            let schemas = Vec::new();
            $( $(schemas.push(crate::export::write_json_schema::<$payload>(_out_dir, stringify!($payload))?);)? )*
            Ok(schemas)
        }
    };
}

impl TryFrom<JSONRPCRequest> for ServerRequest {
    type Error = serde_json::Error;

    fn try_from(value: JSONRPCRequest) -> Result<Self, Self::Error> {
        serde_json::from_value(serde_json::to_value(value)?)
    }
}

server_request_definitions! {
    /// NEW APIs
    /// Sent when approval is requested for a specific command execution.
    /// This request is used for Turns started via turn/start.
    CommandExecutionRequestApproval => "item/commandExecution/requestApproval" {
        params: v2::CommandExecutionRequestApprovalParams,
        response: v2::CommandExecutionRequestApprovalResponse,
    },

    /// Sent when approval is requested for a specific file change.
    /// This request is used for Turns started via turn/start.
    FileChangeRequestApproval => "item/fileChange/requestApproval" {
        params: v2::FileChangeRequestApprovalParams,
        response: v2::FileChangeRequestApprovalResponse,
    },

    /// EXPERIMENTAL - Request input from the user for a tool call.
    ToolRequestUserInput => "item/tool/requestUserInput" {
        params: v2::ToolRequestUserInputParams,
        response: v2::ToolRequestUserInputResponse,
    },

    /// Request input for an MCP server elicitation.
    McpServerElicitationRequest => "mcpServer/elicitation/request" {
        params: v2::McpServerElicitationRequestParams,
        response: v2::McpServerElicitationRequestResponse,
    },

    /// Request approval for additional permissions from the user.
    PermissionsRequestApproval => "item/permissions/requestApproval" {
        params: v2::PermissionsRequestApprovalParams,
        response: v2::PermissionsRequestApprovalResponse,
    },

    /// Execute a dynamic tool call on the client.
    DynamicToolCall => "item/tool/call" {
        params: v2::DynamicToolCallParams,
        response: v2::DynamicToolCallResponse,
    },

    ChatgptAuthTokensRefresh => "account/chatgptAuthTokens/refresh" {
        params: v2::ChatgptAuthTokensRefreshParams,
        response: v2::ChatgptAuthTokensRefreshResponse,
    },

    /// DEPRECATED APIs below
    /// Request to approve a patch.
    /// This request is used for Turns started via the legacy APIs (i.e. SendUserTurn, SendUserMessage).
    ApplyPatchApproval {
        params: v1::ApplyPatchApprovalParams,
        response: v1::ApplyPatchApprovalResponse,
    },
    /// Request to exec a command.
    /// This request is used for Turns started via the legacy APIs (i.e. SendUserTurn, SendUserMessage).
    ExecCommandApproval {
        params: v1::ExecCommandApprovalParams,
        response: v1::ExecCommandApprovalResponse,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct FuzzyFileSearchParams {
    pub query: String,
    pub roots: Vec<String>,
    // if provided, will cancel any previous request that used the same value
    pub cancellation_token: Option<String>,
}

/// Superset of [`codex_file_search::FileMatch`]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS)]
pub struct FuzzyFileSearchResult {
    pub root: String,
    pub path: String,
    pub match_type: FuzzyFileSearchMatchType,
    pub file_name: String,
    pub score: u32,
    pub indices: Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub enum FuzzyFileSearchMatchType {
    File,
    Directory,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS)]
pub struct FuzzyFileSearchResponse {
    pub files: Vec<FuzzyFileSearchResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct FuzzyFileSearchSessionStartParams {
    pub session_id: String,
    pub roots: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS, Default)]
pub struct FuzzyFileSearchSessionStartResponse {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct FuzzyFileSearchSessionUpdateParams {
    pub session_id: String,
    pub query: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS, Default)]
pub struct FuzzyFileSearchSessionUpdateResponse {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct FuzzyFileSearchSessionStopParams {
    pub session_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS, Default)]
pub struct FuzzyFileSearchSessionStopResponse {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct FuzzyFileSearchSessionUpdatedNotification {
    pub session_id: String,
    pub query: String,
    pub files: Vec<FuzzyFileSearchResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct FuzzyFileSearchSessionCompletedNotification {
    pub session_id: String,
}

server_notification_definitions! {
    /// NEW NOTIFICATIONS
    Error => "error" (v2::ErrorNotification),
    ThreadStarted => "thread/started" (v2::ThreadStartedNotification),
    ThreadStatusChanged => "thread/status/changed" (v2::ThreadStatusChangedNotification),
    ThreadArchived => "thread/archived" (v2::ThreadArchivedNotification),
    ThreadUnarchived => "thread/unarchived" (v2::ThreadUnarchivedNotification),
    ThreadClosed => "thread/closed" (v2::ThreadClosedNotification),
    ThreadNameUpdated => "thread/name/updated" (v2::ThreadNameUpdatedNotification),
    ThreadTokenUsageUpdated => "thread/tokenUsage/updated" (v2::ThreadTokenUsageUpdatedNotification),
    TurnStarted => "turn/started" (v2::TurnStartedNotification),
    HookStarted => "hook/started" (v2::HookStartedNotification),
    TurnCompleted => "turn/completed" (v2::TurnCompletedNotification),
    HookCompleted => "hook/completed" (v2::HookCompletedNotification),
    TurnDiffUpdated => "turn/diff/updated" (v2::TurnDiffUpdatedNotification),
    TurnPlanUpdated => "turn/plan/updated" (v2::TurnPlanUpdatedNotification),
    ItemStarted => "item/started" (v2::ItemStartedNotification),
    ItemGuardianApprovalReviewStarted => "item/autoApprovalReview/started" (v2::ItemGuardianApprovalReviewStartedNotification),
    ItemGuardianApprovalReviewCompleted => "item/autoApprovalReview/completed" (v2::ItemGuardianApprovalReviewCompletedNotification),
    ItemCompleted => "item/completed" (v2::ItemCompletedNotification),
    /// This event is internal-only. Used by Codex Cloud.
    RawResponseItemCompleted => "rawResponseItem/completed" (v2::RawResponseItemCompletedNotification),
    AgentMessageDelta => "item/agentMessage/delta" (v2::AgentMessageDeltaNotification),
    /// EXPERIMENTAL - proposed plan streaming deltas for plan items.
    PlanDelta => "item/plan/delta" (v2::PlanDeltaNotification),
    /// Stream base64-encoded stdout/stderr chunks for a running `command/exec` session.
    CommandExecOutputDelta => "command/exec/outputDelta" (v2::CommandExecOutputDeltaNotification),
    CommandExecutionOutputDelta => "item/commandExecution/outputDelta" (v2::CommandExecutionOutputDeltaNotification),
    TerminalInteraction => "item/commandExecution/terminalInteraction" (v2::TerminalInteractionNotification),
    FileChangeOutputDelta => "item/fileChange/outputDelta" (v2::FileChangeOutputDeltaNotification),
    FileChangePatchUpdated => "item/fileChange/patchUpdated" (v2::FileChangePatchUpdatedNotification),
    ServerRequestResolved => "serverRequest/resolved" (v2::ServerRequestResolvedNotification),
    McpToolCallProgress => "item/mcpToolCall/progress" (v2::McpToolCallProgressNotification),
    McpServerOauthLoginCompleted => "mcpServer/oauthLogin/completed" (v2::McpServerOauthLoginCompletedNotification),
    McpServerStatusUpdated => "mcpServer/startupStatus/updated" (v2::McpServerStatusUpdatedNotification),
    AccountUpdated => "account/updated" (v2::AccountUpdatedNotification),
    AccountRateLimitsUpdated => "account/rateLimits/updated" (v2::AccountRateLimitsUpdatedNotification),
    FsChanged => "fs/changed" (v2::FsChangedNotification),
    ReasoningSummaryTextDelta => "item/reasoning/summaryTextDelta" (v2::ReasoningSummaryTextDeltaNotification),
    ReasoningSummaryPartAdded => "item/reasoning/summaryPartAdded" (v2::ReasoningSummaryPartAddedNotification),
    ReasoningTextDelta => "item/reasoning/textDelta" (v2::ReasoningTextDeltaNotification),
    /// Deprecated: Use `ContextCompaction` item type instead.
    ContextCompacted => "thread/compacted" (v2::ContextCompactedNotification),
    ModelRerouted => "model/rerouted" (v2::ModelReroutedNotification),
    Warning => "warning" (v2::WarningNotification),
    GuardianWarning => "guardianWarning" (v2::GuardianWarningNotification),
    DeprecationNotice => "deprecationNotice" (v2::DeprecationNoticeNotification),
    ConfigWarning => "configWarning" (v2::ConfigWarningNotification),
    #[experimental("thread/epiphany/stateUpdated")]
    ThreadEpiphanyStateUpdated => "thread/epiphany/stateUpdated" (v2::ThreadEpiphanyStateUpdatedNotification),
    #[experimental("thread/epiphany/jobsUpdated")]
    ThreadEpiphanyJobsUpdated => "thread/epiphany/jobsUpdated" (v2::ThreadEpiphanyJobsUpdatedNotification),
    FuzzyFileSearchSessionUpdated => "fuzzyFileSearch/sessionUpdated" (FuzzyFileSearchSessionUpdatedNotification),
    FuzzyFileSearchSessionCompleted => "fuzzyFileSearch/sessionCompleted" (FuzzyFileSearchSessionCompletedNotification),
    #[experimental("thread/realtime/started")]
    ThreadRealtimeStarted => "thread/realtime/started" (v2::ThreadRealtimeStartedNotification),
    #[experimental("thread/realtime/itemAdded")]
    ThreadRealtimeItemAdded => "thread/realtime/itemAdded" (v2::ThreadRealtimeItemAddedNotification),
    #[experimental("thread/realtime/transcript/delta")]
    ThreadRealtimeTranscriptDelta => "thread/realtime/transcript/delta" (v2::ThreadRealtimeTranscriptDeltaNotification),
    #[experimental("thread/realtime/transcript/done")]
    ThreadRealtimeTranscriptDone => "thread/realtime/transcript/done" (v2::ThreadRealtimeTranscriptDoneNotification),
    #[experimental("thread/realtime/outputAudio/delta")]
    ThreadRealtimeOutputAudioDelta => "thread/realtime/outputAudio/delta" (v2::ThreadRealtimeOutputAudioDeltaNotification),
    #[experimental("thread/realtime/sdp")]
    ThreadRealtimeSdp => "thread/realtime/sdp" (v2::ThreadRealtimeSdpNotification),
    #[experimental("thread/realtime/error")]
    ThreadRealtimeError => "thread/realtime/error" (v2::ThreadRealtimeErrorNotification),
    #[experimental("thread/realtime/closed")]
    ThreadRealtimeClosed => "thread/realtime/closed" (v2::ThreadRealtimeClosedNotification),

    /// Notifies the user of world-writable directories on Windows, which cannot be protected by the sandbox.
    WindowsWorldWritableWarning => "windows/worldWritableWarning" (v2::WindowsWorldWritableWarningNotification),
    WindowsSandboxSetupCompleted => "windowsSandbox/setupCompleted" (v2::WindowsSandboxSetupCompletedNotification),

    #[serde(rename = "account/login/completed")]
    #[ts(rename = "account/login/completed")]
    #[strum(serialize = "account/login/completed")]
    AccountLoginCompleted(v2::AccountLoginCompletedNotification),

}

client_notification_definitions! {
    Initialized,
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use codex_protocol::ThreadId;
    use codex_protocol::account::PlanType;
    use codex_protocol::parse_command::ParsedCommand;
    use codex_protocol::protocol::RealtimeConversationVersion;
    use codex_protocol::protocol::RealtimeOutputModality;
    use codex_protocol::protocol::RealtimeVoice;
    use codex_utils_absolute_path::AbsolutePathBuf;
    use codex_utils_absolute_path::test_support::PathBufExt;
    use codex_utils_absolute_path::test_support::test_path_buf;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use std::path::PathBuf;

    fn absolute_path_string(path: &str) -> String {
        let path = format!("/{}", path.trim_start_matches('/'));
        test_path_buf(&path).display().to_string()
    }

    fn absolute_path(path: &str) -> AbsolutePathBuf {
        let path = format!("/{}", path.trim_start_matches('/'));
        test_path_buf(&path).abs()
    }

    #[test]
    fn serialize_get_conversation_summary() -> Result<()> {
        let request = ClientRequest::GetConversationSummary {
            request_id: RequestId::Integer(42),
            params: v1::GetConversationSummaryParams::ThreadId {
                conversation_id: ThreadId::from_string("67e55044-10b1-426f-9247-bb680e5fe0c8")?,
            },
        };
        assert_eq!(
            json!({
                "method": "getConversationSummary",
                "id": 42,
                "params": {
                    "conversationId": "67e55044-10b1-426f-9247-bb680e5fe0c8"
                }
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_initialize_with_opt_out_notification_methods() -> Result<()> {
        let request = ClientRequest::Initialize {
            request_id: RequestId::Integer(42),
            params: v1::InitializeParams {
                client_info: v1::ClientInfo {
                    name: "codex_vscode".to_string(),
                    title: Some("Codex VS Code Extension".to_string()),
                    version: "0.1.0".to_string(),
                },
                capabilities: Some(v1::InitializeCapabilities {
                    experimental_api: true,
                    opt_out_notification_methods: Some(vec![
                        "thread/started".to_string(),
                        "item/agentMessage/delta".to_string(),
                    ]),
                }),
            },
        };

        assert_eq!(
            json!({
                "method": "initialize",
                "id": 42,
                "params": {
                    "clientInfo": {
                        "name": "codex_vscode",
                        "title": "Codex VS Code Extension",
                        "version": "0.1.0"
                    },
                    "capabilities": {
                        "experimentalApi": true,
                        "optOutNotificationMethods": [
                            "thread/started",
                            "item/agentMessage/delta"
                        ]
                    }
                }
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn deserialize_initialize_with_opt_out_notification_methods() -> Result<()> {
        let request: ClientRequest = serde_json::from_value(json!({
            "method": "initialize",
            "id": 42,
            "params": {
                "clientInfo": {
                    "name": "codex_vscode",
                    "title": "Codex VS Code Extension",
                    "version": "0.1.0"
                },
                "capabilities": {
                    "experimentalApi": true,
                    "optOutNotificationMethods": [
                        "thread/started",
                        "item/agentMessage/delta"
                    ]
                }
            }
        }))?;

        assert_eq!(
            request,
            ClientRequest::Initialize {
                request_id: RequestId::Integer(42),
                params: v1::InitializeParams {
                    client_info: v1::ClientInfo {
                        name: "codex_vscode".to_string(),
                        title: Some("Codex VS Code Extension".to_string()),
                        version: "0.1.0".to_string(),
                    },
                    capabilities: Some(v1::InitializeCapabilities {
                        experimental_api: true,
                        opt_out_notification_methods: Some(vec![
                            "thread/started".to_string(),
                            "item/agentMessage/delta".to_string(),
                        ]),
                    }),
                },
            }
        );
        Ok(())
    }

    #[test]
    fn conversation_id_serializes_as_plain_string() -> Result<()> {
        let id = ThreadId::from_string("67e55044-10b1-426f-9247-bb680e5fe0c8")?;

        assert_eq!(
            json!("67e55044-10b1-426f-9247-bb680e5fe0c8"),
            serde_json::to_value(id)?
        );
        Ok(())
    }

    #[test]
    fn conversation_id_deserializes_from_plain_string() -> Result<()> {
        let id: ThreadId = serde_json::from_value(json!("67e55044-10b1-426f-9247-bb680e5fe0c8"))?;

        assert_eq!(
            ThreadId::from_string("67e55044-10b1-426f-9247-bb680e5fe0c8")?,
            id,
        );
        Ok(())
    }

    #[test]
    fn serialize_client_notification() -> Result<()> {
        let notification = ClientNotification::Initialized;
        // Note there is no "params" field for this notification.
        assert_eq!(
            json!({
                "method": "initialized",
            }),
            serde_json::to_value(&notification)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_server_request() -> Result<()> {
        let conversation_id = ThreadId::from_string("67e55044-10b1-426f-9247-bb680e5fe0c8")?;
        let params = v1::ExecCommandApprovalParams {
            conversation_id,
            call_id: "call-42".to_string(),
            approval_id: Some("approval-42".to_string()),
            command: vec!["echo".to_string(), "hello".to_string()],
            cwd: PathBuf::from("/tmp"),
            reason: Some("because tests".to_string()),
            parsed_cmd: vec![ParsedCommand::Unknown {
                cmd: "echo hello".to_string(),
            }],
        };
        let request = ServerRequest::ExecCommandApproval {
            request_id: RequestId::Integer(7),
            params: params.clone(),
        };

        assert_eq!(
            json!({
                "method": "execCommandApproval",
                "id": 7,
                "params": {
                    "conversationId": "67e55044-10b1-426f-9247-bb680e5fe0c8",
                    "callId": "call-42",
                    "approvalId": "approval-42",
                    "command": ["echo", "hello"],
                    "cwd": "/tmp",
                    "reason": "because tests",
                    "parsedCmd": [
                        {
                            "type": "unknown",
                            "cmd": "echo hello"
                        }
                    ]
                }
            }),
            serde_json::to_value(&request)?,
        );

        let payload = ServerRequestPayload::ExecCommandApproval(params);
        assert_eq!(request.id(), &RequestId::Integer(7));
        assert_eq!(payload.request_with_id(RequestId::Integer(7)), request);
        Ok(())
    }

    #[test]
    fn serialize_chatgpt_auth_tokens_refresh_request() -> Result<()> {
        let request = ServerRequest::ChatgptAuthTokensRefresh {
            request_id: RequestId::Integer(8),
            params: v2::ChatgptAuthTokensRefreshParams {
                reason: v2::ChatgptAuthTokensRefreshReason::Unauthorized,
                previous_account_id: Some("org-123".to_string()),
            },
        };
        assert_eq!(
            json!({
                "method": "account/chatgptAuthTokens/refresh",
                "id": 8,
                "params": {
                    "reason": "unauthorized",
                    "previousAccountId": "org-123"
                }
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_server_response() -> Result<()> {
        let response = ServerResponse::CommandExecutionRequestApproval {
            request_id: RequestId::Integer(8),
            response: v2::CommandExecutionRequestApprovalResponse {
                decision: v2::CommandExecutionApprovalDecision::AcceptForSession,
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(8));
        assert_eq!(response.method(), "item/commandExecution/requestApproval");
        assert_eq!(
            json!({
                "method": "item/commandExecution/requestApproval",
                "id": 8,
                "response": {
                    "decision": "acceptForSession"
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_mcp_server_elicitation_request() -> Result<()> {
        let requested_schema: v2::McpElicitationSchema = serde_json::from_value(json!({
            "type": "object",
            "properties": {
                "confirmed": {
                    "type": "boolean"
                }
            },
            "required": ["confirmed"]
        }))?;
        let params = v2::McpServerElicitationRequestParams {
            thread_id: "thr_123".to_string(),
            turn_id: Some("turn_123".to_string()),
            server_name: "demo_mcp".to_string(),
            request: v2::McpServerElicitationRequest::Form {
                meta: None,
                message: "Allow this request?".to_string(),
                requested_schema,
            },
        };
        let request = ServerRequest::McpServerElicitationRequest {
            request_id: RequestId::Integer(9),
            params: params.clone(),
        };

        assert_eq!(
            json!({
                "method": "mcpServer/elicitation/request",
                "id": 9,
                "params": {
                    "threadId": "thr_123",
                    "turnId": "turn_123",
                    "serverName": "demo_mcp",
                    "mode": "form",
                    "_meta": null,
                    "message": "Allow this request?",
                    "requestedSchema": {
                        "type": "object",
                        "properties": {
                            "confirmed": {
                                "type": "boolean"
                            }
                        },
                        "required": ["confirmed"]
                    }
                }
            }),
            serde_json::to_value(&request)?,
        );

        let payload = ServerRequestPayload::McpServerElicitationRequest(params);
        assert_eq!(request.id(), &RequestId::Integer(9));
        assert_eq!(payload.request_with_id(RequestId::Integer(9)), request);
        Ok(())
    }

    #[test]
    fn serialize_get_account_rate_limits() -> Result<()> {
        let request = ClientRequest::GetAccountRateLimits {
            request_id: RequestId::Integer(1),
            params: None,
        };
        assert_eq!(request.id(), &RequestId::Integer(1));
        assert_eq!(request.method(), "account/rateLimits/read");
        assert_eq!(
            json!({
                "method": "account/rateLimits/read",
                "id": 1,
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_client_response() -> Result<()> {
        let cwd = absolute_path("/tmp");
        let response = ClientResponse::ThreadStart {
            request_id: RequestId::Integer(7),
            response: v2::ThreadStartResponse {
                thread: v2::Thread {
                    id: "67e55044-10b1-426f-9247-bb680e5fe0c8".to_string(),
                    forked_from_id: None,
                    preview: "first prompt".to_string(),
                    ephemeral: true,
                    model_provider: "openai".to_string(),
                    created_at: 1,
                    updated_at: 2,
                    status: v2::ThreadStatus::Idle,
                    path: None,
                    cwd: cwd.clone(),
                    cli_version: "0.0.0".to_string(),
                    source: v2::SessionSource::Exec,
                    agent_nickname: None,
                    agent_role: None,
                    git_info: None,
                    name: None,
                    epiphany_state: None,
                    turns: Vec::new(),
                },
                model: "gpt-5".to_string(),
                model_provider: "openai".to_string(),
                service_tier: None,
                cwd: cwd.clone(),
                instruction_sources: vec![absolute_path("/tmp/AGENTS.md")],
                approval_policy: v2::AskForApproval::OnFailure,
                approvals_reviewer: v2::ApprovalsReviewer::User,
                sandbox: v2::SandboxPolicy::DangerFullAccess,
                permission_profile: Some(
                    codex_protocol::models::PermissionProfile::from_legacy_sandbox_policy(
                        &codex_protocol::protocol::SandboxPolicy::DangerFullAccess,
                        cwd.as_path(),
                    )
                    .into(),
                ),
                reasoning_effort: None,
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(7));
        assert_eq!(response.method(), "thread/start");
        assert_eq!(
            json!({
                "method": "thread/start",
                "id": 7,
                "response": {
                    "thread": {
                        "id": "67e55044-10b1-426f-9247-bb680e5fe0c8",
                        "forkedFromId": null,
                        "preview": "first prompt",
                        "ephemeral": true,
                        "modelProvider": "openai",
                        "createdAt": 1,
                        "updatedAt": 2,
                        "status": {
                            "type": "idle"
                        },
                        "path": null,
                        "cwd": absolute_path_string("tmp"),
                        "cliVersion": "0.0.0",
                        "source": "exec",
                        "agentNickname": null,
                        "agentRole": null,
                        "gitInfo": null,
                        "name": null,
                        "turns": []
                    },
                    "model": "gpt-5",
                    "modelProvider": "openai",
                    "serviceTier": null,
                    "cwd": absolute_path_string("tmp"),
                    "instructionSources": [absolute_path_string("tmp/AGENTS.md")],
                    "approvalPolicy": "on-failure",
                    "approvalsReviewer": "user",
                    "sandbox": {
                        "type": "dangerFullAccess"
                    },
                    "permissionProfile": {
                        "network": {
                            "enabled": true,
                        },
                        "fileSystem": {
                            "entries": [
                                {
                                    "path": {
                                        "type": "special",
                                        "value": {
                                            "kind": "root",
                                        },
                                    },
                                    "access": "write",
                                },
                            ],
                        },
                    },
                    "reasoningEffort": null
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_view_response() -> Result<()> {
        let request = ClientRequest::ThreadEpiphanyView {
            request_id: RequestId::Integer(8),
            params: v2::ThreadEpiphanyViewParams {
                thread_id: "thr_123".to_string(),
                lenses: vec![v2::ThreadEpiphanyViewLens::Pressure],
            },
        };
        assert_eq!(request.method(), "thread/epiphany/view");

        let response = ClientResponse::ThreadEpiphanyView {
            request_id: RequestId::Integer(8),
            response: v2::ThreadEpiphanyViewResponse {
                thread_id: "thr_123".to_string(),
                lenses: vec![v2::ThreadEpiphanyViewLens::Pressure],
                scene: None,
                jobs: Vec::new(),
                roles: None,
                planning: None,
                pressure: Some(v2::ThreadEpiphanyPressure {
                    status: v2::ThreadEpiphanyPressureStatus::Ready,
                    level: v2::ThreadEpiphanyPressureLevel::Low,
                    basis: v2::ThreadEpiphanyPressureBasis::ModelContextWindow,
                    used_tokens: Some(10),
                    model_context_window: Some(100),
                    model_auto_compact_token_limit: Some(100),
                    remaining_tokens: Some(90),
                    ratio_per_mille: Some(100),
                    should_prepare_compaction: false,
                    note: "Context window usage is below the compaction prep threshold."
                        .to_string(),
                }),
                reorient: None,
                crrc: None,
                coordinator: None,
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(8));
        assert_eq!(response.method(), "thread/epiphany/view");
        assert_eq!(
            json!({
                "method": "thread/epiphany/view",
                "id": 8,
                "response": {
                    "threadId": "thr_123",
                    "lenses": ["pressure"],
                    "pressure": {
                        "status": "ready",
                        "level": "low",
                        "basis": "modelContextWindow",
                        "usedTokens": 10,
                        "modelContextWindow": 100,
                        "modelAutoCompactTokenLimit": 100,
                        "remainingTokens": 90,
                        "ratioPerMille": 100,
                        "shouldPrepareCompaction": false,
                        "note": "Context window usage is below the compaction prep threshold."
                    }
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_role_result_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyRoleResult {
            request_id: RequestId::Integer(9),
            response: v2::ThreadEpiphanyRoleResultResponse {
                thread_id: "thr_123".to_string(),
                role_id: v2::ThreadEpiphanyRoleId::Verification,
                source: v2::ThreadEpiphanyRolesSource::Live,
                state_status: v2::ThreadEpiphanyReorientStateStatus::Ready,
                state_revision: Some(4),
                binding_id: "verification-review-worker".to_string(),
                status: v2::ThreadEpiphanyRoleResultStatus::Completed,
                job: Some(v2::ThreadEpiphanyJob {
                    id: "verification-review-worker".to_string(),
                    kind: v2::ThreadEpiphanyJobKind::Specialist,
                    scope: "role-scoped verification/review".to_string(),
                    owner_role: "epiphany-verifier".to_string(),
                    launcher_job_id: Some("epiphany-launch-2".to_string()),
                    authority_scope: Some("epiphany.role.verification".to_string()),
                    backend_job_id: Some("backend-2".to_string()),
                    status: v2::ThreadEpiphanyJobStatus::Completed,
                    items_processed: Some(1),
                    items_total: Some(1),
                    progress_note: Some("1/1 items completed.".to_string()),
                    last_checkpoint_at_unix_seconds: None,
                    blocking_reason: None,
                    active_thread_ids: Vec::new(),
                    linked_subgoal_ids: vec!["phase-6".to_string()],
                    linked_graph_node_ids: vec!["state-spine".to_string()],
                }),
                finding: Some(v2::ThreadEpiphanyRoleFinding {
                    role_id: v2::ThreadEpiphanyRoleId::Verification,
                    verdict: Some("pass".to_string()),
                    summary: Some("Evidence covers the bounded change.".to_string()),
                    next_safe_move: Some("Promote the verified patch.".to_string()),
                    checkpoint_summary: None,
                    scratch_summary: None,
                    files_inspected: vec!["src/lib.rs".to_string()],
                    frontier_node_ids: vec!["state-spine".to_string()],
                    evidence_ids: vec!["ev-1".to_string()],
                    artifact_refs: Vec::new(),
                    runtime_result_id: Some("result-verification-1".to_string()),
                    runtime_job_id: Some("backend-2".to_string()),
                    open_questions: Vec::new(),
                    evidence_gaps: Vec::new(),
                    risks: Vec::new(),
                    state_patch: None,
                    self_patch: None,
                    self_persistence: None,
                    job_error: None,
                    item_error: None,
                }),
                note: "Verification role specialist completed. Next safe move: Promote the verified patch."
                    .to_string(),
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(9));
        assert_eq!(response.method(), "thread/epiphany/roleResult");
        assert_eq!(
            json!({
                "method": "thread/epiphany/roleResult",
                "id": 9,
                "response": {
                    "threadId": "thr_123",
                    "roleId": "verification",
                    "source": "live",
                    "stateStatus": "ready",
                    "stateRevision": 4,
                    "bindingId": "verification-review-worker",
                    "status": "completed",
                    "job": {
                        "id": "verification-review-worker",
                        "kind": "specialist",
                        "scope": "role-scoped verification/review",
                        "ownerRole": "epiphany-verifier",
                        "launcherJobId": "epiphany-launch-2",
                        "authorityScope": "epiphany.role.verification",
                        "backendJobId": "backend-2",
                        "status": "completed",
                        "itemsProcessed": 1,
                        "itemsTotal": 1,
                        "progressNote": "1/1 items completed.",
                        "linkedSubgoalIds": ["phase-6"],
                        "linkedGraphNodeIds": ["state-spine"]
                    },
                    "finding": {
                        "roleId": "verification",
                        "verdict": "pass",
                        "summary": "Evidence covers the bounded change.",
                        "nextSafeMove": "Promote the verified patch.",
                        "filesInspected": ["src/lib.rs"],
                        "frontierNodeIds": ["state-spine"],
                        "evidenceIds": ["ev-1"],
                        "runtimeResultId": "result-verification-1",
                        "runtimeJobId": "backend-2"
                    },
                    "note": "Verification role specialist completed. Next safe move: Promote the verified patch."
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_freshness_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyFreshness {
            request_id: RequestId::Integer(8),
            response: v2::ThreadEpiphanyFreshnessResponse {
                thread_id: "thr_123".to_string(),
                source: v2::ThreadEpiphanyFreshnessSource::Live,
                state_revision: Some(3),
                retrieval: v2::ThreadEpiphanyRetrievalFreshness {
                    status: v2::ThreadEpiphanyRetrievalFreshnessStatus::Stale,
                    semantic_available: Some(true),
                    last_indexed_at_unix_seconds: Some(1_744_500_000),
                    indexed_file_count: Some(12),
                    indexed_chunk_count: Some(48),
                    dirty_paths: vec![PathBuf::from("src/router.rs")],
                    note: "Retrieval catalog is stale; 1 dirty path(s) need refresh."
                        .to_string(),
                },
                graph: v2::ThreadEpiphanyGraphFreshness {
                    status: v2::ThreadEpiphanyGraphFreshnessStatus::Stale,
                    graph_freshness: Some("stale".to_string()),
                    checkpoint_id: Some("ck-5".to_string()),
                    dirty_path_count: 1,
                    dirty_paths: vec![PathBuf::from("src/router.rs")],
                    open_question_count: 1,
                    open_gap_count: 0,
                    note: "Graph freshness is stale; frontier has 1 dirty path(s), 1 open question id(s), and 0 open gap id(s)."
                        .to_string(),
                },
                watcher: v2::ThreadEpiphanyInvalidationInput {
                    status: v2::ThreadEpiphanyInvalidationStatus::Changed,
                    watched_root: Some(PathBuf::from("/repo")),
                    observed_at_unix_seconds: Some(1_744_500_123),
                    changed_path_count: 1,
                    changed_paths: vec![PathBuf::from("src/router.rs")],
                    graph_node_ids: vec!["router-flow".to_string()],
                    active_frontier_node_ids: vec!["router-flow".to_string()],
                    note: "Watcher observed 1 recent changed path(s) touching 1 mapped graph node(s), including 1 active frontier node(s)."
                        .to_string(),
                },
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(8));
        assert_eq!(response.method(), "thread/epiphany/freshness");
        assert_eq!(
            json!({
                "method": "thread/epiphany/freshness",
                "id": 8,
                "response": {
                    "threadId": "thr_123",
                    "source": "live",
                    "stateRevision": 3,
                    "retrieval": {
                        "status": "stale",
                        "semanticAvailable": true,
                        "lastIndexedAtUnixSeconds": 1744500000,
                        "indexedFileCount": 12,
                        "indexedChunkCount": 48,
                        "dirtyPaths": ["src/router.rs"],
                        "note": "Retrieval catalog is stale; 1 dirty path(s) need refresh."
                    },
                    "graph": {
                        "status": "stale",
                        "graphFreshness": "stale",
                        "checkpointId": "ck-5",
                        "dirtyPathCount": 1,
                        "dirtyPaths": ["src/router.rs"],
                        "openQuestionCount": 1,
                        "openGapCount": 0,
                        "note": "Graph freshness is stale; frontier has 1 dirty path(s), 1 open question id(s), and 0 open gap id(s)."
                    },
                    "watcher": {
                        "status": "changed",
                        "watchedRoot": "/repo",
                        "observedAtUnixSeconds": 1744500123,
                        "changedPathCount": 1,
                        "changedPaths": ["src/router.rs"],
                        "graphNodeIds": ["router-flow"],
                        "activeFrontierNodeIds": ["router-flow"],
                        "note": "Watcher observed 1 recent changed path(s) touching 1 mapped graph node(s), including 1 active frontier node(s)."
                    }
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_context_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyContext {
            request_id: RequestId::Integer(8),
            response: v2::ThreadEpiphanyContextResponse {
                thread_id: "thr_123".to_string(),
                source: v2::ThreadEpiphanyContextSource::Live,
                state_status: v2::ThreadEpiphanyContextStateStatus::Ready,
                state_revision: Some(3),
                context: v2::ThreadEpiphanyContext {
                    graph: v2::ThreadEpiphanyGraphContext {
                        architecture_nodes: vec![epiphany_state_model::EpiphanyGraphNode {
                            id: "state-spine".to_string(),
                            title: "State spine".to_string(),
                            purpose: "Keep understanding typed".to_string(),
                            ..Default::default()
                        }],
                        architecture_edges: vec![epiphany_state_model::EpiphanyGraphEdge {
                            id: Some("edge-state-read".to_string()),
                            source_id: "state-spine".to_string(),
                            target_id: "client-read".to_string(),
                            kind: "projects".to_string(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                    frontier: Some(epiphany_state_model::EpiphanyGraphFrontier {
                        active_node_ids: vec!["state-spine".to_string()],
                        ..Default::default()
                    }),
                    checkpoint: None,
                    investigation_checkpoint: Some(
                        epiphany_state_model::EpiphanyInvestigationCheckpoint {
                            checkpoint_id: "ix-1".to_string(),
                            kind: "source_gathering".to_string(),
                            focus: "Audit the compaction seam.".to_string(),
                            next_action: Some(
                                "Re-gather source if this packet is stale.".to_string(),
                            ),
                            ..Default::default()
                        },
                    ),
                    observations: vec![epiphany_state_model::EpiphanyObservation {
                        id: "obs-1".to_string(),
                        summary: "Context shard is read-only.".to_string(),
                        source_kind: "smoke".to_string(),
                        status: "ok".to_string(),
                        evidence_ids: vec!["ev-1".to_string()],
                        ..Default::default()
                    }],
                    evidence: vec![epiphany_state_model::EpiphanyEvidenceRecord {
                        id: "ev-1".to_string(),
                        kind: "test".to_string(),
                        status: "ok".to_string(),
                        summary: "Context shard serialized.".to_string(),
                        ..Default::default()
                    }],
                },
                missing: v2::ThreadEpiphanyContextMissing {
                    graph_node_ids: vec!["missing-node".to_string()],
                    ..Default::default()
                },
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(8));
        assert_eq!(response.method(), "thread/epiphany/context");
        assert_eq!(
            json!({
                "method": "thread/epiphany/context",
                "id": 8,
                "response": {
                    "threadId": "thr_123",
                    "source": "live",
                    "stateStatus": "ready",
                    "stateRevision": 3,
                    "context": {
                        "graph": {
                            "architectureNodes": [
                                {
                                    "id": "state-spine",
                                    "title": "State spine",
                                    "purpose": "Keep understanding typed"
                                }
                            ],
                            "architectureEdges": [
                                {
                                    "source_id": "state-spine",
                                    "target_id": "client-read",
                                    "kind": "projects",
                                    "id": "edge-state-read"
                                }
                            ]
                        },
                        "frontier": {
                            "active_node_ids": ["state-spine"]
                        },
                        "investigationCheckpoint": {
                            "checkpoint_id": "ix-1",
                            "kind": "source_gathering",
                            "disposition": "resume_ready",
                            "focus": "Audit the compaction seam.",
                            "next_action": "Re-gather source if this packet is stale."
                        },
                        "observations": [
                            {
                                "id": "obs-1",
                                "summary": "Context shard is read-only.",
                                "source_kind": "smoke",
                                "status": "ok",
                                "evidence_ids": ["ev-1"]
                            }
                        ],
                        "evidence": [
                            {
                                "id": "ev-1",
                                "kind": "test",
                                "status": "ok",
                                "summary": "Context shard serialized."
                            }
                        ]
                    },
                    "missing": {
                        "graphNodeIds": ["missing-node"]
                    }
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_graph_query_request_and_response() -> Result<()> {
        let request = ClientRequest::ThreadEpiphanyGraphQuery {
            request_id: RequestId::Integer(24),
            params: v2::ThreadEpiphanyGraphQueryParams {
                thread_id: "thr_123".to_string(),
                query: v2::ThreadEpiphanyGraphQuery {
                    kind: v2::ThreadEpiphanyGraphQueryKind::FrontierNeighborhood,
                    node_ids: vec!["state-spine".to_string()],
                    edge_ids: Vec::new(),
                    paths: vec![PathBuf::from("src/lib.rs")],
                    symbols: Vec::new(),
                    edge_kinds: vec!["calls".to_string()],
                    direction: Some(v2::ThreadEpiphanyGraphQueryDirection::Both),
                    depth: Some(1),
                    include_links: Some(true),
                },
            },
        };

        assert_eq!(request.method(), "thread/epiphany/graphQuery");
        assert_eq!(
            json!({
                "method": "thread/epiphany/graphQuery",
                "id": 24,
                "params": {
                    "threadId": "thr_123",
                    "query": {
                        "kind": "frontierNeighborhood",
                        "nodeIds": ["state-spine"],
                        "paths": ["src/lib.rs"],
                        "edgeKinds": ["calls"],
                        "direction": "both",
                        "depth": 1,
                        "includeLinks": true
                    }
                }
            }),
            serde_json::to_value(&request)?,
        );

        let response = ClientResponse::ThreadEpiphanyGraphQuery {
            request_id: RequestId::Integer(24),
            response: v2::ThreadEpiphanyGraphQueryResponse {
                thread_id: "thr_123".to_string(),
                source: v2::ThreadEpiphanyContextSource::Live,
                state_status: v2::ThreadEpiphanyContextStateStatus::Ready,
                state_revision: Some(2),
                graph: v2::ThreadEpiphanyGraphContext {
                    architecture_nodes: vec![epiphany_state_model::EpiphanyGraphNode {
                        id: "state-spine".to_string(),
                        title: "State spine".to_string(),
                        purpose: "Persist Epiphany state.".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                frontier: None,
                checkpoint: None,
                matched: v2::ThreadEpiphanyGraphQueryMatched {
                    node_ids: vec!["state-spine".to_string()],
                    edge_ids: Vec::new(),
                    paths: Vec::new(),
                    symbols: Vec::new(),
                    edge_kinds: Vec::new(),
                },
                missing: v2::ThreadEpiphanyGraphQueryMissing::default(),
            },
        };

        assert_eq!(response.method(), "thread/epiphany/graphQuery");
        assert_eq!(
            json!({
                "method": "thread/epiphany/graphQuery",
                "id": 24,
                "response": {
                    "threadId": "thr_123",
                    "source": "live",
                    "stateStatus": "ready",
                    "stateRevision": 2,
                    "graph": {
                        "architectureNodes": [{
                            "id": "state-spine",
                            "title": "State spine",
                            "purpose": "Persist Epiphany state."
                        }]
                    },
                    "matched": {
                        "nodeIds": ["state-spine"]
                    },
                    "missing": {}
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_reorient_result_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyReorientResult {
            request_id: RequestId::Integer(9),
            response: v2::ThreadEpiphanyReorientResultResponse {
                thread_id: "thr_123".to_string(),
                source: v2::ThreadEpiphanyReorientSource::Live,
                state_status: v2::ThreadEpiphanyReorientStateStatus::Ready,
                state_revision: Some(5),
                binding_id: "reorient-worker".to_string(),
                status: v2::ThreadEpiphanyReorientResultStatus::Completed,
                job: Some(v2::ThreadEpiphanyJob {
                    id: "reorient-worker".to_string(),
                    kind: v2::ThreadEpiphanyJobKind::Specialist,
                    scope: "reorient-guided checkpoint resume".to_string(),
                    owner_role: "epiphany-reorient".to_string(),
                    launcher_job_id: Some("epiphany-launch-1".to_string()),
                    authority_scope: Some("epiphany.reorient.resume".to_string()),
                    backend_job_id: Some("backend-1".to_string()),
                    status: v2::ThreadEpiphanyJobStatus::Completed,
                    items_processed: Some(1),
                    items_total: Some(1),
                    progress_note: Some("1/1 items completed.".to_string()),
                    last_checkpoint_at_unix_seconds: Some(1_744_700_000),
                    blocking_reason: None,
                    active_thread_ids: Vec::new(),
                    linked_subgoal_ids: vec!["phase-6".to_string()],
                    linked_graph_node_ids: vec!["state-spine".to_string()],
                }),
                finding: Some(v2::ThreadEpiphanyReorientFinding {
                    mode: Some("resume".to_string()),
                    summary: Some("Checkpoint remains source-grounded.".to_string()),
                    next_safe_move: Some("Continue the read-back slice.".to_string()),
                    checkpoint_still_valid: Some(true),
                    files_inspected: vec!["src/lib.rs".to_string()],
                    frontier_node_ids: vec!["state-spine".to_string()],
                    evidence_ids: vec!["ev-1".to_string()],
                    artifact_refs: Vec::new(),
                    runtime_result_id: Some("result-reorient-1".to_string()),
                    runtime_job_id: Some("backend-1".to_string()),
                    job_error: None,
                    item_error: None,
                }),
                note:
                    "Reorientation worker completed. Next safe move: Continue the read-back slice."
                        .to_string(),
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(9));
        assert_eq!(response.method(), "thread/epiphany/reorientResult");
        assert_eq!(
            json!({
                "method": "thread/epiphany/reorientResult",
                "id": 9,
                "response": {
                    "threadId": "thr_123",
                    "source": "live",
                    "stateStatus": "ready",
                    "stateRevision": 5,
                    "bindingId": "reorient-worker",
                    "status": "completed",
                    "job": {
                        "id": "reorient-worker",
                        "kind": "specialist",
                        "scope": "reorient-guided checkpoint resume",
                        "ownerRole": "epiphany-reorient",
                        "launcherJobId": "epiphany-launch-1",
                        "authorityScope": "epiphany.reorient.resume",
                        "backendJobId": "backend-1",
                        "status": "completed",
                        "itemsProcessed": 1,
                        "itemsTotal": 1,
                        "progressNote": "1/1 items completed.",
                        "lastCheckpointAtUnixSeconds": 1744700000,
                        "linkedSubgoalIds": ["phase-6"],
                        "linkedGraphNodeIds": ["state-spine"]
                    },
                    "finding": {
                        "mode": "resume",
                        "summary": "Checkpoint remains source-grounded.",
                        "nextSafeMove": "Continue the read-back slice.",
                        "checkpointStillValid": true,
                        "filesInspected": ["src/lib.rs"],
                        "frontierNodeIds": ["state-spine"],
                        "evidenceIds": ["ev-1"],
                        "runtimeResultId": "result-reorient-1",
                        "runtimeJobId": "backend-1"
                    },
                    "note": "Reorientation worker completed. Next safe move: Continue the read-back slice."
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_job_interrupt_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyJobInterrupt {
            request_id: RequestId::Integer(13),
            response: v2::ThreadEpiphanyJobInterruptResponse {
                cancel_requested: true,
                interrupted_thread_ids: vec!["worker-thread-1".to_string()],
                revision: 4,
                changed_fields: vec![v2::ThreadEpiphanyStateUpdatedField::JobBindings],
                epiphany_state: epiphany_state_model::EpiphanyThreadState {
                    revision: 4,
                    job_bindings: vec![epiphany_state_model::EpiphanyJobBinding {
                        id: "specialist-work".to_string(),
                        kind: epiphany_state_model::EpiphanyJobKind::Specialist,
                        scope: "role-scoped specialist work".to_string(),
                        owner_role: "epiphany-harness".to_string(),
                        authority_scope: Some("epiphany.specialist".to_string()),
                        linked_subgoal_ids: Vec::new(),
                        linked_graph_node_ids: Vec::new(),
                        blocking_reason: Some(
                            "Launch explicitly to resume specialist work.".to_string(),
                        ),
                    }],
                    ..Default::default()
                },
                job: v2::ThreadEpiphanyJob {
                    id: "specialist-work".to_string(),
                    kind: v2::ThreadEpiphanyJobKind::Specialist,
                    scope: "role-scoped specialist work".to_string(),
                    owner_role: "epiphany-harness".to_string(),
                    launcher_job_id: None,
                    authority_scope: Some("epiphany.specialist".to_string()),
                    backend_job_id: None,
                    status: v2::ThreadEpiphanyJobStatus::Blocked,
                    items_processed: None,
                    items_total: None,
                    progress_note: None,
                    last_checkpoint_at_unix_seconds: None,
                    blocking_reason: Some(
                        "Launch explicitly to resume specialist work.".to_string(),
                    ),
                    active_thread_ids: Vec::new(),
                    linked_subgoal_ids: Vec::new(),
                    linked_graph_node_ids: Vec::new(),
                },
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(13));
        assert_eq!(response.method(), "thread/epiphany/jobInterrupt");
        assert_eq!(
            json!({
                "method": "thread/epiphany/jobInterrupt",
                "id": 13,
                "response": {
                    "cancelRequested": true,
                    "interruptedThreadIds": ["worker-thread-1"],
                    "revision": 4,
                    "changedFields": ["jobBindings"],
                    "epiphanyState": {
                        "revision": 4,
                        "job_bindings": [{
                            "id": "specialist-work",
                            "kind": "specialist",
                            "scope": "role-scoped specialist work",
                            "owner_role": "epiphany-harness",
                            "authority_scope": "epiphany.specialist",
                            "blocking_reason": "Launch explicitly to resume specialist work."
                        }]
                    },
                    "job": {
                        "id": "specialist-work",
                        "kind": "specialist",
                        "scope": "role-scoped specialist work",
                        "ownerRole": "epiphany-harness",
                        "authorityScope": "epiphany.specialist",
                        "status": "blocked",
                        "blockingReason": "Launch explicitly to resume specialist work."
                    }
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_update_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyUpdate {
            request_id: RequestId::Integer(12),
            response: v2::ThreadEpiphanyUpdateResponse {
                revision: 2,
                changed_fields: vec![v2::ThreadEpiphanyStateUpdatedField::Objective],
                epiphany_state: epiphany_state_model::EpiphanyThreadState {
                    revision: 2,
                    objective: Some("Keep the map honest".to_string()),
                    ..Default::default()
                },
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(12));
        assert_eq!(response.method(), "thread/epiphany/update");
        assert_eq!(
            json!({
                "method": "thread/epiphany/update",
                "id": 12,
                "response": {
                    "revision": 2,
                    "changedFields": ["objective"],
                    "epiphanyState": {
                        "revision": 2,
                        "objective": "Keep the map honest"
                    }
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_state_updated_notification() -> Result<()> {
        let notification = ServerNotification::ThreadEpiphanyStateUpdated(
            v2::ThreadEpiphanyStateUpdatedNotification {
                thread_id: "thr_123".to_string(),
                source: v2::ThreadEpiphanyStateUpdatedSource::Update,
                revision: 7,
                changed_fields: vec![
                    v2::ThreadEpiphanyStateUpdatedField::Objective,
                    v2::ThreadEpiphanyStateUpdatedField::Evidence,
                ],
                epiphany_state: epiphany_state_model::EpiphanyThreadState {
                    revision: 7,
                    objective: Some("Keep the map live".to_string()),
                    ..Default::default()
                },
            },
        );
        assert_eq!(
            json!({
                "method": "thread/epiphany/stateUpdated",
                "params": {
                    "threadId": "thr_123",
                    "source": "update",
                    "revision": 7,
                    "changedFields": ["objective", "evidence"],
                    "epiphanyState": {
                        "revision": 7,
                        "objective": "Keep the map live"
                    }
                }
            }),
            serde_json::to_value(&notification)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_state_updated_job_launch_notification() -> Result<()> {
        let notification = ServerNotification::ThreadEpiphanyStateUpdated(
            v2::ThreadEpiphanyStateUpdatedNotification {
                thread_id: "thr_123".to_string(),
                source: v2::ThreadEpiphanyStateUpdatedSource::JobLaunch,
                revision: 8,
                changed_fields: vec![v2::ThreadEpiphanyStateUpdatedField::JobBindings],
                epiphany_state: epiphany_state_model::EpiphanyThreadState {
                    revision: 8,
                    ..Default::default()
                },
            },
        );
        assert_eq!(
            json!({
                "method": "thread/epiphany/stateUpdated",
                "params": {
                    "threadId": "thr_123",
                    "source": "jobLaunch",
                    "revision": 8,
                    "changedFields": ["jobBindings"],
                    "epiphanyState": {
                        "revision": 8
                    }
                }
            }),
            serde_json::to_value(&notification)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_jobs_updated_notification() -> Result<()> {
        let notification = ServerNotification::ThreadEpiphanyJobsUpdated(
            v2::ThreadEpiphanyJobsUpdatedNotification {
                thread_id: "thr_123".to_string(),
                source: v2::ThreadEpiphanyJobsUpdatedSource::RuntimeProgress,
                state_revision: Some(7),
                jobs: vec![v2::ThreadEpiphanyJob {
                    id: "specialist-work".to_string(),
                    kind: v2::ThreadEpiphanyJobKind::Specialist,
                    scope: "runtime-bound specialist work".to_string(),
                    owner_role: "epiphany-harness".to_string(),
                    launcher_job_id: Some("launcher-specialist".to_string()),
                    authority_scope: Some("epiphany.specialist".to_string()),
                    backend_job_id: Some("job-123".to_string()),
                    status: v2::ThreadEpiphanyJobStatus::Running,
                    items_processed: Some(1),
                    items_total: Some(3),
                    progress_note: Some("Runtime agent job is running.".to_string()),
                    last_checkpoint_at_unix_seconds: Some(1_744_500_123),
                    blocking_reason: None,
                    active_thread_ids: vec!["worker-thread".to_string()],
                    linked_subgoal_ids: vec!["phase6".to_string()],
                    linked_graph_node_ids: vec!["job-surface".to_string()],
                }],
            },
        );
        assert_eq!(
            json!({
                "method": "thread/epiphany/jobsUpdated",
                "params": {
                    "threadId": "thr_123",
                    "source": "runtimeProgress",
                    "stateRevision": 7,
                    "jobs": [{
                        "id": "specialist-work",
                        "kind": "specialist",
                        "scope": "runtime-bound specialist work",
                        "ownerRole": "epiphany-harness",
                        "launcherJobId": "launcher-specialist",
                        "authorityScope": "epiphany.specialist",
                        "backendJobId": "job-123",
                        "status": "running",
                        "itemsProcessed": 1,
                        "itemsTotal": 3,
                        "progressNote": "Runtime agent job is running.",
                        "lastCheckpointAtUnixSeconds": 1744500123,
                        "activeThreadIds": ["worker-thread"],
                        "linkedSubgoalIds": ["phase6"],
                        "linkedGraphNodeIds": ["job-surface"]
                    }]
                }
            }),
            serde_json::to_value(&notification)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_config_requirements_read() -> Result<()> {
        let request = ClientRequest::ConfigRequirementsRead {
            request_id: RequestId::Integer(1),
            params: None,
        };
        assert_eq!(
            json!({
                "method": "configRequirements/read",
                "id": 1,
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_account_login_api_key() -> Result<()> {
        let request = ClientRequest::LoginAccount {
            request_id: RequestId::Integer(2),
            params: v2::LoginAccountParams::ApiKey {
                api_key: "secret".to_string(),
            },
        };
        assert_eq!(
            json!({
                "method": "account/login/start",
                "id": 2,
                "params": {
                    "type": "apiKey",
                    "apiKey": "secret"
                }
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_account_login_chatgpt() -> Result<()> {
        let request = ClientRequest::LoginAccount {
            request_id: RequestId::Integer(3),
            params: v2::LoginAccountParams::Chatgpt,
        };
        assert_eq!(
            json!({
                "method": "account/login/start",
                "id": 3,
                "params": {
                    "type": "chatgpt"
                }
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_account_login_chatgpt_device_code() -> Result<()> {
        let request = ClientRequest::LoginAccount {
            request_id: RequestId::Integer(4),
            params: v2::LoginAccountParams::ChatgptDeviceCode,
        };
        assert_eq!(
            json!({
                "method": "account/login/start",
                "id": 4,
                "params": {
                    "type": "chatgptDeviceCode"
                }
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_account_logout() -> Result<()> {
        let request = ClientRequest::LogoutAccount {
            request_id: RequestId::Integer(5),
            params: None,
        };
        assert_eq!(
            json!({
                "method": "account/logout",
                "id": 5,
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_account_login_chatgpt_auth_tokens() -> Result<()> {
        let request = ClientRequest::LoginAccount {
            request_id: RequestId::Integer(6),
            params: v2::LoginAccountParams::ChatgptAuthTokens {
                access_token: "access-token".to_string(),
                chatgpt_account_id: "org-123".to_string(),
                chatgpt_plan_type: Some("business".to_string()),
            },
        };
        assert_eq!(
            json!({
                "method": "account/login/start",
                "id": 6,
                "params": {
                    "type": "chatgptAuthTokens",
                    "accessToken": "access-token",
                    "chatgptAccountId": "org-123",
                    "chatgptPlanType": "business"
                }
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_get_account() -> Result<()> {
        let request = ClientRequest::GetAccount {
            request_id: RequestId::Integer(6),
            params: v2::GetAccountParams {
                refresh_token: false,
            },
        };
        assert_eq!(
            json!({
                "method": "account/read",
                "id": 6,
                "params": {
                    "refreshToken": false
                }
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn account_serializes_fields_in_camel_case() -> Result<()> {
        let api_key = v2::Account::ApiKey {};
        assert_eq!(
            json!({
                "type": "apiKey",
            }),
            serde_json::to_value(&api_key)?,
        );

        let chatgpt = v2::Account::Chatgpt {
            email: "user@example.com".to_string(),
            plan_type: PlanType::Plus,
        };
        assert_eq!(
            json!({
                "type": "chatgpt",
                "email": "user@example.com",
                "planType": "plus",
            }),
            serde_json::to_value(&chatgpt)?,
        );

        Ok(())
    }

    #[test]
    fn serialize_list_models() -> Result<()> {
        let request = ClientRequest::ModelList {
            request_id: RequestId::Integer(6),
            params: v2::ModelListParams::default(),
        };
        assert_eq!(
            json!({
                "method": "model/list",
                "id": 6,
                "params": {
                    "limit": null,
                    "cursor": null,
                    "includeHidden": null
                }
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_list_collaboration_modes() -> Result<()> {
        let request = ClientRequest::CollaborationModeList {
            request_id: RequestId::Integer(7),
            params: v2::CollaborationModeListParams::default(),
        };
        assert_eq!(
            json!({
                "method": "collaborationMode/list",
                "id": 7,
                "params": {}
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_fs_get_metadata() -> Result<()> {
        let request = ClientRequest::FsGetMetadata {
            request_id: RequestId::Integer(9),
            params: v2::FsGetMetadataParams {
                path: absolute_path("tmp/example"),
            },
        };
        assert_eq!(
            json!({
                "method": "fs/getMetadata",
                "id": 9,
                "params": {
                    "path": absolute_path_string("tmp/example")
                }
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_fs_watch() -> Result<()> {
        let request = ClientRequest::FsWatch {
            request_id: RequestId::Integer(10),
            params: v2::FsWatchParams {
                watch_id: "watch-git".to_string(),
                path: absolute_path("tmp/repo/.git"),
            },
        };
        assert_eq!(
            json!({
                "method": "fs/watch",
                "id": 10,
                "params": {
                    "watchId": "watch-git",
                    "path": absolute_path_string("tmp/repo/.git")
                }
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_list_experimental_features() -> Result<()> {
        let request = ClientRequest::ExperimentalFeatureList {
            request_id: RequestId::Integer(8),
            params: v2::ExperimentalFeatureListParams::default(),
        };
        assert_eq!(
            json!({
                "method": "experimentalFeature/list",
                "id": 8,
                "params": {
                    "cursor": null,
                    "limit": null
                }
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_background_terminals_clean() -> Result<()> {
        let request = ClientRequest::ThreadBackgroundTerminalsClean {
            request_id: RequestId::Integer(8),
            params: v2::ThreadBackgroundTerminalsCleanParams {
                thread_id: "thr_123".to_string(),
            },
        };
        assert_eq!(
            json!({
                "method": "thread/backgroundTerminals/clean",
                "id": 8,
                "params": {
                    "threadId": "thr_123"
                }
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_realtime_start() -> Result<()> {
        let request = ClientRequest::ThreadRealtimeStart {
            request_id: RequestId::Integer(9),
            params: v2::ThreadRealtimeStartParams {
                thread_id: "thr_123".to_string(),
                output_modality: RealtimeOutputModality::Audio,
                prompt: Some(Some("You are on a call".to_string())),
                session_id: Some("sess_456".to_string()),
                transport: None,
                voice: Some(RealtimeVoice::Marin),
            },
        };
        assert_eq!(
            json!({
                "method": "thread/realtime/start",
                "id": 9,
                "params": {
                    "threadId": "thr_123",
                    "outputModality": "audio",
                    "prompt": "You are on a call",
                    "sessionId": "sess_456",
                    "transport": null,
                    "voice": "marin"
                }
            }),
            serde_json::to_value(&request)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_realtime_start_prompt_default_and_null() -> Result<()> {
        let default_prompt_request = ClientRequest::ThreadRealtimeStart {
            request_id: RequestId::Integer(9),
            params: v2::ThreadRealtimeStartParams {
                thread_id: "thr_123".to_string(),
                output_modality: RealtimeOutputModality::Audio,
                prompt: None,
                session_id: None,
                transport: None,
                voice: None,
            },
        };
        assert_eq!(
            json!({
                "method": "thread/realtime/start",
                "id": 9,
                "params": {
                    "threadId": "thr_123",
                    "outputModality": "audio",
                    "sessionId": null,
                    "transport": null,
                    "voice": null
                }
            }),
            serde_json::to_value(&default_prompt_request)?,
        );

        let null_prompt_request = ClientRequest::ThreadRealtimeStart {
            request_id: RequestId::Integer(9),
            params: v2::ThreadRealtimeStartParams {
                thread_id: "thr_123".to_string(),
                output_modality: RealtimeOutputModality::Audio,
                prompt: Some(None),
                session_id: None,
                transport: None,
                voice: None,
            },
        };
        assert_eq!(
            json!({
                "method": "thread/realtime/start",
                "id": 9,
                "params": {
                    "threadId": "thr_123",
                    "outputModality": "audio",
                    "prompt": null,
                    "sessionId": null,
                    "transport": null,
                    "voice": null
                }
            }),
            serde_json::to_value(&null_prompt_request)?,
        );

        let default_prompt_value = json!({
            "method": "thread/realtime/start",
            "id": 9,
            "params": {
                "threadId": "thr_123",
                "outputModality": "audio",
                "sessionId": null,
                "transport": null,
                "voice": null
            }
        });
        assert_eq!(
            serde_json::from_value::<ClientRequest>(default_prompt_value)?,
            default_prompt_request,
        );

        let null_prompt_value = json!({
            "method": "thread/realtime/start",
            "id": 9,
            "params": {
                "threadId": "thr_123",
                "outputModality": "audio",
                "prompt": null,
                "sessionId": null,
                "transport": null,
                "voice": null
            }
        });
        assert_eq!(
            serde_json::from_value::<ClientRequest>(null_prompt_value)?,
            null_prompt_request,
        );

        Ok(())
    }

    #[test]
    fn serialize_thread_status_changed_notification() -> Result<()> {
        let notification =
            ServerNotification::ThreadStatusChanged(v2::ThreadStatusChangedNotification {
                thread_id: "thr_123".to_string(),
                status: v2::ThreadStatus::Idle,
            });
        assert_eq!(
            json!({
                "method": "thread/status/changed",
                "params": {
                    "threadId": "thr_123",
                    "status": {
                        "type": "idle"
                    },
                }
            }),
            serde_json::to_value(&notification)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_realtime_output_audio_delta_notification() -> Result<()> {
        let notification = ServerNotification::ThreadRealtimeOutputAudioDelta(
            v2::ThreadRealtimeOutputAudioDeltaNotification {
                thread_id: "thr_123".to_string(),
                audio: v2::ThreadRealtimeAudioChunk {
                    data: "AQID".to_string(),
                    sample_rate: 24_000,
                    num_channels: 1,
                    samples_per_channel: Some(512),
                    item_id: None,
                },
            },
        );
        assert_eq!(
            json!({
                "method": "thread/realtime/outputAudio/delta",
                "params": {
                    "threadId": "thr_123",
                    "audio": {
                        "data": "AQID",
                        "sampleRate": 24000,
                        "numChannels": 1,
                        "samplesPerChannel": 512,
                        "itemId": null
                    }
                }
            }),
            serde_json::to_value(&notification)?,
        );
        Ok(())
    }

    #[test]
    fn mock_experimental_method_is_marked_experimental() {
        let request = ClientRequest::MockExperimentalMethod {
            request_id: RequestId::Integer(1),
            params: v2::MockExperimentalMethodParams::default(),
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("mock/experimentalMethod"));
    }

    #[test]
    fn thread_epiphany_role_result_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyRoleResult {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyRoleResultParams {
                thread_id: "thr_123".to_string(),
                role_id: v2::ThreadEpiphanyRoleId::Verification,
                binding_id: None,
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/roleResult"));
    }

    #[test]
    fn thread_epiphany_freshness_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyFreshness {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyFreshnessParams {
                thread_id: "thr_123".to_string(),
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/freshness"));
    }

    #[test]
    fn thread_epiphany_context_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyContext {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyContextParams {
                thread_id: "thr_123".to_string(),
                graph_node_ids: vec!["state-spine".to_string()],
                graph_edge_ids: Vec::new(),
                observation_ids: Vec::new(),
                evidence_ids: Vec::new(),
                include_active_frontier: Some(true),
                include_linked_evidence: Some(true),
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/context"));
    }

    #[test]
    fn thread_epiphany_reorient_result_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyReorientResult {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyReorientResultParams {
                thread_id: "thr_123".to_string(),
                binding_id: Some("reorient-worker".to_string()),
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/reorientResult"));
    }

    #[test]
    fn thread_epiphany_job_interrupt_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyJobInterrupt {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyJobInterruptParams {
                thread_id: "thr_123".to_string(),
                expected_revision: Some(3),
                binding_id: "specialist-work".to_string(),
                reason: Some("Stop the specialist smoke job.".to_string()),
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/jobInterrupt"));
    }

    #[test]
    fn thread_epiphany_update_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyUpdate {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyUpdateParams {
                thread_id: "thr_123".to_string(),
                expected_revision: Some(1),
                patch: v2::ThreadEpiphanyUpdatePatch {
                    objective: Some("Keep the map honest".to_string()),
                    ..Default::default()
                },
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/update"));
    }

    #[test]
    fn thread_epiphany_state_updated_notification_is_marked_experimental() {
        let notification = ServerNotification::ThreadEpiphanyStateUpdated(
            v2::ThreadEpiphanyStateUpdatedNotification {
                thread_id: "thr_123".to_string(),
                source: v2::ThreadEpiphanyStateUpdatedSource::Promote,
                revision: 0,
                changed_fields: vec![v2::ThreadEpiphanyStateUpdatedField::Churn],
                epiphany_state: epiphany_state_model::EpiphanyThreadState::default(),
            },
        );
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&notification);
        assert_eq!(reason, Some("thread/epiphany/stateUpdated"));
    }

    #[test]
    fn thread_epiphany_jobs_updated_notification_is_marked_experimental() {
        let notification = ServerNotification::ThreadEpiphanyJobsUpdated(
            v2::ThreadEpiphanyJobsUpdatedNotification {
                thread_id: "thr_123".to_string(),
                source: v2::ThreadEpiphanyJobsUpdatedSource::RuntimeProgress,
                state_revision: Some(1),
                jobs: vec![v2::ThreadEpiphanyJob {
                    id: "specialist-work".to_string(),
                    kind: v2::ThreadEpiphanyJobKind::Specialist,
                    scope: "runtime-bound specialist work".to_string(),
                    owner_role: "epiphany-harness".to_string(),
                    launcher_job_id: Some("launcher-specialist".to_string()),
                    authority_scope: Some("epiphany.specialist".to_string()),
                    backend_job_id: Some("job-123".to_string()),
                    status: v2::ThreadEpiphanyJobStatus::Running,
                    items_processed: Some(1),
                    items_total: Some(3),
                    progress_note: Some("Runtime agent job is running.".to_string()),
                    last_checkpoint_at_unix_seconds: Some(1_744_500_123),
                    blocking_reason: None,
                    active_thread_ids: vec!["worker-thread".to_string()],
                    linked_subgoal_ids: vec!["phase6".to_string()],
                    linked_graph_node_ids: vec!["job-surface".to_string()],
                }],
            },
        );
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&notification);
        assert_eq!(reason, Some("thread/epiphany/jobsUpdated"));
    }

    #[test]
    fn thread_realtime_start_is_marked_experimental() {
        let request = ClientRequest::ThreadRealtimeStart {
            request_id: RequestId::Integer(1),
            params: v2::ThreadRealtimeStartParams {
                thread_id: "thr_123".to_string(),
                output_modality: RealtimeOutputModality::Audio,
                prompt: Some(Some("You are on a call".to_string())),
                session_id: None,
                transport: None,
                voice: None,
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/realtime/start"));
    }
    #[test]
    fn thread_realtime_started_notification_is_marked_experimental() {
        let notification =
            ServerNotification::ThreadRealtimeStarted(v2::ThreadRealtimeStartedNotification {
                thread_id: "thr_123".to_string(),
                session_id: Some("sess_456".to_string()),
                version: RealtimeConversationVersion::V1,
            });
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&notification);
        assert_eq!(reason, Some("thread/realtime/started"));
    }

    #[test]
    fn thread_realtime_output_audio_delta_notification_is_marked_experimental() {
        let notification = ServerNotification::ThreadRealtimeOutputAudioDelta(
            v2::ThreadRealtimeOutputAudioDeltaNotification {
                thread_id: "thr_123".to_string(),
                audio: v2::ThreadRealtimeAudioChunk {
                    data: "AQID".to_string(),
                    sample_rate: 24_000,
                    num_channels: 1,
                    samples_per_channel: Some(512),
                    item_id: None,
                },
            },
        );
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&notification);
        assert_eq!(reason, Some("thread/realtime/outputAudio/delta"));
    }

    #[test]
    fn command_execution_request_approval_additional_permissions_is_marked_experimental() {
        let params = v2::CommandExecutionRequestApprovalParams {
            thread_id: "thr_123".to_string(),
            turn_id: "turn_123".to_string(),
            item_id: "call_123".to_string(),
            approval_id: None,
            reason: None,
            network_approval_context: None,
            command: Some("cat file".to_string()),
            cwd: None,
            command_actions: None,
            additional_permissions: Some(v2::AdditionalPermissionProfile {
                network: None,
                file_system: Some(v2::AdditionalFileSystemPermissions {
                    read: Some(vec![absolute_path("/tmp/allowed")]),
                    write: None,
                    glob_scan_max_depth: None,
                    entries: None,
                }),
            }),
            proposed_execpolicy_amendment: None,
            proposed_network_policy_amendments: None,
            available_decisions: None,
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&params);
        assert_eq!(
            reason,
            Some("item/commandExecution/requestApproval.additionalPermissions")
        );
    }
}
