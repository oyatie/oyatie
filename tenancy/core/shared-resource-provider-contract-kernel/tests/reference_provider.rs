//! The in-crate REFERENCE resource provider: a deterministic in-memory
//! implementation of [`ResourceProvider`] that exists to prove the harness
//! itself (it is the harness fixture — test infrastructure, not a product
//! artifact). Three deliberately nonconformant wrappers prove the harness
//! actually catches violations (masterplan no-false-green rule).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use shared_resource_provider_contract_kernel::conformance::{
    ConformanceFixture, check_async_delete_operation, check_create_idempotency,
    check_idempotent_put, check_operation_ledger_semantics, check_read_after_write,
    check_stable_pagination, run_all_checks,
};
use shared_resource_provider_contract_kernel::{
    CancellationMetadata, CompensationMetadata, CreateOutcome, IdempotencyKey, ListEntry,
    Operation, OperationLedgerEntry, OperationPhase, OperationState as LedgerState, Page,
    PageRequest, ProviderError, ProviderFuture, PutOutcome, ResourceName, ResourceProvider,
    RetryPolicy, WriteDisposition,
};
use serde::{Deserialize, Serialize};

/// The resource payload exercised by the reference fixture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Document {
    title: String,
    revision: u32,
}

/// What an idempotency key was first applied to (the dedup record).
#[derive(Debug, Clone, PartialEq)]
enum AppliedWrite {
    Create {
        name: String,
        payload: Document,
    },
    Put {
        name: String,
        payload: Document,
    },
    Delete {
        name: String,
        operation_name: String,
    },
}

/// Operation lifecycle inside the reference provider: pending operations
/// complete after one poll so the harness exercises the pending->done path.
#[derive(Debug, Clone)]
enum ReferenceOperationState {
    Pending {
        remaining_polls: u32,
        target: ResourceName,
        ledger: OperationLedgerEntry,
    },
    Terminal(Operation),
}

#[derive(Debug, Default)]
struct ReferenceProvider {
    items: BTreeMap<String, Document>,
    applied: BTreeMap<String, AppliedWrite>,
    operations: BTreeMap<String, ReferenceOperationState>,
    operation_seq: u64,
}

impl ReferenceProvider {
    fn resource_orn(name: &ResourceName) -> String {
        format!(
            "orn:oya:local-test:account-test:{}:{}/{}",
            name.collection(),
            name.collection(),
            name.resource_id()
        )
    }

    fn delete_ledger_entry(
        operation_id: &str,
        idempotency_key: &IdempotencyKey,
        name: &ResourceName,
        state: LedgerState,
        observed_generation: u64,
        transition_sequence: u64,
    ) -> OperationLedgerEntry {
        OperationLedgerEntry {
            operation_id: operation_id.to_owned(),
            idempotency_key: idempotency_key.as_str().to_owned(),
            request_hash: format!("fixture-hash:delete:{name}"),
            resource_orn: Self::resource_orn(name),
            desired_generation: 2,
            observed_generation,
            state,
            phase: OperationPhase::OperationLedger,
            tenant_account_project: "tenant-test/account-test/project-test".to_owned(),
            region_cell: "local-test/cell-0001".to_owned(),
            principal: "principal:test-harness".to_owned(),
            audit_chain_id: format!("audit-chain/{operation_id}"),
            retry_policy: RetryPolicy {
                backoff: "bounded-exponential-jitter".to_owned(),
                max_attempts: 3,
                retry_classification: "transient".to_owned(),
            },
            cancellation: CancellationMetadata {
                cancel_safe: true,
                audit_required: true,
            },
            compensation: CompensationMetadata {
                required: false,
                strategy: "none".to_owned(),
            },
            transition_sequence,
        }
    }
}

impl ResourceProvider for ReferenceProvider {
    type Resource = Document;

