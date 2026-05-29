---
id: ADR-0363
status: Accepted
planning_impact: true
deciders: council-architecture, founder
date: 2026-05-26
owner: council-architecture
supersedes: [ADR-0110, ADR-0112, ADR-0113]
superseded_by: []
amended_by: [ADR-0510]
amends: [ADR-0116]
related: [ADR-0053, ADR-0111, ADR-0116, ADR-0131, ADR-0173, ADR-0247, ADR-0248, ADR-0349, ADR-0357, ADR-0361, ADR-0362]
related_specs: [/specs/gitops-vcs-replacement.json, /registry/vcs/changeset-event-log.json]
session_context:
  authored: 2026-05-26
  basis: "Founder deep-interview 2026-05-26 — 'absorb Foundry into Intelligence in a way that makes sense'; 'do we even need vcs? retire vcs, we have jenkins + git'; 'don't reinvent the wheel'; 'we'll use git as is — don't even oya git wrap'; 'we don't use github merge queue, we use selfhosted forgejo'."
purpose: Retire the bespoke agentic-VCS layer (oya vcs CLI + oya git wrapper + changeset-state machine + merge-queue + webhook-receiver) in favour of the standard self-hosted substrate (git + Jenkins + self-hosted Forgejo). Narrow the oya toolchain to its differentiated core — the governance-gate engine. Absorb Foundry's AI-agent platform into the Intelligence microservice; keep Governance as its own service.
---

# ADR-0363: Retire bespoke agentic-VCS; Foundry→Intelligence; `oya` is a governance-gate engine

## Status

Accepted — 2026-05-26. Supersedes ADR-0110 (changeset state machine), ADR-0112 (webhook-driven invocation), ADR-0113 (vcs orchestrator end-to-end). Amends ADR-0116 (which parked external coordination tooling behind a manual-bootstrap seam — that seam is now removed, not deployed).

## Context

Two threads converged in the 2026-05-26 founder interview:

1. **The Foundry name was eradicated** (ADR-0362 + the #181–#184 cutover): the former `oya-foundry-*` crates were renamed across three namespaces — `oya-intelligence-*` (116, the AI-agent platform), `oya-governance-*` (39 + 2, the quality/fitness gates), `oya-vcs-*` (20, the agentic-VCS substrate). `microservices/foundry/` (597 files) was kept as a now-name-mismatched doc shell.

2. **The agentic-VCS substrate was questioned.** Evidence: of the 20 `oya-vcs-*` crates, only **2** are wired (the `oya-vcs-admission` + `oya-vcs-provider-execution` CI checks); ~13 (changeset-state machine, merge-queue, promotion-controller, webhook-receiver per ADR-0110/0112/0113) have **0–1 dependents** and were never deployed. The `oya vcs` CLI ratchet and the `oya git` wrapper duplicate plain git.

The founder's principle: **don't reinvent the wheel; use git as-is.** And the substrate is **self-hosted Forgejo** (per ADR-0173/0247/0248: vendor-lock-in avoidance + self-hosting doctrine; GitHub is the temporary bootstrap source-of-truth per ADR-0247, migrating to a self-hosted git-host µservice), **not** GitHub.

Best-practice research (2026-05-26, Forgejo v15.0.2 / v11.x LTS) confirms Forgejo natively provides — and is GPLv3+ (OSI-clean, not BSL/SSPL):
- Protected branches with **required status checks**, required reviews, dismiss-stale, restrict-push.
- **External-CI gating**: Jenkins posts commit statuses via Forgejo's Commit Status API; required-status-check contexts gate merges.
- **Webhooks** (PR opened/updated/merged, push) to drive automation.
- **Auto-merge** ("merge when checks succeed").
- **NOT** a GitHub-style speculative **merge queue** (open request forgejo#5102, no milestone).

So `git + self-hosted Forgejo + Jenkins + branch-protection/required-checks/auto-merge` is the standard self-hosted trunk-based-dev substrate. The bespoke changeset-state-machine / merge-queue / webhook-receiver / `oya vcs` / `oya git` re-implement what that substrate already ships.

## Decision

### 1. Change-coordination substrate = git + Jenkins + self-hosted Forgejo
Adopt the standard self-hosted substrate. Forgejo provides PRs, branch protection, required status checks (Jenkins → Forgejo Commit Status API), webhooks, and auto-merge. GitHub remains the **bootstrap** host until the self-hosted Forgejo µservice is stood up (ADR-0247 post-bootstrap; tracked separately, NOT in this ADR's scope).

### 2. Retire the bespoke agentic-VCS layer
Retire — because git + Forgejo + Jenkins provide it natively:
- The `oya vcs` CLI ratchet (`claim`/`work`/`done`/`promote`).
- The `oya git` wrapper — **use plain `git` as-is** (no drop-in wrapper).
- The dormant orchestration crates: changeset-state machine, merge-queue, merge-queue-conflict, promotion-controller, webhook-receiver, ci-fix-loop-dispatcher, ast-index, polyglot-indexer, lockstore, changebundle (~13 crates, 0–1 dependents).
- ADR-0110/0112/0113 are superseded; `registry/vcs/changeset-event-log.json` + the event-router are frozen as historical evidence (not deleted, not active).

**Kept:** the 2 wired CI checks (`oya-vcs-admission`, `oya-vcs-provider-execution`) — recast as plain Jenkins-posted **Forgejo required status checks** and **moved into the Governance service** (they gate repo-entry = a governance concern).

### 3. Merge queue: deferred, not built
Forgejo has no native merge queue. At current PR volume, **auto-merge + required status checks suffice** (each PR's checks run against its own head). A merge queue (external `bors`/`gitea-mq`, or adopting Forgejo's feature if/when forgejo#5102 lands) is adopted **only if/when concurrent-PR volume + semantic-conflict risk justify it** — and even then, *adopt*, don't reinvent. ADR-0111 (speculative merge-queue) is parked under this condition.

### 4. `oya` is a governance-gate engine, not a VCS/CI tool
Narrow the toolchain to its **differentiated core**:
- **Keep:** `oya gate` — the ~20 oyatie-specific governance checks with **no off-the-shelf equivalent** (honest-claims, claim-ceiling, aspirational-enforcement, cohesion, plane-class, data-class, banned-primitives, glossary-vocabulary, no-grouping, authority-cohesion, ADR/doc/catalog consistency, hyperscaler-*-claims). This is the AI-slop-defense / executable doctrine — the reason `oya` exists. `oya verify` (local CI mirror) is retained as a convenience over the gates + cargo.
- **Let standard tools do their job:** cargo (fmt/check/clippy/nextest), cargo-deny (license/bans/advisories), Trivy/cosign/Syft (supply-chain), Opengrep, gitleaks — run **natively in the Jenkins lane** (ADR-0361), not re-wrapped by `oya`.
- **Retire:** `oya vcs`, `oya git` (per §2).

### 5. Foundry → Intelligence absorption; Governance stays its own service
- `microservices/foundry/` is **absorbed into `microservices/intelligence/`** (the AI-agent platform: adapters, runtime, supervisor, eval, capability, rag, account, dashboard). The `foundry/` dir is retired; its docs/contracts/manifest are reconciled into `intelligence/`.
- **Governance remains its own service** (`microservices/governance/`). Rationale is **layering**: the governance gates validate *every* microservice including Intelligence, so Governance cannot be a part of the thing it validates (that would be circular). It keeps its gates + the 2 admission/provider checks.
- **No `microservices/vcs/` service** is created.

## Rejected alternatives
- **Keep the agentic-VCS substrate / `oya vcs` / `oya git`** — rejected: reinvents git + Forgejo + Jenkins natives; ~13 crates dormant with 0 deployment ("don't reinvent the wheel"; "use git as is").
- **Fold Governance into Intelligence** — rejected: layering violation (a validator cannot live inside what it validates).
- **Stand up a `vcs` microservice** — rejected: no live domain; 2 gates are governance, the rest is dormant.
- **Build a merge queue now** — rejected: Forgejo lacks one but auto-merge + required checks suffice at current scale; adopt (not build) later only if volume demands.
- **GitHub merge-queue/branch-protection as the substrate** — rejected: contradicts the self-hosted/OSI posture (ADR-0173/0247/0248); GitHub is bootstrap-only.

## Consequences

### Positive
- One less bespoke subsystem to maintain; coordination rides standard, OSI-clean, self-hostable tooling (git + Forgejo GPLv3+ + Jenkins).
- `oya`'s value is sharpened to what only it can do (governance-as-code / AI-slop-defense).
- Intelligence becomes the single coherent home for the AI-agent platform; Governance's independence is principled (layering).

### Negative / risk
- **High-blast-radius rewire**: `oya vcs`/`oya git` are the *current* canonical coordination surface (CLAUDE.md, `tools/hooks/_canonical-primitives.md`, SessionStart hooks all inject "use `oya git`/`oya vcs`"). Retiring them flips the agent operating contract to plain `git`. This is mechanical but wide → staged PR with its own review (PR-3 below).
- The 2 admission/provider gate-apps currently shell out to `oya vcs` — their rework must preserve the actual checks (prereq audit).
- No merge queue → at high concurrency, semantic conflicts between concurrently-merged PRs aren't caught; accepted at current scale, revisit per §3.

### Migration (staged, each its own verified PR)
1. **PR-1** — absorb `microservices/foundry/` → `microservices/intelligence/` (doc/contract reconciliation; build unaffected). (task #44)
2. **PR-2** — delete the ~13 dormant orchestration crates; supersede ADR-0110/0112/0113; freeze the changeset-event-log. (task #45)
3. **PR-3** — retire `oya vcs` + `oya git`; rework the 2 gates → Governance as Jenkins-posted Forgejo required checks; flip CLAUDE.md / `_canonical-primitives.md` / SessionStart hooks / `registry/vcs/` to plain `git`. (task #46)
4. **Deferred** — ADR-0357 vertical-slice crate nesting (task #10), and the GitHub→Forgejo host migration (ADR-0247 post-bootstrap), are separate efforts.

## Verification
- After each PR: `cargo build --workspace` green, `cargo fmt --check` clean, `oya gate run-all` green (uncontended; rebuild `oya` first).
- `oya gate validate architecture-boundaries` green (catalog records move with crates).
- `oya gate validate aspirational-enforcement` green (no binding claim references a retired lane/command).
- No residual `oya vcs` / `oya git` invocations in CLAUDE.md, hooks, `_canonical-primitives.md`, or gate code after PR-3.

## References
- ADR-0110 / 0112 / 0113 — agentic-VCS (superseded here).
- ADR-0116 — retire external agent-coordination tooling (amended: manual-bootstrap seam removed).
- ADR-0173 / 0247 / 0248 — vendor-lock-in avoidance + self-hosting doctrine; self-hosted Forgejo as the forge target.
- ADR-0361 — Jenkins-native CI; ADR-0362 — flat-only catalog; ADR-0349 — CI farm.
- Forgejo capabilities (v15.0.2 / v11.x LTS, GPLv3+): forgejo.org/docs (protection, webhooks, auto-merge), forgejo#5102 (no native merge queue), LWN GPLv3+ relicense.
- Founder deep-interview 2026-05-26 (session_context above).
