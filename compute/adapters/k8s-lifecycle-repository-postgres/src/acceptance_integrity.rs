use compute_k8s_api::{
    CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE, CloudComputeK8sAcceptanceRepositoryError as Error,
    CloudComputeK8sAcceptedCreateIntent, CloudComputeK8sOperationKey,
    CloudComputeK8sOperationSnapshot, cloud_compute_k8s_create_intent_fingerprint,
    validate_cloud_compute_k8s_operation_snapshot,
};
use shared_resource_provider_contract_kernel::OperationState;
use sqlx::{Row, postgres::PgRow};

pub(crate) fn validate_key(key: &CloudComputeK8sOperationKey) -> Result<(), Error> {
    if key.tenant_id.trim().is_empty()
        || key.principal_id.trim().is_empty()
        || key.idempotency_key.trim().is_empty()
        || key.surface != CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE
    {
        return Err(Error::IntegrityViolation);
    }
    Ok(())
}

pub(crate) fn verified_snapshot(
    row: &PgRow,
    key: &CloudComputeK8sOperationKey,
    expected_resource: Option<&str>,
) -> Result<CloudComputeK8sOperationSnapshot, Error> {
    let contract: String = row.try_get("request_contract").map_err(integrity)?;
    let state: Option<String> = row.try_get("operation_state").map_err(integrity)?;
    match (contract.as_str(), state.as_deref()) {
        ("trusted_envelope", None) => return Err(Error::OperationContractMismatch),
        ("pending_intent", Some("accepted")) => {}
        _ => return Err(Error::IntegrityViolation),
    }
    let resource: String = row.try_get("resource_id").map_err(integrity)?;
    if expected_resource.is_some_and(|expected| expected != resource) {
        return Err(Error::ResourceMismatch);
    }
    let schema: i32 = row.try_get("schema_version").map_err(integrity)?;
    let kind: Option<String> = row.try_get("receipt_kind").map_err(integrity)?;
    let complete: bool = row.try_get("receipt_complete").map_err(integrity)?;
    if schema != crate::SCHEMA_VERSION || kind.as_deref() != Some("create") || !complete {
        return Err(Error::IntegrityViolation);
    }
    let json: Option<serde_json::Value> = row.try_get("receipt_json").map_err(integrity)?;
    let digest: Option<String> = row.try_get("receipt_digest").map_err(integrity)?;
    let json = json.ok_or(Error::IntegrityViolation)?;
    if Some(crate::canonical_json::json_digest(&json).map_err(integrity)?) != digest {
        return Err(Error::IntegrityViolation);
    }
    let receipt: CloudComputeK8sAcceptedCreateIntent =
        serde_json::from_value(json).map_err(integrity)?;
    let accepted_at: i64 = row
        .try_get("accepted_at_epoch_seconds")
        .map_err(integrity)?;
    if u64::try_from(accepted_at).map_err(integrity)? != receipt.accepted_at_epoch_seconds {
        return Err(Error::IntegrityViolation);
    }
    let fingerprint: String = row.try_get("request_fingerprint").map_err(integrity)?;
    if cloud_compute_k8s_create_intent_fingerprint(&receipt.intent).map_err(integrity)?
        != fingerprint
    {
        return Err(Error::IntegrityViolation);
    }
    let snapshot = CloudComputeK8sOperationSnapshot {
        receipt,
        state: OperationState::Accepted,
    };
    validate_cloud_compute_k8s_operation_snapshot(&snapshot, key, &resource, None)
        .map_err(integrity)?;
    Ok(snapshot)
}

fn integrity<T>(_: T) -> Error {
    Error::IntegrityViolation
}
