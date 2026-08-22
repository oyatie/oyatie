# Direction-consensus Planner v1 (2026-05-13)

**Scope:** consensus on the project's current architectural trajectory and the 6 commits landed today.

## RALPLAN-DR summary

### Principles (5)

1. **Active-artifact contract over passive docs.** Every machine-readable artifact declares the 9-capability contract (enforcement / verification / validation / autogen / selfheal / selfupdate / selfmaintain / telemetry / provenance) per ADR-0069 v3.0.0. Drift becomes a CI failure, not a soft alert.
2. **Knowledge graph as substrate.** Project state — relationships, capabilities, features, schemas, tasks, ownership — is modeled as a typed graph (semantic / kinetic / dynamic per Palantir Foundry 3-layer). Registries are views into the graph.
3. **DRY enforcement is structural, not procedural.** Reusable blocks live in a single central registry; consumers reference, never duplicate. Drift-detector planned for nightly DRY-invariant scan.
4. **Zero hand-authored Markdown** except `README.md` (human-readable hub), `CLAUDE.md` + `AGENTS.md` (agent pointer hubs). All other Markdown migrates to JSON/TOML/YAML per markdown-retirement-policy 8 phases.
5. **Grit-only state transitions with honest fallback.** Every agent→repo state transition uses `grit claim → work → done`; SQLite-FK failure mode triggers ADR-0054 ICM `scaffold-locks-oyatie` fallback (logged in claim-matrix HG-GRIT row).

### Decision Drivers (top 3)

1. **Hyperscaler-grade discipline mandated by user** (AWS Config + GCP Asset Inventory + K8s CRD + Cargo workspace.dependencies precedent). Plan-stage maturity = current state; operational-stage maturity = post foundation-prerequisite landing.
2. **Foundation prereq gap is real** (docker/podman/kubectl/cosign/trivy/audit-chain/OpenBao/KMS blocked in parallel session). ~40% of HG-* gates and ~30% of Wave 5/6/7 capabilities depend on foundation. Honest claim boundary respected via 5-tier verification taxonomy.
3. **Don't defer anything** + **plan everything out first** directives. All architect-r17/r18/r19 deferred findings closed in commits b0798b0 + 6938c89; ~250 markdown files queued for retirement with per-file ledger rows.

### Viable Options (2)

**Option α (adopted; current direction) — Continue the meta-layer build-out**

- Strengths: 6 commits today establish active-artifact-contract + knowledge-graph (semantic/kinetic/dynamic) + markdown retirement substrate. Foundation for all subsequent automation. Codex Torvalds-lens (r17 → r18 → r19) consistently ITERATEd then APPROVEd. ADR-0069 Accepted.
- Costs: 9 directive-driven follow-on slices queued (Wave 7 conversion, workflow-task-traceability, Constitution redistribution, CLAUDE.md thinning, Wave 1-5 plan conversion, ADR migration, dogfooding ADR, dev-cli validator wiring). Each is non-trivial.
- Risks: (a) breadth-over-depth — meta-layer expanding without yet operationalizing (validator unwired; lane status=planned); (b) markdown-retirement migration breaks ~12 check crates that grep markdown; (c) parallel-session foundation gap delays operational-stage close.
- Principles satisfied: 1-5 all.

**Option β (alternative) — Pivot to operationalization NOW**

- Strengths: wire `oya gate validate active-artifact-contract` (1 day) → lane flips status=planned to active → the contract becomes REAL not paper. Then begin ops Wave 1 implementation (M02-P19 + M03-P04..P06) per masterplan.
- Costs: leaves 8 directive-driven slices uncloosed (workflow-task-traceability, Palantir-3-layer was just done so OK, Constitution redistribution, Wave 7 JSON, ADR migration, etc.).
- Risks: meta-layer remains paper until validator + lane + workflow-task-traceability all wired; user has been emphasizing the meta-layer in 10+ recent directives.
- Principles satisfied: 1-3 + 5 (skips Principle 4 markdown retirement; defers).

### Recent changes recap (today's 6 commits)

| Commit | Summary |
|---|---|
| `5880ce0` | ralplan-ops-portal v7 + Waves 2-5 Accepted (1655 ln) |
| `3d6de67` | ADR-0069 + active-artifact-contract v3.0.0 + minimal Rust validator (9 tests pass; 3451 ln) |
| `b0798b0` | Close r17 #3/#7/#8 + r18 NEW defects (artifact_profile + 14-row consumer surgery + graph split; 2278 ln) |
| `1f96255` | ADR-0069 closeout stale-text purge per r19 |
| `6938c89` | Phase 1 markdown retirement + Constitution scheduled-for-retirement (1043 ln) |
| `0806f91` | Palantir 3-layer ontology split + README.md human-readable revert (352 ln) |

**Net:** 14 new artifacts (5 specs + 5 registries + 4 plans/ledgers/attestations) + 1 ADR + 1 Rust validator crate + 12 codex consensus outputs archived. ~9000 lines net of machine-readable artifacts landed.

## Consensus question

**Is the direction sound?** Or should we pivot to operationalization (Option β) before further meta-layer expansion?

## Open tensions to resolve

1. **Breadth vs. depth.** Meta-layer (active-artifact-contract, knowledge-graph, retirement policy) is breadth. Operationalization (validator wired, lane active, drift-detector running) is depth. Currently breadth wins.
2. **Foundation prereq dependency.** ~40% of HG-* gates and ops-portal capability promotions wait on parallel-session foundation. Continuing meta-layer means more paper.
3. **Markdown retirement breaks CI.** ~12 check crates grep markdown; converting their inputs to JSON requires per-crate rewrites in PHASE-8. Risk: silent CI regressions.
4. **9 queued slices.** Each user directive adds a slice. Without consolidation, scope grows unbounded.

---

**Awaiting architect r1 + critic r1 codex consensus.**
