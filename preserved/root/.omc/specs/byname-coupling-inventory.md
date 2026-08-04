# By-name coupling inventory — couplings no dependency surface carries

**Generated:** 2026-07-27 · 7 detection lenses × adversarial verification (14 agents)
**Scope:** places where behaviour depends on a crate NAME or PATH expressed as a STRING, with no cargo and no buck edge.
**Why it matters:** 467 crates still move. Cargo and buck edges are rewritten mechanically by `oya-reorg-codemod-app`; ADR-0562 §10.6 states it renames "crate names/idents/labels/path-deps only, **never string literals or non-crate target names**". Every hit below is something a move breaks with no compiler to catch it.

## Totals

- **gate-grammar** — 12 hits
- **policy-json-paths** — 16 hits
- **workflow-yaml** — 15 hits
- **binary-and-env** — 15 hits
- **file-literals** — 13 hits
- **fixtures-evidence** — 12 hits
- **docs-adr-prose** — 13 hits

Severity × failure mode:
```
  25 high	silent
  21 medium	loud
  16 medium	silent
  16 critical	silent
  12 high	loud
   4 low	loud
   2 low	silent
```

**41 of 96 fail SILENTLY** — the corpus empties, the count stays above a whole-tree floor, the gate reports GREEN.

## CRITICAL — silent, no gate detects

### ci/facade/endpoint-authorization-coverage/src/lib.rs:471

