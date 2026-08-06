# MONOREPO-CONFORMANCE AUDIT + MIGRATION L2→L11 READINESS

> **STATUS: pending-approval.** Board task #20 (AUDIT) + prep for #7 (the std-first PRs) / #52 (AP6 migration deltas).
> **Authoring location:** `linux/docs/audit/initial-sweep-2026-06-06/` — READ-ONLY on `/Users/jasonlee/Developer/source` (a background executor owns that tree, mid-commit on `cleanup/whole-tree-2026-06-07`). NO source mutation, NO git add/commit, NO push. Every claim below cites a real file path + line or a real command's real output.
> **Date:** 2026-06-08.
> **Companion docs (existing, in this dir):** `MIGRATION-PLAN-RESYNC.md` (the L1–L11 layer model + §2.5 gate list); `OYA-CI-CONFORMANCE-FLOOR-PLAN.md`; `UNIFIED-EXECUTION-PLAN.md`.

---

## 0. Why this doc exists now (the new constraint)

The MIGRATION-PLAN-RESYNC.md `§2.5` conformance gates were a *plan*. They are now **LIVE blocking checks**: the four conformance-floor gates (`bnf-layer-suffix`, `manifest-hygiene`, `cargo-prefix`, `brand-residue`) plus the four accounting/agreement gates (`total-accounting`, `cross-artifact-agreement`, `staleness-reaper`, `automation-ratchet`) are enabled in the producer's config and fan into the single required check `oya-ci-required` (ADR-0515). The firewall is a **shrink-only ratchet**: it BLOCKS any PR that introduces a NEW violation key not already in the frozen baseline.

The migration must therefore import L2→L11 **without tripping the gate that is now defending the canon** — i.e. without a *self-DoS*. This audit characterizes, gate-by-gate, what each remaining layer's crate NAMES + manifests would do to the live floor, and sequences the imports so each lane lands floor-green (the proven L1 office shape), never RED-then-fix.

---

## 1. The live floor — verified ground truth

### 1.1 The config-driven gate set (SSOT)

The policy is DATA, loaded from the repo-root config by the producer.

- **Repo-root config:** `/Users/jasonlee/Developer/source/oya-ci.toml`. Enables 8 gates (`[[gates.enabled]]`), 7 with `input_kind = "producer-face"` and one (`cloud-ci-brand-residue`) with `input_kind = "raw-corpus-collector"`.
- **Closed-schema config kernel:** `/Users/jasonlee/Developer/source/libs/oya-ci-config/src/lib.rs`. `#[serde(deny_unknown_fields)]` throughout; the bundled default reproduces today's `const`s byte-for-byte (test `bundled_default_matches_todays_naming_consts`, lib.rs:698–712).
- The **13-role BNF enum** (ADR-0056) is `[naming].allowed_roles` in oya-ci.toml:14 and `default_allowed_roles()` lib.rs:221–240:
  `kernel · domain · usecase · app · adapter · infrastructure · cli · rest · grpc · graphql · worker · sdk · api`.
  The test `bundled_default_matches_todays_naming_consts` asserts `allowed_roles.len() == 13` and `!contains("runtime")` (lib.rs:704–705).
- **Required prefix:** `oya-` (`[naming].required_prefix`, oya-ci.toml:13).
- **Manifest required flags** (`[manifest].required_flags`, oya-ci.toml:127 + lib.rs:403–415):
  `version_workspace · rust_version_workspace · publish_false · license · lints_workspace · lib_doctest_false`.
- **Forbidden vocab stems** (`[[vocab.forbidden_stems]]`, oya-ci.toml:22–35 + lib.rs:314–327):
  `foundry · forgejo · jenkins · oya-vcs` (each with a `forbidden_<stem>` code). Carve-outs (oya-ci.toml:37–88): the deny-list SSOT paths themselves, `oya/intelligence/_legacy-foundry/`, `evidence/audit-chain.jsonl`, `*.generated.json`, and the line-level `palantir` exemption (Palantir-Foundry is a competitor proper noun).

> Note: `forge` is named in MIGRATION-PLAN-RESYNC §1i/§1j as a target FORBID stem, but it is **NOT** in the live `[[vocab.forbidden_stems]]` table (oya-ci.toml only lists `foundry/forgejo/jenkins/oya-vcs`). `talos-`/`kuberos`/`oyatie-`/`oyago`/`oyapy`/`oyaoffice` are codename-rename targets, NOT live brand-residue stems. The live brand-residue gate only fires on the 4 stems above.

