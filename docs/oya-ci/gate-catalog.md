# oya-ci gate catalog

Each gate is a thin, pure projection: the producer builds an input, the gate's `evaluate_keyed`
turns it into a set of `Finding{code, key}`, and the firewall ratchets the per-(gate,code) keys
against the committed baseline. Gates are config-declared in `[[gates.enabled]]`; a repo enables
the gates of the packs it uses.

## Input KINDs (the §3.5 INPUT-BINDING abstraction)

Each enabled gate declares HOW its current keys are sourced:

- **`producer-face`** — the producer builds a face (a `Value`), and the gate's pure
  `evaluate_keyed(&face)` produces the keys. Seventeen gates use this; each binds one `face`.
- **`raw-corpus-collector`** — the keys arrive ALREADY GROUPED `code -> keys` from a raw-corpus
  census the binary runs over the tracked text files (NOT a face, NOT `evaluate_keyed`).
  `cloud-ci-brand-residue` uses this.
- **`frozen-empty-meta`** — the gate contributes NO current keys to
  `gate-baseline.generated.json`; its codes exist only in the disposition table and are stamped
  permanently-empty. `cloud-ci-freshness` uses this because it is enforced by a dedicated
  Buck/GitHub job rather than the accounting-registry producer.

## Packs

- **`core` (language-agnostic):** total-accounting, cross-artifact-agreement, automation-ratchet,
  staleness-reaper (all `producer-face`); brand-residue (`raw-corpus-collector`). Collectors
  operate on tracked text + the ADR/markdown corpus + git history — no language assumption.
- **`rust-cargo`:** bnf-layer-suffix + manifest-hygiene (both `producer-face`). Collectors
  enumerate `Cargo.toml`; consume the `[naming]` + `[manifest]` policy.
- **`rust-cargo-workspace`:** cargo-prefix + workspace-glob-coverage + target-parity (all
  `producer-face`). Collectors enumerate workspace members, crate manifest directories, and
  Buck target parity. Freshness is the standalone `frozen-empty-meta` job for Cargo.lock member
  parity and generated-face byte parity.
- **`agent-wiring`:** enforcement-liveness (`producer-face`). The collector compares tracked
  hook scripts against Claude and Codex project hook wiring.
- **`catalog`:** slo-coverage (`producer-face`). The collector expands catalog record globs.
- **`security`:** license-policy, zero-static-secrets, and load-balancer-inventory
  (`producer-face`). The collectors consume workspace license declarations, tracked-corpus
  credential candidates, and tenant-facing edge/load-balancer inventory.
- **`region-env-readiness`:** multi-region-disposition, sovereign-tenant-pin, and
  tenant-environment-tier (all `producer-face`). The collectors consume accepted readiness
  contracts from service manifests/docs, API gateway/tenant pin contracts, and tenancy/Cedar
  environment-tier surfaces; legacy `oya gate validate ...` names are local-feedback provenance
  only, not protected merge authority.

A non-Rust repo enables `core` only; oyatie enables `core + rust-cargo + rust-cargo-workspace +
agent-wiring + catalog + security + region-env-readiness`.

## The gates

