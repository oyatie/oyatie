# SESSION HANDOFF — 2026-08-01/02 (pipeline + reorg drive)

## 0-LATEST — state at 2026-08-02 03:30 (read this before §0 below, which is now history)

**MERGED:** #1509 concurrency · #1510 ADR-0633 · #1511 dark lanes · #1512 python · #1513
`--find-renames`. Trunk was repaired by **#1514** (not by me — it landed the same fix concurrently).

**OPEN:** #1515 attribution floor · #1517 govcheck dark gate · #1518 ADR-0634 review model ·
#1519 cache-canary false green · (#1516 not mine).

**THE CORRECTION THAT MATTERS MOST.** I twice reported merge state from a PRESENT-TENSE rollup
query. That is wrong by construction — it reports what checks say NOW, not at the merge instant.
#1507 did NOT "merge with 3 red required contexts": `oya-ci-required` was **PENDING** and concluded
red **33m45s after the commit was on trunk**. Policy was never CONSULTED (`enforce_admins: false`),
which is worse than a violation. Measured properly (join `completedAt < mergedAt` on the PR's own
head): **2 of the last 30 merged PRs (6.7%) merged with the required context observed green.**
Memory: `merges-do-not-wait-for-the-required-context`. **Always join on completedAt < mergedAt.**

