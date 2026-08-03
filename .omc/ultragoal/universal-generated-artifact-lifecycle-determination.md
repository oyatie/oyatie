# Universal Generated-Artifact Lifecycle — Synthesis Determination

Status: **Determination (SYNTHESIS)** — merges the HERMETICITY + NO-SHELL PRODUCER facet
into one coherent, founder-grade plan, verified against the as-shipped PR #828 branch
`chore/decommit-generated-faces` (freshness gate `cloud/cloud-ci/gates/oya-cloud-ci-freshness-app/src/lib.rs`
1290 lines read; control-plane gate `oya-cloud-ci-generated-artifact-control-plane-app/src/lib.rs`
read; `registry/generated-artifact-control-plane.json` read; `infra/ci/materialize-cloud-ci-generated-faces.sh`
read; ADR-0551 / ADR-0552 / ADR-0595 read on-branch; `.github/workflows/oya-ci-required.yml`
materialization wiring read).

door: one-way (the policy class "generated artifacts are derived, not committed merge surfaces;
regeneration is manifest-driven, not hand-glued" is a one-way commitment once the no-shell
materializer and the pure-gate refactor land).

Net verdict: **APPROVE with mandatory reshape.** PR #828's *policy* (de-commit pure-derivation
faces; teach the two gates a `not-tracked-in-git` mode) is SOUND and should land as the
**bootstrap**. But #828 as shipped FAILS the 7-property bar on **HERMETICITY** and **AUTOMATED**:
the regeneration path is encoded THREE times (shell script + freshness-gate `Command::new` + CI
YAML `needs:`), the verdict-bearing freshness gate reaches the clock (`SystemTime::now()`), the
pid (`process::id()`), `buck2`, and `git` directly, and the manifest's typed `generator` block is
**decorative** (validated, never consumed). The fix is a universal capability —
**`oya-ci-materializer`** — a neutral, manifest-driven materialization engine (pure planner +
thin impure executor) that all three encodings collapse onto, plus the refactor of #828's gates
into **pure predicates fed by the engine** rather than self-materializing. This is the standard
"rule-based generator with declared inputs/outputs scheduled by the build graph"
(Bazel/Buck genrule + Skyframe analysis→action phases), reimplemented Rust-native — exactly the
proven-patterns-Rust-reimplementation doctrine.

---

## 0. The class, the friction, and why it is universal

Every monorepo that commits generated artifacts hits this class: a generated file is a pure
function of source, the file is committed, every PR touching source rewrites a multi-thousand-line
blob, and PRs collide on it. The committed copy serves **no gate invariant** that derivation does
not serve better (ADR-0595 §8 proved this for oyatie: the firewall already re-derives via the
merge-base materialization; the only consumer of committed *bytes* was the freshness byte-parity
predicate). The universal cure is **derive-don't-commit**, made sound by a **regenerate-twice
determinism canary** (the cold-vs-warm integrity-canary doctrine), with **regeneration driven by
declared manifest data, not hand-glued per repo**.

oyatie's instance: 6 producer faces (~15 MB, prime offender the 11.6 MB accounting registry) under
`cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/`. The same shape recurs in a
TypeScript repo committing `src/__generated__/schema.ts` + `gen/api-client/`, a Go repo committing
`*.pb.go`, a JVM repo committing `generated-sources/`, an SDK repo committing `generated-types/` —
all already matched by the manifest's `generated_path_rules` (verified present: `.generated.ts`,
`.generated.d.ts`, `.pb.go`, `.pb.rs`, `__generated__`, `gen`, `generated-sources`,
`generated-types`).

The friction-as-process-failure rule: do not fix this only in oyatie. Build the **engine + policy
contract + control plane + public adoption path** so any repo gets it.

---

## 1. The public policy contract (schema)

The contract is **already 90% present** in `registry/generated-artifact-control-plane.json` and
its validating gate. The synthesis makes the existing decorative `generator` block **load-bearing**
and adds the consumption-order derivation. NO new top-level surface invented; the schema is an
*extension* of the shipped one (schema_version 1 → 2, additive only).

### 1.1 Per-artifact `generator` block (PROMOTED from decorative to the sole source of truth)

```jsonc
"generator": {
  "runner":            "buck2" | "oya-ci-native-controller" | "<future-owned-runner-id>",
  // canonical-prefixed target the runner resolves to an executable/action:
  //   runner=buck2                     → "//..."        (validated: must start with //)
  //   runner=oya-ci-native-controller  → "oya-ci://generated-artifact-controller/..."
  //   any new runner                   → its own registered canonical prefix (see §1.4)
  "generator_target":  "//cloud/cloud-ci/.../accounting-registry-app:...-bin",
  "operation_id":      "emit-accounting-face",          // stable verb the tool dispatches on
  "parameters":        { "face": "registry" },          // map<string,string>, all non-empty
  "input_contract":    ["repo-root","declared-source-inputs","scm-facts-snapshot"],
  "output_mode":       "stdout-json" | "declared-artifact-path-write" | "controller-materialized"
}
```

`input_contract` is the **DAG edge source**: an artifact whose `input_contract` references an
output another artifact produces (canonically `"scm-facts-snapshot"`, produced by the
`scm-facts-boundary-snapshot` artifact) sequences AFTER it. The engine derives the topological
order from these strings — **no `EMITTER_TARGET`/`PRODUCER_TARGET` constants, no hand-authored
`needs:`**. This is where #828's hardcoded `EMITTER_TARGET`/`PRODUCER_TARGET`/`PRODUCER_FACES`
consts and the shell's two literal `//...` targets all dissolve into data.

### 1.2 `materialization_mode` (unchanged from #828; the de-commit class is `not-tracked-in-git`)

The six modes shipped in #828 stand. `not-tracked-in-git` = pure derivation, intentionally absent
from git, derived on demand. The control-plane gate already EXEMPTS it from
`declared_path_not_tracked` and FORBIDS re-tracking it
(`generated_artifact_not_tracked_path_is_tracked`, the one-way door).

### 1.3 NEW invariant (closes the #828 gap): de-commit class MUST be materializable

Every artifact in `materialization_mode == "not-tracked-in-git"` MUST carry a `generator` block the
planner can lower — i.e. `runner` + `operation_id` + `output_mode` present and a non-`controller-
materialized` output for anything a gate reads as bytes. A de-commit-class row with no way to
regenerate it would be a face that can never be reconstructed: a fail-OPEN hole. New control-plane
finding `generated_artifact_not_tracked_without_materializable_generator`.

### 1.4 NEW: the runner registry (the anti-arbitrary-command guard, §3.5)

