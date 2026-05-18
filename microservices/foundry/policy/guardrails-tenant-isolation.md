---
doc_class: PolicySpec
title: Tenant Isolation Specification
microservice: foundry-guardrails
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-foundry-guardrails
deciders: council-architecture, ops-security, axis-foundry-guardrails, council-privacy
related_adrs: [ADR-0022, ADR-0028, ADR-0117, ADR-0139, ADR-0131, ADR-0140]
related_artifacts:
  - microservices/foundry/threat-model.md (Trust Boundaries, T-S-01, T-I-03, T-E-02)
  - microservices/foundry/dpia.md (R-05, R-15)
  - microservices/foundry/policy/tenant-scope.cedar
  - microservices/foundry/policy/ci-scope.cedar
  - microservices/foundry/policy/auditor-scope.cedar
  - microservices/foundry/policy/public-read.cedar
review_cadence: quarterly + on every classifier-model rollout + on every Cedar bundle change
doc_status: published
---

# Tenant Isolation Specification (foundry-guardrails µservice)

## Purpose

Define the load-bearing tenant-isolation invariants of the foundry-guardrails substrate. Reference for SOC 2 / ISO 27001 / GDPR Art. 32 / KR PIPA Art. 23 + 29 / HIPAA §164.312(a)(1) reviewers asking *"how does foundry-guardrails prevent tenant-A's prompt from being processed against tenant-B's Cedar overlay, OR tenant-A from reading tenant-B's decisions?"*

## Tenant Identity Model

### Tenant ID derivation

Identical to observability per the foundry-runtime sibling shape:

```text
canonical_tenant_id   = <opaque-string-issued-at-onboarding>   (NOT stored in foundry-guardrails)
hashed_tenant_id      = sha256(canonical_tenant_id ++ deployment_salt)[..16]
X-Scope-OrgID header  = "tenant:" + hashed_tenant_id
```

Properties:
- `canonical_tenant_id` never received by foundry-guardrails; OpenBao tenant-resolver returns only hashed form.
- `deployment_salt` per-pack secret; rotated every 12 months; rotation event audit-emitted.
- `hashed_tenant_id` truncation to 16 hex chars (64-bit); collision-free for 10⁶ tenants.
- Mapping recoverable only by OpenBao; foundry-guardrails operators never see canonical IDs.

### Reserved tenant IDs

| Reserved tenant | Purpose | Write authority | Read authority |
|---|---|---|---|
| `tenant:oya-ci` | CI / rule-mutation / classifier-deploy lane | per `ci-scope.cedar` | per ci-scope |
| `tenant:oya-self` | foundry-guardrails self-observability metrics | foundry-guardrails platform components | foundry-guardrails operators + CI |
| `tenant:oya-aggregate` | anonymised cross-tenant aggregates for parity-matrix; DP-noise-protected | aggregator job; ε ≤ 1 per `policy/dp-analysis.md` | public dashboards (non-sensitive) |

## Postgres Rule-Store Tenancy Invariants

### TI-01: Row-level security (Postgres RLS) on every tenant-scoped table

Tables `rule_definitions`, `cedar_fragments`, `false_positive_escalations`, `decision_history_meta`, `tenant_overlay_registry` all have RLS enabled with policies enforcing `current_setting('app.tenant_id') = tenant_id`.

PostgreSQL connection-level setting `app.tenant_id` is set by the application after authenticating; never user-controllable from outside the typed `RuleStore` port.

LEAN lane `oya-foundry-fitness-postgres-rls-enforced` verifies RLS is on for every tenant-scoped table.

### TI-02: Typed RuleStore port — no raw SQL

The kernel `RuleStore` trait is the ONLY interface to Postgres rule-store. Direct SQL access from application code refused by `oya-foundry-fitness-no-raw-sql` lane (AST inspection).

### TI-03: Per-row `tenant_id` + `pack` columns

Every tenant-scoped row carries `tenant_id` + `pack` columns. Cross-tenant joins refused by RLS. Cross-pack joins additionally refused by per-pack-database deployment (each pack has its own Postgres cluster).

### TI-04: Pack-pinned database

Per pack: one HA primary + 2 RR Postgres cluster. Cross-pack queries impossible at network layer (NetworkPolicy + pack-pinned DNS).

### TI-05: Audit-mutation log (append-only)

`audit_mutation_log` table is append-only via Postgres trigger refusing UPDATE / DELETE. Mutations on rule_definitions emit log row + audit-chain seal.

### TI-06: Connection pool per tenant request

Application-side connection pool sets `app.tenant_id` at acquire-time + resets at release. Pool re-use never leaks tenancy.

LEAN lane `oya-foundry-fitness-pg-pool-tenancy-reset` validates.

## Cedar Engine Tenancy Invariants

### TI-07: Per-tenant Cedar overlay composition

Per-tenant Cedar overlay fragments are loaded at runtime from Postgres + composed under the base default-deny bundle. Composition is per-tenant scope; no cross-tenant overlay leak possible (each invocation evaluates against the principal's tenant's overlay + the base).

### TI-08: Cedar input bounded

REST API enforces request payload size ≤ 10 MB; Cedar engine timeout 10ms per evaluation; both prevent denial-of-service via large input.

### TI-09: Cedar deny-overrides semantics

Every Cedar bundle composed under deny-overrides. Any matching `forbid` overrides any matching `permit`. Defence-in-depth.

### TI-10: Cedar bundle integrity

