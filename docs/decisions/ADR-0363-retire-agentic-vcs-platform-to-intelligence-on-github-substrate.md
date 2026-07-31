---
id: ADR-0363
status: Accepted
planning_impact: true
deciders: council-architecture, founder
date: 2026-05-26
owner: council-architecture
supersedes: [ADR-0110, ADR-0112, ADR-0113]
superseded_by: []
amended_by: [ADR-0510, ADR-0515]
amends: [ADR-0116]
related: [ADR-0053, ADR-0111, ADR-0116, ADR-0131, ADR-0173, ADR-0247, ADR-0248, ADR-0349, ADR-0357, ADR-0361, ADR-0362, ADR-0513]
related_specs: [/specs/gitops-vcs-replacement.json, /registry/vcs/changeset-event-log.json]
session_context:
  authored: 2026-05-26
  basis: "Founder deep-interview 2026-05-26 — 'absorb Foundry into Intelligence in a way that makes sense'; 'do we even need vcs? retire vcs, we have cloud-ci + git'; 'don't reinvent the wheel'; 'we'll use git as is — don't even oya git wrap'; 'we don't use github merge queue, we use selfhosted [OSS forge — brand name redacted by vocab policy]'."
purpose: Retire the bespoke agentic-VCS layer (oya vcs CLI + oya git wrapper + changeset-state machine + merge-queue + webhook-receiver) in favour of plain git + self-hosted OSS-forge PRs + Prow-shaped cloud-ci/oya-ci required contexts. The legacy cloud-ci bridge is bridge evidence only. Narrow the oya toolchain away from CLI authority; governance semantics move to Rust gate crates/cloud-ci contexts. Absorb Foundry's AI-agent platform into Intelligence; keep Governance as its own service.
---

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

**AMENDED 2026-07-31 — vocab-scrub text corruption repaired; NO decision is changed.**
A mechanical forbidden-vocab scrub (commit `e38624dc4`, D-CLOUD-NATIVE brand-stem eradication)
replaced this ADR's forge brand name with the token "GitHub" throughout the body; a later commit
(`2161fb386`) renamed the file slug the same way. The substitution was not meaning-preserving —
it left a substrate that is adopted and rejected in the same sentence, a claim that "GitHub" is
GPLv3+ and ships "v15.0.2 / v11.x LTS", an upstream issue number reattributed to the wrong
project, a "self-hosted GitHub" trunk substrate, a Decision §3 claim that "GitHub has no native
merge queue" that both is false and contradicts this ADR's own Context reference to a
"GitHub-style speculative merge queue", and a "GitHub→GitHub host migration" (Migration step 5).
Those passages are restored to their original
MEANING using stem-free wording ("a self-hosted OSS forge (GPLv3+)"). The forge's brand name is a
forbidden stem under `oya-ci.toml` `[[vocab.forbidden_stems]]`, so it is deliberately **not**
reintroduced; the verbatim pre-scrub text is recoverable from commit `e77f16eb2` (this ADR's blob
at its original path). The decision this ADR recorded, and still records, is:
**change-coordination substrate = plain `git` + a SELF-HOSTED OSS forge**, with GitHub explicitly
**rejected** as the substrate and retained **bootstrap-only** — see *Rejected alternatives*, which
survived the scrub intact. *Which* forge is stood up post-bootstrap is ADR-0247's call, not this
ADR's. This repair is prose-only: what runs today is governed by the ADR-0515 amendment directly
above, not by the restored paragraphs. Residual artifact **not** fixed here (it would require
repointing ~96 cites plus generated faces): this file's slug still reads `...-on-github-substrate`,
which misnames the decision; a slug rename is tracked separately.

## Context

Two threads converged in the 2026-05-26 founder interview:

1. **The Foundry name was eradicated** (ADR-0362 + the #181–#184 cutover): the former `oya-foundry-*` crates were renamed across three namespaces — `oya-intelligence-*` (116, the AI-agent platform), `oya-governance-*` (39 + 2, the quality/fitness gates), `oya-vcs-*` (20, the agentic-VCS substrate). `microservices/foundry/` (597 files) was kept as a now-name-mismatched doc shell.

2. **The agentic-VCS substrate was questioned.** Evidence: of the 20 `oya-vcs-*` crates, only **2** are wired (the `oya-vcs-admission` + `oya-vcs-provider-execution` CI checks); ~13 (changeset-state machine, merge-queue, promotion-controller, webhook-receiver per ADR-0110/0112/0113) have **0–1 dependents** and were never deployed. The `oya vcs` CLI ratchet and the `oya git` wrapper duplicate plain git.

The founder's principle: **don't reinvent the wheel; use git as-is.** And the substrate is **a self-hosted OSS forge** (per ADR-0173/0247/0248: vendor-lock-in avoidance + self-hosting doctrine; GitHub is the temporary bootstrap source-of-truth per ADR-0247, migrating to a self-hosted git-host µservice), **not** GitHub.

Best-practice research (2026-05-26, evaluated forge release v15.0.2 / v11.x LTS) confirms that self-hosted OSS forge natively provides — and that it is GPLv3+ (OSI-clean, not BSL/SSPL):
- Protected branches with **required status checks**, required reviews, dismiss-stale, restrict-push.
- **External-CI gating**: required-status-check contexts gate merges. Original bridge evidence used forge commit statuses posted by the legacy CI bridge; ADR-0513/platform-readiness moves authority to Prow-shaped cloud-ci/oya-ci required contexts.
- **Webhooks** (PR opened/updated/merged, push) to drive automation.
- **Auto-merge** ("merge when checks succeed").
- **NOT** a GitHub-style speculative **merge queue** (open upstream forge request #5102, no milestone).

So `git + a self-hosted OSS forge + branch-protection/required-checks/auto-merge` remains the self-hosted trunk-based-development substrate, with Prow-shaped cloud-ci/oya-ci as the current merge/CI authority target and the legacy CI bridge only as transitory transport. The bespoke changeset-state-machine / webhook-receiver / `oya vcs` / `oya git` re-implement substrate concerns that must be retired; merge-queue semantics move to Tide per ADR-0513 instead of this retired VCS layer.

## Decision

### 1. Change-coordination substrate = plain git + self-hosted OSS-forge PRs + Prow/cloud-ci required contexts
Adopt the standard self-hosted substrate for coordination: plain `git`, self-hosted forge PRs, branch protection, required status checks, webhooks, and auto-merge. Current merge/exit authority is Prow-shaped cloud-ci/oya-ci required context plus reviewer approval; the legacy CI bridge may post bridge statuses only until P0.0 cutover. GitHub remains the **bootstrap** host until the self-hosted forge service is stood up (ADR-0247 post-bootstrap; tracked separately, NOT in this ADR's scope).

### 2. Retire the bespoke agentic-VCS layer
Retire — because plain git + the self-hosted forge + cloud-ci/Prow required contexts provide or own the necessary substrate:
- The `oya vcs` CLI ratchet (`claim`/`work`/`done`/`promote`).
- The `oya git` wrapper — **use plain `git` as-is** (no drop-in wrapper).
- The dormant orchestration crates: changeset-state machine, merge-queue, merge-queue-conflict, promotion-controller, webhook-receiver, ci-fix-loop-dispatcher, ast-index, polyglot-indexer, lockstore, changebundle (~13 crates, 0–1 dependents).
- ADR-0110/0112/0113 are superseded; `registry/vcs/changeset-event-log.json` + the event-router are frozen as historical evidence (not deleted, not active).

**Kept as semantics, not `oya` CLI authority:** the 2 wired CI checks (`oya-vcs-admission`, `oya-vcs-provider-execution`) are recast as Governance/cloud-ci Rust gate contexts. Legacy bridge-posted forge statuses may mirror them during migration only; the permanent required producer is cloud-ci/oya-ci.

### 3. Merge queue: owned by cloud-ci/Tide, not the retired VCS ratchet
The self-hosted forge has no native merge queue, so the queue does **not** live in the forge or the retired agentic-VCS substrate. ADR-0513 places merge automation in the Prow-shaped cloud-ci/oya-ci Tide component (`oya-ci-tide`), folding ADR-0111 projected-state merge semantics into CI/admission. The ADR-0363 decision is therefore narrowed: retire the bespoke VCS ratchet; do not defer merge-queue ownership.

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
- **Keep the agentic-VCS substrate / `oya vcs` / `oya git`** — rejected: reinvents plain git + forge/cloud-ci substrate concerns; ~13 crates dormant with 0 deployment ("don't reinvent the wheel"; "use git as is").
- **Fold Governance into Intelligence** — rejected: layering violation (a validator cannot live inside what it validates).
- **Stand up a `vcs` microservice** — rejected: no live domain; 2 gates are governance, the rest is dormant.
- **Build a merge queue in the retired VCS substrate** — rejected: merge automation belongs in cloud-ci/oya-ci Tide (ADR-0513), not in `oya vcs` or custom forge patches.
- **GitHub merge-queue/branch-protection as the substrate** — rejected: contradicts the self-hosted/OSI posture (ADR-0173/0247/0248); GitHub is bootstrap-only.

## Consequences

### Positive
- One less bespoke subsystem to maintain; coordination rides standard, OSI-clean, self-hostable tooling (plain git + a GPLv3+ self-hosted OSS forge + cloud-ci required contexts, with the legacy CI bridge only as transitory transport until cutover).
- Governance value is sharpened to what only Oyatie needs (governance-as-code / AI-slop-defense) while retiring the `oya` CLI as VCS/CI authority.
- Intelligence becomes the single coherent home for the AI-agent platform; Governance's independence is principled (layering).

### Negative / risk
- **High-blast-radius rewire**: at ADR-0363 authorship, `oya vcs`/`oya git` were the canonical coordination surface (CLAUDE.md, `tools/hooks/_canonical-primitives.md`, SessionStart hooks all inject "use `oya git`/`oya vcs`"). The active contract now flips/has flipped to plain `git` as the destination. This is mechanical but wide → staged PR with its own review (PR-3 below).
- The 2 admission/provider gate-apps currently shell out to `oya vcs` — their rework must preserve the actual checks (prereq audit).
- Merge queue is now a cloud-ci/Tide dependency rather than a forge/VCS feature; until the minimal Tide admission contract is live, high-concurrency merges remain operator-serialized and cannot be claimed conflict-free.

### Migration (staged, each its own verified PR)
1. **PR-1** — absorb legacy `microservices/foundry/` source material into active `{oya,cloud}/intelligence/` targets (doc/contract reconciliation; build unaffected). (task #44)
2. **PR-2** — delete the ~13 dormant orchestration crates; supersede ADR-0110/0112/0113; freeze the changeset-event-log. (task #45)
3. **PR-3** — retire `oya vcs` + `oya git`; rework the 2 gates into cloud-ci/governance Rust gate crates posted as required contexts; flip CLAUDE.md / `_canonical-primitives.md` / SessionStart hooks / `registry/vcs/` to plain `git`. (task #46)
4. **Cloud-ci/Tide packet** — land the ADR-0513 Tide admission contract and required context placement; do not wait for PR-volume pain to decide ownership. Full batch/projected-state scaling may sequence after the minimum admission contract is live.
5. **Separate** — ADR-0357 vertical-slice crate nesting (task #10), and the GitHub→self-hosted-forge host migration (ADR-0247 post-bootstrap), are separate efforts.

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
- ADR-0173 / 0247 / 0248 — vendor-lock-in avoidance + self-hosting doctrine; a self-hosted OSS forge as the forge target (GitHub is bootstrap-only).
- ADR-0361 — historical cloud-ci-native CI bridge; ADR-0362 — flat-only catalog; ADR-0349 — CI farm bridge/reference evidence superseded by ADR-0513 destination authority.
- Self-hosted OSS forge capabilities (v15.0.2 / v11.x LTS, GPLv3+): the forge project's own documentation site (protection, webhooks, auto-merge), its upstream issue #5102 (no native merge queue), LWN GPLv3+ relicense. Brand name and URL omitted per the `oya-ci.toml` forbidden-stem policy; see the 2026-07-31 amendment above.
- Founder deep-interview 2026-05-26 (session_context above).
