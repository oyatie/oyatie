---
doc_class: Program-Operations-Journal
doc_status: published
entry_id: W0-B-20260809-neutral-split-gate
wave: W0-B
run_id: g006-u1-neutral-split-gate-20260809
incident_class: planned-wave-increment
recorded_at: 2026-08-09
terminal_state: passed
---
# W0-B 2026-08-09 neutral split gate

## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-09) |
|---|---|---|
| Repository baseline | `origin/dev` @ `885794461` (this branch's merge-base) | Current baseline. |
| Upstream Kubernetes pin | `v1.36.1`; annotated tag object `5b824a493a7ca248b726b6ea09d53842b9b992c2`; peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | Unchanged from W0-A/W0-B. Not consumed by this run — no snapshot is taken. |
| Engine | `build/port-engine/*`, v0 | Unchanged by this run. It becomes a SCANNED root here; no engine code is edited. |
| Neutral rule pack | `specs/port-rules/**`, v0 | The root is CREATED by this run, carrying one canary record and no language rule. |
| Corpus rule policy | `specs/k8s-port/rules/**`, v0 — unauthored | Still not in force. |
| Go front end | Bootstrap extractor; strategy ruled | Still not in force. No ruling made or assumed here. |
| Reproducibility tuple / receipt schema | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Six axes unchanged. No receipt emitted by this run. |
| Program authority | ADR-0637 / ADR-0638 (archived; live via apex ADR-0704) | Accepted 2026-08-05; W0 only. |

## Entry identity

- **Stable entry ID:** `W0-B-20260809-neutral-split-gate`.
- **Wave:** W0-B. **Run ID:** `g006-u1-neutral-split-gate-20260809`.
- **Judgment class:** planned wave increment — G006 Deliverable 4, the mechanical enforcement of the
  language/corpus split.
- **Recorded:** 2026-08-09.
- **Branch:** `impl/g006-language-rule-pack`, cut from `origin/dev` @ `885794461`. No pull request:
  the lane opens exactly one, from its Land phase.
- **Governing contract:** `docs/programs/k8s-port/MAPPING-G006-go-rust-language-pack.md`, §2 D5.

## Scope and inputs

- **Base SHA:** `885794461`. **Head at authoring:** the commit this entry accompanies.
- **Touched:** `ci/facade/k8s-program-docs/src/lib.rs`,
  `ci/facade/k8s-program-docs/tests/live_tree.rs`, `specs/port-rules/canary/NEUTRALITY-CANARY-000.md`
  (new root), `specs/reachability-registry.json` (one prefix), and this entry.
- **Untouched, deliberately:** `os/`, `specs/k8s-port/divergence-ledger.json`,
  `governance/check/adr-citation-closure/adr-citation-closure-policy.json`,
  `docs/programs/k8s-port/wave-registry.rdoc`, and every file under `build/port-engine/`.
- **Receipt axes:** NO RECEIPT EXISTS for this run. Nothing was emitted; the six axes are typed by
  the engine and carry no value. Recorded explicitly rather than left blank.

## Judgment

What was judged: whether the split between neutral language rules and corpus rules can be enforced
mechanically rather than by convention, and whether the enforcement can be shown to FIRE rather than
asserted to.

Evidence considered, and the results:

- **The check belongs in the existing R-DOC gate crate, not a new one.** That crate already walks
  `specs/port-rules/**` recursively and already parses `rule_kind: neutral|corpus`; it enforced
  nothing about content, which was precisely the gap. A sibling gate would have needed five further
  surfaces — a required-workflow row, a catalog entry, a reachability prefix, an affected-set row,
  and a self-conformance entry — each of which has already reddened a pull request in this
  repository. The neutral engine's compile-time const scan was also ruled out, and by measurement
  rather than by preference: its own source records that two more const passes crossed the
  `long_running_const_eval` budget.
- **The needle list is DUPLICATED, not shared.** `ci/facade/*` depending on `build/*` is a layer
  inversion, and a shared crate for five byte strings is not worth a new dependency edge. The
  duplication is five literals with a comment naming the other copy and pointing at the derivation.
- **The scan roots are hard-required, not skipped when absent.** A missing root and a clean root
  must not look alike. `collect_files` already fails closed on a missing directory and that is the
  wanted behaviour, so both roots go through it unguarded.
- **Exactly one file needed an exemption, and it was measured, not assumed.** A case-insensitive
  scan of all seven files under the engine root matched exactly one:
  `build/port-engine/core/port-engine-kernel/tests/neutrality.rs`, the kernel's own neutrality
  proof, which must spell the needles out to demonstrate they bite. It is exempt by EXACT
  repository-relative path — not a prefix, not a glob, not a list — because an exemption that can
  widen without a code change is how a gate starts passing because it observes nothing.
- **The green needed its own liveness probe, and a second one.** Two were added. A zero-artifact
  neutral scan is unconditionally RED, so an empty scan cannot pass as a clean one. And because a
  unit test hands the scan its own inputs and therefore cannot prove the walk REACHED either root on
  disk, that assertion was placed against the live tree instead.

## Change disposition

**No language rule changed, and none exists.** The exact reason: `specs/port-rules/lang/go-rust/**`
is unauthored — this run creates the rule ROOT and the enforcement over it, not its contents.

One record is added, `NEUTRALITY-CANARY-000`, under `specs/port-rules/canary/`, which ADR-0637 D1
names as neutral rule data in its own right. It is a liveness canary and not a translation rule, and
its identifier is deliberately outside the `GO-RUST-<FAMILY>-<NNN>` grammar the language rules are
allocated from so that it can never be mistaken for one or collide with one.

Gate changes, by class:

1. **Two finding codes**, `R-DOC-NEUTRAL-CORPUS-TOKEN` and `R-DOC-NEUTRAL-SCAN-EMPTY`. The second is
   not decoration: without it a scan that reached nothing would be indistinguishable from a scan
   that found nothing.
2. **Path as well as contents.** A rule named for a domain noun carries the token in its filename,
   which is also its rule identifier, while its body stays spotless. Scanning only the body would
   have missed the form that survives a body review.
3. **Case-insensitive matching**, matching the neutral engine's own predicate.

## Gate result

**No red gate remained at the close of this run.** Recorded explicitly, per the schema.

The insert → red → remove demonstration is the deliverable's own bar — "prove it fires" — and a unit
test does not discharge it, because a unit test never touches the tree. The literal output of both
halves is recorded in the run report accompanying this entry: a single corpus token inserted into
`specs/port-rules/canary/NEUTRALITY-CANARY-000.md`, the gate RED naming
`R-DOC-NEUTRAL-CORPUS-TOKEN` and the offending path, the token removed, the gate GREEN again.

## Reproduction

- **Commands:** `buck2 test //ci/facade/k8s-program-docs:ci-k8s-program-docs-gate
  //ci/facade/k8s-program-docs:ci-k8s-program-docs-unittest`, run from the lane worktree root.
  buck2 is canonical; `cargo build/test/check/clippy` are hook-blocked in this repository and were
  not used.
- **Configuration identity:** default `.buckconfig` of the worktree; no `--config` override, no
  remote execution.
- **Resources:** single developer workstation (darwin/arm64), all commands local, no network input,
  no cluster. Other lanes were building concurrently on the same project root during this run, which
  is why the base-versus-head failing-set comparison is reported with its limitation named rather
  than as a clean number.
- **External inputs:** none.

## Review

- **Reviewer role:** pending. This entry accompanies the implementation commit; the lane's review
  pass runs separately from authoring, as the repository requires.
- **Findings resolved during authoring, by the author:** the naive form of this check scanned all of
  `build/port-engine/**` and would have gone RED on the landed tree, on the kernel's own neutrality
  proof. Caught by measuring the token hits across the root BEFORE writing the scan rather than by
  running it afterwards.
- **Deferred findings:** none open.

## Terminal state

`passed`, for the deliverable this entry covers.

W0-B as a whole remains incomplete and this entry does not close it. Its wave-registry row is
untouched and still reads `completed=false`, so the R-DOC completed-wave predicate does not fire
against it.

Still owed by the surrounding lane, and named so the green here is not read as more than it is:

- **The language rules themselves.** `specs/port-rules/lang/go-rust/**` holds no rule. The
  enforcement exists before its subject, which is the correct order and not a completed pack.
- **A neutrality claim stronger than a canary set.** Five needles are not a decision procedure for
  "specific to one source repository", and no finite list is. Review carries the rest.
- **The universality test.** A pack that is about a language rather than about one repository can
  only be told apart by running its shape matchers against a different repository, and no matcher
  exists in owned Rust yet.

## Graduation links

- Program authority: ADR-0637 and ADR-0638, both archived under `docs/adr-archive/` and live via
  apex ADR-0704. ADR-0637 D1 is what names `canary/**` as neutral rule data.
- Lane contract: `docs/programs/k8s-port/MAPPING-G006-go-rust-language-pack.md` §2 D5 (where the
  split gate lives) and §3 D6 (the test that separates a language rule from a corpus rule).
- Preceding wave entry: `W0-B-20260809-port-engine-kernel.md` in this lane, and its PR #1621, which
  landed the neutral engine this run now scans.
- Rule identifiers touched: `NEUTRALITY-CANARY-000` (created).
