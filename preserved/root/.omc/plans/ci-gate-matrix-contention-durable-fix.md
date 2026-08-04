# Durable fix: the census gate's fourth validator has zero test callers

**Status: PENDING APPROVAL** — read-only analysis. No repo file changed except this plan.
Mode: RALPLAN-DR, DELIBERATE. Iteration 4 (Critic ITERATE on iteration 3).
Evidence: `origin/dev` @ `d11567a1a` via `git show`. The working tree is `preserve/hermes-w1-dirty-20260630`
and its `ci/` differs; nothing below was read from the checkout.

**The change is ONE LINE.** Iteration 3 proposed a lib extraction plus three test arms (723 lines of plan).
Verification of iteration 3's own premises killed most of it. What survives is one appended statement.

---

## 1. The defect, and what already covers it

`src/bin/adr-census-epoch-receipt-gate.rs:122-139` — `validate_gate_from_event` chains four validators.
Three of them have many integration callers. The fourth, `validate_dormant_p3_epoch_policy_for_event`
(`lib.rs:1319`), appears **zero** times in `tests/snapshot_integration.rs`. Only matrix leg 1's chained
`buck2 run` reaches the composition.

Five facts, each verified this iteration, that shrink the gap to one line:

1. **The `?` chain's fourth stage is reachable in an existing test.** `snapshot_integration.rs:1543-1562`
   runs stage #1 (`select_census_event_from_event`), stage #2 (`validate_census_event_transition`), then
   emits and runs stage #3 (`validate_adr_census_epoch_receipt_for_event`) — in the gate's order, on the
   gate's types, holding a real `ValidatedCensusEventSelection` at `:1553`.
2. **Stage #4 takes exactly that value.** `pub fn validate_dormant_p3_epoch_policy_for_event(selection: &ValidatedCensusEventSelection)`
   (`lib.rs:1319-1323`). The target already deps `:ci-scm-facts-snapshot-lib` (`BUCK:115-116`).
3. **Stage #4's failure mode is already tested.** `:2058-2068` mutates `governance/corpus/doc-parser/src/lib.rs`
   in a fixture and asserts `contains("parser source set is invalid")` — through `dormant_p3_epoch_fingerprint`,
   which reaches the *same* builder (`build_p3_adr_census_epoch_receipt_at_revision`).
4. **The binary's composition is NOT untestable.** `BUCK:61-67` declares `rust_test ci-scm-facts-snapshot-gate`
   with `crate_root = "src/bin/adr-census-epoch-receipt-gate.rs"`. The bin crate compiles as a test crate;
   its 8 `#[test]`s run; `validate_gate_from_event` is an in-crate private fn a sibling test can call today.
5. **Four candidate hosts exist**, not one: `let validated = …` at `:1490`, `:1553`, `:1683`, `:1739`.

So the realizable net-new coverage is **one edge**: that stage #3 returning `Ok` chains into stage #4.

---

## 2. PRINCIPLES

**P1 — Cover the composition where it can already be reached.** Iteration 3's P1 asserted a four-call chain
in a `rust_binary` is "untestable as a binary." Fact 4 falsifies it. Extraction was never required.

**P2 — Hermetic coverage and live attestation are two obligations.** A hermetic test proves the logic runs on
synthetic input; it does not prove the real candidate tree passes the real provider event. Leg 1 stays.

**P3 — A plan longer than its change is itself the defect.** Iteration 3 was 723 lines for a one-line gap.

---

## 3. DECISION DRIVERS

**D1 — No gate may become dark, no live attestation silently substituted.** Any option whose failure mode is
"runs never, passes always" or "runs hermetically, attests nothing" is disqualified.

**D2 — Workflow edits are asymmetrically expensive.** `automation-language-policy` is an exact merge-base
ceiling keyed `<workflow>::<job>::<step name>`: observed∉baseline ⇒ `unbaselined` (born-blocking); observed >
baseline ⇒ `growth`; observed < baseline ⇒ `baseline_stale`. `.github/actions` does not exist on `origin/dev`,
so any composite action is a born-blocking new key. Zero-workflow options are strictly cheaper.

**D3 — Zero new BUCK files.** A new BUCK file re-partitions filegroups and expires derived floors.

---

## 4. OPTIONS

### M0 — RECOMMENDED. Append one line to the existing test at `:1562`

```rust
validate_dormant_p3_epoch_policy_for_event(&validated)
    .expect("the gate's fourth stage must run on a stage-3-validated selection");
```

Plus the import. No new fixture, no new arm, no BUCK change, no `write_census_control` sequencing, no
canonical-path problem.

