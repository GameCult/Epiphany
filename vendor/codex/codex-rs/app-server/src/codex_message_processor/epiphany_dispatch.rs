use codex_app_server_protocol::ClientRequest;

use super::CodexMessageProcessor;
use super::ConnectionId;
use super::ConnectionRequestId;

impl CodexMessageProcessor {
    pub(super) async fn maybe_handle_epiphany_request(
        &self,
        connection_id: ConnectionId,
        request: ClientRequest,
    ) -> Option<ClientRequest> {
        let to_connection_request_id = |request_id| ConnectionRequestId {
            connection_id,
            request_id,
        };

        match request {
            ClientRequest::ThreadEpiphanyView { request_id, params } => {
                self.thread_epiphany_view(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyRoleLaunch { request_id, params } => {
                self.thread_epiphany_role_launch(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyRoleResult { request_id, params } => {
                self.thread_epiphany_role_result(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyRoleAccept { request_id, params } => {
                self.thread_epiphany_role_accept(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyFreshness { request_id, params } => {
                self.thread_epiphany_freshness(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyContext { request_id, params } => {
                self.thread_epiphany_context(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyGraphQuery { request_id, params } => {
                self.thread_epiphany_graph_query(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyReorientLaunch { request_id, params } => {
                self.thread_epiphany_reorient_launch(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyReorientResult { request_id, params } => {
                self.thread_epiphany_reorient_result(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyReorientAccept { request_id, params } => {
                self.thread_epiphany_reorient_accept(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyIndex { request_id, params } => {
                self.thread_epiphany_index(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyDistill { request_id, params } => {
                self.thread_epiphany_distill(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyPropose { request_id, params } => {
                self.thread_epiphany_propose(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyPromote { request_id, params } => {
                self.thread_epiphany_promote(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyJobLaunch { request_id, params } => {
                self.thread_epiphany_job_launch(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyJobInterrupt { request_id, params } => {
                self.thread_epiphany_job_interrupt(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyUpdate { request_id, params } => {
                self.thread_epiphany_update(to_connection_request_id(request_id), params)
                    .await;
            }
            ClientRequest::ThreadEpiphanyRetrieve { request_id, params } => {
                self.thread_epiphany_retrieve(to_connection_request_id(request_id), params)
                    .await;
            }
            other => return Some(other),
        }

        None
    }
}
