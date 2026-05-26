---
handoff_id: 2026-05-17-claude-to-pr143-agent
created_by: claude-opus-4-7
created_at: 2026-05-17T23:30:00Z
audience: PR #143 agent (oya-microservice-flat-layout-buildout-2026-05-17 branch)
companion_handoff: .omc/state/sessions/HANDOFF-2026-05-17-microservice-flat-layout-buildout.md
session_outcome: 14 PRs landed (#126-140); next: integrate with PR #143 µservice substrate
---

# Handoff: claude session → PR #143 agent

## TL;DR — what to add to PR #143 (18 items, prioritized)

**Critical (block production):** 1-3
**High (block exit gates):** 4-7
**Connect strangler:** 8-11
**Medium (post-substrate):** 12-13
**Cross-cutting backlog:** 14-18

## What landed on `origin/dev` from this session (14 PRs: #126-#140)

| PR | Surface |
|---|---|
| #126 | foundry/PRD.md industry competitive audit (§11a-d) |
| #127 | workflow-studio competitive audit + 16 patterns + hyperscaler bar |
| #128 | cloud PRD §14 hyperscaler audit + portfolio meta-audit |
| #129 | 4 enterprise PRDs v1.0.0 → v1.1.0 (Workday/ADP/NetSuite/QuickBooks parity) |
| #130 | Connect super-app expansion — 4 sub-PRDs (social/shorts/network/anonymous) |
| #131 | 4 original Connect PRDs (suite/mail/messenger/calendar) industry audit |
| #132 | IP acceptance_criteria sample fix (5 worst IPs) + F-IP-AC-BACKFILL-CORPUS |
| #133 | masterplan + sequencing + MASTERPLAN.md consolidation |
| #134 | standards docs (8 files) enforced_by refs + advisory→F-PENDING-* |
| #135 | ADR-0125/0126/0127 (rewritten by you to advisory; 0125+0127 retired) |
| #136 | registry consolidation — milestone-audit + score-cards + KG nodes |
| #137 | hyperscaler-architecture-invariants spec (35 INV-* — advisory catalog) + ADR-0128 |
| #138 | ChangeSet schema in plan-schema.json + oya-check-honest-claims crate (11 tests) |
| #139 | KG deprecation — registry/knowledge-graph-semantic.json deleted + ADR-0130 (now superseded by your ADR-0130) + migrated to specs/products/ontology.json#type_system |
| #140 | oya-check-aspirational-enforcement crate (4 scanners, 18 tests, catches 218 violations on dev) |

## Critical contradictions in my session's outputs that need fixing

### 1. "Products" and "Suites" framing is dissolved (per ADR-0132)
My session is riddled with `specs/products/*` paths, `sub_products[]` arrays, "Connect product", "Enterprise suite" framing — all contradict ADR-0132's no-product no-suite forward direction.

### 2. "Fitness ARE the Governance" — wording reconciliation
Confirmed in `microservices/governance/PRD.md`: "The historical `oya-governance-*` working name retires here; the canonical name is `governance`". My crates + ADR-0128 + standards docs + score-cards all reference retired `oya-governance-*` naming. Must remap to `microservices/governance/` IP-NNN structure.

## 18 items to add (in dispatch order)

### Critical (block production safety)

**1. F-PORTFOLIO-LLM-CAPABILITY-CIRCUIT-BREAKER** → IPs in `microservices/foundry-supervisor/` AND `microservices/foundry-runtime/`. Add `CapabilityRun { max_retry_budget: u32, circuit_breaker_threshold: f32, circuit_state: enum {closed, half_open, open} }`. Workflow-engine + ontology API mirror. Production safety on runaway LLM loops. Source: PR #128 meta-audit.

**2. F-ADR-0008 + ADR-0015 + ADR-0053 critical contradictions** → resolve before any µservice IP cites them. ADR-0008 + 0015 have live `## Open questions` in accepted status. ADR-0053 literal `<placeholder>` in audit-chain emission ID.

**3. F-FITNESS-ASPIRATIONAL-ENFORCEMENT against #143's branch** → run `cargo run -p oya-check-aspirational-enforcement -- --repo-root .` on your branch; fix any new violations introduced by the 1,515 artifacts before merge. Live tool catches 218 violations on dev today.

### High (block exit gates)

**4. F-PORTFOLIO-PER-TENANT-RATE-LIMIT** → new substrate µservice `microservices/tenancy-rate-limit/` OR per-API-µservice IP. Token-bucket per-tenant + shuffle-sharding on `(tenant_id, capability_id)`. Required at foundry-runtime, workflow-engine, ontology API surfaces. Return 429+Retry-After.

**5. F-FOUNDRY-PROVIDER-DEGRADED-SHED** → `microservices/foundry-providers/` IP: `ProviderRunQueue.shed_policy: enum {none, hard_503, queue_drop}` + all-providers-degraded fast-fail.

**6. F-WORKFLOW-STUDIO-GOLDEN-SIGNALS** → `microservices/workflow-engine/` observability contract: traffic (`active_sessions_per_second`), errors (`failed_saves_per_min`), saturation (`crdt_merge_queue_depth`). Currently only latency + availability.

**7. F-HONEST-CLAIMS against #143's branch** → `cargo test -p oya-check-honest-claims` against your branch + `oya gate validate honest-claims`. Fix any "v1.1 / v2 / later / follow-on" deferral phrases in the 1,515 artifacts.

### Connect strangler completion (per ADR-0134)

**8. F-CONNECT-MICROSERVICE-PROMOTION** → create 4 new µservices (NO Connect prefix, NO Connect parent):
- `microservices/social/` (Instagram + Snapchat — visual social, AR camera, stories, close-friends rings, dual-context Ontology follow-graphs)
- `microservices/shorts/` (TikTok + Reels — FYP per `context_kind`, duets, stitches, LIVE, creator monetization isolation)
- `microservices/network/` (LinkedIn — Ontology identity graph, recruiter Foundry copilots, open-to-work employer-protection in personal pillar)
- `microservices/anonymous/` (Blind — BLAKE3+HSM anonymity, no FK from AnonPost to identity, four-eyes legal hold for court-order reveal only)
- Each ~70 artifacts per #143's template; each cites Bominal-ADR-0208 dual-context directly (no Connect intermediary).

**9. Supersede ADR-0126** "Connect super-app expansion" → new ADR (next free number, e.g., ADR-0136) describes the 4 µservices independently with no parent framing. Mark ADR-0126 status → superseded_by: [the new ADR].

**10. Delete `specs/products/connect/*.json`** after content migrates to µservices. Per OP-11 no compat seams.

**11. Delete `specs/products/enterprise/*.json`** + distribute content to relevant existing µservices (workflow-engine for HR workflows, foundry-runtime for capability surfaces, etc.). Per ADR-0132 enterprise-suite dissolution.

### Medium (post-substrate)

**12. F-PRODUCT-TO-MICROSERVICE-MIGRATION** (cross-cutting) — strangler completion sweep:
- Move ALL `specs/products/*` content into `microservices/<name>/PRD.md`
- Rename `per_product_required_compliance` → `per_microservice_required_compliance` in `specs/hyperscaler-architecture-invariants.json`
- Re-key `specs/score-cards.json` entries from product to µservice
- Update registry/milestone-audit/index.json product refs to µservice refs
- Delete `specs/products/` directory entirely
- Audit ALL my session's PRs #126-140 outputs for "product" / "suite" framing and remap

**13. F-FITNESS-IS-GOVERNANCE-MIGRATION** — wording reconciliation:
- Migrate `crates/oya-check-aspirational-enforcement/` into `microservices/governance/` per IP-002/003 tier-a batch migration pattern
- Migrate `crates/oya-check-honest-claims/` into `microservices/governance/`
- Update ADR-0128 `enforced_by` fields from `oya-governance-*` to governance µservice IP-NNN refs
- Update score-cards lane names
- Update standards docs (PR #134) F-PENDING-* citations from `oya-governance-*` to governance µservice
- Cross-reference governance µservice FR-11 (hyperscaler-maturity-claim-gate) + FR-12 (retired-vocabulary lane)

### High (gates downstream work)

**14. Promote ADR-0128 hyperscaler invariants from advisory → binding** via new governance µservice lane that scans every `microservices/*/PRD.md` for INV-* citations per `specs/hyperscaler-architecture-invariants.json#per_microservice_required_compliance` (renamed per #12). Wire as required CI context.

**15. Propagate ChangeSet schema (PR #138) to all 1,515 PR #143 IPs:**
- Each IP frontmatter gets: `changeset_id`, `depends_on_changesets[]`, `parallel_safe_with_changesets[]`, `serialize_with_changesets[]`, `enables[]`, `acceptance_status: ga` (no `mvp`/`v2-pending`/`1.1-deferred` — forbidden per `specs/plan-schema.json`)
- Enables topological-sort slice-selection per ADR-0129

### Multi-session backlog (after #143 lands)

**16. F-PORTFOLIO-ERROR-BUDGET-BURN-RATE** → portfolio-wide. Every µservice observability contract adds `error_budget_policy: { fast_burn: 5x/1h → page, slow_burn: 1x/6h → ticket }`. Wire via ADR-0114 canary-observability.

**17. F-IP-AC-BACKFILL-CORPUS** → 202 legacy IPs in `.omc/plans/milestones/` need `acceptance_criteria` blocks. PR #143's new IPs MUST include them at authoring time. Multi-session fleet — 1 PR per phase folder. PR #132 has the proven pattern.

**18. 96 missing fitness crates** per `registry/stub-audit/2026-05-17/missing-fitness-crates.json` — top 10 critical/high should be authored as IPs in `microservices/governance/` (since governance is the new home for all check crates).

## Verification stance for #143

Before #143 merges, verify:
- [ ] `cargo run -p oya-check-aspirational-enforcement` on your branch — fix any new violations
- [ ] `cargo test -p oya-check-honest-claims` — pass clean
- [ ] Every IP has `acceptance_status: ga` (no `mvp`/`v2-pending`/`1.1-deferred`)
- [ ] Every µservice PRD cites its required INV-* set per `per_microservice_required_compliance`
- [ ] ChangeSet edge fields propagated to all 1,515 IPs
- [ ] Connect content (specs/products/connect/) migrated to 4 new µservices OR deleted with successor ADR
- [ ] No "product" / "suite" framing in new µservice PRDs

## Locked decisions from interview (2026-05-17T23:20Z)

| Decision | User pick |
|---|---|
| Fitness > Governance precedence | Fitness IS the Governance µservice — same enforcement, new name (confirmed via `microservices/governance/PRD.md`) |
| ADR-0126 fate | Retire + supersede via new ADR documenting 4 µservices without Connect parent |
| 4 µservice names | Drop `connect-` prefix: `microservices/{social,shorts,network,anonymous}/` |
| `specs/products/` fate | Delete entirely after content migrates |
| F-PORTFOLIO-* fixuptasks | Author as IPs IN PR #143 branch (block #143 merge until critical-safety items shipped) |
| Stub audit (847 findings) integration | Input to #143 cleanup pass via aspirational-enforcement lane |
| Connect 4 sub-PRDs migration | Promote to 4 new µservices (~70 artifacts each per #143 template) |
| ADR-0128 binding | Now via new governance µservice lane scanning per-µservice INV-* citations |

## Provenance for the agent

| Artifact | Path |
|---|---|
| Stub audit baseline | `registry/stub-audit/2026-05-17/CONSOLIDATED.md` + 5 JSONL files |
| Missing fitness crates | `registry/stub-audit/2026-05-17/missing-fitness-crates.json` (99 entries) |
| Industry-pattern evidence | `evidence/autoresearch/*-industry-audit-*.json` |
| Portfolio meta-audit | `evidence/autoresearch/hyperscaler-pattern-meta-audit-1779012603.json` |
| Session fixuptasks | `registry/fixuptasks.jsonl` tail (12 from this session, tagged with `claude-durable-goal-2026-05-17*`) |
| Completion report | `evidence/goals/durable-goal-completion-report-1779013500.json` |
| Cross-correlation backlog | `registry/stub-audit/2026-05-17/missing-fitness-crates.json` |

## Open questions to surface to user (NOT for autonomous resolution)

1. **F-ADR-0008/0015 open-questions content** — these ADRs have live unresolved questions in accepted status. Resolving them requires user input on the actual unresolved decisions (BEHAVIORAL_TENANT_PRODUCT cross-tenant aggregate flow; sub-context naming inside an axis). Do not autonomously resolve.

2. **Governance µservice retiring `oya-governance-*` naming** — does this also retire the required CI context names on dev branch protection (`oya-governance-supply-chain`, `oya-governance-cohesion`, `oya-governance-api-semver`, `oya-governance-protection-context-match`)? If yes, branch protection needs admin-scope re-config to point at governance µservice's IP-NNN check names.

3. **F-MERGE-QUEUE-WEBHOOK-POLLER-WIRING** (ADR-0124 phase 2) — depends on Wave-B webhook receiver deployment. Still external-blocker?

## My session's 4 critical artifacts the #143 agent should preserve

These are GREEN-FIELD shipped this session — don't accidentally regress:

1. **`oya-check-aspirational-enforcement` crate** (PR #140) — 4 scanners, 18 tests, catches 218 live violations. The chained-enforcement meta-lane.
2. **`oya-check-honest-claims` crate** (PR #138) — 7 deferral-phrase categories, 11 tests. Day-1 honest-claims enforcement.
3. **`specs/hyperscaler-architecture-invariants.json`** (PR #137) — 35 INV-* canonical with named industry sources.
4. **`specs/plan-schema.json` `acceptance_status` enum + 5 ChangeSet edge fields** (PR #138) — forbids `mvp`/`v2-pending`/`1.1-deferred` values.

If these need to move (per #13 governance migration), preserve content + tests; only relocate paths.