    fn create<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, CreateOutcome<Document>> {
        Box::pin(async move {
            let key = idempotency_key.as_str().to_owned();
            if let Some(applied) = self.applied.get(&key) {
                return match applied {
                    AppliedWrite::Create { name: n, payload }
                        if *n == name.to_string() && *payload == resource =>
                    {
                        Ok(CreateOutcome {
                            resource: payload.clone(),
                            replayed: true,
                        })
                    }
                    _ => Err(ProviderError::IdempotencyKeyReuse { key }),
                };
            }
            if self.items.contains_key(&name.to_string()) {
                return Err(ProviderError::AlreadyExists {
                    name: name.to_string(),
                });
            }
            self.items.insert(name.to_string(), resource.clone());
            self.applied.insert(
                key,
                AppliedWrite::Create {
                    name: name.to_string(),
                    payload: resource.clone(),
                },
            );
            Ok(CreateOutcome {
                resource,
                replayed: false,
            })
        })
    }

    fn put<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, PutOutcome<Document>> {
        Box::pin(async move {
            let key = idempotency_key.as_str().to_owned();
            if let Some(applied) = self.applied.get(&key) {
                return match applied {
                    AppliedWrite::Put { name: n, payload }
                        if *n == name.to_string() && *payload == resource =>
                    {
                        Ok(PutOutcome {
                            resource: payload.clone(),
                            disposition: WriteDisposition::Replayed,
                        })
                    }
                    _ => Err(ProviderError::IdempotencyKeyReuse { key }),
                };
            }
            let disposition = if self.items.contains_key(&name.to_string()) {
                WriteDisposition::Replaced
            } else {
                WriteDisposition::Created
            };
            self.items.insert(name.to_string(), resource.clone());
            self.applied.insert(
                key,
                AppliedWrite::Put {
                    name: name.to_string(),
                    payload: resource.clone(),
                },
            );
            Ok(PutOutcome {
                resource,
                disposition,
            })
        })
    }

    fn get<'a>(&'a self, name: &'a ResourceName) -> ProviderFuture<'a, Document> {
        Box::pin(async move {
            self.items
                .get(&name.to_string())
                .cloned()
                .ok_or_else(|| ProviderError::NotFound {
                    name: name.to_string(),
                })
        })
    }

    fn list<'a>(
        &'a self,
        collection: &'a str,
        request: &'a PageRequest,
    ) -> ProviderFuture<'a, Page<ListEntry<Document>>> {
        Box::pin(async move {
            let prefix = format!("{collection}/");
            let start_at = request
                .page_token
                .as_ref()
                .map(|token| token.as_str().to_owned());
            let mut items = Vec::new();
            let mut next_page_token = None;
            for (key, value) in &self.items {
                if !key.starts_with(&prefix) {
                    continue;
                }
                if let Some(start) = &start_at
                    && key < start
                {
                    continue;
                }
                if items.len() as u32 == request.page_size {
                    next_page_token = Some(
                        shared_resource_provider_contract_kernel::PageToken::new(key.clone())
                            .map_err(|e| ProviderError::Internal {
                            message: e.to_string(),
                        })?,
                    );
                    break;
                }
                let name =
                    ResourceName::try_from(key.clone()).map_err(|e| ProviderError::Internal {
                        message: e.to_string(),
                    })?;
                items.push(ListEntry {
                    name,
                    resource: value.clone(),
                });
            }
            Ok(Page {
                items,
                next_page_token,
            })
        })
    }

    fn delete<'a>(
        &'a mut self,
        name: &'a ResourceName,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, Operation> {
        Box::pin(async move {
            let key = idempotency_key.as_str().to_owned();
            if let Some(applied) = self.applied.get(&key) {
                return match applied {
                    AppliedWrite::Delete {
                        name: n,
                        operation_name,
                    } if *n == name.to_string() => self.snapshot_operation(operation_name),
                    _ => Err(ProviderError::IdempotencyKeyReuse { key }),
                };
            }
            if !self.items.contains_key(&name.to_string()) {
                return Err(ProviderError::NotFound {
                    name: name.to_string(),
                });
            }
            self.operation_seq += 1;
            let operation_id = format!("delete-{:06}", self.operation_seq);
            let operation_name = format!("operations/{operation_id}");
            let ledger = Self::delete_ledger_entry(
                &operation_id,
                idempotency_key,
                name,
                LedgerState::Running,
                1,
                1,
            );
            self.operations.insert(
                operation_name.clone(),
                ReferenceOperationState::Pending {
                    remaining_polls: 1,
                    target: name.clone(),
                    ledger: ledger.clone(),
                },
            );
            self.applied.insert(
                key,
                AppliedWrite::Delete {
                    name: name.to_string(),
                    operation_name: operation_name.clone(),
                },
            );
            Operation::pending(operation_name, ledger).map_err(|e| ProviderError::Internal {
                message: e.to_string(),
            })
        })
    }

    fn poll_operation<'a>(&'a mut self, operation_name: &'a str) -> ProviderFuture<'a, Operation> {
        Box::pin(async move {
            let state = self
                .operations
                .get(operation_name)
                .cloned()
                .ok_or_else(|| ProviderError::NotFound {
                    name: operation_name.to_owned(),
                })?;
            match state {
                ReferenceOperationState::Pending {
                    remaining_polls,
                    target,
                    ledger,
                } => {
                    if remaining_polls > 1 {
                        self.operations.insert(
                            operation_name.to_owned(),
                            ReferenceOperationState::Pending {
                                remaining_polls: remaining_polls - 1,
                                target,
                                ledger: ledger.clone(),
                            },
                        );
                        return Operation::pending(operation_name.to_owned(), ledger).map_err(
                            |e| ProviderError::Internal {
                                message: e.to_string(),
                            },
                        );
                    }
                    self.items.remove(&target.to_string());
                    let terminal_ledger = OperationLedgerEntry {
                        observed_generation: ledger.desired_generation,
                        state: LedgerState::Succeeded,
                        transition_sequence: ledger.transition_sequence + 1,
                        ..ledger
                    };
                    let terminal = Operation::succeeded(
                        operation_name.to_owned(),
                        terminal_ledger,
                        serde_json::json!({ "deleted": target.to_string() }),
                    )
                    .map_err(|e| ProviderError::Internal {
                        message: e.to_string(),
                    })?;
                    self.operations.insert(
                        operation_name.to_owned(),
                        ReferenceOperationState::Terminal(terminal.clone()),
                    );
                    Ok(terminal)
                }
                ReferenceOperationState::Terminal(operation) => Ok(operation),
            }
        })
    }

    fn operation_ledger_entry<'a>(
        &'a self,
        operation_name: &'a str,
    ) -> ProviderFuture<'a, OperationLedgerEntry> {
        Box::pin(async move {
            match self.operations.get(operation_name) {
                Some(ReferenceOperationState::Terminal(operation)) => {
                    Ok(operation.metadata.clone())
                }
                Some(ReferenceOperationState::Pending { ledger, .. }) => Ok(ledger.clone()),
                None => Err(ProviderError::NotFound {
                    name: operation_name.to_owned(),
                }),
            }
        })
    }
}

