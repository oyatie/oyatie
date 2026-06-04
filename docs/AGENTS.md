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
    > /specs/root-hub-pointers.json
    > docs/AGENTS.md (until /specs/agent-operating-contract.json PHASE-5 promotion)
    > tools/agent-skills/AGENTS.md (inherited base from addyosmani/agent-skills MIT — universal intent→skill mapping, anti-rationalization, persona/skill/command orchestration; oyatie overlays via this file and wins on conflict)
    > machine-readable specs and registries under .omc/
    > docs/ authority files during markdown-retirement compatibility
    > tools/agent-skills/CLAUDE.md (informational; describes vendored subtree, not oyatie)
    > repo-root Redirect-class files (non-authoritative; lane-thin)
    > working drafts (never authoritative)
purpose: "Operating-Contract: Oyatie Agent Operating Contract."
doc_status: published
---
# Oyatie Agent Operating Contract

## ADR-0516 interim lane-unlocker amendment (2026-06-03)

For imminent P00/P0 work, GitHub/GitHub Actions is the temporary lane-unlocker for dev so product, infra, and cloud lanes can run concurrently. There is no retired external SCM/CI/CD substrates interim authority. Buck2 remains build/test/check authority. Exact tombstones for retired external substrate names live in `/specs/retired-external-substrate-registry.json` so active guidance can stay generic. The native destination remains a cloud native, Kubernetes-native, hyperscaler native Oyatie SCM/CI/CD/cloud workspace substrate that adopts proven Prow/Sapling/Piper/CitC/Buck2 patterns without wholesale reinvention. This amendment is not P0.0 green and does not make GitHub permanent.

## Machine-readable authority — [root hub pointers](..//specs/root-hub-pointers.json)

## Workspace doctrine — applies to every documentation / file / workflow

