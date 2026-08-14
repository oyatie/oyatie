# Kernel unblock design — `kernel-move-plan.BLOCKED.json` resolution card

**Class:** mixed (refactor → move) · **Capability span:** single (`kernel` meta dir, kuberos framekernel)
**Authority re-queried:** ADR-0701 (live apex), ADR-0562 §8 Fork 2, ADR-0512 kernel carve-out, ADR-0611 (Superseded by ADR-0704; kernel/ asterinas workspace landed on dev), `specs/integ-branch-envelopes.json#W-cloud-leaf-cloud-kernel` judgment
**Status:** planning-only / blocked — **NON-DISPATCHING**, explicitly subordinate to `/specs/masterplan.json#masterplan_v2`. The live `planning_entry_contract` is `state: open` with `binding_plan_approval_allowed: false` and `dispatch_allowed: false`, so no execution-wave dispatch may originate from this card until the §Pre-dispatch gates clear. NOT dispatcher-ready.
**Suggested owner profile:** reorg worker with codemod (`tools/oya-reorg-codemod-app`) + nested-workspace expertise; platform-governance reviewer.

## Source context

`specs/reorg/kernel-move-plan.BLOCKED.json` encodes the settled destination shape for the rung-0
kuberos framekernel (de-brand MOVE-11): `hal-kernel` → `kernel/ports/hal`, arch adapters →
`kernel/adapters/{arch-aarch64,arch-x86-64}`, `frame-kernel`/`ksync-kernel`/`user-layout-kernel`/
`app` → `kernel/core/*`, plus 13 nested test/user-program crates riding with their parents.
It is fail-closed-blocked on mechanically proven defects (B1–B3). `cloud/cloud-kernel` holds
**20** crate dirs (7 top-level + 13 riding) of the 24-crate `cloud/` inventory (the retired
`legacy_root_freeze` — dropped with the module-membership gate by ADR-0718 — listed exactly
20 `cloud/cloud-kernel/*` rows + 4 `cloud/cloud-os/*` rows). The apparent twenty-first
`Cargo.toml` is the workspace root, not a crate. `cloud/` is therefore **not** zeroed
by this card: the four `cloud/cloud-os/*` crates remain under `cloud/` until their own `os/` lane.

## Blocker B1 — destination workspace shape conflict

**Class:** design/decision (not a code edit).

**Fresh dry-run evidence (2026-08-14, dev tip):** `oya-reorg-codemod dry-run --plan
specs/reorg/kernel-move-plan.BLOCKED.json` now fails with `move target already exists:
"kernel/Cargo.toml"` — the plan's artifact move of `cloud/cloud-kernel/Cargo.toml` collides
with the ADR-0611 workspace root that landed at `kernel/` (edition 2024, rust-version 1.97.1).
B2's workspace-dependency refusal no longer fires. This confirms the destination-occupation
blocker and motivates the re-anchor below.

`kernel/` is already a nested Cargo workspace from ADR-0611 (asterinas boundary + real-boot):
edition 2024, rust-version 1.97.1, stable, std host crates. The kuberos framekernel needs edition
2021, rust-version 1.85, `nightly-2026-02-28` + `build-std` for `aarch64-unknown-none-softfloat`.
One tree cannot carry two values of `rust-toolchain.toml`, `.cargo/config.toml`, or
`[workspace.package]` edition.

**Design decision (proposed):** give each rung its own nested workspace under the `kernel/` meta
dir — the ADR-0512 carve-out already sanctions nested workspaces for the kernel rung:

```text
kernel/
  kuberos/            # rung 0: the no_std framekernel workspace (Cargo.toml + lock + toolchain + .cargo/config)
    ports/hal/
    adapters/arch-aarch64/  adapters/arch-x86-64/
    core/{frame,ksync,user-layout,app}/
  asterinas/           # ADR-0611 workspace (renamed from the current flat kernel/core + kernel/harness)
```

- `kernel/kuberos/` becomes the move-plan's new destination prefix (all 7 moves + 13 riding crates
  re-anchor from `kernel/` → `kernel/kuberos/`).
- `kernel/` itself stays the meta-dir root (OWNERS + the two nested workspaces), matching the
  registry charter (`kernel/` owns_crates=true, rung 0).
