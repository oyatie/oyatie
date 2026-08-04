# G026 tools app ↔ libs kernel pairing — 2026-08-02

State: **PLANNING_ONLY — face-birth prerequisite, not a move plan**
Authority: `origin/dev` path existence only.

## Why this exists

Destination-existence audit rejected all 17 draft tools MOVE leaves as invented. Eight governance tools apps have **exact libs kernel siblings**. The safe face-birth order is kernel-first (G025 / governance-check family), then tools app as face binary — never tools-only leaf invention.

## Paired rows (8/8 kernels exist)

| tools app | libs kernel | Draft face leaf (not yet born) |
|---|---|---|
| `tools/oya-governance-adapter-with-no-importer-app` | `libs/oya-governance-adapter-with-no-importer-kernel` | `governance/check/adapter-with-no-importer` |
| `tools/oya-governance-adr-shape-app` | `libs/oya-governance-adr-shape-kernel` | `governance/check/adr-shape` |
| `tools/oya-governance-authoritative-tracked-app` | `libs/oya-governance-authoritative-tracked-kernel` | `governance/check/authoritative-tracked` |
| `tools/oya-governance-banned-primitives-app` | `libs/oya-governance-banned-primitives-kernel` | `governance/check/banned-primitives` |
| `tools/oya-governance-portfolio-citation-app` | `libs/oya-governance-portfolio-citation-kernel` | `governance/check/portfolio-citation` |
| `tools/oya-governance-predictable-naming-app` | `libs/oya-governance-predictable-naming-kernel` | `governance/check/predictable-naming` |
| `tools/oya-governance-purpose-audit-app` | `libs/oya-governance-purpose-kernel` | `governance/check/purpose-audit` (or `purpose` — pick one leaf, don't dual-name) |
| `tools/oya-governance-sunset-lifecycle-app` | `libs/oya-governance-sunset-lifecycle-kernel` | `governance/check/sunset-lifecycle` |

## Required face-birth sequence (plan-only description)

1. G025 executable plan (separate PR): move each libs kernel → born `governance/check/<leaf>` with BUCK + catalog ArtifactMove + debrand `check-<leaf>`.
2. Observe protected green + no dual home under `libs/`.
3. G026 executable plan: move tools app onto the **same** leaf as facade/binary (or delete tools app if the check face binary already covers the CLI-retirement surface).
4. Never author step 3 JSON before step 1 destinations exist on origin/dev.

## Unpaired tools rows (still CLASS_KNOWN_LEAF_MISSING / KEEP_PENDING)

Merge drivers, checkout-guard, architecture-graph-generator, buck-test-wiring, xtask-metadata-augment, adapter-substitution-test, fabric-loop-state, bot-autofix, lane-supervisor, tooling-agent-read, reorg-codemod — no automatic kernel pair from this scan. They stay blocked on destination proof or KEEP_PENDING authority work.

## Non-claims

- Pairing is not authorization to move.
- Leaf names in the table are **candidates**; collision check against live 56 `governance/check` names required at plan-author time (none of these eight leaf strings exist today).