impl ReferenceProvider {
    fn snapshot_operation(&self, operation_name: &str) -> Result<Operation, ProviderError> {
        match self.operations.get(operation_name) {
            Some(ReferenceOperationState::Terminal(operation)) => Ok(operation.clone()),
            Some(ReferenceOperationState::Pending { ledger, .. }) => {
                Operation::pending(operation_name.to_owned(), ledger.clone()).map_err(|e| {
                    ProviderError::Internal {
                        message: e.to_string(),
                    }
                })
            }
            None => Err(ProviderError::NotFound {
                name: operation_name.to_owned(),
            }),
        }
    }
}

struct ReferenceFixture;

impl ConformanceFixture for ReferenceFixture {
    type Provider = ReferenceProvider;

    fn fresh_provider(&self) -> ReferenceProvider {
        ReferenceProvider::default()
    }

    fn collection(&self) -> &str {
        "documents"
    }

    fn resource_payload(&self, ordinal: u32) -> Document {
        Document {
            title: format!("Document {ordinal}"),
            revision: ordinal,
        }
    }
}

#[tokio::test]
async fn reference_provider_passes_idempotent_put() {
    check_idempotent_put(&ReferenceFixture).await.unwrap();
}

#[tokio::test]
async fn reference_provider_passes_create_idempotency() {
    check_create_idempotency(&ReferenceFixture).await.unwrap();
}

#[tokio::test]
async fn reference_provider_passes_read_after_write() {
    check_read_after_write(&ReferenceFixture).await.unwrap();
}

#[tokio::test]
async fn reference_provider_passes_stable_pagination() {
    check_stable_pagination(&ReferenceFixture).await.unwrap();
}

#[tokio::test]
async fn reference_provider_passes_async_delete_operation() {
    check_async_delete_operation(&ReferenceFixture)
        .await
        .unwrap();
}

