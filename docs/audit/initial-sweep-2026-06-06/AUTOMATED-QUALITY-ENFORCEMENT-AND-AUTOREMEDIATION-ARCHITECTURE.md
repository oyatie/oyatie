# AUTOMATED QUALITY ENFORCEMENT + AUTO-REMEDIATION ENGINE — ARCHITECTURE

**STATUS: pending-approval** · door:one-way · authored against the live engine in `/Users/jasonlee/Developer/source/cloud/cloud-ci/`

> This is a DESIGN. It mutates nothing. It generalizes the engine that already exists; it does not reinvent it. Every claim below cites code that was read.

## §0. The engine we generalize (grounding, not aspiration)

The contract is uniform and verified across all seven floor gates:

- **Gate = pure function over a producer-built face.** `evaluate_keyed(&Value) -> BTreeSet<Finding>` where `Finding { code: String, key: String }`, and `evaluate` is its bare-code projection (`oya-cloud-ci-manifest-hygiene-app/src/lib.rs:48-128`; identical shape in `bnf-layer-suffix/src/lib.rs:78-194`, `cargo-prefix/src/lib.rs:60-169`, `automation-ratchet/src/lib.rs:98-159`). The gate touches no filesystem; the producer does all I/O (`manifest-hygiene/src/lib.rs:9-12`).
- **Producer is hermetic.** `oya-cloud-ci-accounting-registry-app/src/main.rs` reads `git-facts.generated.json` + the declared sources, never shells out (`main.rs:5-9`, `:73-118`), and `build_gate_baseline` freezes today's keys per `(gate, code)` (`main.rs:228`).
- **Firewall = two pure predicates over data.** `compare` computes `regressions = current \ baseline` and FAILS a code iff `mode == "baseline-block-on-new" && !regressions.is_empty()` (`firewall-app/src/lib.rs:140-216`); `ratchet_growth` forbids any proposed baseline key not in the committed set unless founder-signed in `gate-baseline.signoff.json` (`firewall-app/src/lib.rs:218-246`, the ONE-WAY DOOR at `:90-124`).
- **Disposition is DATA.** `baseline-block-on-new` vs `advisory-until-infra`, `frozen_empty`, and `infra_prereq` live in `gate-disposition.json`; flipping advisory→blocking is a reviewed data edit, not a code change (`oya-ci-config/src/bundled/gate-disposition.json:2`).
- **Config is closed-schema + zero-config-default.** `OyaCiConfig` is `#[serde(deny_unknown_fields)]` with `bundled_default()` reproducing today's policy byte-for-byte (`oya-ci-config/src/lib.rs:72-161`); a gate carries an `input_kind` (`producer-face` / `raw-corpus-collector` / `frozen-empty-meta`) and a bound `face` (`oya-ci-config/src/lib.rs:574-610`); an adopter tunes everything by dropping an `oya-ci.toml` (`oya-ci.toml:104-142`).

**The one structural addition this design makes to the trait:** a gate gains an optional sibling to detection —

```rust
// new in oya-ci-gate-contract (the published trait crate, §WS-D / D-EXT in
// OYA-CI-PRODUCT-ARCHITECTURE-PLAN.md:339): detection's twin.
pub enum Remediation {
    AutoFix(Edit),          // deterministic codemod: existing artifact -> compliant
    AutoGenerate(NewFile),  // scaffold a missing compliant artifact
    None,                   // no safe auto path -> block-and-surface (last resort)
}
pub trait Gate {
    fn evaluate_keyed(&self, face: &Value) -> BTreeSet<Finding>;          // TODAY (unchanged)
    fn remediate(&self, finding: &Finding, face: &Value) -> Remediation;  // NEW: ships WITH the gate
}
```

`remediate` is as pure as `evaluate_keyed`: it returns a *described* edit (a `(path, byte-range, replacement)` or a `(path, body)`), it does NOT write. A separate, privileged delivery process (the PR-bot, §C/§E) applies and proposes. This keeps the dangerous capability (write access) out of the gate entirely — the gate stays unit-testable and sandboxed exactly as today.

---

## §A. THE FIVE ENFORCED QUALITY PROPERTIES

For each property: the FALSIFIABLE signal, the gate mapping, the `input_kind`, and the config seam. The two "soft" properties are split explicitly into *gated* vs *advisory* — the design does not pretend the un-mechanizable part is mechanizable.

### A.1 documentation (fully gateable)

