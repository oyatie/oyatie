---
id: ADR-0633
title: "Enforcement belongs to the layer that OWNS the fact: T1 mutation coupling, T2 non-emptiability, a promotion gate that keeps false-positive checks out of gates, and a naming rule with a rename test"
status: Proposed
doc_status: published
planning_impact: true
deciders: founder
date: 2026-08-02
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
depends_on: [ADR-0562, ADR-0280]
amends: []
related: [ADR-0631, ADR-0615, ADR-0512, ADR-0132, ADR-0515, ADR-0523]
milestone: W3
---

# ADR-0633: enforcement belongs to the layer that owns the fact

## Status

**Proposed — 2026-08-02.** Landed Proposed, not Accepted: a fresh `Accepted` status REDs
cross-artifact-agreement until its evidence propagates.

Generalizes ADR-0631 D10 (crate PLACEMENT must be a build constraint) into a layer-assignment rule
for every enforced fact. **ADR-0631 D7–D10 are UNMERGED**, on branch
`docs/adr-0631-policy-plane-and-burndown` — cited here as proposed, not landed.

## Context

Every measurement below was re-taken in this lane at `8bda90b2e` (`origin/dev`). Where it differs
from the figure in the commissioning brief, the measured value is used and the delta noted.

| fact | measured | probe | brief said |
|---|---|---|---|
| `BUCK` files | **906** | `find . -name BUCK` | 906 ✓ |
| `visibility = ["PUBLIC"]` declarations | **2229** | `grep -rho` over `--include=BUCK`, 3 pattern shapes agree | 2223 (+6 delta, cause NOT established) |
| `BUCK` files with any `visibility` | **901** of 906 | `grep -rc visibility` | — |
| `within_view` occurrences repo-wide | **0** | 2 probes + known-positive control | 0 ✓ |
| gate crates under `ci/` | **55** (137,695 LOC Rust) | `find ci -name Cargo.toml` | part of 165 |
| gate crates under `governance/check/` | **56** | `ls governance/check` | part of 165 |
| `ci/` gate crates naming `BUCK` in `.rs` | **17** | `grep -rl BUCK ci --include='*.rs'` | 16 |
| of those, crates that spawn `buck2` | **4** | `grep Command::new \| grep -i buck` | 4 ✓ |
| of those, crates using buck2 as a QUERY engine | **1** | subcommand inspection | 4 |
| `rust_test` targets under `//ci/...` | **118** | static BUCK census | — |
| `rust_binary` targets under `//ci/...` | **30** | static BUCK census | — |
| `registry/catalog/` rows | **748** | `ls registry/catalog` | — |

Three of these deserve their correction stated rather than buried.

**The `within_view` zero is real, and was confirmed the way a total is supposed to be confirmed.**
A repo-wide zero is exactly the shape that means "the probe is broken". Two differently-scoped greps
(BUCK-only; every file, every extension) both returned 0, and a known-positive control — a scratch
`BUCK` containing `within_view = ["//libs/..."]` — returned 1 through the same probe. The probe can
see the token. The token is not there. **Buck2's downward constraint is unused in this repository.**

**"4 gates use buck2 as a query engine" is generous by 3x.** Of the four crates that spawn `buck2`,
only `affected-target-set` issues a query (`uquery`). `crate-registration` and
`generated-artifact-freshness` call `buck2 build`; `scm-facts-snapshot` calls `buck2 run` to
re-execute itself in a historical worktree. **Exactly one gate in the fleet asks buck2 what the
graph contains.** The other sixteen that reason about `BUCK` re-derive the graph by parsing text.

**The static BUCK census is trustworthy, and this is checkable.** Its count of `rust_test` targets
under `//ci/...` is 118 — identical to the authoritative `buck2 uquery "kind('.*_test', //ci/...)"`
enumeration recorded in `.github/workflows/oya-ci-required.yml:230`. The counting method agrees with
buck2 on the one population where both numbers exist.

### The claim "111 gate crates have no CI caller" does not survive re-measurement, and what replaces it is worse

`oya-ci-required.yml:568` runs `buck2 test //ci/...` — a wildcard. All 55 `ci/` gate crates are
reached. Reporting them as uncalled would have been wrong.

What is actually true is more specific:

1. **`governance/check/**` — 56 crates, 57 `rust_test` targets — is named by no CI lane.** The only
   occurrence of the string `governance/check` anywhere in `.github/` or the `Makefile` is a
   *comment* at `oya-ci-required.yml:670`. Confirmed by a second probe across all workflow files.