- The ADR-0611 asterinas crates move `kernel/{core,harness}` → `kernel/asterinas/{core,harness}`
  in the SAME PR (a small intra-kernel move) so both workspaces have clean roots.

**Authority-change prerequisite (envelope + apex conflict):** the accepted
`specs/integ-branch-envelopes.json#W-cloud-leaf-cloud-kernel` judgment (`judgment_status: done`,
`land_status: ready_for_integ_kernel`) requires one rung-0 META workspace with the framekernel
absorbed into `kernel/{core,harness}` and names that as the forever shape, including the
`finops_unit_cost` challenge "one nested kernel workspace forever avoids duplicate CI graphs".
The live apex `docs/decisions/ADR-0704-k8s-port-live-apex.md` (which SUPERSEDES ADR-0611)
similarly normatively places the Asterinas deliverables at `kernel/core/...` via its folded
ADR-611 residual (lines 59, 74-76) and requires refinements to land as apex amendments (line 68).
This card's two-workspace topology (kuberos + asterinas) therefore **amends both judgments**; the
toolchain/edition conflict above is the mechanical evidence for the amendment. The amendments
(OVERRULE of `W-cloud-leaf-cloud-kernel` to the two-nested-workspace shape, plus the ADR-0704
apex amendment re-homing the ADR-611 residual paths to `kernel/asterinas/...`, each with the
five-field doctrine record) ship in ONE governance PR — sequencing step 0 — a true prerequisite
that must land before any blocker-resolution PR dispatches; they are NOT bundled into the B1 split
PR, and codemod text substitution is NOT an ADR amendment. This card neither silently overrules
the envelope/apex nor dispatches against them — see §Pre-dispatch gates.

**B1a — cross-workspace transfer mechanism (whole-workspace transfer).** Every re-anchored crate
move goes from the `cloud/cloud-kernel` workspace to the distinct `kernel/kuberos` workspace.
`validate_workspace_ownership` (`tools/oya-reorg-codemod-app/src/plan.rs:544-560`) rejects exactly
this case with `WorkspaceSpan` before any writes. Resolution: the codemod gains a tested
**whole-workspace transfer** mode — when the plan co-moves the old workspace-root artifacts
alongside the crates, `validate_workspace_ownership` admits the move because the destination
workspace IS the relocated source workspace (the workspace root is a co-moving artifact in the
same plan).

**Workspace-identity artifact set vs meta-root ownership.** The admission predicate keys on the
source workspace's ACTUAL workspace-identity artifacts, defined SEPARATELY from meta-root
ownership files: workspace identity = `Cargo.toml` + `Cargo.lock`, plus a workspace-local
`rust-toolchain.toml` and/or `.cargo/config.toml` when present. Meta-root ownership files
(`OWNERS` at the `kernel/` meta-root) are NOT workspace-identity artifacts: they stay at the
meta-root (§Design decision keeps `kernel/` as the meta-dir root with `OWNERS`), so a literal
"every tracked root file must co-move" predicate would either move `OWNERS` and violate the final
shape, or reject the asterinas transfer for "incomplete" inventory. The mode therefore admits a
move whose plan co-moves the source workspace's workspace-identity artifact set (and leaves
meta-root `OWNERS` in place) — a per-workspace root-relocation case, unit-tested for both the
kuberos root (`Cargo.toml` + `Cargo.lock` + `rust-toolchain.toml` + `.cargo/config.toml`) and the
asterinas root (`Cargo.toml` + `Cargo.lock` only), with the `OWNERS`-stays preservation case
explicitly covered. All other cross-workspace moves stay fail-closed (`WorkspaceSpan` unchanged).
Unit + integration tests prove both the admit and the unchanged refusal.

**Destination-root rebase is part of the transfer mode.** The existing
`rewrite_owning_workspaces` (`tools/oya-reorg-codemod-app/src/plan.rs:323-348`) groups moves by
SOURCE workspace and re-expresses `members`/`exclude` against that OLD root — correct for
in-place moves, wrong for a relocated workspace root: for step 1 a target like
`kernel/asterinas/core/...` would be written into `kernel/Cargo.toml` as `asterinas/core/...`,
and once that manifest sits at `kernel/asterinas/Cargo.toml`, Cargo would look under
`kernel/asterinas/asterinas/core/...`. The whole-workspace transfer mode must therefore rewrite
all manifest-relative paths (`members`, `exclude`, `[workspace.dependencies]` path deps) against
the DESTINATION workspace root in the same run — a tested part of the mode for both the kuberos
and asterinas transfers, so `cargo metadata` resolves post-move.