A top-level `runner_registry` declares each allowed runner and its canonical target prefix:

```jsonc
"runner_registry": [
  { "runner_id": "buck2",
    "canonical_target_prefix": "//",
    "lowering": "build-target-then-exec",
    "irreducible_glue_adr": "ADR-0523" },
  { "runner_id": "oya-ci-native-controller",
    "canonical_target_prefix": "oya-ci://generated-artifact-controller/",
    "lowering": "in-process-reconciler-call" }
  // adopters add e.g. { "runner_id":"node-codegen", "canonical_target_prefix":"npm://", ... }
]
```

The control-plane gate already enforces canonical prefixes per runner (verified: `validate_generator`
lib.rs:344-362 requires `//` for buck2 and `oya-ci://generated-artifact-controller/` for the
controller). The registry generalizes this: a runner not in the registry is RED; a target not
matching its registered prefix is RED. **There is NO `runner: "shell"` escape hatch** — that would
re-introduce exactly the ambient shell this determination retires.

### 1.5 Derived view: `consumption_order` (NOT authored — computed by the engine)

The engine exposes a pure derivation `materialize_closure(P)` = the transitive set of artifacts that
must be materialized before any consumer reads path `P`, topologically ordered. This REPLACES the
hand-wired CI `needs: producer-regen` and the per-leg re-materialization. It is a *function of
manifest data*, identical on any repo.

---

## 2. The neutral engine (kernel) + its pure predicate

Two crates, libs-split, with a hard purity boundary:

```
cloud/cloud-ci/oya-ci-materializer-kernel/   # PURE. rust_library. dep = serde_json only.
                                             # zero I/O, clock, net, rand, subprocess, git.
cloud/cloud-ci/oya-ci-materializer-app/      # IMPURE EXECUTOR. The ONLY component permitted
                                             # ADR-0523 irreducible-glue (buck2 bootstrap + the
                                             # scm-facts emitter's git boundary). Not a verdict.
```

### 2.1 Kernel — the pure planner (the engine signature)

```rust
// oya-ci-materializer-kernel/src/lib.rs — PURE, deterministic, no ambient inputs.

pub enum MaterializeScope {
    /// Single-pass: materialize each artifact in the closure once.
    Consume { target_paths: BTreeSet<String> },
    /// Determinism canary: build+emit once, run producers TWICE, capture both buffers.
    /// Multiplicity is a STRUCTURAL property of the plan, so no future edit can
    /// reintroduce the double-`buck2 build` race #828 discovered by hand.
    DeterminismCanary { target_paths: BTreeSet<String> },
}

pub enum Runner { Buck2 { target: String },
                  NativeController { target: String },
                  Registered { runner_id: String, target: String } }

pub enum OutputSink { DeclaredPath(String),       // declared-artifact-path-write
                      Stdout,                      // stdout-json
                      ControllerMaterialized,      // controller-owned
                      TwoCapturedBuffers }         // canary leg (a, b compared in-kernel)

pub struct MaterializeStep {
    pub artifact_id: String,
    pub runner: Runner,
    pub operation_id: String,
    pub params: BTreeMap<String, String>,
    pub output: OutputSink,
    pub multiplicity: u8,                          // 1 normal, 2 canary producer leg
}

pub struct MaterializePlan { pub steps: Vec<MaterializeStep> }  // topologically ordered

/// THE ENGINE SIGNATURE — pure analysis phase. Reads ONLY the manifest. No filesystem, no
/// clock, no buck2, no git. Returns Err on a manifest that cannot be lowered (e.g. a
/// not-tracked-in-git artifact with no generator, an unregistered runner, a non-canonical target).
pub fn plan(manifest: &ControlPlane, scope: MaterializeScope)
    -> Result<MaterializePlan, PlanError>;

/// THE VERDICT PREDICATE — pure. Fed materialized bytes by the executor; never materializes.
/// Determinism: byte-compare the two canary buffers. Freshness: for committed-class faces,
/// committed == regenerated; for not-tracked-in-git faces, the determinism canary IS the check.
pub fn evaluate(materialized: &[(ArtifactId, Bytes)],
                committed:    &[(ArtifactId, Bytes)],   // empty for de-commit class
                manifest:     &ControlPlane)
    -> Findings;

/// Derived consumption-order view (the universal CI-ordering source).
pub fn materialize_closure(manifest: &ControlPlane, target_paths: &BTreeSet<String>)
    -> Result<MaterializePlan, PlanError>;
```

