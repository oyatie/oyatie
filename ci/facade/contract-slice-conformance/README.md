# contract-slice-conformance gate

Paved-road owned-Rust/Buck2 cloud-ci gate that **replaces the fleet-wide
`scripts/tests/*_check.py` "contract slice" validators**. It is the class-fix for
the recurring anti-pattern where a spec slice's only enforcement is a new Python
script that `oya-ci-required` never runs (and that only mirrors the constants it
hardcodes).

The gate is a pure evaluator (`evaluate_configured(policy, corpus)`): it never
shells out, spawns an interpreter, mutates files, or reads ambient state. All
per-slice rules are **data**, sharded one slice per file under `slices/`.

## `contract-slice-policy.json` is GENERATED — do not hand-edit it

Every slice used to be one entry in the single, hand-edited
`contract-slice-policy.json`, which meant every new-slice PR touched that one
shared file — landing any one of them re-dirtied every other open slice PR.
`slices/<slice_id>.json` (one committed file per slice, human-editable,
self-contained) is now the source of truth; `contract-slice-policy.json` is
the GENERATED aggregate the gate reads, merged in deterministic
(sorted-by-`slice_id`) order. Its committed path is unchanged because
`specs/root-hub-pointers.json`, a `governed_surfaces` entry in
`compliance-pack-schema.json`, and several ADRs cite it by exact path — only
who's allowed to edit it changed.

**Adding a slice = adding ONE new file; no shared-file edit.**

## Add a contract slice (no new Python, shell, CLI, or crate)

1. Commit your slice's spec JSON (e.g. `specs/<your-slice>.json`).
2. Add `slices/<your-slice>.json`, containing exactly the slice object (the
   file's own `slice_id` must match its filename's stem):

   ```json
   {
     "slice_id": "<your-slice>",
     "spec_path": "specs/<your-slice>.json",
     "required_fields": ["field_a", "nested.field_b"],
     "enum_constraints": [{ "field": "status", "allowed": ["Proposed-target"] }],
     "required_array_members": [{ "field": "source_adrs", "members": ["ADR-0341"] }],
     "forbidden_markers": ["production ready", "runtime auto-rebalance is implemented"],
     "source_migration_slice": [
       {
         "legacy_path": "scripts/tests/<your-slice>_check.py",
         "replacement_target": "//ci/facade/contract-slice-conformance:ci-contract-slice-conformance-gate",
         "disposition": "retired_primary_path"
       }
     ]
   }
   ```

   - `spec_path` is **repo-root-relative** (e.g. `specs/<your-slice>.json`).
   - `required_array_members` asserts a dotted string-array field is a superset of
     the declared members (source ADRs, nonclaim families, filters, …).
   - `forbidden_markers` are extra per-slice substrings that must not appear in the
     spec (e.g. runtime-overclaim phrases), on top of the universal CLI/interpreter set.
   - `source_migration_slice` is optional; include it when you are **retiring a
     Python validator** so the gate proves it is replaced, not run in parallel.
     Then `git rm` the `.py` and drop its `rust-first-automation-policy.json`
     exception in the same PR.

3. Regenerate the aggregate:
   `buck2 run //ci/facade/contract-slice-conformance:oya-cloud-ci-materialize-contract-slice-policy-bin`
   (or `cargo run --bin oya-cloud-ci-materialize-contract-slice-policy` locally),
   and commit the resulting `contract-slice-policy.json` diff alongside your new
   fragment. The materializer refuses to write (nonzero exit, no partial file) on
   any fragment-loader finding below.
4. `buck2 test //ci/facade/contract-slice-conformance:ci-contract-slice-conformance-gate`
   and `buck2 test //ci/facade/contract-slice-conformance:ci-contract-slice-conformance-fragments-gate`
   (the latter proves the committed aggregate is still byte-identical to what the
   fragments produce).

### Fragment-loader findings (fail closed, before any slice rule ever runs)

