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
        allowed_surfaces: surfaces.iter().map(|s| (*s).to_string()).collect(),
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

fn delete_cloud_compute_k8s_cluster_from_api(
    repository: &mut DeleteTestRepository,
    request: CloudComputeK8sClusterDeleteApiRequest,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    let verifier = trusted_delete_verifier_for(&request);
    delete_cloud_compute_k8s_cluster_from_api_with_authorization_verifier(
        repository,
        request,
        &verifier,
    )
}

fn delete_cluster(
    repository: &mut DeleteTestRepository,
    request: CloudComputeK8sClusterDeleteApiRequest,
) -> Result<CloudComputeK8sClusterDeleteSuccessResponse, CloudComputeK8sApiError> {
    let verifier = trusted_delete_verifier_for(&request);
    delete_cluster_with_authorization_verifier(repository, request, &verifier)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DeleteTestRepositoryEntry {
    resource_id: compute_resource::ResourceId,
    receipt: CloudComputeK8sDeleteReceipt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DeleteTestRepositoryState {
    catalog: CloudComputeCatalog,
    entries: BTreeMap<CloudComputeK8sDeleteOperationKey, DeleteTestRepositoryEntry>,
}

#[derive(Clone, Debug)]
struct DeleteTestRepository {
    state: Arc<Mutex<DeleteTestRepositoryState>>,
    fail_next_commit: bool,
    next_receipt_override: Option<CloudComputeK8sDeleteReceipt>,
}

impl DeleteTestRepository {
    fn new(catalog: CloudComputeCatalog) -> Self {
        Self {
            state: Arc::new(Mutex::new(DeleteTestRepositoryState {
                catalog,
                entries: BTreeMap::new(),
            })),
            fail_next_commit: false,
            next_receipt_override: None,
        }
    }

    fn entry_count(&self) -> usize {
        self.state.lock().expect("repository lock is healthy").entries.len()
    }

    fn is_empty(&self) -> bool {
        self.entry_count() == 0
    }

    fn catalog_snapshot(&self) -> CloudComputeCatalog {
        self.state
            .lock()
            .expect("repository lock is healthy")
            .catalog
            .clone()
    }

    fn snapshot(&self) -> DeleteTestRepositoryState {
        self.state.lock().expect("repository lock is healthy").clone()
    }

    fn from_restored_snapshot(snapshot: DeleteTestRepositoryState) -> Self {
        Self {
            state: Arc::new(Mutex::new(snapshot)),
            fail_next_commit: false,
            next_receipt_override: None,
        }
    }

    fn fail_next_commit(&mut self) {
        self.fail_next_commit = true;
    }

    fn return_next_receipt(&mut self, receipt: CloudComputeK8sDeleteReceipt) {
        self.next_receipt_override = Some(receipt);
    }
}

impl CloudComputeK8sDeleteRepository for DeleteTestRepository {
    fn commit_deletion(
        &mut self,
        command: CloudComputeK8sDeleteCommand,
    ) -> Result<CloudComputeK8sDeleteReceipt, CloudComputeK8sDeleteRepositoryError> {
        if let Some(receipt) = self.next_receipt_override.take() {
            return Ok(receipt);
        }
        let mut current = self
            .state
            .lock()
            .map_err(|_| CloudComputeK8sDeleteRepositoryError::Unavailable)?;
        if let Some(entry) = current.entries.get(&command.operation_key) {
            if entry.resource_id == command.resource_id {
                return Ok(entry.receipt.clone());
            }
            return Err(CloudComputeK8sDeleteRepositoryError::IdempotencyKeyReused {
                idempotency_key: command.operation_key.idempotency_key,
            });
        }

        let mut next = current.clone();
        let cluster = next
            .catalog
            .request_kubernetes_cluster_deletion(&command.resource_id)
            .map_err(|error| match error {
                KubernetesClusterMutationError::UnknownCluster => {
                    CloudComputeK8sDeleteRepositoryError::ClusterNotFound
                }
            })?;
        let receipt = CloudComputeK8sDeleteReceipt {
            cluster,
            request_id: command.request_id,
        };
        next.entries.insert(
            command.operation_key,
            DeleteTestRepositoryEntry {
                resource_id: command.resource_id,
                receipt: receipt.clone(),
            },
        );

        if self.fail_next_commit {
            self.fail_next_commit = false;
            return Err(CloudComputeK8sDeleteRepositoryError::Unavailable);
        }
        *current = next;
        Ok(receipt)
    }
}

/// Populate the catalog with one cluster so delete tests have something to find.
fn catalog_with_cluster() -> (CloudComputeCatalog, CloudComputeK8sCreateIdempotencyLedger) {
    let mut catalog = CloudComputeCatalog::default();
    let mut create_ledger = CloudComputeK8sCreateIdempotencyLedger::default();
    create_cloud_compute_k8s_cluster_from_api(
        &mut catalog,
        &mut create_ledger,
        request("req-setup-delete", "idem-setup-delete"),
    )
    .expect("setup cluster create succeeds");
    (catalog, create_ledger)
}

fn delete_repository_with_cluster() -> DeleteTestRepository {
    let (catalog, _) = catalog_with_cluster();
    DeleteTestRepository::new(catalog)
}

fn stored_cluster_lifecycle(
    repository: &DeleteTestRepository,
) -> (KubernetesClusterState, KubernetesClusterDesiredState) {
    let catalog = repository.catalog_snapshot();
    let cluster = catalog
        .kubernetes_clusters()
        .find(|cluster| cluster.resource_id.value.value == CLUSTER_ID)
        .expect("setup cluster remains in the catalog");
    (cluster.state.value, cluster.desired_state.value)
}
