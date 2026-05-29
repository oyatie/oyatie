---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-ci-fitness-consolidation
impl_plan_id: IP-010-aggregation-indexer-full-stack
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, aggregation-index-generation]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: oya-governance-aggregation-indexer-{kernel..app} (full stack)

## Intent

Full `aggregation-indexer` BC delivery (kernel + domain + usecase + api + adapter + rest + worker + sdk + app). Reads per-µservice sources; regenerates central indices; refuses hand-edits at PR-time per F-04 mitigation.

## ChangeSet boundary

9 crates (full BC); plus the BLOCKER lane `oya-check-aggregation-index-generation` if not already migrated.

## Concrete File Targets

| Crate | Files |
|---|---|
| `-kernel` | `src/entities.rs` (`IndexEntry`, `Aggregation`, `DivergenceReport`), `src/ports.rs` (`PerMicroserviceSourceReader`, `CentralIndexWriter`, `DivergenceReporter`) |
| `-domain` | `src/aggregation_algebra.rs` (deterministic ordering rules), `src/divergence.rs` |
| `-usecase` | `src/prd_index_orchestrator.rs`, `src/catalog_aggregator.rs`, `src/spec_aggregator.rs` |
| `-adapter` | `src/fs_source_reader.rs`, `src/git_central_index_writer.rs` (scoped PAT; pre-push hook) |
| `-rest` | `src/handlers/aggregation.rs` |
| `-worker` | `src/main.rs` (per-PR + 5-min cron coalescing) |
| `-sdk` | `src/client.rs` |
| `-app` | `src/main.rs` |

## Code Shape

```rust
// usecase/src/prd_index_orchestrator.rs
pub async fn regenerate_prd_index(
    reader: &dyn PerMicroserviceSourceReader,
    writer: &dyn CentralIndexWriter,
) -> Result<Aggregation, UsecaseError> {
    let sources = reader.walk_per_microservice_sources().await?;
    let index = aggregate_prds(&sources);
    writer.write_path("docs/prds/INDEX.md", &index.render()).await?;
    Ok(index)
}
```

```rust
// adapter/src/git_central_index_writer.rs
pub struct ScopedGitWriter {
    pat: SecretString,
    allow_listed_paths: HashSet<PathBuf>,
}

#[async_trait::async_trait]
impl CentralIndexWriter for ScopedGitWriter {
    async fn write_path(&self, path: &str, content: &str) -> Result<(), KernelError> {
        // Pre-push hook per T-E-03: assert path ∈ allow_listed_paths
        if !self.allow_listed_paths.contains(Path::new(path)) {
            return Err(KernelError::ScopeOverrun(path.to_string()));
        }
        // git commit + push with scoped PAT
        todo!()
    }
}
```

```rust
// worker/src/main.rs (coalescing)
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let coalescer = Coalescer::new(Duration::from_secs(15 * 60));
    loop {
        let _ = coalescer.wait_for_burst().await;
        regenerate_all().await?;
    }
}
```

## Acceptance Gates

```bash
cargo check -p oya-governance-aggregation-indexer-{kernel,domain,usecase,api,adapter,rest,worker,sdk,app}
cargo nextest run --workspace
cargo run -p oya-dev-cli -- gate validate aggregation-index-generation
cargo run -p oya-dev-cli -- gate validate composition-root-only --crate oya-governance-aggregation-indexer-app
```

## Test Plan

| Test | Verifies |
|---|---|
| `kernel::test_aggregation_serde` | entity stability |
| `domain::test_deterministic_ordering` | Invariant 1 (per `lane-execution.md`) |
| `usecase::test_idempotent_across_3_runs` | Invariant 6 |
| `adapter::test_scope_overrun_refused` | T-E-03 mitigation |
| `worker::test_coalescing_15min_window` | scaling per `capacity-model.md` |

## Halt Conditions

- Regen non-deterministic across 2 runs → halt; investigate ordering rules.
- Hand-edit detected → halt; engage `runbooks/aggregation-rebuild.md` §C.
- PAT scope overrun → halt; engage ops-security.

## Next IP

[`IP-011-industry-best-practice-conformance-lane.md`](IP-011-industry-best-practice-conformance-lane.md)

## References

- ADR-0131 §"What stays central" + §"What moves".
- ADR-0115 (registry consolidation).
- `microservices/governance/threat-model.md` T-E-03.
- `microservices/governance/runbooks/aggregation-rebuild.md`.

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the explicit counterpart hook required by ADR-0328 D-20. Governance parity is evaluated against GitHub Advanced Security, SonarQube, Snyk, Trivy, Open Policy Agent, Backstage TechDocs, and Renovate. The implementation must state which of those controls it closes or deliberately does not target before promotion.
