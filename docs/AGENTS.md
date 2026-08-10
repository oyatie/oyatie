---
doc_class: Operating-Contract
shape: null
length_cap: 500
authority_tier: 2
excludes:
- path: /specs/root-hub-pointers.json
  reason: Machine-readable entry-point registry; this contract is discovered through
    it.
- path: /specs/master-plan-sequencing.json
  reason: Historical sequencing sidecar; after masterplan v2 it is provenance-only and
    not a live plan authority.
- path: /specs/markdown-retirement-policy.json
  reason: Markdown lifecycle and root-hub survival policy; cited and not duplicated
    fully.
- path: docs/DOC-CATALOG.md
  reason: Legacy pre-PHASE-5 projection and trigger taxonomy; non-authoritative migration input only.
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

Every agent MUST treat [ADR-0346](decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md), [ADR-0347](decisions/ADR-0347-governance-fitness-bulk-rename.md), [ADR-0348](decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md), and [ADR-0349](decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md) as active operating-contract doctrine for their non-CI obligations until superseded or amended by a newer ADR. For CI/CD enforcement, [ADR-0515](decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md) is the current single-truth amendment: GitHub Actions + branch protection are the live runner/authority until explicit owned-runner cutover, the cloud-ci Rust gate apps produce the one protected `oya-ci-required` context, and ADR-0513/Prow/Jenkins/legacy `oya` CLI governance wording is superseded provenance or local-feedback evidence only. No GitHub Actions status/check outside `oya-ci-required` may be promoted as protected-branch authority.

| ADR | Operating-contract binding | Enforced-by lanes agents MUST preserve |
|---|---|---|
| ADR-0346 (amended by ADR-0515/platform-readiness) | ADR-0346's retired `./bin/oya verify --ci-required` / `oya verify` paths are historical/provenance-only. Do not invoke or recreate the tracked `bin/oya` shim. The old full-mirror semantics survive only as migration input while porting them into cloud-ci/Rust gate contexts, and must never be extended or treated as protected-branch merge/exit authority. | The only merge authority is the single protected `oya-ci-required` context plus Rust gate packets; do not add new `oya` CLI CI authority. |
| ADR-0347 | Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request; the rename is name-only and lane invariants remain preserved. | `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, `oya-governance-rename-inventory-presence`. |
| ADR-0348 | Cellular topology MUST support autosharding, auto-rebalance, and dynamic sharding through manifest-declared `sharding_automation` blocks, honoring residency, reversibility, and audit-chain emission. | `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, `oya-governance-tenant-migration-reversibility`. |
| ADR-0349 (amended by ADR-0359/ADR-0361/ADR-0515) | Jenkins (LTS) and Prow-shaped wording are bridge/historical substrates, not destination CI authority. GitHub Actions is the current ADR-0515 live runner/producer for the canonical cloud-ci pipeline until explicit owned-runner cutover, not a separate parallel authority; ArgoCD/Argo Rollouts remain CD bridge/reference adapters where separately authorized. | Preserve existing bridge lanes only as transition evidence; do not add new Jenkins/Groovy, Prow, legacy `oya` CLI CI authority, or any GitHub Actions status/check outside `oya-ci-required` as protected-branch authority. Destination lanes are cloud-ci Rust gate packets surfaced through `oya-ci-required` plus ArgoCD tenant-isolation/deploy audit lanes. |

## Independent review discipline — active; multispectrum file convention retired

The deleted `/specs/multispectrum-review.json` evidence-file convention is retired with the external coordination / Oya VCS / Jenkins-adapter admission path (ADR-0116, ADR-0363, ADR-0515; see commit `fd06b0ad2`). Agents MUST NOT emit new `evidence/multispectrum/*.json` files or treat that deleted spec as a live gate.

The review practice survives: run independent reviewer-agent passes and preserve concrete review evidence in the PR's `## Code Review` / quality-gate artifacts. Multi-lens review remains encouraged for high-risk work, but it is expressed through reviewer agents, cloud-ci/oya-ci gate packets, and typed quality-gate evidence — not through standalone multispectrum evidence files.

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

## Doctrine survival (binding)

INV-DOC-9: doctrine that exists only in a plan file or chat is **not** survived. It MUST live here plus the owning ADR / envelopes / PORTABLE surfaces agents actually load. Cite the Amendment C operating-patterns catalog and the reflection corpus for provenance — never external brand or corpus names.