2. Those targets are nonetheless *executed*, by `buck2 test //...` at line 848 — but that invocation
   is a **merge-base baseline whose exit code is explicitly discarded** (`|| true`), feeding a
   per-target health ratchet.
3. **A ratchet blocks only on a NEW break relative to merge-base.** A gate that passes *vacuously* —
   because its scan matched nothing — contributes a green verdict and is indistinguishable from a
   gate that passed because the tree is clean.

That third point is the whole problem. **The fleet's dominant failure is not the unwired gate; it is
the wired gate whose probe went empty.** `aspirational-enforcement` (which lives at
`governance/check/aspirational-enforcement`, outside `//ci/...`) went to zero observed sites after a
`check-*` rename and reported clean. No dispatch audit detects that, because dispatch was never the
thing that broke.

### Why "the earliest layer that can express it" is not a rule

It is directionally right and mechanically useless: *expressible* has no test. Any fact can be
"expressed" at any layer by writing a scanner for it — which is precisely how a repository acquires
165 gate crates. A usable rule has to be decidable by running something.

## Decision

### D1 — a check belongs to the layer that OWNS the fact, and ownership is exactly two tests

Assign each enforced fact to the **lowest-numbered layer that passes BOTH T1 and T2**. Not the
earliest that *can express* it — the lowest that *owns* it.

```
T1  MUTATION COUPLING
    Changing the fact requires editing an artifact the enforcer reads.

T2  NON-EMPTIABILITY
    No refactor that PRESERVES the fact can shrink the enforcer's input set.
```

#### T1 — acceptance test (runnable, per check C enforcing fact F)

```
GIVEN  check C and the fact F it enforces
STEP 1 Write a minimal tree edit E that makes F false.
       (For a gate, E is the fixture you would add to prove RED.)
STEP 2 Enumerate C's declared input set:
         buck2 uquery --output-attribute srcs '//<path>:<target>'
       plus every path C opens at runtime that is not in srcs (policy JSON, scan roots).
STEP 3 PASS iff E touches at least one path in STEP 2's set.

FAIL  ⇒ F can change without any input of C changing. C is a PROXY detector:
        it infers F from a correlate. Reassign, or make the correlate an input.
```

#### T2 — acceptance test (runnable, per check C)

```
GIVEN  check C, and its self-reported population counter N (see D3)
STEP 1 Record N on the unmodified tree.
STEP 2 Apply a FACT-PRESERVING refactor R. R must not repair a single violation.
       The three that matter here, applied in order:
         R1  relocate a governed directory one level deeper (e.g. <cap>/ -> app/<cap>/)
         R2  rename a governed directory or crate-name prefix
         R3  move one violating package to a sibling root
STEP 3 Re-run C. Record N'.
STEP 4 PASS iff  N' == N  OR  C is RED.

FAIL  ⇒ R emptied the probe while every violating instance survived.
        This is a FALSE GREEN, not a coverage gap.
```

**T1 alone passes broken things, and the example is in-tree.**
`ci/facade/facade-core-layering` enforces ADR-0562's rule that a `facade` crate reaches its own
capability's `core` only through `ports`.

- **T1: PASS.** A `facade -> core` edge lives in a `BUCK` file, and the gate reads
  `<cap>/facade/<pkg>/BUCK` (`src/lib.rs:183-186`). Adding the edge necessarily edits an input.
- **T2: FAIL.** The scan enumerates capabilities with `dir_names(repo_root)` — **immediate
  subdirectories only** (`src/lib.rs:172-176`). Apply R1: relocate a capability to `app/<cap>/` and
  it is no longer an immediate subdirectory. `capabilities_scanned` drops by one, the gate stays
  green, and every violating `facade -> core` edge inside it survives untouched.

This is not hypothetical deferred to never. **ADR-0562 mandates `app/<product>/` for 2+-capability
tenant compositions; `app/` does not exist yet** (verified — absent at `8bda90b2e`), so the break is
*scheduled*, not speculative. The same defect class already has two realized instances: the
`aspirational-enforcement` emptying above, and both authz gates lacking `app/` in their scan roots.

