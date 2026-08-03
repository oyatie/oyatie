//! Pure domain guard for production-active mail sending-domain authentication.
//!
//! This module models the no-I/O admission decision that sits before SMTP
//! delivery. It checks tenant/domain isolation plus SPF, DMARC, and DKIM
//! posture evidence; live DNS lookups, OpenBao key reads, cryptographic
//! signing, and SMTP delivery remain adapter/runtime responsibilities.

use data_boundary_kernel::{Classified, DataClass};

pub const DEFAULT_DKIM_ROTATION_AGE_SECONDS: u64 = 365 * 24 * 60 * 60;
pub const NON_CLAIM: &str = "pure domain admission guard only; no DNS lookup, crypto signing, OpenBao, SMTP, or live deliverability claim";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SendingDomainActivationMode {
    ProductionActive,
    QuarantineOnly,
    ReviewOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SendingDomainAuthAction {
    Allow,
    Block,
    Quarantine,
    Review,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SendingDomainAuthReason {
    Authenticated,
    MissingRequiredEvidence,
    TenantMismatch,
    SenderDomainMismatch,
    DomainNotVerified,
    SpfMissing,
    DmarcMissing,
    DmarcNoneForProduction,
    DkimMissing,
    DkimExpired,
    DkimRotationInFuture,
    DkimRotationStale,
    DkimTenantMismatch,
    DkimDomainMismatch,
    UnsupportedDkimAlgorithm,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DmarcDomainPolicy {
    None,
    Quarantine,
    Reject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DkimSigningAlgorithm {
    Ed25519Sha256,
    RsaSha256,
    RsaSha1,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DkimSigningEvidence {
    pub tenant_id: Classified<String>,      // data_class: INTERNAL_ONLY
    pub signing_domain: Classified<String>, // data_class: INTERNAL_ONLY
    pub selector: Classified<String>,       // data_class: INTERNAL_ONLY
    pub key_version_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub algorithm: DkimSigningAlgorithm,    // data_class: INTERNAL_ONLY
    pub evidence_ref: Classified<String>,   // data_class: INTERNAL_ONLY
    pub rotated_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
    pub expires_at_epoch_seconds: u64,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SendingDomainAuthenticationInput {
    pub activation_mode: SendingDomainActivationMode, // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,                // data_class: INTERNAL_ONLY
    pub domain_tenant_id: Classified<String>,         // data_class: INTERNAL_ONLY
    pub sender_domain: Classified<String>,            // data_class: INTERNAL_ONLY
    pub envelope_from_domain: Classified<String>,     // data_class: INTERNAL_ONLY
    pub authenticated_principal_ref: Classified<String>, // data_class: INTERNAL_ONLY
    pub domain_verified: bool,                        // data_class: INTERNAL_ONLY
    pub spf_record_present: bool,                     // data_class: INTERNAL_ONLY
    pub dmarc_policy: Option<DmarcDomainPolicy>,      // data_class: INTERNAL_ONLY
    pub dkim: Option<DkimSigningEvidence>,            // data_class: INTERNAL_ONLY
    pub now_epoch_seconds: u64,                       // data_class: INTERNAL_ONLY
    pub max_dkim_rotation_age_seconds: u64,           // data_class: INTERNAL_ONLY
    pub request_id: Classified<String>,               // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SendingDomainAuthenticationVerdict {
    pub action: SendingDomainAuthAction, // data_class: INTERNAL_ONLY
    pub reason: SendingDomainAuthReason, // data_class: INTERNAL_ONLY
    pub audit_event_label: String,       // data_class: INTERNAL_ONLY
    pub tenant_id: Classified<String>,   // data_class: INTERNAL_ONLY
    pub sender_domain: Classified<String>, // data_class: INTERNAL_ONLY
    pub request_id: Classified<String>,  // data_class: INTERNAL_ONLY
    pub dkim_selector_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub dkim_key_version_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub dkim_evidence_ref: Option<String>, // data_class: INTERNAL_ONLY
    pub non_claim: &'static str,         // data_class: INTERNAL_ONLY
}

pub fn evaluate_sending_domain_authentication(
    input: &SendingDomainAuthenticationInput,
) -> SendingDomainAuthenticationVerdict {
    if required_text_missing(input) {
        return hard_block(input, SendingDomainAuthReason::MissingRequiredEvidence);
    }
    if !same_token(&input.tenant_id.value, &input.domain_tenant_id.value) {
        return hard_block(input, SendingDomainAuthReason::TenantMismatch);
    }
    if !same_domain(
        &input.sender_domain.value,
        &input.envelope_from_domain.value,
    ) {
        return hard_block(input, SendingDomainAuthReason::SenderDomainMismatch);
    }
    if !input.domain_verified {
        return hard_block(input, SendingDomainAuthReason::DomainNotVerified);
    }
    if !input.spf_record_present {
        return incomplete(input, SendingDomainAuthReason::SpfMissing);
    }
    match input.dmarc_policy {
        Some(DmarcDomainPolicy::Reject | DmarcDomainPolicy::Quarantine) => {}
        Some(DmarcDomainPolicy::None) => {
            return incomplete(input, SendingDomainAuthReason::DmarcNoneForProduction);
        }
        None => return incomplete(input, SendingDomainAuthReason::DmarcMissing),
    }
    let Some(dkim) = input.dkim.as_ref() else {
        return incomplete(input, SendingDomainAuthReason::DkimMissing);
    };
    if required_dkim_text_missing(dkim) {
        return hard_block(input, SendingDomainAuthReason::MissingRequiredEvidence);
    }
    if !same_token(&input.tenant_id.value, &dkim.tenant_id.value) {
        return hard_block(input, SendingDomainAuthReason::DkimTenantMismatch);
    }
    if !same_domain(&input.sender_domain.value, &dkim.signing_domain.value) {
        return hard_block(input, SendingDomainAuthReason::DkimDomainMismatch);
    }
    if !dkim.algorithm.supported_for_signing() {
        return hard_block(input, SendingDomainAuthReason::UnsupportedDkimAlgorithm);
    }
    if dkim.rotated_at_epoch_seconds > input.now_epoch_seconds {
        return incomplete(input, SendingDomainAuthReason::DkimRotationInFuture);
    }
    if dkim.expires_at_epoch_seconds <= input.now_epoch_seconds {
        return incomplete(input, SendingDomainAuthReason::DkimExpired);
    }
    if input
        .now_epoch_seconds
        .saturating_sub(dkim.rotated_at_epoch_seconds)
        > input.max_dkim_rotation_age_seconds
    {
        return incomplete(input, SendingDomainAuthReason::DkimRotationStale);
    }

    SendingDomainAuthenticationVerdict {
        action: SendingDomainAuthAction::Allow,
        reason: SendingDomainAuthReason::Authenticated,
        audit_event_label: audit_event_label(SendingDomainAuthReason::Authenticated),
        tenant_id: input.tenant_id.clone(),
        sender_domain: input.sender_domain.clone(),
        request_id: input.request_id.clone(),
        dkim_selector_ref: Some(dkim.selector.value.clone()),
        dkim_key_version_ref: Some(dkim.key_version_ref.value.clone()),
        dkim_evidence_ref: Some(dkim.evidence_ref.value.clone()),
        non_claim: NON_CLAIM,
    }
}

pub fn classified_internal(value: impl Into<String>) -> Classified<String> {
    Classified::new(value.into(), DataClass::InternalOnly)
}

impl DkimSigningAlgorithm {
    pub const fn supported_for_signing(self) -> bool {
        matches!(self, Self::Ed25519Sha256 | Self::RsaSha256)
    }
}

fn hard_block(
    input: &SendingDomainAuthenticationInput,
    reason: SendingDomainAuthReason,
) -> SendingDomainAuthenticationVerdict {
    verdict(input, SendingDomainAuthAction::Block, reason)
}

fn incomplete(
    input: &SendingDomainAuthenticationInput,
    reason: SendingDomainAuthReason,
) -> SendingDomainAuthenticationVerdict {
    verdict(input, incomplete_action(input.activation_mode), reason)
}

fn incomplete_action(mode: SendingDomainActivationMode) -> SendingDomainAuthAction {
    match mode {
        SendingDomainActivationMode::ProductionActive => SendingDomainAuthAction::Block,
        SendingDomainActivationMode::QuarantineOnly => SendingDomainAuthAction::Quarantine,
        SendingDomainActivationMode::ReviewOnly => SendingDomainAuthAction::Review,
    }
}

fn verdict(
    input: &SendingDomainAuthenticationInput,
    action: SendingDomainAuthAction,
    reason: SendingDomainAuthReason,
) -> SendingDomainAuthenticationVerdict {
    SendingDomainAuthenticationVerdict {
        action,
        reason,
        audit_event_label: audit_event_label(reason),
        tenant_id: input.tenant_id.clone(),
        sender_domain: input.sender_domain.clone(),
        request_id: input.request_id.clone(),
        dkim_selector_ref: None,
        dkim_key_version_ref: None,
        dkim_evidence_ref: None,
        non_claim: NON_CLAIM,
    }
}

fn audit_event_label(reason: SendingDomainAuthReason) -> String {
    format!("mail.sending_domain_authentication.{}", reason.label())
}

impl SendingDomainAuthReason {
    const fn label(self) -> &'static str {
        match self {
            Self::Authenticated => "authenticated",
            Self::MissingRequiredEvidence => "missing_required_evidence",
            Self::TenantMismatch => "tenant_mismatch",
            Self::SenderDomainMismatch => "sender_domain_mismatch",
            Self::DomainNotVerified => "domain_not_verified",
            Self::SpfMissing => "spf_missing",
            Self::DmarcMissing => "dmarc_missing",
            Self::DmarcNoneForProduction => "dmarc_none_for_production",
            Self::DkimMissing => "dkim_missing",
            Self::DkimExpired => "dkim_expired",
            Self::DkimRotationInFuture => "dkim_rotation_in_future",
            Self::DkimRotationStale => "dkim_rotation_stale",
            Self::DkimTenantMismatch => "dkim_tenant_mismatch",
            Self::DkimDomainMismatch => "dkim_domain_mismatch",
            Self::UnsupportedDkimAlgorithm => "unsupported_dkim_algorithm",
        }
    }
}

fn required_text_missing(input: &SendingDomainAuthenticationInput) -> bool {
    let text_missing = [
        &input.tenant_id.value,
        &input.domain_tenant_id.value,
        &input.sender_domain.value,
        &input.envelope_from_domain.value,
        &input.authenticated_principal_ref.value,
        &input.request_id.value,
    ]
    .iter()
    .any(|value| value.trim().is_empty());

    text_missing
        || normalized_domain(&input.sender_domain.value).is_empty()
        || normalized_domain(&input.envelope_from_domain.value).is_empty()
}

fn required_dkim_text_missing(dkim: &DkimSigningEvidence) -> bool {
    let text_missing = [
        &dkim.tenant_id.value,
        &dkim.signing_domain.value,
        &dkim.selector.value,
        &dkim.key_version_ref.value,
        &dkim.evidence_ref.value,
    ]
    .iter()
    .any(|value| value.trim().is_empty());

    text_missing || normalized_domain(&dkim.signing_domain.value).is_empty()
}

fn same_token(left: &str, right: &str) -> bool {
    left.trim() == right.trim()
}

fn same_domain(left: &str, right: &str) -> bool {
    normalized_domain(left) == normalized_domain(right)
}

fn normalized_domain(value: &str) -> String {
    value.trim().trim_end_matches('.').to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aligned_input() -> SendingDomainAuthenticationInput {
        SendingDomainAuthenticationInput {
            activation_mode: SendingDomainActivationMode::ProductionActive,
            tenant_id: classified_internal("tenant:acme"),
            domain_tenant_id: classified_internal("tenant:acme"),
            sender_domain: classified_internal("example.com"),
            envelope_from_domain: classified_internal("example.com"),
            authenticated_principal_ref: classified_internal("user:alice"),
            domain_verified: true,
            spf_record_present: true,
            dmarc_policy: Some(DmarcDomainPolicy::Reject),
            dkim: Some(DkimSigningEvidence {
                tenant_id: classified_internal("tenant:acme"),
                signing_domain: classified_internal("example.com"),
                selector: classified_internal("s20260525a"),
                key_version_ref: classified_internal("dkim-key:v1"),
                algorithm: DkimSigningAlgorithm::Ed25519Sha256,
                evidence_ref: classified_internal("evidence:dkim"),
                rotated_at_epoch_seconds: 1_700_000_000,
                expires_at_epoch_seconds: 1_800_000_000,
            }),
            now_epoch_seconds: 1_710_000_000,
            max_dkim_rotation_age_seconds: 365 * 24 * 60 * 60,
            request_id: classified_internal("req-1"),
        }
    }

    #[test]
    fn complete_production_active_posture_allows_sending_domain() {
        let verdict = evaluate_sending_domain_authentication(&aligned_input());

        assert_eq!(verdict.action, SendingDomainAuthAction::Allow);
        assert_eq!(verdict.reason, SendingDomainAuthReason::Authenticated);
        assert_eq!(
            verdict.audit_event_label,
            "mail.sending_domain_authentication.authenticated"
        );
        assert_eq!(verdict.dkim_selector_ref.as_deref(), Some("s20260525a"));
        assert_eq!(verdict.dkim_key_version_ref.as_deref(), Some("dkim-key:v1"));
    }

    #[test]
    fn production_active_domain_without_spf_blocks_before_smtp_delivery() {
        let mut input = aligned_input();
        input.spf_record_present = false;

        let verdict = evaluate_sending_domain_authentication(&input);

        assert_eq!(verdict.action, SendingDomainAuthAction::Block);
        assert_eq!(verdict.reason, SendingDomainAuthReason::SpfMissing);
    }

    #[test]
    fn production_active_domain_requires_enforcing_dmarc_policy() {
        for policy in [None, Some(DmarcDomainPolicy::None)] {
            let mut input = aligned_input();
            input.dmarc_policy = policy;

            let verdict = evaluate_sending_domain_authentication(&input);

            assert_eq!(verdict.action, SendingDomainAuthAction::Block);
            assert!(matches!(
                verdict.reason,
                SendingDomainAuthReason::DmarcMissing
                    | SendingDomainAuthReason::DmarcNoneForProduction
            ));
        }
    }

    #[test]
    fn production_active_domain_requires_current_dkim_evidence() {
        let mut missing = aligned_input();
        missing.dkim = None;
        assert_eq!(
            evaluate_sending_domain_authentication(&missing).reason,
            SendingDomainAuthReason::DkimMissing
        );

        let mut expired = aligned_input();
        expired.dkim.as_mut().unwrap().expires_at_epoch_seconds = 1_700_000_001;
        assert_eq!(
            evaluate_sending_domain_authentication(&expired).reason,
            SendingDomainAuthReason::DkimExpired
        );

        let mut future_rotation = aligned_input();
        future_rotation
            .dkim
            .as_mut()
            .unwrap()
            .rotated_at_epoch_seconds = future_rotation.now_epoch_seconds + 1;
        assert_eq!(
            evaluate_sending_domain_authentication(&future_rotation).reason,
            SendingDomainAuthReason::DkimRotationInFuture
        );

        let mut stale = aligned_input();
        stale.max_dkim_rotation_age_seconds = 1;
        assert_eq!(
            evaluate_sending_domain_authentication(&stale).reason,
            SendingDomainAuthReason::DkimRotationStale
        );
    }

    #[test]
    fn tenant_domain_and_dkim_mismatch_rejects_with_redacted_reason_labels() {
        let mut tenant_mismatch = aligned_input();
        tenant_mismatch.domain_tenant_id = classified_internal("tenant:other");
        assert_eq!(
            evaluate_sending_domain_authentication(&tenant_mismatch).reason,
            SendingDomainAuthReason::TenantMismatch
        );

        let mut domain_mismatch = aligned_input();
        domain_mismatch.envelope_from_domain = classified_internal("evil.example");
        assert_eq!(
            evaluate_sending_domain_authentication(&domain_mismatch).reason,
            SendingDomainAuthReason::SenderDomainMismatch
        );

        let mut missing_normalized_domain = aligned_input();
        missing_normalized_domain.sender_domain = classified_internal(".");
        missing_normalized_domain.envelope_from_domain = classified_internal(".");
        assert_eq!(
            evaluate_sending_domain_authentication(&missing_normalized_domain).reason,
            SendingDomainAuthReason::MissingRequiredEvidence
        );

        let mut dkim_tenant_mismatch = aligned_input();
        dkim_tenant_mismatch.dkim.as_mut().unwrap().tenant_id = classified_internal("tenant:other");
        let verdict = evaluate_sending_domain_authentication(&dkim_tenant_mismatch);

        assert_eq!(verdict.action, SendingDomainAuthAction::Block);
        assert_eq!(verdict.reason, SendingDomainAuthReason::DkimTenantMismatch);
        assert!(!verdict.audit_event_label.contains("example.com"));
        assert!(!verdict.audit_event_label.contains("tenant:acme"));
    }

    #[test]
    fn unsupported_dkim_algorithm_and_unverified_domain_reject() {
        let mut unsupported = aligned_input();
        unsupported.dkim.as_mut().unwrap().algorithm = DkimSigningAlgorithm::RsaSha1;
        assert_eq!(
            evaluate_sending_domain_authentication(&unsupported).reason,
            SendingDomainAuthReason::UnsupportedDkimAlgorithm
        );

        let mut unverified = aligned_input();
        unverified.domain_verified = false;
        assert_eq!(
            evaluate_sending_domain_authentication(&unverified).reason,
            SendingDomainAuthReason::DomainNotVerified
        );
    }

    #[test]
    fn incomplete_non_production_modes_quarantine_or_review_without_live_delivery_claims() {
        let mut quarantine = aligned_input();
        quarantine.activation_mode = SendingDomainActivationMode::QuarantineOnly;
        quarantine.dkim = None;
        let quarantine_verdict = evaluate_sending_domain_authentication(&quarantine);
        assert_eq!(
            quarantine_verdict.action,
            SendingDomainAuthAction::Quarantine
        );
        assert_eq!(
            quarantine_verdict.reason,
            SendingDomainAuthReason::DkimMissing
        );
        assert!(quarantine_verdict.non_claim.contains("no DNS lookup"));

        let mut review = aligned_input();
        review.activation_mode = SendingDomainActivationMode::ReviewOnly;
        review.dmarc_policy = Some(DmarcDomainPolicy::None);
        let review_verdict = evaluate_sending_domain_authentication(&review);
        assert_eq!(review_verdict.action, SendingDomainAuthAction::Review);
        assert_eq!(
            review_verdict.reason,
            SendingDomainAuthReason::DmarcNoneForProduction
        );
    }
}
