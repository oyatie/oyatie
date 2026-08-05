---
purpose: "Canonical testing standard for the oyatie workspace. Defines the Test Pyramid 2.0 (unit / integration / contract / e2e / property / fuzz), mandates `cargo nextest run --workspace --all-features --no-fail-fast` as the evidence run."
doc_status: published
---

---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Canonical testing standard for the oyatie workspace. Defines the Test Pyramid 2.0
  (unit / integration / contract / e2e / property / fuzz), mandates
  `cargo nextest run --workspace --all-features --no-fail-fast` as the evidence run,
  requires `proptest` / `quickcheck` for invariants, `cargo-mutants` for mutation
  testing on kernel/domain code, `cargo-fuzz` for unsafe and FFI surfaces, sets the
  `cargo-llvm-cov` coverage budget, and codifies the 14-day flaky-test SLA. Resolves
  the `standards/testing.md` forward-reference sentinel in
  `docs/AGENTS.md` §During-change discipline.
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
planned_enforcement_ref: oya-governance-test-evidence
companion_docs:
  - docs/QA-TEST-STRATEGY.md
  - docs/standards/code-style-rust.md
  - docs/standards/error-handling.md
  - docs/standards/observability.md
related_adrs:
  - ADR-0053
  - ADR-0052
  - ADR-0054
---

# Testing

## Doctrinal authority — [decision-principles.json](../../specs/decision-principles.json) + [forbidden-operations.json](../../specs/forbidden-operations.json)

Tests are the executable evidence that the system meets its contracts. This
standard governs how to write, organize, and run them across `oya-*` crates.
[`docs/QA-TEST-STRATEGY.md`](../QA-TEST-STRATEGY.md) sets per-axis test
strategy; this standard sets the cross-axis floor.

## 1. The Test Pyramid 2.0

The hyperscaler consensus in 2025–2026 is an expanded pyramid:

```
                ┌──────────────────┐
                │       E2E        │   1–5%   user-journey scenarios
                ├──────────────────┤
                │     Contract     │   5–10%  consumer/provider contract tests
                ├──────────────────┤
                │   Integration    │  15–25%  cross-module + adapter wiring
                ├──────────────────┤
                │      Unit        │  60–70%  pure functions, kernel + domain
                ├──────────────────┤
                │ Property + Fuzz  │   5–10%  invariants + boundary probing
                └──────────────────┘
```

| Tier | Where it lives | Runner | Frequency |
|---|---|---|---|
| Unit | `#[cfg(test)] mod tests` in same file | `cargo nextest` | every PR |
| Integration | `tests/` directory of each crate | `cargo nextest` | every PR |
| Contract | `tests/contract/` per consumer/provider; `contracts/` schemas | `cargo nextest` + pact-style verifier | every PR |
| E2E | `oya-intelligence-e2e-*` runtime | `cargo nextest` w/ env tag | merge-queue + nightly |
| Property | `proptest` or `quickcheck` inside `tests/properties/` | `cargo nextest` | every PR (short config); nightly (long config) |
| Fuzz | `fuzz/` per crate; `cargo-fuzz` | `cargo fuzz run` | nightly + on diff to unsafe surfaces |
| Mutation | n/a | `cargo-mutants` | nightly on kernel/domain |

