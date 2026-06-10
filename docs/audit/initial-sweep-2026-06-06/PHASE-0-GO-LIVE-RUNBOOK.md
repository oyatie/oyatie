# PHASE-0 FIREWALL — GO-LIVE RUNBOOK

> **Founder-paired. Needs GitHub admin for exactly ONE step (the ruleset flip).**
> This runbook is **READ-ONLY** on `/Users/jasonlee/Developer/source`. It mutates NOTHING in source
> and proposes NO live branch-protection change as part of authoring. The founder applies the one
> admin step (§2) and runs the prove-it-blocks acceptance test (§3).
>
> **Authoring provenance:** assembled from a read-only pass over the real source tree on 2026-06-07.
> Every file/line citation below was read this session. All "live GitHub" facts (the current required
> set, the `P0.0_RED` verdict) are sourced from the **checked-in baseline**
> `specs/phase0-ci-enforcement-baseline.json` (observed_at `2026-06-02T16:19:36Z`) — NOT re-queried
> against the live API this session. The founder MUST re-snapshot live state in §2 STEP A before flipping.

> **⚠ MEASURED LIVE STATE (re-queried via `gh`, 2026-06-07) — SUPERSEDES the stale snapshot below where they conflict.**
> The 2026-06-02 baseline is **out of date**. The authoritative live `dev` required set, read directly this session
> (`gh api repos/jason931225/oyatie/branches/dev/protection/required_status_checks`), is:
> ```
> { "contexts": ["github-lane-unlocker-required"], "strict": false }
> ```
> So the real BEFORE-state of the go-live flip is the **single** context `github-lane-unlocker-required` — NOT the
> `["cargo-fmt","cargo-check","cargo-clippy","cargo-nextest","oya-pr-review"]` set the 2026-06-02 snapshot recorded,
> and NOT `oya-ci-required` (required by nobody live, confirmed). Two consequences for §2:
> 1. **Replace-vs-add is a real founder decision:** flip to `["oya-ci-required"]` (replace the lane-unlocker) OR
>    `["github-lane-unlocker-required","oya-ci-required"]` (add alongside it). What `github-lane-unlocker-required`
>    actually gates must be understood before it is dropped — do not blindly replace it.
> 2. The §2.1 / §2-STEP-A / rollback before→after snippets that name the `cargo-*`+`oya-pr-review` set are
>    **stale**; the live revert target is `["github-lane-unlocker-required"]`. STEP A's re-snapshot remains
>    mandatory (it is the rollback anchor), but its EXPECT line is now `["github-lane-unlocker-required"]`.
> `gh` is authed as `jason931225` with `repo`+`workflow` scope (owner=admin), so the operator CAN drive both the
> CI-wiring (§1) and the ruleset flip (§2) via `gh` — but the flip stays **founder-authorized** (door:one-way).

---

## §0 — HONEST STATUS (read this first; no false "already enforced" claims)

The string `oya-ci-required` appears as a *target* in three checked-in places, and **all three self-disclaim**:

- `.github/branch-protection.yaml:55-56` lists `required_status_checks: [oya-ci-required]` for `dev` — but the
  file header (`branch-protection.yaml:1-5`) states it "is not Phase-0 exit authority until a trusted
  cloud-ci/oya-ci required context is live."
- `infra/branch-protection/dev.json:9-11` lists `"contexts": ["oya-ci-required"]` — header `_comment` (line 2):
  "not Phase-0 exit authority until a trusted cloud-ci/oya-ci producer is live and applied."
- `oya/ci-controller/crates/oya-ci-controller-kernel/src/lib.rs:472` declares
  `pub const GATE_CONTEXT: &str = "oya-ci-required";`.

**What is ACTUALLY enforced live today (the gap):** per the baseline
`specs/phase0-ci-enforcement-baseline.json` (`live_github_branch_protection`, observed `2026-06-02T16:19:36Z`),
the live `dev` required set is:

```
["cargo-fmt", "cargo-check", "cargo-clippy", "cargo-nextest", "oya-pr-review"]
```

