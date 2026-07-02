//! # oya-shared-resource-provider-contract-kernel
//!
//! The uniform resource-provider contract-test harness (FD-001 contract-lock
//! seed). Every platform service runs these generic conformance checks
//! against its resource handlers, so resource semantics are identical across
//! the catalog — the same play AWS runs with the Smithy protocol test suites
//! and Google with AIP conformance:
//!
//! - **Idempotent PUT** — replaying a PUT with the same client idempotency
//!   key is a no-op that returns the original outcome (AIP-134 full replace;
//!   AWS idempotent-PutX semantics).
//! - **No duplicate create** — retrying a create under the same client-UUID
//!   idempotency key returns the original resource and never creates a
//!   second one (AIP-155 request ids; EC2 RunInstances client tokens).
//! - **Read-after-write equality** — a get immediately after a write returns
//!   exactly the written resource.
//! - **Stable pagination** — cursor pagination yields every resource exactly
//!   once in a stable total order across repeated walks (AIP-158).
//! - **AIP-151 operations** — async mutations return an operation resource
//!   (`operations/...`, `done`, response XOR error) that is pollable and
//!   immutable once terminal.
//!
//! The harness is a trait + generic test fns, pure and IO-free. The
//! in-memory reference provider lives in `tests/` as the fixture that proves
//! the harness itself (test infrastructure, per the masterplan
//! no-false-green rule: the harness must demonstrably catch violations).
//!
//! ADR-0083 Tier-3: production code carries no unwrap/expect/panic.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]

use core::future::Future;
use core::pin::Pin;
use std::fmt;

use serde::{Deserialize, Serialize};

pub mod conformance;

/// Maximum page size any provider must accept.
pub const MAX_PAGE_SIZE: u32 = 1000;
/// Required name prefix for AIP-151 operation resources.
pub const OPERATION_NAME_PREFIX: &str = "operations/";

/// A contract-shape error raised while constructing harness types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractShapeError {
    /// The idempotency key is not a canonical RFC 4122 textual UUID.
    MalformedIdempotencyKey { value: String },
    /// The resource name is not `collection/resource-id` in slug form.
    MalformedResourceName { value: String },
    /// The page token is empty.
    EmptyPageToken,
    /// The page size is zero or exceeds [`MAX_PAGE_SIZE`].
    PageSizeOutOfRange { requested: u32 },
    /// The operation name lacks the [`OPERATION_NAME_PREFIX`].
    MalformedOperationName { value: String },
    /// The operation ledger entry is missing required AIP-151/control-plane metadata.
    MalformedOperationLedger { message: String },
    /// The operation's done/result shape disagrees with its ledger state.
    InvalidOperationState { message: String },
}

impl fmt::Display for ContractShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedIdempotencyKey { value } => {
                write!(
                    f,
                    "idempotency key {value:?} is not a canonical RFC 4122 UUID"
                )
            }
            Self::MalformedResourceName { value } => {
                write!(f, "resource name {value:?} is not `collection/resource-id`")
            }
            Self::EmptyPageToken => write!(f, "page token must be non-empty"),
            Self::PageSizeOutOfRange { requested } => {
                write!(f, "page size {requested} is outside 1..={MAX_PAGE_SIZE}")
            }
            Self::MalformedOperationName { value } => {
                write!(
                    f,
                    "operation name {value:?} lacks the {OPERATION_NAME_PREFIX:?} prefix"
                )
            }
            Self::MalformedOperationLedger { message } => {
                write!(f, "malformed operation ledger entry: {message}")
            }
            Self::InvalidOperationState { message } => {
                write!(f, "invalid operation state: {message}")
            }
        }
    }
}

impl std::error::Error for ContractShapeError {}

fn is_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '.' | '_'))
}

/// A client-supplied idempotency key: canonical RFC 4122 textual UUID
/// (8-4-4-4-12 hex groups), normalized to lowercase. Precedent: AIP-155
/// request ids and AWS client tokens, both of which require client-generated
/// UUIDs so retries are deduplicated server-side.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct IdempotencyKey(String);