**B1b — no pre-created move-plan artifact targets; the workspace root transfers ATOMICALLY with
its crates.** The split PR does NOT pre-create the `kernel/kuberos/` workspace root. A root
manifest moved ahead of its members cannot be gate-green: `cloud/cloud-kernel/Cargo.toml` names
its members through relative `crates/...` paths, so a manifest sitting at `kernel/kuberos/`
while the seven crates still live under `cloud/cloud-kernel/crates` fails `cargo metadata`
(members unresolvable). A transitional manifest (empty/stub members) is REJECTED: it would need a
second ownership rule and an intermediate state that the codemod's ownership validation cannot
represent. Instead, the move PR performs ONE atomic `apply_plan` run that transfers the
workspace-root artifacts (`Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.cargo/config.toml`)
AND the 7 crates + 13 riding crates together, under the B1a whole-workspace-transfer mode. The
transfer mode rewrites the moved manifest's `members`/`exclude` and `[workspace.dependencies]`
path deps against the DESTINATION root in the same run (B1a destination-root rebase — the stock
`rewrite_owning_workspaces` rebases against the source root and would emit
`asterinas/core/...`-style doubled prefixes), so `cargo metadata` resolves at `kernel/kuberos/`
post-move. Because nothing is pre-created, `apply_plan`'s `TargetExists` preflight
(`src/plan.rs:103-119`) never fires. Do NOT pre-create any artifact that a later move-plan will
transfer.

**B1c — rewrite runtime paths during the asterinas re-anchor.** The real-boot binaries retain
default receipt paths (`kernel/harness/asterinas-real-boot/receipts/...` in
`kernel/harness/asterinas-real-boot/src/main.rs:23-24` and `src/boot.rs:48-50`, plus the
`kernel/target/artifacts/*` ISO/log destinations). The **soak binary** hard-codes the same
meta-root tree in `src/soak.rs:39-40` (`DEFAULT_ISO` =
`kernel/target/artifacts/asterinas-nixos-0.17.2-x86_64.iso` and `DEFAULT_RUN_DIR_BASE` =
`kernel/target/asterinas-soak/runs`). The reorg codemod rewrites crate identifiers and BUCK
labels but NOT arbitrary Rust string paths, so the re-anchor acceptance must include rewriting
ALL of these runtime paths — receipts, ISO/log, and soak run dir — to
`kernel/asterinas/harness/asterinas-real-boot/...` / `kernel/asterinas/target/...`, defining an
explicit re-anchored destination for the `kernel/target` paths, and adding tests (including the
integration-test literals that assert those default paths) that the binaries write into the
re-anchored tree — `cargo metadata` alone cannot catch this.

**B1d — rewrite linker-script paths before accepting the moved workspace (P1).** The
content-preserved `cloud/cloud-kernel/.cargo/config.toml` passes
`-Tcrates/oya-cloud-kernel-arch-{aarch64,x86-64}-adapter/linker.ld`, but after the move those
files live at `kernel/kuberos/adapters/arch-{aarch64,x86-64}/linker.ld`. `cargo metadata` does not
invoke the linker, so the acceptance criterion must require (1) the `rustflags` paths in the
transferred `.cargo/config.toml` rewritten to the new adapter face layout, and (2) a **real build**
of both configured targets (`aarch64-unknown-none-softfloat` + `x86_64-unknown-none`) — not just
`cargo metadata` — as move-PR verification.

**B1e — move and retarget the kernel workspace gitignore.** `cloud/cloud-kernel/.gitignore` is the
only tracked top-level workspace file absent from the current artifact list; it would remain behind
and keep `cloud/cloud-kernel` alive. The transfer must include `.gitignore` and rewrite its
workspace-relative patterns (`/target`, `/crates/.../tests-host/target`, `/out/*.elf`,
`/out/*.bin`, `/out/*.log`, `/out/.stage-b-target/`) for the new `kernel/kuberos/` layout
(`/adapters/...`, `/out/*.elf`, ...) so generated ELF/bin/log outputs stay ignored.

