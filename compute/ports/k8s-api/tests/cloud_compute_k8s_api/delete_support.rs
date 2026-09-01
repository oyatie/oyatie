fn delete_boundary_for(
    request_id: &str,
    idempotency_key: &str,
) -> CloudComputeK8sApiBoundaryContext {
    CloudComputeK8sApiBoundaryContext {
        request_id: request_id.to_string(),
        tenant_id: "ten_alpha".to_string(),
        idempotency_key: idempotency_key.to_string(),
    }
}

fn delete_authorization_for(
    principal_id: &str,
    surfaces: &[&str],
) -> CloudComputeK8sApiAuthorization {
    let decision_id = format!("authz_del_{principal_id}");
    CloudComputeK8sApiAuthorization {
        tenant_id: "ten_alpha".to_string(),
        principal_id: principal_id.to_string(),
        decision_id: decision_id.clone(),
        allowed_surfaces: surfaces
            .iter()
            .map(|surface| (*surface).to_string())
            .collect(),
        proof: Some(authorization_proof_for(
            principal_id,
            CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
            &decision_id,
        )),
    }
}

fn delete_request(
    request_id: &str,
    idempotency_key: &str,
) -> CloudComputeK8sClusterDeleteApiRequest {
    CloudComputeK8sClusterDeleteApiRequest {
        path_cluster_id: CLUSTER_ID.to_string(),
        boundary: delete_boundary_for(request_id, idempotency_key),
        principal: principal_for("sp_compute"),
        authorization: delete_authorization_for(
            "sp_compute",
            &[CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE],
        ),
    }
}

fn trusted_delete_verifier_for(
    request: &CloudComputeK8sClusterDeleteApiRequest,
) -> CloudComputeK8sTrustedAuthorizationVerifier {
    CloudComputeK8sTrustedAuthorizationVerifier::new(K8S_AUTHZ_EVALUATION_EPOCH_SECONDS)
        .with_authorization_proof(authorization_proof_for(
            &request.principal.principal_id,
            CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE,
            &request.authorization.decision_id,
        ))
}

async fn delete_cloud_compute_k8s_cluster_from_api(
    repository: &LifecycleTestRepository,
    request: CloudComputeK8sClusterDeleteApiRequest,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    let verifier = trusted_delete_verifier_for(&request);
    delete_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        repository,
        request,
        &verifier,
    )
    .await
}