### Rules carry why

- **achieves:** stop blind obedience and silent drift of load-bearing MUST rules.
- **origin:** why-less rules became cargo-cult; failures could not be challenged.
- **rule:** every load-bearing MUST records five fields — achieves, origin, rule, ensure, overturn_when. Rules are hypotheses amended via challenge → OVERRULE → version bump; never silent drift.
- **ensure:** reviewer audit of five-field presence on new MUST; `#anti_drift` version bump on OVERRULE.
- **overturn_when:** a recorded challenge shows the five fields false or incomplete AND a replacement rule with five fields lands same-wave.

### Anti-drift core pointers

- **achieves:** single enumeration SSOT; prose never re-lists envelope contents.
- **origin:** duplicated root/hub/freeze lists drifted from policy-as-data.
- **rule:** every material change declares `docs_touched[]` + `docs_action`; enumerations live ONLY in [`specs/integ-branch-envelopes.json`](../specs/integ-branch-envelopes.json) — cite JSON pointers under `#anti_drift`, `#roots`, `#planes`, `#hubs.paths`, and the other keys listed at `#anti_drift.prose_must_cite_not_enumerate` (do **not** re-list contents); load-bearing doc updates land same-wave with code; unverified tips marked stale.
- **ensure:** Claim packet fields at `#anti_drift.doc_packet_required_fields`; drift-grep via `#anti_drift.drift_grep`; Done-Definition D2 same-PR doc update.
- **overturn_when:** `#anti_drift.anti_drift_doctrine_version` bumps with a recorded OVERRULE replacing the packet/pointer rules.

### Hindsight + beads awareness (binding)

- **achieves:** no freelanced work; no silent repeat of known-failed patterns.
- **origin:** agents acted without work-item ownership or memory of prior failure.
- **rule:** at Design, cite the owning bead work-item id (`.beads/`, `oyatie-*`) — no bead → create/elevate, don't freelance; consult hindsight (memory recall where available, else the SSOT pre-mortem/discoveries) before acting and never repeat a known-failed pattern without a recorded challenge; at Operate, retain the lesson and update bead state after any friction/fix/OVERRULE. Recalled facts are tips, not truth — re-verify stale SHAs (new HEAD → new evidence).
- **ensure:** commit/PR cites bead id; Operate retain + bead state update on friction.
- **overturn_when:** a first-principles challenge shows the ritual blocks delivery AND a replacement ownership+memory ritual lands with five fields.

### Observation ≠ APPROVE; role separation

- **achieves:** preserve merge integrity and blast-radius discipline.
- **origin:** logs/CI green / chat observation treated as APPROVE; roles collapsed.
- **rule:** observation (logs/CI/reviews) ≠ merge APPROVE authority; orchestrate ≠ implement ≠ babysit.
- **ensure:** reviewer APPROVE + green `oya-ci-required` remain distinct; coordinator/worker split in this contract.
- **overturn_when:** a recorded OVERRULE replaces the admission model with an equally fail-closed alternative.

### Survival rule itself (INV-DOC-9)