`oya-ci-required` is required by **nobody live**. Worse, `oya-pr-review`'s producer returns **HTTP 501**
(not shipped — `dev.json:8`), so the live set is partly broken. The baseline's
`overall_verdict` (line ~213) is literally:

```
"P0.0_RED_blocked_until_cloud_ci_required_context_is_live"
```

**`[ENFORCED]` is, today, an empty set with respect to the firewall.** The 4 keystone gates are
**born-blocking in `cargo test`** (real RED on today's corpus — see §1.3) but **NOTHING reflects that
verdict into a GitHub merge decision**. This runbook is the step that flips those born-blocking-in-test
gates into a **live merge firewall**.

**The producer is SHADOW/non-blocking today.** The `GitHubCommitStatusPoster`
(`oya/ci-controller/crates/oya-ci-controller-github-adapter/src/lib.rs`) exists and is HTTP-proven via
httpmock (`tests/github_status_poster.rs`), posting `oya-ci-required` with the correct body+headers. But:
1. it is not yet run as a PR presubmit that posts on candidate SHAs, and
2. the gate Job it spawns runs `buck2 build/test` over trusted targets
   (`oya-ci-controller-k8s-adapter/src/lib.rs:108-122`) — it does **NOT** yet run the 4 new gate cargo
   tests + `registry-drift`. §1 states exactly what must change to make `oya-ci-required` reflect the
   4-gate verdict.

---

## §1 — CI-WIRING (autonomous; built in SHADOW; no admin needed)

**Goal:** a GitHub Actions workflow that, on every `dev` PR, builds + runs the 4 gate cargo tests +
`registry-drift` over the live tree, and reflects the aggregate pass/fail into the `oya-ci-required`
commit-status — so that flipping the ruleset in §2 turns a real verdict into a merge decision.

This is **SHADOW** until §2: the workflow posts the status and the job is visible in PR checks, but
GitHub does not yet *require* it, so a red `oya-ci-required` does not yet block merge. Building and
landing this workflow PR is **autonomous** — it is itself just a normal `dev` PR that the *current*
live required set (`cargo-*` + `oya-pr-review`) gates.

### 1.1 The four gate cargo-test entrypoints + registry-drift (the firewall payload)

Read this session — crate name → `cargo test -p <crate>` entrypoint (all live under
`cloud/cloud-ci/gates/`):

| Gate | Crate (`-p`) | Integration test | Born-blocking self-test (live corpus) |
|---|---|---|---|
| GATE-1 cross-artifact-agreement | `cross-artifact-agreement` | `tests/cross_artifact_agreement.rs::cross_artifact_fixtures_execute_red_green_cases` | `::gate1_is_born_blocking_on_the_live_corpus` |
| GATE-2 total-accounting | `total-accounting` | `tests/total_accounting.rs::total_accounting_fixtures_execute_red_green_cases` | `::gate2_is_born_blocking_on_the_live_corpus` |
| GATE-3 staleness-reaper | `staleness-reaper` | `tests/staleness_reaper.rs::staleness_reaper_fixtures_execute_red_green_cases` | `::gate3_is_born_blocking_on_the_live_corpus` |
| GATE-4 automation-ratchet | `automation-ratchet` | `tests/automation_ratchet.rs::automation_ratchet_fixtures_execute_red_green_cases` | `::gate4_is_born_blocking_on_the_live_corpus` |
| registry-drift | `registry-drift` | `tests/registry_drift.rs::committed_faces_equal_regenerated` | (n/a — drift is the assertion) |
| (producer dep) | `accounting-registry-producer` | (binary; invoked by the gate tests via `cargo run -p accounting-registry-producer -- --repo-root <root> --stdout [--face <name>]`) | — |

The born-blocking self-tests are the heart of the firewall — they assert each gate goes **RED on today's
real tree** (e.g. GATE-2 `total_accounting.rs` asserts `report.violations.contains("unowned")` with
`unowned > 1000`; GATE-1 asserts `generated_face_drift` + `dual_decision_collision` + `supersession_half_edge`).
Running `cargo test -p <crate>` runs **both** the fixture corpus and the live-corpus self-test.

### 1.2 The workflow YAML (matches existing conventions)

Conventions matched from the two existing workflows read this session:
- Toolchain pin: `dtolnay/rust-toolchain@21dc36fb71dd22e3317045c0c31a3f4249868b17` (the exact SHA used in
  both `backbone-microservices-ci.yml` and `docs-graph-drift.yml`), `toolchain: stable`.
- Cache: `Swatinem/rust-cache@9d47c6ad4b02e050fd481d890b2ea34778fd09d6` with a `shared-key`.
- `runs-on: ubuntu-latest`, `actions/checkout@v4`, `permissions: { contents: read, statuses: write }`,
  a `concurrency` group, `CARGO_TERM_COLOR: always`, `CARGO_INCREMENTAL: "0"`, an explicit `timeout-minutes`.

Author this as `.github/workflows/cloud-ci-firewall.yml` on the wiring PR:

```yaml
name: cloud-ci-firewall

# Phase-0 firewall payload: build + run the 4 keystone gate cargo tests + registry-drift
# over the live tree, then reflect the AGGREGATE verdict into the oya-ci-required commit
# status. SHADOW until the dev ruleset requires oya-ci-required (see
# docs/audit/initial-sweep-2026-06-06/PHASE-0-GO-LIVE-RUNBOOK.md §2). Posting here is a
# bridge for go-live proof; the K8s oya-ci-controller (GitHubCommitStatusPoster) remains
# the production producer of the same context.

on:
  pull_request:
    branches: [dev]
  push:
    branches: [dev]

# statuses:write is what lets this job POST a commit status for oya-ci-required.
# contents:read is the default-minimum for checkout.
permissions:
  contents: read
  statuses: write

concurrency:
  group: cloud-ci-firewall-${{ github.workflow }}-${{ github.head_ref || github.run_id }}
  cancel-in-progress: true

jobs:
  oya-ci-required:
    # IMPORTANT: the GitHub *check name* shown to branch protection is "<workflow> / <job-name-or-name>".
    # The commit STATUS we POST below (context: oya-ci-required) is the firewall's required context.
    # Branch protection in §2 requires the POSTED STATUS context "oya-ci-required", not this job's check.
    name: oya-ci-required
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4
        with:
          # Need the PR HEAD SHA (not the merge commit) so the posted status lands on the
          # candidate SHA that branch protection evaluates.
          ref: ${{ github.event.pull_request.head.sha || github.sha }}
          fetch-depth: 0  # gates use git log/commit dates (staleness-reaper, producer last-touch)

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@21dc36fb71dd22e3317045c0c31a3f4249868b17
        with:
          toolchain: stable

      - name: Cache cargo registry + target
        uses: Swatinem/rust-cache@9d47c6ad4b02e050fd481d890b2ea34778fd09d6
        with:
          shared-key: cloud-ci-firewall

      - name: Post pending oya-ci-required
        env:
          GH_TOKEN: ${{ github.token }}
          HEAD_SHA: ${{ github.event.pull_request.head.sha || github.sha }}
        run: |
          set -euo pipefail
          gh api --method POST "repos/${GITHUB_REPOSITORY}/statuses/${HEAD_SHA}" \
            -f state=pending \
            -f context=oya-ci-required \
            -f description="running trusted required gate (4 keystone gates + registry-drift)" \
            -f target_url="${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}"

      - name: Run the 4 keystone gates + registry-drift over the live tree
        id: gates
        env:
          CARGO_TERM_COLOR: always
          CARGO_INCREMENTAL: "0"
        run: |
          set -euo pipefail
          # cargo test -p <crate> runs BOTH the RED/GREEN fixture corpus AND the
          # born-blocking live-corpus self-test for each gate. registry-drift asserts
          # committed == regenerated for all four generated faces.
          cargo test \
            -p cross-artifact-agreement \
            -p total-accounting \
            -p staleness-reaper \
            -p automation-ratchet \
            -p registry-drift \
            -- --nocapture

      - name: Post terminal oya-ci-required
        if: always()
        env:
          GH_TOKEN: ${{ github.token }}
          HEAD_SHA: ${{ github.event.pull_request.head.sha || github.sha }}
          OUTCOME: ${{ steps.gates.outcome }}
        run: |
          set -euo pipefail
          if [ "${OUTCOME}" = "success" ]; then
            STATE=success
            DESC="oya-ci-required: all 4 keystone gates + registry-drift passed"
          else
            STATE=failure
            DESC="oya-ci-required: a keystone gate or registry-drift failed"
          fi
          gh api --method POST "repos/${GITHUB_REPOSITORY}/statuses/${HEAD_SHA}" \
            -f state="${STATE}" \
            -f context=oya-ci-required \
            -f description="${DESC}" \
            -f target_url="${GITHUB_SERVER_URL}/${GITHUB_REPOSITORY}/actions/runs/${GITHUB_RUN_ID}"
```

> **Note on the posted status vs. the K8s controller.** The production producer of `oya-ci-required` is the
> deployed `oya-ci-controller` (the `GitHubCommitStatusPoster`,
> `oya/ci-controller/crates/oya-ci-controller-github-adapter/src/lib.rs`). This workflow is the **Phase-0
> go-live bridge** that lets the founder prove RED/GREEN/tamper (§3) without first standing up the cluster.
> Both post the *same context* on the *same candidate SHA*; whichever is wired, the §2 ruleset is identical.

### 1.3 What MUST CHANGE in the SHADOW producer to make it reflect the gate verdict

The K8s controller producer is shadow today for two concrete, file-cited reasons. To make
`oya-ci-required` reflect the **4-gate verdict** (not just `buck2 build/test`), change:

1. **The gate Job command.** Today `oya-ci-controller-k8s-adapter/src/lib.rs:108-122` builds a `gate_cmd`
   that runs:
   ```
   buck2 uquery 'kind(".*test.*", //...)' | sort -u > /workspace/trusted-test-targets.txt
   ...
   xargs -a /workspace/trusted-build-targets.txt buck2 build
   xargs -a /workspace/trusted-test-targets.txt buck2 test
   ```
   The 4 gates are deliberately **off the buck2 build-graph** ("G-INTEGRITY track: no buck2 build-graph dep",
   per each gate `Cargo.toml`), so the buck2 test sweep does **not** include them. The command must be
   extended (or the controller's required-target inventory must add) the 4 gate cargo tests +
   `registry-drift`, so the verdict the controller posts is the 4-gate verdict. The token-isolation
   invariant (the runner Job must NOT receive `GITHUB_CI_TOKEN`, `lib.rs:167` + test `lib.rs:517`) stays
   unchanged.
2. **The presubmit trigger.** The controller must be invoked as a `dev` PR presubmit that posts on the
   candidate PR-head SHA. Until that is live in-cluster, the §1.2 GitHub Actions workflow is the bridge
   that posts the identical context — this is the autonomous, shadow-buildable path.

Tide alignment (if the merge bridge is in play): set
`OYA_TIDE_REQUIRED_STATUS_CONTEXT="oya-ci-required"` (tide kernel default is `"oya-ci-gate"`,
per PHASE-0-FIREWALL-PLAN §4.2 line 201/4.1) so the merge predicate reads the same context the producer posts.

---

## §2 — THE RULESET CHANGE (the ONLY GitHub-admin step) — FOUNDER-PAIRED

This is the single step that needs **GitHub admin** (Administration:write). It flips `dev` from gating the
legacy `cargo-*`+`oya-pr-review` set to gating `oya-ci-required`.

### 2.1 EXACT before → after of `dev` required_status_checks

**BEFORE (live, per baseline `specs/phase0-ci-enforcement-baseline.json` `live_github_branch_protection`,
observed 2026-06-02T16:19:36Z — RE-SNAPSHOT FIRST, see STEP A):**

```json
{ "strict": false,
  "contexts": ["cargo-fmt", "cargo-check", "cargo-clippy", "cargo-nextest", "oya-pr-review"] }
```

**AFTER (the checked-in target — `infra/branch-protection/dev.json:6-12`, mirrored in
`.github/branch-protection.yaml:55-56`):**

```json
{ "strict": false,
  "contexts": ["oya-ci-required"] }
```

- `strict: false` is intentional (per `dev.json` `_strict_rationale` + ADR-0124: breaks the O(N²) rebase
  cascade; the merge bridge enforces contexts without GitHub up-to-date enforcement). Keep it `false`.
- **Per-gate contexts:** none. The firewall posts ONE aggregate context (`oya-ci-required`) that is red
  unless all 4 gates + registry-drift pass. Do NOT add `total-accounting`/`cross-artifact-agreement`/etc.
  as separate required contexts — the checked-in target is exactly `["oya-ci-required"]`, and per-gate
  contexts would re-introduce the multi-context drift the baseline is trying to retire.
- `oya-pr-review` is **intentionally dropped** (its producer returns HTTP 501 — `dev.json:8`; requiring it
  deadlocks every PR). It returns only in Phase-2 once the reviewer-agent endpoint is live.

### 2.2 The exact commands the founder runs

There is a repo apply script — `scripts/branch-protection-apply.sh` — but note its **config default is
`infra/branch-protection/dev.json`** (the JSON), not the YAML, and it patches **only**
`required_status_checks` (it does not touch signatures/linear-history/etc.). It requires `gh`, `jq`,
`cargo`, and (in CI) a `GH_TOKEN` with **Administration** permission (`GITHUB_TOKEN` cannot request it —
script lines 67-70).

```bash
# All commands run from the source repo root, against jason931225/oyatie, branch dev.
# Auth: gh CLI logged in as a GitHub ADMIN of the repo (or GH_TOKEN with Administration:write).

# --- STEP A — RE-SNAPSHOT live state FIRST (rollback baseline + G0 drift-detect) ---
gh api repos/jason931225/oyatie/branches/dev/protection/required_status_checks \
  --jq '.contexts' | tee /tmp/dev-required-before.json
#   EXPECT (per baseline): ["cargo-fmt","cargo-check","cargo-clippy","cargo-nextest","oya-pr-review"]
#   If this is ALREADY ["oya-ci-required"] -> CP-AUTH-FLIP tripped: HALT, do not proceed (see §5).

# --- STEP B — APPLY the change (the one admin mutation) ---
# Preferred: the repo's own script (validates non-empty contexts, re-checks after apply).
scripts/branch-protection-apply.sh --apply --repo jason931225/oyatie --branch dev \
  --config infra/branch-protection/dev.json

# Equivalent raw gh api (use ONLY if the script is unavailable; PATCHes just the contexts):
gh api --method PATCH \
  repos/jason931225/oyatie/branches/dev/protection/required_status_checks \
  -F strict=false \
  -f 'contexts[]=oya-ci-required'

# --- STEP C — CONFIRM the live set is exactly the target ---
gh api repos/jason931225/oyatie/branches/dev/protection/required_status_checks \
  --jq '.contexts' | tee /tmp/dev-required-after.json
#   EXPECT: ["oya-ci-required"]
```

> The apply script also runs `cargo run -q -p oya-dev-cli -- gate validate protection-context-match`
> at the end (script lines 129-132). Per PHASE-0-FIREWALL-PLAN §4.2 (line 200), that `oya-dev-cli`
> dependency carries legacy-CLI authority and is slated to be replaced by the bespoke policy engine
> `evaluate_phase0_ci_policy`; for go-live the raw `gh api` PATCH + the §3 RED/GREEN proof is the
> authority, and the script's trailing validate is advisory only.

**This STEP B is the only action in this entire runbook that mutates live GitHub state and requires admin.**

---

## §3 — RED / GREEN / TAMPER GO-LIVE PROOF (the acceptance test) — FOUNDER-PAIRED

Go-live is **NOT** done when the ruleset is flipped. It is done only when a **RED PR is provably
un-mergeable**. Run all three on real PRs against `dev` after §2.

### 3.1 GREEN PR → `oya-ci-required=success` → mergeable

1. Open a trivially-correct PR against `dev` (e.g. a no-op doc line) where all 4 gates + registry-drift pass.
   > Caveat (honest): the gates are **born-blocking on today's corpus** (§1.1) — GATE-2 fires `unowned`,
   > GATE-1 fires `generated_face_drift`/`dual_decision_collision`/`supersession_half_edge`, etc. A truly
   > GREEN PR is only achievable once the Phase-1 amendment lanes have cleared those live exhibits, OR by
   > pointing this proof at the green RED/GREEN *fixture* path. For the go-live mechanics proof, the
   > load-bearing assertion is the RED PR in §3.2 (that GitHub refuses the merge). Record the GREEN result
   > honestly: if the live corpus is still red, state "GREEN-on-fixtures proven; GREEN-on-corpus pending
   > Phase-1."
2. Assert the status on the candidate SHA:
   ```bash
   SHA=$(gh pr view <PR> --json headRefOid --jq .headRefOid)
   gh api repos/jason931225/oyatie/commits/${SHA}/statuses \
     --jq '[.[] | select(.context=="oya-ci-required")][0] | {context, state}'
   #   EXPECT: {"context":"oya-ci-required","state":"success"}
   ```
3. Assert mergeability:
   ```bash
   gh pr view <PR> --json mergeable,mergeStateStatus --jq '{mergeable, mergeStateStatus}'
   #   EXPECT: mergeable == "MERGEABLE" (mergeStateStatus not BLOCKED on oya-ci-required)
   ```

### 3.2 RED PR (known-bad the gates actually catch) → `oya-ci-required=failure` → GitHub REFUSES merge

Pick a known-bad the gates demonstrably catch (these are the exact exhibits the self-tests assert):

- **Foundry-residue with no justification (GATE-2 `total-accounting`):** add a file under a path the
  producer classifies, with no OWNERS / no justification, so the registry row is `unowned`/`unjustified`
  and `total-accounting` returns `Verdict::Red` with `violations` containing `"unowned"`. (The live tree
  already trips this; an added `oya-foundry-*`-style residue file makes it an isolated, attributable RED.)
- **OR a half-supersession edge (GATE-1 `cross-artifact-agreement`):** add an ADR whose front-matter
  `supersedes: ADR-XXXX` while `ADR-XXXX` does not list it back → `supersession_half_edge` fires →
  `Verdict::Red`.
- **OR a hand-edited generated face (`registry-drift`):** hand-edit one byte of
  `cloud/cloud-ci/gates/accounting-registry-producer/accounting-registry.generated.json` (or any of the
  4 faces) → `committed_faces_equal_regenerated` fails with `REGISTRY DRIFT`.

Then:
```bash
SHA=$(gh pr view <RED_PR> --json headRefOid --jq .headRefOid)
gh api repos/jason931225/oyatie/commits/${SHA}/statuses \
  --jq '[.[] | select(.context=="oya-ci-required")][0] | {context, state}'
#   EXPECT: {"context":"oya-ci-required","state":"failure"}

gh pr view <RED_PR> --json mergeable,mergeStateStatus --jq '{mergeable, mergeStateStatus}'
#   EXPECT: mergeStateStatus == "BLOCKED"  (GitHub refuses the merge on the failing required context)

# And prove the merge button is dead: attempting to merge must FAIL.
gh pr merge <RED_PR> --squash || echo "MERGE CORRECTLY REFUSED (firewall live)"
#   EXPECT: gh pr merge returns non-zero; GitHub rejects because oya-ci-required != success.
```

**Go-live is done ONLY when the RED PR above is provably un-mergeable.** Capture the `state:failure`
status JSON + the refused-merge output as the go-live receipt.

### 3.3 TAMPER PR (edit a gate script in the PR) → STILL failure (trunk-sourced)

The gate definitions must come from **trusted trunk/controller state**, never the candidate PR tree
(`infra/branch-protection/dev.json` `_phase0_p0_0_note`; baseline `gate_definition_source`). Prove tamper
cannot self-pass:

1. In a PR, edit a gate's source/test to try to force GREEN (e.g. weaken an assertion in
   `cloud/cloud-ci/gates/total-accounting/tests/total_accounting.rs`, or neuter the producer).
2. The verdict-of-record must **still** be `oya-ci-required=failure`, because:
   - **Production controller path:** the K8s controller runs the gate from trusted trunk/controller state,
     not the candidate tree, and the runner Job has NO token (`oya-ci-controller-k8s-adapter/src/lib.rs:167`,
     test `lib.rs:517`) — a malicious PR cannot post its own `oya-ci-required` status.
   - **GitHub Actions bridge path (§1.2):** the firewall verdict is the AGGREGATE of all 5 cargo tests;
     a tamper that breaks any one (or that `registry-drift` catches as a hand-edited face) keeps the
     aggregate red. The `GITHUB_TOKEN` posting the status is the workflow's own token with `statuses:write`,
     scoped to that run — it is not exfiltratable into a forged "success" by editing PR code, because the
     POST happens in the trusted workflow steps, not in gate code.
3. Assert:
   ```bash
   SHA=$(gh pr view <TAMPER_PR> --json headRefOid --jq .headRefOid)
   gh api repos/jason931225/oyatie/commits/${SHA}/statuses \
     --jq '[.[] | select(.context=="oya-ci-required")][0].state'
   #   EXPECT: "failure"   (tamper cannot self-pass; trunk-sourced + token-isolated)
   ```

---

## §4 — HALT LINE: AUTONOMOUS vs FOUNDER-PAIRED (GitHub admin)

| Step | Lane | Needs GitHub admin? |
|---|---|---|
| §1 CI-wiring PR (`cloud-ci-firewall.yml`), built in SHADOW | **AUTONOMOUS** — a normal `dev` PR gated by the current live set | No |
| §1.3 SHADOW-producer changes (gate Job runs the 4 gate tests; presubmit trigger; tide context) | **AUTONOMOUS** (code/config PRs) | No |
| **§2 The ruleset flip** (`dev` required → `oya-ci-required`) | **FOUNDER-PAIRED — door:one-way + sign-off + CREDENTIALS** | **YES — Administration:write** |
| §3.1 GREEN-PR proof | **FOUNDER-PAIRED** (post-flip acceptance) | No (read-only `gh api`/`gh pr`) |
| §3.2 RED-PR prove-it-blocks (the real go-live gate) | **FOUNDER-PAIRED** (post-flip acceptance) | No (the refusal is GitHub's) |
| §3.3 TAMPER-PR proof | **FOUNDER-PAIRED** (post-flip acceptance) | No |

The autonomous lane can land everything in §1 **without enabling enforcement** (shadow posting only). The
firewall does NOT bite until the founder runs §2 STEP B. Per PHASE-0-FIREWALL-PLAN §6.3 gate #3, FE-1
producer go-live is **door:one-way + sign-off + CREDENTIALS** (the GitHub admin credential to apply branch
protection + snapshot live state). The founder also owns the §3 prove-it-blocks acceptance.

---

## §5 — CREDENTIAL DISCIPLINE + ROLLBACK TRIGGER

### 5.1 Credential discipline (`GITHUB_CI_TOKEN`)

- **Scope:** `statuses:write` only (post commit statuses for `oya-ci-required`). No `contents:write`,
  no `administration` on the producer token.
- **Holder:** the **controller (crier) ONLY**. Source-of-truth: `OpenBao` KV path
  `secret/oya/ci/github-ci-token`, field `token`, synced via ExternalSecret
  (`oya/ci-controller/iac/k8s/helm/templates/externalsecret.yaml`) into the controller Deployment env
  `GITHUB_CI_TOKEN` (`templates/deployment.yaml:92-96`).
- **NEVER on the runner.** The gate Job that runs untrusted PR code is explicitly denied the token:
  `oya/ci-controller/crates/oya-ci-controller-k8s-adapter/src/lib.rs:167` ("GITHUB_CI_TOKEN MUST NOT be
  injected here … a malicious PR could exfiltrate it and post arbitrary commit statuses"), enforced by the
  test at `lib.rs:517` (`env.iter().all(|var| var.name != "GITHUB_CI_TOKEN")`). **Do not relax this.**
- **The admin credential is separate.** The §2 ruleset flip needs an `Administration:write` credential
  (founder-held GitHub admin / `gh` admin login). That is a DIFFERENT, higher-privilege credential than
  the producer's `statuses:write` token and is used ONCE for the flip — never wired into any pod or runner.
- **GitHub Actions bridge token (§1.2):** the workflow posts via the per-run `${{ github.token }}` /
  `GITHUB_TOKEN` with workflow `permissions: { statuses: write }` — ephemeral, run-scoped, not a stored
  secret, not exfiltratable by PR code (the POST runs in trusted workflow steps).

### 5.2 Rollback trigger (if go-live wedges merges)

If flipping `oya-ci-required` live **wedges every PR** (e.g. the producer never posts, or posts only
`failure` because the live corpus is still born-red and Phase-1 hasn't cleared it), this is **CP-PRODUCER-RED
/ CP-AUTH-FLIP** (PHASE-0-FIREWALL-PLAN §6.4): **HARD HALT — back out, do not iterate live.**

**Exact revert of the ruleset** (restore the snapshot taken in §2 STEP A):

```bash
# Restore the EXACT pre-flip contexts captured in /tmp/dev-required-before.json.
gh api --method PATCH \
  repos/jason931225/oyatie/branches/dev/protection/required_status_checks \
  -F strict=false \
  -f 'contexts[]=cargo-fmt' \
  -f 'contexts[]=cargo-check' \
  -f 'contexts[]=cargo-clippy' \
  -f 'contexts[]=cargo-nextest' \
  -f 'contexts[]=oya-pr-review'

# Confirm the revert:
gh api repos/jason931225/oyatie/branches/dev/protection/required_status_checks --jq '.contexts'
#   EXPECT: the pre-flip set from /tmp/dev-required-before.json
```

> Use the **actual** captured `before` set, not a hardcoded list, in case live drift differed from the
> baseline. The revert is a single `required_status_checks` PATCH — it does not touch signatures, linear
> history, force-push/deletion rules, or the merge-queue config, so it is atomic and reversible.

Other locked checkpoints (HALT → backlog to founder, do NOT iterate):
- **CP-AUTH-FLIP / G0:** §2 STEP A shows `dev` is ALREADY `["oya-ci-required"]` before you flip → someone
  else flipped it → HALT, founder decides.
- **CP-GATE-SELFTEST-FAIL:** a gate's born-blocking self-test does NOT reproduce its live exhibit as RED →
  the gate is fake-green → it MUST NOT be relied on as blocking; do not declare go-live.
- **CP-BUCK2-LINUX:** the full build-correctness firewall (buck2 whole-graph green on the Linux runner) is
  a separate, still-open checkpoint; the 4 G-INTEGRITY gates ship regardless (no build-graph dep), but
  "build-correctness as a required context" stays gated on it.

---

## §6 — GO-LIVE CHECKLIST (the receipts that make Phase-0 §7 EXIT criterion #2 true)

- [ ] §1 `cloud-ci-firewall.yml` merged to `dev` (shadow posting `oya-ci-required` on PR-head SHAs).
- [ ] §1.3 SHADOW producer reflects the 4-gate verdict (gate Job runs the 4 gate tests + registry-drift;
      presubmit posts on candidate SHA; tide context aligned).
- [ ] §2 STEP A live `required_status_checks` snapshotted to `/tmp/dev-required-before.json`
      (CP-AUTH-FLIP check passed).
- [ ] §2 STEP B ruleset flipped → live set == `["oya-ci-required"]` (FOUNDER + admin; door:one-way sign-off).
- [ ] §3.1 GREEN proof recorded (`state:success` + mergeable; note fixtures-vs-corpus honestly).
- [ ] §3.2 RED-PR proof recorded (`state:failure` + `mergeStateStatus:BLOCKED` + refused `gh pr merge`).
- [ ] §3.3 TAMPER-PR proof recorded (`state:failure`; trunk-sourced + token-isolated).
- [ ] Exit-gate fixture `tc-0.12-current-red-p0-0-live-context-missing` flips from RED (baseline verdict
      no longer `P0.0_RED`).

**Phase-0 §7 EXIT criterion #2 is satisfied only when the RED PR is provably un-mergeable AND the recorded
`gh api …/required_status_checks == ["oya-ci-required"]`.**
