# Reorg plan — `kernel/` + `os/` (kuberos de-brand), task #6

Status: PLAN for founder review. No code changes. Read-only inspection of `origin/dev`
(fetched 2026-07-09). Author: general-purpose research agent.

Authorities: ADR-0562 (capability-first repo org + closed capability registry),
ADR-0611 (land Asterinas real-boot foundation under `kernel/`), ADR-0512 (kernel
nested-workspace carve-out), ADR-0280 (substrate DAG), ADR-0538 (glob-only workspace
members), ADR-0563 (rename-aware baseline engine + committed move-plan), ADR-0555
(born-accounting), ADR-0532/0533 (de-brand profile). Playbook:
`.omc/ultragoal/strangler-move-playbook.md`. Registry: `specs/capability-registry.json`.

---

## 0. Executive summary — task #6 is TWO independent halves with very different readiness

Task #6 ("kernel/ + os/ de-brand") is not one move. Inspecting `origin/dev` shows the two
halves are decoupled and asymmetric:

| Half | Source | Target | Readiness |
|------|--------|--------|-----------|
| **os/** | `cloud/cloud-os/` (41 crates) | `os/` (rung 1, root workspace) | **READY** — a clean, mechanical strangler move; largest to date but only 3 external dependents |
| **kernel/** | `cloud/cloud-kernel/` (20 crates) | `kernel/` (rung 0, nested workspace) | **BLOCKED** — needs a founder pivot-ADR to decide cloud-kernel's disposition first; NOT a mechanical registry move |

Key facts driving this split:

- `kernel/` **already exists and is already populated with the *canonical* content** and
  **already de-branded**: ADR-0611 landed `kernel/core/asterinas-boundary` (cargo
  `kernel-asterinas-boundary`) + `kernel/harness/asterinas-real-boot` (cargo
  `kernel-asterinas-real-boot`). Evidence: `git ls-tree -r origin/dev kernel/`.
- The registry (`specs/capability-registry.json` `meta_directory_absorbs`) says
  `kernel/ ← cloud/cloud-kernel`. But ADR-0611 §Context explicitly states: *"The broader
  product decision (Asterinas AS the canonical kuberos kernel, and the multi-shard Wave-1
  roadmap) remains a separate founder-authored pivot ADR; nothing here decides that."* Per
  the Asterinas CANONICAL pivot, the ground-up `cloud-kernel` (frame-kernel/hal-kernel/…) is
  **demoted to reference**. That pivot ADR is **not yet authored**. Until it is, the registry
  literal (`cloud-kernel → kernel/ as canonical`) contradicts the pivot and MUST NOT be
  executed as-is. This is the top open decision (§4.1).
- `os/` does **not** exist yet on `origin/dev` (`git ls-tree origin/dev` — no `os` entry). It
  is a pure target; the move creates it.

Recommended sequencing: **execute the os/ half now** (it is unblocked and foundation-clean);
**gate the kernel/ half** behind the founder pivot-ADR (§4.1).

---

## 1. Current-state map (evidence: `git ls-tree -r origin/dev`)

### 1.1 `kernel/` — EXISTS, canonical Asterinas content, de-branded (ADR-0611)

```
kernel/Cargo.toml                                   # nested/excluded workspace (ADR-0512 carve-out)
kernel/Cargo.lock                                   # own lockfile (tracked)
kernel/OWNERS
kernel/core/asterinas-boundary/                     # cargo: kernel-asterinas-boundary (zero-dep pin crate)
  pins/asterinas-release-v0.17.2.json               #   black-box upstream ISO pin (digest + marker set)
kernel/harness/asterinas-real-boot/                 # cargo: kernel-asterinas-real-boot (dev-only QEMU harness)
  src/{lib,boot,main,soak}.rs                        #   + bins: -fetch-verify / -boot / -soak
  tests/{boot_finalize,soak}_integration.rs
```
Workspace: `kernel/Cargo.toml` members = `["core/asterinas-boundary","harness/asterinas-real-boot"]`,
`resolver = "2"`. Excluded from root workspace (root `Cargo.toml` `exclude` includes `"kernel"`).
Sub-folds are the floor-rung bespoke set: `core/` + `harness/` (NOT the four capability faces).

### 1.2 `os/` — DOES NOT EXIST yet (pure target)

`git ls-tree origin/dev` lists no `os` top-level entry; registered as a meta-dir in
`specs/capability-registry.json` (`meta_directories[].dir == "os/"`, rung 1, `owns_crates: true`)
but empty on disk.

### 1.3 `cloud/cloud-os/` — the Talos-class node OS, 41 crates (SOURCE for os/)

`git ls-tree -r origin/dev cloud/cloud-os/` → `crates/` (41) + `manifest.json` (the only
non-crate artifact). Crate face-suffix census (`sed`/`sort`/`uniq`):

| suffix | count | Talos components (examples) |
|--------|-------|------------------------------|
| `-domain` | 37 | trustd, apid, machined, kubelet, kubernetes, kubespan, siderolink, etcd, cri, network, block, security, upgrade, install, imager, … |
| `-api`    | 1  | `proto-api` (Talos gRPC message/service defs) |
| `-app`    | 2  | `init-app` (real PID 1; bins `init`,`svc`), `difftest-app` |
| `-kernel` | 1  | `kernel` (os-internal kernel crate) |

All crates named `oya-cloud-os-<leaf>` — **double-branded** (`oya-` + `cloud-os-`) and
path-doubled (`cloud/cloud-os`). `cloud-os` is a **root workspace member** (matched by the root
`Cargo.toml` glob `cloud/*/crates/oya-*`; it is NOT in the root `exclude` list — only
`cloud/cloud-kernel` and `kernel` are). 17 `.rs` files carry `#![no_std]` but the crates resolve
fine in the std-targeted root workspace today, so **os/ stays a root-workspace member** (no
nested carve-out needed — that is kernel-only).

