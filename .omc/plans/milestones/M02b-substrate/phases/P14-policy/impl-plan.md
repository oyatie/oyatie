---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02b-substrate
phase: P14-policy
impl_plan_id: IP-001-policy-kernel-scaffold
status: pending
owner: council-architecture
blocked_by:
- impl_plan: P03-identity/IP-001
  reason: AuthzRequest.principal_id resolved from identity kernel User/Employee types
acceptance_lanes:
- cargo-check
- cargo-build
- cargo-clippy
- cargo-nextest
- cargo-deny
- lean-a1
- lean-a2
- lean-a3
- lean-a4
purpose: "Scaffolds all 9 policy crates across 2 BCs (policy-engine, policy-rule-packs), authors the complete Postgres DDL for `policy.tenant_rule_packs` and `policy.evaluation_log`, integrates the `cedar-policy` crate as the evaluation adapter."
---
# IP-001-policy-kernel-scaffold: Scaffold Policy Engine + Rule-Packs Kernel/Domain/Application/Adapter/gRPC/App — Cedar DDL + Port Traits

## Intent

Scaffolds all 9 policy crates across 2 BCs (policy-engine, policy-rule-packs), authors
the complete Postgres DDL for `policy.tenant_rule_packs` and `policy.evaluation_log`,
integrates the `cedar-policy` crate as the evaluation adapter, and implements all sealed
port traits. After this IP merges, every µservice in M02 can route authorization decisions
through `PolicyEvaluator::evaluate()` without owning any Cedar logic.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `Cargo.toml` | update | Add 9 policy crate workspace members; add cedar-policy = "4" to [workspace.dependencies] |
| `crates/oya-policy-engine-kernel/Cargo.toml` | create | Zero framework deps; async-trait + serde + uuid |
| `crates/oya-policy-engine-kernel/src/lib.rs` | create | pub mod types; pub mod ports; pub mod errors |
| `crates/oya-policy-engine-kernel/src/types.rs` | create | AuthzRequest, AuthzDecision, PolicyEffect, PolicyEntity, EvalLogEntry, EvalLogFilter |
| `crates/oya-policy-engine-kernel/src/ports.rs` | create | PolicyEvaluator + EvaluationLogStore — sealed |
| `crates/oya-policy-engine-kernel/src/errors.rs` | create | PolicyError enum |
| `crates/oya-policy-engine-domain/Cargo.toml` | create | Depends on kernel only |
| `crates/oya-policy-engine-domain/src/lib.rs` | create | AuthzRequestBuilder; effect_summary(); is_regulated_resource() |
| `crates/oya-policy-engine-application/Cargo.toml` | create | Depends on domain + kernel |
| `crates/oya-policy-engine-application/src/lib.rs` | create | EvaluateAuthzUseCase; BatchEvaluateUseCase; RecordEvalLogUseCase |
| `crates/oya-policy-engine-adapter/Cargo.toml` | create | Depends on application + domain + kernel + cedar-policy + sqlx |
| `crates/oya-policy-engine-adapter/src/lib.rs` | create | module declarations |
| `crates/oya-policy-engine-adapter/src/cedar_evaluator.rs` | create | CedarPolicyEvaluator: impl PolicyEvaluator; loads per-tenant rule packs; evaluates AuthzRequest |
| `crates/oya-policy-engine-adapter/src/pg_eval_log.rs` | create | PgEvaluationLogStore: impl EvaluationLogStore; tenant_id RLS |
| `crates/oya-policy-engine-adapter/src/audit_bridge.rs` | create | On Deny for regulated resource → calls AuditEventStore port (from oya-audit-chain-kernel) |
| `crates/oya-policy-engine-grpc/Cargo.toml` | create | Depends on application + kernel; tonic |
| `crates/oya-policy-engine-grpc/src/lib.rs` | create | PolicyService gRPC handler: Evaluate + BatchEvaluate |
| `crates/oya-policy-engine-app/Cargo.toml` | create | Composition root |
| `crates/oya-policy-engine-app/src/main.rs` | create | DI assembly |
| `crates/oya-policy-rule-packs-kernel/Cargo.toml` | create | Zero framework deps |
| `crates/oya-policy-rule-packs-kernel/src/lib.rs` | create | RulePackStore port; RulePack + RulePackVersion + RulePackDraft + RulePackId types |
| `crates/oya-policy-rule-packs-application/Cargo.toml` | create | Depends on rule-packs-kernel |
| `crates/oya-policy-rule-packs-application/src/lib.rs` | create | UpsertRulePackUseCase; GetActiveRulePacksUseCase; RollbackRulePackUseCase |
| `crates/oya-policy-rule-packs-adapter/Cargo.toml` | create | Depends on rule-packs-application + kernel + sqlx |
| `crates/oya-policy-rule-packs-adapter/src/lib.rs` | create | PgRulePackStore: impl RulePackStore; versioned storage; atomic activation |
| `contracts/policy.proto` | create | Protobuf: PolicyService rpc Evaluate / BatchEvaluate |
| `migrations/policy/V001__policy_schema.sql` | create | Full DDL: policy.tenant_rule_packs + policy.evaluation_log (see Code Shape) |
| `docs/standards/bounded-contexts.md` | update | Register policy-engine + policy-rule-packs BCs |

