---
doc_class: Operating-Contract
shape: null
length_cap: 360
authority_tier: 2
excludes:
- path: /specs/root-hub-pointers.json
  reason: Machine-readable entry-point registry; this contract is discovered through
    it.
- path: /specs/master-plan-sequencing.json
  reason: Primitive policy, plain-git sequencing, and ChangeSet promotion; cited and
    not duplicated fully.
- path: /specs/markdown-retirement-policy.json
  reason: Markdown lifecycle and root-hub survival policy; cited and not duplicated
    fully.
- path: docs/DOC-CATALOG.md
  reason: Per-doc lifecycle protocol and trigger taxonomy.
- path: docs/templates/pull-request-template.md
  reason: PR body shape; cited and not inlined.
- path: docs/decisions/
  reason: Architectural rationale; ADRs are decision records, not operating contracts.
- path: docs/teams/
  reason: Per-team norms and charters.
- path: docs/SPEC.md
  reason: Surface enumeration; this contract operates on surfaces, does not enumerate
    them.
- path: docs/standards/
  reason: Cross-cutting authoring norms; this contract names them, does not inline
    them.
- path: docs/MISTAKES-LEDGER.md
  reason: Failure-mode catalog; this contract operates the doctrine, does not catalog
    it.
authority_chain_declaration: |
  system / developer / user instructions
    > CLAUDE.md + docs/AGENTS.md (authoritative agent entry + operating contract until explicit /specs/agent-operating-contract.json PHASE-5 promotion evidence)
    > /specs/root-hub-pointers.json (redirect/index hub; pointer cohesion, not an override of CLAUDE/docs/AGENTS)
    > installed agent-runtime skill and role catalog (for Codex: ~/.codex/skills + ~/.codex/agents; project .codex overlays only when intentionally checked in)
    > machine-readable specs and registries under /specs, /registry, /evidence, and /templates
    > docs/ authority files during markdown-retirement compatibility
    > external/upstream skill documentation (informational only; not vendored into this repo)
    > working drafts (never authoritative)
purpose: "Operating-Contract: Oyatie Agent Operating Contract."
doc_status: published
---
# Oyatie Agent Operating Contract

## Machine-readable authority — [root hub pointers](..//specs/root-hub-pointers.json)

## Workspace doctrine — applies to every documentation / file / workflow

