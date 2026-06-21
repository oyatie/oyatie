//! Tenant lifecycle usecase — the tenant lifecycle control plane as a
//! uniform resource provider.
//!
//! Implements the locked G001 `ResourceProvider` contract
//! (`oya-shared-resource-provider-contract-kernel`) over the
//! `TenantLifecycleStore` port, with the tenant aggregate and closed
//! lifecycle state machine from `oya-shared-platform-contracts-kernel`.
//! Precedent: Azure ARM resource providers and Google AIP-151/155 — one
//! uniform contract per resource, client-UUID idempotency, and async
//! mutations as pollable operation resources.
//!
//! Invariants this layer enforces (the storage port stays dumb):
//! - Every written tenant passes the contract `Tenant::validate()`.
//! - A tenant is BORN in the initial lifecycle state; `create` rejects
//!   anything else and `put` may never change `state` — lifecycle moves
//!   happen ONLY through the AIP-151 operation ledger, so exactly one
//!   decision algorithm (the contract transition function) governs state.
//! - `delete` IS the `Retire` transition (terminal; the id is never
//!   reused): one path, not a parallel delete algorithm.
//! - Terminal ledger entries are immutable; replays under the same
//!   client-UUID idempotency key return the original outcome.
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

pub mod reconcile;

use core::future::Future;
use core::pin::Pin;

use oya_shared_platform_contracts_kernel::tenancy::{
    Tenant, TenantLifecycleOperation, TenantLifecycleState,
};
use oya_shared_resource_provider_contract_kernel::{
    CreateOutcome, IdempotencyKey, ListEntry, Operation, OperationError, Page, PageRequest,
    PageToken, ProviderError, PutOutcome, ResourceName, ResourceProvider, WriteDisposition,
};
use tenancy_tenant_lifecycle_kernel::{
    AppliedWriteRecord, OperationRecord, StoreError, TenantLifecycleStore,
};

/// The collection lifecycle resources live in (AIP-122 `tenants/<id>`).
pub const TENANT_COLLECTION: &str = "tenants";

fn store_error(error: StoreError) -> ProviderError {
    ProviderError::Internal {
        message: error.to_string(),
    }
}

fn shape_error(
    error: oya_shared_resource_provider_contract_kernel::ContractShapeError,
) -> ProviderError {
    ProviderError::Internal {
        message: error.to_string(),
    }
}

/// The smallest key strictly greater than `key` (ordered-scan resume point).
fn next_key_after(key: &str) -> String {
    let mut next = key.to_owned();
    next.push('\0');
    next
}

fn validate_tenant(tenant: &Tenant) -> Result<(), ProviderError> {
    tenant.validate().map_err(|violations| {
        let details: Vec<String> = violations.iter().map(ToString::to_string).collect();
        ProviderError::InvalidArgument {
            message: details.join("; "),
        }
    })
}

/// The tenant lifecycle control plane, generic over the storage port so the
/// G03 persistence adapter (and test fixtures) plug in behind it.
#[derive(Debug)]
pub struct TenantLifecycleProvider<S: TenantLifecycleStore> {
    store: S,
}

impl<S: TenantLifecycleStore> TenantLifecycleProvider<S> {
    /// Wrap a storage port.
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Start an async lifecycle transition (AIP-151): records a pending
    /// operation in the ledger and returns it. The transition itself is
    /// applied when the operation completes (on poll), through the contract
    /// transition function — never anywhere else.
    ///
    /// Replays under the same idempotency key return the SAME operation
    /// resource; the same key with different parameters fails with
    /// [`ProviderError::IdempotencyKeyReuse`].
    pub async fn apply_lifecycle(
        &mut self,
        name: &ResourceName,
        operation: TenantLifecycleOperation,
        idempotency_key: &IdempotencyKey,
    ) -> Result<Operation, ProviderError> {
        let key = idempotency_key.as_str().to_owned();
        if let Some(applied) = self.store.get_applied(&key).await.map_err(store_error)? {
            return match applied {
                AppliedWriteRecord::Lifecycle {
                    name: applied_name,
                    operation: applied_operation,
                    operation_name,
                } if applied_name == name.to_string() && applied_operation == operation => self
                    .store
                    .get_operation(&operation_name)
                    .await
                    .map_err(store_error)?
                    .map(|record| record.operation)
                    .ok_or(ProviderError::NotFound {
                        name: operation_name,
                    }),
                _ => Err(ProviderError::IdempotencyKeyReuse { key }),
            };
        }
        if self
            .store
            .get_tenant(&name.to_string())
            .await
            .map_err(store_error)?
            .is_none()
        {
            return Err(ProviderError::NotFound {
                name: name.to_string(),
            });
        }
        let seq = self.store.next_operation_seq().await.map_err(store_error)?;
        let operation_name = format!("operations/lifecycle-{seq:06}");
        let pending = Operation::pending(operation_name.clone()).map_err(shape_error)?;
        self.store
            .put_operation(
                &operation_name,
                &OperationRecord {
                    operation: pending.clone(),
                    kind: operation,
                    target: name.to_string(),
                },
            )
            .await
            .map_err(store_error)?;
        self.store
            .put_applied(
                &key,
                &AppliedWriteRecord::Lifecycle {
                    name: name.to_string(),
                    operation,
                    operation_name,
                },
            )
            .await
            .map_err(store_error)?;
        Ok(pending)
    }

