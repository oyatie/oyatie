---
id: ADR-0363
status: Superseded
planning_impact: true
deciders: council-architecture, founder
date: 2026-05-26
owner: council-architecture
supersedes: [ADR-0110, ADR-0112, ADR-0113]
superseded_by: [ADR-0701]
amended_by: [ADR-0510, ADR-0515]
amends: [ADR-0116]
related: [ADR-0053, ADR-0111, ADR-0116, ADR-0131, ADR-0173, ADR-0247, ADR-0248, ADR-0349, ADR-0357, ADR-0361, ADR-0362, ADR-0513]
related_specs: [/specs/gitops-vcs-replacement.json, /registry/vcs/changeset-event-log.json]
session_context:
  authored: 2026-05-26
  basis: "Founder deep-interview 2026-05-26 — 'absorb Foundry into Intelligence in a way that makes sense'; 'do we even need vcs? retire vcs, we have cloud-ci + git'; 'don't reinvent the wheel'; 'we'll use git as is — don't even oya git wrap'; 'we don't use github merge queue, we use selfhosted github'."
purpose: Retire the bespoke agentic-VCS layer (oya vcs CLI + oya git wrapper + changeset-state machine + merge-queue + webhook-receiver) in favour of plain git + GitHub (interim) PRs + Prow-shaped cloud-ci/oya-ci required contexts. The legacy cloud-ci bridge is bridge evidence only. Narrow the oya toolchain away from CLI authority; governance semantics move to Rust gate crates/cloud-ci contexts. Absorb Foundry's AI-agent platform into Intelligence; keep Governance as its own service.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0363: Retire bespoke agentic-VCS; Foundry→Intelligence; `oya` is a governance-gate engine

## Status

Accepted — 2026-05-26. Supersedes ADR-0110 (changeset state machine), ADR-0112 (webhook-driven invocation), ADR-0113 (vcs orchestrator end-to-end). Amends ADR-0116 (which parked external coordination tooling behind a manual-bootstrap seam — that seam is now removed, not deployed).

**Historical amendment recorded by ADR-0513 / platform-readiness 2026-06-02 (ADR-0513 is now Superseded):** this ADR still retires the bespoke agentic-VCS CLI/wrapper
and dormant changeset ratchet. Its former no-Tide-queue and cloud-ci-as-destination readings are no
longer current. Merge automation belongs in the Prow-shaped cloud-ci/oya-ci Tide component, and `oya` CLI entrypoints
are retirement/migration wrappers rather than CI authority.

**Current amendment — ADR-0515:** plain Git and protected GitHub PRs remain the live substrate;
`oya-ci-required` is the sole merge-status authority. ADR-0513 is retained only as nonbinding
history and does not amend this ADR's current meaning.

## Context

Two threads converged in the 2026-05-26 founder interview:

1. **The Foundry name was eradicated** (ADR-0362 + the #181–#184 cutover): the former `oya-foundry-*` crates were renamed across three namespaces — `oya-intelligence-*` (116, the AI-agent platform), `oya-governance-*` (39 + 2, the quality/fitness gates), `oya-vcs-*` (20, the agentic-VCS substrate). `microservices/foundry/` (597 files) was kept as a now-name-mismatched doc shell.

2. **The agentic-VCS substrate was questioned.** Evidence: of the 20 `oya-vcs-*` crates, only **2** are wired (the `oya-vcs-admission` + `oya-vcs-provider-execution` CI checks); ~13 (changeset-state machine, merge-queue, promotion-controller, webhook-receiver per ADR-0110/0112/0113) have **0–1 dependents** and were never deployed. The `oya vcs` CLI ratchet and the `oya git` wrapper duplicate plain git.

The founder's principle: **don't reinvent the wheel; use git as-is.** And the substrate is **GitHub (interim)** (per ADR-0173/0247/0248: vendor-lock-in avoidance + self-hosting doctrine; GitHub is the temporary bootstrap source-of-truth per ADR-0247, migrating to a self-hosted git-host µservice), **not** GitHub.

Best-practice research (2026-05-26, GitHub v15.0.2 / v11.x LTS) confirms GitHub natively provides — and is GPLv3+ (OSI-clean, not BSL/SSPL):
- Protected branches with **required status checks**, required reviews, dismiss-stale, restrict-push.
- **External-CI gating**: required-status-check contexts gate merges. Original bridge evidence used cloud-ci-posted GitHub commit statuses; ADR-0513/platform-readiness moves authority to Prow-shaped cloud-ci/oya-ci required contexts.
- **Webhooks** (PR opened/updated/merged, push) to drive automation.
- **Auto-merge** ("merge when checks succeed").
- **NOT** a GitHub-style speculative **merge queue** (open request github#5102, no milestone).

So `git + GitHub (interim) + branch-protection/required-checks/auto-merge` remains the self-hosted trunk-based-development substrate, with Prow-shaped cloud-ci/oya-ci as the current merge/CI authority target and the legacy CI bridge only as transitory transport. The bespoke changeset-state-machine / webhook-receiver / `oya vcs` / `oya git` re-implement substrate concerns that must be retired; merge-queue semantics move to Tide per ADR-0513 instead of this retired VCS layer.

## Decision

### 1. Change-coordination substrate = plain git + GitHub (interim) PRs + Prow/cloud-ci required contexts
Adopt the standard self-hosted substrate for coordination: plain `git`, GitHub PRs, branch protection, required status checks, webhooks, and auto-merge. Current merge/exit authority is Prow-shaped cloud-ci/oya-ci required context plus reviewer approval; the legacy CI bridge may post bridge statuses only until P0.0 cutover. GitHub remains the **bootstrap** host until the GitHub (interim) service is stood up (ADR-0247 post-bootstrap; tracked separately, NOT in this ADR's scope).

### 2. Retire the bespoke agentic-VCS layer
Retire — because plain git + GitHub + cloud-ci/Prow required contexts provide or own the necessary substrate:
- The `oya vcs` CLI ratchet (`claim`/`work`/`done`/`promote`).
- The `oya git` wrapper — **use plain `git` as-is** (no drop-in wrapper).
- The dormant orchestration crates: changeset-state machine, merge-queue, merge-queue-conflict, promotion-controller, webhook-receiver, ci-fix-loop-dispatcher, ast-index, polyglot-indexer, lockstore, changebundle (~13 crates, 0–1 dependents).
- ADR-0110/0112/0113 are superseded; `registry/vcs/changeset-event-log.json` + the event-router are frozen as historical evidence (not deleted, not active).

**Kept as semantics, not `oya` CLI authority:** the 2 wired CI checks (`oya-vcs-admission`, `oya-vcs-provider-execution`) are recast as Governance/cloud-ci Rust gate contexts. Legacy cloud-ci-posted GitHub statuses may mirror them during migration only; the permanent required producer is cloud-ci/oya-ci.

### 3. Merge queue: owned by cloud-ci/Tide, not the retired VCS ratchet
GitHub has no native merge queue, so the queue does **not** live in GitHub or the retired agentic-VCS substrate. ADR-0513 places merge automation in the Prow-shaped cloud-ci/oya-ci Tide component (`oya-ci-tide`), folding ADR-0111 projected-state merge semantics into CI/admission. The ADR-0363 decision is therefore narrowed: retire the bespoke VCS ratchet; do not defer merge-queue ownership.

### 4. `oya` CLI is retired from VCS/CI authority
Narrow the toolchain to its **differentiated core while removing CLI authority**:
- **Preserve semantics, not CLI authority:** oyatie-specific governance checks (honest-claims, claim-ceiling,
  aspirational-enforcement, cohesion, plane-class, data-class, banned-primitives, glossary-vocabulary,
  no-grouping, authority-cohesion, ADR/doc/catalog consistency, hyperscaler-*-claims) remain required, but they are
  ported into Rust gate crates/cloud-ci required contexts. `oya gate` and `oya verify` may remain only as legacy/local
  migration wrappers until their semantics are available through cloud-ci/oya-ci.
- **Let standard tools do their job:** cargo (fmt/check/clippy/nextest), cargo-deny (license/bans/advisories), Trivy/cosign/Syft (supply-chain), Opengrep, gitleaks — run natively as cloud-ci jobs/adapters, not re-wrapped as permanent `oya` CLI authority.
- **Retire:** `oya vcs`, `oya git`, and any use of `oya verify`/`oya gate` as protected-branch CI producers.

**Local lane-liveness bridge (FRIC-1781110000):** until cloud-ci owns durable lane orchestration, a retirement-marked local bridge may supervise headless agent lanes without becoming merge authority. The bridge surfaces are `tools/oya-lane-supervisor-app/BUCK`, `tools/oya-lane-supervisor-app/Cargo.toml`, `tools/oya-lane-supervisor-app/OWNERS`, `tools/oya-lane-supervisor-app/src/lib.rs`, `tools/oya-lane-supervisor-app/src/main.rs`, `registry/catalog/oya-lane-supervisor-app.yaml`, `registry/catalog/OWNERS`, `.omc/ultragoal/OWNERS`, `.omc/ultragoal/TEAMMATE-PREAMBLE.md`, `.omc/ultragoal/friction-ledger.jsonl`, `.omc/ultragoal/premise.txt`, and `.omc/ultragoal/review-verdict.txt`. They exist only to detect dead/stalled local lanes and preserve the plain-git protected-PR workflow; the durable destination remains cloud-ci/oya-ci required contexts.

### 5. Foundry → Intelligence absorption; Governance stays its own service
- The legacy `microservices/foundry/` material is **absorbed into Intelligence** (active target `{oya,cloud}/intelligence/` per ADR-0131/ADR-0512 amendment: adapters, runtime, supervisor, eval, capability, rag, account, dashboard). The `foundry/` dir is retired; its docs/contracts/manifest are reconciled into Intelligence.
- **Governance remains its own service** (active target `{oya,cloud}/governance/`). Rationale is **layering**: the governance gates validate every service including Intelligence, so Governance cannot be a part of the thing it validates (that would be circular). It keeps its gate semantics + the 2 admission/provider checks as cloud-ci contexts.
- **No `microservices/vcs/` service** is created.

## Rejected alternatives
- **Keep the agentic-VCS substrate / `oya vcs` / `oya git`** — rejected: reinvents plain git + GitHub/cloud-ci substrate concerns; ~13 crates dormant with 0 deployment ("don't reinvent the wheel"; "use git as is").
- **Fold Governance into Intelligence** — rejected: layering violation (a validator cannot live inside what it validates).
- **Stand up a `vcs` microservice** — rejected: no live domain; 2 gates are governance, the rest is dormant.
- **Build a merge queue in the retired VCS substrate** — rejected: merge automation belongs in cloud-ci/oya-ci Tide (ADR-0513), not in `oya vcs` or GitHub custom patches.
- **GitHub merge-queue/branch-protection as the substrate** — rejected: contradicts the self-hosted/OSI posture (ADR-0173/0247/0248); GitHub is bootstrap-only.

## Consequences

### Positive
- One less bespoke subsystem to maintain; coordination rides standard, OSI-clean, self-hostable tooling (plain git + GitHub GPLv3+ + cloud-ci required contexts, with the legacy CI bridge only as transitory transport until cutover).
- Governance value is sharpened to what only Oyatie needs (governance-as-code / AI-slop-defense) while retiring the `oya` CLI as VCS/CI authority.
- Intelligence becomes the single coherent home for the AI-agent platform; Governance's independence is principled (layering).

### Negative / risk
- **High-blast-radius rewire**: at ADR-0363 authorship, `oya vcs`/`oya git` were the canonical coordination surface (CLAUDE.md, `tools/hooks/_canonical-primitives.md`, SessionStart hooks all inject "use `oya git`/`oya vcs`"). The active contract now flips/has flipped to plain `git` as the destination. This is mechanical but wide → staged PR with its own review (PR-3 below).
- The 2 admission/provider gate-apps currently shell out to `oya vcs` — their rework must preserve the actual checks (prereq audit).
- Merge queue is now a cloud-ci/Tide dependency rather than a GitHub/VCS feature; until the minimal Tide admission contract is live, high-concurrency merges remain operator-serialized and cannot be claimed conflict-free.

### Migration (staged, each its own verified PR)
1. **PR-1** — absorb legacy `microservices/foundry/` source material into active `{oya,cloud}/intelligence/` targets (doc/contract reconciliation; build unaffected). (task #44)
2. **PR-2** — delete the ~13 dormant orchestration crates; supersede ADR-0110/0112/0113; freeze the changeset-event-log. (task #45)
3. **PR-3** — retire `oya vcs` + `oya git`; rework the 2 gates into cloud-ci/governance Rust gate crates posted as required contexts; flip CLAUDE.md / `_canonical-primitives.md` / SessionStart hooks / `registry/vcs/` to plain `git`. (task #46)
4. **Cloud-ci/Tide packet** — land the ADR-0513 Tide admission contract and required context placement; do not wait for PR-volume pain to decide ownership. Full batch/projected-state scaling may sequence after the minimum admission contract is live.
5. **Separate** — ADR-0357 vertical-slice crate nesting (task #10), and the GitHub→GitHub host migration (ADR-0247 post-bootstrap), are separate efforts.

## Verification
- After each PR: `cargo build --workspace` green, `cargo fmt --check` clean, and the required protected-branch
  cloud-ci/governance context green. Until the P0.0 cloud-ci required context is live, legacy `oya gate`/`oya verify`
  output is migration evidence only and cannot be the merge/exit authority.
- Cloud-ci/Rust gate packet `architecture-boundaries` green (catalog records move with crates).
- Cloud-ci/Rust gate packet `aspirational-enforcement` green (no binding claim references a retired lane/command).
- No residual `oya vcs` / `oya git` invocations in CLAUDE.md, hooks, `_canonical-primitives.md`, or gate code after PR-3.

## References
- ADR-0110 / 0112 / 0113 — agentic-VCS (superseded here).
- ADR-0116 — retire external agent-coordination tooling (amended: manual-bootstrap seam removed).
- ADR-0173 / 0247 / 0248 — vendor-lock-in avoidance + self-hosting doctrine; GitHub (interim) as the forge target.
- ADR-0361 — historical cloud-ci-native CI bridge; ADR-0362 — flat-only catalog; ADR-0349 — CI farm bridge/reference evidence superseded by ADR-0513 destination authority.
- GitHub capabilities (v15.0.2 / v11.x LTS, GPLv3+): github.org/docs (protection, webhooks, auto-merge), github#5102 (no native merge queue), LWN GPLv3+ relicense.
- Founder deep-interview 2026-05-26 (session_context above).

## Historical residual from ADR-103 (E3 fold 2026-08-06)

**Title:** Grit cutover inventory of legacy primitives

**Preserved decision gist:** Adopt this ADR as the canonical inventory of legacy primitives addressed by the grit cutover, with sanctioned replacements and (where applicable) retirement timing. | Legacy primitive | Replacement | Status | |---|---|---| | Direct `git` from agents | `grit claim`/`grit done` + `oya-tooling-agent-read log/diff/pr-view` | Banned (banned-primitives lane enforces) | | Direct `gh` from agents | `oya-tooling-agent-read pr-view`/`pr-comments` | Banned | | `git rebase` / `git merge` by agents | Controller-owned merge queue (M01-P07 IP-007) | Banned | | Local bash lock files | grit claim → work → done

_Source file archived after fold; full body in git history / docs/adr-archive/._

## Historical residual from ADR-113 (E3 fold 2026-08-06)

**Title:** ADR-0113-vcs-orchestrator-end-to-end

**Preserved decision gist:** `oya vcs done` becomes the canonical kickoff for the agentic pipeline. New invocation shape: ``` oya vcs done --changeset <id> # ULID from `oya vcs claim` [--subscribe <url>] # webhook URL for state-change events [--wait] # opt-in synchronous wait (default: async) [--cost-budget-usd <n>] # per-changeset USD cap (default: $10) [--cost-budget-tokens <n>] # per-changeset token cap (default: 2M) [--max-agent-invocations <n>] # per-changeset invocation cap (default: 50) [--draft] # open PR as draft [--title <text>] [--body <text>] ``` ### Async-by-default contract 1. Agent invokes `oya vcs done --c

_Source file archived after fold; full body in git history / docs/adr-archive/._

## Historical residual from ADR-223 (E3 fold 2026-08-06)

**Title:** ADR-0223-oya-git-drop-in-surface-with-explicit-policy-verbs

**Preserved decision gist:** Adopt `oya git <git-subcommand>` as a drop-in wrapper around git: - It invokes git for the requested subcommand. - It preserves git stdout, stderr, and exit status. - It may emit `oya:` informational diagnostics only where they do not change machine-readable git output. - It emits an audit-ledger event for each invocation. - The ledger is a local side channel under `.git/oya/ledger/audit-chain.jsonl`, not a tracked worktree artifact. - Ledger rows must avoid raw argument capture and absolute local paths. Keep policy lifecycle verbs explicit. `oya vcs <claim|work|verify|done|status|symbols|queu

_Source file archived after fold; full body in git history / docs/adr-archive/._

## Historical residual from ADR-112 (E3 fold 2026-08-06)

**Title:** ADR-0112-webhook-driven-intelligence-agent-invocation

**Preserved decision gist:** A new webhook-receiver app receives GitHub webhook deliveries and routes them to Foundry agents. ### Receiver shape - HTTP endpoint exposed at `/webhook/github` (hosted on the Foundry control plane — initial deployment is the existing Anthropic-API substrate; future deployments may move to a dedicated mesh service). - Verifies `X-Hub-Signature-256` HMAC against the webhook secret stored in OpenBao at `sref://openbao/oya/foundry/github-webhook-secret` (per the SecretReference contract). - Dedups by `X-GitHub-Delivery` header (GitHub-supplied UUID, unique per delivery, stable across redeliveries

_Source file archived after fold; full body in git history / docs/adr-archive/._

## Historical residual from ADR-110 (E3 fold 2026-08-06)

**Title:** ADR-0110-changeset-state-machine

**Preserved decision gist:** A changeset advances through a CLOSED ENUM of 9 states. Transitions are MONOTONIC (no backwards moves) and EVENT-SOURCED (every transition appends one row to the `changeset-event-log`). ### The 9 states | # | State | Owner | Entry trigger | Exit trigger | |---|---|---|---|---| | 0 | `opened` | claim agent | `oya vcs claim --intent ...` | first edit lands in worktree | | 1 | `working` | claim agent | first edit | `oya vcs verify` invoked | | 2 | `verified` | claim agent | `oya vcs verify` returns OK | `oya vcs done` invoked | | 3 | `pr_open` | `oya vcs done` orchestrator | PR opened against `de

_Source file archived after fold; full body in git history / docs/adr-archive/._
