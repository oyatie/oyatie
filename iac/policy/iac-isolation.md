---
doc_class: PolicySpec
title: IaC Apply-Scope Isolation Specification
microservice: cloud-iac
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-cloud-iac
deciders: architecture-governance, ops-security, axis-cloud-iac, privacy-governance
related_adrs: [ADR-0028, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - iac/threat-model.md (Trust Boundary 2, T-T-03, T-E-04)
  - iac/dpia.md (R-01)
  - iac/policy/tenant-scope.cedar
  - iac/policy/ci-scope.cedar
  - iac/policy/auditor-scope.cedar
  - iac/policy/public-read.cedar
review_cadence: quarterly + on every ArgoCD / Flux version upgrade
doc_status: published
---

# IaC Apply-Scope Isolation Specification (cloud-iac µservice)

## Purpose

Define the load-bearing apply-scope-isolation invariants of the cloud-iac substrate. **Cross-µservice apply is forbidden by default.** This document is the authoritative reference for SOC 2 examiners (CC6.1 / CC6.6), ISO 27001 auditors (A.5.15 / A.8.3 / A.8.4), GDPR Art. 32 reviewers, and HIPAA §164.312(a)(1) reviewers asking *"how does cloud-iac prevent µservice-A's apply from mutating µservice-B's resources?"*

## Apply-Scope Identity Model

### Scope derivation

```text
microservice_id     = <ms-slug> (matches microservices/<ms>/ folder name)
declared_namespaces = parsed from microservices/<ms>/iac/<chart>/values.yaml + targetNamespace
declared_cluster_roles = parsed from microservices/<ms>/iac/<chart>/templates/*.yaml (RBAC kinds)
apply_scope         = {
  microservice: microservice_id,
  pack: <pack-id>,
  environment: <staging | production>,
  namespaces: declared_namespaces,
  cluster_roles: declared_cluster_roles
}
```

Properties:
- The `microservice_id` is derived from the IaC source path; cannot be forged.
- `declared_namespaces` is bounded; max 5 namespaces per µservice (enforced at PR-time via LEAN check).
- `declared_cluster_roles` must reference roles within the µservice's declared namespaces; cross-namespace cluster role refused.
- Scope is sealed at PR-merge time; runtime mutation of scope without a successor-IP PR refused.

### Reserved scopes

The following apply-scopes are RESERVED and never issued for customer-µservice deployments:

| Reserved scope | Purpose | Write authority |
|---|---|---|
| `system:cloud-iac` | cloud-iac itself bootstrap apply | only `iac-applier-worker` SPIFFE identity for cloud-iac's own substrate |
| `system:cloud-k8s` | cluster substrate apply | only cloud-k8s µservice's SPIFFE identity |
| `system:cloud-secrets` | OpenBao substrate apply | only cloud-secrets µservice's SPIFFE identity |
| `system:observability` | observability stack apply | only observability µservice's SPIFFE identity |

Any inbound apply with `apply_scope.microservice = system:*` from a source other than the authorised SPIFFE identity is **rejected at iac-applier-worker** with HTTP 403 and emits `oya_cloud_iac_reserved_scope_violation_total` metric (alert on > 0 over 5m).

### Per-tenant scope inheritance

Each µservice may declare tenant-bound subsets (e.g., a tenant-specific overlay). The tenant's scope is inherited from the µservice's pack assignment; tenants cannot escalate scope beyond their µservice's declared bounds.

## Apply-Scope Invariants

### Invariant ISO-01: per-µservice apply scope enforced server-side

The iac-applier-worker validates every apply against the µservice's declared `apply_scope`. Resources outside scope refused with HTTP 403 and `apply_scope_violation` audit event. There is no exception. The CI lane `oya-cloud-iac-iac-apply-scope` validates this at PR-time; PRs that mutate outside declared scope fail to merge.

### Invariant ISO-02: Cluster RBAC enforces namespace-scoped admin

The iac-applier-worker ServiceAccount has `namespace-scoped admin` RBAC role on the declared `apply_scope.namespaces` only. The ServiceAccount has NO cluster-admin role. Cross-namespace mutation refused at the Kubernetes apiserver layer (defense-in-depth).

CI lane: `oya-check-applier-rbac-scope` validates RBAC bindings stay namespace-scoped.

### Invariant ISO-03: Cross-µservice apply forbidden by default

If a µservice's IaC references resources owned by another µservice (e.g., a ServiceAccount in another µservice's namespace), the apply is refused. Cross-µservice integration must flow through Workflow events or Ontology reads/writes, NOT through direct resource cross-references.

