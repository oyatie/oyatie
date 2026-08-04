# G026 tools disposition draft — 2026-08-02

State: `PLANNING_ONLY_NOT_ACTIVATED`
Authority: origin/dev path census + registry absorb notes + existing destination faces.
This is **not** an executable codemod move plan. No PR. No code moved.

## Non-claims
- Independent transport analysis failed (`encrypted_content` decrypt). This draft is coordinator-direct planning only.
- No tools mega-capability invented.
- No second registry invented.
- No MOVE execution until: independent review + protected CI + promoted observation of predecessor corpus repair + explicit codemod schema plan.

## Destination grammar (existing only)
- Delivery fabric / CI absorb: `ci/facade/*` (capability `ci`, dag_node `delivery-fabric`)
- Governance checks: `governance/check/*` (already absorbs many oya-check-* / governance app surfaces)
- Build graph helpers: `build/*` or `ci/facade/*` only when a live consumer face exists
- Reorg codemod already lives under tools and is the **executor**, not a destination of itself

## Draft matrix (21/21)

| crate | path | disposition | draft destination | evidence | blockers |
|---|---|---|---|---|---|
| `fixup-ledger-merge-driver-app` | `tools/fixup-ledger-merge-driver-app` | MOVE | `ci/facade/fixup-ledger-merge-driver` (or scm-facts adjacent) | ADR-0626 + `.gitattributes` consumer; structural merge driver | confirm face grammar; catalog ArtifactMove |
| `oya-cargo-lock-merge-driver-app` | `tools/oya-cargo-lock-merge-driver-app` | MOVE | `ci/facade/cargo-lock-merge-driver` | sibling structural merge driver class | same as above |
| `oya-friction-ledger-merge-driver-app` | `tools/oya-friction-ledger-merge-driver-app` | MOVE | `ci/facade/friction-ledger-merge-driver` | sibling structural merge driver class | same as above |
| `oya-governance-adr-shape-app` | `tools/oya-governance-adr-shape-app` | MOVE | `governance/check/adr-shape` | governance check family already under governance/check | debrand leaf name; no ports invented |
| `oya-governance-authoritative-tracked-app` | `tools/oya-governance-authoritative-tracked-app` | MOVE | `governance/check/authoritative-tracked` | governance check family | same |
| `oya-governance-banned-primitives-app` | `tools/oya-governance-banned-primitives-app` | MOVE | `governance/check/banned-primitives` | governance check family | same |
| `oya-governance-portfolio-citation-app` | `tools/oya-governance-portfolio-citation-app` | MOVE | `governance/check/portfolio-citation` | governance check family | same |
| `oya-governance-predictable-naming-app` | `tools/oya-governance-predictable-naming-app` | MOVE | `governance/check/predictable-naming` | governance check family | same |
| `oya-governance-purpose-audit-app` | `tools/oya-governance-purpose-audit-app` | MOVE | `governance/check/purpose-audit` | governance check family | same |
| `oya-governance-sunset-lifecycle-app` | `tools/oya-governance-sunset-lifecycle-app` | MOVE | `governance/check/sunset-lifecycle` | governance check family | same |
| `oya-governance-adapter-with-no-importer-app` | `tools/oya-governance-adapter-with-no-importer-app` | MOVE | `governance/check/adapter-with-no-importer` | governance check family; fixture-ish name | confirm not a permanent dual authority test harness only |
| `oya-buck-test-wiring-app` | `tools/oya-buck-test-wiring-app` | MOVE | `ci/facade/buck-test-wiring` or `build/buck-test-wiring` | build-graph wiring helper | pick one destination class after importer census |
| `oya-xtask-metadata-augment-app` | `tools/oya-xtask-metadata-augment-app` | MOVE | `ci/facade/xtask-metadata-augment` | metadata augmentation for workspace accounting | confirm against owned Cargo.lock lifecycle surfaces |
| `oya-architecture-graph-generator-app` | `tools/oya-architecture-graph-generator-app` | KEEP_PENDING / REFACTOR | uncertain — may land under governance projection or ci facade | generator of architecture graphs; no existing exact face | destination undecided; mark uncertain |
| `oya-reorg-codemod-app` | `tools/oya-reorg-codemod-app` | KEEP_THEN_MOVE_LATE | `ci/facade/reorg-codemod` only after tools tail empties | this is the move executor; moving it mid-program creates self-host churn | **serial after** other tools moves; do not move first |
| `oya-checkout-guard-app` | `tools/oya-checkout-guard-app` | MOVE | `ci/facade/checkout-guard` | checkout guard / local bridge class | confirm not retired CLI surface only |
| `oya-lane-supervisor-app` | `tools/oya-lane-supervisor-app` | KEEP_PENDING | uncertain | agent/lane supervision; may be retirement-marked CLI | founder CLI retirement + fabric ownership |
| `oya-fabric-loop-state-app` | `tools/oya-fabric-loop-state-app` | KEEP_PENDING | uncertain — delivery fabric related | fabric loop state; may belong under ci/delivery-fabric faces | map to ADR-0516..0535 owner face before move |
| `oya-bot-autofix-app` | `tools/oya-bot-autofix-app` | KEEP_PENDING / DELETE_CANDIDATE | uncertain | bot autofix; may conflict with external-agent-tooling retirement ADR-0116 | require retirement scan before MOVE |
| `oya-adapter-substitution-test-app` | `tools/oya-adapter-substitution-test-app` | MOVE | colocated test fixture under the capability it tests, else `ci/facade/*-fixtures` | test app, not product surface | find sole importer; do not invent permanent tools home |
| `oya-tooling-agent-read` | `tools/oya-tooling-agent-read` | KEEP_PENDING / DELETE_CANDIDATE | uncertain | agent-read tooling; ADR-0116 retirement pressure | retirement scan first |