**B1e-2 — retarget the repository-level generated-output protections (move PRs).** The workspace
`.gitignore` is not the only ignore surface: the REPO-ROOT `.gitignore` also pins old paths —
`cloud/cloud-kernel/out/conformance/` and `cloud/cloud-kernel/out/stress-summary.txt`
(`.gitignore:182-183`) and `kernel/harness/*/receipts/` (`.gitignore:195`). After steps 1 and 3
relocate the harness and `out/`, those patterns no longer match the new
`kernel/kuberos/out/...` and `kernel/asterinas/harness/...` outputs, so regenerated run
artifacts become commit-visible. The scratch classifier's exact stress path is likewise still
frozen at the old location in BOTH unit-class policy copies
(`ci/facade/artifact-inventory-registry/src/unit-class-policy.json:16-17` and
`libs/oya-ci-config/src/bundled/unit-class-policy.json:16-17` — `scratch-kernel-conformance`
contains `cloud-kernel/out/conformance/`, `scratch-kernel-stress` exact
`cloud/cloud-kernel/out/stress-summary.txt`), so the zero-tolerance scratch gate would no longer
recognize the relocated stress report. Each move PR must retarget these repo-level ignore and
classification rules to the new `kernel/kuberos/out/...` / `kernel/asterinas/harness/...` paths —
in the same PR that relocates the outputs.

**B1f — reconcile the lockfile corpus across the split and move PRs.** The supply-chain audit gate
(`ci/facade/supply-chain-audit/supply-chain-audit-policy.json#lockfile_corpus`) authoritatively
pins the current corpus (`Cargo.toml`, `cloud/cloud-kernel/Cargo.toml`,
`cloud/cloud-kernel/crates/.../tests-host/Cargo.toml`, `kernel/Cargo.toml`), and its live test
(`ci/facade/supply-chain-audit/tests/supply_chain_audit.rs:163-186`) asserts that exact corpus.
The split PR re-anchors the asterinas pair (`kernel/Cargo.toml` → `kernel/asterinas/Cargo.toml`),
so it must update `lockfile_corpus` (+ `min_lockfiles`) AND the committed test expectation in the
same PR. The move PR then performs the atomic kuberos transfer: it adds the
`kernel/kuberos/Cargo.toml` pair and re-points the `cloud/cloud-kernel` pair and the `tests-host`
pair to their `kernel/kuberos` destinations — again with the policy + test transition in the same
PR, never silent.

**B1g — retarget the Asterinas matrix and pin paths during the re-anchor.** The re-anchor moves
`kernel/{core,harness}` → `kernel/asterinas/{core,harness}`, but the following hand-maintained
projections still point at the old tree and are NOT rewritten by the codemod (which rewrites
Cargo/BUCK/Rust identifiers and `docs/decisions` anchors only, not arbitrary YAML/JSON fields).
The former `registry/catalog/` bookkeeping — including the two asterinas catalog rows whose
`traceability.source_crate` pointed at `kernel/{core,harness}` — was retired outright by ADR-0718,
so there is no catalog left to retarget; only the in-tree matrix and pin artifacts below remain.
- `kernel/core/asterinas-abi-matrix/matrix/abi-matrix-v0.1.0.json` (`$id`,
  `asterinas_pin.pin_manifest` = `kernel/core/asterinas-boundary/pins/...`, and
  `probe_harness.path` = `kernel/harness/asterinas-abi-probe`);
- `kernel/core/asterinas-boundary/pins/asterinas-release-v0.17.2.json` itself: its embedded
  canonical `$id` = `https://docs.oyatie.com/kernel/core/asterinas-boundary/pins/asterinas-release-v0.17.2.json`
  (digest-embedded via `include_str!`, so the pin and the manifest cannot drift).

The split PR must retarget BOTH of the above — including the pin artifact's own `$id`, not just
the matrix pointer to it — to the `kernel/asterinas/...` destinations, and add a stale
`kernel/{core,harness}` path check to the B1 acceptance criteria covering the matrix AND pin
artifacts (no governed matrix/pin artifact may still reference the vacated tree).