Exception path: explicit cross-µservice DAGs require architecture-governance approval + Cedar policy entitlement at `policy/cross-microservice-apply.cedar` (Slice D extension). Use cases (audited): cloud-iac applying its own substrate that touches workload-cluster system namespaces (bootstrap paradox, IP-015).

### Invariant ISO-04: ArgoCD Application scope-attestation

Every ArgoCD Application resource carries a `oya/scope-attestation` annotation signed by iac-applier-worker. The ArgoCD admission webhook rejects unsigned or scope-mismatched Applications. This prevents an attacker who compromised an ArgoCD admin token from creating Applications outside cloud-iac's scope-enforcement chain.

### Invariant ISO-05: SLSA L3 attestation required for every chart

Every Helm chart applied by cloud-iac must have an SLSA L3 build-provenance attestation verifiable against Sigstore Fulcio / Rekor. Unsigned charts refused at iac-applier-worker pre-apply.

CI lane: `oya-cloud-iac-provenance-slsa-l3` validates attestation chain.

### Invariant ISO-06: Apply rate limits per µservice

| Limit | Default | Enforcement |
|---|---|---|
| Max concurrent applies per µservice | 1 | iac-applier-worker queue serialisation |
| Max apply rate per µservice per hour | 6 | rate-limit at applier |
| Max rollback rate per µservice per 24h | 3 | rollback-engine rate-limit |

Excess returns HTTP 429.

### Invariant ISO-07: Apply-state index per-tenant read authorisation via Cedar

Every read request (Grafana UI, REST API, SDK) on the iac-state-index passes through a Cedar policy evaluator (per ADR-0140) that enforces:

```cedar
permit (
  principal,
  action in [Action::"read_apply_state", Action::"read_drift_report", Action::"read_provenance"],
  resource is ApplyStateRecord
) when {
  principal has microservice_scope &&
  resource has microservice &&
  resource.microservice in principal.microservice_scope
};
```

(Full fragment at `policy/tenant-scope.cedar`.) Non-matching reads return 403 + emit `oya_cloud_iac_unauthorized_read_attempt_total`.

## Failure Modes

### FM-01: ArgoCD admission webhook bypass attempt

**Behaviour:** Attacker attempts to create an ArgoCD Application without `oya/scope-attestation` annotation. ArgoCD admission webhook rejects; iac-applier emits `apply_scope_violation` event.

**Tenant impact:** None (write rejected).

**Detection:** `oya_cloud_iac_admission_violation_total > 0` over 5m fires page.

**Recovery:** Trace source; revoke credentials; root-cause.

### FM-02: Cluster RBAC drift (someone grants applier SA cluster-admin)

**Behaviour:** CI lane refuses merge; continuous RBAC-validator alarms via live cluster diff.

**Tenant impact:** Caught pre-merge OR within 5min via live-cluster diff.

**Detection:** `oya-check-applier-rbac-scope` lane + RBAC-state validator.

**Recovery:** Auto-rollback to declared RBAC via ArgoCD; ops-security incident if cause is intentional.

### FM-03: SLSA L3 attestation chain broken (Rekor outage; Fulcio outage; Cosign verify failure)

**Behaviour:** Apply refused; verification failure event emitted; alert fires.

**Tenant impact:** Promotion held pending substrate recovery.

**Detection:** `oya_cloud_iac_slsa_verify_failure_total > 0`.

**Recovery:** If transient (provider outage): retry. If chart actually unsigned: refuse + require chart re-signing.

### FM-04: Cross-µservice apply via shared resource (e.g., shared ConfigMap mutated)

**Behaviour:** Apply refused at scope-validation; manifest must be split into per-µservice apply units.

**Tenant impact:** Apply refused; PR refused; µservice owner notified.

**Detection:** `oya_cloud_iac_cross_microservice_violation_total > 0`.

**Recovery:** Refactor IaC: shared resources should be managed by the substrate µservice (cloud-k8s for namespace bootstrap, cloud-secrets for secrets) and referenced (not mutated) by µservices.

### FM-05: Apply-state index drift (live cluster ≠ index record)

**Behaviour:** drift-detector emits `DriftDetected` event; index updated to reflect live state OR reconciler reverts live state to match index (depending on whether mutation was authorised).

**Tenant impact:** Brief; reconciliation within ≤1h cycle.

**Detection:** drift-detector emits `DriftDetected{microservice, drift_score}` per cycle.

**Recovery:** Auto-reconcile via ArgoCD; if cause is unauthorised, ops-security incident.

### FM-06: Reserved-scope write from non-authorised SPIFFE identity

**Behaviour:** iac-applier-worker rejects; metric emitted; alert fires.

**Tenant impact:** None (write rejected).

