// CN-PIPL-2021 Cedar Fragment: consent-gating
// fragment_id: pack-cn-pipl-2021-consent-gating
// version: 1.0.0
// binding_adr: ADR-0251
// regulation: PIPL Articles 13–15 — consent as lawful basis; separate per-purpose consent;
//             right to withdraw; 12-month renewal requirement.
//
// This fragment enforces:
//   1. Every PI_CN_PIPL processing action requires a valid, non-expired per-purpose consent record.
//   2. Consent records older than 12 months are treated as expired and processing is denied.
//   3. Withdrawal of consent is honored immediately; no grace period for withdrawal.
//
// Hyperscaler precedent:
//   Alibaba Cloud's DMP (Data Management Platform) and Tencent Cloud's privacy center both
//   implement per-purpose consent gating with explicit renewal workflows to comply with PIPL
//   and the CAC App Personal Information Collection Rules (2022).

namespace pack::cn_pipl_2021;

// -----------------------------------------------------------------------
// Rule 1: Forbid processing PI_CN_PIPL data without a valid per-purpose consent.
//
// context.cn_pipl_consent_record must carry:
//   - purpose_id matching the action's declared purpose
//   - status == "active"
//   - collected_at within the last 12 months (P1Y)
//   - withdrawn == false
// -----------------------------------------------------------------------
forbid(
  principal,
  action in oyatie::Action::"pi::process",
  resource
)
when {
  resource.data_class == "PI_CN_PIPL" &&
  (
    context.cn_pipl_consent_record == null ||
    context.cn_pipl_consent_record.status != "active" ||
    context.cn_pipl_consent_record.withdrawn == true ||
    context.cn_pipl_consent_record.purpose_id != context.processing_purpose_id
  )
}
advice {
  "reason": "Processing PI_CN_PIPL data requires a valid, active, non-withdrawn per-purpose consent record. PIPL Article 14 mandates separate consent per purpose.",
  "audit_event_class": "CnPiplConsentDeny",
  "regulation_reference": "PIPL Article 13 (lawful basis); PIPL Article 14 (separate per-purpose consent)"
};

// -----------------------------------------------------------------------
// Rule 2: Forbid processing when consent record is older than 12 months.
//
// consent_age_days is computed by the consent substrate and injected into
// Cedar context at evaluation time.
// -----------------------------------------------------------------------
forbid(
  principal,
  action in oyatie::Action::"pi::process",
  resource
)
when {
  resource.data_class == "PI_CN_PIPL" &&
  context.cn_pipl_consent_record != null &&
  context.cn_pipl_consent_age_days > 365
}
advice {
  "reason": "CN-PIPL consent record has expired (older than 12 months). Consent renewal is required before processing can resume.",
  "audit_event_class": "CnPiplConsentExpiredDeny",
  "regulation_reference": "PIPL Article 14; CAC guidance on consent renewal; CN-PIPL-2021 pack policy: 12-month renewal interval"
};

// -----------------------------------------------------------------------
// Rule 3: Honor consent withdrawal immediately.
//
// Once context.cn_pipl_consent_record.withdrawn == true, all processing
// of that subject's PI_CN_PIPL data is denied regardless of other conditions.
// -----------------------------------------------------------------------
forbid(
  principal,
  action,
  resource
)
when {
  resource.data_class == "PI_CN_PIPL" &&
  context.cn_pipl_consent_record != null &&
  context.cn_pipl_consent_record.withdrawn == true
}
advice {
  "reason": "Consent has been withdrawn by the data subject. Processing of their PI_CN_PIPL data is immediately prohibited.",
  "audit_event_class": "CnPiplConsentWithdrawnDeny",
  "regulation_reference": "PIPL Article 15 (right to withdraw consent; withdrawal must not be harder than collection)"
};
