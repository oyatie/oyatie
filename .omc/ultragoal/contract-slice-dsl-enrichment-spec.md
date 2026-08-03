# Contract-Slice DSL Enrichment — full-fidelity spec (founder-approved 2026-07-10)

Root-cause fix for the contract-slice conversion whack-a-mole. Gate: `ci/facade/contract-slice-conformance`.

## Ground truth (load-bearing)
- **dev today has 7 slice keys**: `required_fields`, `required_true_fields`, `enum_constraints` (string leaf only), `required_array_members` (string superset), `required_object_array_members` (nested member_key/member_required_fields/member_enum_constraints), `forbidden_markers` (per-slice + universal `FORBIDDEN_SPEC_MARKERS`, case-insensitive whole-spec), `source_migration_slice`. Backstop: `check_keys` + `contract_slice_unknown_policy_key`.
- `conditional_assertions`/`field_implies_required`/`exact_members`/`exact_array_fields` are NOT on dev — they're on the **unmerged #1296 branch**. `scalar_str` is on the **#1294 branch**. The two branches added **DISJOINT primitives to the SAME `RECOGNIZED_SLICE_KEYS`/enum loop → they conflict on rebase; neither is a superset.**
- #1290/#1297 did NOT touch lib.rs — they coped by (a) distorting specs (talos flattened `matrix` array→object; compliance downgraded exact-set to superset) and (b) silently dropping (prefixItems order, ==false, hex shapes, nonclaim wording, path existence).
- **Fix = one consolidation PR lands the UNION (hardened) FIRST; the 4 conversions then carry data-only policy entries + never touch lib.rs.**