impl IdempotencyKey {
    /// Parse and normalize a canonical textual UUID.
    pub fn new(value: impl Into<String>) -> Result<Self, ContractShapeError> {
        let value = value.into();
        let normalized = value.to_ascii_lowercase();
        let bytes = normalized.as_bytes();
        let well_formed = bytes.len() == 36
            && normalized.char_indices().all(|(i, c)| match i {
                8 | 13 | 18 | 23 => c == '-',
                _ => c.is_ascii_hexdigit(),
            });
        if well_formed {
            Ok(Self(normalized))
        } else {
            Err(ContractShapeError::MalformedIdempotencyKey { value })
        }
    }

    /// The normalized key text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for IdempotencyKey {
    type Error = ContractShapeError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<IdempotencyKey> for String {
    fn from(key: IdempotencyKey) -> Self {
        key.0
    }
}

/// A relative resource name in AIP-122 shape: `collection/resource-id`,
/// both segments slug-form.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ResourceName {
    collection: String,
    resource_id: String,
}

impl ResourceName {
    /// Build a resource name from its two segments.
    pub fn new(
        collection: impl Into<String>,
        resource_id: impl Into<String>,
    ) -> Result<Self, ContractShapeError> {
        let collection = collection.into();
        let resource_id = resource_id.into();
        if is_slug(&collection) && is_slug(&resource_id) {
            Ok(Self {
                collection,
                resource_id,
            })
        } else {
            Err(ContractShapeError::MalformedResourceName {
                value: format!("{collection}/{resource_id}"),
            })
        }
    }

    /// The collection segment.
    #[must_use]
    pub fn collection(&self) -> &str {
        &self.collection
    }

    /// The resource-id segment.
    #[must_use]
    pub fn resource_id(&self) -> &str {
        &self.resource_id
    }
}

impl fmt::Display for ResourceName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.collection, self.resource_id)
    }
}

impl TryFrom<String> for ResourceName {
    type Error = ContractShapeError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.split_once('/') {
            Some((collection, resource_id)) if !resource_id.contains('/') => {
                Self::new(collection, resource_id)
            }
            _ => Err(ContractShapeError::MalformedResourceName { value }),
        }
    }
}

impl From<ResourceName> for String {
    fn from(name: ResourceName) -> Self {
        name.to_string()
    }
}

/// An opaque pagination cursor (AIP-158: tokens are opaque to clients).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageToken(String);

impl PageToken {
    /// Build a non-empty page token.
    pub fn new(value: impl Into<String>) -> Result<Self, ContractShapeError> {
        let value = value.into();
        if value.is_empty() {
            Err(ContractShapeError::EmptyPageToken)
        } else {
            Ok(Self(value))
        }
    }

    /// The raw token text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A list-page request: bounded page size + optional cursor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageRequest {
    pub page_size: u32,                // data_class: INTERNAL_ONLY
    pub page_token: Option<PageToken>, // data_class: INTERNAL_ONLY
}

impl PageRequest {
    /// First page with the given size (`1..=MAX_PAGE_SIZE`).
    pub fn first(page_size: u32) -> Result<Self, ContractShapeError> {
        if page_size == 0 || page_size > MAX_PAGE_SIZE {
            return Err(ContractShapeError::PageSizeOutOfRange {
                requested: page_size,
            });
        }
        Ok(Self {
            page_size,
            page_token: None,
        })
    }

    /// The page after `token` with the same size.
    #[must_use]
    pub fn after(&self, token: PageToken) -> Self {
        Self {
            page_size: self.page_size,
            page_token: Some(token),
        }
    }
}

/// One listed entry: the resource plus its name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListEntry<R> {
    pub name: ResourceName, // data_class: TENANT_SCOPED
    pub resource: R,        // data_class: TENANT_SCOPED
}

/// One page of list results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Page<T> {
    pub items: Vec<T>,                      // data_class: TENANT_SCOPED
    pub next_page_token: Option<PageToken>, // data_class: INTERNAL_ONLY
}

/// Structured terminal error of an operation (AIP-193-shaped code+message).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationError {
    pub code: String,    // data_class: INTERNAL_ONLY
    pub message: String, // data_class: INTERNAL_ONLY
}

