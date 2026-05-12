# Oyatie — Testing Standard

> **Owner:** `axis-foundry` (engineering platform).
> **Companion:** [QA-TEST-STRATEGY.md](../QA-TEST-STRATEGY.md), `cargo nextest` test runner, [TOOLCHAIN.md](../TOOLCHAIN.md).

## 1. Test pyramid (per Google testing model)

Per [QA-TEST-STRATEGY.md §1](../QA-TEST-STRATEGY.md). Summary:

- **Unit** (highest count; < 100ms each)
- **Integration** (mid; < 10s each)
- **Component** (one bounded context; < 60s)
- **End-to-end** (cross-axis; minutes)
- **Property** (per pure-function module; seconds-minutes)
- **Fuzz** (per parser / decoder; minutes-hours)
- **Snapshot** (insta; < 1s each)
- **Visual regression** (frontend; seconds-minutes)
- **Performance / load** (per release; minutes-hours)
- **Security / pen-test** (per service quarterly; hours-days)
- **Eval** (per Foundry capability)
- **Replay** (against past traces)
- **Chaos** (per release-candidate)
- **DR drill** (per quarter per region)
- **Cross-tenant isolation fuzz** (per quarter per axis)

## 2. Required per change class

Per [QA-TEST-STRATEGY.md §2](../QA-TEST-STRATEGY.md). Bug fix → unit + integration; feature → unit + integration + component; cross-axis contract → + E2E spanning the contract; etc.

## 3. Fixture discipline

- **No mocks in integration tests** (per project memory + ADR-0024). Hit a real Postgres, real KMS test instance, real Foundry sandbox.
- Test-fixture seed: per-tenant + per-region + per-class (PHI / PCI / PIPA-Art23 fixtures stored encrypted per ADR-0043 even in test).
- Per-test idempotency: tests run in any order, in parallel, on a fresh slate.
- Insta snapshots stored under `tests/snapshots/`; reviewer must explicitly approve via `cargo insta review`.

## 4. Test runner — `cargo nextest` (canonical)

Per project memory + ADR-0024. Never bare `cargo test`.

```sh
cargo nextest run --workspace --all-features --no-fail-fast
```

Test sharding via `--partition count:N/M`.

## 5. Companion local-dev tools

Per [TOOLCHAIN.md §3 + standards/code-style.md §1](code-style.md):

- **`bacon`** (Apache-2 / MIT) — background watcher; auto re-runs `check / clippy / nextest` on save. Engineer's primary feedback loop. Project ships curated `bacon.toml` jobs.
- **`cargo-machete`** (Apache-2 / MIT) — finds unused deps; per-PR + per-quarter sweep.
- **sccache** — compilation cache local + S3-remote; 60-90% incremental hit.

## 6. Flaky-test policy

- Test failing intermittently auto-emits `EVT-FLAKY-TEST-DETECTED`
- Foundry capability `flaky.test.classify` quarantines to `flaky/` lane
- Quarantined tests do not block PRs but DO block release candidates if not fixed in 14 days
- Owner team gets weekly digest of their flaky tests

## 7. Performance + benchmark policy

- Per-surface hot-path benchmarks tagged with `#[bench]`
- Bench gate: regression > 20% from baseline = block
- Baselines stored at `bench/baselines/<surface>.json`; Cosign-signed per ADR-0039
- Re-baseline only with explicit `ops-sre-reliability` + change-author approval

## 8. Coverage targets per axis

Per [QA-TEST-STRATEGY.md §7](../QA-TEST-STRATEGY.md):
- Foundation kernels: ≥ 95% line + branch
- Domain layer: ≥ 90%
- App layer: ≥ 85%
- Adapters: ≥ 80% (with integration test focus)
- API/worker: ≥ 80%
- Per-capability eval set: ≥ 20 cases minimum

## 9. Eval-as-test (Foundry-specific)

Per ADR-0024:
- Per-capability eval set runs in CI
- Per-PR replay against past traces
- Pass-threshold per capability
- Per-region linguistic eval

## 10. Sources
[QA-TEST-STRATEGY.md](../QA-TEST-STRATEGY.md), ADR-0024 (eval harness + replay), ADR-0039 (supply chain — bench baselines signed), ADR-0043 (secrets — test fixtures encrypted), `cargo nextest` docs, `bacon` docs, `cargo-machete` docs, project memory rust test runner.