| Pros | Cons |
|---|---|
| Closes the measured gap: the never-called fourth validator gains a caller, reached through the gate's own stage order | Green-path only; does not prove stage #4's error propagates out of the chain (that mode is already covered at `:2058-2068`) |
| One line, one file, one crate. No workflow, no baseline, no BUCK, no new policy | Covers the **library** stages, not the bin's `validate_gate_from_event` — see gaps G1/G2 |
| Four viable hosts; a red one is swapped, not redesigned | Leaves leg 1 the sole live provider-attested check |
| `git revert` of one line | |

### M1 — extract the composition into `pub fn validate_census_gate_for_event` (lib)

Iteration 3's recommendation. Moves the four calls from the bin into the lib so an integration test can call
the chain as one unit; the bin becomes env-read + one call.

| Pros | Cons |
|---|---|
| The chain has a single named expression a test can call as a unit | **Premise falsified.** Fact 4: the chain is already callable in-crate from `ci-scm-facts-snapshot-gate` |
| Would host future stage-5 additions in a covered place | The **binary** the workflow runs is still executed only by matrix leg 1 — after M1, the composition that actually gates merges remains untested |
| | Delivers the same one edge M0 delivers, at ~15 moved lines across 2 files |
| | Perturbs two `P3_PRODUCER_GATE_SOURCE_PATHS` entries (`lib.rs:120-137`), so it moves the dormant P3 fingerprint — safe, but a reviewer must be walked through it |
| | Alone it is a refactor with a deferred guard: the shape this plan exists to criticize |

Per the Critic, M1 must carry `output: &Path` rather than hard-wiring `repo_root.join(ADR_CENSUS_EPOCH_RECEIPT_PATH)`
— which means the canonical-path choice **stays in the bin and stays uncovered** either way.

### A — dissolve `gate:` (retire leg 1, delete `matrix.include:`) over several PRs

| Pros | Cons |
|---|---|
| The contended `include:` line ceases to exist | **Deletes the only live-tree, provider-attested census check** — the substitution D1 forbids |
| Removes an inline-shell baseline key legitimately | RED on arrival without a prior PR relaxing `rust_first_automation_hygiene.rs:432-437`, whose message reads *"the live scm-facts census receipt gate must retain provider-event identity"* |

**Verdict: deferred, not rejected.** Revisit when the owned cloud-ci runner can inject a provider-event tuple
as a first-class hermetic input — then attestation relocates rather than disappears.

### THE JUDGMENT CALL — (A) M0 alone

Choosing **M0 alone; M1 not landed; remaining gaps recorded below.**

**Against (B) M0 + M1 + follow-up (a) in one PR.** M1's justification was "a `rust_binary`'s chain is
untestable." `BUCK:61-67` refutes it. M1 would move a chain from a place where it is already callable to
another place where it is callable, and the artifact the workflow actually runs stays uncovered either way.
The Critic's bundling condition — follow-up (a) — was framed as a `srcs` change to `ci-scm-facts-snapshot-unittest`
(`BUCK:82`); that target is rooted at `src/lib.rs` and cannot compile the bin's `#[cfg(test)] mod tests`, which
already runs under `ci-scm-facts-snapshot-gate`. The bundle's guard is both unnecessary and mis-specified.

**Against (C) M0 now, M1 as a later plan.** Same falsified premise, deferred. A later plan for M1 would be a
plan for a refactor with no coverage justification. If the bin's composition is later worth covering, the
correct shape is a test in the bin's own `mod tests` plus parameterizing `validate_gate_from_event`'s env
reader (the bin already factors `event_context_from_facts(read: impl FnMut…)` for exactly this) — not a lib
extraction. Recorded as G1.

**What (A) does NOT cover, plainly:**
- **G1** — the bin's `validate_gate_from_event`: the env read, the `?` chain as written there, and the choice
  of `repo_root.join(ADR_CENSUS_EPOCH_RECEIPT_PATH)` (`bin:136`). Covered only by matrix leg 1.
- **G2** — stage #4's error propagating *out* of the chain. Its failure mode is covered (`:2066`); the
  propagation edge is a Rust language property of `?`, not a repo property.
- **G3** — the `gate:` job, `matrix.include:`, the 9-entry fan-in `needs:`, and the never-executed binaries.

---

## 5. PRE-MORTEM

**PM1 — the appended line acquires a skip guard.** `snapshot_integration.rs:193-197` already contains
`if std::env::var("OYA_CI_COMMAND_PROBE_MODE")… { return; }` — a `#[test]` silently green without an env var,
in this very file. A guard placed *above* an assertion passes; assertion precision is no defense.
*Mitigation:* AC#3's diff grep. *Indicator:* any `return;` or `std::env::var` guard added near a census test.

**PM2 — provider attestation substituted by hermetic coverage.** Someone lands Option A citing "the composition
is now covered." It is — on synthetic input. Lost is the only check against the real candidate tree with the
real provider tuple, including `merge_group`, whose tuple differs from `pull_request`.
*Mitigation:* AC#4/AC#5 pin leg 1 byte-identical. *Indicator:* any PR editing `rust_first_automation_hygiene.rs:432`,
`gate_registration.rs:1294`, or `cross_artifact_agreement.rs:1095`.

