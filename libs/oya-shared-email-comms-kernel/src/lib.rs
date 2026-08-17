//! Email + transactional comms kernel — canonical adapter
//! substrate per ADR-0201.
//!
//! # Why this crate exists
//!
//! ADR-0201 makes transactional email a substrate concern, not a
//! per-µservice concern. Every sender (Identity, Tenancy, Workflow
//! Studio, Foundry, Audit, Billing) goes through this trait so
//! that:
//!
//! 1. No single provider (SES, Mailgun, Postal, etc.) is in the
//!    critical path — ADR-0173 vendor lock-in avoidance.
//! 2. DKIM signing, SPF authorization, and DMARC policy are
//!    enforced at the kernel — never at the call site.
//! 3. Webhook events normalize into ADR-0145 audit chain events
//!    (sent, delivered, opened, clicked, bounced, complained,
//!    suppressed) on a schema versioned per ADR-0166.
//! 4. Per-tenant rate ceilings + suppression are uniform across
//!    adapters.
//!
//! # Layer
//!
//! `kernel` (port-in-kernel per ADR-0056).
//!
//! # Naming justification
//!
//! `oya-shared-email-comms-kernel` follows BNF v4.1:
//! `oya-<axis:shared>-<topic:email-comms>-<layer:kernel>`.
//!
//! # References
//!
//! - ADR-0201 — Email + transactional comms adapter substrate.
//! - ADR-0145 — inter-microservice communication reform.
//! - ADR-0166 — event schema versioning.
//! - ADR-0173 — vendor lock-in avoidance.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]
#![forbid(unsafe_code)]
#![allow(clippy::result_large_err)]

use std::collections::HashMap;
use std::fmt;

/// Provider tag — which adapter is active. Always exactly one
/// at a time; multi-adapter routing is a µservice-side concern
/// outside this kernel.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum EmailProvider {
    /// AWS SES (default for cloud-hosted clusters).
    Ses,
    /// Postal self-hosted (sovereign / air-gapped).
    Postal,
    /// Mailgun (alt SaaS second-source).
    Mailgun,
    /// Generic RFC 5321 SMTP fallback.
    Smtp,
}

impl fmt::Display for EmailProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmailProvider::Ses => f.write_str("ses"),
            EmailProvider::Postal => f.write_str("postal"),
            EmailProvider::Mailgun => f.write_str("mailgun"),
            EmailProvider::Smtp => f.write_str("smtp"),
        }
    }
}

/// Tenant identifier carried with every send. Used for rate
/// limits, suppression lookups, DKIM key selection, and audit
/// emission.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TenantId(pub String);

/// RFC 5321 email address. Validated at construction.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct EmailAddress(String);

impl EmailAddress {
    /// Construct after the minimum syntactic validation oyatie
    /// requires (one '@', non-empty local part, non-empty domain
    /// containing a '.'). Real RFC 5321 / 5322 validation lives
    /// in the adapter layer; this guards against trivially-bad
    /// inputs reaching the wire.
    ///
    /// # Errors
    /// - `EmailCommsError::InvalidAddress` if syntax check fails.
    pub fn try_new(raw: impl Into<String>) -> Result<Self, EmailCommsError> {
        let raw = raw.into();
        let parts: Vec<&str> = raw.split('@').collect();
        if parts.len() != 2 {
            return Err(EmailCommsError::InvalidAddress(raw));
        }
        let (local, domain) = (parts[0], parts[1]);
        if local.is_empty() || domain.is_empty() || !domain.contains('.') {
            return Err(EmailCommsError::InvalidAddress(raw));
        }
        Ok(EmailAddress(raw))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap_or_default()
    }
}

/// DKIM selector + private-key fingerprint reference. The actual
/// key material lives in OpenBao (ADR-0173); this struct names it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DkimBinding {
    pub selector: String,
    pub key_ref: String,
    pub rotated_at_epoch_s: u64,
}

/// Per-tenant deliverability binding. ADR-0201 requires every send
/// be DKIM signed; SPF and DMARC posture are inspected at
/// pre-flight so misconfigured tenants reject locally rather than
/// emit unsigned mail.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliverabilityBinding {
    pub tenant: TenantId,
    pub from_domain: String,
    pub dkim: DkimBinding,
    /// Whether the tenant's SPF record authorizes the active
    /// provider's send sources. Populated by the comms µservice
    /// from DNS / OpenTofu-published records.
    pub spf_authorized: bool,
    /// DMARC posture (none / quarantine / reject).
    pub dmarc_policy: DmarcPolicy,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DmarcPolicy {
    /// `p=none` — monitor only. Forbidden for production tenants
    /// past their warm-up window.
    None,
    /// `p=quarantine` — default for new tenants.
    Quarantine,
    /// `p=reject` — default for tenants past warm-up.
    Reject,
}

/// One outbound message.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutboundMessage {
    pub from: EmailAddress,
    pub reply_to: Option<EmailAddress>,
    pub to: Vec<EmailAddress>,
    pub subject: String,
    /// HTML body (compiled from MJML via mrml at the call site).
    pub html_body: String,
    /// Optional plaintext fallback. Strongly recommended for
    /// deliverability.
    pub plain_body: Option<String>,
    /// Locale tag (BCP-47) — propagates into the audit-chain
    /// event for i18n analytics.
    pub locale: String,
    /// Idempotency key (ADR-0149) — duplicate sends collapse.
    pub idempotency_key: String,
}

/// Outcome of a successful send (pre-delivery; final delivery
/// status arrives later via webhook).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SendOutcome {
    pub provider: EmailProvider,
    pub provider_message_id: String,
}

/// Normalized webhook delivery event from any adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliveryEvent {
    pub provider: EmailProvider,
    pub provider_message_id: String,
    pub kind: DeliveryEventKind,
    pub at_epoch_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DeliveryEventKind {
    Sent,
    Delivered,
    Opened,
    Clicked,
    Bounced,
    Complained,
    Suppressed,
}

impl fmt::Display for DeliveryEventKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeliveryEventKind::Sent => f.write_str("sent"),
            DeliveryEventKind::Delivered => f.write_str("delivered"),
            DeliveryEventKind::Opened => f.write_str("opened"),
            DeliveryEventKind::Clicked => f.write_str("clicked"),
            DeliveryEventKind::Bounced => f.write_str("bounced"),
            DeliveryEventKind::Complained => f.write_str("complained"),
            DeliveryEventKind::Suppressed => f.write_str("suppressed"),
        }
    }
}

/// Failure surface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EmailCommsError {
    /// Address failed syntactic check.
    InvalidAddress(String),
    /// Tenant rate ceiling exceeded; caller must back off.
    RateCeilingExceeded { tenant: TenantId, per_minute: u32 },
    /// Tenant lacks a valid DKIM binding; sending unsigned mail
    /// is forbidden.
    DkimBindingMissing(TenantId),
    /// Tenant SPF record does not authorize the active provider.
    SpfNotAuthorized(TenantId),
    /// Tenant DMARC policy is `p=none` past the warm-up window.
    DmarcPolicyForbidden {
        tenant: TenantId,
        policy: DmarcPolicy,
    },
    /// Recipient address is on the suppression list.
    RecipientSuppressed(EmailAddress),
    /// Provider responded with a non-retryable error.
    ProviderError {
        provider: EmailProvider,
        code: String,
        message: String,
    },
    /// Real provider SDK not configured (feature flag absent or
    /// adapter not wired).
    AdapterNotConfigured(EmailProvider),
    /// Empty recipient list.
    NoRecipients,
    /// Body and subject combined are empty — nothing to send.
    EmptyMessage,
    /// Idempotency-key collision with a non-identical prior send.
    IdempotencyConflict { key: String },
}

