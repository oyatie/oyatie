# Architect r1 (Hyperscaler lens) — Full-project Consensus 2026-05-13

## Verdict
ITERATE

## Session
n/a

## Steelman synthesis (strongest case FOR current architecture)
The architecture is directionally correct for a hyperscaler-grade repo control plane. The active-artifact contract is the AWS Config/Kubernetes CRD move: separate resource data from compliance/control metadata, declare capabilities centrally, then let validators and lanes decide what is allowed. The three-layer ontology is the right Palantir Foundry pattern: semantic types define what exists, kinetic actions define allowed mutations, and dynamic sources expose live state. The DRY registry borrows the Cargo workspace.dependencies/Maven BOM instinct: centralize versioned reusable primitives so consumers reference rather than copy. Markdown retirement is coherent if JSON/TOML/YAML become canonical resources and Markdown becomes projection or audit archive. Grit as the repo-state primitive is also correct: for agent-scale work, symbol claims and atomic done semantics beat a homegrown web of coordination kernels.

The strongest case is that Oyatie is designing its own internal cloud asset inventory before the product estate gets bigger. At 10k artifacts, manual docs are already dead; only typed assets, ownership rows, graph edges, validators, and reconciliation loops can survive. The planner also states the honest baseline: 0/10 HG gates operational, 0 capabilities operational, validator kernel exists but is not wired. That honesty is a strength because it prevents false maturity claims.

## Scale-failure modes (at 10k artifacts / 100 µservices / 1M edges)
1. Registry monolith pressure: single JSON registries will become hot files, merge bottlenecks, slow parse targets, and poor query engines. At 10k artifacts they need sharding, stable IDs, indexes, and generated projections.
2. Graph without storage/query model: 1M edges cannot be operated by grep plus hand-curated JSON. Need materialized indexes, edge freshness, orphan detection, impact queries, and ownership-aware traversal.
3. Policy without admission: today many standards are declared but not blocking. At 100 microservices, planned lanes become theater unless write paths fail before bad artifacts land.
4. Reconciliation gap: hyperscalers do not trust declarations. They run controllers that continuously compare desired state to observed state and emit drift/compliance state.
5. Evidence cardinality explosion: per-artifact 9-capability evidence creates 90k capability states at 10k artifacts. Without rollups, freshness windows, and sampled/detail views, operators drown.
6. Markdown migration blast radius: retiring 250+ Markdown files while check crates still parse Markdown can silently break validators and citations unless migrated behind fixtures and consumers.
7. Grit fallback risk: ICM scaffold locks are acceptable only as degraded mode. If they become normal, Oyatie has two coordination systems and no single source of truth.

## Hyperscaler reference comparison
- AWS Config: matches on resource inventory + rule/compliance intent; falls short because compliance evaluation is not yet continuously run, aggregated, or remediated.
- GCP Asset Inventory: matches on asset graph ambition; falls short on export/query/IAM-style access model, indexed relationship traversal, and historical snapshots.
- K8s CRD + admission: matches on schemas and declared resources; falls short because admission is not wired into `oya`, grit/pre-done, or CI. No controller reconciles desired vs observed state yet.
- Cargo workspace.dependencies + Maven BOM: matches via DRY registry and central block/version ownership; falls short until consumers are mechanically required to reference the canonical block and duplicate inline patterns fail.
- Linkerd CRD: matches the narrow-resource + controller direction; falls short because Oyatie’s CRDs/specs are broad and numerous before a small controller loop proves the pattern.
- Palantir Foundry: matches the semantic/kinetic/dynamic ontology source pattern; falls short on operational ontology functions, lineage queries, permissioned graph views, and live state backfill.

## Principle violations (per planner's 5 principles)
1. Active-artifact contract over passive docs: PARTIAL/VIOLATED. The contract exists, but drift is not yet a CI failure; lane status is planned.
2. Knowledge graph as substrate: PARTIAL. The split is sound, but no query path or enforcement path proves registries are graph views.
3. DRY enforcement structural: PARTIAL. Reusable block registry exists, but duplicate detection and consumer-reference checks are planned.
4. Zero hand-authored Markdown: AT RISK. The policy is clear, but migration before consumer rewiring will break check crates and lose narrative intent.
5. Grit-only state transitions: PARTIAL. The principle is correct, but direct tool exceptions and ICM fallback must stay auditable, temporary, and narrower than grit.

## Specific architectural amendments to reach hyperscaler bar
1. Define one resource-controller pattern before adding new meta-layer classes: `desired registry row -> admission validation -> reconcile -> status/evidence`.
2. Make the vertical enforcement loop the first controller: `oya-dev-cli gate validate active-artifact-contract`, failing fixture, active lane, grit pre-done/pre-claim hook, and evidence bundle.
3. Split large registries by stable resource kind or shard key, with generated aggregate indexes. Do not hand-edit monoliths at 10k scale.
4. Add graph materialization: generate `nodes`, `edges`, reverse indexes, unresolved refs, owners, freshness, and impact queries from canonical registries.
5. Promote capability state to Kubernetes-like `spec/status`: declared desired capability is spec; observed validator/lane/evidence result is status.
6. Add admission severity levels: block, warn, report; only operational claims can use block.
7. Make DRY enforceable: duplicate-pattern scan, `consumer_ref` resolution, canonical block version pinning, and automated consumer-count recomputation.
8. Keep markdown retirement consumer-led: every migration must update the consuming validator/generator and include a fixture proving the old failure mode is caught.
9. Treat ICM scaffold locks as degraded mode with expiry and alert; grit remains the single normal state-transition system.
10. Add scale SLOs for the control plane: max validation runtime, max graph build time, max stale-state window, max registry shard size.

## Direction-consensus narrowing (freeze net-new meta-layer)
Sound. The freeze is exactly the hyperscaler move. AWS Config, GCP Asset Inventory, K8s, Linkerd, Cargo, and Maven all win by making a small set of primitives operational before expanding surface area. New meta-layer classes now would increase state without increasing control. Allow only enforcement-loop work, stale-pointer repair, consumer rewiring, and drift-reducing migrations with failing fixtures.

## Recommended next-action
APPROVE current direction only with this amendment: no net-new meta-layer class until the vertical enforcement loop is operational end-to-end.

Execution order:
1. Wire `oya-dev-cli gate validate active-artifact-contract`.
2. Add failing fixture for an applicable machine-readable artifact missing a registry row.
3. Flip `lean-a-active-artifact-contract` to active and make `scripts/check.sh` or equivalent CI invoke it.
4. Add grit pre-done/pre-claim validation or a narrow fallback if grit FK remains blocked.
5. Emit evidence/status for the lane result.
6. Materialize the first graph edges from the validated registry row.
7. Only then resume consumer-backed markdown/ADR/wave migrations.
