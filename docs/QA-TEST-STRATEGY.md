---
purpose: Oyatie — QA + Test Strategy
doc_status: published
---

# Oyatie — QA + Test Strategy

> **Status:** Draft v0.1 — 2026-05-09.
> **Owner:** `axis-foundry` (engineering platform).
> **Companion:** [RELEASE-MANAGEMENT.md](RELEASE-MANAGEMENT.md), [SECURITY-PROGRAM.md](SECURITY-PROGRAM.md), `docs/standards/testing.md`.

## 1. Test pyramid (per Google testing model)

| Level | Goal | Quantity | Speed |
|---|---|---|---|
| Unit | One function / one branch | Highest count | < 100ms each |
| Integration | One aggregate / one adapter set | Mid | < 10s each |
| Component | One bounded context | Low-mid | < 60s each |
| End-to-end | Cross-axis flow | Lowest | minutes |
| Property | Pure-function modules; per ADR + #71 | Per pure-fn | seconds-minutes |
| Fuzz | Parsers / decoders; per #68 | Per parser | minutes-hours |
| Snapshot (insta) | Stable output shapes | Mid | < 1s each |
| Visual regression | Frontend pixel diffs | Low; key UIs | seconds-minutes |
| Performance / load | Hot paths | Per release | minutes-hours |
| Security / pen-test | Per-service quarterly | Per service | hours-days |
| Eval (Foundry capabilities) | Per capability | Per capability | seconds-minutes |
| Replay (against past traces) | Per capability + per axis | Per change | seconds-minutes |
| Chaos (fault injection) | Per release-candidate | Few | hours |
| DR drill (region failover) | Per quarter per region | One per quarter | hours |
| Cross-tenant isolation fuzz | Per quarter per axis (#129) | Per axis | hours |

## 2. Required test classes per change

| Change class | Required tests |
|---|---|
| Bug fix | Unit + Integration covering the regression |
| Feature add | Unit + Integration + Component; per-test golden snapshots for stable outputs |
| Cross-axis contract change | Unit + Integration + E2E spanning the contract; cross-axis cohesion-fitness |
| Plane class change | Cross-plane review; per-plane test coverage |
| Schema change | Schema-migration dry-run + backward-read for ≥ 2 versions |
| Privacy / data-class change | Privacy-class taxonomy unit tests + DSR-cascade integration |
| New Foundry capability | Per-capability eval set (golden + adversarial + per-region linguistic) |
| Per-axis preview-gate | Universal SLO baseline + cohesion fitness + license gate + DSR drill |

## 3. Fixture discipline

- **No mocks in integration tests** (per project memory). Hit a real Postgres, real KMS test instance, real Foundry sandbox.
- Test-fixture seed: per-tenant + per-region + per-class (PHI / PCI / PIPA-Art23 fixtures stored encrypted per ADR-0043 even in test).
- Per-test idempotency: tests can run in any order, in parallel, on a fresh slate.
- Insta snapshots stored under `tests/snapshots/`; reviewer approval flows through the Buck2 snapshot-review target.

## 4. Test runner

Buck2 is the authoritative build/test/check surface. Rust test execution should
flow through Buck2 targets that invoke the Rust test runner and LLVM
source-based coverage as needed. Direct Cargo/nextest commands are local mirror
loops only; never treat bare `cargo test` as authority.

```sh
buck2 test //...
```

Test sharding is owned by Buck2/Prow scheduling; local nextest partitioning is
acceptable only as developer evidence.

### Companion local-dev tools

- **Buck2 daemon + graph cache** — primary local feedback loop. Prefer Buck2
  target selection because it reuses graph state, narrows invalidation, and maps
  directly to Prow evidence.
- **Buck2 dependency-hygiene target** — owns unused-dependency detection. It may
  wrap Rust ecosystem analyzers internally, but PR evidence cites the Buck2
  target instead of a standalone Cargo tool. This keeps dependency bloat checks
  useful without adding another local feedback loop or shared authority surface.

Local pre-push evidence should cite the Buck2 target bundle that covers format,
lint, tests, dependency hygiene, and architecture-boundary checks. Prow posts the
trusted `oya-ci-required` status for protected-branch authority.

## 5. Flaky-test policy

- A test failing intermittently auto-emits `EVT-FLAKY-TEST-DETECTED`
- Foundry capability `flaky.test.classify` quarantines to `flaky/` lane
- Quarantined tests do not block PRs but DO block release candidates if not fixed in 14 days
- Owner team gets weekly digest of their flaky tests

## 6. Performance + benchmark policy

- Per-surface hot-path benchmarks tagged with `#[bench]`; per ADR + Issue #72
- Bench gate: regression > 20% from baseline = block
- Baselines stored at `bench/baselines/<surface>.json`; Cosign-signed
- Re-baseline only with explicit `ops-sre-reliability` + change-author approval

## 7. Coverage targets (per axis)

- Foundation kernels: ≥ 95% line + branch
- Domain layer: ≥ 90%
- App layer: ≥ 85%
- Adapters: ≥ 80% (with integration test focus over unit)
- API/worker: ≥ 80%
- Per-capability eval set: ≥ 20 cases per capability minimum

## 8. Sources
`docs/standards/testing.md`, ADR-0039, Google testing book, project memory rust test runner, CLAUDE.md.