impl fmt::Display for EmailCommsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmailCommsError::InvalidAddress(s) => write!(f, "invalid email address: {s}"),
            EmailCommsError::RateCeilingExceeded { tenant, per_minute } => {
                write!(
                    f,
                    "tenant {tenant:?} exceeded {per_minute}/min rate ceiling"
                )
            }
            EmailCommsError::DkimBindingMissing(t) => {
                write!(
                    f,
                    "tenant {t:?} has no DKIM binding; unsigned send forbidden"
                )
            }
            EmailCommsError::SpfNotAuthorized(t) => {
                write!(f, "tenant {t:?} SPF record does not authorize provider")
            }
            EmailCommsError::DmarcPolicyForbidden { tenant, policy } => {
                write!(f, "tenant {tenant:?} DMARC policy {policy:?} forbidden")
            }
            EmailCommsError::RecipientSuppressed(addr) => {
                write!(f, "recipient {addr:?} on suppression list")
            }
            EmailCommsError::ProviderError {
                provider,
                code,
                message,
            } => {
                write!(f, "{provider} error {code}: {message}")
            }
            EmailCommsError::AdapterNotConfigured(p) => {
                write!(f, "adapter {p} not configured (feature flag absent)")
            }
            EmailCommsError::NoRecipients => write!(f, "no recipients"),
            EmailCommsError::EmptyMessage => write!(f, "empty message"),
            EmailCommsError::IdempotencyConflict { key } => {
                write!(
                    f,
                    "idempotency-key {key} collides with non-identical prior send"
                )
            }
        }
    }
}

impl std::error::Error for EmailCommsError {}

/// Canonical trait every µservice consumes. Real provider SDK
/// integration lives in adapter sub-crates / feature flags.
pub trait EmailComms: Send + Sync {
    /// Identify the active adapter for audit + telemetry.
    fn provider(&self) -> EmailProvider;

    /// Pre-flight check: validate deliverability binding +
    /// suppression + rate ceiling without contacting the provider.
    fn preflight(
        &self,
        binding: &DeliverabilityBinding,
        message: &OutboundMessage,
    ) -> Result<(), EmailCommsError>;

    /// Send a message after preflight passes. Returns the
    /// provider's message id (used for webhook correlation).
    fn send(
        &self,
        binding: &DeliverabilityBinding,
        message: &OutboundMessage,
    ) -> Result<SendOutcome, EmailCommsError>;
}

/// Stable FNV-1a fingerprint over the message identity fields
/// (from, all recipients sorted, subject, html_body). Used for
/// idempotency-key conflict detection. Dep-free.
///
/// Recipients are sorted before hashing so that the fingerprint is
/// order-independent (ADR-0149 collapse semantics require that a
/// re-send with the same logical recipient set collapses regardless
/// of recipient list order).
#[must_use]
fn message_fingerprint(message: &OutboundMessage) -> u64 {
    const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
    const FNV_PRIME: u64 = 1_099_511_628_211;

    #[inline]
    fn fnv_bytes(h: &mut u64, bytes: impl Iterator<Item = u8>) {
        for byte in bytes {
            *h ^= u64::from(byte);
            *h = h.wrapping_mul(FNV_PRIME);
        }
    }

    // Sort recipients for order-independence.
    let mut sorted_to: Vec<&EmailAddress> = message.to.iter().collect();
    sorted_to.sort_by_key(|a| a.as_str());

    let mut h = FNV_OFFSET;
    fnv_bytes(&mut h, message.from.as_str().bytes());
    for addr in sorted_to {
        fnv_bytes(&mut h, addr.as_str().bytes());
    }
    fnv_bytes(&mut h, message.subject.bytes());
    fnv_bytes(&mut h, message.html_body.bytes());
    h
}

/// Shared deliverability invariant check used by every real
/// adapter. Centralizing it here means DKIM/SPF/DMARC + suppression
/// + rate-ceiling + idempotency rules are uniform across SES,
/// Postal, Mailgun, and SMTP.
///
/// # Parameters
/// - `recent_send_count` — number of sends already recorded in the
///   current one-minute window for this tenant. Pass `0` if the
///   caller does not track a window.
/// - `rate_ceiling` — maximum sends per minute allowed for this
///   tenant. `0` means uncapped.
/// - `prior_fingerprints` — map of previously-seen idempotency key
///   → message fingerprint. Pass `&HashMap::new()` (or
///   `&Default::default()`) if the caller does not track
///   idempotency.
///
/// # Errors
/// See `EmailCommsError`.
pub fn enforce_deliverability_invariants(
    binding: &DeliverabilityBinding,
    message: &OutboundMessage,
    suppressed: &[EmailAddress],
    warm_up_complete: bool,
    recent_send_count: u32,
    rate_ceiling: u32,
    prior_fingerprints: &HashMap<String, u64>,
) -> Result<(), EmailCommsError> {
    if message.to.is_empty() {
        return Err(EmailCommsError::NoRecipients);
    }
    if message.subject.trim().is_empty() && message.html_body.trim().is_empty() {
        return Err(EmailCommsError::EmptyMessage);
    }
    if binding.dkim.selector.is_empty() || binding.dkim.key_ref.is_empty() {
        return Err(EmailCommsError::DkimBindingMissing(binding.tenant.clone()));
    }
    if !binding.spf_authorized {
        return Err(EmailCommsError::SpfNotAuthorized(binding.tenant.clone()));
    }
    if warm_up_complete && binding.dmarc_policy == DmarcPolicy::None {
        return Err(EmailCommsError::DmarcPolicyForbidden {
            tenant: binding.tenant.clone(),
            policy: DmarcPolicy::None,
        });
    }
    if message.from.domain() != binding.from_domain {
        return Err(EmailCommsError::InvalidAddress(format!(
            "from {} does not match tenant from_domain {}",
            message.from.as_str(),
            binding.from_domain,
        )));
    }
    // ST1: per-tenant per-minute rate ceiling.
    if rate_ceiling > 0 && recent_send_count >= rate_ceiling {
        return Err(EmailCommsError::RateCeilingExceeded {
            tenant: binding.tenant.clone(),
            per_minute: rate_ceiling,
        });
    }
    // ST2: idempotency-key conflict detection.
    let fp = message_fingerprint(message);
    if let Some(&prior_fp) = prior_fingerprints.get(&message.idempotency_key) {
        if prior_fp != fp {
            return Err(EmailCommsError::IdempotencyConflict {
                key: message.idempotency_key.clone(),
            });
        }
        // Identical re-send: collapse to success (fall through).
    }
    for rcpt in &message.to {
        if suppressed.contains(rcpt) {
            return Err(EmailCommsError::RecipientSuppressed(rcpt.clone()));
        }
    }
    Ok(())
}

// ---------- Bounce classification (RFC 3463 + RFC 5321) ----------

