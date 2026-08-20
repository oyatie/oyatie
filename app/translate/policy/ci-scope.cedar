// translate µservice — CI-scope policy fragment
// Cedar v4.2
//
// Purpose: enforce per-lane CI principal scope on translate µservice mutations
//          (TM updates, termbase imports, engine pin changes, EU AI Act
//          disclosure schema updates, policy/data-residency.md edits).
// Owner:   ops-security
// Audit:   every deny emits oya_translate_ci_unauthorized_attempt_total
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

forbid (
  principal,
  action,
  resource
);

// =============================================================================
// PERMIT 1 — CI principals scoped to specific lanes mutate µservice surfaces
// =============================================================================

permit (
  principal in CiPrincipal::?p,
  action in [
    Action::"branch_protection.read",
    Action::"branch_protection.update_lane",
    Action::"hyperscaler_gates.update_HG_TRANSLATE"
  ],
  resource == Resource::"github://oyatie/oyatie"
)
when {
  principal has lane &&
  principal.lane in [
    "oya-translate-credential-isolation",
    "oya-translate-data-residency-correctness",
    "oya-translate-eu-ai-act-disclosure",
    "oya-translate-quality-regression",
    "oya-translate-sdk-codegen"
  ]
};

// =============================================================================
// PERMIT 2 — engine adapter version pin updates require 2-person principal
// =============================================================================

permit (
  principal in CiPrincipal::?p,
  action == Action::"engine_pin.update",
  resource in EnginePin::?ep
)
when {
  principal has two_person_approval &&
  principal.two_person_approval == true &&
  principal has lane &&
  principal.lane in ["oya-translate-engine-pin-update"]
};

// =============================================================================
// PERMIT 3 — TM bulk-export by CI for audit reconstruction (Scenario D)
// =============================================================================

permit (
  principal in CiPrincipal::?p,
  action == Action::"tm.export.audit",
  resource in TenantResource::?r
)
when {
  principal has lane &&
  principal.lane == "oya-translate-audit-export" &&
  principal has audit_request_id &&
  resource has tenant_id
};

// =============================================================================
// FORBID — CI principals NEVER read tenant content directly
// =============================================================================

forbid (
  principal in CiPrincipal::?p,
  action in [
    Action::"translate.invoke",
    Action::"tm.lookup_value",
    Action::"doc.read_content"
  ],
  resource
);

// =============================================================================
// FORBID — CI principals NEVER resolve credentials
// =============================================================================

forbid (
  principal in CiPrincipal::?p,
  action == Action::"credential.resolve",
  resource
);