Canonical doctrine: [`/specs/oyatie-doctrine.json`](..//specs/oyatie-doctrine.json) v1.0.0. **Principles P0..P9** (agentic-primary, machine-optimized, programmatic-where-possible, deterministic-where-it-matters, enforce-in-every-thing, iterate-until-consensus, no-silent-regression, Bominal-inheritance, canonical-base-+-localization, no-sprawl) bind every PR.

Workflow Studio product surface inverts P0 (human-ergonomic-first, no-code-first, SDK as enrichment). See `oyatie-doctrine.json#scope_clarifications`.

## Wave 15-ZF doctrine refs — ADR-0346..ADR-0349

Every agent MUST treat [ADR-0346](decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md), [ADR-0347](decisions/ADR-0347-foundry-fitness-to-governance-bulk-rename.md), and [ADR-0348](decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md) as active operating-contract doctrine until superseded by a newer ADR. ADR-0349 is historical CI/CD provenance and is superseded for interim use by ADR-0516 plus `/specs/retired-external-substrate-registry.json`.

| ADR | Operating-contract binding | Enforced-by lanes agents MUST preserve |
|---|---|---|
| ADR-0346 (amended 2026-06-02) | Local pre-push verification is Buck2-backed shift-left evidence only. It MUST NOT grant protected-branch or Phase-0 exit authority; during ADR-0516 the automated `github-lane-unlocker-required` context gates dev, while `oya-ci-required` remains the native cutover target only. | `specs/buck2-authority-policy.json`, `//:buck2-authority-policy-check`, `infra/ci/buck2-affected-gate.sh`, `//:github-lane-unlocker-bridge-check`, future cloud-ci/oya-ci required-context producer. |
| ADR-0347 | Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request; the rename is name-only and lane invariants remain preserved. | `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, `oya-governance-rename-inventory-presence`. |
| ADR-0348 | Cellular topology MUST support autosharding, auto-rebalance, and dynamic sharding through manifest-declared `sharding_automation` blocks, honoring residency, reversibility, and audit-chain emission. | `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, `oya-governance-tenant-migration-reversibility`. |
| ADR-0349 (superseded for interim use by ADR-0516) | Historical retired external CI/CD self-hostable CI/CD planning is not interim authority. ADR-0516 makes GitHub/GitHub Actions the temporary dev lane-unlocker, keeps Buck2 as build/test/check authority, rejects retired external SCM/CI/CD substrates interim authority, and preserves native cloud native/Kubernetes-native/hyperscaler-native cutover. | `//:github-lane-unlocker-bridge-check`, `//:buck2-authority-policy-check`, `infra/ci/buck2-affected-gate.sh`, ADR-0516 claim-boundary evidence lanes. |


## P00 repo hygiene automation

Git/worktree, branch/merge, repository publication, disk/workspace, Kubernetes workload, and documentation-sprawl hygiene are governed by `/specs/repo-hygiene-automation.json` and checked with:

```sh
buck2 build //:repo-hygiene-automation-check
```

Shared docs, root indexes, registries, and workflows stay pointer-thin and should route detail to disjoint lane-owned shards. New Markdown defaults to registered/lane-owned or archived; stale docs older than 3 days are audit/archive candidates before deletion.

## Multispectrum review bar — required on every change

Every changeset (agentic OR human-authored) MUST emit a multispectrum evidence file at `/evidence/multispectrum/<change_id>-<unix_ts>.json` conforming to [`/specs/multispectrum-review.json`](..//specs/multispectrum-review.json) v2.4.0 (`evidence_schema`). The governance gate (`oya gate run-all`, run locally and in the ADR-0516 temporary GitHub/GitHub Actions bridge until native cutover — `oya` is a governance-gate engine, not a VCS) plus the seam-discipline lane `oya-check-dependency-seam` REFUSE the changeset when:

- evidence file absent OR
- declared `change_class_id` not in {CC-1..CC-7} OR
- required facets (F1..F13 except F12-reserved when applicable; A-family policy-adherence facets for policy-touching changes; plus `M1`/`M2` when `meta_review_triggered`) missing OR
- mandatory artifacts per the rigor matrix missing.

This applies to **agentic flow** AND **dev flow**. Agentic flow is the primary consumer; the spec is read at PR-open / gate-run time. Plain `git` + the ADR-0516 temporary GitHub/GitHub Actions lane-unlocker + Buck2 + `oya gate` is the interim canonical path; ADR-0363 records the retired bespoke ratchet. It is re-evaluated each iterative-fix-loop cycle. See [`docs/standards/multispectrum-review.md`](standards/multispectrum-review.md) for the human gateway and [`/registry/fixuptasks.jsonl`](..//registry/fixuptasks.jsonl) for the bounded-deferral registry.

This is the single contract every agent (Claude Code, Codex, Gemini, OMC subagents, Foundry capabilities) and every human contributor honors before changing the repository. It is dual-audience: every directive is simultaneously a human-readable instruction and a machine-extractable typed artifact (RFC-2119 keyword + named path / lane / validator).

Before changing this repo, read `/specs/root-hub-pointers.json` first, then this contract. The retired Constitution concept is redistributed through the root hub, master-plan specs, RACI ownership, and sanctioned-primitive specs.

## Authority precedence

The higher source wins on conflict.

```
system / developer / user instructions
  > /specs/root-hub-pointers.json
  > docs/AGENTS.md (until /specs/agent-operating-contract.json PHASE-5 promotion)
  > tools/agent-skills/AGENTS.md (inherited base from addyosmani/agent-skills MIT)
  > machine-readable specs and registries under .omc/
  > docs/ authority files during markdown-retirement compatibility
  > tools/agent-skills/CLAUDE.md (informational; describes vendored subtree, not oyatie)
  > repo-root Redirect-class files (non-authoritative; lane-thin)
  > working drafts (never authoritative)
```

The chain is mirrored from `/specs/root-hub-pointers.json` and the markdown-retirement policy. The `oya-governance-authority-cohesion` lane validates pointer cohesion during PHASE-5 migration.

`tools/agent-skills/AGENTS.md` is the inherited base from `addyosmani/agent-skills` (MIT) — universal intent→skill mapping, anti-rationalization, persona/skill/command orchestration. Oyatie governance (this file) OVERLAYS and WINS on conflict per Bominal-inheritance precedence (`feedback_bominal_inheritance_precedence`). See `tools/agent-skills/INHERITANCE.md` for the full pattern.

## RFC-2119 normative-language statement

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **NOT RECOMMENDED**, **MAY**, and **OPTIONAL** in this document are to be interpreted as described in BCP 14 [[RFC2119](https://www.rfc-editor.org/rfc/rfc2119)] [[RFC8174](https://www.rfc-editor.org/rfc/rfc8174)] when, and only when, they appear in all capitals, as shown here.

Lowercase forms ("you must", "should consider") have their normal English meanings and carry no normative force.

## Canonical doc map

For any question, route to its authority. Click the link; do not duplicate inline.

| Question | Authority |
|---|---|
| Intent→skill mapping, lifecycle phases, anti-rationalization, persona/skill/command orchestration | [`tools/agent-skills/AGENTS.md`](../tools/agent-skills/AGENTS.md) (inherited from addyosmani/agent-skills MIT — oyatie overlays via this file; see [`tools/agent-skills/INHERITANCE.md`](../tools/agent-skills/INHERITANCE.md)) |
| Universal skill catalog (23 lifecycle skills) | [`tools/agent-skills/skills/`](../tools/agent-skills/skills/) |
| Reusable agent personas (code-reviewer, security-auditor, test-engineer) | [`tools/agent-skills/agents/`](../tools/agent-skills/agents/) |
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
5. **Confirm license posture.** New dependencies MUST clear the Buck2 license-policy lane. AGPL / GPL / SSPL / BUSL / RSAL are not permitted in product code. *Why:* license drift is hard to undo. *Test:* Buck2-invoked license-policy evidence exits 0.
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
- **Buck2 for dev-loop and evidence.** Prefer Buck2 build/test targets and `infra/ci/buck2-affected-gate.sh` for fast feedback. Final local evidence runs Buck2 affected/full targets; the ADR-0516 automated `github-lane-unlocker-required` context is interim merge authority, and `oya-ci-required` returns only after native cutover evidence.
## Sanctioned primitives

Agent coordination uses plain `git`. ADR-0363 retires the prior wrapper/ratchet
substrate; do not reintroduce an agentic VCS wrapper. An agent works on an
isolated worktree branch and opens a pull request against `dev`, which enters
the governance pipeline:
ADR-0516 temporary GitHub/GitHub Actions lane-unlocker context + reviewer
APPROVE gate merge readiness until the native Oyatie SCM/CI/CD substrate cuts
over. The destination is a pure-Rust, Sapling-compatible Oyatie SCM plus
cloud-native release conveyor seams, not interim retired external SCM/CI/CD substrates. `oya`
is local/bridge governance evidence only.

The fenced block below is the machine-readable agent surface. Human-facing terminal examples may live outside fences.

<!-- agent-instructions:start -->
sanctioned_primitives:
  - git
  - buck2
  - oya-gate-local-evidence
  - oya-verify-local-evidence
required_sequence:
  - isolated worktree branch per agent lane (scaffold-managed; one lane = one worktree)
  - commit and push on that lane
  - open a PR against dev               # enters the governance pipeline
  - automated github-lane-unlocker-required GitHub Actions context + Buck2 evidence + reviewer APPROVE gate merge readiness until native cutover
scaffold_protocol:
  mechanism: per-agent isolated worktree plus admission-gate concurrent-safe-paths
  adr: docs/decisions/ADR-0363-retire-agentic-vcs-foundry-to-intelligence-forgejo-substrate.md
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
- [ ] **D9** Buck2 test evidence passes for the affected/full target set. *Test:* `buck2 test ...` or `infra/ci/buck2-affected-gate.sh` output pasted in `## Verification`.
- [ ] **D10** Buck2 lint/clippy target evidence passes with warnings treated as failures. *Test:* Buck2 output.
- [ ] **D11** Buck2-native license-policy evidence passes. *Test:* lane output.
- [ ] **D12** Buck2 authority policy passes and the interim `github-lane-unlocker-required` context is green before merge; native `oya-ci-required` returns only after cutover evidence. *Test:* policy output + required-context evidence.
- [ ] **D13** Performance changes carry benchmark + ≥2 stress-scenario evidence. *Test:* `oya-governance-perf-evidence` lane.
- [ ] **D14** Schema migrations ship up + down + dry-run + per-tenant + per-cell rollback. *Test:* `oya-governance-schema-migration` lane.
- [ ] **D15** PR body has all 5 canonical traceability H2 sections plus automated `## Code Review`. *Test:* `traceability-validator` + `oya-pr-review` lanes.
- [ ] **D16** Audit-chain emission `EVT-*` ID referenced in `## Evidence`. *Test:* `oya-governance-audit-emission` lane.
- [ ] **D17** [`MISTAKES-LEDGER.md`](MISTAKES-LEDGER.md) <!-- forward-reference: wave-1 --> row added if this change is a mechanical prevention shipped for a prior failure. *Test:* `oya-governance-mistakes-ledger-cite` lane.
- [ ] **D18** [`CHANGELOG.md`](CHANGELOG.md) <!-- forward-reference: wave-1 --> row added if this change touches a canonical doc. *Test:* `oya-governance-changelog-row` lane.

If any box is unchecked, the change is not complete. Loop back; do not declare success.

## Repository topology

| Path | Purpose |
|---|---|
| [`docs/`](.) | Canonical engineering doc tree. Authority. |
| [`docs/raw/`](raw/) <!-- forward-reference: wave-1 --> | Working drafts. Never authoritative. |
| [`crates/`](../crates/) <!-- forward-reference: wave-1 --> | Flat-crates target: `oya-<context>-<role>[-<capability>]/`. |
| `infra/`, `scripts/`, `registry/` | Supporting implementation and governance tree; `registry/catalog/` is the live crate catalog. |
| `modules/`, `services/`, `platform/`, `tools/` | Retired legacy implementation roots; do not recreate. |
| `registry/capability-templates/` | Capability records + metering events (Foundry-consumed). |
| `contracts/` | Per-cross-axis contract spec files (OpenAPI, Protobuf, AsyncAPI). |
| Repo root (`README.md`, `CLAUDE.md`, `AGENTS.md`) | Redirect-class discovery files. Non-authoritative. ≤25 lines each. Lane: `oya-governance-redirect-thinness`. |

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

Legacy OMC magic-keyword routing remains compatibility-only while the plain-git plus ADR-0516 temporary GitHub/GitHub Actions closeout path finishes landing. It does not own forward repo-state closure; GitHub branch protection, GitHub Actions temporary required checks, Buck2 evidence, and governance admission do. Detail in [`standards/claude-code-harness.md`](standards/claude-code-harness.md) <!-- forward-reference: wave-2 -->.

Cancellation: `/oh-my-claudecode:cancel` only after re-walking §"Done-Definition checklist."

Boundary: do not edit `~/.claude/` from project sessions — user-machine state.

Self-test: `npm --prefix /Users/home/.codex test` before relying on hook / harness changes.

### Codex (OpenAI Codex CLI)

The Codex CLI loads `AGENTS.md` at workspace creation, per the cross-tool [AGENTS.md convention](https://agents.md). Repo-root `AGENTS.md` is a Redirect-class file pointing to this contract.

Build / test commands: `buck2 build`, `buck2 test`, `infra/ci/buck2-affected-gate.sh github-mirror/dev HEAD`, `pnpm build`, `pnpm test` (Node 24 LTS by default; Node 26 Current only when a lane explicitly needs it). Lint: Buck2 lint targets, direct `rustfmt` when needed, `pnpm lint`.

Active integration: `.codex/skills/` holds project skills. Coordination follows §Sanctioned primitives; workspace setup is owned by the runtime and claim lifecycle, not by repo-local bootstrap scripts.

Cancellation: terminate the Codex run; the orchestrator records the partial state for replay.

### Gemini (Gemini CLI)

The Gemini CLI loads `GEMINI.md` if present at repo root, else falls back to `AGENTS.md`. If admitted, repo-root `GEMINI.md` is a Redirect-class file pointing to this contract.

Tool mapping: Gemini uses different tool names than Claude Code; the cross-tool AGENTS.md spec gives the mapping (also embedded in [`standards/multi-agent-tool-map.md`](standards/multi-agent-tool-map.md) <!-- forward-reference: wave-2 -->).

Build / test commands: same as Codex appendix (Buck2-first; Cargo only for documented production release optimization evidence).

Cancellation: terminate the Gemini run; same orchestrator-replay semantics.

### Legacy OMC (oh-my-claudecode subagents)

OMC subagents run inside Claude Code via `Skill` / `Agent` tool calls. This surface is compatibility-only for existing sessions and historical evidence; new agentic closeout routes through plain `git`, ADR-0516 temporary GitHub/GitHub Actions required checks, Buck2 evidence, and `oya gate` governance evidence.

Subagent catalog: `executor`, `architect`, `verifier`, `code-reviewer`, `silent-failure-hunter`, `tdd-guide`, `doc-updater`, `planner`, `critic`, `debugger`, `tracer`, `explore`, `designer`, `writer`, `qa-tester`. Route per change class.

Skill catalog: `/oh-my-claudecode:autopilot`, `/ralph`, `/team`, `/ultrawork`, `/verify`, `/cancel`, `/ralplan`, `/deep-interview`, `/trace`, `/plan`. Cancellation: see "Long-running loop rule" above.

State: legacy OMC writes to `.omc/state/`, `.omc/notepad.md`, `.omc/project-memory.json`, `.omc/plans/`, `.omc/research/`, `.omc/logs/`. Treat as session-scoped/provenance unless an existing tracked milestone artifact is being superseded by governance evidence in the plain-git/GitHub-Actions-temporary-bridge path.

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
