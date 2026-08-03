# Gateway capability — face-mapping (move-7, from workflow wiq2d2jic; mapping SOUND, critic NEEDS-FIX was completeness/extra-artifact items, NOT mapping defects)

Dispatch move-7 executor AFTER move-6 (cell) post-merge GREEN. Worktree /Users/jasonlee/oyatie-worktrees/p9-gateway (RECREATE on the then-current dev tip before executing). gateway = API gateway / SSOT edge (dag_node api-gateway); clean (NOT a violation source). absorbs oya/api-gateway(0 crates -> phase-2) + oya/connector(10 crates). ALL 10 crates are SaaS connector ADAPTERS -> gateway/adapters/. Gateway is an ALL-ADAPTERS capability for now (core/ports arrive when api-gateway crates are authored — acceptable incremental state). Each adapter's SOLE dep is libs/oya-shared-connector-kernel (stays in libs/, OUT of scope; if it later re-homes -> a future move). ZERO intra-capability edges, ZERO reverse deps into these crates.

## Final mapping (10 crates, all adapters/, collision-free, dep-legal)
| old_crate | new_path | cargo | face |
|---|---|---|---|
| oya-connector-adp-adapter | gateway/adapters/adp-connector | gateway-adp-connector | adapters |
| oya-connector-epic-fhir-adapter | gateway/adapters/epic-fhir-connector | gateway-epic-fhir-connector | adapters |
| oya-connector-gusto-adapter | gateway/adapters/gusto-connector | gateway-gusto-connector | adapters |
| oya-connector-netsuite-adapter | gateway/adapters/netsuite-connector | gateway-netsuite-connector | adapters |
| oya-connector-quickbooks-adapter | gateway/adapters/quickbooks-connector | gateway-quickbooks-connector | adapters |
| oya-connector-rippling-adapter | gateway/adapters/rippling-connector | gateway-rippling-connector | adapters |
| oya-connector-salesforce-adapter | gateway/adapters/salesforce-connector | gateway-salesforce-connector | adapters |
| oya-connector-slack-adapter | gateway/adapters/slack-connector | gateway-slack-connector | adapters |
| oya-connector-teams-adapter | gateway/adapters/teams-connector | gateway-teams-connector | adapters |
| oya-connector-workday-adapter | gateway/adapters/workday-connector | gateway-workday-connector | adapters |

## EXTRA steps beyond the standard code-move (connectors are catalog/SLO/registry-tracked — some gate-ENFORCED, can't defer):
1. STANDARD protocol (do as always): root Cargo.toml += `gateway/*/*` glob; registry absorbs_current_dirs += gateway; membership scan_roots+allowed_top_level_dirs += gateway; acyclicity crate_root_globs += gateway/*/* + unclassified_roots += gateway; gateway/OWNERS; ADR §10.x verbatim paths; reachability seeds; committed specs/reorg/gateway-move-plan.json + manifest regen; Cargo.lock sync.
2. specs/capability-registry.json §6 membership_lint_coverage: the pre-move path mappings reference oya/connector — update so the membership gate maps the new gateway/adapters/* crates (this is part of the standard registry edit but VERIFY the §6 coverage block specifically).
3. registry/catalog/oya-connector-<vendor>-adapter.yaml (10 per-crate SLO/catalog files) — the slo-coverage gate reads registry/catalog/*.yaml (oya-ci.toml slo_coverage.catalog_record_globs). RENAME/update each to the new crate id (gateway-<vendor>-connector). RUN the slo-coverage + cargo-prefix + naming gates and fix whatever REDs.
4. scripts/reject-retired-grouping-wording.sh (line ~37 hardcodes crates/oya-connector-netsuite-adapter + registry/catalog/oya-connector-netsuite-adapter.yaml) + scripts/tests/reject-retired-grouping-wording.test.sh — update the hardcoded example paths (if this script runs in CI it will RED on the moved path). NOTE .sh is transitional but must stay green.
5. registry/stores/registry-store.json + registry/dependency-rationales.json — rename per-crate keys; DECIDE the registry-store `capability` field value (was connector-*; set to gateway or keep — match what the membership/registry gates expect).
6. BRAND SCRUB (de-brand, comment-only, residue-removal): each crate src/lib.rs leads with `//! oya-connector-<vendor>-adapter` self-name doc comment -> update to the new name; slack lib.rs:18 also has a stale `oya-intelligence-adapter-anthropic-api-kernel` doc cross-ref -> scrub. (These are not deps; comment-only.)
7. docs/standards/crate-naming-convention.md cites netsuite+salesforce connector names as examples -> refresh. tasks/adr-0357-crate-classification.json entries -> update. (docs/audit census + root scratch files = descriptive, phase-2 #62/#60 unless a gate REDs.)

## Verify: full gate suite GREEN vs merge-base (esp. slo-coverage + the reject-grouping script + membership + naming), forbidden_* 0 regression, grep-clean of old crate tokens in moved crates + any gate-enforced artifact. If a gate REDs on a non-code artifact, UPDATE that artifact (don't defer/signoff) — it's gate-enforced. Truly-descriptive drift (docs/audit) -> phase-2. NO signoff doors.