**A build constraint passes both structurally, and the reason is worth stating precisely: the
constraint lives in the same file as the thing it constrains.** A `visibility` attribute sits in the
same `BUCK` file as the target it governs. Any move that carries the edge carries the rule, because
they are the same bytes. T1 holds because editing the dep *is* editing the constraint's file. T2
holds vacuously — **there is no probe to get wrong, because there is no probe.** The enforcer is
buck2's own loading phase, and its "input set" is the build graph, which cannot be shrunk by a
refactor that preserves the graph.

#### The layer table

Lower number = owns more. Assign to the lowest layer passing T1 ∧ T2.

| # | layer | mechanism | fails at | has a probe? |
|---|---|---|---|---|
| L0 | **build constraint** | buck2 `visibility`, `within_view` | build-file evaluation | **no** |
| L1 | **type system** | Rust types, sealed traits, newtypes | compile | **no** |
| L2 | **schema at load** | `serde(deny_unknown_fields)`, required fields, `ok_or` on a policy key | process start / first parse | no |
| L3 | **gate** | owned-Rust check in CI | gate run | **yes — T2 applies** |
| L4 | **review / doc** | ADR prose, checklist | never mechanically | n/a |

L0–L2 have no probe to empty, so T2 is satisfied by construction and only T1 must be checked.
**L3 is the only layer where T2 can fail, which is why T2 exists.** L4 enforces nothing and is a
valid destination only for facts no lower layer owns *and* which fail the D2 promotion gate.

### D2 — the promotion gate: ownership is the destination, false-positive rate is the ticket

A check that passes T1 ∧ T2 for layer L is *assigned* to L. Whether it may **block** is a separate
question, decided by the SWE@Google ch20 criteria:

| criterion | test |
|---|---|
| **actionable** | the failure message names a mechanical fix the author can apply without judgement |
| **no effective false positives** | see the acceptance test below |
| **correctness, not style** | a violation is wrong, not merely unlike the house preference |

#### Acceptance test for "no effective false positives" (runnable, per check C)

```
GIVEN  check C, proposed for blocking status
STEP 1 Run C over the last 100 merged commits (or all history if fewer).
STEP 2 For each firing, classify: TRUE (author would change the code)
                              or  FALSE (author would suppress/baseline/waive).
       Classification is by the OWNING team, not by C's author.
STEP 3 PASS iff FALSE / (TRUE + FALSE) == 0 over the sample.

Any nonzero false-positive rate FAILS. "Low" does not pass — an effective false
positive is one the author must work around, and one per week is enough to teach
the fleet that the gate is noise.
```

Two consequences, both deliberate:

- **A check that OWNS its fact but has false positives stays ADVISORY IN ITS DESTINATION LAYER.**
  It does not get promoted to a gate to buy enforcement, and it does not get demoted to L4 prose.
  It sits at L, reporting, until the false positives are fixed. Advisory-at-L0/L1 means the
  constraint is written but the violation is baselined, not that the constraint moves.
- **A check with no owner below CI is a CI gate permanently. That is a legitimate answer.** Facts
  about the repository *as a whole* — "every ADR referenced by a spec exists", "the generated face
  matches its source" — have no build-file, no type, and no schema that owns them. Ruling them
  "should have been a build constraint" is how a doctrine becomes unimplementable. They stay at L3,
  and they carry T2's obligations (D3) forever.

### D3 — every L3 check reports its scanned population, and zero is RED

T2's acceptance test needs a number to compare. A check that cannot say how much it looked at cannot
be tested for emptiability, so:

```
GIVEN  any check C at L3
RULE   C emits a population counter N in its machine-readable output.
RULE   N == 0 is RED, never "clean".

ACCEPTANCE TEST (RED/GREEN, both directions required):
  RED   Point C at an empty directory. Assert C exits non-zero AND the failure
        text names the empty population — not a generic pass.
  GREEN Point C at the real tree. Assert N > 0 and equals an independently
        derived count (a second probe of different shape).
  A test that only demonstrates GREEN proves nothing: the vacuous pass IS the bug.
```

In-tree precedent, already correct: `facade-core-layering` reports `capabilities_scanned` and
`facade_packages_scanned`, and its own source states the reason — *"a scan that enumerates nothing
is distinguishable from a tree that is clean — the two are otherwise identical at the finding level,
and the first is a broken gate"* (`src/lib.rs:143-145`). That gate already emits the counter it needs
to fail T2 loudly; **it does not yet treat a drop in that counter as a failure.** D3 closes that,
and it is the cheapest repair in this ADR.

### D4 — naming: a name may be a key IFF the tool that owns it fails loudly at rename

