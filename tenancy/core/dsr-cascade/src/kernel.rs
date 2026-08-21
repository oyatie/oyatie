//! DSR cascade kernel: entities, value objects and ports.
//!
//! Every type the Wave-15 scaffold published (`DsrRequestId`, `DsrKind`,
//! `DsrRequest`, `ErasureReceipt`, `ProofOfErasure`, `DsrRequestRepository`,
//! `DsrKernelError`) is preserved here and re-exported from the crate root,
//! so the published contract is unchanged; fields, ports and error variants
//! were added around it.
//!
//! # Tenancy is part of the identity
//!
//! A [`DsrRequestId`] is a CALLER-SUPPLIED string with no uniqueness rule
//! beyond "not blank", so per-tenant numbering (`dsr-1`, `dsr-2`, …) collides
//! across tenants by accident, not only by attack. Nothing in this crate may
//! therefore address a request by its id alone: [`DsrRequestKey`] pairs the
//! tenant with the id, every repository read takes a key, and the tenant is
//! bound into the Merkle leaf so a receipt cannot be replayed under another
//! tenant.

use core::fmt;

/// Largest handler-supplied failure detail this crate will carry.
///
/// [`HandlerFailure::detail`] is the only field in the crate whose contents
/// are wholly third-party controlled: it is whatever an arbitrary downstream
/// microservice's erasure error path produced, and it commonly quotes the
/// failing statement, which quotes the subject. It is bounded on the way in
/// ([`HandlerFailure::new`]) and on the way out
/// ([`HandlerFailure::bounded_detail`], which the cascade runner and the
/// [`fmt::Display`] impl both use) so an unbounded, unreviewed string cannot
/// ride the error channel out of the trust boundary.
pub const MAX_HANDLER_DETAIL_BYTES: usize = 256;

/// Marker appended to a detail that was cut at [`MAX_HANDLER_DETAIL_BYTES`].
const TRUNCATION_MARKER: &str = "…[truncated]";

/// Seconds since the Unix epoch.
///
/// Time is always a PARAMETER in this crate: nothing under `domain` or
/// `usecase` reads a clock, so every SLA decision is reproducible from its
/// inputs alone.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Timestamp(pub i64); // data_class: INTERNAL_ONLY

impl Timestamp {
    /// Whole seconds from `self` to `other`, or `None` on i64 overflow.
    #[must_use]
    pub const fn seconds_until(self, other: Self) -> Option<i64> {
        other.0.checked_sub(self.0)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}s", self.0)
    }
}

/// Identifier of a data-subject request. TENANT-LOCAL: see
/// [`DsrRequestKey`], which is the only thing that addresses a request.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct DsrRequestId(pub String); // data_class: INTERNAL_ONLY

impl fmt::Display for DsrRequestId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// The globally unique identity of a DSR request: tenant plus tenant-local
/// request id.
///
/// Every repository read and every Merkle computation is keyed on this, so
/// two tenants using the same request id can never touch one another's
/// receipts or discharge one another's erasure obligation.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct DsrRequestKey {
    pub tenant: String,        // data_class: INTERNAL_ONLY
    pub request: DsrRequestId, // data_class: INTERNAL_ONLY
}

impl DsrRequestKey {
    /// Build a key from its parts.
    #[must_use]
    pub fn new(tenant: &str, request: &DsrRequestId) -> Self {
        Self {
            tenant: tenant.to_owned(),
            request: request.clone(),
        }
    }
}

impl fmt::Display for DsrRequestKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}/{}", self.tenant, self.request)
    }
}

/// The regulatory pack a request is judged under; it fixes the statutory
/// response window (see [`crate::domain::sla_deadline`]).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum RegulatoryPack {
    /// EU GDPR.
    Eu,
    /// South Korea PIPA.
    Kr,
    /// India DPDPA.
    In,
    /// Brazil LGPD (15-day window).
    Br,
    /// US healthcare under a BAA (7-day window).
    UsHc,
    /// Any pack without a shorter bespoke window; defaults to 30 days.
    Default,
}

