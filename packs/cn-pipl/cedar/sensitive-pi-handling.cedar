// CN-PIPL-2021 Cedar Fragment: sensitive-pi-handling
// fragment_id: pack-cn-pipl-2021-sensitive-pi-handling
// version: 1.0.0
// binding_adr: ADR-0251
// regulation: PIPL Article 28 — sensitive personal information includes: biometric data,
//             religious beliefs, specially-designated medical and health information, financial
//             accounts, tracking location information, and personal information of persons under
//             14. Processing requires: (a) specific and sufficient necessity, (b) DPIA per Article
//             55, (c) separate consent, (d) individual notification, (e) security measures.
//
// Hyperscaler precedent:
//   Alibaba Cloud's PIPL compliance toolkit (阿里云PIPL合规工具包) requires merchants to
//   register sensitive data types separately, obtain a DPIA sign-off, and enable enhanced
//   encryption (SM4 + HSM) before activating any sensitive-PI processing pipeline.

namespace pack::cn_pipl_2021;

// -----------------------------------------------------------------------
// Rule 1: Forbid processing PI_CN_SENSITIVE without a completed DPIA on file.
// -----------------------------------------------------------------------
forbid(
  principal,
  action in oyatie::Action::"pi::process",
  resource
)
when {
  resource.data_class == "PI_CN_SENSITIVE" &&
  context.cn_pipl_dpia_completed != true
}
advice {
  "reason": "Processing sensitive personal information (PI_CN_SENSITIVE) requires a completed Data Protection Impact Assessment (DPIA) per PIPL Article 55. No signed DPIA found for this processing activity.",
  "audit_event_class": "CnPiplSensitivePiDpiaMissingDeny",
  "regulation_reference": "PIPL Article 55 (personal information protection impact assessment); PIPL Article 28 (sensitive personal information requires heightened protection)"
};

// -----------------------------------------------------------------------
// Rule 2: Forbid processing PI_CN_SENSITIVE without separate explicit consent
//         specifically for the sensitive category.
// -----------------------------------------------------------------------
forbid(
  principal,
  action in oyatie::Action::"pi::process",
  resource
)
when {
  resource.data_class == "PI_CN_SENSITIVE" &&
  (
    context.cn_pipl_sensitive_consent_obtained != true ||
    context.cn_pipl_sensitive_consent_category != resource.sensitive_pi_category
  )
}
advice {
  "reason": "Sensitive personal information requires separate explicit consent for the specific sensitive category (biometric, medical, financial, location, religious, or minor data). Generic PI consent is insufficient.",
  "audit_event_class": "CnPiplSensitivePiConsentDeny",
  "regulation_reference": "PIPL Article 28 (separate consent for sensitive personal information); PIPL Article 29"
};

// -----------------------------------------------------------------------
// Rule 3: Require encryption_at_rest and encryption_in_transit context
//         flags to be true for any PI_CN_SENSITIVE processing action.
// -----------------------------------------------------------------------
forbid(
  principal,
  action in oyatie::Action::"pi::process",
  resource
)
when {
  resource.data_class == "PI_CN_SENSITIVE" &&
  (
    context.encryption_at_rest_confirmed != true ||
    context.encryption_in_transit_confirmed != true
  )
}
advice {
  "reason": "Sensitive personal information (PI_CN_SENSITIVE) requires encryption at rest and in transit to be confirmed before processing. Encryption_byok_required is true for this pack.",
  "audit_event_class": "CnPiplSensitivePiEncryptionDeny",
  "regulation_reference": "PIPL Article 28; CN-PIPL-2021 pack encryption_byok_required = true; ADR-0251 §D-10"
};