/// Terminal outcome of an operation: response XOR error.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationResult {
    Response(serde_json::Value),
    Error(OperationError),
}

/// Durable control-plane state for an AIP-151 long-running operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationState {
    Accepted,
    Validating,
    Queued,
    Running,
    WaitingForReconciler,
    Succeeded,
    Failed,
    CancelRequested,
    Cancelled,
    Compensating,
    RolledBack,
}

impl OperationState {
    /// Whether this state is terminal per the control-plane operation contract.
    #[must_use]
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Failed | Self::Cancelled | Self::RolledBack
        )
    }

    /// Whether the control-plane operation state machine allows this state to
    /// transition to `next`.
    #[must_use]
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Accepted, Self::Validating)
                | (Self::Accepted, Self::CancelRequested)
                | (Self::Validating, Self::Queued)
                | (Self::Validating, Self::Failed)
                | (Self::Validating, Self::CancelRequested)
                | (Self::Queued, Self::Running)
                | (Self::Queued, Self::CancelRequested)
                | (Self::Running, Self::WaitingForReconciler)
                | (Self::Running, Self::Succeeded)
                | (Self::Running, Self::Failed)
                | (Self::Running, Self::CancelRequested)
                | (Self::Running, Self::Compensating)
                | (Self::WaitingForReconciler, Self::Running)
                | (Self::WaitingForReconciler, Self::CancelRequested)
                | (Self::CancelRequested, Self::Cancelled)
                | (Self::CancelRequested, Self::Failed)
                | (Self::Compensating, Self::RolledBack)
                | (Self::Compensating, Self::Failed)
        )
    }
}

/// The control-plane pipeline phase owning the current operation ledger row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationPhase {
    ApiGateway,
    ResourceRegistry,
    OperationLedger,
    WorkflowReconciler,
    BackendActuationBoundary,
}

/// Retry metadata persisted in the operation ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetryPolicy {
    pub backoff: String,              // data_class: INTERNAL_ONLY
    pub max_attempts: u32,            // data_class: INTERNAL_ONLY
    pub retry_classification: String, // data_class: INTERNAL_ONLY
}

/// Retry classifications allowed by
/// `specs/cloud-control-plane-operation-contract.json#idempotency_retry_cancel_contract`.
pub const ALLOWED_RETRY_CLASSIFICATIONS: &[&str] = &[
    "transient",
    "quota",
    "policy",
    "dependency",
    "operator_required",
];

/// Cancellation metadata persisted in the operation ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CancellationMetadata {
    pub cancel_safe: bool,    // data_class: INTERNAL_ONLY
    pub audit_required: bool, // data_class: INTERNAL_ONLY
}

/// Compensation metadata persisted in the operation ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompensationMetadata {
    pub required: bool,   // data_class: INTERNAL_ONLY
    pub strategy: String, // data_class: INTERNAL_ONLY
}

/// Durable operation-ledger row required before acknowledging a mutating
/// resource-provider request. This mirrors
/// `specs/cloud-control-plane-operation-contract.json#operation_ledger_entry`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationLedgerEntry {
    pub operation_id: String,               // data_class: INTERNAL_ONLY
    pub idempotency_key: String,            // data_class: INTERNAL_ONLY
    pub request_hash: String,               // data_class: INTERNAL_ONLY
    pub resource_orn: String,               // data_class: TENANT_SCOPED
    pub desired_generation: u64,            // data_class: INTERNAL_ONLY
    pub observed_generation: u64,           // data_class: INTERNAL_ONLY
    pub state: OperationState,              // data_class: INTERNAL_ONLY
    pub phase: OperationPhase,              // data_class: INTERNAL_ONLY
    pub tenant_account_project: String,     // data_class: TENANT_SCOPED
    pub region_cell: String,                // data_class: TENANT_SCOPED
    pub principal: String,                  // data_class: INTERNAL_ONLY
    pub audit_chain_id: String,             // data_class: INTERNAL_ONLY
    pub retry_policy: RetryPolicy,          // data_class: INTERNAL_ONLY
    pub cancellation: CancellationMetadata, // data_class: INTERNAL_ONLY
    pub compensation: CompensationMetadata, // data_class: INTERNAL_ONLY
    pub transition_sequence: u64,           // data_class: INTERNAL_ONLY
}