/// Which right under GDPR Art. 15-20 / LGPD / DPDPA / CCPA is exercised.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DsrKind {
    /// Right of access.
    Access,
    /// Right to erasure (GDPR Art. 17) — the only kind this cascade runs.
    Erasure,
    /// Right to rectification.
    Rectification,
    /// Right to data portability.
    Portability,
}

/// A submitted data-subject request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DsrRequest {
    pub id: DsrRequestId,        // data_class: INTERNAL_ONLY
    pub tenant_id: String,       // data_class: INTERNAL_ONLY
    pub subject_id: String,      // data_class: PII_QUASI_IDENTIFIER
    pub kind: DsrKind,           // data_class: INTERNAL_ONLY
    pub pack: RegulatoryPack,    // data_class: INTERNAL_ONLY
    pub requested_at: Timestamp, // data_class: INTERNAL_ONLY
}

/// One microservice's signed-off proof that it erased the subject.
///
/// `merkle_leaf` is the evidence digest the microservice itself produced
/// (what it deleted, over which keyspace). The cascade never trusts it as
/// the tree leaf directly: the tree leaf is a domain-separated hash that
/// also binds the tenant, the request id and the microservice name, so a
/// receipt cannot be replayed under a different tenant, request or service.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErasureReceipt {
    pub tenant: String,        // data_class: INTERNAL_ONLY
    pub request: DsrRequestId, // data_class: INTERNAL_ONLY
    pub microservice: String,  // data_class: INTERNAL_ONLY
    pub merkle_leaf: [u8; 32], // data_class: INTERNAL_ONLY
}

impl ErasureReceipt {
    /// The request identity this receipt belongs to.
    #[must_use]
    pub fn key(&self) -> DsrRequestKey {
        DsrRequestKey::new(&self.tenant, &self.request)
    }
}

/// A dual-control (two-person rule) waiver allowing a proof to be sealed
/// while receipts are still missing. Halt condition of IP-009: without one
/// of these, an incomplete cascade may never be sealed.
///
/// The approver fields name natural persons, so they are classed as
/// quasi-identifying rather than as internal constants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DpoOverride {
    pub first_approver: String,  // data_class: PII_QUASI_IDENTIFIER
    pub second_approver: String, // data_class: PII_QUASI_IDENTIFIER
    pub reason: String,          // data_class: INTERNAL_ONLY
}

/// The sealed certificate: a Merkle root over every microservice receipt.
///
/// `covered_microservices` is the cascade plan the certificate ASSERTS
/// coverage of, stated by name rather than by count, so a regulator can read
/// what was claimed instead of inferring it from an integer.
/// `expected_microservices` is retained as its cardinality.
///
/// `receipts` may legitimately be a SUPERSET of `covered_microservices`: a
/// microservice decommissioned mid-window leaves a genuine receipt behind
/// that the current plan no longer names, and surplus evidence must not make
/// the certificate unobtainable (see [`crate::domain::compute_proof_of_erasure`]).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofOfErasure {
    pub tenant: String,                     // data_class: INTERNAL_ONLY
    pub request: DsrRequestId,              // data_class: INTERNAL_ONLY
    pub merkle_root: [u8; 32],              // data_class: INTERNAL_ONLY
    pub receipts: Vec<ErasureReceipt>,      // data_class: INTERNAL_ONLY
    pub covered_microservices: Vec<String>, // data_class: INTERNAL_ONLY
    pub expected_microservices: usize,      // data_class: INTERNAL_ONLY
    pub sealed_at: Timestamp,               // data_class: INTERNAL_ONLY
    pub dpo_override: Option<DpoOverride>,  // data_class: INTERNAL_ONLY
}

impl ProofOfErasure {
    /// The request identity this certificate is about.
    #[must_use]
    pub fn key(&self) -> DsrRequestKey {
        DsrRequestKey::new(&self.tenant, &self.request)
    }
}

