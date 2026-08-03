# G030 non-code corpus baseline census — 2026-08-02

State: **PLANNING_ONLY — MECHANICAL BASELINE, NOT DISPOSITION COMPLETE, NOT ACTIVATED**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
No deletion, freeze, graph-wire edit, registry mutation, push, or activation occurred.

## Exact live baseline

Command class used for every count below:

```bash
git ls-tree -r --name-only origin/dev
```

| Metric | Count |
|---|---:|
| Total tracked files | **18,886** |
| `.md` | **5,342** |
| `.yaml` + `.yml` | **5,861** |
| `.json` | **1,755** |
| `.toml` | **1,001** |
| **G030 focus family (md+yaml/yml+json+toml)** | **13,959** |

The durable-goal prose “roughly 13,950” is therefore the live focus family rounded; the countable SSOT for before/after is **13,959**, not 13,950 and not total tracked files.

### Focus family by top-level prefix (largest)

| Prefix | `.md` | `.yaml/.yml` | `.json` | `.toml` | Focus total | Notes |
|---|---:|---:|---:|---:|---:|---|
| `oya/` | 1,612 | 4,103 | 626 | 180 | **6,521** | includes the 48 non-code product/capability shells |
| `docs/` | 2,670 | 47 | 419 | 1 | **3,137** | governance, audit, and transitional documentation |
| `cloud/` | 736 | 393 | 55 | 46 | **1,230** | product/service docs + contracts |
| `registry/` | 4 | 761 | 47 | 4 | **816** | machine SSOT density |
| `specs/` | 2 | 1 | 357 | 0 | **360** | machine plan/policy density |
| `evidence/` | 61 | 0 | 128 | 0 | **189** | historical/audit density |
| root authority | 4 | 0 | 0 | 0 | **4** | `README.md`, `CLAUDE.md`, `AGENTS.md`, `HANDOFF.md` |

Rust/BUCK density for context only: `.rs` **2,474**, `BUCK` **912**. Those are outside the G030 focus family.

## Existing controllers (reuse; do not invent a second corpus registry)

G030 is not greenfield. Live owned machinery already covers total accounting and several related faces:

| Controller | Role for G030 |
|---|---|
| `ci/facade/artifact-inventory-registry` | Producer: expands tracked paths + unit/TTL policy into accounting faces (CI-materialized, not committed on trunk). |
| `ci/facade/artifact-accountability` | Pure total-accounting evaluator: `unaccounted`, `unowned`, `unjustified`, `unreachable`, `no_ttl_class`, hand-edit drift, `scratch_artifact`. |
| `ci/facade/corpus-index-coverage` | Corpus-index coverage gate over declared inventory. |
| `ci/facade/stale-artifact-detection` | Stale/dark artifact detection face. |
| `ci/facade/generated-artifact-freshness` / `generated-artifact-policy` | Generated-face lifecycle (already de-committed on trunk; do not re-commit `*.generated.json`). |
| `governance/check/active-artifact-contract` | Active-artifact contract kernel; selected by affected-set policy for protected routes. |
| `specs/markdown-retirement-policy.json` | Accepted markdown retirement policy; execution owned by masterplan MPV2-0003. Root survival set is `README.md`, `CLAUDE.md`, `AGENTS.md`, plus founder-thin `HANDOFF.md`. |
| `specs/masterplan.json#masterplan_v2` | Sole live plan authority for countable reduction sequencing. |

`accounting-registry.generated.json` and sibling faces are **not** present on `origin/dev` by design (ADR-0613 de-commit). Counts of accounted vs unaccounted rows must be taken from a CI/local producer materialization, never by hand-authoring a generated face.

## Disposition classes (binding vocabulary)

Every focus-family path eventually lands in exactly one class. Classes are **not** auto-assigned by extension alone.

