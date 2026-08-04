# Gate-fleet disposition against the post-reorg tree

**Generated:** 2026-07-27 · 50 gates × adversarial verification (16 agents, 1.83M tokens)
**Question:** which `ci/facade` gates survive the capability-first reorg, which must change, and which silently stop working?

## Summary

- **already-vacuous** — 1
- **becomes-vacuous** — 5
- **keep-as-is** — 20
- **must-change** — 24

**Zero gates need removal.** Every gate enforces something that survives. But half are keyed to what disappears, and six stop checking silently.

## BECOMES-VACUOUS / ALREADY-VACUOUS — the silent class (all currently wired)

### automation-coverage — becomes-vacuous

**Legacy coupling:** Indirect but decisive: its live corpus is produced by `collect_enforcement_inputs`, whose dominant row source is the path-substring filter `oya-governance`. Its own `mentions_oya_cli` detector is additionally keyed to the retired `oya-dev-cli` brand literal and is ALREADY dead against the live package name.

**Change required:** Two fixes, both upstream-and-here. (a) Replace the `oya-governance` path-substring source (artifact-inventory-registry/src/main.rs:4011-4016) with a capability-root-derived governance predicate, or this gate's corpus drops ~56 of ~58 surfaces with no RED. (b) Add a real corpus floor to tests/automation_ratchet.rs — the current floors (`!surfaces.is_empty()` :143, `advisory_count > 0` :224, `oya_cli_count > 0` :225) are all satisfied by the 2 surviving governance-lane rows, so they cannot detect the collapse. (c) src/lib.rs:279-287 `mentions_oya_cli` matches `"oya-dev-cli"`/`"cargo run -p oya-dev-cli"`; the live package is `marketplace-dev-cli` (marketplace/facade/dev-cli/Cargo.toml:2) — this detector is already partly vacuous today and must key on a de-branded literal set.

**Evidence:** Wired: .github/workflows/oya-ci-required.yml:161 ("gate · automation-ratchet (GATE-4, polices gates)"). The gate leg runs the PRODUCER over the live tree, not a static seed: ci/facade/automation-coverage/BUCK `ci-automation-coverage-gate` sets `OYA_CI_PRODUCER_BIN = $(exe //ci/facade/artifact-inventory-registry:...-app-bin)`, and tests/automation_ratchet.rs:130 "the producer's enforcement-inventory face over the live tree". Corpus derivation chain: artifact-inventory-registry/src/main.rs:602 `build_automation_matrix(&enforcement)` ← :4011 `let governance_substr = cfg.enforcement.governance_crate_substr` (= "oya-governance", oya-ci.toml `[enforcement]`) ← :4015-4016 `p.contains(&governance_substr) && p.ends_with("/Cargo.toml")`. Live count today: 56. Post-de-brand those crates live under `governance/**` with names like `governance-gate-catalog-domain` — the substring is absent, the rows vanish, and NOTHING reddens. Gate's own Rust is otherwise shape-agnostic (pure evaluator, corpus = `specs/phase0-automation-matrix.json` seed + producer face; `specs/` survives per module-membership/capability-membership-policy.json non_crate_top_level_dirs).

### crate-layer-suffix — becomes-vacuous

**Legacy coupling:** Corpus is filtered by the oya- brand prefix in the PRODUCER, and the gate's own independent census re-applies the same filter. Its 12-value role vocabulary is the name-SUFFIX axis, which ADR-0562 replaces with a path face.

**Change required:** Do not just repoint required_prefix — that converts a silent green into a ~44% bnf_unknown_role RED storm, because the de-brand drops the layer dir from the name (iam/adapters/cloud-oci -> iam-cloud-oci, tail `oci`). The layer axis moved from the name suffix to the path face. Either rewrite the gate to assert the PATH face (<capability>/{core,ports,adapters,facade}/...) — at which point it duplicates module-membership's face check and facade-core-layering — or retire it. Treat this as a rewrite-or-remove decision, not a policy edit.

