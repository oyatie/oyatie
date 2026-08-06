---
id: ADR-0540
title: "Cargo workspace to Buck2 target parity gate"
status: Rejected
planning_impact: false
deciders: founder
date: 2026-06-10
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
depends_on: [ADR-0083, ADR-0132, ADR-0363, ADR-0515, ADR-0538, ADR-0539]
amends: []
related: [ADR-0017, ADR-0083, ADR-0131, ADR-0132, ADR-0363, ADR-0515, ADR-0538, ADR-0539]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0540: Cargo workspace to Buck2 target parity gate

## Status

**Proposed - 2026-06-10 (authored for founder sign-off; door: one-way).**

## Context

FRIC-1781063357 and FRIC-008(b) describe a false-green class in which a Rust workspace member can
carry test code that Cargo sees but Buck2 never compiles or runs. ADR-0538 made workspace member
enumeration canonical through `libs/oya-workspace-members-kernel`; ADR-0539 made stale lock and
generated-face inputs first-diagnosis failures. The remaining gap is target parity: every root
workspace member must have a tracked `BUCK` file, and members with Rust tests must expose a
`rust_test` target so the canonical Buck lane can execute them.

The mechanical base measurement on `dev` for G011 found 817 workspace members:

- 0 members missing a tracked `BUCK` file.
- 634 members with Rust test code and no `rust_test` target.
- 74 members without Rust test code and no `rust_test` target, which are benign until tests are
  added.

The candidate adds the target-parity gate crate itself with a `rust_test` target, so the unwired
test debt remains the exact mechanically-derived 634-key baseline.

## Decision

Add `ci/facade/build-target-parity` as a pure cloud-ci gate.

NAME: oya-cloud-ci-target-parity-app
JUSTIFICATION:
- microservice = cloud-ci: the cloud-ci admission product owns gate execution per ADR-0515.
- bc-tokens = target-parity: the bounded concern is Cargo member to Buck target parity.
- layer = app: the crate is an executable CI gate surface with pure evaluator logic.
- exemptions claimed: none.

The accounting-registry producer emits a `target_parity` face with one row per tracked workspace
member:

```json
{
  "member_path": "libs/oya-example-domain",
  "has_buck": true,
  "has_rust_test_target": false,
  "has_test_code": true
}
```

The evaluator emits stable `Finding{code,key,remediation}` rows:

- `member_missing_buck`: the workspace member has no tracked `<member>/BUCK`. This code is
  frozen-empty and born-blocking.
- `member_test_code_without_rust_test_target`: the workspace member has Rust test code but no
  `rust_test(` target in `<member>/BUCK`. This code is `baseline-block-on-new`; the current 634
  mechanically-derived keys are frozen so new unwired tests fail.

Required remediation text includes:

```text
declare a rust_test target in <member>/BUCK (see any gates/* BUCK for the stanza shape) and ensure `buck2 test <target>` passes
```

The gate is registered in `oya-ci.toml`, `libs/oya-ci-config`, the oya-ci gate catalog, and the
`oya-ci-required` workflow matrix. Generated cloud-ci faces remain generator-owned and must be
materialized with:

```text
buck2 run //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin -- --repo-root .
```

## Consequences

### Concrete file and crate changes

| Path / Crate | Change type | BNF v4.1 name | Layer |
|---|---|---|---|
| `ci/facade/build-target-parity/` | create gate crate | `oya-cloud-ci-target-parity-app` | app |
| `ci/facade/artifact-inventory-registry/` | emit `target_parity` face and baseline keys | `oya-cloud-ci-accounting-registry-app` | app |
| `libs/oya-ci-config/` and `oya-ci.toml` | register gate face and disposition data | `oya-ci-config-kernel` | kernel |
| `.github/workflows/oya-ci-required.yml` | add one gate matrix line | - | - |
| `docs/oya-ci/gate-catalog.md` | document gate, input kind, key shape, and frozen-empty code | - | - |
| `tools/oya-buck-test-wiring-app/` | local bridge generator (retirement-marked CLI per ADR-0363); adds `rust_test` targets in batch | `oya-buck-test-wiring-app` | app |

