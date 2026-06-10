# STEP-1 TRIAGE — CLI-Governance → Firewall Migration (inventory + disposition)

STATUS: triage-complete
ROLE: Step-1 deliverable of `CLI-GOVERNANCE-TO-FIREWALL-MIGRATION-PLAN.md` (§6 Task Flow STEP 1). Confirms/revises the plan's PROVISIONAL "CANONICAL-BLOCKING: 6" cut against full evidence.
MODE: READ-ONLY synthesis. Nothing in `/Users/jasonlee/Developer/source` was mutated. This artifact lives in the `linux` audit tree (writable; not a source mutation).
VERIFIED-AGAINST: real files in `/Users/jasonlee/Developer/source` (CWD confirmed `…/source`, no contamination). Anchors re-read live this pass: `oya-governance-predictable-naming-kernel/src/lib.rs:32,:164`, `oya-dev-cli/src/commands/gate/architecture_boundaries.rs:1-56`, `oya-dev-cli/src/lib.rs:31`, `workspace_topology_gate.rs`/`workspace_hygiene_gate.rs`, committed gate-baseline gate-ids, `find ./libs -maxdepth 1 -name 'oya-check-*'` = 72.
INPUTS MERGED: 111 per-crate dispositions (72 `oya-check-*` + 38 `oya-governance-*` kernels + the catalog domain crate) · dev-cli gate-surface inventory (~115 lanes) · gate-catalog-domain inventory (107 `AGGREGATED_VALIDATE_LANES`, 37 orphan) · recomputed BNF count.

---

## 0. Headline rulings (what changed vs the plan's provisional cut)

| Plan provisional | Step-1 ruling | Change |
|---|---|---|
| CANONICAL-BLOCKING = 6 {S1 BNF-suffix, S2 manifest-hygiene, S3 cargo-prefix, S4 retired-vocab→brand-residue, S5 data_class, S6 slot2-NETNEW} | **CONFIRMED as the blocking *target* set, with one promotion + one reclass** | partial revise |
| MINIMAL pre-M-lane floor = {S1, S2} | **CONFIRMED minimal, but PROMOTE S3 cargo-prefix → floor {S1, S2, S3}** | revise |
| BNF debt ≈ 110 (F-0023 hypothesis) | **RECOMPUTED = 79** (full S1-gate rule) | replace |
| S1 source = pure `predictable-naming-kernel::check()` (NOT architecture_boundaries.rs) | **CONFIRMED** by live re-read | confirm |
| S2 source = net-new predicate (no pure crate owns full field-set) | **CONFIRMED, with partial reuse of topology/hygiene arms** | confirm + refine |
| slot2 (S6) = NET-NEW, needs-infra | **RECLASS to ADVISORY-until-infra** (no detector exists; cannot author equivalence proof; needs-infra) | revise |