/// Why one microservice's erasure handler could not complete.
///
/// `detail` is third-party text. A handler MUST NOT put subject identifiers
/// in it; this crate cannot enforce that, so it bounds the field instead and
/// classes it as quasi-identifying, which is what a downstream sink needs to
/// know before it persists or forwards the string.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandlerFailure {
    pub detail: String, // data_class: PII_QUASI_IDENTIFIER
}

impl HandlerFailure {
    /// A failure whose detail is bounded at [`MAX_HANDLER_DETAIL_BYTES`].
    #[must_use]
    pub fn new(detail: &str) -> Self {
        Self {
            detail: bound_detail(detail),
        }
    }

    /// The detail, bounded at [`MAX_HANDLER_DETAIL_BYTES`].
    ///
    /// `detail` is a public field, so a handler can bypass
    /// [`HandlerFailure::new`]; every read path in this crate goes through
    /// here instead of through the field.
    #[must_use]
    pub fn bounded_detail(&self) -> String {
        bound_detail(&self.detail)
    }
}

/// Cut `detail` to [`MAX_HANDLER_DETAIL_BYTES`] on a UTF-8 char boundary,
/// marking the cut so a truncated diagnostic cannot be mistaken for a
/// complete one.
fn bound_detail(detail: &str) -> String {
    if detail.len() <= MAX_HANDLER_DETAIL_BYTES {
        return detail.to_owned();
    }
    let mut end = MAX_HANDLER_DETAIL_BYTES;
    while end > 0 && !detail.is_char_boundary(end) {
        end -= 1;
    }
    let mut bounded = detail.get(..end).unwrap_or_default().to_owned();
    bounded.push_str(TRUNCATION_MARKER);
    bounded
}

impl fmt::Display for HandlerFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "erasure handler failed: {}",
            self.bounded_detail()
        )
    }
}

impl std::error::Error for HandlerFailure {}

/// A microservice's erasure handler: erase the subject and return the
/// evidence digest of what was erased.
pub trait ErasureHandler {
    /// Erase the subject named by `request` from this microservice's stores.
    ///
    /// Implementations MUST be idempotent: the cascade guards against
    /// re-invocation, but a crashed run may still land here twice.
    ///
    /// A returned [`HandlerFailure::detail`] MUST NOT contain subject
    /// identifiers or record contents; it is carried into cascade state and
    /// rendered by [`HandlerFailure`]'s `Display`, and this crate can only
    /// bound its length, not judge its contents.
    fn erase(&self, request: &DsrRequest) -> Result<[u8; 32], HandlerFailure>;
}

impl<T: ErasureHandler + ?Sized> ErasureHandler for std::sync::Arc<T> {
    fn erase(&self, request: &DsrRequest) -> Result<[u8; 32], HandlerFailure> {
        (**self).erase(request)
    }
}

/// The catalog of microservices a cascade must fan out to.
pub trait MicroserviceRegistry {
    /// Every microservice currently holding tenant data, in any order; the
    /// cascade planner imposes the deterministic order itself.
    fn list_active(&self) -> Result<Vec<String>, DsrKernelError>;

    /// The registered erasure handler for `microservice`, if it has one.
    ///
    /// `None` is the IP-009 halt condition "microservice registered without
    /// a DSR handler" and surfaces as a failed cascade step, never as a
    /// silently short tree.
    fn handler(&self, microservice: &str) -> Option<&dyn ErasureHandler>;
}

/// Durable state for DSR requests, their receipts and their proofs.
///
/// Every read is keyed on [`DsrRequestKey`], never on a bare
/// [`DsrRequestId`]: an id is tenant-local and collides across tenants.
pub trait DsrRequestRepository {
    /// Register a request. Re-opening the same (tenant, id) is a no-op, not
    /// an error, so a retried submission does not lose the original
    /// receipts. A DIFFERENT tenant presenting the same id opens a separate
    /// record.
    fn open(&self, request: &DsrRequest) -> Result<(), DsrKernelError>;

    /// Append a receipt. Rejects a second receipt for the same
    /// (tenant, request, microservice) triple with
    /// [`DsrKernelError::DuplicateReceipt`].
    fn append_receipt(&self, receipt: &ErasureReceipt) -> Result<(), DsrKernelError>;

