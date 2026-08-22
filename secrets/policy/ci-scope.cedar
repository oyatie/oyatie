// cloud-secrets µservice — CI scope policy fragment
// Cedar v4; conforms to schema.cedarschema
//
// Purpose: bound the CI runner's read access to cloud-secrets resources.
//          CI runs the LEAN-A11 raw-secret-emission lane, the policy-scope
//          lane, IaC validation, and integration tests against a sandbox
//          OpenBao instance. CI never reads real tenant secrets.
//
// Owner:   ops-security + axis-governance
// References: secret-isolation.md §"TI-03 SecretReference is the law"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMIT 1 — CI runner reads policy + IaC + contracts (no secrets)
// =============================================================================

permit (
  principal == CIRunner::"ci",
  action in [
    Action::"read_policy_fragment",
    Action::"read_iac_manifest",
    Action::"read_contract",
    Action::"read_catalog_record",
    Action::"read_capability_card",
    Action::"read_runbook"
  ],
  resource in Repo::?r
)
when {
  resource has microservice &&
  resource.microservice == "cloud-secrets"
};

// =============================================================================
// PERMIT 2 — CI runner reads sandbox OpenBao tenant for integration tests
// =============================================================================
//
// A reserved sandbox tenant `tenant:cisandbox0000000` lives in a CI-only
// OpenBao instance (separate from prod packs). CI integration tests read +
// write within this sandbox namespace only.

permit (
  principal == CIRunner::"ci",
  action in [
    Action::"resolve_secret_reference",
    Action::"write_secret_reference",
    Action::"rotate_secret",
    Action::"revoke_secret"
  ],
  resource in TenantNamespace::"tenant:cisandbox0000000"
);

// =============================================================================
// PERMIT 3 — CI runner emits LEAN-A11 scan findings (write-only to a fixed bucket)
// =============================================================================

permit (
  principal == CIRunner::"ci",
  action == Action::"emit_scan_finding",
  resource in ScanFindingsBucket::"cloud-secrets-leak-scan-findings"
);

// =============================================================================
// FORBID — CI runner never resolves prod-pack secrets
// =============================================================================

forbid (
  principal == CIRunner::"ci",
  action == Action::"resolve_secret_reference",
  resource
)
unless {
  resource has tenant_id &&
  resource.tenant_id == "tenant:cisandbox0000000"
};

// =============================================================================
// FORBID — CI runner cannot access HSM signing or attestation operations
// =============================================================================

forbid (
  principal == CIRunner::"ci",
  action in [
    Action::"hsm_sign",
    Action::"hsm_attest",
    Action::"hsm_generate_kek",
    Action::"hsm_rotate_kek",
    Action::"hsm_export_partition_state"
  ],
  resource
);

// =============================================================================
// FORBID — CI runner cannot access break-glass operations
// =============================================================================

forbid (
  principal == CIRunner::"ci",
  action in [
    Action::"break_glass_admin",
    Action::"unseal_recovery",
    Action::"seal_namespace",
    Action::"reset_root_token"
  ],
  resource
);
