---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-studio-preview
phase: P01-visual-authoring-substrate
impl_plan_id: IP-009-license-gate-cedar-full
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workflow + ops-security
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-cedar-preview-required]
depends_on: [IP-004]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: license-gate-cedar — full BC (7 layers)

## Intent

Author the `license-gate-cedar` BC's full crate set: per-seat Cedar enforcement at editor open + per-action, audit-row emission on every decision, Postgres-backed seat-attribution store (append-only; Ed25519-signed at insert). Default-deny fail-closed. Implements PRD §"Security" per-seat licensing + threat-model T-T-05 + T-D-07.

## ChangeSet boundary

Seven crates:
- `oya-workflow-studio-license-gate-cedar-{kernel,domain,usecase,api,adapter,adapter-postgres,sdk}`

Per ADR-0105 Amendment 3 backend-qualified `adapter-postgres`.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-workflow-studio-license-gate-cedar-kernel/{Cargo.toml,src/{lib.rs,entities.rs,ports.rs}}` | create |
| `src/crates/oya-workflow-studio-license-gate-cedar-domain/{Cargo.toml,src/{lib.rs,cedar_eval.rs,entitlement.rs},tests/{per_seat_cedar.rs,default_deny.rs}}` | create |
| `src/crates/oya-workflow-studio-license-gate-cedar-usecase/{Cargo.toml,src/lib.rs}` | create |
| `src/crates/oya-workflow-studio-license-gate-cedar-api/{Cargo.toml,src/lib.rs}` | create |
| `src/crates/oya-workflow-studio-license-gate-cedar-adapter/{Cargo.toml,src/lib.rs}` | create |
| `src/crates/oya-workflow-studio-license-gate-cedar-adapter-postgres/{Cargo.toml,src/{lib.rs,postgres_impl.rs},tests/integration.rs,migrations/0001_init.sql}` | create |
| `src/crates/oya-workflow-studio-license-gate-cedar-sdk/{Cargo.toml,src/{lib.rs,client.rs}}` | create |
| `microservices/workflow-studio/catalog/oya-workflow-studio-license-gate-cedar-*.yaml` | create | 7 catalog records |

## Code Shape

`license-gate-cedar-domain/src/cedar_eval.rs`:

```rust
use cedar_policy::{Authorizer, Decision, Request, PolicySet};

pub struct LicenseEvaluator {
    authorizer: Authorizer,
    policy_set: PolicySet,
}

impl LicenseEvaluator {
    pub fn new(policy_set: PolicySet) -> Self {
        Self { authorizer: Authorizer::new(), policy_set }
    }

    /// FAIL-CLOSED: any error or "no permit matched" → deny.
    pub fn evaluate(&self, request: &Request) -> EvalResult {
        let response = self.authorizer.is_authorized(request, &self.policy_set, &cedar_policy::Entities::empty());
        match response.decision() {
            Decision::Allow => EvalResult::Allow,
            Decision::Deny => EvalResult::Deny { reasons: response.diagnostics().reason().cloned().collect() },
        }
    }
}

#[derive(Debug, Clone)]
pub enum EvalResult {
    Allow,
    Deny { reasons: Vec<cedar_policy::PolicyId> },
}
```

`license-gate-cedar-adapter-postgres/migrations/0001_init.sql`:

```sql
-- Append-only seat-attribution log.
CREATE TABLE seat_license_attributions (
    tenant_id TEXT NOT NULL,
    principal_oidc_sub TEXT NOT NULL,
    decision TEXT NOT NULL CHECK (decision IN ('allow', 'deny_no_seat', 'deny_revoked', 'deny_unknown_principal')),
    seat_count_used INTEGER NOT NULL,
    seat_count_limit INTEGER NOT NULL,
    cedar_policy_sha TEXT NOT NULL,
    decided_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ed25519_signature TEXT NOT NULL,
    PRIMARY KEY (tenant_id, principal_oidc_sub, decided_at)
);

-- Append-only: refuse UPDATE / DELETE.
CREATE OR REPLACE FUNCTION reject_mutation() RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'seat_license_attributions is append-only';
END; $$ LANGUAGE plpgsql;

CREATE TRIGGER reject_update BEFORE UPDATE ON seat_license_attributions
    FOR EACH ROW EXECUTE FUNCTION reject_mutation();
CREATE TRIGGER reject_delete BEFORE DELETE ON seat_license_attributions
    FOR EACH ROW EXECUTE FUNCTION reject_mutation();

