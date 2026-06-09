---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
impl_plan_id: IP-004-iac-renderer-domain-usecase
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-iac
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-004: oya-cloud-iac-iac-renderer-{domain,usecase,api}

## Intent

Implement the `domain` + `usecase` + `api` layers for iac-renderer per ADR-0105. Domain holds pure render math (dependency ordering, content-digest computation). Usecase orchestrates rendering via ports. API holds protocol-neutral typed I/O contracts.

## ChangeSet boundary

Three new crates: `-domain`, `-usecase`, `-api`. Catalog rows added.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-iac/src/crates/oya-cloud-iac-iac-renderer-domain/{Cargo.toml,src/lib.rs,src/dependency_ordering.rs,src/digest.rs}` | create |
| `microservices/cloud-iac/src/crates/oya-cloud-iac-iac-renderer-usecase/{Cargo.toml,src/lib.rs,src/orchestrator.rs}` | create |
| `microservices/cloud-iac/src/crates/oya-cloud-iac-iac-renderer-api/{Cargo.toml,src/lib.rs,src/requests.rs,src/responses.rs}` | create |
| `microservices/cloud-iac/catalog/oya-cloud-iac-iac-renderer-{domain,usecase,api}.yaml` | create |

## Code Shape

```rust
// domain/src/digest.rs
pub fn compute_content_digest(manifests: &[RenderedFragment]) -> ContentDigest {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    let mut sorted: Vec<_> = manifests.iter().collect();
    sorted.sort_by_key(|m| (&m.kind, &m.namespace, &m.name));
    for m in sorted {
        hasher.update(&m.canonical_bytes());
    }
    ContentDigest(hasher.finalize().into())
}

// domain/src/dependency_ordering.rs
pub fn topological_order(charts: &[ChartSource]) -> Result<Vec<&ChartSource>, RenderError> {
    // Kahn's algorithm over chart deps; refuses cycles
    ...
}
```

```rust
// usecase/src/orchestrator.rs
pub struct RenderOrchestrator<C, K, T, E> {
    chart_reader: C,
    kustomize_reader: K,
    terraform_planner: T,
    event_emitter: E,
}

impl<C, K, T, E> RenderOrchestrator<C, K, T, E>
where
    C: ChartSourceReader,
    K: KustomizeOverlayReader,
    T: TerraformPlanComputer,
    E: RenderEventEmitter,
{
    pub async fn render(&self, microservice: &str, sha: &str, pack: &str, env: Environment) -> Result<RenderedManifest, RenderError> {
        let charts = self.chart_reader.read(microservice, sha).await?;
        let overlays = self.kustomize_reader.read(microservice, sha, pack).await?;
        // ... apply overlays + topological order + plan terraform + compute digest
        let manifest = ...;
        self.event_emitter.emit_render_completed(&manifest).await?;
        Ok(manifest)
    }
}
```

## Acceptance Gates

```bash
cargo check --workspace -p oya-cloud-iac-iac-renderer-domain -p oya-cloud-iac-iac-renderer-usecase -p oya-cloud-iac-iac-renderer-api --all-features
cargo build --workspace -p oya-cloud-iac-iac-renderer-domain -p oya-cloud-iac-iac-renderer-usecase -p oya-cloud-iac-iac-renderer-api --all-features
cargo clippy --workspace -p oya-cloud-iac-iac-renderer-domain -p oya-cloud-iac-iac-renderer-usecase -p oya-cloud-iac-iac-renderer-api --all-features -- -D warnings
cargo nextest run --workspace -p oya-cloud-iac-iac-renderer-domain -p oya-cloud-iac-iac-renderer-usecase -p oya-cloud-iac-iac-renderer-api --all-features
cloud-ci/oya-ci governance gate `lean-a1` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
cloud-ci/oya-ci governance gate `layer-correctness` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
```

## Test Plan

| Test | Layer | Verifies |
|---|---|---|
| `test_digest_determinism` | domain | identical inputs → identical digest 100% of runs |
| `test_topological_order_no_cycle` | domain | DAG ordered; cycles refused |
| `test_topological_order_property` | domain | property test: random DAGs are ordered correctly |
| `test_orchestrator_happy_path` | usecase | mocked ports; verify output digest |
| `test_orchestrator_chart_read_fail` | usecase | error propagated |
| `test_api_request_serde` | api | request types roundtrip |

## Halt Conditions

- Domain code imports adapter — refactor.
- Usecase imports adapter directly — must use port.
- Any non-deterministic operation in digest computation — refactor.

## Next IP

[`IP-005-iac-renderer-adapter-trio.md`](IP-005-iac-renderer-adapter-trio.md)

## References

- ADR-0105.
- PRD §"Bounded Contexts" port-trait table.