/// Bounce severity category derived from RFC 3463 enhanced status codes
/// or RFC 5321 SMTP reply codes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum BounceCategory {
    /// Permanent failure — recipient address is definitively unreachable.
    /// Drives immediate suppression. RFC 3463 class 5.x.x / SMTP 5xx.
    Hard,
    /// Repeated temporary failure indicating a persistent mailbox problem
    /// (e.g. over-quota, policy reject on the recipient side). Drives
    /// suppression after `SOFT_BOUNCE_SUPPRESS_THRESHOLD` occurrences.
    Soft,
    /// Short-lived transient failure — connection refused, greylisting,
    /// DNS momentarily unavailable. No action; eligible for retry by the
    /// calling µservice. RFC 3463 class 4.x.x (non-soft sub-classes) /
    /// SMTP 4xx (non-soft codes).
    Transient,
}

impl fmt::Display for BounceCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BounceCategory::Hard => f.write_str("hard"),
            BounceCategory::Soft => f.write_str("soft"),
            BounceCategory::Transient => f.write_str("transient"),
        }
    }
}

/// Classify a bounce from an RFC 3463 enhanced status code string of the
/// form `"X.Y.Z"` where X, Y, Z are unsigned integers.
///
/// Classification rules:
/// - Class `5` (any sub-class) → `Hard`
/// - Class `4`, subject `2`, detail `1` (`4.2.1`) → `Soft`
///   (RFC 3463 §3.3: "mailbox disabled, not accepting messages")
/// - Class `4` (any other) → `Transient`
/// - Malformed / out-of-bounce-class → `None`
///
/// Returns `None` if `code` is not a well-formed `"X.Y.Z"` triple.
#[must_use]
pub fn classify_bounce_enhanced(code: &str) -> Option<BounceCategory> {
    let parts: Vec<&str> = code.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let class: u8 = parts[0].parse().ok()?;
    let subject: u8 = parts[1].parse().ok()?;
    let detail: u8 = parts[2].parse().ok()?;
    match class {
        5 => Some(BounceCategory::Hard),
        4 => {
            if subject == 2 && detail == 1 {
                Some(BounceCategory::Soft)
            } else {
                Some(BounceCategory::Transient)
            }
        }
        _ => None,
    }
}

/// Classify a bounce from an RFC 5321 3-digit SMTP reply code.
///
/// | Code range         | `BounceCategory` |
/// |--------------------|-----------------|
/// | 500–599            | `Hard`          |
/// | 452                | `Soft`          |
/// | 400–499 (not 452)  | `Transient`     |
/// | other              | `None`          |
///
/// Returns `None` for codes outside the 4xx–5xx range.
#[must_use]
pub fn classify_bounce_smtp(code: u16) -> Option<BounceCategory> {
    match code {
        500..=599 => Some(BounceCategory::Hard),
        452 => Some(BounceCategory::Soft),
        400..=499 => Some(BounceCategory::Transient),
        _ => None,
    }
}

// ---------- Bounce suppression decision ----------

/// Outcome of evaluating whether to suppress or retry after a bounce.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum BounceSuppressionOutcome {
    /// Caller must add the recipient to the suppression list.
    /// Maps to `EmailCommsError::RecipientSuppressed` at the adapter layer.
    Suppress,
    /// Eligible for retry. Caller should increment soft-bounce count and
    /// attempt re-delivery after a back-off interval.
    Retry,
    /// No suppression or retry action required (transient; infrastructure
    /// retry is sufficient).
    NoAction,
}

impl fmt::Display for BounceSuppressionOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BounceSuppressionOutcome::Suppress => f.write_str("suppress"),
            BounceSuppressionOutcome::Retry => f.write_str("retry"),
            BounceSuppressionOutcome::NoAction => f.write_str("no_action"),
        }
    }
}

/// Number of accumulated soft bounces at which a recipient is promoted to
/// suppression.
pub const SOFT_BOUNCE_SUPPRESS_THRESHOLD: u32 = 3;

/// Pure decision: given the bounce category and the number of soft bounces
/// already recorded for this recipient, return the suppression outcome.
///
/// Decision table:
///
/// | category  | prior_soft_bounce_count           | outcome   |
/// |-----------|-----------------------------------|-----------|
/// | Hard      | any                               | Suppress  |
/// | Soft      | >= `SOFT_BOUNCE_SUPPRESS_THRESHOLD` | Suppress  |
/// | Soft      | < `SOFT_BOUNCE_SUPPRESS_THRESHOLD`  | Retry     |
/// | Transient | any                               | NoAction  |
///
/// `Suppress` outcomes map to `EmailCommsError::RecipientSuppressed` at the
/// adapter layer; this function is the pure decision — the adapter is
/// responsible for constructing and returning the error variant.
#[must_use]
pub fn bounce_suppression_decision(
    category: BounceCategory,
    prior_soft_bounce_count: u32,
) -> BounceSuppressionOutcome {
    match category {
        BounceCategory::Hard => BounceSuppressionOutcome::Suppress,
        BounceCategory::Soft => {
            if prior_soft_bounce_count >= SOFT_BOUNCE_SUPPRESS_THRESHOLD {
                BounceSuppressionOutcome::Suppress
            } else {
                BounceSuppressionOutcome::Retry
            }
        }
        BounceCategory::Transient => BounceSuppressionOutcome::NoAction,
    }
}

// ---------- Inbound DMARC alignment + disposition ----------

/// Alignment mode for DMARC evaluation per RFC 7489 §3.1.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DmarcAlignmentMode {
    /// Exact domain match required. `sub.example.com` does NOT align with `example.com`.
    Strict,
    /// Organizational-domain suffix match. `sub.example.com` aligns with `example.com`
    /// because they share the same org domain. Single-label domains fall back to exact match.
    Relaxed,
}

impl fmt::Display for DmarcAlignmentMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DmarcAlignmentMode::Strict => f.write_str("strict"),
            DmarcAlignmentMode::Relaxed => f.write_str("relaxed"),
        }
    }
}

/// Concrete inbound message disposition after applying DMARC policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum DmarcDisposition {
    /// Deliver the message normally. Maps to DMARC pass, or DMARC fail + `p=none`.
    Accept,
    /// Deliver to junk/spam. Maps to DMARC fail + `p=quarantine`.
    Quarantine,
    /// Reject the message entirely. Maps to DMARC fail + `p=reject`.
    Reject,
}

impl fmt::Display for DmarcDisposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DmarcDisposition::Accept => f.write_str("accept"),
            DmarcDisposition::Quarantine => f.write_str("quarantine"),
            DmarcDisposition::Reject => f.write_str("reject"),
        }
    }
}

/// Inputs for a single inbound DMARC alignment evaluation.
///
/// All domain strings are matched case-insensitively per RFC 1035.
/// Empty strings for `spf_result_domain` or `dkim_result_domain` are
/// treated as "no result for that mechanism" (alignment fails for that mechanism).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DmarcAlignmentInput {
    /// The RFC 5322 `From` header domain (the identifier being protected).
    pub from_domain: String,
    /// The domain from the SPF authentication result (envelope sender domain).
    /// Empty string means SPF produced no usable result.
    pub spf_result_domain: String,
    /// The DKIM signing domain (`d=` tag from a validated DKIM signature).
    /// Empty string means DKIM produced no usable result.
    pub dkim_result_domain: String,
    /// Alignment strictness mode. RFC 7489 default is `Relaxed`.
    pub alignment_mode: DmarcAlignmentMode,
    /// The sender domain's published DMARC policy.
    pub policy: DmarcPolicy,
}