| Class | Meaning | Deletion allowed? |
|---|---|---|
| `ROOT_AUTHORITY` | Root survival set under markdown-retirement policy | No |
| `MACHINE_SSOT` | Specs/registry/policy JSON/YAML that gates or producers read | No (edit/migrate with consumer proof) |
| `GRAPH_WIRED_INPUT` | Declared seed/input of a required Buck2/gate package | No until consumer rewrite |
| `GENERATED_FACE` | Producer output; CI-materialized or explicit lifecycle | Never hand-commit; lifecycle already owned |
| `IMMUTABLE_AUDIT` | Evidence/history retained as provenance | No content rewrite; freeze/archive only |
| `TRANSITIONAL_TOOL_REQUIRED` | Tool/host still requires the file shape | No until replacement lands |
| `DARK_BUREAUCRACY` | No code/build consumer, no authority pointer, not audit-retained | Candidate freeze/delete only after dual proof |
| `UNCLASSIFIED` | Default until dual consumer/authority proof completes | No mutation |

### Dual-proof rule (anti false-ABSENT / false-UNUSED)

A path may enter `DARK_BUREAUCRACY` only when **both** hold:

1. **Consumer proof:** no required gate, producer, Buck2 package, workflow seed, or registry authority row names it as live input. Absence of a single `rg` hit is insufficient; use producer tracked-path membership + affected-set seed declarations + gate policy literals + package `srcs`/`$(location)` edges.
2. **Authority proof:** not retained by markdown-retirement policy, masterplan ladder, ADR/evidence retention, or an explicit founder exception.

This is the same class of failure that produced the false `cloud/cloud-kernel` ABSENT reading: one probe shape is not existence or unusedness.

### Anti-vacuity assertions (must ship with any reduction PR)

1. Before count and after count use the same command family against the same immutable tip class (`git ls-tree -r --name-only <tip>` + extension filter).
2. Reduction delta = deleted or frozen-out-of-tip paths, not renamed extensions and not moved under an ignored directory.
3. Every deleted path has a `DARK_BUREAUCRACY` row with dual proof recorded in the PR body or owned evidence face.
4. Required gates that previously consumed any deleted path remain green on a RED/GREEN fixture pair, or the consumer rewrite is in the same PR.
5. Root survival set cardinality remains exactly four markdown files at repo root (`README`, `CLAUDE`, `AGENTS`, thin `HANDOFF`).
6. No new committed `*.generated.json` appears.
7. Focus-family count cannot be “improved” by adding binary assets or by excluding previously counted extensions without an explicit schema version bump of this census.

## Smallest safe next slices (ordered)

| Slice | Action | Activation gate |
|---|---|---|
| G030-A | This baseline + controller map (done here) | planning only |
| G030-B | Materialize producer faces in an isolated worktree / CI job and emit class histograms for the 13,959 paths using existing unit/TTL policy — **read-only report** | no trunk edit |
| G030-C | Classify the four root authority files + `specs/` + `registry/` as `ROOT_AUTHORITY` / `MACHINE_SSOT` with consumer citations | plan-only or tiny docs/policy PR after review |
| G030-D | First freeze/delete batch: only paths that already fail total-accounting or stale-artifact as dark **and** pass dual proof; countable delta ≥ 1 with anti-vacuity tests | independent APPROVE + protected CI; after G028/#1526 health preferred so FULL baseline is trusted |
| G030-E | `oya/` non-code shells (6,521 focus files) join capability/app migration lanes (G026) rather than a bulk delete | per owning capability/app face birth |

Do **not** open with `docs/` mass deletion. `docs/` is the single largest markdown prefix and mixes audit history, transitional projections, and still-referenced hubs.

## Interaction with adjacent goals

- **G026:** the 48 non-code `oya/*` shells contribute heavily to the 6,521 `oya/` focus files; they are not automatic DELETE under G030.
- **G036/G037:** quality-lane and check-kernel darkness is about execution reachability, not file deletion. Do not retire lane rows by deleting their docs alone.
- **G028/#1526:** trusted FULL baseline is required before any large accounting-facing deletion train; G030-B read-only materialization can still proceed locally.
- **G023:** `cloud/cloud-kernel` deletion is a code/tree deletion with same-commit projection cleanup; it is not a G030 markdown reduction, though it will move the focus-family count when docs under that tree go away.

## Independent review

Agent explore/review lanes for this slice failed transport (`encrypted_content` decrypt). Codex and ouroboros review channels remain fused/quota-blocked. **No APPROVE inferred.** This document is coordinator-computed planning evidence only.

## Non-claims

- Not a deletion PR.
- Not a claim that 13,959 files are unused.
- Not a second corpus registry.
- Not permission to hand-edit generated faces.
- Not a completion of MPV2-0003.