impl OperationLedgerEntry {
    /// Validate the metadata-only operation-ledger contract: write-before-ack
    /// idempotency key, request hash, audit-chain linkage, generation bounds,
    /// retry/cancel/compensation metadata, and monotonic sequence presence.
    pub fn validate(&self) -> Result<(), ContractShapeError> {
        if !is_slug(&self.operation_id) {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: format!("operation_id {:?} is not slug-shaped", self.operation_id),
            });
        }
        IdempotencyKey::new(self.idempotency_key.clone()).map_err(|error| {
            ContractShapeError::MalformedOperationLedger {
                message: error.to_string(),
            }
        })?;
        if self.request_hash.is_empty() {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: "request_hash must be non-empty".to_owned(),
            });
        }
        if !self.resource_orn.starts_with("orn:") || !self.resource_orn.contains('/') {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: format!("resource_orn {:?} is not ORN-shaped", self.resource_orn),
            });
        }
        if self.desired_generation == 0 || self.observed_generation > self.desired_generation {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: format!(
                    "generation bounds invalid: desired={}, observed={}",
                    self.desired_generation, self.observed_generation
                ),
            });
        }
        for (field, value) in [
            (
                "tenant_account_project",
                self.tenant_account_project.as_str(),
            ),
            ("region_cell", self.region_cell.as_str()),
            ("principal", self.principal.as_str()),
            ("audit_chain_id", self.audit_chain_id.as_str()),
            ("retry_policy.backoff", self.retry_policy.backoff.as_str()),
            (
                "retry_policy.retry_classification",
                self.retry_policy.retry_classification.as_str(),
            ),
            ("compensation.strategy", self.compensation.strategy.as_str()),
        ] {
            if value.is_empty() {
                return Err(ContractShapeError::MalformedOperationLedger {
                    message: format!("{field} must be non-empty"),
                });
            }
        }
        if self.retry_policy.max_attempts == 0 {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: "retry_policy.max_attempts must be non-zero".to_owned(),
            });
        }
        if !ALLOWED_RETRY_CLASSIFICATIONS.contains(&self.retry_policy.retry_classification.as_str())
        {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: format!(
                    "retry_policy.retry_classification {:?} is not one of {:?}",
                    self.retry_policy.retry_classification, ALLOWED_RETRY_CLASSIFICATIONS
                ),
            });
        }
        if !self.cancellation.audit_required {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: "cancellation.audit_required must be true".to_owned(),
            });
        }
        if self.transition_sequence == 0 {
            return Err(ContractShapeError::MalformedOperationLedger {
                message: "transition_sequence must be non-zero".to_owned(),
            });
        }
        Ok(())
    }
}

/// An AIP-151-shaped operation resource for async mutations. The
/// constructors enforce the structural invariant `done == result.is_some()`:
/// a pending operation has no result, a terminal one always has exactly one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub name: String,                    // data_class: INTERNAL_ONLY
    pub done: bool,                      // data_class: INTERNAL_ONLY
    pub metadata: OperationLedgerEntry,  // data_class: INTERNAL_ONLY
    pub result: Option<OperationResult>, // data_class: INTERNAL_ONLY
}

impl Operation {
    fn checked_name(name: impl Into<String>) -> Result<String, ContractShapeError> {
        let name = name.into();
        if name.len() > OPERATION_NAME_PREFIX.len() && name.starts_with(OPERATION_NAME_PREFIX) {
            Ok(name)
        } else {
            Err(ContractShapeError::MalformedOperationName { value: name })
        }
    }

    fn checked_name_and_ledger(
        name: impl Into<String>,
        metadata: &OperationLedgerEntry,
    ) -> Result<String, ContractShapeError> {
        let name = Self::checked_name(name)?;
        metadata.validate()?;
        let expected = format!("{OPERATION_NAME_PREFIX}{}", metadata.operation_id);
        if name == expected {
            Ok(name)
        } else {
            Err(ContractShapeError::MalformedOperationLedger {
                message: format!(
                    "operation name {name:?} must match ledger operation_id {:?}",
                    metadata.operation_id
                ),
            })
        }
    }