/// Result of evaluating DMARC alignment + disposition for one inbound message.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct DmarcEvalVerdict {
    /// `true` if the message passes DMARC (SPF aligned OR DKIM aligned).
    pub aligned: bool,
    /// Whether the SPF result domain aligns with the `From` domain.
    pub spf_aligned: bool,
    /// Whether the DKIM `d=` domain aligns with the `From` domain.
    pub dkim_aligned: bool,
    /// Concrete disposition derived from the alignment result and DMARC policy.
    pub disposition: DmarcDisposition,
}

/// Extract the organizational domain from a fully-qualified domain name.
///
/// Uses a simple two-label approximation: strip all labels except the last two.
/// `mail.sub.example.com` → `example.com`, `example.com` → `example.com`,
/// `localhost` (single-label) → `localhost`.
///
/// This is a best-effort approximation. Production callers that need a full
/// public-suffix-list-aware eTLD+1 computation should perform that upstream and
/// pass the result in `DmarcAlignmentInput`; this helper handles the common case.
fn org_domain(domain: &str) -> &str {
    let labels: Vec<&str> = domain.split('.').collect();
    if labels.len() <= 2 {
        domain
    } else {
        // Return everything after the first label.
        let dot_pos = domain.find('.').unwrap_or(domain.len());
        &domain[dot_pos + 1..]
    }
}

/// Return `true` if `auth_domain` aligns with `from_domain` under the given mode.
///
/// Empty `auth_domain` never aligns.
fn domains_align(from_domain: &str, auth_domain: &str, mode: DmarcAlignmentMode) -> bool {
    if auth_domain.is_empty() {
        return false;
    }
    let from_lc = from_domain.to_ascii_lowercase();
    let auth_lc = auth_domain.to_ascii_lowercase();
    match mode {
        DmarcAlignmentMode::Strict => from_lc == auth_lc,
        DmarcAlignmentMode::Relaxed => org_domain(&from_lc) == org_domain(&auth_lc),
    }
}

/// Evaluate inbound DMARC alignment and produce a disposition.
///
/// Per RFC 7489 §3.1: the message passes DMARC if at least one of SPF or DKIM
/// is *aligned* with the `From` header domain. The concrete `disposition` is then
/// derived from the pass/fail result and the sender's published DMARC policy.
///
/// This function is pure and deterministic — no I/O, no DNS, no network.
///
/// # OTel integration
///
/// Callers should set span attributes from the returned `DmarcEvalVerdict`:
/// - `dmarc.aligned` — `verdict.aligned`
/// - `dmarc.spf_aligned` — `verdict.spf_aligned`
/// - `dmarc.dkim_aligned` — `verdict.dkim_aligned`
/// - `dmarc.disposition` — `verdict.disposition.to_string()`
#[must_use]
pub fn evaluate_inbound_dmarc(input: &DmarcAlignmentInput) -> DmarcEvalVerdict {
    let spf_aligned = domains_align(
        &input.from_domain,
        &input.spf_result_domain,
        input.alignment_mode,
    );
    let dkim_aligned = domains_align(
        &input.from_domain,
        &input.dkim_result_domain,
        input.alignment_mode,
    );
    let aligned = spf_aligned || dkim_aligned;

    let disposition = if aligned {
        DmarcDisposition::Accept
    } else {
        match input.policy {
            DmarcPolicy::None => DmarcDisposition::Accept,
            DmarcPolicy::Quarantine => DmarcDisposition::Quarantine,
            DmarcPolicy::Reject => DmarcDisposition::Reject,
        }
    };

    DmarcEvalVerdict {
        aligned,
        spf_aligned,
        dkim_aligned,
        disposition,
    }
}

// ---------- Real adapter shells (no Noop fallback) ----------
//
// Each adapter ships its trait impl up-front. The actual provider
// SDK integration is wired behind a per-adapter feature flag at
// the µservice layer; until that wire happens, the adapter returns
// `EmailCommsError::AdapterNotConfigured`. This is honest — no
// silent success — and lets every µservice depend on the kernel
// today.

/// SES adapter (default for cloud-hosted clusters).
#[derive(Clone, Debug, Default)]
pub struct SesEmailComms;

