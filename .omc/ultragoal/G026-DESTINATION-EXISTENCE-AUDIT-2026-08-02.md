# G026 destination-existence audit — 2026-08-02

State: **PLANNING_ONLY — REJECT for executable move-plan authorship**
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48` via `git cat-file -e`.
Independent transport for the disposition challenge lane: **FAILED** (encrypted_content). This audit is coordinator-mechanical and is itself a hard blocker, not an APPROVE substitute.

## Tools cardinality (confirmed)

| Class | Count | Evidence |
|---|---:|---|
| `tools/` immediate dirs | **23** | includes non-crate `hooks/`, `opensk-vendored/` |
| `tools/**/Cargo.toml` crate roots | **21** | matches prior draft matrix |

## Destination face existence (draft MOVE targets)

Every proposed leaf destination from `G026-TOOLS-DISPOSITION-DRAFT-2026-08-02.md` was probed on `origin/dev`.

| Draft destination | Exists on origin/dev? |
|---|---|
| `ci/facade/fixup-ledger-merge-driver` | **MISSING** |
| `ci/facade/cargo-lock-merge-driver` | **MISSING** |
| `ci/facade/friction-ledger-merge-driver` | **MISSING** |
| `ci/facade/buck-test-wiring` | **MISSING** |
| `ci/facade/xtask-metadata-augment` | **MISSING** |
| `ci/facade/architecture-graph-generator` | **MISSING** |
| `ci/facade/reorg-codemod` | **MISSING** |
| `ci/facade/checkout-guard` | **MISSING** |
| `ci/facade/adapter-substitution-test` | **MISSING** |
| `governance/check/adr-shape` | **MISSING** |
| `governance/check/authoritative-tracked` | **MISSING** |
| `governance/check/banned-primitives` | **MISSING** |
| `governance/check/portfolio-citation` | **MISSING** |
| `governance/check/predictable-naming` | **MISSING** |
| `governance/check/purpose-audit` | **MISSING** |
| `governance/check/sunset-lifecycle` | **MISSING** |
| `governance/check/adapter-with-no-importer` | **MISSING** |

**17/17 invented leaf destinations.** Parent classes exist (`ci/facade` = 55 packages; `governance/check` = 56 packages), but none of the draft leaf faces do.

Nearby live packages that are **not** automatic destinations (name only, no equivalence claim):

- `ci/facade/slo-coverage`, `ci/facade/service-tier-metadata` (no merge-driver / checkout-guard / architecture-graph-generator face)
- `governance/check` already has many kernels (`adr-citation`, `codeowners-mirror`, …) but **none** of the seven draft governance leaf names

## Ruling

1. **No executable `specs/reorg/tools-*-move-plan.json` may be authored** while destinations are invented leaf paths. A move plan that names a non-existent face is dark wiring / second home invention.
2. Reclassify draft rows:
   - destination class known at **parent** level only (`ci/facade/*` family, `governance/check/*` family) → keep as **CLASS_KNOWN_LEAF_MISSING**
   - leaf path must be created by a **born face PR** (package + BUCK + catalog row) **before** or **atomically inside** the move plan, never assumed
3. Preferred sequencing (still plan-only):
   1. Pair each tools app with its **libs kernel** sibling when one exists (e.g. `tools/oya-governance-adr-shape-app` ↔ `libs/oya-governance-adr-shape-kernel`) and land the kernel under the real `governance/check/<leaf>` face first (G025 / governance-check move plan lane).
   2. Only then move the tools app as the face binary/facade onto that born leaf.
   3. Codemod executor stays last (`KEEP_THEN_MOVE_LATE`).
4. Independent disposition review still required after destinations are real; transport failure ≠ APPROVE.

## Corrected counters (tools)

| Class | Count |
|---|---:|
| MOVE with **existing** leaf destination | **0** |
| CLASS_KNOWN_LEAF_MISSING (was “MOVE”) | **16** |
| KEEP_PENDING / authority reconciliation | **4** |
| KEEP_THEN_MOVE_LATE (`oya-reorg-codemod-app`) | **1** |
| DELETE_CANDIDATE | **0** |

## Non-claims

- No code moved. No PR. No registry edit.
- Parent-class existence is not leaf-face existence.
- This audit does not authorize inventing a `tools` mega-capability.
