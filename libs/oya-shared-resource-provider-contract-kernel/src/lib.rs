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

/// An AIP-151-shaped operation resource for async mutations. The
/// constructors enforce the structural invariant `done == result.is_some()`:
/// a pending operation has no result, a terminal one always has exactly one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub name: String,                    // data_class: INTERNAL_ONLY
    pub done: bool,                      // data_class: INTERNAL_ONLY
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

    /// A still-running operation.
    pub fn pending(name: impl Into<String>) -> Result<Self, ContractShapeError> {
        Ok(Self {
            name: Self::checked_name(name)?,
            done: false,
            result: None,
        })
    }

    /// A terminal, successful operation.
    pub fn succeeded(
        name: impl Into<String>,
        response: serde_json::Value,
    ) -> Result<Self, ContractShapeError> {
        Ok(Self {
            name: Self::checked_name(name)?,
            done: true,
            result: Some(OperationResult::Response(response)),
        })
    }

    /// A terminal, failed operation.
    pub fn failed(
        name: impl Into<String>,
        error: OperationError,
    ) -> Result<Self, ContractShapeError> {
        Ok(Self {
            name: Self::checked_name(name)?,
            done: true,
            result: Some(OperationResult::Error(error)),
        })
    }

    /// Surface the structural invariant for operations received over the
    /// wire (where constructors were not in control).
    pub fn validate(&self) -> Result<(), ContractShapeError> {
        Self::checked_name(self.name.clone())?;
        if self.done == self.result.is_some() {
            Ok(())
        } else {
            Err(ContractShapeError::MalformedOperationName {
                value: format!("{} (done/result mismatch)", self.name),
            })
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
    ) -> Pin<
        Box<dyn Future<Output = Result<CreateOutcome<Self::Resource>, ProviderError>> + Send + 'a>,
    >;

    /// Full-replace upsert of `name` (AIP-134). Replays under the same
    /// idempotency key MUST be no-ops returning the original outcome with
    /// [`WriteDisposition::Replayed`].
    fn put<'a>(
        &'a mut self,
        name: &'a ResourceName,
        resource: Self::Resource,
        idempotency_key: &'a IdempotencyKey,
    ) -> Pin<
        Box<dyn Future<Output = Result<PutOutcome<Self::Resource>, ProviderError>> + Send + 'a>,
    >;

    /// Read `name`, exactly as last written.
    fn get<'a>(
        &'a self,
        name: &'a ResourceName,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Resource, ProviderError>> + Send + 'a>>;

    /// List a `collection` page in a stable total order.
    fn list<'a>(
        &'a self,
        collection: &'a str,
        request: &'a PageRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<Page<ListEntry<Self::Resource>>, ProviderError>> + Send + 'a>,
    >;

    /// Async delete of `name`: returns an AIP-151 operation. Replays under
    /// the same idempotency key MUST return the SAME operation resource.
    fn delete<'a>(
        &'a mut self,
        name: &'a ResourceName,
        idempotency_key: &'a IdempotencyKey,
    ) -> Pin<Box<dyn Future<Output = Result<Operation, ProviderError>> + Send + 'a>>;

    /// Poll an operation by name. Terminal operations are immutable.
    fn poll_operation<'a>(
        &'a mut self,
        operation_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Operation, ProviderError>> + Send + 'a>>;
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let pending = Operation::pending("operations/op-1").unwrap();
        assert!(!pending.done);
        assert!(pending.result.is_none());
        pending.validate().unwrap();

        let ok = Operation::succeeded("operations/op-1", serde_json::json!({})).unwrap();
        assert!(ok.done);
        ok.validate().unwrap();

        let failed = Operation::failed(
            "operations/op-2",
            OperationError {
                code: "failed_precondition".to_owned(),
                message: "resource busy".to_owned(),
            },
        )
        .unwrap();
        assert!(matches!(failed.result, Some(OperationResult::Error(_))));

        assert!(Operation::pending("op-without-prefix").is_err());
        assert!(Operation::pending("operations/").is_err());

        let forged = Operation {
            name: "operations/op-3".to_owned(),
            done: true,
            result: None,
        };
        assert!(forged.validate().is_err());
    }
}