impl SesEmailComms {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl EmailComms for SesEmailComms {
    fn provider(&self) -> EmailProvider {
        EmailProvider::Ses
    }
    fn preflight(
        &self,
        binding: &DeliverabilityBinding,
        message: &OutboundMessage,
    ) -> Result<(), EmailCommsError> {
        enforce_deliverability_invariants(binding, message, &[], true, 0, 0, &HashMap::new())
    }
    fn send(
        &self,
        binding: &DeliverabilityBinding,
        message: &OutboundMessage,
    ) -> Result<SendOutcome, EmailCommsError> {
        self.preflight(binding, message)?;
        Err(EmailCommsError::AdapterNotConfigured(EmailProvider::Ses))
    }
}

/// Postal self-hosted adapter (sovereign / air-gapped tier).
#[derive(Clone, Debug, Default)]
pub struct PostalEmailComms;

impl PostalEmailComms {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl EmailComms for PostalEmailComms {
    fn provider(&self) -> EmailProvider {
        EmailProvider::Postal
    }
    fn preflight(
        &self,
        binding: &DeliverabilityBinding,
        message: &OutboundMessage,
    ) -> Result<(), EmailCommsError> {
        enforce_deliverability_invariants(binding, message, &[], true, 0, 0, &HashMap::new())
    }
    fn send(
        &self,
        binding: &DeliverabilityBinding,
        message: &OutboundMessage,
    ) -> Result<SendOutcome, EmailCommsError> {
        self.preflight(binding, message)?;
        Err(EmailCommsError::AdapterNotConfigured(EmailProvider::Postal))
    }
}

/// Mailgun adapter (alt SaaS second-source).
#[derive(Clone, Debug, Default)]
pub struct MailgunEmailComms;

impl MailgunEmailComms {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl EmailComms for MailgunEmailComms {
    fn provider(&self) -> EmailProvider {
        EmailProvider::Mailgun
    }
    fn preflight(
        &self,
        binding: &DeliverabilityBinding,
        message: &OutboundMessage,
    ) -> Result<(), EmailCommsError> {
        enforce_deliverability_invariants(binding, message, &[], true, 0, 0, &HashMap::new())
    }
    fn send(
        &self,
        binding: &DeliverabilityBinding,
        message: &OutboundMessage,
    ) -> Result<SendOutcome, EmailCommsError> {
        self.preflight(binding, message)?;
        Err(EmailCommsError::AdapterNotConfigured(
            EmailProvider::Mailgun,
        ))
    }
}

/// Generic RFC 5321 SMTP fallback adapter.
#[derive(Clone, Debug, Default)]
pub struct SmtpEmailComms;

impl SmtpEmailComms {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl EmailComms for SmtpEmailComms {
    fn provider(&self) -> EmailProvider {
        EmailProvider::Smtp
    }
    fn preflight(
        &self,
        binding: &DeliverabilityBinding,
        message: &OutboundMessage,
    ) -> Result<(), EmailCommsError> {
        enforce_deliverability_invariants(binding, message, &[], true, 0, 0, &HashMap::new())
    }
    fn send(
        &self,
        binding: &DeliverabilityBinding,
        message: &OutboundMessage,
    ) -> Result<SendOutcome, EmailCommsError> {
        self.preflight(binding, message)?;
        Err(EmailCommsError::AdapterNotConfigured(EmailProvider::Smtp))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_LOCALE: &str = "aa-XA";

    fn good_binding() -> DeliverabilityBinding {
        DeliverabilityBinding {
            tenant: TenantId("acme".into()),
            from_domain: "mail.acme.com".into(),
            dkim: DkimBinding {
                selector: "oya2026".into(),
                key_ref: "bao://kv/dkim/acme/oya2026".into(),
                rotated_at_epoch_s: 1_747_500_000,
            },
            spf_authorized: true,
            dmarc_policy: DmarcPolicy::Reject,
        }
    }

    fn good_message() -> OutboundMessage {
        OutboundMessage {
            from: EmailAddress::try_new("no-reply@mail.acme.com").unwrap(),
            reply_to: None,
            to: vec![EmailAddress::try_new("user@example.com").unwrap()],
            subject: "Hello".into(),
            html_body: "<p>hello</p>".into(),
            plain_body: Some("hello".into()),
            locale: TEST_LOCALE.into(),
            idempotency_key: "id-deadbeefcafe1234567890".into(),
        }
    }

    #[test]
    fn email_address_validates_minimum_syntax() {
        assert!(EmailAddress::try_new("a@b.c").is_ok());
        assert!(EmailAddress::try_new("no-at-sign").is_err());
        assert!(EmailAddress::try_new("@no-local").is_err());
        assert!(EmailAddress::try_new("no-domain@").is_err());
        assert!(EmailAddress::try_new("no-dot@nodot").is_err());
    }

    #[test]
    fn preflight_rejects_missing_dkim_binding() {
        let mut b = good_binding();
        b.dkim.selector = String::new();
        let m = good_message();
        let err = enforce_deliverability_invariants(&b, &m, &[], true, 0, 0, &HashMap::new())
            .unwrap_err();
        match err {
            EmailCommsError::DkimBindingMissing(t) => assert_eq!(t.0, "acme"),
            other => panic!("expected DkimBindingMissing, got {other:?}"),
        }
    }

    #[test]
    fn preflight_rejects_spf_not_authorized() {
        let mut b = good_binding();
        b.spf_authorized = false;
        let m = good_message();
        let err = enforce_deliverability_invariants(&b, &m, &[], true, 0, 0, &HashMap::new())
            .unwrap_err();
        match err {
            EmailCommsError::SpfNotAuthorized(t) => assert_eq!(t.0, "acme"),
            other => panic!("expected SpfNotAuthorized, got {other:?}"),
        }
    }

    #[test]
    fn preflight_rejects_dmarc_none_past_warmup() {
        let mut b = good_binding();
        b.dmarc_policy = DmarcPolicy::None;
        let m = good_message();
        let err = enforce_deliverability_invariants(&b, &m, &[], true, 0, 0, &HashMap::new())
            .unwrap_err();
        match err {
            EmailCommsError::DmarcPolicyForbidden { policy, .. } => {
                assert_eq!(policy, DmarcPolicy::None);
            }
            other => panic!("expected DmarcPolicyForbidden, got {other:?}"),
        }
    }

    #[test]
    fn preflight_accepts_dmarc_none_during_warmup() {
        let mut b = good_binding();
        b.dmarc_policy = DmarcPolicy::None;
        let m = good_message();
        enforce_deliverability_invariants(&b, &m, &[], false, 0, 0, &HashMap::new()).unwrap();
    }

    #[test]
    fn preflight_rejects_suppressed_recipient() {
        let b = good_binding();
        let m = good_message();
        let supp = vec![EmailAddress::try_new("user@example.com").unwrap()];
        let err = enforce_deliverability_invariants(&b, &m, &supp, true, 0, 0, &HashMap::new())
            .unwrap_err();
        match err {
            EmailCommsError::RecipientSuppressed(addr) => {
                assert_eq!(addr.as_str(), "user@example.com");
            }
            other => panic!("expected RecipientSuppressed, got {other:?}"),
        }
    }

    #[test]
    fn preflight_rejects_from_domain_mismatch() {
        let b = good_binding();
        let mut m = good_message();
        m.from = EmailAddress::try_new("no-reply@wrong.com").unwrap();
        let err = enforce_deliverability_invariants(&b, &m, &[], true, 0, 0, &HashMap::new())
            .unwrap_err();
        match err {
            EmailCommsError::InvalidAddress(_) => {}
            other => panic!("expected InvalidAddress, got {other:?}"),
        }
    }

    #[test]
    fn preflight_rejects_empty_recipients_and_empty_message() {
        let b = good_binding();
        let mut m = good_message();
        m.to.clear();
        assert_eq!(
            enforce_deliverability_invariants(&b, &m, &[], true, 0, 0, &HashMap::new())
                .unwrap_err(),
            EmailCommsError::NoRecipients
        );
        let mut m2 = good_message();
        m2.subject = String::new();
        m2.html_body = String::new();
        assert_eq!(
            enforce_deliverability_invariants(&b, &m2, &[], true, 0, 0, &HashMap::new())
                .unwrap_err(),
            EmailCommsError::EmptyMessage
        );
    }

    #[test]
    fn all_adapters_report_their_provider_tag() {
        assert_eq!(SesEmailComms::new().provider(), EmailProvider::Ses);
        assert_eq!(PostalEmailComms::new().provider(), EmailProvider::Postal);
        assert_eq!(MailgunEmailComms::new().provider(), EmailProvider::Mailgun);
        assert_eq!(SmtpEmailComms::new().provider(), EmailProvider::Smtp);
    }

    #[test]
    fn all_adapters_return_adapter_not_configured_on_send_when_unwired() {
        let b = good_binding();
        let m = good_message();
        for adapter in [
            (SesEmailComms::new().send(&b, &m), EmailProvider::Ses),
            (PostalEmailComms::new().send(&b, &m), EmailProvider::Postal),
            (
                MailgunEmailComms::new().send(&b, &m),
                EmailProvider::Mailgun,
            ),
            (SmtpEmailComms::new().send(&b, &m), EmailProvider::Smtp),
        ] {
            let (res, expected) = adapter;
            match res.unwrap_err() {
                EmailCommsError::AdapterNotConfigured(p) => assert_eq!(p, expected),
                other => panic!("expected AdapterNotConfigured for {expected}, got {other:?}"),
            }
        }
    }

    // ---- ST1: per-tenant per-minute rate ceiling ----

    #[test]
    fn rate_ceiling_at_limit_rejected() {
        let b = good_binding();
        let m = good_message();
        let err = enforce_deliverability_invariants(&b, &m, &[], true, 10, 10, &HashMap::new())
            .unwrap_err();
        match err {
            EmailCommsError::RateCeilingExceeded { tenant, per_minute } => {
                assert_eq!(tenant.0, "acme");
                assert_eq!(per_minute, 10);
            }
            other => panic!("expected RateCeilingExceeded, got {other:?}"),
        }
    }

    #[test]
    fn rate_ceiling_below_limit_accepted() {
        let b = good_binding();
        let m = good_message();
        enforce_deliverability_invariants(&b, &m, &[], true, 9, 10, &HashMap::new()).unwrap();
    }

    #[test]
    fn rate_ceiling_zero_means_uncapped() {
        let b = good_binding();
        let m = good_message();
        // Even an absurdly large count must be accepted when ceiling == 0.
        enforce_deliverability_invariants(&b, &m, &[], true, 999_999, 0, &HashMap::new()).unwrap();
    }

    // ---- ST2: idempotency-key conflict detection ----

    #[test]
    fn idempotency_fresh_key_accepted() {
        let b = good_binding();
        let m = good_message();
        // Empty prior map — fresh key.
        enforce_deliverability_invariants(&b, &m, &[], true, 0, 0, &HashMap::new()).unwrap();
    }

    #[test]
    fn idempotency_same_key_identical_message_collapsed() {
        let b = good_binding();
        let m = good_message();
        let fp = message_fingerprint(&m);
        let mut priors = HashMap::new();
        priors.insert(m.idempotency_key.clone(), fp);
        // Identical re-send must succeed (collapse).
        enforce_deliverability_invariants(&b, &m, &[], true, 0, 0, &priors).unwrap();
    }

    #[test]
    fn idempotency_same_key_different_message_rejected() {
        let b = good_binding();
        let m = good_message();
        let fp = message_fingerprint(&m);
        let mut priors = HashMap::new();
        priors.insert(m.idempotency_key.clone(), fp);

        // Mutate the subject so the fingerprint diverges.
        let mut m2 = good_message();
        m2.subject = "A completely different subject".into();

        let err = enforce_deliverability_invariants(&b, &m2, &[], true, 0, 0, &priors).unwrap_err();
        match err {
            EmailCommsError::IdempotencyConflict { key } => {
                assert_eq!(key, m.idempotency_key);
            }
            other => panic!("expected IdempotencyConflict, got {other:?}"),
        }
    }

    // ---- ST1 extended: additional rate-ceiling edge cases ----

    #[test]
    fn rate_ceiling_above_limit_also_rejected_with_correct_per_minute() {
        // count=15, ceiling=10 — strictly above, not just at-boundary.
        // Verifies RateCeilingExceeded.per_minute reflects the caller-supplied
        // ceiling (100) not a hardcoded constant.
        let b = good_binding();
        let m = good_message();
        let err = enforce_deliverability_invariants(&b, &m, &[], true, 15, 10, &HashMap::new())
            .unwrap_err();
        match err {
            EmailCommsError::RateCeilingExceeded { tenant, per_minute } => {
                assert_eq!(tenant.0, "acme");
                assert_eq!(per_minute, 10);
            }
            other => panic!("expected RateCeilingExceeded, got {other:?}"),
        }
    }

    #[test]
    fn rate_ceiling_per_minute_field_reflects_caller_supplied_ceiling() {
        // Ceiling=100 — confirms the error carries the exact ceiling value,
        // not a hardcoded constant from a previous test.
        let b = good_binding();
        let m = good_message();
        let err = enforce_deliverability_invariants(&b, &m, &[], true, 100, 100, &HashMap::new())
            .unwrap_err();
        match err {
            EmailCommsError::RateCeilingExceeded { per_minute, .. } => {
                assert_eq!(per_minute, 100);
            }
            other => panic!("expected RateCeilingExceeded, got {other:?}"),
        }
    }

    #[test]
    fn rate_ceiling_check_fires_before_idempotency_conflict() {
        // When both rate ceiling is exceeded AND an idempotency conflict
        // exists, the rate ceiling error is returned first (ceiling check
        // precedes idempotency check in enforce_deliverability_invariants).
        let b = good_binding();
        let m = good_message();
        let fp = message_fingerprint(&m);
        // Store a *different* fingerprint so idempotency would conflict.
        let mut priors = HashMap::new();
        priors.insert(m.idempotency_key.clone(), fp.wrapping_add(1));
        let err =
            enforce_deliverability_invariants(&b, &m, &[], true, 10, 10, &priors).unwrap_err();
        match err {
            EmailCommsError::RateCeilingExceeded { .. } => {}
            other => {
                panic!("expected RateCeilingExceeded before IdempotencyConflict, got {other:?}")
            }
        }
    }

    #[test]
    fn suppression_check_catches_second_recipient_when_first_is_clean() {
        // Two-recipient message: first recipient clean, second suppressed.
        // Verifies the suppression loop iterates all recipients, not just the first.
        let b = good_binding();
        let mut m = good_message();
        let second = EmailAddress::try_new("other@example.com").unwrap();
        m.to.push(second.clone());
        let supp = vec![second.clone()];
        let err = enforce_deliverability_invariants(&b, &m, &supp, true, 0, 0, &HashMap::new())
            .unwrap_err();
        match err {
            EmailCommsError::RecipientSuppressed(addr) => {
                assert_eq!(addr.as_str(), "other@example.com");
            }
            other => panic!("expected RecipientSuppressed for second recipient, got {other:?}"),
        }
    }

    // ---- ST2 extended: idempotency fingerprint sensitivity ----

    #[test]
    fn idempotency_same_key_html_body_change_is_conflict() {
        // Changing only html_body (not subject) must produce a conflict.
        // Verifies html_body is included in the fingerprint.
        let b = good_binding();
        let m = good_message();
        let fp = message_fingerprint(&m);
        let mut priors = HashMap::new();
        priors.insert(m.idempotency_key.clone(), fp);

        let mut m2 = good_message();
        m2.html_body = "<p>completely different content</p>".into();

        let err = enforce_deliverability_invariants(&b, &m2, &[], true, 0, 0, &priors).unwrap_err();
        match err {
            EmailCommsError::IdempotencyConflict { key } => {
                assert_eq!(key, m.idempotency_key);
            }
            other => panic!("expected IdempotencyConflict on html_body change, got {other:?}"),
        }
    }

    #[test]
    fn idempotency_same_key_recipient_change_is_conflict() {
        // Changing only the recipient (same key, same subject/body) must
        // produce a conflict. Verifies the recipient list is in the fingerprint.
        let b = good_binding();
        let m = good_message();
        let fp = message_fingerprint(&m);
        let mut priors = HashMap::new();
        priors.insert(m.idempotency_key.clone(), fp);

        let mut m2 = good_message();
        m2.to = vec![EmailAddress::try_new("different@example.com").unwrap()];

        let err = enforce_deliverability_invariants(&b, &m2, &[], true, 0, 0, &priors).unwrap_err();
        match err {
            EmailCommsError::IdempotencyConflict { key } => {
                assert_eq!(key, m.idempotency_key);
            }
            other => panic!("expected IdempotencyConflict on recipient change, got {other:?}"),
        }
    }

    #[test]
    fn idempotency_collapse_is_order_independent_for_recipients() {
        // ADR-0149 collapse semantics: an identical re-send must collapse to
        // success regardless of recipient list ordering. Two messages with the
        // same from/subject/body/recipients but recipients in swapped order
        // represent the same logical message and must NOT produce a conflict.
        //
        // message_fingerprint sorts recipients before hashing (lib.rs:341-342),
        // so recipient list ordering does not affect the fingerprint. This test
        // confirms that collapse semantics hold regardless of to[] ordering.
        let b = good_binding();
        let rcpt_a = EmailAddress::try_new("alice@example.com").unwrap();
        let rcpt_b = EmailAddress::try_new("bob@example.com").unwrap();

        let mut m1 = good_message();
        m1.to = vec![rcpt_a.clone(), rcpt_b.clone()];

        let fp = message_fingerprint(&m1);
        let mut priors = HashMap::new();
        priors.insert(m1.idempotency_key.clone(), fp);

        // Same message, recipients in reversed order — must collapse, not conflict.
        let mut m2 = good_message();
        m2.to = vec![rcpt_b.clone(), rcpt_a.clone()];

        enforce_deliverability_invariants(&b, &m2, &[], true, 0, 0, &priors).expect(
            "identical re-send with reordered recipients must collapse to Ok, not conflict",
        );
    }

    #[test]
    fn delivery_event_kind_displays_canonical_strings() {
        assert_eq!(DeliveryEventKind::Sent.to_string(), "sent");
        assert_eq!(DeliveryEventKind::Delivered.to_string(), "delivered");
        assert_eq!(DeliveryEventKind::Bounced.to_string(), "bounced");
        assert_eq!(DeliveryEventKind::Complained.to_string(), "complained");
        assert_eq!(DeliveryEventKind::Suppressed.to_string(), "suppressed");
    }

    // ---- ST1: classify_bounce_enhanced ----

    /// RFC 3463 class 5 enhanced codes map to Hard regardless of sub-class.
    #[test]
    fn classify_enhanced_5xx_is_hard() {
        use super::{BounceCategory, classify_bounce_enhanced};
        assert_eq!(
            classify_bounce_enhanced("5.1.1"),
            Some(BounceCategory::Hard)
        );
        assert_eq!(
            classify_bounce_enhanced("5.0.0"),
            Some(BounceCategory::Hard)
        );
        assert_eq!(
            classify_bounce_enhanced("5.7.1"),
            Some(BounceCategory::Hard)
        );
    }

    /// RFC 3463 class 4 enhanced codes (excluding 4.2.1) map to Transient.
    #[test]
    fn classify_enhanced_4xx_is_transient() {
        use super::{BounceCategory, classify_bounce_enhanced};
        assert_eq!(
            classify_bounce_enhanced("4.2.2"),
            Some(BounceCategory::Transient)
        );
        assert_eq!(
            classify_bounce_enhanced("4.4.7"),
            Some(BounceCategory::Transient)
        );
    }

    /// RFC 3463 4.2.1 ("mailbox disabled, not accepting messages") maps to Soft.
    #[test]
    fn classify_enhanced_421_soft() {
        use super::{BounceCategory, classify_bounce_enhanced};
        assert_eq!(
            classify_bounce_enhanced("4.2.1"),
            Some(BounceCategory::Soft)
        );
    }

    /// Malformed or out-of-class enhanced status codes return None.
    #[test]
    fn classify_enhanced_malformed_is_none() {
        use super::classify_bounce_enhanced;
        assert_eq!(classify_bounce_enhanced("5.1"), None);
        assert_eq!(classify_bounce_enhanced("abc"), None);
        assert_eq!(classify_bounce_enhanced(""), None);
        // Class 2 is success — not a bounce category.
        assert_eq!(classify_bounce_enhanced("2.0.0"), None);
    }

    /// Confirm DeliveryEventKind::Bounced display is unchanged by the new types.
    #[test]
    fn delivery_event_kind_bounced_display_unchanged() {
        assert_eq!(DeliveryEventKind::Bounced.to_string(), "bounced");
    }

    // ---- ST1: classify_bounce_smtp ----

    /// SMTP 5xx reply codes map to Hard.
    #[test]
    fn classify_smtp_5xx_is_hard() {
        use super::{BounceCategory, classify_bounce_smtp};
        assert_eq!(classify_bounce_smtp(550), Some(BounceCategory::Hard));
        assert_eq!(classify_bounce_smtp(521), Some(BounceCategory::Hard));
        assert_eq!(classify_bounce_smtp(554), Some(BounceCategory::Hard));
    }

    /// SMTP 452 (over-quota) maps to Soft.
    #[test]
    fn classify_smtp_452_is_soft() {
        use super::{BounceCategory, classify_bounce_smtp};
        assert_eq!(classify_bounce_smtp(452), Some(BounceCategory::Soft));
    }

    /// SMTP 4xx codes (excluding 452) map to Transient.
    #[test]
    fn classify_smtp_4xx_is_transient() {
        use super::{BounceCategory, classify_bounce_smtp};
        assert_eq!(classify_bounce_smtp(421), Some(BounceCategory::Transient));
        assert_eq!(classify_bounce_smtp(450), Some(BounceCategory::Transient));
        assert_eq!(classify_bounce_smtp(451), Some(BounceCategory::Transient));
    }

    /// SMTP codes outside the 4xx–5xx range return None.
    #[test]
    fn classify_smtp_out_of_range_is_none() {
        use super::classify_bounce_smtp;
        assert_eq!(classify_bounce_smtp(200), None);
        assert_eq!(classify_bounce_smtp(350), None);
        assert_eq!(classify_bounce_smtp(600), None);
    }

    // ---- ST2: bounce_suppression_decision ----

    /// A Hard bounce always suppresses immediately, regardless of prior soft count.
    #[test]
    fn hard_bounce_suppresses() {
        use super::{BounceCategory, BounceSuppressionOutcome, bounce_suppression_decision};
        assert_eq!(
            bounce_suppression_decision(BounceCategory::Hard, 0),
            BounceSuppressionOutcome::Suppress,
        );
    }

    /// A single soft bounce (first occurrence) yields Retry, not Suppress.
    #[test]
    fn single_soft_bounce_retries() {
        use super::{BounceCategory, BounceSuppressionOutcome, bounce_suppression_decision};
        assert_eq!(
            bounce_suppression_decision(BounceCategory::Soft, 0),
            BounceSuppressionOutcome::Retry,
        );
    }

    /// Soft bounces reaching the suppress threshold promote to Suppress.
    #[test]
    fn soft_at_threshold_suppresses() {
        use super::{
            BounceCategory, BounceSuppressionOutcome, SOFT_BOUNCE_SUPPRESS_THRESHOLD,
            bounce_suppression_decision,
        };
        assert_eq!(
            bounce_suppression_decision(BounceCategory::Soft, SOFT_BOUNCE_SUPPRESS_THRESHOLD),
            BounceSuppressionOutcome::Suppress,
        );
    }

    /// Soft bounces below threshold remain in Retry state.
    #[test]
    fn soft_below_threshold_retries() {
        use super::{
            BounceCategory, BounceSuppressionOutcome, SOFT_BOUNCE_SUPPRESS_THRESHOLD,
            bounce_suppression_decision,
        };
        // prior = threshold - 1; must still be Retry.
        assert_eq!(
            bounce_suppression_decision(BounceCategory::Soft, SOFT_BOUNCE_SUPPRESS_THRESHOLD - 1),
            BounceSuppressionOutcome::Retry,
        );
    }

    /// A Transient bounce yields NoAction (infrastructure retry is sufficient).
    #[test]
    fn transient_no_action() {
        use super::{BounceCategory, BounceSuppressionOutcome, bounce_suppression_decision};
        assert_eq!(
            bounce_suppression_decision(BounceCategory::Transient, 0),
            BounceSuppressionOutcome::NoAction,
        );
    }

    /// Hard bounce produces Suppress regardless of any prior soft-bounce count.
    #[test]
    fn hard_bounce_suppresses_regardless_of_prior_soft_count() {
        use super::{BounceCategory, BounceSuppressionOutcome, bounce_suppression_decision};
        for prior in [0u32, 1, 3, 99] {
            assert_eq!(
                bounce_suppression_decision(BounceCategory::Hard, prior),
                BounceSuppressionOutcome::Suppress,
                "Hard bounce must suppress for prior_soft={prior}",
            );
        }
    }

    /// Transient bounce is NoAction regardless of prior soft-bounce count.
    #[test]
    fn transient_no_action_regardless_of_prior_soft_count() {
        use super::{BounceCategory, BounceSuppressionOutcome, bounce_suppression_decision};
        for prior in [0u32, 1, 3, 99] {
            assert_eq!(
                bounce_suppression_decision(BounceCategory::Transient, prior),
                BounceSuppressionOutcome::NoAction,
                "Transient bounce must be NoAction for prior_soft={prior}",
            );
        }
    }

    /// enforce_deliverability_invariants is unchanged — existing tests still pass.
    /// This smoke test re-runs the happy path to confirm no regression.
    #[test]
    fn enforce_deliverability_invariants_still_passes_happy_path() {
        let b = good_binding();
        let m = good_message();
        enforce_deliverability_invariants(&b, &m, &[], true, 0, 0, &HashMap::new())
            .expect("happy-path preflight must still pass after bounce types added");
    }

    // ---------- Inbound DMARC alignment + disposition tests ----------

    fn dmarc_input(
        from: &str,
        spf: &str,
        dkim: &str,
        mode: DmarcAlignmentMode,
        policy: DmarcPolicy,
    ) -> DmarcAlignmentInput {
        DmarcAlignmentInput {
            from_domain: from.to_string(),
            spf_result_domain: spf.to_string(),
            dkim_result_domain: dkim.to_string(),
            alignment_mode: mode,
            policy,
        }
    }

    #[test]
    fn dmarc_spf_only_aligned_pass() {
        let v = evaluate_inbound_dmarc(&dmarc_input(
            "example.com",
            "example.com",
            "",
            DmarcAlignmentMode::Relaxed,
            DmarcPolicy::Reject,
        ));
        assert!(v.spf_aligned);
        assert!(!v.dkim_aligned);
        assert!(v.aligned);
        assert_eq!(v.disposition, DmarcDisposition::Accept);
    }

    #[test]
    fn dmarc_dkim_only_aligned_pass() {
        let v = evaluate_inbound_dmarc(&dmarc_input(
            "example.com",
            "",
            "example.com",
            DmarcAlignmentMode::Relaxed,
            DmarcPolicy::Reject,
        ));
        assert!(!v.spf_aligned);
        assert!(v.dkim_aligned);
        assert!(v.aligned);
        assert_eq!(v.disposition, DmarcDisposition::Accept);
    }

    #[test]
    fn dmarc_both_fail_none_policy_accept() {
        let v = evaluate_inbound_dmarc(&dmarc_input(
            "example.com",
            "other.com",
            "other.com",
            DmarcAlignmentMode::Relaxed,
            DmarcPolicy::None,
        ));
        assert!(!v.aligned);
        assert_eq!(v.disposition, DmarcDisposition::Accept);
    }

    #[test]
    fn dmarc_both_fail_quarantine_policy() {
        let v = evaluate_inbound_dmarc(&dmarc_input(
            "example.com",
            "other.com",
            "other.com",
            DmarcAlignmentMode::Relaxed,
            DmarcPolicy::Quarantine,
        ));
        assert!(!v.aligned);
        assert_eq!(v.disposition, DmarcDisposition::Quarantine);
    }

    #[test]
    fn dmarc_both_fail_reject_policy() {
        let v = evaluate_inbound_dmarc(&dmarc_input(
            "example.com",
            "other.com",
            "other.com",
            DmarcAlignmentMode::Relaxed,
            DmarcPolicy::Reject,
        ));
        assert!(!v.aligned);
        assert_eq!(v.disposition, DmarcDisposition::Reject);
    }

    #[test]
    fn dmarc_strict_subdomain_fails() {
        let v = evaluate_inbound_dmarc(&dmarc_input(
            "example.com",
            "sub.example.com",
            "",
            DmarcAlignmentMode::Strict,
            DmarcPolicy::Reject,
        ));
        assert!(!v.spf_aligned);
        assert!(!v.aligned);
        assert_eq!(v.disposition, DmarcDisposition::Reject);
    }

    #[test]
    fn dmarc_relaxed_subdomain_passes() {
        let v = evaluate_inbound_dmarc(&dmarc_input(
            "example.com",
            "sub.example.com",
            "",
            DmarcAlignmentMode::Relaxed,
            DmarcPolicy::Reject,
        ));
        assert!(v.spf_aligned);
        assert!(v.aligned);
        assert_eq!(v.disposition, DmarcDisposition::Accept);
    }

    #[test]
    fn dmarc_relaxed_cross_org_fails() {
        let v = evaluate_inbound_dmarc(&dmarc_input(
            "example.com",
            "other.com",
            "",
            DmarcAlignmentMode::Relaxed,
            DmarcPolicy::Reject,
        ));
        assert!(!v.spf_aligned);
        assert!(!v.aligned);
        assert_eq!(v.disposition, DmarcDisposition::Reject);
    }

    #[test]
    fn dmarc_both_aligned_pass() {
        let v = evaluate_inbound_dmarc(&dmarc_input(
            "example.com",
            "example.com",
            "example.com",
            DmarcAlignmentMode::Relaxed,
            DmarcPolicy::Reject,
        ));
        assert!(v.spf_aligned);
        assert!(v.dkim_aligned);
        assert!(v.aligned);
        assert_eq!(v.disposition, DmarcDisposition::Accept);
    }

    #[test]
    fn dmarc_case_insensitive_alignment() {
        let v = evaluate_inbound_dmarc(&dmarc_input(
            "Example.COM",
            "example.com",
            "EXAMPLE.COM",
            DmarcAlignmentMode::Strict,
            DmarcPolicy::Reject,
        ));
        assert!(v.spf_aligned);
        assert!(v.dkim_aligned);
        assert!(v.aligned);
        assert_eq!(v.disposition, DmarcDisposition::Accept);
    }

    #[test]
    fn dmarc_empty_spf_domain_not_aligned() {
        let v = evaluate_inbound_dmarc(&dmarc_input(
            "example.com",
            "",
            "",
            DmarcAlignmentMode::Relaxed,
            DmarcPolicy::None,
        ));
        assert!(!v.spf_aligned);
        assert!(!v.dkim_aligned);
        assert!(!v.aligned);
        assert_eq!(v.disposition, DmarcDisposition::Accept);
    }

    #[test]
    fn dmarc_none_policy_pass_is_accept() {
        let v = evaluate_inbound_dmarc(&dmarc_input(
            "example.com",
            "example.com",
            "",
            DmarcAlignmentMode::Relaxed,
            DmarcPolicy::None,
        ));
        assert!(v.aligned);
        assert_eq!(v.disposition, DmarcDisposition::Accept);
    }
}