Canonical doctrine: [`/specs/oyatie-doctrine.json`](..//specs/oyatie-doctrine.json) v1.0.0. **Principles P0..P9** (agentic-primary, machine-optimized, programmatic-where-possible, deterministic-where-it-matters, enforce-in-every-thing, iterate-until-consensus, no-silent-regression, Bominal-inheritance, canonical-base-+-localization, no-sprawl) bind every PR.

Workflow Studio product surface inverts P0 (human-ergonomic-first, no-code-first, SDK as enrichment). See `oyatie-doctrine.json#scope_clarifications`.

## Wave 15-ZF doctrine refs — ADR-0346..ADR-0349

Every agent MUST treat [ADR-0346](decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md), [ADR-0347](decisions/ADR-0347-governance-fitness-bulk-rename.md), [ADR-0348](decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md), and [ADR-0349](decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md) as active operating-contract doctrine for their non-CI obligations until superseded or amended by a newer ADR. For CI/CD enforcement, [ADR-0515](decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md) is the current single-truth amendment: GitHub Actions + branch protection are the live runner/authority until explicit owned-runner cutover, the cloud-ci Rust gate apps produce the one protected `oya-ci-required` context, and ADR-0513/Prow/Jenkins/legacy `oya` CLI governance wording is superseded provenance or local-feedback evidence only.

| ADR | Operating-contract binding | Enforced-by lanes agents MUST preserve |
|---|---|---|
| ADR-0346 (amended by ADR-0515) | `./bin/oya verify --ci-required` / `oya verify` are optional legacy local-feedback evidence only. They MUST NOT be extended or treated as protected-branch merge/exit authority; preserve old semantics only as provenance/local diagnostics while required semantics live in cloud-ci Rust gate packets. | The only merge authority is the single protected `oya-ci-required` context; do not add new `oya` CLI CI authority. |
| ADR-0347 | Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request; the rename is name-only and lane invariants remain preserved. | `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, `oya-governance-rename-inventory-presence`. |
| ADR-0348 | Cellular topology MUST support autosharding, auto-rebalance, and dynamic sharding through manifest-declared `sharding_automation` blocks, honoring residency, reversibility, and audit-chain emission. | `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, `oya-governance-tenant-migration-reversibility`. |
| ADR-0349 (amended/superseded for CI by ADR-0515) | Jenkins and Prow-shaped wording are historical/provenance only. GitHub Actions is the current ADR-0515 live runner for the canonical cloud-ci pipeline until explicit owned-runner cutover, not a parallel authority; ArgoCD/Argo Rollouts remain CD bridge/reference adapters where separately authorized. | Preserve bridge references only as provenance; do not add new Jenkins/Groovy, Prow, or `oya` CLI CI authority. Destination lanes are cloud-ci Rust gate contexts aggregated into `oya-ci-required` plus ArgoCD tenant-isolation/deploy audit lanes. |

## Multispectrum review bar — required on every change

Every changeset (agentic OR human-authored) MUST emit a multispectrum evidence file at `/evidence/multispectrum/<change_id>-<unix_ts>.json` conforming to [`/specs/multispectrum-review.json`](..//specs/multispectrum-review.json) v2.4.0 (`evidence_schema`). ADR-0515 owns protected merge authority: cloud-ci Rust gate packets run by GitHub Actions and aggregate into `oya-ci-required`; legacy `oya gate` / `oya verify` output is optional local-feedback/provenance evidence only and cannot satisfy merge authority. Those governance checks plus the seam-discipline lane `oya-check-dependency-seam` REFUSE the changeset when:

- evidence file absent OR
- declared `change_class_id` not in {CC-1..CC-7} OR
- required facets (F1..F13 except F12-reserved when applicable; A-family policy-adherence facets for policy-touching changes; plus `M1`/`M2` when `meta_review_triggered`) missing OR
- mandatory artifacts per the rigor matrix missing.

This applies to **agentic flow** AND **dev flow**. Agentic flow is the primary consumer; the spec is read at PR-open / gate-run time. Plain `git` + protected PR against `dev` remains the coordination path; the merge/exit CI authority is the single protected `oya-ci-required` context produced by the cloud-ci gate apps per ADR-0515. Jenkins/Prow/ADR-0513 CI wording is historical provenance, and `oya gate` output is optional local feedback only. ADR-0363 records the retired bespoke ratchet. It is re-evaluated each iterative-fix-loop cycle. See [`docs/standards/multispectrum-review.md`](standards/multispectrum-review.md) for the human gateway and [`/registry/fixuptasks.jsonl`](..//registry/fixuptasks.jsonl) for the bounded-deferral registry.

This is the single contract every agent (Claude Code, Codex, Gemini, OMC subagents, Foundry capabilities) and every human contributor honors before changing the repository. It is dual-audience: every directive is simultaneously a human-readable instruction and a machine-extractable typed artifact (RFC-2119 keyword + named path / lane / validator).

Before changing this repo, read `/specs/root-hub-pointers.json` first, then this contract. The retired Constitution concept is redistributed through the root hub, master-plan specs, RACI ownership, and sanctioned-primitive specs.

## Authority precedence

The higher source wins on conflict.

```
system / developer / user instructions
  > CLAUDE.md + docs/AGENTS.md (authoritative agent entry + operating contract until explicit /specs/agent-operating-contract.json PHASE-5 promotion evidence)
  > /specs/root-hub-pointers.json (redirect/index hub; pointer cohesion, not an override of CLAUDE/docs/AGENTS)
  > installed agent-runtime skill and role catalog (for Codex: ~/.codex/skills + ~/.codex/agents)
  > machine-readable specs and registries under /specs, /registry, /evidence, and /templates
  > docs/ authority files during markdown-retirement compatibility
  > external/upstream skill documentation (informational only; not vendored into this repo)
  > working drafts (never authoritative)
```

The chain is aligned with `/specs/root-hub-pointers.json` discoverability and the markdown-retirement policy while keeping CLAUDE.md + docs/AGENTS.md authoritative until explicit PHASE-5 promotion evidence lands. A missed PHASE-5 deadline does not automatically promote the projection; the `oya-governance-authority-cohesion` lane validates pointer cohesion during reconciliation.

The installed agent-runtime skill and role catalog provides universal intent→skill mapping, anti-rationalization, persona/skill/command orchestration, and role prompts. Oyatie governance (this file) OVERLAYS and WINS on conflict per Bominal-inheritance precedence (`feedback_bominal_inheritance_precedence`). The retired `tools/agent-skills/` vendor tree is intentionally absent; agents should use their installed runtime surfaces instead of repo-local duplicated copies.

## RFC-2119 normative-language statement

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** in this document are to be interpreted as described in BCP 14 [[RFC2119](https://www.rfc-editor.org/rfc/rfc2119)] [[RFC8174](https://www.rfc-editor.org/rfc/rfc8174)] when, and only when, they appear in all capitals, as shown here.

Lowercase forms ("you must", "should consider") have their normal English meanings and carry no normative force.

## Canonical doc map

For any question, route to its authority. Click the link; do not duplicate inline.

| Question | Authority |
|---|---|
| Intent→skill mapping, lifecycle phases, anti-rationalization, persona/skill/command orchestration | Installed agent-runtime skill catalog (Codex default: `~/.codex/skills`; project `.codex/skills` only when intentionally checked in) |
| Universal skill catalog | Installed runtime skills, discovered by the active agent surface; no repo-vendored duplicate |
| Reusable agent personas / roles | Installed runtime roles (Codex default: `~/.codex/agents`; set `agent_type` explicitly for OMX subagents) |
| Project mission, decision rights, prohibited primitives, amendments | [`MASTERPLAN.md`](MASTERPLAN.md), [`/specs/master-plan-sequencing.json`](..//specs/master-plan-sequencing.json), [`RACI-OWNERSHIP.md`](RACI-OWNERSHIP.md) |
| Bootstrap routing for the canonical tree | [`README.md`](README.md) |
| Architecture, planes, cross-axis contracts, cohesion thesis | [`DESIGN.md`](DESIGN.md) <!-- forward-reference: wave-1 --> |
| Surfaces (capabilities, APIs, events, indexes, ad slots, cloud resources) | [`SPEC.md`](SPEC.md) <!-- forward-reference: wave-1 --> |
| North star, axes, scope, success metrics, decision log | [`PRD.md`](PRD.md) <!-- forward-reference: wave-1 --> |
| Wave sequence, per-wave gate criteria | [`ROADMAP.md`](ROADMAP.md) <!-- forward-reference: wave-1 --> |
| Per-doc lifecycle and update protocol | [`DOC-CATALOG.md`](DOC-CATALOG.md) |
| Doc-class taxonomy, voice, dual-audience rules | [`standards/doc-style.md`](standards/doc-style.md) <!-- forward-reference: wave-1 --> |
| Architectural decisions (ADR pack) | [`ADR-INDEX.md`](ADR-INDEX.md) <!-- forward-reference: wave-1 --> |
| Recurring failure modes + mechanical preventions | [`MISTAKES-LEDGER.md`](MISTAKES-LEDGER.md) <!-- forward-reference: wave-1 --> |
| Per-axis product PRDs | [`products/`](products/) <!-- forward-reference: wave-1 --> |
| Per-team charters | [`teams/`](teams/) <!-- forward-reference: wave-1 --> |
| Per-region packs | [`regional-packs/`](regional-packs/) <!-- forward-reference: wave-1 --> |
| Runbooks (incident, DR, on-call, per-service) | [`RUNBOOKS-INDEX.md`](RUNBOOKS-INDEX.md) <!-- forward-reference: wave-1 --> |
| Templates (PR, ADR, capability, runbook, etc.) | [`templates/`](templates/) <!-- forward-reference: wave-1 --> |
| Privacy / security / compliance | [`PRIVACY-PROGRAM.md`](PRIVACY-PROGRAM.md) <!-- forward-reference: wave-1 -->, [`SECURITY-PROGRAM.md`](SECURITY-PROGRAM.md) <!-- forward-reference: wave-1 -->, [`COMPLIANCE-MATRIX.md`](COMPLIANCE-MATRIX.md) <!-- forward-reference: wave-1 --> |
| Release / incident / on-call | [`RELEASE-MANAGEMENT.md`](RELEASE-MANAGEMENT.md) <!-- forward-reference: wave-1 -->, [`INCIDENT-MANAGEMENT.md`](INCIDENT-MANAGEMENT.md) <!-- forward-reference: wave-1 -->, [`standards/on-call.md`](standards/on-call.md) <!-- forward-reference: wave-1 --> |
| Glossary (canonical vocabulary) | [`GLOSSARY.md`](GLOSSARY.md) <!-- forward-reference: wave-1 --> |
| Machine-readable mirrors of the catalog | [`machine-readable/`](machine-readable/) <!-- forward-reference: wave-1 --> |

## Pre-flight checklist

Before any change, every agent and every human MUST complete these items.

1. **Identify the change class.** Feature / bugfix / refactor / migration / docs / chore / capability / plugin / runbook / ADR / pack-update. *Why:* a class-blind change misses class-specific validators. *Test:* PR body's `## Issue` section names the class.
2. **Read the canonical authority for the change class.** Use the §"Canonical doc map" table. *Why:* one-paragraph orientation prevents the most common failure (acting on stale repo memory). *Test:* PR `## Traceability` cites the doc(s) read.
3. **Confirm Data Use Boundary.** Every new field on a kernel struct MUST carry a `data_class` annotation. *Why:* cross-pillar flows that bypass `data_class` violate the cohesion principle. *Test:* `oya-governance-data-class` lane.
4. **Confirm autonomy ceiling.** Capability bindings MUST declare T1 / T2 / T3 / T4 in the capability record. Tier uplift MUST land an accompanying Cedar policy + runtime gate. *Why:* config-flag tier uplift bypasses the audit chain. *Test:* `oya-governance-autonomy-ceiling` lane.
5. **Confirm license posture.** New dependencies MUST clear `cargo deny check`. AGPL / GPL / SSPL / BUSL / RSAL are not permitted in product code. *Why:* license drift is hard to undo. *Test:* `cargo deny check` exit 0.
6. **Search MISTAKES-LEDGER for the failure-mode class.** *Why:* re-introducing a fixed defect is a regression. *Test:* PR `## Traceability` cites the relevant `MFL-NNNN` row OR a "no prior row" search note.
7. **Identify the per-change-class reviewer agent.** *Why:* the reviewer signs `## Code Review` at merge time; no signature, no merge. *Test:* §"Per-change-class reviewer agents" table below; merge-gate hook validates.
8. **For cross-axis contract changes:** apply the cross-axis review label per [`checklists/cross-axis-contract-change.md`](checklists/cross-axis-contract-change.md) <!-- forward-reference: wave-1 -->; notify consumer-axis teams. *Why:* silent cross-axis changes break consumers. *Test:* PR label + `oya-governance-cross-axis-notify` lane.
9. **For hook / harness / CLI changes:** run the harness self-test first. *Why:* a broken hook silently disables every downstream gate. *Test:* harness self-test command (per harness; see §"Per-agent appendices").

## Per-change-class reviewer agents

Each change class has a designated reviewer agent that runs proactively on the PR and signs `## Code Review` at merge time.

| Change class | Reviewer agent |
|---|---|
| `*.rs` | `rust-reviewer` |
| `*.ts` / `*.tsx` / `*.js` / `*.jsx` | `typescript-reviewer` |
| `*.py` | `python-reviewer` |
| Migrations / SQL | `database-reviewer` |
| Auth / secret / payment paths | `security-reviewer` |
| Privacy / consent / DSR paths | `privacy-reviewer` |
| New feature or bugfix | `tdd-guide` (TDD enforcement) |
| Error-handling change | `silent-failure-hunter` |
| API or contract change | `doc-updater` |
| Doc-only change | `doc-style-reviewer` |
| Capability publish | `capability-reviewer` |
| Performance change | `perf-reviewer` |

The reviewer-agent verdict is `APPROVE` or `REQUEST CHANGES`. The PR body's `## Code Review` section MUST contain the agent name, the verdict, and the resolved + deferred items. Without this section the merge gate refuses (`scripts/hooks/guard-pr-merge-review.mjs`, PreToolUse on `Bash`).

## During-change discipline

While the change is in flight, every agent and every human MUST observe these rules.

- **No `--no-verify`, no hook bypass, no signing skip.** Hook failure is a signal; the fix is the underlying issue.
- **No untyped values at API boundaries.** Use the result types prescribed in [`standards/error-handling.md`](standards/error-handling.md) <!-- forward-reference: wave-1 -->.
- **No new struct fields in kernel crates without `data_class`.** Pre-commit blocks; respect it.
- **No quarantining flaky tests without a 14-day fix SLA.** Quarantine assigns the test to the `flaky/` lane; the SLA is tracked.
- **No editing legacy retired paths.** If a path was retired in a consolidation event, do not recreate it.
- **Bacon for dev-loop, nextest for evidence.** Prefer `bacon check / clippy / nextest` for fast feedback. Final evidence runs `cargo nextest run --workspace --all-features --no-fail-fast` per [`standards/testing.md`](standards/testing.md) <!-- forward-reference: wave-1 -->.
- **Kanban coordinator / worker split.** The board-steward role is the portfolio/architecture coordinator: it evaluates architecture, system design, completed and upcoming work, maturity gaps, documentation/procedure/process health, regressions, and Kanban decomposition/prioritization. Dispatcher-assigned workers execute scoped implementation, review, verification, and PR evidence lanes in isolated worktrees. The coordinator MUST NOT become the default implementation worker unless explicitly assigned as that lane worker.
- **Blockers become work.** A coordinator that finds a blocker MUST create/link a dispatcher-ready resolution card with source context, blocker class, acceptance criteria, verification path, suggested owner/profile, and dependency/conflict notes. Do not convert blockers into ad hoc coordinator implementation unless the coordinator is explicitly assigned as worker for that lane.
- **Autonomous merge boundary.** Autonomous merge authority exists only when the PR is fully reviewed, review threads are resolved, the required `oya-ci-required` context is green, the branch has no merge conflict, and branch protection is satisfied. Green CI alone is insufficient.

## Sanctioned primitives

Agent coordination uses plain `git`. ADR-0363 retires the prior wrapper/ratchet
substrate; do not reintroduce an agentic VCS wrapper. ADR-0515 retires CLI
governance and makes GitHub Actions + branch protection the live CI runner until
explicit owned-runner cutover. An agent works on an isolated worktree branch and
opens a pull request against `dev`, which enters the governance pipeline:
the single protected `oya-ci-required` context + reviewer APPROVE gate merge
readiness. `oya gate` / `oya verify` output is optional local feedback or
provenance only; it is never protected-branch CI authority.

The fenced block below is the machine-readable agent surface. Human-facing terminal examples may live outside fences.

<!-- agent-instructions:start -->
sanctioned_primitives:
  - git
legacy_local_feedback_primitives_not_merge_authority:
  - oya-gate
  - oya-verify
required_sequence:
  - isolated worktree branch per agent lane (scaffold-managed; one lane = one worktree)
  - commit and push on that lane
  - open a PR against dev               # enters the governance pipeline
  - fully reviewed, review threads resolved, no merge conflict, branch protection satisfied,
    and single protected `oya-ci-required` context green (legacy CLI evidence optional/local only)
  - squash merge after review threads resolve
  - post-merge product-completion packet: promoted SHA `oya-ci-required` green,
    rollout verification, rollback note, observability check, browser UX/user-story evidence,
    and release-governance/release-note impact (Release Please applies only when a live repo config/workflow exists)
coordinator_worker_split:
  coordinator: portfolio steward owns architecture, system design, maturity,
    regression audit, and Kanban decomposition/prioritization
  worker: dispatcher-assigned implementation/review worker owns scoped edits, tests,
    review, and PR evidence
  boundary: coordinator is not the default implementation worker unless explicitly assigned
blocker_policy: queue/link dispatcher-ready resolution cards with source context,
  blocker class, acceptance criteria, verification path, suggested owner/profile,
  and dependency/conflict notes unless explicitly assigned as that lane worker
scaffold_protocol:
  mechanism: per-agent isolated worktree plus admission-gate concurrent-safe-paths
  adr: docs/decisions/ADR-0363-retire-agentic-vcs-platform-to-intelligence-on-github-substrate.md
<!-- agent-instructions:end -->

## PR shape

Every PR uses [`templates/pull-request-template.md`](templates/pull-request-template.md) <!-- forward-reference: wave-1 -->. The template prescribes 5 traceability H2 sections plus the automated reviewer-agent `## Code Review` section, CI-enforced by `traceability-validator` and `oya-pr-review`:

1. `## Issue` — `Closes #<n>` or `Refs #<n>`.
2. `## Summary` — 1–3 bullets on what + why.
3. `## Verification` — pass/fail line per check; reviewer-agent verdict pasted.
4. `## Traceability` — catalog records touched, cross-axis contracts touched, ADRs cited.
5. `## Evidence` — audit-chain emission ID; foundation-bypass (if any); per-pack regulator-watch impact (if any).

The automated reviewer pipeline supplies `## Code Review` with the reviewer-agent name, verdict, and resolved + deferred items. The merge gate refuses any merge without this section.

## Done-Definition checklist

Before declaring any change complete, every agent and every human MUST re-walk these items. Each box has a typed artifact (a command, a lane, or an explicit `(advisory)` marker).

- [ ] **D1** All §"Pre-flight checklist" items checked. *Test:* per-item reviewer audit on PR.
- [ ] **D2** Affected canonical docs updated in this same PR per [`DOC-CATALOG.md`](DOC-CATALOG.md). *Test:* `oya-governance-doc-catalog` lane.
- [ ] **D3** New ADRs (if any) authored from [`templates/adr-template.md`](templates/adr-template.md) <!-- forward-reference: wave-1 --> with all required sections. *Test:* `oya-governance-adr-shape` lane.
- [ ] **D4** New runbooks (if any) authored from [`templates/runbook-template.md`](templates/runbook-template.md) <!-- forward-reference: wave-1 -->; discoverable in [`RUNBOOKS-INDEX.md`](RUNBOOKS-INDEX.md) <!-- forward-reference: wave-1 -->. *Test:* `oya-governance-runbook-index-resolves` lane.
- [ ] **D5** New capabilities (if any) ship: capability record, eval set (golden + adversarial + linguistic), autonomy tier, audit-chain topic, Cosign signing. *Test:* `oya-governance-capability-publish` lane.
- [ ] **D6** New schemas (if any) carry `oyatie.data_class = "..."` per field. *Test:* `oya-governance-data-class` lane.
- [ ] **D7** Per-PR fitness lanes pass: `oya-governance-{license, data-class, cohesion, glossary, adr-citation, brand-residue, bypass, flat-crates, runbook-index-resolves, doc-catalog}`. *Test:* CI status check.
- [ ] **D8** Per-change-class reviewer agent ran; verdict captured in `## Code Review`. *Test:* merge-gate hook (`scripts/hooks/guard-pr-merge-review.mjs`).
- [ ] **D9** `cargo nextest run --workspace --all-features --no-fail-fast` passes. *Test:* command output pasted in `## Verification`.
- [ ] **D10** `cargo clippy --workspace --all-features --all-targets -- -D warnings` passes. *Test:* command output.
- [ ] **D11** `cargo deny check` passes. *Test:* command output.
- [ ] **D12** Required `oya-ci-required` context and Rust gate packets pass for the change class. *Test:* required
  status/evidence bundle. Legacy `oya verify` output is optional local mirror evidence, never a
  completion/merge authority.
- [ ] **D13** Performance changes carry benchmark + ≥2 stress-scenario evidence. *Test:* `oya-governance-perf-evidence` lane.
- [ ] **D14** Schema migrations ship up + down + dry-run + per-tenant + per-cell rollback. *Test:* `oya-governance-schema-migration` lane.
- [ ] **D15** PR body has all 5 canonical traceability H2 sections plus automated `## Code Review`. *Test:* `traceability-validator` + `oya-pr-review` lanes.
- [ ] **D16** Audit-chain emission `EVT-*` ID referenced in `## Evidence`. *Test:* `oya-governance-audit-emission` lane.
- [ ] **D17** [`MISTAKES-LEDGER.md`](MISTAKES-LEDGER.md) <!-- forward-reference: wave-1 --> row added if this change is a mechanical prevention shipped for a prior failure. *Test:* `oya-governance-mistakes-ledger-cite` lane.
- [ ] **D18** [`CHANGELOG.md`](CHANGELOG.md) <!-- forward-reference: wave-1 --> row added if this change touches a canonical doc. *Test:* `oya-governance-changelog-row` lane.
- [ ] **D19** Post-merge product-completion packet recorded after squash merge:
  promoted commit `oya-ci-required` status URL, rollout verification, rollback note,
  observability/golden-signal check, browser UX/user-story evidence, and
  release-governance/release-note impact (Release Please applies only when a live repo config/workflow exists).
  *Test:* PR comment or release evidence bundle linked
  from `## Evidence`; see [`checklists/pre-merge.md`](checklists/pre-merge.md)
  §"After merge".
- [ ] **D20** Agent observations harvested before closeout: review chat,
  review-agent output, scratch/workspace notes, PR comments, and Kanban
  comments; dedupe against active cards; then create/link follow-up, maturity,
  feature-improvement, or fix cards, or document duplicates/no-action rationale.
  New/linked cards MUST include: source context, classification,
  affected card/PR/artifact, acceptance criteria, verification path,
  suggested owner/profile, and dependencies/conflict notes. *Test:* Kanban card/comment
  links or explicit duplicate/no-action note in `## Evidence` / completion packet.

If any box is unchecked, the change is not complete. Loop back; do not declare success.

## Repository topology

| Path | Purpose |
|---|---|
| [`docs/`](.) | Canonical engineering doc tree. Authority. |
| [`docs/raw/`](raw/) <!-- forward-reference: wave-1 --> | Working drafts. Never authoritative. |
| `{oya,cloud}/<service>/crates/<crate>` + `libs/<lib>/` | Canonical implementation homes per ADR-0131/ADR-0512 platform-readiness amendment. Top-level `crates/` is legacy/removal-candidate until verified migration. |
| `infra/`, `scripts/`, `registry/` | Supporting implementation and governance tree; `registry/catalog/` is the live crate catalog. |
| `modules/`, `services/`, `platform/`, `tools/` | Retired legacy implementation roots; do not recreate. |
| `registry/capability-templates/` | Capability records + metering events (Foundry-consumed). |
| `contracts/` | Per-cross-axis contract spec files (OpenAPI, Protobuf, AsyncAPI). |
| Repo root (`README.md`, `CLAUDE.md`, `AGENTS.md`) | Authoritative entry/operating-contract discovery surfaces. `CLAUDE.md` and this file are binding for agents; `/specs/root-hub-pointers.json` remains the redirect hub. Thinness lint may apply to redirect/index helper files only; it does not demote CLAUDE.md or docs/AGENTS.md. |

## Boundaries

- Every agent MUST NOT touch `/Users/home/Documents/GitHub/claude-code` (read-only reference).
- Every agent MUST preserve user state — no removal of unrelated files, processes, or worktrees.
- Local `AGENTS.md` files (under sub-directories) MAY narrow context but MUST NOT lower the bar set by this canonical contract.
- `docs/raw/` MUST be treated as throwaway. Never cite from `docs/raw/` in canonical docs.
- The implementation rebrand (`oyatie-*` → `oya-*`) MUST proceed as a coordinated multi-batch migration; blanket-sed is forbidden.
- Risky actions (force-push, hard-reset, package downgrade, migration to shared infra, sending external messages) MUST be confirmed with the user before execution unless the user has authorized the scope in advance.

## Long-running loop rule

When operating in a Ralph / autopilot / ultrawork / team loop, the loop MUST re-walk §"Done-Definition checklist" against the latest state before exiting. Loops MUST NOT exit silently.

The cancellation contract is `/oh-my-claudecode:cancel`. Cancel only when the change is complete and verified, OR when the loop is structurally blocked.

## Per-agent appendices

Each appendix is ≤40 lines. Per-agent harness deltas only — no rule duplication from above.

### Claude Code <a id="claude-claude-code"></a>

The Claude Code harness loads `CLAUDE.md` at session start (memory-bootstrap convention per [Anthropic docs](https://docs.anthropic.com/en/docs/claude-code/memory)). Repo-root `CLAUDE.md` is a Redirect-class file pointing to this contract.

Always-loaded skills (project-level): `coding-standards`, `tdd-workflow`, `superpowers:test-driven-development`, `superpowers:verification-before-completion`, `superpowers:systematic-debugging`, `search-first`. Language and domain skills load from file context (`rust-*`, `frontend-*`, `postgres-patterns`, `healthcare-phi-compliance`).

Active hooks (PreToolUse / PostToolUse / Stop / SessionStart): merge-review gate (`scripts/hooks/guard-pr-merge-review.mjs`), pre-push gate, telemetry, loop-cancellation enforcement, memory bootstrap.

Legacy OMC magic-keyword routing remains compatibility-only while the plain-git/GitHub/cloud-ci closeout path finishes landing. It does not own forward repo-state closure; branch protection, cloud-ci required checks, and governance admission do. Retired Jenkins/`oya` bridge references are historical/local evidence only, never current authority. Detail in [`standards/claude-code-harness.md`](standards/claude-code-harness.md) <!-- forward-reference: wave-2 -->.

Cancellation: `/oh-my-claudecode:cancel` only after re-walking §"Done-Definition checklist."

Boundary: do not edit `~/.claude/` from project sessions — user-machine state.

Self-test: `npm --prefix /Users/home/.codex test` before relying on hook / harness changes.

### Codex (OpenAI Codex CLI)

The Codex CLI loads `AGENTS.md` at workspace creation, per the cross-tool [AGENTS.md convention](https://agents.md). Repo-root `AGENTS.md` is a Redirect-class file pointing to this contract.

Build / test commands: `cargo build`, `cargo test`, `cargo nextest run --workspace --all-features --no-fail-fast`, `cargo clippy --all-features --all-targets -- -D warnings`, `pnpm build`, `pnpm test` (Node 20). Lint: `cargo clippy`, `pnpm lint`.

Active integration: `.codex/skills/` holds project skills. Coordination follows §Sanctioned primitives; workspace setup is owned by the runtime and claim lifecycle, not by repo-local bootstrap scripts.

Cancellation: terminate the Codex run; the orchestrator records the partial state for replay.

### Gemini (Gemini CLI)

The Gemini CLI loads `GEMINI.md` if present at repo root, else falls back to `AGENTS.md`. If admitted, repo-root `GEMINI.md` is a Redirect-class file pointing to this contract.

Tool mapping: Gemini uses different tool names than Claude Code; the cross-tool AGENTS.md spec gives the mapping (also embedded in [`standards/multi-agent-tool-map.md`](standards/multi-agent-tool-map.md) <!-- forward-reference: wave-2 -->).

Build / test commands: same as Codex appendix.

Cancellation: terminate the Gemini run; same orchestrator-replay semantics.

### Legacy OMC (oh-my-claudecode subagents)

OMC subagents run inside Claude Code via `Skill` / `Agent` tool calls. This surface is compatibility-only for existing sessions and historical evidence; new agentic closeout routes through plain `git`, GitHub Actions + branch protection as the ADR-0515 live runner, the protected `oya-ci-required` context, and reviewer governance evidence. Retired Jenkins/`oya gate` references are historical/local evidence only, never merge authority.

Subagent catalog: `executor`, `architect`, `verifier`, `code-reviewer`, `silent-failure-hunter`, `tdd-guide`, `doc-updater`, `planner`, `critic`, `debugger`, `tracer`, `explore`, `designer`, `writer`, `qa-tester`. Route per change class.

Skill catalog: `/oh-my-claudecode:autopilot`, `/ralph`, `/team`, `/ultrawork`, `/verify`, `/cancel`, `/ralplan`, `/deep-interview`, `/trace`, `/plan`. Cancellation: see "Long-running loop rule" above.

State: legacy OMC writes to `.omc/state/`, `.omc/notepad.md`, `.omc/project-memory.json`, `.omc/plans/`, `.omc/research/`, `.omc/logs/`. Treat `.omc/` and `.omx/` as local-only, gitignored session state/provenance; live machine-readable authority belongs under `/specs`, `/registry`, `/evidence`, and `/templates`.

## Anti-overlap

This contract does not cover:

- **Machine-readable authority registry** — see [`/specs/root-hub-pointers.json`](..//specs/root-hub-pointers.json).
- **Per-doc lifecycle protocol** — see [`DOC-CATALOG.md`](DOC-CATALOG.md).
- **PR template body** — see [`templates/pull-request-template.md`](templates/pull-request-template.md) <!-- forward-reference: wave-1 -->.
- **Architectural rationale per decision** — see [`decisions/`](decisions/) <!-- forward-reference: wave-1 --> indexed at [`ADR-INDEX.md`](ADR-INDEX.md) <!-- forward-reference: wave-1 -->.
- **Per-team norms** — see [`teams/`](teams/) <!-- forward-reference: wave-1 -->.
- **Surface enumeration** — see [`SPEC.md`](SPEC.md) <!-- forward-reference: wave-1 -->.
- **Cross-cutting authoring norms** (code style, testing, security review, etc.) — see [`standards/`](standards/) <!-- forward-reference: wave-1 -->.
- **Failure-mode catalog** — see [`MISTAKES-LEDGER.md`](MISTAKES-LEDGER.md) <!-- forward-reference: wave-1 -->.

The full machine-readable list is in this file's front-matter `excludes:` block.

## Sources scanned

- 2026-05-10 — initial draft authored from agentic-workflow best practice (Anthropic CLAUDE.md memory + cross-tool AGENTS.md convention) + RFC-2119 + RFC-8174 + Diátaxis + openai/symphony benchmark.
