# oya-ci gate catalog

Each gate is a thin, pure projection: the producer builds an input, the gate's `evaluate_keyed`
turns it into a set of `Finding{code, key}`, and the firewall ratchets the per-(gate,code) keys
against the committed baseline. Gates are config-declared in `[[gates.enabled]]`; a repo enables
the gates of the packs it uses.

## Input KINDs (the §3.5 INPUT-BINDING abstraction)

Each enabled gate declares HOW its current keys are sourced:

- **`producer-face`** — the producer builds a face (a `Value`), and the gate's pure
  `evaluate_keyed(&face)` produces the keys. Eleven gates use this; each binds one `face`.
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

A non-Rust repo enables `core` only; oyatie enables `core + rust-cargo + rust-cargo-workspace +
agent-wiring + catalog`.

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
| `cloud-ci-workspace-glob-coverage` | rust-cargo-workspace | producer-face (`workspace_glob_coverage`) | `workspace_member_explicit_path`, `crate_dir_not_covered` |
| `cloud-ci-target-parity` | rust-cargo-workspace | producer-face (`target_parity`) | `member_missing_buck` (frozen-empty), `member_test_code_without_rust_test_target` |
| `cloud-ci-enforcement-liveness` | agent-wiring | producer-face (`enforcement_liveness`) | `hook_unwired_without_stub_marker` (frozen-empty), `hook_wiring_mirror_drift` (frozen-empty), `wired_hook_missing_file` (frozen-empty) |
| `cloud-ci-freshness` | rust-cargo-workspace | frozen-empty-meta | `lock_missing_member_package`, `lock_stale_member_version`, `lock_orphan_path_package`, `generated_face_stale` |
| `cloud-ci-friction-accounting` | governance | standalone self-test (own committed baseline) | `friction_policy_gate_id_mismatch` (frozen-empty), `friction_missing_required_field`, `friction_unknown_status` (frozen-empty), `friction_no_disposition` (born-blocking-clean), `friction_closed_without_evidence`, `friction_accepted_risk_without_evidence`, `friction_duplicate_primary_row` (frozen-empty), `friction_orphan_update_row` |
| `cloud-ci-canonical-json` | governance | standalone self-test (zero baseline) | `json_not_canonical` (born-blocking-empty), `json_parse_error` (born-blocking-empty), `json_duplicate_key` (born-blocking-empty) |
| `cloud-ci-embedded-asset-hermeticity` | hermeticity | standalone self-test (own committed baseline) | `embedded_asset_unmapped_include` (born-blocking frozen-empty), `embedded_asset_policy_gate_id_mismatch` (frozen-empty); non-blocking skips: `skip_non_literal_argument`, `skip_absolute_literal`, `skip_build_output_path`, `skip_no_owning_target`, `skip_buck_unparseable` |
| `cloud-ci-kernel-purity` | rust-cargo-workspace | standalone self-test (born-blocking, no baseline) | `KP-TRANSIENT-DEP-CARGO` (born-blocking-clean), `KP-TRANSIENT-DEP-BUCK` (born-blocking-clean), `KP-UNRESOLVED-PATH-DEP` (born-blocking-clean), `KP-STALE-EXCEPTION`, `KP-EMPTY-SCAN`, `KP-POLICY-GATE-ID-MISMATCH` |

For `cloud-ci-freshness` generated-face remediation, `oya-cloud-ci-face-settle --settle --commit`
enforces the content-first, faces-only settle protocol after the content commit lands.

