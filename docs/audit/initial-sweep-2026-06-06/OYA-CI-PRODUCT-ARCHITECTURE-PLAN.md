# OYA-CI PRODUCT-ARCHITECTURE PLAN (north-star)

> **STATUS: pending-design** · **NOT door:one-way** · ralplan(--deliberate) consensus loop · PLANNER REVISION pass (post Architect+Critic ITERATE/SPLIT)
> **Authored against:** `source` @ `cleanup/whole-tree-2026-06-07` HEAD `ca5e5efe5`
> **Scope:** DESIGN ONLY / NORTH-STAR. Mutates nothing in `source`. The only write is this doc (in the `linux` audit dir).
>
> **WHAT THIS DOC IS — read first.** This is the captured PRODUCT north-star for oya-ci, holding the workstreams that go BEYOND the migration-critical conformance floor. **Each workstream below is a SEPARATE FUTURE ralplan/campaign.** Every one is **gated AFTER the conformance floor + the migration, and NONE ever blocks them.** This document is **NOT a door:one-way commitment** — nothing here is ratified by signing it. It is to be ratified **workstream-by-workstream**, each in its own future ralplan, when its time comes. Treat the contents as design intent and verified groundwork, not as an approved plan of record.
>
> **The floor it sits on:** the migration-unblocking, decision-ready FLOOR (config-driven §2.5 gates, the engine-vs-policy seam, the gate input-binding abstraction, Stages 0–3+D, the byte-for-byte backward-compat invariant) lives in **`OYA-CI-CONFORMANCE-FLOOR-PLAN.md`** (same dir; STATUS pending-approval, door:one-way). That floor plan carries a one-paragraph forward-pointer to THIS doc. The floor stands alone and never depends on anything here.
>
> **SPLIT note (this revision):** this doc is the repurposed remainder of the former single `OYA-CI-PRODUCTIZATION-PLAN.md`, split per the Architect+Critic consensus (MF-6). All §WS-D…§WS-I content, the extension RALPLAN-DR rigor (principles 6–14, drivers D4–D7, options D1–D3/E1–E3/F1–F4), the extension pre-mortems (PM-5…PM-13), the expanded extension test lanes, the R-1..R-5 deep-research flags, the OQ-1..OQ-20 open questions, and the ADR addenda §9b/§9c/§9d were RELOCATED here (not lost). The floor's §1–§3, the floor ADR, and the floor RALPLAN-DR are in the floor doc.
>
> **MF reframes applied in this revision (see §0):** MF-2 (git-history vs hermeticity), MF-3 (reproducibility born-advisory), MF-5 (bot-vs-CI lifecycle) are reframed from "floor blockers" into explicit STAGE ENTRY-GATES on their respective workstreams here; MF-4 is reconciled (the `Finding`/`evaluate_keyed` mutation is a §WS-D workstream that carries its OWN byte-parity proof, vs the floor's FIXED-engine scoping).

---

## §0. WORKSTREAM REGISTER + ENTRY-GATES (read before any workstream)

Each row is a SEPARATE future campaign, ratified independently, gated as shown. **None blocks the floor or the migration.** The "entry-gate" is the condition that must be TRUE before that workstream's own ralplan opens.

| WS | Title | Earliest stage | Entry-gate (must hold before the workstream's ralplan opens) |
|---|---|---|---|
| **§WS-D** | Third-party gate contract / SDK (extensibility substrate) | Stage 4 | Floor landed (Stage 3+D) + migration unblocked. **MF-4:** §WS-D MUTATES the `Finding`/`evaluate_keyed` surface (hoist `Finding` into `oya-ci-gate-contract`, wrap `evaluate_keyed` in `trait Gate`) — it carries its OWN byte-parity proof (in-tree gates re-expressed via the contract still reproduce 79/233+all). This is distinct from the floor's FIXED-engine (ratchet/firewall/registry-drift/sign-off), which neither doc mutates. |
| **§WS-F** | Hermetic execution backends (`cargo`\|`buck2`\|`bazel`) | Stage 3.5 | Floor landed. **MF-2 entry-gate:** for git-derived gates (staleness / last-touch), DECIDE the hermeticity stance — content-addressed-history-input vs cargo-only-for-those — before a hermetic backend may claim to reproduce those gates' verdicts (a git-history read is not a declared Buck2/Bazel input by default). Scope the hermetic-backend reproducibility claim to the gates whose inputs ARE content-addressable; git-derived gates are explicitly carved until this is settled. |
| **§WS-G** | Reproducible builds + managed dev-env + strict dep versioning | Stage 3.5 | Floor landed. **MF-3 entry-gate:** reproducibility gates are **born-ADVISORY until the infra is clean** — they ship using the EXISTING `advisory-until-infra` disposition mechanism (verified in `gate-disposition.json`), only promoted to `baseline-block-on-new` once the verified `deny.toml`-wired-but-absent-`scripts/check.sh` GAP is actually closed and the gates run green. No reproducibility claim ships as a blocking gate before its enforcement exists. |
| **§WS-E** | Cloud-scale execution control-plane + runner | Stage 5 | Floor landed + migration unblocked + §WS-D + §WS-F. Owned-plane execution hard-gated; live matrix unchanged until runner #2 re-proves byte-parity. |
| **§WS-H** | Dependabot-equivalent dep-update bot | Stage 6 | §WS-E host exists (a bot needs the control-plane host). One bot on the §WS-I framework. |
| **§WS-I** | Repo-automation-bot framework + 19-bot suite | Stage 6 | §WS-E host exists. **MF-5 entry-gate:** the principle that "ONE control-plane hosts BOTH gates and bots" is DOWNGRADED from a stated principle to an OPEN DESIGN QUESTION — a bot's event-reactive lifecycle (webhook → react → act) is NOT the same shape as a CI verdict-fan-in (gates → join → required-context). Whether one plane cleanly hosts both, or they are co-resident-but-distinct services, is resolved in the §WS-I/§WS-E ralplan, not asserted (OQ-20). |

> **The byte-for-byte backward-compat green-invariant remains supreme across every workstream:** any change a workstream makes (the §WS-D surface hoist, any hermetic backend, any reproducibility gate, any runner) must reproduce the floor baseline (total-accounting=48633 · brand-residue=4494 · manifest-hygiene=233 · cross-artifact=168 · automation-ratchet=153 · bnf=79 · staleness=64) before it may gate `dev`. `cargo` + the live `oya-ci-required` matrix stay the floor/authority throughout.

---

## RALPLAN-DR SUMMARY (extension dimensions — north-star)

> These principles/drivers/options EXTEND the floor's RALPLAN-DR (which lives in the floor doc). They are captured here as the product north-star, ratified workstream-by-workstream.

### EXTENSION principles
6. **The contract is the product; the contract must be small, stable, and semver'd.** A third-party gate is "easy to write" only if the surface it implements is tiny and frozen. The surface ALREADY exists in proven form — `evaluate_keyed(&Value) -> BTreeSet<Finding{code,key}>` (verified in every gate, e.g. bnf lib.rs:139, Finding lib.rs:65) — but it is REDEFINED per-gate today. §WS-D HOISTS that surface into ONE published, semver'd crate (`oya-ci-gate-contract`). We publish the one already battle-tested.
7. **Discovery is data-driven and generalizes; nothing is hardcoded.** The `gate_registration` meta-test (`gate_registration.rs:51,88,145`, verified) ALREADY discovers in-tree gates by reading the `cloud/cloud-ci/gates/` directory; §WS-D generalizes THIS to external packs declared by coordinate, registration staying an enforced invariant. (Note the floor's §3.5 distinction: the directory-scan set and the baseline KIND-set are NOT identical — brand-residue's collector lives at `libs/`, the `registry_drift` code is not a gate dir — so generalization must respect input KIND.)
8. **Trust is declared + enforced, never assumed.** A third-party gate runs arbitrary code over the consuming repo. A manifest DECLARES capabilities; the consuming repo OPTS IN per-pack; the existing supply-chain/license/SLSA/image-signing gates are the PRECEDENT. Default-deny: an unsigned/over-privileged external gate is not enabled.
9. **One logic, two runners — the execution plane is an abstraction over the SAME gate logic.** The gate logic is runner-agnostic by construction. The workflow file ALREADY states this verbatim ("one logic, two runners", verified `oya-ci-required.yml` 76-77). §WS-E is a NEW SCHEDULER/EXECUTOR around an UNCHANGED gate core.
10. **Scale work is a separate campaign AFTER the floor; it never touches the live required context.** §WS-E is staged LAST and gated so the live matrix keeps running unchanged until an owned runner re-proves byte-parity.
11. **A hermetic build backend is the execution MODE that ports hermeticity to adopters; `cargo` is the universal fallback; the runner-MODE is orthogonal to the cloud control-plane.** The gates are ALREADY Buck2 targets (verified); "run them via `buck2 test`" is a change of EXECUTION SUBSTRATE, not gate code. `buck2`/`bazel` modes = sandboxed declared inputs + content-addressed cache + affected-set → reproducible on any adopter's box. `cargo` mode = the zero-build-system portable fallback. The hermetic backend is OPTIONAL, never a hard dependency (PM-8).
12. **Backend-agnosticism: one gate logic, N hermetic build backends — the engine never names a backend.** Buck2 and Bazel are both Starlark build systems giving sandboxed actions + RBE + content-addressed cache + affected-set. The MODE axis generalizes to a `HERMETIC-BUILD-BACKEND` trait — `cargo` | `buck2` (ours) | `bazel` (adopter-facing) — behind which the gate logic is invariant. Drift risk is PM-10.
13. **Reproducibility is end-to-end, pinned, and expressed as config-driven gates — not a one-off CI flag.** Every input pinned + sandboxed + content-addressed: toolchain (`rust-toolchain.toml`=`1.95.0`, verified), dep closure (`Cargo.lock` 12376 lines verified, `--locked`, `cargo deny` via `deny.toml` verified), build actions (§WS-F backend sandbox), dev-env. **Verified GAP:** `deny.toml` CLAIMS `scripts/check.sh`+CI wiring that DOES NOT EXIST (verified zero hits) — §WS-G closes this, born-advisory (MF-3).
14. **Everything-as-config-driven-automation: one config model, one contract/SDK pattern, one control-plane, one hermetic-backend story.** *(MF-5 caveat: "one control-plane hosts both gates and bots" is an OPEN DESIGN QUESTION, not an asserted principle — a bot's event-reactive lifecycle differs from CI verdict-fan-in; see §0 + OQ-20.)*

### EXTENSION Decision Drivers
- **D4 — Extensibility without ossification.** A third-party MUST ship a gate outside the monorepo against a stable, semver'd contract — without freezing too early or churning. Capability negotiation + additive-only evolution + a deprecation window (PM-7).
- **D5 — Hyperscale readiness without scope-creep.** The execution plane must be cloud-scale-capable per canonical patterns — without ballooning into a rebuild that stalls the migration. Hard sequencing: floor first, contract/SDK next, control-plane last (PM-6).
- **D6 — Hermeticity portable to adopters.** `cargo` mode is portable but NOT hermetic; a hermetic backend (`buck2`/`bazel`) delivers reproducible, content-addressed, affected-incremental runs an adopter gets out of the box. Pulls hermetic-backend EARLIER than the cloud plane; D1 keeps `cargo` the always-present fallback.
- **D7 — Adopter reach without backend lock-in, and reproducibility/automation as PRODUCT surface.** A Bazel shop is excluded if hermeticity is Buck2-only; the `HERMETIC-BUILD-BACKEND` trait resolves this. Reproducible-builds (§WS-G), dep-updates (§WS-H), repo-automation bots (§WS-I) are first-class adopter-facing capabilities, all config-driven and gated, all staged as own-campaigns.

### Viable Options — EXTENSION dimension 1: the third-party gate-contract shape (§WS-D)

| | **Option D1 — Trait-object plugin (dynamic `dylib`/cdylib + ABI)** | **Option D2 — Published source crate implementing a semver'd `evaluate_keyed` contract; external packs are deps + a manifest** ⭐ RECOMMENDED | **Option D3 — Out-of-process gate: any executable speaking JSON stdin/stdout (face in → findings out)** |
|---|---|---|---|
| Shape | `oya-ci-gate-contract` defines `trait Gate`; external gates ship a `cdylib`; the producer `dlopen`s them. | `oya-ci-gate-contract` publishes `Finding`, `evaluate_keyed` signature, `GateManifest`. An external gate is a normal crate depending on the semver'd contract; discovery by coordinate/path + manifest, generalizing the directory-driven `gate_registration`. Matches what the engine ALREADY does (pure fn over `&Value`). | Contract is a wire protocol: a gate is any binary reading a face JSON on stdin, writing `[{code,key}]` on stdout, + a sidecar manifest. Language-agnostic for the author. |
| Pros | True dynamic loading; no recompile to add a gate. | Smallest delta (contract already IS a pure fn); Rust type-safety; trivial to test; reuses producer fan-in + ratchet untouched; manifest gives capability declaration. | Author writes in ANY language; strong process isolation (sandbox = process boundary; serves D4/trust). |
| Cons | Rust has NO stable ABI — `dylib` plugins fragile across compiler versions; defeats "stable contract"; sandboxing a `dlopen`'d lib is hard. | External gate must be Rust (or a Rust shim) to link; non-Rust authors need the D3 escape hatch. | Per-gate process spawn + JSON marshalling overhead; the wire schema becomes the semver'd thing; weaker compile-time guarantees. |
| Risk to D4/trust | HIGH | LOW | LOW-MEDIUM |
| Verdict | Rejected (Rust-ABI fragility) | **Chosen as PRIMARY** | **Adopted as the SECONDARY out-of-process path** (any-language + strong-sandbox escape hatch) |

**Why D2 primary + D3 secondary:** D2 is the lowest-delta, type-safe path matching the engine's pure-function reality and gives the founder's "stable, versioned contract … semver'd crate so a gate can live OUTSIDE the monorepo" directly. D3 is adopted as a SECONDARY co-designed path: the same `GateManifest` + `Finding{code,key}` schema over a process boundary — the strongest sandbox for untrusted code (PM-5). One contract, two carriers.

### Viable Options — EXTENSION dimension 2: the cloud-scale execution plane (§WS-E)

| | **Option E1 — Adopt an existing engine wholesale (Tekton/Argo/Prow)** | **Option E2 — Owned thin control-plane orchestrating the EXISTING producer+gate matrix, GitHub Actions as runner #1 + an owned runner #2** ⭐ RECOMMENDED | **Option E3 — Stay on GitHub Actions matrix; scale via ARC autoscaling + remote cache** |
|---|---|---|---|
| Shape | Lift gates onto a third-party control-plane; gates run as that system's task primitive. | A small owned plane (trigger intake → run record → scheduler → fan-out → status back as `oya-ci-required`), with a `Runner` abstraction whose first impl is the LIVE matrix (unchanged) and second is an owned fleet. Borrows the canonical PATTERNS, not the whole system. | No control-plane; GitHub-native merge queue + ARC autoscaling + a remote cache. |
| Pros | Battle-tested schedulers. | Preserves "one logic, two runners"; the live context never moves until parity re-proven; owned plane enables multi-tenancy + cost-attribution as a PRODUCT. | Lowest effort; no owned plane. |
| Cons | Heavy operational dependency; couples gate core to a foreign lifecycle; large adoption surface mid-migration. | Building a plane is real work → hard-gated to Stage 5 (PM-6). | Not a product; no multi-tenancy/cost-attribution; ties the ceiling to GitHub. |
| Risk to D1 / migration (D5) | HIGH | LOW (staged last, live matrix untouched until parity re-proven) | LOW (but caps the product) |
| Verdict | Rejected as primary (borrow PATTERNS, not the engine) | **Chosen** | **Adopted as the Stage-5a interim** (ARC + remote cache on the existing runner; on-ramp, not destination) |

### Viable Options — EXTENSION dimension 3: the gate EXECUTION MODE / backend (§WS-F)

| | **F1 — `cargo test` only (status quo)** | **F2 — single hermetic backend as a HARD dependency** | **F3 — Dual MODE: `cargo` fallback + `buck2` hermetic** | **F4 — N-backend: `cargo` fallback + `HERMETIC-BUILD-BACKEND` over `buck2` AND `bazel`** ⭐ RECOMMENDED |
|---|---|---|---|---|
| Hermeticity for others | NONE | FULL but excludes non-backend shops | FULL for Buck2, GRACEFUL for the rest | FULL for Buck2 AND Bazel shops, GRACEFUL for cargo-only |
| Adopter barrier | zero | HIGH (forces a build system on everyone) | zero floor + opt-in ceiling | zero floor + opt-in ceiling in the adopter's OWN build system |
| New risk | — | — | — | backend abstraction LEAKS/DRIFTS (PM-10) — thin trait + shared conformance |
| Risk to D1/D6/D7 | LOW to D1, FAILS D6 | HIGH to D1 + violates "optional, never a hard dep" | LOW to D1, SATISFIES D6 | LOW to D1, SATISFIES D6 + D7 |
| Verdict | the fallback half of F4 | **Rejected** (PM-8) | the buck2-only specialization of F4 (the first backend we land) | **Chosen** |

**Why F4:** the founder mandated Bazel reach; Buck2 and Bazel are the same SHAPE (Starlark, sandboxed actions, RBE, content-addressed cache, `rdeps`), so the hermetic mode is a TRAIT with two impls, not two forks. WE keep buck2 internally; `bazel` is the adopter-facing sibling; `cargo` is the universal fallback. F2 (any single backend as a hard dependency) stays rejected (PM-8). **MF-2 entry-gate applies:** git-derived gates (staleness/last-touch) need a hermeticity stance (content-addressed-history-input vs cargo-only-for-those) before a hermetic backend claims to reproduce them.

### EXTENSION Pre-mortem (failure scenarios + mitigations)
- **PM-5 — A third-party gate runs malicious code over the repo.** *Mitigation (default-deny, defense-in-depth):* no dynamic Rust plugins (D1 rejected); in-process (D2) gates are SOURCE-reviewed deps in the consumer's own lockfile (existing `Cargo.lock` + `oya-check-supply-chain`/`oya-check-license-policy`, verified); out-of-process (D3) is the sandbox (separate process, faces on stdin, `network: none`, read-only/no-fs by default, no secrets in env); capabilities DECLARED in `GateManifest` + ENFORCED by the runner; gate-packs SIGNED + provenance-attested (cosign/SLSA precedent, verified); a gate can only ADD findings — the shrink-only ratchet + one-way door mean it cannot lower a baseline or false-green another gate.
- **PM-6 — The cloud-execution rebuild scope-creeps and blocks the migration.** *Mitigation:* §WS-E is DESIGN-ONLY here, execution hard-gated to Stage 5, AFTER floor + §WS-D. Its OWN ralplan; live matrix unchanged until runner #2 re-proves byte-parity. Non-goal stamped.
- **PM-7 — The gate contract ossifies too early or churns and breaks external gates.** *Mitigation:* the contract is intentionally MINIMAL (the surface already proven); additive-only evolution + semver; capability negotiation (`contract_version` matched against a supported range); a conformance suite as the regression guard.
- **PM-8 — A hermetic backend becomes a hard adopter dependency / non-backend repos get no value.** *Mitigation:* `cargo` mode is a first-class, permanently-supported FALLBACK (= today's live matrix, verified); mode detected/declared, never assumed; gate logic mode-agnostic; docs frame the backend as the opt-in hermeticity/scale upgrade; F2 explicitly rejected.
- **PM-9 — Affected-set false-negative silently skips a gate → a real violation merges.** *Mitigation:* the existing `infra/ci/buck2-affected-gate.sh` is FAIL-CLOSED (verified — a graph-relevant change with no owner is FATAL, not a silent pass; `uquery`/`rdeps` error fails the gate); the producer/config/baseline are DECLARED Buck2 inputs so `rdeps()` marks dependent gates affected (never zero); periodic FULL runs as backstop; the required context stays full-run until R-2 settled.
- **PM-10 — The `HERMETIC-BUILD-BACKEND` abstraction leaks/drifts between buck2 and bazel.** *Mitigation:* deliberately MINIMAL trait (`affected_targets`/`run_gate`/`cache_key`/`regen_producer`); a backend-conformance suite asserts buck2 ≡ bazel byte-identical verdicts + affected-sets on the same fixture; WE land + maintain only `buck2`, `bazel` is ADVISORY until it passes; `cargo` is the always-present floor.
- **PM-11 — The repo-automation-bot suite scope-creeps and blocks the migration/floor.** *Mitigation:* §WS-I is DESIGN-ONLY, hard-gated LAST (after floor, §WS-D, §WS-E host). Bots land incrementally, highest-value first; NONE a prerequisite. Non-goal stamped.
- **PM-12 — The dep-update bot opens a PR storm.** *Mitigation:* grouped + scheduled by default (cadence, not one-per-crate over a 12376-line lockfile); every PR gated by the full oya-ci; auto-merge-on-green OPT-IN + conservative; ignore/pin/group config; security/RustSec bumps prioritized + ungrouped.
- **PM-13 — Reproducibility is CLAIMED but unverified (the precedent-gap trap).** *Mitigation (the gate IS the proof; MF-3 born-advisory):* every reproducibility claim is an EXECUTABLE config-driven gate, born-ADVISORY via the existing `advisory-until-infra` disposition until the enforcement infra is clean, only then promoted to blocking; the byte-reproducible-artifact proof is a second-machine e2e; the dev-env bootstrap materializes the SAME pinned toolchain + locked deps; R-4 flags full bit-identity as a tracked goal, not asserted-done.

### Expanded Test Plan (EXTENSION lanes — north-star)
- **(WS-D)** `oya-ci-gate-contract` `Finding`/`evaluate_keyed`/`GateManifest` round-trip + semver compat; a reference external gate compiles against the published contract, REDs its RED fixture and `∅` for GREEN (born-blocking); **the §WS-D surface hoist STILL reproduces 79/233+all byte-for-byte (the MF-4 §WS-D byte-parity proof)**; the generalized `gate_registration` discovers an external pack by coordinate identically to an in-tree gate and still FAILS on an unregistered gate; the D3 runner enforces declared capabilities (a gate exceeding its manifest is killed and the run REDs).
- **(WS-E)** a `Runner` conformance test: the same producer+gate set under runner #1 (Actions shim) and runner #2 (owned-fleet shim) produces byte-identical baselines; sharding is verdict-invariant; dedup + retry don't change the verdict.
- **(WS-F MODE/backend parity)** the same gate set under `cargo` and `buck2` produces byte-identical per-gate verdicts + baseline (the MODE-parity proof); a FULL `buck2 test` and an AFFECTED run agree on every verdict (PM-9); a producer/baseline/config change marks dependent gates AFFECTED (never zero); a graph-relevant change with no owner is FATAL; the `buck2` and `bazel` backends emit BYTE-IDENTICAL verdicts + affected-sets (PM-10, `bazel` ADVISORY until it passes). **MF-2:** git-derived gates are carved from the hermetic reproduction claim until their hermeticity stance is decided.
- **(WS-G reproducibility — born-advisory, MF-3)** the lockfile-drift / `cargo deny check` / toolchain-pin / audit gates run as config-driven gates born-ADVISORY (the `advisory-until-infra` disposition), the byte-for-byte baseline (79/233+all) holds with them present; the `deny.toml` GAP is closed before any of them is promoted to blocking; a second-machine artifact-diff e2e is the bit-reproducibility proof (R-4 tracked, not asserted).
- **(WS-H)** a dep-update PR that breaks a gate is BLOCKED by the full fan-in; a clean bump passes; auto-merge-on-green (opted in) fires only for patch/compatible bumps; a PR-storm fixture yields ONE grouped PR per cadence (PM-12).
- **(WS-I)** the bot SDK manifest + event-handler contract round-trips; a reference bot REDs a RED commit-message fixture; the framework's webhook-signature verification rejects a bad HMAC (the `oya-shared-webhook-delivery-kernel` precedent); a bot's CI-gating verdict feeds the SAME fan-in and can only ADD findings (cannot weaken the ratchet, mirroring PM-5). **MF-5:** the bot-host-vs-CI-fan-in lifecycle question is resolved in this workstream's ralplan (OQ-20), not assumed.
- **Observability (all WS):** run-level traces, per-gate/per-tenant metrics, structured run-id-keyed logs, SLOs; external gate `contract_version` + pack digest + signature stamped into provenance; the active MODE/backend + cache hit/miss + affected-vs-full stamped; the toolchain channel + `Cargo.lock` digest + `cargo deny`/audit verdicts stamped; per-bot structured events + bot SLOs.

---

## §WS-D — THIRD-PARTY GATE CONTRIBUTION INFRASTRUCTURE (the substrate, not a marketplace)

> **Stage 4. Entry-gate:** floor landed + migration unblocked. **MF-4:** this workstream MUTATES the `Finding`/`evaluate_keyed` surface and carries its OWN byte-parity proof (the in-tree gates re-expressed via the contract still reproduce 79/233+all). The floor's FIXED engine (ratchet/firewall/registry-drift/sign-off) is NOT touched by this.

**Headline (the gate-contract/SDK shape):** publish the ALREADY-PROVEN gate surface as ONE small, semver'd crate `oya-ci-gate-contract` so a gate can live OUTSIDE the monorepo; discover + register external gate-packs by coordinate/path using a generalization of the existing directory-driven `gate_registration` meta-test (respecting input KIND, see floor §3.5); declare + enforce per-gate capabilities and sign gate-packs so untrusted third-party code is opt-in and sandboxed; ship a scaffold + RED/GREEN-fixture conformance so writing a gate is as easy as authoring a GitHub Action.

> **Verified basis (cited):** `pub fn evaluate_keyed(input: &Value) -> BTreeSet<Finding>` (e.g. `oya-cloud-ci-bnf-layer-suffix-app/src/lib.rs:139`) + `pub struct Finding { code: String, key: String }` (lib.rs:65), `Finding` redefined per-gate (16 copies across 6 cloud-ci gates + 10 oya-check libs, verified). Discovery is already data-driven: `gate_registration.rs` reads `cloud/cloud-ci/gates/` for any dir with a `Cargo.toml` (`gate_crate_dirs`, line 51) and enforces registration (lines 88, 145). `GATE_IDS` (lib.rs:462) is replaced by `cfg.gates.enabled` in the FLOOR.

### WS-D.1 The stable, versioned gate contract / SDK
**New published crate `oya-ci-gate-contract`** (a `sdk`-role lib; semver'd; the ONLY thing an external gate depends on). It HOISTS, verbatim, the surface copy-pasted per gate today:
- `pub struct Finding { pub code: String, pub key: String }` — lifted from each gate's local copy (the engine/ratchet already consume exactly this shape via `current_keys_per_gate` → `group_findings`, lib.rs:505,565).
- `pub trait Gate { const ID: &'static str; const CONTRACT_VERSION: ...; fn evaluate_keyed(input: &serde_json::Value) -> BTreeSet<Finding>; fn manifest() -> GateManifest; }` — a thin trait whose `evaluate_keyed` is byte-identical in behavior to today's free function (an in-tree gate adopts it around its existing body — no logic change, preserving byte-parity — the MF-4 §WS-D proof).
- `pub struct GateManifest { id, codes, input_face, config_keys, language_pack, contract_version, capabilities }`.
- `#![forbid(unsafe_code)]`, I/O-free, pure.

**Versioning + compat (D4 / PM-7):** semver; additive-only minors; breaking changes MAJOR behind a deprecation window; CHANGELOG + compat test matrix; capability negotiation via `contract_version`.

### WS-D.2 Packaging, discovery, registration (generalize the existing mechanism)
- **External-pack declaration:** extend `[gates].enabled` (floor §3.1) so an entry may be EITHER an in-tree gate id OR an external coordinate `{ pack, source = "registry"|"path"|"git", ref, digest, signature }`; a `[gate_packs]` table declares trusted sources + signing keys. Still a CLOSED schema.
- **Discovery generalization:** the `gate_registration` directory scan generalizes from "read the gates dir" to "read the gates dir ∪ declared external packs"; the producer's KIND-dispatch (floor §3.5) resolves each `producer-face` gate's evaluator across in-tree + external. The completeness invariant (no gate goes unregistered) is PRESERVED and extended. *(Respect floor §3.5: discovery and the baseline KIND-set are not identical sets — a `raw-corpus-collector` or `frozen-empty-meta` contribution is registered differently from a directory-scanned `producer-face` gate crate.)*
- **No hardcoding:** `GATE_IDS` already deleted in the floor; the gate set is fully `cfg`-driven.

### WS-D.3 Trust / security of third-party code (default-deny; see PM-5)
- **Two carriers, two trust models:** in-process Rust gate (D2) admitted only as a SOURCE dependency the consumer compiles (review = existing `Cargo.lock` + supply-chain/license flow, verified); out-of-process (D3) is the SANDBOX (separate process, faces on stdin, `network: none`, read-only/no-fs by default, no secrets in env).
- **Declared + enforced capabilities:** `GateManifest.capabilities { fs, network, faces, config_keys }`; the runner ENFORCES (kills + REDs a gate that exceeds its declaration).
- **Signed + provenance-attested packs:** reuse `oya-check-slsa-l3-evidence-grounded` + `oya-check-image-signing-discipline` (cosign/SLSA, verified); default-deny on unsigned.
- **Ratchet integrity:** a gate can only ADD `Finding`s; the shrink-only ratchet + one-way door (firewall `ratchet_growth` lib.rs:223 / `SignOff` lib.rs:94) mean a compromised gate cannot lower the baseline or false-green elsewhere.

### WS-D.4 Contributor ergonomics ("write your first gate")
- **Scaffold:** `oya-ci gate new <id>` generates a crate depending on `oya-ci-gate-contract`, a stub `evaluate_keyed`, a `GateManifest`, and RED+GREEN fixtures pre-wired (born-blocking).
- **Local run:** `oya-ci gate test` / `oya-ci run --gate <id>` (the `act` analogue).
- **Conformance to be accepted:** implements the contract; declares a valid capability manifest; ships RED+GREEN fixtures + born-blocking; pure/`forbid(unsafe_code)` (in-process) or capability-bounded (out-of-process); passes the generalized `gate_registration`.

### WS-D.5 Hermetic-by-construction packaging (the §WS-F tie-in)
A third-party gate authored as a Buck2/Bazel target via the `oya_ci_gate(...)` macro (§WS-F.5) inherits the hermetic mode FOR FREE; the `GateManifest.capabilities` map directly onto the backend's action declared-inputs/sandbox, so the §WS-D trust boundary and the §WS-F hermeticity boundary are ONE boundary declared once. An external gate-pack in `[gates].enabled` is in the build graph, so `rdeps`/affected-set + CAS apply identically. (The D3 out-of-process carrier remains the any-language/strong-sandbox path.)

---

## §WS-E — CLOUD-SCALE EXECUTION ARCHITECTURE (control-plane + runner; DESIGN-ONLY, Stage-5 gated)

> **Stage 5. Entry-gate:** floor landed + migration unblocked + §WS-D + §WS-F. Owned-plane execution hard-gated (PM-6); the live required context never moves until an owned runner re-proves byte-parity.

**Headline:** an owned, thin control-plane — trigger intake → run record → scheduler → shard fan-out across an autoscaling worker fleet → fan-in verdict → status posted back to the forge as the `oya-ci-required` required context — built around a `Runner` abstraction whose first impl is the LIVE GitHub Actions matrix (unchanged) and second is an owned scale-to-zero↔burst fleet. It borrows canonical hyperscale PATTERNS (Prow tide/merge-queue + plank/sinker, Tekton results, Argo DAG sharding, Bazel/Buck2 RBE + content-addressed remote cache, ARC autoscaling) around an UNCHANGED gate core ("one logic, two runners").

> **Verified basis:** the workflow already commits to this ("one logic, two runners", `oya-ci-required.yml` 76-77; `workflow_call`-renames-checks caveat 74-75). Triggers `workflow_dispatch`/`pull_request`/`merge_group` are wired (23-26, verified) — merge-queue is live. The gate core sees only faces + baselines, never a runner — so a new plane is additive.

### WS-E.1 Control plane (how it's controlled)
- **Trigger sources:** PR / push / merge-queue / cron / manual. Each creates a **run record** (id, repo, ref, trigger, tenant, config-digest).
- **Run lifecycle:** queued → scheduled → sharded → executing → fan-in verdict → status posted back as the SINGLE required context `oya-ci-required` (name MUST be preserved — the `workflow_call` caveat applies to the owned plane too).
- **Status authority:** the fan-in verdict is green IFF every constituent gate lane is green (existing semantics preserved).

### WS-E.2 Scheduling (how it's scheduled)
Queueing + prioritization + fairness/quotas; merge-queue/batching (Prow tide); concurrency control; dedup of redundant runs; cron for scheduled conformance sweeps.

### WS-E.3 Distributed execution + scale (how it's run) — patterns mapped
| Canonical pattern | oya-ci mapping |
|---|---|
| Argo DAG / Tekton pipeline | the gate matrix is a fan-out DAG; shard gates across workers; fan-in is the existing `oya-ci-required` join. |
| Bazel/Buck2 affected-set | only run gates whose INPUT changed (designed in §WS-F; the repo already uses Buck2, verified). |
| RBE + content-addressed remote cache | cache build + producer-face outputs by content digest; reuse across runs/tenants (the producer's "generate-once-share" is this in miniature, verified). |
| GitHub ARC autoscaling | runner #1 interim scaling (Stage 5a). |
| Prow plank/sinker | owned-plane run-executor + run-GC/reaper. |
| Hermetic + reproducible builds | the byte-for-byte determinism the engine ALREADY enforces (registry-drift committed==regenerated) is the cache-correctness precondition. |
- **Idempotency + retry + flake-handling:** runs idempotent on (ref, config-digest); transient retries; flaky gates quarantined WITHOUT changing the verdict.
- **[R-1]:** owned-runner sandbox substrate (Firecracker microVM vs gVisor vs container vs WASM) for the out-of-process/untrusted gate path — follow-up research, NOT asserted.
- **[R-2]:** affected-set false-negative safety — dedicated design + the Buck2/Bazel literature; flagged, not asserted.

### WS-E.4 The `Runner` abstraction ("one logic, two runners")
A `Runner` trait: `schedule(run) -> shards`, `execute(shard) -> gate_results`, `report(verdict)`. Runner #1 = GitHub Actions matrix (live, unchanged). Runner #2 = owned autoscaling fleet. Cutover is a per-runner byte-parity proof, never a flag-day.

### WS-E.5 Cost + multi-tenancy + observability at scale
Multi-tenancy + quotas (precedent: `oya-shared-tenant-quota-kernel`, `oya-check-tenant-cost-labels-coverage` ADR-0199, `oya-check-cost-budget`, verified); cost attribution; observability (precedent: `oya-check-otel-trace-propagation`/`oya-check-slo-coverage`/`oya-check-metric-cardinality`, verified); SLOs (time-to-first-feedback, p95 run latency, required-context post-back latency).

### WS-E.6 Staging discipline (PM-6)
DESIGN-ONLY. Execution is Stage 5, AFTER floor + §WS-D. Interim = Stage 5a (ARC + remote cache on the existing matrix). The owned plane (5b) is its OWN ralplan; the live matrix runs unchanged until runner #2 re-proves byte-parity. **Non-goal stamped:** Stage 5 does not begin until the floor has landed and the migration is unblocked.

---

## §WS-F — HERMETIC EXECUTION BACKENDS (`cargo`|`buck2`|`bazel`; the founder's "hermeticity for others"; Stage 3.5)

> **Stage 3.5. Entry-gate:** floor landed. **MF-2 entry-gate (git-history vs hermeticity):** git-derived gates (staleness-reaper / last-touch-class) read git history, which is NOT a declared Buck2/Bazel sandbox input by default. Before a hermetic backend claims to reproduce those gates' verdicts, DECIDE the stance — content-addressed-history-input (feed the relevant history range as a declared, content-addressed input) vs cargo-only-for-those-gates (keep git-derived gates on the cargo fallback). Scope the hermetic-reproduction claim to the gates whose inputs ARE content-addressable; git-derived gates are explicitly carved until this is settled. This is a §WS-F design entry-gate, not a floor blocker.

**Headline (the runner execution-MODE taxonomy):** ONE gate logic, TWO modes selected by capability/config, orthogonal to the §WS-E `Runner` (WHERE) — (a) `cargo` mode = the portable, zero-build-system FALLBACK (= today's live `oya-ci-required` matrix, verified `oya-ci-required.yml:98,119,138`); (b) `buck2`/`bazel` modes = the HERMETIC modes (each gate runs as a build-system target with sandboxed DECLARED inputs, content-addressed cache, affected-target selection), reproducible on any adopter's box. The §WS-E cloud plane ORCHESTRATES whichever mode but is NOT a prerequisite. Hermetic backends are OPTIONAL, never a hard dependency (PM-8); cargo-mode is permanent.

> **Verified basis (cited, against `source`):**
> - `.buckconfig` + `.buckroot` exist at source root (verified); real Buck2 cell graph.
> - **Every gate is ALREADY a Buck2 target** — each `cloud/cloud-ci/gates/<gate>/BUCK` defines `rust_library` + `rust_test -unittest` + (producer-coupled) a `-gate` `rust_test`; the PRODUCER is the `rust_binary -bin`. So "run gates via Buck2" runs EXISTING targets — no gate-code change. *(Verified-correction: the gates are `rust_library`+`rust_test`; only the producer is a `rust_binary`.)*
> - **CI runs gates via `cargo test` TODAY, NOT Buck2** (`oya-ci-required.yml:98,119,138`; zero `buck2` hits in `.github/workflows/`, verified) — so buck2-MODE is genuinely NEW work.
> - **The affected-set substrate already exists and is FAIL-CLOSED** — `infra/ci/buck2-affected-gate.sh` (6186B, executable, verified): `git merge-base` diff → classify graph-relevant changes → `buck2 uquery owner()` (BUCK-file → whole-package) → `buck2 uquery 'rdeps(//..., %Ss)' @argfile` → `buck2 build`+`test`; a graph-relevant change with no owner or any `uquery`/`rdeps` error is a FATAL refusal (documented as fixing the `2>/dev/null||true` false-pass bug; @argfile+`%Ss` to 1689 owners).
> - **The canon already sanctions Buck2 as the CI substrate** — ADR-0513 retains `buck2-affected-gate.sh`; ADR-0514 specifies hermetic toolchain → `buck2 build/test with NativeLink CAS/RE` + trunk-sourced affected-gate + structured buck2-event-log observability (verified).

### WS-F.1 The unified runner model (ONE model, three orthogonal axes — reconciles §WS-E)
A run is described by three orthogonal choices; the gate logic is invariant under all three:
1. **MODE / BACKEND** *(§WS-F)*: `cargo` (fallback) | `buck2` | `bazel`. Selected by capability detection + config.
2. **RUNNER** *(§WS-E.4)*: runner #1 = GitHub Actions matrix (live) | runner #2 = owned fleet. Carries whichever MODE.
3. **CONTROL-PLANE** *(§WS-E.1-E.3)*: GitHub-native (today) | owned thin plane (Stage 5b). Distributes the hermetic mode via RBE at scale.

`Runner::execute(shard)` simply invokes the gate in the selected MODE. The `Runner` conformance proof (§WS-E) and the MODE/backend-parity proof (§WS-F.4) are the two independent byte-parity gates.

### WS-F.2 Hermetic mode (design)
- **Invocation:** a gate runs as `buck2 test //cloud/cloud-ci/gates/<gate>:<gate>-unittest` (EXISTING targets, verified); producer regen via `buck2 run …-bin -- --face baseline`.
- **Sandboxed declared inputs:** make the producer faces, the baseline, and `oya-ci.toml` EXPLICIT declared inputs of the gate run (today implicit reads of `specs/*`/`docs/*`), so the sandbox is complete and the affected-graph (WS-F.3) is correct.
- **Content-addressed cache:** unchanged actions served from CAS (later §WS-E RBE) keyed by input digest — the engine's determinism (registry-drift committed==regenerated) is the cache-correctness precondition.
- **Affected-target selection:** `infra/ci/buck2-affected-gate.sh` (verified) selects only gates whose input closure changed; fail-closed safety is PM-9's mitigation.
- **Wiring (Stage 3.5):** add a hermetic-mode lane ADVISORY-parallel to the cargo lanes (NOT replacing them); fan-in stays named `oya-ci-required` (workflow_call caveat).

### WS-F.3 Affected-set correctness (the key risk; PM-9; R-2)
Every gate consumes a producer-built face, so the producer / a baseline / `oya-ci.toml` is a SHARED upstream input. False-negative safety requires a change to any shared input to mark ALL dependent gates affected, never zero. Approach (all fail-closed, layered): model shared inputs as real Buck2 deps so `rdeps()` returns the dependent gates; fail closed on graph gaps (verified FATAL refusal); a full-run backstop (periodic + every-merge + merge-queue batch); affected-set is an OPTIMIZATION of an already-green full run, never the sole gate, until R-2 is settled. **MF-2:** git-derived gates are carved from the affected-set/hermetic claim until their content-addressed-history stance is decided.

### WS-F.4 Mode/backend-parity proof + roll-out (live-green discipline, mirrors §WS-E)
The hermetic mode may NOT gate anything until proven verdict-identical to cargo-mode: the MODE-parity test asserts the same per-gate verdicts + a byte-identical regenerated baseline (79/233+all) under `cargo test -p <gate>` and `buck2 test //...<gate>`. Roll-out: (1) advisory parallel lane proving parity; (2) once durable, hermetic-mode (full run) MAY become the required context with cargo retained as fallback; (3) affected-set only after WS-F.3/R-2, always with the full-run backstop. The live `oya-ci-required` (cargo-mode) stays GREEN + authoritative throughout.

### WS-F.5 Hermeticity-for-others = the headline adopter benefit (+ §WS-D tie-in)
An adopter WITH a hermetic backend gets reproducible, cacheable, affected-incremental runs OUT OF THE BOX; an adopter WITHOUT one falls back to `cargo` (still works). A third-party gate authored via the `oya_ci_gate(...)` macro inherits sandboxing + caching + affected-set hermetic-by-construction; `GateManifest.capabilities` map directly onto the backend's declared-inputs/sandbox — the §WS-D trust model and the §WS-F hermetic boundary are the SAME boundary expressed once. **One contract, two carriers (§WS-D) × one logic, N backends (§WS-F).**
- **`oya_ci_gate(...)` macro (design):** a thin Buck2/Bazel macro that, given `(id, srcs, contract_dep, faces, config_keys, capabilities)`, emits the gate's `rust_library`+`rust_test` with the contract crate as a dep and declared faces/config as inputs.

### WS-F.7 The `HERMETIC-BUILD-BACKEND` abstraction (bazel generalization)
- **The trait (deliberately minimal — PM-10):** `HermeticBuildBackend { fn affected_targets(&self, diff) -> BTreeSet<Target>; fn run_gate(&self, target) -> GateVerdict; fn cache_key(&self, target) -> Digest; fn regen_producer(&self) -> FaceSet; }`. Only the four capabilities BOTH Starlark tools share are above the trait.
- **`buck2` impl (verified-grounded):** `buck2 test //…<gate>`, affected-set via the verified fail-closed `infra/ci/buck2-affected-gate.sh`, producer regen via `buck2 run …-bin`. WE maintain this backend.
- **`bazel` impl (adopter-facing):** the SAME gate crates as `rules_rust` targets; affected-set via `bazel query 'rdeps(...)'`; cache/RBE via Bazel's protocol; the `oya_ci_gate(...)` macro gets a `bazel`-rule sibling; capabilities lower onto Bazel sandbox EXACTLY as onto Buck2.
- **`cargo` fallback:** unchanged universal floor.
- **Backend-conformance proof (PM-10):** `buck2` and `bazel` MUST emit byte-identical per-gate verdicts + affected-sets on the same diff/fixture; `bazel` stays ADVISORY until it passes; WE land only `buck2`, `bazel` is demand-driven.
- **[R-5]:** the `rules_rust`-vs-prelude mapping, the Bazel affected-set false-negative-safety argument (the bazel analogue of R-2), and the Bazel RBE/remote-cache choice — deferred to the bazel-backend's own design (OQ-19).

### WS-F.6 Staging discipline
Hermetic mode is **Stage 3.5** — a hermetic run mode AROUND/AFTER the config-driven floor; it does NOT require the owned cloud plane. Advisory-parallel first, then MAY become the required context with cargo as permanent fallback; affected-set gated on WS-F.3/R-2. **Non-goal stamped:** hermetic-mode never removes cargo-mode and never becomes the required context until proven verdict-identical (WS-F.4).

---

## §WS-G — REPRODUCIBLE BUILDS + MANAGED DEV-ENV + STRICT DEPENDENCY VERSIONING (config-driven, born-advisory)

> **Stage 3.5. Entry-gate:** floor landed. **MF-3 entry-gate (born-advisory-until-clean):** reproducibility gates ship born-ADVISORY using the EXISTING `advisory-until-infra` disposition mechanism (verified in `gate-disposition.json`), and are promoted to `baseline-block-on-new` ONLY once their enforcement infra is clean (the verified `deny.toml`-wired-but-`scripts/check.sh`-absent GAP is actually closed and the gates run green). No reproducibility claim ships as a blocking gate before its enforcement exists (PM-13). This is the mechanism, not a new invention.

**Headline:** make EVERY build input pinned + sandboxed + content-addressed and ENFORCE it with executable, config-driven gate-pack gates. Three pillars: (G.a) reproducible builds — pinned toolchain (`rust-toolchain.toml`=`1.95.0`, verified), `--locked` + content-addressed builds (ties to §WS-F), byte-reproducible artifacts (tracked goal, R-4); (G.b) managed dev-environments — a reproducible `oya-ci dev-env` bootstrap so contributor-local == CI == adopter; (G.c) strict dependency versioning — `Cargo.lock` committed (verified, 12376 lines) + CI-enforced no-drift, `cargo deny` (verified `deny.toml`), audited closure.

> **Verified basis + the precedent GAP this closes (cited):** `rust-toolchain.toml` pinned `channel="1.95.0"` (verified); `deny.toml` at root with a real policy (licenses allow-list + RustSec `[advisories]` yanked=deny + `[bans]`, verified); `Cargo.lock` committed (275606B/12376 lines, verified); `oya-check-dependency-seam` (ADR-0092 D13) cargo-audit-shell with failing/passing fixtures, report-only D14 (verified); supply-chain/license/vendor precedents (`oya-check-supply-chain` ADR-0039, `oya-check-vendor-recency`, `oya-check-vendor-lockin-discipline` ADR-0173, `oya-check-license-policy`, verified). **THE GAP:** `deny.toml`'s header CLAIMS `scripts/check.sh`+CI wiring — but `scripts/check.sh` DOES NOT EXIST and no workflow invokes `cargo deny`/`cargo audit`/`--locked`/`--frozen` (verified zero hits). The policy is authored but UNENFORCED — §WS-G's gates are the enforcement (PM-13's "reproducibility-claims-unverified" trap, already live in the repo).

### WS-G.1 Reproducible builds (G.a)
Toolchain-pin gate (vs `[repro].toolchain_channel`, default `rust-toolchain.toml`=`1.95.0`); `--locked` enforcement + lockfile-drift gate; content-addressed builds delegated to the §WS-F backend; byte-reproducible artifacts as the tracked GOAL (R-4 / OQ-18 — path-remap, `SOURCE_DATE_EPOCH`, codegen determinism), near-term posture is content-addressed + `--locked` + pinned-toolchain + audited, with a second-machine artifact-diff e2e as the proof, NOT asserted as already-bit-identical.

### WS-G.2 Managed dev-environments (G.b)
`oya-ci dev-env` bootstrap materializes the SAME pinned toolchain + `--locked` closure + gate toolchain locally as CI uses (local==CI==adopter). **Carrier choice (OQ-15):** `rust-toolchain.toml` pin (present) + ONE of {devcontainer, nix flake, `mise`/`.tool-versions`, an `oya-ci dev-env` Rust subcommand}. The repo TODAY has Dockerfiles but NO `.devcontainer`/`flake.nix`/`shell.nix`/`.tool-versions` (verified) — the carrier is NET-NEW; the pin exists to build on. Config-driven via `oya-ci.toml` `[dev_env]`.

### WS-G.3 Strict dependency versioning (G.c)
Lockfile-drift gate; `cargo deny check` gate (closes the verified GAP); audit/RustSec gate (promotes `oya-check-dependency-seam` cargo-audit-shell, verified report-only, to blocking — AFTER born-advisory clears, MF-3); vendor precedents reconciled into the pack; multi-ecosystem note (Cargo now; buck2/bazel third-party cells + GitHub-Actions pins are FUTURE targets, folding into §WS-I sync-repo-settings + §WS-H).

### WS-G.4 Gate-pack expression + staging
All §WS-G gates are CONFIG-DRIVEN gates in a new **`reproducibility` gate-pack** (alongside `core` + `rust-cargo`); an adopter ENABLES reproducibility via `[gates].enabled`. The byte-for-byte baseline invariant (79/233+all) must hold with them present. **Born-advisory (MF-3):** each ships via the `advisory-until-infra` disposition, promoted to blocking only after its enforcement infra is clean. **Non-goal stamped:** no reproducibility claim ships as a comment; each is an executable gate; bit-identity is a tracked goal (R-4), asserted nowhere as already-done.

---

## §WS-H — DEPENDABOT-EQUIVALENT: RUST-NATIVE AUTOMATED DEPENDENCY UPDATES (a bot on the §WS-I framework)

> **Stage 6. Entry-gate:** §WS-E host exists. ONE bot on the §WS-I framework; tied to §WS-G strict-versioning.

**Headline:** a Rust-native bot that detects outdated (`cargo outdated`-class) + vulnerable (`cargo audit`/RustSec) deps, opens GROUPED + SCHEDULED bump PRs, and lets oya-ci gate every PR — so an update that breaks ANY gate cannot merge.

> **Verified basis:** `oya-check-dependency-seam` cargo-audit-shell (ADR-0092 D13, report-only D14, failing/passing fixtures, verified) — the detection primitive; `deny.toml [advisories]` (RustSec, yanked=deny, verified) — the vuln policy; `Cargo.lock` (committed, verified) — the bump target; `oya-governance-pr-merge-gate-kernel` (pure-Rust merge-on-green, "no shellscript no mjs all rust", verified) — the auto-merge precedent. NO `.github/dependabot.yml` (verified); the all-Rust doctrine forbids adopting GitHub's YAML dependabot wholesale.

### WS-H.1 Detection
Outdated via a `cargo outdated`-class resolver over `Cargo.lock`; vulnerable via the RustSec DB (the `deny.toml [advisories]` + dependency-seam precedent), PRIORITIZED + ungrouped (PM-12f).

### WS-H.2 Bump-PR generation (grouped, scheduled — PM-12)
Grouped + scheduled by default (by ecosystem / semver-tier / config group) on a CADENCE, never one-per-crate over a 12376-line lockfile; config via `oya-ci.toml` `[dep_update]` (cadence, grouping, ignore/pin, auto-merge policy).

### WS-H.3 Gated-by-oya-ci (the whole point)
Every dep-update PR runs the FULL oya-ci fan-in (incl. the §WS-G gates once they clear born-advisory): a breaking bump stays RED. Auto-merge-on-green OPT-IN + conservative (patch/compatible only, via the verified merge-gate-kernel + §WS-E tide), never majors.

### WS-H.4 Multi-ecosystem + framework tie-in
Cargo now; buck2/bazel `third-party/` + prelude + GitHub-Actions pins are LATER ecosystems. §WS-H is implemented AS a §WS-I framework bot (the canonical first non-trivial bot), gated to the §WS-I staging.

---

## §WS-I — RUST-NATIVE REPO-AUTOMATION-BOT FRAMEWORK + SUITE (Rust equivalents of googleapis/repo-automation-bots)

> **Stage 6 (VERY LAST). Entry-gate:** §WS-E host exists (bots are webhook/event-driven, they NEED a host). **MF-5 entry-gate (bot-vs-CI lifecycle):** the earlier principle that "ONE control-plane hosts BOTH gates and bots" is DOWNGRADED to an OPEN DESIGN QUESTION (OQ-20). A bot's lifecycle is event-REACTIVE (forge webhook → react → act on the forge); a CI run is verdict-FAN-IN (gates → join → required-context). These are not obviously the same shape. Whether one plane cleanly hosts both, or they are co-resident-but-distinct services sharing the trigger-intake + observability substrate, is RESOLVED in this workstream's ralplan, not assumed here.

**Headline:** a Rust-native, webhook/event-driven REPO-AUTOMATION-BOT FRAMEWORK — a bot HOST + a bot SDK (paralleling the §WS-D gate SDK) so bots are productized, config-driven, third-party-contributable — plus the individual bots as Rust equivalents of the 19 bots inventoried from `github.com/googleapis/repo-automation-bots`. DESIGN-ONLY here; the LAST campaign; never blocks the floor, the migration, or the live green (PM-11). §WS-H is ONE bot on this framework.

> **Verified basis (cited):** `oya-shared-webhook-delivery-kernel` (ADR-0169: HMAC-SHA256 `Oya-Signature` + exponential-backoff retry + DLQ, verified) — the clean, forbidden-vocab-free webhook substrate; `oya-governance-pr-merge-gate-kernel` (pure-Rust merge-on-green replacing a RETIRED Node merge-gate, verified) — the merge-on-green precedent + all-Rust anchor; `oya-check-release-pack` + `oya-shared-semver-check-cli` + `docs/automation/changelog-pipeline.md` (verified) — release-please/changelog precedents; `oya-check-license-policy` + `oya-governance-license-policy-kernel` (verified) — license-header precedents; `CODEOWNERS` (verified) — assignment precedent. **CAUTION (verified):** an existing `oya/ci-webhook-gateway` references "Jenkins" — FORBIDDEN VOCAB (active eradication Tasks #25/#48); §WS-I builds on the CLEAN `oya-shared-webhook-delivery-kernel`, NOT the Jenkins-tainted gateway, and every bot-host dependency must carry no forbidden-vocab.

### WS-I.1 The framework (the bot HOST + the bot SDK)
- **Host:** webhook/event-driven (forge events: PR opened, push, comment, check-suite, release) via the verified `oya-shared-webhook-delivery-kernel` (HMAC-verified), dispatching to matching bots and posting back. **MF-5:** whether the host IS the §WS-E control-plane or a co-resident-but-distinct service is the OQ-20 open question — the event-reactive bot lifecycle vs the CI verdict-fan-in are not assumed to be one mechanism.
- **Bot SDK (parallels §WS-D):** ONE semver'd `oya-ci-bot-contract`-style crate publishing the bot trait (`fn on_event(&self, event, ctx) -> BotActions`), a `BotManifest` (id, subscribed events, capabilities), and the action vocabulary. Capability-declared + ENFORCED + signed exactly like §WS-D gate-packs (PM-5 reused).
- **Config-driven:** `oya-ci.toml` `[bots]` enables bots + per-bot config, mirroring `[gates].enabled`.
- **Third-party-contributable:** the same "write your first bot" scaffold + RED/GREEN conformance as §WS-D's "write your first gate".

### WS-I.2 The suite — repo-automation-bots → Rust-equivalent mapping (19 bots, WebFetched + mapped)

| # | google bot | Rust-equivalent | verified precedent |
|---|---|---|---|
| 1 | release-please | `oya-bot-release` (conventional-commit → bump + changelog + release PR) | `oya-check-release-pack` + `oya-shared-semver-check-cli` + `docs/automation/changelog-pipeline.md` |
| 2 | release-trigger | folded into `oya-bot-release` (trigger phase) | §WS-E trigger-intake |
| 3 | blunderbuss | `oya-bot-assign` (auto-assign by CODEOWNERS/area) | `CODEOWNERS` |
| 4 | auto-label | `oya-bot-autolabel` (path/area-derived labels) | path-derived |
| 5 | auto-approve | `oya-bot-autoapprove` (config-matched, gated by full oya-ci) | `oya-governance-pr-merge-gate-kernel` |
| 6 | merge-on-green | `oya-bot-merge` (merge-on-`oya-ci-required`-green + tide) | merge-gate-kernel + §WS-E tide |
| 7 | do-not-merge | a `oya-bot-merge` policy rule (label-gate) | merge-gate-kernel |
| 8 | conventional-commit-lint | `oya-bot-commitlint` (born-blocking RED on non-conventional msg) | feeds `oya-bot-release` |
| 9 | license-header-lint | `oya-bot-header` (or a `core`-pack gate) | `oya-check-license-policy` + `oya-governance-license-policy-kernel` |
| 10 | header-checker | merged into `oya-bot-header` | as above |
| 11 | sync-repo-settings | `oya-bot-reposettings` (declarative from `oya-ci.toml`) | config-driven |
| 12 | repo-metadata-lint | `oya-bot-repometa` (gate-shaped) | gate contract (§WS-D) candidate |
| 13 | policy | `oya-bot-policy` (repo-config conformance) | config-driven |
| 14 | label-sync | a `oya-bot-reposettings` capability | config-driven |
| 15 | snippet-bot | `oya-bot-snippet` (gate-shaped) | gate contract (§WS-D) candidate |
| 16 | generated-files-bot | `oya-bot-genfiles` | registry-drift/generated-faces precedent |
| 17 | failurechecker | `oya-bot-failurecheck` (watch §WS-E run records) | §WS-E run-record + plank/sinker |
| 18 | flakybot | `oya-bot-flaky` (consume §WS-E flake-quarantine) | §WS-E flake-handling |
| 19 | cherry-pick-bot | `oya-bot-cherrypick` (label-driven backport) | §WS-E scheduler + merge-gate |
| 20 | trusted-contribution | `oya-bot-trusted` (trust-list → allow CI on external PRs) | §WS-E trigger-intake + §WS-D PM-5 trust |
| (—) | §WS-H dep-update | `oya-bot-depupdate` (outdated+RustSec → grouped bump PRs) | `oya-check-dependency-seam` cargo-audit-shell (the canonical first bot) |

**Design notes:** several "bots" (#12/#15/#16) are GATE-SHAPED and may be authored as §WS-D gates; the framework lets a contribution be EITHER (bot if event-reactive, gate if face-evaluating) — but per MF-5 the bot-vs-gate lifecycle distinction is REAL, not cosmetic (a bot reacts to an event; a gate evaluates a face within a CI run). The release family (#1/#2/#8) consolidates into `oya-bot-release` + `oya-bot-commitlint`; the merge family (#5/#6/#7/#14-as-policy) onto the verified merge-gate-kernel + §WS-E tide.

### WS-I.3 Trust + host + observability
Trust: §WS-D PM-5 reused verbatim (bots declare capabilities, host ENFORCES, artifacts signed, default-deny; a bot's CI-gating verdict can only ADD findings). Host + observability: run-id-keyed structured events, bot SLOs, per-bot/per-tenant metrics (otel/slo/cost precedents). **MF-5:** the host's relationship to the §WS-E plane is OQ-20.

### WS-I.4 Staging discipline (PM-11)
DESIGN-ONLY. The LAST campaign with its OWN ralplan, strictly after the floor, §WS-D, and on/after the §WS-E host. Bots land INCREMENTALLY, highest-value first (`oya-bot-release` + `oya-bot-depupdate` + `oya-bot-commitlint`). **Non-goal stamped:** no bot is built until the host exists and the migration is unblocked; NONE is a floor/migration/live-green prerequisite.

---

## ADR ADDENDA (extension dimensions)

> These ADR addenda extend the floor ADR (in the floor doc). They are captured as the north-star ADR record, to be ratified workstream-by-workstream (NOT door:one-way).

### 9b. ADR ADDENDUM — extensibility (third-party contribution) + cloud-scale execution
- **Decision:** **(D-EXT)** publish the proven gate surface as ONE semver'd crate `oya-ci-gate-contract` (`Finding`/`evaluate_keyed`/`GateManifest`); generalize the directory-driven `gate_registration` discovery to external packs declared by coordinate (respecting input KIND, floor §3.5); declare + ENFORCE per-gate capabilities; sign + attest packs; ship a scaffold + RED/GREEN conformance; offer an out-of-process JSON carrier as the any-language/strong-sandbox secondary path. **(D-SCALE)** an owned thin control-plane around a `Runner` abstraction (runner #1 = live matrix unchanged, runner #2 = owned fleet), borrowing canonical hyperscale patterns, with multi-tenancy/quotas/cost/observability. DESIGN-ONLY; staged AFTER the floor.
- **Decision Drivers:** D4 extensibility-without-ossification; D5 hyperscale-without-scope-creep (+ binding D1/D2/D3).
- **Alternatives considered — D-EXT:** D1 dynamic `dylib` — rejected (no stable Rust ABI; weak sandbox). D3 out-of-process — adopted SECONDARY. D2 semver'd source-crate + manifest — chosen PRIMARY. **D-SCALE:** E1 Tekton/Argo/Prow wholesale — rejected as primary (foreign-lifecycle coupling), PATTERNS borrowed. E3 GitHub-only — adopted as Stage-5a interim. E2 owned thin plane + `Runner` — chosen.
- **Why chosen:** D2+D3 give the founder's "stable, versioned contract … outside the monorepo … as easy as a GitHub Action" with the smallest change + a real sandbox. E2 gives "architect for hyperscale" while honoring "owned runner later, one-logic-two-runners" and the supreme constraint that the migration is never blocked.
- **Consequences (MF-4 reconciliation):** the gate SURFACE becomes a published API the project versions — this MUTATES `Finding`/`evaluate_keyed` (vs the floor's FIXED ratchet/firewall/registry-drift/sign-off), and §WS-D carries its OWN byte-parity proof (in-tree gates via the contract still reproduce 79/233+all); the trust boundary expands to third-party code (default-deny capabilities + signed packs + ratchet integrity); the execution plane becomes a cloud product; Stage 5 is its own deferred campaign; the byte-for-byte invariant remains supreme across both dimensions.
- **Follow-ups:** OQ-5 contract trait shape + semver policy; OQ-6 gate-pack signing key custody + trusted-source governance; OQ-7 [R-1] owned-runner sandbox substrate; OQ-8 [R-2] affected-set false-negative safety; OQ-9 [R-3] control-plane state store + scheduler (build vs adopt); OQ-10 merge-queue/tide semantics + forge-status posting preserving the `oya-ci-required` context name.

### 9c. ADR ADDENDUM — a hermetic execution MODE / backend
- **Decision (D-MODE):** make a hermetic build backend a first-class, OPTIONAL execution MODE (`buck2` first, `bazel` sibling), distinct from and EARLIER than the §WS-E cloud plane, with `cargo` the permanent universal FALLBACK. The gates are ALREADY Buck2 targets (verified); hermetic-mode runs those EXISTING targets with sandboxed declared inputs + content-addressed cache + affected-set (the verified fail-closed `infra/ci/buck2-affected-gate.sh`). Land as Stage 3.5, advisory-parallel until verdict-identical, then it MAY become the required context with cargo retained. Adopt the unified 3-axis runner model (MODE × RUNNER × CONTROL-PLANE).
- **Decision Drivers:** D6 hermeticity-portable-to-adopters (+ binding D1/D2/D3) + the Accepted ADR-0513/0514 CI substrate.
- **Alternatives considered:** F1 cargo-only (the fallback half; fails D6 alone); F2 any single backend as a HARD dependency — rejected (PM-8; the "hermetic for us / little value to others" anti-pattern); F3/F4 dual/N-backend — chosen (F4 generalizes F3 to bazel).
- **Why chosen:** delivers "buck2 support … hermeticity for others" as a REAL earlier substrate while keeping the backend OPTIONAL; changes the execution SUBSTRATE, not the gate logic; reconciles with Accepted ADR-0513/0514.
- **Consequences:** a hermetic, reproducible, cacheable, affected-incremental run mode that ports to adopters; external gate-packs hermetic-by-construction via the `oya_ci_gate(...)` macro. Cost: a new hermetic-mode CI lane + the work to make producer faces/baseline/`oya-ci.toml` EXPLICIT inputs; a permanent obligation to keep cargo-mode first-class; affected-set fail-closed + backstopped + gated on R-2. **MF-2:** git-derived gates carved from the hermetic-reproduction claim until their content-addressed-history stance is decided. The byte-for-byte invariant remains supreme.
- **Follow-ups:** OQ-11 the `oya_ci_gate(...)` macro shape + cell; OQ-12 making producer faces/baseline/`oya-ci.toml` explicit Buck2 inputs without breaking the existing graph; OQ-13 the mode-selection surface; OQ-14 local + remote CAS/RE backend choice (NativeLink); OQ-8 [R-2] affected-set false-negative safety (gating before affected-set may replace the full-run gate).

### 9d. ADR ADDENDUM — bazel backend + reproducibility/dev-env/versioning + dependabot-equiv + repo-automation-bot suite + polish
- **Decision (D-BACKEND):** generalize §WS-F to a `HERMETIC-BUILD-BACKEND` trait (`buck2` ours + `bazel` adopter sibling + `cargo` fallback; Option F4); a backend-conformance proof (buck2 ≡ bazel) is mandatory, `bazel` ADVISORY until it passes (PM-10). **(D-REPRO §WS-G):** reproducible-builds + dev-env + strict-versioning as config-driven `reproducibility`-gate-pack gates, **born-ADVISORY via the existing `advisory-until-infra` disposition (MF-3)**, closing the verified `deny.toml` GAP; every claim an executable gate; bit-identity a tracked goal (R-4). **(D-DEPBOT §WS-H):** a Rust-native dependabot-equivalent (`oya-bot-depupdate`), each PR gated, opt-in conservative auto-merge; ONE bot on §WS-I. **(D-BOTS §WS-I):** a Rust-native repo-automation-bot FRAMEWORK + SDK + the 19-bot mapping, config-driven + capability-declared/enforced/signed + third-party-contributable, on the verified clean precedents (NOT the Jenkins gateway). **(D-POLISH):** one config model + one contract/SDK pattern + one control-plane + one hermetic-backend story — **with the MF-5 caveat that "one control-plane hosts both gates and bots" is an OPEN DESIGN QUESTION (OQ-20), not an asserted invariant.** DESIGN-ONLY.
- **Decision Drivers:** D7 adopter-reach-without-lock-in + reproducibility/automation-as-product (+ binding D1/D2/D3/D6) + the all-Rust doctrine + Accepted ADR-0039/0092/0169/0173/0513/0514.
- **Alternatives considered:** D-BACKEND F2 (any single backend hard-dep) rejected (PM-8). D-DEPBOT GitHub YAML dependabot wholesale rejected (all-Rust; no oya-ci gating; PM-12). D-BOTS Node.js googleapis bots / foreign host rejected (retired Node merge-gate; foreign-lifecycle coupling); one big monolith rejected (no third-party contribution). D-REPRO comment-only enforcement rejected (the verified `deny.toml` GAP IS this failure; PM-13).
- **Why chosen:** F4 gives bazel reach with the smallest change (a trait + a second binding) bounding drift (PM-10); §WS-G/H/I deliver the founder's reproducibility/dep-update/repo-bot mandates each on a VERIFIED precedent, config-driven, STAGED LATER as own-campaigns so the floor stays first and the live green is never threatened.
- **Consequences:** backend-agnostic hermeticity, end-to-end enforced reproducibility (born-advisory until clean, MF-3), a gated dep-update bot, and a third-party-contributable bot suite. Obligations: maintain the backend-conformance suite (PM-10); keep `cargo` + the live matrix the floor/authority; reproducibility gates must be EXECUTABLE (PM-13); the dep-bot grouped/gated/opt-in (PM-12); the bot suite hard-gated LAST (PM-11) on forbidden-vocab-free precedents; the bot-vs-CI lifecycle question (MF-5/OQ-20) resolved in the §WS-I ralplan. The byte-for-byte invariant remains supreme across ALL dimensions.
- **Follow-ups:** OQ-15 dev-env carrier; OQ-16 dep-bot grouping/cadence/auto-merge policy + sanctioned RustSec ignores; OQ-17 bot SDK trait shape + semver + the gate-vs-bot boundary; OQ-18 [R-4] full byte-for-byte Rust build reproducibility; OQ-19 [R-5] the bazel backend mapping (`rules_rust` vs prelude, bazel affected-set safety, RBE choice); **OQ-20 [MF-5] the bot-host's relationship to the §WS-E control-plane (a §WS-E sub-component vs a co-resident-but-distinct service) — the event-reactive-bot vs CI-verdict-fan-in lifecycle question — + forbidden-vocab audit of every bot-host dependency.**

---

## DEEP-RESEARCH FLAGS (R-1..R-5) + OPEN QUESTIONS (OQ-1..OQ-20) — REGISTER

**Deep-research flags (asserted nowhere; each gates its workstream):**
- **R-1** — owned-runner sandbox substrate (Firecracker microVM vs gVisor vs container vs WASM) for untrusted/out-of-process gates (§WS-E; OQ-7).
- **R-2** — affected-set computation with false-negative safety (Buck2/Bazel affected-targets literature) (§WS-F/§WS-E; OQ-8). Gating before affected-set may replace the full-run gate.
- **R-3** — control-plane state store + scheduler (build vs adopt Tekton/Argo/Prow components) (§WS-E; OQ-9).
- **R-4** — full byte-for-byte Rust build reproducibility (build-path remapping, `SOURCE_DATE_EPOCH`, codegen determinism) (§WS-G; OQ-18). Near-term posture: content-addressed + `--locked` + pinned-toolchain + audited; bit-identity a tracked goal.
- **R-5** — the `bazel` backend mapping (`rules_rust` vs prelude, bazel affected-set false-negative safety = the R-2 analogue, bazel RBE/remote-cache choice) (§WS-F.7; OQ-19).

**Open questions (OQ-1..OQ-20):** OQ-1 config format (TOML vs JSON) [floor-owned]; OQ-2 composite-action vs copy-in-matrix as the canonical external surface [floor §5]; OQ-3 when to trigger separate-repo extraction (→ §WS-D); OQ-4 first non-Rust pack target (→ §WS-E); OQ-5 contract trait shape + semver; OQ-6 gate-pack signing key custody + trusted-source governance; OQ-7 [R-1]; OQ-8 [R-2]; OQ-9 [R-3]; OQ-10 owned-runner forge-status posting preserving the `oya-ci-required` context name; OQ-11 `oya_ci_gate(...)` macro shape + cell; OQ-12 explicit Buck2 inputs for faces/baseline/config; OQ-13 mode-selection surface; OQ-14 CAS/RE backend choice (NativeLink); OQ-15 dev-env carrier; OQ-16 dep-bot grouping/cadence/auto-merge + sanctioned RustSec ignores; OQ-17 bot SDK trait shape + gate-vs-bot boundary; OQ-18 [R-4]; OQ-19 [R-5]; **OQ-20 [MF-5] bot-host vs §WS-E control-plane lifecycle relationship + forbidden-vocab audit of bot-host deps.**

---

*PLANNER REVISION pass complete (2026-06-08). This PRODUCT doc was SPLIT from the former single productization plan per Architect+Critic consensus (MF-6); it holds §WS-D/§WS-E/§WS-F/§WS-G/§WS-H/§WS-I + all R-/OQ- + ADR addenda §9b/§9c/§9d, RELOCATED (not lost). STATUS: pending-design, NOT door:one-way — ratified workstream-by-workstream, each its own future ralplan, all gated AFTER the floor + migration, none blocking them. MF-2/MF-3/MF-5 reframed as STAGE ENTRY-GATES (§0 register + §WS-F/§WS-G/§WS-I headers); MF-4 reconciled (§WS-D mutates the Finding/evaluate_keyed surface with its own byte-parity proof, vs the floor's FIXED engine). All verified facts + RALPLAN-DR rigor + ADRs preserved. The migration-critical floor lives in `OYA-CI-CONFORMANCE-FLOOR-PLAN.md`.*
