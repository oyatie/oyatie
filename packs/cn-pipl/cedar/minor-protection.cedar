// CN-PIPL-2021 Cedar Fragment: minor-protection
// fragment_id: pack-cn-pipl-2021-minor-protection
// version: 1.0.0
// binding_adr: ADR-0251
// regulation: PIPL Article 31 — processing personal information of persons under 14 requires
//             separate consent from the minor's guardian.
//             CAC Regulations on the Protection of Minors in Cyberspace, effective 2024-01-01 —
//             additional protections for minors' online activities including age verification,
//             guardian consent, and default-restricted feature surfaces.
//
// Hyperscaler precedent:
//   Tencent's real-name verification system (防沉迷系统) enforces under-14 guardian consent
//   and time-restrictions on gaming. Alibaba Cloud's PIPL compliance templates include separate
//   minor-consent workflows for merchant platforms processing data of persons under 14.

namespace pack::cn_pipl_2021;

// -----------------------------------------------------------------------
// Rule 1: Forbid processing personal information of users under 14 without
//         verified guardian consent.
// -----------------------------------------------------------------------
forbid(
  principal,
  action in oyatie::Action::"pi::process",
  resource
)
when {
  resource.data_class in ["PI_CN_PIPL", "PI_CN_SENSITIVE"] &&
  context.data_subject_age_years < 14 &&
  context.cn_pipl_guardian_consent_verified != true
}
advice {
  "reason": "Processing personal information of persons under 14 requires separate, verified consent from the minor's guardian or other guardian. PIPL Article 31 mandates this additional protection.",
  "audit_event_class": "CnPiplMinorProtectionDeny",
  "regulation_reference": "PIPL Article 31; CAC Regulations on the Protection of Minors in Cyberspace (effective 2024-01-01)"
};

// -----------------------------------------------------------------------
// Rule 2: Forbid profiling or automated decision-making targeting users under 14.
// -----------------------------------------------------------------------
forbid(
  principal,
  action in [
    oyatie::Action::"pi::profiling",
    oyatie::Action::"pi::automated-decision"
  ],
  resource
)
when {
  context.data_subject_age_years < 14
}
advice {
  "reason": "Profiling and automated decision-making targeting persons under 14 is prohibited under PIPL Article 31 and CAC Minors Cyberspace Regulations 2024.",
  "audit_event_class": "CnPiplMinorProfilingDeny",
  "regulation_reference": "PIPL Article 31; CAC Regulations on the Protection of Minors in Cyberspace, Article 23 (2024-01-01)"
};

// -----------------------------------------------------------------------
// Rule 3: Forbid cross-border transfer of personal information of under-14 users.
// -----------------------------------------------------------------------
forbid(
  principal,
  action == oyatie::Action::"data::cross-border-transfer",
  resource
)
when {
  resource.data_class in ["PI_CN_PIPL", "PI_CN_SENSITIVE"] &&
  context.data_subject_age_years < 14
}
advice {
  "reason": "Cross-border transfer of personal information of persons under 14 is unconditionally forbidden under PIPL Article 31 combined with Article 40 data localization requirements.",
  "audit_event_class": "CnPiplMinorCrossBorderDeny",
  "regulation_reference": "PIPL Article 31; PIPL Article 40; CAC Regulations on the Protection of Minors in Cyberspace (2024-01-01)"
};
