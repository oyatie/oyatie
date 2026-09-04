use core::future::Future;
use core::pin::Pin;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{
    IdempotencyKey, ListEntry, Operation, OperationLedgerEntry, Page, PageRequest, ResourceName,
};

/// Provider errors, gRPC-canonical-code-shaped (AIP-193 error model).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderError {
    /// Create of a name that already exists (under a NEW idempotency key).
    AlreadyExists { name: String },
    /// The named resource (or operation) does not exist.
    NotFound { name: String },
    /// An idempotency key was reused with different parameters.
    IdempotencyKeyReuse { key: String },
    /// The request is malformed.
    InvalidArgument { message: String },
    /// A state precondition failed.
    FailedPrecondition { message: String },
    /// Provider-internal failure.
    Internal { message: String },
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyExists { name } => write!(f, "already exists: {name}"),
            Self::NotFound { name } => write!(f, "not found: {name}"),
            Self::IdempotencyKeyReuse { key } => {
                write!(f, "idempotency key {key} reused with different parameters")
            }
            Self::InvalidArgument { message } => write!(f, "invalid argument: {message}"),
            Self::FailedPrecondition { message } => write!(f, "failed precondition: {message}"),
            Self::Internal { message } => write!(f, "internal: {message}"),
        }
    }
}

impl std::error::Error for ProviderError {}

/// Boxed async result returned by resource-provider ports. The alias keeps the
/// dependency-free port shape readable while preserving explicit `Future` +
/// `Pin` semantics at the boundary.
pub type ProviderFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, ProviderError>> + Send + 'a>>;

/// How a PUT landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WriteDisposition {
    /// The resource did not exist; this write created it.
    Created,
    /// The resource existed; this write replaced it.
    Replaced,
    /// The idempotency key was already applied; nothing changed.
    Replayed,
}

/// Outcome of a PUT.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PutOutcome<R> {
    pub resource: R,                   // data_class: TENANT_SCOPED
    pub disposition: WriteDisposition, // data_class: INTERNAL_ONLY
}

/// Outcome of a create.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateOutcome<R> {
    pub resource: R, // data_class: TENANT_SCOPED
    /// True when this call was deduplicated against an earlier create that
    /// used the same idempotency key.
    pub replayed: bool, // data_class: INTERNAL_ONLY
}

/// The uniform resource-provider contract. Methods are async (the durable
/// store behind a provider performs real I/O); the kernel itself stays
/// IO-free and dependency-free by modelling async with a return-position
/// boxed future — `core::future::Future` + `core::pin::Pin` + `Box::pin`, no
/// `async-trait` / `futures` dep (ADR-0376 rejects async-trait for ports;
/// the blessed `ProviderInvocationTransport` port shape). Transport bindings
/// adapt this trait at their own layer.
pub trait ResourceProvider {
    /// The resource payload type.
    type Resource: Clone + PartialEq + fmt::Debug;

    /// Create `name`. Replays under the same idempotency key MUST return the
    /// original resource with `replayed = true`; the same key with different
    /// parameters MUST fail with [`ProviderError::IdempotencyKeyReuse`]; an
    /// existing name under a new key MUST fail with
    /// [`ProviderError::AlreadyExists`].
    fn create<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Self::Resource,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, CreateOutcome<Self::Resource>>;

    /// Full-replace upsert of `name` (AIP-134). Replays under the same
    /// idempotency key MUST be no-ops returning the original outcome with
    /// [`WriteDisposition::Replayed`].
    fn put<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Self::Resource,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, PutOutcome<Self::Resource>>;

    /// Read `name`, exactly as last written.
    fn get<'a>(&'a self, name: &'a ResourceName) -> ProviderFuture<'a, Self::Resource>;

    /// List a `collection` page in a stable total order.
    fn list<'a>(
        &'a self,
        collection: &'a str,
        request: &'a PageRequest,
    ) -> ProviderFuture<'a, Page<ListEntry<Self::Resource>>>;

    /// Async delete of `name`: returns an AIP-151 operation. Replays under
    /// the same idempotency key MUST return the SAME operation resource.
    fn delete<'a>(
        &'a mut self,
        name: &'a ResourceName,
        idempotency_key: &'a IdempotencyKey,
    ) -> ProviderFuture<'a, Operation>;

    /// Poll an operation by name. Terminal operations are immutable.
    fn poll_operation<'a>(&'a mut self, operation_name: &'a str) -> ProviderFuture<'a, Operation>;

    /// Read the durable operation-ledger row backing an AIP-151 operation.
    fn operation_ledger_entry<'a>(
        &'a self,
        operation_name: &'a str,
    ) -> ProviderFuture<'a, OperationLedgerEntry>;
}