**Evidence:** Producer: ci/facade/artifact-inventory-registry/src/main.rs:862 `let prefix = cfg.naming.required_prefix.as_str();` then :872 `if name.starts_with(prefix) { names.insert(name); }` — prefix resolves to "oya-" from oya-ci.toml:27 (default at libs/oya-ci-config/src/lib.rs:467-469). The -gate leg's independent census re-applies the SAME filter: ci/facade/crate-layer-suffix/tests/bnf_layer_suffix.rs:91 `let prefix = NamingConfig::default().required_prefix;` and :113 `if name.starts_with(&prefix)`. Its doc-comment at tests/bnf_layer_suffix.rs:88-89 DESIGNS THE SHRINK IN: "Self-adjusts through future de-brands: the census shrinks in lockstep with the face as crates lose the oya- prefix, so this stays valid without ever needing a bump." There is no corpus floor anywhere; the only tripwire is `<empty-rows>` fail-closed at literally zero rows (ci/facade/crate-layer-suffix/src/lib.rs:163-172), which fires only after the LAST crate de-brands. Second, independent break quantified against the committed move-plans: 47 of 107 planned moves de-brand to a name whose trailing dash-segment is not in the 12-value role set (oya-ci.toml:28) — including this gate's own `ci-crate-layer-suffix` (specs/reorg/ci-move-plan.json); top non-canonical tails are policy(6), coverage(4), parity(3). CI leg: .github/workflows/oya-ci-required.yml:162.

### crate-name-prefix — becomes-vacuous

**Legacy coupling:** Entirely built on the `oya-` brand prefix. `NamingConfig::default().required_prefix == "oya-"` is the bundled policy (ci/facade/crate-name-prefix/src/lib.rs:19-21, :192-195). Its corpus is gated on that same prefix upstream in the producer.

**Change required:** Repurpose, do not merely repoint. Replace the prefix predicate with the ADR-0562 path=namespace identity rule: `package.name` MUST equal the de-branded path tail (capability path segments joined with `-`), which is the existing `cargo_prefix_name_path_mismatch` arm generalised. Concretely: (a) delete `cargo_prefix_scope`/`is_advisory_row` (main.rs:894-902, lib.rs:135-137, :173-175) — the advisory escape hatch is the vacuity mechanism; (b) drop `required_prefix` from the blocking path and assert name==path-tail unconditionally; (c) add a blocking-row FLOOR to tests/cargo_prefix.rs so an all-advisory / zero-blocking-row corpus is RED, not Green. If the founder does not want a name==path rule, the honest disposition is `remove` — the prefix shape does not survive the de-brand at all.

**Evidence:** THE VACUITY CHAIN IS CLOSED AND CONFIRMED. (1) Producer classifies scope by prefix: ci/facade/artifact-inventory-registry/src/main.rs:894-902 — `fn cargo_prefix_scope(crate_id, package_name, required_prefix)` returns "blocking" ONLY `if crate_id.starts_with(required_prefix) && package_name.starts_with(required_prefix)`, else "advisory". (2) Gate skips advisory rows: ci/facade/crate-name-prefix/src/lib.rs:173-175 `if is_advisory_row(row) { continue; }`. (3) The fail-closed guards CANNOT catch this: lib.rs:163-166 fires `<empty-rows>` only when the `rows` ARRAY is empty — but advisory rows still populate it. (4) The corpus is root-agnostic so it never shrinks: main.rs:907-911 `scan_valid_member_dirs` -> `oya_workspace_members_kernel::scan_member_dirs(repo_root)`, i.e. root [workspace].members globs, not a path list. (5) The live-corpus test only asserts `rows.len() > 500` (ci/facade/crate-name-prefix/tests/cargo_prefix.rs:90-95) and then `findings.is_empty() => assert Green` (:112-118) — there is NO floor on blocking rows and NO ceiling on advisory rows. NET: once the de-brand lands, all ~500+ rows are advisory, `rows.len() > 500` still passes, blocking findings == 0, verdict GREEN, gate checks nothing, forever, silently. Wired at .github/workflows/oya-ci-required.yml:164. SECOND LOSS: the surviving-value check `cargo_prefix_name_path_mismatch` (lib.rs:107-109, :268-280 — package name must equal the member-path crate-id) is ALSO skipped for advisory rows, and that predicate is precisely the de-brand grammar rule (`iam/adapters/cloud-oci` -> `iam-cloud-oci`). The reorg deletes the one check it should have kept.

