---
title: "ROBUSTNESS LANE — FALSE-ENFORCEMENT REGISTER (false / thin / flaky / advisory-shell)"
lane: justify-account-robustness / 10-robustness-enforcement
charter: D-DOCTRINE (d) ROBUST-NOT-FALSE + claim-ceiling #21
date: 2026-06-06
mode: READ-ONLY (no file edited; this is the only artifact written)
scope_repo: /Users/jasonlee/Developer/source  (the live enforcement surface)
extends: backlog-reconciliation/20-verify-ci.md, 20-verify-foundry-hygiene.md, synthesis/decision-record-oyatie-canon.md D-DOCTRINE
verdict: MULTIPLE confirmed false/thin/advisory-shell gates. The single most-required CI context (oya-ci-required) is posted by NO live producer; the gate that exists to PREVENT that exact silent-bypass would itself fail against the live config; three "required context" lists disagree; ~96 advisory/deferred/planned lanes present-as-enforcement in the human-readable mirror.
---

# 10 — ROBUSTNESS: FALSE-ENFORCEMENT REGISTER

**Charter test applied (D-DOCTRINE (d) + claim-ceiling #21):** for every claimed gate —
is it WIRED as a *blocking* context, ADVISORY, PLANNED, or UNWIRED? Is it proven by
RED/GREEN fixtures (does a known-bad input actually fail it)? Does any ADR/spec/doc make a
claim-ceiling word ("enforced / production-ready / hyperscaler-grade / isolated / retired /
done / complete / parity / full / automatic") without evidence?

**Citation discipline:** every load-bearing claim cites path + line + verbatim snippet,
read directly from the on-disk source tree. Coverage bounds stated in §0.

---

## §0 — COVERAGE + WHAT I DID NOT COVER (no silent caps)

**Covered (read in full or grepped exhaustively):**
- `docs/standards/ci-lanes.md` (156 lines, full) — the human-readable lane mirror.
- `registry/quality/lanes.yaml` (813 lines; status distribution + per-lane fields parsed programmatically).
- `infra/branch-protection/dev.json` (full) + `.github/branch-protection.yaml` (full).
- `.github/workflows/backbone-microservices-ci.yml` (618 lines, full) — the ONLY GHA workflow.
- `Jenkinsfile` (root, 36 lines, full) + `infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy` (143 lines, full) + `infra/ci/jenkins/reported-status-contexts.json` (full).
- `tools/governance/adr-0221-governance-gates.sh` (119 lines, full) — the 4 ADR-0221 bash gates.
- `libs/oya-check-protection-context-match/src/lib.rs` (kernel logic, lines 48-230) + its runner `oya/developer-sdk/.../protection_context_match_gate.rs` (sourcing logic, lines 1-237).
- `libs/oya-governance-bypass-kernel/src/lib.rs` (expiry logic + tests) + `registry/foundation-bypasses/byp_adr_0346_oya_verify_ci_mirror.yaml` (full).
- `libs/oya-governance-claim-ceiling-kernel/src/lib.rs` (116 lines, full) + `libs/oya-check-claim-ceiling/tests/claim_ceiling.rs` (test shape).
- `oya/developer-sdk/.../commands/gate/run_all.rs` (header + DEFERRED_GATES, lines 1-90) + the `AGGREGATED_VALIDATE_LANES` array in `libs/oya-governance-gate-catalog-domain/src/lib.rs` (109 entries).
- Governance-crate enumeration (59 dirs: 39 `libs/`, 17 `tools/`, plus kernels) + tree-shape census.
- `docs/machine-readable/catalog.json` (`axes_count` field).

**NOT covered / bounded — and why:**
1. I did **NOT** exhaustively read the body of all 59 `oya-governance-*` crates nor all 109 aggregator lanes' implementations — I verified *wiring* (where they are invoked) and spot-checked fixture presence (ADR-0221 ×4, claim-ceiling, bypass-kernel, protection-context-match). A per-crate RED/GREEN fixture audit of all 59 crates is the recommended follow-up; I report the wiring-level false-enforcement, which is the higher-order defect.
2. I did **NOT** query the live GitHub API for the *actually-applied* branch-protection ruleset (read-only, no network mutation; and the repo's own files disclaim that the checked-in config is applied — see FE-1). My claims about the *checked-in* required contexts are exact; claims about the *live GitHub* state rely on the repo's self-disclaimers.
3. The foundry residue recount here (4,110 files, fast exclusion set) differs from the sibling lane's 4,714 because I used a quicker exclusion set; I defer to `20-verify-foundry-hygiene.md`'s 4,714/36,210/780 census-of-record and do not re-derive it.
4. `prd-axis-coverage` / `diataxis-doc-class` "active vs not": I confirmed they are absent from `registry/quality/lanes.yaml` and present only in docs/specs/evidence — i.e. NOT registered lanes. I did not trace every doc mention.

---

## §1 — THE ENFORCEMENT TOPOLOGY (the load-bearing context)

The source repo is **mid-migration and runs on a self-hosted gate substrate that the repo's
own files say is not proven live**:

- **Git remote is a MIRROR.** `git remote -v` → `github-mirror  https://www.github.com/jason931225/oyatie`. GitHub is explicitly a mirror, not the authoritative gate host. Current branch: `feat/oya-ci-tide`.
- **The authoritative gate is meant to be Jenkins+Forgejo**, not GitHub Actions. `Jenkinsfile:1-4` (verbatim): *"Root CI orchestrator (ADR-0361) — the canonical repo-wide gate that **replaces the retired GitHub Actions workflows**. Jenkins reports its consolidated status contexts (oya-verify, oya-supply-chain, oya-pr-review) to the PR; branch protection requires those."*
- **Only ONE GitHub-Actions workflow file exists:** `.github/workflows/backbone-microservices-ci.yml`. Its jobs are named `backbone-microservices-ci-${{ matrix.microservice }}` (line 117) and `backbone-microservices-ci-governance-smoke` (line 303). The governance-smoke job runs exactly ONE governance gate: `./bin/oya gate validate cargo-prefix` (line 313). **No other governance lane runs in any executing CI.**
- **The 91 active lanes + the 109-entry `gate run-all` aggregator run only via `oyaCiLane(service:'repo')`** — a Jenkins shared-library call (`Jenkinsfile:18-19` → `oyaCiLane.groovy`). That farm's liveness is disclaimed by the repo itself (FE-1).

This topology is itself the root cause of most findings below: **the gate *logic* is largely
real and fixture-backed, but the gate *wiring* lands on a producer the repo says is not live,
and the human-readable mirror presents advisory/planned lanes as if they block.**

---

## §2 — FALSE-ENFORCEMENT REGISTER (per-claim)

Severity key: **P0** = claims-to-block-but-blocks-nothing (false-green / silent-bypass);
**P1** = blocking-logic-real-but-no-live-producer or self-contradicting config;
**P2** = advisory/planned presented as enforcement, or stale generated registry.

---

### FE-1 (P0/P1) — The sole required CI context `oya-ci-required` has NO live producer; branch-protection is a target, not an enforced state

- **What it claims:** `.github/branch-protection.yaml:55-56` and `infra/branch-protection/dev.json:9-11` both make `oya-ci-required` the ONE required status check that gates every PR merge to `dev` (the default branch). ci-lanes / the whole governance program ride on this being enforced.
- **Real state:** Both files **explicitly disclaim that this is live enforcement.**
  - `dev.json:2` (verbatim): *"Shadow/target branch-protection config for the dev branch. scripts/branch-protection-apply.sh consumes it as a local bridge artifact only; P0.0 records live drift and **this file is not Phase-0 exit authority until a trusted cloud-ci/oya-ci producer is live and applied.**"*
  - `branch-protection.yaml:2-5` (verbatim): *"As of the P0.0 gap packet, **live GitHub dev protection does not reflect this file exactly**; drift is recorded in specs/phase0-ci-enforcement-baseline.json. This file is not Phase-0 exit authority until a trusted cloud-ci/oya-ci required context is live."*
  - The producer of `oya-ci-required` is the Rust kernel `oya/ci-controller/crates/oya-ci-controller-kernel/src/lib.rs:471` (`pub const GATE_CONTEXT: &str = "oya-ci-required";`). Nothing wires this controller into an executing CI (grep for the controller in any `.yml`/`Jenkinsfile` returns only infra Helm/ArgoCD/netpol manifests — deployment scaffolding, not a running producer with proof).
- **Gap:** The single point of enforcement for the entire branch is a context that, by the repo's own admission, is posted by a not-yet-live producer against a branch-protection config that is not actually applied. This is **advisory-shell at the apex**: everything downstream claims "branch protection requires green checks," but the required check has no proven poster.
- **Claim-ceiling #21 hits:** any doc/ADR asserting CI is "enforced" / branch protection is "live" against this config violates the ceiling. The files themselves are honest (they disclaim) — the violation is in any *upstream* claim that treats P0.0 as done.
- **Fix-to-make-it-real:** (1) Stand up the `oya-ci-controller` producer and PROVE it posts `oya-ci-required` on a candidate SHA (capture a real check_run). (2) Apply the ruleset and snapshot the live GitHub `required_status_checks` JSON. (3) Feed that snapshot to `protection-context-match --live-required-contexts` so the gate asserts live==canonical (the runner already supports this; see FE-2). Until (1)-(3), no doc may claim CI enforcement is live.

---

### FE-2 (P0) — `protection-context-match` exists to kill silent-bypass, yet the LIVE config is exactly the silent-bypass it forbids; the gate would FAIL if run

- **What it claims:** Lane `oya-governance-protection-context-match` (`ci-lanes.md:72`, verbatim): *"every required-status-check context in .github/branch-protection.yaml is the `name:` field of some workflow job (prevents silent-bypass where GitHub waits forever for a context no workflow posts)."* The kernel doc-comment (`oya-check-protection-context-match/src/lib.rs:113-119`) restates this as the machine-checkable encoding of feedback_no_silent_regression.
- **Real state — the live config IS the forbidden pattern:**
  - Required contexts (branch-protection) = exactly `["oya-ci-required"]`.
  - The runner unions two producer sources (`protection_context_match_gate.rs:170-215`): the Jenkins `reported-status-contexts.json` job names AND `.github/workflows/*.yml` job names.
  - `reported-status-contexts.json:8-26` lists 17 contexts: `cargo-fmt … oya-verify … oya-governance-* … oya-pr-review`. **`oya-ci-required` is NOT among them.**
  - The only workflow's job names are `backbone-microservices-ci-*` and `backbone-microservices-ci-governance-smoke`. **`oya-ci-required` is NOT among them.**
  - The kernel does pure exact string-match with **no external-producer carve-out** (`lib.rs:146-152`: `if !job_name_set.contains(context.as_str()) { return Err(ContextMissingFromWorkflows{...}) }`).
  - ⇒ If this gate were actually run against the live files, it returns `ContextMissingFromWorkflows{ context: "oya-ci-required" }` — i.e. it would BLOCK. The fact that the repo sits in this state means the gate is **NOT being run in any executing CI** (consistent with FE-1: the Jenkins farm that runs it is not proven live, and the one GHA workflow does not invoke it).
- **Gap:** A gate whose entire purpose is "no required context without a producer" is itself a victim of "no producer that runs it." It is fixture-backed in unit tests (the kernel has `#[cfg(test)]` cases) but **unproven against the live config**, which currently violates its own invariant. This is the canonical false-green: the check exists, reads correct, and is not firing.
- **Fix-to-make-it-real:** Run `oya gate validate protection-context-match` in an executing presubmit (even the one live GHA workflow) on every PR; it will go RED today and force FE-1's resolution. Add a deliberate RED fixture to CI proving it blocks when `oya-ci-required` has no producer (the unit test exists; the *wired* RED proof does not).

---

### FE-3 (P1) — THREE disagreeing "required context" lists (config drift = faulty enforcement, the exact D-DOCTRINE symptom)

- **What it claims:** A single coherent set of required merge gates.
- **Real state — three lists, none equal:**
  1. **branch-protection** (`dev.json:9-11`, `branch-protection.yaml:55-56`): `[oya-ci-required]` — 1 context.
  2. **oyaCiLane.groovy** posts to Forgejo (`oyaCiLane.groovy:24-32`): 16 contexts — `cargo-fmt, cargo-check, cargo-clippy, cargo-nextest, cargo-deny, oya-verify, oya-vcs-admission, oya-vcs-provider-execution, oya-governance-supply-chain, oya-governance-cohesion, oya-governance-api-semver, oya-governance-honest-claims, oya-governance-aspirational-enforcement, oya-governance-banned-primitives, oya-governance-protection-context-match, oya-governance-dependency-seam`. **No `oya-ci-required`.**
  3. **reported-status-contexts.json:8-26**: 17 contexts = the groovy 16 **plus `oya-pr-review`**. **No `oya-ci-required`.**
  - Cross-doc contradiction: `Jenkinsfile:3` says branch protection requires *"oya-verify, oya-supply-chain, oya-pr-review"* — a FOURTH list, and `oya-supply-chain` is not the registry id (`oya-governance-supply-chain` is). And `branch-protection.yaml:48-51` says `oya-pr-review` is intentionally ABSENT and returns HTTP 501.
- **Gap:** The set that GitHub actually enforces (`oya-ci-required`) is produced by no one in lists 2/3; the sets that producers actually post (lists 2/3) are required by no one in list 1. **The producer/required intersection is empty.** This is precisely the "drift + contradiction = faulty process + enforcement" that D-DOCTRINE names as the disease (`decision-record:178`).
- **Fix-to-make-it-real:** Pick ONE source of truth for required contexts (the registry), GENERATE branch-protection.yaml + dev.json + reported-status-contexts.json + the groovy list from it (generated-not-hand-maintained, per D-DOCTRINE total-accounting), and make `protection-context-match` assert all four equal the generated set + the live GitHub snapshot.

---

### FE-4 (P2, EXTENDS confirmed exhibit) — ADR-0363 "Foundry eradicated" is still false; residue persists

- **What it claims:** `ADR-0363:35` (verbatim, per sibling lane): *"The Foundry name was eradicated (ADR-0362 + the #181–#184 cutover)…"* — a claim-ceiling #21 "retired/eradicated" word.
- **Real state:** Live residue persists. Sibling census-of-record (`20-verify-foundry-hygiene.md:60-65`): **4,714 files / 36,210 occurrences / 780 `oya-foundry-*` files** contain `foundry`. My quick recount (looser exclusions) = **4,110 files** — same order of magnitude, same disproof. The brand also leaks into Cedar principals (`oyatie.foundry.*`) and a live `docs/prds/foundry.md status: Accepted` vs `specs/microservices/foundry.json status: Retired` divergence (sibling §a). **ci-lanes.md itself still lists `foundry-eval-nightly` (line 118) and `axis-foundry` as a lane owner (lines 8, 63, 95) and `foundry-tool` as a wasm sandbox class (line 90)** — the retired brand is wired into the live lane catalog.
- **Gap:** An Accepted ADR asserts a completed eradication that the tree disproves by 4,700+ files; the enforcement catalog still references the dead brand. The `oya-governance-brand-residue` lane (`ci-lanes.md:47`) claims to be a "tautological brand transition check" but evidently does not block this residue (else the tree could not be in this state) — likely advisory or not run (same root as FE-2).
- **Fix-to-make-it-real:** Execute the sense-routed rename (sibling §a CC-1/L2.0), fix the ADR-0363 claim to past-tense-with-residue-count or "in progress," and make `oya-governance-brand-residue` a BLOCKING lane proven by a RED fixture (a file containing `foundry` outside carve-outs must fail it).

---

### FE-5 (P2, EXTENDS confirmed exhibit) — `prd-axis-coverage` + `diataxis-doc-class`: DEFINED, NOT ACTIVE (not even registered lanes)

- **What it claims:** D-DOCTRINE (`decision-record:181`) flags these as live evidence the fear is real. The charter calls them "defined-not-active" / "planned-not-blocking."
- **Real state — worse than advisory; they are NOT lanes at all:**
  - `prd-axis-coverage`: appears ONLY in `docs/DOC-CATALOG.md` and `docs/machine-readable/catalog.json`. **Absent from `registry/quality/lanes.yaml`** (grep returns nothing). It is a documented concept with no lane id, no `status`, no `check_command` — therefore cannot block.
  - `diataxis-doc-class`: appears in `evidence/debate/*.json`, `docs/architecture/transition-classification-2026-05-21.json`, and `libs/oya-governance-substance-bar/src/lib.rs` — but is **not a registered lane** in lanes.yaml. The Diátaxis folder-topology reorg is itself "NOT yet executed" (`decision-record:187`).
- **Gap:** Two governance concepts the doctrine treats as active enforcement are not even in the lane registry. Any claim that doc-axis/doc-class coverage is "enforced" is false (claim-ceiling #21).
- **Fix-to-make-it-real:** Either register them as lanes with a `check_command` + RED/GREEN fixtures and wire into the aggregator, OR demote every doc claim that implies they enforce. Total-accounting: a concept that is "worth enforcing" must be reachable from lanes.yaml or be archived.

---

### FE-6 (P2, REVISES confirmed exhibit) — "22 oya-governance-* crates unwired" is STALE; there are 59, and the wiring gap is real but differently shaped

- **What it claims (prior exhibit):** "22 `oya-governance-*` crates NOT wired into CI" (`decision-record:181`).
- **Real state:** There are **59** `oya-governance-*` crate dirs in the canonical tree (39 under `libs/`, 17 under `tools/`, plus kernels). They are NOT a separate `crates/oya-governance-*` set — the original "22" likely counted an earlier layout. The real wiring picture: all **91 active lanes carry a `check_command`** (verified: 0 active lanes lack one) and **109 lane entries are in `AGGREGATED_VALIDATE_LANES`** (`gate-catalog-domain/src/lib.rs`), so the lanes are *individually runnable* and *aggregated*. The gap is that the aggregator + lanes run only via the Jenkins farm (FE-1/FE-2), so "wired into the aggregator" ≠ "runs in an executing CI."
- **Gap:** The exhibit's *number* (22) is stale and should be corrected to 59-crate / 91-lane / 109-aggregated; the *substance* (governance lanes don't actually fire in executing CI) is CONFIRMED and is the FE-1/FE-2 producer-liveness gap, not a per-crate "unwired" gap.
- **Fix-to-make-it-real:** Correct the count in the doctrine to 59/91/109. Resolve FE-1 (live producer) so the aggregator actually runs. Then a per-crate RED/GREEN fixture audit (each of the 59 crates must have a test proving a known-bad input fails) is the remaining robustness proof.

---

### FE-7 (P1) — `axes_count: 6` in the generated catalog is STALE vs 7 (hand-edited-not-generated symptom)

- **What it claims:** `docs/machine-readable/catalog.json` is the machine-readable 7-product-axis model (`decision-record:187` D-DOCORG; catalog has a `validation_lane`).
- **Real state:** `catalog.json:12` (verbatim): `"axes_count": 6,` — stale vs the 7-axis model the doctrine asserts (`decision-record:181` names this exact drift). A field that should be *generated* from the axis list carries a hand-set number that disagrees with reality.
- **Gap:** Generated-not-hand-maintained (D-DOCTRINE total-accounting) is violated: the count is hand-set and drifted. If the catalog had a generator + a check that `axes_count == len(axes)`, this could not drift.
- **Fix-to-make-it-real:** Generate `axes_count` from the axis array; add a lane that fails when `axes_count != len(axes)`; regenerate. Same class as the ADR-INDEX/decisions.json "hand-edited-not-generated" exhibit (sibling §b: `decisions.json next_adr` STALE).

---

### FE-8 (P1) — Foundation-bypass `byp_adr_0346_oya_verify_ci_mirror` is EXPIRED (2 days) — and it bypasses the very CI-mirror that would make local==CI real

- **What it claims:** `byp_adr_0346_oya_verify_ci_mirror.yaml:6` (verbatim rationale): *"…opens a time-boxed migration bypass until oya verify --ci-required mirrors cargo fmt + cargo check + cargo clippy + cargo nextest + **oya gate run-all** and blocks on exit-0 of each mandatory step."* `regression_window_days: 14`, `created_at_epoch_days: 20594`.
- **Real state:** Expiry = 20594 + 14 = **20608**. Today (2026-06-06) = epoch-day **20610**. The bypass is **EXPIRED by 2 days.** It bypasses the requirement that `oya verify --ci-required` actually mirror + block on `oya gate run-all` — i.e. the bypass is what lets local verification NOT yet equal the full gate set.
- **Robust part (give credit):** The bypass-kernel logic IS robust and fixture-backed. `libs/oya-governance-bypass-kernel/src/lib.rs:43-44` returns a Block on `expired_bypass_count > 0`, and tests prove it: `blocks_on_expired_entries_in_block_phase` (line 87) + `warns_when_over_allowed_in_warn_phase` (line 96). So *if* the foundation-bypass lane runs in block-phase, this expired entry WOULD block today.
- **Gap:** Same as FE-1/FE-2 — the lane that would catch the expired bypass runs only on the not-proven-live farm. So an expired bypass sits unflagged, silently still permitting the `oya verify`↔`gate run-all` gap. Until the producer is live, the expiry enforcement is advisory-in-effect despite robust logic.
- **Fix-to-make-it-real:** Run `oya gate validate foundation-bypass` in the one executing presubmit; it should go RED on the expired entry today. Either renew with justification or close the bypass by making `oya verify --ci-required` actually invoke + block on `oya gate run-all` (the bypass's own exit condition).

---

### FE-9 (P0) — D-PURESPLIT "ERADICATE everything else" is violated by FOUR coexisting service trees in the live tree

- **What it claims:** D-PURESPLIT (`decision-record:172`, RULED, door:one-way, verbatim): *"Pure split: exactly two service trees — `oya/` (products) + `cloud/` (platform) — **ERADICATE everything else** (services/, platforms/, microservices/, stray service-shaped trees)."* "retired/eradicate" = claim-ceiling words.
- **Real state — the tree census disproves "eradicated":**
  - `oya/`: 88 top-level dirs ✓ (intended)
  - `cloud/`: 26 ✓ (intended)
  - `libs/`: 169, `tools/`: 32, `crates/`: 3 — shared-code trees (arguably allowed, but `crates/` coexisting with the `oya/`-internal crate layout, e.g. `oya/developer-sdk/crates/`, is a split-brain).
  - **`services/`: 6 top-level dirs — STILL PRESENT** (D-PURESPLIT says eradicate).
  - **`platforms/`: 1 dir — STILL PRESENT** (D-PURESPLIT says eradicate).
  - `microservices/`: 0 top-level dirs (the glob matched 0 under maxdepth-1, but the lane catalog + workflow still reference `microservices/<ms>/...` paths extensively — `backbone-microservices-ci.yml:35-105`, `ci-lanes.md` lane `lean-a5`, etc.). So the tree is partly cut but the *enforcement surface still assumes microservices/*.
- **Gap:** A door:one-way "eradicate everything else" ruling is contradicted by live `services/` (6) + `platforms/` (1) + the `crates/`-vs-`oya/.../crates/` split-brain + workflow/lane paths still rooted at `microservices/`. There is no lane that BLOCKS a service dir existing outside `oya/`+`cloud/` (D-PURESPLIT's "enforced: a service dir exists ONLY under oya/ or cloud/" is asserted but I found no registered lane enforcing it).
- **Fix-to-make-it-real:** Add a BLOCKING lane "service-tree-purity" with a RED fixture (a dir under `services/`/`platforms/`/`microservices/` must fail) before declaring D-PURESPLIT done; migrate or archive the 6+1 residual dirs; collapse `crates/` into the pure-split layout. Until then, no doc may claim the pure split is complete (claim-ceiling #21 "complete").

---

### FE-10 (P2) — Naming collision: TWO different "claim-ceiling" gates; the ci-lanes one is real, the doctrine should disambiguate

- **What it claims:** `ci-lanes.md:21` lane `oya-governance-claim-ceiling` = *"prevent unshipped stability, security, and supply-chain claims above foundation evidence"* (ADR-0037). The charter's claim-ceiling #21 = the banned-word ceiling.
- **Real state:** There are TWO unrelated "claim-ceiling" implementations:
  1. `libs/oya-check-claim-ceiling` — the ci-lanes ADR-0037 one. **Fixture-backed and robust:** `tests/claim_ceiling.rs` has the RED test `foundation_claim_ceiling_blocks_unshipped_stability_security_and_supply_chain_claims` (line 29) + GREEN `accepts_preview_source_only_records` (line 9). This gate's LOGIC is genuine (give credit).
  2. `libs/oya-governance-claim-ceiling-kernel` — a DIFFERENT concept: an agent claim-chain-DEPTH ratchet (`lib.rs:3` "M02-P03-IP-002"; ADR-0054 autonomy ceiling). Compares `observed_claim_depth <= configured_ceiling`. Unrelated to the banned-word/maturity claim ceiling.
- **Gap:** Same name, two meanings — a Linus-taste "no special cases / good names" violation that will mislead any reader auditing "the claim-ceiling." Not false-enforcement per se (both are real), but a naming-robustness defect that lets a reader assume the wrong gate covers #21.
- **Fix-to-make-it-real:** Rename the depth-ratchet to `agent-claim-depth-ceiling`; reserve "claim-ceiling" for the maturity/banned-word gate. Confirm the maturity gate's banned-word list (the #21 vocabulary) is sourced from a single registry the gate reads (the kernel I read is the *depth* one; the #21 word-list source is in `libs/oya-check-claim-ceiling` / `oya-governance-gate-catalog-domain` and should be the single SoT).

---

## §3 — WHAT IS ACTUALLY ROBUST (credit where due, so the fix-list is honest)

- **The ADR-0221 ×4 bash gates are genuinely RED/GREEN fixture-backed.** `tools/governance/adr-0221-governance-gates.sh` (full read): each of `vacuous-green`, `orphan-citation`, `version-pin`, `buildability-line-count` builds a known-BAD fixture that MUST trigger the hook (`require_contains`) AND a CLEAN fixture that must NOT (`require_not_contains`) — e.g. lines 52-58 prove the vacuous-green hook fires on `assert!(true)` and stays silent on `assert_eq!(2+2,4)`. This is exactly the charter's "proven by RED/GREEN fixtures that it actually BLOCKS." The DEFECT is wiring (no executing CI invokes this script — grep shows only `registry/` + docs reference it), not logic.
- **The bypass-kernel, claim-ceiling (ADR-0037), and protection-context-match KERNELS have real blocking logic + unit fixtures.** Their failure mode is uniformly **FE-1: no live producer runs them.**
- **All 91 active lanes carry a `check_command`** and **109 are aggregated** — the catalog is internally consistent and individually executable.

**The unifying root cause:** the gate *logic* is largely real; the gate *enforcement* is advisory because the producer (Jenkins/Forgejo farm + `oya-ci-controller`) is not proven live, the branch-protection config is a disclaimed target, and the human-readable mirror presents advisory/planned lanes as blocking.

---

## §4 — CLAIM-CEILING #21 SWEEP (claims without evidence)

| Source | Claim-ceiling word | Evidence state | Verdict |
|---|---|---|---|
| `ADR-0363:35` | "Foundry was **eradicated**" | 4,714 residue files (FE-4) | FALSE |
| D-PURESPLIT `decision-record:172` | "**ERADICATE** everything else" (door:one-way) | `services/`(6)+`platforms/`(1) live (FE-9) | NOT-YET (must not claim complete) |
| `Jenkinsfile:1` | GHA "**retired**" / Jenkins "**replaces**" | Jenkins farm liveness disclaimed (FE-1); 1 GHA workflow still live | OVERSTATED |
| `reported-status-contexts.json:5` | "**replacing** the retired .github/workflows job-name source" | workflow still present + still a producer source in the runner (FE-2) | OVERSTATED |
| ci-lanes §1.1 | "active lanes **block any merge**" | block depends on FE-1 producer | CONDITIONAL — not currently true |
| `design_spec_maturity_claims_gate.rs:9` | the one allowed "**hyperscaler-grade** design maturity" claim | bounded + gated by a real lane | ALLOWED (this is the correctly-fenced exception) |
| `catalog.json:12` | `axes_count: 6` | 7-axis model (FE-7) | STALE |

Any doc asserting CI is "enforced" / branch-protection is "live" / the pure split is
"complete" / Foundry is "retired" currently trips the ceiling without evidence.

---

## §5 — COUNT: CLAIMED-vs-ACTUALLY-BLOCKING GATES

- **Lanes claimed in the human-readable mirror (`ci-lanes.md`):** ~96 (foundation + per-PR + nightly + per-release tables).
- **Lanes registered active in `registry/quality/lanes.yaml`:** **91 active + 5 planned = 96.**
- **Lanes carrying advisory/deferred language in their own purpose text** (parsed from lanes.yaml + ci-lanes.md): **≥17 explicitly say "advisory" / "deferred" / "until … lands" / "flips to BLOCKER at …"** (e.g. otel-trace-propagation, audit-chain-seal, authz-tier, tenant-cost-labels, backup-retention, vector-store, olap-tier, wasm-runtime, iac-tier, a11y, i18n, compliance-evidence, realtime-transport, lean-a5 "flips to BLOCKER at M02-P22", lean-a-* "advisory until …"). These present in the catalog as enforcement but self-declare non-blocking.
- **Lanes wired into the 109-entry `gate run-all` aggregator:** **109** — runnable AND aggregated.
- **Lanes that actually BLOCK a merge in an executing CI today:** **effectively ZERO via the required context.** The only required branch-protection context (`oya-ci-required`) has no proven live producer (FE-1); the only executing GHA workflow runs ONE governance lane (`cargo-prefix`) on a narrow backbone path-filter and posts non-required context names (FE-2). The aggregator + the other 90 lanes block only on the not-proven-live Jenkins farm.

**Headline ratio:** ~96 lanes claimed-as-enforcement / 91 registered-active / 109 aggregated /
**0 proven-blocking-in-executing-CI** (pending FE-1 live producer). ≥17 self-declared advisory.

---

## §6 — FIX CAMPAIGN (ordered; feeds the unified amendment campaign, Task #22)

1. **FE-1 → resolve the apex:** stand up + PROVE the `oya-ci-required` producer; apply the ruleset; snapshot live GitHub required-checks. Nothing else is real until this is.
2. **FE-2/FE-3 → single SoT for required contexts**, generated into all four config artifacts + asserted by `protection-context-match --live-required-contexts`. Add a wired RED proof.
3. **FE-8 → run foundation-bypass in presubmit**; renew or close the expired ADR-0346 bypass by making `oya verify --ci-required` block on `oya gate run-all`.
4. **FE-9 → add a BLOCKING service-tree-purity lane** (RED fixture); migrate/archive `services/`(6)+`platforms/`(1); collapse `crates/` split-brain. Do not claim the split complete until green.
5. **FE-4 → execute sense-routed foundry rename; fix ADR-0363's "eradicated" claim; make `brand-residue` blocking** (RED fixture).
6. **FE-5 → register or demote `prd-axis-coverage` + `diataxis-doc-class`** (lane + fixtures, or drop the enforcement claim).
7. **FE-7 → generate `axes_count`; add count==len(axes) check.** Same generator discipline as the ADR-INDEX/decisions.json regenerate (sibling Wave-3).
8. **FE-6 → correct the doctrine count** 22 → 59 crates / 91 lanes / 109 aggregated; then per-crate RED/GREEN fixture audit.
9. **FE-10 → rename the agent-claim-depth ceiling**; reserve "claim-ceiling" for the #21 maturity gate.

**Cross-charter note:** every fix above is the D-DOCTRINE prescription applied — *generated-not-hand-maintained* (FE-3/FE-7), *every gate proven by RED/GREEN that it BLOCKS* (FE-2/FE-4/FE-8/FE-9), *no advisory-shell-claiming-enforced* (FE-5, §4), *total-accounting reachability* (FE-5/FE-9). The single biggest lever is FE-1: most "real logic, no enforcement" findings collapse once a live producer actually runs the already-built gate set and posts the required context.
