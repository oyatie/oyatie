# GATE-PREFIX-RENAME-PLAN — F-0025 decoupled sub-item

**Status:** PLAN-READY (no source mutation performed; this doc is the only artifact written).
**Scope:** Apply the canonical `oya-` crate-name prefix to the 7 firewall gate crates under
`cloud/cloud-ci/gates/` of `/Users/jasonlee/Developer/source` (branch `cleanup/whole-tree-2026-06-07`).
**Authority:** `FINDINGS-LEDGER.md` F-0025 (the firewall-gate prefix is "the one clearly-valid
sub-item, decoupled from the enum question"); ADR-0056 canonical BNF `oya-<microservice>-(<bc-tokens>-)?<layer>`
("FIRST token is `oya`" for every oya-* Rust crate).
**Date:** 2026-06-07.

---

## 0. The two naming layers (do NOT conflate)

There are TWO distinct identifier families on these crates. **Only Layer A is in scope for F-0025.**

| Layer | What it is | Current form | In scope? |
|-------|-----------|--------------|-----------|
| **A. Build identity** | Cargo `[package].name`, Cargo `[[bin]].name`, BUCK target `name`, `cargo run -p <name>`, workspace `[workspace.members]` paths | `total-accounting`, `cloud-ci-firewall`, `accounting-registry-producer`, … (no `oya-` prefix) | **YES — rename to `oya-cloud-ci-*`** |
| **B. Runtime gate-ID strings** | `GATE_ID` consts, the `gates: { ... }` keys in fixtures + baseline JSON, ADR-0515 prose, `specs/phase0-automation-matrix.json` ids | already `cloud-ci-total-accounting`, `cloud-ci-cross-artifact-agreement`, … | **NO — already prefixed at the `cloud-ci-` level; leave untouched** |

Layer B is the wire/data contract consumed by fixtures and the committed baseline. Renaming it would
churn every RED/GREEN fixture + the 3.5 MB baseline face for zero F-0025 benefit and re-open the
"does the gate ID need `oya-`?" question that F-0025 explicitly did NOT decide. **This plan touches Layer A only.**
Crate lib `name` underscores (`total_accounting`, `cloud_ci_firewall`, …) and the BUCK `crate =`/`crate_root =`
attrs are the Rust crate (import) identity; see §3 for whether they move.

---

## 1. Rename table (Layer A — old → new)

Prefix rule: `oya-cloud-ci-` (microservice = `cloud-ci`, the gate name becomes the remaining tokens).
This keeps every crate ADR-0056-conformant (`oya-` first token) and groups all 7 under the one `cloud-ci`
microservice. No existing workspace member uses `oya-cloud-ci-*` today (verified: 0 collisions tree-wide).

| # | Dir (UNCHANGED) | Cargo `[package].name` old → new | `[[bin]].name` old → new | BUCK lib target old → new | BUCK unittest target old → new | BUCK gate target old → new |
|---|------|------|------|------|------|------|
| 1 | `total-accounting` | `total-accounting` → `oya-cloud-ci-total-accounting` | — | `total-accounting` → `oya-cloud-ci-total-accounting` | `total-accounting-unittest` → `oya-cloud-ci-total-accounting-unittest` | `total-accounting-gate` → `oya-cloud-ci-total-accounting-gate` |
| 2 | `cross-artifact-agreement` | `cross-artifact-agreement` → `oya-cloud-ci-cross-artifact-agreement` | — | same → `oya-cloud-ci-cross-artifact-agreement` | `…-unittest` → `oya-cloud-ci-cross-artifact-agreement-unittest` | `…-gate` → `oya-cloud-ci-cross-artifact-agreement-gate` |
| 3 | `staleness-reaper` | `staleness-reaper` → `oya-cloud-ci-staleness-reaper` | — | same → `oya-cloud-ci-staleness-reaper` | `…-unittest` → `oya-cloud-ci-staleness-reaper-unittest` | `…-gate` → `oya-cloud-ci-staleness-reaper-gate` |
| 4 | `automation-ratchet` | `automation-ratchet` → `oya-cloud-ci-automation-ratchet` | — | same → `oya-cloud-ci-automation-ratchet` | `…-unittest` → `oya-cloud-ci-automation-ratchet-unittest` | `…-gate` → `oya-cloud-ci-automation-ratchet-gate` |
| 5 | `registry-drift` | `registry-drift` → `oya-cloud-ci-registry-drift` | — | same → `oya-cloud-ci-registry-drift` | (none) | `registry-drift-gate` → `oya-cloud-ci-registry-drift-gate` |
| 6 | `cloud-ci-firewall` | `cloud-ci-firewall` → `oya-cloud-ci-firewall` | — | same → `oya-cloud-ci-firewall` | `cloud-ci-firewall-unittest` → `oya-cloud-ci-firewall-unittest` | `cloud-ci-firewall-gate` → `oya-cloud-ci-firewall-gate` |
| 7 | `accounting-registry-producer` | `accounting-registry-producer` → `oya-cloud-ci-accounting-registry-producer` | `accounting-registry-producer` → `oya-cloud-ci-accounting-registry-producer` | `accounting-registry-producer` → `oya-cloud-ci-accounting-registry-producer` | `…-unittest` → `oya-cloud-ci-accounting-registry-producer-unittest` | `accounting-registry-producer-bin` → `oya-cloud-ci-accounting-registry-producer-bin` |

**Note on #6:** `cloud-ci-firewall` becomes `oya-cloud-ci-firewall` (NOT `oya-cloud-ci-cloud-ci-firewall`) — the
microservice token `cloud-ci` is already its prefix; collapsing the duplicate keeps it BNF-clean. Confirm this
deduplication with the founder; the mechanical alternative `oya-cloud-ci-firewall` is what is used throughout
this plan.

**Directory names are intentionally NOT renamed.** All BUCK package labels (`//cloud/cloud-ci/gates/<dir>:…`)
key off the directory, not the target name; keeping dirs stable avoids touching the ~17.6k `path` rows in the
generated registry that reference `cloud/cloud-ci/gates/<dir>/...` file paths. (If founder wants dirs renamed too,
that is a SEPARATE, larger blast — see Risk R5.)

---

## 2. Reference-update list (exhaustive — every file that must change)

Grouped by the rename it tracks. Counts are real grep hits on the source tree.

### 2.1 Workspace manifest
- `Cargo.toml` lines 235–241 — `[workspace.members]` paths are dir-based, so they **do NOT change** if dirs
  stay. **However** `Cargo.lock` carries `name = "<pkg>"` entries (lines 14, 11113 for total-accounting; 10, 1155;
  13, 10621; etc.) — these regenerate automatically on the next `cargo` invocation; do not hand-edit. (Lock is
  in `serialize_global_artifacts` per goal.json — coordinate so no other slice is mid-flight on it.)

### 2.2 Each gate's own `Cargo.toml` (7 files)
- `[package].name` (and `[[bin]].name` for producer). Plus the producer's `[dependencies]` path-deps to the 4
  gate libs (`accounting-registry-producer/Cargo.toml:31-34`):
  `total-accounting = { path = "../total-accounting" }` → `oya-cloud-ci-total-accounting = { path = "../total-accounting" }`
  (×4: total-accounting, cross-artifact-agreement, staleness-reaper, automation-ratchet). Path stays; the dep KEY
  is the package name and MUST change.

### 2.3 Each gate's own `BUCK` (7 files)
- `name =` on the `rust_library`, `rust_test`(s), and (producer) `rust_binary` per §1 table.
- Cross-package BUCK `deps` labels — the LABEL keys off dir so the path part is stable, but the **target name
  after the colon** changes. Concrete edits:
  - `accounting-registry-producer/BUCK` deps (lines 10–13, 26–29, 42–45): `//cloud/cloud-ci/gates/total-accounting:total-accounting`
    → `//…/total-accounting:oya-cloud-ci-total-accounting` (×4 gate libs, ×3 occurrences each = 12 edits) + the
    self-dep `//…/accounting-registry-producer:accounting-registry-producer` (line 41) → `…:oya-cloud-ci-accounting-registry-producer`.
  - The 5 corpus-gate BUCKs each have `":self-lib"` + `"//…/accounting-registry-producer:accounting-registry-producer"`:
    - `total-accounting/BUCK:35-36`, `cross-artifact-agreement/BUCK:36-37`, `staleness-reaper/BUCK:35-36`,
      `automation-ratchet/BUCK:36-37`, `cloud-ci-firewall/BUCK:35-36`, `registry-drift/BUCK:22-23`.
    - `:total-accounting` → `:oya-cloud-ci-total-accounting`, `:cloud-ci-firewall` → `:oya-cloud-ci-firewall`,
      `:registry-drift` → `:oya-cloud-ci-registry-drift`, etc. (the `:self` dep) AND
      `//…/accounting-registry-producer:accounting-registry-producer` → `…:oya-cloud-ci-accounting-registry-producer`
      (the producer dep, in all 6 corpus gates).

### 2.4 Integration-test runtime invocations (HARD — runtime, not compile) — 5 files
The 5 corpus/firewall integration tests shell out to the producer via `Command::new(env!("CARGO")).arg("run").arg("-p").arg("accounting-registry-producer")`.
`-p` takes the **Cargo package name**, so each MUST become `oya-cloud-ci-accounting-registry-producer` or the
test fails at runtime:
- `cloud/cloud-ci/gates/registry-drift/tests/registry_drift.rs:61`
- `cloud/cloud-ci/gates/cloud-ci-firewall/tests/firewall.rs:52`
- `cloud/cloud-ci/gates/total-accounting/tests/total_accounting.rs:206`
- `cloud/cloud-ci/gates/cross-artifact-agreement/tests/cross_artifact_agreement.rs:180`
- `cloud/cloud-ci/gates/staleness-reaper/tests/staleness_reaper.rs:189`
- `cloud/cloud-ci/gates/automation-ratchet/tests/automation_ratchet.rs:194`

### 2.5 Producer provenance string consts (PROVENANCE — forces regen) — 1 file
- `accounting-registry-producer/src/lib.rs:35` `PRODUCER_TARGET = "//cloud/cloud-ci/gates:accounting-registry-producer"`
  → decide new label. **Two sub-decisions:** (a) the colon-target should match the renamed buck target
  (`oya-cloud-ci-accounting-registry-producer` or `…-bin`); (b) the path `gates:` is currently SHORT (omits the
  `/accounting-registry-producer` package dir) — it is a provenance label string, not resolved by buck, so it can
  be left structurally but SHOULD be made the renamed target for honesty.
- `accounting-registry-producer/src/lib.rs:446` `FIREWALL_TARGET = "//cloud/cloud-ci/gates:cloud-ci-firewall"`
  → `…:oya-cloud-ci-firewall`.
- **These two strings are emitted into `_provenance.producer_target` of every generated face: 17,627 occurrences
  in `accounting-registry.generated.json` + 1 in decision-crosswalk + 1 in enforcement-inventory + 2 in
  gate-baseline.** They are byte-diffed by `registry-drift`. ⇒ The faces MUST be REGENERATED by re-running the
  producer (§4), never hand-edited.

### 2.6 If Layer B is NOT changed (recommended): NO edits to
- `GATE_ID` consts (src/lib.rs of each gate), `GATE_IDS` array (producer lib.rs:449-453), fixtures under
  `specs/fixtures/**`, `gate-disposition.json`, ADR-0515, `specs/phase0-automation-matrix.json`,
  `specs/hyperscaler-production-readiness-claim-contract.json`, instructions-store.json. These name the RUNTIME
  gate id (`cloud-ci-*`), which is out of scope. **Confirm with founder that Layer B stays.**

### 2.7 Doc/comment + soft references (LABEL-only; update for honesty, not correctness)
- `docs/decisions/ADR-0515-…md` — the §"crates carried" list (lines 36–42) names the crates by their
  build-identity in prose; update the 7 list items to the new package names. (The `cloud-ci-*` GATE references
  elsewhere in 0515 are Layer B — leave.)
- `ADR-INVENTORY.tsv:290` (the ADR-0515 row's crate list) — same label refresh.
- `goal.json:34` `prompt_markdown` contains `cargo test -p cloud-ci-firewall --workspace` — this is durable-goal
  PROSE that drives future sessions; it is a `-p` package ref and SHOULD be updated to
  `oya-cloud-ci-firewall` so future runs invoke the right package. (Not a build script, so not a hard failure, but
  stale if left.)
- Internal doc-comments referencing sibling crates by old name (e.g. producer lib.rs doc-comments, Cargo.toml
  header comments) — cosmetic; refresh opportunistically.

### 2.8 Generated faces (DO NOT hand-edit; regenerate)
- `accounting-registry.generated.json`, `gate-baseline.generated.json`, `decision-crosswalk.generated.json`,
  `enforcement-inventory.generated.json`, `ttl-policy.generated.json` — outputs only; §4 regenerates them.

---

## 3. Rust crate (import) identity — separate decision

The lib `crate` name is the snake_case form (`total_accounting`, `cloud_ci_firewall`,
`accounting_registry_producer`) in both `Cargo.toml [lib].name` and BUCK `crate =`/`crate_root` unchanged path.
**Recommendation: leave the lib crate (`crate = "total_accounting"`, `use total_accounting::…`) UNCHANGED.**
Cargo allows a package named `oya-cloud-ci-total-accounting` to expose a lib crate named `total_accounting` via
`[lib] name`. Keeping the import path stable means **zero `use` / `extern crate` churn** in the producer and tests
(the producer does `use total_accounting::…` etc.). This is the minimal-blast choice and is ADR-0056-legal (the
BNF governs the *package/member* name; the lib crate name is an import detail). If the founder wants import
identity to also carry the prefix, that is an OPTIONAL extra step adding ~8 `use`-statement edits — flag, don't
assume.

---

## 4. Producer-regen + gate-smoke verification (the GREEN proof)

Run from `/Users/jasonlee/Developer/source` (source repo, founder-go required before any of this):

1. **Edit Layer-A names** per §2.2–§2.5 (and §2.7 label refresh). Do NOT touch generated faces or Layer B.
2. **Regenerate all 5 faces** (mandatory — provenance strings changed):
   `cargo run -p oya-cloud-ci-accounting-registry-producer -- --repo-root . --out-dir cloud/cloud-ci/gates/accounting-registry-producer`
   (the binary writes registry + 4 companion faces). The `gate-baseline.generated.json` regenerates via
   `--face baseline` path used by the firewall test; the producer's default run emits the registry + companions —
   verify all 5 mtimes advance.
3. **Build + lint:**
   - `cargo build --workspace` (or buck2 build of the 7 targets)
   - `cargo clippy --workspace --all-targets -- -D warnings`
   - `cargo fmt --all -- --check`
4. **Gate-smoke (the RED/GREEN proofs must stay GREEN):**
   - `cargo test -p oya-cloud-ci-total-accounting` (corpus gate + unit)
   - `cargo test -p oya-cloud-ci-cross-artifact-agreement`
   - `cargo test -p oya-cloud-ci-staleness-reaper`
   - `cargo test -p oya-cloud-ci-automation-ratchet`
   - `cargo test -p oya-cloud-ci-registry-drift` ← **the canary**: byte-diffs committed vs regenerated faces; if
     §2 missed a provenance string or the regen was skipped, this goes RED. Must be GREEN.
   - `cargo test -p oya-cloud-ci-firewall` ← regenerates baseline over live tree + runs compare/ratchet
     predicates; must be GREEN on the frozen corpus.
5. **Buck2 parity (if buck is the CI driver):** `buck2 build //cloud/cloud-ci/gates/...` and
   `buck2 test //cloud/cloud-ci/gates/...:` resolve under the new target names; confirm the cross-package deps
   (renamed colon-targets) resolve.
6. **Idempotency:** re-run step 2; `git diff` on the 5 faces must be EMPTY (committed == regenerated).
7. **Grep sweep (no stragglers):** `grep -rn -- "-p accounting-registry-producer\|-p cloud-ci-firewall\|-p total-accounting" .`
   returns nothing under `cloud/cloud-ci/gates/**` + `goal.json`; `grep -rn 'path = "../\(total-accounting\|...\)"'`
   confirms dep KEYS renamed.

---

## 5. Risk assessment (these are wired into the LIVE firewall gates)

| ID | Risk | Severity | Mitigation |
|----|------|----------|------------|
| **R1** | **registry-drift trip from missed provenance string.** The producer `_provenance.producer_target` appears 17,627× in the registry face + others; if the consts are renamed but the faces are not regenerated (or a string is missed), `registry-drift-gate` goes RED and the firewall blocks. | **HIGH** | §4 step 2 is mandatory; §4 step 4 canary + step 6 idempotency catch it. Treat regen as part of the same atomic change. |
| **R2** | **Runtime test break from `cargo run -p` mismatch (§2.4).** These are NOT caught by the compiler — they fail only when the test runs the renamed binary. Easy to miss in a name-only sed. | **HIGH** | §2.4 lists all 6 call sites explicitly; §4 step 4 runs every corpus gate, which all exercise the producer subprocess. |
| **R3** | **Buck cross-package dep label drift (§2.3).** The colon-target after the dir path must match the renamed buck `name`; a stale `:accounting-registry-producer` breaks `buck2 build` of 6 gates + the producer self-dep. | **MED** | §2.3 enumerates the 12+ producer deps + 6 corpus-gate deps; §4 step 5 buck parity check. |
| **R4** | **Cargo.lock / serialize_global_artifacts contention.** Lock regen + Cargo.toml are global-serialized per goal.json; a concurrent slice mid-flight on the lock collides. | **MED** | Do the rename as a single isolated change with no other Cargo.toml/lock slice in flight (goal.json rule #8). Lock regenerates automatically; commit it. |
| **R5** | **Scope creep to directory rename.** If dirs are renamed too, ~17.6k `path` rows in the registry face + every BUCK package label + workspace member path churn. | **MED (if attempted)** | EXPLICITLY OUT OF SCOPE here. Dirs stay; only target/package NAMES move. Surface dir-rename as a separate decision if founder wants it. |
| **R6** | **Layer-A/Layer-B conflation.** Renaming the runtime `cloud-ci-*` GATE_IDs would churn every fixture + 3.5 MB baseline and re-open an undecided question. | **MED** | §0 + §2.6 fence Layer B off. Founder confirms Layer B stays before execution. |
| **R7** | **`oya-cloud-ci-firewall` dedup ambiguity (§1 note #6).** Two readings: `oya-cloud-ci-firewall` (deduped) vs `oya-cloud-ci-cloud-ci-firewall`. | **LOW** | Plan uses the deduped form; one-line founder confirm. |
| **R8** | **Doc/inventory staleness if §2.7 skipped.** ADR-0515 + ADR-INVENTORY.tsv + goal.json would name dead crate ids. | **LOW** | Label-only; included in §2.7; cross-artifact-agreement gate does NOT key on these package names (it keys on Layer-B gate ids), so it won't block — but honesty/SSOT requires the refresh. |

**Net:** Mechanically low-complexity (name substitution + one regen) but operationally HIGH-stakes because the
4 born-blocking gates + the firewall ratchet + registry-drift are the live merge gate. The two failure modes that
bite are R1 (skip the regen) and R2 (miss a `cargo run -p`). Both are fully covered by the §4 verification, which
must run GREEN end-to-end before commit. Do it as ONE atomic, founder-go'd, source-mutation change on
`cleanup/whole-tree-2026-06-07`, committing WIP first per the consolidation rule.

---

## 6. Evidence index (real paths, source repo)

- Gate crates: `cloud/cloud-ci/gates/{total-accounting,cross-artifact-agreement,staleness-reaper,automation-ratchet,registry-drift,cloud-ci-firewall,accounting-registry-producer}/{Cargo.toml,BUCK,src/lib.rs,tests/*.rs}`
- Workspace members: `Cargo.toml:235-241`
- Producer provenance consts: `accounting-registry-producer/src/lib.rs:35` (PRODUCER_TARGET), `:446` (FIREWALL_TARGET)
- Producer path-deps: `accounting-registry-producer/Cargo.toml:31-34`
- `cargo run -p` runtime call sites: registry_drift.rs:61, firewall.rs:52, total_accounting.rs:206, cross_artifact_agreement.rs:180, staleness_reaper.rs:189, automation_ratchet.rs:194
- Generated faces (regen targets, registry-drift-protected): `accounting-registry-producer/*.generated.json` (5 files; producer_target string 17,627× in the registry face)
- F-0025: `docs/audit/initial-sweep-2026-06-06/FINDINGS-LEDGER.md:48`
- Naming authority: ADR-0056 (`docs/decisions/ADR-0700-ci-admission-live-apex.md:11,79-101`)
- Collision check: 0 existing `oya-cloud-ci-*` members tree-wide; 719 of ~726 members already carry `oya-` (the 7 gates are the only exceptions)
