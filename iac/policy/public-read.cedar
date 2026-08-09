// cloud-iac µservice — public-read policy fragment
// Cedar v4
//
// Purpose: govern anonymous / unauthenticated access to non-sensitive public surfaces
//          (chart-allowlist schema validation, SLSA attestation lookups in Rekor's public
//          transparency log, anonymous health-status of the cloud-iac substrate itself).
// Owner:   ops-security + gtm-marketing
// References: threat-model.md T-I-04; dpia.md §"Cross-µservice aggregations"
// Imports: microservices/governance/policy/cedar-canonical-imports.cedar (SWEEP-I Slice 1 canonical envelope per ADR-0064)

// =============================================================================
// PERMIT 1 — anonymous read of platform health (system:cloud-iac self-SLO)
// =============================================================================
//
// The public status page surfaces cloud-iac's own substrate availability
// (render p99, apply p99, drift-coverage SLO) without any per-µservice attribution.

permit (
  principal == User::"anonymous",
  action == Action::"read_substrate_status",
  resource in SubstrateStatus::?s
)
when {
  resource has system_id &&
  resource.system_id == "system:cloud-iac" &&
  resource has data_class &&
  resource.data_class == "PUBLIC"
};

// =============================================================================
// PERMIT 2 — anonymous validation of Helm chart against allowlist + signature
// =============================================================================
//
// The public endpoint accepts a chart digest + signature claim and returns:
//   - whether the digest matches a chart in the public allowlist
//   - whether the Sigstore signature verifies against Fulcio + Rekor
// It does NOT reveal whether the chart is currently deployed in any oyatie µservice.

permit (
  principal == User::"anonymous",
  action == Action::"validate_chart_signature",
  resource in PublicEndpoint::"chart-signature-validator"
);

// =============================================================================
// PERMIT 3 — anonymous read of SLSA L3 attestation from Rekor (public transparency log)
// =============================================================================
//
// Rekor entries are public by design (the entire point of a transparency log).
// cloud-iac mirrors a read-cache of relevant Rekor entries; anonymous reads of
// PUBLIC-classed attestations are permitted.

permit (
  principal == User::"anonymous",
  action == Action::"read_slsa_attestation_public",
  resource in SlsaAttestation::?a
)
when {
  resource has data_class &&
  resource.data_class == "PUBLIC" &&
  resource has source &&
  resource.source == "rekor"
};

// =============================================================================
// FORBID — anonymous cannot read any non-public class
// =============================================================================

forbid (
  principal == User::"anonymous",
  action,
  resource
)
when {
  resource has data_class &&
  !(resource.data_class == "PUBLIC")
};

// =============================================================================
// FORBID — anonymous cannot read any µservice-scoped apply data
// =============================================================================
//
// Per-µservice apply state / drift reports / provenance records are all
// INTERNAL_ONLY / BEHAVIORAL_TENANT_PRODUCT / AUDIT classed; never public.

forbid (
  principal == User::"anonymous",
  action in [
    Action::"read_apply_state",
    Action::"read_drift_report",
    Action::"read_provenance",
    Action::"read_render_history",
    Action::"read_rollback_history",
    Action::"read_iac_manifests",
    Action::"read_audit_chain",
    Action::"read_argo_application_status"
  ],
  resource
);

// =============================================================================
// FORBID — anonymous cannot write or mutate anything
// =============================================================================

forbid (
  principal == User::"anonymous",
  action in [
    Action::"apply_iac",
    Action::"rollback_iac",
    Action::"write_apply_state",
    Action::"write_drift_report",
    Action::"write_provenance",
    Action::"write_artifact",
    Action::"mutate_argo_application",
    Action::"rotate_secret"
  ],
  resource
);

// =============================================================================
// FORBID — schema-validation endpoints cannot exfiltrate the submitted manifest
// =============================================================================
//
// The endpoints return conformance feedback only; submissions are not persisted,
// logged, or transmitted off-host. This is enforced at the endpoint code level
// and at this Cedar layer (no read_artifact / write_artifact for anonymous).

forbid (
  principal == User::"anonymous",
  action in [
    Action::"read_artifact",
    Action::"write_artifact",
    Action::"delete_artifact"
  ],
  resource
);
