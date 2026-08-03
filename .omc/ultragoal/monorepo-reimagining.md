# Monorepo Reimagining — grounded (G/M/A/MS) + doubt-checked

**Method:** 4 grounded web-research agents (google3/Bazel · fbsource/Buck2 · Amazon Brazil · MS 1ES/VFS) → opus synthesis → opus design → opus adversarial doubt. Raw: `scratchpad/codex-parallel-dev-plan-output.txt`? no — `tasks/wpbjimkvg.output` + `subagents/workflows/wf_c2eff6ba-1fe/journal.jsonl`. Sol Ultra cross-check QUEUED (codex auth down).
**Verdict:** direction CORRECT + substantially landed; **HOLDS with-fixes** — 3 load-bearing claims falsified by inspection.

## The invariant (what G/M do that scales)
**The tree = the build graph = the ownership map.** One directory = one build package = one ownership unit = one node in an enforced dep DAG. Causal chain: explicit fine-grained target graph (keystone) → affected-set CI + default-private visibility are DERIVED from it. One-version vendored deps (single ingestion) + build-at-head (atomic refactor) + per-directory OWNERS (sublinear review) + artifacts-out-of-tree. Amazon is the counter-example (polyrepo + version-sets = no graph-level maintainability/scalability). Clean-arch `core|ports|adapters|facade` = **Google's target-visibility discipline given human-readable directory names** (facade/ports=public API, core=default-private, adapters=the sole third-party linker) — preserved deliberately for portability + stack agility, NOT flattened.

## The correct shape (design)
FLAT capability-first, **no `app/` bucket**, tier as metadata not a directory (ADR-0132 no-grouping):
- **Owned-stack spine** (upward-only): `kernel/` (Asterinas KEEP) → `os/` → `k8s/`.
- **Platform capabilities** (tier=platform): iam, **policy (NEW — extracted PDP)**, data, storage, network, dns, compute, cell, secrets, kms, messaging, intelligence, observability, audit, compliance, tenancy, billing, workflow, ci.
- **Product capabilities** (tier=product, flat single-concern): accounting, crm, drive, mail, meet, sheets, emr, hr, payroll, console, … (depend DOWN on platform; never sideways-by-convention).
- **Foundation:** `libs/` (pure ZERO-I/O kernels, upward-only; CI-gate consumers in `ci/` never here) · `third-party/` (sole reindeer-vendored one-version ingestion) · `contracts/` `registry/` `packs/` `governance/corpus` `docs/` `specs/` · **`.archive/`** (Framekernel + zero-crate scaffolds, OUT of the build graph).
- Each capability: `core/ ports/ adapters/ facade/ observability/ slos/ manifest.json OWNERS BUCK .facts/`.

**Honest verdict: COMPLETION + ENFORCEMENT of the already-ratified shape, not a new topology.** The flat de-branded caps ALREADY EXIST on origin/dev beside the deprecating `cloud/cloud-*` + `oya/*` strangler sources. Only `policy/` is genuinely new.

## What doubt FALSIFIED (the fiction vs the landed)
1. **The enforcement is aspirational, not mechanical.** There is **NO `package_group` tier-DAG in any BUCK file** on origin/dev; the registry has no buck2 binding; `slos/` is absent from all 17 platform/spine caps. So "STRUCTURAL, tree=graph physically cannot drift, slos-before-promotion" is **lint-advisory today** — and it's the justification for the flat ~90-cap root.
2. **Clean-arch already broken where it matters most.** `data/` (sqlx/RLS/etcd — the canonical evictable-infra cap) has **NO `adapters/`**; `compute/` (Pulsar/Oxia/cell tier) has **NO `ports/`** (no cutover seam). The two caps that most need the seam already violate it.
3. **The tree is a sketch.** It drops ~12 real roots (tools/ scripts/ bin/ infra/ iac/ templates/ tasks/ plan/ benchmarks/ evidence/ marketplace/ comms/ — tools/scripts/bin are the actual junk-drawers + violate shell/CLI-retirement), leaves **~80 of 82 `oya/` dirs unmapped** (the hard boundary disputes — data-pipeline→data/? api-gateway→gateway/? eventing→messaging/? — undesigned), and `tenancy/` bundles 4 single-concern services (its own ADR-0132 violation). Live 3-way dup: `flags/` vs `oya/feature-flags` vs `oya/oya-flags`; `gateway/` vs `api-gateway` vs `ci-webhook-gateway`. 187 `libs/oya-*` still brand-prefixed.

