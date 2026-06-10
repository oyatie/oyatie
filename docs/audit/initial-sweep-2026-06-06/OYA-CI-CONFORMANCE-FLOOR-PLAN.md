# OYA-CI CONFORMANCE-FLOOR PLAN

> **STATUS: pending-approval** · **door:one-way** · ralplan(--deliberate) consensus loop · PLANNER REVISION pass (post Architect+Critic ITERATE/SPLIT)
> **Mode:** DELIBERATE (high-risk: refactor of the single live required check on `dev`)
> **Authored against:** `source` @ `cleanup/whole-tree-2026-06-07` HEAD `ca5e5efe5` (re-verified SOURCE-FORCED this pass)
> **Scope:** DESIGN ONLY. Mutates nothing in `source`. The only write is this doc (in the `linux` audit dir).
> **This is the MIGRATION-CRITICAL document.** It is the config-driven conformance FLOOR that unblocks the migration (Task #7/#55). It is split out from the product north-star (see forward-pointer below) so it can be signed off CLEANLY and FAST. Everything in here is decision-ready.
>
> **Directive (founder):** "ci is a product itself … it can't be hermetic for us once and not work on other projects or be of little value to others." This FLOOR plan delivers the migration-prerequisite half of that: turn the Phase-0 firewall+gates+producer from an oyatie-hardcoded conformance system into a CONFIG-DRIVEN, PORTABLE, DOCUMENTED engine — WITHOUT touching the proven ratchet/firewall/registry-drift/sign-off engine and WITHOUT ever pushing RED to `dev`.
>
> **SPLIT note (this revision):** This document was split from the former single `OYA-CI-PRODUCTIZATION-PLAN.md` per the Architect+Critic consensus (MF-6). It carries §1–§3 (the engine-vs-policy seam, the FLOOR config schema, the config-loader, Stages 0–3+D), the byte-for-byte backward-compat invariant + acceptance, the floor RALPLAN-DR, and the floor ADR. The product workstreams (§WS-D third-party gate SDK, §WS-E cloud control-plane, §WS-F hermetic exec backends, §WS-G reproducibility/dev-env, §WS-H dep-bot, §WS-I repo-automation-bot suite, and R-1..R-5 / OQ-1..OQ-20) now live in a SEPARATE, NON-door:one-way product doc — see the forward-pointer.

---

> ### FORWARD-POINTER (one paragraph) → the product north-star
> The broader productization (third-party gate SDK, cloud control-plane, hermetic build backends buck2|bazel, reproducibility/dev-env/versioning, the dependabot-equivalent, and the 19-bot repo-automation suite) lives in **`OYA-CI-PRODUCT-ARCHITECTURE-PLAN.md`** (same dir). That doc is **pending-design, NOT door:one-way** — it is the captured product north-star, to be ratified workstream-by-workstream, each as its OWN future ralplan/campaign, **all gated AFTER this floor + the migration, and NONE ever blocking them**. This floor never depends on that doc. The migration-unblock checkpoint is end-of-Stage-3+D, in THIS document, and stands alone.

---

## RALPLAN-DR SUMMARY (read first) — FLOOR scope only

### Principles (5)
1. **The engine is generic; only policy is oyatie.** The ratchet (compare-mode + ratchet-invariant), the `Finding{code,key}` / `evaluate_keyed(&Value)->BTreeSet<Finding>` contract, `build_gate_baseline`, registry-drift byte-parity, the `gate_registration` meta-test, and the one-way sign-off door are ALREADY policy-free. The floor extracts the policy that still hides in `const`s + producer `collect_*`, and changes NOTHING about the proven engine.
2. **Config is DATA, validated by a closed schema** — honoring the existing doc-as-data doctrine. The exemplars already exist on disk: `unit-class-policy.json`, `ttl-policy.json`, `gate-disposition.json` are `include_str!`-embedded JSON parsed by `Policy::from_strs` / `build_gate_baseline`. The floor promotes these from compiled-in to repo-rooted config, and adds the naming/vocab/manifest tables alongside them.
3. **Backward-compat green-invariant is the supreme acceptance test.** oyatie's own config MUST reproduce TODAY's faces byte-for-byte (counts in §1, re-verified this session). Nothing RED is ever pushed to `dev`.
4. **Zero-config does something useful; full-config does everything.** A repo with no `oya-ci.toml` gets sane defaults (language-agnostic gates on, an empty-but-present policy). Adding config widens enforcement. Valuable on first contact.
5. **The floor never blocks the migration, and the product never blocks the floor.** The conformance floor {Stage 1, Stage 2, Stage 3} is the migration prerequisite (Task #7/#55). It is staged so the floor lands config-driven AND the migration proceeds. All product workstreams are out of this doc by construction.

### Decision Drivers (top 3)
- **D1 — Live-green preservation.** `oya-ci-required` is the single required check on `dev`. Any refactor that flips it RED mid-flight is unacceptable. This dominates sequencing.
- **D2 — Portability with shared value.** Config must let ANOTHER repo adopt oya-ci, but not be so free-form that every repo is bespoke and the product has no shared spine (defaults + a closed schema + a gate catalog are the shared value).
- **D3 — Minimal churn to the proven engine.** The ratchet+firewall+drift machinery is RED/GREEN-proven and is the crown jewel. Touch it as little as possible; route config in through the producer's input-building, not by rewriting gate evaluators.

### Viable Options (overall FLOOR approach)

| | **Option A — Big-bang: full config-loader + all policy now** | **Option B — Incremental: config-extract the floor gates first, broaden later** ⭐ RECOMMENDED | **Option C — Separate `oya-ci` repo extracted now** |
|---|---|---|---|
| Shape | New `oya-ci-config` crate; ALL policy (`const`s + producer paths + vocab + manifest set) moved to `oya-ci.toml` in one campaign; then re-land config-driven. | Land the config crate + schema; migrate ONLY the policy the in-flight floor needs (naming prefix+enum+carve-outs, manifest field-set, vocab stems, enabled-gates+dispositions) to config, proving the byte-for-byte invariant gate-by-gate; defer the full gate-pack/roots/ttl/owners externalization to a follow-on; floor lands config-driven, migration proceeds. | Physically split the engine into a publishable `oya-ci` repo/crate now; oyatie consumes it as a dependency + ships its config. |
| Pros | One coherent landing; product story complete in one go. | Smallest blast radius per step; live-green invariant proven incrementally; unblocks the migration fastest. | Cleanest product boundary; forces the policy/engine split to be real. |
| Cons | Large simultaneous diff over the live required check = highest risk to D1; long-lived branch; hard to bisect a byte-parity break. | Two landing waves; the "full product" is a documented follow-on (in the product doc), not done day one. | Premature: cross-repo release plumbing + versioning before the seam is even proven; highest coordination cost while the migration is in flight; violates D3. |
| Risk to D1 | HIGH | LOW | MEDIUM |
| Verdict | Rejected (see invalidation) | **Chosen** | Deferred to the product doc (a later, separate campaign), not now |

**Why B over A:** A's value is "done in one go," but its cost is a large diff over the single live required check — directly hostile to D1, the dominant driver. B reaches the same FLOOR end-state but proves the byte-for-byte invariant one gate at a time, so any parity break is bisectable to a single small change. B also unblocks the migration soonest.

**Why C is deferred (not chosen):** C is the right *eventual* shape (a published `oya-ci` others depend on), but doing it now adds cross-repo release/versioning plumbing on top of an unproven policy/engine seam while the migration is mid-flight. Publishable-boundary extraction is a product-doc workstream (it consumes the seam THIS floor proves in-monorepo). Adopting C now would be scope-creep that blocks the migration.

### Pre-mortem (4 FLOOR failure scenarios + mitigations)
- **PM-1 — Config refactor breaks the live green.** The producer's `collect_bnf_layer_suffix` / `collect_manifest_hygiene` change behavior under config and the baseline no longer reproduces 79/233 → `oya-ci-required` goes RED on `dev`. *Mitigation:* every step ships behind a "oyatie-config == bundled-defaults" equivalence test that diffs the regenerated faces against the committed faces BEFORE any `const` is deleted; registry-drift already byte-diffs committed==regenerated and stays in the gate set the whole time; land config plumbing with the OLD `const`s as the config DEFAULT first, flip to file-loading second, delete `const`s last.
- **PM-2 — Config so flexible it has no shared value (every repo bespoke).** If the schema is open-ended, two adopters share nothing and the "product" is just a config parser. *Mitigation:* the schema is CLOSED (unknown keys are an error, mirroring `Policy::from_strs` strictness); the gate CATALOG + DEFAULTS are the shared spine; language-agnostic gates ship enabled by default; only the *tables* (prefix/enum/vocab/manifest-set/roots) are per-repo.
- **PM-3 — Engine still leaks oyatie paths/roots after "extraction."** The producer hardcodes `specs/masterplan.json`, `specs/root-hub-pointers.json`, `docs/decisions/`, `docs/governance-lanes/...`, the `oya-` filter (main.rs:296, 360), `oya-governance` enforcement scan (main.rs:587). If any survive, the engine is not portable. *Mitigation:* a dedicated audit step greps the producer + gate crates for every literal oyatie path/prefix and asserts each is either (a) sourced from config, or (b) explicitly a default value in the config schema; the "fresh-repo smoke test" (a tiny non-oyatie fixture repo) is an acceptance gate — it must produce a non-erroring baseline with NO oyatie paths present.
- **PM-4 — Productization scope-creep blocks the migration.** "Make it a real product" balloons into the separate-repo + non-Rust packs + full externalization before the floor lands, stalling Task #7/#55. *Mitigation:* this FLOOR doc scopes Stages 0–3+D to ONLY what the migration needs; ALL product workstreams are in a separate, explicitly-LATER, non-door:one-way product doc that NEVER blocks the floor or the migration; the floor-lands-config-driven milestone (end of Stage 3+D) is the migration-unblock checkpoint.

### Expanded Test Plan (DELIBERATE — FLOOR lanes)
- **Unit:** config-loader parses each table; closed-schema rejects unknown keys; defaults materialize when a section is absent; `Policy::from_config` == `Policy::from_bundled` on oyatie's config (the existing `from_strs` tests are reused/extended). Hyphenated code keys (e.g. `forbidden_oya-vcs`) round-trip through the closed schema unchanged (see §3.5 / MF-7).
- **Integration (the KEY acceptance — backward-compat green-invariant):** run the producer with oyatie's checked-in config and assert the regenerated `gate-baseline.generated.json` is **byte-identical** to the committed one — i.e. bnf=79, manifest-hygiene=233, total-accounting=48633, cross-artifact=168, automation-ratchet=153, staleness=64, brand-residue=4494 keys, all unchanged. registry-drift byte-parity holds. **(MF-1 acceptance, MANDATORY):** after the `GATE_IDS` array is deleted and the gate set is `cfg.gates.enabled`-driven, EACH of the three input KINDs (§3.5) reproduces byte-for-byte — specifically: brand-residue (4494 keys, sourced via its **raw-corpus-collector** binding, NOT `evaluate_keyed`) AND the `registry_drift` CODE under `cloud-ci-total-accounting` (a **frozen-empty** code stamped via the disposition join, NOT in `GATE_IDS`, NOT a face) both reproduce exactly, alongside the 6 producer-face gates.
- **e2e:** the full `oya-ci-required` workflow stays GREEN on a `dev`-like checkout through each step; the `gate_registration` meta-test still enumerates all gate ids; a "fresh-repo smoke test" — a minimal non-oyatie fixture repo with its own `oya-ci.toml` — produces a valid baseline that names ZERO oyatie paths.
- **Observability:** each gate lane keeps its descriptive `matrix.label`; the firewall fan-in verdict prints per-gate counts; config provenance (digest of the loaded config) is stamped into the baseline `_provenance` so a config change is visible in registry-drift.

### ADR block — see [§9 ADR](#9-adr--productize-the-oya-ci-conformance-floor-as-a-config-driven-portable-engine).

---

## 1. CONTEXT (verified this session — paths + counts cited)

The Phase-0 firewall is LIVE and is the single required check on `dev`:

- **Workflow:** `.github/workflows/oya-ci-required.yml` (168 lines). A `strategy.matrix` of homogeneous `cargo test -p <crate>` gate lanes with descriptive `matrix.label`s (lines 88–93), plus bespoke `producer-regen` (line 42), `registry-drift` (line 104), `cloud-ci-firewall` (line 126) jobs, and an `oya-ci-required` fan-in (line 144). **Verified caveat (lines 74–75):** deliberately a matrix, NOT a `workflow_call` reusable workflow, because a called workflow renames published check-runs (`<caller> / <job>`) and would break the `oya-ci-required` required-context name. **This caveat constrains the distribution design (§5).** The reusable-matrix + descriptive-label work has ALREADY LANDED in the live workflow (verified) — this floor references and preserves it; it is not re-done.
- **Producer:** `cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app` — `rust_binary`, `src/main.rs` (967 lines) + `src/lib.rs` (852 lines). Emits faces + `gate-baseline.generated.json` via `build_gate_baseline`. `GATE_IDS` is a hardcoded `[&str; 7]` (lib.rs:462). `GateInputs` (lib.rs:478) + `current_keys_per_gate` (lib.rs:505) build each gate's per-(code,key) map; `build_gate_baseline` (lib.rs:585) joins it with the disposition table.
- **Firewall:** `cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/src/lib.rs` (431 lines) — shrink-only ratchet, two PURE DATA-over-DATA predicates (compare-mode lib.rs:12, ratchet-invariant lib.rs:21), `baseline-block-on-new` blocks only NEW keys, sign-off door (`SignOff`, lib.rs:94) reads `gate-baseline.signoff.json`. **No oyatie policy — fully generic.**
- **registry-drift:** `cloud/cloud-ci/gates/registry-drift/` — committed==regenerated byte-parity test. **(Verified distinction, MF-1):** "registry-drift" the WORKFLOW JOB / directory-scanned gate crate is DISTINCT from the `registry_drift` CODE that appears in the baseline under the `cloud-ci-total-accounting` gate (gate-disposition.json:14, `frozen_empty: true`). See §3.5.
- **Config-as-DATA exemplars (already on disk, `include_str!`-embedded):** `src/unit-class-policy.json`, `src/ttl-policy.json`, `src/gate-disposition.json` (lib.rs:30,32,453). `gate-disposition.json` is per-(gate,code) `mode`/`infra_prereq`/`frozen_empty` — the canonical config-as-data pattern to generalize.
- **One-way door:** `gate-baseline.signoff.json` (founder-signed additions allowlist).
- **Gate ids today:** `GATE_IDS: [&str; 7]` (lib.rs:462, re-verified this session) = `cloud-ci-total-accounting`, `cloud-ci-cross-artifact-agreement`, `cloud-ci-automation-ratchet`, `cloud-ci-staleness-reaper`, `cloud-ci-bnf-layer-suffix`, `cloud-ci-manifest-hygiene`, `cloud-ci-brand-residue`.

**Live baseline key counts (verified from the committed `gate-baseline.generated.json` this session):**
`total-accounting=48633 · brand-residue=4494 · manifest-hygiene=233 · cross-artifact-agreement=168 · automation-ratchet=153 · bnf-layer-suffix=79 · staleness-reaper=64`. **These are the byte-for-byte targets for the backward-compat invariant.**

---

## 2. THE PRODUCTIZATION GAP (the hardcoded oyatie policy — surveyed + cited)

The engine is policy-free; the POLICY hides in three places. Every literal verified this session:

### 2.1 Naming policy — `const`s in the naming kernel
`libs/oya-governance-predictable-naming-kernel/src/lib.rs`:
- `REQUIRED_PREFIX: &str = "oya-"` (line 14)
- `ALLOWED_ROLES: [&str; 13]` — the ADR-0056 layer enum (line 32)
- `CHECK_FAMILY_PREFIX = "oya-check-"` (line 52)
- `BACKEND_SUFFIXES: [&str; 9]` (line 57)
- `DOCTRINAL_CARVE_OUTS: [&str; 1] = ["oya-tooling-agent-read"]` (line 71)

Consumed by the S1 gate `cloud/cloud-ci/gates/oya-cloud-ci-bnf-layer-suffix-app/src/lib.rs::resolve_naming` (line 117) which calls `is_check_family` / `is_doctrinal_carve_out` / `is_backend_qualified_adapter` then `check()`. **All `const` — pure oyatie policy.**

### 2.2 Forbidden-vocab policy — `const`s in the brand crate
`libs/oya-check-brand-residue/src/forbidden_vocab.rs`:
- `FORBIDDEN_VOCAB_STEMS: &[ForbiddenStem]` = foundry/forgejo/jenkins/oya-vcs (line 68)
- `CARVE_OUT_RULES` (the per-file/per-line exemption table — already DATA-shaped, just compiled-in)

### 2.3 Producer-embedded policy — paths, prefixes, manifest field-set, roots
`cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/src/main.rs`:
- `oya-` crate-name filter in `collect_bnf_layer_suffix` (line 296) and `collect_manifest_hygiene` (line 360); `third-party/` exclusion (lines 291, 353).
- The manifest field-set — `ManifestFlags` struct (line 333) + `parse_manifest_flags` (line 386): version.workspace / rust-version.workspace / publish=false / license / lints.workspace / lib / lib.doctest=false. **Cargo/Rust-specific.**
- Repo-root marker `specs/root-hub-pointers.json` in `discover_repo_root` (line 174).
- Reachability sources: `specs/masterplan.json`, `specs/root-hub-pointers.json`, `docs/DOC-CATALOG.md`, workspace `Cargo.toml` members (`resolve_reachability`, line 838).
- Justification source: `docs/decisions/` ADR corpus (`resolve_justifications`, line 902); crosswalk: `specs/masterplan.json` + `specs/master-plan-sequencing.json` (line 504).
- Enforcement scan: `oya-governance*` crates + `docs/governance-lanes/diataxis-doc-class.md` / `prd-axis-coverage.md` + ADR `verified_by: oya gate/oya gen` lines (`collect_enforcement_inputs`, line 587).
- OWNERS resolution: nearest up-tree `OWNERS` file (`resolve_owners`, line 798).
- ttl budgets + unit-class rules: `src/unit-class-policy.json` + `src/ttl-policy.json` (already DATA, embedded).
- enabled-gates + order: `GATE_IDS` (lib.rs:462, hardcoded array).
- **Brand-residue collector:** `collect_brand_residue` (main.rs:257) — see §3.5 (MF-1); this is NOT a producer face, it is a raw-corpus collector that pre-groups `code -> keys`.

### 2.4 Gate language-coupling (verified — drives the gate-pack boundary)
| Gate | Coupling | Pack |
|---|---|---|
| total-accounting | generic (face-shape only) | language-agnostic |
| cross-artifact-agreement | generic | language-agnostic |
| automation-ratchet | generic | language-agnostic |
| staleness-reaper | generic | language-agnostic |
| brand-residue | generic (text scan) | language-agnostic |
| **bnf-layer-suffix** | **Rust/Cargo (naming kernel + Cargo.toml)** | **rust-cargo pack** |
| **manifest-hygiene** | **Rust/Cargo (Cargo.toml field-set)** | **rust-cargo pack** |

The gate EVALUATORS are thin: bnf's `evaluate_keyed` (bnf lib.rs:139) is already a pure `{"rows":[{"crate_name":...}]}` → Findings function. The Cargo/Rust assumption lives in (a) the producer's `collect_*` that BUILDS the face (enumerating Cargo.toml), and (b) the naming-kernel `const`s. **So the language-coupling is in input-construction + the policy tables, not in the ratchet.** This is the load-bearing fact for the config-flow (§3) and gate-pack (§3.4) designs.

---

## 3. THE DESIGN

### 3.1 Config schema (headline)
**Format + location:** `oya-ci.toml` at repo root (single, human-authored, TOML for human-edit ergonomics) — OR `.oya-ci/config.json` if a repo prefers JSON-as-data symmetry with the generated faces. **Decision deferred to founder** (Open Question OQ-1); the loader supports one canonical format, with the schema identical either way. Whatever the format, it is parsed into typed structs and **validated by a CLOSED schema** (unknown keys = error), mirroring the existing strict `Policy::from_strs`.

**Closed schema — every section maps to a surveyed oyatie literal:**

```
[repo]
roots          = ["."]                      # repo-root marker dir(s); replaces specs/root-hub-pointers.json discovery
path_filters   = { exclude = ["third-party/"] }   # tracked-path exclusions (producer collect_*)

[naming]                                     # replaces naming-kernel consts (§2.1)
required_prefix      = "oya-"
allowed_roles        = ["kernel","domain","usecase","app","adapter","infrastructure","cli","rest","grpc","graphql","worker","sdk","api"]
check_family_prefix  = "oya-check-"
backend_suffixes     = ["fake","inmemory","aws","oci","gcp","azure","postgres","redis","sqlite"]
doctrinal_carve_outs = ["oya-tooling-agent-read"]

[vocab]                                      # replaces forbidden_vocab consts (§2.2)
forbidden_stems = [
  { stem = "foundry", code = "forbidden_foundry" },
  { stem = "forgejo", code = "forbidden_forgejo" },
  { stem = "jenkins", code = "forbidden_jenkins" },
  { stem = "oya-vcs", code = "forbidden_oya-vcs" },     # NOTE the hyphenated code key — see §3.5 / MF-7
]
carve_outs = [ { kind = "path_prefix", value = "_legacy-foundry/", reason = "archive" }, ... ]

[manifest]                                   # replaces ManifestFlags field-set (§2.3) — rust-cargo pack
required_flags = ["version_workspace","rust_version_workspace","publish_false","license","lints_workspace"]

[reachability]                               # replaces resolve_reachability sources
sources = ["specs/masterplan.json","specs/root-hub-pointers.json","docs/DOC-CATALOG.md","cargo-members"]

[justification]
adr_dir = "docs/decisions"
crosswalk_specs = ["specs/masterplan.json","specs/master-plan-sequencing.json"]

[owners]
file_name = "OWNERS"                         # nearest-up-tree marker

[enforcement]                                # replaces collect_enforcement_inputs literals
governance_crate_glob = "oya-governance*"
governance_lanes = ["docs/governance-lanes/diataxis-doc-class.md","docs/governance-lanes/prd-axis-coverage.md"]

[ttl]                                        # subsumes ttl-policy.json (already DATA)
# by_unit_class table (carried over verbatim)

[unit_class]                                 # subsumes unit-class-policy.json (already DATA)
# rules table (carried over verbatim)

[gates]                                      # subsumes + relocates gate-disposition.json
enabled = ["cloud-ci-total-accounting","cloud-ci-cross-artifact-agreement","cloud-ci-automation-ratchet","cloud-ci-staleness-reaper","cloud-ci-bnf-layer-suffix","cloud-ci-manifest-hygiene","cloud-ci-brand-residue"]   # replaces GATE_IDS array
# per-gate: input_kind = "producer-face" | "raw-corpus-collector" | "frozen-empty-meta"   (see §3.5 / MF-1)
# per-(gate,code): mode | infra_prereq | frozen_empty  (the gate-disposition.json body, verbatim)
```

**Sensible DEFAULTS (zero-config does something useful):** with NO `oya-ci.toml` present, the loader materializes a default config that enables ONLY the language-agnostic gates (brand-residue with an EMPTY forbidden-stem list = no-op until configured; total-accounting/staleness/cross-artifact/automation with empty source tables = present but quiet), `roots=["."]`, no naming/manifest gates (Rust-specific, off by default), an empty sign-off. Result: a fresh repo gets a valid, GREEN, empty-but-present baseline and a ratchet that activates as the repo fills its config — never an error, never a false RED.

### 3.2 FIXED vs CONFIGURABLE boundary (explicit)

> **MF-4 SCOPING NOTE (this revision):** "FIXED engine" here is scoped to the **ratchet / firewall / registry-drift / sign-off** machinery ONLY. The `Finding` / `evaluate_keyed` / `current_keys_per_gate` SURFACE is FIXED *for the duration of THIS floor plan* (the floor does not touch it — see §3.3), but it is NOT permanently frozen: the product doc's third-party gate-contract workstream (§WS-D) HOISTS `Finding` into a shared `oya-ci-gate-contract` crate and wraps `evaluate_keyed` in a `trait Gate`. That mutation carries its OWN byte-parity proof in the product doc (Stage 4), separate from and after this floor. The floor's byte-for-byte invariant does NOT depend on that future change; this floor leaves the surface exactly as-is.

**FIXED / GENERIC — reused UNCHANGED IN THIS FLOOR (the crown jewels, do not touch here):**
- **the ratchet (permanently fixed):** compare-mode (`current \ baseline`) + ratchet-invariant (growth-only-shrinks) — firewall lib.rs:12,21
- **registry-drift committed==regenerated byte-parity (permanently fixed)**
- **`baseline-block-on-new` semantics; `frozen_empty`; `advisory-until-infra` (permanently fixed)**
- **the `gate-baseline.signoff.json` one-way door (permanently fixed)** (firewall `SignOff`, lib.rs:94)
- **canonical-JSON emission + provenance digest (permanently fixed)**
- the `Finding{code,key}` + `evaluate_keyed(&Value)->BTreeSet<Finding>` contract — *fixed in this floor; MUTATED by §WS-D (product doc, Stage 4) with its own byte-parity proof*
- `build_gate_baseline` mechanism + `current_keys_per_gate` — *the disposition-join + KIND-dispatch is the ONE engine point this floor touches (§3.5); the join/ordering semantics are preserved byte-for-byte*
- the `gate_registration` directory-driven meta-test — *fixed in this floor; generalized to external coordinates by §WS-D (product doc)*

**CONFIGURABLE / POLICY — moves to `oya-ci.toml`:** everything in §2.1, §2.2, §2.3 — naming tables, vocab tables, manifest field-set, roots/path-filters, reachability/justification/owners/enforcement sources, ttl + unit-class tables, enabled-gates + per-(gate,code) modes + per-gate input KIND (§3.5).

### 3.3 Config-loader + config-flow (minimize engine churn)
**New crate:** `oya-ci-config` (a `kernel`-role lib under the naming convention) — pure, I/O-free parse + closed-schema validation into typed structs (`OyaCiConfig { repo, naming, vocab, manifest, reachability, justification, owners, enforcement, ttl, unit_class, gates }`).

**Config-flow decision (the key architectural choice):** config flows through the **PRODUCER**, NOT into the gate evaluators. Rationale, evidence-backed:
- The gate `evaluate_keyed(&Value)` functions are already pure over a face the producer builds. The oyatie coupling is in (a) the producer's `collect_*` (which paths/prefixes to scan) and (b) the naming-kernel `const`s.
- So: the producer loads `OyaCiConfig` once (replacing `Policy::from_bundled()` at main.rs:98 with `Policy::from_config(&cfg)` and threading `cfg` into each `collect_*`), and the naming-kernel `check()` takes a `&NamingPolicy` parameter (sourced from `cfg.naming`) instead of reading `const`s.
- The gate `evaluate_keyed` signatures STAY `&Value -> BTreeSet<Finding>` — UNCHANGED in this floor. The bnf gate keeps reading the kernel but the kernel now takes injected policy (smallest change, keeps the gate a thin projection).
- `build_gate_baseline` reads `cfg.gates` (enabled list + per-gate input KIND + dispositions) instead of `GATE_IDS` + `GATE_DISPOSITION_JSON` — the ONE engine touch-point, designed precisely in §3.5.

This keeps the firewall + ratchet + registry-drift + drift meta-test **byte-for-byte untouched** — they only ever see faces + baselines, never config.

### 3.5 The gate INPUT-BINDING abstraction (MF-1 — the floor-blocker; verified against `source`)

> **This section resolves the consensus CRITICAL finding.** The earlier framing ("loop over `cfg.gates.enabled` resolving each gate's `evaluate_keyed`" / "free generalization of `gate_registration`") was OVER-SIMPLIFIED and is RETRACTED. Verification against `source` shows `current_keys_per_gate` (lib.rs:505–561) is a hardcoded per-name dispatch over **THREE distinct input KINDS**, not one. The config-driven floor must let each enabled gate DECLARE its input KIND, and the two engine loops must dispatch on that KIND while preserving BTreeMap iteration order + the disposition-completeness join.

**The three input KINDS (each verified, with the exact `source` mechanism):**

1. **`producer-face`** — the SIX gates whose keys come from running the gate's pure evaluator over a producer-built face:
   `cloud-ci-total-accounting`, `cloud-ci-cross-artifact-agreement`, `cloud-ci-automation-ratchet`, `cloud-ci-staleness-reaper`, `cloud-ci-bnf-layer-suffix`, `cloud-ci-manifest-hygiene`.
   Mechanism (verified lib.rs:505–555): `out.insert("<gate>", group_findings(<crate>::evaluate_keyed(inputs.<face>).into_iter().map(|f| (f.code, f.key))))`. The face is one of the `GateInputs` fields (`total_accounting`, `cross_artifact`, `automation_ratchet`, `staleness`, `bnf_layer_suffix`, `manifest_hygiene` — lib.rs:478–504). This is the KIND the over-simplified framing assumed was universal — it is only 6 of the 7.

2. **`raw-corpus-collector`** — `cloud-ci-brand-residue`, whose keys are NOT produced by `evaluate_keyed` and are NOT a producer face:
   Mechanism (verified): `out.insert("cloud-ci-brand-residue", inputs.brand_residue.clone())` (lib.rs:560) — the keys arrive ALREADY GROUPED as `&BTreeMap<String, BTreeSet<String>>` (the `GateInputs.brand_residue` field, lib.rs:558). They are pre-grouped by `collect_brand_residue` (main.rs:257) which scans the **raw tracked corpus** through `oya_check_brand_residue::census_findings` / `forbidden_vocab` (the carve-out-aware `code -> keys` builder) — NOT a face, NOT `evaluate_keyed`. The producer comment (lib.rs:556–559) is explicit: "computed from the raw tracked files (not a generated face), so it is supplied already grouped rather than re-derived here."

3. **`frozen-empty-meta`** — the `registry_drift` CODE (NOT a gate), stamped via the disposition table:
   Mechanism (verified): `registry_drift` is a CODE under the `cloud-ci-total-accounting` gate in `gate-disposition.json:14` (`{"mode": "baseline-block-on-new", "frozen_empty": true}`). It is NOT in `GATE_IDS` and NOT a producer face. `build_gate_baseline` (lib.rs:585–645) iterates `for gate in GATE_IDS`, joins each gate against `disp_gates.get(gate)`, and for every `(code, disp)` in the disposition emits the code's keys — but when `frozen_empty == true` it FORCES `keys = Vec::new()` regardless of `current` (lib.rs:617–625). So `registry_drift`'s emptiness is DATA stamped by the disposition join, not produced by any collector. *(Several other codes are also `frozen_empty: true` — e.g. `ratchet_regression`, `duplicate_row_id`, `reap_without_report` under their gates; the `frozen-empty-meta` KIND is the general case of "a code whose keys are stamped-empty by the disposition, not collected.")*
   `gate_registration.rs` (verified lib lines 51, 88, 145) directory-scans `cloud/cloud-ci/gates/*` for any dir with a `Cargo.toml`; it sees the `registry-drift` **crate** (a real gate-crate dir, distinct from the `registry_drift` baseline code) and does NOT see `brand-residue`'s collector as a separate gate (brand-residue's logic lives at `libs/oya-check-brand-residue/`, not as a producer-scanned gate dir). **So the directory-driven discovery and the baseline KIND-set are NOT the same set** — a fact the over-simplified "free generalization of gate_registration" framing missed.

**The config-driven dispatch (how the loops change):**
- Each entry in `cfg.gates.enabled` carries an `input_kind` (one of the three above) plus, for `producer-face`, which face it binds.
- `current_keys_per_gate` becomes a loop over `cfg.gates.enabled` that DISPATCHES on `input_kind`:
  - `producer-face` → `group_findings(<resolved-evaluator>(inputs.<bound-face>) ...)` (the existing path for the 6).
  - `raw-corpus-collector` → insert the collector's pre-grouped `code -> keys` map verbatim (the existing brand-residue path; the collector is bound by config, its output shape is the same `BTreeMap<String,BTreeSet<String>>`).
  - `frozen-empty-meta` → contributes NO `current` keys (the disposition join forces empty); the code exists only in the disposition table under its owning gate.
- `build_gate_baseline`'s emission loop iterates the **config-ordered** enabled gates (preserving the canonical baseline order that `GATE_IDS` fixed today) and, for each, performs the SAME disposition-completeness join (`for (code, disp) in disp_codes`) it does now — including the `frozen_empty` force-empty branch UNCHANGED. **BTreeMap/BTreeSet ordering is preserved end-to-end** (the determinism that makes committed==regenerated hold).
- `GATE_IDS` is deleted (Stage 3); the gate set + each gate's KIND + the disposition come entirely from `cfg`.

**Stage-3 acceptance for MF-1 (MANDATORY, added):** after `GATE_IDS` deletion, the regenerated `gate-baseline.generated.json` is byte-identical to the committed one across ALL three KINDS — explicitly: the 6 `producer-face` gates reproduce their counts (total-accounting=48633, cross-artifact=168, automation-ratchet=153, staleness=64, bnf=79, manifest=233); **brand-residue (4494, via its raw-corpus-collector binding) reproduces byte-for-byte**; and **`registry_drift` (frozen-empty, via the disposition join under `cloud-ci-total-accounting`) reproduces byte-for-byte** (zero keys, `frozen_empty: true` stamped). The canonical baseline ORDER is unchanged. registry-drift byte-parity holds.

### 3.4 Gate-packs (so oya-ci isn't Rust-locked) — floor-scoped
A **gate-pack** is a named set of gates + the input-collectors that build their inputs (each collector declares its input KIND, §3.5):
- **`core` pack (language-agnostic):** total-accounting, cross-artifact-agreement, automation-ratchet, staleness-reaper (all `producer-face`), brand-residue (`raw-corpus-collector`). Collectors operate on tracked text files + ADR/markdown corpus + git history — no language assumption.
- **`rust-cargo` pack:** bnf-layer-suffix + manifest-hygiene (both `producer-face`). Collectors enumerate `Cargo.toml`; consume the naming-kernel + manifest field-set from config.
- **Future packs (product doc, documented follow-on):** `npm-package`, `go-module`, etc. — designed in the product doc, NOT in this floor.

`oya-ci.toml` `[gates].enabled` selects gates across packs; a pack a repo doesn't use simply isn't enabled (its collectors never run). This is the portability mechanism: oyatie enables `core + rust-cargo`; a non-Rust repo enables `core` only.

---

## 4. WORK OBJECTIVES + GUARDRAILS (FLOOR)

**Must Have:**
- `oya-ci.toml` closed-schema config + `oya-ci-config` loader crate.
- The gate INPUT-BINDING abstraction (§3.5) — each enabled gate declares its input KIND; the two engine loops dispatch on KIND preserving order + the disposition join.
- oyatie's checked-in config reproduces TODAY's faces byte-for-byte across ALL THREE KINDS (the §3.5 Stage-3 acceptance).
- The FIXED engine (§3.2, ratchet/firewall/registry-drift/sign-off) is byte-for-byte unchanged.
- `oya-ci-required` stays GREEN on `dev` through every step.
- Floor docs (§6) + updated migration doc.

**Must NOT Have:**
- No change to the ratchet/firewall/registry-drift/drift-meta-test code.
- No change to the `Finding`/`evaluate_keyed` SURFACE in this floor (that mutation is §WS-D, product doc — see §3.2 MF-4 note).
- No RED ever pushed to `dev`.
- No open-ended config (closed schema only).
- No separate-repo extraction in this plan (product-doc workstream).
- No non-Rust packs, no hermetic backends, no cloud plane, no bots in this plan (all product-doc workstreams).

---

## 5. DISTRIBUTION / PRODUCT SURFACE (floor-scoped)

How another project adopts the oya-ci FLOOR:
1. **The config:** drop an `oya-ci.toml` at repo root (the primary surface).
2. **The binary/crate:** the producer + firewall + gate crates, consumed in-monorepo NOW (Option B).
3. **The GitHub Actions surface:** the matrix pattern is DONE and stays (verified ALREADY LANDED in the live workflow). **Honor the verified caveat (workflow lines 74–75):** a `workflow_call` reusable workflow renames published check-runs and breaks the required-context name. So for external repos, ship a **composite action** (`uses:`-able, does NOT rename check-runs) that runs the producer-regen + gate lanes, plus a documented copy-in matrix template — NOT a reusable `workflow_call`. The required-context name `oya-ci-required` is preserved by keeping the fan-in job in the consumer's own workflow.
4. **Examples:** a worked non-oyatie `oya-ci.toml` + the fresh-repo smoke fixture.

**MF-7 — adopter workflow-lane question (stated):** today the `registry-drift` and `cloud-ci-firewall` lanes are BESPOKE jobs (workflow lines 104, 126), distinct from the homogeneous `cargo test -p <crate>` matrix. **Open design question (OQ-FLOOR-2):** do these two bespoke lanes become CONFIG-DRIVEN (declared in `oya-ci.toml` so an adopter inherits them generically) or do they stay copy-in workflow boilerplate in the composite-action template? The floor's position: the matrix lanes are config/template-driven; the firewall fan-in + registry-drift remain workflow-shaped (they are the engine's invocation, not policy), shipped in the composite-action template with the `oya-ci-required` fan-in kept in the consumer's own workflow to preserve the required-context name. Resolution is deferred to founder (recorded in open-questions).