#[tokio::test]
async fn reference_provider_passes_operation_ledger_semantics() {
    check_operation_ledger_semantics(&ReferenceFixture)
        .await
        .unwrap();
}

#[tokio::test]
async fn reference_provider_passes_the_full_contract() {
    let violations = run_all_checks(&ReferenceFixture).await;
    assert!(violations.is_empty(), "{violations:#?}");
}

// ---------------------------------------------------------------------------
// Nonconformant wrappers: prove the harness CATCHES contract violations.
// ---------------------------------------------------------------------------

/// Forgets the idempotency dedup log for create: a retried create collides
/// with its own first attempt instead of replaying.
#[derive(Debug, Default)]
struct NonReplayingCreateProvider(ReferenceProvider);

impl ResourceProvider for NonReplayingCreateProvider {
    type Resource = Document;

    fn create<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        _idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, CreateOutcome<Document>> {
        Box::pin(async move {
            if self.0.items.contains_key(&name.to_string()) {
                return Err(ProviderError::AlreadyExists {
                    name: name.to_string(),
                });
            }
            self.0.items.insert(name.to_string(), resource.clone());
            Ok(CreateOutcome {
                resource,
                replayed: false,
            })
        })
    }

    fn put<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, PutOutcome<Document>> {
        self.0.put(name, resource, idempotency_key)
    }

    fn get<'a>(&'a self, name: &'a ResourceName) -> ProviderFuture<'a, Document> {
        self.0.get(name)
    }

    fn list<'a>(
        &'a self,
        collection: &'a str,
        request: &'a PageRequest,
    ) -> ProviderFuture<'a, Page<ListEntry<Document>>> {
        self.0.list(collection, request)
    }

    fn delete<'a>(
        &'a mut self,
        name: &'a ResourceName,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, Operation> {
        self.0.delete(name, idempotency_key)
    }

    fn poll_operation<'a>(&'a mut self, operation_name: &'a str) -> ProviderFuture<'a, Operation> {
        self.0.poll_operation(operation_name)
    }

    fn operation_ledger_entry<'a>(
        &'a self,
        operation_name: &'a str,
    ) -> ProviderFuture<'a, OperationLedgerEntry> {
        self.0.operation_ledger_entry(operation_name)
    }
}

struct NonReplayingCreateFixture;

impl ConformanceFixture for NonReplayingCreateFixture {
    type Provider = NonReplayingCreateProvider;

    fn fresh_provider(&self) -> NonReplayingCreateProvider {
        NonReplayingCreateProvider::default()
    }

    fn collection(&self) -> &str {
        "documents"
    }

    fn resource_payload(&self, ordinal: u32) -> Document {
        ReferenceFixture.resource_payload(ordinal)
    }
}

#[tokio::test]
async fn harness_catches_create_that_does_not_replay() {
    let violation = check_create_idempotency(&NonReplayingCreateFixture)
        .await
        .unwrap_err();
    assert_eq!(violation.check, "create_idempotency");
}

/// Reports a replayed PUT as a replace: the visible-state contract holds but
/// the disposition lies, which the harness must flag.
#[derive(Debug, Default)]
struct MisreportingPutProvider(ReferenceProvider);

impl ResourceProvider for MisreportingPutProvider {
    type Resource = Document;

    fn create<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, CreateOutcome<Document>> {
        self.0.create(name, resource, idempotency_key)
    }

    fn put<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, PutOutcome<Document>> {
        Box::pin(async move {
            self.0
                .put(name, resource, idempotency_key)
                .await
                .map(|outcome| {
                    if outcome.disposition == WriteDisposition::Replayed {
                        PutOutcome {
                            resource: outcome.resource,
                            disposition: WriteDisposition::Replaced,
                        }
                    } else {
                        outcome
                    }
                })
        })
    }

    fn get<'a>(&'a self, name: &'a ResourceName) -> ProviderFuture<'a, Document> {
        self.0.get(name)
    }

