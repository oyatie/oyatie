---
id: ADR-0138
status: Superseded
deciders: council-architecture, axis-foundry, ops-sre-reliability, ops-release-management
date: 2026-05-18
owner: council-architecture
supersedes: []
superseded_by: []
supersession_note: "Foundry Strangler to a dead address (microservices/foundry/ no longer exists); Strangler template reusable. Archived per D-DISPOSITIONS-RATIFIED: ARCHIVE-5, C-11."
related: [ADR-0056, ADR-0105, ADR-0110, ADR-0114, ADR-0123, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0134, ADR-0136, ADR-0137]
related_memory: [feedback_no_silent_regression, feedback_bominal_inheritance_precedence]
related_specs:
  - /specs/microservices/foundry.json
  - /specs/per-microservice-flat-layout.json
session_context:
  authored: 2026-05-18
  pattern_source: |
    This ADR follows ADR-0134 (Connect-dissolution Strangler) as its pattern
    template. Both ADRs apply the agent-skills deprecation-and-migration
    SKILL.md §"Strangler Pattern" + §"Verification" to dissolve a multi-
    µservice topology into the consolidated topology required by the parent
    ADR (ADR-0126 for Connect; ADR-0136 for foundry).
purpose: |
  Operationalise the foundry 6-way → 1-way consolidation via Strangler
  Pattern. Govern path remapping, deprecation notices, zero-active-usage
  verification, soak period, and the terminal-state deletion of the six
  prior paths.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0138: Foundry six-path deprecation — Strangler migration

## Status

Accepted — 2026-05-18.

## Date

2026-05-18.

## Context

ADR-0136 consolidates foundry into one µservice at `microservices/foundry/`
with six internal BCs. ADR-0137 names the BCs and inter-BC dependency
rules. This ADR operationalises the migration off the prior six paths:

- `microservices/foundry-runtime/`     → consolidated into `microservices/foundry/`
- `microservices/foundry-supervisor/`  → consolidated into `microservices/foundry/`
- `microservices/foundry-eval/`        → consolidated into `microservices/foundry/`
- `microservices/foundry-evidence/`    → consolidated into `microservices/foundry/`
- `microservices/foundry-guardrails/`  → consolidated into `microservices/foundry/`
- `microservices/foundry-providers/`   → consolidated into `microservices/foundry/`

**Current state** (at ADR acceptance, 2026-05-18):

- The six source dirs have been physically removed by the consolidation
  ChangeSet that accepts ADR-0136 / ADR-0137 / ADR-0138.
- All 493 source artefacts preserved under the consolidated tree (see
  §Verification below).
- Cross-reference audit (`grep -rn` for the six source paths across
  docs/, specs/, registry/, microservices/) reports zero external
  consumer dependencies at consolidation time. The only references that
  survive are inside (a) this ADR + ADR-0136 + ADR-0137 + (b) the
  `bc-sources/<bc>/*.md` archive (where historical per-BC documents
  reference their own former paths — this is intra-archive self-reference,
  not external dependency).

The zero-external-consumer state is a feature of this consolidation
landing as a single atomic operation rather than a phased Strangler:
because the six source µservices were never wired into a live deployment
or consumer surface (M01 launch is post-this-ADR), there are no live
consumers to migrate over a soak period.

However, per `feedback_no_silent_regression.md` and the Strangler
discipline established by ADR-0134, we still apply a formal Strangler
migration to **guarantee** that no future consumer that researches the
git history can naively use the dead paths. The Strangler shape adapted
for "zero current consumers" is:

1. **Phase 0** (instantaneous, this ChangeSet): atomic consolidation —
   the six dirs are removed in the same ChangeSet that authors the
   consolidated tree.
2. **Phase 1** (REPORT-ONLY, 6 months): CI lane reports any incoming
   reference to the six dead paths. Expected count: zero. Any non-zero
   triggers an investigation.
3. **Phase 2** (BLOCKER, after 6-month soak): CI lane refuses any PR
   that adds a reference to the dead paths. Anyone authoring such a
   reference must reroute to `microservices/foundry/`.
4. **Phase 3** (terminal): no further action; the dead paths are
   permanently retired by absence.

