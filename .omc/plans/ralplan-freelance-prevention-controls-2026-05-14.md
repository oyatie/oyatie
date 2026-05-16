---
doc_class: RalplanPreventionControls
shape: anchor
status: pending architect+critic
date: 2026-05-14
created_by: ralplan --prevention --short (post-realignment, session 7e0309c2)
canonical_authority: /specs/cross-cutting/decision-principles.json + /specs/cross-cutting/forbidden-operations.json
authority_chain: "docs/MASTERPLAN.md \u2192 .omc/plans/consensus-masterplan-2026-05-13.md\
  \ \u2192 .omc/plans/ralplan-ops-freelance-realignment-2026-05-14.md \u2192 this\
  \ plan"
mode: SHORT
purpose: Auto-backfilled purpose for ralplan-freelance-prevention-controls-2026-05-14.md
---
# RALPLAN — Freelance-misalignment prevention controls (2026-05-14)

## §0 Reframe

The 20-commit ops-Wave-1 freelance class (resolved via `ralplan-ops-freelance-realignment-2026-05-14.md`) was a *predictable* failure mode under the current control set: lean-a10 isn't operational yet, no ADR-supersession-graph sub-check exists, no Bominal-inheritance violation sub-check exists, and the grit Stop hook accepts `claim --intent` text without grounding-citation requirements. This plan adds the four smallest controls that would have **mechanically blocked** the freelance class.

Eliminated failure mode: *"freelance code authored without plan grounding + simultaneously-Accepted ADRs in contradiction class + transport-stack picks differing from Bominal inheritance — all four landing simultaneously with no preventive lane firing."*

## §1 Principles (4; SHORT mode)

1. **Mechanical > judgment.** Every control here is a registry + extractor + lane rule. No new "agents must remember to check X" rules.
2. **Extend existing crates, don't author new ones.** Workspace is blocked behind 297-missing-crate dirs; new crate authoring requires resolution. Existing `oya-check-adr-index/` + `oya-check-adr-citation/` + `oya-check-authority-cohesion/` + `oya-check-active-artifact-contract/` (all on disk at HEAD `4d6bf91`) are extension targets.
3. **Registry-first, lane-second.** JSON registries (declarative; pure data; no compile dependency) land **TODAY**. The lane sub-checks that consume them land at M02-P20 IP-005 expansion per docs sub-plan §6(f) dispatch sequence. This decouples "wire the contract" from "wire the enforcer".
4. **Stop-hook is the lowest-cost preventive measure.** It runs every turn end already; adding regex validation on `grit claim --intent` text is a single JSON edit. No new infrastructure.

## §2 Decision drivers (top 3)

1. **Prevent the same failure class from recurring** (ADR-0090↔R5 contradiction + Bominal-ADR-0209 inheritance-by-omission). The realignment plan **cures** the specific 20 commits; this plan **prevents** the next class.
2. **Smallest-actionable per user preference.** Four controls; three land today as pure registry/hook edits; one (lane sub-check authoring) defers to existing M02-P20 IP-005 dispatch capacity.
3. **No workspace-resolution dependency.** All four controls land BEFORE the 297-missing-crate blocker clears. The Stop hook + JSON registries are compile-independent; the lane sub-check authoring is queued in an existing impl-plan, not a new one.

## §3 Viable options (≥2)

**Option Ω — New top-level lane crate (`oya-check-freelance-prevention`).**
- Pros: single dedicated home for all four controls.
- Cons: requires new crate authoring → blocked behind 297-missing-crate workspace blocker. Also duplicates work that fits naturally into existing `oya-check-adr-index` / `oya-check-adr-citation` / `oya-check-architecture` (when it lands) families. **Rejected.**

**Option α — Extend existing crates + add declarative JSON registries + Stop-hook regex (RECOMMENDED).**
- Pros: no new crates; registries land today; Stop hook lands today; lane sub-checks ride existing M02-P20 IP-005 expansion. All four controls operational by M02-P20 exit.
- Cons: four control surfaces (one per existing crate) instead of one — but that's the right shape because each control validates a distinct contract class. **Chosen.**

## §4 Pre-mortem (3 scenarios)

### Scenario 1: Stop-hook regex too strict; legitimate work blocked
- **Trigger:** grit `claim --intent` text doesn't happen to mention a phase ID + Accepted ralplan in the regex shape (e.g., agent cites `M02-substrate/P20` instead of `M02-P20`).
- **Blast radius:** legitimate authoring blocked; agent retries with reworded intent; productivity tax.
- **Prevention:** §6(d) gate accepts both shapes: `M\d\d-P\d\d` AND `M\d\d-[a-z]+(-[a-z]+)*/P\d\d-[a-z]+(-[a-z]+)*`. Plus accept Wave designator `Wave \d` cross-references. Plus accept impl-plan reference `IP-\w+`. The regex is permissive on grounding *form* but strict on grounding *presence*.
- **Detection:** Stop hook emits structured log line `grit-claim-intent-reject: <claim-id> <reason>`; lane fixture covers known-good + known-reject examples.
- **Rollback:** disable hook with `.claude/settings.local.json` revert; restore prior empty `{}` state.