    fn list<'a>(
        &'a self,
        collection: &'a str,
        request: &'a PageRequest,
    ) -> ProviderFuture<'a, Page<ListEntry<Document>>> {
        self.0.list(collection, request)
    }

    fn delete<'a>(
        &'a mut self,
        name: &'a ResourceName,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, Operation> {
        self.0.delete(name, idempotency_key)
    }

    fn poll_operation<'a>(&'a mut self, operation_name: &'a str) -> ProviderFuture<'a, Operation> {
        self.0.poll_operation(operation_name)
    }

    fn operation_ledger_entry<'a>(
        &'a self,
        operation_name: &'a str,
    ) -> ProviderFuture<'a, OperationLedgerEntry> {
        self.0.operation_ledger_entry(operation_name)
    }
}

struct MisreportingPutFixture;

impl ConformanceFixture for MisreportingPutFixture {
    type Provider = MisreportingPutProvider;

    fn fresh_provider(&self) -> MisreportingPutProvider {
        MisreportingPutProvider::default()
    }

    fn collection(&self) -> &str {
        "documents"
    }

    fn resource_payload(&self, ordinal: u32) -> Document {
        ReferenceFixture.resource_payload(ordinal)
    }
}

#[tokio::test]
async fn harness_catches_misreported_put_replay() {
    let violation = check_idempotent_put(&MisreportingPutFixture)
        .await
        .unwrap_err();
    assert_eq!(violation.check, "idempotent_put");
    assert!(violation.detail.contains("replayed"), "{violation}");
}

/// Returns an AIP-151 operation but refuses the durable ledger lookup. This
/// catches providers that put operation-looking metadata in the response while
/// skipping the operation-ledger write-before-ack contract.
#[derive(Debug, Default)]
struct MissingOperationLedgerProvider(ReferenceProvider);

impl ResourceProvider for MissingOperationLedgerProvider {
    type Resource = Document;

    fn create<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, CreateOutcome<Document>> {
        self.0.create(name, resource, idempotency_key)
    }

    fn put<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, PutOutcome<Document>> {
        self.0.put(name, resource, idempotency_key)
    }

    fn get<'a>(&'a self, name: &'a ResourceName) -> ProviderFuture<'a, Document> {
        self.0.get(name)
    }

    fn list<'a>(
        &'a self,
        collection: &'a str,
        request: &'a PageRequest,
    ) -> ProviderFuture<'a, Page<ListEntry<Document>>> {
        self.0.list(collection, request)
    }

    fn delete<'a>(
        &'a mut self,
        name: &'a ResourceName,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, Operation> {
        self.0.delete(name, idempotency_key)
    }

    fn poll_operation<'a>(&'a mut self, operation_name: &'a str) -> ProviderFuture<'a, Operation> {
        self.0.poll_operation(operation_name)
    }

    fn operation_ledger_entry<'a>(
        &'a self,
        operation_name: &'a str,
    ) -> ProviderFuture<'a, OperationLedgerEntry> {
        Box::pin(async move {
            Err(ProviderError::NotFound {
                name: operation_name.to_owned(),
            })
        })
    }
}

struct MissingOperationLedgerFixture;

impl ConformanceFixture for MissingOperationLedgerFixture {
    type Provider = MissingOperationLedgerProvider;

    fn fresh_provider(&self) -> MissingOperationLedgerProvider {
        MissingOperationLedgerProvider::default()
    }

    fn collection(&self) -> &str {
        "documents"
    }

    fn resource_payload(&self, ordinal: u32) -> Document {
        ReferenceFixture.resource_payload(ordinal)
    }
}

#[tokio::test]
async fn harness_catches_missing_operation_ledger_row() {
    let violation = check_operation_ledger_semantics(&MissingOperationLedgerFixture)
        .await
        .unwrap_err();
    assert_eq!(violation.check, "operation_ledger");
    assert!(
        violation.detail.contains("ledger read after delete"),
        "{violation}"
    );
}

/// Returns the same operation name on delete replay but mutates the operation
/// response metadata, proving the ledger check rejects a response that no
/// longer snapshots the durable ledger row.
#[derive(Debug, Default)]
struct MismatchedReplayOperationProvider {
    inner: ReferenceProvider,
    delete_replay_counts: BTreeMap<String, u32>,
}

impl ResourceProvider for MismatchedReplayOperationProvider {
    type Resource = Document;

    fn create<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, CreateOutcome<Document>> {
        self.inner.create(name, resource, idempotency_key)
    }

    fn put<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, PutOutcome<Document>> {
        self.inner.put(name, resource, idempotency_key)
    }

