---
doc_class: PolicySpec
title: Runtime Isolation Specification
microservice: foundry-runtime
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + axis-foundry-runtime
deciders: council-architecture, ops-security, axis-foundry-runtime, council-privacy
related_adrs: [ADR-0022, ADR-0025, ADR-0028, ADR-0117, ADR-0139, ADR-0131, ADR-0140 (retired per ADR-0145)]
related_artifacts:
  - microservices/foundry/threat-model.md (Trust Boundaries 2 + 3; T-S-01, T-I-01, T-E-01)
  - microservices/foundry/dpia.md (R-02, R-05, R-15, R-17)
  - microservices/foundry/policy/tenant-scope.cedar
  - microservices/foundry/policy/ci-scope.cedar
  - microservices/foundry/policy/auditor-scope.cedar
  - microservices/foundry/policy/public-read.cedar
review_cadence: quarterly + on every Redis/Postgres/Istio/SPIRE version upgrade
doc_status: published
---

# Runtime Isolation Specification (foundry-runtime µservice)

## Purpose

Define the load-bearing isolation invariants of the foundry-runtime substrate. This document is the authoritative reference for SOC 2 examiners (CC6.1 / CC6.2 / CC6.6), ISO 27001 auditors (A.5.15 / A.8.2 / A.8.3 / A.8.12), GDPR Art. 32 reviewers, KR PIPA Art. 23 / Art. 29 reviewers, HIPAA §164.312(a)(1) reviewers, and EU AI Act Art. 15 cybersecurity reviewers asking *"how does foundry-runtime prevent tenant-A from invoking tenant-B's capability, reading tenant-B's session, or escalating beyond its autonomy ceiling?"*

## Tenant Identity Model

### Tenant ID derivation

```text
canonical_tenant_id   = <opaque-string-issued-at-onboarding>
hashed_tenant_id      = sha256(canonical_tenant_id ++ deployment_salt)[..16]
X-Scope-OrgID header  = "tenant:" + hashed_tenant_id
```

- `canonical_tenant_id` lives in OpenBao tenant-resolver; runtime never sees raw.
- `deployment_salt` per-runtime-cluster, rotated 12mo.
- 16 hex chars (64 bits) → ~10¹⁹ collision-free.

### Reserved tenant IDs

The following `X-Scope-OrgID` values are RESERVED:

| Reserved tenant | Purpose | Write authority | Read authority |
|---|---|---|---|
| `tenant:oya-ci` | CI lane reads | reserved-namespace metric emitters only | CI runners via short-lived OpenBao read keys |
| `tenant:oya-self` | runtime self-observability (its own SLOs, capacity metrics) | runtime platform components | runtime operators + CI |
| `tenant:oya-system` | system capability templates (oyatie-owned built-in capabilities) | `foundry-supervisor` only | all tenants (read-only, for capability instantiation) |

Any inbound write with `X-Scope-OrgID = tenant:oya-*` from a non-authorised SPIFFE identity is **rejected at REST boundary** with HTTP 403 + emits `oya_tenant_reserved_id_violation_total` (alert > 0 / 5m).

### Tenant scope enumeration

```yaml
tenant_scope:
  enum: [trial, production, sandbox, internal]
```

Drives capacity-allocator + retention-policy + autonomy-ceiling defaults but does NOT relax isolation invariants.

## Runtime Isolation Invariants

### Invariant TI-01: Valkey tenant-prefix mandatory

Every Valkey key written by SessionStore MUST be prefixed `<hashed_tenant_id>:` regardless of code path. The LEAN check `oya-check-session-prefix-isolation` greps for any unprefixed Valkey op in `oya-foundry-runtime-session-state-adapter-redis` source; presence fails the lane. Integration test asserts cross-tenant read returns empty (not an error — returns empty so adversarial probing yields no signal).

### Invariant TI-02: Per-tenant Valkey ACL

Per-tenant Valkey ACL restricts the application user's command set to read/write on prefix `<hashed_tenant_id>:*` only. Cross-prefix reads return `(error) NOPERM`. Admin commands (`FLUSHDB`, `CONFIG SET`, `KEYS *`) require JIT OpenBao elevation + 2-person rule.

### Invariant TI-03: Postgres row-level security (RLS) on every multi-tenant table

Every Postgres table carrying tenant data declares `ALTER TABLE <t> ENABLE ROW LEVEL SECURITY` with a policy keyed on `tenant_id = current_setting('app.tenant_id')::text`. Application code MUST `SET LOCAL app.tenant_id = '<hashed_tenant_id>'` at the top of every transaction; the SessionStore adapter enforces this in connection-pool acquisition. CI lane `oya-check-postgres-rls-coverage` enforces RLS presence on every tenant-data table.

### Invariant TI-04: Cross-tenant query refusal at REST boundary (server-side)