---

## 6. DOCUMENTATION (first-class, founder-mandated) — floor-scoped

Floor docs (location: `docs/oya-ci/` in `source`, generated where possible via the doc-as-data path):
- **Quick-start / adoption** — drop-in `oya-ci.toml`, enable the composite action, see your first baseline.
- **Config reference** — every schema section + defaults (generated from the closed schema so it never drifts: schema → human-readable, doc-as-data), including the per-gate `input_kind` declaration (§3.5).
- **Gate catalog** — each gate, its pack, its input KIND, its codes, its config inputs (generated from the gate registry).
- **Firewall / ratchet mental model** — shrink-only ratchet, baseline-block-on-new, the one-way door.
- **The reusable-matrix pattern** (already landed) + the `workflow_call` caveat (why composite, not reusable).
- **Non-oyatie example repo** walkthrough.
- **Update** `CLI-GOVERNANCE-TO-FIREWALL-MIGRATION-PLAN.md` (`/Users/jasonlee/Developer/linux/docs/audit/initial-sweep-2026-06-06/`, 50KB) to reflect the config-driven floor (the migration consumes config-driven gates).

---

## 7. TASK FLOW (Option B — staged, live-green-preserving) — FLOOR ONLY

| Stage | Step | Acceptance |
|---|---|---|
| **0** | Create `oya-ci-config` crate (loader + closed schema + typed structs, incl. per-gate `input_kind`). Land with oyatie's CURRENT `const`/JSON values encoded as the BUNDLED DEFAULT (no file required yet). | Unit tests: schema parses, unknown-key rejected, defaults materialize; hyphenated code keys (`forbidden_oya-vcs`) round-trip (MF-7). Engine untouched. `oya-ci-required` GREEN. |
| **1** | Thread config into the PRODUCER: `Policy::from_config`, `collect_*` take `&cfg`, naming-kernel `check(&NamingPolicy)`. `const`s become the config DEFAULT (not yet deleted). | **Byte-for-byte invariant:** regenerated baseline == committed (79/233 + all). registry-drift byte-parity. `oya-ci-required` GREEN. |
| **2** | Add oyatie's `oya-ci.toml` (or `.oya-ci/config.json`) encoding TODAY's policy + each gate's input KIND; flip the producer to load the file (default = bundled if absent). Stamp config-digest into baseline `_provenance`. | Same byte-for-byte invariant holds with file-loaded config. Fresh-repo smoke fixture produces a valid empty baseline with ZERO oyatie paths. |
| **3** | Implement the §3.5 INPUT-BINDING dispatch in `current_keys_per_gate` + `build_gate_baseline` (KIND-dispatch over `cfg.gates.enabled`, preserving order + the disposition join). Delete the now-dead `const`s (naming kernel §2.1, vocab §2.2, producer literals §2.3) and **`GATE_IDS`**; introduce the `core` + `rust-cargo` gate-pack abstraction. **Floor lands config-driven → migration unblock checkpoint.** | **MF-1 acceptance (MANDATORY):** after `GATE_IDS` deletion, ALL THREE KINDS reproduce byte-for-byte — the 6 producer-face gates; brand-residue=4494 via its raw-corpus-collector binding; `registry_drift` frozen-empty via the disposition join. No `const` policy remains (PM-3 audit: grep finds zero oyatie literals not sourced from config/default). Invariant still holds. `oya-ci-required` GREEN. **Migration (Task #7/#55) may proceed.** |
| **D** | Documentation (§6) authored alongside Stages 1–3; migration doc updated. **(Stages 1–3 + D = the conformance floor; this group UNBLOCKS the migration.)** | Docs build; config-reference + gate-catalog (incl. input KIND) generated from schema/registry (doc-as-data). |

> **End of the FLOOR.** Everything beyond Stage 3+D (hermetic backends, third-party SDK, cloud plane, reproducibility gate-pack, dep-bot, repo-automation bots) is in `OYA-CI-PRODUCT-ARCHITECTURE-PLAN.md`, each its own gated campaign, NONE a prerequisite for this floor, the migration, or the live green.

---

## 8. SUCCESS CRITERIA (FLOOR)
- oyatie's config reproduces today's faces **byte-for-byte across all three input KINDS** (the supreme acceptance test; §3.5).
- The FIXED engine (ratchet/firewall/registry-drift/sign-off) is byte-for-byte unchanged.
- `oya-ci-required` never RED on `dev` through the refactor.
- A non-oyatie fixture repo produces a valid baseline with zero oyatie paths.
- Floor lands config-driven; migration unblocked (end of Stage 3 + D).
- Floor docs published; migration doc updated.

---

## 9. ADR — Productize the oya-ci CONFORMANCE FLOOR as a config-driven, portable engine

- **Decision:** Extract all oyatie POLICY (naming/vocab/manifest tables, roots/sources, enabled-gates + per-gate input KIND + dispositions) from compiled-in `const`s + producer literals into a CLOSED-schema `oya-ci.toml` loaded by a new `oya-ci-config` crate; introduce the gate **INPUT-BINDING** abstraction (three KINDs: `producer-face` | `raw-corpus-collector` | `frozen-empty-meta`, §3.5) so the gate set + each gate's input source are config-declared; keep the ratchet/firewall/registry-drift/sign-off ENGINE byte-for-byte generic, and leave the `Finding`/`evaluate_keyed` SURFACE untouched in this floor (it is mutated separately by the product doc's §WS-D with its own proof, §3.2 MF-4 note); introduce a gate-pack abstraction (`core` language-agnostic + `rust-cargo`). Land incrementally (Option B), proving a byte-for-byte backward-compat invariant gate-by-gate and KIND-by-KIND.
- **Decision Drivers:** D1 live-green preservation; D2 portability with shared value; D3 minimal engine churn.
- **Alternatives considered:** A (big-bang full extraction now) — rejected: largest diff over the live required check, hostile to D1, non-bisectable parity breaks. C (separate publishable `oya-ci` repo now) — deferred to the product doc: premature cross-repo plumbing on an unproven seam, blocks the migration (PM-4).
- **Why chosen:** B reaches the same FLOOR end-state as A with the lowest blast radius per step, proves the supreme invariant incrementally (and per input KIND, §3.5), and unblocks the migration soonest — directly serving the dominant driver D1 while honoring the founder's product directive. The INPUT-BINDING abstraction (vs the over-simplified "one evaluate_keyed loop") is forced by the verified three-KIND reality and is the floor-blocker MF-1 resolves.
- **Consequences:** the engine becomes a reusable product spine; config is DATA validated by a closed schema (extends the existing doc-as-data doctrine); the gate set + each gate's input KIND become config-declared; the `build_gate_baseline` disposition join + ordering are preserved byte-for-byte; a config-digest enters baseline provenance; ALL broader product work is explicitly out of this door:one-way doc and into the non-door:one-way product doc.
- **Follow-ups:** OQ-1 config format (TOML vs JSON); OQ-FLOOR-2 (MF-7) whether the bespoke `registry-drift`/`cloud-ci-firewall` workflow lanes become config-driven for adopters vs stay composite-action template boilerplate; the publishable-boundary extraction and first non-Rust pack target are product-doc workstreams.

---

*PLANNER REVISION pass complete (2026-06-08). This FLOOR doc was SPLIT from the former single productization plan per Architect+Critic consensus (MF-6); MF-1 (input-binding) is resolved here as the floor-blocker; MF-4 scoping note added (§3.2); MF-7 hyphenated-code + adopter-lane question recorded (§3.1, §5). All product workstreams relocated (not lost) to `OYA-CI-PRODUCT-ARCHITECTURE-PLAN.md`. SOURCE-FORCED verification: `current_keys_per_gate` lib.rs:505–561, `build_gate_baseline` lib.rs:585–645, `collect_brand_residue` main.rs:257, `gate-disposition.json` (registry_drift code line 14, forbidden_oya-vcs line 59), `gate_registration.rs` directory-scan + workflow/needs enforcement, `GATE_IDS:[&str;7]` lib.rs:462 — all re-checked this pass. door:one-way — founder sign-off gates the transition out of pending-approval.*