async fn delete_cluster(
    repository: &LifecycleTestRepository,
    request: CloudComputeK8sClusterDeleteApiRequest,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    let verifier = trusted_delete_verifier_for(&request);
    delete_cluster_with_authorization_verifier(repository, request, &verifier).await
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LifecycleTestClusterEntry {
    desired_spec: CloudComputeK8sClusterCreateRequest,
    record: CloudComputeK8sClusterRecord,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum LifecycleTestOperationEntry {
    Create {
        fingerprint: String,
        receipt: CloudComputeK8sCreateReceipt,
    },
    Delete {
        resource_id: compute_resource::ResourceId,
        receipt: CloudComputeK8sDeleteReceipt,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct LifecycleTestRepositoryState {
    clusters: BTreeMap<String, LifecycleTestClusterEntry>,
    operations: BTreeMap<CloudComputeK8sOperationKey, LifecycleTestOperationEntry>,
}

#[derive(Clone, Debug, Default)]
struct LifecycleTestRepositoryControl {
    fail_next_commit: bool,
    next_create_receipt_override: Option<CloudComputeK8sCreateReceipt>,
    next_delete_receipt_override: Option<CloudComputeK8sDeleteReceipt>,
}

#[derive(Clone, Debug, Default)]
struct LifecycleTestRepository {
    state: Arc<Mutex<LifecycleTestRepositoryState>>,
    control: Arc<Mutex<LifecycleTestRepositoryControl>>,
}

impl LifecycleTestRepository {
    fn snapshot(&self) -> LifecycleTestRepositoryState {
        self.state.lock().expect("repository lock is healthy").clone()
    }

    fn from_restored_snapshot(snapshot: LifecycleTestRepositoryState) -> Self {
        Self {
            state: Arc::new(Mutex::new(snapshot)),
            control: Arc::new(Mutex::new(LifecycleTestRepositoryControl::default())),
        }
    }

    fn fail_next_commit(&self) {
        self.control
            .lock()
            .expect("repository control lock is healthy")
            .fail_next_commit = true;
    }

    fn return_next_create_receipt(&self, receipt: CloudComputeK8sCreateReceipt) {
        self.control
            .lock()
            .expect("repository control lock is healthy")
            .next_create_receipt_override = Some(receipt);
    }

    fn return_next_delete_receipt(&self, receipt: CloudComputeK8sDeleteReceipt) {
        self.control
            .lock()
            .expect("repository control lock is healthy")
            .next_delete_receipt_override = Some(receipt);
    }

    fn take_create_receipt_override(&self) -> Option<CloudComputeK8sCreateReceipt> {
        self.control
            .lock()
            .expect("repository control lock is healthy")
            .next_create_receipt_override
            .take()
    }

    fn take_delete_receipt_override(&self) -> Option<CloudComputeK8sDeleteReceipt> {
        self.control
            .lock()
            .expect("repository control lock is healthy")
            .next_delete_receipt_override
            .take()
    }

    fn should_fail_commit(&self) -> bool {
        let mut control = self
            .control
            .lock()
            .expect("repository control lock is healthy");
        let should_fail = control.fail_next_commit;
        control.fail_next_commit = false;
        should_fail
    }

    fn cluster_count(&self) -> usize {
        self.state
            .lock()
            .expect("repository lock is healthy")
            .clusters
            .len()
    }

    fn create_operation_count(&self) -> usize {
        self.operation_count(|entry| matches!(entry, LifecycleTestOperationEntry::Create { .. }))
    }

    fn delete_operation_count(&self) -> usize {
        self.operation_count(|entry| matches!(entry, LifecycleTestOperationEntry::Delete { .. }))
    }

    fn operation_count(&self, matches_entry: impl Fn(&LifecycleTestOperationEntry) -> bool) -> usize {
        self.state
            .lock()
            .expect("repository lock is healthy")
            .operations
            .values()
            .filter(|entry| matches_entry(entry))
            .count()
    }

    fn cluster_record(&self, resource_id: &str) -> Option<CloudComputeK8sClusterRecord> {
        self.state
            .lock()
            .expect("repository lock is healthy")
            .clusters
            .get(resource_id)
            .map(|entry| entry.record.clone())
    }
}

impl CloudComputeK8sLifecycleRepository for LifecycleTestRepository {
    fn commit_create<'a>(
        &'a self,
        command: CloudComputeK8sCreateCommand,
    ) -> CloudComputeK8sRepositoryFuture<
        'a,
        Result<CloudComputeK8sCreateReceipt, CloudComputeK8sLifecycleRepositoryError>,
    > {
        Box::pin(async move {
            if let Some(receipt) = self.take_create_receipt_override() {
                return Ok(receipt);
            }

            let mut current = self
                .state
                .lock()
                .map_err(|_| CloudComputeK8sLifecycleRepositoryError::Unavailable)?;
            if let Some(entry) = current.operations.get(&command.operation_key) {
                return match entry {
                    LifecycleTestOperationEntry::Create {
                        fingerprint,
                        receipt,
                    } if fingerprint == &command.fingerprint => Ok(receipt.clone()),
                    LifecycleTestOperationEntry::Create { .. } => {
                        Err(CloudComputeK8sLifecycleRepositoryError::IdempotencyKeyReused {
                            idempotency_key: command.operation_key.idempotency_key,
                        })
                    }
                    LifecycleTestOperationEntry::Delete { .. } => {
                        Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation)
                    }
                };
            }
            if command.cluster.resource_id != command.desired_spec.resource_id
                || command.cluster.tenant_id != command.desired_spec.tenant_id
                || command.cluster.tenant_id != command.operation_key.tenant_id
                || command.cluster.desired_state != "present"
                || command.operation_key.surface != CLOUD_COMPUTE_K8S_CLUSTER_CREATE_SURFACE
            {
                return Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation);
            }

            let mut next = current.clone();
            if next.clusters.contains_key(&command.cluster.resource_id) {
                return Err(CloudComputeK8sLifecycleRepositoryError::ClusterAlreadyExists);
            }
            let receipt = CloudComputeK8sCreateReceipt {
                cluster: command.cluster.clone(),
                request_id: command.request_id,
            };
            next.clusters.insert(
                command.cluster.resource_id.clone(),
                LifecycleTestClusterEntry {
                    desired_spec: command.desired_spec,
                    record: command.cluster,
                },
            );
            next.operations.insert(
                command.operation_key,
                LifecycleTestOperationEntry::Create {
                    fingerprint: command.fingerprint,
                    receipt: receipt.clone(),
                },
            );
            if self.should_fail_commit() {
                return Err(CloudComputeK8sLifecycleRepositoryError::Unavailable);
            }
            *current = next;
            Ok(receipt)
        })
    }

    fn commit_deletion<'a>(
        &'a self,
        command: CloudComputeK8sDeleteCommand,
    ) -> CloudComputeK8sRepositoryFuture<
        'a,
        Result<CloudComputeK8sDeleteReceipt, CloudComputeK8sLifecycleRepositoryError>,
    > {
        Box::pin(async move {
            if let Some(receipt) = self.take_delete_receipt_override() {
                return Ok(receipt);
            }

            let mut current = self
                .state
                .lock()
                .map_err(|_| CloudComputeK8sLifecycleRepositoryError::Unavailable)?;
            if let Some(entry) = current.operations.get(&command.operation_key) {
                return match entry {
                    LifecycleTestOperationEntry::Delete {
                        resource_id,
                        receipt,
                    } if resource_id == &command.resource_id => Ok(receipt.clone()),
                    LifecycleTestOperationEntry::Delete { .. } => {
                        Err(CloudComputeK8sLifecycleRepositoryError::IdempotencyKeyReused {
                            idempotency_key: command.operation_key.idempotency_key,
                        })
                    }
                    LifecycleTestOperationEntry::Create { .. } => {
                        Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation)
                    }
                };
            }
            if command.operation_key.surface != CLOUD_COMPUTE_K8S_CLUSTER_DELETE_SURFACE {
                return Err(CloudComputeK8sLifecycleRepositoryError::IntegrityViolation);
            }

            let mut next = current.clone();
            let cluster = next
                .clusters
                .get_mut(&command.resource_id.value)
                .ok_or(CloudComputeK8sLifecycleRepositoryError::ClusterNotFound)?;
            if cluster.record.tenant_id != command.operation_key.tenant_id {
                return Err(CloudComputeK8sLifecycleRepositoryError::ClusterNotFound);
            }
            cluster.record.desired_state = "deleted".to_string();
            let receipt = CloudComputeK8sDeleteReceipt {
                cluster: cluster.record.clone(),
                request_id: command.request_id,
            };
            next.operations.insert(
                command.operation_key,
                LifecycleTestOperationEntry::Delete {
                    resource_id: command.resource_id,
                    receipt: receipt.clone(),
                },
            );
            if self.should_fail_commit() {
                return Err(CloudComputeK8sLifecycleRepositoryError::Unavailable);
            }
            *current = next;
            Ok(receipt)
        })
    }
}

async fn delete_repository_with_cluster() -> LifecycleTestRepository {
    let repository = LifecycleTestRepository::default();
    create_cloud_compute_k8s_cluster_from_api(
        &repository,
        request("req-setup-delete", "idem-setup-delete"),
    )
    .await
    .expect("setup cluster create succeeds");
    repository
}

fn stored_cluster_lifecycle(repository: &LifecycleTestRepository) -> (String, String) {
    let cluster = repository
        .cluster_record(CLUSTER_ID)
        .expect("setup cluster remains in the repository");
    (cluster.state, cluster.desired_state)
}
