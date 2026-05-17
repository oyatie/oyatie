---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-agentic-slo-gated-promotion
impl_plan_id: IP-006-slo-engine-adapter
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-observability
acceptance_lanes: [cargo-check, cargo-nextest, lean-a1, oya-governance-mimir-tenancy-enforced]
---

# IP-006: oya-observability-slo-engine-adapter + adapter-mimir

## Intent

Two new crates: `-adapter` (generic adapters: OpenSLO YAML reader, Git refs store) + `-adapter-mimir` (backend-qualified per ADR-0105 §"Amendment 3": Mimir HTTP client emitting PromQL queries + writes per `tenant-isolation.md`).

## ChangeSet boundary

Two crates split for the same IP because they are tightly coupled (the YAML reader's output feeds the Mimir client). Both implement kernel port traits. Real Mimir HTTP only in `-adapter-mimir`; `-adapter` is backend-agnostic.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/observability/src/crates/oya-observability-slo-engine-adapter/Cargo.toml` | create |
| `.../-adapter/src/{lib.rs,openslo_yaml_reader.rs,git_refs_store.rs}` | create |
| `microservices/observability/src/crates/oya-observability-slo-engine-adapter-mimir/Cargo.toml` | create |
| `.../-adapter-mimir/src/{lib.rs,client.rs,tenant_header.rs,verdict_emitter.rs}` | create |
| `Cargo.toml` (workspace) | update — add both members |
| `microservices/observability/catalog/{oya-observability-slo-engine-adapter,oya-observability-slo-engine-adapter-mimir}.yaml` | create |

## Crate Naming

```
NAME: oya-observability-slo-engine-adapter
JUSTIFICATION: microservice=observability; bc=slo-engine; layer=adapter; ADR-0105 13-value enum

NAME: oya-observability-slo-engine-adapter-mimir
JUSTIFICATION: microservice=observability; bc=slo-engine; layer=adapter; backend-suffix=mimir per ADR-0105 §"Amendment 3" (`*-adapter-<backend>` pattern)
```

## Code Shape

```rust
// adapter-mimir/src/client.rs
use oya_observability_slo_engine_kernel::*;

pub struct MimirClient {
    http: reqwest::Client,
    base_url: url::Url,
    api_key_provider: Box<dyn ApiKeyProvider>,
}

#[async_trait]
impl PrometheusClient for MimirClient {
    async fn instant_query(&self, promql: &str, tenant: &MimirTenant) -> Result<InstantVector, PromqlError> {
        let api_key = self.api_key_provider.fetch(tenant).await?;
        let resp = self.http.get(self.base_url.join("/prometheus/api/v1/query")?)
            .query(&[("query", promql)])
            .header("X-Scope-OrgID", tenant.x_scope_org_id())
            .bearer_auth(api_key.expose_secret())
            .send().await?;
        // ... validate + parse
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-observability-slo-engine-adapter --all-features
cargo check -p oya-observability-slo-engine-adapter-mimir --all-features
cargo nextest run -p oya-observability-slo-engine-adapter --all-features
cargo nextest run -p oya-observability-slo-engine-adapter-mimir --all-features
# Integration against Mimir test container:
cargo nextest run -p oya-observability-slo-engine-adapter-mimir --test mimir_integration
cargo run -p oya-dev-cli -- gate validate mimir-tenancy-enforced
```

## Test Plan

Per PHASE-01 adapter class: 1 test per port-impl method + ≥2 against real backend (Mimir test container) + 0 e2e. Coverage 85% line / 75% branch.

| Test | Verifies |
|---|---|
| `test_openslo_yaml_parse_v1` | parses canonical OpenSLO v1.0 fixtures |
| `test_mimir_client_tenant_header` | every request carries `X-Scope-OrgID` |
| `test_mimir_client_rejects_wildcard_tenant` | refuses `tenant=*` queries (matches `tenant-isolation.md` TI-03) |
| `test_verdict_emitter_signs_ed25519` | every emit carries a SPIFFE-identity-signed payload |
| `integration_mimir_round_trip` | real Mimir container; write + read back |

## Halt Conditions

- Any port-impl reachable WITHOUT going through `kernel` trait — refactor
- Hard-coded tenant IDs in source — fail; pull from `MimirTenantResolver`

## Next IP

[`IP-007-slo-engine-rest.md`](IP-007-slo-engine-rest.md)

## References

- `microservices/observability/policy/tenant-isolation.md` TI-01..TI-07
- `/specs/agentic-slo-gated-promotion.json` §"mimir_multi_tenancy"
