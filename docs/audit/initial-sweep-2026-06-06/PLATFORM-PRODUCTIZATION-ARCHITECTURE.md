# PLATFORM PRODUCTIZATION ARCHITECTURE (capstone)

> **STATUS: pending-approval** · **DOOR: one-way** (founder sign-off required before any mutation or any ADR/masterplan write this doc proposes)
> **AUTHORED: 2026-06-08**
> **Authored against:** `source` @ `cleanup/whole-tree-2026-06-07` (READ-ONLY; a background hermetic-build verifier is BUILDING in that tree — this doc mutated NOTHING there, wrote no files except this one, ran no git add/commit/push).
> **Scope:** DESIGN/SYNTHESIS ONLY. This is the capstone that integrates the four facet fragments (A product-taxonomy, B third-party-adoption, C extensibility/SDK/marketplace, D distribution/versioning/governance-automation) into ONE coherent product-line architecture, buildable against the real buck2 + reindeer + oya-ci substrate that exists today. The only write is this doc.

---

## §0. The founder apex directive and the acceptance bar

> "Productize everything to canonical. Any other project, or any other person, should be able to utilize our tools, ci, cd, build, pipeline, etc."

**ACCEPTANCE TEST (the bar the whole design must satisfy):** an EXTERNAL project/person adopts our build + CI + CD + pipeline + dev-env + tools in THEIR repo, via *config + our products as substrate*, **WITHOUT forking oyatie**. The class to match is **GitHub Actions + Bazel**: you reference a published, versioned product (a `uses:` line, a `MODULE.bazel` dep, a pinned binary) and supply only your own config — you never vendor the vendor's monorepo.

**The single load-bearing insight (verified, not aspirational):** the cleanest existing seam in the tree — `oya-ci-config` — ALREADY proves the model. `libs/oya-ci-config/src/lib.rs` is a closed-schema (`#[serde(deny_unknown_fields)]`, verified at lib.rs:73 + 16 more sites), I/O-free TOML loader; its own test `partial_toml_overrides_only_the_named_section` (lib.rs:798) constructs a config with `required_prefix = "acme-"` and asserts it parses — i.e. **a non-oyatie repo's policy already works through the engine**. Productization is the act of generalizing that one proven seam to ALL six-plus products: **engine generic + behavior-as-DATA + oyatie is merely the first adopter of its own config.**

This doc does NOT redesign the technical substrate. The hermetic/zero-shell build model and the VCS-agnostic facts seam are GIVENS, designed in siblings (§6). This doc designs the **PRODUCT / ADOPTION layer ON TOP** of them.

---

## §1. THE PRODUCT LINE — seven canonical products