The local bridge generator files owned by this ADR are:
`tools/oya-buck-test-wiring-app/BUCK`,
`tools/oya-buck-test-wiring-app/Cargo.toml`,
`tools/oya-buck-test-wiring-app/OWNERS`,
`tools/oya-buck-test-wiring-app/src/lib.rs`,
`tools/oya-buck-test-wiring-app/src/main.rs`,
`tools/oya-buck-test-wiring-app/fixtures/binary_only.input.txt`,
`tools/oya-buck-test-wiring-app/fixtures/library_append.expected.txt`,
`tools/oya-buck-test-wiring-app/fixtures/library_append.input.txt`,
`tools/oya-buck-test-wiring-app/fixtures/library_with_tests.generated.expected.txt`,
`tools/oya-buck-test-wiring-app/fixtures/library_with_tests.input.txt`.
The multispectrum evidence bundle for the initial wiring batch is
`evidence/multispectrum/g011-rust-test-wiring-generator-20260610-1781107105.json`.
The ownership files added with this ADR are
`evidence/multispectrum/OWNERS`.

### Integration via Workflow + Ontology

Not applicable. This ADR changes repository admission checks only; it does not emit Workflow
events, consume Workflow events, or write Ontology objects.

### Positive

- New Rust test code cannot silently remain outside the canonical Buck lane.
- Missing member `BUCK` files are born-blocking with a frozen-empty baseline.
- The existing 634-key unwired-test debt is mechanically frozen instead of hand-curated.
- The gate reuses ADR-0538 workspace-member resolution and ADR-0539 generated-face freshness.

### Negative

- Existing unwired-test debt remains visible until remediated member by member.
- The producer must read member `BUCK` files and tracked Rust sources to compute the face.
- Simple text detection of `rust_test(` is intentionally conservative; generated or macro-built
  target declarations must still materialize as visible BUCK stanzas.

### Operational

- Buck2 remains the binding local verification surface for the new gate and producer wiring.
- The firewall baseline is updated by running the approved materializer, not by hand-editing
  `*.generated.json`.
- Future remediation PRs shrink the 634-key baseline by adding `rust_test` targets and proving
  `buck2 test <target>` passes.

## Clean Architecture Impact

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` | Affected | App crate depends inward on config and workspace-member kernels only through producer wiring. |
| `cross-product-refusal` | Not affected | No product boundary is introduced. |
| `port-location` | Not affected | No new port traits. |
| `layer-correctness` | Affected | New gate declares the `app` layer in its BNF name. |
| `composition-root-only` | Not affected | No long-running composition root is introduced. |
| `sdk-kernel-only` | Not affected | No SDK kernel boundary change. |

## Alternatives Considered

**Alternative 1 - Let Cargo continue to be the only source of Rust test discovery**
- Description: rely on Cargo test discovery and leave Buck target coverage implicit.
- Pros: no new gate or baseline.
- Cons: preserves the exact false-green class from FRIC-1781063357.
- Reason rejected: canonical CI is moving to Buck; tests unseen by Buck are not merge evidence.

**Alternative 2 - Block all current missing `rust_test` targets immediately**
- Description: make `member_test_code_without_rust_test_target` frozen-empty on day one.
- Pros: fastest route to complete parity.
- Cons: would block unrelated work on 634 pre-existing members.
- Reason rejected: baseline-block-on-new preserves progress while preventing additional debt.

**Alternative 3 - Hand-curate the baseline**
- Description: create an approved list of currently unwired members by manual review.
- Pros: could encode exceptions.
- Cons: invites stale, subjective, and incomplete debt accounting.
- Reason rejected: the set must be mechanically derived from the producer face.

## Verification

- `buck2 build //ci/facade/build-target-parity:oya-cloud-ci-target-parity-app`
- `buck2 test //ci/facade/build-target-parity:oya-cloud-ci-target-parity-app-unittest`
- `buck2 test //ci/facade/build-target-parity:oya-cloud-ci-target-parity-app-gate`
- `buck2 test //ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-unittest`
- `buck2 test //ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin-unittest`
- `buck2 test //ci/facade/baseline-ratchet:oya-cloud-ci-firewall-app-gate`
- `buck2 test //ci/facade/baseline-ratchet:oya-cloud-ci-firewall-app-gate-registration`

## References

- FRIC-1781063357: target parity false-green class.
- FRIC-008(b): Rust test code can exist without a Buck `rust_test` target.
- ADR-0538: globbed root workspace membership and coverage gate.
- ADR-0539: cloud-ci freshness gate for lock and generated-face byte parity.
- ADR-0515: cloud-ci required status context as merge authority.
