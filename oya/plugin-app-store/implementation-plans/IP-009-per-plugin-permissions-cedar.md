---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M04-ecosystem-substrate
phase: P01-plugin-app-store-substrate
impl_plan_id: IP-009-per-plugin-permissions-cedar
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion, vetting-pipeline-correctness, per-plugin-permission-enforcement]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009-per-plugin-permissions-cedar: per-plugin Cedar policy fragment generator + install-time grant capture

## Intent

Generate a per-plugin Cedar policy fragment from the plugin manifest's declared capabilities; capture the tenant's grant decisions at install time; materialize the per-installation Cedar policy into the central evaluator owned by governance.

This IP advances PRD AC criteria per `microservices/plugin-app-store/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `oya-plugin-app-store-per-plugin-permissions-kernel`
- `oya-plugin-app-store-per-plugin-permissions-domain`
- `oya-plugin-app-store-per-plugin-permissions-usecase`
- `oya-plugin-app-store-per-plugin-permissions-adapter-cedar`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-per-plugin-permissions-domain/src/policy_template.rs` | create | Cedar policy template per declared capability |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-per-plugin-permissions-domain/src/grant_capture.rs` | create | TenantGrantSet capture + diff |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-per-plugin-permissions-adapter-cedar/src/lib.rs` | create | Cedar policy materializer adapter |

| `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/plugin-app-store/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
pub fn materialize(manifest: &PluginManifest, grants: &TenantGrantSet) -> String {
    let mut policy = String::new();
    for cap in &manifest.declared_capabilities {
        if grants.granted.contains(&cap.name) {
            policy.push_str(&format!(
                "permit(principal in Tenant::"{}", action == Action::"{}", resource in Tenant::"{}");",
                grants.tenant_id, cap.name, grants.tenant_id));
        }
    }
    policy
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
cargo check -p oya-plugin-app-store-per-plugin-permissions-kernel --all-features
cargo build -p oya-plugin-app-store-per-plugin-permissions-kernel --all-features
cargo clippy -p oya-plugin-app-store-per-plugin-permissions-kernel --all-features -- -D warnings
cargo nextest run -p oya-plugin-app-store-per-plugin-permissions-kernel --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p oya-plugin-app-store-per-plugin-permissions-kernel --no-deps
buck2 build //:quality-lane-registry-authority-check # lane=lean-a1 --microservice plugin-app-store
buck2 build //:quality-lane-registry-authority-check # lane=lean-a2 --microservice plugin-app-store
buck2 build //:quality-lane-registry-authority-check # lane=port-location --microservice plugin-app-store
buck2 build //:quality-lane-registry-authority-check # lane=layer-correctness --microservice plugin-app-store
buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice plugin-app-store
buck2 build //:quality-lane-registry-authority-check # lane=authority-cohesion
```

Domain-specific gates introduced or exercised by this IP:

```bash
buck2 build //:quality-lane-registry-authority-check # lane=vetting-pipeline-correctness --microservice plugin-app-store
buck2 build //:quality-lane-registry-authority-check # lane=per-plugin-permission-enforcement --microservice plugin-app-store
```

## Test Plan

| Test | Verifies |
|---|---|
| `test_policy_generation_per_capability` | One permit per granted capability |
| `test_policy_denies_ungranted_capability` | Cedar evaluator denies on ungranted |
| `test_grant_diff_idempotent` | Same grant set produces byte-identical policy |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/plugin-app-store/tests/fixtures/ip-009-per-plugin-permissions-cedar/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- Generated policy permits an ungranted capability.
- Cedar evaluator returns Allow for a forbidden action.

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/plugin-app-store/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/plugin-app-store/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-009-per-plugin-permissions-cedar`
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

[`IP-010-per-plugin-rate-limit`](IP-010-per-plugin-rate-limit.md)

## References

- ADR-0007 (Cedar)
- PRD §per-plugin-permissions