### hook-wiring — becomes-vacuous

**Legacy coupling:** Its corpus is defined by a hardcoded `tools/hooks/` path prefix in the producer, plus a buck2 `$(location //tools/hooks:...)` env wiring. `tools/` disappears in the target tree.

**Change required:** Make the hook-corpus root policy DATA (a declared prefix passed alongside `--enforcement-liveness-hooks-dir`) instead of the `tools/hooks/` literal at main.rs:1576/1453, and repoint the BUCK `$(location //tools/hooks:top-level-hook-scripts)` label. Independently, add an empty-corpus fail-closed finding to `hook-wiring/src/lib.rs` (a `min_expected_hooks` floor / `<empty-rows>` code, matching what package-manifest-hygiene and license-policy already do) so a zero-hook corpus can never read GREEN. Without that floor the two substantive codes (`hook_unwired_without_stub_marker`, `hook_wiring_mirror_drift`) go dark silently; only `wired_hook_missing_file` (driven off the wiring files, not the hooks dir) survives.

**Evidence:** Producer corpus filter: `ci/facade/artifact-inventory-registry/src/main.rs:1575-1580` — `fn is_top_level_hook_script(path: &str) -> bool { let Some(name) = path.strip_prefix("tools/hooks/") else { return false }; ... }`; same file:1453 `const HOOKS_DIR: &str = "tools/hooks";` and :1582-1584 `hook_file_name` strips `{HOOKS_DIR}/`. Row emission loops that filter: main.rs:1534-1553. The gate itself has NO empty-corpus guard: `ci/facade/hook-wiring/src/lib.rs:130-141` only REDs when `rows` is missing/non-array; an EMPTY `rows` array falls straight through the loop at :141 and returns an empty finding set → `Report::from_codes` (:66-71) → Verdict::Green. Buck wiring: `ci/facade/hook-wiring/BUCK:34` `"OYA_CI_ENFORCEMENT_LIVENESS_HOOKS_DIR": "$(location //tools/hooks:top-level-hook-scripts)"`. Wired at `.github/workflows/oya-ci-required.yml:170`. Note the producer DOES fail closed if the hooks *dir* is missing (main.rs:1488-1493), but that check passes as soon as CI is repointed at the relocated dir — while the tracked-path filter still looks for `tools/hooks/`, so zero hook rows are emitted and the gate greens.

### package-manifest-hygiene — becomes-vacuous

**Legacy coupling:** Corpus is filtered by the `oya-` brand prefix in the producer — the exact same mechanism as the confirmed `crate-layer-suffix` instance.

**Change required:** Delete the `name.starts_with(prefix)` filter at main.rs:3378 (first-party membership should come from the workspace-member scan, as `collect_license_policy` already does via `scan_valid_member_dirs`, main.rs:956-981), or replace the brand filter with a path/workspace-membership predicate. Additionally add a `min_expected_crates`-style floor to the gate rather than relying on the all-empty `<empty-rows>` guard, so a corpus that silently shrinks from 467 to 5 also REDs.

**Evidence:** `ci/facade/artifact-inventory-registry/src/main.rs:3360-3379`: `fn collect_manifest_hygiene(...)` → `let prefix = cfg.naming.required_prefix.as_str();` (:3365) … `if !name.starts_with(prefix) { continue; }` (:3378); doc comment at :3357 says "Scoped to `oya-*`". `required_prefix = "oya-"` is configured at `oya-ci.toml:27`. The gate crate itself is pure and brand-agnostic (`ci/facade/package-manifest-hygiene/src/lib.rs:95-147`). Partial mitigation: the gate DOES fail closed on a fully empty corpus — `lib.rs:105-108` emits `<empty-rows>` → RED. So the failure profile is: as crates de-brand one by one, they silently drop out of scope and the gate greens on the shrinking remainder; only at 100% de-brand does it RED. It is silent for the entire migration window, i.e. exactly when the checking matters. Wired at `.github/workflows/oya-ci-required.yml:163`.