**B1h — retarget owner-required coverage for the kuberos linker scripts (move PR).** After the
move, `ci/facade/affected-target-set/affected-set-policy.json:21-24` still requires Buck ownership
only for the two old linker-script paths
(`cloud/cloud-kernel/crates/oya-cloud-kernel-arch-{aarch64,x86-64}-adapter/linker.ld`). Both
adapter `BUCK` files intentionally define no target, so a later edit to
`kernel/kuberos/adapters/arch-{aarch64,x86-64}/linker.ld` would miss `require_owner_patterns` and
escalate to a full `//...` run that still skips those empty packages — losing the current
`RefuseUnowned` protection. The move PR must therefore replace the two
`require_owner_patterns` entries with the new
`kernel/kuberos/adapters/arch-{aarch64,x86-64}/linker.ld`
paths and update the gate's committed assertions, so future linker changes cannot become
false-green.

**B1i — preserve the kuberos manifest at its declared forever path (move PR).** The re-anchored
plan would move `cloud/cloud-kernel/manifest.json` to `kernel/kuberos/manifest.json`, but the
service manifest's declared forever path is `kernel/manifest.json`: the SLO-coverage gate
(`ci/facade/slo-coverage/tests/slo_coverage.rs:172-237`) requires the forever path and accepts
`cloud/cloud-kernel/manifest.json` only as a transitional fallback, so after the source is removed
neither accepted path would exist and `cloud_manifest_paths` fails before validating any SLOs;
`ci/facade/affected-target-set/affected-set-policy.json` also keys the manifest's synthetic
dependencies specifically on `kernel/manifest.json`. Retargeting the manifest internals and the
tier-classification key is therefore insufficient. Either keep the service manifest at the
meta-root forever path (`kernel/manifest.json`) in the move PR — amending the plan artifact
destination from `kernel/kuberos/manifest.json` to `kernel/manifest.json` — or explicitly amend
both consumers AND the authority that declared that path forever. The manifest-internals rewrite
+ tier-classification reprojection (step 4) still applies, just at the forever path.

**Acceptance criteria:** one nested workspace per rung; `cargo metadata` resolves in both (the
kuberos root only after its atomic transfer — nothing resolves at a pre-created stub root); no
cross-workspace path-deps between kuberos and asterinas (they are disjoint ladders); the B1a
whole-workspace transfer is test-proven — admission keys on the source workspace's
workspace-identity artifact set (asterinas: `Cargo.toml` + `Cargo.lock`; kuberos: + toolchain +
config) with meta-root `OWNERS` explicitly preserved, and manifest-relative paths are rebased
against the DESTINATION root; the B1b atomic root+crates transfer leaves no pre-created target and
no transitional manifest; the B1c runtime paths (receipts, ISO/log, AND soak run dir) are
rewritten and tested; the B1d linker paths are rewritten and both targets build for real; the B1e
workspace AND repo-root `.gitignore`/scratch-classifier rules retarget to the new outputs in the
same PRs; the B1f lockfile corpus and test transition in their owning PRs; the B1g matrix AND pin
projections retarget to `kernel/asterinas/...` with a stale `kernel/{core,harness}` check; the B1h
`require_owner_patterns` retargets to `kernel/kuberos/adapters/arch-{aarch64,x86-64}/linker.ld`
with committed assertions; the B1i manifest lands at its declared forever path
(`kernel/manifest.json`) or its consumers + authority are amended; the `oya-ci-required.yml`
`Kernel workspace tests` step retargets with step 1 and gains Kuberos coverage with step 3; the
spent move-plan is deleted/parked in the move PR closeout; the host-harness verification is
portable (host-target override + explicit harness test command in step 2); the envelope AND
ADR-0704 apex authority change (§step 0) lands.

## Blocker B2 — codemod does not rewrite `[workspace.dependencies]`

**Class:** RESOLVED-ON-DEV (2026-08-14 re-verify).

