---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-guardrails-safety-and-policy-enforcement
impl_plan_id: IP-007-content-safety-rule-engine-kernel-and-postgres-adapter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-guardrails
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, data-class, rule-store-migrations-up-to-date]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007: oya-foundry-guardrails-content-safety-rule-engine-kernel + adapter-postgres

## Intent

Two crates: `-kernel` (port traits + `ContentCategory`, `RuleDefinition`, `RuleEvaluation`, `CategoryScore` entities) + `-adapter-postgres` (Postgres rule-store client; backend-qualified per ADR-0105 + sealed crate boundary). Implements `RuleStore` and `ContentSafetyRuleEvaluator` traits.

## ChangeSet boundary

Two crates: kernel + postgres adapter. Postgres is the canonical rule-store per PRD §"Bounded Contexts".

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-foundry-guardrails-content-safety-rule-engine-kernel/Cargo.toml` | create |
| `.../-kernel/src/{lib.rs,entities.rs,ports.rs,errors.rs}` | create |
| `src/crates/oya-foundry-guardrails-content-safety-rule-engine-adapter-postgres/Cargo.toml` | create |
| `.../-adapter-postgres/src/{lib.rs,client.rs,migrations.rs,rls.rs}` | create |
| `Cargo.toml` (workspace) | update |
| `catalog/{...kernel,...adapter-postgres}.yaml` | create |

## Crate Naming

```
NAME: oya-foundry-guardrails-content-safety-rule-engine-kernel
JUSTIFICATION: microservice=foundry-guardrails; bc=content-safety-rule-engine; layer=kernel

NAME: oya-foundry-guardrails-content-safety-rule-engine-adapter-postgres
JUSTIFICATION: microservice=foundry-guardrails; bc=content-safety-rule-engine; layer=adapter; backend=postgres per ADR-0105 §"Amendment 3" (Postgres canonical for relational rule-store with row-level tenancy)
```

## Code Shape

```rust
// kernel/src/entities.rs
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentCategory {
    Toxicity, SelfHarm, Sexual, Violence, Minors,
    Hate, Weapons, Illegal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RuleDefinition {
    #[data_class(INTERNAL_ONLY)]
    pub rule_id: String,
    #[data_class(INTERNAL_ONLY)]
    pub category: ContentCategory,
    #[data_class(INTERNAL_ONLY)]
    pub version: u32,
    #[data_class(INTERNAL_ONLY)]
    pub pack: String,
    #[data_class(INTERNAL_ONLY)]
    pub tenant_id: Option<String>,    // None = pack-default
    #[data_class(INTERNAL_ONLY)]
    pub threshold: f64,
    #[data_class(INTERNAL_ONLY)]
    pub status: RuleStatus,   // shadow | enforce | sunset
    #[data_class(AUDIT)]
    pub author: String,        // SPIFFE id
    #[data_class(AUDIT)]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait RuleStore: Send + Sync + Sealed {
    async fn list_for_pack(&self, pack: &str, tenant: Option<&str>) -> Result<Vec<RuleDefinition>, KernelError>;
    async fn record_mutation(&self, mutation: &RuleMutation) -> Result<(), KernelError>;
}

#[async_trait]
pub trait ContentSafetyRuleEvaluator: Send + Sync + Sealed {
    async fn evaluate(&self, text: &str, ctx: &EvalContext) -> Result<RuleEvaluation, KernelError>;
}
```

## Acceptance Gates

```bash
cargo check -p oya-foundry-guardrails-content-safety-rule-engine-kernel --all-features
cargo check -p oya-foundry-guardrails-content-safety-rule-engine-adapter-postgres --all-features
cargo nextest run -p oya-foundry-guardrails-content-safety-rule-engine-kernel --all-features
cargo nextest run -p oya-foundry-guardrails-content-safety-rule-engine-adapter-postgres --all-features --test postgres_integration
cargo run -p oya-dev-cli -- gate validate rule-store-migrations-up-to-date
cargo run -p oya-dev-cli -- gate validate data-class --crate oya-foundry-guardrails-content-safety-rule-engine-kernel
```

## Test Plan

Per adapter class: 1 test per port-impl method + ≥2 against real Postgres (testcontainers).

| Test | Verifies |
|---|---|
| `test_rule_definition_serde` | entity roundtrip |
| `test_rule_status_transitions` | shadow→enforce→sunset valid |
| `integration_postgres_list_for_pack` | RLS scopes to pack |
| `integration_postgres_cross_tenant_refused` | tenant-A query refused tenant-B rows |
| `integration_postgres_mutation_append_only` | UPDATE forbidden by trigger |

## Halt Conditions

- Direct SQL not through typed RuleStore port — refactor.
- Postgres RLS disabled — refuse merge.

## Next IP

[`IP-008-jailbreak-detector-ensemble.md`](IP-008-jailbreak-detector-ensemble.md)

## References

- ADR-0056, ADR-0105, ADR-0140 (retired per ADR-0145).
- `policy/tenant-isolation.md` (RLS spec).
- `iac/postgres/migrations/002-rls-policies.sql`.

## Wave 15 counterpart anchor

- Counterparts: AWS Bedrock Guardrails, OpenAI Moderation, Anthropic safety tooling, and NVIDIA NeMo Guardrails.
- Gap closure: this IP closes inline prompt, output, autonomy, jailbreak, and false-positive-budget enforcement before tenant-visible release.
- Evidence source: `microservices/intelligence/competitor-parity-matrix.md` plus the BC-local parity archive under `microservices/intelligence/bc-sources/` when present.