    /// Store-level observation INCLUDING tombstones — the reconciler needs
    /// to see `Retired` to distinguish "never existed" from "terminally
    /// retired" (public `get` hides both as not-found).
    pub async fn observe_stored(
        &self,
        name: &ResourceName,
    ) -> Result<Option<Tenant>, ProviderError> {
        self.store
            .get_tenant(&name.to_string())
            .await
            .map_err(store_error)
    }

    /// Complete a pending ledger entry: evaluate the contract transition
    /// against the CURRENT tenant state and persist the outcome. Exactly one
    /// terminal write per operation; terminal entries are never touched.
    async fn complete_operation(
        &mut self,
        operation_name: &str,
        record: OperationRecord,
    ) -> Result<Operation, ProviderError> {
        let terminal = match self
            .store
            .get_tenant(&record.target)
            .await
            .map_err(store_error)?
        {
            None => Operation::failed(
                operation_name.to_owned(),
                OperationError {
                    code: "not_found".to_owned(),
                    message: format!("{} no longer exists", record.target),
                },
            )
            .map_err(shape_error)?,
            Some(tenant) => match tenant.apply_operation(record.kind) {
                Err(violation) => Operation::failed(
                    operation_name.to_owned(),
                    OperationError {
                        code: "failed_precondition".to_owned(),
                        message: violation.to_string(),
                    },
                )
                .map_err(shape_error)?,
                Ok(transitioned) => {
                    if transitioned.state == TenantLifecycleState::Retired {
                        // Retired is terminal and the id is never reused:
                        // the record stays as a TOMBSTONE (hidden from the
                        // readable surface, blocking name reuse forever).
                        // Physical removal is the crypto-shred offboarding
                        // path, a separate Cedar-gated G02 flow.
                        self.store
                            .put_tenant(&record.target, &transitioned)
                            .await
                            .map_err(store_error)?;
                        Operation::succeeded(
                            operation_name.to_owned(),
                            serde_json::json!({ "retired": record.target }),
                        )
                        .map_err(shape_error)?
                    } else {
                        self.store
                            .put_tenant(&record.target, &transitioned)
                            .await
                            .map_err(store_error)?;
                        let response =
                            serde_json::to_value(&transitioned).map_err(|error| {
                                ProviderError::Internal {
                                    message: error.to_string(),
                                }
                            })?;
                        Operation::succeeded(operation_name.to_owned(), response)
                            .map_err(shape_error)?
                    }
                }
            },
        };
        self.store
            .put_operation(
                operation_name,
                &OperationRecord {
                    operation: terminal.clone(),
                    kind: record.kind,
                    target: record.target,
                },
            )
            .await
            .map_err(store_error)?;
        Ok(terminal)
    }
}

impl<S: TenantLifecycleStore + Send + Sync> ResourceProvider for TenantLifecycleProvider<S> {
    type Resource = Tenant;