## Group A — consolidate the two branches into dev, then harden (all CodeRabbit fail-closed)
- **scalar_str** (#1294): enum/array members match numeric/bool leaves (numeric pins authored as strings).
- **exact_array_fields** `[{field,values}]` ordered no-extras equality. **BUG:** `filter_map(as_str)` drops non-string extras → must count non-strings as mismatch.
- **required_markers** `[{field,markers}]`: subtree contains all. Fail-closed on empty/missing markers or non-string field.
- **skip_universal_markers** bool: narrow opt-out (never skips slice's own forbidden_markers); gate on migration/shared justification.
- **exact_members** bool on object-array: reverse membership (no undeclared member_key).
- **conditional_assertions** (nested): **fail-closed** — EXACTLY ONE selector (`when_member`/`when_member_in`) and EXACTLY ONE mode (`must_equal`/`must_be_true`/`must_contain`/`must_subset_of`); `must_subset_of` rejects non-strings. (Today: malformed selector→applies-to-all; both selectors silently accepted; multiple modes take first — CodeRabbit Major ×2.)
- **field_implies_required** (nested): fail-closed on missing/empty then_required_fields or non-string if_field.
- **nested-dotted member fields**: keep (residency capability_overrides.enforcement.lane_id).
- **Invariant for the PR: every new primitive fails closed on its own malformed shape** (same doctrine as check_keys). Each CodeRabbit Major = one instance.

## Group B — genuinely missing, add (T1/T2)
- **B1 `required_false_fields`** (T1): mirror required_true_fields; code `contract_slice_field_not_false`. (compliance non_necessary_default==false)
- **B2 positive `any_of` quantifier** (T1): add `quantifier:any_of|all_of` (+ optional `scope:whole_spec`) to required_markers; any_of REDs only when NO marker matches. (compliance require_explicit_nonclaim; talos non_claims wording)
- **B3 forbidden hardening**: (a) **separator normalization (REAL BUG):** `production-ready` bypasses `production ready` — normalize `[^a-z0-9]+→" "` on haystack+needle before contains (T1); (b) field-scoped forbidden `[{field,markers}]` (T2) — talos can_claim_now without tripping cannot_claim_yet; (c) subtree exclusion `marker_exclude_fields` (T2) — claim_boundary self-trip.
- **B4 `field_patterns`** `[{field,pattern}]` regex (T2): **regex="1" is ALREADY a workspace dep (Cargo.toml:850) → not adhoc.** Covers Ed25519 `[0-9a-f]{128}`, SHA-256 `[0-9a-f]{64}`, region id `^[a-z]{2}(_[a-z0-9]+)+$|^global_multi$`, base64url len86. Bad pattern = `contract_slice_bad_pattern` fail-closed.
- **B5 `exact_projected_sequence`** `[{field,member_field,values}]` (T2): ordered equality of a projected field across array-of-objects. (compliance canonical_purposes ORDER; release stage flow)
- **B6 `array_cardinality`** `[{field,min?,max?,unique_by?}]` (T2): (compliance entries>=2; talos id uniqueness)
- **B7 projected value-set coverage** `[{field,member_field,exact_values}]` (T2): non-key field set == fixed set. (residency REGISTRY_CLASSES; catches copy-paste dup)

## Group C — architectural, OUT (bound with a founder-visible ADR)
- C1 cross-spec_path reference joins (residency ~40% joins). C2 cross-fixture NEGATIVE join. C3 filesystem path-existence (talos). C4 non-JSON YAML corpus + raw-text regex (release ~70%). C5 full JSON-Schema-instance validation (residency offlineChannelProtocol).
- **Boundary line:** this gate validates ONE committed JSON doc's internal shape per slice (extendable to N in isolation via optional T2.5 `additional_specs`). Cross-document/filesystem/YAML/full-schema = a DIFFERENT capability (a cross-reference/registry-integrity or schema validator) → a SEPARATE owned-Rust gate, recorded in the ADR, not silently dropped.

## Recommendation
- **Ship: Group A (consolidated+hardened) + B-T1 + B-T2.** Makes compliance / residency(single-spec) / talos / finops **byte-faithful**; removes spec distortions. regex is free (workspace-pinned) → include hex/length shapes.
- **Optional T2.5 `additional_specs`** `[{spec_path,<same primitives>}]` (multi-spec, NO joins) as a follow-up to unlock release's JSON parts. `live_corpus` test helper already loads every spec_path.
- **Bound T3 (C1–C5) with an ADR. Release/#1294 is the honest casualty (~70% YAML+cross-doc)** — route its JSON parts via T2.5, its YAML/cross-doc to a separate gate; do not force it here.

## Implementation plan (own PR to dev FIRST; the 4 conversions rebase onto it, data-only)
Confined to `src/lib.rs` (primitives + #[cfg(test)]), `tests/contract_slice_conformance.rs` (live-corpus RED cases asserting exact Finding.key), `README.md`, `Cargo.toml` (+regex={workspace=true}), `BUCK` (+third-party//:regex to both targets). No new BUCK target/OWNERS; existing crate so no register_crate.
TDD order: (1) consolidate Group A (union RECOGNIZED_SLICE_KEYS; port #1296 tests) → both green; (2) harden Group A RED-first per CodeRabbit; (3) B1→B2→B3; (4) B4(regex)→B5→B6→B7; (5) optional T2.5; (6) live-corpus RED cases; (7) README + boundary ADR (Proposed).
Backward-compat: exemplar/CELL-002/FINOPS/rollback-audit slices stay GREEN (tests assert exact keys); new keys additive; every new key in RECOGNIZED_SLICE_KEYS same commit.

## Source refs
- Python: `git show 389f4a340:scripts/tests/compliance_pack_contract_slice_check.py` · `9dd37161f:…/residency_contract_slice_check.py` · `d98f2ed5b:…/talos_001_substrate_slice_check.py` · `f15d8adae:…/release_001_runtime_safety_check.py`
- Branch deltas: #1294 `work-hermes-t_62cf60fe-release-001` (scalar_str); #1296 `work-hermes-t_215c8b00-residency-001` (the six).
