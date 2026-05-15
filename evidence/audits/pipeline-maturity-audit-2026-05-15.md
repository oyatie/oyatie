# Pipeline maturity audit — 2026-05-15

## Overall verdict

**PARTIAL — with URGENT critical blocker.**

The substrate is heavily scaffolded (M-CC-P00 9/9 IPs marked complete; 14+ fitness-lane crates shipped; Oya VCS kernel crates present), but the *runtime* pipeline is not closed. Today the four-layer Branch-pipeline FINAL-FINAL model (worktree → local main → dev → staging) cannot run an agent from claim → deploy unsupervised because: (1) branch protection is NOT deployed on GitHub (`main` returns `Branch not protected` from the GitHub API — see Stage 6 below), (2) `grit` (the only sanctioned primitive) hangs under a 7.5 GB sqlite registry, (3) no unified `oya gate run-all` entry exists, (4) no automated reviewer-agent fires on PR open, (5) no staging deploy / canary / rollback automation exists, and (6) the mistakes-ledger lane named in memory has zero crate or workflow on disk.

Net: the architecture is mature on paper, the *enforcement surface* is partial, and several Layer-2/Layer-3 gates do not exist in any executable form.

---

## Per-stage maturity matrix

| Stage | Maturity | Evidence | Top blocker |
|---|---|---|---|
| 1 Claim (Layer 0) | **partial** | `grit` binary present at `/Users/jasonlee/.cargo/bin/grit`; sqlite registry at `.grit/registry.db` is **7.5 GB** (7681 MB); ADR-0054 documents icm-coordination-lock fallback as canonical at v0.3.0; no preflight runbook directory exists (`docs/runbooks/sanctioned-primitives/` is absent — repeat-mistake control #1 missing) | Replace grit at runtime (M-CC-P00 IP-005 ratchet shipped per IP status: complete, but agents in today's session still bypassed grit using plain `git` — the ratchet is not yet *enforced*) |
| 2 Work (Layer 0) | **scaffolded** | Per-agent worktree convention documented (`docs/runbooks/agentic-pipeline/grit-parallel-claim-demo.md`); no FS-level enforcement; `.grit/worktrees/` directory exists | Add claim-scoped FS sandbox (post-Oya-VCS); today agents can write outside their claim |
| 3 Verify | **partial** | `oya-foundry-fitness-*` crate family present: 19 fitness-app crates + 11 fitness-kernel crates under `tools/` and `crates/`; cargo lints DENY-clean (commit `09664f4` family); `cargo fmt/clippy/check/nextest` callable as binaries via `pr-tests.yml`. **NO `oya-dev-cli gate run-all` aggregate entry on disk** (only per-lane invocation in CI); no `oya gate` umbrella subcommand surfaced; mistakes-ledger lane (`oya-foundry-fitness-mistakes-ledger`) named in memory has **zero crates and zero workflows** on disk | Ship `oya gate run-all` aggregator + the missing `oya-foundry-fitness-mistakes-ledger-{kernel,app}` pair |
| 4 Done (Layer 1 → Layer 2 entry) | **partial** | `oya-foundry-vcs-cli-ratchet-kernel` present (IP-005); `oya-foundry-vcs-promotion-controller-kernel` present (IP-004); but **no top-level `oya` or `grit-compat` binary callable** by an agent today — kernels exist, the CLI binary surface does not. Today's session: PR #3 created via manual `gh pr create`, not via `grit done` | Land the actual `oya` CLI binary that wraps `claim/work/done/promote` (kernels exist; binary glue missing) |
| 5 PR review (Layer 2 gate 1) | **scaffolded** | `oya-foundry-vcs-review-mergequeue-kernel` exists (IP-007); **no auto-invocation on PR open** — no GitHub workflow that dispatches a reviewer-agent; no per-spectrum subagent wiring per `feedback_consensus_debate_spectrum_lens_subagents`. Today's PR #3 had reviewer manually dispatched | Build a `pr-review.yml` workflow that fans out a multispectrum subagent panel and posts APPROVE/REJECT as a required-check |
| 6 CI (Layer 2 gate 2) | **partial → URGENT** | `.github/branch-protection.yaml` *declares* 9 required checks (`cargo-fmt`, `cargo-check`, `cargo-clippy`, `cargo-nextest`, `oya-vcs-admission`, `oya-vcs-provider-execution`, `oya-foundry-fitness-supply-chain`, `-cohesion`, `-api-semver`). **Live GitHub API returns `Branch not protected` on `main` — the ruleset is NOT deployed.** Today's PR #3 CI rollup: `cargo fmt --check` FAILURE + `cargo clippy -D warnings` FAILURE on the most recent run; the PR can still merge because the ruleset isn't enforced server-side. Three cascading infrastructure failures in this session (broken action SHA → missing nextest profile → missing shebang) — none were caught pre-PR; no fitness lane fired retroactively | **URGENT: an admin must deploy the ruleset.** The configured-but-not-deployed gap is the single most expensive maturity hole today. |
| 7 Auto-merge (Layer 2 → Layer 3 entry) | **scaffolded** | GitHub repo metadata: `allow_auto_merge: false`, `delete_branch_on_merge: false`; no label/commit convention drives auto-merge; no merge-queue label pipeline configured | Flip `allow_auto_merge: true` + add a `merge-when-green` label workflow once branch protection is live |
| 8 Staging deploy (Layer 3) | **none** | `deploy/gitops/` directory exists but no canary controller, no cohort exposure logic, no rollback automation found at any depth ≤4; zero workflows for staging deploy; no `oya-*-staging-*` or `oya-*-canary-*` crates on disk | Layer 3 is a stub. Needs an entire IP (staging-deploy controller + canary cohort + rollback) — not yet scoped in M-CC-P00. |
| 9 Audit chain | **partial** | `oya-audit-chain-domain`, `oya-audit-chain-usecase`, `oya-audit-chain-file-adapter` crates exist; `registry/audit-chain/` present; **no evidence that every pipeline stage transition emits an audit event** today (PR-creation, CI start/finish, merge, deploy are not wired to the audit-chain); replay tooling not surfaced as a CLI subcommand | Wire stage-transition emitters into the actual CI workflows + add `oya audit replay` subcommand |
| 10 Repeat-mistake prevention | **scaffolded** | Memory `feedback_repeat_mistake_prevention.md` describes 5-control stack (preflight + ledger + lane + ICM + citation probe). Reality on disk: **0/5 of these controls are implemented in code.** No `docs/runbooks/sanctioned-primitives/preflight.md`, no `docs/templates/mistakes-ledger-row-template.md`, no `oya-foundry-fitness-mistakes-ledger-coverage` crate, no `oya-foundry-fitness-mistakes-ledger-app`. Today's 3 cascading CI failures are exactly the repeat-class signature the ledger is meant to capture — none triggered any automated control | **Highest-ROI gap.** Ship the ledger lane (kernel + app + template) — until then, every CI infrastructure regression is paid for with a fresh commit cycle. |

---

## Top 5 blockers preventing unsupervised pipeline runs

1. **Branch protection not deployed on GitHub.** `.github/branch-protection.yaml` declares 9 required checks; `gh api repos/.../branches/main/protection` returns 404 "Branch not protected". Layer 2's CI gate is *unenforced*. Fix: admin deploys the ruleset (one-time GitHub UI/API action). **Effort tier: trivial. Impact: critical.** **URGENT.**
2. **No `oya gate run-all` aggregator + no `oya` CLI binary on disk.** Kernels exist (`oya-foundry-vcs-cli-ratchet-kernel`, `-promotion-controller-kernel`, 30+ fitness-* crates) but no top-level `oya` binary the agent can invoke to (a) run every fitness lane locally before PR, (b) execute `claim/work/done/promote`. Today agents fall back to direct `git` + `gh`. Fix: scaffold `tools/oya-cli` thin wrapper crate that dispatches into the existing kernels. **Effort tier: 1 IP (≈1–2 days). Impact: very high — unblocks Stages 1, 3, 4 simultaneously.**
3. **Mistakes-ledger lane does not exist.** Memory directive `feedback_repeat_mistake_prevention.md` mandates 5 controls; **zero** are on disk. Today's session paid 3 commit cycles to debug 3 CI infrastructure regressions (broken action SHA, missing nextest profile, missing shebang) — exactly the repeat-class signature the ledger is meant to prevent. Fix: ship `oya-foundry-fitness-mistakes-ledger-{kernel,app}` + template + preflight runbook. **Effort tier: 1 IP. Impact: high — compounds with every future PR.**
4. **No reviewer-agent auto-dispatch on PR open.** `oya-foundry-vcs-review-mergequeue-kernel` exists per IP-007 but no `.github/workflows/pr-review.yml` fans out the multispectrum subagent panel. Today the reviewer was hand-dispatched via Skill tool. Fix: workflow + a `pr-review` adapter that calls the kernel + posts a Check Run with APPROVE/REJECT. **Effort tier: 1 IP. Impact: high — closes Layer 2 gate 1.**
5. **Grit registry is 7.5 GB and hangs; no preflight; agents are silently bypassing.** ADR-0054 says grit is the *sanctioned* primitive yet today's commit log shows agents using plain `git` (`git mv`, `git commit`, `gh pr create`) — i.e. the sanctioned-primitive contract is broken in reality. CLAUDE.md `sunset_note` says this transitions away on Oya-VCS go-live; today neither grit nor Oya VCS is the actual runtime. Fix: deploy the IP-005 ratchet binary AND add a `oya-foundry-fitness-banned-primitives` check that fails-fast when `git`/`gh` appears in a PR diff command-log. **Effort tier: 1 IP. Impact: high — closes the "is anyone actually using grit?" credibility hole.**

(Honorable mention #6: Layer 3 staging-deploy/canary/rollback is essentially un-built — needs its own M-CC-P01-class phase.)

---

## Maturity by layer

- **Layer 0 (worktree)** — *partial.* Per-agent worktree convention exists; grit fallback is documented; no FS sandbox enforcement.
- **Layer 1 (local main autonomous)** — *partial.* `grit done` kernel exists; the binary surface that an agent can call does not. In practice agents `git commit` directly.
- **Layer 2 (dev shared-world)** — *scaffolded with URGENT gap.* CI workflows exist; required-checks list exists; **branch protection ruleset not deployed**; reviewer-agent not auto-dispatched; auto-merge disabled. Two of three Layer-2 gates are unenforced.
- **Layer 3 (staging canary)** — *none.* No canary controller, no cohort logic, no rollback automation, no staging workflow. Layer 3 is a placeholder.

---

## Recommended next-3 IPs to drive maturity

1. **IP-MCC-P01-001 — `oya` CLI binary + `oya gate run-all` aggregator.** Thin wrapper crate `tools/oya-cli` dispatching to existing kernels (`oya-foundry-vcs-*-kernel`, all `oya-foundry-fitness-*-kernel`). Single entry point closes Stages 1/3/4 simultaneously and ends the silent `git`/`gh` bypass. **Highest ROI.**
2. **IP-MCC-P01-002 — Mistakes-ledger lane + preflight runbook.** Ship `oya-foundry-fitness-mistakes-ledger-{kernel,app}` + `docs/runbooks/sanctioned-primitives/preflight.md` + `docs/templates/mistakes-ledger-row-template.md` + wire into the new `oya gate run-all`. Implements the 5-control stack from `feedback_repeat_mistake_prevention.md` that is currently 0/5 on disk. Closes Stage 10.
3. **IP-MCC-P01-003 — PR-review automation + branch-protection deployment.** Author `.github/workflows/pr-review.yml` (multispectrum subagent fan-out → APPROVE/REJECT Check Run), deploy `.github/branch-protection.yaml` to live GitHub (admin action), flip `allow_auto_merge: true`. Closes Stages 5 + 6 + 7 in one phase. **URGENT sub-step (branch-protection deploy) can ship same-day independent of the workflow build.**

After these three, Stage 8 (staging deploy / canary / rollback) becomes the natural next milestone phase — but it is large enough to warrant its own M-CC-P02.

---

## Status

- Overall pipeline maturity: **PARTIAL** (substrate mature on paper; runtime enforcement partial; Layer 3 absent)
- **URGENT:** Branch-protection ruleset is *declared but not deployed* — `main` is currently unprotected on GitHub. Single highest-leverage fix.