    /// A still-running operation.
    pub fn pending(
        name: impl Into<String>,
        metadata: OperationLedgerEntry,
    ) -> Result<Self, ContractShapeError> {
        let name = Self::checked_name_and_ledger(name, &metadata)?;
        if metadata.state.is_terminal() {
            return Err(ContractShapeError::InvalidOperationState {
                message: format!(
                    "pending operation cannot carry terminal state {:?}",
                    metadata.state
                ),
            });
        }
        Ok(Self {
            name,
            done: false,
            metadata,
            result: None,
        })
    }

    /// A terminal, successful operation.
    pub fn succeeded(
        name: impl Into<String>,
        metadata: OperationLedgerEntry,
        response: serde_json::Value,
    ) -> Result<Self, ContractShapeError> {
        let name = Self::checked_name_and_ledger(name, &metadata)?;
        if metadata.state != OperationState::Succeeded {
            return Err(ContractShapeError::InvalidOperationState {
                message: format!(
                    "successful operation must carry succeeded ledger state, got {:?}",
                    metadata.state
                ),
            });
        }
        Ok(Self {
            name,
            done: true,
            metadata,
            result: Some(OperationResult::Response(response)),
        })
    }

    /// A terminal, failed operation.
    pub fn failed(
        name: impl Into<String>,
        metadata: OperationLedgerEntry,
        error: OperationError,
    ) -> Result<Self, ContractShapeError> {
        let name = Self::checked_name_and_ledger(name, &metadata)?;
        if !metadata.state.is_terminal() || metadata.state == OperationState::Succeeded {
            return Err(ContractShapeError::InvalidOperationState {
                message: format!(
                    "failed operation must carry failed/cancelled/rolled_back ledger state, got {:?}",
                    metadata.state
                ),
            });
        }
        Ok(Self {
            name,
            done: true,
            metadata,
            result: Some(OperationResult::Error(error)),
        })
    }