### Scenario 2: ADR-contradiction registry becomes a write-only list (agents add contradiction pairs but never resolve them)
- **Trigger:** known contradiction lands without supersession plan; registry grows; nothing ratchets.
- **Blast radius:** registry becomes documentation theater instead of an enforcement contract.
- **Prevention:** every row carries `resolution_trigger` field naming the milestone/phase by which the contradiction MUST resolve (analog to dep-seam-phaseout R5's trigger DSL). Past-due rows escalate via lean-a10 finding `adr-contradiction-overdue: <pair>`.
- **Detection:** quarterly review of registry.
- **Rollback:** retire the contradiction registry; depend on case-by-case ADR-supersession PR review instead.

### Scenario 3: Bominal-inheritance overrides registry diverges from MASTERPLAN §3 table
- **Trigger:** Master plan §3 override table edited; JSON registry not synced.
- **Blast radius:** lane fails or passes on stale rules; silent enforcement drift (the exact failure class this plan prevents).
- **Prevention:** lane sub-check has parity sub-check — registry MUST round-trip-equal MASTERPLAN §3 table parsed as TOML/Markdown table. Drift fails the lane immediately.
- **Detection:** parity sub-check in fixture; CI catches drift on next PR.
- **Rollback:** restore registry from MASTERPLAN §3 (single source of truth wins).

## §5 Test plan (SHORT)

| Tier | Coverage |
|---|---|
| **Static (registries)** | `/registry/adr-contradictions.json` + `/registry/bominal-inheritance-overrides.json` validate against their own JSON schemas at parse time. |
| **Stop-hook fixture** | 3 known-good intents (each with both regex citations) pass; 3 known-reject (missing phase / missing ralplan / missing both) reject with structured reason. |
| **Future lane fixture (M02-P20 IP-005 expansion)** | Each sub-check (ADR-contradiction-graph; Bominal-inheritance-violation) ships golden fixtures: ADR-0090↔R5 known pair fails the contradiction-graph lane until supersession edge lands; freelance hyper-transport choice without override registry row fails the Bominal lane. |

## §6 Specific decisions (a-d, one per control)

### (a) Control 1 — Accelerate lean-a10-regression authoring to TODAY (as report-only)

**Current state:** lean-a10 is registered in `registry/quality/lanes.yaml` as "planned (registered, scaffold-only)" per ops-portal v7 §6(b). Authoring target phase: M02-P21 IP-X.

**Acceleration:** extend the existing M02-P21 impl-plan path to ADD a TODAY-landable scaffold task: author `crates/oya-check-regression/` skeleton WITH the ADR-0090↔R5 golden fixture, lane registered `--report-only` from authoring date. Promotion to BLOCKER stays at M02-P21 per existing plan.

**Gating:** this control's TODAY-land is blocked behind 297-missing-crate workspace resolution. The plan-level edit (adding the fixture spec to the M02-P21 impl-plan) lands today; the crate authoring rides the resolution.

**Today's deliverable:** add §"Acceleration: ADR-contradiction golden fixture" section to `.omc/plans/milestones/M02-substrate/phases/P21-architecture-planes-green/impl-plans/IP-X-regression-lane.md` (NEW IP file if not present; extend if present).

### (b) Control 2 — `/registry/adr-contradictions.json` (declarative; consumed by `oya-check-adr-index` extension)

**Today's deliverable:** seed registry with ADR-0090 ↔ R5-pending-as-ADR-0091 pair. Schema mirrors `active-artifact-contract` pattern: `_schema_ref`, `_artifact_id`, `_meta`, `contradictions[]`. Each row carries:

- `pair_id` (kebab-case)
- `adr_a` + `adr_a_status` (Accepted/Proposed/Superseded)
- `adr_b` + `adr_b_status`
- `contradiction_class` (e.g., "transport-stack-direction")
- `resolution_trigger` (milestone/phase by which one MUST supersede the other)
- `superseding_pr` (filled when resolution lands)

**Future lane:** `oya-check-adr-index --sub-check=contradiction-graph` (added to existing `crates/oya-check-adr-index/src/lib.rs` at M02-P20 IP-005 expansion). The sub-check parses `docs/decisions/ADR-*.md` frontmatter, builds the {Accepted, Superseded-by, Supersedes} graph, and fails if any registry row's pair is in mutual-Accepted state without a supersession edge.

### (c) Control 3 — `/registry/bominal-inheritance-overrides.json` (declarative; consumed by `oya-check-architecture --bominal-inheritance` extension)

**Today's deliverable:** seed registry from MASTERPLAN §3 9-row override table verbatim. Schema: each row carries `override_id`, `oyatie_decision`, `bominal_canonical_ref` (Bominal ADR ID), `scope` (e.g., transport-stack / glossary / workflow-placement), `adr_cite_oyatie` (the oyatie ADR that records the override).

**Future lane:** `oya-check-architecture --bominal-inheritance` (folded into whichever check crate hosts `--canonical-base-neutrality` and `--cross-pack-refusal` per ADR-0064 §7 §8; that crate is the masterplan §Follow-up item #6 deliverable). Sub-check loads MASTERPLAN §3 + this registry; verifies parity; for every oyatie ADR/ralplan picking a transport/framework/glossary differing from Bominal, registry row MUST exist OR ADR/ralplan MUST cite supersession.

### (d) Control 4 — Stop-hook grit-claim --intent regex gate (`.claude/settings.local.json`)

**Today's deliverable:** extend the empty `.claude/settings.local.json` to register a Stop hook (or PreToolUse hook on Bash matching `grit claim`) that validates the `--intent` text against the regex pair:

- **Required phase citation:** `(M\d\d-[a-z-]+/P\d\d|M\d\d-P\d\d|Wave \d|IP-\w+)` — accepts any of the canonical phase-ID forms used across the codebase.
- **Required ralplan grounding:** `(ralplan-[a-z0-9-]+-2\d{3}-\d{2}-\d{2}|consensus-masterplan-2\d{3}-\d{2}-\d{2}|ADR-\d{4})` — the claim must cite at least one Accepted plan or ADR.

**Both regexes must match** the `--intent` text. Otherwise the hook rejects the grit invocation BEFORE the claim lands; agent sees a structured error pointing at this plan §6(d).

**Scope limitation:** the hook fires only when the grit command modifies `crates/oya-*/` paths. Read-side grit operations (`grit status`, `grit show-session`, `grit symbols`) are exempt — they don't author code.

**Fallback if oyatie has no grit infra installed locally yet:** the hook is wired but reports `grit-claim-intent-gate: dormant (grit not invoked yet in oyatie)` until the first grit claim hits. No action needed before that.

## §7 Risk register

| ID | Risk | Mitigation |
|---|---|---|
| R1 | Stop-hook regex too strict; legitimate work blocked | §4 Scenario 1 — permissive regex on form, strict on presence |
| R2 | Registry rows accumulate without resolution (write-only list) | §4 Scenario 2 — `resolution_trigger` field + lean-a10 overdue finding |
| R3 | Registry diverges from MASTERPLAN §3 single source | §4 Scenario 3 — parity sub-check in lane fixture |
| R4 | Lane authoring delayed past M02-P20 IP-005 expansion | M02-P22 exit gate naturally pulls forward; same enforcement pattern as lean-a5/a6/a7 |
| R5 | Stop hook silently fails to fire (config typo) | Hook fixture in test plan §5; turn-end auto-injection (existing pattern per session-start context) verifies live |

## §8 ADR record

- **Decision**: Author 4 prevention controls (this plan §6 a-d). Three land TODAY (registries + Stop hook); one (lean-a10 acceleration) lands at M02-P21 per existing dispatch but with TODAY-authored fixture spec. No new top-level lane crate.
- **Drivers**: mechanical-not-judgment; smallest-actionable; no-workspace-dependency; reuse existing crates.
- **Alternatives considered**: Option Ω (new top-level lane crate) — rejected; blocked by workspace + duplicates existing crate purposes.
- **Why chosen**: Option α extends existing crates (`oya-check-adr-index`, `oya-check-architecture` when it lands) + adds declarative JSON registries + adds Stop-hook regex; no compile dependencies for today's lands.
- **Consequences**:
  - Positive: the four controls compose with existing lane infrastructure; ADR-0090↔R5 class is mechanically caught next time; agent freelance is gated on grit-claim grounding citation.
  - Negative: M02-P20 IP-005 expansion absorbs additional sub-check authoring scope (already wide; this is incremental).
  - Neutral: Bominal-inheritance lane composes with ADR-0064 canonical-base-neutrality lane (same parent crate).
- **Follow-ups**:
  1. TODAY: §6(b) + §6(c) registries + §6(d) hook.
  2. TODAY: add fixture spec line to M02-P21 impl-plan per §6(a).
  3. **M02-P20 IP-005 expansion dispatch:** extend `oya-check-adr-index` with `--contradiction-graph` sub-check; extend `oya-check-architecture` (or successor) with `--bominal-inheritance` sub-check.
  4. **M02-P21 IP-X regression-lane dispatch:** author `oya-check-regression` with ADR-0090↔R5 golden fixture.
  5. **Quarterly review (per §4 Scenario 2):** sweep ADR-contradictions registry; resolve or escalate overdue pairs.

## §9 Verification status

| Round | Architect | Critic | Iteration delta |
|---|---|---|---|
| 1 (draft) | _pending_ | _pending_ | initial draft (2026-05-14) |

Acceptance criteria (SHORT-mode 4 dimensions):

1. **Mechanical enforceability** — every control is a registry + extractor + lane rule, never "agents must remember".
2. **No workspace dependency for today's lands** — §6(b)+(c)+(d) all land before the 297-missing-crate blocker clears.
3. **Composes with existing infrastructure** — §6(b) folds into `oya-check-adr-index`; §6(c) folds into the masterplan §Follow-up canonical-base-neutrality lane; §6(d) is a single JSON edit.
4. **Each control has a regression fixture** — §5 enumerates known-good + known-reject pairs per control.