- **Falsifiable signal:** every public surface that the producer can enumerate from code/spec has a corresponding doc artifact reachable from the masterplan. Reuse the existing reachability + crosswalk machinery: an ADR-class decision with no spec/masterplan/roadmap propagation is `unpropagated_decision`/`orphan_decision` (live in `cross-artifact-agreement`, `gate-disposition.json:20-21`); a tracked public crate/module with no justifying ADR is `undocumented_public_surface` (NEW code over the existing `resolve_justifications` index, `main.rs:1037-1092`). "Has a doc" is mechanical; "doc is *good*" is NOT gated (see §F).
- **Gate:** new `cloud-ci-doc-coverage`. `evaluate_keyed` emits `Finding{code:"undocumented_public_surface", key:"<crate-or-module-path>"}`. Shrink-only ratchet freezes today's undocumented surfaces; only a NEW public surface without a doc blocks.
- **`input_kind`:** `producer-face` bound to a new `GateFace::DocCoverage` (the producer already builds the justification index and ADR crosswalk — `main.rs:668-729`, `:1037-1092`).
- **Config seam:** `[doc]` section — `required_for = ["pub-crate","pub-module","public-api"]`, plus `doc_catalog`/`adr_dir` (`oya-ci-config/src/lib.rs:430-489`). Default reproduces today's reachability behavior, so adopting repos with no docs see only a frozen baseline, not a wall of new RED.

### A.2 hermeticity (fully gateable — the signal already exists in the build graph)

