# ci.toml — config reference

`ci.toml` is the single, human-authored policy file at the repo root. It is parsed by the
`ci-config-kernel` crate into typed structs and validated by a **CLOSED schema**: an unknown
key (top-level or nested) is a hard error, and a malformed file fails the producer LOUDLY rather
than silently reverting to defaults. Every section is optional — an absent section falls back to
the compiled-in **bundled default** (the values below). The bundled default reproduces oyatie's
historical hardcoded policy byte-for-byte.

This reference is the source-of-truth schema. The producer reads ONLY this config for policy; no
policy is hardcoded in the producer.

## `[repo]`

| Key | Default | Meaning |
|---|---|---|
| `root_markers` | `["specs/root-hub-pointers.json"]` | files the producer walks up-tree to find the repo root |
| `path_excludes` | `["third-party/"]` | path prefixes excluded from the `collect_*` scans (matches a top-level prefix OR a nested `/<prefix>`) |

## `[naming]` — the predictable-naming policy (consumed by the `cloud-ci-bnf-layer-suffix` gate)

| Key | Default |
|---|---|
| `required_prefix` | `"oya-"` |
| `allowed_roles` | the 12 canonical layer values: `kernel domain usecase app adapter infrastructure cli rest grpc worker sdk api` |
| `check_family_prefix` | `"check-"` (self-layering check family — no declared role required) |
| `backend_suffixes` | `fake inmemory aws oci gcp azure postgres redis sqlite` (the `*-adapter-<backend>` qualifier set) |
| `doctrinal_carve_outs` | `["tooling-agent-read"]` (names locked by a higher contract, exempt from the suffix rule) |

## `[vocab]` — the forbidden-vocab shrink-only-ratchet policy (consumed by `cloud-ci-brand-residue`)

`[[vocab.forbidden_stems]]` is an array of `{ stem, code }` rows: each `stem` is matched
case-insensitively as a substring, and a file containing it is frozen under `code` (a per-stem
firewall code; hyphenated codes are supported and round-trip unchanged). Adding a stem WIDENS the
boundary — its current occurrences baseline on the next regen, and every later occurrence beyond the
baseline is RED. oyatie's checked-in `ci.toml` declares the active set; see it for the exact
stems and codes.

`[[vocab.carve_outs]]` is an array of `{ kind, value, exempt_stems, reason }` rows. `kind` is one
of `path_prefix` / `path_exact` / `path_suffix` (drops the whole file) or `line_contains_ci`
(lower-cased contains). Path rules omit `exempt_stems`. Every `line_contains_ci` rule must list the
exact forbidden stem(s) it may suppress in `exempt_stems`; matching the marker never exempts an
unlisted stem on the same line. Carve-outs cover deny-list definition files, generated faces,
intentional historical archives, append-only audit chains, and narrowly named structural or
proper-noun occurrences.

This conditional requirement is schema v2. Candidate configs missing `exempt_stems` fail closed.
Frozen-reference regeneration retains a bounded v1 compatibility path: a historical line rule
without the field expands in memory to every forbidden stem, exactly reproducing v1's whole-line
exception semantics without weakening candidate validation.

## `[manifest]` — the rust-cargo manifest-hygiene field-set (declared for `cloud-ci-manifest-hygiene`)

| Key | Default |
|---|---|
| `required_flags` | `version_workspace rust_version_workspace publish_false license lints_workspace lib_doctest_false` |

(In the floor, the manifest gate evaluates the fixed field-set; this section declares it as the
portable, documented policy. `lib_doctest_false` is required only when a crate has a `[lib]` table.)

## `[reachability]` / `[justification]` / `[owners]` / `[enforcement]` — source paths

| Section.key | Default |
|---|---|
| `reachability.masterplan` | `"specs/masterplan.json"` |
| `reachability.root_hub` | `"specs/root-hub-pointers.json"` |
| `reachability.doc_catalog` | `"docs/DOC-CATALOG.md"` |
| `justification.adr_dir` | `"docs/decisions"` |
| `justification.roadmap` | `"specs/master-plan-sequencing.json"` |
| `owners.file_name` | `"OWNERS"` (nearest-up-tree marker) |
| `enforcement.governance_crate_substr` | `"governance"` |
| `enforcement.governance_lanes` | `["docs/governance-lanes/diataxis-doc-class.md", "docs/governance-lanes/prd-axis-coverage.md"]` |

## `[ttl]` / `[unit_class]` — the carry-over DATA tables

These two tables (the carve-out classification + the TTL budgets) are carried as the existing JSON
DATA. Leave the sections empty (`[ttl]` / `[unit_class]` with no keys) to use the bundled tables;
set `inline_json = """{ ... }"""` to override with a full inline JSON document.

## `[gates]` — the enabled gate set + dispositions

`[[gates.enabled]]` is an array of gate specs, one per enabled gate:

| Key | Meaning |
|---|---|
| `id` | the gate id (matches its crate + its firewall baseline section) |
| `input_kind` | how the gate's CURRENT keys are sourced — `producer-face`, `raw-corpus-collector`, or `frozen-empty-meta` (see [the gate catalog](./gate-catalog.md) §input KINDs) |
| `face` | for `producer-face` gates only: which producer face it binds (`total_accounting` / `cross_artifact` / `automation_ratchet` / `staleness` / `bnf_layer_suffix` / `manifest_hygiene` / `cargo_prefix` / `slo_coverage` / `workspace_glob_coverage`) |

`gates.disposition_json` (optional, `inline_json`-style) carries the per-(gate,code)
`mode` / `infra_prereq` / `frozen_empty` disposition table; absent ⇒ the bundled table. A
disposition row flip (advisory-until-infra → baseline-block-on-new, or freezing a code) is a DATA
edit here, never a code change.