## Parallelism — the REAL bottleneck (answers the app/ question)
Not `app/` (the design eliminated it). The genuine cross-lane serializers are the design's OWN central hubs: **closed registry `bounded-contexts.json`** (every cap add contends), **one-version `third-party/`** (every dep bump serializes), **`contracts/` SSOT** (every cross-cap API change serializes), and the **spine** (a `k8s/` port change invalidates the whole-repo affected-set). Fix = shard the registry into per-capability fragments + per-capability contract ownership + sequence spine/hub changes.

## CROSS-MODEL VERDICT (terra-ultra, codex/GPT lens) — SUPERSEDES the contested opus calls

Terra (resumed after sol hit capacity) **disagreed with opus on 4/5 contested points with committed evidence**, and caught the category-error both opus lenses missed. The reconciled answer:

**CONVERGES with opus:** capability-first semantics, clean-arch preservation, disposition trichotomy (move/archive/keep), "completion + enforcement not new topology," de-brand, .archive for genuinely superseded.

**TERRA CORRECTS opus:**
1. **NOT a flat ~90 root — use a fixed KIND axis.** The registry closes capabilities at **23** (not 90); `microservice-tier-classification.json` = 101 *services* (55 substrate/46 product); 33 `app` product compositions. Opus conflated capability/service/product into one dir. ADR-0132 forbids bundle *capabilities*, NOT *namespace directories* (opus's conflation). **Correct top-level:**
   ```
   kernel/  os/  base/  cap/<capability>/{core,ports,adapters,facade}/  app/<product>/  governance/  build/  third-party/
   ```
   k8s is a capability, not a "spine tier." `cap/` is an inert namespace (no monolithic manifest/BUCK) → no write-contention.
2. **RETAIN `app/<product>` — do NOT eliminate it.** ADR-0562 explicitly defines `app/<product>` as the composition ring for surfaces wiring 2+ capabilities (`:80-97,138-151`); 33 `oya/*` surfaces map to it; `oya/application` is exactly such a composition (8 crates, deps across 11 caps). Glob workspace membership + per-product BUCK/PACKAGE = no shared write surface. Opus's app/-kills-parallelism premise is false.
3. **Enforcement keystone: `package_group` is BAZEL, not Buck2, and NOT first.** Buck2 uses `visibility` + `within_view` (first-order only) + inherited `PACKAGE` files. Committed: 938 BUCK files, **ZERO** PACKAGE/within_view/package_group; prelude is Buck2's bundled external (not repo-owned/pinned). Enforcement is eventually right but comes AFTER authority+ontology reconciliation + a canary.
4. **"only adapters depend on third-party" is UNSOUND.** 733/1035 third-party occurrences are legitimately outside adapters (core uses Ed25519/BLAKE3/Serde). **Enforce dependency CLASS (volatile provider/driver vs stable primitive), not path-provenance.**
5. **Clean-arch "holes" are path-taxonomy gaps, not violations.** compute's cutover seam EXISTS (`ComputeProviderVmPort` in compute/core/domain:428-494; AWS/OCI adapters depend inward = valid hexagonal). data's real issue = `data/facade/analytics-app` wires a ClickHouse adapter still under `libs/`. Fix: move volatile provider clients to `data/adapters/`; don't add empty dirs to satisfy a sketch.

**THE BLOCKER both opus lenses missed — AUTHORITY/ONTOLOGY is unreconciled:**
- **ADR-0562 + ADR-0615 are still `Proposed`** despite describing founder-ratified one-way decisions.
- Registry = 23 caps + **24 unique dag_node labels**, but `substrate-dependency-dag.json` = only **10 nodes/42 edges** → **14 registry DAG labels have no DAG node** (api-gateway, billing, comms, compliance, compute, console, control-plane, delivery-fabric, flags, iac, marketplace, messaging, network, storage).
- Registry absorbs `oya/connector`→gateway, but connector's README says it is NOT the gateway (outbound integration/OAuth/webhook).
- Registry maps `oya/application`→`app/`, which opus deletes.
→ "tree=graph=ownership" enforcement would compile an incomplete + contradictory ontology.

**Single strongest objection both opus lenses missed:** they collapse **capability + service + product + ownership** into one namespace unit. The committed model = 23 capabilities / 101 services / 33 product compositions — a flat "~90" root codifies the WRONG ontology.

**82-dir ambiguous-set disposition (terra, no archive-for-zero-crates):** analytics→`cap/data` MOVE · api-gateway→`cap/gateway` MOVE · application→`app/application` MOVE · **connector→`cap/integration` NOT gateway — FOUNDER CALL** · data-pipeline→`cap/data` MOVE · data-warehouse→`cap/data` MOVE · eventing→already `messaging/` KEEP · ontology→`cap/data` MOVE (future-split founder call) · detection→`cap/intelligence` MOVE (boundary review) · **developer-sdk→DECOMPOSE (app/developer-platform + engines split) FOUNDER CALL** · ops→already `cap/console` KEEP · ops-dashboard-control-center→DECOMPOSE (console shell + `app/ops-console/<vertical>`) · plugin-app-store→`cap/marketplace` MOVE · search→already `cap/data` KEEP · consent-graph→`cap/iam` MOVE.

**RANKED FIRST MOVES (terra — authority before code):**
1. **Resolve authority** — formally Accept/amend ADR-0562 + ADR-0615; choose the kind-axis; preserve `app/`.
2. **Reconcile the ontology** — one accepted mapping across capability/service/product/owner/SLO/DAG; repair the 24-label/10-node mismatch.
3. **Publish the full 82-dir disposition ledger** (move dest / roadmap basis / owner / archive proof; escalate connector, developer-sdk, ontology/detection splits).
4. **Define the edge model** (direct vs transitive, face-to-face allowed edges, composition-root exceptions, dependency classes).
5. **Pin + prove Buck2 policy mechanics** (own+pin the prelude; a `PACKAGE`/`within_view` canary across core/adapter/facade + negative fixtures + Cargo/Buck parity).
6. **THEN enforcement** — dual Cargo+Buck acyclicity lint + Buck2 direct-edge controls; advisory→warn→fail-closed after coverage.
7. **Migrate one capability/app-product at a time** — reversible codemods, glob workspaces, generated projections; NO bulk archive/root rewrite.

## Required fixes (keystone first) — SUPERSEDED by terra's ranked first moves above
1. **Land the `package_group` tier DAG in BUCK** (products→platform→k8s→os→kernel; libs upward-only; adapters=sole third-party linker) + bind capability existence to registry membership at graph-construction — until then, downgrade all "physically cannot drift" language to "lint-advisory."
2. **Complete the `oya/` 82-dir MOVE/ARCHIVE/KEEP mapping** incl. the ambiguous substrate-shaped dirs (analytics, api-gateway, data-pipeline, data-warehouse, eventing, ontology, connector, search, …) + rule on the ~12 omitted roots (tools/scripts/bin → retirement).
3. **Fix the clean-arch holes:** `compute/` gets `ports/`+`observability/`; `data/` gets `adapters/` (its sqlx/RLS/etcd edges have no legal home today); land `slos/` — or stop asserting uniform-shape as current fact.
4. **Resolve the live duplications** (flags/gateway/observability three-ways) + de-brand the 187 `libs/oya-*`.
5. **De-bundle `tenancy/`** into single-concern targets (re-home managed-k8s-sla-observability → observability).
6. **Shard the central hubs** (registry fragments + per-cap contracts) or retract the "no central bottleneck" claim.