---

## Crate Naming

```
NAME: oya-policy-engine-kernel
JUSTIFICATION:
- microservice = policy: Cedar authorization µservice; ADR-0056 v4.1
- bc-tokens = engine: separate from rule-packs BC; Cedar evaluation loop
- layer = kernel: sealed ports PolicyEvaluator + EvaluationLogStore; pure types
- exemptions claimed: none

NAME: oya-policy-rule-packs-kernel
JUSTIFICATION:
- microservice = policy, bc-tokens = rule-packs: tenant-editable Cedar policy bundles
- layer = kernel: RulePackStore sealed port; RulePack version types
- exemptions claimed: none
```

---

## Code Shape

### `crates/oya-policy-engine-kernel/src/types.rs`

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type TenantId = Uuid;
pub type PrincipalId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthzRequest {
    pub principal_type: String,   // "User" | "Employee" | "System" | "Llm" | "Workflow"
    pub principal_id: Option<PrincipalId>,
    pub action: String,           // Cedar action: "Read" | "Write" | "Apply" | ...
    pub resource_type: String,    // Cedar resource type: "Object" | "WorkflowRun" | ...
    pub resource_id: Option<Uuid>,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PolicyEffect { Allow, Deny }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthzDecision {
    pub effect: PolicyEffect,
    pub determining_policies: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalLogEntry {
    pub log_id: Uuid,
    pub tenant_id: TenantId,
    pub principal_type: String,
    pub principal_id: Option<PrincipalId>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub effect: PolicyEffect,
    pub evaluated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct EvalLogFilter {
    pub principal_id: Option<PrincipalId>,
    pub effect: Option<PolicyEffect>,
    pub resource_type: Option<String>,
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub to: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: u32,
}
```

### `migrations/policy/V001__policy_schema.sql`

```sql
CREATE SCHEMA IF NOT EXISTS policy;

-- Per-tenant Cedar rule packs (versioned)
CREATE TABLE policy.tenant_rule_packs (
    pack_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    pack_name text NOT NULL,
    cedar_policy text NOT NULL,       -- raw Cedar policy text
    version int NOT NULL DEFAULT 1,
    is_active bool NOT NULL DEFAULT false,
    created_by uuid NOT NULL,
    activated_at timestamptz NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE policy.tenant_rule_packs FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON policy.tenant_rule_packs
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_rule_pack_active ON policy.tenant_rule_packs (tenant_id, pack_name)
    WHERE is_active = true;
CREATE INDEX idx_rule_packs_tenant ON policy.tenant_rule_packs (tenant_id, is_active);
COMMENT ON TABLE policy.tenant_rule_packs IS 'distribution_column:tenant_id';

-- Evaluation log (for audit; DENY on regulated resources forwarded to audit-chain)
CREATE TABLE policy.evaluation_log (
    log_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id uuid NOT NULL,
    principal_type text NOT NULL,
    principal_id uuid NULL,
    action text NOT NULL,
    resource_type text NOT NULL,
    resource_id uuid NULL,
    effect text NOT NULL CHECK (effect IN ('allow','deny')),
    determining_policies jsonb NOT NULL DEFAULT '[]'::jsonb,
    evaluated_at timestamptz NOT NULL DEFAULT now()
) PARTITION BY RANGE (evaluated_at);
ALTER TABLE policy.evaluation_log FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON policy.evaluation_log
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_eval_log_tenant_effect ON policy.evaluation_log (tenant_id, effect, evaluated_at DESC);
COMMENT ON TABLE policy.evaluation_log IS 'distribution_column:tenant_id';

-- Monthly partitions bootstrapped for current + next 3 months (migration creates these)
CREATE TABLE policy.evaluation_log_2026_05 PARTITION OF policy.evaluation_log
    FOR VALUES FROM ('2026-05-01') TO ('2026-06-01');
```

---

## Acceptance Gates

```bash
cargo check --workspace --all-features                               # exit 0
cargo build --workspace --all-features                               # exit 0
cargo clippy --workspace --all-features -- -D warnings               # exit 0
cargo nextest run --workspace --all-features                         # exit 0
cargo nextest run -p oya-policy-engine-adapter --test cedar_determinism   # exit 0
cargo nextest run -p oya-policy-engine-adapter --test audit_chain_bridge  # exit 0
cargo deny check                                                     # exit 0
oya gate validate lean-a1 --phase P14-policy
oya gate validate lean-a2 --phase P14-policy
oya gate validate lean-a3 --phase P14-policy
oya gate validate lean-a4 --phase P14-policy
oya gate validate shardability --phase P14-policy
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_authz_request_builder` | All required fields present; missing action returns Err |
| `test_policy_effect_deny_default` | Empty rule pack → Deny (Cedar default-deny semantics) |
| `test_cedar_determinism` | Same request + same rule pack → same decision across 1000 iterations |
| `test_eval_log_filter_by_effect` | EvalLogFilter with effect=Deny filters correctly |
| `test_regulated_resource_detection` | is_regulated_resource() returns true for PHI/PCI/PIPA types |

### Integration tests

| Test name | What it verifies |
|---|---|
| `integration_cedar_per_tenant_isolation` | Tenant A rule pack does not affect tenant B evaluation |
| `integration_rule_pack_versioning` | Upsert v2 pack; rollback to v1; active pack is v1 |
| `integration_audit_bridge_deny` | DENY on resource_type=PHI → audit-chain event appended |
| `integration_eval_log_rls` | Tenant A cannot read tenant B evaluation log |

### Load test

```javascript
// tests/load/smoke-policy.js
export const options = {
  vus: 200, duration: '60s',
  thresholds: {
    http_req_duration: ['p(99)<10'],   // p99 ≤10ms — tightest target in M02
    http_req_failed: ['rate<0.001'],
  },
};
```

| Scenario | Target | Pass criterion |
|---|---|---|
| Single authz evaluation | p99 ≤10ms at 10k RPS | `http_req_duration{p(99)}<10` |
| Batch evaluation (10 requests) | p99 ≤50ms at 2k RPS | `http_req_duration{p(99)}<50` |

---

## Clean Architecture Compliance

| Crate | Layer | Imports | Forbidden |
|---|---|---|---|
| `oya-policy-engine-kernel` | `kernel` | (nothing project-internal) | all other |
| `oya-policy-engine-domain` | `domain` | `kernel` | `application`, `adapter`, presentation, `app` |
| `oya-policy-engine-application` | `application` | `domain`, `kernel` | `adapter`, presentation, `app` |
| `oya-policy-engine-adapter` | `adapter` | `application`, `domain`, `kernel`, `cedar-policy` (external) | presentation, `app` |
| `oya-policy-rule-packs-adapter` | `adapter` | `rule-packs-application`, `rule-packs-kernel` | presentation, `app` |

Cross-product: `oya-policy-engine-adapter` imports `oya-audit-chain-kernel` for the
`AuditEventStore` port — kernel-only import, permitted by clean-arch rule (sdk/kernel
deps between µservices allowed).

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent council-architecture \
  --intent "IP-001-policy-kernel-scaffold: Cedar engine + rule packs" \
  --ttl 3600 \
  crates/oya-policy-engine-kernel/src/lib.rs::PolicyEvaluator \
  crates/oya-policy-engine-kernel/src/lib.rs::EvaluationLogStore \
  crates/oya-policy-rule-packs-kernel/src/lib.rs::RulePackStore \
  migrations/policy/V001__policy_schema.sql::policy.tenant_rule_packs
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-001-policy-kernel-scaffold merged; Cedar engine live; per-tenant rule packs versioned; evaluation log partitioned; audit-chain bridge for DENY on regulated resources; next: IP-002-policy-cedar-integration" \
  -i high \
  -k "M02,P14,IP-001,policy"
```

---

## Halt Conditions

1. `cedar-policy` crate API incompatibility with the `AuthzRequest` shape — escalate to architect.
2. Cedar evaluation non-determinism detected in `test_cedar_determinism` after 3 fix attempts.
3. LEAN-A2: policy adapter importing a product crate — escalate; add to audit bridge via port only.
4. `audit_chain_bridge` test fails after 3 attempts — escalate; do NOT weaken the fail-closed behavior.

---

## Next IP Pointer

`IP-002-policy-cedar-integration.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- ADR-0056 (BNF v4.1), ADR-0007 (Cedar), ADR-0028 (audit-chain)
- M02b-substrate-schema-foundation §6-N (policy outlined)