A name is a lossy encoding of a decision nobody recorded. It may nonetheless serve as a key when
renaming it breaks something immediately and visibly.

```
MECHANICAL TEST (per name N used as a key by tool T)
STEP 1 Rename N to N' everywhere N is DEFINED. Do not update any consumer.
STEP 2 Run T.
STEP 3 N is a legitimate key iff T fails, AND the failure text names N or N'.

If nothing goes red, N was never a key — T was reading a correlate, and the
rename silently changed T's answer. That is a T2 failure sourced in a name.
```

#### The enumeration: where a name legitimately IS the contract

| name kind | owning tool | failure on rename | loud? |
|---|---|---|---|
| **buck2 target label** | buck2 loading | dependent's `deps` string does not resolve: `Unknown target` | **YES** |
| **buck2 cell name** | buck2 cell resolution | `unknown cell` at config load | **YES** |
| **Rust path / crate name** | rustc | `E0432`/`E0433` unresolved import | **YES** |
| **policy JSON key read via `ok_or`** | the gate's own loader | `policy: missing 'faces.facade'` | **YES** — in-tree, `facade-core-layering/src/lib.rs:148-166` |
| **wire field name (serde)** | serde, at runtime | field silently absent → `Default`, unless `deny_unknown_fields` + no `Option`/`default` | **NO by default** |
| **crate-name PREFIX used as a scan key** | nothing | scan matches zero, reports clean | **NO** — in-tree, `aspirational-enforcement` after the `check-*` rename |
| **path glob in a gate policy** | nothing | glob matches zero, reports clean | **NO** — the D1/T2 mechanism |
| **CLI verb** | n/a | CLI surfaces are retirement-marked; no new name may be keyed here | **n/a** |

Rows 1–4 pass the rename test: the name IS the contract, and may be relied on. Rows 5–7 fail it and
**may not be used as a key**. Two remedies, per row kind:

- **Wire field names** become legitimate keys by adding `#[serde(deny_unknown_fields)]` and a
  round-trip golden test. Acceptance test: rename a field in the struct, leave the golden fixture
  unchanged, assert the test fails naming the field. Without that test the rename is silent, and a
  silent wire rename is a data-loss bug rather than a lint.
- **Prefixes and globs** are replaced by an identity that survives renames. This repo already has
  one: the `registry/catalog/<crate_id>.yaml` row (748 rows at `8bda90b2e`) carries the owning
  `capability:` facet. R6c and the `aspirational-enforcement` repair both re-keyed onto that facet
  **precisely so a rename could not change the answer** — this decision generalizes that from two
  incident fixes into the rule.

### D5 — generalizing ADR-0631 D10: placement is not a special case

ADR-0631 D10 (proposed, unmerged) rules that crate PLACEMENT must be a build constraint rather than
an inferred-from-names gate. **D1 subsumes it, and placement is not special.**

Placement's fact is "which capability owns this crate". Run the tests: a name-keyed placement gate
passes T1 (moving a crate edits paths the gate reads) and fails T2 (R1/R2 relocate or rename the
governed root and the scan empties). A `visibility` declaration passes both — the constraint is in
the moved file. So D1 assigns placement to L0, which is exactly D10's ruling, **derived rather than
asserted**.

The generalization: every fact D10 reasons about — layering direction, face discipline, dependency
rank — has the same shape and lands at L0 for the same reason. ADR-0631 D10 stands; it is the first
instance of D1, not an independent rule. **Where the two are read together, D1 governs**, because
D10's justification ("a gate can be emptied, a build constraint cannot") *is* T2.

One thing D1 adds that D10 does not have: **D10 does not say what to do with the facts that cannot
descend.** D2 answers that — they stay at L3 permanently and carry D3's population counter.

### D6 — first migration step: tighten `audit`, in one PR

`visibility` defaults are permissive, so tightening is incremental by construction. This is the
step; a sibling lane executes it. **This ADR does not implement it.**

**Why `audit`:** S0, 18 `BUCK` files, 24 targets, all currently `visibility = ["PUBLIC"]`. It has
**no `facade/` face at all**, so `facade-core-layering` does not scan it and the two changes cannot
interact. `secrets` was rejected as the candidate: `secrets-kms-operator-app` already sits in
`facade-core-layering`'s frozen violation baseline, so its fallout would be entangled.

**Exactly which targets get which visibility:**

