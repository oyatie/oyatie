---
purpose: "Canonical testing standard for the oyatie workspace. Defines the Test Pyramid 2.0, mandates Buck2 test evidence, Buck2-native LLVM source-based coverage, and dual Cargo+Buck2 local mutation ergonomics."
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
  `buck2 test //... --show-output` (or the trusted cloud-ci/oya-ci Buck2 target inventory) as the evidence run,
  requires `proptest` / `quickcheck` for invariants, Buck2-native LLVM source-based coverage
  for coverage evidence, dual Cargo+Buck2 manifests for local mutation testing, deletion-tagged
  mutation/fuzz bridges to be invoked through Buck2 targets for CI authority, and codifies the
  14-day flaky-test SLA. Resolves
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
| Unit | `#[cfg(test)] mod tests` in same file | `buck2 test` | every PR |
| Integration | `tests/` directory of each crate | `buck2 test` | every PR |
| Contract | `tests/contract/` per consumer/provider; `contracts/` schemas | `buck2 test` + pact-style verifier target | every PR |
| E2E | `oya-foundry-e2e-*` runtime | Buck2 e2e test target w/ env tag | merge-queue + nightly |
| Property | `proptest` or `quickcheck` inside `tests/properties/` | `buck2 test` | every PR (short config); nightly (long config) |
| Fuzz | `fuzz/` per crate; Buck2-wrapped fuzz harness | Buck2 fuzz target (deletion-tagged bridge until native) | nightly + on diff to unsafe surfaces |
| Mutation | Cargo workspace manifests retained beside Buck2 targets | Local `cargo mutants`; CI/nightly Buck2 mutation target wrapper | local dev + nightly on kernel/domain |