`cloud-ci-friction-accounting` (ADR-0544) is a standalone born-blocking self-test, NOT a
producer-face/raw-corpus gate routed through the central `gate-baseline.generated.json` firewall (the
producer's `RawCorpusCollector` dispatch is hardwired to the single brand-residue collector). It runs
as its `oya-cloud-ci-friction-accounting-app-gate` buck2 `rust_test` under the binding
`buck2 test //cloud/cloud-ci/...` CI job (and a labeled per-crate matrix check), and owns its own
reviewed shrink-only `friction-accounting-baseline.json` + ceilings (FRIC-1781112000 anti-laundering)
rather than the central baseline. Same firewall *semantics* (frozen-empty + shrink-only legacy debt),
local enforcement. All policy — the ledger path, the free-text status taxonomy, the evidence rules —
is DATA in `friction-accounting-policy.json`, so the gate runs on any repo by repointing the policy.

`cloud-ci-canonical-json` (ADR-0546) is a standalone born-blocking self-test, NOT a producer-face
gate. It walks every tracked `*.json` under the policy's `governed_roots` (read-only filesystem),
canonicalizes each committed file with a **self-contained lexical canonicalizer** (a hand-written
lexer → CST → formatter, NOT `serde_json::to_string_pretty`, so the gate's output is independent of
the `serde_json` `preserve_order`/`arbitrary_precision` feature union reindeer applies workspace-wide),
and flags any file whose committed bytes differ from the canonical re-serialization. It runs as its
`oya-cloud-ci-canonical-json-app-gate` buck2 `rust_test` under the binding `buck2 test
//cloud/cloud-ci/...` CI job (and a labeled per-crate matrix check). The live corpus is GREEN at a
**zero baseline** (all three codes born-blocking-empty), so any NEW non-canonical governed json fails
closed; the `--fix` mode of the gate binary is the one-command fixer (local bridge feedback only — the
gate test is the merge authority). The canonical form (`ensure_ascii=false` literal UTF-8, 2-space
indent, source key order, trailing newline) is chosen consistent with the faces serializer
(`accounting-registry::to_canonical_json`); the `*.generated.json` faces and `specs/fixtures/` are
excluded (single-owner: faces are owned byte-verbatim by freshness, fixtures by their consuming gates).
All policy — governed roots, canonical-form params, exclusions — is DATA in `canonical-json-policy.json`,
so the gate runs on any repo by repointing the policy. Converts FRIC-1781130000 (escaped↔literal
serialization churn).

`cloud-ci-embedded-asset-hermeticity` (ADR-0545) is a standalone born-blocking self-test that
statically asserts every Rust `include_str!`/`include_bytes!` string-literal path resolves, inside the
owning Buck target's `__srcs` tree, to a member of {srcs short-paths ∪ `mapped_srcs` VALUES} — so an
embedded asset mapped to the wrong sandbox location (FRIC-1781131000: `couldn't read …`, masked as
`missing rmeta`) cannot reach the merge queue. It reimplements Bazel/Buck hermetic-action
missing-input detection Rust-native. The blocking `embedded_asset_unmapped_include` code is
born-frozen-empty (the live corpus is hermetic after the webhook cedar-adapter prerequisite fix);
sites the conservative lexical parser cannot fully resolve are surfaced as non-blocking `skip_*`
codes, baselined shrink-only with reviewed ceilings in `embedded-asset-hermeticity-baseline.json`
(FRIC-1781112000 anti-laundering) — never silent, never verdict-flipping. Per the founder
automation-default directive (2026-06-11), the gate ships an auto-remediator: `buck2 run
//cloud/cloud-ci/gates/oya-cloud-ci-embedded-asset-hermeticity-app:oya-cloud-ci-embedded-asset-hermeticity-fixer
-- --fix` DERIVES and APPLIES the corrected `mapped_srcs` entry (or the cedar comprehension rewrite)
for an unmapped asset — the default developer/agent path — while the `*-gate` rust_test is the
enforcement backstop whose failure detail prints the exact `--fix` command. All policy (scan roots,
extension sets, build-output dirs) is DATA in `embedded-asset-hermeticity-policy.json`, so the gate
runs on any repo by repointing the policy. Key shape: `embedded-asset-hermeticity` keys are
`<target>::<crate-relative .rs>::<include literal>` for bound sites, or `<file>:<line>` for early
skips (non-literal / build-output / no-owning-target); `<policy>` for the gate-id-mismatch sentinel.
`cloud-ci-kernel-purity` (ADR-0547) is likewise a standalone born-blocking self-test, NOT a
producer-face gate. It enforces the clean-architecture cutover seam: a crate named `*-kernel` or
`*-core` (the cutover-stable interface seam) — and every workspace-internal crate reachable through
its path-dependency closure — must carry zero ADR-0510 transient-tech dependencies (kube, sqlx,
rustls, the AWS SDKs, etcd). It runs as its `oya-cloud-ci-kernel-purity-app-gate` buck2 `rust_test`
(plus a labeled per-crate matrix check). All kernel/core crates are pure today, so it ships
born-blocking with NO baseline file: any new kernel-with-transient-dep fails closed on arrival.
The kernel globs, the transient denylist, the per-crate exceptions, and the liveness floor are all
DATA in `kernel-purity-policy.json` (R0), so the gate runs on any repo by repointing the policy.
The denylist targets transient infra adapters only — legitimate cutover-stable kernel primitives
(`aws-lc-rs`, `libc`, `zeroize`, `tokio`) are absent from it, and `kube` exact + `kube-` hyphen
prefix denies the kube-* family without matching the owned `kuberos`/`oya-cloud-kernel-*` crates.
Automation-default (founder directive 2026-06-11): the `oya-cloud-ci-kernel-purity-app-bin` binary
detects and reports by default and auto-fixes the derivable subset under `--fix` (**Cargo.toml
only** — BUCK `--fix` is descoped to refusal-only pending the `oya-buck-syntax-kernel` fixer
harness, FRIC-1781200001). A dead transient *normal* dep declared but unreferenced in src/build.rs
is removed from Cargo.toml mechanically. Four safety bounds keep the Cargo fixer from corrupting a
manifest: (i) build-deps are never auto-fixed; (ii) the remover is table-aware (never touches
`[dev-dependencies]`/`[features]`); (iii) renamed deps are never auto-fixed; (iv) deps referenced
in `[features]` in any syntax (`dep:X`, `X`, `X/feat`, `X?/feat`, across all dep tables) are never
auto-fixed — `collect_features_referenced_deps` enforces this. After every Cargo edit, `cargo
metadata` is run as a semantic revalidation gate; on failure all pre-images are restored (CRITICAL-A
layer 2 rollback). A dep that IS used in source, is renamed, is a build-dep, or is feature-backed
is a design action, printed with a reason-specific next step but never auto-applied. The buck2
`rust_test` gate is the blocking backstop.

## Key shapes (what a `key` identifies)

- total-accounting / staleness: the registry row `path`.
- cross-artifact: a decision id.
- automation-ratchet: a surface/row id.
- brand-residue: the file path containing a stem (per-file, NOT per-line — stable under in-file
  edits; only fully cleaning a file shrinks the set).
- bnf / manifest: the crate name.
- cargo-prefix: the workspace member path.
- slo-coverage: the catalog crate id.
- workspace-glob-coverage: the raw member entry or crate manifest directory.
- target-parity: the workspace member path.
- enforcement-liveness: the hook path, or `<wiring_file>:<command_path>` for missing referenced
  hook files.
- freshness: the workspace member path, sourceless lock package name, or generated face filename.
- friction-accounting: the friction `id` (per-friction, folded across its event-sourced append rows);
  `<policy>` for the gate-id-mismatch sentinel.
- canonical-json: the repo-relative tracked json file path (per-file).
- kernel-purity: `<kernel>:<closure-node>:<dep>` for a transient dependency (naming the kernel, the
  closure crate that carries the dep, and the dep); `<crate>:<dep>` for a stale exception; `<policy>`
  for the empty-scan and gate-id-mismatch sentinels.

## frozen-empty codes

A `frozen_empty: true` disposition forces a code's baseline to be permanently empty regardless of
current keys, so ANY occurrence is NEW debt the firewall blocks. `registry_drift` (under
total-accounting), `ratchet_regression` + `duplicate_row_id` (under automation-ratchet),
`reap_without_report` (under staleness), `member_missing_buck` (under target-parity), all
`cloud-ci-enforcement-liveness` codes, all `cloud-ci-freshness` codes, and
`friction_policy_gate_id_mismatch` + `friction_unknown_status` + `friction_duplicate_primary_row`
(under friction-accounting) are frozen-empty meta codes — they cannot accumulate a baseline.