## Counters (superseded 2026-08-02 destination-existence audit)

Earlier draft counters claimed 14 MOVE with known destinations. **REJECTED.**

Authoritative correction in `G026-DESTINATION-EXISTENCE-AUDIT-2026-08-02.md`:

| Class | Count |
|---|---:|
| MOVE with **existing** leaf destination on origin/dev | **0** |
| CLASS_KNOWN_LEAF_MISSING (parent class only; leaf invented) | **16** |
| KEEP_PENDING / authority reconciliation | **4** |
| KEEP_THEN_MOVE_LATE (`oya-reorg-codemod-app`) | **1** |
| DELETE_CANDIDATE | **0** (retirement census withdrew both candidates) |

Importer census + retirement census remain valid as non-delete evidence. They do **not** authorize executable plans.

## Sequencing rules
1. Do **not** open a tools move-plan PR until leaf destinations **exist** (born face PR) **and** G019 corpus repair (#1526) is promoted and observed green.
2. Pair tools apps with libs kernel siblings first; land kernels under real `governance/check/<leaf>` faces (G025 / governance-check plan) before moving tools apps.
3. Codemod executor (`oya-reorg-codemod-app`) moves last.
4. Uncertain rows stay KEEP_PENDING — no guessed capability and no invented leaf.
5. Executable plan requires real destination paths on origin/dev + schema keys matching landed intelligence plans + independent APPROVE + protected CI.

## Next activation criteria
- [ ] Born leaf faces for every CLASS_KNOWN_LEAF_MISSING row (or atomic face-birth inside the move plan)
- [ ] Independent disposition review (transport currently failing — not APPROVE)
- [x] Retirement scan for bot-autofix + tooling-agent-read + lane-supervisor → KEEP_PENDING, DELETE_CANDIDATE=0
- [x] Exact importer census for adapter-substitution-test-app and architecture-graph-generator
- [ ] Codemod MovePlan JSON authored only after above
- [x] oya product/CI tail census (see `G026-OYA-PRODUCT-TAIL-CENSUS-2026-08-02.md`)