| Gate id | Pack | Input KIND (face) | Violation codes |
|---|---|---|---|
| `cloud-ci-total-accounting` | core | producer-face (`total_accounting`) | `unaccounted`, `unowned`, `unjustified`, `unreachable`, `no_ttl_class`, `registry_drift` (frozen-empty) |
| `cloud-ci-cross-artifact-agreement` | core | producer-face (`cross_artifact`) | `generated_face_drift`, `dual_decision_collision`, `supersession_half_edge`, `unpropagated_decision`, `orphan_decision`, `status_disagreement` |
| `cloud-ci-automation-ratchet` | core | producer-face (`automation_ratchet`) | `advisory_claiming_enforced`, `blocking_invariant_mapped_to_oya_cli`, `ratchet_regression` (frozen-empty), `enforceable_or_automatable_marked_human_judgment`, `duplicate_row_id` (frozen-empty), `unknown_classification`, `missing_or_empty_required_field` |
| `cloud-ci-staleness-reaper` | core | producer-face (`staleness`) | `stale_over_budget_unreachable`, `untyped_staleness`, `reap_without_report` (frozen-empty) |
| `cloud-ci-brand-residue` | core | raw-corpus-collector | one `forbidden_<stem>` code per configured `[vocab]` stem |
| `cloud-ci-bnf-layer-suffix` | rust-cargo | producer-face (`bnf_layer_suffix`) | `bnf_unknown_role`, `bnf_role_mismatch`, `bnf_missing_oya_prefix`, `bnf_empty_after_prefix`, `bnf_undeclared_role`, `bnf_undeclared_context`, `bnf_name_uppercase` |
| `cloud-ci-manifest-hygiene` | rust-cargo | producer-face (`manifest_hygiene`) | `manifest_missing_version_workspace`, `manifest_missing_rust_version_workspace`, `manifest_missing_publish_false`, `manifest_missing_license`, `manifest_missing_lints_workspace`, `manifest_missing_lib_doctest_false` |
| `cloud-ci-cargo-prefix` | rust-cargo-workspace | producer-face (`cargo_prefix`) | `cargo_prefix_violation`, `cargo_prefix_name_path_mismatch`, `cargo_prefix_unresolvable` |
| `cloud-ci-slo-coverage` | catalog | producer-face (`slo_coverage`) | `slo_missing_or_blank_slo`, `slo_empty_crate_id`, `slo_no_catalog_records` |
| `cloud-ci-license-policy` | security | producer-face (`license_policy`) | `license_policy_missing_license`, `license_policy_unknown_license`, `license_policy_forbidden_license`, `license_policy_review_required`, `license_policy_no_workspace_members` |
| `cloud-ci-zero-static-secrets` | security | producer-face (`zero_static_secrets`) | `static_secret_detected` (frozen-empty), policy/exception/corpus shape codes (frozen-empty) |
| `cloud-ci-load-balancer-inventory` | security | producer-face (`load_balancer_inventory`) | `load_balancer_inventory_missing`, `unclassified_tenant_facing_load_balancer`, `direct_mail_workload_load_balancer`, `mail_workload_not_clusterip`, `non_http_edge_missing_authority`, `non_http_edge_port_not_authorized`, `non_http_edge_missing_control`, `mail_public_ingress_bypass` |
| `cloud-ci-multi-region-disposition` | region-env-readiness | producer-face (`multi_region_disposition`) | `manifest_missing_multi_region_disposition`, `manifest_invalid_multi_region_disposition`, `multi_region_doc_missing`, `multi_region_doc_missing_required_section`, `disposition_doc_mismatch`, `active_passive_missing_rpo_rto`, `deployment_shape_mismatch` |
| `cloud-ci-sovereign-tenant-pin` | region-env-readiness | producer-face (`sovereign_tenant_pin`) | `tenant_pin_gate_id_mismatch`, `tenant_pin_no_scenarios`, `tenant_pin_no_admitted_scenario`, `tenant_pin_no_misdirected_scenario`, `tenant_pin_row_missing_field`, `tenant_pin_invalid_tenant_id`, `tenant_pin_home_region_not_allowed`, `tenant_pin_strict_residency_not_single_home_region`, `tenant_pin_allowed_cell_not_admitted`, `tenant_pin_admitted_status_not_202`, `tenant_pin_current_cell_not_allowed`, `tenant_pin_misdirected_status_not_421`, `tenant_pin_location_header_missing` |
| `cloud-ci-tenant-environment-tier` | region-env-readiness | producer-face (`tenant_environment_tier`) | `env_tier_fixture_missing`, `api_key_prefix_unmapped`, `test_key_routes_to_prod`, `outbound_mode_unenforced`, `prod_destructive_ack_missing`, `cedar_key_grant_missing`, `audit_chain_env_tier_missing`, `workflow_default_tier_not_test`, `tier_model_budget_hook_missing` |
| `cloud-ci-workspace-glob-coverage` | rust-cargo-workspace | producer-face (`workspace_glob_coverage`) | `workspace_member_explicit_path`, `crate_dir_not_covered` |
| `cloud-ci-target-parity` | rust-cargo-workspace | producer-face (`target_parity`) | `member_missing_buck` (frozen-empty), `member_test_code_without_rust_test_target` |
| `cloud-ci-enforcement-liveness` | agent-wiring | producer-face (`enforcement_liveness`) | `hook_unwired_without_stub_marker` (frozen-empty), `hook_wiring_mirror_drift` (frozen-empty), `wired_hook_missing_file` (frozen-empty) |
| `cloud-ci-freshness` | rust-cargo-workspace | frozen-empty-meta | `lock_missing_member_package`, `lock_stale_member_version`, `lock_orphan_path_package`, `generated_face_stale` |

For `cloud-ci-freshness` generated-face remediation, `oya-cloud-ci-face-settle --settle --commit`
enforces the content-first, faces-only settle protocol after the content commit lands.

## Key shapes (what a `key` identifies)

- total-accounting / staleness: the registry row `path`.
- cross-artifact: a decision id.
- automation-ratchet: a surface/row id.
- brand-residue: the file path containing a stem (per-file, NOT per-line — stable under in-file
  edits; only fully cleaning a file shrinks the set).
- bnf / manifest: the crate name.
- cargo-prefix: the workspace member path.
- slo-coverage: the catalog crate id.
- license-policy: the workspace package manifest path.
- zero-static-secrets: the candidate file/line key or policy-row key.
- load-balancer-inventory: the resource id or manifest path.
- workspace-glob-coverage: the raw member entry or crate manifest directory.
- target-parity: the workspace member path.
- multi-region-disposition: the service id or manifest path.
- sovereign-tenant-pin: the routing fixture id.
- tenant-environment-tier: the env-tier fixture id.
- enforcement-liveness: the hook path, or `<wiring_file>:<command_path>` for missing referenced
  hook files.
- freshness: the workspace member path, sourceless lock package name, or generated face filename.

## frozen-empty codes

A `frozen_empty: true` disposition forces a code's baseline to be permanently empty regardless of
current keys, so ANY occurrence is NEW debt the firewall blocks. `registry_drift` (under
total-accounting), `ratchet_regression` + `duplicate_row_id` (under automation-ratchet),
`reap_without_report` (under staleness), `member_missing_buck` (under target-parity), all
`cloud-ci-enforcement-liveness` codes, all `cloud-ci-freshness` codes, and the codes explicitly
marked `frozen_empty` in `src/gate-disposition.json` are frozen-empty meta codes — they cannot
accumulate a baseline.
