//! gRPC subsystem: tonic transport for the workload-identity services.
//!
//! Serves `WorkloadAuthorizer` + `WorkloadTokenValidator` from
//! `identity-workload-rest::grpc` (the shared delivery core — both RPC
//! services delegate to the same use-cases as the REST surface, so the two
//! protocols can never drift).

use std::future::Future;
use std::sync::Arc;

use tonic::transport::Server;
use tonic::transport::server::TcpIncoming;

use iam_identity_workload_app::{RevocationDenylist, WorkloadPrincipalRepository};
use iam_identity_workload_authz_cedar::WorkloadAuthorizer;
use iam_identity_workload_rest::grpc::{
    WorkloadAuthorizerServer, WorkloadGrpcServer, WorkloadTokenValidatorServer,
};
use iam_identity_workload_rest::{AuditSink, WorkloadAuthzState};

/// Serve both workload-identity gRPC services on `incoming` until `shutdown`
/// resolves (graceful drain).
///
/// # Errors
/// Returns the tonic transport error when serving fails.
pub async fn serve<R, D, A, S, F>(
    state: Arc<WorkloadAuthzState<R, D, A, S>>,
    incoming: TcpIncoming,
    shutdown: F,
) -> Result<(), tonic::transport::Error>
where
    R: WorkloadPrincipalRepository + Send + Sync + 'static,
    D: RevocationDenylist + Send + Sync + 'static,
    A: WorkloadAuthorizer + Send + Sync + 'static,
    S: AuditSink + 'static,
    F: Future<Output = ()> + Send + 'static,
{
    Server::builder()
        .add_service(WorkloadAuthorizerServer::new(WorkloadGrpcServer::new(
            Arc::clone(&state),
        )))
        .add_service(WorkloadTokenValidatorServer::new(WorkloadGrpcServer::new(
            state,
        )))
        .serve_with_incoming_shutdown(incoming, shutdown)
        .await
}