Sources:
[Frontiers — Test Pyramid 2.0](https://www.frontiersin.org/journals/artificial-intelligence/articles/10.3389/frai.2025.1695965/full),
[Full Scale — Modern Test Pyramid Guide](https://fullscale.io/blog/modern-test-pyramid-guide/),
[Number Analytics — Fuzz Testing](https://www.numberanalytics.com/blog/mastering-fuzz-testing-in-property-testing).

## 2. The mandatory evidence run

Per [`docs/AGENTS.md`](../AGENTS.md) D9, every PR's `## Verification`
section MUST paste the output of:

```sh
buck2 test //... --show-output
```

Rules:

1. The cloud-ci/oya-ci controller snapshots trusted Buck2 test targets from trunk/controller state before candidate checkout.
2. The evidence run MUST execute the full selected Buck2 target inventory; affected-only subsets are feedback, never merge or Phase-0 exit authority.
3. Local fast feedback MAY use narrower Buck2 targets, but PR evidence names the exact Buck2 targets and Build ID.
4. Coverage evidence MUST come from a Buck2 target that builds/runs LLVM source-based coverage instrumentation (`rustc -C instrument-coverage` plus `llvm-profdata`/`llvm-cov` or a Buck2 rule wrapping that pipeline). Tarpaulin is not the canonical coverage surface for this monorepo and MUST NOT be added as required CI/PR evidence.
5. Mutation testing MAY run locally through Cargo (`cargo mutants` or `cargo nextest`-backed cargo-mutants) because the workspace intentionally keeps `Cargo.toml` / `Cargo.lock` beside Buck2 targets for developer ergonomics. Local Cargo mutation output is advisory until captured by a Buck2 target or trusted cloud-ci/oya-ci lane.
6. Any deletion-tagged bridge for nextest/fuzz/mutation MUST be invoked by a Buck2 target and carry a retirement path; raw Cargo commands are not CI/build/test authority.

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

### 5.2 Fuzz tests (Buck2-wrapped fuzz harness + libFuzzer)

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

## 6. Mutation testing — dual Cargo+Buck2 harness

Run nightly against `oya-*-kernel` and `oya-*-domain` crates. Target:
**≥ 80% caught mutants** on kernel/domain. Application-layer mutation
testing is optional (high cost, low signal).

Oyatie intentionally maintains a dual-build Rust setup:

- `Cargo.toml` / `Cargo.lock` stay present for local Rust-native workflows,
  including `cargo mutants`, `cargo nextest`, IDE metadata, and crate
  ecosystem compatibility.
- Buck2 `BUCK` targets remain the build/test/CI authority. Where crates.io
  dependencies enter Buck2, reindeer-style generation or an equivalent
  generated-BUCK path owns the third-party crate graph.
- Local Cargo mutation testing is encouraged for fast kernel/domain feedback,
  but PR merge evidence cites the Buck2 target or cloud-ci/oya-ci lane that
  captured the mutation run.

Findings categories:

- **Caught** (test killed the mutant): OK.
- **Missed** (mutant survives): file an issue; either add a test or
  document why the surviving mutation is semantically equivalent.
- **Unviable** (mutant doesn't compile): ignored.

Source-backed rationale:

- The Cargo workspace model supports common commands across all workspace
  members, shared `Cargo.lock`, and shared workspace metadata:
  <https://doc.rust-lang.org/cargo/reference/workspaces.html>.
- Buck2's own bootstrapping path can build with Cargo or Buck2 and uses
  reindeer to generate `BUCK` files for Rust crates from crates.io:
  <https://buck2.build/docs/about/bootstrapping/>.
- Reindeer describes itself as tooling for importing Rust crates and generating
  Buck build rules for monorepos: <https://github.com/facebookincubator/reindeer>.
- `cargo-mutants` is the local Rust mutation tool of record for this dual-build
  path; it runs against non-flaky tests through local Rust-native runner
  integrations, including nextest-backed local execution:
  <https://mutants.rs/>.

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

## 8. Coverage budget — Buck2-native LLVM source-based coverage

- Workspace target: **≥ 80% line coverage** on changed files (delta
  coverage), **≥ 70%** absolute on kernel/domain crates.
- Adapters, runtimes, and pure-CLI binaries: **≥ 50%** is acceptable.
- Generated code (`build.rs` outputs, proto stubs): excluded.
- Coverage is **advisory** at PR time and **enforced** at wave-gate
  reviews; a regression of >5 percentage points blocks the wave.

Coverage is generated natively through Buck2, not Tarpaulin:

1. Buck2 owns the coverage target inventory and the Build ID for the evidence
   run.
2. The coverage rule instruments Rust compilation with `-C instrument-coverage`.
3. Tests run under `LLVM_PROFILE_FILE` so each test shard emits `.profraw`
   profiles without clobbering parallel shards.
4. The rule merges profiles with `llvm-profdata` and emits text/HTML/JSON
   reports through `llvm-cov`.
5. The PR or wave-gate evidence records the Buck2 target, Build ID, report path,
   changed-file delta, and excluded generated paths.

The lane `oya-governance-coverage-delta` runs on every PR and emits
a comment with deltas; merge-blocking is reserved for the wave gate.

Source-backed rationale:

- `rustc` supports instrumentation-based coverage through
  `-C instrument-coverage` and records counters plus coverage maps for Rust
  libraries and binaries:
  <https://doc.rust-lang.org/rustc/instrument-coverage.html>.
- LLVM source-based coverage uses the `llvm-profdata` merge step and
  `llvm-cov` report/export step:
  <https://clang.llvm.org/docs/SourceBasedCodeCoverage.html>.
- Buck2 exposes target-based build/test/run commands; the repository coverage
  workflow MUST be expressed as Buck2 targets rather than ad-hoc local commands:
  <https://buck2.build/docs/users/commands/>.

## 9. Flaky-test 14-day SLA

Per [`forbidden-operations.json`](../../specs/forbidden-operations.json) FO-06:

1. A test MAY be `#[ignore = "flaky-NNNN"]` only after a `MISTAKES-LEDGER`
   row is filed (class: `mechanical` if there is a fix; `cultural`
   otherwise).
2. The fix SLA is **14 calendar days**.
3. Day 7 escalates to the owning team; day 14: the lane
   `oya-governance-flaky-sla` opens a
   blocking PR check on the owning crate.
4. Resolution requires either fix (delete `#[ignore]`) or retirement
   (delete the test with ADR-tracked rationale).

## 10. CI gates summary

Per [`docs/AGENTS.md`](../AGENTS.md) Done-Definition:

- **D9** — Buck2 test evidence passes (`buck2 test //... --show-output` or trusted cloud-ci/oya-ci target inventory).
- **D10** — Buck2 lint/static-analysis targets pass; raw Cargo lint commands are not CI authority.
- **D11** — Buck2-invoked dependency/license/advisory policy targets pass.
- **D13** — performance changes carry benchmark + ≥ 2 stress scenarios.

This standard adds:

- `oya-governance-test-evidence` — Buck2 test evidence pasted in PR.
- `oya-governance-fuzz-coverage` — fuzz harness mandatory where named.
- `oya-governance-mutation-budget` — nightly mutation budget on
  kernel/domain.
- `oya-governance-flaky-sla` — 14-day SLA enforcement.

## 11. Anti-patterns

1. **Disabling a test with `#[ignore]` without a `MISTAKES-LEDGER` row.**
2. **Running raw Cargo test/lint/check commands for CI evidence instead of Buck2 targets.**
3. **Adding Tarpaulin as the monorepo coverage authority** instead of Buck2-native LLVM source-based coverage.
4. **Treating local Cargo mutation testing as merge authority** instead of local advisory feedback or Buck2/cloud-ci captured evidence.
5. **Property test with low case-count to "save CI time"** — use the
   nightly long config instead, not a degraded PR config.
6. **Adding `unsafe` without a fuzz harness.**
7. **Integration test hitting a real provider** — use the
   `ProviderAdapter` trait + a fake.

## 12. Sources scanned

- [`.omc/scratch/hyperscaler-best-practices-2026-05-12.md`](../../.omc/scratch/hyperscaler-best-practices-2026-05-12.md)
  Domain 2 "Testing" + Domain 3 (nextest/fuzz/Kani surfaced through Buck2 targets).
- [Frontiers — Test Pyramid 2.0](https://www.frontiersin.org/journals/artificial-intelligence/articles/10.3389/frai.2025.1695965/full).
- [nextest book](https://nexte.st/).
- [proptest](https://docs.rs/proptest), [quickcheck](https://docs.rs/quickcheck).
- [cargo-mutants](https://mutants.rs/), [cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz).
- [rustc instrumentation-based coverage](https://doc.rust-lang.org/rustc/instrument-coverage.html).
- [LLVM source-based coverage](https://clang.llvm.org/docs/SourceBasedCodeCoverage.html).
- [Buck2 commands](https://buck2.build/docs/users/commands/) and
  [Buck2 bootstrapping / reindeer](https://buck2.build/docs/about/bootstrapping/).
- ADR-0003 (audit chain), ADR-0015 (flat crates).