The three changes:
1. **PROMOTE cargo-prefix (S3) into the pre-M-lane FLOOR.** It is the ONLY deferred discipline enforced at full strength TODAY (`backbone-microservices-ci.yml:305` — the lone live `oya gate validate` consumer in all of `.github/workflows/`). Deferring it makes the migration window strictly weaker than the status quo on cargo-prefix. It is shape (b) pure (`oya-intelligence-cargo-prefix-domain::validate_cargo_prefix`, lib.rs:31 — confirmed imported) with LOW self-DoS and the lowest authoring cost. The plan's §3a already names this the "strong candidate for promotion"; Step-1 makes the call: **floor = {S1, S2, S3}** (3 serial producer folds + 3 signoff doors).
2. **RECLASS slot2 (S6) from NET-NEW-BLOCKING to ADVISORY-until-infra.** There is no `oya-check-slot2*` lib, no `oya gate validate slot2` arm, and no orphan catalog lane for it — the inventory found ZERO existing detector. A blocking gate with no old detection has no equivalence-oracle to record (the plan's MANDATORY acceptance rung), and it is `needs-infra` (cannot baseline until the M-lanes ADD services). Authoring a net-new blocking detector with no diff target is exactly the higher-cost case the plan flags as "may move to ADVISORY." Ruling: **ADVISORY (needs-infra); promote to blocking after services exist and RED/GREEN fixtures define the discipline.**
3. **The BNF count is 79, not ~110.** See §4.

Net: the blocking *target* stays at six disciplines, but only **five are real ports** (S1, S2, S3, S4, S5) and **slot2 is advisory-until-infra**. The mandatory pre-M-lane floor is **{S1, S2, S3}**.

---

## 1. S1 source confirmation (BNF layer-suffix §2.5#4)

**CONFIRMED: S1 = the PURE `oya-governance-predictable-naming-kernel::check()` — NOT `architecture_boundaries.rs`.**

Live re-read evidence:
- `libs/oya-governance-predictable-naming-kernel/src/lib.rs:164` — `pub fn check(rows: &[CrateNaming]) -> Result<NamingReport, NamingError>`, FS-free (takes pre-resolved `CrateNaming` rows; the corpus walk is the caller's job → re-homed into the producer `collect_bnf_layer_suffix`).
- `:32` — `pub const ALLOWED_ROLES: [&str; 13]` = `{kernel, domain, usecase, app, adapter, infrastructure, cli, rest, grpc, graphql, worker, sdk, api}`; test at `:~406` asserts `len()==13` and excludes `application`/`runtime`/`test`.
- Three carve-outs are intrinsic to `check()`: `is_check_family` (`oya-check-*`, CHECK_FAMILY_PREFIX), `is_backend_qualified_adapter` (BACKEND_SUFFIXES = 9 values `{fake,inmemory,aws,oci,gcp,azure,postgres,redis,sqlite}`), `is_doctrinal_carve_out`. Violation taxonomy = `NamingViolationKind` (`NameContainsUppercase`, `MissingOyaPrefix`, `EmptyAfterPrefix`, role-mismatch/unknown-role) → maps cleanly onto §2.5#4 finding-codes.
- **Disproof of the conflation trap:** `architecture_boundaries.rs:1-56` module header lists FOUR invariants; invariant #4 = the role-based **`ALLOWED_DEPENDENCY_ROLES` matrix** (the hexagonal IMPORT-MATRIX = §2.5#5, lane S7), and it does `std::process::Command` (`cargo metadata`) + FS walks (impure shape c). It *references* the predictable-naming-kernel ALLOWED_ROLES as a separate enum source. So `architecture-boundaries` ⇒ §2.5#5 (S7 backfill, advisory), and §2.5#4 layer-suffix is the SEPARATE pure `check()`. The dev-cli surface inventory independently flags the same: "`architecture-boundaries` must be mapped to §2.5#5, and #4 is a different (unwired) detector."

**Reuse shape = (a) pure lib dep; equivalence-oracle = UNIT-DIFF** (new-gate `evaluate_keyed` keys vs `check()`'s `NamingReport` on identical `CrateNaming` rows). NOT end-to-end.

---

## 2. S2 source confirmation (manifest-hygiene §2.5#7)

**CONFIRMED: shape (c) — extract a NET-NEW pure Cargo.toml field-predicate + re-home the manifest read into the producer `collect_manifest_hygiene`; reuse the existing `workspace-topology`/`workspace-hygiene` arm logic for the fields they already cover.**

Live re-read evidence:
- The arms EXIST: `oya-dev-cli/src/commands/gate/mod.rs:768` (`validate workspace-topology`) and `:2059` (`validate workspace-hygiene`), backed by `workspace_topology_gate.rs` / `workspace_hygiene_gate.rs`.
- **No single arm covers the full §2.5#7 field-set.** The topology-gate fixtures (`workspace_topology_gate.rs:586,:598`) show only `name`/`edition`/`version`/`license`; the full set required by §2.5#7 — `resolver="2"`, `version.workspace=true`, `publish=false`, `license`, `[lints] workspace=true`, `[lib] doctest=false` — is NOT demonstrably owned by one arm.
- **No `oya-check-*` lib owns the field-set.** `ls libs/ | grep -iE 'manifest|workspace-hygiene|cargo-toml'` → only `oya-gen-microservice-manifests-app` (a generator, not a checker). The 111-crate dispositions contain no manifest-hygiene check kernel.
- Both arms are dev-cli-only and **not** live in CI (the only live `oya gate validate` consumer is `cargo-prefix`).

Therefore §2.5#7 has no pure crate to wrap: extract a pure predicate, reuse whatever topology/hygiene field-checks already exist, net-new the uncovered fields, re-home the FS read into the producer. **Equivalence-oracle = END-TO-END** (capture OLD `oya gate validate workspace-hygiene`/`workspace-topology` stdout → normalize to keys → assert == new `evaluate_keyed` key-set). Self-DoS LOW.

---

## 3. Full triage table (grouped by disposition)

§2.5 column maps to the migration-conformance grammar (`MIGRATION-PLAN-RESYNC.md:136-156`); "—" = no §2.5 mapping. reuse-shape: (a) pure-lib dep · (b) pure-domain dep · (c) extract-pure-predicate-from-impure + re-home I/O · n/a = DROP. The 5 live firewall gate-ids (OVERLAP) are `cloud-ci-{total-accounting, cross-artifact-agreement, staleness-reaper, automation-ratchet, brand-residue}`.

### 3.1 BLOCKING (the migration-conformance floor + post-import backfill)

| crate / discipline | what it checks | §2.5 | reuse-shape | justification |
|---|---|---|---|---|
| **oya-governance-predictable-naming-kernel** (S1) | crate names vs `oya-<ctx>-<bc>-<cap>-<role>` BNF, role ∈ ALLOWED_ROLES(13) + 3 carve-outs | **#4** | **(a)** | THE pure layer-suffix predicate (`check()` lib.rs:164, FS-free). Live app consumer at `tools/oya-governance-predictable-naming-app/`. Pre-M-lane FLOOR S1. Baseline TODAY's 79 violations `mode:baseline-block-on-new`; unit-diff oracle. NOT `architecture_boundaries.rs`. |
| **(manifest-hygiene gate — NET-NEW)** (S2) | `resolver="2"`, `version.workspace`, `publish=false`, `license`, `[lints] workspace`, `[lib] doctest=false` | **#7** | **(c)** | No pure crate owns the full field-set (confirmed §2). Reuse partial topology/hygiene arm logic; net-new uncovered fields; re-home FS read into `collect_manifest_hygiene`. Pre-M-lane FLOOR S2. End-to-end oracle. |
| **oya-intelligence-cargo-prefix-domain** (S3) | every workspace member carries the `oya-` package prefix (ADR-0017) | #4-adjacent | **(b)** | PROMOTED to FLOOR. Pure domain fn `validate_cargo_prefix` imported at `oya-dev-cli/src/lib.rs:31`. **LOAD-BEARING TODAY** — lone live CI consumer (`backbone-microservices-ci.yml:305`). Port must go live BEFORE the CLI call is retired (Step-4.0 hard gate). Unit-diff oracle. |
| **oya-check-retired-vocabulary** (S4) | no live doc mentions any retired CLI subcommand/binary/crate term | **#11** | **(a)** | Pure kernel (`validate_retired_vocabulary` lib.rs:147). Post-import backfill — add its codes INTO the EXISTING live `cloud-ci-brand-residue` gate (NOT a new gate; dedupe pre-mortem b). |
| **oya-check-data-class** (S5) | every kernel struct field carries `#[data_class(...)]` or is in legacy-allowance | **#12** | **(a)** | Pure kernel (`validate_data_class_fitness`). Consumed by `oya-dev-cli` (data_class_gates.rs); cited by 5 microservice PRDs. Post-import backfill blocking. Unit-diff oracle. |
| **oya-check-dependency-seam** | dep-rationale registry coverage + seam-import scope + cargo-audit | **#8** | **(c)** | IMPURE (std::fs reads + spawns `cargo audit`, lib.rs:11-13/:147). Maps to §2.5#8. Backfill (S8 family): seam+registry subchecks extractable as pure fns over read data; cargo-audit subcheck stays impure. Advisory→blocking via ratchet. |
| **oya-governance-naming-justifications** | each microservice manifest cites BNF v4 + 12-layer enum in `naming_justifications` | **#4** | **(c)** | IMPURE (fs::read_dir + read_to_string, lib.rs:90); standalone non-workspace crate. Directly gates §2.5#4 at manifest level (sibling to S1's crate-name level). Pure `validate_proof`/`parse_naming_field` extractable. Currently unwired — blocking-target under MIG-PREREQ #55. |
| **oya-governance-provider-coupling-kernel** | bans provider tokens (anthropic/openai/gemini/claude/codex) in non-adapter crates | **#5** | **(a)** | Pure kernel (`check()` lib.rs:88). The hexagonal IMPORT-MATRIX shape (§2.5#5). No app runner yet. Backfill S7 (own ADR per F1) — promote from advisory after the monolith split; this is the pure half. |

Notes on §2.5#9 vendor/supply-chain family (all BLOCKING-grade but BACKFILL via S8, advisory-until-wired): `oya-check-license-policy`, `oya-check-supply-chain`, `oya-check-vendor-recency`, `oya-check-vendor-lockin-discipline` (maps #9, sole catalog-BLOCKER but dev-cli-only), `oya-check-slsa-l3-evidence-grounded`, `oya-check-image-signing-discipline`, `oya-governance-license-policy-kernel`, `oya-governance-supply-chain-kernel`. These are listed ADVISORY in §3.2 (their disposition in the 111-input is ADVISORY) but carry a §2.5#9 mapping and are the S8 backfill targets.

### 3.2 ADVISORY (report-only; promote via ratchet later)

All are pure (shape a) unless noted; all "keep — real governance value, no live-gate overlap, not a §2.5 migration-conformance check OR not yet wired." Grouped for density.

| group | crates | §2.5 | reuse-shape | justification (shared) |
|---|---|---|---|---|
| a11y / client / i18n / mobile | oya-check-a11y-discipline, oya-check-client-stack-discipline, oya-check-i18n-coverage, oya-check-mobile-native | — | a | Pure kernels, ADR-declared advisory or dev-cli-only; no live-gate overlap. |
| ADR/doc hygiene | oya-check-adr-citation, oya-check-adr-index, oya-check-adr-placeholders, oya-check-aspirational-enforcement, oya-check-authority-cohesion, oya-check-doc-axis(c-impure), oya-check-doc-catalog, oya-check-documentation-system, oya-check-glossary-coverage, oya-check-glossary-vocabulary, oya-check-honest-claims, oya-check-placeholder-debt, oya-check-readme-coverage, oya-check-substance-bar(via gov), oya-governance-adr-shape-kernel, oya-governance-doc-freshness-kernel, oya-governance-doc-style-kernel, oya-governance-substance-bar(c-impure), oya-governance-no-template-stamping(c-impure), oya-governance-purpose-kernel | — | a / c | Doc-quality / docs-consistency governance; dev-cli-only or unwired; not migration-conformance. doc-axis/substance-bar/no-template-stamping/byok are impure shape-c. |
| API / contract governance | oya-check-active-artifact-contract, oya-check-cedar-fragment-coverage, oya-check-cohesion, oya-check-cursor-pagination-coverage, oya-check-event-schema-versioning, oya-check-id-discipline, oya-check-idempotency-key-coverage, oya-check-openapi-rest-route-parity, oya-governance-upstream-api-drift-kernel | — | a | Pure API-contract invariants (ADR-0149/0150/0154/0156 etc.); dev-cli-only/unwired; no live-gate overlap. |
| security / authz / supply-chain (#9 backfill targets) | oya-check-authz-tier-discipline, oya-check-high-risk-auto-decision-refusal, oya-check-image-signing-discipline(#9), oya-check-license-policy(#9), oya-check-supply-chain(#9), oya-check-slsa-l3-evidence-grounded(#9), oya-check-step-up-auth-coverage, oya-check-vendor-recency(#9), oya-check-vendor-lockin-discipline(#9), oya-governance-image-discipline-kernel, oya-governance-license-policy-kernel(#9), oya-governance-supply-chain-kernel, oya-governance-tos-policy-kernel, oya-governance-eval-domain(b), oya-governance-eval-usecase(b) | #9 (subset) | a / b | Security-grade & supply-chain invariants. The #9-mapped ones are the S8 backfill candidates. eval-domain is live-consumed by production apps (not a firewall gate). |
| ops / SRE / data-tier | oya-check-backup-retention-discipline, oya-check-container-base-image, oya-check-cost-budget(b — prod domain, misnamed), oya-check-iac-tier-discipline, oya-check-layered-architecture-discipline(#5 sibling), oya-check-metric-cardinality, oya-check-olap-tier-discipline, oya-check-otel-trace-propagation, oya-check-ontology-projection-coverage, oya-check-perf-budget, oya-check-benchmark, oya-check-realtime-transport-tier, oya-check-rpo-rto-coverage, oya-check-runbook-freshness, oya-check-runbook-index, oya-check-saga-shape, oya-check-shardability, oya-check-slo-coverage, oya-check-statelessness, oya-check-tenant-cost-labels-coverage, oya-check-vector-store-discipline, oya-check-wasm-runtime-discipline, oya-governance-image-discipline-kernel | — | a / b | ADR-grounded ops/infra invariants; advisory/report-only by design or unwired. cost-budget is a production domain lib misnamed `oya-check-`, not a gate. |
| process / governance plumbing | oya-check-audit-chain-seal-coverage, oya-check-codeowners-mirror, oya-check-pr-traceability, oya-check-pre-push, oya-check-protection-context-match, oya-check-quality-lane, oya-check-raci-coverage, oya-check-release-pack, oya-governance-adapter-with-no-importer-kernel, oya-governance-agentic-navigability-kernel, oya-governance-authoritative-tracked-kernel, oya-governance-banned-primitives-kernel, oya-governance-bypass-kernel, oya-governance-claim-ceiling-kernel, oya-governance-cohesion-kernel, oya-governance-lifecycle-kernel, oya-governance-mistakes-ledger-kernel, oya-governance-orphan-detection-kernel, oya-governance-portfolio-citation-kernel, oya-governance-pr-merge-gate-kernel, oya-governance-pr-traceability-kernel, oya-governance-pre-push-kernel, oya-governance-quality-lane-kernel, oya-governance-sunset-lifecycle-kernel, oya-governance-architecture-map-freshness-kernel(overlap staleness-reaper), oya-governance-byok-disambiguation(c) | — | a / c | Process-hygiene / ratchet kernels. Several are pure forward-plan kernels with zero live consumers (kept advisory, NOT dropped, because invariant is real). architecture-map-freshness conceptually adjacent to staleness-reaper but distinct (digest vs row-age). |
| catalog / claim governance | oya-check-claim-ceiling(b), oya-check-compliance-evidence-coverage, oya-check-no-grouping | — | a / b | Pure; advisory. no-grouping doc-comment notes ADR-0132 wanted a BLOCKER lane never wired — promotable. |
| §2.5#11 transition (kept advisory, fed into S4) | oya-check-brand-residue | #11 | a | **This IS the kernel of the live `cloud-ci-brand-residue` gate** (imported by `oya-cloud-ci-accounting-registry-app` main.rs:26). MUST be KEPT (it is not a duplicate — it is the pure half feeding the live gate). S4 promotes brand-residue + retired-vocabulary into one gate. |
| SSOT plumbing (foundational, not a gate) | oya-governance-gate-catalog-domain | — | a | The AGGREGATED_VALIDATE_LANES SSOT consumed by the dev-cli gate dispatcher; foundational data crate, not itself a blocking gate. |
| slot2 (RECLASSIFIED) | (no crate — net-new) | #6 | n/a | **ADVISORY-until-infra.** No detector exists; needs-infra; no equivalence-oracle possible. Promote to blocking once services exist + RED/GREEN fixtures define it. |

### 3.3 DROP (with zero-consumer proof)

Only crates that are genuinely dead (scaffold stubs with zero live consumers) are dropped. Per pre-mortem (c), each carries a grep/Cargo.lock zero-consumer proof. NOTE: the plan's headline "DROP brand-residue as duplicate" is **REVISED** — `oya-check-brand-residue` is NOT dropped (it is the live gate's kernel; see §3.2). The real DROPs are the two retired Wave-3-I scaffold stubs.

| crate | what it checks | §2.5 | reuse-shape | justification + zero-consumer proof |
|---|---|---|---|---|
| **oya-governance-capability-tier-coverage** | scaffold (ADR-0316): would verify per-microservice capability-tier entries; `enforce_*` always returns `EnforcementStatus::Scaffolded` (lib.rs:42), no real check | — | n/a | DROP — retired scaffold, never enforced. **Proof:** `Cargo.lock` contains `oya-governance-capability-tier-coverage` exactly 1× (own `[[package]]` only). `has_wired_buck2_target=false`, `maps_to_oya_cli=false` in enforcement-inventory. No runner imports it outside its own scaffold_contract test; `.github/workflows/` + `cloud/cloud-ci/` references are accounting-baseline generated JSON only — no live job, no Rust dep. (Caveat: catalog inventory maps `foundry-capability-schema` lane → this crate by name-alias, but that lane has no live dispatcher — it is an orphan catalog string, not a live consumer.) |
| **oya-governance-cedar-coverage** | scaffold (ADR-0243): would verify every public API endpoint has a Cedar policy; `enforce_cedar_coverage` always returns `Scaffolded` (lib.rs:42) | — | n/a | DROP — retired scaffold, never enforced. **Proof:** `Cargo.lock` contains `oya-governance-cedar-coverage` exactly 1× (own `[[package]]` only). `has_wired_buck2_target=false`, `maps_to_oya_cli=false`. No `.github/workflows/` reference outside generated accounting JSON. (Caveat: catalog maps `cross-tenant-access-fuzz` lane → this crate by name-alias; orphan catalog string, no live dispatcher.) |

Other scaffold stubs (`oya-governance-audit-event-emission`, `oya-governance-pack-overlay-completeness`) are kept ADVISORY (not dropped) per their input dispositions: their ADRs (0263/0251) are real and the scaffolds are intentional forward-plans — DROP only the two whose `retired_or_live=retired` (capability-tier-coverage, cedar-coverage). The long tail of retired/obsolete catalog references (`oya-vcs-*` at gate-catalog-domain lib.rs:263 per ADR-0363; the three `scripts/*.sh` merge-queue helpers :264-266; four `bash tools/governance/adr-0221-*.sh` :256-259) are NON-GATE-COMMAND list entries (the legacy `scripts/check.sh` body lifted verbatim pending deletion) — they are NOT crates and are recorded for `registry/vocabulary/retired.yaml` reconciliation (plan F3), not dropped as crates here.

**Disposition coverage:** 111 crate-dispositions = BLOCKING-target 8 listed + 2 DROP + 101 ADVISORY (the §3.2 groups). Every one of the 72 `oya-check-*` + 38 `oya-governance-*` + 1 catalog crate has a row. Plus the 37 orphan catalog lanes (no backing crate) are accounted as aspirational/legacy-script lanes — disposition: not-a-crate, reconcile in retired.yaml (F3); the 14 `cloud-iac-*` orphans + architecture/workspace/hyperscaler orphans have no implementing crate in this tree.

---

## 4. Recomputed BNF layer-suffix violation count

**REPLACES [F-0023] "~110" → EXACT = 79** (full S1-gate rule).

Method: `git ls-files '*Cargo.toml'` → 728 tracked manifests. Parse `[package].name`; exclude 1 virtual/workspace-root (no `[package]`), 1 `.claire/worktrees/` dedup copy, 1 non-`oya-*` first-party (`registry-drift`). Canonical unique first-party `oya-*` crates = **725**. A crate is a violation iff its last dash-token ∉ ALLOWED_ROLES(13), AFTER the gate's three carve-outs cross-checked against `oya-governance-predictable-naming-kernel::check()` (lib.rs:164) + ALLOWED_ROLES (lib.rs:32):
- `is_check_family` — `oya-check-*` (72 crates exempt),
- `is_backend_qualified_adapter` — `*-adapter-<backend>`, backend ∈ BACKEND_SUFFIXES(9) (22 crates),
- `is_doctrinal_carve_out` — `oya-tooling-agent-read` (1 crate).

Two transparency numbers vs the plan's "~110":
- **79** = the number the FULL S1 gate emits (recommended replacement for F-0023).
- **102** = if ONLY `oya-check-*` is exempt and the backend-adapter + doctrinal carve-outs are NOT honored (narrow literal reading). The 23-crate gap = 22 backend-qualified adapters + 1 doctrinal carve-out, which `check()` explicitly does NOT flag.
- The plan's "~110" is an over-estimate in BOTH readings.

Top violating last-tokens: `-contract` 11, `-evidence` 10, `-service` 6; `-coverage/-emission/-manifest/-policy/-scaffold/-selfhosted/-sqlx` 2 each; 45 singletons (`-billing, -cost, -identity, -flags, -s3, -kafka, -stripe, -openai, -openbao, -valkey, …`). No snake_case/uppercase trailing-segment violations (0). No `-core/-runtime/-port/-api-contracts` last-tokens.

Caveat on gate fidelity: `check()` actually evaluates the *declared* `role` (catalog/Cargo metadata) and raises `RoleMismatch`/`UnknownRole`/`UndeclaredRole` against it; the trailing dash-token is the *inferred* role for the mismatch comparison. The 79 figure is the pure trailing-segment (BNF layer-suffix) count, which is precisely what F-0023 measures. Crate-level violation count is unchanged by also-declared-bad roles.

**Burndown commitment (ADR Consequence C2):** baseline the 79 with `mode:baseline-block-on-new` (FROZEN/shrink-only at import); burn down to ZERO before L1 office. Tracked as the burndown line in this artifact: **BNF-debt = 79 → target 0 by L1 office.**

---

## 5. Confirmed cut + floor

**CANONICAL-BLOCKING (target set, mapped to §2.5 + reuse-shape):**
| S | discipline | §2.5 | shape | floor vs backfill | oracle |
|---|---|---|---|---|---|
| S1 | BNF layer-suffix (`predictable-naming-kernel::check()`) | #4 | (a) | **FLOOR (pre-M-lane)** | unit-diff |
| S2 | manifest-hygiene (net-new predicate) | #7 | (c) | **FLOOR (pre-M-lane)** | end-to-end |
| S3 | cargo-prefix (`cargo-prefix-domain::validate_cargo_prefix`) | ADR-0017/#4-adj | (b) | **FLOOR (PROMOTED — load-bearing today)** | unit-diff |
| S4 | retired-vocab → into live brand-residue | #11 | (a) | post-import backfill | unit-diff |
| S5 | data_class (`oya-check-data-class`) | #12 | (a) | post-import backfill | unit-diff |
| S6 | slot2 registration | #6 | n/a (net-new) | **ADVISORY-until-infra (RECLASSED)** | none possible |

**MINIMAL pre-M-lane FLOOR = {S1, S2, S3}** (revised from {S1,S2}): 3 serial producer folds (`GateInputs` `main.rs:114`, `GATE_IDS:[&str;N]` `lib.rs:462`, `build_gate_baseline` `main.rs:121`) + 3 founder signoff doors. S4/S5 = post-import backfill before L1 office. S6 = advisory-until-infra. S7 (hexagonal import-matrix §2.5#5, pure half = `provider-coupling-kernel` + impure `architecture_boundaries.rs`) and S8 (dependency-seam #8 / vendor-license #9 family) backfill before L1 office under their own ADR.

**Nothing mis-classified that survives scrutiny except:** (1) S3 must be in the floor (load-bearing today — deferring it regresses below status quo); (2) S6 cannot be blocking (no detector, no oracle, needs-infra); (3) `oya-check-brand-residue` must NOT be dropped (it is the live gate's kernel — the plan's "DROP brand-residue" headline conflated the *re-port* with the *kernel*; we drop the re-port idea, keep the kernel).

---

## 6. Surprises / contradictions with the plan

1. **The entire oya-dev-cli gate family is effectively un-load-bearing in enforced CI except cargo-prefix.** Of ~115 lanes, exactly ONE (`cargo-prefix` @ `backbone-microservices-ci.yml:305`) is invoked by any workflow — and that workflow is NOT a required status check (only `oya-ci-required` is, per `branch-protection.yaml:55`, which runs the separate cloud-ci gate crates, not dev-cli lanes). This SHARPENS the plan: the migration is mostly net-new firewall wiring, not "replace live CLI checks." Only S3's retirement touches a live consumer (Step-4.0 gh-api hard gate).
2. **11 dev-cli lanes are always-SUCCESS advisory** (authz-tier, tenant-cost-labels, backup-retention, vector-store, olap-tier, wasm-runtime, iac-tier @ mod.rs:869-975 + a11y, i18n, compliance-evidence, realtime-transport @ 1340-1399); 3 are report-only-unless-`--enforce` (dependency-blessed-allowlist, http-stack, workspace-topology); 2 are ADR-0145 DEFERRED-advisory (otel-trace-propagation, audit-chain-seal-coverage); doc-axis is hybrid. Confirms the plan's "must-assert blocking-with-attribution ≠ report-only count" distinction (§Test Plan Observability).
3. **37 of 107 catalog lanes are ORPHANS** — present in `AGGREGATED_VALIDATE_LANES` but with NO same-named backing crate (incl. 14 `cloud-iac-*`, plus `architecture-boundaries`, `cargo-prefix`, `workspace-hygiene`, `api-semver`, etc.). The catalog is a data-only artifact lifted from `scripts/check.sh` with no live dispatcher in this tree. These are aspirational/legacy-script lanes, NOT crates — reconcile into retired.yaml (F3), do not treat as droppable crates.
4. **Two catalog name-aliases point at DROP scaffolds:** `foundry-capability-schema`→`capability-tier-coverage` and `cross-tenant-access-fuzz`→`cedar-coverage`. Both target crates are retired Scaffolded stubs (DROP §3.3). The aliases are orphan catalog strings with no live dispatcher — they do NOT make the stubs load-bearing.
5. **`registry-drift` is NOT a 6th live gate-id** — confirmed it is a `code` (`frozen_empty:true`) folded into `cloud-ci-total-accounting`. Live gate-ids = 5 (re-verified in committed baseline). Matches the plan's count correction.
6. **`oya-check-cost-budget` is a production domain lib misnamed `oya-check-`** (imported at runtime by `oya-intelligence-adapter-domain` + `oya-application-app`). Not a governance gate — advisory + note the misnomer; do not wire as a firewall gate.

---

## 7. Acceptance (§6 STEP-1 rung)

- [x] The 6-item cut CONFIRMED-with-revision (S3 promoted to floor; S6 reclassed advisory-until-infra; brand-residue kernel kept not dropped).
- [x] Every 72 `oya-check-*` + 38 `oya-governance-*` + 1 catalog crate has a disposition (§3); 37 orphan catalog lanes accounted as non-crate/retired.
- [x] Each KEEP names its reuse shape (a/b/c) + equivalence-oracle regime (unit-diff vs end-to-end).
- [x] slot2 has an explicit blocking-vs-advisory ruling (ADVISORY-until-infra).
- [x] Every DROP has a grep/Cargo.lock-proven zero live consumer (§3.3).
- [x] OVERLAP with the 5 live gate-ids flagged (brand-residue kernel = `cloud-ci-brand-residue`; architecture-map-freshness ≈ staleness-reaper, distinct).
- [x] BNF debt recomputed (79, method in §4) with burndown-to-zero target line.
- [x] S1 source confirmed PURE `predictable-naming-kernel::check()` (NOT architecture_boundaries.rs = §2.5#5).
- [x] S2 manifest-hygiene source/shape confirmed (shape c, net-new predicate + partial topology/hygiene reuse, no pure crate owns the field-set).