Cedar bundle stored in ConfigMap (production)/in-process compile (sidecar); SHA tracked + verified at hot-reload time. Tamper attempts emit `oya_cedar_bundle_integrity_violation_total` metric.

## Classifier-Model Tenancy Invariants

### TI-11: Stateless classifier-model-serving pods

Classifier-serving pods accept prompt input + return verdict + score. NO per-tenant state retained between requests. Pods cannot leak tenant state.

LEAN lane `oya-foundry-fitness-statelessness --microservice foundry-guardrails` validates.

### TI-12: Per-pack classifier-model artifacts

Classifier-model artifacts pack-pinned per `policy/data-residency.md`. Cross-pack invocation impossible (network-isolated). Per-pack signing key.

### TI-13: Cosign signature verification at pod-start + at-runtime SHA double-check

Pod-start init container verifies Cosign signature. At-runtime each inference request carries expected model SHA; serving pod refuses mismatched SHA.

## Audit Trail

Every cross-tenant boundary event audit-chain-emitted per Bominal ADR-0028:

| Event | Emitter | Fields | Retention |
|---|---|---|---|
| Cross-tenant rule-store query attempt | Postgres pg_audit log | `attempted_tenant_id, principal, query_hash, timestamp` | ≥ 1y; 6y HIPAA |
| Cross-tenant Cedar overlay load attempt | Cedar engine | `attempted_tenant_id, expected_tenant_id, policy_id, timestamp` | ≥ 1y |
| Unauthorised classifier-model invocation | classifier-serving | `principal_spiffe, expected_spiffe, model_id, timestamp` | ≥ 1y |
| Cedar bundle integrity violation | Cedar engine | `expected_sha, observed_sha, timestamp` | ≥ 1y |
| Classifier-model integrity violation | classifier-serving init | `model_id, expected_sha, observed_sha, timestamp` | ≥ 1y |
| Cosign-key rotation | OpenBao | `prev_key_id, new_key_id, rotated_by, timestamp` | indefinite |
| Rule mutation | rule-store-writer SA | `rule_id, version, action, author_spiffe, commit_sha, pr_id, prior_version, timestamp` | indefinite (git history) |
| FP escalation | rest-endpoint | `escalation_id, decision_id, tenant_id, reason, timestamp` | ≥ 1y |

Audit log replicated to audit-chain µservice for Merkle sealing.

## Failure Modes (cross-references `failure-modes.md`)

| FM | Failure | Tenant impact | Severity |
|---|---|---|---|
| FM-05 | Cedar default-deny config drift | Potential cross-tenant entitlement leakage | Sev-1 |
| FM-12 | Cross-tenant rule leak detected | Potential confidentiality breach | Sev-1 |
| FM-13 | Pack misroute | Regulatory residency violation | Sev-1 |
| FM-14 | Classifier-model integrity violation | Affected pods stay down; HA replicas absorb | Sev-1/2 |

## Per-Pack Overlay

### pack-kr

- KR PIPA Art. 29 → TI-01..TI-13.
- Audit retention: ≥ 1y baseline; 3y for `tenant_scope: production` aligning with KR-FSS.
- PIPA Art. 23 sensitive: hashed tenant ID + auxiliary = sensitive; salt rotation per Art. 29.

### pack-us-healthcare

- §164.312(a)(1) → TI-01..TI-13.
- §164.312(b) → Audit Trail.
- §164.312(e)(1) → mTLS + TLS 1.3.
- §164.502(b) → typed port; minimum-necessary by construction.
- Audit retention extended to ≥ 6y.
- BAA in `legal/baa-template.md`.

### pack-eu (GDPR + EU AI Act)

- Art. 32(1)(a) pseudonymisation → X-Scope-OrgID.
- Art. 32(1)(b) confidentiality + integrity → TI-01..TI-13 + audit-chain.
- Art. 32(1)(c) availability → HA + per-tenant rate limits.
- Art. 32(1)(d) testing → annual pen-test + quarterly chaos drill.
- Art. 25 by design + default → pseudonymisation + multi-tenancy default-enabled.
- EU AI Act Art. 15 → classifier integrity + Cedar default-deny + per-pack residency.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlay at `regional-packs/<pack>/foundry-guardrails-tenant-isolation-overlay.md`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate postgres-rls-enforced --microservice foundry-guardrails` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate no-raw-sql --microservice foundry-guardrails` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate pg-pool-tenancy-reset --microservice foundry-guardrails` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage --microservice foundry-guardrails` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate statelessness --microservice foundry-guardrails` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate classifier-model-cosign-signed` — exit 0.
- Annual pen-test against tenant boundary.
- Quarterly chaos drill: induce cross-tenant rule-store query attempt + cross-tenant Cedar overlay load attempt; verify rejection + alerting.

## References

- ADR-0022, ADR-0028, ADR-0117, ADR-0139, ADR-0131, ADR-0140.
- `microservices/foundry/threat-model.md` §"Trust Boundaries" + T-S-01, T-I-03, T-E-02.
- `microservices/foundry/dpia.md` R-05, R-15.
- `microservices/foundry/policy/{tenant-scope, ci-scope, auditor-scope, public-read}.cedar`.
- `microservices/foundry/policy/data-residency.md`.
- `microservices/observability/policy/tenant-isolation.md` (sibling shape).
- PostgreSQL Row-Level Security — `postgresql.org/docs/16/ddl-rowsecurity.html`.
- SPIFFE / SPIRE — `spiffe.io`.
- OpenBao — `openbao.org`.
- Cedar v4 — `docs.cedarpolicy.com`.