- **achieves:** doctrine survives across agent sessions.
- **origin:** plan-only / chat-only law vanished when sessions reset.
- **rule:** doctrine MUST live in this contract + owning ADR/envelopes/PORTABLE; plan/chat alone is not survived.
- **ensure:** this section present; pointers to ADR-0711 Amendment D, `specs/integ-branch-envelopes.json#anti_drift`, and PORTABLE Amendment D.
- **overturn_when:** PHASE-5 promotion moves the operating contract AND this survival section migrates atomically with evidence.

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
| Project mission, decision rights, prohibited primitives, amendments | [`/specs/masterplan.json`](..//specs/masterplan.json), [`RACI-OWNERSHIP.md`](RACI-OWNERSHIP.md) |
| Bootstrap routing for the canonical tree | [`README.md`](README.md) |
| Architecture, planes, cross-axis contracts, cohesion thesis | [`DESIGN.md`](DESIGN.md) <!-- forward-reference: wave-1 --> |
| Surfaces (capabilities, APIs, events, indexes, ad slots, cloud resources) | [`SPEC.md`](SPEC.md) <!-- forward-reference: wave-1 --> |
| North star, axes, scope, success metrics, decision log | [`PRD.md`](PRD.md) <!-- forward-reference: wave-1 --> |
| Human plan projection / archived roadmap provenance | [`MASTERPLAN.md`](MASTERPLAN.md), [`ROADMAP.md`](ROADMAP.md) |
| Markdown lifecycle and current doc-authority routing | [`/specs/markdown-retirement-policy.json`](../specs/markdown-retirement-policy.json), [`/specs/root-hub-pointers.json`](../specs/root-hub-pointers.json); [`DOC-CATALOG.md`](DOC-CATALOG.md) is legacy migration input only |
| Doc-class taxonomy, voice, dual-audience rules | [`standards/doc-style.md`](standards/doc-style.md) <!-- forward-reference: wave-1 --> |
| Architectural decisions (ADR pack) | [`ADR-INDEX.md`](ADR-INDEX.md) <!-- forward-reference: wave-1 --> |
| Recurring failure modes + mechanical preventions | [`MISTAKES-LEDGER.md`](MISTAKES-LEDGER.md) <!-- forward-reference: wave-1 --> |
| Per-axis product PRDs | [`products/`](products/) <!-- forward-reference: wave-1 --> |
| Per-team charters | [`teams/`](teams/) <!-- forward-reference: wave-1 --> |
| Per-region packs | [`regional-packs/`](regional-packs/) <!-- forward-reference: wave-1 --> |
| Runbooks (incident, DR, on-call, per-service) | [`RUNBOOKS-INDEX.md`](RUNBOOKS-INDEX.md) <!-- forward-reference: wave-1 --> |
| Templates (PR, ADR, capability, runbook, etc.) | [`templates/`](templates/) <!-- forward-reference: wave-1 --> |
| Privacy / security / compliance | [`PRIVACY-PROGRAM.md`](PRIVACY-PROGRAM.md) <!-- forward-reference: wave-1 -->, [`security-program/security-program.json`](security-program/security-program.json) <!-- forward-reference: wave-1 -->, [`COMPLIANCE-MATRIX.md`](COMPLIANCE-MATRIX.md) <!-- forward-reference: wave-1 --> |
| Release / incident / on-call | [`RELEASE-MANAGEMENT.md`](RELEASE-MANAGEMENT.md) <!-- forward-reference: wave-1 -->, [`INCIDENT-MANAGEMENT.md`](INCIDENT-MANAGEMENT.md) <!-- forward-reference: wave-1 -->, [`standards/on-call.md`](standards/on-call.md) <!-- forward-reference: wave-1 --> |
| Glossary (canonical vocabulary) | [`GLOSSARY.md`](GLOSSARY.md) <!-- forward-reference: wave-1 --> |
| Machine-readable mirrors of the catalog | [`machine-readable/`](machine-readable/) <!-- forward-reference: wave-1 --> |

## Bounded delivery and preservation

Parallel work starts only after preparation. Pin the authority and base SHA, assign each lane one
isolated worktree and non-overlapping ownership, name its reviewer and integration order, select
the checks that prove the lane, and record CPU, memory, disk, and IOPS limits. Run one
representative lane through authoring, review, repair, and verification before wider fan-out; a red
or incomplete pilot blocks expansion.

Preservation is lane-first and chronological. Record every lane's base/head, ownership, commands,
test and review results, dependencies, and terminal state. Record empty, no-op, interrupted, and
unknown outcomes explicitly; absence of findings is not evidence of success. Before cleanup, every
enumerated lane MUST resolve to a remotely readable durable anchor; encrypted quarantine stored
off-machine or otherwise durably beyond the machine being wiped, with a verified ciphertext hash
and a successful clean-room decrypt-and-restore traversal using externally recoverable identities;
or documented and reviewed explicit intentional discard, with zero unresolved lanes. Re-query live
Git/GitHub state, prove
each remote archive commit and tree are readable, and retain terminal anchors for every useful
lane. [`/specs/masterplan.json`](../specs/masterplan.json) remains the live work-item source of
truth; GitHub issues are intake, coordination, and blocker mirrors that link to their masterplan v2
items, and [`HANDOFF.md`](../HANDOFF.md) remains a thin redirect.

Only durable useful work belongs in signed remote Git history. Secrets, credential-bearing
machine configuration, raw `.omx`/`.omc` runtime state, caches, and generated build output MUST NOT
be preserved as source. Distill useful local planning into its owning issue or canonical artifact;
archive refs are recovery inputs, never integration bases or current authority.

Evidence-grounded policy, regulatory, and compliance claims identify the exact source and whether
it is primary, its immutable version or retrieval date, effective date, jurisdiction and
applicability, missing or conflicting authority, and the resulting claim ceiling. Revalidate when
the source, date, applicability, or product behavior changes. This method paraphrases operational
lessons reviewed from [Bun's Rust rewrite account](https://bun.com/blog/bun-in-rust) (retrieved
2026-08-04), [gaebal-gajae's archive](https://blog.gaebal-gajae.dev/archive.html) (retrieved
2026-08-04), and
[`it-legal` at `5624ff1`](https://github.com/jclab-joseph/it-legal/tree/5624ff14e673863ec3b5645155742691a74ef152);
none is Oyatie, legal, or product authority.

## Pre-flight checklist

Before any change, every agent and every human MUST complete these items.

1. **Identify the change class.** Feature / bugfix / refactor / migration / docs / chore / capability / plugin / runbook / ADR / pack-update. *Why:* a class-blind change misses class-specific validators. *Test:* PR body's `## Issue` section names the class.
2. **Read the canonical authority for the change class.** Use the §"Canonical doc map" table. *Why:* one-paragraph orientation prevents the most common failure (acting on stale repo memory). *Test:* PR `## Traceability` cites the doc(s) read.
3. **Confirm Data Use Boundary.** Every new field on a kernel struct MUST carry a `data_class` annotation. *Why:* cross-pillar flows that bypass `data_class` violate the cohesion principle. *Test:* `oya-governance-data-class` lane.
4. **Confirm autonomy ceiling.** Capability bindings MUST declare T1 / T2 / T3 / T4 in the capability record. Tier uplift MUST land an accompanying Cedar policy + runtime gate. *Why:* config-flag tier uplift bypasses the audit chain. *Test:* `oya-governance-autonomy-ceiling` lane.
5. **Confirm license posture.** New dependencies MUST clear the Buck2/cloud-ci supply-chain lane. AGPL / GPL / SSPL / BUSL / RSAL are not permitted in product code. *Why:* license drift is hard to undo. *Test:* supply-chain gate target exits 0.
6. **Search MISTAKES-LEDGER for the failure-mode class.** *Why:* re-introducing a fixed defect is a regression. *Test:* PR `## Traceability` cites the relevant `MFL-NNNN` row OR a "no prior row" search note.
7. **Identify the per-change-class reviewer agent.** *Why:* the target reviewer contract signs `## Code Review` at merge time; no signature, no merge once the trusted reviewer producer is live. *Test:* §"Per-change-class reviewer agents" table below; `F-PR5-06` tracks the current live-enforcement gap.
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

The reviewer-agent verdict is `APPROVE` or `REQUEST CHANGES`. The PR body's `## Code Review` section MUST contain the agent name, the verdict, and the resolved + deferred items. GH #983 adds a PR metadata preflight that refuses blocked/pending-review PR title or body markers and refuses merge-ready body validation without this section.

**REVIEW-ADMISSION-GAP-LIVE-BOUNDARY (F-PR5-06):** F-PR5-06 remains open. PR #964 merged with green `oya-ci-required`, empty `reviewDecision`, and only an owner `COMMENTED` review, so the GH #983 title/body packet is not a cloud-enforced review admission gate. It narrows PR metadata hygiene only; formal GitHub `reviewDecision`, reviewer-author separation, and branch-protection drift reconciliation remain tracked by `registry/fixuptasks.jsonl#F-PR5-06`.

## During-change discipline

While the change is in flight, every agent and every human MUST observe these rules.

- **No `--no-verify`, no hook bypass, no signing skip.** Hook failure is a signal; the fix is the underlying issue.
- **No untyped values at API boundaries.** Use the result types prescribed in [`standards/error-handling.md`](standards/error-handling.md) <!-- forward-reference: wave-1 -->.
- **No new struct fields in kernel crates without `data_class`.** Pre-commit blocks; respect it.
- **No quarantining flaky tests without a 14-day fix SLA.** Quarantine assigns the test to the `flaky/` lane; the SLA is tracked.
- **No editing legacy retired paths.** If a path was retired in a consolidation event, do not recreate it.
- **Buck2 for evidence.** Local editor loops are advisory; final evidence comes from targeted `buck2 test` / `buck2 build` plus cloud-ci gate packets per [`standards/testing.md`](standards/testing.md) <!-- forward-reference: wave-1 -->.
- **Portfolio/architecture coordinator / worker split.** The capability-neutral portfolio/architecture coordinator evaluates architecture, system design, completed and upcoming work, maturity gaps, documentation/procedure/process health, regressions, and work-item decomposition/prioritization. Dispatcher-assigned workers execute scoped implementation, review, verification, and PR evidence lanes in isolated worktrees. The coordinator MUST NOT become the default implementation worker unless explicitly assigned as that lane worker.
- **Blockers become work.** A coordinator that finds a blocker MUST create/link a dispatcher-ready resolution card with source context, blocker class, acceptance criteria, verification path, suggested owner/profile, and dependency/conflict notes. Do not convert blockers into ad hoc coordinator implementation unless the coordinator is explicitly assigned as worker for that lane.
- **Autonomous merge boundary.** Autonomous merge authority exists only when the PR is fully reviewed, review threads are resolved, the required `oya-ci-required` context is green, the branch has no merge conflict, and branch protection is satisfied. Green CI alone is insufficient.

## Sanctioned primitives

Agent coordination uses plain `git`. ADR-0363 retires the prior wrapper/ratchet
substrate; do not reintroduce an agentic VCS wrapper. ADR-0515 retires CLI
governance and makes GitHub Actions + branch protection the live CI runner until
explicit owned-runner cutover. An agent works on an isolated worktree branch and
opens a pull request against `dev`, which enters the governance pipeline:
the single protected `oya-ci-required` context + reviewer APPROVE gate merge
readiness. GH #983 folds PR title/body hygiene into `oya-ci-required`, while
F-PR5-06 still owns live review-admission closure. `oya gate` / `oya verify`
output is optional local feedback or provenance only; it is never
protected-branch CI authority.

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
    and single protected `oya-ci-required` context green; PR title/body hygiene flows
    through `oya-ci-required`, while F-PR5-06 still owns live review-admission closure
    and legacy CLI evidence remains optional/local only
  - squash merge after review threads resolve
  - post-merge product-completion packet: promoted SHA `oya-ci-required` green,
    rollout verification, rollback note, observability check, browser UX/user-story evidence,
    and release-governance/release-note impact (Release Please applies only when a live repo config/workflow exists)
coordinator_worker_split:
  coordinator: portfolio/architecture coordinator owns architecture, system design, maturity,
    regression audit, and work-item decomposition/prioritization
  worker: dispatcher-assigned implementation/review worker owns scoped edits, tests,
    review, and PR evidence
  boundary: coordinator is not the default implementation worker unless explicitly assigned
blocker_policy: queue/link dispatcher-ready resolution cards with source context,
  blocker class, acceptance criteria, verification path, suggested owner/profile,
  and dependency/conflict notes unless explicitly assigned as that lane worker
scaffold_protocol:
  mechanism: per-agent isolated worktree plus admission-gate concurrent-safe-paths
  adr: docs/decisions/ADR-0701-monorepo-capability-live-apex.md
<!-- agent-instructions:end -->

## PR shape

Every PR uses [`templates/pull-request-template.md`](templates/pull-request-template.md) <!-- forward-reference: wave-1 -->. The template prescribes 5 traceability H2 sections plus the automated reviewer-agent `## Code Review` section. Target enforcement is `traceability-validator` plus the `oya-ci-required` PR metadata preflight:

1. `## Issue` — `Closes #<n>` or `Refs #<n>`.
2. `## Summary` — 1–3 bullets on what + why.
3. `## Verification` — pass/fail line per check; reviewer-agent verdict pasted.
4. `## Traceability` — catalog records touched, cross-axis contracts touched, ADRs cited.
5. `## Evidence` — audit-chain emission ID; foundation-bypass (if any); per-pack regulator-watch impact (if any).

The automated reviewer pipeline supplies `## Code Review` with the reviewer-agent name, verdict, and resolved + deferred items, and the metadata packet binds the PR title plus the reviewed PR body/traceability sections. The preflight rejects blocked/pending-review title/body markers and missing or negative review evidence before `oya-ci-required` can pass, without closing F-PR5-06's live review-producer gap.

## Done-Definition checklist

Before declaring any change complete, every agent and every human MUST re-walk these items. Each box has a typed artifact (a command, a lane, or an explicit `(advisory)` marker).

- [ ] **D1** All §"Pre-flight checklist" items checked. *Test:* per-item reviewer audit on PR.
- [ ] **D2** Affected canonical docs updated in this same PR. *Test:* current cross-artifact/canonical-JSON checks plus reviewer inspection. `DOC-CATALOG.md` is a legacy projection; a live machine-catalog producer/gate is still a PHASE-5 prerequisite and must not be claimed as active.
- [ ] **D3** New ADRs (if any) authored from [`templates/adr-template.md`](templates/adr-template.md) <!-- forward-reference: wave-1 --> with all required sections. *Test:* `oya-governance-adr-shape` lane.
- [ ] **D4** New runbooks (if any) authored from [`templates/runbook-template.md`](templates/runbook-template.md) <!-- forward-reference: wave-1 -->; discoverable in [`RUNBOOKS-INDEX.md`](RUNBOOKS-INDEX.md) <!-- forward-reference: wave-1 -->. *Test:* `oya-governance-runbook-index-resolves` lane.
- [ ] **D5** New capabilities (if any) ship: capability record, eval set (golden + adversarial + linguistic), autonomy tier, audit-chain topic, Cosign signing. *Test:* `oya-governance-capability-publish` lane.
- [ ] **D6** New schemas (if any) carry `oyatie.data_class = "..."` per field. *Test:* `oya-governance-data-class` lane.
- [ ] **D7** Applicable per-PR fitness lanes actually wired into `oya-ci-required` pass. Historical lane names in prose are not evidence that a producer is live. *Test:* the PR-head `oya-ci-required` job/packet inventory plus the change-class gate mapping.
- [ ] **D8** Per-change-class reviewer agent ran; verdict captured in `## Code Review`. *Test:* `oya-ci-required` PR metadata preflight plus reviewer audit on PR; live review-admission closure remains F-PR5-06.
- [ ] **D9** Targeted `buck2 test <target(s)>` passes. *Test:* command output pasted in `## Verification`.
- [ ] **D10** Targeted `buck2 build <target(s)>` and relevant cloud-ci lint/static-analysis gate packets pass. *Test:* command output.
- [ ] **D11** Buck2/cloud-ci supply-chain lane passes. *Test:* command output or required context evidence.
- [ ] **D12** Required cloud-ci/oya-ci context and Rust gate packets pass for the change class. *Test:* required
  status/evidence bundle. Legacy `oya` CLI output is historical/local advisory only and never a
  completion/merge authority.
- [ ] **D13** Performance changes carry benchmark + ≥2 stress-scenario evidence. *Test:* `oya-governance-perf-evidence` lane.
- [ ] **D14** Schema migrations ship up + down + dry-run + per-tenant + per-cell rollback. *Test:* `oya-governance-schema-migration` lane.
- [ ] **D15** PR body has all 5 canonical traceability H2 sections plus automated `## Code Review`. *Test:* `traceability-validator` and the `oya-ci-required` PR metadata preflight.
- [ ] **D16** Audit-chain emission `EVT-*` ID referenced in `## Evidence`. *Test:* `oya-governance-audit-emission` lane.
- [ ] **D17** [`MISTAKES-LEDGER.md`](MISTAKES-LEDGER.md) <!-- forward-reference: wave-1 --> row added if this change is a mechanical prevention shipped for a prior failure. *Test:* `oya-governance-mistakes-ledger-cite` lane.
- [ ] **D18** [`CHANGELOG.md`](CHANGELOG.md) <!-- forward-reference: wave-1 --> row added if this change touches a canonical doc. *Test:* `oya-governance-changelog-row` lane.
- [ ] **D19** Post-merge product-completion packet recorded after squash merge:
  promoted commit `oya-ci-required` status URL, rollout verification, rollback note,
  observability/golden-signal check, browser UX/user-story evidence, and Release
  Please/release-note impact. *Test:* PR comment or release evidence bundle linked
  from `## Evidence`; see [`checklists/pre-merge.md`](checklists/pre-merge.md)
  §"After merge".

If any box is unchecked, the change is not complete. Loop back; do not declare success.

## Repository topology

| Path | Purpose |
|---|---|
| [`docs/`](.) | Canonical engineering doc tree. Authority. |
| [`docs/raw/`](raw/) <!-- forward-reference: wave-1 --> | Working drafts. Never authoritative. |
| Registered capability roots with `core/`, `ports/`, `adapters/`, `facade/`; `app/<product>/` for multi-capability compositions | Canonical destination topology per ADR-0562 as amended by ADR-0615. Existing `{oya,cloud}/...`, `libs/`, and top-level `crates/` paths are migration inventory until their strangler moves are verified. |
| `infra/`, `scripts/`, `registry/` | Supporting implementation and governance tree; `registry/catalog/` is the live crate catalog. |
| `modules/`, `services/`, `platform/`, `tools/` | Retired legacy implementation roots; do not recreate. |
| `registry/capability-templates/` | Capability records + metering events (Foundry-consumed). |
| `contracts/` | Per-cross-axis contract spec files (OpenAPI, Protobuf, AsyncAPI). |
| Repo root (`README.md`, `CLAUDE.md`, `AGENTS.md`, `HANDOFF.md`) | Founder-authorized Markdown survival set. `HANDOFF.md` is a thin fresh-session redirect only, never a plan/backlog/status authority. `CLAUDE.md` and this file are binding for agents; `/specs/root-hub-pointers.json` remains the redirect hub. Thinness lint may apply to redirect/index helper files only; it does not demote CLAUDE.md or docs/AGENTS.md. |

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

Active hooks — SSOT is [`.claude/settings.json`](../.claude/settings.json), which the `enforcement-liveness` face resolves against `tools/hooks/`; this list is a mirror, not an authority. PreToolUse/Bash: `main-checkout-guard.sh`, `local-authority-enforcer.sh`, `no-cargo-enforcer.sh`, `stale-tool-suggester.sh`. PreToolUse/Task: `pre-dispatch-guide.sh`. PostToolUse/Edit|MultiEdit|Write: `spec-version-pin-suggester.sh`, `adr-orphan-detect.sh`, `vacuous-green-gate-detect.sh`. PostToolUse/Bash|WebFetch|WebSearch: `injection-content-scanner.sh`. Stop: `stop-did-you-forget-suggester.sh`. There is no SessionStart hook, and no merge-review, pre-push, telemetry, loop-cancellation, or memory-bootstrap hook — the prior text named five behaviours and one file (`scripts/hooks/guard-pr-merge-review.mjs`), none of which existed in-tree.

Legacy OMC magic-keyword routing remains compatibility-only while the plain-git/GitHub/cloud-ci closeout path finishes landing. It does not own forward repo-state closure; branch protection, cloud-ci required checks, and governance admission do. Jenkins/`oya` bridge contexts are transitional evidence only. The former harness standard [`standards/claude-code-harness.md`](standards/claude-code-harness.md) is a **retirement tombstone** (ADR-0619 / RR-HARNESS-0619) — not live procedure; use this contract + ADR-0515, and optionally the local `.grok/` mm-delivery kit (not merge authority).

Cancellation: `/oh-my-claudecode:cancel` only after re-walking §"Done-Definition checklist."

Boundary: do not edit `~/.claude/` from project sessions — user-machine state.

Self-test: `npm --prefix /Users/home/.codex test` before relying on hook / harness changes.

### Codex (OpenAI Codex CLI)

The Codex CLI loads `AGENTS.md` at workspace creation, per the cross-tool [AGENTS.md convention](https://agents.md). Repo-root `AGENTS.md` is a Redirect-class file pointing to this contract.

Build / test commands: targeted `buck2 build <target(s)>` and `buck2 test <target(s)>`; UI-only surfaces may also use `pnpm build`, `pnpm test`, and `pnpm lint` (Node 20) as local evidence when relevant.

Active integration: `.codex/skills/` holds project skills. Coordination follows §Sanctioned primitives; workspace setup is owned by the runtime and claim lifecycle, not by repo-local bootstrap scripts.

Cancellation: terminate the Codex run; the orchestrator records the partial state for replay.

### Gemini (Gemini CLI)

The Gemini CLI loads `GEMINI.md` if present at repo root, else falls back to `AGENTS.md`. If admitted, repo-root `GEMINI.md` is a Redirect-class file pointing to this contract.

Tool mapping: Gemini uses different tool names than Claude Code; the cross-tool AGENTS.md spec gives the mapping (also embedded in [`standards/multi-agent-tool-map.md`](standards/multi-agent-tool-map.md) <!-- forward-reference: wave-2 -->).

Build / test commands: same as Codex appendix.

Cancellation: terminate the Gemini run; same orchestrator-replay semantics.

### Legacy OMC (oh-my-claudecode subagents) — compatibility / provenance only

**Not live authority** (ADR-0619, ADR-0116). Do not open new work that depends on OMC/OMX/GJC/Hermes brands as coordination primitives. New agentic closeout routes through plain `git`, GitHub (interim) branch protection, cloud-ci/oya-ci required checks (`oya-ci-required` per ADR-0515), and reviewer governance evidence. Optional local multi-model delivery uses `.grok/` (mm-delivery) when present — process kit only, never merge authority.

OMC subagents (when still running inside an existing Claude Code session) use `Skill` / `Agent` tool calls. Catalog names below are historical inventory for residual sessions, not a forward skill map.

Subagent catalog (historical): `executor`, `architect`, `verifier`, `code-reviewer`, `silent-failure-hunter`, `tdd-guide`, `doc-updater`, `planner`, `critic`, `debugger`, `tracer`, `explore`, `designer`, `writer`, `qa-tester`.

Skill catalog (historical): `/oh-my-claudecode:autopilot`, `/ralph`, `/team`, `/ultrawork`, `/verify`, `/cancel`, `/ralplan`, `/deep-interview`, `/trace`, `/plan`. Cancellation for residual loops: see "Long-running loop rule" above.

State: legacy OMC may write to `.omc/state/`, `.omc/notepad.md`, `.omc/project-memory.json`, `.omc/plans/`, `.omc/research/`, `.omc/logs/`. Treat `.omc/`, `.omx/`, and `.gjc/` as local-only, gitignored session state/provenance; live machine-readable authority belongs under `/specs`, `/registry`, `/evidence`, and `/templates`.

## Anti-overlap

This contract does not cover:

- **Machine-readable authority registry** — see [`/specs/root-hub-pointers.json`](../specs/root-hub-pointers.json).
- **Historical per-doc catalog design** — [`DOC-CATALOG.md`](DOC-CATALOG.md) is non-authoritative migration input until PHASE-5 promotion; current lifecycle/routing comes from [`/specs/markdown-retirement-policy.json`](../specs/markdown-retirement-policy.json) and [`/specs/root-hub-pointers.json`](../specs/root-hub-pointers.json).
- **PR template body** — see [`templates/pull-request-template.md`](templates/pull-request-template.md) <!-- forward-reference: wave-1 -->.
- **Architectural rationale per decision** — see [`decisions/`](decisions/) <!-- forward-reference: wave-1 --> indexed at [`ADR-INDEX.md`](ADR-INDEX.md) <!-- forward-reference: wave-1 -->.
- **Per-team norms** — see [`teams/`](teams/) <!-- forward-reference: wave-1 -->.
- **Surface enumeration** — see [`SPEC.md`](SPEC.md) <!-- forward-reference: wave-1 -->.
- **Cross-cutting authoring norms** (code style, testing, security review, etc.) — see [`standards/`](standards/) <!-- forward-reference: wave-1 -->.
- **Failure-mode catalog** — see [`MISTAKES-LEDGER.md`](MISTAKES-LEDGER.md) <!-- forward-reference: wave-1 -->.

The full machine-readable list is in this file's front-matter `excludes:` block.

## Sources scanned

- 2026-08-10 — INV-DOC-9 doctrine survival (binding) + DOC-UPDATE same-wave co-change amendment; bead `oyatie-dxz.5` under docs-governance epic `oyatie-dxz`; Amendment C operating-patterns catalog / reflection corpus (brand-free).
- 2026-05-10 — initial draft authored from agentic-workflow best practice + RFC-2119 + RFC-8174 + Diátaxis (historical; do not treat external product names as authority).