The single-build / run-producer-twice invariant — #828's hand-written
`regenerate_faces_twice_with_buck2` ("build tools ONCE + emit scm-facts ONCE + run only the
producer twice, to avoid a double `buck2 build` racing a mid-run rebuild") — becomes a **structural
property of the plan**: in `DeterminismCanary` scope, build+emit steps have `multiplicity: 1` and
producer steps have `multiplicity: 2` with `OutputSink::TwoCapturedBuffers`. The race cannot be
re-introduced by an edit because it is the shape of the data, not a hand-coded loop.

### 2.2 App — the thin impure executor (the no-shell producer contract, §4)

```rust
// oya-ci-materializer-app/src/main.rs — the ONLY impure surface.
//   buck2 run //cloud/cloud-ci/oya-ci-materializer-app -- --scope consume   --paths <P,...>
//   buck2 run //cloud/cloud-ci/oya-ci-materializer-app -- --scope canary    --paths <P,...>
//
// For each MaterializeStep:
//   Runner::Buck2{target}          -> buck2 build <target> --show-output; exec tool with
//                                     operation_id + params lowered to declared call;
//                                     route stdout/file to the declared OutputSink.
//   Runner::NativeController{...}   -> in-process typed reconciler call (no subprocess).
//   Runner::Registered{...}        -> the registered lowering (e.g. node-codegen) — still
//                                     constrained by the runner_registry canonical prefix.
// ALL Command::new(buck2|git|emitter|producer), --show-output parsing, and clock/pid temp-paths
// live HERE and ONLY here. This is the verbatim move of build_face_tools / emit_scm_facts /
// regenerate_producer_faces OUT of the freshness gate.
```

The executor is a **producer**, not a verdict, so its ADR-0523-sanctioned impurity (buck2 bootstrap
= ledger item 1; scm-facts emitter's git = ledger item 2) is allowed. The **gate's** impurity (which
#828 has today) is NOT — and is removed by §6.

---

## 3. Anti-forgery / merge-base anchoring (construction > reaction; not candidate-forgeable)

Four anti-forgery properties, the first two preserving #828's hard-won fixes and the rest closing
the holes the #828 review found:

**3.1 The de-commit exemption keys on the CANONICAL FULL PATH, never the basename.** #828's
`read_decommitted_face_names` (lib.rs) deliberately matches an artifact's `materialization_mode ==
not-tracked-in-git` AND its `path` against the gate's canonical `GENERATED_FACE_PATHS` set, so a
candidate-controlled manifest row at `anything/scm-facts.generated.json` cannot collapse to a
committed face's basename and silently retire its byte-parity check. **This logic MOVES into the
kernel unchanged** — it is exactly the policy the planner needs (which declared paths are de-commit
class). In the universal engine the canonical set is `manifest.artifacts[].path` itself (full paths,
manifest-declared), so the basename-forgery hole is structurally impossible: the engine only ever
matches on declared full paths, never reconstructs a key from a basename.

**3.2 The frozen ratchet reference comes from the merge-base materialization, never the candidate.**
ADR-0551 already anchors the firewall's ratchet on the face *as committed at*
`git merge-base <base_ref> HEAD`, materialized out-of-graph by the scm-facts emitter (the single git
boundary), `base_ref` configurable in `ratchet-policy.json` (policy-as-data, R0). The materializer
engine PRESERVES this: the merge-base baseline is a distinct materialization step (runner=buck2,
the emitter target, with `--merge-base-baseline`) that the firewall consumes. **The ratchet never
reads candidate bytes**, so a PR cannot forge a looser baseline. This directly honors the
gate-baseline PR/push-asymmetry memory: the frozen reference is merge-base-anchored for the RATCHET;
every other predicate (de-commit exemption, determinism, drift) evaluates the CANDIDATE tree.

**3.3 The one-way re-tracking door.** #828's `generated_artifact_not_tracked_path_is_tracked` makes
re-committing a de-committed face a hard RED. KEPT, in the kernel. A future PR cannot silently
revert the de-commit.

**3.4 Determinism is the load-bearing integrity canary.** With byte-parity-to-committed gone for
de-commit-class faces, a non-deterministic producer must HARD-FAIL (regenerate-twice byte-mismatch
→ RED), never silently green. This is the cold-vs-warm cache integrity-canary that makes
derive-don't-commit sound. The canary is a pure in-kernel `a == b` over the two captured buffers.

**3.5 No arbitrary-command runner.** §1.4's `runner_registry` + per-runner canonical-prefix +
no-`shell`-runner rule prevents a malicious adopter manifest from declaring an arbitrary subprocess
generator. The control-plane gate enforces it; the kernel `plan()` returns `Err` for an
unregistered runner or a non-canonical target. (Future hardening: pinned-digest binding per runner.)

---

## 4. The no-shell producer contract (replaces `materialize-cloud-ci-generated-faces.sh`)

The shell script does four impure things: (1) `awk`-parse `rust-toolchain.toml` + `rustup toolchain
install`; (2) `buck2 build <2 hardcoded targets> --show-output` + `awk`-parse paths; (3) exec
`$emitter --repo-root --out`; (4) exec `$producer --repo-root --scm-facts`. The freshness gate
re-implements (2)(3)(4) impurely in Rust. Both, and the CI YAML ordering, collapse onto the
materializer:

- **Toolchain bootstrap** (the `rustup` step) is the runner's responsibility under buck2's own
  toolchain provisioning (or a thin per-runner `toolchain_bootstrap` binding in the executor),
  NOT a step the engine encodes. On the buck2-native path it disappears entirely.
- **Tool build + exec** (2)(3)(4) become `MaterializeStep`s emitted by the pure planner from the
  `generator` blocks. The executor lowers them. The two hardcoded `//...` targets become
  `manifest.artifacts[].generator.generator_target` data.
- **Output routing** uses plan-declared sinks (named per `artifact_id`, content-addressed by the
  producer's `source_inputs_digest` per ADR-0595), **not clock-seeded temp files**. #828's
  `temporary_scm_facts_path()` / `temporary_volatile_facts_path()` (which seed from
  `SystemTime::now()` + `process::id()` — two forbidden hermeticity inputs that today live in the
  verdict crate) are DELETED; their nondeterminism is removed by deterministic plan-declared sinks.

**The CI ordering becomes a declared dependency, universally.** On a buck2-native runner the gate
`rust_test` takes the materialized artifact as a `$(location)` dep on the materializer's output, so
**buck2's own scheduler enforces materialize-before-gate** — no YAML `needs:` at all. On a bridge
runner (today's GitHub Actions) the `producer-regen` job + `needs:` edges become a GENERATED
PROJECTION of `materialize_closure(P)`, not a hand-maintained surface. (Caveat — the scm-facts
emitter's git makes its step non-hermetic, so the buck2 genrule path requires the emitter's
materialized/committed snapshot as its declared input, which ADR-0552's stable/volatile split
already provides; where a producer cannot be a declared-output genrule, the engine degrades to the
generated-CI-ordering path — still strictly better than hand-wired `needs:`.)

---

## 5. Productization + any-repo adoption + synthetic-repo conformance proof

### 5.1 The product surfaces (engine + policy packs + control plane + public contract)

- **Engine**: `oya-ci-materializer-kernel` (pure planner+predicate) + `oya-ci-materializer-app`
  (executor). Repo-agnostic; zero hardcoded paths/targets in `-kernel`.
- **Policy pack**: `registry/generated-artifact-control-plane.json` IS the policy pack. The manifest
  + `runner_registry` + `generated_path_rules` are all the data an adopter supplies.
- **Control plane**: the `oya-ci-native-controller` runner + the
  `final_tree_materialization.portable_runner_contract` (already in the manifest) — a CRD/operator
  surface that reconciles declared artifacts on the protected branch (cloud-native, not CLI). The
  materializer-app is the imperative bridge; the controller is the W5 destination.
- **Public contract**: the manifest's `public_product_contract` string already states it — "Any
  repository adopting oya-ci can provide this manifest plus an SCM-facts snapshot to the Rust gate."
  This determination ADDS: "...and the materializer engine plans+executes regeneration from the
  same manifest; the gate is a pure predicate over materialized bytes."

### 5.2 Any-repo adoption path (TypeScript example, zero engine changes)

A TS monorepo committing `src/__generated__/schema.ts` (from `schema.graphql`) and `gen/api-client/`
(from `openapi.yaml`) — colliding on multi-thousand-line generated files, the exact #828 class:

1. Drop the generated files from git (already matched by the shipped `generated_path_rules`:
   `.generated.ts`, `__generated__` component, `gen` component).
2. Register a runner: `runner_registry += { runner_id:"node-codegen", canonical_target_prefix:"npm://",
   lowering:"npx-codegen" }`.
3. Declare two manifest rows, `materialization_mode: not-tracked-in-git`, with `generator` blocks:
   `{ runner:"node-codegen", generator_target:"npm://codegen/graphql", operation_id:"emit-graphql-types",
   parameters:{schema:"schema.graphql"}, input_contract:["repo-root","schema.graphql"],
   output_mode:"declared-artifact-path-write" }` and an api-client row whose `input_contract`
   references the schema output (so the planner orders it AFTER).
4. The SAME `plan()` orders the steps, the SAME executor runs codegen + a 2nd canary pass, the SAME
   `evaluate()` verifies regenerate-twice byte-stability and never reads a committed copy. CI ordering
   is `materialize_closure("gen/api-client/")` — derived, not hand-edited.

**No engine code changes — only a runner binding + manifest rows.** The merge-conflict class is
eliminated for that repo with zero oyatie-specific assumptions.

### 5.3 Synthetic-repo conformance proof (the productization gate)

Ship a `oya-ci-materializer-conformance` test crate that runs the engine against a **fixture
synthetic repo** under `cloud/cloud-ci/oya-ci-materializer-kernel/tests/fixtures/synthetic-repo/`
containing ONLY: a `control-plane.json` with two artifacts (a buck2-runner face and a
node-codegen-runner face), a `runner_registry`, and tiny deterministic + intentionally-non-
deterministic producer stubs. Proof obligations (all pure, no real buck2/git):

- **CP-1 plan-determinism**: `plan(manifest, Consume)` is byte-identical across runs and across
  two distinct fixture working directories (no clock/pid/path leakage).
- **CP-2 topological-order**: the api-client step orders strictly after the schema step purely from
  `input_contract`, with NO target strings in engine code.
- **CP-3 canary-catches-nondeterminism**: feeding the non-deterministic stub's two buffers to
  `evaluate(DeterminismCanary)` yields a RED finding; the deterministic stub yields GREEN.
- **CP-4 single-build invariant**: the `DeterminismCanary` plan has exactly one build+emit step and
  `multiplicity: 2` producer steps (asserted on the plan, not on execution) — the race cannot recur.
- **CP-5 anti-forgery**: a fixture manifest row at `evil/scm-facts.generated.json` does NOT exempt
  the canonical `scm-facts.generated.json` (full-path keying); an unregistered runner and a
  non-canonical target both yield `plan() = Err`.
- **CP-6 repo-agnosticism (the universality proof)**: the engine produces a valid plan for the TS
  fixture with ZERO oyatie paths in the plan code — only fixture manifest data drives it.

CP-1..CP-6 GREEN is the **conformance certificate**: the engine is universal, hermetic, anti-
forgery, and the canary is load-bearing. This crate is itself the public adoption fixture.

---

## 6. Strangler execution plan (smallest-first; gates NEVER dark; no flag-day)

The discipline: never remove a gate input before its readers are repointed and its integrity
predicate is flipped. The materializer must be proven byte-identical to the shell BEFORE the shell
is deleted. Each step ends with the full gate fleet green (buck2 + freshness + affected-set +
control-plane), per the buck2-build-green≠CI-green memory (regen lock+faces + run freshness/affected-
set, not just `buck2 build`).

- **E0 — land #828 as the bootstrap (already on `chore/decommit-generated-faces`).** It establishes
  the `not-tracked-in-git` mode, the de-commit exemption, the one-way re-tracking door, and the
  determinism canary. It is the policy floor the engine builds on. Merge it AS-IS (it is correct;
  it is just not yet universal). #828 does NOT become dead — it becomes the **first conformance
  case** (oyatie's 6 faces) the universal engine must reproduce.

- **E1 — author the kernel (pure planner + predicate), no behavior change.** New crate
  `oya-ci-materializer-kernel`: `plan()`, `evaluate()`, `materialize_closure()`, plus the
  conformance crate (§5.3) GREEN. Move #828's pure `read_decommitted_face_names` (full-path keying),
  `evaluate_face_freshness`, `evaluate_face_determinism` INTO the kernel unchanged (they are already
  pure). At this point nothing consumes the kernel yet — additive, gates unaffected.

- **E2 — author the executor (impure -app), prove byte-parity to the shell.** New crate
  `oya-ci-materializer-app`: verbatim move of `build_face_tools` / `emit_scm_facts` /
  `regenerate_producer_faces` out of the freshness gate into the executor; consume the kernel plan.
  Add a **cross-runner canary**: `materializer-app --scope consume` output == `materialize-cloud-ci-
  generated-faces.sh .` output, byte-for-byte, for all 6 faces. GREEN required. Shell + gate-self-
  materialization both still live; nothing deleted.

- **E3 — repoint the freshness + registry-drift gates onto the engine (the keystone).** In the SAME
  change: (a) delete the freshness gate's private regeneration — `EMITTER_TARGET`/`PRODUCER_TARGET`/
  `PRODUCER_FACES`/`FACES_DIR` consts, `build_face_tools`/`emit_scm_facts`/`regenerate_producer_faces`,
  the `temporary_*_path` clock/pid seeds, the `Command::new("buck2"|"git")` helpers, and the
  `read_committed_generated_faces` `read_dir` scan; (b) point `check_repo` at the kernel `evaluate()`
  fed by executor-supplied buffers. The gate becomes a PURE predicate. The lock-freshness half
  (`evaluate_lock_freshness`) is already pure and stays. NEVER do (a) before (b) — same change, no
  dark window. Run the full fleet.

- **E4 — flip CI ordering to the engine + delete the shell.** Replace the 5 shell call-sites in
  `.github/workflows/oya-ci-required.yml` (the `producer-regen` job + the in-job re-materializations
  in registry-drift/firewall/build-health) with `buck2 run //cloud/cloud-ci/oya-ci-materializer-app
  -- --scope <consume|canary>`. On the buck2-native gate path, add the `$(location)` dep so buck2
  schedules materialize-before-gate and the `needs:` edges become generated. DELETE
  `infra/ci/materialize-cloud-ci-generated-faces.sh`. Update `FACE_REMEDIATION_COMMAND`, the
  `rust-first-automation-policy.json:40` ledger row, and the freshness fixtures
  (`tests/freshness.rs`) that assert the shell path → point at the `buck2 run` command. The
  materializer is **removed from any irreducible-glue expectation** (it was never on the closed
  ADR-0523 ledger of two — only buck2 bootstrap + the git emitter are; the executor inherits exactly
  those two and adds nothing).

- **E5 — author ADR-0596 (the Rust-native materialization controller).** Record the
  `oya-ci-materializer` crate pair as the Rust successor to the shell, closing ADR-0595's explicit
  TODO ("The materializer remains a shell script ... transitional/irreducible-glue (ADR-0523) pending
  a Rust-native materialization controller"). Add the §1.3 control-plane assertion
  (`not_tracked_without_materializable_generator`) and the §1.4 `runner_registry` validation. Mark
  the executor as the imperative bridge to the W5 `oya-ci-native-controller` (CRD/operator)
  destination — the engine is cloud-native-ready (the controller runner already exists in the
  manifest taxonomy).

- **E6 — converge with corpus / cellular-hub (non-blocking).** Publish each commit's materialized
  faces to CAS keyed by commit SHA for historical/offline inspection (ADR-0595 §8 STEP 7). Per-
  capability shards become a WHERE-to-read optimization. Do not block E0-E5 on corpus delivery.

scm-facts itself stays committed for now (ADR-0595 explicitly defers it: it is a declared hermetic
input to ~20 gate tests requiring a dedicated repoint pass). The engine is built so that scm-facts'
eventual de-commit is *just another manifest row flip + the `$(location)` repoint* — no engine
change.

---

## 7. What #828 becomes

**KEEP #828 as the bootstrap; do NOT reshape it before merge.** It is correct, it is the policy
floor, and reshaping it would delay the merge-conflict-killing de-commit. The reshape is the
SUCCESSOR work (E1-E5), landed strangler-style on top. Concretely:

- #828's `materialization_mode: not-tracked-in-git`, the control-plane exemption + one-way door, and
  the determinism canary are the **permanent policy contract** — unchanged by the engine.
- #828's freshness-gate self-materialization (`Command::new`, clock/pid temp paths) is **transitional
  scaffolding** that E3 removes by moving it into the executor. It is the price of landing the
  de-commit before the engine exists; the engine pays it back.
- #828's pure predicates (`read_decommitted_face_names`, `evaluate_face_freshness`,
  `evaluate_face_determinism`) are **harvested into the kernel** unchanged (E1).
- oyatie's 6 faces become the **first conformance case**: after E3, the universal engine materializes
  them from the same manifest, and CP-1..CP-6 prove the engine is universal beyond oyatie.

---

## 8. 7-property bar — closed?

| Property | #828 as shipped | After this synthesis (E0-E5) |
|---|---|---|
| **UNIVERSAL** | manifest is R0 but regeneration is 3× hardcoded glue | engine reads ONLY manifest data; zero hardcoded paths in `-kernel`; CP-6 proves repo-agnosticism |
| **PRODUCTIZED** | gate + manifest, but no engine | engine + policy pack (manifest+runner_registry) + control plane (controller runner + portable_runner_contract) + public contract + conformance fixture |
| **HERMETIC** | FAIL — verdict gate reaches clock/pid/buck2/git | kernel pure (serde_json only); all impurity confined to the producer `-app` (ADR-0523 sanctioned); verdict is a pure predicate over bytes |
| **AUTOMATED** | shell `FACE_REMEDIATION_COMMAND`; flag-only beyond that | engine ships its own auto-fix (`materializer-app --scope consume` + `face-settle`); CI ordering auto-derived |
| **CLOUD-NATIVE** | shell + YAML `needs:` | manifest-declared deps → buck2 `$(location)` scheduler; controller runner = CRD/operator W5 destination; no-CLI-shaped |
| **MODERN/right-tool** | bespoke twice-runner + awk-parsing | standard genrule/Skyframe analysis→action phases, Rust-native |
| **LATEST-INFO** | — | verified against the live #828 branch + ADR-0551/0552/0595 on-branch, not memory |

Enforcement-layering: construction > reaction — the de-commit + the no-shell engine + the
auto-derived ordering are CONSTRUCTION (the surface cannot conflict, the glue cannot drift); the
control-plane gate + determinism canary + one-way door are the REACTION backstop. The anti-forgery
fix (full-path keying, merge-base-anchored ratchet) is preserved and moved into the kernel.

---

## 9. Risks

- **R1 — executor stays impure (by necessity).** It execs buck2 and (transitively) the scm-facts
  emitter's git. Sanctioned by ADR-0523 (two-item ledger). A reviewer MUST verify no `Command::new`
  / `SystemTime` / `git` creeps back into the `-kernel` or any gate the way it lives in freshness
  today. The kernel `#![forbid]` lacks a dependency on std::process — enforce via a deny-lint /
  banned-API gate on `-kernel`.
- **R2 — runner pluggability vs supply-chain.** A generic runner that execs arbitrary subprocesses
  is a supply-chain surface. Mitigated by the `runner_registry` + canonical-prefix + no-`shell`-
  runner rule (§1.4/§3.5); future hardening = pinned-digest binding. NEVER ship `runner: shell`.
- **R3 — determinism canary cost.** Regenerate-twice doubles producer time for the 11.6 MB registry.
  #828 already pays this; the engine inherits it. Run the canary on the materialize/registry-drift
  lane ONLY (single-writer/many-reader), preserving #828's shared-artifact topology — the engine's
  `materialize_closure` must NOT regress to per-leg double-build.
- **R4 — buck2 `$(location)` on a side-effecting producer.** Works only where the producer is a
  declared-output genrule; the scm-facts emitter's git is non-hermetic, so its genrule takes the
  ADR-0552 stable/volatile snapshot as input. Where a producer cannot be a genrule, degrade to the
  generated-CI-ordering path (still better than hand-wired `needs:`).
- **R5 — migration ordering.** The byte-parity cross-runner canary (E2) MUST be green before the
  shell deletion (E4); the freshness self-materialization removal (E3) MUST be the same change that
  points it at the engine, never before. No-flag-day discipline (ADR-0595 precedent).

---

## 10. Supersedes / feeds

- **Completes** ADR-0595 (its explicit "materializer remains a shell script ... pending a Rust-native
  controller" TODO) via the new ADR-0596.
- **Depends on / preserves** ADR-0551 (merge-base frozen ratchet — anti-forgery §3.2) and ADR-0552
  (stable/volatile scm-facts split — enables the genrule input §4/R4). Does NOT supersede them.
- **Feeds** ADR-0558 (faces merge driver — after de-commit + engine, the only residual merge surface
  is the human signoff door; scope #125 there).
- **Feeds** the corpus live-AST-graph (#128) and the cellular-hub shape: derive-don't-commit +
  content-addressed materialization is the CI-faces instance of de-globalizing the registry; the
  corpus later swaps the derivation ENGINE (syn extractor) for the buck2 producer; faces stay
  derived either way. Non-blocking.

---

## Adversarial critique

Hostile, fresh-context review of the determination above against the AS-SHIPPED #828 code on
`chore/decommit-generated-faces` (freshness gate `freshness-app/src/lib.rs` @1290 lines read;
control-plane gate `generated-artifact-control-plane-app/src/lib.rs` @1593 lines read;
`registry/generated-artifact-control-plane.json` @420 lines read; `materialize-cloud-ci-generated-faces.sh`
@63 lines read; ADR-0595/0551/0552/0547 read on-branch; `oya-ci-required.yml` materialize wiring read).
Goal: REFUTE that the design is genuinely UNIVERSAL + PRODUCTIZED + non-forgeable. Verdict at the end.

### What survives the hostile pass (verified true, not taken on narration)

- **The HERMETICITY indictment is REAL, not rhetorical.** Confirmed in source: `freshness.rs:8`
  `use std::time::{SystemTime, UNIX_EPOCH}`; `:1212-1230` `temporary_scm_facts_path()` /
  `temporary_volatile_facts_path()` seed temp paths from `SystemTime::now()` + `std::process::id()`;
  `:920/:940` `Command::new(&tools.emitter|producer)`; `:985..:1081` five `Command::new("git")`;
  `:1124` `Command::new("buck2")`. These live in the VERDICT crate. The determination's §0/§8 claim
  is accurate.
- **The "decorative generator block" indictment is REAL.** Confirmed: `freshness.rs` reads ZERO of
  `generator`/`generator_target`/`operation_id`/`input_contract`/`output_mode` (grep empty). The
  control-plane gate `validate_generator` (`cpgate.rs:283-430`) only VALIDATES the block's shape; no
  code CONSUMES it to drive regeneration. The shell and the freshness gate hardcode the targets
  instead (`freshness.rs:36-39` `EMITTER_TARGET`/`PRODUCER_TARGET`/`PRODUCER_FACES`;
  `materialize.sh:19-21` two literal `//...`). So "validated-but-decorative" and "regeneration
  encoded 3×" are both verified.
- **The runner-registry universalization is NOT gold-plating.** Confirmed `cpgate.rs:110`
  `const GENERATOR_RUNNERS: [&str; 2] = ["buck2", "oya-ci-native-controller"]` — a Rust const. An
  adopter genuinely CANNOT add a `node-codegen` runner without editing the gate's source, so
  promoting it to manifest `runner_registry` data is load-bearing for UNIVERSAL, not decoration.
- **schema_version is 1** (`registry:3`), so the §1 "1→2 additive extension" framing is correct.
- **ADR-0595/0551/0552/0547 all exist on-branch**; the firewall frozen reference IS merge-base
  anchored (`firewall/src/lib.rs:12-51`). The determination's anti-forgery §3.2 description of the
  EXISTING ratchet is faithful.

So the determination is well-grounded and not stale. The refutation is not "it's wrong about #828";
it is "three universalization moves either OPEN a hole #828 had accidentally closed, or under-specify
the airtightness they claim." Concretely:

### MUST-FIX 1 (CRITICAL, hostile-check #2 + #5) — the universal de-commit exemption is FORGEABLE; §3.1's "structurally impossible" is a non-sequitur

This is the load-bearing defect. In #828 the freshness byte-parity exemption is computed by
`read_decommitted_face_names` (`freshness.rs:827-859`), which admits a manifest row ONLY if
`GENERATED_FACE_PATHS.contains(&path)` (`:854`) — a HARDCODED 7-path allow-list (`:27-35`). That
hardcoded list is precisely what makes the candidate-controllable manifest non-forgeable for
freshness: a PR cannot add a row for an arbitrary path and have byte-parity skipped, because the
path is rejected unless it is one of the 7 canonical faces.

§1 and §3.1 of the determination DELETE that allow-list ("In the universal engine the canonical set
is `manifest.artifacts[].path` itself (full paths, manifest-declared), so the basename-forgery hole
is structurally impossible"). The reasoning is a **non-sequitur**: full-path keying defeats
*basename collision* (the specific hole #828 closed at `:849-856`), but it does NOT defeat *adding a
new full-path row*. The manifest is read from the CANDIDATE tree (`freshness.rs:828`
`repo_root.join(CONTROL_PLANE_MANIFEST)`; `evaluate_keyed(manifest, scm_facts)` consumes the
candidate manifest), so a candidate PR can append
`{ "path": "<any path it wants to stop byte-checking>", "materialization_mode": "not-tracked-in-git",
"generator": {...} }`, and the universal `evaluate()` would then SKIP byte-parity for that path
(the de-commit branch, `freshness.rs:673-675/693-697`). The hardcoded allow-list that blocked this
is gone. The §3.4 determinism canary does NOT close it — the canary proves the producer is
deterministic, NOT that the de-commit was legitimate; a forged row pointing at a deterministic
producer passes the canary and still evades the committed-byte ratchet. This is the same class as
the gate-baseline PR/push-asymmetry memory ("could this pass at PR-tier but fail on the integrated
tip?") and the #828-review basename hole — re-opened one layer up.

FIX (must be in §3.1, normative, not narration): the SET of de-commit-class paths must be anchored
to a reference the candidate cannot rewrite, exactly as the ratchet baseline is (§3.2). Three
acceptable airtight forms, pick one and SPECIFY it:
  (a) **merge-base manifest anchoring** — the exemption set = paths that are `not-tracked-in-git`
      in the control-plane manifest AS COMMITTED AT `git merge-base <base_ref> HEAD` (materialized
      by the scm-facts emitter, same out-of-graph boundary as the gate-baseline face). A candidate
      adding a NEW de-commit row gets NO exemption until it has landed on the base — newly-de-committed
      paths in the same PR are byte-checked against their (still-present, about-to-be-removed)
      committed copy, or the de-commit is a two-PR ratchet. This mirrors ADR-0551 exactly and is the
      construction>reaction answer.
  (b) **transition-guarded exemption** — a path may flip to `not-tracked-in-git` only in a PR whose
      diff also `git rm`s the committed copy AND the row was `merge-candidate-regenerated`/committed
      at the merge-base; the engine RED-flags a `not-tracked-in-git` row whose path was NOT a
      tracked generated output at the merge-base (i.e. "you cannot de-commit something that was
      never legitimately committed-and-generated").
  (c) at minimum, retain an allow-list **as policy data** keyed to the merge-base manifest, not a
      free candidate field. What is NOT acceptable is the determination's current "trust
      `manifest.artifacts[].path` from the candidate tree." Until §3.1 names one of these, the
      universal engine is STRICTLY MORE FORGEABLE than the #828 bootstrap it replaces — a regression.

Cross-check on the control-plane side: `evaluate_keyed` (`cpgate.rs:1018-1041`) ALSO keys the
`not-tracked-in-git` exemption on the candidate manifest with no merge-base anchor and no allow-list.
Its blast radius is smaller (it only suppresses `declared_path_not_tracked`, and a non-generated
source file isn't matched by `generated_path_rules` so de-committing it isn't laundered here), but
the determination claims §3.3's one-way door is airtight — and the door
(`generated_artifact_not_tracked_path_is_tracked`, `cpgate.rs:1045-1050`) only fires if the path is
STILL tracked. A PR that de-commits AND flips the mode in the same change trips nothing. The door
stops RE-tracking, not the initial illegitimate de-commit. §3.3 should state this boundary honestly:
the door is a re-tracking ratchet, not a de-commit-legitimacy check; legitimacy is MUST-FIX-1's job.

### MUST-FIX 2 (HIGH, hostile-check #4) — "kernel pure" is asserted but not ENFORCEABLE by the cited gate; the producer contract still smuggles a clock by omission

§2.1/§8 claim the kernel is pure "serde_json only" and R1 says enforce purity "via a deny-lint /
banned-API gate on `-kernel`." But the cited owned mechanism, ADR-0547's kernel-purity gate, keys
its deny set on **crate dependencies in the static dep graph** (`*-kernel`/`*-core` may not depend on
denylisted transient-tech CRATES), NOT on banned std APIs. `std::process::Command`, `std::time::SystemTime`,
`std::fs`, `std::net` are all in `std` — adding a `use std::process::Command` to a kernel introduces
NO new crate edge, so ADR-0547's gate would pass a kernel that shells out. The determination's
hermeticity guarantee therefore rests on a control that does not exist yet. This is not fatal (the
fix is small) but it must be NAMED as new work, not hand-waved as "enforce via the gate":
  FIX: §2/§9-R1 must specify a NEW banned-symbol predicate (clippy `disallowed_methods`/
  `disallowed_types` for `std::process::*`, `std::time::SystemTime`, `std::time::Instant`,
  `std::net::*`, `std::fs::*`, `std::env::*`, plus `rand`/clock crates) wired as a gate over
  `-kernel`, OR a `#![forbid]`-equivalent. Absent that, the kernel/app split is a NAMING convention,
  and "verdict is a pure predicate over bytes" is unverified-by-construction. Note also §4 DELETES
  the clock/pid temp paths but the executor still needs SOME sink path — the determination says
  "deterministic content-addressed sinks (source_inputs_digest per ADR-0595)"; that is correct and
  clock-free ONLY if the digest is computed from inputs, never `now()`. Make that a stated executor
  invariant (the digest MUST NOT mix in wall-clock/pid), or the clock creeps back into the sink name.

### MUST-FIX 3 (MEDIUM, hostile-check #1 + #3) — residual oyatie-specifics in what is called "the engine," and an unproven adoption claim

§5.2 asserts a TS repo adopts "with ONLY a runner binding + 2 manifest rows, zero engine changes,"
and §5.3 makes CP-6 the universality certificate. But several oyatie-specific assumptions are baked
into the CONTRACT, not just the bootstrap, and would surface as engine/contract changes for a real
non-oyatie adopter:
  (i) The merge-base anchoring of the exemption set (MUST-FIX-1) and of the ratchet (§3.2) assumes a
      git SCM with `git merge-base` and an `origin/dev`-style base ref. A non-git adopter (hg/jj/
      perforce, or a repo with a different default branch) needs this as POLICY DATA. ADR-0551's
      `base_ref` is already policy-data; the determination must state that BASE-REF AND SCM-KIND are
      adopter policy, or CP-6 silently smuggles git.
  (ii) The scm-facts boundary (the "single git boundary," §2.2/§4) is an oyatie concept. The
      determination treats `scm-facts-snapshot` as a canonical `input_contract` token (§1.1) and the
      synthetic fixture (§5.3) is required to model it. A TS adopter codegen-from-`schema.graphql`
      has NO scm-facts dependency; forcing that token into the universal contract is an oyatie leak.
      FIX: `scm-facts-snapshot` must be a REGISTERED artifact id like any other (it already is —
      `cloud-ci-scm-facts-boundary-snapshot`), referenced by id, with NO privileged status in the
      kernel. Verify the kernel never special-cases the string "scm-facts-snapshot" (today's shell
      DOES special-case it, `materialize.sh:11/62`). State this as a kernel invariant + a CP test.
  (iii) §5.3's conformance crate lives at
      `oya-ci-materializer-kernel/tests/fixtures/synthetic-repo/` and is described as proving
      repo-agnosticism, but a fixture authored BY oyatie, IN the oyatie tree, asserting oyatie's own
      engine is repo-agnostic, is a weak certificate (it proves the engine doesn't hardcode the
      FIXTURE's paths, not that it doesn't hardcode OYATIE's). CP-6 is necessary but not sufficient.
      FIX: strengthen CP-6 to a PROPERTY test — generate N random manifests with random paths/runners
      and assert plan validity + zero literal path/target strings reachable in `-kernel` (a source
      grep test: `-kernel` source contains no `//cloud/`, no `oya-cloud-ci-`, no `cloud/cloud-ci`).
      That grep-test IS the universality certificate; the single hand-authored fixture is not.

### NOT-A-BLOCKER but worth stating (hostile-check #6 — over-engineering)

The two-crate kernel/app split, the `MaterializeScope`/`Runner`/`OutputSink` enums, and the
structural-multiplicity canary are JUSTIFIED, not gold-plating — each maps to a verified #828 defect
(3× regeneration encoding; clock-in-verdict; hand-coded twice-runner at `freshness.rs:893`
`regenerate_faces_twice_with_buck2`). One genuine over-reach: §1.5's `consumption_order` derived view
+ §4's "buck2 `$(location)` scheduler dep replaces YAML `needs:`" is presented as in-scope, but R4 +
§4's own caveat admit the scm-facts emitter's git makes the genrule path non-hermetic, so the
buck2-native ordering DEGRADES to a generated `needs:` projection in practice. Recommend §4/E4
explicitly SCOPE the `$(location)` path as ASPIRATIONAL/future and ship the generated-`needs:`
projection as the E4 deliverable, so the plan does not promise a buck2-scheduler outcome it then
caveats away. (The simpler design — generated `needs:` projection from `materialize_closure` — meets
the bar; the `$(location)` path is the optimization, not the contract.)

### Verdict checks summary

1. Hardcoded oyatie paths in engine logic that should be policy? — PARTIALLY UNADDRESSED: the
   determination correctly moves targets→`generator` data and runners→`runner_registry`, but (a) the
   `scm-facts-snapshot` token is a privileged string the shell special-cases and the kernel must NOT
   (MUST-FIX-3.ii), and (b) base-ref/SCM-kind must be stated as policy data (MUST-FIX-3.i). The
   universality certificate must be a source-grep property test, not one fixture (MUST-FIX-3.iii).
2. Candidate PR can forge a policy entry to evade tracking/freshness? — **YES, AS WRITTEN.** §3.1
   removes the only thing (#828's hardcoded allow-list) that made the candidate manifest
   non-forgeable for freshness, and replaces it with candidate-tree trust. Merge-base anchoring of
   the EXEMPTION SET is asserted nowhere; only the ratchet baseline is anchored. **MUST-FIX-1, blocking.**
3. Runs on a non-oyatie repo, or does adoption need oyatie gates/manifests? — adoption needs the
   manifest (fine, that's the policy pack) + a runner binding (fine) BUT also implicitly a git SCM +
   scm-facts modeling (MUST-FIX-3). With MUST-FIX-3 fixed, yes; as written, the "zero engine changes"
   claim is unproven.
4. Producer contract truly no-shell + hermetic + deterministic? — the DESIGN is (executor isolates
   impurity; deterministic content-addressed sinks). But "kernel pure" is not enforceable by the
   cited ADR-0547 gate (banned-CRATE not banned-API), so purity is convention not construction until
   MUST-FIX-2 lands a banned-symbol check.
5. Any de-committed artifact loses a content-correctness invariant (fail-open)? — YES, transitively,
   via MUST-FIX-1: a forged de-commit row makes the freshness verdict fail-OPEN for that path (skips
   byte-parity, relies only on the determinism canary which does not check legitimacy). Closed by
   MUST-FIX-1.
6. Over-engineered? — NO for the kernel/app split and the canary (each maps to a real defect). The
   `$(location)` scheduler claim over-promises; scope it aspirational (non-blocking).
7. Execution keeps gates never-dark + strangler-safe? — YES. E0→E6 ordering is sound: bootstrap
   first, kernel additive, executor byte-parity-proven before shell deletion, gate self-materialization
   removed in the SAME change that repoints to the engine (E3), shell deleted only after the
   cross-runner canary is green (E4). This honors the buck2-build-green≠CI-green memory and the
   no-flag-day discipline. The ONE addition required: E3/E5 must also land MUST-FIX-1's merge-base
   exemption anchoring and MUST-FIX-2's banned-symbol kernel gate IN the same keystone, or the
   universal engine ships strictly more forgeable than the bootstrap — make these explicit E3/E5
   acceptance criteria, not deferred hardening.

### Final verdict: **NEEDS-REVISION**

The determination is well-grounded, faithful to the as-shipped #828 code, and its core reshape
(neutral kernel + thin executor collapsing the 3× regeneration encoding, with the canary as a
structural plan property) is the correct, non-over-engineered answer. It is NOT sound as written
because of ONE blocking defect and two high/medium gaps:
  - **MUST-FIX-1 (blocking):** §3.1 makes the de-commit exemption set candidate-forgeable by deleting
    #828's hardcoded allow-list without replacing it with a merge-base-anchored (or transition-guarded)
    exemption set. As written, the "universal" engine is strictly more forgeable than the bootstrap.
    The "structurally impossible" claim is a non-sequitur (full-path keying ≠ no-new-row).
  - **MUST-FIX-2 (high):** "kernel pure" is unenforceable by the cited ADR-0547 gate (banned-crate,
    not banned-API). A banned-symbol gate over `-kernel` must be named as E1/E5 work, not assumed.
  - **MUST-FIX-3 (medium):** `scm-facts-snapshot` privileged-string + git/base-ref assumptions are
    oyatie leaks into the contract; CP-6 must become a source-grep property test to be a real
    universality certificate.
With MUST-FIX-1 specified (pick anchoring form a/b/c), MUST-FIX-2 named, and MUST-FIX-3 folded into
the contract + conformance crate, the design meets the 7-property bar and is APPROVE-able. Until then,
APPROVE the #828 bootstrap (E0) — which is non-forgeable precisely BECAUSE of the hardcoded allow-list
the universal reshape must not delete without a merge-base-anchored replacement — and treat E1-E6 as
NEEDS-REVISION pending the three fixes.

---

## E1 resolution of critic must-fixes

**Shipped:** `libs/oya-ci-materializer-kernel` on `feat/oya-ci-materializer-e1` (ADR-0597).
28 tests GREEN (14 unit + 14 conformance CP-1..CP-6). Build: `buck2 build //libs/oya-ci-materializer-kernel/...` PASS.

**MF-2 (banned-symbol purity — HIGH):** Resolved by construction + enforced by test.
The kernel source uses ZERO banned std symbols (`std::process`, `std::time::SystemTime`,
`std::time::Instant`, `std::net::`, `std::fs::`, `std::env::`, `rand`). Enforced by
`mf2_no_banned_symbols_in_kernel_source` in `tests/conformance.rs`: a source-grep over the kernel
`src/` at test runtime. This fills the gap ADR-0547's kernel-purity gate (dep-CRATE ban, not
std-SYMBOL ban) left open. Any future edit that introduces a banned symbol will fail the test.

**MF-3 (no oyatie leaks — MEDIUM):** Resolved by construction + enforced by test.
The kernel source contains ZERO hardcoded oyatie literals (`//cloud/`, `oya-cloud-ci-`,
`cloud/cloud-ci`). Enforced by `cp6_mf3_no_oyatie_literals_in_kernel_source` in
`tests/conformance.rs`: same source-grep pattern. The synthetic-repo fixture
(`tests/fixtures/synthetic-repo/control-plane.json`) contains a TypeScript repo with `buck2` +
`node-codegen` runners; CP-6 proves the engine produces a valid plan with zero engine changes,
only fixture data. `scm-facts-snapshot` has no privileged kernel status — it is a plain
`input_contract` token like any other, matched against `artifact_id` / `operation_id` values.

**MF-1 (merge-base anchor — CRITICAL, E3 precondition):** NOT built in E1 (additive slice; gates
unaffected). The v2 contract is shaped to accept it without a schema break: `evaluate()` takes
`manifest: &ControlPlane` as a separate parameter so E3 can pass the merge-base-materialised
manifest for the exemption-set computation without any API change. The
`mf1_evaluate_accepts_separate_manifest_parameter` conformance test proves this API shape is
present and that swapping the manifest changes the verdict (candidate de-commit manifest vs.
merge-base committed manifest). E3 MUST pass the merge-base manifest — this is a hard E3
acceptance criterion, not a deferred concern.
