//! The in-crate REFERENCE resource provider: a deterministic in-memory
//! implementation of [`ResourceProvider`] that exists to prove the harness
//! itself (it is the harness fixture — test infrastructure, not a product
//! artifact). Two deliberately nonconformant wrappers prove the harness
//! actually catches violations (masterplan no-false-green rule).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use oya_shared_resource_provider_contract_kernel::conformance::{
    ConformanceFixture, check_async_delete_operation, check_create_idempotency,
    check_idempotent_put, check_read_after_write, check_stable_pagination, run_all_checks,
};
use oya_shared_resource_provider_contract_kernel::{
    CreateOutcome, IdempotencyKey, ListEntry, Operation, Page, PageRequest, ProviderError,
    PutOutcome, ResourceName, ResourceProvider, WriteDisposition,
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
enum OperationState {
    Pending {
        remaining_polls: u32,
        target: String,
    },
    Terminal(Operation),
}

#[derive(Debug, Default)]
struct ReferenceProvider {
    items: BTreeMap<String, Document>,
    applied: BTreeMap<String, AppliedWrite>,
    operations: BTreeMap<String, OperationState>,
    operation_seq: u64,
}

impl ResourceProvider for ReferenceProvider {
    type Resource = Document;

    fn create<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> Pin<Box<dyn Future<Output = Result<CreateOutcome<Document>, ProviderError>> + Send + 'a>>
    {
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
    ) -> Pin<Box<dyn Future<Output = Result<PutOutcome<Document>, ProviderError>> + Send + 'a>> {
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

    fn get<'a>(
        &'a self,
        name: &'a ResourceName,
    ) -> Pin<Box<dyn Future<Output = Result<Document, ProviderError>> + Send + 'a>> {
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
    ) -> Pin<Box<dyn Future<Output = Result<Page<ListEntry<Document>>, ProviderError>> + Send + 'a>>
    {
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
        })
    }

    fn delete<'a>(
        &'a mut self,
        name: &'a ResourceName,
        idempotency_key: &'a IdempotencyKey,
    ) -> Pin<Box<dyn Future<Output = Result<Operation, ProviderError>> + Send + 'a>> {
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
            let operation_name = format!("operations/delete-{:06}", self.operation_seq);
            self.operations.insert(
                operation_name.clone(),
                OperationState::Pending {
                    remaining_polls: 1,
                    target: name.to_string(),
                },
            );
            self.applied.insert(
                key,
                AppliedWrite::Delete {
                    name: name.to_string(),
                    operation_name: operation_name.clone(),
                },
            );
            Operation::pending(operation_name).map_err(|e| ProviderError::Internal {
                message: e.to_string(),
            })
        })
    }

    fn poll_operation<'a>(
        &'a mut self,
        operation_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Operation, ProviderError>> + Send + 'a>> {
        Box::pin(async move {
            let state = self
                .operations
                .get(operation_name)
                .cloned()
                .ok_or_else(|| ProviderError::NotFound {
                    name: operation_name.to_owned(),
                })?;
            match state {
                OperationState::Pending {
                    remaining_polls,
                    target,
                } => {
                    if remaining_polls > 1 {
                        self.operations.insert(
                            operation_name.to_owned(),
                            OperationState::Pending {
                                remaining_polls: remaining_polls - 1,
                                target,
                            },
                        );
                        return Operation::pending(operation_name.to_owned()).map_err(|e| {
                            ProviderError::Internal {
                                message: e.to_string(),
                            }
                        });
                    }
                    self.items.remove(&target);
                    let terminal = Operation::succeeded(
                        operation_name.to_owned(),
                        serde_json::json!({ "deleted": target }),
                    )
                    .map_err(|e| ProviderError::Internal {
                        message: e.to_string(),
                    })?;
                    self.operations.insert(
                        operation_name.to_owned(),
                        OperationState::Terminal(terminal.clone()),
                    );
                    Ok(terminal)
                }
                OperationState::Terminal(operation) => Ok(operation),
            }
        })
    }
}

impl ReferenceProvider {
    fn snapshot_operation(&self, operation_name: &str) -> Result<Operation, ProviderError> {
        match self.operations.get(operation_name) {
            Some(OperationState::Terminal(operation)) => Ok(operation.clone()),
            Some(OperationState::Pending { .. }) => Operation::pending(operation_name.to_owned())
                .map_err(|e| ProviderError::Internal {
                    message: e.to_string(),
                }),
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
    check_async_delete_operation(&ReferenceFixture).await.unwrap();
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
    ) -> Pin<Box<dyn Future<Output = Result<CreateOutcome<Document>, ProviderError>> + Send + 'a>>
    {
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
    ) -> Pin<Box<dyn Future<Output = Result<PutOutcome<Document>, ProviderError>> + Send + 'a>> {
        self.0.put(name, resource, idempotency_key)
    }

    fn get<'a>(
        &'a self,
        name: &'a ResourceName,
    ) -> Pin<Box<dyn Future<Output = Result<Document, ProviderError>> + Send + 'a>> {
        self.0.get(name)
    }

    fn list<'a>(
        &'a self,
        collection: &'a str,
        request: &'a PageRequest,
    ) -> Pin<Box<dyn Future<Output = Result<Page<ListEntry<Document>>, ProviderError>> + Send + 'a>>
    {
        self.0.list(collection, request)
    }

    fn delete<'a>(
        &'a mut self,
        name: &'a ResourceName,
        idempotency_key: &'a IdempotencyKey,
    ) -> Pin<Box<dyn Future<Output = Result<Operation, ProviderError>> + Send + 'a>> {
        self.0.delete(name, idempotency_key)
    }

    fn poll_operation<'a>(
        &'a mut self,
        operation_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Operation, ProviderError>> + Send + 'a>> {
        self.0.poll_operation(operation_name)
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
    ) -> Pin<Box<dyn Future<Output = Result<CreateOutcome<Document>, ProviderError>> + Send + 'a>>
    {
        self.0.create(name, resource, idempotency_key)
    }

    fn put<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Document,
        idempotency_key: &'a IdempotencyKey,
    ) -> Pin<Box<dyn Future<Output = Result<PutOutcome<Document>, ProviderError>> + Send + 'a>> {
        Box::pin(async move {
            self.0.put(name, resource, idempotency_key).await.map(|outcome| {
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

    fn get<'a>(
        &'a self,
        name: &'a ResourceName,
    ) -> Pin<Box<dyn Future<Output = Result<Document, ProviderError>> + Send + 'a>> {
        self.0.get(name)
    }

    fn list<'a>(
        &'a self,
        collection: &'a str,
        request: &'a PageRequest,
    ) -> Pin<Box<dyn Future<Output = Result<Page<ListEntry<Document>>, ProviderError>> + Send + 'a>>
    {
        self.0.list(collection, request)
    }

    fn delete<'a>(
        &'a mut self,
        name: &'a ResourceName,
        idempotency_key: &'a IdempotencyKey,
    ) -> Pin<Box<dyn Future<Output = Result<Operation, ProviderError>> + Send + 'a>> {
        self.0.delete(name, idempotency_key)
    }

    fn poll_operation<'a>(
        &'a mut self,
        operation_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Operation, ProviderError>> + Send + 'a>> {
        self.0.poll_operation(operation_name)
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
