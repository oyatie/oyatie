# Direction Consensus v1 (2026-05-13) — pending approval

## Verdict

**Adopt α (continue current direction) WITH NARROWED AMENDMENT.**

Both architect r1 and critic r1 returned ITERATE on planner v1, but converge on the same synthesis. Consensus position below incorporates architect r1's amendment + critic r1's narrowing.

## Consensus position

### What stays (α as adopted)

1. Active-artifact contract v3.0.0 (ADR-0069 Accepted) + 9-capability declaration.
2. Knowledge-graph 3-layer split (semantic / kinetic / dynamic per Palantir).
3. DRY enforcement via reusable-building-blocks-registry.
4. Markdown retirement per policy (except README + CLAUDE + AGENTS at root).
5. Grit-only state transitions with ADR-0054 ICM scaffold-lock fallback.
6. 5-tier verification taxonomy + S3a (plan-stage) vs S3b (operational-stage) split.
7. Honest claim boundaries (claim-matrix can/cannot/required).

### What freezes (architect r1 + critic r1 narrowing)

**FREEZE net-new meta-layer CLASSES** until the first vertical enforcement loop is operational. Specifically blocked: new top-level capability registries, new layer-schemas, new policy specs, new claim-matrix kinds.

### What is ALLOWED during the freeze (critic r1 narrowing)

- **Enforcement-loop work** (highest priority): wire `crates/oya-check-active-artifact-contract` into `oya-dev-cli`; flip `lean-a-active-artifact-contract` from `planned` to `active`; add failing fixture for unregistered machine-readable artifact; record green CI run URL.
- **Stale-pointer repair**: e.g., the `/specs/root-hub-pointers.json` stale `knowledge-graph-catalog.json` pointer (FIXED inline this batch per critic r1 finding #3).
- **Consumer rewiring**: when an artifact moves (e.g., catalog → 3-layer split), update all inbound references.
- **Migration slices** (Constitution-content redistribution, Wave plan conversions, ADR migration): only when each slice **reduces drift** AND adds a failing fixture OR active lane.

### What is BLOCKED

- New free-standing meta-layer surfaces (e.g., a "workflow-task-traceability schema" as a separate spec without consumer-backed validator).
- Wave 7 JSON conversion **unless** Wave 7 has a consumer that breaks without it (it doesn't yet — `pending approval` only) → blocked until consumer demand.
- Constitution content redistribution as plain re-org → allowed ONLY if it adds an active CI lane that fails when redistributed content drifts from source.
- ADR migration (88 files) → blocked until validator is operational; each ADR migration must be paired with a check-crate refactor proving JSON-consumption works.

### Acceptance criteria for the next slice (vertical enforcement loop)

Per architect r1 synthesis, the next accepted slice MUST hit all 8 hops:

1. **Tracked registry row** in `/registry/artifact-capabilities-registry.json` (already exists; ≥1 row)
2. **Schema validation** that fails on schema-violating rows
3. **Validator runtime** = `crates/oya-check-active-artifact-contract::validate` (already exists; 12 unit tests pass)
4. **`oya` command** = `oya-dev-cli gate validate active-artifact-contract` subcommand (NEW: write this)
5. **grit/pre-claim or pre-done validation** (or scaffold-lock ICM equivalent during grit-FK fallback) (NEW or existing)
6. **CI lane active** = `lean-a-active-artifact-contract` status flips from `planned` to `active` in `registry/quality/lanes.yaml`
7. **Evidence bundle** emitted recording the lane's green CI run URL
8. **Graph edge update** — kinetic action `CreateArtifact` triggers; semantic graph reflects the new row

A FAILING FIXTURE must exist: an artifact under `applicable_paths_glob` without a registry row → validator returns exit code != 0 → CI lane red.

### Critic r1 fixes addressed inline this batch

- **#3 (stale pointer)**: `/specs/root-hub-pointers.json` `knowledge_graph_catalog` row replaced with 3-layer split rows (`knowledge_graph_semantic` + `knowledge_graph_kinetic` + `knowledge_graph_dynamic`).

### Critic r1 fixes deferred to next slice (the vertical enforcement loop will close them)

- **#1** (Option β fair-rewrite): consensus position now treats β's first move as wiring the validator — same as architect r1 synthesis. Resolved by reframing.
- **#2** (narrowed architect amendment): captured above ("What is ALLOWED during the freeze").
- **#4** (`lean-a-active-artifact-contract` honest until oya-dev-cli dispatch exists + scripts/check.sh runs it + failing fixture): closed by the vertical-enforcement-loop slice acceptance criteria.

## Honest gaps (per critic r1)

1. Planner v1 had verified-claims FAIL (cited `/evidence/audits/consensus/2026-05-13-direction/planner-v1.md` which was not yet HEAD-tracked at architect r1 time + cited deleted catalog path). Both fixed by this commit.
2. Planner v1 honest-claims FAIL (claimed "Principle 1 satisfied" while CI not blocking). Consensus position rewords: Principle 1 = ASPIRATION; vertical-enforcement-loop slice CONVERTS it to operational.
3. Linus-grade FAIL (stale pointer = silent regression). The contract is supposed to mechanically prevent this; until the validator runs in CI it cannot. Closing by the vertical-enforcement-loop slice.

## ADR (per ralplan skill step 6 requirement)

**Decision:** Adopt α (continue meta-layer direction) under critic-r1-narrowed amendment.

**Drivers:**
- User mandates 9-capability + DRY + knowledge-graph + automation everywhere.
- Architect r1 + critic r1 converge that meta-layer ≠ enforcement; enforcement primitive must land before more meta-layer.
- Foundation prereqs (cosign/trivy/audit-chain/OpenBao/KMS) block ~40% of operational gates regardless of direction.

**Alternatives considered:**
- **Option β (pure pivot to operationalization)**: rejected because the architect+critic synthesis IS operationalization-first; α-with-amendment is β functionally.
- **Continue α unamended**: rejected per architect r1 steelman ("control plane built faster than control made effective").

**Why chosen:** the narrowed α preserves prior work (no rollback of 14 artifacts + 1 ADR + 1 crate + 6 commits) while focusing the next slice on the load-bearing enforcement path.

**Consequences:**
- Positive: meta-layer paper becomes operational; HG-DOCS + HG-GRIT + HG-TEST gates close; failing fixture proves drift prevention.
- Negative: 8 of 9 directive-driven queued slices BLOCKED until vertical loop lands (workflow-task-traceability, Wave 7 JSON, Constitution redistribution, ADR migration, CLAUDE.md thinning, dogfooding ADR all defer).

**Follow-ups:**
1. Implement vertical enforcement loop (oya-dev-cli + active lane + failing fixture + evidence + graph edge).
2. After loop active, resume queued slices class-by-class with consumer-backed migration.
3. Foundation handoff: as cosign/trivy/audit-chain land, promote dependent capabilities to operational.

---

**Status: pending approval (per ralplan skill rule).** Awaiting user decision: approve via team / approve via ralph / approve after context clear / request changes / reject.
