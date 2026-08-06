---
title: Architecture-Invariant Register (corpus + tree-wide) — charter robustness lens
status: complete
date: 2026-06-06
lane: ARCH-INVARIANTS (justify-account-robustness)
scope_root: /Users/jasonlee/Developer/source   # the consolidation SOURCE monorepo
mode: READ-ONLY. Every cell cites path + line/verbatim. No file mutated.
extends:
  - ../monorepo-conformance/00-CONFORMANCE-REGISTER.md   # migrant-sibling fit (their lane)
  - ../synthesis/decision-record-oyatie-canon.md         # founder doctrine + D-CONFORM
charter_lens: (a) hyperscaler one-version/visibility-fence/generated-registry · (b) Linus taste no-special-cases/stable-contracts · (c) arch invariants BNF-13/hexagonal/parallel-lanes/min-blast-radius/data_class · (d) ROBUST-NOT-FALSE every claimed gate proven by RED/GREEN that actually BLOCKS · (e) TOTAL ACCOUNTING
coverage_statement: |
  COVERED tree-wide (real source only): oya/ cloud/ libs/ crates/ tools/ = 723 workspace crates;
  registry/catalog/*.yaml = 903 records; registry/quality/lanes.yaml = 97 lanes;
  ADR-0056, ADR-0105 (decision + standard), ADR-0360, ADR-0366; the live enforcers
  (oya-dev-cli architecture_boundaries.rs + data_class_gates.rs), the orphaned
  oya-governance-predictable-naming-kernel, the SCAFFOLD oya-shared-architecture-check-cli,
  and oya-check-layered-architecture-discipline.
  NOT COVERED (explicit, no silent caps): (1) I did NOT execute any gate or `cargo metadata`
  — enforcement reality is judged from source + fixtures + lane wiring, not a live run, so
  "does it BLOCK in CI" is inferred from code+roster, not observed on the farm. (2) Buck2
  visibility counted by grep over BUCK literals, not by `buck2 cquery` (binary not run). (3)
  no_std purity sampled by `#![no_std]` presence in `*-kernel/src/lib.rs` only (136 crates) +
  forbidden-dep grep on `*-kernel/Cargo.toml`; I did NOT build kernels on a no_std target. (4)
  migrant siblings (linux/stack, oyago, oyapy, office, claude, codex) are OUT — owned by the
  conformance register; this lane is the SOURCE tree the siblings merge INTO.
---

# Architecture-Invariant Register — SOURCE tree (oyatie/source)

Per-invariant verdict: **DEFINED?** (is the rule machine-readable + authored) · **ENFORCED?**
(is there a gate that actually BLOCKS, proven by RED/GREEN fixtures, AND is it wired into the
live lane roster) · **ACTUAL-STATE** (violations counted tree-wide). Severity per charter.

The headline distinction this lane forces: a rule can be **DEFINED + have a real fixture-backed
checker + still be UNENFORCED** because the checker is (a) not wired to any lane, (b) wired to the
WRONG check (copy-paste), (c) a pure SCAFFOLD that returns Ok, or (d) report-only. Three of those
four false-enforcement shapes are present here — exactly the founder's "advisory-shell that CLAIMS
to enforce but does not."

---

## REGISTER (one row per sub-invariant)

| # | Invariant | DEFINED | ENFORCED | Severity | Headline |
|---|-----------|---------|----------|----------|----------|
| I1a | BNF 13-layer enum is machine-readable | YES (two copies) | n/a | HIGH | TWO disagreeing enums: standard doc vs naming-kernel |
| I1b | crate terminal token ∈ closed enum (R-002) | YES | **NO** (checker orphaned) | **HIGH** | predictable-naming-kernel real+fixtured but wired to ZERO lanes |
| I1c | catalog role ∈ closed enum (R-064 BLOCKER) | YES | **NO** | **HIGH** | 137 catalog roles outside enum; live gate WHITELISTS them |
| I2a | hexagonal dep-direction / import matrix | YES | PARTIAL (real gate, drifted table) | MED | architecture-boundaries real+fixtured BUT role-table accepts retired roles |
| I2b | ports-in-kernel / impls-in-adapter | YES (R-010/0056) | **NO** | MED | no checker reads code shape; role is catalog-declared, not verified |
| I2c | no_std-kernel | YES (R-006/7/8) | **NO** | MED | 0/136 kernels declare `#![no_std]` |
| I3a | affected-targets parallel build (ADR-0360) | YES | ASPIRATIONAL | LOW | ADR-0360 still `Proposed`, evidence-blocked |
| I3b | one-lane-one-path / conflict-free (ADR-0366) | YES (Accepted) | ASPIRATIONAL | MED | all 6 deliverables `verified_by` gates that do not exist |
| I4a | min-blast-radius: visibility fences | cells DEFINED | **NO** | **HIGH** | 781/832 BUCK targets are `["PUBLIC"]`; 0 PACKAGE files |
| I4b | one-version rule | YES | PARTIAL | MED | 636 inherit `version.workspace`; 66 hardcode a version |
| I4c | cross-microservice refusal (LEAN-A2) | YES | **NO (false-enforcement)** | **HIGH** | lane dispatches `cedar-fragment-coverage` — wrong check |
| I4d | tenant-boundary oya→cloud | YES | REPORT-ONLY | MED | gate prints, never fails |
| I5 | data_class on kernel fields | YES | **YES (genuine)** | LOW | real fixtured gate, live lane; 289-row legacy escape hatch |
| I6a | package.name == dir basename | YES | YES | none | 0/723 mismatch — CONFORMANT |
| I6b | oya-* prefix | YES | YES | none | enforced live by architecture-boundaries |

---

## I1 — BNF 13-layer enum (ADR-0056 / ADR-0105)

### DEFINED — yes, but TWO machine-readable enums that DISAGREE (Linus "good data structure" + hyperscaler "generated-not-hand-maintained" both violated)
- **Enum A (the live enforced one):** `libs/oya-governance-predictable-naming-kernel/src/lib.rs:32` —
  `pub const ALLOWED_ROLES: [&str; 13] = [ "kernel","domain","usecase","app","adapter","infrastructure","cli","rest","grpc","graphql","worker","sdk","api" ]`. A test at `:406-411` locks `len()==13`, asserts `contains("api")`, `contains("usecase")`, `!contains("application")`, `!contains("runtime")`, `!contains("test")`.
- **Enum B (the standard doc):** `docs/standards/layer-enum-adr-0105.md:64-88` — L-001..L-013 = `kernel,domain,usecase,app,adapter,infrastructure,rest,grpc,graphql,worker,cli,sdk,check`. **This list has NO `api` and HAS `check`.** The ADR-0056 decision doc (`ADR-0056-...md:69`) has a 12-value list with `application` (not `usecase`, not `api`, not `check`).
- **Three authored enums, three different membership sets:** {0056 decision}=…application… · {0105 standard}=…+check,−api · {naming-kernel}=…+api,+nothing-named-check (check handled via `is_check_family` prefix at `:52`). The enum is hand-maintained in ≥3 places and has drifted — it is NOT generated from one SSOT. **Charter (c)+(a) violation: the canonical layer enum is not a single machine-readable source.**

### ENFORCED — NO for the parts that matter (I1b, I1c)
- The kernel that DOES validate terminal-token∈enum and suffix==declared-role is `oya-governance-predictable-naming-kernel` (real logic at `:227` `if !ALLOWED_ROLES.contains(&declared.as_str())`, `:234` declared!=inferred mismatch; fixtured tests `:453 api_layer_accepted`, `:459 runtime_role_no_longer_accepted`). It is wrapped by `tools/oya-governance-predictable-naming-app`.
- **But it is wired to ZERO lanes.** `grep predictable-naming registry/quality/lanes.yaml` = no match; not invoked in `oya/developer-sdk/crates/oya-dev-cli/src/commands/gate/run_all.rs`. So the only checker that enforces the closed enum + R-002 (suffix==role) **does not run in the 97-lane roster.** This is the founder's confirmed "22 oya-governance-* crates unwired" pattern, instantiated for the keystone layer invariant.

### ACTUAL-STATE — live violations the unenforced rule would catch
- **74 crate dirs** under `oya/ cloud/ */crates/` carry a terminal token that is NOT in the enum AND is not `oya-check-*` AND is not a sanctioned `*-adapter-<backend>` (BACKEND_SUFFIXES = fake/inmemory/aws/oci/gcp/azure/postgres/redis/sqlite, `naming-kernel:57`). Examples (verbatim dir names): `oya-identity`, `oya-billing`, `oya-cost`, `oya-flags`, `oya-meter` (NO layer suffix at all); `oya-accounting-journal-runtime`, `oya-cloud-iac-runtime`, `oya-hr-employment-runtime`, `oya-payroll-run-runtime` (`runtime` — explicitly removed from enum); `oya-tenant-rbac-application` (`application` — retired by ADR-0106); `*-service` ×6 (`oya-itsm-service-management-service`, `oya-data-warehouse-tenant-olap-service`, …); `oya-authn-device-firmware`, `oya-marketplace-doc-set-scaffold`, `oya-application-shell-frontend-prototype`, `oya-itsm-incident-room`, `oya-itsm-status-update`, `oya-itsm-postmortem`, `oya-itsm-on-call-schedule`; backend-adapters NOT in BACKEND_SUFFIXES: `-adapter-s3`, `-adapter-openbao`, `-adapter-stripe`, `-adapter-adyen`, `-adapter-capi`, `-adapter-cedar`, `-adapter-selfhosted`, `-adapter-openai`.
- **137 catalog records carry roles outside the closed enum** (`registry/catalog/*.yaml`, `grep '^role:' | sort | uniq -c`): `api`×45 (allowed by naming-kernel B, NOT by standard A), `runtime`×55, `application`×23 (retired ADR-0106), `test`×11, `bindings`×3. The naming-kernel test explicitly asserts `runtime`/`application`/`test` are REJECTED — yet 89 such records sit in the catalog because the rejecting checker never runs.
- **Dual-home smell:** both `registry/catalog/oya-audit-chain-application.yaml` and `oya-audit-chain-usecase.yaml` exist; same for `oya-dsr-application.yaml`+`oya-dsr-usecase.yaml`, `oya-foundry-eval-application.yaml`. Migration-in-flight (application→usecase) left both rows live → TOTAL-ACCOUNTING violation (stale rows, no TTL).
- Catalog count 903 vs 723 workspace crates → **~180 catalog records with no matching workspace crate** (orphan/stale) — accounting gap (not separately enforced by architecture-boundaries, which only checks the forward direction crate→record).

---

## I2 — Clean/hexagonal kernel-adapter-app + no_std-kernel/std-adapter

### I2a dependency-direction import matrix — DEFINED yes; ENFORCED PARTIAL (real but drifted)
- The live enforcer is `oya/developer-sdk/crates/oya-dev-cli/src/commands/gate/architecture_boundaries.rs`, dispatched by lane `lean-a1-architecture` (BLOCKER, `lanes.yaml`: `check_command: ... gate validate architecture-boundaries`). **This is a GENUINE gate** — it runs `cargo metadata --no-deps` (`:315`), builds the edge set, and rejects forbidden role edges (`:549 "forbidden dependency edge"`), with RED/GREEN self-tests (`:782 expect_self_test_app_to_app_forbidden`, `:1148 kernel_depending_on_app_is_forbidden`, `:1199 unknown_role_is_rejected`). It is in `run_all.rs` (`"architecture-boundaries"`). GOOD — this one is robust-not-false.
- **BUT three taste/robustness defects:**
  1. **Role source is the catalog `role:` field, NOT the crate-name terminal token.** `load_catalog_role_records` (`:380`) reads `registry/catalog/<name>.yaml` `role:`. There is NO cross-check that suffix==role (that is the orphaned predictable-naming kernel's job). So a crate named `oya-foo-kernel` with catalog `role: app` passes the matrix on its `app` edges — R-002 (`layer-enum-adr-0105.md:105`) is unenforced here.
  2. **The role table has DRIFTED PERMISSIVE and re-admits retired roles.** `allowed_dependency_roles()` (`:56-176`) whitelists `application` (`:63`), `runtime` (`:140`), `test` (`:139`), `bindings` (`:112`), `api` (`:85`) — i.e. it ACCEPTS the very roles the naming-kernel REJECTS. The two gates contradict each other (Linus "no special cases" + hyperscaler one-version both violated). `api` is given an unusually wide edge set incl. `app` (`:85`), so an `api`-role crate importing an `app` crate passes — contradicting ADR-0105 R-051/LAY-SB-025 (sdk/transport must not import app internals).
  3. ADR-0105 R-064 says "unknown layer values as BLOCKER." This gate does reject truly-unknown roles (`:534 "unknown role"`) but only AFTER the table legitimizes 5 non-enum roles, so the BLOCKER fires for `wibble` but not for `runtime`/`application`. **The enum is enforced loosely; the standard's strict closed enum is not.**

### I2b ports-in-kernel / impls-in-adapter — DEFINED (ADR-0056 "Port location: kernel"; R-010) ; ENFORCED NO
- No checker reads code shape to verify a port trait lives in kernel and its impl in adapter. The role is a hand-declared catalog string; nothing parses `trait …` / `impl … for`. The ADR-0105 standard's `crates/oya-dev-cli/src/layered_architecture_gates.rs` cross-ref (`layer-enum-adr-0105.md:21,476`) **does not exist** (`ls` → No such file). The `oya-shared-architecture-check-cli` subcommand `layer-correctness` (which would "check declared layer matches actual code shape") is a SCAFFOLD (see I-FALSE below).

### I2c no_std-kernel — DEFINED (ADR-0105 R-006/7/8: kernel MUST NOT do net/fs I/O or depend on async runtimes); ENFORCED NO
- **0 of 136 `*-kernel` crates declare `#![no_std]`** in `src/lib.rs` (tree-wide grep). The kernels are std crates. The D-CONFORM ruling and conformance register treat no_std as a kernel ideal (item 27); the SOURCE tree itself does not meet it. (The framekernel under linux/stack is the only genuine no_std tree, and it is excluded from this workspace.)
- 2 kernel manifests import async/IO drivers: `cloud/cloud-intelligence/crates/oya-cloud-intelligence-kernel/Cargo.toml` and `oya/ci-controller/crates/oya-ci-controller-kernel/Cargo.toml` reference tokio/sqlx/reqwest — direct R-006/7/8 candidates (verify exact dep at lane time; grep matched the manifest). No gate checks this.

---

## I3 — Parallelizable builds + lanes (ADR-0360 affected-targets, ADR-0366 one-lane-one-path)

### ADR-0360 (affected-targets) — DEFINED, ASPIRATIONAL
- `docs/adr-archive/ADR-0360-ci-pipeline-optimization-program.md:1` `status: Proposed`; `:24` "Evidence-blocked: every throughput/cache-hit/latency claim stays `blocked_until_required_evidence_is_green` until measured on the CI farm." `:34` the `oya verify --affected` mode is described as ADDITIVE and not yet built; `--ci-required` "remains the authoritative whole-workspace mirror." So affected-target precision is a DESIGN, not a running capability. Honest about it (no false claim) — but it is aspirational, so the "parallelizable build" invariant is not realized in the source today.

### ADR-0366 (self-enforcing pipeline / one-lane-one-path) — DEFINED Accepted, ENFORCEMENT ASPIRATIONAL
- `ADR-0366-...md:1` `status: Accepted`. All six deliverables D1–D6 (`:19-42`) cite `verified_by: oya gate validate {concurrent-safe-paths|merge-queue-health|self-repair-coverage|definition-of-done|error-budget-policy|dora-metrics}`. **NONE of those six gate names appears in `registry/quality/lanes.yaml` (97 lanes) NOR in `run_all.rs`.** So an Accepted door:one-way ADR claims 6 verifying gates, zero of which exist. This is a ROBUST-NOT-FALSE flag: the ADR's `verified_by` contract is unbacked. The conflict-free "one-lane-one-path" ownership-sharding (D1 concurrent-safe-paths) is the load-bearing parallelism invariant and it has no live gate.
- NOTE the irony vs charter (d): ADR-0366 EXISTS precisely to prevent "thin scaffolds without substance," yet its own verification surface is a thin scaffold.

---

## I4 — MINIMAL SHARED BLAST RADIUS (the founder's central arch invariant)

### I4a visibility fences — cells DEFINED, fence NOT USED (HIGH)
- `.buckconfig` defines cells (`root/prelude/toolchains/none/third-party`) and a target-platform detector — the buck2 substrate for visibility EXISTS.
- **But the fence is a no-op:** `grep 'visibility =' oya cloud libs` → **781 targets `visibility = ["PUBLIC"]`** vs **51 scoped (non-PUBLIC)** entries, and **4 `visibility = []`**. ~94% of all targets are globally visible. There are **0 `PACKAGE` files** (no package-level visibility default to fence a directory subtree). The Google/Meta "visibility fence = minimal blast radius" pattern (charter a) is configured-but-unused. Any crate can depend on any crate as far as Buck2 visibility is concerned; coupling is unbounded at the build-graph layer.

### I4b one-version rule — DEFINED, ENFORCED PARTIAL
- 636 members inherit `version.workspace = true`; **66 hardcode `version = "x.y.z"`** (e.g. `oya/workflow-engine/crates/oya-workflow-engine-state-machine-kernel/Cargo.toml`, `oya/identity/crates/oya-identity/Cargo.toml`, `oya/intelligence/crates/oya-intelligence-attribution-kernel/Cargo.toml`). The one-version invariant is ~91% adhered but not gated to 100% (no lane asserts `version.workspace`). Charter (a) one-version is partially false-enforced (relies on convention).

### I4c cross-microservice refusal (LEAN-A2) — DEFINED, FALSE-ENFORCEMENT (HIGH)
- Lane `lean-a2-bounded-contexts` (`lanes.yaml`): `severity: BLOCKER`, `source: ADR-0056`, `purpose: microservice-isolation — no cross-µservice deps except via workflow/ontology`. Its `check_command:` is **`cargo run -p oya-dev-cli -- gate validate cedar-fragment-coverage`** — a Cedar-policy-fragment coverage check that has NOTHING to do with cross-microservice dependency isolation. **This is a verbatim copy-paste false-enforcement:** a BLOCKER lane named for bounded-context isolation runs an unrelated check and will go green regardless of cross-µservice import violations. The cross-microservice-refusal invariant (the core of "bounded contexts / minimal coupling") is therefore UNENFORCED while CLAIMING BLOCKER status. (The ADR-0056 §"Clean architecture CI enforcement matrix" `:211` lists `oya-shared-bounded-contexts-check-cli (LEAN-A2)` as the intended enforcer — that CLI is not the wired check.)

### I4d tenant-boundary (oya→cloud, the D-LAYER substrate/product seam) — DEFINED, REPORT-ONLY
- `architecture_boundaries.rs:558-599`: computes every `oya/`-crate → `cloud/`-crate edge; if any exist it PRINTS them and does NOT fail (`:591 "REPORT-ONLY"`). Fixtured at `:1393 tenant_boundary_oya_to_cloud_dep_is_report_only` asserting `errors` is empty + count==1. So the dogfood substrate↔product boundary (D-LAYER) is advisory; a regression adding oya→cloud coupling will not block.

---

## I5 — data_class on kernel fields — DEFINED + GENUINELY ENFORCED (the one robust exemplar)
- `oya/developer-sdk/crates/oya-dev-cli/src/data_class_gates.rs` walks every `*-kernel` member's `src/**.rs` (`:61` `if !crate_name.ends_with("-kernel")`), parses struct fields, and calls `validate_data_class_fitness` (kernel `libs/oya-check-data-class`). Lane `oya-governance-data-class` is `status: active`, `source: ADR-0008`, in the foundation roster. This is a real fail-closed fitness function. ROBUST-NOT-FALSE: PASSES.
- **Caveat (TOTAL-ACCOUNTING):** the escape hatch `registry/data-class/legacy-unannotated-fields.tsv` has **289 rows** — 289 kernel fields grandfathered as unannotated. The gate is real but ~289 fields ride an allowance ledger with no visible TTL/burn-down here; worth a sunset check (not a false-enforcement, but an accounting debt).

## I6 — package.name==basename + oya-* prefix — CONFORMANT
- **0/723** `package.name` ≠ dir-basename mismatches (corrected tree-wide scan over oya/cloud/libs/crates/tools). CONFORMANT.
- oya-* prefix is live-enforced by `architecture_boundaries.rs:461` (fixtured `:1169 missing_oya_prefix_is_rejected`) on the real BLOCKER lane. CONFORMANT.

---

## CONFIRMED FALSE-ENFORCEMENT EXHIBITS (charter d — extend, do not re-derive)

These EXTEND the founder's named live exhibits (0363 "Foundry eradicated", 0511→0513 missing supersession, dup-0377, prd-axis/diataxis defined-not-active, 22 oya-governance-* unwired) with the arch-invariant lens:

1. **`oya-shared-architecture-check-cli` is a pure SCAFFOLD.** `libs/oya-shared-architecture-check-cli/src/main.rs:42-74`: ALL 7 subcommands — including `DependencyDirection`, `LayerCorrectness`, `NamingCollision`, `MetadataSchema`, and `Report` — print `"… : SCAFFOLD (populated in Shard 1)"` and `return Ok(())`. It enforces NOTHING. This is the ADR-0056-named LEAN-A1 orchestrator (`main.rs:7 "LEAN-A1: Clean Architecture enforcement orchestrator"`); the real work was moved into `oya-dev-cli` but the scaffold crate still ships as a green-returning shell. A lane pointed at it would pass vacuously.

2. **`oya-check-layered-architecture-discipline` does NOT check the ADR-0056/0105 layer architecture.** Despite the name (and lane `oya-governance-layered-architecture-discipline`, `severity: blocker`), `libs/oya-check-layered-architecture-discipline/src/lib.rs:1` enforces **ADR-0148/0182/0183/0184** — Cilium-vs-Istio / gateway-vs-mesh / Cedar-vs-Kyverno / Valkey-vs-Memcached mesh-config overlap. It is a *deployment-topology* checker wearing a *code-layering* name. (It IS fixtured + real for ITS domain — `:427-581` RED/GREEN — so not vacuous; it is a NAMING false-affordance: a reader/auditor sees "layered-architecture-discipline BLOCKER active" and reasonably concludes the hexagonal layer enum is gated. It is not.) Its `tests/` dir is empty (`.gitkeep` only) — all tests are inline, so the ADR-0105 R-055/LAY-SB-033 "fixture file per refused shape" convention is met inline but the external fixtures dir is a stub.

3. **`lean-a2-bounded-contexts` BLOCKER lane runs `cedar-fragment-coverage`** (I4c) — wrong-check copy-paste; cross-µservice isolation unenforced.

4. **ADR-0366 (Accepted, door:one-way) cites 6 `verified_by` gates that do not exist** (I3b) — unbacked verification contract on an Accepted ADR.

5. **The orphaned strict enforcer:** `oya-governance-predictable-naming-kernel` is real + fixtured but wired to zero lanes (I1b) — the one checker that would catch the 74 non-enum suffixes and 137 non-enum catalog roles is dark.

---

## ENFORCED vs ASPIRATIONAL — bottom line

**Genuinely ENFORCED (real fixtured gate, wired to a live lane, fail-closed):**
- data_class on kernel fields (I5) — the robust exemplar.
- package.name==basename + oya-* prefix (I6) — via architecture-boundaries.
- dependency-direction role-edge matrix (I2a) — REAL and fixtured, BUT with a drifted-permissive role table that re-admits 5 retired/non-enum roles and reads role from catalog not crate-name (so it is enforced *loosely*, not to the standard's strict closed enum).

**DEFINED-but-NOT-ENFORCED / FALSE-ENFORCED (the charter's target set):**
- closed 13-layer enum on crate-name suffix + catalog role (I1b/I1c) — orphaned checker, 74+137 live violations, 3 disagreeing enum copies.
- ports-in-kernel/impls-in-adapter code-shape check (I2b) — no checker; cited file absent.
- no_std-kernel (I2c) — 0/136 kernels.
- cross-microservice refusal / bounded-context coupling (I4c) — BLOCKER lane runs the WRONG check.
- visibility fences / minimal blast radius (I4a) — 781/832 targets PUBLIC, 0 PACKAGE files.
- one-version (I4b) — 66 hardcoded versions, no gate.
- tenant-boundary oya→cloud (I4d) — report-only.

**ASPIRATIONAL (honestly labeled, not yet real):**
- ADR-0360 affected-targets (Proposed, evidence-blocked).
- ADR-0366 one-lane-one-path + 6 self-enforcement gates (Accepted, gates absent).

**Net charter judgment:** the SOURCE tree's architecture invariants are **strong on the cheap-to-verify mechanical ones (naming prefix, basename, data_class) and weak-to-false on the expensive structural ones that actually bound blast radius** — the layer enum, bounded-context coupling, and visibility fences. The single most dangerous pattern for the imminent consolidation is that two BLOCKER-labeled architecture lanes (LEAN-A1 scaffold path via shared-architecture-check-cli; LEAN-A2 cedar-coverage copy-paste) and one mis-named BLOCKER (layered-architecture-discipline) give a **false green** on exactly the hexagonal/bounded-context invariants the conformance register (its GAPS §3 items 1,3) already says the migration lanes need. The conformance register asked the ralplan to ADD these gates for the migrants; this lane shows the SOURCE itself does not yet enforce them, so "merge green" today does not mean "layer-conformant" even for the existing 723 crates.
