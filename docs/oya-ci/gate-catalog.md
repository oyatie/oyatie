# oya-ci gate catalog

Each gate is a thin, pure projection: the producer builds an input, the gate's `evaluate_keyed`
turns it into a set of `Finding{code, key}`, and the firewall ratchets the per-(gate,code) keys
against the FROZEN merge-base baseline — the `gate-baseline.generated.json` face as committed at
`git merge-base <base_ref> HEAD` (ADR-0551; never the PR-local copy, which the settle protocol
itself regenerates). FROZEN-POLICY-WINS (FRIC-1781280000): the policy facts that select that
frozen reference (`base_ref`, `face_path`) are themselves read from the merge-base tree against
an out-of-band bootstrap ref, so a same-PR `ratchet-policy.json` repoint cannot affect the PR's
own frozen reference; and a sign-off door entry whose key exists nowhere (frozen face, current,
proposed) is flagged inert and must be retired (FRIC-1781280001). Per the founder
automation-default directive (2026-06-12), the inert retirement ships as an auto-remediator:
`buck2 run //cloud/cloud-ci/gates/oya-cloud-ci-firewall-app:oya-cloud-ci-firewall-signoff-fixer
-- --fix` derives and applies the retirement (reparse-and-refuse self-validation; audit record
appended to `_sign_off_retirements`), while the firewall gate's inert-door RED is the
enforcement backstop whose failure detail prints that exact command. A NEW code class with live
violations defaults to blocking in compare-mode (a code absent at the merge-base uses its
proposed stamp), so advisory-first onboarding of a new code requires the sign-off door for the
initial key set — the reviewed disposition flip later freezes it. Gates are config-declared in
`[[gates.enabled]]`; a repo enables the gates of the packs it uses.

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
| `cloud-ci-cross-artifact-agreement` | core | producer-face (`cross_artifact`) | `generated_face_drift`, `dual_decision_collision`, `decision_id_mismatch` (frozen-empty), `phantom_decision_citation` (frozen-empty), `supersession_half_edge`, `unpropagated_decision`, `orphan_decision`, `status_disagreement` |
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
| `cloud-ci-no-graphql-without-adr` | governance | standalone self-test (candidate-tree, EMPTY frozen baseline) | `NGQL-FORBIDDEN-LIB` (born-blocking-empty), `NGQL-SCHEMA-FILE` (born-blocking-empty), `NGQL-LOCK-FORBIDDEN` (born-blocking-empty), `NGQL-EMPTY-SCAN`, `NGQL-POLICY-GATE-ID-MISMATCH`, `NGQL-POLICY-MALFORMED` |
| `cloud-ci-authz-coverage` | rust-cargo-workspace | standalone self-test (born-blocking vs frozen baseline of known-unauthenticated surfaces) | `AC-UNAUTHENTICATED-CONTROL-PLANE` (issue #770 / AUTH-005; blocks only NEW unauthenticated axum control planes — mutating route or per-resource path with no router-level auth layer and no per-handler authz guard), `AC-STALE-BASELINE` (shrink-only self-clean), `AC-EMPTY-SCAN`, `AC-POLICY-GATE-ID-MISMATCH`, `AC-POLICY-MALFORMED` |
| `cloud-ci-affected-set` | buck2-workspace | binding workspace-coverage lane (ADR-0554: merge-base diff → `owner()`/`rdeps()` cone on `pull_request`; full workspace on `merge_group`/`push`/`dispatch`) | lane verdict = the buck2 build+test result of the decided set; REFUSE on owner-required files with no owning target; escape-trigger classes and EVERY derivation failure escalate fail-closed to FULL (never skip); the PR FULL tier is a BUILD-HEALTH RATCHET (D6) — blocks build REGRESSIONS vs the merge-base, grandfathers pre-existing build debt (no flag-day) |
| `cloud-ci-supply-chain-audit` | rust-cargo-workspace | standalone self-test (born-blocking; matches `Cargo.lock` against the vendored content-addressed RustSec advisory mirror) | `SCA-VULN` (security advisory affecting a locked crate, not ignored), `SCA-UNMAINTAINED` (unmaintained advisory affecting a locked crate, `unmaintained_policy=all`, not ignored), `SCA-STALE-IGNORE` (shrink-only self-clean of an ignore that suppresses nothing live), `SCA-MIRROR-MALFORMED` (manifest `content_hash`/`advisory_count` desync — fail-closed), `SCA-MIRROR-UNDERFLOW` (mirror below `min_advisories` floor — fail-closed vs a vacuously-green truncated snapshot), `SCA-POLICY-GATE-ID-MISMATCH`, `SCA-POLICY-MALFORMED`. Owned pure-Rust replacement for the reverted #974 shell cargo-audit/deny (`serde_json`+`toml`+`semver` only — no `rustsec`/`git2`/`libgit2`, no shell/network/clock; the network/clock refresh is a separate owned reconciler, deferred). See ADR-0605. |
| `cloud-ci-cloud-resource-contracts` | cloud-contracts | standalone self-test (born-blocking; policy + typed JSON corpus) | Rust/API-shaped replacement for the P0 Python cloud-resource validators. The gate evaluates `cloud-resource-contracts-policy.json` plus the six configured spec inputs, preserving claim-boundary, operation-contract, and enforceability-facet checks without invoking Python. Key codes are prefixed `cloud_resource_contract_*`, `cloud_operation_*`, and `cloud_enforceability_*`; use `//cloud/cloud-ci/gates/oya-cloud-ci-cloud-resource-contracts-app:oya-cloud-ci-cloud-resource-contracts-app-gate` as the merge-authority target. |

For `cloud-ci-freshness` generated-face remediation, `oya-cloud-ci-face-settle --settle --commit`
enforces the content-first, faces-only settle protocol after the content commit lands.

This paragraph is the CANONICAL settle+verify protocol statement (FRIC-1781250000,
FRIC-1781234047/ADR-0552; other documents point here instead of restating it). The committed
faces are a pure function of the tracked TREE state: history-derived volatile facts (per-path
`last_touch_commit`, commit timestamps, the aging anchor) live in the UNTRACKED
`scm-volatile-facts.generated.json` snapshot beside the emitter, never in a committed face —
so neither a later commit, nor a squash-merge to the base branch (which rewrites every lane
commit id), can un-settle settled faces. Commits that change face-relevant tree content
(tracked-path set, ownership/justification/reachability sources, gate inputs) still un-settle
them. The settle commit should therefore be the FINAL commit before push, and
`oya-cloud-ci-face-settle --verify` is the REQUIRED last step before EVERY push, explicitly
including pushes the worker believes are content-only. `--verify` is read-only (it never writes
to the repository): against a working tree asserted byte-identical to the committed tree (HEAD),
it regenerates the faces in memory/tempdir and runs the freshness gate's OWN full check —
generated-face byte parity AND `Cargo.lock` member parity — and on any finding exits nonzero
with the per-face stale list / lock findings and the exact remediation commands
(`oya-cloud-ci-face-settle --settle --commit` for faces, `cargo metadata >/dev/null` plus a
content commit for the lock). The cloud-ci freshness gate (ADR-0539) remains the canonical
enforcement backstop per the enforcement-layering doctrine; `--verify` is the automation-default
local check (ADR-0548 D6) in front of it, never a substitute for it.

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
detects and reports by default and auto-fixes the derivable subset under `--fix` — the Cargo.toml
dep line plus its dead `third-party//:<dep>` rust_library BUCK edge, the latter via the shared
`oya-buck-syntax-kernel` sound parser + write-through fixer harness (ADR-0549 closed the
FRIC-1781200001 refusal-only descope; unsound BUCK shapes still refuse with the file
byte-identical). A dead transient *normal* dep declared but unreferenced in src/build.rs is
removed mechanically. Five safety bounds (ADR-0547 D6) keep the fixer from corrupting a manifest
or removing a live dep: (i) build-deps are never auto-fixed; (ii) the remover is table-aware
(never touches `[dev-dependencies]`/`[features]`); (iii) renamed deps are never auto-fixed;
(iv) deps referenced in `[features]` in any syntax (`dep:X`, `X`, `X/feat`, `X?/feat`, across all
dep tables) are never auto-fixed — `collect_features_referenced_deps` enforces this; (v) deps
declared `optional = true` are never auto-fixed even with zero own-manifest `[features]` mentions
— an optional dep exports an implicit cargo feature a SIBLING workspace member can request, which
neither the own-manifest scan nor `cargo metadata --no-deps` can see (FRIC-1781210000). After the
edits, `cargo metadata` is run as a semantic revalidation gate; on failure ALL pre-images are
restored — keyed by path with the FIRST pre-image winning, so a manifest edited twice rolls back
to its ORIGINAL content (CRITICAL-A layer 2 rollback). A dep that IS used in source, is renamed,
is a build-dep, is optional, or is feature-backed is a design action, printed with a
reason-specific next step but never auto-applied. The buck2 `rust_test` gate is the blocking
backstop.

`cloud-ci-no-graphql-without-adr` (ADR-0565) is likewise a standalone born-blocking self-test, NOT
a producer-face gate. It enforces the founder zero-GraphQL doctrine: the canonical owned API surface
is REST + gRPC + AsyncAPI + realtime (SSE / WebSocket / gRPC-streaming), and GraphQL returns ONLY by
a future ADR that explicitly reverses ADR-0565. The drop PR (#775 / ADR-0565) deletes every GraphQL
artifact; this gate is the enforcement half (enforcement-layering: the drop is construction, the gate
is the recurrence backstop). It fails closed if the candidate tree reintroduces, WITHOUT citing an
ALLOWLISTED + VALIDATED authorizing (reversing) ADR, ANY of: a GraphQL execution/parse library in
ANY `Cargo.toml` in the tree (`async-graphql`, `juniper`, `graphql-parser`, `cynic`, `apollo-*`, …) —
members AND non-members, resolving `[workspace.dependencies]` renames and `{ workspace = true }`
inheritance; a forbidden GraphQL crate in the resolved `Cargo.lock` graph (`NGQL-LOCK-FORBIDDEN`, the
transitive catch); or a `.graphql`/`.graphqls`/`.gql`/`.gqls`/`.sdl` schema file. It runs as its
`oya-cloud-ci-no-graphql-without-adr-app-gate` buck2 `rust_test` (plus a labeled matrix check).
CANDIDATE-TREE EVALUATION, NOT a frozen merge-base: the collector is a hermetic, read-only `fs` scan
of EVERY `Cargo.toml` in the live tree + the `Cargo.lock` graph + a `.graphql`/`.graphqls`/`.gql`/
`.gqls`/`.sdl` file walk (no `cargo`/`buck2` shell-out) — so the verdict is identical at PR-tier and
push-tier, avoiding the gate-baseline-pr-push-asymmetry false-green (a GraphQL artifact added on dev
between branch-point and merge cannot pass PR-tier and only fail on the integrated tip). The post-drop
tree is GraphQL-free, so it ships born-blocking with an EMPTY frozen baseline: any new GraphQL library,
schema file, or transitive lock crate fails closed on arrival. The ADR escape-hatch is NOT a bare-token
match: a forbidden artifact launders ONLY by citing an `ADR-NNNN` that is BOTH enumerated in the policy
`authorizing_adrs` allowlist (EMPTY today — nothing authorizes GraphQL) AND validated against the real
`docs/decisions` tree (an Accepted ADR that reverses ADR-0565). So a fabricated/typo id (`ADR-9999`,
`ADR-05650`) cannot launder, and a file can never self-launder by naming the forbidding ADR. KNOWN
LIMITATION: an inline-SDL string literal / derive macro with no schema file is not caught by the
schema-file walk, but any real GraphQL server needs a GraphQL library, which the manifest legs + the
`Cargo.lock` leg DO catch. The forbidden crate set (exact + prefix), the schema extensions, the
forbidding-ADR id, the `authorizing_adrs` allowlist, the `decisions_dir`, and the excluded dirs are all
DATA in `no-graphql-without-adr-policy.json` (R0), so the gate runs on any repo by repointing the policy.

## Key shapes (what a `key` identifies)

- total-accounting / staleness: the registry row `path`.
- cross-artifact: a decision id; for `decision_id_mismatch`, the producer's
  `<file>:<filename-id>!=<front-matter-id>` entry; for `phantom_decision_citation`, the
  producer's `<cited-id>@<source-path>` citation edge — an `ADR-NNNN` cited from a governed
  surface (a decision body, the roadmap/sequencing artifact, or the masterplan `bound_adrs`)
  with NO decision file on disk (the phantom-0397 exhibit, audit register H-19, healed
  2026-06-12 by minting `docs/decisions/ADR-0397-pulsar-oxia-canonical-event-bus.md`;
  FRIC-1781430000). Frozen-empty: the pre-existing phantom inventory is grandfathered
  shrink-only DATA in the producer (each id ledgered with its citation sites); remediation is
  mint-the-record-at-the-cited-number (reconstruction, status Proposed) or retarget the
  citation — never silently re-baseline. Decision NUMBER ALLOCATION is mechanical
  (FRIC-1781320000): the accounting-registry producer's `--next-adr` mode prints the next free
  `ADR-NNNN` derived from the tree (max over filename AND front-matter ids, plus one) — lanes
  allocate by running it, never by convention or leader memory; the crosswalk face carries the
  same value as `next_free_id`. Durable destination: content-addressed decision identity
  (ADR-0541 corpus direction), where numbering races vanish by construction.
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

## exists-but-unaccounted codes (ADR-0555 conversion)

`unowned`, `unreachable`, `no_ttl_class` (total-accounting) and `untyped_staleness`
(staleness-reaper) are `baseline-block-on-new`: a NEW artifact that is not ownership-registered
(nearest-ancestor `OWNERS`) and reachability-registered (workspace-member containment, an exact
mention in masterplan/root-hub/DOC-CATALOG, or a reviewed `specs/reachability-registry.json`
prefix) is **unmergeable** — the pre-conversion corpus debt is grandfathered mechanically by the
ADR-0551 merge-base frozen baseline (shrink-only burn-down; zero growth without the sign-off
door). A FAIL is never a bare flag: each code's disposition carries `remediation` DATA — the
exact registration edit, or the precise design decision needed (who owns this? what points at
this?) — stamped into `gate-baseline.generated.json` and printed by the firewall next to the
offending keys. The producer's `--fix-owners <dir>=<owner>` / `--fix-reachability
<prefix>=<anchor>` bridges apply a decided registration and self-validate (transitional local
bridges per `cli_surface_policy` — the gate test is the merge authority; their successors are the
ADR-0548 D3 reconcilers). The ONE deliberately-advisory survivor is
`stale_over_budget_unreachable`: its keys enter by TIME passing, not by PR action, so
admission-blocking would blame PRs for age accrued on other clocks — its convergence surface is
the staleness-reaper archival reconciler, and `unreachable`-at-creation now starves its growth.
