// intelligence µservice — refusal-baseline policy fragment
// Cedar v4
//
// Purpose: minimum refusal floor. Refuses CSAM, violence-glorification, self-harm, extremism,
//          COPPA-protected-data without age-gating, PCI-scope, EU AI Act Annex III without FRIA.
// Owner:   council-privacy + ops-security + ops-legal
// References: ARCHITECTURE.md §7; threat-model.md T-T-01..T-T-02; dpia.md R-04..R-05, R-11, R-16.
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar
//
// CHANGE-CONTROL: 2-person-rule + multispectrum-review v2.4.0 + per-pack ops-legal sign-off
// on every mutation. CI lane oya-governance-refusal-baseline-floor protects the categories
// listed below from being removed.

// =============================================================================
// FORBID — CSAM category (hard floor; no exception)
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has classification &&
  resource.classification has csam_signal &&
  resource.classification.csam_signal == true
};

// =============================================================================
// FORBID — Violence-glorification category
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has classification &&
  resource.classification has violence_glorification_signal &&
  resource.classification.violence_glorification_signal == true
};

// =============================================================================
// FORBID — Self-harm category (without crisis-line referral wrap)
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has classification &&
  resource.classification has self_harm_signal &&
  resource.classification.self_harm_signal == true &&
  resource has consumer_safe_wrap &&
  resource.consumer_safe_wrap == false
};

// =============================================================================
// FORBID — Extremism category (terrorism / hate-group recruitment)
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has classification &&
  resource.classification has extremism_signal &&
  resource.classification.extremism_signal == true
};

// =============================================================================
// FORBID — COPPA: children-under-13 personal data without parental-consent grant
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has classification &&
  resource.classification has coppa_signal &&
  resource.classification.coppa_signal == true &&
  resource has parental_consent_grant_id &&
  resource.parental_consent_grant_id == null
};

// =============================================================================
// FORBID — PCI scope (cardholder data) — hard refusal; intelligence not PCI-eligible
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has classification &&
  resource.classification has pci_pan_signal &&
  resource.classification.pci_pan_signal == true
};

// =============================================================================
// FORBID — KR PIPA Art. 23 sensitive data without consent grant (pack-kr only)
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has pack &&
  resource.pack == "pack-kr" &&
  resource has classification &&
  resource.classification has pipa_art23_sensitive_signal &&
  resource.classification.pipa_art23_sensitive_signal == true &&
  resource has consent_grant_id &&
  resource.consent_grant_id == null
};

// =============================================================================
// FORBID — PHI dispatch without BAA-signed-provider (pack-us-healthcare)
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has pack &&
  resource.pack == "pack-us-healthcare" &&
  resource has classification &&
  resource.classification has phi_signal &&
  resource.classification.phi_signal == true &&
  resource has selected_provider &&
  resource.selected_provider has baa_signed &&
  resource.selected_provider.baa_signed == false
};

// =============================================================================
// FORBID — GDPR Art. 9 sensitive data without lawful basis (pack-eu)
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has pack &&
  resource.pack == "pack-eu" &&
  resource has classification &&
  resource.classification has gdpr_art9_sensitive_signal &&
  resource.classification.gdpr_art9_sensitive_signal == true &&
  resource has lawful_basis_art9 &&
  resource.lawful_basis_art9 == null
};