| Code | Meaning |
|------|---------|
| `contract_slice_fragment_parse_error` | a `slices/*.json` file is not valid JSON, or has no string `slice_id` field |
| `contract_slice_fragment_duplicate_slice_id` | two fragments (any filenames) declare the same `slice_id`; both are excluded from the aggregate — there is no principled way to pick a winner |

A per-slice key typo (e.g. `required_field` instead of `required_fields`) inside
a fragment is **not** a load-time finding — it is caught the same way it always
was, by the existing `contract_slice_unknown_policy_key` check once the merged
policy reaches `evaluate_configured`.

## What the gate enforces per slice

| Code | Meaning |
|------|---------|
| `contract_slice_primary_path_not_rust` | policy `primary_execution_path` is not `rust_buck2_cloud_ci_gate` |
| `contract_slice_policy_has_no_slices` | policy declares zero slices |
| `contract_slice_spec_absent` | declared `spec_path` is not in the corpus |
| `contract_slice_missing_required_field` | a `required_fields` dotted path is absent/null |
| `contract_slice_enum_violation` | an `enum_constraints` field value is not allowed |
| `contract_slice_missing_array_member` | a `required_array_members` array field is missing a declared member |
| `contract_slice_forbidden_marker` | the spec contains a retired-CLI / interpreter marker (`python3`, `oya gate`, `kubectl apply`, …) or a per-slice forbidden phrase |
| `contract_slice_migration_not_retired` | a migration row is not `disposition: retired_primary_path` |
| `contract_slice_migration_bad_target` | migration `replacement_target` is not a `//ci/facade/…-gate` target |
| `contract_slice_migration_bad_legacy` | migration `legacy_path` is not a `.py`/`.sh` interpreter script |

### Full-fidelity primitives (contract-slice DSL enrichment)

Enum/array-member checks are **type-preserving string equality by default** (a spec number
`90` does NOT satisfy `allowed: ["90"]`); add `match_scalar: true` to a constraint/requirement
to canonicalize a numeric/bool leaf authored as a string literal. Every primitive below fails
**closed** on its own malformed shape — a mistyped string-list config (a non-string element in
`values`/`allowed`/`members`/`markers`/…), a wrong-typed cardinality bound, or an empty
pattern each emits a `*_malformed` finding (e.g. `contract_slice_malformed_policy_value`) rather
than silently dropping the element. Forbidden-marker matching is **fail-safe**: both marker and
scanned text are canonicalized to an `[a-z0-9]`-only sequence (lowercase; every space,
punctuation, and zero-width/format char dropped) and compared as a substring, so
`production ready` catches `production-ready`, `production<U+200B>ready`, and every separator or
zero-width obfuscation with one rule. This is intentionally OVER-STRICT for a prohibition check
(a legitimate `preproductionreadying` token would also trip `productionready`) — a false RED is
safe; a false GREEN would hide a prohibited claim. Bidi-**reorder** controls
(U+202A–202E, U+2066–2069) make rendered order differ from logical order, so their PRESENCE in any
scanned leaf is itself a violation (`contract_slice_bidi_control_in_content`) rather than
stripped-then-matched (which would leave the reversed text). Plain directional marks (U+200E/200F)
and general non-ASCII (i18n) are not rejected:

| Slice key | Code(s) | Meaning |
|-----------|---------|---------|
| `required_true_fields` / `required_false_fields` | `contract_slice_field_not_true` / `_field_not_false` | a dotted field must be boolean `true` / `false` |
| `exact_array_fields` `[{field,values}]` | `contract_slice_array_not_exact` | a string array must equal `values` exactly (order + no extras; a non-string element is a mismatch) |
| `required_object_array_members[].exact_members` (bool) | `contract_slice_unexpected_object_array_member` | no member key beyond the declared set |
| `required_object_array_members[].conditional_assertions` | `contract_slice_conditional_field_not_equal` / `_not_true` / `_missing_contains` / `_not_subset` and `_conditional_assertion_bad_selector` / `_multiple_modes` / `_no_mode` / `_bad_mode` | per-member pins with EXACTLY ONE selector (`when_member`/`when_member_in`) + EXACTLY ONE mode (`must_equal`/`must_be_true`/`must_contain`/`must_subset_of`) |
| `required_object_array_members[].field_implies_required` | `contract_slice_conditional_required_field_absent` / `_field_implies_required_malformed` | when a member flag is `true`, companion fields become required (arrays must be non-empty) |
| `required_markers` `[{field,markers,quantifier?,scope?}]` | `contract_slice_required_marker_missing` / `_required_marker_none_present` / `_required_markers_malformed` | field-scoped (or `scope:whole_spec`) content markers; `quantifier:any_of` REDs only when none present |
| `forbidden_markers` (fail-safe canonical match) | `contract_slice_forbidden_marker` / `contract_slice_bidi_control_in_content` | `[a-z0-9]`-only collapse: `production-ready` trips `production ready`; a bidi-reorder control in content is itself RED |
| `forbidden_field_markers` `[{field,markers}]` | `contract_slice_forbidden_field_marker` / `_forbidden_field_markers_malformed` | a phrase forbidden only inside a dotted sub-tree |
| `marker_exclude_fields` `[field,…]` | (carve-out) | named sub-trees excluded from the whole-spec forbidden scan (a `claim_boundary` may quote what it forbids) |
| `field_patterns` `[{field,pattern}]` | `contract_slice_pattern_mismatch` / `_bad_pattern` | a dotted scalar must match a regex (hex/id/base64url shapes); malformed regex fails closed |
| `exact_projected_sequence` `[{field,member_field,values}]` | `contract_slice_projected_sequence_mismatch` | ordered + length-exact projection of a member field across an array-of-objects |
| `array_cardinality` `[{field,min?,max?,unique_by?}]` | `contract_slice_array_below_min` / `_above_max` / `_not_unique` / `_cardinality_bad_field` / `_cardinality_malformed` | size bounds + uniqueness of a projected key |
| `projected_value_sets` `[{field,member_field,exact_values}]` | `contract_slice_projected_set_missing` / `_unexpected` / `_bad_field` | the SET of projected values must equal a fixed class set |

## Known scope (and what is out) — ADR-0618

This gate validates the **internal shape of one committed JSON document per slice** (extendable
to N documents evaluated in isolation via an optional `additional_specs` follow-up, with **no
joins** between them). Out of scope, and owed to a **separate owned-Rust cross-reference /
registry-integrity (and format-aware) gate** per
[ADR-0618](../../../docs/decisions/ADR-0700-ci-admission-live-apex.md):

- **C1** cross-document reference joins (a value in doc A must exist in doc B).
- **C2** cross-fixture negative joins (a value in one fixture must be absent from another).
- **C3** filesystem path-existence.
- **C4** non-JSON/YAML corpora + raw-text regex.
- **C5** full JSON-Schema-instance validation.

The boundary test: *if a check needs a second document's contents, the filesystem, or a non-JSON
parser to decide pass/fail, it is not a contract-slice check* — route it to the cross-reference
gate backlog rather than distorting a spec to fit.

**Obfuscation boundary.** The deterministic forbidden-marker check covers case + separator +
zero-width + bidi-**reorder** obfuscation. Visually-**confusable homoglyph** substitution
(non-ASCII lookalikes, e.g. Greek/Cyrillic `рrоduction`) is explicitly **out of the deterministic
gate's scope**: it is unbounded (the full Unicode confusables space), legitimate i18n legitimately
uses non-ASCII, and it is caught by the **advisory LLM/NLI + human review** layer per ADR-0617's
deterministic-invariants-plus-advisory doctrine — the same evidence-admissibility boundary. This is
a principled scope line, not a silent drop (ADR-0618).