Corollary: the concurrency fix makes verdicts EXIST; it does not make anyone WAIT for them. Two
independent defects, one fixed. The second is ADR-0634 D8 (#1518).

**REORG STATE, CORRECTED.** All ten `specs/reorg/*.json` plans are FULLY EXECUTED — zero `old_path`
sources survive, including `os-move-plan` (41 crates). The note calling it "the single active plan"
is STALE. The reorg is blocked on **plan AUTHORSHIP**, not execution. Moves are per-CAPABILITY, not
per-crate (#1498 moved 56 in one PR), so ~6-10 moves remain, not 351.

Remaining 351: `libs` 129 flat · `oya/intelligence` **78** · `cloud/cloud-kernel` 21 · `tools` 21 ·
`oya/office` 19 · `oya/community` 14 · `oya/application` 8 · long tail of 1-6.

**Why moves must be serial, now with numbers:** #1498's move touched **117 files outside the moved
trees** — root `Cargo.toml`, `Cargo.lock`, three global policy JSONs, and ~50 files in the
`dev-cli` hub. Every move hits the same globals.

**IN FLIGHT:** workflow `wf_d6a52eb6-4b9` — `cloud/cloud-kernel` → `kernel/` (ADR-0562 §8 Fork 2,
Accepted). Chosen because disposition is done, fan-out is ZERO, and its blocker (#1513) merged today.
**NEXT:** `oya/intelligence` (78) → `cloud-intelligence`; destination decided, no plan authored.

**FOUNDER DISPOSITIONS OUTSTANDING:** (1) branch protection / ADR-0634 D8; (2) the friction ledger —
gate is fine, corpus is dead (last row 2026-06-21, 498 commits ago, 119/189 open). Automate intake,
or demote to provenance archive. It is being treated as the former while behaving as the latter.

---

## 0. RESUME HERE — state at 2026-08-02 (workflow landed, trunk repaired) — SUPERSEDED by 0-LATEST

**`dev` WAS RED AND IS THE THING TO WATCH.** Tip `11a59590c` (#1507) failed
`corpus-index-coverage` + `lifecycle-status`, which blocked every open PR. Repaired on branch
`fix/trunk-corpus-coverage-and-lifecycle` (worktree
`scratchpad/wt-devctl`) — **verify the full `//ci/...` run finished green before raising it.**

**OPEN PRs (all raised this session, all based on the RED dev tip — rebase after the trunk fix lands):**

| PR | lane | state |
|---|---|---|
| #1509 | postsubmit concurrency (ONE token, `ref`→`sha`) | red only on inherited dev breakage |
| #1510 | ADR-**0633** enforcement-layer doctrine (renumbered, 0632 was taken by #1507) | — |
| #1511 | 80 dark quality lanes + gated hook mirror | red only on inherited dev breakage |
| #1512 | Python BUCK generators retired | `affected-set` GREEN (cone missed the broken gates) |
| #1513 | explicit `--find-renames` (unblocks every capability move) | — |

**THE TWO DEFECTS THAT COMPOUNDED, and this is the session's headline.** In one 22-second window:
`66ebd511c` green → `31e962cb6` **CANCELLED WITHOUT RUNNING A SINGLE JOB** → `11a59590c` red.
1. **#1507 merged with THREE required contexts in FAILURE** — nothing blocks a red merge (task #10,
   now evidence rather than theory).
2. **The merge burst cancelled the evidence** that would have caught it (#1509 fixes this).
   Nothing blocks a red merge AND nothing observes the result.

**THE NEW FAILURE CLASS worth carrying forward:** a new BUCK file is a **census event**, not a
build-file edit. #1507's six `filegroup`-only BUCK files re-partitioned 4541 YAML files (`oya/`
absorbed 4082) because buck2 assigns every file to its NEAREST ANCESTOR package. Coverage did not
move — files went from *outside the graph* to *inside a package that does not index them*. Memory:
`new-buck-file-repartitions-the-corpus`. Its sharper half: the gate's anti-vacuity floor was
`min_expected_yaml_files / 2`, a **derived** floor silently encoding "most YAML is unpackaged"; when
that expired it fired on real progress. A floor guarding an assumption belongs in the policy as its
own reviewable number.

**FOUNDER DECISIONS OUTSTANDING:** #10 branch protection — but see task #18, my earlier
recommendation to enable required reviews was WRONG and would make things worse.

**THREE DISCIPLINES** now recorded as complementary, not overlapping — Gajae = OPERATIONAL, Bun =
MIGRATION, it-legal = DRAFTING (see the memory `drafting-procedure-obligation-to-executable-spec`).
The drafting one is the least applied: **write the acceptance test inline, never a description of
one.** Prose specs are how this repo got 19 dark gates and a validator that validates only fixtures.



Supersedes `SESSION-HANDOFF-2026-07-10.md` as the live handoff. Operational scratch (`.omc/` is
gitignored). Numbers here were measured, not estimated; where a number was later corrected the
correction is shown.

## 1. The trigger and what it exposed

PR #1473 was a **one-line diff** adding `os/OWNERS`. It exposed four independent defect classes:

1. `unjustified` + `unreachable` +1 each → `dev` red (a governance marker needs a hand-written row)
2. unmapped path → `Decision::Full` → 2179 targets instead of a 125-target cone
3. that FULL run **starved the self-hosted runner until the process died**, reporting nothing
4. concurrent jobs on that fleet race on a shared `~/.rustup`

None are about the file. Every one is a class.

## 2. PRs (all raised this session)

**MERGED 2026-08-01 06:46-06:47 (batch): #1474, #1475, #1476, #1477, #1478.**
Still OPEN: #1479 (conflicts with the merged #1477 — both add a FLOOR to `justification_ref` in
`build_registry`; ORDER IS LOAD-BEARING, the OWNERS floor must run BEFORE the reached=>justified
fallback so `reachable_from` already holds `owners-schema` when `.first()` reads it), #1480, #1481.


| PR | Fixes | Notes |
|---|---|---|
| #1474 | ADR-0562 §0 path-anchor reading rule | 175 dead anchors are the DESIGN; **0 dead LIVE** |
| #1475 | **KEYSTONE** — buck2 build/test env asymmetry | nothing merges without it, see §3 |
| #1476 | codemod merge-base SPOF (`None => false`) | one git failure wedges every CI leg |
| #1477 | OWNERS accounted BY CONSTRUCTION | registry 124→75 rows, **zero regression** |
| #1478 | unmapped path → FULL; + per-step timeouts | cone 125/2179 (5.7%) |
| #1479 | reachability implies justification | `unjustified` 16546→**11947** (−4599) |
| #1480 | cargo-lock merge driver silent data loss | depends on #1479 |
| #1481 | **restore tier enforcement to capability roots** | see §4; 15 of 24 capabilities now DECLARE a tier, projection deleted |

## 3. THE DEADLOCK — read this before splitting any CI fix

Two required legs (`kernel-purity`, `ADR census epoch receipt`) were red for the SAME root cause.
**A PR fixing only one of them can never merge** — it stays red on the other. The fixes can exist,
be correct, be verified, and be individually unmergeable. Exits: batch into one PR (done, #1475) or
admin override. With ~50 required legs and a single fan-in context, this recurs as the fleet degrades.

Root cause: buck2 **build** actions inherit the full ~70-var daemon env; buck2 **test** actions
sanitize to 8 (`BUCK2_DAEMON_UUID, BUCK_BUILD_ID, HOME, LOGNAME, PATH, PWD, TMPDIR, USER`) — **no
`RUSTUP_HOME`, no `CARGO_HOME`**. Remember this; it explains any "works in build, fails in test".

## 4. Biggest finding — the reorg LAUNDERED tier violations

`unclassified_roots` grew **3 → 27**, one entry per capability move. A crate under such a root gets
no tier, so **every dependency edge touching it skips the ADR-0245/0280 comparison**. Rule R6 fired
on undeclared roots and prescribed "declare the root" — declaring it in `unclassified_roots` silenced
R6 while leaving the skip intact. **A detector satisfied by disabling the thing it protects.**

- **681 of 901 crates (75.6%) tier-unenforced.** `service_roots` holds only `cloud`, `oya` — the
  LEGACY roots. Every new capability home is unenforced.
- **Real defects were laundered, not fixed**: the gate's own test recorded 3 cloud-kms → residency
  S-RANK-INVERSIONs as "burned down by move-19". The edges were never fixed — relocating one
  endpoint into an unenforced root removed them from the comparison. Same for move-21 saas-bench.
- First reclassified slice (audit, cell, network, observability, secrets — 49 crates) surfaced
  **9 pre-existing inversions**.

**Design landed on (verified safe AND already required):** capabilities DECLARE their tier in
`specs/capability-registry.json` (`tier` + `substrate_dag_position.stratum`). `tier_facets()` at
`layer-dependency-acyclicity/src/lib.rs:603` ALREADY reads those fields and the gate is currently RED
with 5× `TDA-CAPABILITY-TIER-UNRESOLVED` because they are missing. Nothing blocks the edit: no JSON
schema governs the file, no `deny_unknown_fields`, all readers parse untyped. Canonical-json governs
FORMATTING only (`sort_keys: false` — preserve hand-authored key order).

**Open for founder ruling:** some capabilities have NO defensible tier (`iam` spans S1+S3). Those
must be RED and non-baselineable, not invented.

## 5. The recurring failure mode — three instances, three detection channels

A fix reintroduced the defect class it was fixing, three times, each caught at a different stage:

| Fix | How it relapsed | Caught by |
|---|---|---|
| cargo-lock driver | exact-arity destructure; git passes `%O %A %B %P` → usage branch → `%A` never written | adversarial verifier |
| tier enforcement | R6c tested tier PRESENT not RANKABLE; `forward-declared` has no S-rank → 4 capabilities (80 crates) pass while comparing nothing | code review |
| scm-facts (#1475) | declared a generated face with no producer wired | CI |

**All three passed the checks upstream of where they were caught.** Read the diff and all three look
complete and well-argued. The lesson: a fix for a defect CLASS must be tested against the class, not
the instance — and authoring/verification must stay in separate lanes.

## 6. Bureaucracy audit — I was WRONG, correct the framing

I called the accounting apparatus ceremony. The trend data refutes it: aggregate tolerated
**68,750 → 47,830 over 33 days**; `total-accounting` alone shed 25,114; every gate FLAT or SHRINKING
from birth, none growing. That is a working ratchet.

The real defects are narrower: (a) six of nine shrinking baselines **flat for 22–39 days** — burn-down
stalled; (b) `gate-baseline.generated.json` (47,830 keys) de-committed 2026-07-10 → **21 days of zero
in-tree observability** (my call; fix is to emit counts as a METRIC, not re-commit the file);
(c) two baselines born in the last 4 days tolerate 99.8% and 22% of their own corpora.

**Build-graph coverage (settles the "just use buck2" idea):** 6005/18581 = 32.3% in-a-BUCK-package,
3510 = 18.9% declared-as-input — **two denominators 1.7× apart, never quote interchangeably**.
Residual 12576 (67.7%) is 89.7% markdown + policy JSON/YAML/TOML. **But Rust is 97.2% covered**
(2400/2470), so on SOURCE the registry buys almost nothing. Scope any computed rule accordingly.

## 6b. TEMPLATED DOC/CONFIG REPLICATION — measured 2026-08-01 on origin/dev

The corpus is dominated by generator-stamped templates, not distinct documents:

| family | copies | | family | copies |
|---|---:|---|---|---:|
| `ux-flow.md` | **179** | | `dpia.md` | 87 |
| `story.md` | **179** | | `hot-split.md` | 82 |
| `integration-test-plan.md` | **179** | | `cold-merge.md` | 82 |
| `handshake.md` | **179** | | `auto-rebalance.md` | 82 |
| `README.md` | 410 | | `IP-ADR-0339-Shared-IaC-Modules.md` | 82 |

**Four families at EXACTLY 179** = one generator, one template per service, emitted together.
~1,211 md files in 8 families. YAML similarly: `values.yaml` 220, `Chart.yaml` 214,
`kustomization.yaml` 147, `deployment.yaml` 126 — per-service Helm/k8s boilerplate, much of it INERT
(see [[gitops-declaration-wired-to-nothing]]). TOML is CLEAN: 901 of 1,008 are `Cargo.toml`.

**Requirement this places on the code-graph schema** (fold into the in-flight design): CONTENT-ADDRESSED
node identity gives duplication detection for free — N byte-identical files must converge to ONE node
carrying N paths, so template families surface without a separate near-duplicate detector.
Near-identical (template with the service name swapped) is the harder, more valuable case and may be
a follow-up rather than in the minimal schema.

**BOUNDARY, must be stated in the schema rationale:** the graph must NOT license
"unreferenced markdown ⇒ delete". A runbook is read by humans, not referenced by code, and today
67.7% of files are outside the build graph, so absence of a reference is the NORM, not evidence of
death. See [[absence-of-proxy-is-not-absence-of-thing]].

## 6c. LATE-SESSION FINDINGS (all measured 2026-08-01)

**BRANCH PROTECTION ENFORCES ALMOST NOTHING** (GitHub API, verified). `dev`: required contexts =
`['oya-ci-required']` ONLY · required approving reviews = **NONE** · strict (up-to-date) = **False** ·
enforce_admins = **False** · required_signatures = False. Only conversation-resolution is on. This
CONTRADICTS docs/AGENTS.md + CLAUDE.md `completion_gate` ("reviewer-agent APPROVE plus cloud-ci
green"). The completion gate is DARK WIRING at the branch-protection layer. Consequence: 12 PRs
merged this session, several with only the orchestrator's own verification as review.

**DEV WEDGED — a gate blocks its own repair.** #1485 (intelligence relocation) left
`foundry-settings-template-drift-adapter` pointing at a removed path. #1486 repairs it but fails on
`affected-set` step 9 "Materialize merge-base build + test baselines", which evaluates the MERGE-BASE
= broken dev. Exits: admin-merge #1486, or make the materializer tolerate a broken base when the
CANDIDATE repairs it. **ALSO A FALSE GREEN: #1485 merged GREEN and still broke dev** — the pre-merge
affected-set cone did not cover the stale pointers. **A relocation needs a WHOLE-TREE referrer sweep,
not a cone check.**

**`gh` IS NOT INSTALLED ON THE OWNED RUNNER.** `build-health: TRUSTED BASELINE REFUSED — could not
execute gh api ...: No such file or directory; the cold merge-base rebuild runs instead`. The
trusted-baseline fast path is DEAD and every run does a full cold rebuild. It fails SOFT, which is
why nobody noticed. Compounds with the 1.81-effective-runner shortfall.

**RUN OBSERVABILITY IS A SCHEMA + A SCHEMA-CHECKER. THERE IS NO RUN.** 2 schemas (1059 lines) + a
1669-line validator + 13 fixtures + a BLOCKING CI step (oya-ci-required.yml:416) — validating that
the fixtures match the schemas. Every producer path is PAPER. `infra_red` is "a string the validator
will accept and nothing will ever write". `no_verdict` / `verified_empty` / `fix-infra` /
`needs-human` / `permitted_next_action` do not exist in any form. A real typed producer exists
(`GateRunObservabilityPhase`, 7 variants, `oya-ci-controller-kernel`) but reads K8s Job state, only
ever emits `Accepted`, and no workflow runs it. **Copy the pattern from `ci/facade/build-cache-policy`**
— structured record in, typed verdict out, "never grepped from logs".

**CODE GRAPH — corrections to the earlier design.** NEITHER Kythe NOR Glean content-addresses node
IDENTITY (Kythe VName = 5 plain strings; Glean ids are opaque). Identity must be a NOMINAL tuple;
the content-address is a SHALLOW ATTRIBUTE (no child digests) — already ratified in ADR-0541 D1 and
already implemented in `governance/corpus/core`. **v1 MUST NOT add an aggregate/roll-up node** or it
becomes a Merkle tree where every edit touches the root. Only TWO edge kinds (`Contains`, `Refs`);
reachability is a QUERY, never a materialized edge — a materialized view goes stale, which is
precisely what `specs/reachability-registry.json` does today. **The corpus substrate EXISTS and is
buck2-wired; EDGES are the entire gap.**

**`governance/` DECLARES `owns_crates: false` BUT HOLDS 5 CRATES** (`governance/corpus/{core,
doc-parser,extract,work-area-rust-parser,work-area-tree-kernel}`). Registry already disagrees with
the tree. Resolve before moving more in: is `owns_crates` read by anything (R6b restricts
`unclassified_roots` to registry meta_directories), and if `governance/` legitimately owns crates the
registry needs an explicit AMENDMENT, not a silent widening.

**`libs/` SPLIT (founder-approved, fan-out measured, control-validated):** 9 crates fan-out>=3 -> a
SMALL `base/` · 15 fan-out 2 -> judgment · 87 fan-out 1 -> `<capability>/core/` · 74 fan-out 0 -> OFF
LIMITS pending liveness disposition. NOT a wholesale `libs/`->`base/` move: Google's `//base` is SMALL,
and ADR-0562 §10.27/10.28 homed libs crates into `messaging/core/`, not a shared root.
Within that batch: `CheckMode` is duplicated in 4 crates and NEVER USED ("4 copies of a type that
does nothing but test itself", ~120 lines, zero call sites) · **12 crates are already DEAD** (10
declared dev-cli deps with zero call sites, 2 with no caller anywhere; 3 ship `unimplemented!()`
bodies) · `oya-check-cost-budget` is NOT a gate — a runtime budget-ledger used by `intelligence/` and
`oya/application`; moving it to a gate root INVERTS product->ci. DELETE the dead weight BEFORE moving.
`*-coverage`/`*-discipline` are NAMING CONVENTIONS, not algorithm families — collapsing on the suffix
would be a wrong refactor.

## 6d. BRANCHES PUSHED AND HELD — all blocked behind #1486 unwedging dev

| branch | what |
|---|---|
| `fix/ci-active-artifact-relocation-20260801` | **#1486 — the wedge-breaker.** Needs admin-merge |
| `feat/typed-terminal-states` | 5 typed states + permitted-next-action; three-way separation proven on real jobs |
| `reorg/libs-fanout1-batch` | 56 check kernels -> `governance/check`; 4 dead `CheckMode` copies deleted |
| `corpus-yaml-graph-slice` | YAML artifact class into the build graph as a buck2 action |
| `reorg/l7-orphan-rs-disposition` | unblinds the ADR-0540 test-wiring generator: 0 -> 38 candidates |

**THE 70 UNWIRED `.rs` ARE MERGE LANDMINES, NOT INERT CODE.** ADR-0554's affected-set pack sets
`require_owner_patterns: ["**/*.rs"]`, so touching ANY of the 70 resolves to
`Decision::RefuseUnowned` and **the PR cannot merge** (proven end-to-end: a no-op edit to
`oya/itsm/tests/integration.rs` -> exit 2). The ADR-0540 paved-road generator was BLIND BY
CONSTRUCTION — (a) `if buck_text.contains("rust_test(") { continue; }` blanket-skipped every crate
whose UNIT test was wired but whose `tests/*.rs` were not, which is exactly the RefuseUnowned shape;
(b) one member with computed Starlark aborted the whole run with `Err`, hiding every other unwired
member, making `--check` repo-wide unusable. **The 67 `cloud/cloud-kernel` files are a SUBSET of
these 70**, so they are landmines too — which argues for WIRING over deleting unless the
superseded-by-Asterinas case is proven.

**THE UNIFYING PATTERN OF THE WHOLE SESSION:** every finding was a tool or gate reporting ZERO
because it COULD NOT SEE, not because there was nothing there. The wiring generator returned 0
candidates and an error. `aspirational-enforcement` matches nothing after a rename. `gh` is absent so
the baseline lookup fails soft to a cold rebuild. `unclassified_roots` was parsed and never read
while disabling tier enforcement for 681 crates. Run-observability validates only fixtures. In every
case the surface reading was "nothing to do" and the truth was "cannot look."

## 6e. ENFORCEMENT-LAYER AUDIT — measured 2026-08-02, the largest finding of the session

**THE BUILD GRAPH'S LAYERING PRIMITIVES ARE ENTIRELY UNUSED.**

    906   BUCK files
    2223  targets declaring visibility = ["PUBLIC"]
    1155  targets declaring visibility = []        (default-private tests/bins)
       0  occurrences of `within_view`             <-- repo-wide, ZERO

Meanwhile **16 gate crates TEXT-PARSE BUCK files** to re-derive layering facts the graph already
holds; only **4** use buck2 as a query engine. `ci/facade/facade-core-layering/src/lib.rs:9` states
the reasoning out loud — it correctly diagnoses that the manifest is not the graph, then moves ONE
STEP SIDEWAYS (parsing build files as text) instead of one step DOWN (declaring the constraint those
files already support). **That is the whole finding in one crate.**

**111 of 165 gate crates have ZERO merge authority** — and it is ONE cause applied 111 times, not
111 judgements. All 56 `governance/check/*` are pure kernels whose only repo-wide importer is
`marketplace/facade/dev-cli/BUCK`, and `dev-cli` appears in ZERO workflows (CLAUDE.md: CLI is "never
merge authority"). Independently verified: **56 crates, 5 have a `tests/` dir, exactly 1 reaches the
real repo** — 55 of 56 assert nothing about the tree while running green as unit tests over synthetic
fixtures. Plus 25 of 39 `libs/oya-governance-*` have no importer at all.
The earlier "19 dark gates" and this "111" are NESTED, different measurements: 19 = never runs at
all; 111 = runs but never reaches the tree.

Inventory: **165 gate crates, ~189k LOC** (ci/facade 54 @ 136,774 · governance/check 56 @ 24,556 ·
libs/oya-check-* 16 · libs/oya-governance-* 39).

### The doctrine that came out of it — OWNERSHIP, not "earliest layer"

"The earliest layer that can express it" was REJECTED as directionally right but mechanically
useless: *expressible* has no test. Replaced with two tests any reviewer can run:

  **T1 mutation coupling** — changing the fact requires editing an artifact the enforcer reads.
  **T2 non-emptiability** — no refactor that PRESERVES the fact can shrink the enforcer's input set.

T1 alone passes broken things: `facade-core-layering` passes T1 (an edge needs a BUCK edit, it reads
BUCK) but FAILS T2 (its scan is scoped `<cap>/facade/*/BUCK`, so relocating a capability empties the
probe while every violating edge survives). **That is exactly what emptied `aspirational-enforcement`.**
A build constraint passes both structurally: **the constraint lives in the same file as the thing it
constrains, so any move that carries the edge carries the rule. There is no probe to get wrong
because there is no probe.**

**The promotion gate** (the missing axis, from SWE@Google ch20): actionable with a mechanical fix ·
**no effective false positives** · correctness not style. So **ownership is the destination,
false-positive rate is the ticket**. A check that owns its fact but has false positives stays
ADVISORY IN ITS DESTINATION LAYER — it does not stay a gate.

**Naming rule, now testable:** a name may be a key **iff the tool that owns it FAILS LOUDLY on
rename**. Mechanical test: rename the thing and see whether anything goes red. If nothing does, the
key was never a key — it was a coincidence.

**Classification highlights:** `port-placement` and `caller-supplied-authorization` belong in the
TYPE SYSTEM, not a text scan (a `VerifiedPrincipal` newtype makes the caller-supplied-authz defect
UNREPRESENTABLE). `crate-name-prefix` / `crate-layer-suffix` / `package-manifest-hygiene` belong in a
FORMATTER — and the first two are name-keyed by construction.

## 7. Known-broken, not yet fixed

- **2 dark gates**: `crate-registration`, `planning-projection` — build targets, ZERO workflow refs
- **`resolve_reachability` uses `masterplan.contains(path)`** — naive substring; root path `OWNERS`
  matches any occurrence anywhere. Every short path over-matched.
- **`ci-repo-root-hygiene-gate` + `ci-generated-artifact-policy-gate` fail in ANY clean worktree**
  (need `scm-facts.generated.json` from the CI materialize step) → no local signal to any verifier
- **`libs/oya-crate-registrar-app`'s `to_canonical_json` sorts keys recursively**, contradicting the
  `sort_keys: false` policy → next `register_crate` reorders `specs/capability-registry.json` whole
- **Runner fleet is the binding constraint**: dev push run sat **40 min queued**; `maxRunners: 3`
- **70 `.rs` in neither build graph**: 67 under `cloud/cloud-kernel/**` (also a root Cargo exclude —
  invisible to BOTH graphs), 3 orphans. **1123 non-Rust** referenced by no BUCK target (794 `.cedar`,
  150 `.proto`, 148 `.tf`).
- `dto-authz-trust` matrix label claims "~92 instances"; file holds 65 (all-time max 73)

## 8. Held / next

- **G1** (merge-driver registration + detector) — its `.gitconfig` is genuinely unregistered and
  CORRECTLY red. Fix: register `tools/hooks/` as a **PREFIX** (like `specs/reorg/`) not two exact
  paths. Sequence AFTER #1477 merges — same file.
- **A1** (ci-tide move plan) — unblocked by #1479, uncommitted in worktree `wf_18a9e839-868-4`
- **Reorg execution** — resume only after #1477/#1479 and the tier work land; every capability move
  edits the reachability registry and would otherwise deepen the enforcement hole.

## 9. Conduct notes that paid off

- The canonical checkout is STALE (1598 dirty, `preserve/hermes-w1-dirty-20260630`). **Never grep
  it.** Use `git show origin/dev:<path>`. Verified untouched at session end.
- Use the repo's OWN author-side checker before pushing a new file:
  `buck2 run //ci/facade/artifact-inventory-registry:oya-cloud-ci-accounting-registry-app-bin -- --repo-root . --check-paths <path>`
  It exists (FRIC #1328) and is NOT wired into the required path. It predicted all 5 blocked files.
- `gh run view --log-failed` returns EMPTY when a runner dies; `gh api .../jobs/<id>` shows the step
  stuck `in_progress` with `conclusion: null`, and the check-run ANNOTATION carries the real verdict.
  Logs may 404 (BlobNotFound) or need `--allow-escape-sequences`.