### 1.4 `cloud/cloud-kernel/` — the ground-up Asterinas-OSTD reimplementation, 20 crates

`git ls-tree -r origin/dev cloud/cloud-kernel/`: a SEPARATE `no_std` bare-metal workspace (own
`Cargo.toml [workspace]`, own `Cargo.lock`, own `rust-toolchain.toml`; root `Cargo.toml`
`exclude` includes `"cloud/cloud-kernel"`). Crates: `oya-cloud-kernel-{app, frame-kernel,
hal-kernel, ksync-kernel, user-layout-kernel, arch-x86-64-adapter, arch-aarch64-adapter}` (+ the
arch adapters' user-* sub-src trees). This is the ground-up kernel the Asterinas pivot demotes to
**reference**.

### 1.5 `k8s/` — the rung above os/ (mostly ALREADY migrated — the precedent)

`k8s/` already exists in capability-first shape `k8s/{core,ports,adapters,facade,observability}/`
with the four `managed-k8s-*` product cells refactored into
`k8s/{core,ports,adapters,facade}/<cell>-<role>`. Evidence of completeness: all four
`cloud/managed-k8s-*` dirs now hold **0 `Cargo.toml`** (crates moved out; husk docs remain) and
`cloud/cloud-k8s/` holds **0 `Cargo.toml`** (docs-only; the owned control-plane `core/` is not yet
built). This is ADR-0562 strangler move 13 (root `Cargo.toml` `k8s/*/*` glob). **k8s is out of
scope for task #6** except as the reference precedent — its managed-k8s→k8s move is exactly the
pattern os/ follows.

### 1.6 External dependents (the cross-capability seams a move must rewrite)

- **cloud-os**: 3 external dependent crates, all importing exactly **one** crate
  `oya-cloud-os-trustd-domain` via `path = "../../../cloud/cloud-os/crates/oya-cloud-os-trustd-domain"`:
  - `iam/adapters/identity-workload-svid-operator-k8s`
  - `iam/adapters/identity-workload-svid-trustd`
  - `iam/facade/cloud-pdp-app`
  (`git grep -l oya-cloud-os- origin/dev -- '*/Cargo.toml'`, excluding cloud-os itself.) The
  edge is iam→os (Tier-1 → rung-1 floor) = a legal downward edge, no acyclicity violation.
- **cloud-kernel**: **0 external dependents** (`git grep -l oya-cloud-kernel- … | grep -v cloud/cloud-kernel/`
  is empty). Every cargo/BUCK/Rust reference is intra-`cloud-kernel` — a fully self-contained
  move, like the iac §10.6 precedent.

---

## 2. Target shape (capability-first, de-branded, path = namespace)

### 2.1 Doctrine note — kernel/ and os/ are FLOOR RUNGS, not four-face capabilities

ADR-0562 §1 places `kernel/` (rung 0) and `os/` (rung 1) as **meta/floor directories**, distinct
from `<capability>/` dirs. §3 placement rule 1 routes them directly ("the kuberos kernel →
`kernel/`; the node OS → `os/`") **without** mandating the strict `core|ports|adapters|facade`
face-split that §4/§6 impose on capability crates. Precedent: `kernel/` already uses bespoke folds
(`core/` + `harness/`). So the faces apply to os/ as a *classification aid where they fit*, not as
a hard four-fold requirement. The task-prompt's "each core|ports|adapters|facade" is satisfied by
mapping the crates onto the faces they naturally occupy (below); the floor-rung status means an
unused face is simply absent (os/ has no adapters/ under the recommended mapping — flagged).

### 2.2 `os/` before→after mapping (recommended face classification)

De-brand: drop the `oya-cloud-os-` prefix to the capability slug `os-`; cargo name = path-tail leaf
(the fold is carried by the path + manifest `face:` facet, matching the §10.5/§10.6 precedent).

| before (`cloud/cloud-os/crates/…`) | after (path) | cargo name | face |
|---|---|---|---|
| `oya-cloud-os-<X>-domain` ×37 | `os/core/<X>-domain` | `os-<X>-domain` | `core` |
| `oya-cloud-os-kernel` | `os/core/kernel` | `os-kernel` | `core` |
| `oya-cloud-os-proto-api` | `os/ports/proto-api` | `os-proto-api` | `ports` |
| `oya-cloud-os-init-app` | `os/facade/init-app` | `os-init-app` | `facade` |
| `oya-cloud-os-difftest-app` | `os/facade/difftest-app` | `os-difftest-app` | `facade` |

Result: `os/core/*` (38) + `os/ports/proto-api` + `os/facade/{init,difftest}-app`. One root
workspace glob `os/*/*` covers all (ADR-0538). `os/OWNERS` seeded.

**Classification caveats to confirm before the codemod run (§4.4):**
- The 38→`core/` default treats every `-domain` as engine. A few Talos domains wrap *transient
  infra* (`etcd-domain` wraps etcd; `runtime-cri-domain` wraps a CRI; arguably `siderolink-domain`
  wraps WireGuard) and are **adapters/ candidates** under a strict face read. Recommended: keep
  them in `core/` for the crate-first move (minimal reshaping, matches the current flat shape) and
  handle any adapter re-facing as a later narrow split (ADR-0562 §7), OR classify them to
  `os/adapters/*` now if the founder prefers a strict first pass. Flag for founder pick.
- `difftest-app` is a test/diff tool, not a deployable — `os/facade/difftest-app` is defensible
  (single-capability app IS a facade, §6) but a `harness`/bespoke fold (mirroring
  `kernel/harness/`) is the alternative. Flag.

### 2.3 `kernel/` target (already substantially achieved; disposition of cloud-kernel is the open part)

- **Canonical kernel content**: already at `kernel/core/asterinas-boundary` +
  `kernel/harness/asterinas-real-boot`, already de-branded (`kernel-*`). **No move needed.**
- **Ground-up `cloud/cloud-kernel`**: target depends on the founder pivot decision (§4.1). Under
  the Asterinas-pivot-conformant branch, it is **reference**, not canonical — it does NOT become
  the `kernel/` engine. Candidate homes (all need founder ruling): `kernel/reference/*` (retained
  as a nested sub-workspace under the already-excluded `kernel/`), archived (git history retains
  it), or `third-party/` (there is a `third-party/` top-level dir). A new top-level `reference/`
  dir is **forbidden** by the §6 membership lint (not in `kernel|os|base|governance|build|app` and
  not a registered capability), so "reference" must live under an existing sanctioned root.
- **Registry reconciliation**: `specs/capability-registry.json`
  `meta_directory_absorbs[kernel/].current_dirs = ["cloud/cloud-kernel"]` must be **amended** to
  encode reference-demotion (e.g. mark it reference, or repoint to the chosen home), not executed
  literally as "cloud-kernel becomes the canonical kernel engine."

---

## 3. Ordered strangler move plan (foundation-first, one-move-per-PR)

Per `.omc/ultragoal/strangler-move-playbook.md`: **SERIAL, one capability per PR**, executed by
`tools/oya-reorg-codemod-app` (NOT hand-moved), each move a **pure structural rename+rewire** with
**NO face materialization** (revised 2026-07-09 — a move commits zero `*.generated.json`
projection/accounting faces; the only committed face is `specs/reorg/move-manifest.generated.json`,
regenerated deterministically by the codemod from the committed move-plan).

DAG position (ADR-0280 §D-1 / `specs/substrate-dependency-dag.json`): kernel (rung 0) < os (rung 1)
< cell < Tier-1 ten. Both are the recursion floor — deepest substrate, but with the fewest
dependents (0 and 3), so they are clean leaves to move.

### MOVE A — `os/` (cloud/cloud-os → os/) — READY, execute now

The largest single move to date (41 crates vs prior 2/5/17/18), but mechanically clean.

**Scope**: relocate all 41 `cloud/cloud-os/crates/*` per the §2.2 table; rewrite the 3 iam
dependents' cargo path-deps + dep name + Rust `use`/`extern crate` idents mechanically
(`oya_cloud_os_trustd_domain` → `os_trustd_domain`, path `../../../os/core/trustd-domain`).

**Committed move-plan (ADR-0563, MANDATORY, exactly one)**: `specs/reorg/os-move-plan.json` —
the codemod's `MovePlan` (old_dir→new_dir bijection, the 41 pairs). Naming matches the existing
`specs/reorg/<capability>-move-plan.json` convention. The move-manifest regenerates from it.

**Per-move contract edits (playbook (a)–(e); paths updated for the #1216 CI relocation
`cloud/cloud-ci/… → ci/facade/…`)**:
- (a) root `Cargo.toml` workspace members: add glob **`os/*/*`** (verify it matches all 41 moved
  dirs and over-matches nothing; resolver stays `libs/oya-workspace-members-kernel`, ADR-0538).
- (b) `specs/capability-registry.json`: `meta_directory_absorbs[os/].current_dirs` stays
  `["cloud/cloud-os"]` (pre-move read); ensure the membership scan can map `os/*` post-move. Add
  own-slug `os` to the mapping if the membership gate REDs `MEM-NEW-UNMAPPED-CRATE os/…` (the
  #744 self-slug lesson).
- (c) **`ci/facade/module-membership/capability-membership-policy.json`**: add `os` to
  `scan_roots` + `allowed_top_level_dirs`.
- (d) **`ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json`**: add
  `os/*/*` to `crate_root_globs` + `os` to `unclassified_roots`.
- (e) born-accounting (ADR-0555): `os/OWNERS`; ADR-0562 §10.x justification anchor for the new
  paths; `specs/reachability-registry.json` seed for the `os/` tree.

**Conditional contract surfaces (check during the move)**:
- Embedded-asset hermeticity: cloud-os has **1** include site
  (`oya-cloud-os-init-app/tests/boot_config_address.rs`, a test). If it is `include_str!`/
  `include_bytes!` of a repo asset, add `os` to
  `ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-policy.json` `scan_roots` and
  relabel the hermeticity baseline old→new (playbook include-site lesson). Verify; likely a
  test-only fixture but confirm.
- **Co-move**: cloud-os has **no** `slos/`, **no** `catalog/*.yaml` (0 catalog entries found),
  and only `cloud/cloud-os/manifest.json` as a non-crate artifact. So there is effectively
  nothing to co-move beyond registry + Cargo.lock + move-manifest. The `manifest.json` husk stays
  (phase-2, task #62) or is deleted; the dir approaches messaging's clean-vanish case, not iac's
  242-artifact case.

**Gates each must pass (playbook)**:
- HARD GATE: codemod full-tree buck2 dry-run → `cargo_ok=true AND buck_ok=true AND clean=true`;
  **fail-closed if `buck_ok` is null** (null-as-pass is the false-green class).
- Freshness / affected-set: after the move, `buck2 build`+`buck2 test` the 41 moved crates + the
  3 iam dependents; then the **full firewall ratchet vs merge-base** — materialize CI faces via
  the #1216-relocated materializer
  `//ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin`, then
  `buck2 test //ci/facade/...` (running individual gates is a false-green class; the merge-base
  ratchet is authoritative). Membership + acyclicity **0 regression** against the frozen baselines
  (62-unmapped / 12-violation). `cargo metadata --locked` clean; `kernel/`… n/a; root `Cargo.lock`
  synced. grep-clean of `oya-cloud-os`/`oya_cloud_os` anywhere. Conflict-marker sweep.
- Post-merge dev push-tier verify GREEN (push-tier ≠ PR-tier; ref the gate-baseline
  PR/push-asymmetry discipline) before starting any next move.
- `git status` after the move shows ONLY renames/rewires + `specs/reorg/os-move-plan.json` +
  `specs/reorg/move-manifest.generated.json` + registry + policy files + `Cargo.lock` — **zero
  other `*.generated.json` faces** (the 2026-07-09 no-materialization invariant).

**Review**: independent fresh-context adversarial pass (playbook step 3), pinned to PR head, via
`gh pr diff`/`gh api` or a detached throwaway worktree — never mutate the canonical checkout.

### MOVE B — `kernel/` (cloud/cloud-kernel disposition) — BLOCKED on founder pivot-ADR

**Do not execute as a registry-literal move.** Sequence:

1. **Founder authors the Asterinas-pivot ADR** (the one ADR-0611 defers) deciding cloud-kernel's
   fate: canonical (registry-literal) vs **reference** (pivot-conformant, recommended) vs archive.
2. **Amend `specs/capability-registry.json`** `meta_directory_absorbs[kernel/]` to match the
   decision (reference-demotion, not "cloud-kernel = canonical engine").
3. **If reference/relocate**: run the strangler move `cloud/cloud-kernel → <chosen home under a
   sanctioned root>` (e.g. `kernel/reference/*`). Mechanically this is the **cleanest possible
   move — 0 external dependents**, all rewrites intra-capability (iac §10.6 class). Nested-workspace
   nuance: cloud-kernel is its own excluded `no_std` workspace; if it lands under `kernel/` it
   either (i) merges into kernel/'s nested workspace (only if their sysroot/toolchain pins are
   compatible — verify; ADR-0611's Asterinas boundary is std-host dev tooling, cloud-kernel is
   bare-metal `no_std` — likely INCOMPATIBLE, so keep it a distinct nested sub-workspace and keep
   the root `exclude` entry, repointed) or (ii) stays a separate excluded workspace at its new
   path. Update root `Cargo.toml` `exclude` (`cloud/cloud-kernel` → new path), membership
   `allowed_top_level_dirs`/`scan_roots`, acyclicity globs, reachability-registry, OWNERS.
4. **If archive**: remove `cloud/cloud-kernel` (git history retains it), drop the root `exclude`
   entry + the registry `meta_directory_absorbs` line, and record the archive rationale in the
   pivot ADR. No new home needed.

The canonical `kernel/` content (asterinas-boundary + real-boot) is already home and de-branded —
**no move is required for it**.

---

## 4. Risks, dependencies, open founder decisions

### 4.1 OPEN DECISION (blocking, kernel/ half) — cloud-kernel disposition

The registry says `kernel/ ← cloud/cloud-kernel`; the Asterinas CANONICAL pivot demotes the
ground-up cloud-kernel to reference; the deciding pivot ADR is unauthored (ADR-0611 explicitly
defers it). **Executing the registry literal would install the ground-up kernel as the canonical
engine — the opposite of the ratified pivot.** Founder must author the pivot ADR and pick
canonical / reference-relocate / archive before MOVE B. Recommended: reference-relocate to
`kernel/reference/` as a retained distinct nested workspace (or archive), and amend the registry.

### 4.2 Asterinas black-box constraints (ADR-0562 rule-gated adapters, ADR-0611)

The canonical kernel is the *unmodified upstream Asterinas v0.17.2 release ISO* pinned by digest
behind a zero-dep boundary crate (`kernel/core/asterinas-boundary`) + a closed boot-ready marker
set; evidence is a *real* QEMU boot (anti-simulation, receipts gitignored + regenerated by CI).
Implications for this reorg: (a) `kernel/` is a **nested/excluded workspace** — never fold it into
the root workspace or the os/ move; the only sanctioned nested workspace (ADR-0512 carve-out, root
`Cargo.toml` `exclude "kernel"`). (b) The Asterinas pin/marker/harness crates are *not* subject to
capability face re-classification — they are the floor rung's bespoke `core/`+`harness/` folds. (c)
Any cloud-kernel relocation must preserve its independent `no_std`/bare-metal toolchain pin (own
`rust-toolchain.toml` + `Cargo.lock`); it cannot share kernel/'s or the root lockfile.

### 4.3 Interplay with the in-flight CI-gate reorg (#1216)

The CI tree relocated `cloud/cloud-ci/… → ci/facade/…` in #1216. The playbook still cites the old
`cloud/cloud-ci/gates/…` paths; the **current** policy files this move edits are:
`ci/facade/module-membership/capability-membership-policy.json`,
`ci/facade/layer-dependency-acyclicity/tier-dependency-acyclicity-policy.json`,
`ci/facade/embedded-asset-hermeticity/embedded-asset-hermeticity-policy.json`. The firewall ratchet
target is `buck2 test //ci/facade/...` and the materializer is
`//ci/facade/generated-artifact-freshness:oya-cloud-ci-materialize-generated-faces-bin`. Risk: a
move authored against the stale playbook paths would edit non-existent files and false-green. Use
the `ci/facade/…` paths verified above. Also: don't collide with any concurrent ci-capability move
(serial discipline — the shared registry/policy/baseline state is the collision surface).

### 4.4 os/ face-classification correctness (largest move)

41 crates in one PR is 2–8× prior moves. The mechanical risk is low (codemod-driven, HARD-gated on
`cargo_ok && buck_ok && clean`), but the *classification* (§2.2) is a judgement call the codemod
cannot make: default all-`-domain`→`core/` vs strict adapters/ for the transient-infra wrappers
(etcd/cri/siderolink); difftest-app facade vs harness. Recommend the founder ratify the §2.2 table
(or the strict-adapter variant) before the codemod run — re-facing after landing is a narrow-split
ADR (§7), more expensive than getting it right in the one move.

### 4.5 Dependency ordering / no acyclicity regression

kernel (rung 0) and os (rung 1) are below all capabilities; the only cross-capability edge is
iam→os (`trustd-domain`), a legal downward edge. Neither move introduces a new upward/cyclic edge,
so both should land at **0 acyclicity regression** against the frozen 12-violation baseline. cloud-os
and cloud-kernel are NOT in the playbook's deferred violation-source list (kms/network/billing/
intelligence/community/application), confirming both are clean leaves.

### 4.6 Baselines & rename-aware engine

Both moves relocate crates that may carry accepted per-file total-accounting debt
(`unjustified`/`unowned`/`unreachable`) and frozen tier-dep/brand-residue baselines. The ADR-0563
rename-aware baseline engine relabels these old→new automatically from the committed
`specs/reorg/<cap>-move-plan.json` — **provided exactly one move-plan is committed per PR**
(the engine no-ops without it, causing false-RED). This is a hard protocol step, not optional.

### 4.7 Residual / phase-2

De-brand *residue* (binary `[[bin]] name`, `OYA_CLOUD_OS_*` / `CLOUD_OS_*` runtime constants and
env-var contracts, `microservices/…`/`target/…` path string literals) is **NOT** rewritten by the
crate codemod — it is the ADR-0532/0533 de-brand profile lane + codemod-hardening task #63,
deferred (gate-green, non-corrupting). The `cloud/cloud-os/manifest.json` husk and any left-behind
non-crate artifacts are homed in phase-2 (task #62). Note both explicitly in the move PR so review
does not read them as incomplete.

---

## 5. Recommended execution order

1. **MOVE A (os/)** — unblocked; execute per §3 MOVE A. One PR, codemod-driven, 41 crates,
   `specs/reorg/os-move-plan.json`, full firewall ratchet + 0-regression baselines.
2. **Founder pivot-ADR for cloud-kernel** (§4.1) — unblocks MOVE B.
3. **MOVE B (kernel/)** — per the ADR's disposition (reference-relocate recommended, or archive);
   the canonical Asterinas content is already home, so MOVE B is only about cloud-kernel.

Nothing in this plan moves a crate — it is the reviewed spec the two strangler PRs implement.
