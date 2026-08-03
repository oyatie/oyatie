# G030-I loop-recovery pattern and empirical-scorecard consumer proof — 2026-08-02

State: **PLANNING_ONLY — FOUR ROWS GRAPH-WIRED; STRUCTURAL JOIN PROVEN; DETECTOR-EQUIVALENCE GAP RECORDED**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G030-H-REORG-PLAN-CONSUMER-LANDED-RETENTION-PROOF-2026-08-02.md`.  
No registry row, gate, test, score card, policy, PR, GitOps declaration, or cluster state was changed.

## Result

The three `registry/loop-recovery-patterns/*.json` rows and their shared empirical scorecard are executable graph inputs, not deletion candidates:

| Path | Machine consumer | Disposition |
|---|---|---|
| `registry/loop-recovery-patterns/broken-action-sha.json` | Rust gate enumerates every JSON row, parses required fields, validates referenced score-card IDs and mistakes-ledger IDs, and enforces active → `pre_push_blocker=true` | `GRAPH_WIRED_INPUT` |
| `registry/loop-recovery-patterns/missing-nextest-profile-ci.json` | same directory consumer and fail-closed joins | `GRAPH_WIRED_INPUT` |
| `registry/loop-recovery-patterns/missing-shell-shebang.json` | same directory consumer and fail-closed joins | `GRAPH_WIRED_INPUT` |
| `registry/check-empirical-evidence/score-card-pre-push-loop-recovery-patterns.json` | `specs/score-cards.json` names its exact path; the Rust gate requires every score-card `empirical_evidence_path` to be a readable file | `GRAPH_WIRED_INPUT — EXISTENCE CONTRACT` |

This promotes four rows from the G030 protected-only queue. The reconciled totals become **152 `MACHINE_SSOT` + 906 `GRAPH_WIRED_INPUT` + 118 `POLICY_PROTECTED_MACHINE_ARTIFACT` = 1,176**. The remaining protected queue is 19 fixture residuals plus 99 non-fixture rows.

## Executable consumer path

`marketplace/facade/dev-cli/src/loop_recovery_patterns_gate.rs` owns the reader:

1. defaults `patterns_dir` to `registry/loop-recovery-patterns`;
2. enumerates and sorts every `.json` file;
3. fails if the directory is unreadable or contains zero JSON rows;
4. parses each row as JSON and requires a unique `pattern_id`;
5. constrains `status` to `active|candidate|retired`;
6. requires non-empty trigger, failure mode, detection query, recovery action, owner, evidence, and source fields;
7. requires at least one deterministic score-card reference and rejects unknown IDs against `specs/score-cards.json`;
8. requires at least one mistakes-ledger reference and rejects unknown IDs against `registry/mistakes-ledger.json`;
9. requires a boolean `pre_push_blocker`, and requires it true for every active row;
10. fails if the directory contains zero active blocker rows.

The current immutable rows are all `status=active`, all set `pre_push_blocker=true`, and join respectively to `MFL-0014`, `MFL-0015`, and `MFL-0016`. Each also cites its detector-specific score card plus the shared `score-card:pre-push:loop-recovery-patterns` row.

The shared score-card inventory row points exactly to `registry/check-empirical-evidence/score-card-pre-push-loop-recovery-patterns.json`. The gate rejects any score card whose empirical path is not a readable file. The shared query is self-referential (`oya gate validate loop-recovery-patterns`), so the implementation deliberately does not recurse; the surrounding gate is its executable form.

## Buck2 and affected-set evidence

The repository contains a dedicated Buck2 Rust test target:

`root//marketplace/facade/dev-cli:marketplace-dev-cli-loop-recovery-patterns`

Its integration test executes `oya gate validate loop-recovery-patterns`, requires success and a validation-passed receipt, and rejects an unknown flag. The affected-target-set test explicitly expects this target. This proves Buck graph visibility and affected-set intent. It does **not**, by itself, prove that every protected required CI run executed the target at this immutable tip; this census does not upgrade Buck target existence into a required-context execution claim.

## Detector/evidence semantic gap

The gate proves a structural join, not full detector/fixture equivalence:

- `detection_query` in each pattern is checked only for non-empty text; it is not required to equal the query of any referenced score card.
- The nextest pattern stores `grep -q '[profile.ci]' ...`; the actual score-card inventory uses `grep -qF '[profile.ci]' ...` and the Rust implementation performs a literal file-content check before optionally running nextest.
- The shell pattern stores `grep -RLn '^#!' scripts/ | head -1`; the score-card inventory and Rust implementation instead inspect executable files recursively and fail if an executable lacks a shebang.
- `evidence_refs` and `sources_scanned` are required non-empty arrays, but their referenced paths/fragments are not resolved by this gate.
- The empirical scorecard file is required to exist, but this lane does not parse its `check_id`, status, prevented-incident refs, caught-regression refs, or promotion basis.

Therefore the correct classification is graph-wired input with a **structural contract**, not proof that every prose detector or evidence assertion is executable. This is an enforcement-gap observation for G012/G030 follow-up, not authority to delete, rewrite, or demote the rows.

## Anti-vacuity boundary

Proven anti-vacuity:

- non-empty score-card inventory;
- non-empty pattern directory;
- non-empty required fields/arrays;
- at least one active blocker in the pattern directory;
- referenced score-card IDs and mistakes-ledger IDs must exist;
- empirical evidence path must identify a readable file.

Not proven by the current integration test:

- row-by-row negative fixtures for a missing detector reference, missing evidence target, mismatched detector query, or malformed empirical scorecard content;
- exact expected pattern count of three;
- protected required-context execution at the immutable authority tip.

The integration test's score-card command count is intentionally environment-dependent: two commands in a hermetic lane without `cargo-nextest`, three when nextest is installed. The Rust gate still verifies the `[profile.ci]` literal when nextest is unavailable, but that is not a full nextest graph/list proof.

## Verification boundary

Evidence came from immutable source and exact searches at `b651080374113aeb57500eecbd9d1326f0404e48`: the three registry rows, score-card inventory and empirical row, Rust gate, integration test, Buck target, and affected-set expectation. No local CLI execution is used as merge authority.

An independent Explore audit retried this family and failed with the same encrypted-content transport error. It remains `FAILED_TRANSPORT_NOT_APPROVE`; the mechanical proof is not an independent review approval.

## Non-actions and non-claims

- No registry or empirical row edited or deleted.
- No detector query normalized or rewritten.
- No claim that prose evidence references are mechanically resolved.
- No claim that the Buck target necessarily executed in protected required CI.
- No new generated face, move-plan JSON, or multispectrum evidence surface.
- No independent APPROVE; transport failure remains non-approval.
