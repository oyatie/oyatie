---
doc_class: Program-Operations-Journal
doc_status: published
entry_id: W0-B-20260809-port-engine-kernel
wave: W0-B
run_id: port-engine-w0b-20260809
incident_class: planned-wave-increment
recorded_at: 2026-08-09
terminal_state: partial
---
# W0-B 2026-08-09 port-engine kernel

## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-09) |
|---|---|---|
| Repository baseline | `origin/dev` @ `1d31052774ef580553a5ff81014849bb38d6e327` (this branch's merge-base) | Current baseline. Rebased onto it mid-run after PR #1620 merged. |
| Upstream Kubernetes pin | `v1.36.1`; annotated tag object `5b824a493a7ca248b726b6ea09d53842b9b992c2`; peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | Unchanged from W0-A. Not consumed by this run — no snapshot is taken yet. |
| Engine | `build/port-engine/*`, v0 | LANDING in this run: one crate, `build/port-engine/core/port-engine-kernel`. Seams and refusals only; not in force as a producer. |
| Neutral rule pack | `specs/port-rules/**`, v0 — unauthored | Still not in force. This crate defines the pack's SHAPE (`RulePack`), never its data. |
| Corpus rule policy | `specs/k8s-port/rules/**`, v0 — unauthored | Still not in force. |
| Go front end | Bootstrap extractor; strategy ruled | Still not in force. `SourceModel` is the seam it will inhabit; no producer exists. |
| Reproducibility tuple / receipt schema | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Six axes now TYPED in the engine (`Receipt`, `RECEIPT_AXES`) and enforced by `verify`. No receipt has been EMITTED, so no axis carries a value. |
| Program authority | ADR-0637 / ADR-0638 (archived; live via apex ADR-0704) | Accepted 2026-08-05; W0 only. W1+ remains unapproved. |

## Entry identity

- **Stable entry ID:** `W0-B-20260809-port-engine-kernel`.
- **Wave:** W0-B. **Run ID:** `port-engine-w0b-20260809`.
- **Judgment class:** planned wave increment plus a two-round adversarial review disposition.
- **Recorded:** 2026-08-09.
- **Branch:** `port/engine-w0-skeleton-retry3`. **Pull request:** #1621, base `dev`.
- **Masterplan work item:** MPV2-0046 (neutral engine, SourceModel bootstrap, rule pack v0).
- **Worktree:** `.claude/worktrees/wf_553045af-c17-7`.
- **Head at authoring:** `8ac8b74764f2e0985f968374124b2315e8c7d715`. The live PR head is authoritative and is deliberately not pinned here, because committing this journal advances it.

## Scope and inputs

- **Base SHA:** `1d31052774ef580553a5ff81014849bb38d6e327` (merge-base with `origin/dev`).
- **Head SHA at authoring:** `8ac8b74764f2e0985f968374124b2315e8c7d715`, plus the review-disposition commits this journal accompanies.
- **Kubernetes pin:** unchanged and unconsumed; see the baseline header.
- **Receipt axes:** NO RECEIPT EXISTS for this run, and the reason is structural rather than an omission — nothing has been emitted. The six axes are typed and compared by the engine; they are populated by an adapter that W0-B has not landed. Recorded explicitly rather than left blank.
- **Landed tree:** `build/port-engine/core/port-engine-kernel/{Cargo.toml,BUCK,src/lib.rs,tests/seams.rs,tests/neutrality.rs}`, its `registry/catalog/port-engine-kernel.yaml` row, two `OWNERS` files.
- **Governance surfaces touched:** `governance/check/adr-citation-closure/adr-citation-closure-policy.json` (census re-freeze), `ci/facade/scan-root-liveness` policies (three forward declarations retired), `ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json` (new root declared).

## Judgment

What was judged: whether the neutral engine's seams can be landed as W0 authorizes — real seams, real refusals, no front end, no rule data, no corpus — and whether the neutrality claim the crate makes about itself is true.

Evidence considered, and the results:

- **Neutrality is a build error, not a test.** The kernel reads its own bytes with `include_str!` at compile time and refuses to compile on a forbidden sequence. The claim that "the kernel is exactly this file" was FALSE in the first attempt: the unscanned-code needles were whitespace-sensitive substrings guarding whitespace-insensitive grammar productions, and `pub mod\nsecond;` compiled a corpus-carrying second file with the gate green. That escape was reproduced before it was fixed. The needles are now matched on identifier boundaries, and four planted defects were run and all four now fail the build with E0080.
- **The neutrality CLAIM was overstated and is now calibrated.** The scan rejects a fixed canary set of five needles. It is not a decision procedure, no finite needle list could be, and the module docs now say so in the crate itself rather than in a review reply.
- **Every fail-closed refusal is exercised by an in-memory fake.** No filesystem, no clock, no front end, no rule corpus. 29 seam tests.
- **Three seams accepted a value they then treated as authoritative without proving they could.** Found by review, judged live, fixed in this run — see Change disposition.

## Change disposition

**No rule change.** The exact reason: `specs/port-rules/**` is unauthored, so there is no rule ID to touch. This run lands the engine that will later consume rule data; the data itself is a separate lane of the same wave. No corpus rule policy under `specs/k8s-port/rules/**` was created or modified either.

Engine changes, by class rather than by review thread:

1. **Fail closed on an input the kernel cannot draw a conclusion from.** Three seams took a value on trust and then reasoned from it.
   - `plan` recorded a pack's declared rules with `entry().or_insert()`, so a pack declaring `[r1, r2, r1]` silently kept the first position and dropped the second. Whether the ambiguity was ever caught depended on what `rules_for` happened to answer. Now `PortError::DuplicateRule`, refused at the declaration exactly as `DuplicateUnit` and `DuplicateRegion` are.
   - `LanguagePair::slug` tested two conditions derived from the one hostile byte a previous review named, while its own doc promised ONE path component. `a/b` rendered `a/b-c`; an absolute slug would have made a later `Path::join` discard the namespace root entirely. Replaced with an allowlist grammar derived from the value's use, and the separator's exclusion from that grammar is now a compile-time assertion instead of a prose claim.
   - `verify` compared six receipt axes without asking whether they said anything. Asymmetrically incomplete receipts — a populated previous against an all-empty current — made every axis "differ", so an unfilled receipt manufactured a six-axis explanation for an arbitrary byte change. Now `Delta::IncompleteReceipt`, placed strictly AFTER the unchanged early return so a receipt that decided nothing cannot redden an identical tree.
2. **A new top-level root must be declared on the surfaces that enumerate roots.** `build/` is this repository's first crate-bearing meta root. `tier-dependency-acyclicity-policy.json` derives its whole universe from `crate_root_globs`, so the crate was invisible to the gate — and invisible to the gate's own anti-silent-exemption rule too, since that rule only fires for crates already discovered. Two lines: `build/*/*/*` in `crate_root_globs` (four segments, not the repo's usual three — the crate sits at `build/port-engine/core/port-engine-kernel`, and a `build/*/*` glob would have matched nothing, which is the same false green in a new spelling), and `build` in `unclassified_roots`, which is registry-sanctioned because `build/` is already one of the seven `meta_directories` of the CLOSED capability registry.
3. **`scan-root-liveness` forward declarations retired.** Three exemptions named "created when `build/` lands its first crate" as their expiry event. That event is this run, so they are gone and `build` is a live audited root. `app/`, `base/` and `policy/` remain absent and remain declared; whoever lands their first crate owes the same retirement.
4. **Citation census re-frozen by ATTRIBUTION, not by recomputation.** `files_scanned` moves because tracked files were added. The distinction from a narrowed scan matters and was established by measurement: the tracked-add and scanned-add deltas agree, and every other pinned term and every finding count is unchanged, which a narrowing would have moved. Details are in the policy's own `_port_engine_w0_add_2026_08_09` and `_port_engine_w0b_journal_2026_08_09` notes.

## Gate result

**No red gate remained at the close of this run.** Recorded explicitly, per the schema.

Reds encountered and their root causes:

- `governance/check/adr-citation-closure` — RED on the pinned census. Root cause: the run adds tracked files with scanned extensions, and the census is pinned by equality precisely so an add and a narrowing cannot be confused. Resolved by re-freezing with attribution (Change disposition 4). GREEN after.
- `ci/facade/scan-root-liveness` — RED on `forward_declarations_are_all_still_absent`. Root cause: a declared-absent path arrived, which is the gate working as designed. Resolved by retiring the three declarations. GREEN after.
- `ci/facade/layer-dependency-acyclicity` — was GREEN over an INCOMPLETE graph, which is the harder failure. Root cause: a new top-level root not enumerated in the policy the gate derives its universe from. Resolved by declaring it (Change disposition 2).

Evidence that the acyclicity fix is real and not a second false green, since a green gate over an unchanged universe would look identical: with `build/*/*/*` added but `build` withheld from `unclassified_roots`, the gate REDs and names the crate —

```text
[TDA-UNDECLARED-ROOT] build: crate root `build` is declared in none of
`service_roots`/`capability_roots`/`unclassified_roots`, so its crates (e.g.
`build/port-engine/core/port-engine-kernel`) carry no tier and EVERY dependency edge
touching them is SKIPPED
```

so the glob does resolve the four-segment path and both lines are load-bearing. With both lines present, `frozen_baseline_is_exactly_the_live_violation_set` passes, i.e. the baseline did NOT move: the crate has an empty `[dependencies]` and its BUCK target lists no first-party deps, so it contributes zero edges and zero new violation subjects.

Final runs on the disposition tree:

```text
buck2 test //build/port-engine/...
  test result: ok. 29 passed; 0 failed; 0 ignored (seams)
  Tests finished: Pass 2. Fail 0. Timeout 0. Fatal 0. Build failure 0

buck2 test //build/port-engine/... //ci/facade/layer-dependency-acyclicity/... \
  //ci/facade/scan-root-liveness/... //ci/facade/crate-catalog-coverage/... \
  //ci/facade/module-membership/... //governance/check/adr-citation-closure/...
  Tests finished: Pass 12. Fail 0. Timeout 0. Fatal 0. Build failure 0
```

The preceding commit's evidence over the wider 18-target set was `Tests finished: Pass 18. Fail 0`, and `presubmit` was SUCCESS on the pre-rebase head.

## Reproduction

- **Commands:** the two `buck2 test` invocations quoted above, run from the worktree root. buck2 is canonical; `cargo build/test/check/clippy` are hook-blocked in this repository and were not used.
- **Configuration identity:** default `.buckconfig` of the worktree; no `--config` override, no remote execution (`Cache hits: 0%`, all commands local).
- **Resources:** single developer workstation (darwin/arm64). The acyclicity gate walks the whole tree and takes ~19s; the kernel's 29 seam tests and its neutrality proof run in under 4s. Peak observed command concurrency 64, all local. No network input (`Network: Up: 0B`), no external service, no cluster.
- **External inputs:** none. The kernel has zero dependencies by design — no clock, no rand, no net, no filesystem — which is what lets the receipt/verify seam be proven without any of the machinery it will later drive.
- **Worktree limits:** one isolated worktree, one lane. The lane was rebased once mid-run onto the merged PR #1620.

## Review

- **Reviewer role:** adversarial code review agent (codex), two rounds against PR #1621.
- **Round 1 — 7 threads.** 6 accepted, 1 accepted in part. Accepted: missing `data_class` annotations; forgeable `verify` input (the changed-set argument was removed so the diff is derived from the trees); pair-slug collision; `rules_for` order not proven; duplicate declared regions collapsing into a set; no renderer-side error variant. Accepted in part: the verify thread's other half — that a moved axis should be RED — was REFUSED, because the program authority says an UNEXPLAINED change is red and a moved axis is precisely what explains one; classifying it red inverts the decision. The five-canary neutrality thread's REMEDY was refused (deriving needles from corpus policy would make the neutral kernel read the corpus it is defined by not knowing) while its CLAIM was accepted and the docs calibrated.
- **Round 2 — 15 threads.** 3 accepted, 2 accepted in part, 10 refuted.
  - Accepted, and all three are the same class: the duplicate rule declaration, the slug guard derived from a named byte rather than the value's use, and the unregistered new top-level root.
  - Accepted in part: the incomplete-receipt thread — the false acceptance is real but runs the OPPOSITE way to the mechanism the thread described (two EQUALLY incomplete receipts differ on nothing and fall to `Unexplained`, which is the strictest verdict, not a false Green; the false Green needs ASYMMETRY). Fixed in the direction the finding points, not the one it names. And this journal itself — the obligation is real and stated by the program's own index, while the framing that it is a red gate or a merge blocker is not: the fail-closed predicate is "a COMPLETED wave has zero journal entries", and W0-B is not completed by this run.
  - Refuted with evidence, 10: six were findings already fixed by the preceding commit whose threads were never closed; the rest were a receipt invariant that the authority does not state, a `#[non_exhaustive]` request against a `publish = false` crate with zero dependents in a repository where 19 of 1140 files with a `pub enum` use it, and a masterplan dispatch hold that is repo-global and under which the immediately preceding wave already merged.
- **Deferred findings:** none open from either round. Two threads were judged and left with the refuted half explicitly recorded rather than silently dropped.
- **Review evidence reference:** PR #1621 review threads, replies carrying the per-thread evidence.

## Terminal state

`partial`.

This run lands the ENGINE CRATE ONLY. MPV2-0046 covers "neutral engine, SourceModel bootstrap, and rule pack v0", and only the first has landed. W0-B is therefore NOT completed by this entry, which is also why the R-DOC completed-wave predicate does not fire against it.

Still owed by W0-B, and the durable blocker for each:

- **SourceModel bootstrap** — the Go front end that inhabits the `SourceModel` seam. No extractor and no snapshot is admitted yet.
- **Neutral rule pack v0** — `specs/port-rules/**` is unauthored. The engine defines its shape and refuses a pack that contradicts it; there is no pack.
- **A first emitted receipt** — the six axes are typed and compared, but no axis has ever carried a value, so the determinism claim is proven over fakes and not yet over the corpus.

## Graduation links

- Program authority: ADR-0637 (owned deterministic Go-to-Rust port engine) and ADR-0638 (mechanically maintained Kubernetes Rust port), both archived under `docs/adr-archive/` and live via apex ADR-0704.
- Tier-3 no-unwrap/expect/panic rule in production code: ADR-0083.
- Registered regenerable region identity, which `RegionId` names: ADR-0597.
- Repo layout authority for the capability registry this run's root declaration rests on: ADR-0562 as amended by ADR-0615.
- Preceding wave entry: `W0-A-20260805-gjc-handoff.md` in this lane, and its PR #1561.
- This run: PR #1621.
- Masterplan work item: MPV2-0046.