    fn create<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Tenant,
        idempotency_key: &'a IdempotencyKey,
    ) -> Pin<Box<dyn Future<Output = Result<CreateOutcome<Tenant>, ProviderError>> + Send + 'a>>
    {
        Box::pin(async move {
            let key = idempotency_key.as_str().to_owned();
            if let Some(applied) = self.store.get_applied(&key).await.map_err(store_error)? {
                return match applied {
                    AppliedWriteRecord::Create {
                        name: applied_name,
                        tenant,
                    } if applied_name == name.to_string() && tenant == resource => {
                        Ok(CreateOutcome {
                            resource: tenant,
                            replayed: true,
                        })
                    }
                    _ => Err(ProviderError::IdempotencyKeyReuse { key }),
                };
            }
            validate_tenant(&resource)?;
            if resource.state != TenantLifecycleState::initial() {
                return Err(ProviderError::InvalidArgument {
                    message: format!(
                        "tenants are created in the {:?} state; lifecycle moves go through operations",
                        TenantLifecycleState::initial()
                    ),
                });
            }
            if self
                .store
                .get_tenant(&name.to_string())
                .await
                .map_err(store_error)?
                .is_some()
            {
                return Err(ProviderError::AlreadyExists {
                    name: name.to_string(),
                });
            }
            self.store
                .put_tenant(&name.to_string(), &resource)
                .await
                .map_err(store_error)?;
            self.store
                .put_applied(
                    &key,
                    &AppliedWriteRecord::Create {
                        name: name.to_string(),
                        tenant: resource.clone(),
                    },
                )
                .await
                .map_err(store_error)?;
            Ok(CreateOutcome {
                resource,
                replayed: false,
            })
        })
    }

    fn put<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Tenant,
        idempotency_key: &'a IdempotencyKey,
    ) -> Pin<Box<dyn Future<Output = Result<PutOutcome<Tenant>, ProviderError>> + Send + 'a>> {
        Box::pin(async move {
            let key = idempotency_key.as_str().to_owned();
            if let Some(applied) = self.store.get_applied(&key).await.map_err(store_error)? {
                return match applied {
                    AppliedWriteRecord::Put {
                        name: applied_name,
                        tenant,
                    } if applied_name == name.to_string() && tenant == resource => {
                        Ok(PutOutcome {
                            resource: tenant,
                            disposition: WriteDisposition::Replayed,
                        })
                    }
                    _ => Err(ProviderError::IdempotencyKeyReuse { key }),
                };
            }
            validate_tenant(&resource)?;
            let existing = self
                .store
                .get_tenant(&name.to_string())
                .await
                .map_err(store_error)?;
            let disposition = match &existing {
                Some(current) => {
                    if current.state == TenantLifecycleState::Retired {
                        return Err(ProviderError::FailedPrecondition {
                            message: format!(
                                "{} is retired; tenant ids are never reused",
                                name
                            ),
                        });
                    }
                    if current.state != resource.state {
                        return Err(ProviderError::FailedPrecondition {
                            message: format!(
                                "put may not change lifecycle state ({:?} -> {:?}); use lifecycle operations",
                                current.state, resource.state
                            ),
                        });
                    }
                    WriteDisposition::Replaced
                }
                None => {
                    if resource.state != TenantLifecycleState::initial() {
                        return Err(ProviderError::InvalidArgument {
                            message: format!(
                                "tenants are created in the {:?} state; lifecycle moves go through operations",
                                TenantLifecycleState::initial()
                            ),
                        });
                    }
                    WriteDisposition::Created
                }
            };
            self.store
                .put_tenant(&name.to_string(), &resource)
                .await
                .map_err(store_error)?;
            self.store
                .put_applied(
                    &key,
                    &AppliedWriteRecord::Put {
                        name: name.to_string(),
                        tenant: resource.clone(),
                    },
                )
                .await
                .map_err(store_error)?;
            Ok(PutOutcome {
                resource,
                disposition,
            })
        })
    }

    fn get<'a>(
        &'a self,
        name: &'a ResourceName,
    ) -> Pin<Box<dyn Future<Output = Result<Tenant, ProviderError>> + Send + 'a>> {
        Box::pin(async move {
            match self
                .store
                .get_tenant(&name.to_string())
                .await
                .map_err(store_error)?
            {
                // Tombstones are invisible on the read surface.
                Some(tenant) if tenant.state != TenantLifecycleState::Retired => Ok(tenant),
                _ => Err(ProviderError::NotFound {
                    name: name.to_string(),
                }),
            }
        })
    }

    fn list<'a>(
        &'a self,
        collection: &'a str,
        request: &'a PageRequest,
    ) -> Pin<Box<dyn Future<Output = Result<Page<ListEntry<Tenant>>, ProviderError>> + Send + 'a>>
    {
        Box::pin(async move {
            let prefix = format!("{collection}/");
            let mut start_at = request
                .page_token
                .as_ref()
                .map(|token| token.as_str().to_owned());
            let mut items = Vec::new();
            let mut next_page_token = None;
            // Scan in key order, skipping tombstones, until the page fills
            // (plus one look-ahead key for the next cursor) or the store is
            // exhausted. Each chunk advances strictly, so this terminates.
            'walk: loop {
                let chunk = self
                    .store
                    .scan_tenants(
                        &prefix,
                        start_at.as_deref(),
                        request.page_size.saturating_add(1),
                    )
                    .await
                    .map_err(store_error)?;
                let chunk_len = chunk.len() as u32;
                for (key, tenant) in chunk {
                    if items.len() as u32 == request.page_size {
                        if tenant.state != TenantLifecycleState::Retired {
                            next_page_token = Some(PageToken::new(key).map_err(shape_error)?);
                            break 'walk;
                        }
                        // A tombstone at the boundary: keep looking for a real
                        // successor before claiming another page exists.
                        start_at = Some(next_key_after(&key));
                        continue;
                    }
                    start_at = Some(next_key_after(&key));
                    if tenant.state == TenantLifecycleState::Retired {
                        continue;
                    }
                    let name = ResourceName::try_from(key).map_err(shape_error)?;
                    items.push(ListEntry {
                        name,
                        resource: tenant,
                    });
                }
                if chunk_len < request.page_size.saturating_add(1) {
                    break; // store exhausted
                }
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
        // Delete IS the Retire transition: one decision algorithm.
        Box::pin(async move {
            self.apply_lifecycle(name, TenantLifecycleOperation::Retire, idempotency_key)
                .await
        })
    }

    fn poll_operation<'a>(
        &'a mut self,
        operation_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Operation, ProviderError>> + Send + 'a>> {
        Box::pin(async move {
            let record = self
                .store
                .get_operation(operation_name)
                .await
                .map_err(store_error)?
                .ok_or_else(|| ProviderError::NotFound {
                    name: operation_name.to_owned(),
                })?;
            if record.operation.done {
                return Ok(record.operation);
            }
            self.complete_operation(operation_name, record).await
        })
    }
}