    /// Surface the structural invariant for operations received over the
    /// wire (where constructors were not in control).
    pub fn validate(&self) -> Result<(), ContractShapeError> {
        Self::checked_name_and_ledger(self.name.clone(), &self.metadata)?;
        if self.done != self.result.is_some() {
            return Err(ContractShapeError::InvalidOperationState {
                message: format!("{} done/result mismatch", self.name),
            });
        }
        if self.done != self.metadata.state.is_terminal() {
            return Err(ContractShapeError::InvalidOperationState {
                message: format!(
                    "{} done flag {:?} disagrees with ledger state {:?}",
                    self.name, self.done, self.metadata.state
                ),
            });
        }
        match (&self.result, self.metadata.state) {
            (Some(OperationResult::Response(_)), OperationState::Succeeded) | (None, _) => Ok(()),
            (Some(OperationResult::Error(_)), state)
                if state.is_terminal() && state != OperationState::Succeeded =>
            {
                Ok(())
            }
            (Some(OperationResult::Response(_)), state) => {
                Err(ContractShapeError::InvalidOperationState {
                    message: format!("response result cannot accompany ledger state {state:?}"),
                })
            }
            (Some(OperationResult::Error(_)), state) => {
                Err(ContractShapeError::InvalidOperationState {
                    message: format!("error result cannot accompany ledger state {state:?}"),
                })
            }
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn operation_ledger(operation_id: &str, state: OperationState) -> OperationLedgerEntry {
        OperationLedgerEntry {
            operation_id: operation_id.to_owned(),
            idempotency_key: "00000000-0000-4000-8000-000000000001".to_owned(),
            request_hash: format!("fixture-hash:{operation_id}"),
            resource_orn: "orn:oya:local-test:account-test:documents/documents/doc-1".to_owned(),
            desired_generation: 2,
            observed_generation: if state.is_terminal() { 2 } else { 1 },
            state,
            phase: OperationPhase::OperationLedger,
            tenant_account_project: "tenant-test/account-test/project-test".to_owned(),
            region_cell: "local-test/cell-0001".to_owned(),
            principal: "principal:test".to_owned(),
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
            transition_sequence: 1,
        }
    }

    #[test]
    fn idempotency_key_accepts_and_normalizes_canonical_uuids() {
        let key = IdempotencyKey::new("00000000-0000-4000-8000-00000000002A").unwrap();
        assert_eq!(key.as_str(), "00000000-0000-4000-8000-00000000002a");
    }

    #[test]
    fn idempotency_key_rejects_non_uuid_shapes() {
        for bad in [
            "",
            "not-a-uuid",
            "00000000-0000-4000-8000-00000000002", // too short
            "00000000-0000-4000-8000-00000000002az", // too long
            "00000000000040008000000000000020abcd", // no dashes
            "zzzzzzzz-0000-4000-8000-00000000002a", // non-hex
        ] {
            assert!(IdempotencyKey::new(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn resource_name_round_trips_through_aip_122_string_form() {
        let name = ResourceName::new("documents", "doc-1").unwrap();
        assert_eq!(name.to_string(), "documents/doc-1");
        let parsed: ResourceName = serde_json::from_str("\"documents/doc-1\"").unwrap();
        assert_eq!(parsed, name);
        assert_eq!(serde_json::to_string(&name).unwrap(), "\"documents/doc-1\"");
    }

    #[test]
    fn resource_name_rejects_malformed_forms() {
        assert!(ResourceName::new("Docs", "doc-1").is_err());
        assert!(ResourceName::new("documents", "").is_err());
        for bad in ["documents", "documents/a/b", "/doc-1", "documents/"] {
            assert!(
                serde_json::from_str::<ResourceName>(&format!("{bad:?}")).is_err(),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn page_request_bounds_are_enforced() {
        assert!(PageRequest::first(0).is_err());
        assert!(PageRequest::first(MAX_PAGE_SIZE + 1).is_err());
        let first = PageRequest::first(3).unwrap();
        let token = PageToken::new("cursor-1").unwrap();
        let next = first.after(token.clone());
        assert_eq!(next.page_size, 3);
        assert_eq!(next.page_token, Some(token));
        assert!(PageToken::new("").is_err());
    }

    #[test]
    fn operation_constructors_enforce_done_result_coupling() {
        let pending_ledger = operation_ledger("op-1", OperationState::Running);
        let pending = Operation::pending("operations/op-1", pending_ledger.clone()).unwrap();
        assert!(!pending.done);
        assert!(pending.result.is_none());
        assert_eq!(pending.metadata, pending_ledger);
        pending.validate().unwrap();

        let ok_ledger = operation_ledger("op-1", OperationState::Succeeded);
        let ok = Operation::succeeded("operations/op-1", ok_ledger, serde_json::json!({})).unwrap();
        assert!(ok.done);
        ok.validate().unwrap();

        let failed_ledger = operation_ledger("op-2", OperationState::Failed);
        let failed = Operation::failed(
            "operations/op-2",
            failed_ledger,
            OperationError {
                code: "failed_precondition".to_owned(),
                message: "resource busy".to_owned(),
            },
        )
        .unwrap();
        assert!(matches!(failed.result, Some(OperationResult::Error(_))));

        assert!(
            Operation::pending(
                "op-without-prefix",
                operation_ledger("op-without-prefix", OperationState::Running)
            )
            .is_err()
        );
        assert!(
            Operation::pending("operations/", operation_ledger("", OperationState::Running))
                .is_err()
        );

        let forged = Operation {
            name: "operations/op-3".to_owned(),
            done: true,
            metadata: operation_ledger("op-3", OperationState::Running),
            result: None,
        };
        assert!(forged.validate().is_err());
    }

    #[test]
    fn operation_ledger_rejects_unknown_retry_classification() {
        let mut ledger = operation_ledger("op-1", OperationState::Running);
        ledger.retry_policy.retry_classification = "eventually".to_owned();
        let error = ledger.validate().unwrap_err();
        assert!(
            error
                .to_string()
                .contains("retry_policy.retry_classification"),
            "{error}"
        );
    }
}