`rewrite_dep_tables_recursive` now recurses into `[workspace.dependencies]` and its
dev/build siblings (the code comment cites the ADR-0512 carve-out's 5-of-7 edges explicitly).
Landed via PR #1523 (`fa1292c89 fix(reorg-codemod): make the oracle see the workspace it is
migrating`). Re-verified at `origin/dev@4a4f71a14` by reading the source and dry-running the
blocked plan: the workspace-dependency refusal no longer fires.

**Remaining B2 action:** none (the codemod change is on trunk). B2 is NOT a blocker and no
sequencing step dispatches a duplicate enhancement for it.

## Blocker B3 — escaping include!/include_bytes!/include_str! literals

**Class:** SPLIT into B3a + B3b (2026-08-14 re-verify). The measured escaping set is **27**
literal sites: **21** workspace-root ELF embeds (`include_bytes!` in the two adapter
`src/user.rs` files — 9 aarch64 + 12 x86-64, matching the 21 frozen
`skip_build_output_path` keys in
`ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-baseline.json`) plus **6**
`tests-host` `include!` literals naming old sibling crate dirs.

**B3a — the 21 workspace-root ELF embeds: sanctioned codemod behavior DESIGNED; implementation
in flight (PR #1965 open, not merged).** The design specifies two move-invariant classes the
refusal gate must accept: depth-preserving moves with untouched targets, and targets that co-move
with the workspace (the `../../../out/*.elf` shape — the kuberos workspace relocates as a unit and
`out/` rides the move as artifact co-moves). Fixture + unit tests prove both; fail-closed
otherwise. The enhancement itself is PR #1965 (`feat(reorg-codemod): accept move-invariant
escaping literals (kernel unblock B3a)`) — **OPEN at 2026-08-14, not on dev** — so B3a clears
only when #1965 lands on dev AND the re-anchored plan dry-runs with zero
`UnrewritablePathLiteral` refusals. Until then the refusal gate on trunk still fires; the card
must not be treated as
dispatcher-ready on B3a alone. **Move-PR companion (baseline relabel):** the move changes all 21
path-qualified keys in `embedded-asset-hermeticity-baseline.json` (`skip_build_output_path`). The
live gate asserts exact set equality (`tests/embedded_asset_hermeticity.rs:110-119`) and the
codemod does not rewrite arbitrary JSON, so the move PR must include a **review-visible path
relabel** of that baseline to the new `kernel/kuberos/` adapter paths WITHOUT raising its ceilings
(`skip_build_output_path` ceiling stays 21).

**B3b — the 6 tests-host `include!` literals naming old sibling crate dirs: REMAINS, kernel-lane
scoped.** They name the OLD directory outright, so no move preserves them; the pre-move fix must
re-home the shared pure layout/signal/timekeep modules so the host harness (its own
`[workspace]` root escaping the kernel's build-std config) can consume them WITHOUT forking the
single source of truth — the host harness must depend on / remain colocated with the production
`user-layout-kernel` module sources, never copy them into in-crate fixtures (a fixture copy would
let production layout/signal/VFS/timekeep changes break the kernel while these host tests stay
green). This is a kernel-toolchain-harness refactor that requires the nightly `build-std`
toolchain to verify (host harness + no_std crates must both still compile). Do NOT attempt blind.
Sequencing: this slice runs in the kernel lane with toolchain verification, and the kuberos move
waits on it. Not on the kuberos-move critical path (libs/oya is the larger migration; the
20-row baseline burn is retired with ADR-0718).

## Sequencing

0. **Envelope + apex authority change (own governance PR — true prerequisite).** Land BOTH: (a)
   the `integ-branch-envelopes.json#W-cloud-leaf-cloud-kernel` amendment (OVERRULE to the
   two-nested-workspace shape, five-field doctrine record), and (b) an
   `docs/decisions/ADR-0704-k8s-port-live-apex.md` apex amendment. ADR-0704 is the live apex that
   SUPERSEDES ADR-0611: its folded ADR-611 residual (lines 59, 74-76) normatively places the
   Asterinas deliverables at `kernel/core/...` under the `kernel/` workspace, and line 68 requires
   further body refinements to land as apex amendments — so step 1 would contradict live authority
   even after the envelope OVERRULE alone, and the codemod's automatic text substitution is NOT an
   ADR amendment. The apex amendment re-homes the ADR-611 residual's Asterinas paths to
   `kernel/asterinas/...` (and any `kernel/target` runtime-destination refinements it names). This
   step dispatches FIRST, before any blocker-resolution PR, so later steps are not dispatching
   against the unamended envelope or the unamended apex.
1. **B1 asterinas re-anchor + codemod whole-workspace-transfer mode (own PR, gate-green).**
   Asterinas re-anchor (`kernel/{core,harness}` → `kernel/asterinas/{core,harness}`), landing the
   B1a codemod whole-workspace-transfer mode (tested: workspace-identity artifacts co-move while
   meta-root `OWNERS` stays; manifest-relative paths rebase against the DESTINATION root;
   `WorkspaceSpan` still fail-closed otherwise). The asterinas move itself is a
   whole-workspace transfer — `kernel/Cargo.toml` (the ADR-0611 root) co-moves to
   `kernel/asterinas/Cargo.toml` with its `members` re-expressed, exactly the atomic pattern B1a
   admits. The PR also carries B1c runtime-path rewrite + tests (incl. soak), B1g
   matrix/pin retarget + stale `kernel/{core,harness}` check, the B1f split-side
   supply-chain corpus + test transition (`kernel` → `kernel/asterinas`), the B1e-2 repo-root
   ignore/classifier retarget for the relocated harness (`kernel/harness/*/receipts/` → new
   `kernel/asterinas/harness/...` outputs), AND the required-workflow retarget: the `Kernel
   workspace tests` step in `.github/workflows/oya-ci-required.yml:127-129` runs
   `cargo test --locked --manifest-path kernel/Cargo.toml`, which fails once that manifest moves —
   retarget it to `kernel/asterinas/Cargo.toml` in this same PR so the prerequisite PR can receive
   its required status. NO `kernel/kuberos/` root is created here (B1b: nothing to pre-create, no
   transitional manifest). (The former B2 codemod step is already on trunk via #1523.)
2. **B3 pre-move include refactor (own PR, behavior-preserving, gate-green).** B3b re-home so the
   host harness consumes the production module sources (no fixture copies); verify on the nightly
   `build-std` toolchain. **Portable host-harness verification:** the excluded `tests-host` crate
   is not exercised by either workspace test, and its own `.cargo/config.toml` forces
   `aarch64-apple-darwin` — so the generic "verify on the toolchain" instruction cannot supply
   gate evidence on the Linux required runner (it either skips the harness or fails for the
   unavailable
   Apple target). This PR must require a portable host-target override or config change (e.g.
   clear `build.target` to the runner host, or run with an explicit `--target <host>` override)
   and wire an explicit harness test command into the PR's verification.
3. **Re-anchor + execute the move-plan (own PR).** Plan file re-committed under `specs/reorg/`
   with the `kernel/kuberos/` destinations; ONE atomic `apply_plan` run transfers the
   workspace-root artifacts (Cargo.toml, Cargo.lock, rust-toolchain.toml, .cargo/config.toml)
   AND the 7 crates + 13 riding crates together under the B1a mode (B1b: destinations are free,
   `TargetExists` never fires, members re-expressed for the DESTINATION root). Includes B1d linker
   config rewrite + real build of both targets, B1e `.gitignore` transfer/retarget, B1e-2 repo-root
   ignore/classifier retarget for the relocated `out/` (`cloud/cloud-kernel/out/...` → new
   `kernel/kuberos/out/...` in both unit-class policy copies + repo `.gitignore`), B1f move-side
   supply-chain corpus + test transition (`cloud/cloud-kernel` + tests-host pairs →
   `kernel/kuberos`), the B1h affected-target-set owner-pattern retarget, the B1i manifest
   forever-path decision (below), the B3a baseline path relabel
   (`embedded-asset-hermeticity-baseline.json`, ceilings unchanged), the manifest/projection
   stage (below), the Kuberos workspace test addition to `oya-ci-required.yml` (mirroring the
   step-1 retarget), AND the **spent-plan retirement**: after the plan applies, delete or PARK the
   re-anchored executable `*-move-plan.json` so `reorg_at_most_one_executable_move_plan`
   (`ci/facade/baseline-ratchet/tests/gate_registration.rs:2376-2410`) does not count two live
   plans when the next rehome lands; singleton applies, then is retired in the same PR's closeout.
4. **Manifest + projection retarget in the same move PR.** `manifest.json` and `specs/` are
   non-crate artifacts that `apply_plan` moves content-preserving without rewriting JSON: the
   manifest keeps `metadata_file`/`destination_path`/ownership references under the deleted
   `cloud/cloud-kernel` tree, and `specs/microservice-tier-classification.json` keys the manifest
   by its old path (`cloud/cloud-kernel/manifest.json`). The
   `tier_classification_projection_is_the_governed_manifest_corpus` gate asserts exact equality
   with the live manifest census and will reject the move. The move PR therefore includes an
   explicit artifact rewrite/reprojection stage (manifest internals + tier-classification key) and
   a **stale-source-path check** (no governed artifact may still reference
   `cloud/cloud-kernel/`). Per B1i the manifest must land at its declared forever path
   `kernel/manifest.json` (or the SLO/affected-set consumers + authority amended) — the
   reprojection keys off whichever path the SLO-coverage gate actually accepts.
5. **`cloud/cloud-kernel` ends at zero crate dirs in the same PR.** The `legacy_root_freeze`
   baseline this step previously burned was retired with the module-membership gate by ADR-0718 —
   no regeneration/burn step remains (nothing to hand-edit). The four `cloud/cloud-os/*` crates
   remain under `cloud/` and are NOT part of this card — they belong to the separate `os/` lane
   (thread-verified: `cloud/` is not zeroed by this card).

## Pre-dispatch gates

This card is planning-only/blocked and NON-DISPATCHING. Dispatch is TIERED so that
blocker-resolution PRs can run before the blockers they resolve are closed (no dependency cycle),
and each tier requires ONLY prerequisites completed by EARLIER steps — never a step's own
deliverables:

**Tier A — governance prerequisites (before step 0 dispatch):**
- `masterplan_v2.planning_entry_contract` transitions from `state: open` to closed
  (`binding_plan_approval_allowed` and `dispatch_allowed` become true);
- the kernel-unblock work item + its dependency edges are registered in
  `/specs/masterplan.json#masterplan_v2.work_items` (+ `dependency_edges`) via the
  coordinator-owned single-writer mutation path — no competing dispatch surface.

**Tier B — authority-change gate (before step 1/2 dispatch):** step 0's own deliverable is a
prerequisite for the blocker-resolution PRs that follow it:
- the `integ-branch-envelopes.json#W-cloud-leaf-cloud-kernel` authority change AND the
  `docs/decisions/ADR-0704-k8s-port-live-apex.md` apex amendment (both in step 0's governance PR)
  are merged — the live apex supersedes ADR-0611, so the re-anchor must not run against it.

**Tier C — move-execution gate (before step 3/4/5 dispatch):** restricted to what steps 0–2
actually complete; the move PR's own deliverables (B1d, B1e, B1f move-side, B1h, B3a baseline
relabel, manifest/projection stage, burn) are NOT gates on their own dispatch:
- steps 1–2 cleared: B1a mode test-proven (incl. destination-root rebase + OWNERS preservation),
  B1c runtime paths rewritten + tested, B1g matrix/pin retarget + stale check landed, B1e-2
  repo-root ignore/classifier retarget landed, B1f split-side corpus transition landed, the
  `oya-ci-required.yml` Kernel-workspace-test retarget landed, and B3b re-homed (no fixture
  copies) with portable host-harness verification;
- B3a's PR #1965 lands on dev and the re-anchored dry-run is zero-refusal;
- the move-plan singleton (`specs/reorg/*-move-plan.json`) is free.

Blockers are resolved by their own PRs (steps 1–2) and are NOT required closed before those PRs
dispatch — full blocker closure is reserved for the final move-plan execution (Tier C).

## Non-goals

- No touching `os/` (separate lane, already dest-verified); the four `cloud/cloud-os/*` crates
  stay under `cloud/` until that lane.
- No behavior change to the framekernel; the move is byte-preserving modulo paths (except the
  explicitly rewritten config/linker/gitignore/runtime-path artifacts above).
- No merge of the two nested workspaces into one (rejected: unresolvable toolchain/edition values).