-- Citus shard by tenant_id.
SELECT create_distributed_table('seat_license_attributions', 'tenant_id');

-- RLS.
ALTER TABLE seat_license_attributions ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON seat_license_attributions
    USING (tenant_id = current_setting('app.current_tenant_id', true));
```

## Acceptance Gates

```bash
cargo check -p oya-workflow-studio-license-gate-cedar-kernel \
  -p oya-workflow-studio-license-gate-cedar-domain \
  -p oya-workflow-studio-license-gate-cedar-usecase \
  -p oya-workflow-studio-license-gate-cedar-api \
  -p oya-workflow-studio-license-gate-cedar-adapter \
  -p oya-workflow-studio-license-gate-cedar-adapter-postgres \
  -p oya-workflow-studio-license-gate-cedar-sdk
cargo nextest run -p oya-workflow-studio-license-gate-cedar-domain --test per_seat_cedar
cargo nextest run -p oya-workflow-studio-license-gate-cedar-domain --test default_deny
cargo nextest run -p oya-workflow-studio-license-gate-cedar-adapter-postgres --test integration -- --include-ignored
cargo run -p oya-dev-cli -- gate validate cedar-preview-required --microservice workflow-studio
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_per_seat_cedar` | AC-08; seat-overage refuses editor open + audit row emitted |
| `test_default_deny` | unconfigured action → deny; never accidental allow |
| `test_cedar_evaluator_crash_fails_closed` | evaluator panic → return Deny (T-D-07) |
| `test_postgres_append_only` | UPDATE/DELETE on seat_license_attributions raises exception |
| `test_postgres_rls_cross_tenant_denied` | tenant-A connection sees zero tenant-B rows |
| `test_entitlement_claim_signature_verified` | unsigned entitlement rejected |

## Halt Conditions

- Default-deny test fails — STOP. Fail-closed is load-bearing.
- Postgres UPDATE/DELETE succeeds — STOP. T-T-05 invariant breach.
- Cross-tenant query returns rows — STOP. T-I-01 breach.

## Next IP

[`IP-010-jurisdiction-overlay-renderer-full.md`](IP-010-jurisdiction-overlay-renderer-full.md)

## References

- ADR-0140 (retired per ADR-0145) Cedar policy enforcement.
- threat-model.md T-T-05, T-D-07, T-I-01, T-E-03.
- Cedar v4 docs — `cedarpolicy.com`.
- AWS Cedar paper — "Cedar: A new language for expressive, fast, safe, and analyzable authorization".
- Citus RLS docs — `docs.citusdata.com/en/stable/develop/api_guc.html`.

## Counterpart Anchors
This workflow-studio IP is measured against the local Workflow Studio benchmark envelope: n8n for visual workflow authoring depth, Zapier for broad trigger/action accessibility, Make for visual branching and scenario ergonomics, and Workato for enterprise workflow governance. The IP must keep Oyatie's differentiator intact: canonical workflow_spec.v1 round-trip, Cedar-gated save/publish, tenant-scoped collaboration, and audit evidence rather than counterpart-specific runtime authority.

## Sustainability emission (per ADR-0344)

- metering_trigger_evidence: `microservices/workflow-studio/IP-009-license-gate-cedar-full.md` matched [`emission`, `attribution`, `per_seat`].
- per_call_audit_row_fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`, `provider`, `region`, `cell`.
- carbon_aware_scheduling: eligible only for deferrable work after compliance-pack exclusions and active RTO/RPO floors are satisfied; excluded from realtime Tier 0/1 paths.
- finops_portal_rollup_axes: `tenant`, `product`, `capability`, `provider`, `cell`.
- evidence_paths: [`microservices/workflow-studio/IP-009-license-gate-cedar-full.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/capacity-model.md`, `microservices/workflow-studio/compliance.md`, `microservices/workflow-studio/ARCHITECTURE.md`].

## Pod runtime tier (per ADR-0338)

- pod_runtime_tier: `0`.
- runtime_requirement: Kata Containers plus Cloud Hypervisor REQUIRED.
- justification: tenant-customer code exists in this IP execution path; trigger_terms: [`workflow-studio`].
- surface_evidence_paths: [`microservices/workflow-studio/IP-009-license-gate-cedar-full.md`, `microservices/workflow-studio/manifest.json`, `microservices/workflow-studio/templates/index.json`, `microservices/workflow-studio/templates/schemas/workflow-template.schema.json`, `microservices/workflow-studio/PRD.md`, `microservices/workflow-studio/ARCHITECTURE.md`].
