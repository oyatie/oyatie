pub fn validate_cloud_compute_k8s_operation_snapshot(
    snapshot: &CloudComputeK8sOperationSnapshot,
    expected_key: &CloudComputeK8sOperationKey,
    expected_resource_id: &str,
    expected_intent: Option<&CloudComputeK8sClusterCreateIntent>,
) -> Result<(), CloudComputeK8sAcceptanceRepositoryError> {
    let receipt = &snapshot.receipt;
    if receipt.request_contract != CloudComputeK8sAcceptanceContract::PendingIntent
        || receipt.operation_key.surface != CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE
    {
        return Err(CloudComputeK8sAcceptanceRepositoryError::OperationContractMismatch);
    }
    if receipt.operation_key != *expected_key {
        return Err(CloudComputeK8sAcceptanceRepositoryError::IntegrityViolation);
    }
    if receipt.intent.resource_id != expected_resource_id
        || receipt.intent.tenant_id != receipt.operation_key.tenant_id
    {
        return Err(CloudComputeK8sAcceptanceRepositoryError::ResourceMismatch);
    }
    if snapshot.state != OperationState::Accepted
        || receipt.request_id.trim().is_empty()
        || receipt.accepted_at_epoch_seconds == 0
    {
        return Err(CloudComputeK8sAcceptanceRepositoryError::IntegrityViolation);
    }
    validate_cloud_compute_k8s_create_intent(&receipt.intent)
        .map_err(|_| CloudComputeK8sAcceptanceRepositoryError::IntegrityViolation)?;
    if let Some(expected) = expected_intent {
        let stored = cloud_compute_k8s_create_intent_fingerprint(&receipt.intent)
            .map_err(|_| CloudComputeK8sAcceptanceRepositoryError::IntegrityViolation)?;
        let expected = cloud_compute_k8s_create_intent_fingerprint(expected)
            .map_err(|_| CloudComputeK8sAcceptanceRepositoryError::IntegrityViolation)?;
        if stored != expected {
            return Err(CloudComputeK8sAcceptanceRepositoryError::IdempotencyKeyReused);
        }
    }
    Ok(())
}
