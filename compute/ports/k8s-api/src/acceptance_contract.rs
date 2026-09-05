pub const CLOUD_COMPUTE_K8S_OPERATION_GET_SURFACE: &str = "cloud.compute.k8s.operation.get";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CloudComputeK8sAcceptanceContract {
    PendingIntent,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CloudComputeK8sAcceptedCreateIntent {
    pub request_contract: CloudComputeK8sAcceptanceContract,
    pub operation_key: CloudComputeK8sOperationKey,
    pub intent: CloudComputeK8sClusterCreateIntent,
    pub request_id: String,
    pub accepted_at_epoch_seconds: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CloudComputeK8sOperationSnapshot {
    pub receipt: CloudComputeK8sAcceptedCreateIntent,
    pub state: OperationState,
}
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum CloudComputeK8sOperationLookup {
    Found(CloudComputeK8sOperationSnapshot),
    NotObserved,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sAcceptCreateIntentCommand {
    pub operation_key: CloudComputeK8sOperationKey,
    pub intent: CloudComputeK8sClusterCreateIntent,
    pub request_id: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sReadCreateOperationQuery {
    pub operation_key: CloudComputeK8sOperationKey,
    pub resource_id: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CloudComputeK8sAcceptanceRepositoryError {
    IdempotencyKeyReused,
    OperationContractMismatch,
    ResourceMismatch,
    Unavailable,
    OutcomeUnknown,
    IntegrityViolation,
}
pub trait CloudComputeK8sAcceptanceRepository: Send + Sync {
    fn accept_create_intent<'a>(
        &'a self,
        command: CloudComputeK8sAcceptCreateIntentCommand,
    ) -> CloudComputeK8sRepositoryFuture<'a, Result<CloudComputeK8sOperationSnapshot, CloudComputeK8sAcceptanceRepositoryError>>;
    fn get_create_operation<'a>(
        &'a self,
        query: CloudComputeK8sReadCreateOperationQuery,
    ) -> CloudComputeK8sRepositoryFuture<'a, Result<CloudComputeK8sOperationLookup, CloudComputeK8sAcceptanceRepositoryError>>;
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sCreateAcceptanceApiRequest {
    pub path_cluster_id: String,
    pub boundary: CloudComputeK8sApiBoundaryContext,
    pub principal: CloudComputeK8sApiPrincipal,
    pub authorization: CloudComputeK8sApiAuthorization,
    pub body: CloudComputeK8sClusterCreateIntent,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sOperationReadApiRequest {
    pub path_cluster_id: String,
    pub boundary: CloudComputeK8sApiBoundaryContext,
    pub principal: CloudComputeK8sApiPrincipal,
    pub authorization: CloudComputeK8sApiAuthorization,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CloudComputeK8sCreateAcceptanceResponse {
    pub operation: CloudComputeK8sOperationSnapshot,
}
impl CloudComputeK8sCreateAcceptanceResponse {
    pub const fn status_code(&self) -> u16 {
        202
    }
}