| face | packages | declarations | new `visibility` |
|---|---|---|---|
| `audit/core/*` — the 6 with no external dependents: `emission-domain`, `query-domain`, `retention-cascade-domain`, `sealing-domain`, `verification-domain`, `usecase` | 6 | **8** | `["//audit/..."]` |
| `audit/core/chain-domain` | 1 | 4 | **stays `["PUBLIC"]`** — 6 external dependents |
| `audit/adapters/file` | 1 | 2 | **stays `["PUBLIC"]`** — 2 external dependents |
| `audit/ports/*` — the 10 `*-api` / `*-kernel` packages | 10 | 10 | `["PUBLIC"]` (unchanged; `ports` is the only face with cross-capability visibility) |

**Only 8 of `audit`'s 24 declarations move.** That is the honest size of the step, and the number
was arrived at by measuring rather than by assuming the "clean" capability was clean.

**Predicted surface — the whole point of picking a clean capability is that this is checkable
before the PR is written:**

- Tightening the 6 clean `core` packages surfaces **zero** violations. Verified by two
  differently-shaped probes: a path probe (`grep '//audit/core' --include=BUCK`, excluding
  `audit/`) returns external hits for `chain-domain` only; a name probe for the six target names
  across all 906 `BUCK` files returns **only their own defining files**.
- `audit/core/chain-domain` has **6 external dependent packages across 32 edges** — `marketplace/facade/dev-cli`
  (6), `oya/application/crates/oya-application-app` (18), `observability/core/aggregate` (2),
  `observability/core/api` (2), `iam/adapters/pdp-cedar` (2),
  `oya/intelligence/crates/oya-intelligence-cloud-mutation-domain` (2). Every one is a real
  `core`-reaching-across-capabilities edge — findings, not regressions.
- **`audit/adapters/file` has 2 external dependent packages across 8 edges** —
  `marketplace/facade/dev-cli` (6) and `iam/adapters/pdp-cedar` (2). An adapter reached from
  another capability is a distinct rule from the `core` rule and is NOT dispositioned here.
- **`chain-domain` and `adapters/file` therefore stay PUBLIC in this PR**, with their dependents
  recorded as the advisory baseline. Tightening `chain-domain` is a second PR that must first route
  its 6 dependents through `audit/ports/*`. Shipping the tightening and the 32 repairs together
  would be the Bun defect in reverse: one PR that cannot be reviewed.

**This is the finding, not a caveat.** In the cleanest S0 capability in the repository, **6 of 18
packages** can be tightened with zero fallout. The other 12 are either the cross-capability seam
(`ports`, correctly PUBLIC) or carry live external edges. Anyone budgeting the 2229-declaration
backlog off a "start with a clean capability, it'll be easy" premise should use this ratio.

**Acceptance test for the migration PR (RED and GREEN both required):**

```
RED    Before tightening, add to any package outside audit/ a dep on
       //audit/core/emission-domain:audit-emission-domain. Assert `buck2 build`
       SUCCEEDS. (Proves PUBLIC is what permits it today — the control.)
GREEN  Apply the tightening. Re-run the same build. Assert it FAILS with
       buck2's visibility diagnostic naming the target.
       Then remove the synthetic dep and assert `buck2 build //audit/...` is green.