- **Falsifiable signal:** a build/CI step is hermetic iff it is a declared-input build-system action, not an ambient shell. Mechanically: (a) any `*.sh`/`run:` step in the CI surface that is not a `buck2`/`bazel`/`cargo --locked` invocation; (b) a producer/gate that shells out to ambient `git`/network (the producer already forbids this — `main.rs:5-9`); (c) a checked-in build artifact carrier (the `out/*.elf`/`talos-init.elf` hermeticity-debt class). This is the `automation-ratchet` shape: `blocking_invariant_mapped_to_oya_cli` and `advisory_claiming_enforced` (`automation-ratchet/src/lib.rs:198-229`) already detect "claims enforcement but no wired buck2 target."
- **Gate:** generalize `cloud-ci-automation-ratchet` into a `cloud-ci-hermeticity` family. New codes: `nonhermetic_shell_step` (key = `file#step`), `prebuilt_artifact_carrier` (key = path), `ambient_io_in_gate` (key = crate). The substrate `infra/ci/buck2-affected-gate.sh` and the "every gate is already a Buck2 target" fact (`OYA-CI-HERMETIC-EXECUTION-DESIGN.md:97-124`) make the "is it a declared buck2 action" check decidable.
- **`input_kind`:** `producer-face` for the manifest/CI-step inventory; `raw-corpus-collector` for the prebuilt-artifact scan (mirrors brand-residue's per-file keying, `main.rs:376-402`).
- **Config seam:** `[hermeticity]` — `allowed_step_kinds = ["buck2","bazel","cargo-locked"]`, `artifact_carrier_globs`. Ties to `[repro]` / `[dev_env]`.

### A.3 scalability (PARTIALLY gateable — honest about the boundary)

- **Gated (falsifiable):** structural anti-patterns mechanically detectable from code/config/manifest:
  - `unbounded_fan_in` — a handler/queue declared without a bounded channel/semaphore (AST pattern: an unbounded `mpsc::channel()`/`Vec::push` in an ingest loop with no backpressure type). `ast_grep` structural search; key = `file#fn`.
  - `single_leader_unsharded` — a config/manifest declaring exactly one replica for a stateful path with no partition key. Key = `manifest#service`.
  - `synchronous_unbatched_fanout` — an N+1 loop issuing one RPC per item with no batch API (AST: `await` inside a `for` over a request collection). Key = `file#fn`.
- **Advisory only (NOT gated):** "will this actually scale to X RPS" — a quantitative capacity claim requiring load data the engine lacks; ships `advisory-until-infra` with `infra_prereq: "load-profile-corpus"` (`gate-disposition.json:11-13,34-35`) and reports a count without flipping the verdict (`firewall-app/src/lib.rs:143-145`).
- **Gate:** new `cloud-ci-scalability-patterns`. `input_kind`: `raw-corpus-collector` (AST patterns over the tracked corpus, `code -> keys`, like brand-residue's `collect_*`, `main.rs:376-402`).
- **Config seam:** `[scalability]` — `pattern_packs`, `bounded_channel_types` (allow-list so the detector is repo-tunable), `stateful_path_markers`.

### A.4 hyperscaler-patterns (PARTIALLY gateable)

- **Gated (falsifiable):** presence/absence of named platform primitives in declarative config — decidable because the patterns ARE config:
  - `missing_readiness_probe` / `missing_liveness_probe` — a service manifest with no probe. Key = `manifest#service`.
  - `missing_resource_limits` — a workload with no CPU/mem requests+limits. Key = `manifest#container`.
  - `non_horizontal_scaler` — a stateful workload with no HPA/autoscaler reference. Key = `manifest#service`.
  - `singleton_without_leader_election` — a `replicas>1` workload mutating shared state with no leader-election annotation. Key = `manifest#service`.
- **Advisory only:** "is this the *right* sharding key / cell architecture" — design judgment. Fenced advisory, `infra_prereq: "architecture-review-corpus"`. Reuses `automation-ratchet`'s `enforceable_or_automatable_marked_human_judgment` discipline (`automation-ratchet/src/lib.rs:187-196`).
- **Gate:** new `cloud-ci-hyperscaler-patterns`. `input_kind`: `producer-face` bound to `GateFace::DeployManifests`.
- **Config seam:** `[hyperscaler]` — `manifest_globs`, `required_probes`, `require_resource_limits`, `autoscaler_kinds`. Zero-config default = empty manifest set ⇒ green (`empty_corpus_is_green`, `manifest-hygiene/src/lib.rs:215-217`).

### A.5 cloud-nativeness (PARTIALLY gateable — 12-factor as the falsifiable core)

- **Gated (falsifiable):** the mechanically-checkable 12-factor subset:
  - `config_in_code` — a hardcoded hostname/port/credential/connection-string literal where config should be injected (AST/regex; key = `file#literal-site`). Reuses the brand-residue census shape (`census_findings_with`, `main.rs:392-401`).
  - `non_disposable_state` — local-disk write outside a declared volume/tmp (AST).
  - `logs_not_to_stdout` — a file-logger sink instead of stdout/stderr.
  - `missing_graceful_shutdown` — a server entrypoint with no SIGTERM handler.
- **Advisory only:** "are the service boundaries correctly cloud-native" — architectural. Advisory, `infra_prereq: "service-topology-corpus"`.
- **Gate:** new `cloud-ci-twelve-factor`. `input_kind`: `raw-corpus-collector` for `config_in_code`/`logs_not_to_stdout`; `producer-face` for the entrypoint scan.
- **Config seam:** `[cloud_native]` — `secret_patterns`, `allowed_local_write_paths`, `log_sink_policy`, `entrypoint_globs`.

**Falsifiability honesty stamp:** documentation and hermeticity are fully gated; scalability, hyperscaler-patterns, cloud-nativeness each have a *gated structural core* + a *fenced advisory remainder* that ships `advisory-until-infra` and never flips the verdict until its `infra_prereq` corpus exists. The advisory mechanism is `gate-disposition.json:11-13`'s exact behavior.

---

## §B. THE AUTOMATION-FIRST MODEL — the four tiers

Priority order, per the directive ("automation, auto fix, auto generation comes first … then the enforcement … automated foremost"):

| Tier | Meaning | Engine realization |
|---|---|---|
| 1 · AUTOMATE | the gate runs with zero humans | already true: every gate is a buck2 `rust_test` + the producer regenerates faces |
| 2 · AUTO-FIX | deterministic codemod brings an EXISTING artifact into compliance | new `Remediation::AutoFix(Edit)` |
| 3 · AUTO-GENERATE | scaffold the MISSING compliant artifact | new `Remediation::AutoGenerate(NewFile)` |
| 4 · BLOCK-AND-SURFACE | last resort, only when no safe auto path exists | today's `Finding` + firewall RED — unchanged |

A gate ships with its remediation; the registry rejects a new gate whose codes have neither an AutoFix/AutoGenerate nor an explicit `None`-with-rationale (§D). Concrete remediations:

**Existing §2.5 floor gates:**
- `manifest-hygiene` → **AutoFix**: insert the missing field into `Cargo.toml` (`version.workspace = true`, `publish = false`, `license`, `[lints] workspace = true`, `[lib] doctest = false`); the producer knows the exact missing field per crate (`main.rs:574-589`).
- `bnf-layer-suffix` → **AutoFix behind a confirm gate** (rename is wide-blast): codemod `oya-foo-runtime` → `oya-foo-worker` + all references via `lsp_rename`/`ast_grep_replace`; auto-FIX-as-PR with required human approve, never auto-merge.
- `cargo-prefix` → **AutoFix**: align `[package].name` to the member-path crate-id (producer computes both, `main.rs:475-509`).
- `cross-artifact-agreement` → **AutoFix** the half-edge (add reciprocal `superseded_by`/`supersedes` front-matter, `main.rs:710-712`); **AutoGenerate** the masterplan/roadmap propagation stub for `unpropagated_decision`.
- `brand-residue` → **AutoFix**: token-replace the forbidden stem with its sanctioned replacement from a config rename-map (`main.rs:397-401`).
- `automation-ratchet` → **AutoGenerate**: scaffold the missing wired buck2 gate target stub so the claim becomes true.
- `staleness-reaper` → **AutoGenerate** the reap-report; **None** for source aging (no unsafe auto-delete).

**The five new properties:** documentation `undocumented_public_surface` → **AutoGenerate** a doc/ADR *stub* (structured fields only, NO prose — §F); hermeticity `nonhermetic_shell_step` → **AutoFix** to `buck2 run //target`, `prebuilt_artifact_carrier` → **AutoGenerate** the producing buck2 target + mark carrier for deletion; scalability `synchronous_unbatched_fanout` → **None**, `unbounded_fan_in` → **AutoFix advisory-only**; hyperscaler `missing_readiness_probe`/`missing_resource_limits` → **AutoGenerate** templated block; cloud-native `config_in_code` → **None/advisory**, `logs_not_to_stdout` → **AutoFix**.

**Design rule:** AutoFix is reserved for edits *provably behavior-preserving on a falsifiable signal* (manifest fields, declarative config, renames-with-full-reference-update). Anything that changes runtime *semantics* (extracting config, batching RPCs) is **None/advisory**, never an applied AutoFix. The safety boundary, formalized in §C.

---

## §C. THE SAFETY MODEL for auto-fix (the crux)

Five invariants, each mapped to existing substrate:

1. **Deterministic + reproducible.** `remediate()` is pure (same contract as `evaluate_keyed`); same face → byte-identical edits; testable with the RED/GREEN fixture discipline every gate has. The producer's determinism (committed==regenerated, registry-drift) is the precondition.
2. **Delivered as a PROPOSED, SIGNED, REVIEWABLE PR — never silent in-place mutation.** The applier is `oya-bot-autofix` on the repo-automation-bot framework, built on the clean `oya-shared-webhook-delivery-kernel` (HMAC-SHA256, NOT the Jenkins-tainted gateway). Grouped PR like `oya-bot-depupdate`; signed; merge gated by the FULL `oya-ci-required` fan-in via `oya-governance-pr-merge-gate-kernel`. Wide-blast AutoFix requires human approve, NEVER auto-merged.
3. **Dry-run / preview.** `oya-ci fix --dry-run` renders the diff without proposing; `remediate()` returns *described* edits so preview is just formatting the `Edit` set.
4. **Reversible + idempotent.** Each PR is a revertible commit; idempotence is a gate invariant: re-running `remediate()` on the post-fix face must return `None` (converged) — enforced by a producer self-test (regenerate, assert finding gone).
5. **Auto-fix BURNS DOWN the ratchet (not just freeze).** Today `fixed = baseline \ current` is informational (`firewall-app/src/lib.rs:196-197`). With auto-fix, the bot proactively closes `tolerated` baselined keys via burn-down PRs; each merge moves keys `tolerated → fixed`, SHRINKING the committed baseline. `ratchet_growth` (`:218-246`) already guarantees shrink-only without founder signoff — auto-fix is the engine that *drives* the shrink. It never ADDS keys, so it never touches `_sign_off_additions` (`:90-124`).

**Hard safety fence:** the bot has write access to PROPOSE only; it cannot merge (gated kernel) and cannot bypass any gate (a bot's CI verdict can only ADD findings). `remediate()` runs in the same no-write sandbox as `evaluate_keyed`.

---

## §D. "COMES FIRST" — flipping the default from flag to fix

- **A gate ships WITH its remediation.** `gate_registration` gains an assertion: every registered `(gate, code)` must declare a remediation tier (`auto-fix` | `auto-generate` | `block:<rationale>`). No remediation + no explicit `block`-rationale ⇒ registration fails. "Remediation is first-class alongside detection" becomes a *structural* requirement.
- **The disposition table gains a `remediation` field** (DATA, not code): `{"mode":"baseline-block-on-new","remediation":"auto-fix"}`.
- **The platform default inverts.** Violation → `Finding` + `remediate()` → if `AutoFix`/`AutoGenerate`, the bot opens a fix PR *first*; the RED carries "auto-fix proposed in #NNN". Block-and-surface is the explicit fallback for `Remediation::None`.
- **Burn-down is scheduled, not reactive-only.** `oya-bot-autofix` runs on a cadence over baselined debt, so the baseline shrinks continuously.

---

## §E. COMPOSITION (reference, don't duplicate)

- **Firewall** predicates unchanged; auto-remediation is a new layer that feeds them (burns down `tolerated`, never touches `ratchet_growth`/signoff). `firewall-app/src/lib.rs` needs no edit.
- **Productized platform** (`PLATFORM-PRODUCTIZATION-ARCHITECTURE.md` / `OYA-CI-PRODUCT-ARCHITECTURE-PLAN.md`): `Remediation` published in the semver'd `oya-ci-gate-contract` crate alongside `Finding`/`evaluate_keyed`; a third-party gate ships its remediation through the same contract; any adopter who enables a gate-pack gets enforced+auto-fixed quality, config-driven via `oya-ci.toml`.
- **Lifecycle-hermeticity substrate** (`LIFECYCLE-HERMETICITY-ZERO-SHELL-ARCHITECTURE.md` / `OYA-CI-HERMETIC-EXECUTION-DESIGN.md`): `remediate()` runs as a sandboxed buck2 action with declared inputs, so fix-generation is itself hermetic + cacheable.
- **Repo-automation-bot fleet:** `oya-bot-autofix` is one bot alongside `oya-bot-depupdate`/`oya-bot-release`, sharing the host, signed-capability trust model, and merge-gate-kernel.

---

## §F. PRE-MORTEM (≥4 — each with a falsifiable mitigation)

**PM-1 · Auto-fix corrupts code or fights the author.** Mitigation: AutoFix fenced to behavior-preserving edits (§B); idempotence self-test proves convergence (no oscillation); PRs reviewable+revertible, never silent; a per-(gate,code) "author-declined" suppression list (config DATA). *Falsifiable:* apply→regenerate→second `remediate()` MUST be empty.

**PM-2 · Auto-gen produces doc SLOP.** Mitigation: the generator emits a STUB with structured fields only (signature, owner, `status: stub`, TODO) and NO prose; doc-coverage counts a `status: stub` doc as PRESENT-BUT-INCOMPLETE (advisory `doc_stub_unfilled` on the burn-down). Generated prose forbidden by the ai-slop doctrine. *Falsifiable:* the gate asserts a generated doc contains zero free-prose paragraphs and carries `status: stub`.

**PM-3 · Soft-property gates degenerate into subjective false-positives.** Mitigation: every soft-property gate ships `advisory-until-infra` first, proving its false-positive rate on the burn-down BEFORE it can flip blocking; detector allow-lists are config DATA; the un-mechanizable remainder is explicitly NOT gated. *Falsifiable:* a gate may not flip `advisory → baseline-block-on-new` until its advisory run shows zero false-positives on a labeled fixture corpus.

**PM-4 · Auto-fix on a THIRD PARTY's repo is a trust/security + supply-chain surface.** Mitigation: `remediate()` runs in the same default-deny capability sandbox as `evaluate_keyed` with NO write capability; only the bot is write-capable (PROPOSE-only, cannot merge/bypass); third-party gate-packs are signed+attested+capability-declared (a pack requesting write capability is default-denied); the bot's PR is subject to the adopter's full fan-in + human merge; HMAC-verified webhook kernel, not the forbidden-vocab gateway. *Falsifiable:* a conformance test that a write-capability gate-pack is rejected at registration and an `oya-bot-autofix` PR cannot merge while any gate is RED.

**PM-5 · The ratchet is laundered via auto-fix.** Mitigation: structurally impossible — `ratchet_growth` FAILS on any proposed key not in the committed baseline unless founder-signed; auto-fix only moves keys to `fixed` (shrink) and never touches `_sign_off_additions`. *Falsifiable:* the existing `baseline_growth_without_signoff_is_ratchet_regression` test already proves this.

---

## §G. Trade-offs

| Option | Pros | Cons |
|--------|------|------|
| A · `Remediation` on the gate trait (chosen) | fix ships with detection (the "comes first" requirement is structural); pure + unit-testable; one published contract | mutates the semver'd `oya-ci-gate-contract` surface; every gate must declare a remediation tier |
| B · Remediation as a separate out-of-band tool | zero change to the gate contract | violates "comes first"; drift between detector and fixer |
| C · Bot applies fixes directly to main | fastest burn-down | unsafe — silent mutation, no review, bypasses the merge-gate; rejected |
| D · Gate `remediate()` returns *applied* edits | simpler bot | puts write capability in the sandboxed gate (PM-4); rejected — returns *described* edits only |

## Consensus Addendum

- **Antithesis (against auto-fix-first):** a flag-only firewall has a tiny, auditable trusted base (two pure predicates); a write-capable bot fleet multiplies attack surface (PM-4) + maintenance (every gate needs a tested codemod) for marginal toil savings on safe cases and unacceptable risk on dangerous ones. The honest counter: the directive is explicit + door:one-way, so the question is *how to bound it safely* — hence AutoFix fenced to behavior-preserving edits, everything semantics-changing `None`/advisory.
- **Tradeoff tension:** auto-generate fights anti-slop — aggressive generation fills the repo with structurally-present, substantively-empty artifacts (green-by-slop). Resolved by splitting "exists" (blocking) from "is filled" (advisory `doc_stub_unfilled`) + forbidding generated prose; a future maintainer who makes `doc_stub_unfilled` blocking would re-incentivize slop.
- **Synthesis:** keep flag-and-block as the *fallback tier* (Tier 4), not the default — the firewall's small trusted base is preserved (literally unedited); auto-remediation is strictly-additive, can only shrink the baseline, can only PROPOSE. Safe cases get auto-fix; dangerous cases stay block-and-surface.

---

## §H. ADDENDUM — additional enforced properties (founder directives, 2026-06-08)

This engine generalizes to the full engineering-excellence property set the founder named, all on the same `Finding`/`remediate()` contract + shrink-only ratchet. Captured here (not in a new doc, per the SSOT directive) for the convergence into ADRs:

- **dead-code / dead-file / stale-reference** (extends `staleness-reaper`): `dead_code` (unreferenced pub item via `lsp_find_references`/call-graph reachability; key = `crate#item`), `dead_file` (a tracked file no target/import reaches; key = path), `stale_reference` (a path/symbol/ADR-id referenced that no longer exists; key = `file#ref`). Remediation: **AutoGenerate a reap-report + AutoFix-as-PR the deletion** (deletion is behavior-preserving on the falsifiable "unreferenced" signal — but wide-blast, so PR-with-human-approve like the bnf rename, never silent delete).
- **doc-SSOT / anti-drift** (the founder's "why we consolidate" made enforceable): `duplicate_doc_claim` (the same fact asserted in >1 hand-maintained doc → drift risk; key = claim-hash), `unreachable_doc` (a doc not reachable from the masterplan ⇒ archive, per the reachability principle), `derived_doc_drift` (a hand-maintained doc that should be GENERATED from the SSOT and has drifted). Remediation: **AutoFix** by replacing the duplicate with a generated/derived reference to the single SSOT; **AutoGenerate** the masterplan reachability edge. This is the mechanization of "one SSOT ⇒ no contradiction/drift/staleness."
- **optimization / algorithm-hotspot** : `algorithmic_hotspot` (a super-linear pattern in a hot path — nested loops over the same collection, repeated re-allocation, an O(n²) where an index/map exists; AST + a perf-annotation corpus). Mostly **block-and-surface / advisory** (optimization is semantics-adjacent), with **AutoFix** only for mechanical wins (e.g. `Vec::contains` in a loop → `HashSet`).
- **maintainability / readability / idiomatic / well-written** : `non_idiomatic` (clippy-class lints surfaced as gate findings), `unformatted` (rustfmt drift). Remediation: **AutoFix** is the canonical case — `rustfmt`/`clippy --fix` ARE deterministic behavior-preserving codemods; this is the highest-confidence auto-fix tier. Architectural-excellence remains **advisory** (design judgment, fenced like §A.4's remainder).

**Falsifiability discipline (unchanged):** the mechanical core of each gates; the design-judgment remainder ships advisory-until-infra. Slop guard applies to all auto-generation.
