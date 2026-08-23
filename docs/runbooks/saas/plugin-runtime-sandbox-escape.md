---
doc_status: published
---

# Oyatie Runbook — Plugin Runtime Sandbox Escape

> **Status:** Production procedure authored for the M03-P04/M03-P08 SaaS operator-documentation gate; readiness remains `target_non_claim` until changeset evidence and `presubmit` are green.
> **Owner:** `axis-saas + cloud-intelligence + central governance + ops-security`
> **Severity scope:** Sev 1.
> **Authority:** ADR-0036 plugin substrate trust model, ADR-0534 higher-trust gate marketplace extension, the SaaS Platform PRD, and M03-P04/M03-P08 planning references in `specs/masterplan.json`.
> **Last verified:** 2026-06-09 (SSOT chain checked against HANDOFF.md, registry/stores/*, specs/root-hub-pointers.json, specs/masterplan.json, and docs/products/saas-platform/PRD.md).

## Operator contract
- **Incident channel:** `#inc-saas-plugin-runtime`.
- **Primary invariant:** a production plugin cannot access network, filesystem, environment variables, process spawning, or ungranted capabilities outside the ADR-0036 `PluginContext`.
- **Tenant boundary:** quarantine and revocation are scoped by `tenant_id`, `plugin_id`, `plugin_version`, `installation_id`, `cell_id`, and trust tier.
- **Cloud authority:** runtime isolation changes are applied through the cloud control-plane / Kubernetes cell that hosts the tenant plugin workload. Offline sandbox reproduction is diagnostic only.
- **Audit event:** every quarantine, trust-tier change, install revoke, capability revoke, and recovery release emits `EVT-SAAS-PLUGIN-SANDBOX-ESCAPE-INCIDENT` with `incident_id`, `tenant_id`, `plugin_id`, `plugin_version`, `installation_id`, `cell_id`, `operator_id`, `decision_id`, and `evidence_hash`.
- **Stop condition:** the offending artifact cannot load in any production tenant, affected installations are revoked or pinned to a safe version, audit evidence is sealed, and the vetting gate that missed the escape has an owner.

## Trigger conditions
- Plugin runtime observes syscall, network, file, environment, clock/random, or process access outside the granted `PluginContext`.
- Wasmtime/WASI-P2 host trap indicates attempted host escape, resource-cap bypass, or component interface violation.
- Cosign/Rekor verification fails for an artifact that is installed or loadable.
- A plugin invokes a capability not declared in its manifest or not granted by the tenant.
- Marketplace, tenant admin, or external disclosure reports plugin behavior outside its approved trust tier.
- Gate artifact marketplace signals a stronger trust failure for any CI gate plugin derived from ADR-0534.

## First-response checklist
1. Declare Sev 1; assign incident commander, security lead, and plugin runtime owner.
2. Record `INCIDENT_ID`, `TENANT_ID`, `PLUGIN_ID`, `PLUGIN_VERSION`, `INSTALLATION_ID`, `CELL_ID`, and `TRUST_TIER`.
3. Preserve the plugin artifact digest, manifest, Cosign bundle, Rekor inclusion proof, SBOM, capability grant set, resource-cap snapshot, runtime trap/log excerpt, and audit-chain window.
4. Disable new loads for the exact artifact digest before changing marketplace listing state.
5. Determine whether the escape is artifact-specific, trust-tier-wide, runtime-host-wide, or cell-wide.
6. Emit containment audit evidence before deleting or rotating any operational evidence.

## Containment
- **Artifact-specific:** block the artifact digest globally, quarantine all installations of `plugin_id@plugin_version`, and freeze new installs.
- **Tenant-specific:** revoke the affected tenant installation and deny capability calls from that `installation_id` while preserving other tenants if evidence proves isolation.
- **Trust-tier-wide:** suspend promotion or loading for the affected trust tier; community and experimental tiers stay advisory or non-production per ADR-0036.
- **Runtime-host-wide:** open the plugin runtime circuit breaker in the affected cell and route plugin invocations to safe failure responses; do not run unverified fallback runtimes.
- **Credential or data-class exposure:** rotate only the secrets/capabilities that the evidence shows were reachable; page privacy/compliance for PHI, PCI, financial, or regulated-pack data classes.

## Diagnosis
Classify exactly one primary branch before recovery:

| Branch | Evidence | Required check |
|---|---|---|
| Manifest/capability mismatch | runtime call outside declared capabilities | Diff manifest, tenant grant, Cedar decision, and loaded capability set. |
| Wasmtime/WASI host bug or misconfiguration | host trap or interface escape from a valid artifact | Verify host version, WIT interface, denied imports, and component linker configuration. |
| Resource-cap enforcement failure | CPU/memory/network cap exceeded without abort | Inspect per-plugin per-tenant cap policy, quota counters, and abort audit emission. |
| Signing/provenance failure | Cosign/Rekor/SBOM verification missing or inconsistent | Validate artifact digest, Rekor inclusion, ISV identity, and marketplace listing digest. |
| Trust-tier policy failure | tier allowed a capability or tenant class it should not | Compare requested tier, approved tier, production tenant class, and capability allowlist. |
| CI gate artifact trust failure | ADR-0534 gate plugin could influence merge authority | Confirm gate binding kind, advisory/enforcing state, deterministic harness result, and consumer promotion record. |

## Recovery
1. Keep artifact quarantine active until the root-cause branch has a code/config/policy fix and gate evidence.
2. Revoke affected installations or pin them to the last known-safe artifact digest; do not silently downgrade tenants without audit evidence.
3. Patch runtime host, capability allowlist, Cedar policy, manifest validator, or marketplace vetting gate according to the classified branch.
4. Re-run plugin vetting with signature verification, SBOM/license scan, capability allowlist, resource-cap enforcement, and sandbox escape regression.
5. For ADR-0534 gate artifacts, run the higher-trust conformance harness: deterministic re-run, codes-match-manifest, pure/panic-free behavior, and advisory/enforcing disposition check.
6. Publish tenant and ISV notifications with installation impact, revoked capabilities, safe version, and required tenant action.
7. Re-enable loading by artifact digest and tenant cohort only after security and runtime owners sign off in the incident channel.

## Verify recovery
- Offending artifact digest is denied in production load paths for every affected cell.
- A safe plugin version or revoked installation state is visible for every affected tenant.
- Cosign/Rekor/SBOM/license and capability-policy gates pass for the replacement artifact.
- Runtime metrics show zero unauthorized syscall/import/capability attempts for three evaluator windows.
- Audit-chain contains sealed quarantine, revoke, and resolution events.
- Marketplace listing state and tenant install state agree; no stale install can invoke the quarantined digest.
- If a gate artifact was involved, it is advisory-only until the ADR-0534 higher-trust evidence is green.

## Rollback guardrails
- Do not load unsigned or unverifiable artifacts in production tenants.
- Do not grant raw network, filesystem, environment, clock/random, or process access as a workaround.
- Do not replace Wasmtime/WASI-P2 with an unreviewed runtime during an incident.
- Do not delete plugin evidence before audit-chain seal and security triage are complete.
- Do not restore a listing or installation until runtime, marketplace, tenant, and audit states are consistent.

## Post-incident
- Author the Sev 1 postmortem within the SLA from `docs/INCIDENT-MANAGEMENT.md`.
- Add the missing prevention to plugin vetting, runtime host tests, marketplace admission, or ADR-0534 gate conformance.
- Update tenant-facing and ISV-facing guidance if trust-tier semantics were ambiguous.
- Update this runbook with any branch, metric, or control that was missing.

## Sources
`docs/products/saas-platform/PRD.md`, `docs/teams/axis-saas/CHARTER.md`, `specs/masterplan.json` M03-P04/M03-P08 entries, `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`, `docs/decisions/ADR-0700-ci-admission-live-apex.md`, `docs/INCIDENT-MANAGEMENT.md`, `docs/SLO-CATALOG.md`, `docs/standards/prevention-doctrine.md`.
