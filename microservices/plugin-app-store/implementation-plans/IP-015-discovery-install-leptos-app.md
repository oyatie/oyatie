---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M04-ecosystem-substrate
phase: P01-plugin-app-store-substrate
impl_plan_id: IP-015-discovery-install-leptos-app
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015-discovery-install-leptos-app: discovery + install Leptos app (tenant-facing UI)

## Intent

Leptos web app for tenant operators: browse catalog, view plugin detail, install with permission grant modal, manage subscriptions, view per-plugin audit trail. Design-system parity with workflow-studio per ADR-0065 + ADR-0207 (WCAG 2.2 AA).

This IP advances PRD AC criteria per `microservices/plugin-app-store/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `oya-plugin-app-store-plugin-catalog-app`
- `oya-plugin-app-store-plugin-install-app`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-app/src/main.rs` | modify | wire Leptos SSR + WASM |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-app/src/components/catalog_browse.rs` | create | Leptos component |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-catalog-app/src/components/plugin_detail.rs` | create | Leptos component |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-plugin-install-app/src/components/permission_grant_modal.rs` | create | Apple-App-Store-style modal |

| `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/plugin-app-store/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
#[component]
pub fn PermissionGrantModal(
    plugin: ReadSignal<Plugin>,
    on_grant: Callback<TenantGrantSet>,
    on_deny: Callback<()>,
) -> impl IntoView {
    view! {
        <Modal>
            <h2>{move || plugin.get().display_name}" requests these capabilities"</h2>
            <ul>
                <For each=move || plugin.get().declared_capabilities key=|c| c.name.clone() let:cap>
                    <li><Checkbox label=cap.name.clone() /></li>
                </For>
            </ul>
            <button on:click=move |_| on_grant.call(/*...*/)>"Grant"</button>
            <button on:click=move |_| on_deny.call(())>"Deny"</button>
        </Modal>
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
cargo check -p oya-plugin-app-store-plugin-catalog-app --all-features
cargo build -p oya-plugin-app-store-plugin-catalog-app --all-features
cargo clippy -p oya-plugin-app-store-plugin-catalog-app --all-features -- -D warnings
cargo nextest run -p oya-plugin-app-store-plugin-catalog-app --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p oya-plugin-app-store-plugin-catalog-app --no-deps
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
| `test_leptos_catalog_browse_ssr` | SSR produces valid HTML |
| `test_leptos_permission_grant_modal_a11y` | axe-core passes; keyboard-navigable |
| `test_leptos_install_flow_e2e` | Playwright: browse → install → verify |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/plugin-app-store/tests/fixtures/ip-015-discovery-install-leptos-app/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- axe-core reports WCAG 2.2 AA failures.
- Install flow Playwright e2e fails.

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/plugin-app-store/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/plugin-app-store/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-015-discovery-install-leptos-app`
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

[`(phase exit_gate)`]((phase exit_gate).md)

## References

- ADR-0065 (Leptos)
- ADR-0207 (WCAG 2.2 AA)
- Leptos docs