Every REST endpoint extracts `X-Scope-OrgID` from validated OIDC bearer; Cedar policy at boundary refuses any read/write whose resource `tenant_id` doesn't match. Wildcard tenant queries are forbidden by Cedar fragment.

CI lane: `oya-check-foundry-runtime-no-wildcard-tenant-query` greps for any `tenant=*` or `*` literal in Postgres/Redis call sites; presence fails the lane.

### Invariant TI-05: No client-side tenant filtering as the only check

Application-layer tenant filtering is advisory; the load-bearing checks are server-side (Valkey ACL + Postgres RLS + Cedar at REST). This invariant exists to prevent reliance on a single layer.

### Invariant TI-06: Per-tenant + per-capability rate limits

| Limit | Default | Per tenant_scope | Enforcement |
|---|---|---|---|
| Max concurrent invocations | 100 / tenant | trial=10; production=100+; sandbox=20; internal=10000 | executor rate-limiter |
| Max dispatch rate | 100/sec / tenant | analogous | executor rate-limiter |
| Max session-state ops | 1000/sec / tenant | analogous | session-state adapter-redis |
| Per-capability concurrency | 10 / (tenant × capability_id) | analogous | usecase orchestrator |

Excess returns HTTP 429. Per-tenant overage metric (`oya_foundry_runtime_rate_limit_exceeded_total`) feeds tenant dashboard.

### Invariant TI-07: Reserved capability ID write authority

Capability IDs in the `oya:*` namespace are RESERVED. `foundry-supervisor` is the only authorised writer of `oya:*` capability descriptors; runtime refuses to mirror non-supervisor-signed `oya:*` descriptors.

### Invariant TI-08: AutonomyGate is the first step in usecase dispatch

Per ADR-0022 + threat-model T-E-01, the AutonomyGate port is invoked BEFORE any provider / guardrail / session-state I/O. Refusal emits `AutonomyViolationDetected` + returns 403 to caller. LEAN check `oya-check-autonomy-gate-presence` asserts call-graph order (gate → registry-cache → autonomy-tier-comparison → permit/deny → provider call).

### Invariant TI-09: Provider credentials never resident in runtime pod

Provider credentials live in `foundry-providers` µservice. The runtime asks providers to invoke an LLM with a session-scoped opaque token; runtime never receives the raw credential. Coredumps disabled in production runtime pods. Secret-scanner CI lane sweeps logs + commits. e2e test `provider-credential-isolation` materialises no provider secret in runtime memory under any tested path.

### Invariant TI-10: Sibling µservice traffic mTLS + SPIFFE

Every outbound call from runtime to a sibling (`foundry-providers`, `foundry-guardrails`, `foundry-evidence`, `foundry-supervisor`) is over mTLS with mutual SPIFFE identity verification. Plain HTTP is impossible at the network policy layer (NetworkPolicy default-deny + allowlist by SPIFFE).

### Invariant TI-11: Pod isolation hardening

Every runtime pod:
- Runs as non-root (UID ≥ 10000).
- Mounts root filesystem read-only.
- Enforces `seccomp: RuntimeDefault`.
- Enforces `AppArmor: runtime/default`.
- Carries `capabilities: drop: [ALL]` + minimal `add` per service requirement.
- Uses signed images via Cosign; unsigned image deployment refused by admission controller.
- Has egress NetworkPolicy default-deny except sibling SPIFFE-validated endpoints + Redis/Postgres in-cluster service IPs.

## Failure Modes

### FM-01: Valkey cluster partition / overload

Per `failure-modes.md` FM-01: replication; rate limits; autoscale. Tenant impact: bounded latency, no data loss within ingester window.

### FM-02: Valkey ACL drift

CI refuses merge; live-cluster ACL change reverted automatically; ops-security incident if intentional.

### FM-03: Postgres RLS regression

CI lane `oya-check-postgres-rls-coverage` refuses merge. Live-cluster mutation triggers continuous Helm-state-validator alarm + auto-rollback.

### FM-04: SPIFFE SVID compromise

SVID TTL ≤24h; runtime detects mTLS handshake failure on rotation; HA failover; sibling refusal.

### FM-05: AutonomyGate bypass (code path skips gate)

`oya-check-autonomy-gate-presence` lane refuses merge. Runtime metric `oya_foundry_runtime_dispatch_without_autonomy_gate_total > 0` triggers immediate Sev-1.

### FM-06: Cross-tenant via misconfigured Cedar fragment

CI lane fuzzes Cedar fragments + asserts deny-overrides semantics; mis-fragment fails. Live-cluster Cedar config drift caught by Helm-state-validator.

### FM-07: Provider credential leak via runtime log

Secret-scanner CI lane + log redactor + coredump-disabled pod hardening; on detection, OpenBao rotates credentials at `foundry-providers` and emits `provider_credential_rotated`; runtime drains in-flight invocations bound to old generation.