**PM3 — a fifth stage is added to the bin and never to a test.** M0 covers the four stages as composed *in the
library*; the bin is where stages are actually added.
*Mitigation:* none in this PR — this is G1, recorded, not silently absorbed. *Indicator:* `validate_gate_from_event`
gaining a fifth call.

---

## 6. TEST PLAN

**Unit:** none added. `BUCK:79-81` constrains `ci-scm-facts-snapshot-unittest` to library sources; all four
validators shell out to git.

**Integration:** one appended statement at `:1562`, host
`synthetic_pr_p3_to_p2_pointer_only_rollback_emits_the_fixed_receipt`, target `ci-scm-facts-snapshot-integration`.

*Why it should be green:* `validate_dormant_p3_epoch_policy_at_revision` (`lib.rs:1325-1336`) builds the P3
receipt twice, compares, and validates structure. That builder already succeeds inside fixtures — `:2049`
calls `dormant_p3_epoch_fingerprint` on a `p3_identity_fixture` built from a bare `temp_git_repo`, and
`p3_history_fixture` (`:532-565`, a `git clone --shared` of the source root with `P3_PROTECTED_SOURCE_PATHS`
written in) is a superset of that.

*Fallback if red:* move the line to another `validated` binding — `:1490`, `:1683`, or `:1739`. A red result is
itself a finding (stage #4 rejecting a P2-active rollback selection) and must be reported, not worked around.

**E2E:** leg 1 is unchanged and remains the E2E — it runs `validate_gate_from_event` against the live tree with
the real provider tuple on `pull_request`, `push`, and `merge_group`.

**Observability:** per-target verdicts still print at named-target granularity (`✗ Fail: root//ci/facade/<crate>:…`).
No new baseline entry, so nothing to burn down. Join `completedAt < mergedAt` on `oya-ci-required` for the next
30 merges — a standing repo failure (2/30), unrelated to this change but cheap to observe in this window.

---

## 7. MIGRATION

One PR, one file, one appended statement plus its import. Touches no workflow, no baseline, no BUCK file, no
gate policy. **Rollback: `git revert`.** Leg 1 is unaffected at every point, so no coverage window exists.

**Not in scope:** matrix leg 1; the `gate:` job; `matrix.include:`; the `automation-language-policy` baseline;
the 9-entry fan-in `needs:` list; the never-executed binaries; any new `gate_registration` half.

**Leg-1 chaining note (diagnostic, not a coverage risk).** In `oya-ci-required.yml:328` the non-Windows branch is
`& buck2 test @targetArgs; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }; if (… -eq "scm-facts-snapshot") { & buck2 run …:adr-census-epoch-receipt-gate-bin … }`.
The `exit` pre-empts the live `buck2 run`, so a red hermetic arm makes leg 1 red *before* the live check runs.
The leg is red either way; only the printed failure name differs.

---

## 8. ACCEPTANCE CRITERIA (mechanical)

1. `validate_dormant_p3_epoch_policy_for_event` has ≥ 1 caller in the integration suite:
   ```sh
   git grep -c validate_dormant_p3_epoch_policy_for_event origin/dev -- ci/facade/scm-facts-snapshot/tests/   # 0
   git grep -c validate_dormant_p3_epoch_policy_for_event -- ci/facade/scm-facts-snapshot/tests/              # ≥ 1
   ```
2. The call site is **inside a test that already holds a stage-2 `validated` binding**, so it is reached only
   after stages #1–#3 succeed. Verified by inspection of the diff hunk: the added line's enclosing `#[test]`
   must contain `validate_census_event_transition` and `validate_adr_census_epoch_receipt_for_event` above it.
3. **No skip guard added.** Over the added hunk only:
   ```sh
   git diff -U0 origin/dev...HEAD -- ci/facade/scm-facts-snapshot/tests/snapshot_integration.rs \
     | grep '^+' | grep -n 'return;\|std::env::var'
   ```
   → must be empty. Non-empty is a hard block.
4. **`git diff --stat origin/dev...HEAD -- .github/ ci/facade/automation-language-policy/ '**/BUCK'` is EMPTY.**
   No workflow line, no baseline entry, no BUCK file changed — therefore no `unbaselined`, no `growth`, no
   `baseline_stale`, no filegroup repartition, and no possibility of the D2 trap.
5. Matrix leg 1 is byte-identical to `origin/dev`, and all three coupling assertions still pass:
   `rust_first_automation_hygiene.rs:427`, `gate_registration.rs:1294`, `cross_artifact_agreement.rs:1095`.
6. `buck2 uquery "kind('.*_test', //ci/...)"` post ⊇ pre; probe validated on an injected non-existent target.
   Expected delta: **zero** new targets, zero new test functions, one new statement.
7. Required contexts still `== [oya-ci-required]`, exactly one (ADR-0515).

---

## 9. ADR (draft — for ratification with the change)

**Decision.** Where a gate's enforcement is a *composition* of validators, cover the composition at the cheapest
point where its stages are already reachable — do not refactor to create a test seam that already exists. Where
the gate legitimately requires ambient provider-event context, which a Buck2 test action's sanitized environment
(`BUCK:18`) cannot supply, a workflow-level invocation retains the live provider-attested check. Both, not either.

**Drivers.** (D1) no gate dark, no live attestation substituted; (D2) workflow edits asymmetrically expensive;
(D3) no new BUCK files.

**Alternatives considered.**
*M1 — extract the composition into the lib* — rejected on measurement: `BUCK:61-67` declares a `rust_test` rooted
at the binary's `crate_root`, so the chain is already callable in-crate; and after M1 the binary the workflow runs
is still executed only by matrix leg 1.
*Option A — delete leg 1* — deferred: breaks three ratified assertions and trades live attestation for synthetic
coverage (D1). Revisit when the owned runner can inject a provider tuple hermetically.
*Face-backed event context* — rejected on measurement: the volatile face emits only
`{schema, _comment, head_time_secs, last_touch_commit, commit_author_ts_secs}` (`lib.rs:3770-3776`); the seam
does not exist.
*A `no shared crate_root` invariant* — rejected on measurement: 6 such pairs exist under `ci/`, 5 are the standard
`-bin`/`-unittest` idiom; the rule forbids co-location, not darkness, and the sixth pair is precisely what keeps
the census binary's tests compiled.

**Why chosen.** It makes stage #4 **reachable through the `?` chain** from an executed test — the one edge no
existing test covers — at one line. It claims nothing more: stage #4's failure mode was already covered at
`:2058-2068`, and the bin's own composition remains covered only by leg 1 (G1).

**Consequences.** The fourth validator gains its first test caller. The `gate:` job and `matrix.include:` are
unchanged and remain a deliberately-retained contention surface, now with a recorded reason. G1–G3 remain open
and are stated rather than absorbed.

**Follow-ups.**
**(d) — largest measured surface.** 19 of 30 `rust_binary` targets under `ci/` have **no direct executor** — not
`$(exe)` in any BUCK file, not any non-comment workflow line. Whether their `main()` behavior is covered by the
sibling gate test is **UNMEASURED**: 18 of the 19 sit in a crate that declares a `*-gate` `rust_test`, which is a
presence fact, not a coverage fact. Measuring that is the next plan.
**(a) — cover the bin's composition (G1).** Landing condition, corrected: a `#[test]` in
`src/bin/adr-census-epoch-receipt-gate.rs`'s existing `mod tests` — already compiled and run by
`ci-scm-facts-snapshot-gate` (`BUCK:61-67`) — calling `validate_gate_from_event` with its env reader
parameterized. **Not** a `srcs` change at `BUCK:82`: that target is rooted at `src/lib.rs` and cannot compile the
bin's test module. Cost is the fixture machinery, which lives in `tests/` and is not shared.
**(b) — the liveness invariant** ("every registered gate lane has at least one negative-control arm"): blocked on
the open question below.
**(c) — revisit Option A** once the owned runner can inject a provider-event tuple hermetically.

---

## 10. Open questions

- [ ] Can the deferred liveness invariant be *derived* from the build graph, or does it need a registry? — The
  precedent offered (`gate_registration.rs:1218-1220`) consumes `executed_buck2_test_patterns(&workflow)`, i.e.
  target patterns matched against package directories by `pattern_covers_package`. It derives "which directories
  a `buck2 test` pattern reaches"; it does not consume a stream of test names, and nothing in the graph exposes
  one. Mapping test name → gate lane needs a second hand-kept list. **Unresolved** — this is why (b) is deferred.
- [ ] Are the 19 direct-executor-less `rust_binary` targets under `ci/` covered by their sibling gate tests, or
  should they be deleted? — Unmeasured; entry point for follow-up (d).
- [ ] Is `//ci/facade/baseline-ratchet:oya-cloud-ci-run-terminal-state-bin` intended to stay comment-only? — Its
  comment says it is held alive by acceptance tests awaiting the owned runner: a deliberate parking spot,
  indistinguishable from rot without an expiry date.
- [ ] Will the appended line be green on the `:1553` host? — Not run: this worktree is a stale preserve branch
  whose `ci/` differs from `origin/dev`, so a local `buck2` run would evaluate the wrong tree. Confirm on a clean
  `origin/dev` checkout; §6 names the fallback and requires a red result be reported as a finding.
