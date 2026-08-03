# Corpus Governance Substrate — Phase-0 launch plan (tasks #128 + #129), 2026-06-24

Build-ready design (architect, read-only vs origin/dev). Founder headline: "every code element through AST → grasp the entire code graph; docs/ADRs DERIVED, drift impossible, total-accounting." L1 (syn fact extractor) already shipped #127/ADR-0580. This is L2+L3+L4 + registry sharding.

## Layers
- **L1 code FACTS** (BUILT): `corpus_core::Function`, content-addressed via split blake3 — `signature_hash` (stable id) + `body_hash` (drift signal); `ItemKind`{Fn,Type,Impl,Route,PubItem}; `AstSource` trait. Machine-derived, never authored.
- **L2 typed NODES** (new): one `governance/corpus/nodes/<kind>/<id>.node.json` per node (conflict-free: two ADRs = two files). Kinds: decision/ADR, capability, requirement/prd, product/SKU, goal, milestone, slo, cedar-policy(Inv), gate-decl, friction. FK = ID newtypes; dangling FK = hard load error. (route/sql-table/fn/cedar-policy-code are L1 facts, not authored.)
- **L3 derived VIEWS** (new): pure full-fold over L1+L2 → 6 edges (depends-on, governs, derives, justified-by, backs, satisfied-by). Never stored. Sub-second over few-thousand nodes (no salsa).
- **L4 Cedar liveness GATE** (new): 6 invariant families (liveness/orphan, reference-integrity, format, template, freshness, directive-compliance) over should-be-empty derived relations, via the SHIPPED `libs/oya-shared-pdp-adapter-cedar` (governance + prod authz share one engine). Drift-impossible: governs-edge derived from scope-glob match on fact hashes; ADR markdown re-emitted from node (build artifact, byte-validated). Per-class posture: structural→born fail-closed; liveness/deletion→ratchet-only (telemetry-gated, SCARF lesson); doc/directive→staged report→ratchet→blocking on measured FP.

## Content-addressing
L1: blake3 split-anchor (built). L2: blake3(canonical_json(node)); id = human-readable symbol (SCIP lesson). Edge: blake3(src_hash∥kind∥dst_hash) → a moved signature_hash dangles every edge → gate fires (buck2 dirty-tracking for docs).

## Registry shard (the "12.6MB")
Mostly ALREADY de-committed (#828: accounting-registry 11.6MB + 4 faces). Real work = per-capability `<cap>/.facts/{code-facts,nodes-index,liveness}.generated.json` (not-tracked-in-git) + `governance/corpus/.facts/global-index` materialized MAIN-ONLY (de-globalize per cellular-hub-aware). Reuse `resolve_capability_crates`/`oya-workspace-members-kernel` for partition; carve-out-as-DATA + total-coverage invariant preserved verbatim.

## IDL extractor family (#129 — measured 45.1% true-miss, syn counts include_proto! as 1 opaque unit hiding ~58 RPC items)
Each `impl AstSource`, dispatch by file-ext. **ProtoAstSource (v1 NOW)** → RpcMethod/MessageType (closes the gRPC/contract-surface blind spot). Then OpenApiAstSource (RouteSpec), CedarAstSource (CedarPolicy, reuse cedar-policy crate), SqlAstSource (SqlTable/Column, sqlparser). HIR = W-tier deferred. New `ItemKind`s ride existing Function fact unchanged. Coverage invariant: every `include_proto!("X")` opaque unit must resolve to a parsed `.proto` source-unit → feeds completeness attestation (no false-green).

## Mergeable slices (face-touching = wait for keystone #141)
- **P0-1** add IDL ItemKinds to corpus-core (additive enum + RED tests) — INDEPENDENT
- **P0-2** ProtoAstSource + file-kind dispatch (closes 45% miss) — INDEPENDENT
- **P0-3** L2 node loader (serde + sorted-glob + FK validation) — INDEPENDENT
- **P0-4** L3 full-fold query (6 edge maps, should-be-empty relations) — INDEPENDENT
- **P0-5** L4 Cedar gate REPORT-ONLY face (`corpus-liveness.generated.json`, not-tracked) — INDEPENDENT
- **P0-6** per-capability `.facts/` dual-run + determinism canary (union(shards)==whole-tree) — INDEPENDENT
- **P0-7** repoint ~20 accounting/scm-facts readers → shards+global-index — **FACE-TOUCHING (after #141)**
- **P0-8** global accounting face → main-only; retire per-PR whole-tree regen — **FACE-TOUCHING (after #141)**
- **P0-9** promote structural invariants (ref-integrity/format/template) → born fail-closed + verified-equivalence retire subsumed gates — INDEPENDENT
- **P0-10** OpenAPI/Cedar/SQL sub-extractors (add per opacity report) — INDEPENDENT
NOTE: "keystone-independent" = doesn't need #141 for correctness, but each is still a PR touching committed scm-facts at born-accounting → cascades until #141 de-commits; so still fan out AFTER #141.

## Conservative-v1 boundary (W-tier, NOT now)
rowan/salsa ABSENT (verified); HIR deferred (entire measured miss was include_proto!, zero build-script); no graph-DB / query-lang / inventory-ctor (hermeticity).

## Risks
static-liveness-unsound-for-deletion (ratchet+quarantine) · IDL false-green (coverage attestation) · O(corpus) full-fold (fine at scale now; salsa W-tier) · keystone coupling (P0-7/8 gated on #141) · 2-SoT during gate migration (verified-equivalence retire ADR-0363) · doc-gating unprecedented (staged per-class).

Refs: governance/corpus/{core,extract}, ADR-0580 (#127 spike), ADR-0541 (liveness graph), accounting-registry-app, registry/generated-artifact-control-plane.json, libs/{oya-shared-pdp-adapter-cedar,oya-workspace-members-kernel,oya-ci-materializer-kernel}.