    fn get<'a>(&'a self, name: &'a ResourceName) -> ProviderFuture<'a, Document> {
        self.inner.get(name)
    }

    fn list<'a>(
        &'a self,
        collection: &'a str,
        request: &'a PageRequest,
    ) -> ProviderFuture<'a, Page<ListEntry<Document>>> {
        self.inner.list(collection, request)
    }

    fn delete<'a>(
        &'a mut self,
        name: &'a ResourceName,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, Operation> {
        Box::pin(async move {
            let key = idempotency_key.as_str().to_owned();
            let count = self.delete_replay_counts.get(&key).copied().unwrap_or(0);
            let mut operation = self.inner.delete(name, idempotency_key).await?;
            if count > 0 {
                operation.metadata.request_hash.push_str(":mismatched");
            }
            self.delete_replay_counts.insert(key, count + 1);
            Ok(operation)
        })
    }

    fn poll_operation<'a>(&'a mut self, operation_name: &'a str) -> ProviderFuture<'a, Operation> {
        self.inner.poll_operation(operation_name)
    }

    fn operation_ledger_entry<'a>(
        &'a self,
        operation_name: &'a str,
    ) -> ProviderFuture<'a, OperationLedgerEntry> {
        self.inner.operation_ledger_entry(operation_name)
    }
}

struct MismatchedReplayOperationFixture;

impl ConformanceFixture for MismatchedReplayOperationFixture {
    type Provider = MismatchedReplayOperationProvider;

    fn fresh_provider(&self) -> MismatchedReplayOperationProvider {
        MismatchedReplayOperationProvider::default()
    }

    fn collection(&self) -> &str {
        "documents"
    }

    fn resource_payload(&self, ordinal: u32) -> Document {
        ReferenceFixture.resource_payload(ordinal)
    }
}

#[tokio::test]
async fn harness_catches_replay_operation_that_no_longer_snapshots_ledger() {
    let violation = check_operation_ledger_semantics(&MismatchedReplayOperationFixture)
        .await
        .unwrap_err();
    assert_eq!(violation.check, "operation_ledger");
    assert!(
        violation
            .detail
            .contains("operation metadata must be a snapshot"),
        "{violation}"
    );
}

/// Replays the correct pending operation but returns a mutated terminal
/// operation snapshot once the original operation has completed. The contract
/// requires same-key/same-hash replay to return the existing operation's
/// current state, including after terminal completion.
#[derive(Debug, Default)]
struct TerminalReplayDriftProvider {
    inner: ReferenceProvider,
    delete_counts: BTreeMap<String, u32>,
}

impl ResourceProvider for TerminalReplayDriftProvider {
    type Resource = Document;

    fn create<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, CreateOutcome<Document>> {
        self.inner.create(name, resource, idempotency_key)
    }

    fn put<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, PutOutcome<Document>> {
        self.inner.put(name, resource, idempotency_key)
    }

    fn get<'a>(&'a self, name: &'a ResourceName) -> ProviderFuture<'a, Document> {
        self.inner.get(name)
    }

    fn list<'a>(
        &'a self,
        collection: &'a str,
        request: &'a PageRequest,
    ) -> ProviderFuture<'a, Page<ListEntry<Document>>> {
        self.inner.list(collection, request)
    }

    fn delete<'a>(
        &'a mut self,
        name: &'a ResourceName,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, Operation> {
        Box::pin(async move {
            let key = idempotency_key.as_str().to_owned();
            let count = self.delete_counts.get(&key).copied().unwrap_or(0);
            let mut operation = self.inner.delete(name, idempotency_key).await?;
            if count > 0 && operation.done {
                operation
                    .metadata
                    .request_hash
                    .push_str(":terminal-replay-drift");
            }
            self.delete_counts.insert(key, count + 1);
            Ok(operation)
        })
    }

    fn poll_operation<'a>(&'a mut self, operation_name: &'a str) -> ProviderFuture<'a, Operation> {
        self.inner.poll_operation(operation_name)
    }

    fn operation_ledger_entry<'a>(
        &'a self,
        operation_name: &'a str,
    ) -> ProviderFuture<'a, OperationLedgerEntry> {
        self.inner.operation_ledger_entry(operation_name)
    }
}

struct TerminalReplayDriftFixture;