The unifying spine: **every product is a GENERIC ENGINE whose behavior is pure DATA in a repo-root config**, dogfooded by oyatie via its own config and adoptable by an external repo via THEIR config. Three independently-versioned things compose: **ENGINE (versioned binary/crate) + POLICY (the repo's own config) + RUNNER WRAPPER (action/ruleset)**. An adopter brings only POLICY + a one-line RUNNER reference.

Canonical names below are the *proposed* product identities; the de-oyatie rename set that gets them there is §3, and ratification is gated in §7 (the masterplan-is-generated-from-ADRs rule means names must land as ADRs before any source rename).

### Product 1 — `oya-build` (BUILD)
- **Boundary:** the hermetic build graph + third-party vendoring + toolchain provisioning.
- **Real source state TODAY:** `.buckconfig` (cells: `root`/`prelude`(bundled `external_cell`, verified `.buckconfig:18-19`)/`toolchains`/`third-party`); `reindeer.toml` buckifies Cargo deps into `third-party/`; `toolchains/BUCK` pins system rust/cxx/genrule/python/test toolchains to `rust-toolchain.toml` (1.95.0). The git-facts emitter (`oya-cloud-ci-git-facts-emitter-app`) is the SINGLE out-of-graph VCS boundary so every buck2 action is hermetic. The buck2 RE seam is WIRED (`toolchains/BUCK:5` loads `remote_test_execution_toolchain`) but INERT (`:test` is `noop_test_toolchain`, verified `toolchains/BUCK:6`).
- **Productization gap (verified):** `toolchains/BUCK:24-41` hardcodes host-absolute `/usr/bin/clang` + `/usr/bin/ar` with the self-admitted comment "Hermetic step later: absolute host paths -> cell artifacts" (`toolchains/BUCK:21`); `platforms/BUCK` is darwin-aarch64 host-only; NativeLink RE + hermetic `download_toolchain` are not landed. There is no packaged "adopt-our-build" surface — an external repo would copy these files, not consume a versioned product, and reindeer is invoked as a raw external tool, not wrapped.
- **Product surface:** a versioned buck2 cell/prelude overlay + a reindeer wrapper + portable `toolchains`/`platforms` definitions an external repo **loads, not copies**. Plus a Bazel ruleset (`rules_oya_ci`) that wraps the same engine binaries (§4) so a Bazel shop has an on-ramp (the acceptance class names Bazel explicitly).

### Product 2 — `oya-ci-floor` (CI / FIREWALL — the conformance engine)
- **Boundary:** producer → faces → shrink-only ratchet → registry-drift → gates. **This is already engine/policy-clean and is the most product-ready thing in the tree.**
- **Real source state TODAY:** the deterministic producer `oya-cloud-ci-accounting-registry-app` (reads `oya-ci.toml` via the config kernel, consumes the committed `git-facts.generated.json` face instead of shelling git, emits the accounting faces); the shrink-only ratchet `oya-cloud-ci-firewall-app` whose predicates are ALREADY fully generic over `(gate, code, key)` (verified `firewall-app/src/lib.rs`: `regressions = current \ baseline`, `FAIL = baseline-block-on-new && !regressions.is_empty()`, the `_sign_off_additions` one-way door at lib.rs:91-102); the byte-diff `registry-drift`; 9 gate crate dirs under `cloud/cloud-ci/gates/` (`total-accounting`, `cross-artifact-agreement`, `automation-ratchet`, `staleness-reaper`, `manifest-hygiene`, `cargo-prefix`, `bnf-layer-suffix`, plus the producer + emitter which are NON-gate) each a pure `evaluate_keyed`; the raw-corpus census `libs/oya-check-brand-residue`. POLICY = `oya-ci-config` (closed-schema loader; `bundled_default()` reproduces oyatie's policy byte-for-byte, verified by tests `bundled_default_matches_todays_naming_consts` lib.rs:699 + `...vocab...` lib.rs:715). Adoption docs exist (`docs/oya-ci/{README,quick-start,config-reference,gate-catalog,firewall-model}.md`).
- **Productization gap (verified):** the README scopes OUT exactly what productization needs ("third-party gate SDK, cloud control plane, hermetic build backends, non-Rust gate packs … a separate, later workstream … NOT documented here"). All gates are Cargo-shaped; there is no gate-pack SDK (§4); every crate is `publish = false` (verified: **173** `publish = false` declarations across `libs/` + `cloud/cloud-ci/gates/`) consumed via workspace path deps — no published artifact an external repo can depend on.
- **Product surface:** the closed-schema `oya-ci.toml` + the published producer binary (`--repo-root .`) + the published config-schema crate + the committed faces + the per-repo sign-off door + the **gate-pack SDK** (§4, the missing piece).

### Product 3 — `oya-checks` + `oya-sdk` (TOOLS / SDK — the reusable library tier)
- **Boundary:** the reusable check kernels (`libs/oya-check-*`), governance enforcers (`tools/oya-governance-*`), and platform kernels (`libs/oya-shared-*`). This is the substance the CI/Pipeline products orchestrate.
- **Real source state TODAY:** ~90 `oya-check-*` discipline kernels (a11y, license-policy, perf-budget, otel-trace-propagation, slsa-l3, …); ~18 `oya-governance-*` status/shape/naming enforcers (the `oya-governance-predictable-naming-kernel` + `oya-governance-gate-catalog-domain` are reused BY the cloud-ci gates); ~40 `oya-shared-*` platform kernels (http-runtime, postgres-adapter, outbox-pattern, oidc-client); the 883-entry `registry/catalog/*.yaml` per-crate catalog.
- **Productization gap (verified):** all `publish = false`, workspace-path-only; no `cargo add`-able SDK; the check libs ARE the gate logic but reachable only through oyatie's own producer or the retiring dev-cli. **The single deepest SDK gap is verified:** there is **NO `trait Gate`** anywhere (`grep "trait Gate"` in `cloud/cloud-ci/gates/` + `libs/` returns empty), and the gate CONTRACT (`Finding{code,key}`, `Report`, `Verdict`, `evaluate_keyed`) is **COPIED into 7 separate gate crates** (verified: 7 `struct Finding` definitions across `cloud/cloud-ci/gates/`), not a published shared type.
- **Product surface:** published, versioned crates + a check-authoring contract (the `Finding`/`evaluate_keyed` shape extracted ONCE into `oya-pipeline-gate-sdk`, §4) + the `registry/catalog` schema as the public registration format.

### Product 4 — `oya-pipeline` (PIPELINE / ORCHESTRATION — "one logic, many runners")
- **Boundary:** takes the gate set (from P2/P3) and EXECUTES it — on GitHub Actions today, on the bespoke-Rust Prow tomorrow, identically.
- **Real source state TODAY:** ONE GitHub Actions workflow `.github/workflows/oya-ci-required.yml` — the ONE canonical blocking status check (verified: a gate matrix + producer-regen + registry-drift + firewall + a hermetic `buck2` lane all fanning IN to a zero-command verdict job `oya-ci-required`, branch protection keys on that one context). The workflow states the design law verbatim ("one logic, two runners", `oya-ci-required.yml:77`, D-CICD-AUTHORITY) and documents the critical caveat (a `workflow_call` reusable workflow renames check-runs to `<caller>/<job>`, breaking the required-context name, `oya-ci-required.yml:74-75`). NORTH-STAR (designed, partly built): the bespoke-Rust Prow — `oya/ci-controller/`, `oya/ci-tide/`, `oya/ci-webhook-gateway/` (webhook-gateway is the most mature: full crate set + Dockerfile + cedar + SLOs; controller/tide are kernel-RED/Phase-1). The masterplan home is `specs/bespoke-cloud-toolchain-services.json#cloud-ci` (verified: `oya-ci-api/controller/admission/evidence/tide/tenant-runner`; `not_internal_only: true`).
- **Productization gap (verified):** the workflow is hand-authored, oyatie-specific YAML (literal gate crate names in the matrix, literal `//cloud/cloud-ci/...` buck2 target, literal `oya-ci-required` fan-in name). The quick-start PROMISES "ship a composite action (uses:-able) plus a copy-in matrix template" for external repos, but **no composite action exists** (verified: the only `action.yml` in the tree is a vendored third-party artifact under `buck-out/`). The completeness invariant `gate_registration.rs` GREPS the single hardcoded GHA YAML path + a hardcoded gates DIRECTORY + a hardcoded `NON_GATE_CRATES: [&str; 2]` (verified `firewall-app/tests/gate_registration.rs:28,54`) — GitHub-Actions-specific, not runner-agnostic.
- **Product surface:** a runner-agnostic job spec + a published composite GitHub Action + a copy-in matrix template (external GHA) + a self-hosted CRD (the `OyaCIJob`-class admission contract, masterplan `cloud-ci`). The fan-in "one canonical required check" is the public contract — parameterized to the adopter's OWN context name.

### Product 5 — `oya-cd` (CD / RELEASE — GitOps + progressive delivery)
- **Boundary:** from a green pipeline verdict + a release manifest to deployed, progressively-rolled state.
- **Real source state TODAY:** Argo CD GitOps is the CD engine (`infra/gitops/`: root-app/bootstrap-sync/vcs-argocd-app); Cluster API + Talos own the fleet; OpenTofu owns the Cloudflare edge (`Makefile` bootstrap/plan/apply). `cloud/cloud-iac` is a hexagonal crate set; `oya-dev-cli` embeds cloud-iac module-release-index / provenance / opentofu-validation gates. A rich progressive-delivery spec corpus exists (`docs/advanced-cicd/progressive-delivery/`: blue-green, canary-rail, dark-launch, traffic-mirror, slo-burn-rate-rollback, stable-cohort). `libs/oya-check-release-pack` gates release artifacts. The masterplan home is `specs/bespoke-cloud-toolchain-services.json#cloud-cd` (verified: `oya-cd-release-ledger`, `oya-cd-rollout`, `oya-cd-policy`, `oya-cd-tenant-isolation-gate`; cutover_rule: "Argo remains the bridge/reference … the permanent product contract is oya-cd").
- **Productization gap (verified):** CD is mostly DOCS + infra YAML + IaC gates — progressive-delivery is spec/playbook PROSE with **no runner crate carrying it out**; `oya-cd-release-ledger`/`oya-cd-rollout` are masterplan PACKAGE PLANS (T2 MVP), not implemented. The deploy entrypoint is a `Makefile` coupled to oyatie's own infra (Cloudflare/Talos/CAPI/ops.oyatie.com).
- **Product surface:** an Argo CD application template + a config-driven progressive-delivery runner (today's spec corpus turned executable: canary/blue-green/traffic-mirror/slo-burn-rollback as DATA) + the cloud-iac module-release/provenance gates, all decoupled from oyatie's own Cloudflare/Talos infra ("point at YOUR cluster + YOUR manifest").

### Product 6 — `oya-dev` (DEV-ENV — the local mirror + adoption on-ramp)
- **Boundary:** the local mirror of the cloud lifecycle — provision the pinned toolchain, run the SAME gates locally pre-push, scaffold a conformant repo.
- **Real source state TODAY:** `rust-toolchain.toml` pins the channel (zero CI drift, per workflow comment); `.envrc`/`.config` for direnv; a `Makefile` of human entrypoints; `scripts/`. The de-facto local surface is `oya-dev-cli` (`oya/developer-sdk/crates/oya-dev-cli`, default-run `oya`: `oya check`/`oya gen`/`oya gate` wired to ~80 check/governance crates).
- **Productization gap (verified):** `oya-dev-cli` is the RETIRING governance CLI (task #26 + ADRs call it "the retiring oya CLI"). There is **no positive dev-env product** — searches for an `oya-dev`/`oya new`/`oya init`/scaffolder positive product returned only the deprecated cli. The local pre-push loop that mirrors the cloud gates has no successor.
- **Product surface:** a positive `oya-dev` tool (NOT the retiring `oya` CLI) that runs the P2 producer + P1 build locally, plus an `oya-dev init` scaffolder that drops a conformant config set (`oya-ci.toml` + the runner wrapper + a buck cell overlay) + the first empty baseline. This is THE adoption on-ramp.

### Product 7 — `oya-govbot` (REPO-AUTOMATION — the repo-automation-bots / Dependabot-equivalent)
- **Boundary:** keep an adopted repo healthy: dependency bumps, version/release governance, auto-changelog, EOL/sunset countdowns, branch-protection drift.
- **Real source state TODAY:** the 3-axis versioning strategy doc (`docs/advanced-cicd/release-versioning/release-versioning-strategy.md`, Accepted Tier-1: crate SemVer / product `oya-vX.Y.Z` / external-API hybrid date+stability) names 6 enforcement lanes + 2 ledgers but they are UNBUILT; `libs/oya-shared-semver-check-cli` is a literal SCAFFOLD stub (verified `main.rs:26` prints "semver-check: SCAFFOLD (populated in Shard 1)"); the strict-dep policy `docs/standards/dependency-policy.md` + a real live `deny.toml`; `ADR-0345` (OSS stewardship classes + CVE SLAs) with `specs/oss-stewardship-registry.json` + 7 check crates.
- **Productization gap (verified):** `renovate.json` + `.github/dependabot.yml` are ABSENT (verified: only `deny.toml` exists at root) — the baseline `renovate.json` lives ONLY as a fenced block in the policy doc, so the bump-bot is documented-not-deployed; `ADR-0345/0041/0342/0037` are ALL `Proposed` (verified) despite being load-bearing; `dependency-policy.md §8` is STALE ("Bazel/Buck2 not adopted … GitHub Actions default") contradicting the landed buck2 substrate.
- **Product surface:** a closed-schema `oya-deps.toml` (LTS roster, license allow/deny as DATA, supply-chain triad, the ADR-0345 stewardship registry shape) driving an in-house Rust bump-bot that opens **scm-facts ChangeSets** (provider-neutral, not GitHub PRs) — VCS-agnostic — plus the 6 release-governance gate crates wired into the P2 ratchet, plus the auto-changelog + EOL/sunset countdown + branch-protection-drift bots.

### §1.x — How they COMPOSE (the load-bearing wiring)

```
P3 oya-checks/oya-sdk  ── supplies logic ─────────────────────────┐
                                                                  ▼
P2 oya-ci-floor        ── freezes logic into a config-driven shrink-only ratchet (faces+baseline)
                                                                  │
P1 oya-build           ── compiles/runs the gates hermetically (buck2 sandbox, scm-facts boundary)
                                                                  │
P4 oya-pipeline        ── executes the SAME logic on ANY runner (GHA today / bespoke-Prow next)
                                                                  │  green verdict
                                                                  ▼
P5 oya-cd              ── deploys + progressively rolls on a green verdict (release manifest as DATA)
P7 oya-govbot          ── keeps the adopted repo healthy (deps/versioning/changelog/sunset)
P6 oya-dev             ── mirrors P1+P2 locally + scaffolds adoption of ALL of them
```

The single seam that makes this a PRODUCT LINE and not a monorepo: **every product's behavior is DATA** (`oya-ci.toml`, `.buckconfig` cells, the pipeline job spec, the release manifest, `oya-deps.toml`), the engine is generic and versioned, and **oyatie is just the first adopter of its own config.** P2's engine/policy seam is the proven template; productization is generalizing it across P1/P4/P5/P6/P7.

> **Scope boundary (open question carried up):** the application portfolio (`oya/` — CRM/HR/EMR/office/~70 verticals) is the DOGFOOD/customer-facing SaaS, distinct from these LIFECYCLE-TOOLING products. It is OUT of the product line by default. Whether any of it (the SolidJS UI shell, an `oya-ci-deck`) becomes a productized component is OQ-1 (§9/§openQuestions).

---

## §2. EXTERNAL-ADOPTION ACCEPTANCE TEST — Project Foo, end-to-end, N steps, no fork

**Project Foo** = `acme/widget`, an external Rust(-or-other) shop on GitHub (or a non-git VCS), NOT a fork of oyatie. The narrative generalizes the proven `oya-ci` floor adoption (`docs/oya-ci/quick-start.md`) across all products. The bar is met IFF every step is **config + a pinned product reference**, never a clone of oyatie's tree.

1. **BUILD (P1).** Foo adds a `.buckconfig` that **loads** `oya-build`'s published cell/prelude overlay + a toolchain pin (consume, not copy), and runs `reindeer buckify` via the `oya-build` wrapper to vendor THEIR own `Cargo.lock` into THEIR `third-party/`. Foo's crates build hermetically with no oyatie source present. A Bazel-shop Foo instead adds `rules_oya_ci` to `MODULE.bazel` and gets the gate targets as `load("@oya_ci//:defs.bzl", …)` — the rules wrap the SAME engine binaries.

2. **TOOLS/SDK (P3).** Foo `cargo add`s the published `oya-checks`/`oya-sdk` crates it wants, OR authors its own gate against the public `oya-pipeline-gate-sdk` `trait Gate` (§4) — without touching oyatie source.

3. **CI/FIREWALL (P2).** Foo drops an `oya-ci.toml` at its repo root, starting from `profile = "neutral"` (§3). Foo sets `[naming].required_prefix = "widget-"` (or omits `[naming]` entirely — already proven parseable by the `acme-` test at `oya-ci-config/src/lib.rs:798`), declares its OWN `[vocab].forbidden_stems` (or none), points `[reachability]`/`[justification]` at ITS doc tree (or leaves them empty to disable those gates), sets `[repo].root_markers` to a marker that EXISTS in its repo (e.g. `.git` or `WORKSPACE`), and sets `[output].faces_dir = ".oya-ci/faces"` (§3 NET-NEW key). The closed schema rejects typos LOUDLY. Foo runs the **published producer binary** (`oya-ci-producer --repo-root .`) — not `cargo run` against oyatie's workspace — to emit + commit its own faces + baseline at its current debt. The two-commit new-file settle (already documented) converges committed==regenerated.

4. **PIPELINE (P4).** Foo adds a ~6-line workflow that `uses: oyatie/oya-ci-action@v1` and keeps its OWN fan-in job named `widget-ci-required`, keying branch protection on a name the composite action does NOT rename (respecting the verified `workflow_call` check-run-rename caveat). A self-hosting Foo instead applies the `OyaCIJob` CRD to its cluster — SAME gate logic, different runner ("one logic, many runners", realized for outsiders).

5. **CD (P5).** Foo adds the `oya-cd` Argo CD application template pointed at THEIR cluster + a progressive-delivery config (canary/blue-green as DATA) + their release manifest. At release-cut, `oya-cd-release-ledger` records Foo's release in Foo's tenant partition with Foo's SBOM + cosign + Rekor attestation + a `reproducible: true` attestation, and `oya-cd-rollout` starts the canary/bake/rollback.

6. **DEPS/GOVERNANCE (P7).** Foo drops `oya-deps.toml` (its LTS roster, its license allow/deny set, its stewardship registry + CVE SLAs). On every ChangeSet, the bump-bot proposes bumps against FOO's roster with license/advisory/strict-version gates pre-run; `oya-govbot` enforces Foo's changelog row + EOL/sunset countdowns; the semver-discipline gate blocks SemVer-violating bumps to Foo's public crates.

7. **LOCAL ON-RAMP (P6).** `oya-dev init` scaffolds steps 1–4's config files (neutral `oya-ci.toml` + runner wrapper + buck cell overlay) and the first empty baseline; `oya-dev check` runs the IDENTICAL gates pre-push.

**At no point does Foo fork oyatie, mirror oyatie's directory tree, or inherit oyatie's brand deny-list.** Foo consumes versioned product artifacts and supplies only configuration — the GitHub-Actions + Bazel adoption class. **This is the bar; every gap in §1 and every NET-NEW in §8 exists to close the distance to it.**

---

## §3. THE CONFIG-DRIVEN PUBLIC BOUNDARY (generalized from `oya-ci.toml`)

### §3.1 The proven seam and the trap
`oya-ci-config` is the template: closed schema, every section `#[serde(default)]`, an absent/partial file materializes `bundled_default()`, `digest()` (FNV-1a) stamped into provenance. **But the trap is verified:** `bundled_default()` IS oyatie's policy verbatim — `required_prefix = "oya-"`, `forbidden_stems = [foundry, forgejo, jenkins, oya-vcs]`, `governance_crate_substr = "oya-governance"`, `root_markers = ["specs/root-hub-pointers.json"]`, `doctrinal_carve_outs = ["oya-tooling-agent-read"]` (verified `oya-ci.toml:15-22` + the `bundled_default_matches_todays_*` tests). **Zero-config does NOT mean policy-free; it means oyatie-policy.** An external adopter who drops no config inherits oyatie's brand deny-list and directory layout as REQUIREMENTS.

### §3.2 The four boundary changes (NET-NEW, all additive to the closed schema)
1. **`profile`/`extends` + `neutral_default()`.** Split `bundled_default()` into `OyaCiConfig::neutral()` (policy-free floor — empty `forbidden_stems`, no `required_prefix`, generic `root_markers` defaulting to `.git`, no `governance_lanes`; gates present-but-quiet, zero false-RED, names ZERO oyatie paths) vs `OyaCiConfig::oyatie()` (today's values, kept verbatim for self-host). Add a top-level `profile = "neutral" | "oyatie"` (or `extends = "..."`) key so a repo DECLARES its starting point in config. The closed schema still rejects unknown keys.
2. **`[output].faces_dir`** (default `.oya-ci/faces/`) replaces the verified hardcoded literal `repo_root.join("cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app")` (producer `main.rs:173,179`). Relocation becomes a DATA edit.
3. **`[cross_artifact].sources`** replaces the compiled-in oyatie artifacts (`docs/machine-readable/catalog.json` + `contracts.json` + the `specs/fixtures/phase0-automation-ratchet/` blocking fixture) baked into the cross-artifact + automation faces.
4. **Config-driven test harness:** replace the per-gate test repo-root walk-up that hardcodes `specs/root-hub-pointers.json` (the DEEPEST blocker — the ENGINE is portable but the gate RUNNERS embed the oyatie marker) with a shared harness crate reading `[repo].root_markers` (or an `OYA_CI_REPO_ROOT` env the runner sets). Fixtures resolve under config too.
5. **`schema_version`** key (+ a published `$id`/`$schema` URL like `renovate-schema.json`) so the closed schema can evolve without breaking external adopters.

### §3.3 The de-oyatie rename set (consolidated across ALL four facets)
These are PROPOSALS gated behind §7 ratification (no source rename before the names land as ADRs — masterplan-is-generated-from-ADRs rule).

| Current (oyatie-internal) | Canonical product identity | Source of finding |
|---|---|---|
| `oya-cloud-ci-accounting-registry-app` | `oya-ci-producer` (the conformance-face producer) | A, C |
| `oya-cloud-ci-firewall-app` | `oya-ci-ratchet` (the shrink-only ratchet runner) | A, C |
| `oya-ci-config-kernel` | the publishable config-schema crate (drop the `-kernel` role suffix from its public identity) | A, C, D |
| `cloud/cloud-ci/gates/` path namespace | product-rooted `gate-pack` namespace, decoupled from the `cloud/` app tree | A, C |
| gate ids `oya-cloud-ci-<gate>-app` | `<pack>.<gate>` taxonomy (`core.total-accounting`, `rust-cargo.manifest-hygiene`) — drop `oya-cloud-ci-`/`-app` | C |
| `GateFace` closed enum | string `face_id` + published `face_schema` (open, third-party-nameable) | C |
| copied per-crate `Finding`/`Report`/`Verdict` | `oya-pipeline-gate-sdk` (ONE versioned public crate) | C |
| `oya-dev-cli` (the retiring `oya`) | `oya-dev` (the positive dev-env product, distinct from the deprecated CLI) | A |
| `oya-ci-bespoke-prow` / "bespoke Prow" | `oya-pipeline` (the orchestration product) | A |
| `oya-ci.toml` `required_prefix = "oya-"` ENFORCED | a per-repo config value, not a baked assumption (already overridable; the DEFAULT must move to the oyatie profile) | A, B, D |
| `bundled_default()` (≡ oyatie policy) | split into `neutral_default()` + `oyatie_default()` | B, D |
| `root_markers` default `specs/root-hub-pointers.json` | VCS-neutral default (`.git`), config-only | B |
| producer `out_dir`/`git_facts` default path | `[output].faces_dir` (`.oya-ci/faces/`) | B |
| `_sign_off_additions` founder-singular door | tenant-scoped baseline-exception authority (multi-tenant) | C, B |
| `oya-ci-required` fan-in name | parameterizable to the adopter's own context name | B, C |
| `oya-ci-required.yml` (GHA-specific) | runner-agnostic pipeline manifest + registry-driven completeness invariant | C |
| `NON_GATE_CRATES` hardcoded `[&str;2]` + gates/ grep | declared gate-registry membership (no dir/workflow-path coupling) | C |
| webhook-gateway kernel comment "GitHub→Jenkins→GitHub bridge" + Makefile/ADR Jenkins refs | de-Jenkins'd (residual retired-tool naming) | A |
| `ops.oyatie.com` / `join.oyatie.dev` / Cloudflare-edge coupling in Makefile/CD | config, not baked endpoints | A |
| `oya-vX.Y.Z` hardcoded product-version prefix | a config knob (adopter uses `widget-vX.Y.Z`) | D |
| `foundry-pipeline-mirror.md`, `axis-foundry` owner, "Foundry release capabilities" | de-foundried (FORBIDDEN vocab per `oya-ci.toml` `forbidden_stems`) | D |

---

## §4. EXTENSIBILITY — the gate/plugin SDK + marketplace + cloud-scale execution

### §4.1 The SDK extraction (NET-NEW, lowest-delta, highest-leverage)
Extract ONE versioned crate **`oya-pipeline-gate-sdk`** that holds the contract every gate (1st- and 3rd-party) implements, replacing the verified 7 copied `struct Finding` definitions:
```
trait Gate {
    fn gate_id(&self) -> &str;
    fn codes(&self) -> &[&str];
    fn evaluate_keyed(&self, input: &FaceValue) -> BTreeSet<Finding>;   // SSOT
    fn evaluate(&self, input: &FaceValue) -> Report { /* provided: bare-code projection */ }
}
```
The contract IS the existing pure, I/O-free, panic-free shape (`#![forbid(unsafe_code)]`, ADR-0083 Tier-3 no-panic). Versioned with a SemVer'd `SDK_ABI_VERSION` so a marketplace can refuse an incompatible gate. **Aligns with the sibling `OYA-CI-PRODUCT-ARCHITECTURE-PLAN.md` §WS-D verdict** (Option D2 — published source crate `oya-ci-gate-contract` — chosen PRIMARY; D3 out-of-process JSON-stdio gate adopted SECONDARY for any-language/strong-sandbox). NAMING NOTE: the sibling calls it `oya-ci-gate-contract`; this capstone proposes `oya-pipeline-gate-sdk`. **These must be reconciled to ONE name at ratification (§7).**

### §4.2 The producer's hard wall and the registry
Verified: adding a gate today requires EDITING THE PRODUCER (a new `GateFace` enum variant, a new `GateInputs` field, a new `match` arm) — the exact anti-goal. Replace the compile-time `match` with a runtime **GateRegistry** (`gate_id -> Box<dyn Gate>` in-tree; `gate_id -> ResolvedArtifact` external/wasm) and collapse the per-gate `GateInputs` fields to a generic `BTreeMap<FaceId, Value>`. **The firewall ratchet runner needs ZERO change** — it is already generic over `(gate, code, key)` (verified §1 P2). This is the strongest productization asset: a third-party gate's findings flow through the ratchet unchanged.

### §4.3 Three binding kinds + open config seam
Extend the existing `GateInputKind`: keep `producer-face` (in-tree Rust, static dispatch) for first-party; add `external-artifact` (a published gate binary speaking the JSON face↔findings protocol over a process boundary) and `wasm-component` (`wasm32-wasi-p2` implementing a gate WIT interface in the SAME Wasmtime sandbox `docs/standards/plugin-authoring.md` already specifies). Relax the closed schema at ONE point: `[[gates.enabled]]` gains `source = "marketplace://<ns>/<gate>@<ver>"` (or `crate://`/`path://`), a string `face_id`, and an OPEN `[gates.enabled.params]` table validated against the gate's PUBLISHED param schema (not `deny_unknown_fields`).

### §4.4 Marketplace = REUSE `plugin-authoring.md` for a NEW artifact class "gate"
The full plugin marketplace standard EXISTS (verified `docs/standards/plugin-authoring.md`: manifest `id <ns>.<name>` + version + trust_tier verified-isv|community|experimental, capabilities_required allowlist, resource_caps, cosign-keyless + Rekor + SBOM, Wasmtime+WASI-P2 sandbox, 30% rev-share). **Do NOT reinvent it.** Apply the SAME manifest+trust+signing+sandbox machinery to the "gate" class, adding `sdk_abi_version`, `codes[]`, `face_schema`, `binding_kind`, `default_disposition`. PUBLISH = `oya build gate new` scaffold → `oya gate publish` (cosign + Rekor + SBOM + push). DISCOVER = the `registry/catalog/*.yaml` pattern extended with a `gate` kind → `oya gate add <id>` writes the `[[gates.enabled]]` entry, resolving + verifying signature before enabling.

> **SECURITY POSTURE (load-bearing, see PM-2):** a CI gate has MORE power than a runtime plugin — it blocks/allows merges to a consumer's main branch. The gate marketplace should DEFAULT to a HIGHER bar than runtime plugins: `verified-isv` only by default; community gates run **advisory-mode-only** until the consumer explicitly promotes them to `baseline-block-on-new`. `wasm-component` is the PREFERRED third-party binding (sandbox + capability-deny clock/network/random); `external-artifact` reserved for trusted first-party to avoid an un-sandboxed native-binary attack surface in the pipeline.

### §4.5 Cloud-scale execution
Promote the verified-but-inert buck2 `remote_test_execution_toolchain` (`toolchains/BUCK:5`, `:test` currently `noop`) to live: a NativeLink RE/CAS control-plane (the masterplan `cloud-ci` `oya-ci-tenant-runner-scheduler`) schedules gate lanes as content-addressed remote actions. Each gate's inputs are DECLARED (the hermetic discipline the git-facts emitter already enforces: the committed `git-facts.generated.json` face is a declared input, no action shells out), so actions are deterministic + cacheable across tenants via shared CAS. Third-party wasm gates run in the Wasmtime sandbox AS the remote action — sandbox + RBE compose. The scheduler is multi-tenant; the firewall's per-repo sign-off door becomes the per-tenant authority boundary. **This is the largest NET-NEW; staged LAST (§8, sibling §WS-E) and never touches the live required context until an owned runner re-proves byte-parity.**

---

## §5. DISTRIBUTION + GOVERNANCE-AUTOMATION

### §5.1 Versioning (3-axis, already Accepted, mostly unbuilt)
The strategy doc (`release-versioning-strategy.md`, Tier-1 Accepted) defines: crate/SDK SemVer 2.0.0 (workspace lockstep pre-GA, independent post-GA); product `oya-vX.Y.Z` (config-knobbed prefix for adopters); external-API hybrid (stability segment v1/v1beta1 + date `?api-version=YYYY-MM-DD`); 12-month LTS, 90-day EOL warning, 180-day sunset. **Each product (`oya-build`/`oya-ci-floor`/`oya-pipeline`/`oya-cd`/`oya-dev`/`oya-checks`/`oya-govbot`) carries its OWN SemVer + a published, versioned config schema** (`$id` + `schema_version`) so external adopters pin a product version and a config-schema version independently. A **cross-product compatibility matrix** (which producer pairs with which config-schema, which build overlay, which runner) is NET-NEW and required for an adopter to pin a coherent stack.

### §5.2 Reproducible-build promise (attested, not doctrinal)
Per the hermetic doctrine (fresh clone builds + runs, no prebuilt blobs), surface reproducibility as an explicit, ATTESTED product promise: a `reproducible: true` attestation recorded per release-cut in the `oya-cd-release-ledger`, with hermeticity verified in a FRESH clean checkout (not the warm tree). NOTE the verified GAP the sibling flags: `deny.toml` CLAIMS a `scripts/check.sh` + CI wiring that does NOT exist — reproducibility gates ship BORN-ADVISORY (using the existing `advisory-until-infra` disposition) until that infra is closed and they run green.

### §5.3 Strict deps + Dependabot-equivalent + repo-automation
`oya-deps.toml` makes the LTS roster + license allow/deny (today hardcoded in `deny.toml`) + supply-chain triad (cargo-audit/deny/vet) + the ADR-0345 stewardship registry into DATA. The ACTUATOR is the in-house Rust bump-bot (the `renovate.json`/dependabot replacement, neither of which exists in the tree) that opens **scm-facts ChangeSets** (provider-neutral) with license/advisory/strict-version gates pre-run, routing majors through the breaking-change/deprecation lane. `oya-govbot` unions the auto-changelog lane (`governance-lanes/changelog-row.md`, Accepted), the CVE-SLA countdown, the EOL/sunset ledgers, and the branch-protection-as-code drift checker — all driven by the same closed-schema config, all emitting scm-facts ChangeSets (VCS-agnostic; "git is transitional").

> **Single-product-or-family question (OQ):** whether `oya-govbot` is one product or three separately-versioned sub-surfaces (release-train / deps / repo-bots) is OQ-D1 — resolved at the §7 ADR.

---

## §6. RELATIONSHIP TO THE SIBLING DESIGNS (reference, do NOT duplicate)

This capstone sits ON the siblings; it neither restates nor redesigns them.

- **Substrate — lifecycle hermeticity + zero-shell + buck2-native.** The brief names `LIFECYCLE-HERMETICITY-ZERO-SHELL-ARCHITECTURE.md`; **that exact filename does NOT exist in either tree (verified search of `source/docs/` + the `linux` audit dir).** The live hermetic-execution thread is **`OYA-CI-HERMETIC-EXECUTION-DESIGN.md`** (same audit dir, STATUS pending-approval) — the producer-as-pure-function-of-declared-inputs + the git-facts hermetic boundary + the buck2 sandbox model. The zero-shell posture (fixture binaries `fake-cargo`/`fake-verify-command` replacing shell scripts) is realized in source, not in a doc of the named title. **Treat these as the GIVEN; do not redesign.** (Flagged so the founder can confirm whether the named substrate doc is forthcoming or is shorthand for the hermetic-execution + zero-shell pair.)
- **scm-facts seam — `OYA-CI-VCS-AGNOSTIC-SEAM-REFINEMENT-PLAN.md`** (same dir). The single out-of-graph VCS boundary; "git is transitional"; the `scm-facts` rename + the `ScmFactsSource` trait. Every product's VCS-agnosticism (the bump-bot ChangeSets, the runner-agnostic completeness invariant, branch protection) routes through THIS seam. Do not re-specify it.
- **The oya-ci product thread — `OYA-CI-PRODUCT-ARCHITECTURE-PLAN.md`** (north-star, NOT door:one-way, workstream-ratified) + **`OYA-CI-CONFORMANCE-FLOOR-PLAN.md`** (the floor, pending-approval, door:one-way). This capstone's P2/P3/P4 EXTEND those: §WS-D (gate SDK), §WS-E (cloud control-plane), §WS-F (hermetic backends), §WS-G (reproducibility/dev-env), §WS-H (dep-bot), §WS-I (repo-automation suite) map 1:1 onto P3-SDK / P4-cloud / P1-build / P6-dev+P7-repro / P7-deps / P7-bots respectively. **The byte-for-byte backward-compat green-invariant from those docs remains supreme across every product here.** This capstone's contribution is the CROSS-PRODUCT product-line frame + the external-adoption acceptance test + the unified de-oyatie rename set + canonical placement — NOT a re-derivation of the oya-ci internals.

---

## §7. CANONICAL PLACEMENT (reachability principle — productize into canon, not just a design doc)

Per the masterplan-is-generated-from-ADRs SSOT rule (worth-documenting ⇒ worth-reading ⇒ must be reachable from masterplan/workflow else archive) + the amend=supersede/re-author + door:one-way founder sign-off rules.

**ADRs to author (clean ADR-0000+ series, founder one-way sign-off each):**
1. **ADR — Platform product-line taxonomy + canonical product names** (the seven products + the de-oyatie rename set §3.3). MUST precede any source rename. Reconciles the `oya-pipeline-gate-sdk` vs `oya-ci-gate-contract` name clash (§4.1).
2. **ADR — The config-driven public boundary + `profile`/`neutral_default` + `schema_version`** (§3) — generalizes the proven `oya-ci-config` seam to the product floor.
3. **ADR — Gate/pipeline-step SDK + marketplace (gate artifact class)** (§4) — reconciles `plugin-authoring.md` reuse + the higher CI-gate trust bar.
4. **ADR — Cross-product versioning + reproducible-build attestation + distribution channel** (§5) — and RESOLVE the open distribution-channel question (crates.io vs OCI vs pinned-git-release vs all-three) given the no-external-blob doctrine.

**Proposed ADRs to RESOLVE (ratify or drop — reachability rule forbids unaccounted Proposeds blocking these products):** `ADR-0037` (API stability tiers), `ADR-0041` (gitops/trunk/release-branch-cut), `ADR-0342` (API versioning hybrid), `ADR-0345` (OSS stewardship + CVE SLA) — all verified `Proposed`. Re-author into the clean release-governance ADR series (they predate the buck2 + scm-facts + oya-ci substrate). Also FIX the broken citation in `release-versioning-strategy.md` (cites ADR-0052/0053/0054 = "grit", not versioning) and de-stale `dependency-policy.md §8`.

**Masterplan entries:** the products register against the EXISTING `specs/bespoke-cloud-toolchain-services.json` (verified `not_internal_only: true`, `cloud-ci`/`cloud-cd`, `oya-cd-release-ledger`/`oya-cd-rollout`, roadmap `P-TOOLCHAIN`, tenant T0–T4 with T3 "Tenant offering private preview"). DECISION REQUIRED (OQ-D2): do `oya-build`/`oya-pipeline`/`oya-dev`/`oya-govbot` fold into that spec as NAMED services, or get a NEW top-level masterplan section peer to `bespoke_cloud_toolchain_services`? Either way the masterplan must reach this doc (else it gets archived per the rule).

---

## §8. ROADMAP (cheapest-highest-value-first; reuse vs net-new; relationship to the in-flight buck2 landing + the migration)

> **Hard sequencing inherited from the siblings:** the conformance FLOOR + the migration come first and NOTHING here blocks them. The live `oya-ci-required` matrix + `cargo` stay the authority until an owned runner re-proves byte-parity. The in-flight buck2-hermetic landing (tasks #70/#71: buck2 build+test byte-parity to cargo + the buck2 CI lane) is the PREREQUISITE for P1 productization — it is REUSE, not net-new, and must GREEN before P1 ships.

| # | Step | Reuse / Net-new | Value |
|---|---|---|---|
| R0 | Land the in-flight buck2-hermetic byte-parity + buck2 CI lane (tasks #70/#71); complete the migration | REUSE (in-flight) | Unblocks P1; prerequisite |
| R1 | **Extract `oya-pipeline-gate-sdk`** (`trait Gate` + `Finding`/`Report`/`Verdict` from the 7 copies); migrate built-in gates to `impl Gate`; add `SDK_ABI_VERSION`. Carries its OWN byte-parity proof. | NET-NEW (small delta — contract already IS a pure fn) | Highest leverage / lowest cost; unlocks P3 + all extensibility |
| R2 | **Add `profile`/`neutral_default()` + `schema_version` + `[output].faces_dir` + `[cross_artifact].sources`** to the config; relocate the producer's 2 hardcoded literals to config | NET-NEW (additive to closed schema) | Closes the "zero-config = oyatie-policy" trap; P2 portable |
| R3 | **Config-driven test harness** (shared crate reads `[repo].root_markers`; fixtures under config) — closes the DEEPEST blocker | NET-NEW | Gate RUNNERS become portable, not just the engine |
| R4 | **Publish the engine artifacts** (producer + ratchet + registry-drift + git-facts-emitter + config-schema) as versioned, pinned-by-tag releases (the buck2 binary is already consumed this way) | NET-NEW (packaging, not rearchitecture) | External repos depend on a PRODUCT, not oyatie's workspace |
| R5 | **Composite GitHub Action + copy-in matrix template** (the quick-start already promises it); runner-agnostic registry-driven completeness invariant replacing `gate_registration.rs` grep | NET-NEW | P4 external GHA adoption; meets half the acceptance class |
| R6 | **Runtime GateRegistry + `external-artifact`/`wasm-component` bindings + open `[gates.enabled.params]`**; replace the producer `match` | NET-NEW | Third-party gates without forking the producer |
| R7 | **Gate marketplace** (extend `plugin-authoring.md` for the gate class; `oya build/publish gate`; registry `gate` kind; higher CI-gate trust bar) | REUSE the plugin standard + NET-NEW gate fields | Marketplace; matches GH-Actions-marketplace class |
| R8 | **`oya-build` overlay + reindeer wrapper + portable toolchains/platforms + Bazel `rules_oya_ci`**; close the `/usr/bin/clang` host-path hermeticity + multi-platform | NET-NEW + finish in-flight hermeticity | Build-product adoption; the Bazel half of the acceptance class |
| R9 | **`oya-dev` positive scaffolder** (`oya-dev init`/`check`) replacing the retiring CLI | NET-NEW | The adoption ON-RAMP; collapses steps 1–4 to one command |
| R10 | **`oya-govbot`**: build the 6 release-governance gate crates (semver-check-cli is a SCAFFOLD), the in-house bump-bot (scm-facts ChangeSets), auto-changelog/EOL/branch-protection bots; resolve the 4 Proposed ADRs | NET-NEW + RESOLVE Proposeds | Dependabot/repo-automation-bots equivalent |
| R11 | **`oya-cd` config-driven progressive-delivery runner** (spec corpus → executable) + Argo template decoupled from oyatie infra; build `oya-cd-release-ledger`/`oya-cd-rollout` | NET-NEW (large) | CD-product adoption + reproducible-build attestation |
| R12 | **Cloud-scale execution control-plane** (live NativeLink RE/CAS; multi-tenant scheduler; wasm gates as sandboxed remote actions; per-tenant sign-off authority) | NET-NEW (largest) | Hyperscale; staged LAST; never touches the live required context until byte-parity re-proven |

---

## §9. PRE-MORTEM (failure modes, each with a mitigation)

- **PM-1 — The config surface leaks oyatie assumptions (the most likely silent failure).** Even with `profile = "neutral"`, residual oyatie defaults (the `oya-` prefix, the `forbidden_stems` brand set, the `specs/...` directory layout, the `governance_lanes`) leak into the floor and an external repo silently produces empty/degraded faces instead of an explicit opt-out, giving false confidence. **Mitigation:** the neutral profile DISABLES (not advisory-empties) gates whose sources are absent; a CI lane runs the producer against a SYNTHETIC non-oyatie fixture repo and asserts zero oyatie path literals appear in any face; the `acme-` test (lib.rs:798) is generalized into a full neutral-profile conformance suite. (OQ: disable-vs-advisory-empty for absent sources.)

- **PM-2 — The SDK/marketplace is a security surface (third-party gate code executes over a consumer's repo and can block/allow merges to main).** An un-sandboxed `external-artifact` gate is arbitrary native code in the merge path. **Mitigation:** `wasm-component` is the PREFERRED third-party binding (Wasmtime+WASI-P2 sandbox, capability-deny clock/network/random per `plugin-authoring.md`); `external-artifact` reserved for trusted first-party; the gate marketplace DEFAULTS to a higher bar than runtime plugins (verified-isv only; community gates advisory-mode-only until explicitly promoted); a gate-conformance harness (run twice, diff for determinism; codes match manifest; pure/panic-free) gates the `verified-isv` tier; cosign + Rekor + SBOM provenance verified before enable.

- **PM-3 — Bazel support doubles maintenance.** A native Starlark reimplementation of every gate rule drifts from the buck2 + cargo paths and triples the test matrix. **Mitigation:** the "one logic" principle — `rules_oya_ci` (and the buck2 cell) WRAP the SAME prebuilt engine binary over the SAME `oya-ci.toml` + scm-facts; neither build system re-implements gate logic. Bazel is a thin invocation shim, not a second engine. (OQ: wrap-binary vs native-Starlark — wrap wins on the one-logic principle.)

- **PM-4 — Cloud-scale execution is a huge net-new build that swallows the roadmap.** The NativeLink RE/CAS control-plane + multi-tenant scheduler is the largest single piece and could stall the cheaper, higher-value adoption wins (R1–R9). **Mitigation:** it is staged LAST (R12, sibling §WS-E entry-gate: floor + migration + SDK + hermetic-backend must all be done first); the buck2 RE seam is already WIRED (`toolchains/BUCK:5`) so it is config-activation + a service, not a graph rebuild; external adoption v1 explicitly targets the GitHub-Actions-composite-action + Bazel-overlay class with self-hosted Prow + cloud-scale RE as v2 (resolves the runner-sequencing OQ).

- **PM-5 — Canonical placement balloons the ADR set.** Four new ADRs + four Proposed resolutions + a masterplan section, authored against a tree mid-migration, risks an unaccounted-ADR sprawl that violates the very reachability rule it serves. **Mitigation:** re-author the four Proposeds into the clean consolidated release-governance ADR series (amend=supersede rule) rather than ratifying them piecemeal; each new ADR is a door:one-way founder sign-off; the masterplan placement decision (fold-in vs new-section) is made ONCE (OQ-D2) before any ADR lands; this capstone itself must be reachable from the masterplan or be archived.

- **PM-6 — Published artifacts + the no-external-blob / self-host doctrine collide.** "Any other person should be able to utilize our tools" implies a public channel (crates.io), but the hermetic doctrine forbids external/prebuilt blobs and the bespoke-toolchain spec is tenant-isolated. **Mitigation:** resolve the distribution-channel question at the §7 ADR (likely: OCI artifacts + pinned-git-release for the engine, mirrored content-addressed, with crates.io as an optional convenience mirror, never a build dependency); the `reproducible: true` attestation per release-cut + fresh-clone verification keeps every published artifact reproducible-from-source, satisfying both doctrines.

- **PM-7 — The migration WIP and this product line race.** Productization mutations (R1 SDK extraction, R2 config split) touch the same gate crates the in-flight buck2 byte-parity + migration are landing. **Mitigation:** R0 (buck2 byte-parity + migration) is a hard prerequisite; every productization step carries its OWN byte-parity proof against the floor baseline (total-accounting=48633 · brand-residue=4494 · manifest-hygiene=233 · cross-artifact=168 · automation-ratchet=153 · bnf=79 · staleness=64) before it may gate `dev`; all work in isolated worktrees, founder-go + WIP-commit-first.

---

## §10. OPEN QUESTIONS (carried up for the founder; gate the §7 ADRs)

1. **Portfolio scope:** is `oya/` (the ~70 SaaS verticals + SolidJS shell + `oya-ci-deck`) explicitly OUT of the product line, or is any of it (the UI shell, the deck) a productized component?
2. **Language-agnostic meaning:** does the gate ENGINE stay Rust while gate INPUT-KINDs cover any language, OR is non-Rust gate AUTHORING (a wasm/exec gate-pack ABI) a first-class v1 adoption path?
3. **Distribution channel:** crates.io vs OCI vs pinned-git-release vs all-three, given the self-host/no-external-blob doctrine (PM-6).
4. **Naming SSOT:** ratify the canonical product names (incl. the `oya-pipeline-gate-sdk` vs `oya-ci-gate-contract` clash) as ADRs + masterplan entries BEFORE any rename (masterplan-from-ADRs rule).
5. **Runner sequencing:** is external adoption gated on bespoke-Prow reaching production, or is "GHA composite action + Bazel overlay" the v1 acceptance target with self-hosted Prow as v2 (PM-4)?
6. **CD scope for v1:** Argo-CD-template-only (GitOps), or must the config-driven progressive-delivery runner be built before CD counts as a product?
7. **Masterplan placement:** fold into `bespoke_cloud_toolchain_services` as named services, or a new top-level section?
8. **Govbot product count:** one `oya-govbot` or three independently-versioned sub-products (release-train / deps / repo-bots)?
9. **Neutral-profile default for absent sources:** DISABLE the gate (no false confidence) or run advisory-empty (keeps the dashboard) — PM-1.
10. **Multi-tenant sign-off:** how the founder-singular `_sign_off_additions` one-way door becomes a per-tenant exception authority without weakening the ratchet's tamper-evidence.

---

*End of capstone. STATUS pending-approval · door:one-way. No source mutated, no ADR/masterplan written, no git operations performed. The only artifact is this doc.*
