# Full-project Ralplan Planner v2 (2026-05-13)

**Revision of v1 absorbing architect r1's 10 amendments + 6 scale-failure modes + spec/status pattern.** Architect r1 → ITERATE with concrete next-step + freeze endorsement.

## §1 Current state inventory (unchanged from v1; see planner-v1.md §1)

7 commits today; 14 machine-readable artifacts + 1 ADR + 1 Rust validator crate + 25 codex consensus outputs archived; ~9000 lines net.

## §2 Accumulated user directives (unchanged; v1 §2 lists 25 chronologically)

## §3 Standards in force (unchanged; v1 §3)

14 kinetic actions + 4 workflows + 9 evidence classes + 10 HG gates + 9 stop conditions + 9-cap contract + markdown retirement + 5-tier verification + grit-with-ICM-fallback.

## §4 Queued slices (architect-r1-amended)

Per architect r1 amendment #1: define ONE resource-controller pattern before adding new meta-layer classes. The pattern is `desired registry row → admission validation → reconcile → status/evidence`.

| # | Slice | Allowed under narrowing? | Notes |
|---|---|---|---|
| **VL** | Vertical enforcement loop (the resource-controller pattern proof) | ✅ FIRST | All hops mandatory |
| 1 | Constitution content redistribution + active lane | ✅ AFTER VL | Migration with drift-reducing lane |
| 2 | Wave 1-5 plan conversion to JSON | ⚠️ Consumer-led only | Per architect r1 amendment #8 |
| 3 | Wave 7 v1.0.0 .json | 🛑 BLOCKED | No consumer demand |
| 4 | ADR migration (88 ADRs) | ⚠️ Paired with check-crate refactor | Per amendment #8 |
| 5 | CLAUDE.md thinning | ✅ AFTER VL | Consumer rewiring |
| 6 | Workflow-task-traceability schema | 🛑 BLOCKED net-new | Per amendment #1 |
| 7 | Workflow+Ontology dogfooding ADR | 🛑 BLOCKED net-new | Per amendment #1 |
| 8 | Foundation handoff (cosign/trivy/audit-chain/OpenBao/KMS) | parallel-session | External |
| 9 | Ops Wave 1 impl (M02-P19 + M03-P04..P06 docs/workspace BCs) | ✅ AFTER VL | Real impl |

## §5 Vertical enforcement loop blueprint (architect-r1-amended)

Per architect r1 execution order (lines 56-63), the 7-step ordered slice:

| Step | Action | Acceptance |
|---|---|---|
| 1 | Wire `dev-cli gate validate active-artifact-contract` | subcommand callable; loads `/registry/artifact-capabilities-registry.json`; invokes kernel validator |
| 2 | Add failing fixture for missing-row applicable artifact | fixture: synthetic JSON under applicable_paths_glob without registry row → validator exit ≠ 0 |
| 3 | Flip `lean-a-active-artifact-contract` to active in registry/quality/lanes.yaml; `scripts/check.sh` or CI invokes the subcommand | lane status=active; CI green run URL recorded |
| 4 | Add grit pre-done/pre-claim validation (or narrow ICM fallback while grit FK blocks) | hook script in `scripts/hooks/`; explicit "degraded mode with expiry" for ICM fallback |
| 5 | Emit evidence/status bundle for lane result (per spec/status pattern amendment #5) | artifact at `/evidence/lane-run-${RUN_ID}.json`; conforming to evidence-bundle-template |
| 6 | Materialize first graph edges from validated row (kinetic CreateArtifact action) | edge written to `/registry/graph/edges-2026-05-13.json` OR audit-chain (foundation-blocked alternative) |
| 7 | Resume consumer-backed migrations only after steps 1-6 land | gate: green CI + failing fixture verified |

## §6 Architectural amendments absorbed from architect r1 (10)

### Amendment 1 — Resource-controller pattern is the canonical primitive

`desired registry row → admission validation → reconcile → status/evidence`. ALL future meta-layer extensions must instantiate this pattern; net-new classes blocked until VL proves it.

### Amendment 2 — Vertical enforcement loop IS the first controller

VL slice above is the proof. No other controller declared until VL operational.

### Amendment 3 — Registry sharding policy (scale SLO)

At >100 rows per registry, MUST shard by stable resource kind. Generated aggregate indexes. No hand-editing monoliths >1000 lines. Implementation deferred until artifact-capabilities-registry exceeds 100 rows (currently 10).

### Amendment 4 — Graph materialization layer

Generated outputs from canonical registries: `nodes`, `edges`, `reverse_indexes`, `unresolved_refs`, `owners`, `freshness`, `impact_queries`. Output paths under `/registry/graph/materialized/`. Generator crate `crates/gen-graph-materialize` (planned; post-VL slice).

### Amendment 5 — Kubernetes-like `spec/status` separation

Per-artifact capability declaration becomes:
- **spec** = declared desired state (current `capabilities` field in artifact-capabilities-registry rows)
- **status** = observed runtime state (validator result, lane result, evidence freshness, last-promotion timestamp)

Status auto-updated by validator + reconciler; never hand-edited. Schema extension: capability rows gain `_status` sibling section populated by VL.

### Amendment 6 — Admission severity levels

Three tiers added to `lean-a-*` lane definitions:
- `severity: block` — PR blocked (operational claims only can use this)
- `severity: warn` — PR allowed with warning (planned claims with prerequisite_for_operational)
- `severity: report` — informational only (not-applicable claims)

`lean-a-active-artifact-contract` starts at `severity: warn` until VL lands → flips to `severity: block`.

### Amendment 7 — DRY enforceability (operational, not aspirational)

Drift-detector becomes mandatory consumer of reusable-building-blocks-registry:
- Duplicate-pattern scan across `consumer_refs`
- Consumer-reference resolution check (every cited path is HEAD-tracked)
- Canonical block version pinning enforced
- Automated `consumer_count_resolved` recomputation
- Lane: `lean-a-dry-drift` (severity=warn initially; block when scan stable).

### Amendment 8 — Markdown retirement is CONSUMER-LED

Per architect r1 line 46: every migration MUST update the consuming validator/generator AND include a fixture proving the old failure mode is caught. Reframes markdown-retirement-policy:

| Migration unit | Required pair |
|---|---|
| docs/decisions/ADR-*.md → registry | Check-crate refactor (check-adr-citation + check-adr-index) + failing fixture (old MD citation → new JSON path) |
| docs/microservices/*.md → registry | doc-coverage check-crate refactor + fixture |
| docs/runbooks/*.md → registry | runbook-freshness check-crate refactor + fixture |
| etc. | per-category |

### Amendment 9 — ICM scaffold-lock = degraded mode with expiry

NOT normal operating mode. Add to `/specs/master-plan-sequencing.json`:
- `icm_fallback_max_age_hours: 24`
- `icm_fallback_alert_threshold_hours: 4`
- Alert lane: `lean-a-grit-fallback-stale` flags ICM scaffold-locks older than threshold.

Long-term: investigate grit SQLite FK root cause OR build Oyatie-native VCS replacement (per autonomous-prompt directive).

### Amendment 10 — Control-plane scale SLOs

Add to `/specs/control-plane-slos.json` (NEW; allowed under narrowing as it's the operationalization-spec for VL):
- `validation_runtime_p99_seconds: 5`
- `graph_build_p99_seconds: 30`
- `stale_state_window_max_minutes: 60` (warn) / `240` (alert)
- `registry_shard_max_lines: 1000`
- `registry_row_count_shard_trigger: 100`

## §7 Scale-failure modes acknowledged (per architect r1)

1. Registry monolith pressure — addressed by amendment #3 (sharding).
2. Graph without storage/query model — addressed by amendment #4 (materialization).
3. Policy without admission — addressed by amendment #2 (VL is the admission proof).
4. Reconciliation gap — addressed by amendment #5 (spec/status).
5. Evidence cardinality explosion — addressed by rollups + freshness windows (added to amendment #5; status field carries rolled-up summary).
6. Markdown migration blast radius — addressed by amendment #8 (consumer-led).
7. Grit fallback risk — addressed by amendment #9 (degraded mode + expiry).

## §8 Honest gaps (refined)

1. 0/10 HG gates operational.
2. 0 capabilities operational (status-section per amendment #5 will surface this honestly).
3. Foundation prereqs block ~40% of operational gates.
4. ICM-fallback used 7× today; degraded-mode policy not yet enforced (amendment #9 introduces it).
5. Markdown retirement 0.8% complete; consumer-led migration policy clarified per amendment #8.
6. No drift-detector running (amendment #7 introduces lane).
7. No autogen capability operational.
8. No OpenTelemetry exporters (declared, not running).
9. 5 Wave plans still Markdown (Wave 6 only converted).
10. Direction consensus + full consensus both pending approval.

## §9 Standardization audit (per "Standardize everything" directive)

Per architect r1 endorsement: standardization audit is correct in spirit; needs the 10 amendments to be MECHANICALLY ENFORCED, not policy.

| Workflow | Standard | Mechanical enforcement |
|---|---|---|
| Consensus loop | ✅ kinetic workflow | ⚠️ manual today; codex CLI is the engine |
| Markdown retirement | ✅ policy + ledger + amendment #8 | ❌ until consumer-led validator lands |
| Wave acceptance | ✅ kinetic workflow | ❌ until ledger update is automatic |
| Capability promotion | ✅ kinetic workflow + amendment #5 spec/status | ❌ until validator runs lifecycle rule |
| Grit claim/done | ✅ master-plan-sequencing | ⚠️ amendment #9 makes ICM fallback bounded |
| ICM store | ✅ AGENTS.md mandatory triggers | ❌ agent-discretion |
| Commit format | ❌ undeclared | ❌ no commit-msg hook |
| ADR authoring | ⚠️ template; markdown-bound | ❌ no schema-validator |
| **Drift detection** | ✅ amendment #7 | ❌ until lean-a-dry-drift lands |
| **Spec/status sync** | ✅ amendment #5 | ❌ until VL reconciler lands |

## §10 Conclusion (proposed full consensus)

Adopt α + 10 architectural amendments + freeze narrowing. Concrete next-action: execute the 7-step VL slice. After VL operational:

1. Resume consumer-backed migrations (Constitution, Wave plans, ADRs) class-by-class.
2. Add drift-detector lane.
3. Add spec/status auto-population.
4. Honest baseline at end-of-VL: 1 capability operational (verification on artifact-capabilities-registry); 9 capabilities still planned/blocked.

---

**Awaiting architect r2 (hyperscaler lens) re-review of v2 + critic r1 (Torvalds lens).**