### 1.2 The ratchet mechanics (how a NEW key REDs the required check)

Source: `/Users/jasonlee/Developer/source/cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/src/lib.rs`.

Each gate's baseline entry is `{mode, frozen_empty, keys}` (firewall lib.rs:50–55). Every floor-gate code is `mode: "baseline-block-on-new"` (verified in the live baseline — see §1.3). There are **two independent fail conditions**, both pure DATA-over-DATA:

1. **COMPARE-MODE** (lib.rs:168–216, `CodeReport::fails` at lib.rs:143–146): for each `(gate, code)`, `regressions = current_keys \ baseline_keys`. A code FAILs iff `mode == "baseline-block-on-new" && !regressions.is_empty()`. **One NEW violation key → the gate is RED → `oya-ci-required` is RED.** Keys already in the baseline are `tolerated` (no fail); removed keys are `fixed` (shrink, informational).
2. **RATCHET-INVARIANT** (lib.rs:223–246, `ratchet_growth`): a regen may only SHRINK the committed baseline. Any key a regen would ADD (`proposed \ committed`) is a `ratchet_regression` FAILURE **unless** it is in the founder-signed `_sign_off_additions` allowlist (`gate-baseline.signoff.json`, the ONE-WAY DOOR, lib.rs:90–124). So you **cannot launder a new violation into the baseline by re-running the producer** — that is itself a RED.

**Net self-DoS rule:** importing a crate whose name/manifest produces a violation key that is not already baselined RED's the required check via (1), and the only escape hatches are: pre-conform the crate so it produces NO violation key (the L1 office path), or obtain an explicit founder signoff entry (the one-way door, reserved for true tolerated debt — currently holds only the firewall's own bootstrap substrate, see §1.4).

### 1.3 The live floor-gate baselines (per-code key counts)

Source: `/Users/jasonlee/Developer/source/cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/gate-baseline.generated.json` (3.86 MB, `_provenance.config_digest = fnv1a64:0ad7405174ca4a0c`). All counts below from a real `python3 json.load` over that file:

| Gate | Code | Baseline keys | Key = |
|---|---|---:|---|
| `cloud-ci-bnf-layer-suffix` | `bnf_unknown_role` | **79** | crate NAME (non-canonical trailing segment) |
| | `bnf_missing_oya_prefix` | 0 | crate NAME |
| | `bnf_role_mismatch` / `bnf_undeclared_role` / `bnf_undeclared_context` / `bnf_name_uppercase` / `bnf_empty_after_prefix` | 0 each | crate NAME |
| `cloud-ci-manifest-hygiene` | `manifest_missing_version_workspace` | **68** | crate NAME |
| | `manifest_missing_rust_version_workspace` | **67** | crate NAME |
| | `manifest_missing_lints_workspace` | **48** | crate NAME |
| | `manifest_missing_lib_doctest_false` | **25** | crate NAME |
| | `manifest_missing_publish_false` | **23** | crate NAME |
| | `manifest_missing_license` | 2 | crate NAME |
| `cloud-ci-cargo-prefix` | `cargo_prefix_name_path_mismatch` | 1 (`oya-ci-config`) | crate NAME |
| | `cargo_prefix_violation` / `cargo_prefix_unresolvable` | 0 each | crate NAME |
| `cloud-ci-brand-residue` | `forbidden_foundry` | **3184** | file PATH |
| | `forbidden_jenkins` | **1188** | file PATH |
| | `forbidden_oya-vcs` | 108 | file PATH |
| | `forbidden_forgejo` | 14 | file PATH |

**Read this carefully:** the floor gates already hold a large baselined debt. That debt is *tolerated* (it is the frozen pre-existing corpus). The ratchet only blocks **NEW keys**. A migrated crate that produces a name/path **already in these baselines** is tolerated; a migrated crate that produces a **new** key is a RED.

### 1.4 The signoff one-way door (currently near-empty)

