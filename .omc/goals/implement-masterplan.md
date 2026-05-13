---
goal_id: implement-masterplan
status: active
created: 2026-05-13
trigger: "Implement the masterplan" | "/goal implement-masterplan" | autonomous re-entry
horizon: M02-substrate + M03-first-tenant
authority: durable; supersedes ad-hoc execution prompts
---

# Goal: Implement the masterplan

## 1. Intent

Drive oyatie from current state (M01 cutover landed; 114 crates on BNF v4.1; 4 LEAN lanes BLOCKER) to **M3 closure**: 1 paying KR group tenant live on oyatie with HR / Payroll / Accounting / Connect Pro Mail+Messenger / Application B2B shell / Workflow Studio hero product, all 14 CI lanes BLOCKER, audit chain Ed25519-sealed, ADR-0210 35-checkbox evidence bundle complete.

## 2. Entry point

**`.omc/plans/M01-M03-parallelization-manifest.md`** is the dispatch DAG.

- 14-node critical path: `M01-P05 ✓ → M02-P02 → P12 → P19 → P21 → P22 → M03-P01..P08`
- 15 parallel waves with explicit grit symbol-lock pre-flight (17 CLEAR verdicts)
- Per-wave `Agent(...)` one-liner dispatch script with `subagent_type=oh-my-claudecode:executor`

Read the manifest. Pick the next wave whose `depends_on` is satisfied. Fire its executors in parallel.

## 3. Wave protocol (every executor)

Each phase under `.omc/plans/milestones/<milestone>/phases/<phase>/`:

1. Read `phase-spec.md` (scope, entry/exit gates, naming justification, BCs)
2. Read `impl-plan.md` (concrete file targets, code shape, DDL, port traits, Cedar, Protobuf, load tests, halt conditions)
3. `grit session start <phase-slug>-2026-05-13` → `grit claim --agent <slug> --intent "..." --ttl 3600 <symbols from ## Grit Claim Symbols>`
4. If grit returns no symbol scope (doc-only or symbols not yet registered): fall back per ADR-0054 → `icm store -t scaffold-locks-oyatie -c "<rationale>" -i critical -k "<phase>"` BEFORE any direct git
5. Implement per `## Code Shape` and `## Concrete File Targets`
6. Run universal acceptance gates (§5 below)
7. Run phase-specific gates per impl-plan `## Acceptance Gates`
8. `icm store -t context-oyatie -c "Phase <name> complete" -i high -k "<milestone>,<phase>,phase-complete,<microservice>"`
9. `grit done --agent <slug>` — the rebase/merge/release primitive. **Never manually sequence rebase/merge.**

## 4. Mandatory rules (do not violate)

Read these memory files at session start; they govern every decision:

| Rule | File |
|---|---|
| BNF v4.1 naming + 12-layer enum | `~/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_naming_justification.md` + ADR-0056 |
| Clean architecture (port location, inward-only, cross-product refusal) | `feedback_clean_architecture_requirements.md` |
| Workflow + Ontology = sole adapter layer between products | `feedback_workflow_objectgraph_adapter_layer.md` |
| Autonomous-decision charter (no stubs/placeholders/deferrals; Linus-grade criticism; hyperscaler thinking) | `feedback_autonomous_decision_principles.md` |
| Quality bar (Stripe/Palantir/Linear + p99 ≤50ms read / ≤200ms write / 10k+ RPS per cell / 100M+ users / horizontal scalability mandatory) | `feedback_quality_performance_scalability_bar.md` |
| Bominal inheritance precedence (1:1 with glossary translation; session decisions override) | `feedback_bominal_inheritance_precedence.md` |
| Workflow Studio scope (n8n-class hero product; multi-domain; first hero) | `feedback_workflow_studio_scope.md` |
| Stale removed in reality (not just marked retired) + no compat seams + no dead code | `feedback_autonomous_implementation_artifacts.md` |
| Grit claim/work/done protocol | `feedback_grit_claim_work_done.md` |
| Milestone > Phase > Impl-plan hierarchy | `feedback_milestone_phase_hierarchy.md` |
| Glossary: `shared` not `platform`; `Ontology` not `Object Graph`; `Application` not `Shell` | `feedback_glossary_shared_not_platform.md`, `feedback_glossary_ontology_not_object_graph.md` |
| Flat µservice catalog | `feedback_flat_product_catalog.md` |
| Workflow is shared substrate | `feedback_workflow_is_shared.md` |

