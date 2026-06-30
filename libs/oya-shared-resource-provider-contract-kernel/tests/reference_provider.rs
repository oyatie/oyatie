//! The in-crate REFERENCE resource provider: a deterministic in-memory
//! implementation of [`ResourceProvider`] that exists to prove the harness
//! itself (it is the harness fixture — test infrastructure, not a product
//! artifact). Two deliberately nonconformant wrappers prove the harness
//! actually catches violations (masterplan no-false-green rule).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use oya_shared_resource_provider_contract_kernel::conformance::{
    ConformanceFixture, check_async_delete_operation, check_create_idempotency,
    check_idempotent_put, check_operation_ledger_semantics, check_read_after_write,
    check_stable_pagination, run_all_checks,
};
use oya_shared_resource_provider_contract_kernel::{
    CancellationMetadata, CompensationMetadata, CreateOutcome, IdempotencyKey, ListEntry,
    Operation, OperationLedgerEntry, OperationPhase, OperationState as LedgerState, Page,
    PageRequest, ProviderError, PutOutcome, ResourceName, ResourceProvider, RetryPolicy,
    WriteDisposition,
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

    fn create(
        &mut self,
        name: &ResourceName,
        resource: Document,
        idempotency_key: &IdempotencyKey,
    ) -> Result<CreateOutcome<Document>, ProviderError> {
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
    }

    fn put(
        &mut self,
        name: &ResourceName,
        resource: Document,
        idempotency_key: &IdempotencyKey,
    ) -> Result<PutOutcome<Document>, ProviderError> {
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
    }

    fn get(&self, name: &ResourceName) -> Result<Document, ProviderError> {
        self.items
            .get(&name.to_string())
            .cloned()
            .ok_or_else(|| ProviderError::NotFound {
                name: name.to_string(),
            })
    }

    fn list(
        &self,
        collection: &str,
        request: &PageRequest,
    ) -> Result<Page<ListEntry<Document>>, ProviderError> {
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
                    oya_shared_resource_provider_contract_kernel::PageToken::new(key.clone())
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
    }

    fn delete(
        &mut self,
        name: &ResourceName,
        idempotency_key: &IdempotencyKey,
    ) -> Result<Operation, ProviderError> {
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
    }

    fn poll_operation(&mut self, operation_name: &str) -> Result<Operation, ProviderError> {
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
                    return Operation::pending(operation_name.to_owned(), ledger).map_err(|e| {
                        ProviderError::Internal {
                            message: e.to_string(),
                        }
                    });
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
    }

    fn operation_ledger_entry(
        &self,
        operation_name: &str,
    ) -> Result<OperationLedgerEntry, ProviderError> {
        match self.operations.get(operation_name) {
            Some(ReferenceOperationState::Terminal(operation)) => Ok(operation.metadata.clone()),
            Some(ReferenceOperationState::Pending { ledger, .. }) => Ok(ledger.clone()),
            None => Err(ProviderError::NotFound {
                name: operation_name.to_owned(),
            }),
        }
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

#[test]
fn reference_provider_passes_idempotent_put() {
    check_idempotent_put(&ReferenceFixture).unwrap();
}

#[test]
fn reference_provider_passes_create_idempotency() {
    check_create_idempotency(&ReferenceFixture).unwrap();
}

#[test]
fn reference_provider_passes_read_after_write() {
    check_read_after_write(&ReferenceFixture).unwrap();
}

#[test]
fn reference_provider_passes_stable_pagination() {
    check_stable_pagination(&ReferenceFixture).unwrap();
}

#[test]
fn reference_provider_passes_async_delete_operation() {
    check_async_delete_operation(&ReferenceFixture).unwrap();
}

#[test]
fn reference_provider_passes_operation_ledger_semantics() {
    check_operation_ledger_semantics(&ReferenceFixture).unwrap();
}

#[test]
fn reference_provider_passes_the_full_contract() {
    let violations = run_all_checks(&ReferenceFixture);
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

    fn create(
        &mut self,
        name: &ResourceName,
        resource: Document,
        _idempotency_key: &IdempotencyKey,
    ) -> Result<CreateOutcome<Document>, ProviderError> {
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
    }

    fn put(
        &mut self,
        name: &ResourceName,
        resource: Document,
        idempotency_key: &IdempotencyKey,
    ) -> Result<PutOutcome<Document>, ProviderError> {
        self.0.put(name, resource, idempotency_key)
    }

    fn get(&self, name: &ResourceName) -> Result<Document, ProviderError> {
        self.0.get(name)
    }

    fn list(
        &self,
        collection: &str,
        request: &PageRequest,
    ) -> Result<Page<ListEntry<Document>>, ProviderError> {
        self.0.list(collection, request)
    }

    fn delete(
        &mut self,
        name: &ResourceName,
        idempotency_key: &IdempotencyKey,
    ) -> Result<Operation, ProviderError> {
        self.0.delete(name, idempotency_key)
    }

    fn poll_operation(&mut self, operation_name: &str) -> Result<Operation, ProviderError> {
        self.0.poll_operation(operation_name)
    }

    fn operation_ledger_entry(
        &self,
        operation_name: &str,
    ) -> Result<OperationLedgerEntry, ProviderError> {
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

#[test]
fn harness_catches_create_that_does_not_replay() {
    let violation = check_create_idempotency(&NonReplayingCreateFixture).unwrap_err();
    assert_eq!(violation.check, "create_idempotency");
}

/// Reports a replayed PUT as a replace: the visible-state contract holds but
/// the disposition lies, which the harness must flag.
#[derive(Debug, Default)]
struct MisreportingPutProvider(ReferenceProvider);

impl ResourceProvider for MisreportingPutProvider {
    type Resource = Document;

    fn create(
        &mut self,
        name: &ResourceName,
        resource: Document,
        idempotency_key: &IdempotencyKey,
    ) -> Result<CreateOutcome<Document>, ProviderError> {
        self.0.create(name, resource, idempotency_key)
    }

    fn put(
        &mut self,
        name: &ResourceName,
        resource: Document,
        idempotency_key: &IdempotencyKey,
    ) -> Result<PutOutcome<Document>, ProviderError> {
        self.0.put(name, resource, idempotency_key).map(|outcome| {
            if outcome.disposition == WriteDisposition::Replayed {
                PutOutcome {
                    resource: outcome.resource,
                    disposition: WriteDisposition::Replaced,
                }
            } else {
                outcome
            }
        })
    }

    fn get(&self, name: &ResourceName) -> Result<Document, ProviderError> {
        self.0.get(name)
    }

    fn list(
        &self,
        collection: &str,
        request: &PageRequest,
    ) -> Result<Page<ListEntry<Document>>, ProviderError> {
        self.0.list(collection, request)
    }

    fn delete(
        &mut self,
        name: &ResourceName,
        idempotency_key: &IdempotencyKey,
    ) -> Result<Operation, ProviderError> {
        self.0.delete(name, idempotency_key)
    }

    fn poll_operation(&mut self, operation_name: &str) -> Result<Operation, ProviderError> {
        self.0.poll_operation(operation_name)
    }

    fn operation_ledger_entry(
        &self,
        operation_name: &str,
    ) -> Result<OperationLedgerEntry, ProviderError> {
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

#[test]
fn harness_catches_misreported_put_replay() {
    let violation = check_idempotent_put(&MisreportingPutFixture).unwrap_err();
    assert_eq!(violation.check, "idempotent_put");
    assert!(violation.detail.contains("replayed"), "{violation}");
}