Source: `/Users/jasonlee/Developer/source/cloud/cloud-ci/gates/oya-cloud-ci-firewall-app/gate-baseline.signoff.json`. The only `_sign_off_additions` are 2 `total-accounting` rows admitting the firewall's OWN required-CI substrate (`.github/workflows/oya-ci-required.yml` + `gate_registration.rs`) — a bootstrap exemption (ADR-0515, signed 2026-06-08). **There is NO floor-gate signoff entry.** This proves the doctrine: every prior import (L1 office) landed floor-clean WITHOUT a floor signoff — the bar the L2→L11 lanes must clear.

### 1.5 How each floor gate decides a violation (the exact predicate)

- **BNF layer-suffix** (`oya-cloud-ci-bnf-layer-suffix-app/src/lib.rs`): resolves each crate's trailing dash-segment as its role and runs `oya_governance_predictable_naming_kernel::check`. Carve-outs (lib.rs:131–151): `oya-check-*` (check-family) and `oya-tooling-agent-read` → exempt; `oya-<svc>-adapter-<backend>` → effective role `adapter` (exempt). Everything else: trailing segment must be one of the 13 roles, else `bnf_unknown_role` (key = crate name). Tests confirm `oya-foo-runtime`, `oya-bar-core`, `oya-baz-service` all fire `bnf_unknown_role` (lib.rs:222–229); a non-`oya-` name fires `bnf_missing_oya_prefix` (lib.rs:260–264).
- **Manifest hygiene** (`oya-cloud-ci-manifest-hygiene-app/src/lib.rs:87–119`): one `Finding` per missing flag per crate. `lib_doctest_false` only required when a `[lib]` table is present (lib.rs:114; test `doctest_not_required_without_lib`).
- **Cargo-prefix** (`oya-cloud-ci-cargo-prefix-app/src/lib.rs:137–160`): per-member `validate_cargo_prefix` — both the member-path leaf AND `[package].name` must start with `oya-` and agree. Unprefixed → `cargo_prefix_violation`; prefixed-but-disagree → `cargo_prefix_name_path_mismatch`.
- **Brand-residue**: a raw-corpus collector — any tracked file (outside the carve-outs) containing a forbidden stem emits a key = that file path.

---

## 2. The proven L1 office template (the shape every lane must mirror)

L1 office is DONE and held at `cleanup/whole-tree-2026-06-07`. Verified from real git output:

- Import commit **`03e2a25e8`** — *"L1 office migration: import 13 oyaoffice-* crates into oya/office (floor-conformant) … renamed oyaoffice-* -> oya-office-*[-<bc>]-<layer> per ADR-0056 BNF v4.1"*.
- Settle commit **`18121e6ad`** — *"L1 office migration: faces-only settle (last_touch_commit convergence) … No source change; faces-only. registry-drift now converges (committed == regenerated)."* (This is the held commit named in the task.)

### 2.1 The rename map that was applied (source → home)

Original names at `/Users/jasonlee/Developer/office/crates` (13) → conformant names at `/Users/jasonlee/Developer/source/oya/office` (13). The deltas that mattered for the floor:

| Original (`~/Developer/office/crates`) | Imported (`source/oya/office`) | Floor reason |
|---|---|---|
| `oyaoffice-kernel` | `oya-office-kernel` | brand `oyaoffice-`→`oya-office-` |
| `oyaoffice-search-port` | `oya-office-search-kernel` | **`-port` is NOT a role** → would fire `bnf_unknown_role`; renamed to `-kernel` |
| `oyaoffice-storage-port` | `oya-office-storage-kernel` | `-port` → `-kernel` |
| `oyaoffice-api-contracts` | `oya-office-sheets-api` (et al.) | **`-contracts` is NOT a role** → renamed to `-api` |
| `oyaoffice-drive-api-contracts` | `oya-office-drive-api` | `-contracts` → `-api` |
| `oyaoffice-*-domain` (8) | `oya-office-*-domain` | `-domain` is canonical; brand-only rename |

### 2.2 The manifest hygiene that was applied

Pre-rename manifest (`~/Developer/office/crates/oyaoffice-collab-domain/Cargo.toml`): `version = "0.1.0"` (literal, not workspace), `license.workspace = true`, `repository.workspace = true`, NO `[lib] doctest = false`, plus a `[package.metadata.oyaoffice]` block.

