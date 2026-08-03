# G011 born-accounting register_crate scaffold — design (ADR-05XX Proposed, two-way door)

Productizes this session's dominant friction: every new-crate PR (#783 sqlx, #779/#780 gates) took ~4 CI
round-trips because born-accounting is a derived join over ~6 hand-authored SSOTs with NO single entrypoint.
Founder doctrine: "manual-twice → automate; productize the pipeline." Architect design on FRESH dev @0e12d0028.

## Key insight
The fleet is ALREADY 90% producer-driven: `oya-cloud-ci-accounting-registry-app` reads a few hand-authored
SSOTs and generates 12/14 gate faces (registry-drift = regenerate+byte-diff). Friction = no orchestrator for
the HAND inputs. Fix = thin typed `register_crate()` composing EXISTING producer bridges (--fix-owners,
--fix-reachability, --next-adr — already exist) + 3 new thin writers + the materialize settle. REUSE not reimpl.
Two-way door, additive, generates NO faces itself (producer stays sole generator).

## AUTHORITATIVE new-crate SSOT checklist (deliverable 1 — reusable now)
For a new crate <cap>/<face>/<name>/:
A. intrinsic: Cargo.toml (name oya- == dir leaf; version/rust-version/lints workspace-inherit; publish=false; license; [lib] doctest=false) [cargo-prefix,manifest-hygiene,bnf-layer-suffix]; BUCK (rust_library/binary + rust_test iff test code) [target-parity]; src.
B. hand-authored born-accounting (the friction surface register_crate writes):
  - OWNERS nearest-ancestor covering dir [unowned]
  - JUSTIFICATION: an ADR `## Governed surfaces` fenced block listing EVERY tracked path the crate adds, VERBATIM no globs [unjustified] (resolve_justifications tokenizes exact tracked-path tokens — main.rs:2899; THIS is task #66)
  - workspace member glob in root Cargo.toml [workspace].members (covers dir + grants cargo-members reachability) [workspace-glob-coverage, unreachable]
  - reachability-registry.json entry for NON-crate paths only [unreachable/staleness]
  - capability-registry.json mapping dir→capability (closed set; absorbs_current_* pre-move) [capability-membership]
  - registry/catalog/<crate>.yaml per specs/catalog/canonical-crate-record-schema.json with valid slo: [catalog-liveness/slo-coverage; doctrine-required]
C. new-SERVICE only (NOT per-crate): per-service manifest.json {tier,tier_subtype,dr_tier(+substrate_dag_position)} (e.g. cloud/tenancy/manifest.json:4-8)
D. auto-satisfied: no_ttl_class (producer auto-classify ttl-policy.json), registry_drift (regenerate), cross-artifact/automation-ratchet/enforcement/brand-residue
E. MANDATORY last: infra/ci/materialize-cloud-ci-generated-faces.sh . + commit faces (else registry-drift RED) + face-settle --verify + firewall GO-LIVE.

## register_crate(req) — req = {crate_dir, capability(HUMAN), owning_adr(HUMAN), owner(HUMAN), role, has_lib, has_test_code, catalog{plane,slo}(HUMAN slo), extra_governed_paths}
Human-decision inputs: capability, owning_adr, owner, slo/tier. Machine-generated: OWNERS write, reachability entry,
member glob, capability mapping, ADR verbatim governed-path list, catalog yaml, faces. (automation APPLIES decisions, never invents — ADR-0548 D2)
Reuse map: OWNERS=--fix-owners(exists main.rs:2697); reachability=--fix-reachability(main.rs:2809); adr#=--next-adr(main.rs:181);
NEW thin writers: member-glob (uses oya-workspace-members-kernel to test coverage), capability-mapping (closed-set validated),
ADR governed-path appender (verbatim, idempotent — closes #66), catalog-yaml (schema-driven). Idempotent upsert; kernel plan = diff vs current.

## Crates: libs/oya-crate-registrar-kernel (pure plan) + libs/oya-crate-registrar-app (I/O bridge). Kernel pure (no clock/rand/net).

## 4 slices (leaf-first)
1. KERNEL (leaf, pure): RegisterCrateRequest→RegistrationPlan (ordered typed edits) + validators (capability∈closed-set, role→suffix, verbatim-path enumeration from a file list). RED/GREEN fixtures, zero I/O. The kernel crate itself needs full manual born-accounting up-front (chicken-egg; front-load it per the checklist).
2. 3 thin writers: capability-mapping, ADR governed-path appender (closes #66), catalog-yaml renderer. pure-plan + tiny apply; reuse to_canonical_json + catalog schema.
3. APP orchestration: wire plan → existing bridges (--fix-owners/--fix-reachability/--next-adr) + member-glob writer + slice-2 writers + invoke materialize; self-validate by running each affected gate evaluate_keyed over regenerated faces, assert GREEN before return.
4. composition: shared primitive lane-verify (ADR-0542 slice 4) calls; neutral-profile conformance (ADR-0548 D4); lane-supervisor wire.

## ADR-05XX (allocate via --next-adr): status Proposed; door TWO-WAY; planning_impact true; deciders founder; owner cloud-ci-platform; depends_on [ADR-0515,0548,0555,0562]; related [ADR-0542,0017,0064,0131,0245,0538,0540,0552]; milestone W1. Precedent: cargo new+workspace-register, Bazel gazelle, Nx/Turbo create-package, Backstage scaffolder, OPA ConstraintTemplate — Rust-reimpl.

## Composition: register_crate = the REUSABLE PRIMITIVE lane-verify (ADR-0542 slice 4) invokes per new crate (lane-verify owns lane policy; register_crate owns per-crate mechanics; no dup). Closes #66 (verbatim paths). #70/#71 compose as further plan steps (confirm scope — local-tracker items, not gh issues).

## Risks: producer-bridge coupling (call LIBRARY fns not subprocess; CLI retirement-marked); catalog/slo defaults (seed+require-human-confirm, slo required field not silent default); capability mis-map (closed-set validator + human-required); settle omission (app invokes materialize + asserts faces GREEN before return); verbatim-path drift (idempotent re-enumerate + lane-verify re-runs per push). OUT: rewriting producers; CRD reconciler (ADR-0548 D3 wraps this kernel later); new-service bootstrap; SLO judgment.

## Key refs (fresh dev): oya-ci.toml (12/14 producer-face); accounting-registry-app/src/main.rs:2489(owners)/2585(reach)/2876(cargo-members)/2899(justif)/2697/2809/181(bridges); lib.rs:285/137/229; ttl-policy.json; registry-drift/src/lib.rs; capability-registry.json (closed+frozen baseline); specs/catalog/canonical-crate-record-schema.json; materialize-cloud-ci-generated-faces.sh; cloud/tenancy/manifest.json:4-8 (tier=per-service).

## SLICE-2 STATUS + doubt-driven catch (2026-06-21)
slice-2 writers landed as PR #787 (agent/bornaccount-registrar-writers). Adversarial review (doubt-driven
cycle 1) caught 6 REAL defects in green-passing code — all in the markdown writer + catalog renderer, NONE
caught by the 34 CI gates (which check structure, not writer LOGIC):
  1. (BLOCKER) locate_block took the FIRST ``` after the heading → hijacked a LATER section's code block,
     deleting that section + crediting its lines as forged governed paths. Fix: line-based locate_section
     confined to the heading's own section (stops at next `#` heading/EOF).
  2. (BLOCKER) locate (substring .find) vs existing_block_paths (exact ==) disagreed on fence recognition →
     info-string fence ```text silently dropped all existing paths on re-apply. Fix: one shared line scanner.
  3. (BLOCKER) catalog_yaml interpolated plane/slo raw → newline/`:` forged YAML keys (fail-open). Fix:
     is_safe_yaml_scalar fail-closed validation (InvalidCatalogField).
  4/6. (MAJOR) verbatim-emit vs trimmed-parse non-idempotent + brace-glob the ONLY path validation. Fix:
     reject newline/fence/whitespace (MalformedGovernedPath).
  5. (MAJOR) find_heading prefix-matched `## Governed surfaces (legacy)`. Fix: full-line is_governed_heading.
Fixed in head b5a037497 (+9 RED→GREEN tests, 30 total). Cycle-2 review verified the rewrite (only residual:
unclosed-fence malformed-input latent — non-blocking).

### PRODUCTIZATION (founder mandate: friction = pipeline failure):
Defects 1/2/4 are IDEMPOTENCY + content-preservation violations. Hand-picked happy-path fixtures gave false
confidence (reviewer's words). A PROPERTY-BASED test — `compute(compute(x)) == compute(x)` and "all input
bytes outside the managed block survive" over GENERATED ADR/registry inputs (proptest, pure-Rust, test-only)
— would have caught 1/2/4 mechanically. DOCTRINE for the pipeline: every idempotent file-mutator/writer in
the born-accounting + lane-verify substrate MUST carry a proptest idempotency+preservation harness, not just
fixtures. This is the universal/hermetic detector for the "writer-logic the structural gates can't see" class.
Candidate: a shared `oya-idempotent-writer-proptest` test kernel (slice-4 / lane-verify ADR-0542 composition).

## SLICE-3 PLAN CORRECTION (read-only Explore on origin/dev, 2026-06-21)
DE-RISK FINDING that overturns the slice-3 "reuse map": the 3 producer bridges are NOT importable library
functions today — they are PRIVATE fns inline in the producer BINARY's main.rs, returning CliError:
  - apply_fix_owners(repo_root,&cfg,&scm_facts,spec)->Result<String,CliError>  main.rs:2697  (writes OWNERS; REFUSES if file exists — seeds only)
  - apply_fix_reachability(repo_root,&cfg,&scm_facts,spec)->Result<String,CliError>  main.rs:2809  (appends reachability registry JSON, canonical)
  - --next-adr allocator: NOT a standalone fn — buried in collect_crosswalk_inputs() main.rs:2114-2149 (computes full crosswalk as side-effect just to get next_free_id)
Producer lib.rs exposes only 11 pub fns (Policy::*, build_registry/decision_crosswalk/enforcement_inventory/gate_baseline, to_canonical_json) — NONE of the 3 bridges. Validation helpers (is_valid_owner_principal, load_reachability_registry, ReachabilityRegistration struct) are also main.rs-private.
materialize: infra/ci/materialize-cloud-ci-generated-faces.sh is the ONLY entrypoint (builds scm-facts-emitter + producer bins via buck2, runs `$producer --repo-root --scm-facts` → all 6 faces; no per-face Rust hook, no programmatic API).
gate self-validation: GOOD NEWS — each gate already exposes `pub fn evaluate_keyed(input:&Value)->BTreeSet<Finding>` in its own crate (total-accounting, cross-artifact-agreement, etc.). Slice 3 imports those gate crates directly; no extraction needed there.

=> REVISED SEQUENCING: insert SLICE 2.5 (precursor PR, two-way door, mechanical, independently valuable):
   "Extract producer bridges to lib.rs" — move apply_fix_owners / apply_fix_reachability + validation
   helpers + ReachabilityRegistration into producer lib.rs as `pub fn fix_owners(...) -> Result<FixResult,ProducerError>`
   / `pub fn fix_reachability(...)`, and extract a standalone `pub fn allocate_next_adr_id(adr_dir:&Path)->Result<String,ProducerError>`
   (NOT via full crosswalk). main.rs CLI handlers become thin wrappers calling the new lib fns (CLI is
   retirement-marked, so the binary keeps working but the LOGIC is library-callable). This is the CLI-retirement
   doctrine applied: logic in the library, CLI a thin transient adapter. THEN slice 3 wires plan→lib fns +
   member-glob writer + slice-2 writers + materialize, self-validating via the gate crates' evaluate_keyed.

## SLICE-2.5 STATUS (2026-06-21)
PR #788 (agent/bornaccount-producer-lib-extract, head 588aa61e): behavior-preserving extraction of the 3
producer bridges → lib.rs pub API. Verified BYTE-IDENTICAL faces (twice), 24 tests, clippy net-zero,
allocator parity (ADR-0569). New API (cross-crate, no subprocess; bridges take tracked_paths:&[String], NOT
the binary-only ScmFacts):
  pub fn fix_owners(repo_root,&cfg,tracked_paths:&[String],spec)->Result<String,ProducerError>
  pub fn fix_reachability(repo_root,&cfg,tracked_paths:&[String],spec)->Result<String,ProducerError>
  pub fn allocate_next_adr_id(adr_dir:&Path)->Result<String,ProducerError>
Adversarial review APPROVE — verified the parts byte-identical-faces does NOT cover (the 2 write-bridges
aren't run during materialize): transcription line-for-line; ScmFacts is single-field so &[String] is lossless;
from_bridge re-wraps Io/Validation/Refused→CliError::Io (stderr byte-identical, since OLD bridges raised only
CliError::Io); Serialize/Policy route back through CliError::Producer preserving prefixes. KEY DOCTRINE LESSON:
"byte-identical faces" only validates code in the FACE-GEN path — write-bridges/side-effect paths need their
own coverage. NON-BLOCKING follow-up (drift-lock, founder automation doctrine): the relocated tests assert on
Debug not Display; add one `assert_eq!(format!("{}", CliError::from_bridge(err)), "io: ...")` to mechanically
lock the stderr byte-identity against future from_bridge edits (do in slice 3 or 4). Also lib/main both keep
private read_text/front_matter_lines (harmless dup; pull to lib later).
Once #788 green→merge, slice 3 wires register_crate app: plan→{fix_owners,fix_reachability,allocate_next_adr_id}
lib calls + capability_mapping/adr_governed_paths/catalog_yaml writers (slice 2) + member-glob writer + invoke
materialize + self-validate via each affected gate's pub fn evaluate_keyed(&Value)->BTreeSet<Finding>.

## DEPENDENCY-DIRECTION RESOLUTION (read-only Explore on origin/dev, 2026-06-21) — changes slice 3 home
DOUBT-DRIVEN catch before building slice 3: the orchestrator must call the producer's slice-2.5 lib fns
(fix_owners/fix_reachability/allocate_next_adr_id) which live in cloud/cloud-ci/gates/. But a
`libs/ -> cloud/cloud-ci/gates/` Cargo edge is a LAYER INVERSION (ADR-0280/0245 tier rules; meta-layer CI
code must not be pulled into a shared libs/ crate; ZERO precedent of any libs/ crate depending on cloud-ci).
=> RESOLUTION (minimal thrash, doctrine-aligned, NO moving of just-merged crates):
   - libs/oya-crate-registrar-kernel (pure plan) STAYS in libs/.
   - libs/oya-crate-registrar-app (the slice-2 PURE writers: capability_mapping/adr_governed_paths/catalog_yaml,
     + the slice-3a member-glob writer) STAYS in libs/ (pure, no producer dep, legal downward consumer).
   - The ORCHESTRATOR (slice 3b, the part that calls the producer) is born as a NEW crate under cloud/cloud-ci/
     (match the producer's location convention). It depends DOWNWARD on libs/registrar-kernel + libs/registrar-app
     + libs/workspace-members-kernel, and SAME-LAYER on cloud/cloud-ci/gates/accounting-registry-app (producer).
     All edges legal. slice-2.5 producer lib API is consumed directly (not wasted).
DECOMPOSED slice 3:
   3a = add the 4th PURE writer (workspace member-glob) to libs/oya-crate-registrar-app — edits root Cargo.toml
        [workspace].members to add the glob covering the new crate dir; uses oya-workspace-members-kernel to test
        coverage; pure compute + thin apply + idempotent + RED/GREEN, exactly like the other 3 writers. Mergeable,
        no architecture risk.
   3b = the cloud/cloud-ci/ orchestrator crate: build CurrentState+CapabilitySet loaders (repo I/O), call kernel
        plan_register_crate, dispatch each Edit→writer/bridge, return structured Outcome. (materialize+self-validate
        via gate evaluate_keyed can be 3b or a 3c.)
LESSON: verify cross-crate dependency DIRECTION against tier/DAG rules BEFORE extracting/placing — slice 2.5
extracted the bridges into the producer's own lib.rs assuming the registrar (libs/) would consume them, but the
libs->gates edge is forbidden; the fix is to place the CONSUMER (orchestrator) in cloud-ci, not to move the bridges.