### topology-manifest-contract — already-vacuous

**Legacy coupling:** Brand-string coupling only: ci/facade/topology-manifest-contract/src/lib.rs:9 declares `GATE_ID = "oya-cloud-ci-cell-topology-manifest-contract"`, asserted verbatim against specs/cell-topology-manifest-contract.json at tests/cell_topology_manifest_contract.rs:60 and :151, and embedded again inside the spec's `manifest_field.validator` string `"cloud-ci Rust gate: oya-cloud-ci-cell-topology-manifest-contract-app"` at tests/...:62-64. The Rust crate itself is ALREADY de-branded (`use ci_topology_manifest_contract::GATE_ID` at tests/...:9), so the `oya-` residue is now purely in the gate-ID string and the spec JSON, which must be edited in lockstep or the test REDs (loudly).

**Change required:** Decide between two, do not leave as-is: (a) REMOVE the CI leg and the marker crate, accepting that the cell-topology manifest contract has no live enforcement; or (b) WIRE IT TO THE LIVE CORPUS — port the dev-cli validator's manifest checks into ci/facade/topology-manifest-contract, drive it off the same capability-root manifest enumeration service-tier-metadata needs, and add an empty-scan floor. Either way the de-brand must rewrite GATE_ID (src/lib.rs:9) together with specs/cell-topology-manifest-contract.json's `cloud_ci_gate` and `manifest_field.validator` values in one change. Confidence is medium only on the label: it does verify spec-vs-fixture agreement, so a reviewer could defend it as a live-but-trivial-corpus gate; the facts underneath (zero live manifests read, real validator CI-dead) are high confidence.

**Evidence:** The gate polices ZERO live repository state. ci/facade/topology-manifest-contract/src/lib.rs is 9 lines — a doc comment plus one `pub const GATE_ID`; its own header admits 'The executable validation contract is pinned by the Rust integration test'. That test is three asserts over committed JSON: (1) manifest_contract_declares_atomic_fields... reads specs/cell-topology-manifest-contract.json and asserts the spec declares its own field names, enums and ADR pointers (tests/...:52-132) — a self-consistency tautology; (2) fixture_satisfies_atomic_contract... validates ONE hand-authored file, specs/fixtures/cell-topology-manifest/tenancy-kr-strict.json, against that same spec (tests/...:134-193); (3) root_hub_registers_contract... asserts a specs/root-hub-pointers.json entry exists (tests/...:195-212). No service manifest is ever opened. Meanwhile 2 LIVE manifests do carry `cell_topology` — cloud/cloud-iac/manifest.json and oya/patient-monitoring/manifest.json — and this gate checks neither. The real 1802-line validator lives in the RETIRED CLI at marketplace/facade/dev-cli/src/cloud_iac_cell_topology_gate.rs (header: '`oya gate validate cloud-iac-cell-topology` runner'), hardcodes DEFAULT_MANIFEST `cloud/cloud-iac/manifest.json`, DEFAULT_TOPOLOGY `cloud/cloud-iac/cell-topology/foundation.json`, DEFAULT_CATALOG `cloud/cloud-iac/tofu/modules/catalog.json` (lines 18-20), and appears NOWHERE in .github/workflows/oya-ci-required.yml — so the only component that actually enforces the contract is CI-dead AND dies outright at the reorg when cloud/cloud-iac/ moves. Net: the wired leg (.github/workflows/oya-ci-required.yml:197) buys a green check for two committed JSON files agreeing with each other. Note this is NOT a false-green created by the reorg — it is vacuous today, hence already-vacuous rather than becomes-vacuous.

## MUST-CHANGE