## Decision

The foundry 6-way → 1-way consolidation applies the **Strangler Pattern**
(per agent-skills deprecation-and-migration SKILL.md §"Strangler Pattern")
adapted to the zero-current-consumer state via the **atomic-consolidation
variant**.

### Path remapping table

For any historical reference, the remap is:

| Old path | New path |
|---|---|
| `microservices/foundry-runtime/` | `microservices/foundry/` (BC: runtime) |
| `microservices/foundry-supervisor/` | `microservices/foundry/` (BC: supervisor) |
| `microservices/foundry-eval/` | `microservices/foundry/` (BC: eval) |
| `microservices/foundry-evidence/` | `microservices/foundry/` (BC: evidence) |
| `microservices/foundry-guardrails/` | `microservices/foundry/` (BC: guardrails) |
| `microservices/foundry-providers/` | `microservices/foundry/` (BC: providers) |
| `microservices/foundry-<bc>/PRD.md` | `microservices/foundry/PRD.md` (BC-specific detail at `microservices/foundry/bc-sources/<bc>/PRD.md`) |
| `microservices/foundry-<bc>/PHASE-01-*.md` | `microservices/foundry/PHASE-01-FOUNDRY-FOUNDATION.md` (BC-specific archive at `microservices/foundry/bc-sources/<bc>/PHASE-01-*.md`) |
| `microservices/foundry-<bc>/<concern>.md` (threat-model, dpia, compliance, cost-budget, multi-region, incident-response, capacity-model, failure-modes, sdk-plan, competitor-parity-matrix, backfill-replay) | `microservices/foundry/<concern>.md` (BC-specific archive at `microservices/foundry/bc-sources/<bc>/<concern>.md`) |
| `microservices/foundry-<bc>/IP-NNN-*.md` | `microservices/foundry/IP-MMM-<bc>-*.md` where MMM is the consolidated sequential number: runtime IP-001..015, supervisor IP-016..030, eval IP-031..045, evidence IP-046..060, guardrails IP-061..075, providers IP-076..090. |
| `microservices/foundry-<bc>/catalog/<crate>.yaml` | `microservices/foundry/catalog/<crate>.yaml` (filename unchanged — already BC-qualified by crate naming convention) |
| `microservices/foundry-<bc>/runbooks/<runbook>.md` | `microservices/foundry/runbooks/<bc>-<runbook>.md` |
| `microservices/foundry-<bc>/dashboards/<dash>.json` | `microservices/foundry/dashboards/<bc>-<dash>.json` |
| `microservices/foundry-<bc>/capabilities/<cap>.yaml` | `microservices/foundry/capabilities/<bc>-<cap>.yaml` |
| `microservices/foundry-<bc>/slos/<slo>.yaml` | `microservices/foundry/slos/<bc>-<slo>.yaml` |
| `microservices/foundry-<bc>/contracts/openapi/<spec>.yaml` | `microservices/foundry/contracts/openapi/<bc>-<spec>.yaml` |
| `microservices/foundry-<bc>/contracts/asyncapi/<spec>.yaml` | `microservices/foundry/contracts/asyncapi/<bc>-<spec>.yaml` |
| `microservices/foundry-<bc>/contracts/proto/<file>.proto` | `microservices/foundry/contracts/proto/<bc>-<file>.proto` |
| `microservices/foundry-<bc>/policy/<policy>` | `microservices/foundry/policy/<bc>-<policy>` |
| `microservices/foundry-<bc>/iac/helm/<chart>/` | `microservices/foundry/iac/helm/<bc>/<chart>/` |
| `microservices/foundry-<bc>/iac/kustomize/base/kustomization.yaml` | `microservices/foundry/iac/kustomize/base/<bc>/kustomization.yaml` |
| `microservices/foundry-<bc>/iac/kustomize/overlays/pack-kr/kustomization.yaml` | `microservices/foundry/iac/kustomize/overlays/pack-kr/<bc>/kustomization.yaml` |
| `microservices/foundry-<bc>/iac/terraform/<file>` | `microservices/foundry/iac/terraform/<bc>-<file>` |
| `microservices/foundry-<bc>/iac/cedar/<file>` | `microservices/foundry/iac/cedar/<bc>-<file>` |
| `microservices/foundry-<bc>/iac/postgres/migrations/<file>` | `microservices/foundry/iac/postgres/migrations/<bc>-<file>` |
| `specs/products/foundry/*.json` (never existed at consolidation time) | `specs/microservices/foundry.json` (single consolidated spec) |

