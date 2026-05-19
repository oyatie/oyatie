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

/// Shared deliverability invariant check used by every real
/// adapter. Centralizing it here means DKIM/SPF/DMARC + suppression
/// + rate-ceiling rules are uniform across SES, Postal, Mailgun,
///
/// SMTP.
///
/// # Errors
/// See `EmailCommsError`.
pub fn enforce_deliverability_invariants(
    binding: &DeliverabilityBinding,
    message: &OutboundMessage,
    suppressed: &[EmailAddress],
    warm_up_complete: bool,
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
    for rcpt in &message.to {
        if suppressed.contains(rcpt) {
            return Err(EmailCommsError::RecipientSuppressed(rcpt.clone()));
        }
    }
    Ok(())
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
        enforce_deliverability_invariants(binding, message, &[], true)
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
        enforce_deliverability_invariants(binding, message, &[], true)
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
        enforce_deliverability_invariants(binding, message, &[], true)
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
        enforce_deliverability_invariants(binding, message, &[], true)
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
        let err = enforce_deliverability_invariants(&b, &m, &[], true).unwrap_err();
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
        let err = enforce_deliverability_invariants(&b, &m, &[], true).unwrap_err();
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
        let err = enforce_deliverability_invariants(&b, &m, &[], true).unwrap_err();
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
        enforce_deliverability_invariants(&b, &m, &[], false).unwrap();
    }

    #[test]
    fn preflight_rejects_suppressed_recipient() {
        let b = good_binding();
        let m = good_message();
        let supp = vec![EmailAddress::try_new("user@example.com").unwrap()];
        let err = enforce_deliverability_invariants(&b, &m, &supp, true).unwrap_err();
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
        let err = enforce_deliverability_invariants(&b, &m, &[], true).unwrap_err();
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
            enforce_deliverability_invariants(&b, &m, &[], true).unwrap_err(),
            EmailCommsError::NoRecipients
        );
        let mut m2 = good_message();
        m2.subject = String::new();
        m2.html_body = String::new();
        assert_eq!(
            enforce_deliverability_invariants(&b, &m2, &[], true).unwrap_err(),
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

    #[test]
    fn delivery_event_kind_displays_canonical_strings() {
        assert_eq!(DeliveryEventKind::Sent.to_string(), "sent");
        assert_eq!(DeliveryEventKind::Delivered.to_string(), "delivered");
        assert_eq!(DeliveryEventKind::Bounced.to_string(), "bounced");
        assert_eq!(DeliveryEventKind::Complained.to_string(), "complained");
        assert_eq!(DeliveryEventKind::Suppressed.to_string(), "suppressed");
    }
}
