// intelligence µservice — EU AI Act high-risk (Annex III) policy fragment
// Cedar v4
//
// Purpose: refuse dispatch that matches an EU AI Act Annex III category without a tenant FRIA
//          (Fundamental Rights Impact Assessment per Art. 27) on file.
// Owner:   council-privacy + ops-legal
// References: EU AI Act 2024/1689 Arts. 6, 9, 10, 12, 13, 14, 15, 27 + Annex III;
//             dpia.md R-11; threat-model.md T-AIACT-01..05.
//
// CHANGE-CONTROL: per-pack-eu ops-legal sign-off required; multispectrum-review v2.4.0;
// CI lane oya-governance-eu-ai-act-annex-iii-refusal protects category coverage.

// =============================================================================
// Annex III Category 1 — Biometric identification + categorisation
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has pack &&
  resource.pack == "pack-eu" &&
  resource has annex_iii_categories &&
  "category_1_biometric_identification" in resource.annex_iii_categories &&
  resource has rfia_id &&
  resource.rfia_id == null
};

// =============================================================================
// Annex III Category 2 — Critical infrastructure
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has pack &&
  resource.pack == "pack-eu" &&
  resource has annex_iii_categories &&
  "category_2_critical_infrastructure" in resource.annex_iii_categories &&
  resource has rfia_id &&
  resource.rfia_id == null
};

// =============================================================================
// Annex III Category 3 — Education + vocational training
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has pack &&
  resource.pack == "pack-eu" &&
  resource has annex_iii_categories &&
  "category_3_education_vocational_training" in resource.annex_iii_categories &&
  resource has rfia_id &&
  resource.rfia_id == null
};

// =============================================================================
// Annex III Category 4 — Employment + workers' management + access to self-employment
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has pack &&
  resource.pack == "pack-eu" &&
  resource has annex_iii_categories &&
  "category_4_employment" in resource.annex_iii_categories &&
  resource has rfia_id &&
  resource.rfia_id == null
};

// =============================================================================
// Annex III Category 5 — Essential services + benefits (credit scoring + insurance)
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has pack &&
  resource.pack == "pack-eu" &&
  resource has annex_iii_categories &&
  "category_5_essential_services" in resource.annex_iii_categories &&
  resource has rfia_id &&
  resource.rfia_id == null
};

// =============================================================================
// Annex III Category 6 — Law enforcement
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has pack &&
  resource.pack == "pack-eu" &&
  resource has annex_iii_categories &&
  "category_6_law_enforcement" in resource.annex_iii_categories &&
  resource has rfia_id &&
  resource.rfia_id == null
};

// =============================================================================
// Annex III Category 7 — Migration + asylum + border-control management
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has pack &&
  resource.pack == "pack-eu" &&
  resource has annex_iii_categories &&
  "category_7_migration_asylum_border" in resource.annex_iii_categories &&
  resource has rfia_id &&
  resource.rfia_id == null
};

// =============================================================================
// Annex III Category 8 — Administration of justice + democratic processes
// =============================================================================

forbid (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has pack &&
  resource.pack == "pack-eu" &&
  resource has annex_iii_categories &&
  "category_8_justice_democratic_processes" in resource.annex_iii_categories &&
  resource has rfia_id &&
  resource.rfia_id == null
};

// =============================================================================
// PERMIT — dispatch with valid FRIA on file
// =============================================================================
//
// Once the tenant has executed an FRIA per Art. 27, dispatch may proceed; the FRIA covers human-
// oversight (Art. 14), transparency (Art. 13), accuracy + robustness (Art. 15), and record-
// keeping (Art. 12) for that category.

permit (
  principal,
  action == Action::"dispatch",
  resource in DispatchEnvelope::?d
)
when {
  resource has pack &&
  resource.pack == "pack-eu" &&
  resource has rfia_id &&
  resource.rfia_id != null &&
  resource has rfia_valid_until &&
  context has now &&
  resource.rfia_valid_until > context.now
};
