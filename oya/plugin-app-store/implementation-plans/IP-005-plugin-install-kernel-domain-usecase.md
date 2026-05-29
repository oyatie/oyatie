---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M04-ecosystem-substrate
phase: P01-plugin-app-store-substrate
impl_plan_id: IP-005-plugin-install-kernel-domain-usecase
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005-plugin-install-kernel-domain-usecase: oya-plugin-app-store-plugin-install-{kernel,domain,usecase}

## Intent

Tenant-scoped install flow domain: Installation entity, declared-capability grant capture, plugin version pin, post-install configuration record. Pure domain; orchestrators in usecase. Cross-µservice calls (to per-plugin-permissions for Cedar materialization, to plugin-lifecycle for version state check) are port-mediated.

This IP advances PRD AC criteria per `microservices/plugin-app-store/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `oya-plugin-app-store-plugin-install-kernel`
- `oya-plugin-app-store-plugin-install-domain`
- `oya-plugin-app-store-plugin-install-usecase`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-install-kernel/src/entities.rs` | create | Installation, InstallationConfig, CapabilityGrant |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-install-kernel/src/ports.rs` | create | InstallationStore, CedarMaterializer, LifecycleChecker, RateLimitInitializer ports |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-install-domain/src/install_request.rs` | create | pure InstallRequest validation |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-install-domain/src/capability_grant.rs` | create | pure capability-grant validation against plugin manifest |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-install-usecase/src/install_orchestrator.rs` | create | InstallOrchestrator wires Cedar + lifecycle + rate-limit + storage |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-install-usecase/src/uninstall_orchestrator.rs` | create | UninstallOrchestrator |

| `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/plugin-app-store/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
pub struct InstallOrchestrator<IS, CM, LC, RI, AC>
where
    IS: InstallationStore,
    CM: CedarMaterializer,
    LC: LifecycleChecker,
    RI: RateLimitInitializer,
    AC: AuditChainEmitter,
{
    installation_store: Arc<IS>,
    cedar_materializer: Arc<CM>,
    lifecycle_checker: Arc<LC>,
    rate_limit_initializer: Arc<RI>,
    audit_chain: Arc<AC>,
}

impl<IS,CM,LC,RI,AC> InstallOrchestrator<IS,CM,LC,RI,AC> {
    pub async fn install(&self, request: InstallRequest) -> Result<Installation, InstallError> {
        // 1. Validate request against plugin manifest (declared capabilities present + within tenant policy)
        // 2. Verify plugin version is Published (lifecycle_checker)
        // 3. Allocate installation_id (ULID)
        // 4. Materialize per-installation Cedar policy fragment
        // 5. Initialize per-installation rate-limit token bucket
        // 6. Persist Installation row (Postgres)
        // 7. Emit PluginInstalled audit-chain seal event
        // 8. Return Installation
    }
}
```

Layer assignment compliance (per ADR-0105 13-layer enum):
- `*-kernel` crates declare port traits + value types only; no dependencies on other project crates.
- `*-domain` crates implement pure domain logic; depend on `*-kernel` only.
- `*-usecase` crates orchestrate domain calls; depend on `*-kernel` + `*-domain` only.
- `*-adapter*` crates implement port traits against concrete backends; depend on `*-kernel` + `*-domain` + `*-usecase`; NEVER imported directly by `*-rest` or `*-app`.
- `*-rest` crates expose HTTP routes; depend on `*-kernel` + `*-api` + `*-usecase`.
- `*-worker` crates run long-lived loops; same dependency rules as `*-rest`.
- `*-app` crates are composition roots; the only crates allowed to wire concrete `*-adapter*` instances to `*-usecase` ports.

Port-in-kernel rule (per ADR-0064 SWEEP-I) is enforced by the `port-location` CI lane.

## Acceptance Gates

All gates must exit 0 before this IP is `verified`:

```bash
cargo check -p oya-plugin-app-store-plugin-install-kernel --all-features
cargo build -p oya-plugin-app-store-plugin-install-kernel --all-features
cargo clippy -p oya-plugin-app-store-plugin-install-kernel --all-features -- -D warnings
cargo nextest run -p oya-plugin-app-store-plugin-install-kernel --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p oya-plugin-app-store-plugin-install-kernel --no-deps
cargo run -p oya-dev-cli -- gate validate lean-a1 --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate port-location --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice plugin-app-store
cargo run -p oya-dev-cli -- gate validate authority-cohesion
```

Domain-specific gates introduced or exercised by this IP:

```bash

```

## Test Plan

| Test | Verifies |
|---|---|
| `test_install_request_validation_rejects_undeclared_capability` | Tenant cannot grant a cap the plugin did not declare |
| `test_install_blocked_if_plugin_not_published` | LifecycleChecker integration |
| `test_install_idempotent_on_duplicate_request` | Same tenant + plugin + version → same installation_id |
| `test_uninstall_decommissions_cedar_policy_and_rate_limit` | Cleanup invariant |
| `test_install_emits_audit_chain_seal` | Seal event present for every install |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/plugin-app-store/tests/fixtures/ip-005-plugin-install-kernel-domain-usecase/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- Install completes without Cedar policy materialization.
- Install completes without rate-limit bucket creation.
- Audit chain seal missing for install or uninstall.

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/plugin-app-store/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/plugin-app-store/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-005-plugin-install-kernel-domain-usecase`
- `microservice`: `plugin-app-store`
- `milestone`: `M04-ecosystem-substrate`
- `phase`: `P01-plugin-app-store-substrate`
- `claim_paths`: every glob declared above
- `acceptance_lanes_green`: exhaustive list of CI lanes that ran and exited 0
- `test_count`: {unit, integration, e2e}
- `coverage_pct`: float
- `multispectrum_review_facets`: F1..F9 + A1..A7 + M1..M2 minimum
- `signature`: Ed25519 signing per ADR-0181

## Next IP

[`IP-006-plugin-install-rest-sdk-app`](IP-006-plugin-install-rest-sdk-app.md)

## References

- PRD §plugin-install
- ADR-0007 (Cedar)
