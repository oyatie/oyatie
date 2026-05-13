---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P14-policy
status: Proposed
entry_gate: |
  M02/P03-identity complete; oya-identity-kernel ships with User, Person, Organization,
  Employee types + sealed port traits; cargo check clean; grit done on all P03 symbols;
  ICM phase-handoff row emitted.
exit_gate: |
  All P14 impl-plan acceptance gates green; Cedar engine integrated (cedar-policy crate);
  per-tenant rule packs stored and loaded; evaluation log persisted; 2 BCs registered
  (policy-engine, policy-rule-packs); all crates pass cargo check/build/clippy/nextest/
  deny; oya gate validate lean-a1/a2/a3/a4 exit 0; grit done on all P14 symbols;
  ICM phase-complete row emitted.
depends_on:
  - milestone: M02
    phase: P03-identity
    reason: "Policy engine evaluates Cedar principal types (User, Employee, Organization) from identity kernel; per-tenant rule packs reference identity entity types; AuthzRequest carries principal_id resolved from identity."
owner_team: council-architecture
---

# P14-policy: Policy Substrate — Cedar Engine + Per-Tenant Rule Packs + Evaluation Audit Log

## Purpose

Delivers the policy substrate: the Cedar authorization engine that enforces per-tenant
rule packs across all µservices. Every product µservice routes authorization decisions
through this layer — no product crate implements its own authz logic. Per-tenant rule
packs enable B2B customization (e.g., a hospital tenant adds PHI access policies; a
corporate tenant restricts payroll reads to HR admins only).

Cedar is the chosen policy language per oyatie ADR-0007 (inherited from Bominal ADR-0007
Cedar authorization). Cedar's formally verified evaluation semantics ensure deterministic
authz decisions; the evaluation log provides a complete audit trail for compliance review.

The evaluation log integrates with the audit-chain substrate (P04-audit-chain): every
Deny decision that touches a regulated resource type (PHI, PCI, PIPA, children) is
forwarded to audit-chain for Merkle-sealed evidence. This integration is the load-bearing
bridge between policy enforcement and the cryptographic evidence ledger.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `policy` | `engine` | `crates/oya-policy-engine-kernel/` | `oya-policy-engine-kernel` |
| `policy` | `engine` | `crates/oya-policy-engine-domain/` | `oya-policy-engine-domain` |
| `policy` | `engine` | `crates/oya-policy-engine-application/` | `oya-policy-engine-application` |
| `policy` | `engine` | `crates/oya-policy-engine-adapter/` | `oya-policy-engine-adapter` |
| `policy` | `engine` | `crates/oya-policy-engine-grpc/` | `oya-policy-engine-grpc` |
| `policy` | `engine` | `crates/oya-policy-engine-app/` | `oya-policy-engine-app` |
| `policy` | `rule-packs` | `crates/oya-policy-rule-packs-kernel/` | `oya-policy-rule-packs-kernel` |
| `policy` | `rule-packs` | `crates/oya-policy-rule-packs-application/` | `oya-policy-rule-packs-application` |
| `policy` | `rule-packs` | `crates/oya-policy-rule-packs-adapter/` | `oya-policy-rule-packs-adapter` |
| `policy` | all | `contracts/policy.proto` | — |
| `policy` | all | `migrations/policy/V001__policy_schema.sql` | — |

Naming justification:

```
NAME: oya-policy-engine-kernel
JUSTIFICATION:
- microservice = policy: the policy authorization µservice; registered in
  [workspace.metadata.oya.microservices]; oyatie ADR-0007 (Cedar); ADR-0056 v4.1
- bc-tokens = engine: multiple BCs (engine, rule-packs) at same layer; engine BC
  owns the Cedar evaluation loop; rule-packs BC owns tenant rule storage
- layer = kernel: pure types (AuthzRequest, AuthzDecision, PolicyEntity) + sealed
  port traits (PolicyEvaluator, EvaluationLogStore); ZERO I/O; ADR-0056
- exemptions claimed: none

NAME: oya-policy-rule-packs-kernel
JUSTIFICATION:
- microservice = policy, bc-tokens = rule-packs: tenant-editable Cedar policy
  bundles; versioned; stored per tenant; separate BC from evaluation engine
- layer = kernel: RulePackStore port + RulePack + RulePackVersion types
- exemptions claimed: none
```

### Out-of-scope

