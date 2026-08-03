# G030-E `specs/` + `registry/` semantic-consumer census — 2026-08-02

State: **PLANNING_ONLY — READ-ONLY CITATION / DIRECTORY-CONTRACT CENSUS; NO DECLASSIFICATION**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G030-C-ROOT-SSOT-CITATION-AUDIT-2026-08-02.md`.  
No policy edit, affected-set edit, deletion, freeze, push, generated-face write, or activation occurred.

## Question

Of the **1,176** G030 focus-family paths under `specs/` + `registry/` (md/yaml/yml/json/toml), how many have a measured direct authority/affected-set citation or a required execution/directory consumer?

This census deliberately keeps the remainder **POLICY_PROTECTED_MACHINE_ARTIFACT**. “No citation found by these probes” is not “unused,” and protected rows cannot become `DARK_BUREAUCRACY` without separate dual-negative consumer + authority proof.

## Universe and policy protection

| Prefix | Focus rows | unit class | TTL action | protected |
|---|---:|---|---|---|
| `specs/` | **360** | spec 223 + registry-fixtures 137 | report | true |
| `registry/` | **816** | registry | report | true |
| **total** | **1,176** | | | **all protected** |

Universe command class: `git ls-tree -r --name-only b651080…` + focus extension filter. Same immutable tip as G030-A–D.

## Probe layers (ordered evidence, not one grep)

1. **Affected-set exact synthetic dependency** — exact path strings in `ci/facade/affected-target-set/affected-set-policy.json`.
2. **Root-hub exact live citation** — exact live `specs/` / `registry/` path values referenced by `specs/root-hub-pointers.json` (fragment stripped; stale/deleted prose strings excluded because they do not equal a live focus path).
3. **Execution exact literal** — exact live focus path literal in `ci/`, `governance/`, `build/`, or `.github/` Rust/policy/build/workflow files.
4. **Declared directory contract** — a producer expands a directory glob over the tracked-path universe and a required pure gate consumes the emitted rows. This prevents false “uncited” classification of filename-keyed registries whose individual paths are intentionally not hard-coded.
5. **Policy-only remainder** — still protected; semantic consumer not proven by layers 1–4.

Historical prose outside the execution/authority surfaces is not treated as a consumer. Structural artifact-inventory accounting sees every tracked path, but remains separate from domain-semantic consumption.

## Conservative partition (exactly 1,176)

Precedence: `MACHINE_SSOT` → `GRAPH_WIRED_INPUT` → `POLICY_PROTECTED_MACHINE_ARTIFACT`.

| Disposition | Count | `specs/` | `registry/` | Minimum evidence |
|---|---:|---:|---:|---|
| `MACHINE_SSOT` | **152** | 142 | 10 | exact affected-set seed **or** exact live root-hub authority citation |
| `GRAPH_WIRED_INPUT` | **782** | 24 | 758 | exact execution literal **or** declared directory-contract membership, excluding MACHINE_SSOT overlap |
| `POLICY_PROTECTED_MACHINE_ARTIFACT` | **242** | 194 | 48 | protected policy class; no layer 1–4 proof yet |
| **total** | **1,176** | **360** | **816** | anti-vacuity sum holds |

Raw measured sets before precedence:

| Evidence set | Count | Notes |
|---|---:|---|
| affected-set exact paths | **10** | specs 7 + registry 3 (G030-C) |
| root-hub exact live paths | **150** | union with affected-set = 152 (8 overlap) |
| execution exact literals | **79** | many overlap authority citations / catalog family |
| `registry/catalog/*.{yaml,yml}` directory contract | **748** | producer glob expansion over tracked paths; required catalog/SLO/liveness gates consume the rows |

## Strong directory-contract proof: `registry/catalog/` (748 rows)

This is the dominant correction to a naive exact-literal census.

`ci/facade/artifact-inventory-registry/src/main.rs`:

- `collect_slo_coverage` expands config-declared `slo_coverage.catalog_record_globs` over `tracked_paths` (default `registry/catalog/*.yaml`);
- `collect_catalog_liveness` expands `catalog_liveness.catalog_record_globs` over the same tracked universe;
- catalog row file stem is the crate identity, and `traceability.source_crate` is parsed from each row.

Required consumers include:

- `ci/facade/crate-catalog-coverage` — every live first-party crate must carry `registry/catalog/<package-name>.yaml`;
- catalog liveness / service-catalog parity;
- SLO coverage;
- crate registration and generated producer faces.

Therefore each matching catalog row is `GRAPH_WIRED_INPUT` even when no Rust file hard-codes its full path. Filename-keyed registries are *supposed* to avoid 748 literal constants.

Anti-vacuity: tip contains **748** focus rows under `registry/catalog/`; all match the declared YAML family and are included in the graph-wired set (some also rank MACHINE_SSOT due to root-hub citations).

## Exact affected-set seeds (10)

### `specs/` (7)

1. `specs/artifact-profile-defaults.json`
2. `specs/decision-rights.json`
3. `specs/forbidden-operations.json`
4. `specs/product-protocol-contract.json`
5. `specs/api-contract-ssot-canonical.json`
6. `specs/root-hub-pointers.json`
7. `specs/markdown-retirement-policy.json`

### `registry/` (3)

1. `registry/artifact-capabilities-registry.json`
2. `registry/dependency-rationales.json`
3. `registry/quality/lanes/lean-settings-drift.json`

These are a routing subset, not the full semantic universe.

## Examples of exact execution consumers

| Path | Consumer class |
|---|---|
| `specs/capability-registry.json` | artifact-inventory producer + crate-registration/membership machinery |
| `specs/active-machine-readable-artifact-contract.json` | `governance/check/active-artifact-contract` |
| `specs/cache-warm-license.json` / `specs/cache-warmth-policy.json` | build-cache policy gate |
| `registry/bounded-contexts.json` / `registry/cedar-fragments.json` | Cedar fragment coverage gate |
| `registry/history-only-retirement/control-plane.json` | cross-artifact retirement receipt evaluator |
| `registry/vendor-lockin-phaseout/index.json` | vendor-lockin discipline gate |
| `registry/vocabulary/retired.yaml` | retired-vocabulary gate |

Exact-literal presence proves a reader edge; it does not by itself prove the artifact is globally authoritative. The precedence table labels root-hub/affected-set authority rows MACHINE_SSOT and other required reader edges GRAPH_WIRED_INPUT.

## Policy-protected remainder (242) — investigation queue, not deletion queue

Largest families:

| Family | Rows | Current interpretation |
|---|---:|---|
| `specs/fixtures/` | **131** | test/negative-example reservoir; many exact fixtures are consumed, but no universal whole-directory semantic contract was proven by this census. Keep protected; expand per-gate fixture globs before reclassifying. |
| `specs/design-system/` | **17** | protected design contracts; no execution citation found in scanned surfaces |
| `registry/check-empirical-evidence/` | **14** | protected registry evidence; no generic consumer proven |
| `specs/reorg/` | **10** | some plans/workflows are live, but the remaining ten need move-plan loader / lifecycle-status proof individually; never bulk-delete |
| `specs/lifecycle-configs/` | **8** | protected lifecycle policy; consumer proof incomplete |
| `specs/microservices/` | **7** | protected schemas/contracts; consumer proof incomplete |
| `registry/{accounts,vcs}` | **5 + 5** | protected registry families; consumer proof incomplete |
| `registry/foundation-bypasses/` | **4** | likely governance-sensitive; absence of a scanned literal is especially not declassification |
| other small families | **41** | retain; investigate by owning lane |
| **total** | **242** | |

The 131 fixture rows are the largest next mechanical slice, but they require **gate-specific directory/glob expansion**, not a deletion review. A fixture can be consumed via `include_str!`, Buck `srcs`, test data-dir traversal, or policy slices without its full repo-relative path appearing in Rust.

## What this changes from G030-C

G030-C could prove only 10 exact affected-set seeds and class-level protection. This census adds:

- 150 exact live root-hub citations;
- 79 exact execution path consumers;
- the 748-row catalog directory contract;
- a countable protected-only remainder of 242 rather than calling all 1,176 semantically unresolved.

It does **not** weaken protection for the 242 remainder.

## Next smallest read-only slices

1. **Fixture contract expansion (131):** enumerate each required gate’s Buck `srcs`, Rust `include_str!` / path joins, policy slice arrays, and directory readers. Classify consumed fixtures as GRAPH_WIRED_INPUT; leave the rest protected-only.
2. **Reorg plans (10 protected-only):** compare against the owned reorg codemod plan loader, lifecycle-status policy, and active masterplan lanes. Never infer “inactive” from filename alone.
3. **Design/evidence families (31):** owner/authority citation census; no policy weakening.
4. Only after a path is both consumer-negative **and** authority-declassified may it enter G030 dual-proof. No current path in this 1,176 set is a delete candidate.

## Verification / anti-vacuity

- `152 + 782 + 242 = 1,176`.
- Prefix totals remain `360 specs + 816 registry = 1,176`.
- Catalog contract count **748** is from immutable-tree membership, not filename examples.
- No generated face was materialized or committed.
- No `*.generated.json` changed.
- No policy protection changed.
- No root authority count changed.

## Independent review

Not obtained. A fresh architect lane failed transport with `encrypted_content` decrypt 400 during adjacent G026 work; Codex usage and Ouroboros transport remain fused. **No APPROVE inferred.**

## Non-claims

- Not a claim that root-hub citation is the only form of authority.
- Not a claim that the 242 remainder lacks consumers.
- Not permission to add synthetic dependencies just to improve a histogram.
- Not permission to weaken `protected: true`.
- Not a deletion/freeze PR.
- Not a second registry.
