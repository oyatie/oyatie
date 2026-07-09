# contract-slice-conformance gate

Paved-road owned-Rust/Buck2 cloud-ci gate that **replaces the fleet-wide
`scripts/tests/*_check.py` "contract slice" validators**. It is the class-fix for
the recurring anti-pattern where a spec slice's only enforcement is a new Python
script that `oya-ci-required` never runs (and that only mirrors the constants it
hardcodes).

The gate is a pure evaluator (`evaluate_configured(policy, corpus)`): it never
shells out, spawns an interpreter, mutates files, or reads ambient state. All
per-slice rules are **data** in `contract-slice-policy.json`.

## Add a contract slice (no new Python, shell, CLI, or crate)

1. Commit your slice's spec JSON (e.g. `specs/<your-slice>.json`).
2. Add one entry to `contract-slice-policy.json`:

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

3. `buck2 test //ci/facade/contract-slice-conformance:ci-contract-slice-conformance-gate`.

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