    /// Seal the proof for a request. Rejects a second seal.
    fn finalize(&self, proof: &ProofOfErasure) -> Result<(), DsrKernelError>;

    /// The receipt already recorded for one microservice, if any. This is
    /// the read the cascade uses to stay idempotent WITHOUT re-invoking a
    /// handler that already ran.
    fn receipt(
        &self,
        key: &DsrRequestKey,
        microservice: &str,
    ) -> Result<Option<ErasureReceipt>, DsrKernelError>;

    /// Every receipt recorded for a request, in canonical (microservice
    /// name ascending) order.
    fn receipts(&self, key: &DsrRequestKey) -> Result<Vec<ErasureReceipt>, DsrKernelError>;

    /// The sealed proof for a request, if one exists.
    fn proof(&self, key: &DsrRequestKey) -> Result<Option<ProofOfErasure>, DsrKernelError>;
}

/// Every way the DSR cascade refuses.
///
/// The aggregation failures are DISTINCT variants on purpose: an operator
/// reading "receipts could not be aggregated" cannot tell a decommissioned
/// microservice from a tampered certificate, and those two demand opposite
/// responses.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DsrKernelError {
    /// No such request has been opened for this tenant.
    UnknownRequest,
    /// A receipt already exists for this (tenant, request, microservice).
    DuplicateReceipt,
    /// A leaf set could not be reduced to a Merkle root.
    MerkleAggregationFailed,
    /// The receipt set is empty: there is no proof of erasure over nothing.
    EmptyReceiptSet,
    /// Two receipts claim the same microservice, so the tree is ambiguous.
    DuplicateMicroserviceReceipt,
    /// A receipt belongs to another tenant or another request.
    ForeignReceipt,
    /// A receipt field is too long to encode canonically.
    ReceiptEncodingTooLarge,
    /// A certificate's recomputed Merkle root does not match its sealed
    /// root: receipts were added, removed or altered after sealing.
    RootMismatch,
    /// A certificate's own fields disagree (coverage list not canonical, or
    /// the stated count does not match the coverage list).
    InconsistentProof,
    /// The plan does not belong to the request it was passed with.
    PlanRequestMismatch,
    /// The statutory response window closed with the cascade incomplete.
    ///
    /// Carries the diagnosis the on-call responder needs at the moment a
    /// legal deadline is breached, rather than making them re-derive it.
    SlaBreached {
        /// Tenant that owns the breached request.
        tenant: String,
        /// The breached request.
        request: DsrRequestId,
        /// Microservices that still owe a receipt, in plan order.
        pending: Vec<String>,
        /// The statutory deadline that passed.
        deadline: Timestamp,
        /// The instant the breach was evaluated at.
        now: Timestamp,
    },
    /// The request is structurally invalid (empty id / tenant / subject).
    InvalidRequest,
    /// The cascade runs erasure only; other DSR kinds have their own flows.
    UnsupportedKind,
    /// The registry named no active microservice, so there is nothing to
    /// prove and an "empty" proof would be a lie.
    EmptyCascadePlan,
    /// A proof was already sealed for this request.
    AlreadyFinalized,
    /// Sealing an incomplete cascade needs a valid two-person DPO override.
    DpoOverrideRequired,
    /// The supplied override is not dual control (missing or equal approvers,
    /// or no stated reason).
    InvalidDpoOverride,
    /// Timestamp arithmetic left the representable range.
    TimestampOverflow,
    /// The backing store could not be reached (poisoned lock, lost handle).
    RepositoryUnavailable,
}

impl DsrKernelError {
    /// Whether this error ends the whole cascade rather than one step.
    ///
    /// A step-level error (an unreachable store, a lost append race) must
    /// not abort the pass: the microservices after it in plan order are
    /// still owed their erasure, and starving them is the exact failure the
    /// per-step design exists to prevent. A request-level error (the request
    /// is unknown, invalid, or already sealed) is true of every step, so
    /// continuing would only repeat it.
    #[must_use]
    pub const fn is_request_terminal(&self) -> bool {
        matches!(
            self,
            Self::UnknownRequest
                | Self::AlreadyFinalized
                | Self::InvalidRequest
                | Self::UnsupportedKind
                | Self::PlanRequestMismatch
        )
    }
}

