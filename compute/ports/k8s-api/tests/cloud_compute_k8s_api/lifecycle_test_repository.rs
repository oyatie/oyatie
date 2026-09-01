use super::*;

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
pub(super) struct LifecycleTestRepositoryState {
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
pub(super) struct LifecycleTestRepository {
    state: Arc<Mutex<LifecycleTestRepositoryState>>,
    control: Arc<Mutex<LifecycleTestRepositoryControl>>,
}

impl LifecycleTestRepository {
    pub(super) fn snapshot(&self) -> LifecycleTestRepositoryState {
        self.state
            .lock()
            .expect("repository lock is healthy")
            .clone()
    }

    pub(super) fn from_restored_snapshot(snapshot: LifecycleTestRepositoryState) -> Self {
        Self {
            state: Arc::new(Mutex::new(snapshot)),
            control: Arc::new(Mutex::new(LifecycleTestRepositoryControl::default())),
        }
    }

    pub(super) fn fail_next_commit(&self) {
        self.control
            .lock()
            .expect("repository control lock is healthy")
            .fail_next_commit = true;
    }

    pub(super) fn return_next_create_receipt(&self, receipt: CloudComputeK8sCreateReceipt) {
        self.control
            .lock()
            .expect("repository control lock is healthy")
            .next_create_receipt_override = Some(receipt);
    }

    pub(super) fn return_next_delete_receipt(&self, receipt: CloudComputeK8sDeleteReceipt) {
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

    pub(super) fn cluster_count(&self) -> usize {
        self.state
            .lock()
            .expect("repository lock is healthy")
            .clusters
            .len()
    }

    pub(super) fn create_operation_count(&self) -> usize {
        self.operation_count(|entry| matches!(entry, LifecycleTestOperationEntry::Create { .. }))
    }

    pub(super) fn delete_operation_count(&self) -> usize {
        self.operation_count(|entry| matches!(entry, LifecycleTestOperationEntry::Delete { .. }))
    }

    fn operation_count(
        &self,
        matches_entry: impl Fn(&LifecycleTestOperationEntry) -> bool,
    ) -> usize {
        self.state
            .lock()
            .expect("repository lock is healthy")
            .operations
            .values()
            .filter(|entry| matches_entry(entry))
            .count()
    }

    pub(super) fn cluster_record(&self, resource_id: &str) -> Option<CloudComputeK8sClusterRecord> {
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
                    LifecycleTestOperationEntry::Create { .. } => Err(
                        CloudComputeK8sLifecycleRepositoryError::IdempotencyKeyReused {
                            idempotency_key: command.operation_key.idempotency_key,
                        },
                    ),
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
                    LifecycleTestOperationEntry::Delete { .. } => Err(
                        CloudComputeK8sLifecycleRepositoryError::IdempotencyKeyReused {
                            idempotency_key: command.operation_key.idempotency_key,
                        },
                    ),
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
