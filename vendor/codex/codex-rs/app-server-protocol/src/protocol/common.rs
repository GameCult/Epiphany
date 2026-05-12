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
    #[experimental("thread/epiphany/scene")]
    ThreadEpiphanyScene => "thread/epiphany/scene" {
        params: v2::ThreadEpiphanySceneParams,
        response: v2::ThreadEpiphanySceneResponse,
    },
    #[experimental("thread/epiphany/view")]
    ThreadEpiphanyView => "thread/epiphany/view" {
        params: v2::ThreadEpiphanyViewParams,
        response: v2::ThreadEpiphanyViewResponse,
    },
    #[experimental("thread/epiphany/jobs")]
    ThreadEpiphanyJobs => "thread/epiphany/jobs" {
        params: v2::ThreadEpiphanyJobsParams,
        response: v2::ThreadEpiphanyJobsResponse,
    },
    #[experimental("thread/epiphany/roles")]
    ThreadEpiphanyRoles => "thread/epiphany/roles" {
        params: v2::ThreadEpiphanyRolesParams,
        response: v2::ThreadEpiphanyRolesResponse,
    },
    #[experimental("thread/epiphany/roleLaunch")]
    ThreadEpiphanyRoleLaunch => "thread/epiphany/roleLaunch" {
        params: v2::ThreadEpiphanyRoleLaunchParams,
        response: v2::ThreadEpiphanyRoleLaunchResponse,
    },
    #[experimental("thread/epiphany/roleResult")]
    ThreadEpiphanyRoleResult => "thread/epiphany/roleResult" {
        params: v2::ThreadEpiphanyRoleResultParams,
        response: v2::ThreadEpiphanyRoleResultResponse,
    },
    #[experimental("thread/epiphany/roleAccept")]
    ThreadEpiphanyRoleAccept => "thread/epiphany/roleAccept" {
        params: v2::ThreadEpiphanyRoleAcceptParams,
        response: v2::ThreadEpiphanyRoleAcceptResponse,
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
    #[experimental("thread/epiphany/planning")]
    ThreadEpiphanyPlanning => "thread/epiphany/planning" {
        params: v2::ThreadEpiphanyPlanningParams,
        response: v2::ThreadEpiphanyPlanningResponse,
    },
    #[experimental("thread/epiphany/graphQuery")]
    ThreadEpiphanyGraphQuery => "thread/epiphany/graphQuery" {
        params: v2::ThreadEpiphanyGraphQueryParams,
        response: v2::ThreadEpiphanyGraphQueryResponse,
    },
    #[experimental("thread/epiphany/reorientLaunch")]
    ThreadEpiphanyReorientLaunch => "thread/epiphany/reorientLaunch" {
        params: v2::ThreadEpiphanyReorientLaunchParams,
        response: v2::ThreadEpiphanyReorientLaunchResponse,
    },
    #[experimental("thread/epiphany/reorientResult")]
    ThreadEpiphanyReorientResult => "thread/epiphany/reorientResult" {
        params: v2::ThreadEpiphanyReorientResultParams,
        response: v2::ThreadEpiphanyReorientResultResponse,
    },
    #[experimental("thread/epiphany/reorientAccept")]
    ThreadEpiphanyReorientAccept => "thread/epiphany/reorientAccept" {
        params: v2::ThreadEpiphanyReorientAcceptParams,
        response: v2::ThreadEpiphanyReorientAcceptResponse,
    },
    #[experimental("thread/epiphany/index")]
    ThreadEpiphanyIndex => "thread/epiphany/index" {
        params: v2::ThreadEpiphanyIndexParams,
        response: v2::ThreadEpiphanyIndexResponse,
    },
    #[experimental("thread/epiphany/distill")]
    ThreadEpiphanyDistill => "thread/epiphany/distill" {
        params: v2::ThreadEpiphanyDistillParams,
        response: v2::ThreadEpiphanyDistillResponse,
    },
    #[experimental("thread/epiphany/propose")]
    ThreadEpiphanyPropose => "thread/epiphany/propose" {
        params: v2::ThreadEpiphanyProposeParams,
        response: v2::ThreadEpiphanyProposeResponse,
    },
    #[experimental("thread/epiphany/promote")]
    ThreadEpiphanyPromote => "thread/epiphany/promote" {
        params: v2::ThreadEpiphanyPromoteParams,
        response: v2::ThreadEpiphanyPromoteResponse,
    },
    #[experimental("thread/epiphany/jobLaunch")]
    ThreadEpiphanyJobLaunch => "thread/epiphany/jobLaunch" {
        params: v2::ThreadEpiphanyJobLaunchParams,
        response: v2::ThreadEpiphanyJobLaunchResponse,
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
    #[experimental("thread/epiphany/retrieve")]
    ThreadEpiphanyRetrieve => "thread/epiphany/retrieve" {
        params: v2::ThreadEpiphanyRetrieveParams,
        response: v2::ThreadEpiphanyRetrieveResponse,
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
    SkillsList => "skills/list" {
        params: v2::SkillsListParams,
        response: v2::SkillsListResponse,
    },
    MarketplaceAdd => "marketplace/add" {
        params: v2::MarketplaceAddParams,
        response: v2::MarketplaceAddResponse,
    },
    MarketplaceRemove => "marketplace/remove" {
        params: v2::MarketplaceRemoveParams,
        response: v2::MarketplaceRemoveResponse,
    },
    PluginList => "plugin/list" {
        params: v2::PluginListParams,
        response: v2::PluginListResponse,
    },
    PluginRead => "plugin/read" {
        params: v2::PluginReadParams,
        response: v2::PluginReadResponse,
    },
    AppsList => "app/list" {
        params: v2::AppsListParams,
        response: v2::AppsListResponse,
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
    SkillsConfigWrite => "skills/config/write" {
        params: v2::SkillsConfigWriteParams,
        response: v2::SkillsConfigWriteResponse,
    },
    PluginInstall => "plugin/install" {
        params: v2::PluginInstallParams,
        response: v2::PluginInstallResponse,
    },
    PluginUninstall => "plugin/uninstall" {
        params: v2::PluginUninstallParams,
        response: v2::PluginUninstallResponse,
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
    ExternalAgentConfigDetect => "externalAgentConfig/detect" {
        params: v2::ExternalAgentConfigDetectParams,
        response: v2::ExternalAgentConfigDetectResponse,
    },
    ExternalAgentConfigImport => "externalAgentConfig/import" {
        params: v2::ExternalAgentConfigImportParams,
        response: v2::ExternalAgentConfigImportResponse,
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
    SkillsChanged => "skills/changed" (v2::SkillsChangedNotification),
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
    AppListUpdated => "app/list/updated" (v2::AppListUpdatedNotification),
    ExternalAgentConfigImportCompleted => "externalAgentConfig/import/completed" (v2::ExternalAgentConfigImportCompletedNotification),
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
            server_name: "codex_apps".to_string(),
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
                    "serverName": "codex_apps",
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
    fn serialize_thread_epiphany_scene_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyScene {
            request_id: RequestId::Integer(8),
            response: v2::ThreadEpiphanySceneResponse {
                thread_id: "thr_123".to_string(),
                scene: v2::ThreadEpiphanyScene {
                    state_status: v2::ThreadEpiphanySceneStateStatus::Ready,
                    source: v2::ThreadEpiphanySceneSource::Live,
                    revision: Some(3),
                    objective: Some("Keep the map visible".to_string()),
                    active_subgoal: Some(v2::ThreadEpiphanySceneSubgoal {
                        id: "phase-6".to_string(),
                        title: "Reflect typed state".to_string(),
                        status: "active".to_string(),
                        summary: None,
                        active: true,
                    }),
                    subgoals: Vec::new(),
                    invariant_status_counts: vec![v2::ThreadEpiphanySceneStatusCount {
                        status: "ok".to_string(),
                        count: 2,
                    }],
                    graph: v2::ThreadEpiphanySceneGraph {
                        architecture_node_count: 4,
                        architecture_edge_count: 3,
                        dataflow_node_count: 2,
                        dataflow_edge_count: 1,
                        link_count: 2,
                        active_node_ids: vec!["state-spine".to_string()],
                        active_edge_ids: Vec::new(),
                        open_question_count: 1,
                        open_gap_count: 0,
                        dirty_paths: vec![PathBuf::from("notes/scene.md")],
                        checkpoint_id: Some("ck-1".to_string()),
                        checkpoint_summary: None,
                    },
                    retrieval: Some(v2::ThreadEpiphanySceneRetrieval {
                        workspace_root: PathBuf::from("/workspace"),
                        status: codex_protocol::protocol::EpiphanyRetrievalStatus::Ready,
                        semantic_available: true,
                        index_revision: Some("qdrant-v1".to_string()),
                        indexed_file_count: Some(12),
                        indexed_chunk_count: Some(34),
                        shard_count: 1,
                        dirty_path_count: 0,
                    }),
                    investigation_checkpoint: Some(
                        v2::ThreadEpiphanySceneInvestigationCheckpoint {
                            checkpoint_id: "ix-1".to_string(),
                            kind: "slice_planning".to_string(),
                            disposition: codex_protocol::protocol::EpiphanyInvestigationDisposition::ResumeReady,
                            focus: "Keep the durable resume packet visible.".to_string(),
                            summary: Some("Checkpointed the bounded planning slice.".to_string()),
                            next_action: Some("Patch one typed field next.".to_string()),
                            captured_at_turn_id: Some("turn-7".to_string()),
                            open_question_count: 1,
                            code_ref_count: 2,
                            evidence_count: 1,
                        },
                    ),
                    observations: v2::ThreadEpiphanySceneRecords {
                        total_count: 1,
                        latest: vec![v2::ThreadEpiphanySceneRecord {
                            id: "obs-1".to_string(),
                            kind: "smoke".to_string(),
                            status: "ok".to_string(),
                            summary: "Scene projected".to_string(),
                            code_ref_count: 1,
                        }],
                    },
                    evidence: v2::ThreadEpiphanySceneRecords::default(),
                    churn: Some(v2::ThreadEpiphanySceneChurn {
                        understanding_status: "ready".to_string(),
                        diff_pressure: "low".to_string(),
                        graph_freshness: Some("fresh".to_string()),
                        warning: None,
                        unexplained_writes: Some(0),
                    }),
                    available_actions: vec![
                        v2::ThreadEpiphanySceneAction::Index,
                        v2::ThreadEpiphanySceneAction::Retrieve,
                        v2::ThreadEpiphanySceneAction::Distill,
                        v2::ThreadEpiphanySceneAction::Context,
                        v2::ThreadEpiphanySceneAction::Planning,
                        v2::ThreadEpiphanySceneAction::Jobs,
                        v2::ThreadEpiphanySceneAction::Roles,
                        v2::ThreadEpiphanySceneAction::Coordinator,
                        v2::ThreadEpiphanySceneAction::RoleLaunch,
                        v2::ThreadEpiphanySceneAction::RoleResult,
                        v2::ThreadEpiphanySceneAction::JobLaunch,
                        v2::ThreadEpiphanySceneAction::Freshness,
                        v2::ThreadEpiphanySceneAction::Pressure,
                        v2::ThreadEpiphanySceneAction::Reorient,
                        v2::ThreadEpiphanySceneAction::Crrc,
                        v2::ThreadEpiphanySceneAction::ReorientLaunch,
                        v2::ThreadEpiphanySceneAction::Update,
                        v2::ThreadEpiphanySceneAction::JobInterrupt,
                        v2::ThreadEpiphanySceneAction::Propose,
                        v2::ThreadEpiphanySceneAction::Promote,
                    ],
                },
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(8));
        assert_eq!(response.method(), "thread/epiphany/scene");
        assert_eq!(
            json!({
                "method": "thread/epiphany/scene",
                "id": 8,
                "response": {
                    "threadId": "thr_123",
                    "scene": {
                        "stateStatus": "ready",
                        "source": "live",
                        "revision": 3,
                        "objective": "Keep the map visible",
                        "activeSubgoal": {
                            "id": "phase-6",
                            "title": "Reflect typed state",
                            "status": "active",
                            "active": true
                        },
                        "invariantStatusCounts": [
                            {
                                "status": "ok",
                                "count": 2
                            }
                        ],
                        "graph": {
                            "architectureNodeCount": 4,
                            "architectureEdgeCount": 3,
                            "dataflowNodeCount": 2,
                            "dataflowEdgeCount": 1,
                            "linkCount": 2,
                            "activeNodeIds": ["state-spine"],
                            "openQuestionCount": 1,
                            "openGapCount": 0,
                            "dirtyPaths": ["notes/scene.md"],
                            "checkpointId": "ck-1"
                        },
                        "retrieval": {
                            "workspaceRoot": "/workspace",
                            "status": "ready",
                            "semanticAvailable": true,
                            "indexRevision": "qdrant-v1",
                            "indexedFileCount": 12,
                            "indexedChunkCount": 34,
                            "shardCount": 1,
                            "dirtyPathCount": 0
                        },
                        "investigationCheckpoint": {
                            "checkpointId": "ix-1",
                            "kind": "slice_planning",
                            "disposition": "resume_ready",
                            "focus": "Keep the durable resume packet visible.",
                            "summary": "Checkpointed the bounded planning slice.",
                            "nextAction": "Patch one typed field next.",
                            "capturedAtTurnId": "turn-7",
                            "openQuestionCount": 1,
                            "codeRefCount": 2,
                            "evidenceCount": 1
                        },
                        "observations": {
                            "totalCount": 1,
                            "latest": [
                                {
                                    "id": "obs-1",
                                    "kind": "smoke",
                                    "status": "ok",
                                    "summary": "Scene projected",
                                    "codeRefCount": 1
                                }
                            ]
                        },
                        "evidence": {
                            "totalCount": 0
                        },
                        "churn": {
                            "understandingStatus": "ready",
                            "diffPressure": "low",
                            "graphFreshness": "fresh",
                            "unexplainedWrites": 0
                        },
                        "availableActions": [
                            "index",
                            "retrieve",
                            "distill",
                            "context",
                            "planning",
                            "jobs",
                            "roles",
                            "coordinator",
                            "roleLaunch",
                            "roleResult",
                            "jobLaunch",
                            "freshness",
                            "pressure",
                            "reorient",
                            "crrc",
                            "reorientLaunch",
                            "update",
                            "jobInterrupt",
                            "propose",
                            "promote"
                        ]
                    }
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
    fn serialize_thread_epiphany_jobs_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyJobs {
            request_id: RequestId::Integer(8),
            response: v2::ThreadEpiphanyJobsResponse {
                thread_id: "thr_123".to_string(),
                source: v2::ThreadEpiphanyJobsSource::Live,
                state_revision: Some(3),
                jobs: vec![v2::ThreadEpiphanyJob {
                    id: "retrieval-index".to_string(),
                    kind: v2::ThreadEpiphanyJobKind::Indexing,
                    scope: "/workspace".to_string(),
                    owner_role: "epiphany-core".to_string(),
                    launcher_job_id: None,
                    authority_scope: None,
                    backend_job_id: None,
                    status: v2::ThreadEpiphanyJobStatus::Needed,
                    items_processed: Some(12),
                    items_total: None,
                    progress_note: Some(
                        "Retrieval catalog is stale; refresh is available.".to_string(),
                    ),
                    last_checkpoint_at_unix_seconds: Some(1_744_500_000),
                    blocking_reason: None,
                    active_thread_ids: Vec::new(),
                    linked_subgoal_ids: vec!["phase-6".to_string()],
                    linked_graph_node_ids: vec!["retrieval".to_string()],
                }],
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(8));
        assert_eq!(response.method(), "thread/epiphany/jobs");
        assert_eq!(
            json!({
                "method": "thread/epiphany/jobs",
                "id": 8,
                "response": {
                    "threadId": "thr_123",
                    "source": "live",
                    "stateRevision": 3,
                    "jobs": [
                        {
                            "id": "retrieval-index",
                            "kind": "indexing",
                            "scope": "/workspace",
                            "ownerRole": "epiphany-core",
                            "status": "needed",
                            "itemsProcessed": 12,
                            "progressNote": "Retrieval catalog is stale; refresh is available.",
                            "lastCheckpointAtUnixSeconds": 1744500000,
                            "linkedSubgoalIds": ["phase-6"],
                            "linkedGraphNodeIds": ["retrieval"]
                        }
                    ]
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_roles_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyRoles {
            request_id: RequestId::Integer(8),
            response: v2::ThreadEpiphanyRolesResponse {
                thread_id: "thr_123".to_string(),
                source: v2::ThreadEpiphanyRolesSource::Live,
                state_status: v2::ThreadEpiphanyReorientStateStatus::Ready,
                state_revision: Some(3),
                roles: vec![v2::ThreadEpiphanyRoleLane {
                    id: v2::ThreadEpiphanyRoleId::Verification,
                    title: "Verification / Review".to_string(),
                    owner_role: "epiphany-verifier".to_string(),
                    status: v2::ThreadEpiphanyRoleStatus::Needed,
                    note: "Review evidence before promotion.".to_string(),
                    jobs: vec![v2::ThreadEpiphanyJob {
                        id: "verification".to_string(),
                        kind: v2::ThreadEpiphanyJobKind::Verification,
                        scope: "invariant verification".to_string(),
                        owner_role: "epiphany-verifier".to_string(),
                        launcher_job_id: None,
                        authority_scope: None,
                        backend_job_id: None,
                        status: v2::ThreadEpiphanyJobStatus::Needed,
                        items_processed: None,
                        items_total: None,
                        progress_note: None,
                        last_checkpoint_at_unix_seconds: None,
                        blocking_reason: None,
                        active_thread_ids: Vec::new(),
                        linked_subgoal_ids: Vec::new(),
                        linked_graph_node_ids: Vec::new(),
                    }],
                    authority_scopes: vec!["thread/epiphany/promote".to_string()],
                    recommended_action: Some(v2::ThreadEpiphanySceneAction::Promote),
                }],
                note: "Role ownership is derived read-only.".to_string(),
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(8));
        assert_eq!(response.method(), "thread/epiphany/roles");
        assert_eq!(
            json!({
                "method": "thread/epiphany/roles",
                "id": 8,
                "response": {
                    "threadId": "thr_123",
                    "source": "live",
                    "stateStatus": "ready",
                    "stateRevision": 3,
                    "roles": [
                        {
                            "id": "verification",
                            "title": "Verification / Review",
                            "ownerRole": "epiphany-verifier",
                            "status": "needed",
                            "note": "Review evidence before promotion.",
                            "jobs": [
                                {
                                    "id": "verification",
                                    "kind": "verification",
                                    "scope": "invariant verification",
                                    "ownerRole": "epiphany-verifier",
                                    "status": "needed"
                                }
                            ],
                            "authorityScopes": ["thread/epiphany/promote"],
                            "recommendedAction": "promote"
                        }
                    ],
                    "note": "Role ownership is derived read-only."
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_role_launch_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyRoleLaunch {
            request_id: RequestId::Integer(8),
            response: v2::ThreadEpiphanyRoleLaunchResponse {
                thread_id: "thr_123".to_string(),
                role_id: v2::ThreadEpiphanyRoleId::Modeling,
                revision: 4,
                changed_fields: vec![v2::ThreadEpiphanyStateUpdatedField::JobBindings],
                epiphany_state: codex_protocol::protocol::EpiphanyThreadState {
                    revision: 4,
                    job_bindings: vec![codex_protocol::protocol::EpiphanyJobBinding {
                        id: "modeling-checkpoint-worker".to_string(),
                        kind: codex_protocol::protocol::EpiphanyJobKind::Specialist,
                        scope: "role-scoped modeling/checkpoint maintenance".to_string(),
                        owner_role: "epiphany-modeler".to_string(),
                        launcher_job_id: Some("epiphany-launch-1".to_string()),
                        authority_scope: Some("epiphany.role.modeling".to_string()),
                        backend_job_id: Some("backend-1".to_string()),
                        linked_subgoal_ids: vec!["phase-6".to_string()],
                        linked_graph_node_ids: vec!["state-spine".to_string()],
                        progress_note: Some(
                            "Explicitly launched through the Epiphany authority surface onto the heartbeat backend."
                                .to_string(),
                        ),
                        blocking_reason: None,
                    }],
                    ..Default::default()
                },
                job: v2::ThreadEpiphanyJob {
                    id: "modeling-checkpoint-worker".to_string(),
                    kind: v2::ThreadEpiphanyJobKind::Specialist,
                    scope: "role-scoped modeling/checkpoint maintenance".to_string(),
                    owner_role: "epiphany-modeler".to_string(),
                    launcher_job_id: Some("epiphany-launch-1".to_string()),
                    authority_scope: Some("epiphany.role.modeling".to_string()),
                    backend_job_id: Some("backend-1".to_string()),
                    status: v2::ThreadEpiphanyJobStatus::Pending,
                    items_processed: None,
                    items_total: None,
                    progress_note: Some(
                        "Explicitly launched through the Epiphany authority surface onto the heartbeat backend."
                            .to_string(),
                    ),
                    last_checkpoint_at_unix_seconds: None,
                    blocking_reason: None,
                    active_thread_ids: Vec::new(),
                    linked_subgoal_ids: vec!["phase-6".to_string()],
                    linked_graph_node_ids: vec!["state-spine".to_string()],
                },
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(8));
        assert_eq!(response.method(), "thread/epiphany/roleLaunch");
        assert_eq!(
            json!({
                "method": "thread/epiphany/roleLaunch",
                "id": 8,
                "response": {
                    "threadId": "thr_123",
                    "roleId": "modeling",
                    "revision": 4,
                    "changedFields": ["jobBindings"],
                    "epiphanyState": {
                        "revision": 4,
                        "job_bindings": [{
                            "id": "modeling-checkpoint-worker",
                            "kind": "specialist",
                            "scope": "role-scoped modeling/checkpoint maintenance",
                            "owner_role": "epiphany-modeler",
                            "launcher_job_id": "epiphany-launch-1",
                            "authority_scope": "epiphany.role.modeling",
                            "backend_job_id": "backend-1",
                            "linked_subgoal_ids": ["phase-6"],
                            "linked_graph_node_ids": ["state-spine"],
                            "progress_note": "Explicitly launched through the Epiphany authority surface onto the heartbeat backend."
                        }]
                    },
                    "job": {
                        "id": "modeling-checkpoint-worker",
                        "kind": "specialist",
                        "scope": "role-scoped modeling/checkpoint maintenance",
                        "ownerRole": "epiphany-modeler",
                        "launcherJobId": "epiphany-launch-1",
                        "authorityScope": "epiphany.role.modeling",
                        "backendJobId": "backend-1",
                        "status": "pending",
                        "progressNote": "Explicitly launched through the Epiphany authority surface onto the heartbeat backend.",
                        "linkedSubgoalIds": ["phase-6"],
                        "linkedGraphNodeIds": ["state-spine"]
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
    fn serialize_thread_epiphany_role_accept_response() -> Result<()> {
        let patch = v2::ThreadEpiphanyUpdatePatch {
            graph_frontier: Some(codex_protocol::protocol::EpiphanyGraphFrontier {
                active_node_ids: vec!["state-spine".to_string()],
                ..Default::default()
            }),
            ..Default::default()
        };
        let finding = v2::ThreadEpiphanyRoleFinding {
            role_id: v2::ThreadEpiphanyRoleId::Modeling,
            verdict: Some("checkpoint-update-needed".to_string()),
            summary: Some("Graph frontier should keep state-spine active.".to_string()),
            next_safe_move: Some("Accept the reviewed modeling patch.".to_string()),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: vec!["src/lib.rs".to_string()],
            frontier_node_ids: vec!["state-spine".to_string()],
            evidence_ids: Vec::new(),
            artifact_refs: Vec::new(),
            runtime_result_id: Some("result-modeling-1".to_string()),
            runtime_job_id: Some("backend-1".to_string()),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            state_patch: Some(patch.clone()),
            self_patch: None,
            self_persistence: None,
            job_error: None,
            item_error: None,
        };
        let response = ClientResponse::ThreadEpiphanyRoleAccept {
            request_id: RequestId::Integer(10),
            response: v2::ThreadEpiphanyRoleAcceptResponse {
                revision: 5,
                changed_fields: vec![
                    v2::ThreadEpiphanyStateUpdatedField::AcceptanceReceipts,
                    v2::ThreadEpiphanyStateUpdatedField::GraphFrontier,
                    v2::ThreadEpiphanyStateUpdatedField::Observations,
                    v2::ThreadEpiphanyStateUpdatedField::Evidence,
                ],
                epiphany_state: codex_protocol::protocol::EpiphanyThreadState {
                    revision: 5,
                    ..Default::default()
                },
                role_id: v2::ThreadEpiphanyRoleId::Modeling,
                binding_id: "modeling-checkpoint-worker".to_string(),
                accepted_receipt_id: "accept-modeling-1".to_string(),
                accepted_observation_id: "obs-modeling-1".to_string(),
                accepted_evidence_id: "ev-modeling-1".to_string(),
                applied_patch: patch,
                finding,
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(10));
        assert_eq!(response.method(), "thread/epiphany/roleAccept");
        assert_eq!(
            json!({
                "method": "thread/epiphany/roleAccept",
                "id": 10,
                "response": {
                    "revision": 5,
                    "changedFields": ["acceptanceReceipts", "graphFrontier", "observations", "evidence"],
                    "epiphanyState": {
                        "revision": 5
                    },
                    "roleId": "modeling",
                    "bindingId": "modeling-checkpoint-worker",
                    "acceptedReceiptId": "accept-modeling-1",
                    "acceptedObservationId": "obs-modeling-1",
                    "acceptedEvidenceId": "ev-modeling-1",
                    "appliedPatch": {
                        "graphFrontier": {
                            "active_node_ids": ["state-spine"]
                        }
                    },
                    "finding": {
                        "roleId": "modeling",
                        "verdict": "checkpoint-update-needed",
                        "summary": "Graph frontier should keep state-spine active.",
                        "nextSafeMove": "Accept the reviewed modeling patch.",
                        "filesInspected": ["src/lib.rs"],
                        "frontierNodeIds": ["state-spine"],
                        "runtimeResultId": "result-modeling-1",
                        "runtimeJobId": "backend-1",
                        "statePatch": {
                            "graphFrontier": {
                                "active_node_ids": ["state-spine"]
                            }
                        }
                    }
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
                        architecture_nodes: vec![codex_protocol::protocol::EpiphanyGraphNode {
                            id: "state-spine".to_string(),
                            title: "State spine".to_string(),
                            purpose: "Keep understanding typed".to_string(),
                            ..Default::default()
                        }],
                        architecture_edges: vec![codex_protocol::protocol::EpiphanyGraphEdge {
                            id: Some("edge-state-read".to_string()),
                            source_id: "state-spine".to_string(),
                            target_id: "client-read".to_string(),
                            kind: "projects".to_string(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                    frontier: Some(codex_protocol::protocol::EpiphanyGraphFrontier {
                        active_node_ids: vec!["state-spine".to_string()],
                        ..Default::default()
                    }),
                    checkpoint: None,
                    investigation_checkpoint: Some(
                        codex_protocol::protocol::EpiphanyInvestigationCheckpoint {
                            checkpoint_id: "ix-1".to_string(),
                            kind: "source_gathering".to_string(),
                            focus: "Audit the compaction seam.".to_string(),
                            next_action: Some(
                                "Re-gather source if this packet is stale.".to_string(),
                            ),
                            ..Default::default()
                        },
                    ),
                    observations: vec![codex_protocol::protocol::EpiphanyObservation {
                        id: "obs-1".to_string(),
                        summary: "Context shard is read-only.".to_string(),
                        source_kind: "smoke".to_string(),
                        status: "ok".to_string(),
                        evidence_ids: vec!["ev-1".to_string()],
                        ..Default::default()
                    }],
                    evidence: vec![codex_protocol::protocol::EpiphanyEvidenceRecord {
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
    fn serialize_thread_epiphany_planning_response() -> Result<()> {
        let github_source = codex_protocol::protocol::EpiphanyPlanningSourceRef {
            kind: "github_issue".to_string(),
            provider: Some("github".to_string()),
            repo: Some("GameCult/Epiphany".to_string()),
            issue_number: Some(42),
            url: Some("https://github.com/GameCult/Epiphany/issues/42".to_string()),
            labels: vec!["gui".to_string()],
            imported_at: Some("2026-05-01T05:00:00Z".to_string()),
            ..Default::default()
        };
        let response = ClientResponse::ThreadEpiphanyPlanning {
            request_id: RequestId::Integer(9),
            response: v2::ThreadEpiphanyPlanningResponse {
                thread_id: "thr_123".to_string(),
                source: v2::ThreadEpiphanyContextSource::Live,
                state_status: v2::ThreadEpiphanyContextStateStatus::Ready,
                state_revision: Some(5),
                planning: codex_protocol::protocol::EpiphanyPlanningState {
                    captures: vec![codex_protocol::protocol::EpiphanyPlanningCapture {
                        id: "capture-github-42".to_string(),
                        title: "Import issue backlog".to_string(),
                        confidence: "medium".to_string(),
                        status: "new".to_string(),
                        source: github_source.clone(),
                        ..Default::default()
                    }],
                    backlog_items: vec![codex_protocol::protocol::EpiphanyBacklogItem {
                        id: "backlog-planning-api".to_string(),
                        title: "Expose planning projection".to_string(),
                        kind: "feature".to_string(),
                        summary: "Make planning state queryable by the GUI.".to_string(),
                        status: "ready".to_string(),
                        horizon: "now".to_string(),
                        priority: codex_protocol::protocol::EpiphanyPlanningPriority {
                            value: "p1".to_string(),
                            rationale: "Unblocks the planning GUI.".to_string(),
                            ..Default::default()
                        },
                        confidence: "high".to_string(),
                        product_area: "gui".to_string(),
                        lane_hints: vec!["imagination".to_string()],
                        source_refs: vec![github_source],
                        ..Default::default()
                    }],
                    objective_drafts: vec![codex_protocol::protocol::EpiphanyObjectiveDraft {
                        id: "objdraft-planning-api".to_string(),
                        title: "Build planning API slice".to_string(),
                        summary: "Land typed planning state and read-only projection.".to_string(),
                        source_item_ids: vec!["backlog-planning-api".to_string()],
                        scope: codex_protocol::protocol::EpiphanyObjectiveDraftScope {
                            includes: vec!["thread/epiphany/planning".to_string()],
                            excludes: vec!["GUI planning table".to_string()],
                        },
                        acceptance_criteria: vec![
                            "Projection returns planning counts.".to_string(),
                        ],
                        lane_plan: codex_protocol::protocol::EpiphanyObjectiveDraftLanePlan {
                            imagination: Some("Shape backlog and objective draft.".to_string()),
                            hands: Some("Wire protocol and app-server read.".to_string()),
                            ..Default::default()
                        },
                        review_gates: vec!["Human adoption required".to_string()],
                        status: "draft".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                summary: v2::ThreadEpiphanyPlanningSummary {
                    capture_count: 1,
                    pending_capture_count: 1,
                    github_issue_capture_count: 1,
                    backlog_item_count: 1,
                    ready_backlog_item_count: 1,
                    roadmap_stream_count: 0,
                    objective_draft_count: 1,
                    draft_objective_count: 1,
                    active_objective: Some("Current adopted work".to_string()),
                    note: "Planning substrate is available.".to_string(),
                },
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(9));
        assert_eq!(response.method(), "thread/epiphany/planning");
        assert_eq!(
            json!({
                "method": "thread/epiphany/planning",
                "id": 9,
                "response": {
                    "threadId": "thr_123",
                    "source": "live",
                    "stateStatus": "ready",
                    "stateRevision": 5,
                    "planning": {
                        "captures": [{
                            "id": "capture-github-42",
                            "title": "Import issue backlog",
                            "confidence": "medium",
                            "status": "new",
                            "source": {
                                "kind": "github_issue",
                                "provider": "github",
                                "repo": "GameCult/Epiphany",
                                "issue_number": 42,
                                "url": "https://github.com/GameCult/Epiphany/issues/42",
                                "labels": ["gui"],
                                "imported_at": "2026-05-01T05:00:00Z"
                            }
                        }],
                        "backlog_items": [{
                            "id": "backlog-planning-api",
                            "title": "Expose planning projection",
                            "kind": "feature",
                            "summary": "Make planning state queryable by the GUI.",
                            "status": "ready",
                            "horizon": "now",
                            "priority": {
                                "value": "p1",
                                "rationale": "Unblocks the planning GUI."
                            },
                            "confidence": "high",
                            "product_area": "gui",
                            "lane_hints": ["imagination"],
                            "source_refs": [{
                                "kind": "github_issue",
                                "provider": "github",
                                "repo": "GameCult/Epiphany",
                                "issue_number": 42,
                                "url": "https://github.com/GameCult/Epiphany/issues/42",
                                "labels": ["gui"],
                                "imported_at": "2026-05-01T05:00:00Z"
                            }]
                        }],
                        "objective_drafts": [{
                            "id": "objdraft-planning-api",
                            "title": "Build planning API slice",
                            "summary": "Land typed planning state and read-only projection.",
                            "source_item_ids": ["backlog-planning-api"],
                            "scope": {
                                "includes": ["thread/epiphany/planning"],
                                "excludes": ["GUI planning table"]
                            },
                            "acceptance_criteria": ["Projection returns planning counts."],
                            "lane_plan": {
                                "imagination": "Shape backlog and objective draft.",
                                "hands": "Wire protocol and app-server read."
                            },
                            "review_gates": ["Human adoption required"],
                            "status": "draft"
                        }]
                    },
                    "summary": {
                        "captureCount": 1,
                        "pendingCaptureCount": 1,
                        "githubIssueCaptureCount": 1,
                        "backlogItemCount": 1,
                        "readyBacklogItemCount": 1,
                        "roadmapStreamCount": 0,
                        "objectiveDraftCount": 1,
                        "draftObjectiveCount": 1,
                        "activeObjective": "Current adopted work",
                        "note": "Planning substrate is available."
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
                    architecture_nodes: vec![codex_protocol::protocol::EpiphanyGraphNode {
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
    fn serialize_thread_epiphany_reorient_launch_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyReorientLaunch {
            request_id: RequestId::Integer(8),
            response: v2::ThreadEpiphanyReorientLaunchResponse {
                thread_id: "thr_123".to_string(),
                source: v2::ThreadEpiphanyReorientSource::Live,
                state_status: v2::ThreadEpiphanyReorientStateStatus::Ready,
                state_revision: Some(4),
                decision: v2::ThreadEpiphanyReorientDecision {
                    action: v2::ThreadEpiphanyReorientAction::Regather,
                    checkpoint_status: v2::ThreadEpiphanyReorientCheckpointStatus::ResumeReady,
                    checkpoint_id: Some("ix-1".to_string()),
                    pressure_level: v2::ThreadEpiphanyPressureLevel::High,
                    retrieval_status: v2::ThreadEpiphanyRetrievalFreshnessStatus::Stale,
                    graph_status: v2::ThreadEpiphanyGraphFreshnessStatus::Ready,
                    watcher_status: v2::ThreadEpiphanyInvalidationStatus::Changed,
                    reasons: vec![
                        v2::ThreadEpiphanyReorientReason::CheckpointPathsChanged,
                        v2::ThreadEpiphanyReorientReason::FrontierChanged,
                    ],
                    checkpoint_dirty_paths: vec![PathBuf::from("src/lib.rs")],
                    checkpoint_changed_paths: vec![PathBuf::from("src/lib.rs")],
                    active_frontier_node_ids: vec!["state-spine".to_string()],
                    next_action: "Re-gather source before editing.".to_string(),
                    note: "Resume-ready checkpoint was invalidated by watcher/frontier drift."
                        .to_string(),
                },
                revision: 5,
                changed_fields: vec![v2::ThreadEpiphanyStateUpdatedField::JobBindings],
                epiphany_state: codex_protocol::protocol::EpiphanyThreadState {
                    revision: 5,
                    job_bindings: vec![codex_protocol::protocol::EpiphanyJobBinding {
                        id: "reorient-worker".to_string(),
                        kind: codex_protocol::protocol::EpiphanyJobKind::Specialist,
                        scope: "reorient-guided checkpoint regather".to_string(),
                        owner_role: "epiphany-reorient".to_string(),
                        launcher_job_id: Some("epiphany-launch-1".to_string()),
                        authority_scope: Some("epiphany.reorient.regather".to_string()),
                        backend_job_id: Some("backend-1".to_string()),
                        linked_subgoal_ids: vec!["phase-6".to_string()],
                        linked_graph_node_ids: vec!["state-spine".to_string()],
                        progress_note: Some(
                            "Explicitly launched through the Epiphany authority surface onto the heartbeat backend."
                                .to_string(),
                        ),
                        blocking_reason: None,
                    }],
                    ..Default::default()
                },
                job: v2::ThreadEpiphanyJob {
                    id: "reorient-worker".to_string(),
                    kind: v2::ThreadEpiphanyJobKind::Specialist,
                    scope: "reorient-guided checkpoint regather".to_string(),
                    owner_role: "epiphany-reorient".to_string(),
                    launcher_job_id: Some("epiphany-launch-1".to_string()),
                    authority_scope: Some("epiphany.reorient.regather".to_string()),
                    backend_job_id: Some("backend-1".to_string()),
                    status: v2::ThreadEpiphanyJobStatus::Pending,
                    items_processed: None,
                    items_total: None,
                    progress_note: Some(
                        "Explicitly launched through the Epiphany authority surface onto the heartbeat backend."
                            .to_string(),
                    ),
                    last_checkpoint_at_unix_seconds: None,
                    blocking_reason: None,
                    active_thread_ids: Vec::new(),
                    linked_subgoal_ids: vec!["phase-6".to_string()],
                    linked_graph_node_ids: vec!["state-spine".to_string()],
                },
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(8));
        assert_eq!(response.method(), "thread/epiphany/reorientLaunch");
        assert_eq!(
            json!({
                "method": "thread/epiphany/reorientLaunch",
                "id": 8,
                "response": {
                    "threadId": "thr_123",
                    "source": "live",
                    "stateStatus": "ready",
                    "stateRevision": 4,
                    "decision": {
                        "action": "regather",
                        "checkpointStatus": "resumeReady",
                        "checkpointId": "ix-1",
                        "pressureLevel": "high",
                        "retrievalStatus": "stale",
                        "graphStatus": "ready",
                        "watcherStatus": "changed",
                        "reasons": [
                            "checkpointPathsChanged",
                            "frontierChanged"
                        ],
                        "checkpointDirtyPaths": ["src/lib.rs"],
                        "checkpointChangedPaths": ["src/lib.rs"],
                        "activeFrontierNodeIds": ["state-spine"],
                        "nextAction": "Re-gather source before editing.",
                        "note": "Resume-ready checkpoint was invalidated by watcher/frontier drift."
                    },
                    "revision": 5,
                    "changedFields": ["jobBindings"],
                    "epiphanyState": {
                        "revision": 5,
                        "job_bindings": [{
                            "id": "reorient-worker",
                            "kind": "specialist",
                            "scope": "reorient-guided checkpoint regather",
                            "owner_role": "epiphany-reorient",
                            "launcher_job_id": "epiphany-launch-1",
                            "authority_scope": "epiphany.reorient.regather",
                            "backend_job_id": "backend-1",
                            "linked_subgoal_ids": ["phase-6"],
                            "linked_graph_node_ids": ["state-spine"],
                            "progress_note": "Explicitly launched through the Epiphany authority surface onto the heartbeat backend."
                        }]
                    },
                    "job": {
                        "id": "reorient-worker",
                        "kind": "specialist",
                        "scope": "reorient-guided checkpoint regather",
                        "ownerRole": "epiphany-reorient",
                        "launcherJobId": "epiphany-launch-1",
                        "authorityScope": "epiphany.reorient.regather",
                        "backendJobId": "backend-1",
                        "status": "pending",
                        "progressNote": "Explicitly launched through the Epiphany authority surface onto the heartbeat backend.",
                        "linkedSubgoalIds": ["phase-6"],
                        "linkedGraphNodeIds": ["state-spine"]
                    }
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
    fn serialize_thread_epiphany_reorient_accept_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyReorientAccept {
            request_id: RequestId::Integer(10),
            response: v2::ThreadEpiphanyReorientAcceptResponse {
                revision: 6,
                changed_fields: vec![
                    v2::ThreadEpiphanyStateUpdatedField::AcceptanceReceipts,
                    v2::ThreadEpiphanyStateUpdatedField::Observations,
                    v2::ThreadEpiphanyStateUpdatedField::Evidence,
                    v2::ThreadEpiphanyStateUpdatedField::Scratch,
                ],
                epiphany_state: codex_protocol::protocol::EpiphanyThreadState {
                    revision: 6,
                    observations: vec![codex_protocol::protocol::EpiphanyObservation {
                        id: "obs-reorient-1".to_string(),
                        summary: "Accepted resume reorientation result.".to_string(),
                        source_kind: "reorient_result".to_string(),
                        status: "accepted".to_string(),
                        evidence_ids: vec!["ev-reorient-1".to_string()],
                        ..Default::default()
                    }],
                    recent_evidence: vec![codex_protocol::protocol::EpiphanyEvidenceRecord {
                        id: "ev-reorient-1".to_string(),
                        kind: "reorient_result".to_string(),
                        status: "accepted".to_string(),
                        summary: "Checkpoint remains source-grounded.".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                binding_id: "reorient-worker".to_string(),
                accepted_receipt_id: "accept-reorient-1".to_string(),
                accepted_observation_id: "obs-reorient-1".to_string(),
                accepted_evidence_id: "ev-reorient-1".to_string(),
                finding: v2::ThreadEpiphanyReorientFinding {
                    mode: Some("resume".to_string()),
                    summary: Some("Checkpoint remains source-grounded.".to_string()),
                    next_safe_move: Some("Continue the bounded slice.".to_string()),
                    checkpoint_still_valid: Some(true),
                    files_inspected: Vec::new(),
                    frontier_node_ids: Vec::new(),
                    evidence_ids: Vec::new(),
                    artifact_refs: Vec::new(),
                    runtime_result_id: Some("result-reorient-1".to_string()),
                    runtime_job_id: Some("backend-1".to_string()),
                    job_error: None,
                    item_error: None,
                },
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(10));
        assert_eq!(response.method(), "thread/epiphany/reorientAccept");
        assert_eq!(
            json!({
                "method": "thread/epiphany/reorientAccept",
                "id": 10,
                "response": {
                    "revision": 6,
                    "changedFields": ["acceptanceReceipts", "observations", "evidence", "scratch"],
                    "epiphanyState": {
                        "revision": 6,
                        "observations": [{
                            "id": "obs-reorient-1",
                            "summary": "Accepted resume reorientation result.",
                            "source_kind": "reorient_result",
                            "status": "accepted",
                            "evidence_ids": ["ev-reorient-1"]
                        }],
                        "recent_evidence": [{
                            "id": "ev-reorient-1",
                            "kind": "reorient_result",
                            "status": "accepted",
                            "summary": "Checkpoint remains source-grounded."
                        }]
                    },
                    "bindingId": "reorient-worker",
                    "acceptedReceiptId": "accept-reorient-1",
                    "acceptedObservationId": "obs-reorient-1",
                    "acceptedEvidenceId": "ev-reorient-1",
                    "finding": {
                        "mode": "resume",
                        "summary": "Checkpoint remains source-grounded.",
                        "nextSafeMove": "Continue the bounded slice.",
                        "checkpointStillValid": true,
                        "runtimeResultId": "result-reorient-1",
                        "runtimeJobId": "backend-1"
                    }
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_retrieve_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyRetrieve {
            request_id: RequestId::Integer(8),
            response: v2::ThreadEpiphanyRetrieveResponse {
                query: "checkpoint frontier".to_string(),
                index_summary: v2::ThreadEpiphanyRetrieveIndexSummary {
                    workspace_root: absolute_path("/workspace"),
                    index_revision: Some("query-time-bm25-v1".to_string()),
                    status: codex_protocol::protocol::EpiphanyRetrievalStatus::Ready,
                    semantic_available: true,
                    last_indexed_at_unix_seconds: Some(1_744_500_000),
                    indexed_file_count: Some(12),
                    indexed_chunk_count: Some(34),
                    shards: vec![v2::ThreadEpiphanyRetrieveShardSummary {
                        shard_id: "workspace".to_string(),
                        path_prefix: PathBuf::from("."),
                        indexed_file_count: Some(12),
                        indexed_chunk_count: Some(34),
                        status: codex_protocol::protocol::EpiphanyRetrievalStatus::Ready,
                        exact_available: true,
                        semantic_available: true,
                    }],
                    dirty_paths: vec![PathBuf::from("src/session/mod.rs")],
                },
                results: vec![v2::ThreadEpiphanyRetrieveResult {
                    kind: v2::ThreadEpiphanyRetrieveResultKind::SemanticChunk,
                    path: PathBuf::from("notes/design.md"),
                    score: 2.5,
                    line_start: Some(3),
                    line_end: Some(9),
                    excerpt: Some("checkpoint frontier".to_string()),
                }],
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(8));
        assert_eq!(response.method(), "thread/epiphany/retrieve");
        assert_eq!(
            json!({
                "method": "thread/epiphany/retrieve",
                "id": 8,
                "response": {
                    "query": "checkpoint frontier",
                    "indexSummary": {
                        "workspaceRoot": absolute_path_string("workspace"),
                        "indexRevision": "query-time-bm25-v1",
                        "status": "ready",
                        "semanticAvailable": true,
                        "lastIndexedAtUnixSeconds": 1744500000,
                        "indexedFileCount": 12,
                        "indexedChunkCount": 34,
                        "shards": [
                            {
                                "shardId": "workspace",
                                "pathPrefix": ".",
                                "indexedFileCount": 12,
                                "indexedChunkCount": 34,
                                "status": "ready",
                                "exactAvailable": true,
                                "semanticAvailable": true
                            }
                        ],
                        "dirtyPaths": ["src/session/mod.rs"]
                    },
                    "results": [
                        {
                            "kind": "semanticChunk",
                            "path": "notes/design.md",
                            "score": 2.5,
                            "lineStart": 3,
                            "lineEnd": 9,
                            "excerpt": "checkpoint frontier"
                        }
                    ]
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_index_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyIndex {
            request_id: RequestId::Integer(9),
            response: v2::ThreadEpiphanyIndexResponse {
                index_summary: v2::ThreadEpiphanyRetrieveIndexSummary {
                    workspace_root: absolute_path("/workspace"),
                    index_revision: Some("qdrant-ollama-v1:qwen3-embedding:0.6b".to_string()),
                    status: codex_protocol::protocol::EpiphanyRetrievalStatus::Ready,
                    semantic_available: true,
                    last_indexed_at_unix_seconds: Some(1_744_500_100),
                    indexed_file_count: Some(12),
                    indexed_chunk_count: Some(34),
                    shards: vec![v2::ThreadEpiphanyRetrieveShardSummary {
                        shard_id: "workspace".to_string(),
                        path_prefix: PathBuf::from("."),
                        indexed_file_count: Some(12),
                        indexed_chunk_count: Some(34),
                        status: codex_protocol::protocol::EpiphanyRetrievalStatus::Ready,
                        exact_available: true,
                        semantic_available: true,
                    }],
                    dirty_paths: Vec::new(),
                },
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(9));
        assert_eq!(response.method(), "thread/epiphany/index");
        assert_eq!(
            json!({
                "method": "thread/epiphany/index",
                "id": 9,
                "response": {
                    "indexSummary": {
                        "workspaceRoot": absolute_path_string("workspace"),
                        "indexRevision": "qdrant-ollama-v1:qwen3-embedding:0.6b",
                        "status": "ready",
                        "semanticAvailable": true,
                        "lastIndexedAtUnixSeconds": 1744500100,
                        "indexedFileCount": 12,
                        "indexedChunkCount": 34,
                        "shards": [
                            {
                                "shardId": "workspace",
                                "pathPrefix": ".",
                                "indexedFileCount": 12,
                                "indexedChunkCount": 34,
                                "status": "ready",
                                "exactAvailable": true,
                                "semanticAvailable": true
                            }
                        ],
                        "dirtyPaths": []
                    }
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_distill_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyDistill {
            request_id: RequestId::Integer(10),
            response: v2::ThreadEpiphanyDistillResponse {
                expected_revision: 7,
                patch: v2::ThreadEpiphanyUpdatePatch {
                    observations: vec![codex_protocol::protocol::EpiphanyObservation {
                        id: "obs-123".to_string(),
                        summary: "Smoke passed".to_string(),
                        source_kind: "smoke".to_string(),
                        status: "ok".to_string(),
                        code_refs: Vec::new(),
                        evidence_ids: vec!["ev-123".to_string()],
                    }],
                    evidence: vec![codex_protocol::protocol::EpiphanyEvidenceRecord {
                        id: "ev-123".to_string(),
                        kind: "verification".to_string(),
                        status: "ok".to_string(),
                        summary: "Smoke passed".to_string(),
                        code_refs: Vec::new(),
                    }],
                    ..Default::default()
                },
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(10));
        assert_eq!(response.method(), "thread/epiphany/distill");
        assert_eq!(
            json!({
                "method": "thread/epiphany/distill",
                "id": 10,
                "response": {
                    "expectedRevision": 7,
                    "patch": {
                        "observations": [
                            {
                                "id": "obs-123",
                                "summary": "Smoke passed",
                                "source_kind": "smoke",
                                "status": "ok",
                                "evidence_ids": ["ev-123"]
                            }
                        ],
                        "evidence": [
                            {
                                "id": "ev-123",
                                "kind": "verification",
                                "status": "ok",
                                "summary": "Smoke passed"
                            }
                        ]
                    }
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_propose_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyPropose {
            request_id: RequestId::Integer(12),
            response: v2::ThreadEpiphanyProposeResponse {
                expected_revision: 7,
                patch: v2::ThreadEpiphanyUpdatePatch {
                    churn: Some(codex_protocol::protocol::EpiphanyChurnState {
                        understanding_status: "proposal_ready".to_string(),
                        diff_pressure: "low".to_string(),
                        graph_freshness: Some("proposal".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(12));
        assert_eq!(response.method(), "thread/epiphany/propose");
        assert_eq!(
            json!({
                "method": "thread/epiphany/propose",
                "id": 12,
                "response": {
                    "expectedRevision": 7,
                    "patch": {
                        "churn": {
                            "understanding_status": "proposal_ready",
                            "diff_pressure": "low",
                            "graph_freshness": "proposal"
                        }
                    }
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_promote_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyPromote {
            request_id: RequestId::Integer(11),
            response: v2::ThreadEpiphanyPromoteResponse {
                accepted: false,
                reasons: vec!["verifierEvidence.status must be accepting".to_string()],
                revision: None,
                changed_fields: Vec::new(),
                epiphany_state: None,
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(11));
        assert_eq!(response.method(), "thread/epiphany/promote");
        assert_eq!(
            json!({
                "method": "thread/epiphany/promote",
                "id": 11,
                "response": {
                    "accepted": false,
                    "reasons": ["verifierEvidence.status must be accepting"],
                    "epiphanyState": null
                }
            }),
            serde_json::to_value(&response)?,
        );
        Ok(())
    }

    #[test]
    fn serialize_thread_epiphany_job_launch_response() -> Result<()> {
        let response = ClientResponse::ThreadEpiphanyJobLaunch {
            request_id: RequestId::Integer(12),
            response: v2::ThreadEpiphanyJobLaunchResponse {
                revision: 3,
                changed_fields: vec![
                    v2::ThreadEpiphanyStateUpdatedField::JobBindings,
                    v2::ThreadEpiphanyStateUpdatedField::RuntimeLinks,
                ],
                epiphany_state: codex_protocol::protocol::EpiphanyThreadState {
                    revision: 3,
                    job_bindings: vec![codex_protocol::protocol::EpiphanyJobBinding {
                        id: "specialist-work".to_string(),
                        kind: codex_protocol::protocol::EpiphanyJobKind::Specialist,
                        scope: "role-scoped specialist work".to_string(),
                        owner_role: "epiphany-harness".to_string(),
                        launcher_job_id: None,
                        authority_scope: Some("epiphany.specialist".to_string()),
                        backend_job_id: None,
                        linked_subgoal_ids: Vec::new(),
                        linked_graph_node_ids: Vec::new(),
                        progress_note: None,
                        blocking_reason: None,
                    }],
                    runtime_links: vec![codex_protocol::protocol::EpiphanyRuntimeLink {
                        id: "runtime-link-specialist-work-job-123".to_string(),
                        binding_id: "specialist-work".to_string(),
                        surface: "jobLaunch".to_string(),
                        role_id: "epiphany-harness".to_string(),
                        authority_scope: "epiphany.specialist".to_string(),
                        runtime_job_id: "job-123".to_string(),
                        runtime_result_id: None,
                        linked_subgoal_ids: Vec::new(),
                        linked_graph_node_ids: Vec::new(),
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
                    backend_job_id: Some("job-123".to_string()),
                    status: v2::ThreadEpiphanyJobStatus::Pending,
                    items_processed: None,
                    items_total: None,
                    progress_note: Some("Runtime agent job is pending.".to_string()),
                    last_checkpoint_at_unix_seconds: None,
                    blocking_reason: None,
                    active_thread_ids: Vec::new(),
                    linked_subgoal_ids: Vec::new(),
                    linked_graph_node_ids: Vec::new(),
                },
            },
        };

        assert_eq!(response.id(), &RequestId::Integer(12));
        assert_eq!(response.method(), "thread/epiphany/jobLaunch");
        assert_eq!(
            json!({
                "method": "thread/epiphany/jobLaunch",
                "id": 12,
                "response": {
                    "revision": 3,
                    "changedFields": ["jobBindings", "runtimeLinks"],
                    "epiphanyState": {
                        "revision": 3,
                        "job_bindings": [{
                            "id": "specialist-work",
                            "kind": "specialist",
                            "scope": "role-scoped specialist work",
                            "owner_role": "epiphany-harness",
                            "authority_scope": "epiphany.specialist"
                        }],
                        "runtime_links": [{
                            "id": "runtime-link-specialist-work-job-123",
                            "binding_id": "specialist-work",
                            "surface": "jobLaunch",
                            "role_id": "epiphany-harness",
                            "authority_scope": "epiphany.specialist",
                            "runtime_job_id": "job-123"
                        }]
                    },
                    "job": {
                        "id": "specialist-work",
                        "kind": "specialist",
                        "scope": "role-scoped specialist work",
                        "ownerRole": "epiphany-harness",
                        "authorityScope": "epiphany.specialist",
                        "backendJobId": "job-123",
                        "status": "pending",
                        "progressNote": "Runtime agent job is pending."
                    }
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
                epiphany_state: codex_protocol::protocol::EpiphanyThreadState {
                    revision: 4,
                    job_bindings: vec![codex_protocol::protocol::EpiphanyJobBinding {
                        id: "specialist-work".to_string(),
                        kind: codex_protocol::protocol::EpiphanyJobKind::Specialist,
                        scope: "role-scoped specialist work".to_string(),
                        owner_role: "epiphany-harness".to_string(),
                        launcher_job_id: None,
                        authority_scope: Some("epiphany.specialist".to_string()),
                        backend_job_id: None,
                        linked_subgoal_ids: Vec::new(),
                        linked_graph_node_ids: Vec::new(),
                        progress_note: None,
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
                epiphany_state: codex_protocol::protocol::EpiphanyThreadState {
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
                epiphany_state: codex_protocol::protocol::EpiphanyThreadState {
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
                epiphany_state: codex_protocol::protocol::EpiphanyThreadState {
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
    fn serialize_list_apps() -> Result<()> {
        let request = ClientRequest::AppsList {
            request_id: RequestId::Integer(8),
            params: v2::AppsListParams::default(),
        };
        assert_eq!(
            json!({
                "method": "app/list",
                "id": 8,
                "params": {
                    "cursor": null,
                    "limit": null,
                    "threadId": null
                }
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
    fn thread_epiphany_retrieve_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyRetrieve {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyRetrieveParams {
                thread_id: "thr_123".to_string(),
                query: "checkpoint frontier".to_string(),
                limit: Some(5),
                path_prefixes: vec![PathBuf::from("notes")],
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/retrieve"));
    }

    #[test]
    fn thread_epiphany_scene_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyScene {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanySceneParams {
                thread_id: "thr_123".to_string(),
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/scene"));
    }

    #[test]
    fn thread_epiphany_jobs_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyJobs {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyJobsParams {
                thread_id: "thr_123".to_string(),
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/jobs"));
    }

    #[test]
    fn thread_epiphany_roles_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyRoles {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyRolesParams {
                thread_id: "thr_123".to_string(),
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/roles"));
    }

    #[test]
    fn thread_epiphany_role_launch_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyRoleLaunch {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyRoleLaunchParams {
                thread_id: "thr_123".to_string(),
                role_id: v2::ThreadEpiphanyRoleId::Modeling,
                expected_revision: Some(1),
                max_runtime_seconds: Some(60),
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/roleLaunch"));
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
    fn thread_epiphany_role_accept_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyRoleAccept {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyRoleAcceptParams {
                thread_id: "thr_123".to_string(),
                role_id: v2::ThreadEpiphanyRoleId::Modeling,
                expected_revision: Some(2),
                binding_id: None,
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/roleAccept"));
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
    fn thread_epiphany_planning_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyPlanning {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyPlanningParams {
                thread_id: "thr_123".to_string(),
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/planning"));
    }

    #[test]
    fn thread_epiphany_reorient_launch_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyReorientLaunch {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyReorientLaunchParams {
                thread_id: "thr_123".to_string(),
                expected_revision: Some(4),
                max_runtime_seconds: Some(90),
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/reorientLaunch"));
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
    fn thread_epiphany_reorient_accept_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyReorientAccept {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyReorientAcceptParams {
                thread_id: "thr_123".to_string(),
                expected_revision: Some(5),
                binding_id: Some("reorient-worker".to_string()),
                update_scratch: true,
                update_investigation_checkpoint: false,
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/reorientAccept"));
    }

    #[test]
    fn thread_epiphany_index_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyIndex {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyIndexParams {
                thread_id: "thr_123".to_string(),
                force_full_rebuild: true,
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/index"));
    }

    #[test]
    fn thread_epiphany_distill_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyDistill {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyDistillParams {
                thread_id: "thr_123".to_string(),
                source_kind: "smoke".to_string(),
                status: "ok".to_string(),
                text: "smoke passed".to_string(),
                subject: Some("thread/epiphany/distill".to_string()),
                evidence_kind: None,
                code_refs: Vec::new(),
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/distill"));
    }

    #[test]
    fn thread_epiphany_propose_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyPropose {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyProposeParams {
                thread_id: "thr_123".to_string(),
                observation_ids: vec!["obs-123".to_string()],
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/propose"));
    }

    #[test]
    fn thread_epiphany_promote_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyPromote {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyPromoteParams {
                thread_id: "thr_123".to_string(),
                expected_revision: Some(1),
                patch: v2::ThreadEpiphanyUpdatePatch {
                    objective: Some("Keep the map honest".to_string()),
                    ..Default::default()
                },
                verifier_evidence: codex_protocol::protocol::EpiphanyEvidenceRecord {
                    id: "ev-verifier".to_string(),
                    kind: "verification".to_string(),
                    status: "ok".to_string(),
                    summary: "Verifier accepted promotion".to_string(),
                    code_refs: Vec::new(),
                },
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/promote"));
    }

    #[test]
    fn thread_epiphany_job_launch_is_marked_experimental() {
        let request = ClientRequest::ThreadEpiphanyJobLaunch {
            request_id: RequestId::Integer(1),
            params: v2::ThreadEpiphanyJobLaunchParams {
                thread_id: "thr_123".to_string(),
                expected_revision: Some(2),
                binding_id: "specialist-work".to_string(),
                kind: codex_protocol::protocol::EpiphanyJobKind::Specialist,
                scope: "role-scoped specialist work".to_string(),
                owner_role: "epiphany-harness".to_string(),
                authority_scope: "epiphany.specialist".to_string(),
                linked_subgoal_ids: vec!["phase6".to_string()],
                linked_graph_node_ids: vec!["job-surface".to_string()],
                instruction: "Summarize the bound specialist task.".to_string(),
                input_json: json!({"task": "smoke"}),
                output_schema_json: None,
                max_runtime_seconds: Some(30),
            },
        };
        let reason = crate::experimental_api::ExperimentalApi::experimental_reason(&request);
        assert_eq!(reason, Some("thread/epiphany/jobLaunch"));
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
                epiphany_state: codex_protocol::protocol::EpiphanyThreadState::default(),
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