Post-import manifest (`source/oya/office/oya-office-doc-domain/Cargo.toml`, the proven hygiene template):
```toml
[package]
name = "oya-office-doc-domain"
edition.workspace = true
version.workspace = true        # was version = "0.1.0"
rust-version.workspace = true
publish = false                 # ADDED
license = "Apache-2.0"          # was license.workspace; now explicit Apache-2.0
[lib]
name = "oya_office_doc_domain"
path = "src/lib.rs"
doctest = false                 # ADDED (required when [lib] present)
[lints]
workspace = true
```
All 6 §2.5#7 manifest flags satisfied. The `metadata.oyaoffice` block was dropped.

### 2.3 PROOF the template lands floor-green (no signoff needed)

Real `json.load` over the live baseline searching for `office` in the floor gates:
- `cloud-ci-bnf-layer-suffix`: office keys = **[]**
- `cloud-ci-manifest-hygiene`: office keys = **[]**
- `cloud-ci-cargo-prefix`: office keys = **[]**
- `cloud-ci-brand-residue`: 9 hits — but ALL are pre-existing docs/personas (`docs/adr-archive/ADR-0319-front-middle-back-office-information-barrier.md `docs/personas/office-manager-…`, etc.), NONE are the imported `oya-office-*` crate files.

**Conclusion:** because the crates were renamed + hygiene-fixed BEFORE import, they added ZERO floor-gate violation keys → the import did not RED any floor gate → no floor signoff was required (§1.4). The import DID add `oya/office/**` file paths to the *accounting* gate's RED-but-baselined codes (regen of `gate-baseline.generated.json`, 80 lines + `accounting-registry.generated.json`, 901 lines in `03e2a25e8`), then the faces-only settle (`18121e6ad`) converged registry-drift.

### 2.4 The 6-step per-lane shape (distilled from L1 + MIGRATION-PLAN-RESYNC §2.4)

1. **Inventory + rename map** — enumerate source crate names; map each to `oya-<bizctx>-<layer>` with a canonical 13-enum suffix.
2. **Capture pre-change baseline** — record current floor-gate keys (so the diff is provable).
3. **Copy + rename + manifest-hygiene** — copy first-party crates only; apply the rename map; rewrite every Cargo.toml to the §2.3 template; add `[lib] doctest=false`; drop foreign `metadata.*` blocks; relicense to Apache-2.0.
4. **Workspace add + cargo build/test green** — add members to root `Cargo.toml` (one-version, no nested `[workspace]`); `cargo build/test --workspace`.
5. **Regen + floor-green + freeze** — run the producer to regen the faces; assert the firewall is GREEN (no new floor keys, no ratchet growth); faces-only settle so `committed == regenerated`.
6. **Signed atomic commit (NEVER push)** — one import commit + one faces-settle commit, SSH-signed.

---

## 3. L2→L11 census + per-layer floor audit (against the LIVE gates)

Source-of-truth siblings per MIGRATION-PLAN-RESYNC §2.2. All crate names below are from real `ls`/`grep` over the on-disk sources (absolute paths shown). For each layer: would the names trip `bnf-layer-suffix`? would manifests miss the 6 hygiene flags? would names trip `cargo-prefix`? would they introduce forbidden brand vocab?

### L2 — oyago (transpiler-go-to-rust), source `~/Developer/oyago/crates` (3 crates)
Real names: `oyago-cli`, `oyago-core`, `oyago-runtime`.
- **bnf-layer-suffix: WOULD TRIP (2 new keys).** `-core` and `-runtime` are NOT in the 13-role enum → `bnf_unknown_role` for `oyago-core`, `oyago-runtime` (mirrors the gate test `oya-foo-runtime`/`oya-bar-core` → `bnf_unknown_role`, bnf lib.rs:222–229). `-cli` IS canonical. (After the planned `oyago-* → oya-transpiler-go-to-rust-*` rename, the brand fixes but the SUFFIX must still change: `-core`→`-kernel`/`-domain`, `-runtime`→a real role.)
- **cargo-prefix: WOULD TRIP (3 new keys).** Names start `oyago-`, not `oya-` → `cargo_prefix_violation` for all 3 until renamed.
- **manifest-hygiene: WOULD TRIP.** `~/Developer/oyago/crates/oyago-core/Cargo.toml` has `version = "0.0.0"` (not workspace), NO `publish=false`, NO `[lib] doctest=false`, `license.workspace` (the gate wants a `license` key present — workspace-inherited license satisfies `has_license` only if resolved; safest is explicit). → likely `manifest_missing_version_workspace`, `manifest_missing_publish_false`, `manifest_missing_lib_doctest_false`.
- **brand-residue: FITS.** `grep -ril foundry|jenkins|forgejo|oya-vcs ~/Developer/oyago` → **no matches**.

### L3 — oyapy (transpiler-python-to-rust), source `~/Developer/oyapy/crates` (3 crates)
Real names: `oyapy-cli`, `oyapy-core`, `oyapy-runtime`. **Identical floor profile to L2** (`-core`/`-runtime` non-canonical, `oyapy-` non-prefix, manifests need hygiene). brand-residue: no matches. The cleanest pattern after office — 3 small crates each.

### L4 — claude SDK, source `~/Developer/claude` (1 crate)
Real name: `claude-agent-sdk` (`~/Developer/claude/Cargo.toml`).
- **bnf-layer-suffix: WOULD TRIP.** Trailing segment `-sdk` IS canonical (role #12) — but the name lacks the `oya-` prefix → `bnf_missing_oya_prefix` (bnf lib.rs:117) until renamed.
- **cargo-prefix: WOULD TRIP** (1 new key, `claude-agent-sdk` not `oya-`).
- **manifest-hygiene: AUDIT NEEDED at copy time** (relicense MIT→Apache-2.0 per D-CONFORM #2; add the 6 flags). Home: `cloud/cloud-intelligence/crates/oya-cloud-intelligence-anthropic-claude-adapter` → trailing `-adapter` IS canonical (role #5).
- **brand-residue: FITS** (no matches).

### L5 — codex SDK, source `~/Developer/codex/sdk/rust` (1 crate)
Real names: package `openai-codex-sdk` (lib `openai_codex_sdk`).
- **bnf-layer-suffix / cargo-prefix: WOULD TRIP** (non-`oya-` prefix). Home: NEW sibling `oya-cloud-intelligence-codex-adapter` (trailing `-adapter` canonical). Vendor as `openai-codex-sdk` under a vendor classification (A/B/C registry, §2.5#9).
- **brand-residue: FITS** (no matches).

### L6 — k8s (our apimachinery crates), source `linux/stack/kubernetes/crates` (95 of 139)
Real sample names: `admissionregistration_v1`, `api_equality`, `api_meta`, `apps_v1`, `authentication_v1`, `autoscaling_v1`, `batch_v1`, `core_v1_proto`, `coordination_v1`, … (95 total after removing the 44 `ctrd_*`).
- **bnf-layer-suffix: WOULD TRIP MASSIVELY.** Trailing segments are `_v1`, `_proto`, `_equality`, `_meta`, … — none are canonical roles → `bnf_unknown_role` for ~all 95. Also `bnf_name_uppercase` risk is low (lowercase) but the snake_case `_` is itself non-conforming.
- **cargo-prefix: WOULD TRIP for all 95** — no `oya-` prefix, and snake_case (`_`) member-path leaves. Home: `managed-k8s-control-plane-host` → `oya-cloud-k8s-*` (ADR-0015/0016). **MERGE lane** (not CREATE) — the riskiest reshape next to L11.
- **manifest-hygiene: AUDIT at copy time** (95 manifests).
- **brand-residue: AUDIT** — k8s vendor code is large; must scan post-copy for any `jenkins`/`foundry` strings in vendored comments (low risk but 95 crates is a big surface).
- **This is the single biggest BNF + cargo-prefix self-DoS surface** (~95 new keys each if imported raw).

### L7 — containerd, source `linux/stack/kubernetes/crates` (44 `ctrd_*`)
Real sample names: `ctrd_api_types`, `ctrd_apparmor`, `ctrd_archive_link`, `ctrd_atomicfile`, `ctrd_blockio`, `ctrd_cap`, `ctrd_cio`, … (44 total, verified `ls | grep -c '^ctrd_' = 44`).
- **bnf-layer-suffix: WOULD TRIP for all 44.** `ctrd_*` is snake_case with non-role trailing segments (`_types`, `_apparmor`, `_cap`, …) → `bnf_unknown_role` ×44. MIGRATION-PLAN-RESYNC §2.5#4 explicitly calls out "reject … snake_case `ctrd_*`".
- **cargo-prefix: WOULD TRIP for all 44** (no `oya-`, snake_case). Home: `cloud/cloud-container-runtime` → `oya-cloud-container-runtime-*` (drop snake_case). CREATE lane.
- **manifest-hygiene: AUDIT at copy time.**
- **brand-residue: AUDIT** (containerd is a vendored Go→Rust port; scan post-copy).

### L9 — node OS (talos), source `linux/stack/operating-system` (45 crate manifests)
Real sample names: `talos-apid`, `talos-archiver`, `talos-block`, `talos-cluster`, `talos-controllers`, `talos-core`, `talos-etcd`, `talos-init`, `talos-k8s-control`, `talos-machined`, `talos-network`, `talos-runtime-cri`, … Plus a few bare names: `init`, `svc`, `difftest`, package `talos_init` (underscore).
- **bnf-layer-suffix: WOULD TRIP.** `talos-core` (`-core` non-role), `talos-network`, `talos-block`, `talos-init`, `talos-etcd` etc. — most trailing segments are NOT roles → `bnf_unknown_role`. Bare `init`/`svc`/`difftest` → `bnf_missing_oya_prefix`. `talos_init` (underscore package name) → non-conforming.
- **cargo-prefix: WOULD TRIP for all** (`talos-`/bare, not `oya-`). Home: `cloud/cloud-node-os` → `oya-cloud-node-os-*`. CREATE lane, NORMAL STD (talos is STD 1.96.0, not no_std — per MIGRATION-PLAN-RESYNC §2.1).
- **manifest-hygiene: AUDIT** (45 manifests).
- **brand-residue: AUDIT** (talos is a large vendored OS).

### L10 — docs (+ 13 pilot ADRs), source = linux pilot docs
- **Floor gates: N/A for crate gates** (no crates). **brand-residue applies** to doc PATHS: docs that contain `foundry`/`jenkins`/`forgejo`/`oya-vcs` would add NEW keys. The pilot docs must be scanned before import. ADR renumber is additive (no corpus renumber, D13-amend).

### L11 — framekernel (no_std, LAST), source `linux/stack/kernel` (21 manifests, 12-entry exclude subtree)
Real names: `kernel`, `frame`, `hal`, `ksync`, `arch-x86_64`, `arch-aarch64`, `user_layout`, `user-init-x86_64`, `user-hello-x86_64`, `fsbase-worker-x86_64`, `user-smpdemo`, … (21 total).
- **CRITICAL — workspace-EXCLUDED.** Per MIGRATION-PLAN-RESYNC §2.1/§1f, the ENTIRE `kernel/` subtree (12 no_std workspaces) lands under the `[workspace] exclude` key with its own pinned nightly-2026-02-28 + custom build-std targets. **A crate that is NOT a workspace member is NOT enumerated by the producer's manifest scan → it cannot add a floor-gate key.** This is the escape hatch for the otherwise-catastrophic BNF/cargo-prefix profile (`kernel`, `frame`, `hal`, `arch-x86_64`, `user_layout` would ALL trip if they were members).
- **The open question is whether the gate's repo-scan respects the exclude key.** The producer enumerates "first-party `oya-*` crate package names from the tracked Cargo.toml manifests" (bnf lib.rs:9–11). **VERIFY at execution time:** confirm the producer's manifest enumeration reads `[workspace].members` (excluded trees skipped) and NOT a raw `find **/Cargo.toml` (which would re-include the excluded kernel crates and RED every floor gate). This is the dominant L11 risk and must be proven by pre-lane 0.6 against the real merged root workspace (the 12-entry exclude key).
- **brand-residue still applies to file PATHS** even for excluded trees (the brand gate is a raw-corpus collector over tracked files, NOT gated on workspace membership) — scan the kernel subtree before import; today it is brand-clean for the kernel port but re-verify.

### L8 — DROPPED (no source). Per MIGRATION-PLAN-RESYNC §1b / canon D-CONFORM G4-D2: `cloud-data/db-engine` has no first-party crates; not a migration lane. Confirmed: not audited here because there is no source-of-truth sibling.

---

## 4. SELF-DoS RISK REGISTER (concrete NEW keys + mitigation)

Every row below is a concrete import action that WOULD add a NEW violation key to a live `baseline-block-on-new` floor gate, thus RED `oya-ci-required`. Mitigation column gives the proven escape.

| # | Layer | Gate | NEW keys it would add | Mitigation |
|---|---|---|---|---|
| D1 | L2/L3 | bnf-layer-suffix | `*-core`, `*-runtime` (4) | **pre-rename at source** before import: `-core`→`-kernel`/`-domain`, `-runtime`→canonical role (the L1 `-port`→`-kernel` template). |
| D2 | L2/L3/L4/L5 | cargo-prefix | every non-`oya-` name (8) | **pre-rename to `oya-…`** before adding to workspace (the L1 brand rename). |
| D3 | L6 k8s | bnf-layer-suffix | ~95 `*_v1`/`*_proto`/`*_meta` | **MERGE-rename all 95 to `oya-cloud-k8s-<thing>-<role>`**, drop snake_case. Biggest surface — do incrementally inside the lane, regen-green at the end. |
| D4 | L6 k8s | cargo-prefix | ~95 non-`oya-`+snake_case | same rename as D3 (member-path leaf + package name both `oya-`). |
| D5 | L7 ctrd | bnf-layer-suffix + cargo-prefix | 44 `ctrd_*` snake_case | **rename `ctrd_* → oya-cloud-container-runtime-<thing>-<role>`** (drop snake_case). |
| D6 | L9 node-os | bnf + cargo-prefix | ~45 `talos-*`/bare/`talos_init` | **rename `talos-* → oya-cloud-node-os-<thing>-<role>`**; fix bare `init`/`svc`/`difftest`. |
| D7 | ALL | manifest-hygiene | each crate missing any of 6 flags | **apply the §2.3 manifest template at copy time** (the L1 office shape): `version.workspace`, `rust-version.workspace`, `publish=false`, explicit `license="Apache-2.0"`, `[lints] workspace=true`, `[lib] doctest=false`. |
| D8 | L6/L7/L9/L10/L11 | brand-residue | any imported file containing `foundry`/`jenkins`/`forgejo`/`oya-vcs` | **scan-then-strip at copy** (deny-glob). Siblings L1–L5 are already brand-CLEAN (verified `grep` → no matches), so D8 is a risk ONLY for the larger vendored trees (k8s/ctrd/talos) + docs. |
| D9 | L11 kernel | ALL floor gates | every kernel crate IF the producer scan ignores the exclude key | **prove the producer enumerates `[workspace].members`** (not raw `find`) in pre-lane 0.6; keep the 12-entry `[workspace] exclude`. If the scan is membership-based, L11 adds ZERO crate-gate keys. |
| D10 | ALL | ratchet-invariant | trying to launder a new key by regen | **never regen-to-grow.** The signoff one-way door is founder-only and reserved for true tolerated debt; the migration's posture is pre-conform-to-zero, exactly as L1 office did (no floor signoff). |

**Cross-cutting note (cargo_prefix_name_path_mismatch):** the live baseline already holds `oya-ci-config` under `cargo_prefix_name_path_mismatch` (the package is `oya-ci-config-kernel` but the dir is `oya-ci-config`). The migration must NOT add new dir/name mismatches — each renamed crate's directory leaf must equal its `[package].name` (ADR-0017).

---

## 5. SEQUENCING L2→L11 (cheapest-highest-value-first)

Ordering rationale: (a) std-first / no_std-last (canon §2.1); (b) cheapest = fewest crates + already-conformant suffixes + brand-clean; (c) value = proves the template scales + unblocks the biggest reshapes last when the muscle memory is strongest. Each lane mirrors the L1 6-step shape (§2.4) and must end firewall-GREEN with `committed == regenerated` (no floor signoff).

| Order | Lane | Crates | Cost | Risk | Why here |
|---|---|---:|---|---|---|
| 1 | **L3 oyapy** | 3 | cheapest | low | smallest brand-clean tree; 2 suffix renames (`-core`/`-runtime`) + 3 prefix renames + manifest template. Identical to a 3-crate office. Proves template post-office. |
| 2 | **L2 oyago** | 3 | cheap | low | identical profile to L3 (same renames). Do back-to-back to lock the transpiler pattern. |
| 3 | **L4 claude-SDK** | 1 | trivial | low-med | 1 crate; `-sdk`→`-adapter` home, prefix rename, MIT→Apache relicense. Touches the relicense step early on a tiny surface. |
| 4 | **L5 codex-SDK** | 1 | trivial | low-med | 1 crate; vendor-classify `openai-codex-sdk`; NEW sibling adapter. Exercises the A/B/C vendor-registry step (§2.5#9) on a tiny surface. |
| 5 | **L9 node-os (talos)** | ~45 | medium | med-high | first LARGE std tree; ~45 renames (`talos-*`→`oya-cloud-node-os-*`) + bare-name fixes + brand-scan. CREATE lane. Proves the rename machinery scales before the k8s/ctrd giants. |
| 6 | **L7 containerd** | 44 | medium | high | 44 `ctrd_*` snake_case renames; CREATE lane. Vendored Go→Rust port — brand-scan matters. |
| 7 | **L6 k8s (MERGE)** | 95 | largest | highest (std) | ~95 renames INTO `managed-k8s-control-plane-host` (a MERGE, not CREATE) → also needs the MERGE-surface diff (§2.4). Biggest BNF+cargo-prefix surface. Do after L7 so the snake_case→canonical muscle is proven. |
| 8 | **L10 docs (+13 ADRs)** | 0 crates | low | low | no crate gates; brand-scan doc PATHS + additive ADR renumber. Cheap, but sequence after the code lands so doc cross-links resolve. |
| 9 | **L11 framekernel (no_std)** | 19/21 | special | highest (structural) | LAST. workspace-EXCLUDED (12-entry exclude). The whole lane hinges on D9 (producer scan respects exclude). Own pinned nightly + build-std. brand-scan the subtree. |

**Riskiest layers:** L6 k8s (95 crates, MERGE-surface), L11 framekernel (exclude-key correctness — a single wrong scan REDs everything), L7 containerd (44 snake_case). The cheapest/highest-value early wins are L3/L2/L4/L5 (8 crates total, all brand-clean, identical to the office pattern).

---

## 6. Prerequisites that gate the WHOLE sequence (from MIGRATION-PLAN-RESYNC §3, re-confirmed live)

The migration is NOT first. Before ANY L2 PR:
1. **GATE-BEFORE-START** kernel re-verify GREEN (per MEMORY, largely done).
2. **Pre-lane 0.6** — run against the REAL merged root workspace with the 12-entry `kernel/` exclude key + founder G4 sign-off. **This is also where D9 (producer-scan-respects-exclude) must be proven.**
3. **Phase-0 firewall go-live** — `oya-ci-required` is the single blocking context (ADR-0515). Already LIVE per this task's premise + the signoff bootstrap entry dated 2026-06-08.
4. **A-lanes land first** — foundry/forgejo/jenkins/oya-vcs eradication (#25) on the consolidated base, so siblings land onto a brand-clean canon. (Siblings L1–L5 are themselves brand-clean, but the canon they land INTO still carries 3184+1188+108+14 baselined brand keys; the A-lanes shrink that — they don't gate the floor for the siblings, but they keep the brand baseline shrinking.)
5. **Branch-reconciliation decision** (cleanup-branch ↔ dev).

---

## 7. Open items / things to VERIFY at execution time (no guessing)

- **D9 producer-scan semantics (L11):** I read that the producer "enumerates first-party `oya-*` crate package names from the tracked Cargo.toml manifests" (bnf lib.rs:9–11) but did NOT read the producer's actual enumeration code (`oya-cloud-ci-accounting-registry-app/src/main.rs` / `lib.rs`) to confirm it filters by `[workspace].members` vs raw `find`. **This must be confirmed before L11** — it is the difference between L11 adding 0 keys and L11 adding ~19 RED keys.
- **Manifest-hygiene per-crate audit:** I confirmed the SHAPE of the violation (the 6 flags) and the L2 source manifest gap, but did not open all 95+44+45 manifests. Each lane's step-3 must template every manifest; the gate will catch any miss.
- **`forge` stem:** MIGRATION-PLAN-RESYNC names `forge` as a FORBID target, but it is NOT in the live `[[vocab.forbidden_stems]]` table. If founder wants `forge` enforced, that is a config edit to `oya-ci.toml` (a separate, signed change) — it is NOT currently a floor gate, so it cannot self-DoS the migration today.
- **All sibling source paths verified present on disk** this session (real `ls`): office(13)/oyago(3)/oyapy(3)/claude(1)/codex(1)/k8s+ctrd(139)/talos(45)/kernel(21). No layer's source-of-truth is missing.
