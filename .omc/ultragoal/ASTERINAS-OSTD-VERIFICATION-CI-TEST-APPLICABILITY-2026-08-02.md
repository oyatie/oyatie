# Asterinas / OSTD verification, CI, and test applicability — 2026-08-02

## Findings

1. **OSTD main does not currently integrate Verus.** Its Cargo metadata and repository tree expose no Verus/Z3/SMT proof lane. OSTD's stated assurance mechanism is architectural: encapsulate machine-specific unsafe Rust behind machine-independent safe APIs designed to be sound for safe callers. That is valuable, but it is not machine-checked formal verification.
2. **The applicable Asterinas pattern is a test ladder.** Fast user-mode tests (`make test`), kernel-linked `#[ktest]` tests, boot/regression matrices, architecture variants, and external conformance suites (LTP, gVisor, kselftest, xfstests) cover different failure classes. Representative page-table tests use independent `page_walk` observations, finite exhaustive flag combinations, alignment/overflow/wrap boundaries, expected-panic contracts, huge-page splitting, and neighbor-preservation assertions.
3. **The kernel-test harness is reusable as a method, not code authority.** `#[ktest]` registers metadata in a linker section and returns typed panic/test outcomes. Oyatie already has an Asterinas real-boot harness; adoption should add only missing layers measured against current coverage, not clone a second test framework.
4. **Do not copy Asterinas CI mechanics wholesale.** Current workflows use privileged containers, mutable major action tags and dated image tags rather than immutable digests, command construction through `eval`, incomplete job timeouts, host capability assumptions, and no systematic failure-artifact upload. These conflict with Oyatie's hermetic Buck2/owned-Rust/provenance bars.

## Verus disposition

- Applicable pure kernels: partial-negative selector decision table, receipt predicates, `build_health_verdict` set algebra, baseline-ratchet compare/firewall, and scm-facts retirement transitions.
- Not provable by those kernels: GitHub/API/log authenticity, emitter provenance, clocks/expiry, clean immutable-base construction, candidate exclusion at runtime, Buck2 execution truth, runner loss/OOM/eviction, fan-in wiring, deployment, or user-observed result.
- Current terminal: `BLOCKED_NO_HERMETIC_BUCK2_VERIFIER_TOOLCHAIN`.
- Minimum preparation slice: one coherent Verus/vstd/Rust/Z3 pin, Linux amd64+arm64 provenance, Buck2 rules with network disabled and declared inputs/outputs, proof-result artifact carrying source/toolchain digests and obligations, and a refinement test against the executable Rust kernel. No workflow authority until cold/warm reproducibility and independent review pass.

## Immediate non-ceremonial improvements

- Preserve user-mode → kernel → boot → conformance layering, but derive coverage from the Buck2 graph rather than add hand-maintained matrices.
- For unsafe kernel boundaries, pair documented safety contracts with independent observable oracles and edge-domain enumeration; do not call passing tests a proof.
- Add formal proof only where a pure invariant is stable and load-bearing; keep runtime provenance as executable validation and observation.
- Treat Stage A's mixed whole-job-log parser as non-authorizing. Stage B is blocked until a separately authenticated structured receipt is produced by immutable base-built code from Buck2-owned evidence; syntax, contiguity, and API job provenance do not authenticate the emitter.

## Official sources inspected

- https://github.com/asterinas/asterinas/blob/main/ostd/README.md
- https://github.com/asterinas/asterinas/blob/main/.github/workflows/test_x86.yml
- https://github.com/asterinas/asterinas/blob/main/.github/actions/test/action.yml
- https://github.com/asterinas/asterinas/blob/main/ostd/libs/ostd-test/src/lib.rs
- https://github.com/asterinas/asterinas/blob/main/ostd/src/mm/page_table/test.rs
- https://github.com/asterinas/asterinas/blob/main/kernel/libs/ring-buffer/src/test.rs
- https://github.com/asterinas/asterinas/blob/main/kernel/libs/comp-sys/cargo-component/tests/violate_policy.rs
