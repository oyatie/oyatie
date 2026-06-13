---
id: ADR-0554
title: "Binding buck2 coverage for the full workspace: affected-set lane with fail-closed full-run escalation"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-06-12
door: one-way
owner: council-architecture
supersedes: []
superseded_by: []
amended_by: []
depends_on: [ADR-0515, ADR-0548]
amends: []
related: [ADR-0083, ADR-0111, ADR-0363, ADR-0510, ADR-0539, ADR-0540, ADR-0544, ADR-0551]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W0
---

# ADR-0554: Binding buck2 coverage for the full workspace

## Status

**Proposed - 2026-06-12 (authored for founder sign-off; door: one-way).**

## Context

The single required context `oya-ci-required` (ADR-0515) had exactly one binding buck2 lane, and
it was scoped to `buck2 test //cloud/cloud-ci/...`. The affected-set driver
(`infra/ci/buck2-affected-gate.sh`) ran `continue-on-error: true` — advisory. Consequence: ANY
code outside `cloud/cloud-ci` (oya/*, libs/*, cloud/* services) could merge broken. This was the
pipeline's largest remaining false-green channel (FRIC-1781310000), and it is proven live, three
times:

1. **PR #651 head `cf16525158301ba27eb72c2d5399e0c9e4b826ed`** did not compile (E0433 ×3 in
   oya/identity server.rs) and its SCIM E2E failed — yet in run `27288019517` the
   `buck2 (hermetic build + affected gate tests)` lane reported **SUCCESS**. The fan-in went red
   only via the UNRELATED freshness gate (stale faces); the compile breakage channel itself was
   green.
2. **dev tip `d43d936d5` carries an E0428** (duplicate consts from a mechanical
   legacy-forge→GitHub brand rename) in `//oya/ci-webhook-gateway:oya-ci-webhook-gateway-tests`
   — merged green, fixed in-PR here.
3. **PR #657** (run `27252033863`): the advisory step printed `BUILD FAILED` on
   `//libs/oya-data-sql-adapter-sqlx:oya-data-sql-adapter-sqlx-unittest` (ring linker, also
   reproduced locally) — the job still concluded SUCCESS and the PR merged (FRIC-1781310100).

Founder doctrine bindings: failure is failure in the pipeline; the pipeline is the product
(ADR-0548); automate everything automatable — flagging/red-gating alone is insufficient: the
coverage derivation, the escalation, and the failure explanation must all be machine-produced
with zero manual escape hatches (founder directive 2026-06-12); new automation never ships as
shell (G011).

### Measured cost (decision data, 2026-06-12)

- Local (Apple Silicon dev machine, dev tip): cold `buck2 test //...` ≈ **4m35s** (3947 local
  actions, 0% cache); follow-up full run ≈ **5m45s** wall including test execution
  (Pass 889, 893 rust_test targets, 1855 root-cell targets).
- CI (ubuntu-latest, run `27394934894`): an 18-target affected cone took ≈ **6m34s**
  build+test inside the advisory step on a restored cache; derivation itself (owner + rdeps)
  completed in **< 1s**.
- Full `//...` on a CI runner is unmeasured-cold but bounded by the same action count; the W3
  shared content-addressed cache (NativeLink/CAS) direction and the stable per-dependency-set
  cache key (PR #659) amortize it further over time.

## Decision

Ship the **tiered** design (option c), evaluated against the alternatives below:

- **D1 — Binding affected-set lane on every PR.** A new bespoke gate lane
  `gate · affected-set (ADR-0554)` in `oya-ci-required.yml` (additive; fan-in `needs:` wired;
  registered per the gate-registration meta-test as a Buck target lane) derives the merge-base
  diff's target cone via `buck2 uquery owner()` (per-file, batched, `--json`) + `rdeps()` within
  the policy universe, then runs `buck2 build` AND `buck2 test` on it (build is load-bearing:
  a broken binary no test depends on must still fail). Precedent: Bazel target determination /
  bazel-diff (Tinder); Meta/Google affected-target CI. Rust-native: the engine is
  `cloud/cloud-ci/gates/oya-cloud-ci-affected-set-app` — the G011 Rust successor of the
  transitional `buck2-affected-gate.sh`, which stays untouched as the advisory speed path until
  its removal IP.
- **D2 — Fail-closed escalation IS the automation (zero manual escape hatches).** Escape-trigger
  path classes that the rdeps cone cannot model (`.buckconfig`, `.buckconfig.d/**`,
  `toolchains/**`, `third-party/**` incl. reindeer fixups, `**/*.bzl`, `**/*.bxl`,
  `rust-toolchain.toml`) escalate mechanically to the FULL workspace run; so do deleted
  graph-relevant files, unmappable package definitions, and EVERY derivation failure (git,
  owner-query, rdeps, empty closure). There is no skip path, no label, no allowlist, and no
  human decision anywhere in the lane. Verdict dominance is fixed in the engine:
  `RefuseUnowned > Full > Affected > NoGraphTargets` — an owner-required file with NO owning
  target FAILS the lane outright, because graph-invisible code is not made safe by running
  more targets.
- **D3 — Full-workspace tier at admission/integration.** On `merge_group`
  (ADR-0515 oya-ci-tide admission; inert until the queue is enabled), `push` to dev (every
  landing — strictly stronger than a cron schedule), and `workflow_dispatch`, the lane runs
  `--mode full`: `buck2 build //...` + `buck2 test //...`. This owns the seams PR-cone
  derivation cannot: merge skew between concurrently-green PRs and environment-class drift.
- **D4 — Born pack-shaped (ADR-0548 R0, explicit conformance).** The Rust kernel hardcodes no
  repo path and no oyatie string; all repo facts — escape classes, owner-required classes,
  universe, full-run patterns, cell roots, base ref — are DATA in `affected-set-policy.json`.
  Any buck2 repo adopts the lane by writing its own pack. The kernel fixes only the decision
  SEMANTICS (D2) — that contract is the engine, not a pack value.
- **D5 — Transparency contract.** Every changed file is printed with its mechanical
  classification (`FULL-TRIGGER`/`PACKAGE`/`OWNER`/`NO-GRAPH`), the decision with its reasons,
  the complete decided target list, and — on FAILURE — the exact reproduction command and the
  preserved target argfile path.

### Why the alternatives lost

- **(a) affected-set only:** the cone is sound for source edits but cannot model merge skew or
  graph-semantic escapes by itself; without the full tier those revert to false-green. Rejected
  as incomplete — (c) contains (a).
- **(b) full `//...` binding on every PR:** measured ≈ 4m35s–5m45s locally is affordable, but on
  shared CI runners it multiplies every PR round-trip and — decisively — it blocks EVERY PR on
  ANY pre-existing out-of-cone breakage (two live on dev today), converting a false-green
  channel into a global merge outage with no mechanical relation between blocker and PR. The
  tiered design keeps PR blocking attributable (your cone, your defect) and still runs the full
  workspace at every landing and at queue admission. Revisit binding-full-per-PR when W3
  warm-cache lands (cost) and dev is full-green (blast radius).

### False-negative analysis of the derivation (what escapes the rdeps cone)

| Escape class | Disposition |
|---|---|
| Starlark macros (`**/*.bzl`, `**/*.bxl`) | full-trigger → FULL |
| `.buckconfig`, `.buckconfig.d/**` | full-trigger → FULL |
| `toolchains/**` (incl. toolchain BUCK/bzl) | full-trigger → FULL |
| `third-party/**` (reindeer vendor tree + fixups) | full-trigger → FULL |
| `rust-toolchain.toml` (rustup shim graph input) | full-trigger → FULL |
| deleted source/manifest/BUCK (owner unresolvable at HEAD) | escalate → FULL |
| package file under no configured cell | escalate → FULL |
| owner()/rdeps()/git failure of any kind | escalate → FULL |
| owned non-source assets (`include_str!` srcs) | closed: owner() runs on EVERY existing file, no extension pre-filter (the shell driver's `grep -E '\.rs$|…'` pre-filter was this hole) |
| source file with no owning target | REFUSE (lane fails; FULL would not compile it either) |
| `Cargo.lock` | deliberately NOT a trigger: buck2 never reads it — its graph consumes the vendored `third-party/**` (a trigger); cargo lanes + the ADR-0539 freshness gate own lock hygiene |
| `Cargo.toml` manifests / `build.rs` | package-SIBLING class: no `owner()` BY DESIGN (buck2 never reads manifests — the lane's own first dogfood run refused on its own crate's manifest under an owner-required pack); they SEED the enclosing package (its rdeps cone), and a package-less manifest fails the seed query → FULL |
| unwired tests/examples/benches (live audit 2026-06-12: 100 of 2253 tracked `.rs` files have no owning target — the ADR-0540 `member_test_code_without_rust_test_target` debt class; those tests run NOWHERE today) | REFUSE on touch — a green required context over an edited test that never executes is the silent variant of the defect class; refusal is the fix-on-touch ratchet and the message names the wiring fix |
| buildfile edit (`BUCK`, `BUCK.v2`, `PACKAGE`) | FULL, ALWAYS (basename in `package_definition_basenames` + `**/PACKAGE` escape trigger). Blast radius is NOT bounded by the package's own rdeps: a BUCK edit can add/remove targets dependents resolve, a `BUCK.v2` SHADOWS the `BUCK` dependents load (buck2 default buildfile order `[BUCK.v2, BUCK]`, empirically verified), and a `PACKAGE` evaluates to `[]` (would otherwise look like a plain no-owner file). Reviewer-reproduced silent-PASS class F2; closed |
| `.buckconfig.local` | full-trigger → FULL. Read by buck2, committable; was missing from the trigger list (F2) |
| `cloud/cloud-kernel/**` owned sources (74 of 87 tracked `.rs`; `buck2 uquery //cloud/cloud-kernel/...` → 10 targets, present on dev since 2026-06-10) | ORDINARY owner-required files — seed their cone like any other. **CORRECTION:** an earlier pack `out_of_graph_roots: [cloud/cloud-kernel/**]` exemption was FACTUALLY FALSE (it claimed zero graph targets) and made an owned-kernel compile break PASS as NO-GRAPH-TARGETS — the exact cf16525 class. The exemption is DELETED; there is no path-prefix out-of-graph mechanism |
| `cloud/cloud-kernel/.../user-*-src/**` genuinely-unowned userspace sub-crates (13 sources; own `Cargo.toml` + own `rust-toolchain.toml`, not globbed by the parent BUCK, no buck2 target) | REFUSE on touch (the engine's ordinary unowned-owner-required handling) — never a silent PASS, never a hand exemption. The real residual coverage gap is owned by ledger row FRIC-1781310300 (queued: a dedicated kernel lane buckifies or builds them) |
| buck2 binary pin bump (`infra/ci/install-buck2.sh`) | accepted seam: CI env, not graph; a pin bump mints a new cache key (full rebuild) and the full tier covers it at landing |
| owner-query ERROR while an unowned source exists | escalates FULL (owner data unavailable; loudly logged) instead of refusing — the safest computable response |
| flakiness under full-parallel load | not a derivation seam; deflake-forward policy (observed once: `oya-cloud-os-init-app-unittest`, passes in isolation 155/155) |
| pre-existing workspace build breakage on the FULL tier | GRANDFATHERED by the build-health ratchet (D6): a target failing at BOTH the merge-base and head does not block (shrink-only). NOT a flag-day |
| a target that built at the merge-base but fails at head (FULL tier) | REGRESSION → BLOCK (build-health ratchet, D6); the merge-base baseline is built out-of-band from the merge-base checkout, so it cannot be laundered |
| empty/missing merge-base build-health baseline | REFUSE (fail-closed guard): an empty baseline would grandfather every head failure, so the lane fails rather than false-green |
| FULL-tier TEST failure (a buildable target whose test fails) | NOT yet ratcheted — declared next IP (FULL-tier test-health ratchet over a test report); the cone path already binds the changed code's tests, so this is a known scope boundary, not a silent hole |

### Deliverables (engine + pack files this ADR justifies)

- `cloud/cloud-ci/gates/oya-cloud-ci-affected-set-app/src/lib.rs` — pure decision kernel
  (classification, micro-glob, verdict dominance; no repo facts).
- `cloud/cloud-ci/gates/oya-cloud-ci-affected-set-app/src/main.rs` — composition root
  (git diff + buck2 uquery/build/test adapter; every uncertainty escalates; FULL tier runs the
  D6 build-health ratchet when given a merge-base baseline report).
- `cloud/cloud-ci/gates/oya-cloud-ci-affected-set-app/src/bin/oya-cloud-ci-build-health.rs` —
  the D6 build-health ratchet binary (compares a merge-base build-report against a head
  build-report; blocks regressions, grandfathers pre-existing build debt).
- `cloud/cloud-ci/gates/oya-cloud-ci-affected-set-app/affected-set-policy.json` — the oyatie
  policy pack (all repo facts as DATA).
- `cloud/cloud-ci/gates/oya-cloud-ci-affected-set-app/tests/affected_set.rs` — RED-class +
  fail-closed seam fixtures.
- `cloud/cloud-ci/gates/oya-cloud-ci-affected-set-app/BUCK` /
  `cloud/cloud-ci/gates/oya-cloud-ci-affected-set-app/Cargo.toml` — buck2-primary target
  wiring with cargo target parity (ADR-0540).

## Verification (RED/GREEN, data-under-test)

- **Engine fixtures** (`tests/affected_set.rs`, buck2-native):
  `red_class_cf16525_out_of_scope_source_lands_in_the_affected_seeds` pins the exact PR #651
  diff shape → the out-of-scope identity target MUST be a seed; escape/deletion/refusal/
  determinism seams each have a fixture; the shipped pack must parse and pair with the engine.
- **Live RED run** (recorded in FRIC-1781310000 evidence): with the dev-tip E0428 present,
  the lane's auto mode derives the webhook-gateway cone and FAILS; with the in-PR fix it
  passes. The deliberately-broken-out-of-scope-crate class now blocks.
- **Self-proof:** the PR shipping this lane runs it as a required context with the new crate
  AND the out-of-scope `oya/ci-webhook-gateway` fix inside its own derived cone.

## Consequences

- The cf16525 class is closed for every buck2-OWNED source: no owned code (anywhere, including
  `cloud/cloud-kernel`) can merge without its cone building and testing green under buck2. A
  genuinely-unowned owner-required source (e.g. a kernel userspace sub-crate buck2 does not
  model) REFUSES the merge until it is wired — it can never silently PASS, and there is no
  path-prefix exemption that could re-open the channel (the earlier `out_of_graph_roots`
  exemption was unsound and is deleted; FRIC-1781310300 owns the kernel-userspace coverage gap).
### D6 — Build-health ratchet on the FULL tier (round-3; the flag-day fix)

The FULL tier as first shipped did a hard `buck2 build //...`, so it hard-failed on ANY pre-existing
workspace build breakage — a **flag-day requirement**: the entire workspace had to compile before any
BUCK-touching PR (which escalates to FULL by D2) could merge. dev carries pre-existing build debt
(FRIC-1781310100 sqlx/ring linker, FRIC-1781310400 `oya/ci-controller` E0432, FRIC-1781310500
`oya-shared-backbone-grpc-generated-adapter` buildscript, and the transitive blake3 SIMD-cfg failure),
none caused by this change. A flag-day requirement violates the founder merge-base-ratchet doctrine
(block NEW debt, grandfather pre-existing — FRIC-1781112000 / #698).

The FULL tier on a `pull_request` is therefore a **build-health ratchet**, reusing the ADR-0551
merge-base frozen-baseline pattern:

1. The workflow materializes the **build-health baseline** OUT-OF-BAND: it checks the merge-base
   commit into a SEPARATE git worktree (structurally not the PR candidate tree — the #698 F1
   laundering lesson) and runs `buck2 build //... --keep-going --build-report`, capturing the SET of
   target labels that FAIL at the merge-base.
2. At the PR head it runs the same keep-going build into a head report.
3. `build_health_verdict(baseline_failures, head_failures)` (pure kernel) blocks ONLY on
   **regressions** — set-difference `head_failures − baseline_failures` (a target that built at the
   merge-base but fails at head, or a brand-new failing target). Targets failing at BOTH are
   **grandfathered** (shrink-only burn-down); targets that build at head but failed at the merge-base
   are **fixed** (informational).
4. Output prints regressions (block), pre-existing-red (grandfathered, with count), fixed, and the
   remediation. **Born-blocking for regressions, ratchet for pre-existing** — the exact firewall
   semantics, one layer up at the build level.

Soundness (re-checked against #698 F1): the baseline failure set comes ENTIRELY from the merge-base
build, fed via `--baseline-report`; nothing in the candidate tree feeds it, so a PR cannot launder a
regression by growing its own baseline. A fail-closed guard REFUSES an empty baseline report (which
would grandfather everything). The admission tier (`merge_group`/`push`/`dispatch`) keeps the HARD
full build — the integration tip must be green, no grandfathering. The PR cone path (auto mode,
hard-fail on a NEW break in the changed cone — the cf16525 fixture) is UNCHANGED. SCOPE: the ratchet
governs BUILD health (the cf16525 class is a compile break); a FULL-tier TEST-health ratchet (the same
baseline-diff over a test report) is the declared next IP — the cone path already binds the changed
code's tests.

This makes #702 itself mergeable: its FULL run grandfathers the 4 pre-existing reds → GREEN, while any
NEW build break (proven by the planted-`compile_error!` RED fixture) still blocks. The build-health
baseline is tracked by FRIC-1781350000, which carries the 4 known pre-existing breakages as the initial
grandfathered set (burn-down owned by the breakage-fix lanes).
- The existing `buck2` job's scoped `//cloud/cloud-ci/...` binding lane and the advisory shell
  step become redundant once this lane has soaked; their removal is a follow-up IP (one
  collision-surface edit at a time; this change is purely additive).
- The engine is a paved-road component (ADR-0548): `GatePolicy`-pack-shaped, reusable on any
  buck2 repo unchanged.

### D7 — Trusted-producer baseline-artifact emission + timeout rail (round-4, ship now)

Two changes ship now; a third (the cross-run CONSUMER) is deferred to D8.

**1. The admission tier emits the build-health baseline as a byproduct.** The admission/integration
FULL tier (`merge_group`/`push`/`workflow_dispatch`, `--mode full` with NO `--baseline-report`)
previously ran a hard `buck2 build //...` + `buck2 test //...` (`run_buck`). It now runs
`buck2 build //... --keep-going --build-report <stable path>`, derives the SAME hard verdict from
the report's **failure set being EMPTY** (any non-empty failure set = hard fail — the integration
tip MUST be green, **no grandfathering**, preserving the `run_buck` admission semantics exactly),
and STILL runs `buck2 test //...`. The report is a **pure byproduct**: merge authority is unchanged
(the verdict is identical to the prior hard build, derived from `failing_targets(report).is_empty()`
on the same `parse_build_report`/`failing_targets` kernel the D6 ratchet uses), and it is written to
a `RUNNER_TEMP`-anchored stable path (`build-health-admission-report.json`) the workflow can publish
without guessing a PID. On a trusted **push-to-dev** the workflow uploads it as artifact
`build-health-baseline-<github.sha>` (`retention-days: 90`, `if-no-files-found: error`), gated
`github.event_name == 'push' && github.ref == 'refs/heads/dev'` — **NOT** `merge_group`, **NOT**
`pull_request`. This keeps the artifact namespace clean of non-push (attacker-controllable)
producers, which is part of the deferred D8 consumer's defense. Precedent: ADR-0556 D5 QW-1
(same-run trusted-producer artifact) + the `postmerge-dev-trunk` warmth class
(`specs/cache-warmth-policy.json`) — post-merge dev CI is the canonical trusted populator (Bazel/Google
post-merge-fills-cache pattern), and trunk content is by definition admitted (passed `oya-ci-required`).
The dev SHA's admission report IS the **merge-base-to-be** for the next batch of PRs.

This producer is **sound + harmless**: no merge-authority change (same verdict), no new permissions
(stays under workflow-wide `contents: read`), and **zero laundering surface** because nothing
consumes the artifact yet. It is on the critical path of BOTH the deferred D8 cross-run consumer AND
the ADR-0560 warm-CAS bring-up — emitting the trusted baseline now is the prerequisite both reuse.

**2. The `timeout-minutes: 45` rail.** The `gate-affected-set` job runs the cold full workspace at
admission. A wedged buck2 action or a non-terminating compile would otherwise burn the runner
indefinitely (cold-rebuild runaway/exhaustion). The rail bounds it at **≈4x** the ADR-0554-measured
warm full run (4m35s cold / 5m45s incl. tests, lines 56-58) — it fires only on a genuine runaway,
never on a healthy cold build.

### D8 — Cross-run baseline consumption (DEFERRED, design-of-record, superseded-on-arrival by warm-CAS)

The D7 producer emits the trusted baseline; a future PR FULL tier could **download** the
merge-base's dev-pushed baseline cross-run instead of recomputing it in the cold out-of-band
worktree (the current D6 cold-worktree baseline, which STAYS as-is for PRs). D8 is **deferred** and
recorded here as design-of-record so a future implementer builds the HARDENED version, not the naive
one that reopens the #698 F1 laundering hole.

**(a) Why the naive cross-run download is both dead code AND a laundering hole.**
`actions/download-artifact@v8` cannot fetch a cross-run artifact by name — it only sees the current
run's artifacts unless given an explicit `run-id` + a token with `actions: read`. So the naive
"download `build-health-baseline-<merge_base>`" is **dead code** (resolves nothing). Worse, the
`gh api` artifacts-by-name listing spans **ALL runs including `pull_request`**: a malicious PR can
upload a forged `build-health-baseline-<merge_base>` artifact that a sibling consumer then ingests as
"the merge-base baseline," grandfathering a regression past the ratchet — the exact #698 F1
laundering class, reopened.

**(b) The HARDENED recipe (what D8 MUST do):**
- Grant `actions: read` **job-scoped** on the consuming job — NOT workflow-wide. Job-level
  `permissions` **REPLACE, not merge**, so the job block must **re-declare** `contents: read` too.
- Resolve candidates via
  `gh api repos/{owner}/{repo}/actions/artifacts?name=build-health-baseline-<merge_base>`.
- Apply the **3-CLAUSE TRUSTED-PRODUCER FILTER** to each candidate's `workflow_run`:
  `event == "push" && head_branch == "dev" && head_sha == <merge_base>`. **Reject everything else**,
  including `event == "pull_request"`.
- Download by **artifact ID** via `gh api .../artifacts/{id}/zip` (avoids `download-artifact`'s
  run-id plumbing entirely).
- **Cold fallback** when no qualifying artifact exists: recompute the baseline in the out-of-band
  merge-base worktree (the current D6 path), never false-green.

**(c) Anti-laundering soundness.** Trust flows from the artifact's `workflow_run` **PROVENANCE**
(`push`-write-gated, on the `dev` branch) — **never** the attacker-controllable artifact NAME. A PR
can name an artifact anything, but it cannot forge a `push`+`dev`+`<merge_base>` `workflow_run`
record, so the 3-clause filter rejects it. This is the same "trust the producer's provenance, not the
candidate's input" property as the D6 out-of-band merge-base baseline.

**(d) D8 ships WITH a conformance gate.** A sibling of
`cloud/cloud-ci/gates/oya-cloud-ci-cache-wiring-app/tests/cache_conformance.rs` must assert on the
workflow TEXT that (i) all 3 trusted-producer filter clauses, (ii) the `push && dev` producer gate,
and (iii) the job-scoped `actions: read` are all present — otherwise a future edit silently drops a
clause and reopens F1.

**(e) Cutover.** The entire D8 apparatus (download + filter + conformance gate) is **DELETED, not
reworked**, at warm-CAS bring-up: a content-addressed shared cache (ADR-0560) makes the merge-base
build a cache hit, so there is no artifact to download. D8 is therefore marked
**superseded-on-arrival by the warm-CAS consumer** — implement it only if warm-CAS slips and the
cross-run baseline is needed in the interim.
