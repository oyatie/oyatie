# Critic r1 — Direction Consensus 2026-05-13

## Verdict
ITERATE

## Session
codex thread id `019e246f-61de-7f81-8272-30aae52e274f`

## Quality-Criteria Check
- Principle↔option consistency:    PASS — Planner's 5 principles coherently point toward Option α only if α becomes enforcement-first. Planner itself admits α currently has validator unwired and lane planned, so "Principles satisfied: 1-5 all" is overstated, but the option/principle mapping is directionally coherent.
- Fair alternatives (Option β):     FAIL — β is not presented cleanly. Planner says β's risk is that "meta-layer remains paper until validator + lane + workflow-task-traceability all wired", but β's first move is exactly wiring the validator/lane. That partially straw-mans β by describing it as leaving the contract paper instead of treating it as the minimal proof slice.
- Risk-mitigation clarity:          FAIL — The "Open tensions" list names real risks (breadth-over-depth, foundation dependency, markdown breakage, unbounded slices), but it does not specify mitigations, tripwires, or stop conditions. Architect supplies the first actual mitigation: freeze net-new meta-layer work until one vertical enforcement loop is load-bearing.
- Acceptance-criteria testability:  PASS — Architect's vertical loop is testable: registry row -> schema validation -> validator runtime -> `oya` command -> grit/pre-claim or pre-done validation -> CI lane active -> evidence bundle -> graph edge update. Each hop can be checked by a command, tracked file, failing fixture, lane row/status, and evidence artifact.
- Verification concreteness:        PASS — Architect's next move is concrete: integrate `crates/oya-check-active-artifact-contract` into `oya-dev-cli`, flip `lean-a-active-artifact-contract` only with evidence, add a failing fixture for an unregistered artifact, and wire graph/registry output. Planner is less concrete, but the amendment makes the consensus actionable.

## User-Mandated-Rule Check
- (i)   honest-claims:           FAIL — Planner claims α satisfies all five principles while Principle 1 says drift becomes CI failure, but `lean-a-active-artifact-contract` is still `status: planned`; `scripts/check.sh` does not run `gate validate active-artifact-contract`; and `oya-dev-cli` help/dispatch search shows no active-artifact-contract subcommand.
- (ii)  Linus-grade:             FAIL — There is silent-regression risk already visible: `/specs/root-hub-pointers.json` still points `knowledge_graph_catalog.current_path` at `/registry/knowledge-graph-catalog.json`, while tracked HEAD has `/registry/knowledge-graph-semantic.json`, `knowledge-graph-kinetic.json`, and `knowledge-graph-dynamic.json` instead. That is exactly the kind of stale pointer a load-bearing validator should catch.
- (iii) verified-claims:         FAIL — `git ls-files` confirms these cited files exist: `/specs/active-machine-readable-artifact-contract.json`, `/registry/artifact-capabilities-registry.json`, `/specs/artifact-profile-defaults.json`, `/specs/markdown-retirement-policy.json`, `.omc/ledger/markdown-retirement-ledger.json`, `/specs/root-hub-pointers.json`, `/specs/master-plan-sequencing.json`, `/specs/hyperscaler-gates.json`, `/specs/evidence-taxonomy.json`, `/specs/stop-conditions.json`, `/specs/final-report-schema.json`, `/specs/plan-schema.json`, `docs/decisions/ADR-0709-general-live-apex.md`, and `crates/oya-check-active-artifact-contract/{Cargo.toml,src/lib.rs}`. But `git ls-files` does NOT return `/evidence/audits/consensus/2026-05-13-direction/planner-v1.md`, `/registry/knowledge-graph-catalog.json`, or the architect-named `semantic-knowledge-graph-catalog.json` / `kinetic-knowledge-graph-catalog.json` / `dynamic-knowledge-graph-catalog.json` paths. Actual tracked split paths are `/registry/knowledge-graph-semantic.json`, `/registry/knowledge-graph-kinetic.json`, and `/registry/knowledge-graph-dynamic.json`.
- (iv)  honest-introspection:    PASS — Both planner and architect identify breadth-over-depth, planned enforcement, markdown migration breakage, and foundation gaps. The gap tracking is real; the problem is that some claims still outrun that introspection.

## Architect-amendment soundness audit
Is "freeze new meta-layer; prove vertical enforcement loop first" sound?
- Pros: It attacks the highest-risk failure mode: control-plane declarations without admission/runtime force. It also aligns with honest-claims and Linus-grade by making one validator/lane/fixture/evidence path block drift before more registries accumulate.
- Cons: The wording is too blunt. "Freeze new meta-layer" can be misread as blocking repairs to existing meta-layer correctness, pointer migration, Constitution-content redistribution, or Wave 7 conversion when those are required to make existing promises coherent.
- Over-correction risk: Medium. Freeze net-new meta-layer classes and non-load-bearing expansions; do not freeze corrective migration or consumer rewiring. Constitution redistribution is content repair after a user directive, not a new class. Wave 7 conversion is legitimate only if class-by-class, consumer-backed, and covered by a failing fixture or active lane; otherwise it is more paper.

## Fixes (only if not APPROVE)
1. Rewrite Option β as "operationalize the already-created contract first" rather than a pivot that leaves the meta-layer paper.
2. Narrow the architect amendment to: "freeze net-new meta-layer classes; allow enforcement-loop work, stale-pointer repair, consumer rewiring, and migration slices only when they reduce drift and add a failing fixture or active lane."
3. Correct all cited paths to tracked HEAD names and fix `/specs/root-hub-pointers.json` knowledge-graph pointer before claiming verified-claims.
4. Make `lean-a-active-artifact-contract` honest: keep planned until `oya-dev-cli` dispatch exists, `scripts/check.sh` or CI invokes it, and an unregistered-artifact fixture fails.

## Recommended consensus position
Adopt α only under a narrowed enforcement-first amendment. The next accepted slice is not another ontology/registry/policy surface; it is the active-artifact vertical loop: tracked registry row, schema parse, `crates/oya-check-active-artifact-contract` runtime wired through `oya-dev-cli`, failing unregistered-artifact fixture, active `lean-a-active-artifact-contract` lane in CI/check path, evidence bundle, and graph edge update. After that, resume Constitution redistribution, Wave 7 conversion, and markdown retirement only as consumer-backed migrations with hard validation, not as free-standing meta-layer expansion.