- **affected-target-set** (wired: true) — POLICY DATA only — the Rust kernel is genuinely repo-neutral, but affected-set-policy.json hardcodes 2 `cloud/cloud-kernel/crates/oya-cloud-kernel-arch-*-adapter/linker.ld` owner-required paths and 4 
- **artifact-inventory-registry** (wired: false) — Deepest coupling in the fleet. Hardcodes `tools/hooks` in Rust; filters two faces by the `oya-` brand prefix; filters the enforcement inventory by the `oya-governance` path substring; and its unit-cla
- **automation-language-policy** (wired: true) — Heaviest POLICY-DATA coupling of the seven. rust-first-automation-policy.json names legacy roots in all three scan dimensions (`scan.roots` includes `tools`/`cloud`/`libs`; `interpreter_command_author
- **caller-supplied-authorization** (wired: true) — scan_roots enumerates cloud/, libs/, oya/, tools/ and OMITS the ADR-0562 meta dirs app/, base/, build/. All 65 frozen baseline keys are full legacy paths under oya/ and libs/.
- **build-cache-policy** (wired: true) — Bundled canary policy pins //cloud/cloud-ci/... (a root that ALREADY does not exist); four Rust path consts anchor on specs/ and infra/, both transitional top-level dirs that ADR-0562 folds into gover
- **contract-slice-conformance** (wired: true) — Slice DATA: 16 of 21 spec_paths point under specs/ and 2 under oya/; embedded content assertions cite oya/finops-portal/*, oya/feature-flags/*, and cloud/managed-k8s-*. The Rust literal //ci/facade/ i
- **core-dependency-isolation** (wired: true) — No legacy root in the enforced policy, but the corpus selector is a crate-NAME suffix glob while ADR-0562 relocates the clean-arch seam to a PATH face. coverage_scope prose names the cloud/cloud-kerne
- **canonical-json** (wired: true) — governed_roots is ["specs"] and the exclusion prefix is "specs/fixtures/"; ADR-0562 folds specs/ into governance/. The oya//cloud/ strings in the policy are prose in a _comment, not enforced data.
- **crate-registration** (wired: false) — Three hardcoded `tools/` paths + buck2 labels, plus doc-level `oya/<app product>` / `cloud/cloud-kernel` home assumptions. `tools/` is one of the four roots that disappears.
- **dependency-automation** (wired: true) — Its policy DATA (root `oya-deps.toml`) names `cloud/` legacy paths in `[rust] drift_guard` and `[rust] exclusions`, and the config filename itself carries the `oya-` brand prefix.
- **embedded-asset-hermeticity** (wired: true) — Its policy `scan_roots` is a hand-maintained list of 9 top-level dirs, 4 of which (`cloud`, `oya`, `libs`, `tools`) disappear — and a vanished root is skipped SILENTLY, with no finding.
- **endpoint-authorization-coverage** (wired: true) — Closed 30-entry `scan_roots` allowlist in policy DATA that names `cloud`, `libs`, `oya`, `tools` as scan roots, plus a frozen baseline whose keys embed legacy file paths (`libs/oya-shared-backbone-res
- **gate-self-conformance** (wired: true) — Its `policy_literal_rules` — the rule that keeps hardcoded repo paths OUT of gate production code — enumerates ONLY legacy roots and the brand: `forbidden_prefixes` and `forbidden_contains` are the di
- **generated-artifact-freshness** (wired: true) — Four hardcoded Rust constants naming the disappearing `tools/` root (a directory path and three buck2 target labels), one nested-workspace skip prefix naming `cloud/cloud-kernel/`, and three `oya-`-pr
- **inventory-registry-drift** (wired: true) — Hard-references `//tools/hooks:top-level-hook-scripts` and `//tools/oya-reorg-codemod-app:oya-reorg-codemod` buck labels, a `tools/hooks` path fallback, and two `oya-`-prefixed cargo package names in 
- **module-membership** (wired: true) — Its whole policy + registry mapping DATA is keyed to the current tree: `scan_roots` and `allowed_top_level_dirs` list `cloud`/`oya`/`libs`/`tools`, and the registry's `absorbs_current_dirs` / `absorbs
- **layer-dependency-acyclicity** (wired: true) — Policy DATA names every legacy root: `crate_root_globs` carry `libs/oya-*`, `cloud/*/crates/oya-*`, `oya/*/crates/oya-*`, `oya/office/oya-*`, `tools/oya-*`; `service_roots` is exactly `["cloud","oya"]
- **operator-secret-rbac** (wired: true) — Every operator/manifest path in its policy JSON is rooted at `cloud/` (`cloud/cloud-iam/...`, `cloud/cloud-k8s/...`), which disappears; several also carry the `oya-` brand in k8s object names.
- **policy-deploy-parity** (wired: true) — policy DATA: `baseline.paths` is 67 hardcoded `oya/<service>/iac/k8s/helm/templates/cedar.yaml` paths — the entire grandfathering surface is keyed to the legacy `oya/` root.
- **port-placement** (wired: true) — baseline DATA: 2 of 6 frozen entries key on `oya/<cap>/crates/oya-<cap>-*` member_paths. Policy `forbidden_crate_name_suffixes` exists specifically to reach the flat `oya/*/crates/oya-*` tree that dis
- **repo-root-hygiene** (wired: true) — policy DATA `allowed_root_dirs` — this IS the `allowed_top_level_dirs` marker — explicitly admits `cloud`, `libs`, `oya`, `tools`. Two root-FILE rules are brand-keyed exact matches on `oya-deps.toml` 
- **resource-contract-conformance** (wired: true) — TWO brand-prefixed string constants compiled into PRODUCTION Rust: a buck2 target name `oya-cloud-ci-cloud-resource-contracts-app-gate` (also in policy DATA x3) and the service names `oya-meter` / `oy
- **service-tier-metadata** (wired: true) — ci/facade/service-tier-metadata/tier-field-coverage-policy.json:11-14 declares `governed_service_roots: ["cloud", "oya"]` — both roots disappear. This is POLICY DATA, and it is the sole corpus selecto
- **service-catalog-parity** (wired: true) — The pure kernel (ci/facade/service-catalog-parity/src/lib.rs) is genuinely shape-agnostic — it names no path at all. The coupling is entirely in POLICY DATA one hop away: the reverse-coverage half's c

## KEEP-AS-IS

- action-item-accounting
- artifact-accountability
- baseline-ratchet
- build-target-parity
- cross-artifact-agreement
- crypto-backend-policy
- dependency-graph-acyclicity
- facade-core-layering
- feature-maturity-policy
- generated-artifact-policy
- graphql-usage-policy
- license-policy
- parity-claim-evidence
- planning-projection
- runner-disk-reclaim
- scm-facts-snapshot
- slo-coverage
- stale-artifact-detection
- supply-chain-audit
- workspace-member-coverage

## Verifier disagreements

- **crate-layer-suffix** → must-change: REFUTED on failure mode. The evidence claim "There is no corpus floor anywhere; the only tripwire is <empty-rows> fail-closed at literally zero rows" is FALSE — it stops reading tests/bnf_layer_suffix.rs at the census helper (:88-116) and misses the gate test's two terminal corpus-liveness assertion
- **generated-artifact-policy** → must-change: REFUTED. The record's load-bearing evidence claim is false. It asserts of registry/generated-artifact-control-plane.json: 'zero occurrences of oya/, cloud/, libs/, tools/ (dumped and inspected in full)'. `grep -n 'libs/\|tools/' registry/generated-artifact-control-plane.json` returns FOUR hits, all 
- **hook-wiring** → must-change: REFUTED. The disposition evaluates ci/facade/hook-wiring/src/lib.rs, but CI runs //ci/facade/hook-wiring:ci-hook-wiring-gate (.github/workflows/oya-ci-required.yml matrix run step, `format('//ci/facade/{0}:ci-{0}-unittest //ci/facade/{0}:ci-{0}-gate')`), whose crate_root is tests/enforcement_livenes