**Crate names DO NOT change.** Per ADR-0056 v4.1 BNF + ADR-0136 §Decision:
crates remain named `oya-foundry-<bc>-<feature>-<layer>`; only the parent
directory of the µservice changes. Any tool that resolves crates by name
(cargo, cargo tree, oya-check-authority-cohesion) is unaffected.

### Strangler phases (adapted)

#### Phase 0 — Atomic consolidation  *(this ChangeSet)*

- `git mv` all 493 artefacts from the six source dirs into
  `microservices/foundry/` with BC-prefixed filenames per the remap table.
- Author the consolidated top-level documents (PRD, PHASE-01-FOUNDRY-
  FOUNDATION, threat-model, dpia, compliance, cost-budget, multi-region,
  incident-response, capacity-model, failure-modes, sdk-plan,
  competitor-parity-matrix, backfill-replay).
- Preserve per-BC top-level documents under `bc-sources/<bc>/`.
- Renumber + BC-tag the 90 IPs.
- Consolidate spec to `specs/microservices/foundry.json`.
- Delete the six source directories (now empty after `git mv`).
- Author ADR-0136 / ADR-0137 / ADR-0138.

**Entry gate:** ADR-0136 + ADR-0137 + ADR-0138 drafts complete.
**Exit gate:** Verification §Verification §Phase-0 items below all green.

#### Phase 1 — Soak (REPORT-ONLY)  *(6 months: 2026-05-18 → 2026-11-18)*

- CI lane `oya-governance-foundry-six-path-zero-usage` registered as
  REPORT-ONLY in `.github/branch-protection.yaml` (follow-up
  ChangeSet).
- Lane runs `git grep -E 'microservices/foundry-(runtime|supervisor|eval|evidence|guardrails|providers)'`
  excluding (a) this ADR, ADR-0136, ADR-0137 and (b) the bc-sources/
  archive. Expected match count: 0.
- Any non-zero match in REPORT-ONLY mode pages axis-foundry for
  investigation; this is how a re-introduction would be detected.

**Entry gate:** Phase 0 exit gate green.
**Exit gate:** 6 months elapsed since Phase 0 exit AND no non-zero
match has been reported during the soak.

#### Phase 2 — Enforce (BLOCKER)  *(post-2026-11-18)*

- CI lane promoted to BLOCKER. Any PR introducing a reference to the
  six dead paths refuses to merge until the reference is reformulated
  to use the consolidated paths per the remap table.

**Entry gate:** Phase 1 exit gate green.
**Exit gate:** Phase 2 has no terminal exit; it runs indefinitely.

#### Phase 3 — Terminal *(no action)*

The dead paths are permanently retired by absence. No follow-up ChangeSet
or removal sweep is required (the six dirs were already removed at
Phase 0). This phase exists only to mark in the ADR that we have reached
the terminal Strangler state.

## Alternatives Considered

### (a) Big-Bang consolidation without ADR or lane

- **Pros**:
  - Lowest ceremony.
  - One ChangeSet ships everything.
- **Cons**:
  - Violates `feedback_no_silent_regression.md` — public structural
    changes require an ADR + a CI-enforced sunset window.
  - Violates the Strangler discipline established by ADR-0134 for
    analogous structural deprecations.
  - Future contributors who reference the six dead paths via git
    history have no programmatic warning.
- **Rejected**.

### (b) Phased Strangler with adapter shims

- **Pros**:
  - Matches ADR-0134's phase-1-through-6 pattern most exactly.
  - Would allow live consumers to migrate over a 3-month adapter soak +
    6-week canary.