### 4.1 Load-bearing ADRs (read at session start)

| Rule | ADR |
|---|---|
| BNF v4.1 grammar + 12-layer enum | `docs/decisions/ADR-0056-rust-clean-architecture-bnf.md` |
| Flat µservice catalog | `docs/decisions/ADR-0058-flat-microservice-catalog.md` |
| Workflow + Ontology = ecosystem adapter layer | `docs/decisions/ADR-0059-workflow-ontology-ecosystem-adapter-layer.md` |
| Bominal inheritance precedence + overrides | `docs/decisions/ADR-0060-bominal-inheritance-precedence.md` |
| Application B2B unified shell | `docs/decisions/ADR-0061-application-b2b-unified-shell.md` |
| Quality / Performance / Scalability bar (mandatory PRD sections) | `docs/decisions/ADR-0062-quality-performance-scalability-bar.md` |
| Documentation suite coverage (CI-enforced; LEAN-A5) | `docs/decisions/ADR-0063-documentation-suite-coverage.md` |
| Canonical global base + localization seams / adapters / packs (KR is pack #1) | `docs/decisions/ADR-0064-canonical-base-and-localization-packs.md` |
| Localization pack catalog (anchor) | `docs/localization-packs/INDEX.md` |
| KR pack (pack #1; foundational; M01–M07) | `docs/localization-packs/kr.md` |

### 4.2 Localization architecture (the expectation, set globally)

Every customer-facing µservice has a **canonical global base** (jurisdiction-agnostic) + **localization overlay** chosen per-concern from three forms (per ADR-0064 §1):

- **Seam** — port in canonical base; jurisdiction plugs in a value or thin trait impl via DI. Smallest blast radius. Preferred for statutory rates, tax tables, leave counts, holiday calendars.
- **Adapter** — separate adapter crate translating jurisdiction-specific I/O (EDI / API / portal) into canonical domain types. Preferred for discrete I/O surfaces.
- **Pack** — coherent bundle (seams + adapters + Cedar fragments + Workflow templates + Typst templates + acceptance evidence) per jurisdiction. The unit of release and the unit of doc-suite enforcement.

Choose per-concern, whichever is most appropriate. The forms compose: a pack composes seams + adapters.

Canonical base MUST NOT bake in statutory rates, jurisdiction codes, language strings, or regulatory-authority names. Enforced by `oya-check-architecture --canonical-base-neutrality` (M02-P20 scope).

## 5. Universal phase exit gate

A phase is complete only when **all** of these are true:

```bash
cargo check --workspace --all-features          # exit 0
cargo build --workspace --all-features          # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0
cargo nextest run --workspace --all-features    # exit 0; 0 failures
cargo deny check                                # exit 0
cargo doc --workspace --no-deps                 # exit 0
```

Plus:

- All LEAN / quality CI lanes referenced in phase-spec `acceptance_lanes:` frontmatter green
- Load tests meet Performance Targets per impl-plan `## Load test`
- Audit log emits per Bominal ADR-0028 (Merkle-sealed Ed25519 per `(tenant_id, period)`)
- Phase docs reflect shipped state (no drift between spec and code)
- **Documentation suite coverage** green per ADR-0063 for every µservice the phase touches: canonical PRD + Microservice record + Naming-scope ADR + BC registrations + Phase-Spec + Impl-Plan; plus per-pack overlay (PRD + regulatory ADR + acceptance evidence) for every (pack × µservice) pair in pack scope. Enforced by `lean-a5-doc-coverage` (report-only until M02-P22; BLOCKER thereafter).
- **Canonical-base neutrality**: if the phase touches canonical-base crates, `oya-check-architecture --canonical-base-neutrality` green (no jurisdiction codes / statutory rates / authority names baked into canonical base) per ADR-0064.
- `icm store -t context-oyatie -c "Phase X complete" -i high` row emitted
- `grit done --agent <id>` succeeds

Anything less = **in progress** (never `Complete`).

## 6. Sanctioned primitives

Only `grit`, `icm`, `oya-tooling-agent-read`.

Direct `git`/`gh` requires `icm store -t direct-tool-invocations -c "<rationale>" -i critical` logged BEFORE the call (Directive 12).

## 7. Escalation matrix — halt and surface to user when:

1. **Architectural ambiguity**: phase-spec contradicts an ADR, or two ADRs conflict, or a new ADR is required to proceed (do NOT write a new ADR autonomously without consensus)
2. **Cross-milestone scope creep**: phase implementation needs to mutate files outside its declared `## Scope > In-scope` block and the change is not pre-declared in a sibling phase's `unblocks:` field
3. **Bominal cross-repo gap**: phase requires a Bominal library port that doesn't exist at `/Users/jasonlee/bominal/` — escalate; do NOT invent the missing library

Do NOT halt for: routine test failures (debug), naming nits (apply BNF v4.1 + justify), missing deps within cargo-deny allowlist (add), performance targets under declared budget envelopes (tune).

## 8. Parallelization rules

- **Within a wave**: fire all phases simultaneously (grit symbols disjoint → no collisions)
- **Between waves**: strict sequential; next wave blocks until prior wave's `grit done` events all emit
- **Critical path** is the worst-case lower bound; off-critical-path phases can slip without delaying milestone

## 9. Quality discipline (Linus-mode self-review on every PR)

Before declaring a phase complete, answer:

1. What's the data shape? Show the structs
2. Is there a special case? Why? Can it flatten to the general case?
3. Any abstraction used exactly once? Delete it
4. Any bureaucracy / boilerplate that doesn't carry weight? Delete it
5. Would this fail under 100M users? Show back-of-envelope
6. Bottleneck? (lock contention / cache invalidation / network roundtrips / DB write amp)
7. Test pyramid correct? (unit > integration > e2e; isolation tests on every state-changing endpoint per Bominal ADR-0011)
8. Any `unwrap()` / `expect()` outside startup? Reject
9. Does the dep graph respect the 12-layer matrix? Cite the lane that catches violations

## 10. Durable state machine

Progress is reconstructible without re-reading the conversation:

- **`git log --oneline`** — committed phases (one commit per phase via `grit done`)
- **`icm recall -t context-oyatie -q "phase-complete"`** — phase-complete rows
- **`status:` frontmatter** in `.omc/plans/milestones/<m>/phases/<p>/phase-spec.md` (`Proposed` → `InProgress` → `Complete`)

Re-reading this goal file always produces the same next action: query the manifest, find the next unblocked phase, dispatch.

## 11. Termination criteria

Goal complete when:

- **M02-P22 exit gate** signs off: all 14 CI lanes flipped from `--report-only` to BLOCKER; Application B2B shell deployed to OCI ARM64 Stage 0; sibling-team smoke test green
- **M03-P08 evidence bundle** complete: ADR-0210 35-checkbox closure; 4대보험 EDI green; 연말정산 sealed; legal hold verified; 7-day SLO held
- Final commit on `main` carries tag `m3-tenant-live`

## 12. Re-entry protocol (autonomous loop)

If invoked again with this same goal:

1. Read this file
2. `icm recall -t context-oyatie -q "phase-complete" --limit 50` — reconstruct phase state
3. `git log --oneline -20` — confirm commits match ICM
4. Query manifest for next unblocked phase
5. Dispatch wave executor(s)

No conversation context required. The repo + ICM + this goal file are the durable substrate.

---

**Pointers**:

- Manifest: `.omc/plans/M01-M03-parallelization-manifest.md`
- Phases: `.omc/plans/milestones/{M01-foundation,M02-substrate,M03-first-tenant}/phases/`
- Masterplan narrative: `docs/MASTERPLAN.md`
- Templates: `docs/templates/`
- PRDs: `docs/prds/`
- ADRs: `docs/decisions/`
- Bominal cross-reference: `/Users/jasonlee/bominal/`
- Canonical memories: `~/.claude/projects/-Users-jasonlee-oyatie/memory/`