GUARD  Assert `grep -rc PUBLIC audit --include=BUCK | awk -F: '{s+=$2} END {print s}'`
       drops from 24 to 16, that audit/ports/* is untouched (10 declarations),
       and that chain-domain (4) and adapters/file (2) are untouched.
```

The RED leg is not optional decoration. A tightening PR whose "after" state is green proves nothing
unless the "before" state was shown to permit the edge through the same command.

**Verify with buck2, and report honestly.** Local buck2 may fail on crates.io TLS
(`invalid peer certificate`) for uncached crates; that is a local network condition, not a build
failure, and must be reported as such rather than as a red gate.

## Consequences

- **Every existing L3 gate becomes testable for emptiability, and some will fail.**
  `facade-core-layering` fails T2 today (D1). It is the example because it is otherwise one of the
  better-built gates in the fleet — careful comment handling, fail-closed on unreadable manifests,
  rename-resistant baseline keys. **The T2 defect is orthogonal to gate quality**, which is why a
  code-review pass does not catch it and a mechanical test does.
- **`within_view` goes from 0 uses to being the mechanism D1 assigns the DAG-direction fact to.**
  That is a new dependency on a buck2 feature this repo has never exercised. ADR-0631 D10 records an
  in-tree verification (2026-07-31) that a forbidden dep yields
  `Target's within_view attribute does not allow dependency ...`; that is one observation on a branch
  that has not merged, and D6 deliberately does not depend on `within_view` — it uses `visibility`
  only, which is already in use 2229 times.
- **2229 `PUBLIC` declarations are now a measured backlog rather than a default.** D6 retires 8 of
  them — and D6's measurement says the per-capability yield is 6 packages in 18, so the backlog is
  not 2229 easy edits. At that rate this is a long program; it is also the only item here that
  removes probes rather than adding them.
- **`governance/check/**` (56 crates) needs a dispatch ruling this ADR does not make.** It is named
  by no lane and reached only by a non-propagating baseline. Whether those checks are promoted,
  merged into `//ci/...`, or deleted is a separate decision requiring positive evidence of deadness
  per crate — **absence of a CI caller is not evidence of deadness**, and this ADR explicitly does
  not authorize deleting them.
- **D3 adds a required output field to every L3 check.** Cheap per-check, 111+ checks wide.
- Two-way door: every element is a constraint tightening or an added output field, each revertible
  in isolation. D6 is revertible by reverting one PR.

## Alternatives rejected

**"The earliest layer that can express it."** The rule this replaces. *Expressible* has no test, so
every fact is expressible everywhere by writing a scanner, which is the behaviour that produced the
current fleet.

**Promote checks to gates on ownership alone.** Ownership makes a check *correct*; it does not make
it *quiet*. A gate that owns its fact and fires falsely trains the fleet to route around gates, which
costs more than the fact is worth. Hence D2's separation of destination from blocking status.

**Demote every false-positive check to L4 prose.** The mirror error. It converts a working detector
into a paragraph, and this repository's characteristic defect is prose that nothing enforces.
Advisory-at-its-own-layer preserves the signal.

**Audit the 111 checks for dispatch.** Attempted in this lane and abandoned on measurement: the
premise was wrong (`buck2 test //ci/...` reaches all 55 `ci/` crates), and dispatch is the wrong
axis. A dispatched gate with an empty probe is worse than an undispatched one, because it reports
green. T2 is the axis that matters.

**Require `within_view` in D6.** Rejected for the first step: it is unused repo-wide (0 occurrences),
so the first migration PR would be validating a new mechanism and a new policy simultaneously. `visibility`
is already exercised 2229 times. `within_view` follows once `visibility` tightening has a landed precedent.

## Verification

1. **D1 is applied, not merely stated:** `ci/facade/facade-core-layering` carries a test that applies
   R1 (relocate a scanned capability one level deeper into a fixture tree) and asserts the gate goes
   RED rather than reporting a reduced `capabilities_scanned` with a green verdict.
2. **D3 holds for at least one gate end-to-end:** that same gate exits non-zero when pointed at an
   empty directory, with failure text naming the empty population, and the test proving it fails
   first without the fix (RED) and passes with it (GREEN).
3. **D4 is applied to one live key:** one name-keyed or glob-keyed scan is re-keyed onto the
   `registry/catalog` `capability:` facet, with a test that renames the crate and asserts the check's
   observation count is unchanged.
4. **D6 has landed as one PR**, with: the 6 clean `core` packages at `["//audit/..."]` (8
   declarations); `chain-domain` still `PUBLIC` with its 6 dependents recorded; `adapters/file`
   still `PUBLIC` with its 2 dependents recorded; the RED leg's output attached showing the
   pre-tightening build permitted the synthetic edge; `buck2 build //audit/...` green.
5. **The `PUBLIC` census is re-run and reported** after D6: `audit` 24 -> 16, repo-wide
   2229 -> 2221. A different number means something else moved in the same window and must be
   explained before the PR merges.
6. No claim in this ADR's Context table is cited downstream without its probe. Two figures moved
   under re-measurement and their causes are NOT equally established:
   - **"4 query engines" -> 1 is explained**: the original probe counted crates that spawn `buck2`;
     inspecting the subcommand shows three call `build`/`run`, not a query. Cause verified.
   - **2223 -> 2229 is NOT explained.** Six declarations. It could be dev advancing between the two
     measurements or a difference in probe shape; this lane did not determine which, and does not
     assert one. **Anyone relying on the `PUBLIC` count as a ratchet baseline must re-derive it at
     their own commit rather than citing either number**, and D6's verification step is written to
     catch exactly this by requiring the post-PR census to land on 2221 or be explained.