- **Cons**:
  - **No live consumers exist** to migrate. The prior six µservices
    were scaffolded but never deployed; no callers exist outside the
    git-tree itself.
  - Adapter shims for "no-current-callers" is dead code by construction.
  - Per agent-skills deprecation-and-migration SKILL.md §"Core
    Principles" #1 *"code is a liability"*: authoring + maintaining
    adapter shims for zero callers is the Zombie-Code anti-pattern.
- **Rejected**.

### (c) Atomic consolidation + 6-month REPORT-ONLY → BLOCKER lane  ← **CHOSEN**

- **Pros**:
  - Zero adapter-shim ceremony (no live callers).
  - 6-month soak provides programmatic protection against future
    re-introduction of references to the dead paths.
  - Matches Strangler discipline adapted for zero-current-consumer
    state.
  - Auditable: the CI lane's pass count over 6 months IS the proof of
    zero re-introduction.
  - Compatible with ADR-0114's canary-observability-rollback (the
    REPORT-ONLY → BLOCKER promotion is itself a canary).
- **Cons**:
  - Requires authoring the lane (follow-up IP).
  - Requires `.github/branch-protection.yaml` update (follow-up
    ChangeSet).
- **Accepted**.

### (d) Atomic consolidation only — no lane, no soak

- **Pros**:
  - Lowest follow-up ceremony.
- **Cons**:
  - No programmatic protection against future re-introduction.
  - `feedback_no_silent_regression.md` requires the sunset gate.
- **Rejected**.

## Consequences

### Positive

1. **No silent regression.** The 6-month REPORT-ONLY soak guarantees
   that any re-introduction of a dead path is detected before it lands.
2. **Terminal-state cleanliness.** The six dirs are physically removed
   at Phase 0 and stay removed; no Zombie-Code retention.
3. **bc-sources archive preserves audit-grade content.** All 493
   artefacts preserved; per-BC chapters of the consolidated product
   remain authoritative for BC-internal detail.
4. **Crate-name stability.** No crate renamed; cargo + downstream
   tooling unaffected.
5. **Compatible with the parallel Connect-dissolution.** Foundry's
   Strangler (this ADR) and Connect's Strangler (ADR-0134) follow the
   same skill SKILL.md template; future structural deprecations have
   two prior cases to reference.

### Negative

1. **6-month soak window before BLOCKER promotion.** Anyone whose
   pre-Phase-0 working branch carries a reference to the dead paths
   gets only REPORT-ONLY visibility during the soak; mitigated by
   axis-foundry sweeping branches at Phase 1 entry.
2. **Lane authoring + branch-protection update follow-up.** The
   CI lane is not authored in this ChangeSet; it's queued under
   `microservices/foundry/IP-NNN-foundry-six-path-lane.md` (one of
   the 90 consolidated IPs).

### Migration cost quantification

| Cost class | Quantity | Mean per-unit cost | Total |
|---|---|---|---|
| File moves (git mv) | 493 | ~30 sec each script-driven | ~4 hours scripted |
| Consolidated top-level docs | 13 | ~2 engineer-hours | ~26 engineer-hours |
| ADRs | 3 | ~3 engineer-hours | ~9 engineer-hours |
| Spec consolidation | 1 | ~1 engineer-hour | ~1 engineer-hour |
| Per-BC archive coherence verification | 6 | ~0.5 engineer-hour | ~3 engineer-hours |
| Lane authoring (follow-up IP) | 1 | ~2 engineer-days | ~16 engineer-hours |
| Branch-protection update (follow-up) | 1 | ~0.5 engineer-day | ~4 engineer-hours |
| Soak monitoring (over 6 months) | 6 months | ~30 min/month sweep | ~3 engineer-hours total |
| **Total** | | | **~66 engineer-hours** in the consolidation ChangeSet + ~23 engineer-hours of follow-up over 6 months |

(Per skill SKILL.md §"Common Rationalizations": *"Compare migration cost
to ongoing maintenance cost over 2–3 years."* The prior 6-way split's
ongoing maintenance cost — six PRD drift, six threat-model drift, six
hyperscaler-gate runs per PR — easily ≫ 66 engineer-hours per quarter.)

### Operational

- **New CI lane** (registered in `.github/branch-protection.yaml` via
  follow-up ChangeSet under IP):
  - `oya-governance-foundry-six-path-zero-usage` — REPORT-ONLY from
    2026-05-18 to 2026-11-18; BLOCKER from 2026-11-18 onward.
