---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-meta-iac-pipeline-substrate
impl_plan_id: IP-009-iac-rollback-engine
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-cloud-iac
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: oya-cloud-iac-iac-rollback-{kernel,domain,usecase,api,adapter}

## Intent

Scaffold the iac-rollback BC: kernel + domain + usecase + api + adapter. Coordinate with observability SLO gate's rollback primitive (per ADR-0139): when observability emits `RollbackExecuted` for a release-pointer, iac-rollback reverts the µservice's IaC state to the prior apply.

## ChangeSet boundary

Five new crates per ADR-0105. Catalog rows.

## Concrete File Targets

| Path | Action |
|---|---|
| `iac/src/crates/oya-cloud-iac-iac-rollback-kernel/{Cargo.toml,src/lib.rs,src/entities.rs,src/ports.rs}` | create |
| `iac/src/crates/oya-cloud-iac-iac-rollback-domain/{Cargo.toml,src/lib.rs,src/revert_plan.rs}` | create |
| `iac/src/crates/oya-cloud-iac-iac-rollback-usecase/{Cargo.toml,src/lib.rs,src/rollback_orchestrator.rs}` | create |
| `iac/src/crates/oya-cloud-iac-iac-rollback-api/{Cargo.toml,src/lib.rs}` | create |
| `iac/src/crates/oya-cloud-iac-iac-rollback-adapter/{Cargo.toml,src/lib.rs,src/event_emitter.rs,src/state_revert.rs}` | create |
| `iac/catalog/oya-cloud-iac-iac-rollback-*.yaml` | create (5 rows) |

## Code Shape

```rust
// kernel/src/entities.rs
pub struct RollbackTarget {
    #[data_class(INTERNAL_ONLY)] pub microservice: String,
    #[data_class(INTERNAL_ONLY)] pub pack: String,
    #[data_class(INTERNAL_ONLY)] pub environment: Environment,
    #[data_class(INTERNAL_ONLY)] pub to_apply_id: String,
    #[data_class(AUDIT)]         pub reason: RollbackReason,
}

pub enum RollbackReason {
    FastBurnBreach, SlowBurnBreach, DriftRemediation,
    SupplyChainResponse, ManualOverride, PostMortemRemediation,
}
```

```rust
// usecase/src/rollback_orchestrator.rs
pub struct RollbackOrchestrator<R, E, A> {
    state_reverter: R,
    event_emitter: E,
    applier_client: A,
}

impl<R, E, A> RollbackOrchestrator<R, E, A>
where R: StateRevertPlanComputer, E: RollbackEventEmitter, A: ApplierClient {
    pub async fn rollback(&self, target: &RollbackTarget) -> Result<RollbackResult, RollbackError> {
        // 1. Look up prior apply by to_apply_id
        let prior = self.applier_client.get_apply(target.to_apply_id).await?;
        // 2. Compute state-revert plan
        let plan = self.state_reverter.compute(target, &prior).await?;
        // 3. Re-apply prior manifest set via applier
        let result = self.applier_client.apply_with_rollback_marker(&prior, target.reason).await?;
        // 4. Emit ApplyRolledBack
        self.event_emitter.emit_rolled_back(target, &result).await?;
        Ok(result)
    }
}
```

## Acceptance Gates

```bash
cargo check --workspace -p oya-cloud-iac-iac-rollback-* --all-features
cargo nextest run --workspace -p oya-cloud-iac-iac-rollback-* --all-features
cloud-ci/oya-ci governance gate `layer-correctness` for --microservice cloud-iac is green in the branch-protected `oya-ci-required` context
```

## Test Plan

| Test | Layer | Verifies |
|---|---|---|
| `test_revert_plan_pure` | domain | revert-plan computation |
| `test_rollback_chain_depth_tracking` | domain | rollback-of-rollback depth tracked |
| `test_rollback_orchestrator_happy_path` | usecase | revert applied + event emitted |
| `test_rollback_signature_required` | usecase | unsigned rollback refused |
| `e2e_rollback_drill` | (cross-crate) | inject burn → rollback → cluster reverts ≤2min |

## Halt Conditions

- Rollback applied without audit-chain seal — fail.
- Rollback chain depth > 1 without ExecSponsor escalation path — fail.

## Next IP

[`IP-010-rest-surfaces.md`](IP-010-rest-surfaces.md)

## References

- ADR-0105; ADR-0139 §"Layer-B item 16 — Automated rollback".
- `iac/runbooks/rollback-orchestration.md`.
- `microservices/observability/runbooks/rollback.md` (SLO-gate parent flow).

## DR posture (per ADR-0343)

- Target source: `iac/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `seaweedfs_replicated`, `postgres_wal_g`].
- Surface evidence: `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/seaweedfs-volume-failover.md`, `iac/manifest.json`, `iac/IP-009-iac-rollback-engine.md`.