**Detection:** `oya_cloud_iac_reserved_scope_violation_total > 0`.

**Recovery:** Trace source; revoke credentials; root-cause.

## Audit Trail

Every apply-scope boundary event is audit-chain-emitted per Bominal ADR-0028:

| Event | Emitter | Fields | Retention |
|---|---|---|---|
| Apply scope violation attempt | iac-applier-worker | `attempted_microservice, requested_resource, source_spiffe_id, timestamp, request_id` | ≥ 1y (≥ 6y HIPAA pack) |
| Reserved-scope write attempt | iac-applier-worker | `attempted_scope, source_spiffe_id, timestamp` | ≥ 1y |
| Unauthorised apply-state read attempt | iac-registry-rest (Cedar) | `principal_id, requested_microservice, action, timestamp` | ≥ 1y |
| RBAC drift detected | RBAC-state-validator | `namespace, old_binding, new_binding, detected_at` | ≥ 1y |
| SLSA L3 verify failure | iac-applier-worker | `chart, expected_attestation, actual_attestation, timestamp` | ≥ 1y |
| Cross-µservice apply violation | iac-validator | `microservice_a, microservice_b, attempted_resource` | ≥ 1y |
| ApplyExecuted | iac-applier-worker | `microservice, pack, environment, sha, actor, executed_at, signature` | ≥ 1y (≥ 6y HIPAA pack) |
| ApplyRolledBack | iac-rollback-worker | `microservice, from_sha, to_sha, reason, executed_at, signature` | ≥ 1y |

The audit log is itself stored in iac-state-index + Mimir under `tenant:oya-cloud-iac-self` and replicated to the `audit-chain` µservice for Merkle-tree sealing.

## Per-Pack Overlay

### pack-kr (KR PIPA + ISMS-P)

- KR PIPA Art. 29 (technical safeguards) maps to ISO-01..ISO-07 + FM-01..FM-06.
- Audit log retention ≥ 3y for tenant_scope production aligned with KR-FSS sectoral guidance.

### pack-us-healthcare (HIPAA)

- §164.312(a)(1) (access control) mapping: ISO-01..ISO-07.
- §164.312(b) (audit controls) mapping: Audit Trail table.
- §164.312(c)(1) (integrity): Ed25519 audit-chain seal on apply events.
- Audit retention ≥ 6 years per §164.316(b)(2).
- BAA with Covered Entity tenants includes cloud-iac as sub-processor.

### pack-eu (GDPR + EDPB)

- Art. 32(1)(a) pseudonymisation: tenant identifiers in apply-state index are hashed.
- Art. 32(1)(b) confidentiality + integrity: enforced via ISO-01..ISO-07 + audit-chain.
- Art. 32(1)(c) availability: HA + per-µservice rate limits.
- Art. 32(1)(d) regular testing: annual pen-test + quarterly chaos drill.
- Art. 25 by design and default: scope-isolation default-enabled.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/cloud-iac-isolation-overlay.md`.

## Verification

- cloud-ci/oya-ci governance gate `iac-apply-scope` is green in the branch-protected `oya-ci-required` context — exit 0.
- cloud-ci/oya-ci governance gate `applier-rbac-scope` is green in the branch-protected `oya-ci-required` context — exit 0.
- cloud-ci/oya-ci governance gate `slsa-l3-conformance` is green in the branch-protected `oya-ci-required` context — exit 0.
- cloud-ci/oya-ci governance gate `cedar-fragment-coverage` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context — exit 0.
- Annual pen-test against apply-scope boundary; documented in `runbooks/apply-scope-pentest.md` (Slice D).
- Quarterly chaos drill: induce reserved-scope write + cross-µservice apply attempt; verify rejection + alerting.

## References

- ADR-0028 (Bominal): audit-chain.
- ADR-0117: cloud-native infrastructure + residency.
- ADR-0139: agentic SLO-gated promotion.
- ADR-0131: per-microservice flat layout.
- ADR-0140: Cedar policy enforcement.
- `iac/threat-model.md` §"Trust Boundaries" + T-T-03, T-E-04.
- `iac/dpia.md` R-01.
- `iac/policy/{tenant-scope, ci-scope, auditor-scope, public-read}.cedar`.
- `microservices/observability/policy/tenant-isolation.md` (template).
- ArgoCD admission-webhook docs — `argo-cd.readthedocs.io/en/stable/operator-manual/admission-control/`.
- SPIFFE / SPIRE — `spiffe.io`.
- Sigstore Cosign — `docs.sigstore.dev/cosign/`.
- OpenSSF SLSA — `slsa.dev/spec/v1.0/`.