- **names:** oya-http-router-kernel :: Router::route / .route_service
- **breaks on:** rename
- **kind:** gate anchors ALL surface discovery on the literal call-shape string `".route("` / `".route_service("` of the owned router kernel; no cargo dep, no buck dep (BUCK deps = third-party//:serde_json only)

`for marker in [".route_service(", ".route("]` at src/lib.rs:471 is the ONLY surface-discovery anchor (doc at lib.rs:17-22: "Surface DISCOVERY is anchored on the ROUTE-INTRODUCTION call set"). The method it names is `pub fn route(` at libs/oya-http-router-kernel/src/lib.rs:202. If the base/ move reshapes that method name (mount/handle/add_route), every owned-kernel router produces ZERO surfaces — not UNCLASSIFIED, INVISIBLE — so the gate finds nothing to check and reports GREEN. The only backstop is `min_expected_surfaces: 17` (authz-coverage-policy.json:5), a TOTAL-count floor: 37 non-test .rs files under the scan roots carry `.route(` today, only 6 carry the owned-kernel `route(HttpMethod::` shape, so all 6 can vanish and the floor still passes.

### ci/facade/automation-language-policy/rust-first-automation-policy.json:304

- **names:** oya (top-level root) — scan.roots path list
- **breaks on:** move
- **kind:** gate scope is a hard-coded list of 34 top-level directory-name strings; no buck/cargo edge carries it

scan.roots at :304-338 enumerates 34 roots; of the 31 crate-bearing roots in the tree, `oya` (231 crates) is MISSING, and `base` is absent too. This is a DEMONSTRATED live false-green, not a hypothetical: `oya/feature-flags/reference-implementations/python-asyncio-client.py` is git-tracked, `.py` is in scan.non_rust_extensions ('.go','.js','.mjs','.py','.rb','.sh','.ts'), and it is NOT in `non_rust_exception_baseline` — yet the gate is green because it never walks `oya/`. Every crate the reorg moves into an unenumerated root inherits the same blindness.

### ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-policy.json:5

- **names:** scan_roots = [cloud, oya, libs, tools, infra, marketplace, iam, intelligence, os]
- **breaks on:** move
- **kind:** closed top-level-dir string list that bounds the gate's entire corpus walk

scan_roots (lines 5-15) is a 9-element hardcoded dir-name list with no cargo/buck edge. It is ALREADY STALE from earlier reorg moves: `grep -rl include_str!/include_bytes!` finds 21 Rust files under roots the gate never walks — iac(1), cell(2), network(1), tenancy(3), comms(2), ci(11), kernel(1). Those include sites get zero `embedded_asset_unmapped_include` enforcement and the gate still reports GREEN. `base` and `app` are also absent, so moving oya-http-*-kernel / oya-shared-*-kernel into base/ removes them from the gate entirely. The only floor is `sites_floor: 72` (embedded-asset-hermeticity-baseline.json:8), and the sibling module-membership test states the argument explicitly: 'the min_expected_crates floor is a broken-scan guard, not a coverage guard, and cannot see a partial-root loss' (ci/facade/module-membership/tests/capability_membership.rs:269-271). No test asserts scan_roots covers the crate-owning destination roots.

### ci/facade/endpoint-authorization-coverage/authz-coverage-policy.json:6

- **names:** scan_roots = [audit, billing, cell, ci, cloud, comms, compliance, compute, console, data, flags, gateway, governance, iac, iam, intelligence, k8s, kernel, libs, marketplace, messaging, network, observability, os, oya, secrets, storage, tenancy, tools, workflow]
- **breaks on:** move
- **kind:** closed top-level-dir string list bounding a fail-closed authz gate's discovery surface

scan_roots (lines 6-37) enumerates 30 dir names; `base`, `app`, `build` and `policy` are absent. The gate is otherwise rigorously fail-closed (AC-UNRESOLVED-ROUTE-PATH / AC-UNCLASSIFIED-SURFACE), and its own _comment names `oya-http-router-kernel` as one of the two route grammars it classifies — yet moving that kernel (or any HTTP-surface crate) from libs/ to base/ silently removes it from discovery: a gate that stops recognising its input finds zero violations and reports GREEN. The liveness floor `min_expected_surfaces: 17` (line 5) is a broken-scan guard, not a coverage guard — losing a handful of crates cannot trip it. The sibling module-membership gate has an explicit destination-root proof test (`the_committed_policy_scans_every_crate_owning_destination_root`, tests/capability_membership.rs:264-293); this gate has no equivalent.

### ci/facade/caller-supplied-authorization/dto-authz-trust-policy.json:6

- **names:** scan_roots = [audit, billing, cell, ci, cloud, comms, compliance, compute, console, data, flags, gateway, governance, iac, iam, intelligence, k8s, kernel, libs, marketplace, messaging, network, observability, os, oya, secrets, storage, tenancy, tools, workflow]
- **breaks on:** move
- **kind:** closed top-level-dir string list bounding the caller-supplied-authz (AUTH-005 class-fix) gate

Identical 30-root list (lines 6-37) with no `base`/`app`. oya-shared-platform-contracts-kernel (src/pdp.rs is the repo's PDP contract anchor per specs/regulatory-identity-kyc-policy-evidence-architecture.json:32) sits in libs/ and is slated for base/; once moved, its functions are never scanned for forgeable caller-supplied authorization. The floor is `min_expected_functions: 2000` (line 5) — dropping five crates out of ~697 workspace crates cannot breach it, so DAT-EMPTY-SCAN never fires. No coverage test asserts the root list matches the crate-bearing tree.

### .github/workflows/docs-graph-drift.yml:27

- **names:** tools/oya-architecture-graph-generator-app
- **breaks on:** move-or-rename
- **kind:** GitHub Actions `paths:` trigger filter naming a crate directory as a string prefix (repeated at line 34 for the push trigger)

on.pull_request.paths lists `"tools/oya-architecture-graph-generator-app/**"` (line 27) and on.push.paths repeats it (line 34). `tools/oya-architecture-graph-generator-app/` exists today (BUCK targets verified). The crate carries the `oya-` prefix, so it is in the de-brand move queue. After a move/rename the filter matches nothing: GitHub does not run the workflow and does NOT create a check-run, so nothing turns red. The workflow's own header (lines 17-19) states it is 'Intentionally ABSENT from the branch-protection required set', so no fan-in notices its absence either. The generator's golden + regeneration tests simply stop executing on every future change to the generator. Nothing in ci/facade/ validates workflow path filters: `automation-language-policy` scans .github/workflows only for inline-shell line counts and forbidden `uses:` (ci/facade/automation-language-policy/tests/rust_first_automation_hygiene.rs:261,1296), and `gate_registration.rs:64` reads only `.github/workflows/oya-ci-required.yml`.

### ci/facade/build-cache-policy/src/canary-policy.json:7

- **names:** //cloud/cloud-ci/...
- **breaks on:** move
- **kind:** buck-target-pattern as a string in compiled-in policy JSON (include_str!) naming the whole gate-fleet package cone; consumed as the pinned build set of the ADR-0556 D2 cold integrity canary

`"pinned_targets": ["//cloud/cloud-ci/..."]` at line 7. `cloud/cloud-ci/` DOES NOT EXIST in the tree: `git ls-files 'cloud/cloud-ci/**/BUCK'` = 0, `git ls-files 'cloud/cloud-ci/**/Cargo.toml'` = 0, `ls cloud/cloud-ci` = empty. The gate fleet already moved to `ci/facade/*`. The file is baked into the binary at ci/facade/build-cache-policy/src/lib.rs:664 (`pub const CANARY_POLICY: &str = include_str!("canary-policy.json")`), read at src/main.rs:226-228, and shelled into the cold build at .github/workflows/cache-integrity-canary.yml:88 (`buck2 --isolation-dir canary-cold build @/tmp/canary-targets`). The only assertion over it is ci/facade/build-cache-policy/src/lib.rs:1093 — `assert!(!policy["pinned_targets"].as_array().unwrap().is_empty())` — non-emptiness ONLY, never that the pattern resolves to a live package. This is the trust anchor that licenses warm cache reads fleet-wide.

### ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-policy.json:5

- **names:** base (and iac, cell, tenancy, comms, network, governance, kernel, ci)
- **breaks on:** move
- **kind:** gate-scan-root-allowlist: the ONLY gate that validates include_str!/include_bytes! literals walks a hand-kept 9-entry directory-name list

scan_roots = [cloud, oya, libs, tools, infra, marketplace, iam, intelligence, os] — grep for "base" in the file returns nothing, yet `base` is a DECLARED destination root (ci/facade/module-membership/capability-membership-policy.json:44 lists base in scan_roots, :58 in allowed_top_level_dirs; ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json:36 lists base). The collector at ci/facade/embedded-asset-hermeticity/src/lib.rs:864-867 does `let base = root.join(scan); if !base.exists() { continue; }` — an unlisted root is simply never walked. Policy keys (verified by dump) are: gate_id, schema_version, scan_roots, exclude_path_substrings, rust_extension, embedded_extensions, out_of_scope_path_prefixes, product_contract, purpose — there is NO min_expected_sites floor and NO site-count baseline, so the census can fall and the verdict stays GREEN. The anti-laundering fix EXISTS in this repo but covers only three policies: ci/facade/cross-artifact-agreement/src/registry_policy_sync.rs:127,130,132 enumerates module-membership, repo-root-hygiene and layer-dependency-acyclicity — embedded-asset-hermeticity-policy.json is absent from that list. capability-membership-policy.json:2 states the doctrine verbatim ('SCAN COVERAGE IS DERIVED, NOT HAND-KEPT ... so a destination root can never fall out of lint coverage silently') — embedded-asset-hermeticity is exactly the case that doctrine was written for and does not enjoy it. Present-tense victims already outside the 9 roots: iac/facade/app/tests/cloud_iac_app.rs:80, cell/core/regional-pack/src/lib.rs:125-131, tenancy/adapters/tenant-lifecycle-authz-pdp/src/lib.rs:54, comms/adapters/mail-mailbox-postgres/src/lib.rs:12, network/ports/transport-profile/src/lib.rs:467, kernel/core/asterinas-boundary/src/lib.rs:31, ci/facade/facade-core-layering/src/main.rs:13.

### iam/adapters/pdp-cedar/tests/cedar_pdp_conformance.rs:1497

- **names:** libs/oya-shared-platform-contracts-kernel/cedar/platform.cedarschema
- **breaks on:** move-or-rename
- **kind:** byte-parity drift guard holding the canonical file's repo-relative path (crate dir name = path tail) as a string literal

PAIRS at :1493-1506 hardcodes three canonical paths — :1497 platform.cedarschema, :1501 platform-policies.cedar, :1505 platform-templates.cedar — all under the literal prefix `libs/oya-shared-platform-contracts-kernel/cedar/`. This is one of the five crates slated for base/, and the de-brand makes it a move+rename: BOTH the `libs/` head and the `oya-shared-platform-contracts-kernel` tail change. There IS a cargo edge (iam/adapters/pdp-cedar/Cargo.toml:16) and a buck edge (BUCK:12,31) — but neither carries THIS coupling: the edges link the rlib, the literal is a filesystem path to the crate's data directory, and ADR-0562 §10.6 excludes string literals from the codemod, so the edges get rewritten and the literal does not. Why SILENT rather than a loud panic: repo_root() at :1480-1490 derives from manifest_dir() at :1477, which is `option_env!("CARGO_MANIFEST_DIR")` — undefined under buck2 — and :1507-1514 then `eprintln!(...skipped...); return;` = test PASSES. The comment on :1475-1476 claims 'The cargo lane enforces parity; buck2 skips with a notice', but there is NO cargo lane: `rg -c 'cargo test|cargo nextest' .github/workflows/` returns NONE, and .github/workflows/check-substrates.yml:98 records the 'founder buck2-only directive 2026-05-29'. The buck target iam-pdp-cedar-conformance (BUCK:20-23) is the only thing that runs it, and it takes the skip branch. The guard is a permanent no-op today and stays GREEN after the move.

### iam/facade/cloud-pdp-app/tests/seed_parity.rs:36

- **names:** libs/oya-shared-platform-contracts-kernel/cedar/platform.cedarschema
- **breaks on:** move-or-rename
- **kind:** byte-parity drift guard holding the canonical file's repo-relative path as a string literal

Same shape, second copy: PAIRS at :33-46 hardcodes `libs/oya-shared-platform-contracts-kernel/cedar/platform.cedarschema` (:36), `.../platform-policies.cedar` (:40), `.../platform-templates.cedar` (:44); the assert message at :72 embeds the path a third time ('re-copy from libs/oya-shared-platform-contracts-kernel/cedar/'). Same option_env! skip at :16-18 and the same silent `return` at :47-53. Wired to buck as iam-cloud-pdp-app-seed-parity (iam/facade/cloud-pdp-app/BUCK:163-165), which is the only executor and always takes the skip branch. Confirmed the three copies are byte-identical today (md5 a70aac4c742cdbacac6c2a82a3fca1a2 across libs/oya-shared-platform-contracts-kernel/cedar/, iam/adapters/pdp-cedar/cedar/, iam/facade/cloud-pdp-app/cedar/) — so the duplication is real and the guard protecting it is inert. Cargo edge exists (Cargo.toml:16) but does not carry the path literal, per ADR-0562 §10.6.

### ci/facade/endpoint-authorization-coverage/authz-coverage-policy.json:25

- **names:** libs (scan_roots top-level dir list; `base` absent)
- **breaks on:** move
- **kind:** gate-policy scan-scope: repo top-level dir names as JSON string data, no cargo/buck edge

scan_roots = [audit, billing, cell, ci, cloud, comms, compliance, compute, console, data, flags, gateway, governance, iac, iam, intelligence, k8s, kernel, libs, marketplace, messaging, network, observability, os, oya, secrets, storage, tenancy, tools, workflow] — 30 roots, NO `base`. All five prioritised crates move libs/ -> base/. Once a crate with HTTP routes lands in base/, the engine never walks it: fewer surfaces discovered => fewer AC-UNAUTHENTICATED-CONTROL-PLANE findings => GREEN. The only floor is min_expected_surfaces:17 (line 5), a broken-scan guard that cannot see a partial-root loss (the sibling module-membership gate documents exactly this at ci/facade/module-membership/tests/capability_membership.rs:269-272: 'the min_expected_crates floor is a broken-scan guard, not a coverage guard, and cannot see a partial-root loss'). Contrast ci/facade/module-membership/capability-membership-policy.json:12 which DOES list `base` plus app/kernel/os, added after ci/facade/module-membership/tests/capability_membership.rs:300 found this precise bug ('the ADR-0562 §6 base/-admission rule was VACUOUS while `base` sat outside scan_roots'). The codemod rewrites cargo/buck edges only; ADR-0562 §10.6 leaves this string list untouched.

### ci/facade/caller-supplied-authorization/dto-authz-trust-policy.json:25

- **names:** libs (scan_roots top-level dir list; `base` absent)
- **breaks on:** move
- **kind:** gate-policy scan-scope: repo top-level dir names as JSON string data, no cargo/buck edge

Identical 30-root list to its authz sibling, NO `base`. This is the AUTH-005 forgeable-caller-supplied-authz gate. A crate moved libs/ -> base/ silently exits the corpus; a caller-supplied authorization DTO added in base/ afterwards is never inspected. min_expected_functions:2000 is the only floor and is a broken-scan guard, not a per-root coverage guard. Its frozen baseline does contain one libs/ key (libs/oya-governance-eval-usecase/src/lib.rs#validate_api_binding:6f193126) which WOULD fail loud via DAT-STALE-BASELINE on move — but that is one crate; every other libs/ crate exits silently because a clean crate has no baseline row to go stale. No cargo/buck edge carries this list.

### docs/decisions/ADR-0092-workspace-dependency-seam-policy.md:68

- **names:** crates/oya-http-runtime-hyper-adapter/Cargo.toml
- **breaks on:** move-or-rename
- **kind:** adr-prose-reproducible-verification-command (shell loop + expected-output path, inside a ```bash fence)

ADR-0092 D2 declares the load-bearing hyper-isolation seam and offers it as 'Mechanically verified — empirical seam audit (reproducible)'. Line 68 is `for d in crates/*/Cargo.toml; do`, line 73 is `# Returns exactly: crates/oya-http-runtime-hyper-adapter/Cargo.toml`. `ls -d crates` => 'No such file or directory'; the crate now lives at libs/oya-http-runtime-hyper-adapter. The loop therefore iterates ZERO manifests and prints NOTHING — which reads exactly like the seam being clean. This is the canonical silent-green: the audit stopped recognising its input and now reports 'no violations'. Already broken today; the codemod's rewrite_doc_anchors (tools/oya-reorg-codemod-app/src/plan.rs:766) would have rewritten it had the crates/->libs/ move gone through the codemod, but the glob `crates/*` is not a move old_path so the loop line would be stranded regardless.

### registry/quality/lanes.yaml:86

- **names:** oya-dev-cli
- **breaks on:** rename
- **kind:** quality-lane check_command naming a cargo package as a string (`cargo run -p oya-dev-cli`)

lanes.yaml:86 `check_command: cargo run -p oya-dev-cli -- gate validate aspirational-enforcement` and lanes.yaml:158 the same for `dependency-seam`. The package was renamed: marketplace/facade/dev-cli/Cargo.toml:2 is `name = "marketplace-dev-cli"`. `-p oya-dev-cli` no longer resolves. No cargo or buck edge carries this reference — it is a YAML string. Grep of .github/workflows/*.yml for lanes.yaml / quality-lanes returns nothing, so nothing in the required oya-ci-required matrix runs these lanes and nothing reports their absence. This is the root reason every ADR-prose hit below is ungated: the two gates that could have policed doc-to-crate agreement are already dead by exactly this coupling class.

### libs/oya-check-aspirational-enforcement/src/lib.rs:195

- **names:** oya-check-, oya-governance- (crate-name prefixes)
- **breaks on:** rename
- **kind:** gate tokenizer hardcoding the brand prefix it scans ADR prose for

`for prefix in ["oya-governance-", "oya-check-"]` in enforcement_tokens(). This is the ONLY gate in the repo that resolves crate NAMES out of doc prose against reality (validate_aspirational_enforcement, lib.rs:120 raises MissingCrate when a binding claim names a check crate that is not in known.crate_names). Under the de-brand rename (oya-check-X -> check-X) enforcement_tokens() returns an empty set for every line, `binding_mentions` drops to 0, `violations` is empty, and the gate returns Ok(...) => GREEN with zero claims examined. Compounding: marketplace/facade/dev-cli/src/aspirational_enforcement_gate.rs builds known.crate_names from `--crates-dir` defaulting to `crates`, a directory that no longer exists. The gate is doubly decoupled from the tree it is supposed to police.

### docs/decisions/ADR-0090-hyper-canonical-http-backbone.md:15

- **names:** oya-check-http-stack
- **breaks on:** rename
- **kind:** adr-prose binding enforcement claim ('enforced by `<crate>`') naming a gate crate

'...axum sanctioned where it pays; enforced by `oya-check-http-stack`...' — repeated at ADR-0090:145 as 'codified in specs/http-stack-policy.json, enforced by the `oya-check-http-stack` gate'. `libs/oya-check-http-stack` does not exist. `is_binding_context()` matches 'enforced by' and `is_advisory_context()` does not fire, so this is precisely a MissingCrate violation the aspirational-enforcement gate is built to raise — and it is live-RED-worthy TODAY yet reports nothing, because the lane that invokes it (lanes.yaml:79-86) shells `cargo run -p oya-dev-cli` and is absent from every workflow. Demonstrated proof that this whole class rots undetected.

## HIGH severity

- SILENT · `ci/facade/endpoint-authorization-coverage/authz-coverage-policy.json:6` → base (destination root of the five prioritized crates) — absent from scan_roots (breaks on move)
- LOUD · `ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-policy.json:5` → libs/oya-shared-platform-contracts-kernel (+ 22 unscanned crate-bearing roots) (breaks on move)
- SILENT · `ci/facade/gate-self-conformance/gate-self-conformance-policy.json:4` → ci/facade (gates_root) + 7 gate-crate directory names (breaks on move-or-rename)
- SILENT · `ci/facade/caller-supplied-authorization/dto-authz-trust-policy.json:47` → *Authorization DTO type names + decision field idents (allowed_surfaces / permitted_scopes / caller_ (breaks on rename)
- SILENT · `ci/facade/caller-supplied-authorization/dto-authz-trust-policy.json:6` → base — absent from scan_roots; frozen_dto_authz_trust_instances keyed by `<path>#<fn>:<hash>` (breaks on move)
- SILENT · `ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json:11` → crate_root_globs = [libs/oya-*, cloud/*/crates/oya-*, oya/*/crates/oya-*, oya/office/oya-*, tools/oy (breaks on move-or-rename)
- SILENT · `ci/facade/automation-language-policy/rust-first-automation-policy.json:287` → interpreter_command_authority.roots = [cloud/cloud-ci, ci, os, libs, tools] (breaks on move)
- SILENT · `ci/facade/automation-language-policy/rust-first-automation-policy.json:304` → roots = [scripts, tools, bin, infra, .codex, .github/workflows, cloud, os, audit, billing, cell, ci, (breaks on move)
- SILENT · `oya-ci.toml:156` → governance_crate_substr = "oya-governance" (breaks on rename)
- LOUD · `specs/capability-registry.json:590` → libs/oya-http-middleware-kernel, libs/oya-http-router-kernel (:591), libs/oya-http-runtime-hyper-ada (breaks on move-or-rename)
- LOUD · `ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-baseline.json:47` → libs/oya-shared-platform-contracts-kernel/tests/cedar_policy_validation.rs:18 (also :48 line 19, :49 (breaks on move-or-rename)
- LOUD · `registry/catalog/oya-http-router-kernel.yaml:1` → catalog record filename = crate name (also oya-http-middleware-kernel.yaml, oya-http-runtime-hyper-a (breaks on rename)
- SILENT · `.github/workflows/cache-integrity-canary.yml:73` → //ci/facade/build-cache-policy:oya-cloud-ci-cache-wiring-bin (breaks on move-or-rename)
- SILENT · `.github/workflows/docs-graph-drift.yml:84` → root//tools/oya-architecture-graph-generator-app:oya-architecture-graph-generator (breaks on move-or-rename)
- LOUD · `.github/workflows/oya-ci-required.yml:201` → //libs/oya-workspace-members-kernel:oya-workspace-members-kernel-cargo-differential (breaks on move-or-rename)
- LOUD · `.github/workflows/oya-ci-required.yml:1087` → //libs/oya-data-sql-adapter-sqlx:oya-data-sql-adapter-sqlx-unittest (breaks on move-or-rename)
- SILENT · `oya/consent-graph/iac/helm/consent-graph/templates/revocation-and-workers.yaml:19` → oya-consent-graph-revocation-app (also -projection-gateway-worker :47, -audit-bridge-worker :72) (breaks on move-or-rename)
- SILENT · `cloud/cloud-kms/iac/k8s/helm/values.yaml:29` → oya-cloud-kms-operator-app (breaks on move-or-rename)
- SILENT · `cloud/cloud-iam/iac/k8s/helm/values.yaml:3` → oya-cloud-iam-app (breaks on move-or-rename)
- SILENT · `cloud/cloud-iam/iac/k8s/helm/templates/svid-operator-deployment.yaml:60` → oya-cloud-iam-pdp-svid-operator (breaks on rename)
- SILENT · `cloud/cloud-kms/iac/k8s/helm/templates/operator-deployment.yaml:46` → oya-cloud-kms-operator (breaks on rename)
- SILENT · `.github/workflows/cache-integrity-canary.yml:73` → //ci/facade/build-cache-policy:oya-cloud-ci-cache-wiring-bin (breaks on move-or-rename)
- SILENT · `libs/oya-shared-backbone-proto-contracts-kernel/tests/proto_specs_parity.rs:20` → specs/proto/backbone/*.proto (breaks on move)
- LOUD · `iac/facade/app/tests/cloud_iac_app.rs:80` → iac/tofu/modules/release-index.json (breaks on dir-split)
- LOUD · `intelligence/adapters/authz-cedar-adapter/src/lib.rs:37` → cloud/cloud-intelligence/policy/cloud-intelligence.cedar (breaks on move)
- LOUD · `oya/ci-webhook-gateway/crates/oya-ci-webhook-gateway-authz-cedar-adapter/src/lib.rs:36` → oya/ci-webhook-gateway/policy/ci-webhook-gateway.cedar (breaks on dir-split)
- LOUD · `intelligence/core/openapi-domain/src/lib.rs:6126` → contracts/openapi/foundry/capability-v1.yaml (breaks on move)
- LOUD · `iam/facade/identity-workload-rest/build.rs:29` → oya/identity/contracts/proto/workload.proto (breaks on move)
- LOUD · `libs/oya-shared-backbone-grpc-generated-adapter/build.rs:17` → specs/proto/backbone (breaks on move)
- SILENT · `specs/fixtures/staleness-reaper/tc-SR-bad-untyped-resource.json:16` → cloud/cloud-ci/gates/oya-cloud-ci-staleness-reaper-app/tests/staleness_reaper.rs (breaks on move)
- SILENT · `registry/artifact-capabilities-registry.json:701` → libs/oya-http-router-kernel/Cargo.toml (breaks on move-or-rename)
- SILENT · `registry/dependency-rationales.json:47` → oya-http-runtime-hyper-adapter (breaks on rename)
- SILENT · `ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-policy.json:8` → libs (scan_roots top-level dir list; `base` absent) (breaks on move)
- SILENT · `docs/decisions/ADR-0562-capability-first-repo-organization-and-closed-capability-registry.md:1286` → libs/oya-http-{router,middleware}-kernel (breaks on move-or-rename)
- SILENT · `docs/decisions/ADR-0092-workspace-dependency-seam-policy.md:61` → oya-http-runtime-hyper-adapter (breaks on rename)
- SILENT · `docs/decisions/ADR-0092-workspace-dependency-seam-policy.md:115` → oya-http-router-kernel (breaks on rename)
- SILENT · `docs/decisions/ADR-0569-commission-oya-data-outbox-cdc-adapter-postgres.md:82` → oya-shared-postgres-command-kernel (breaks on rename)

## MEDIUM / LOW

- medium loud · `.github/workflows/oya-ci-required.yml:232` → //ci/facade/<crate>:ci-<crate>-unittest and :ci-<crate>-gate (every gate crate i
- medium loud · `ci/facade/endpoint-authorization-coverage/src/lib.rs:1008` → oya-http-runtime-hyper-adapter :: handler_to_sync
- medium loud · `ci/facade/endpoint-authorization-coverage/src/lib.rs:1725` → oya-http-middleware-kernel :: trait Handler
- medium loud · `ci/facade/endpoint-authorization-coverage/src/lib.rs:967` → oya-http-router-kernel :: enum HttpMethod
- medium loud · `ci/facade/facade-core-layering/facade-core-layering-policy.json:27` → 35 cargo package names (billing-service, iam-tenant-rbac-app, console-workspace-
- medium silent · `ci/facade/service-tier-metadata/tier-field-coverage-policy.json:11` → governed_service_roots = [cloud, oya]
- medium loud · `ci/facade/endpoint-authorization-coverage/authz-coverage-policy.json:98` → libs/oya-shared-backbone-rest-runtime-adapter/src/lib.rs#build_backbone_rest_rou
- medium loud · `ci/facade/caller-supplied-authorization/dto-authz-trust-policy.json:122` → libs/oya-governance-eval-usecase/src/lib.rs#validate_api_binding:6f193126
- medium loud · `oya-ci.toml:56` → path_prefix carve-outs: libs/oya-check-brand-residue/ (:56), libs/oya-ci-config/
- medium loud · `oya-ci.toml:31` → doctrinal_carve_outs = ["oya-tooling-agent-read", "oya-ci-gate-contract"]
- medium loud · `ci/facade/gate-self-conformance/gate-self-conformance-policy.json:41` → allowed_paths = [ci/facade/affected-target-set/src/main.rs, ...] (also :57, :71,
- medium silent · `.github/workflows/oya-ci-required.yml:112` → ci/facade/artifact-inventory-registry/*.generated.json (+ ci/facade/scm-facts-sn
- medium silent · `.github/workflows/oya-ci-required.yml:819` → ci/facade/scm-facts-snapshot/history-only-retirement-facts.generated.json
- medium loud · `.github/workflows/oya-ci-required.yml:104` → //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-face
- medium loud · `.github/workflows/check-substrates.yml:75` → //ci/facade/dependency-graph-acyclicity:ci-dependency-graph-acyclicity
- medium loud · `.github/workflows/oya-ci-required.yml:1219` → //iam/facade/identity-service:iam-identity-service-tests
- medium loud · `.github/workflows/oya-ci-required.yml:1020` → tenancy/adapters/tenant-lifecycle-store-postgres/migrations/0000_runtime_role.sq
- medium loud · `.github/workflows/oya-ci-required.yml:514` → //ci/facade/runner-disk-reclaim:oya-cloud-ci-runner-disk-reclaim-bin
- medium loud · `.github/workflows/oya-ci-required.yml:777` → ci/facade/affected-target-set/affected-set-policy.json
- medium loud · `.github/workflows/oya-ci-required.yml:811` → oya-cloud-ci-materialize-generated-faces
- low loud · `.github/workflows/oya-ci-required.yml:158` → ci/facade/<crate> matrix values (cross-artifact-agreement … scm-facts-snapshot, 
- medium silent · `.github/workflows/check-substrates.yml:76` → //ci/facade/dependency-graph-acyclicity:oya-cloud-ci-substrate-dependency-dag-ac
- medium silent · `tools/oya-fabric-loop-state-app/src/lib.rs:53` → cloud/cloud-ci/crates/oya-cloud-ci-loop-state-app
- medium silent · `secrets/facade/kms-operator-app/src/main.rs:70` → /etc/oya-cloud-kms-operator/tls
- medium silent · `Dockerfile.distroless:16` → oya-intelligence-runtime
- low silent · `oya/application/catalog/oya-application-shell-frontend.yaml:8` → libs/oya-shared-platform-contracts-kernel/src/shell_bff.rs
- medium loud · `.github/workflows/oya-ci-required.yml:811` → usage: oya-cloud-ci-materialize-generated-faces
- medium loud · `.github/workflows/oya-ci-required.yml:298` → //ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-face
- medium loud · `.github/workflows/oya-ci-required.yml:514` → //ci/facade/runner-disk-reclaim:oya-cloud-ci-runner-disk-reclaim-bin
- medium silent · `libs/oya-shared-platform-contracts-kernel/tests/cedar_policy_validation.rs:18` → libs/oya-shared-platform-contracts-kernel/cedar/platform.cedarschema
- medium loud · `libs/oya-check-dependency-seam/src/lib.rs:658` → registry/dependency-rationales.json (and the crate name string "oya-application-
- medium silent · `cloud/cloud-kernel/crates/oya-cloud-kernel-arch-aarch64-adapter/src/user.rs:80` → cloud/cloud-kernel/out/user-smpdemo.elf
- medium silent · `registry/stores/registry-store.json:4702` → oya-http-router-kernel
- medium silent · `registry/graph/architecture-map.json:77` → oya-application-branding-adapter
- medium silent · `specs/fixtures/crate-adr-design-doc-coverage/tc-CRATEADR-002B-good-ci-control-plane-owner-batch.json:108` → cloud/cloud-ci/gates/oya-cloud-ci-enforcement-liveness-app/Cargo.toml
- low loud · `ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-baseline.json:47` → libs/oya-shared-platform-contracts-kernel/tests/cedar_policy_validation.rs:18
- low loud · `specs/capability-registry.json:590` → libs/oya-http-middleware-kernel (and 589,591,592,623,625 for the other four)
- low loud · `registry/catalog/oya-http-router-kernel.yaml:1` → oya-http-router-kernel (the file STEM is the crate id)
- medium silent · `docs/decisions/ADR-0090-hyper-canonical-http-backbone.md:60` → oya-http-{cedar,tenant,telemetry,deadline}-middleware-domain / oya-http-sse-doma
- medium silent · `docs/decisions/ADR-0094-handler-trait-with-associated-error.md:34` → oya-http-middleware-kernel
- medium silent · `docs/decisions/ADR-0593-fail-closed-authz-for-accounting-payroll-money-mutation-control-plane.md:115` → oya-http-middleware-kernel
- medium silent · `docs/decisions/ADR-0154-event-schema-versioning.md:34` → oya-check-event-schema-versioning
- low silent · `docs/decisions/ADR-0566-authz-coverage-gate.md:100` → oya-http-router-kernel

## Verifier refutations (not counted above)

- `.github/workflows/cache-integrity-canary.yml:73`: REFUTED on criterion 4 (fails_loud_or_silent is wrong). The label occurrences are real and correctly located (73, 80, 93, 97, 102, 122, 124, 130 plus the materializer at 67), the workflow is cron-only (30-35), and it is outside branch protection — but every ci
- `secrets/facade/kms-operator-app/src/main.rs:70`: REFUTED — the line exists but the coupling does not. Line 70 does hold `"/etc/oya-cloud-kms-operator/tls"` (the env_or call spans 68-71), and it is the only occurrence in the repo. But the claimed Rust-default <-> Helm-mount join is absent: operator-deployment
- `iac/facade/app/tests/cloud_iac_app.rs:80`: REFUTED on both criterion 1 and criterion 2. (1) referenced_name is a path that does not exist: `find . -name release-index.json` returns exactly one hit, ./cloud/cloud-iac/tofu/modules/release-index.json — there is no iac/tofu/ directory at all, so 'a move th