### FM-08: Pod escape via container CVE

Pod hardening (seccomp + AppArmor + RO FS + non-root + minimal caps); weekly base-image refresh; CVE scanning; signed images. On suspected escape: incident response + pod quarantine + forensic capture.

## Audit Trail

Every cross-tenant boundary event is audit-chain-emitted per Bominal ADR-0028:

| Event | Emitter | Fields | Retention |
|---|---|---|---|
| Tenant spoofing attempt | REST handler | `attempted_tenant_id, source_ip, source_spiffe_id, timestamp, request_id` | ≥1y (HIPAA 6y) |
| Reserved-namespace write attempt | Postgres RLS / Valkey ACL | `target_table_or_key, source_spiffe_id, attempted_tenant_id, timestamp` | ≥1y |
| Unauthorised read attempt | REST Cedar evaluator | `principal_id, requested_tenant_id, action, resource, timestamp` | ≥1y |
| AutonomyViolationDetected | usecase orchestrator | `tenant_id, capability_id, requested_tier, ceiling_at_check_time, timestamp` | indefinite |
| Provider credential rotation | OpenBao + foundry-providers | `provider, prev_credential_id_hash, new_credential_id_hash, rotated_by, timestamp` | ≥1y |
| Deployment salt rotation | OpenBao + ops-security | `prev_salt_hash, new_salt_hash, rotated_by, timestamp` | indefinite |
| Pod admission refusal | Kubernetes admission controller | `pod_image, refused_reason, requestor, timestamp` | ≥1y |
| Cross-pack invocation attempt | OTel collector | `source_pack, target_pack, tenant_id, capability_id, timestamp` | ≥1y |

Audit log itself stored under `tenant:oya-self` in Mimir + replicated to audit-chain µservice for Merkle sealing.

## Per-Pack Overlay

### pack-kr (KR PIPA + ISMS-P + FSC AI Guideline)

- KR PIPA Art. 29 maps to TI-01..TI-11.
- Audit retention ≥1y per PIPA Enforcement Decree Art. 30; extended to 3y for `tenant_scope: production`.
- KR PIPA Art. 23 hashed tenant-id with auxiliary → sensitive; salt rotation per Art. 29.
- KR FSC AI Guideline §3 → AutonomyGate (TI-08) is the human-in-loop technical control.

### pack-us-healthcare (HIPAA)

- §164.312(a)(1) → TI-01..TI-08, TI-10, TI-11.
- §164.312(b) → Audit Trail table.
- §164.312(e)(1) → mTLS + TLS 1.3 (TI-10).
- §164.502(b) → tenant-scope reads enforce least-data.
- Audit retention ≥6y per §164.316(b)(2).
- BAA `legal/baa-template.md`.

### pack-eu (GDPR + EU AI Act + EDPB)

- Art. 32(1)(a) pseudonymisation → hashed tenant_id.
- Art. 32(1)(b) confidentiality + integrity → TI-01..TI-11.
- Art. 32(1)(c) availability → Mimir HA + per-tenant rate limits.
- Art. 32(1)(d) regular testing → annual pen-test + quarterly chaos drill.
- Art. 25 by design + default → pseudonymisation + multi-tenancy default-on.
- EU AI Act Art. 14 → AutonomyGate.
- EU AI Act Art. 15 → TI-08, TI-09, TI-10, TI-11.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/foundry-runtime-isolation-overlay.md`.

## Verification

- `cargo run -p oya-dev-cli -- gate validate session-prefix-isolation` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate postgres-rls-coverage` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate autonomy-gate-presence` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage --microservice foundry-runtime` — exit 0.
- `cargo run -p oya-dev-cli -- gate validate no-wildcard-tenant-query --microservice foundry-runtime` — exit 0.
- Annual pen-test against tenant boundary.
- Quarterly chaos drill: induce reserved-tenant spoofing + autonomy ceiling bypass attempt; verify rejection + alerting.

## References

- ADR-0022; ADR-0025; ADR-0028 (audit-chain — Bominal); ADR-0117; ADR-0139; ADR-0131; ADR-0140.
- `microservices/foundry/threat-model.md` Trust Boundaries 2 + 3; T-S-01, T-I-01, T-E-01.
- `microservices/foundry/dpia.md` R-02, R-05, R-15, R-17.
- `microservices/foundry/policy/{tenant-scope,ci-scope,auditor-scope,public-read}.cedar`.
- `microservices/foundry/policy/data-residency.md`.
- SPIFFE / SPIRE — `spiffe.io`.
- OpenBao — `openbao.org`.
- Valkey ACL — `redis.io/docs/management/security/acl/`.
- Postgres RLS — `postgresql.org/docs/16/ddl-rowsecurity.html`.
- Kubernetes Pod Security Standards — `kubernetes.io/docs/concepts/security/pod-security-standards/`.