impl ConformanceFixture for TerminalReplayDriftFixture {
    type Provider = TerminalReplayDriftProvider;

    fn fresh_provider(&self) -> TerminalReplayDriftProvider {
        TerminalReplayDriftProvider::default()
    }

    fn collection(&self) -> &str {
        "documents"
    }

    fn resource_payload(&self, ordinal: u32) -> Document {
        ReferenceFixture.resource_payload(ordinal)
    }
}

#[tokio::test]
async fn harness_catches_terminal_replay_that_no_longer_snapshots_ledger() {
    let violation = check_operation_ledger_semantics(&TerminalReplayDriftFixture)
        .await
        .unwrap_err();
    assert_eq!(violation.check, "operation_ledger");
    assert!(
        violation.detail.contains("terminal idempotent replay")
            || violation
                .detail
                .contains("operation metadata must be a snapshot"),
        "{violation}"
    );
}

/// Starts an operation in `accepted` and then jumps directly to `succeeded`,
/// which is not one of the allowed state-machine transitions in the control
/// plane operation contract.
#[derive(Debug, Default)]
struct DisallowedTransitionProvider {
    inner: ReferenceProvider,
    forced_ledgers: BTreeMap<String, OperationLedgerEntry>,
}

impl ResourceProvider for DisallowedTransitionProvider {
    type Resource = Document;

    fn create<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, CreateOutcome<Document>> {
        self.inner.create(name, resource, idempotency_key)
    }

    fn put<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, PutOutcome<Document>> {
        self.inner.put(name, resource, idempotency_key)
    }

    fn get<'a>(&'a self, name: &'a ResourceName) -> ProviderFuture<'a, Document> {
        self.inner.get(name)
    }

    fn list<'a>(
        &'a self,
        collection: &'a str,
        request: &'a PageRequest,
    ) -> ProviderFuture<'a, Page<ListEntry<Document>>> {
        self.inner.list(collection, request)
    }

    fn delete<'a>(
        &'a mut self,
        name: &'a ResourceName,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, Operation> {
        Box::pin(async move {
            let mut operation = self.inner.delete(name, idempotency_key).await?;
            if let Some(ledger) = self.forced_ledgers.get(&operation.name) {
                operation.metadata = ledger.clone();
            } else {
                operation.metadata.state = LedgerState::Accepted;
                self.forced_ledgers
                    .insert(operation.name.clone(), operation.metadata.clone());
            }
            Ok(operation)
        })
    }

    fn poll_operation<'a>(&'a mut self, operation_name: &'a str) -> ProviderFuture<'a, Operation> {
        Box::pin(async move {
            let mut operation = self.inner.poll_operation(operation_name).await?;
            if let Some(previous) = self.forced_ledgers.get(operation_name)
                && !previous.state.is_terminal()
            {
                operation.metadata.state = LedgerState::Succeeded;
                operation.metadata.observed_generation = operation.metadata.desired_generation;
                operation.metadata.transition_sequence = previous.transition_sequence + 1;
                self.forced_ledgers
                    .insert(operation_name.to_owned(), operation.metadata.clone());
            }
            Ok(operation)
        })
    }

    fn operation_ledger_entry<'a>(
        &'a self,
        operation_name: &'a str,
    ) -> ProviderFuture<'a, OperationLedgerEntry> {
        Box::pin(async move {
            if let Some(ledger) = self.forced_ledgers.get(operation_name) {
                return Ok(ledger.clone());
            }
            self.inner.operation_ledger_entry(operation_name).await
        })
    }
}

struct DisallowedTransitionFixture;

impl ConformanceFixture for DisallowedTransitionFixture {
    type Provider = DisallowedTransitionProvider;

    fn fresh_provider(&self) -> DisallowedTransitionProvider {
        DisallowedTransitionProvider::default()
    }

    fn collection(&self) -> &str {
        "documents"
    }

    fn resource_payload(&self, ordinal: u32) -> Document {
        ReferenceFixture.resource_payload(ordinal)
    }
}

#[tokio::test]
async fn harness_catches_disallowed_operation_state_transition() {
    let violation = check_operation_ledger_semantics(&DisallowedTransitionFixture)
        .await
        .unwrap_err();
    assert_eq!(violation.check, "operation_ledger");
    assert!(violation.detail.contains("state transition"), "{violation}");
}
