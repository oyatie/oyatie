---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M06-ecosystem-developer-portal
phase: P01-developer-sdk-substrate
impl_plan_id: IP-009-dev-portal-app-submission-flow
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009-dev-portal-app-submission-flow: Plugin submission flow + vetting status stream

## Intent

Developer submits plugin via dev portal: manifest upload + Wasmtime artifact upload + sign with developer's ED25519 key; submission triggers plugin-app-store vetting pipeline; status streams back to portal per stage.

This IP advances PRD AC criteria per `microservices/developer-sdk/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `oya-developer-sdk-dev-portal-usecase`
- `oya-developer-sdk-dev-portal-rest`
- `oya-developer-sdk-dev-portal-app`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/developer-sdk/iac/backstage-extension/src/components/PluginSubmissionFlow.tsx` | create | submission UI |
| `microservices/developer-sdk/src/crates/oya-developer-sdk-dev-portal-rest/src/submission_routes.rs` | create | POST /v1/submissions; SSE /v1/submissions/:id/status |

| `microservices/developer-sdk/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/developer-sdk/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
async fn submit(/*...*/) -> Result<SubmissionId, SubmitError> {
    let signed = sign_artifact(&artifact, &developer.signing_key).await?;
    let submission_id = plugin_app_store_client.submit(SubmitRequest {
        manifest, artifact: signed,
    }).await?;
    Ok(submission_id)
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
cargo check -p oya-developer-sdk-dev-portal-usecase --all-features
cargo build -p oya-developer-sdk-dev-portal-usecase --all-features
cargo clippy -p oya-developer-sdk-dev-portal-usecase --all-features -- -D warnings
cargo nextest run -p oya-developer-sdk-dev-portal-usecase --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p oya-developer-sdk-dev-portal-usecase --no-deps
cargo run -p oya-dev-cli -- gate validate lean-a1 --microservice developer-sdk
cargo run -p oya-dev-cli -- gate validate lean-a2 --microservice developer-sdk
cargo run -p oya-dev-cli -- gate validate port-location --microservice developer-sdk
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice developer-sdk
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice developer-sdk
cargo run -p oya-dev-cli -- gate validate authority-cohesion
```

Domain-specific gates introduced or exercised by this IP:

```bash

```

## Test Plan

| Test | Verifies |
|---|---|
| `test_submission_signs_with_developer_key` | Cosign verifies result |
| `test_submission_cross_calls_plugin_app_store` | plugin-app-store ingests submission |
| `test_status_stream_sse_updates` | Each stage transition produces SSE event |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/developer-sdk/tests/fixtures/ip-009-dev-portal-app-submission-flow/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- Submission accepted without valid signature.
- Status stream misses a stage transition.

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/developer-sdk/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/developer-sdk/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/developer-sdk/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-009-dev-portal-app-submission-flow`
- `microservice`: `developer-sdk`
- `milestone`: `M06-ecosystem-developer-portal`
- `phase`: `P01-developer-sdk-substrate`
- `claim_paths`: every glob declared above
- `acceptance_lanes_green`: exhaustive list of CI lanes that ran and exited 0
- `test_count`: {unit, integration, e2e}
- `coverage_pct`: float
- `multispectrum_review_facets`: F1..F9 + A1..A7 + M1..M2 minimum
- `signature`: Ed25519 signing per ADR-0181

## Next IP

[`IP-010-payout-ach-sepa-kftc-fedwire`](IP-010-payout-ach-sepa-kftc-fedwire.md)

## References

- PRD §dev-portal