impl fmt::Display for DsrKernelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownRequest => formatter.write_str("no such DSR request for this tenant"),
            Self::DuplicateReceipt => {
                formatter.write_str("a receipt already exists for this microservice")
            }
            Self::MerkleAggregationFailed => {
                formatter.write_str("leaves could not be aggregated into a Merkle root")
            }
            Self::EmptyReceiptSet => {
                formatter.write_str("no receipts to aggregate into a proof of erasure")
            }
            Self::DuplicateMicroserviceReceipt => {
                formatter.write_str("two receipts claim the same microservice")
            }
            Self::ForeignReceipt => {
                formatter.write_str("a receipt belongs to another tenant or request")
            }
            Self::ReceiptEncodingTooLarge => {
                formatter.write_str("a receipt field is too long to encode canonically")
            }
            Self::RootMismatch => formatter.write_str(
                "the certificate's recomputed Merkle root does not match its sealed root",
            ),
            Self::InconsistentProof => {
                formatter.write_str("the certificate's own coverage fields disagree")
            }
            Self::PlanRequestMismatch => {
                formatter.write_str("the cascade plan does not belong to this request")
            }
            Self::SlaBreached {
                tenant,
                request,
                pending,
                deadline,
                now,
            } => write!(
                formatter,
                "the statutory DSR deadline passed with the cascade incomplete: \
                 tenant {tenant}, request {request}, deadline {deadline}, now {now}, \
                 still owing receipts: [{}]",
                pending.join(", ")
            ),
            Self::InvalidRequest => formatter.write_str("the DSR request is structurally invalid"),
            Self::UnsupportedKind => formatter.write_str("this cascade runs erasure requests only"),
            Self::EmptyCascadePlan => formatter.write_str("no active microservice to cascade to"),
            Self::AlreadyFinalized => {
                formatter.write_str("a proof of erasure is already sealed for this request")
            }
            Self::DpoOverrideRequired => {
                formatter.write_str("an incomplete cascade needs a DPO override to seal")
            }
            Self::InvalidDpoOverride => {
                formatter.write_str("the DPO override is not valid dual control")
            }
            Self::TimestampOverflow => formatter.write_str("timestamp arithmetic overflowed"),
            Self::RepositoryUnavailable => formatter.write_str("the DSR repository is unavailable"),
        }
    }
}

impl std::error::Error for DsrKernelError {}

impl DsrRequest {
    /// The globally unique identity of this request.
    #[must_use]
    pub fn key(&self) -> DsrRequestKey {
        DsrRequestKey::new(&self.tenant_id, &self.id)
    }

    /// Reject structurally impossible requests before anything is opened.
    ///
    /// # Errors
    /// [`DsrKernelError::InvalidRequest`] when the id, tenant or subject is
    /// blank; [`DsrKernelError::UnsupportedKind`] for a non-erasure kind.
    pub fn validate_for_erasure(&self) -> Result<(), DsrKernelError> {
        if self.id.0.trim().is_empty()
            || self.tenant_id.trim().is_empty()
            || self.subject_id.trim().is_empty()
        {
            return Err(DsrKernelError::InvalidRequest);
        }
        if self.kind != DsrKind::Erasure {
            return Err(DsrKernelError::UnsupportedKind);
        }
        Ok(())
    }
}

impl DpoOverride {
    /// Check the two-person rule: two distinct, named approvers and a
    /// stated reason.
    ///
    /// # Errors
    /// [`DsrKernelError::InvalidDpoOverride`] when dual control is not met.
    pub fn validate(&self) -> Result<(), DsrKernelError> {
        let first = self.first_approver.trim();
        let second = self.second_approver.trim();
        if first.is_empty() || second.is_empty() || first == second || self.reason.trim().is_empty()
        {
            return Err(DsrKernelError::InvalidDpoOverride);
        }
        Ok(())
    }
}
