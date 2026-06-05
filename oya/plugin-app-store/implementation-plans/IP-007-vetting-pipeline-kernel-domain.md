---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M04-ecosystem-substrate
phase: P01-plugin-app-store-substrate
impl_plan_id: IP-007-vetting-pipeline-kernel-domain
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-ecosystem
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, cargo-deny, lean-a1, lean-a2, port-location, layer-correctness, per-microservice-layout, authority-cohesion, vetting-pipeline-correctness, per-plugin-permission-enforcement]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007-vetting-pipeline-kernel-domain: vetting-pipeline kernel + domain (deterministic 8-stage orchestration)

## Intent

Pure kernel + domain for the vetting pipeline: 8 deterministic stages (signature-verification, vulnerability-scan, sandbox-isolation-validation, capability-scope-validation, data-use-boundary-check, accessibility-conformance, ai-act-classification, performance-budget). Each stage is pure-domain + port-mediated; the pipeline orchestrator composes them in fixed order; rejection on first failure with structured reason.

This IP advances PRD AC criteria per `microservices/plugin-app-store/PRD.md` §"Acceptance Criteria"; the durable-completion bar is that each acceptance gate exits 0 and each test in the §"Test Plan" passes deterministically across three consecutive runs (no flakes tolerated, per master-plan §No-silent-regression).

## ChangeSet boundary

New / modified crates:
- `oya-plugin-app-store-vetting-pipeline-kernel`
- `oya-plugin-app-store-vetting-pipeline-domain`

Each crate ships with: `Cargo.toml` declaring layer-correct dependencies (per ADR-0105 13-layer enum); `src/lib.rs` declaring the public surface; in-tree unit tests; a sibling integration test crate where ADR-0105 prescribes one.

ChangeSet authorship rule (per ADR-0110): claim → verify → done → promote. The claim_paths field on the ChangeSet manifest pins the exact globs above; no scope leakage allowed.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-vetting-pipeline-kernel/src/stage.rs` | create | Stage enum + StageResult enum |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-vetting-pipeline-kernel/src/ports.rs` | create | SignatureVerifier, VulnerabilityScanner, IsolationValidator, CapabilityValidator, DataUseBoundaryChecker, AccessibilityChecker, AiActClassifier, PerfBudgetChecker ports |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-vetting-pipeline-domain/src/pipeline.rs` | create | VettingPipeline::run deterministic stage orchestration |
| `microservices/plugin-app-store/src/crates/oya-plugin-app-store-vetting-pipeline-domain/src/rejection_reason.rs` | create | structured rejection-reason types (one per stage) |

| `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<ts>.json` | create | per-IP ChangeSet evidence per ADR-0110 |
| `microservices/plugin-app-store/catalog/<crate-name>.yaml` | create | catalog record per crate; consumed by `oya gate validate authority-cohesion` |

## Code Shape

```rust
pub async fn run<SV, VS, IV, CV, DUB, AC, AI, PB>(
    submission: &PluginSubmission,
    stages: &Stages<SV, VS, IV, CV, DUB, AC, AI, PB>,
) -> VettingDecision
where SV: SignatureVerifier, VS: VulnerabilityScanner, /* ... */ {
    if let Err(r) = stages.signature.verify(submission).await { return VettingDecision::Reject(r.into()); }
    if let Err(r) = stages.vulnerability.scan(submission).await { return VettingDecision::Reject(r.into()); }
    if let Err(r) = stages.isolation.validate(submission).await { return VettingDecision::Reject(r.into()); }
    if let Err(r) = stages.capability.validate(submission).await { return VettingDecision::Reject(r.into()); }
    if let Err(r) = stages.data_use_boundary.check(submission).await { return VettingDecision::Reject(r.into()); }
    if let Err(r) = stages.accessibility.check(submission).await { return VettingDecision::Reject(r.into()); }
    if let Err(r) = stages.ai_act.classify(submission).await { return VettingDecision::Reject(r.into()); }
    if let Err(r) = stages.perf_budget.check(submission).await { return VettingDecision::Reject(r.into()); }
    VettingDecision::Approve
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
cargo check -p oya-plugin-app-store-vetting-pipeline-kernel --all-features
cargo build -p oya-plugin-app-store-vetting-pipeline-kernel --all-features
cargo clippy -p oya-plugin-app-store-vetting-pipeline-kernel --all-features -- -D warnings
cargo nextest run -p oya-plugin-app-store-vetting-pipeline-kernel --all-features
cargo deny check --hide-inclusion-graph
cargo doc -p oya-plugin-app-store-vetting-pipeline-kernel --no-deps
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
| `test_pipeline_order_deterministic` | Stages run in declared order regardless of mock timings |
| `test_pipeline_rejects_on_first_stage_fail` | Later stages skipped if earlier fails |
| `test_rejection_reason_structured` | Reason is parseable into stage + specific cause |
| `test_pipeline_byte_deterministic_decision` | Identical submission → identical decision bytes |
| `test_pipeline_no_stage_skipped_on_approve` | Approve requires every stage to pass |

Coverage thresholds per layer class are inherited from PHASE-01 §"Per-IP Test Coverage Threshold"; this IP must meet them at merge time.

Test fixtures live under `microservices/plugin-app-store/tests/fixtures/ip-007-vetting-pipeline-kernel-domain/`. Determinism rule: every test that produces an artifact must produce byte-identical output on two consecutive runs (`diff -q` exits 0); enforced by the `deterministic-output` CI lane.

## Halt Conditions

This IP MUST halt (no merge; no promotion) if any of the following observe:

- Decision is non-deterministic on identical input.
- Pipeline can be re-ordered at runtime (must be compile-time fixed).
- Any stage's port returns Ok on a known-bad fixture.

Halt detection: each condition is encoded as a CI-failable assertion in the relevant lane; a single failure aborts the merge attempt and emits an entry to `microservices/plugin-app-store/evidence/halt-conditions-log.jsonl`.

## Rollback

If this IP is merged then later discovered to violate an invariant:

1. `cargo run -p oya-dev-cli -- vcs revert --changeset <id>` reverts the workspace.
2. The companion migration (Postgres / Valkey / Cedar policy / OpenBao binding) is reverted by the inverse SQL / inverse policy update emitted alongside this IP under `microservices/plugin-app-store/iac/migrations/rollback/`.
3. The hyperscaler-gate evidence file is moved to `evidence/rollback/<change_id>-rollback.json` with a structured rollback reason.
4. A follow-up IP is filed to re-attempt with the invariant-preserving fix.

## Evidence emission

On successful `oya vcs done`, this IP emits to `microservices/plugin-app-store/evidence/multispectrum/<change_id>-<unix_ts>.json`:

- `ip_id`: `IP-007-vetting-pipeline-kernel-domain`
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

[`IP-008-vetting-pipeline-cosign-trivy-wasmtime`](IP-008-vetting-pipeline-cosign-trivy-wasmtime.md)

## References

- PRD §vetting-pipeline
- ADR-0213 §3