Sources:
[Frontiers — Test Pyramid 2.0](https://www.frontiersin.org/journals/artificial-intelligence/articles/10.3389/frai.2025.1695965/full),
[Full Scale — Modern Test Pyramid Guide](https://fullscale.io/blog/modern-test-pyramid-guide/),
[Number Analytics — Fuzz Testing](https://www.numberanalytics.com/blog/mastering-fuzz-testing-in-property-testing).

## 2. The mandatory evidence run

Per [`docs/AGENTS.md`](../AGENTS.md) D9, every PR's `## Verification`
section MUST paste the output of:

```sh
cargo nextest run --workspace --all-features --no-fail-fast
```

Rules:

1. `--no-fail-fast` is **mandatory** so the full failure surface is
   captured in one run (hyperscaler convention; avoids whack-a-mole).
2. `--all-features` ensures conditional code paths compile and are exercised.
3. Local dev loop SHOULD use `bacon nextest` for fast feedback; the
   evidence run is the canonical artifact (per AGENTS.md §During-change
   discipline).
4. The run MUST emit JUnit + JSON to `target/nextest/` for CI archival.

`cargo-nextest` is the workspace standard test runner — parallel by default,
fault-isolated per test, retries supported, slow-test detection. Source:
[nextest book](https://nexte.st/).

## 3. Unit tests

- Co-located in the source file under `#[cfg(test)] mod tests { ... }`.
- Function names: `<fn_under_test>_<scenario>_<expected>` (e.g.,
  `parse_capability_id_empty_returns_err`).
- No I/O, no global state, no `tokio::runtime::Runtime::new()` per test
  (prefer `#[tokio::test]`).
- Use `pretty_assertions` for diff-friendly `assert_eq!` output.

## 4. Integration tests

- One file per surface under `tests/`. The file is its own crate; pull
  shared fixtures from a `tests/common/mod.rs`.
- Use `testcontainers` or in-process fakes for DB / Redis / queue surfaces.
  Provider SDKs are mocked via the `ProviderAdapter` trait per
  [`dependency-policy.md`](dependency-policy.md) §5.
- Tag flaky-prone tests with `#[ignore = "flaky-NNNN; see MISTAKES-LEDGER"]`
  ONLY after filing the ledger row (§9).

## 5. Property + fuzz testing

Property tests probe **what should hold**; fuzz tests probe **where it
breaks**. Both are mandatory on specific surfaces.

### 5.1 Property tests

- Library: [`proptest`](https://docs.rs/proptest) for new code;
  `quickcheck` accepted on legacy crates.
- Targets: every parser, serializer, codec, state-machine transition, and
  cross-pillar invariant in kernel/domain layers.
- Config: short generators for PR-time (≤ 256 cases, 30 s budget); long
  generators nightly (≤ 32k cases, 30 min budget).
- Counterexamples MUST be checked in to `tests/properties/regressions.txt`
  and converted into a unit test the next PR.

### 5.2 Fuzz tests (`cargo-fuzz` + libFuzzer)

Mandatory on:

- Every `unsafe` block (per [`code-style-rust.md`](code-style-rust.md) §4).
- Every FFI boundary.
- Every public parser/deserializer accepting untrusted input (HTTP, gRPC,
  schema-validated events, OpenAPI bodies).
- The audit-chain shard parser (per ADR-0003 / DOC-CATALOG lane
  `audit-chain-replay`).

Lane: `oya-governance-fuzz-coverage` refuses PRs that add `unsafe`
without a `fuzz_targets/<symbol>.rs` harness, and refuses parser changes
without a regression case.

Nightly job emits any new crash to `MISTAKES-LEDGER.md` and opens an issue
with the minimized reproducer.

Source: [AWS — How Kani is used](https://aws.amazon.com/blogs/opensource/how-open-source-projects-are-using-kani-to-write-better-software-in-rust/).

## 6. Mutation testing — `cargo-mutants`

Run nightly against `oya-*-kernel` and `oya-*-domain` crates. Target:
**≥ 80% caught mutants** on kernel/domain. Application-layer mutation
testing is optional (high cost, low signal).

Findings categories:

- **Caught** (test killed the mutant): OK.
- **Missed** (mutant survives): file an issue; either add a test or
  document why the surviving mutation is semantically equivalent.
- **Unviable** (mutant doesn't compile): ignored.

## 7. Contract tests

For every cross-axis contract under `contracts/`, the provider crate AND
each consumer crate run a contract test that:

1. Loads the same canonical schema (`contracts/openapi/*.yaml`,
   `contracts/proto/*.proto`, `contracts/asyncapi/*.yaml`).
2. Exercises the surface against either a real fixture or a generated mock.
3. Asserts the schema-vs-types parity (per `spec-contract-mirror` lane in
   [DOC-CATALOG.md](../DOC-CATALOG.md) §4).

Adding a consumer without a contract test = `oya-governance-contract-coverage`
fail.

## 8. Coverage budget — `cargo-llvm-cov`

- Workspace target: **≥ 80% line coverage** on changed files (delta
  coverage), **≥ 70%** absolute on kernel/domain crates.
- Adapters, runtimes, and pure-CLI binaries: **≥ 50%** is acceptable.
- Generated code (`build.rs` outputs, proto stubs): excluded.
- Coverage is **advisory** at PR time and **enforced** at wave-gate
  reviews; a regression of >5 percentage points blocks the wave.

The lane `oya-governance-coverage-delta` runs on every PR and emits
a comment with deltas; merge-blocking is reserved for the wave gate.

## 9. Flaky-test 14-day SLA

Per [`forbidden-operations.json`](../../specs/forbidden-operations.json) FO-06:

1. A test MAY be `#[ignore = "flaky-NNNN"]` only after a `MISTAKES-LEDGER`
   row is filed (class: `mechanical` if there is a fix; `cultural`
   otherwise).
2. The fix SLA is **14 calendar days**.
   escalates; day 14: the lane `oya-governance-flaky-sla` opens a
   blocking PR check on the owning crate.
4. Resolution requires either fix (delete `#[ignore]`) or retirement
   (delete the test with ADR-tracked rationale).

## 10. CI gates summary

Per [`docs/AGENTS.md`](../AGENTS.md) Done-Definition:

- **D9** — `cargo nextest run --workspace --all-features --no-fail-fast`
  passes.
- **D10** — `cargo clippy --workspace --all-features --all-targets --
  -D warnings` passes.
- **D11** — `cargo deny check` passes.
- **D13** — performance changes carry benchmark + ≥ 2 stress scenarios.

This standard adds:

- `oya-governance-test-evidence` — nextest evidence pasted in PR.
- `oya-governance-fuzz-coverage` — fuzz harness mandatory where named.
- `oya-governance-mutation-budget` — nightly mutation budget on
  kernel/domain.
- `oya-governance-flaky-sla` — 14-day SLA enforcement.

## 11. Anti-patterns

1. **Disabling a test with `#[ignore]` without a `MISTAKES-LEDGER` row.**
2. **Running `cargo test` instead of `cargo nextest` for evidence.**
3. **Property test with low case-count to "save CI time"** — use the
   nightly long config instead, not a degraded PR config.
4. **Adding `unsafe` without a fuzz harness.**
5. **Integration test hitting a real provider** — use the
   `ProviderAdapter` trait + a fake.

## 12. Sources scanned

- [`hyperscaler-best-practices.md`](hyperscaler-best-practices.md)
  Domain 2 "Testing" + Domain 3 (cargo-nextest, cargo-fuzz, Kani).
- [Frontiers — Test Pyramid 2.0](https://www.frontiersin.org/journals/artificial-intelligence/articles/10.3389/frai.2025.1695965/full).
- [nextest book](https://nexte.st/).
- [proptest](https://docs.rs/proptest), [quickcheck](https://docs.rs/quickcheck).
- [cargo-mutants](https://mutants.rs/), [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz).
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov).
- ADR-0003 (audit chain), ADR-0015 (flat crates).