- Per-jurisdiction Cedar overlays (Bominal ADR-0140) — deferred to M03
- WASM-sandboxed policy function execution — deferred to M03
- Policy authoring UI — deferred to M03 (Workflow Studio scope)

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`IP-001-policy-kernel-scaffold.md`](IP-001-policy-kernel-scaffold.md) | Scaffold all 9 policy crates; full DDL + Cedar entity schema; PolicyEvaluator + RulePackStore ports | pending | `council-architecture` |
| [`IP-002-policy-cedar-integration.md`](IP-002-policy-cedar-integration.md) | Wire cedar-policy crate into adapter; per-tenant rule pack loading; evaluation log persistence | pending | `council-architecture` |
| [`IP-003-policy-audit-chain-bridge.md`](IP-003-policy-audit-chain-bridge.md) | Bridge DENY decisions on regulated resources to audit-chain; fail-closed behavior | pending | `council-architecture` |
| [`IP-004-policy-load-tests.md`](IP-004-policy-load-tests.md) | k6 load tests; authz evaluation p99 ≤10ms at 10k RPS | pending | `council-architecture` |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features               # exit 0
cargo build --workspace --all-features               # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0
cargo doc --workspace --no-deps                      # exit 0; 0 warnings
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --phase P14-policy
oya gate validate lean-a2 --phase P14-policy
oya gate validate lean-a3 --phase P14-policy
oya gate validate lean-a4 --phase P14-policy
```

### Policy-specific gates

```bash
# Cedar evaluation determinism test
cargo nextest run -p oya-policy-engine-adapter --test cedar_determinism  # exit 0
# Deny-on-regulated-resource audit bridge test
cargo nextest run -p oya-policy-engine-adapter --test audit_chain_bridge  # exit 0
# Performance: authz eval p99 ≤10ms
k6 run tests/load/smoke-policy.js --env BASE_URL=http://localhost:3021   # thresholds green
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? | Presentation-only? |
|---|---|---|---|---|
| `oya-policy-engine-kernel` | `kernel` | Yes — PolicyEvaluator, EvaluationLogStore | N/A | No |
| `oya-policy-engine-domain` | `domain` | N/A | N/A | No |
| `oya-policy-engine-application` | `application` | N/A | N/A | No |
| `oya-policy-engine-adapter` | `adapter` | N/A | Yes — CedarPolicyEvaluator, PgEvaluationLogStore | No |
| `oya-policy-engine-grpc` | `grpc` | N/A | No direct adapter import | Yes |
| `oya-policy-engine-app` | `app` | N/A | Unrestricted inward | No |
| `oya-policy-rule-packs-kernel` | `kernel` | Yes — RulePackStore | N/A | No |
| `oya-policy-rule-packs-application` | `application` | N/A | N/A | No |
| `oya-policy-rule-packs-adapter` | `adapter` | N/A | Yes — PgRulePackStore | No |

### Port traits declared in kernel

```rust
// oya-policy-engine-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait PolicyEvaluator: Send + Sync + sealed::Sealed {
    async fn evaluate(&self, tenant_id: TenantId, request: AuthzRequest) -> Result<AuthzDecision, PolicyError>;
    async fn evaluate_batch(&self, tenant_id: TenantId, requests: Vec<AuthzRequest>) -> Result<Vec<AuthzDecision>, PolicyError>;
}

#[async_trait::async_trait]
pub trait EvaluationLogStore: Send + Sync + sealed::Sealed {
    async fn record(&self, tenant_id: TenantId, request: &AuthzRequest, decision: &AuthzDecision) -> Result<(), PolicyError>;
    async fn query(&self, tenant_id: TenantId, filter: EvalLogFilter) -> Result<Vec<EvalLogEntry>, PolicyError>;
}

// oya-policy-rule-packs-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait RulePackStore: Send + Sync + sealed::Sealed {
    async fn get_active(&self, tenant_id: TenantId) -> Result<Vec<RulePack>, PolicyError>;
    async fn upsert(&self, tenant_id: TenantId, pack: RulePackDraft) -> Result<RulePackVersion, PolicyError>;
    async fn rollback(&self, tenant_id: TenantId, pack_id: RulePackId, to_version: u32) -> Result<(), PolicyError>;
}
```

### CI lanes that must green

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P14-policy` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P14-policy` | exit 0 |
| `port-location` | `oya gate validate port-location --phase P14-policy` | exit 0 |
| `layer-correctness` | `oya gate validate layer-correctness --phase P14-policy` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P14-policy` | exit 0 |
| `shardability` | `oya gate validate shardability --phase P14-policy` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `policy-engine` | `policy` | pending |
| `policy-rule-packs` | `policy` | pending |

---

## Grit Claim Symbols

```
crates/oya-policy-engine-kernel/src/lib.rs::PolicyEvaluator
crates/oya-policy-engine-kernel/src/lib.rs::EvaluationLogStore
crates/oya-policy-rule-packs-kernel/src/lib.rs::RulePackStore
crates/oya-policy-engine-adapter/src/lib.rs::CedarPolicyEvaluator
contracts/policy.proto::PolicyService
migrations/policy/V001__policy_schema.sql::policy.tenant_rule_packs
```

TTL: `--ttl 3600`. Fallback: ICM `scaffold-locks-oyatie`.

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P14-policy started; milestone M02-substrate; scope: policy µservice 2 BCs (engine, rule-packs); Cedar integration; entry gate: P03-identity complete" \
  -i high \
  -k "M02,P14,phase-start,policy"

icm store \
  -t context-oyatie \
  -c "Phase P14-policy complete; Cedar engine live; per-tenant rule packs stored; evaluation log persisted; audit-chain bridge for DENY on regulated resources; next: P15-data-boundary" \
  -i high \
  -k "M02,P14,phase-complete,policy"
```

---

## References

- Bominal ADRs inherited: ADR-0007 (Cedar authorization), ADR-0028 (audit-chain)
- oyatie ADRs cited: ADR-0007, ADR-0056 v4.1
- M02-substrate-schema-foundation §6-N (policy outlined; expanded here)
- Memory: `feedback_clean_architecture_requirements.md`