- **Deprecation notices** are NOT authored at the six dead paths
  themselves (they no longer exist). Instead:
  - This ADR (`docs/decisions/ADR-0138-intelligence-six-path-deprecation.md`)
    is the single deprecation-notice-of-record.
  - The bc-sources archive carries a sub-folder per BC; the BC-internal
    references to former paths inside those preserved documents are
    historical and remain accurate to the document's authorship date.
- **No CLI deprecation hint required**: there were no CLI integrations
  with the six dead paths.

## Clean Architecture Impact

| Lane | Impact | Action |
|---|---|---|
| `dependency-direction` (LEAN-A1) | preserved | adapter / port rules per crate fan-out unchanged |
| `per-microservice-layout` (ADR-0131) | improved | one µservice's flat layout vs prior six µservices' flat layouts |
| `foundry-six-path-zero-usage` (NEW) | new REPORT-ONLY→BLOCKER | per Strangler Phase 1 / Phase 2 |
| `foundry-bc-source-coherence` (NEW per ADR-0136) | new BLOCKER | enforce bc-sources archive + top-level coherence |

## Verification

### Phase 0 (this ChangeSet)

- [ ] `find microservices/foundry-runtime microservices/foundry-supervisor microservices/foundry-eval microservices/foundry-evidence microservices/foundry-guardrails microservices/foundry-providers -type d 2>/dev/null` returns empty.
- [ ] `find microservices/foundry -type f | wc -l` returns 493.
- [ ] All 13 consolidated top-level documents exist at
      `microservices/foundry/<doc>.md`.
- [ ] All 6 per-BC archives exist at `microservices/foundry/bc-sources/<bc>/`.
- [ ] All 90 IPs exist at `microservices/foundry/IP-001..IP-090-<bc>-*.md`.
- [ ] `grep -rn 'microservices/foundry-\(runtime\|supervisor\|eval\|evidence\|guardrails\|providers\)' microservices/ docs/ specs/ registry/ | grep -v -E '(docs/decisions/ADR-0136|docs/decisions/ADR-0137|docs/decisions/ADR-0138|microservices/foundry/bc-sources/)'` returns zero hits.
- [ ] `specs/microservices/foundry.json` exists and is parseable JSON.
- [ ] `cargo run -p oya-check-authority-cohesion -- --repo-root .` exits 0.
- [ ] ADR-0136 and ADR-0137 acceptance carried in the same ChangeSet
      (or in immediate predecessors landed at the same head).

### Phase 1 (REPORT-ONLY soak; 2026-05-18 → 2026-11-18)

- [ ] CI lane `oya-governance-foundry-six-path-zero-usage` registered in
      `.github/branch-protection.yaml` as REPORT-ONLY (follow-up IP).
- [ ] Lane runs on every PR + every daily sweep cron.
- [ ] Lane has produced zero non-zero-match reports during the soak (or:
      every non-zero match was investigated + resolved by axis-foundry).

### Phase 2 (BLOCKER post-2026-11-18)

- [ ] Lane promoted to BLOCKER in `.github/branch-protection.yaml`.
- [ ] Lane refuses any PR with a reference to the dead paths.

### Phase 3 (terminal)

- [ ] No further verification required; absence is the verification.

## References

- ADR-0134: Connect-dissolution Strangler migration — pattern template
  for this ADR.
- ADR-0114: Canary observability + rollback — REPORT-ONLY → BLOCKER
  cadence template.
- ADR-0123: Hyperscaler maturity claim gate.
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0132: No-grouping forward policy.
- ADR-0133: Industry best-practice conformance.
- ADR-0136: Foundry as a single µservice — establishes the consolidation.
- ADR-0137: Foundry bounded contexts — names the six BCs.
- agent-skills deprecation-and-migration SKILL.md — Strangler Pattern,
  Verification checklist.
- agent-skills documentation-and-adrs SKILL.md — ADR template authority.
- `feedback_no_silent_regression.md` — public-contract preservation.
- `feedback_repeat_mistake_prevention.md` — programmatic protection
  rationale.
