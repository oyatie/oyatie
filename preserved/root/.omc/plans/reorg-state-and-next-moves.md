# Capability-first reorg — state of play and next moves

Evidence base: `origin/dev` @ `ab75a956b`. All figures re-derivable; commands given.
READ-ONLY assessment. Nothing here has been executed.

## 1. Capability registry vs reality

Registry: **`specs/capability-registry.json`** (`closed: true`, `doctrine_adr: ADR-0562`,
`amendment_adrs: [ADR-0615]`). Not a gate crate — governance DATA read by the lint at
`ci/facade/module-membership/`. Its stated eventual home is `governance/capability-registry.json`.

24 capabilities. **23 of 24 top-level dirs exist**; `policy/` is absent.

| state | dirs |
|---|---|
| exist, populated | cell(8 crates) iam(68) tenancy(22) secrets(10) audit(18) observability(5) data(23) storage(8) compute(8) k8s(18) network(8) gateway(10) messaging(3) intelligence(32) workflow(48) ci(50) iac(5) billing(17) marketplace(5) console(9) compliance(7) comms(24) flags(2) |
| **absent** | **`policy/`** |

**`policy/` gap (unresolved, flag it):** the registry maps `policy` ←
`absorbs_current_dirs: ["policy", "oya/policy"]`. **Neither path exists on `origin/dev`.**
The capability has zero source anywhere. `policy` *is* in the root-hygiene allowlist, so
creating it is legal — but I could not determine what content is meant to fill it.
Likely the Cedar PDP currently living under `iam/`, but that is a guess, not evidence.

Meta directories declared by the registry: `kernel/ os/ base/ governance/ build/ third-party/ app/`.
**Only `kernel/`(3 crates), `governance/`(5), `third-party/` exist. `app/`, `base/`, `os/`, `build/` do not.**

No dir is an empty shell — every existing capability dir has ≥2 crates. Face coverage is uneven:
`gateway/` has only `adapters`+`observability`; `ci/` has no `core/`; `observability/` no `ports/`.
Four dirs carry non-registry faces (`data/ops`, `comms/messenger`, `iac/modules`, `intelligence/testing`).

## 2. Migration completion metric

```
git ls-tree -r -l origin/dev            # size in field 4, path after TAB
# bucket top-level dir: capability(24) | meta(7) | legacy(oya,cloud,libs,tools)
# count paths ending Cargo.toml; sum sizes of paths ending .rs
```

| bucket | crates | % | Rust bytes | % |
|---|---|---|---|---|
| capability dirs | 408 | 44.0 | 22,851,940 | 52.8 |
| meta dirs (kernel/governance/third-party) | 8 | 0.9 | 423,978 | 1.0 |
| **legacy `oya/` `cloud/` `libs/` `tools/`** | **511** | **55.1** | **20,037,163** | **46.3** |
| root workspace `Cargo.toml` | 1 | — | — | — |

**416 of 927 crates migrated (44.9%).** Legacy split: `oya/` 231, `libs/` 189, `cloud/` 62,
`tools/` 29. (`tools/` is a legacy source too — the registry dissolves it into `governance/`+`build/`.)

Remaining 510 legacy crates across **255 second-level dirs**, and — good news — **every one
resolves to a target** under `absorbs_current_dirs` + `membership_lint_coverage`
(`app_products`, `meta_directory_absorbs`, `absorbs_current_crate_globs`). Zero unmapped.

| destination | dirs | crates | Rust bytes |
|---|---|---|---|
| `governance/` (libs/oya-check-*, libs/oya-governance-*, tools/oya-governance-*) | 128 | 128 | 1,864,688 |
| `app/` | 31 | 110 | 4,090,292 |
| `intelligence/` (oya/intelligence remainder) | 1 | 109 | 3,001,020 |
| FROZEN-BASELINE (per-crate, ambiguous) | 60 | 60 | 1,624,896 |
| `os/` (cloud/cloud-os) | 1 | 41 | 5,708,995 |
| `build/` | 22 | 22 | 1,496,457 |
| `kernel/` (cloud/cloud-kernel) | 1 | 20 | 1,452,896 |
| `ci/` | 3 | 12 | 543,470 |
| `data/` | 5 | 5 | 191,398 |
| `messaging/` | 3 | 3 | 29,788 |

## 3. Is there a live move queue? — **No active plan; the ledger and playbook exist.**

- The **applier** exists: `tools/oya-reorg-codemod-app/` (`apply`/`dry-run`/`snapshot`,
  `--revert`, fail-closed; `src/main.rs` header documents the plan schema).
  Self-described "Local bridge tool ONLY — it ships UNUSED until the strangler invokes it."
- There is **no plan generator**. Plans are **hand-authored JSON** committed to `specs/reorg/`.
  Only **4 exist**, all already applied: `messaging-substrate-kernel-move-plan.json` (1 move),
  `iam-pdp-cedar-move-plan.json` (1), `intelligence-move-plan.json` (15 moves/15 artifacts),
  `ci-move-plan.json` (46 moves/1 artifact). Verified applied: `libs/oya-messaging-substrate-kernel`
  and `cloud/cloud-ci` are gone; `messaging/core/substrate-kernel` and `ci/facade/` exist.
- What ADR-0614 de-committed is the **post-apply bijection proof**
  (`specs/reorg/move-manifest.generated.json`, `DEFAULT_MANIFEST_OUT` in `main.rs`), *not* the plans.
- An **ordered ledger of executed moves does exist**: `ADR-0562` §10.4–§10.27, 23 append-only
  records (§10.5 first = messaging; §10.26 = intelligence sub-batch (a); §10.27 = de-brand MOVE-1).
  §10.26 decomposes the intelligence remainder into serial sub-batches **(a)–(g)**; (a) and (b)
  have landed, **(c)–(g) remain** (109 crates + `oya/detection`).
- The **playbook is LOCAL-ONLY** — `.omc/ultragoal/strangler-move-playbook.md` is untracked on
  `origin/dev`. **Lines 105–109 are stale**: they still instruct verifying
  `committed_move_manifest_equals_regenerated` per ADR-0563, which ADR-0614 reversed the same day.
  Anyone following it authors a PR that REDs. Fix before use.
- Regenerate the bijection:
  `buck2 run //tools/oya-reorg-codemod-app:oya-reorg-codemod -- manifest --repo-root <repo> --out specs/reorg/move-manifest.generated.json`
  (`generator_target` in `registry/generated-artifact-control-plane.json`; CI drives it via
  `//ci/facade/generated-artifact-freshness` → `materialize_move_manifest()`).
- **Conclusion: the next move must author a new plan JSON.** `specs/capability-registry.json`
  §`pending_relocations` names 4 future plan files (`storage-`, `observability-`, `compliance-`,
  `comms-move-plan.json`, all "Batch-5") that **do not exist yet**.

## 4. Gates — blocking, but mostly ratchets

All shape gates are crates under `ci/facade/<gate>/`, run as matrix legs of the single required
context **`oya-ci-required`** (`.github/workflows/oya-ci-required.yml` lines 158–195;
`.github/branch-protection.yaml` requires only that context).

| gate | posture | baseline |
|---|---|---|
| `module-membership` (`cloud-ci-capability-membership`) | **ratchet**, born-advisory | **60** crates in `frozen_unmapped_baseline`; flips blocking at 0 |
| `layer-dependency-acyclicity` | **ratchet** (`enforcement: advisory-baseline`) | **8** `TDA-SUBSTRATE-UPWARD` |
| `port-placement` | **ratchet**, born-advisory | **6** |
| `crate-name-prefix` | **ratchet** `baseline-block-on-new`, + de-brand escape hatch (`cargo_prefix_scope:"advisory"` rows never block, `src/lib.rs:135`) | **unknown** (de-committed face, ADR-0616) |
| `crate-layer-suffix` | **ratchet** `baseline-block-on-new` | **unknown** (same face) |
| `repo-root-hygiene` | **BLOCKING**, default-DENY | 49 `allowed_root_dirs` |
| `service-tier-metadata` | **BLOCKING** | 0 |
| `core-dependency-isolation` | **BLOCKING**, `exceptions: []` | 0 |

**It is genuinely ratcheted, not aspirational** — but the ratchet only forbids *new* violations;
nothing forces burn-down. Real remaining shape debt = **74 baselined violations** (60+8+6).
Two baseline counts are unobtainable read-only.

**Blocker for `app/`:** `allowed_root_dirs` in `ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json`
contains `policy` but **does NOT contain `app`, `base`, `os`, or `build`**. Any PR creating `app/`
fails that born-blocking default-DENY gate. `ci/facade/module-membership/capability-membership-policy.json`
*does* allow them (`allowed_top_level_dirs` includes `app`,`base`,`os`,`build`) — so root-hygiene
is the sole blocker.

**Second `app/` defect, easy to miss:** the same membership policy's `scan_roots` lists
29 roots and **omits `app`** (and `base`, `os`, `build`, `third-party`). Crates moved to `app/`
therefore fall out of membership-lint coverage entirely — 110 crates going dark. The
`min_expected_crates: 800` false-green guard would not catch it (927 − 110 = 817). `app` must be
added to `scan_roots` in the same PR that adds it to `allowed_root_dirs`.

## 5. Ordering — leaf-safe vs hubs

Fan-in = number of *other* `Cargo.toml` files mentioning the crate's package name
(derive: `git grep -n '' origin/dev -- '*Cargo.toml'`, extract `[package] name`, count mentions).
Distribution over 510 legacy crates: fan-in 0 → 228, 1–2 → 218, 3–9 → 50, 10–49 → 13, **50+ → 1**.

**Hubs, move last** (fan-in): `libs/oya-data-boundary-kernel` **127**;
`cloud/cloud-os/…/oya-cloud-os-kernel` 40; `libs/oya-shared-postgres-command-kernel` 27;
`oya/intelligence/…/account-domain` 21; `libs/oya-http-router-kernel` 17;
`oya/office/oya-office-kernel` 16; then 15→13: `shared-platform-contracts-kernel`,
`payments/charge-kernel`, `workspace-members-kernel`, `shared-protocol-parity-kernel`,
`http-middleware-kernel`.

**Next 10 moves, cheapest-and-safest first** (all fan-in 0 unless noted):

| # | move | crates | .rs bytes | non-crate co-move |
|---|---|---|---|---|
| 0 | **policy PR:** add `app` (+`base`,`os`,`build`) to `allowed_root_dirs` | 0 | 0 | 0 |
| 1 | `libs/oya-bus-boundary-kernel` → `messaging/` | 1 | 9,928 | ~0 |
| 2 | `libs/oya-queue-boundary-kernel` → `messaging/` | 1 | 9,918 | ~0 |
| 3 | `libs/oya-stream-boundary-kernel` → `messaging/` | 1 | — | ~0 |
| 4 | `tools/oya-governance-doc-status-lifecycle-app` → `governance/` | 1 | 328 | ~0 |
| 5–12 | the other 8 `tools/oya-governance-*-status-lifecycle-app` | 1 ea | 328–474 | ~0 |
| 13 | `oya/workplace-integration` → **`app/`** (first `app/` move) | 1 | 4,191 | **86 files, 7 SLOs** |
| 14 | `oya/docs` → `app/` | 1 | 17,993 | 84 files, 10 SLOs |
| 15 | `oya/slides`,`sheets`,`notes`,`translate`,`sites` → `app/` | 1 ea | 18–36k | 81–103 files, 10–11 SLOs ea |

Steps 1–12 are single-crate, zero-fan-in, near-zero artifact surface — ideal for proving the
codemod pipeline end-to-end before the app products (which carry ~85–120 non-crate files each).
90 legacy dirs total are fan-in-0 and mapped.

Cross-checked (`git grep -l <name> origin/dev -- '*Cargo.toml' | BUCK | '*.rs'`):
`oya-bus-boundary-kernel` → cargo 1 (itself only), BUCK 0, rs 0; `oya-data-boundary-kernel` →
cargo 128, rs 189, 302 files. Ordering holds. BUCK files reference targets by label, not package
name, so BUCK fan-in reads 0 everywhere — that is not "no BUCK impact".

**Step 1 full blast radius** — 6 files: `libs/oya-bus-boundary-kernel/{Cargo.toml,BUCK,src/lib.rs}`,
`Cargo.lock` (regen), `docs/decisions/ADR-0562-*.md`, and `specs/capability-registry.json` (the
`absorbs_current_crate_globs` entry must be retired). That last is what every hand-authored plan
will forget: **the codemod does not move the registry entry.**

## 6. The `app/` question

31 products, 110 crates, 4.09 MB. The roster is machine-readable and closed:
`specs/capability-registry.json` → `membership_lint_coverage.app_products.current_dirs`
(33 entries; 31 have crates — includes `oya/application`). Plus two products to be *assembled*
rather than moved: `app/healthcare` and `app/health-diagnostics`, from `oya/imaging`,
`oya/emergency`, `oya/diagnostics` — all 0-crate scaffolds today (§`pending_relocations`).

**Minimum correct first step is two PRs:**
1. Add `app` to `allowed_root_dirs` in `ci/facade/repo-root-hygiene/root-workspace-hygiene-policy.json`.
   Without this the gate rejects the dir. (Do `base`/`os`/`build` in the same PR — same defect.)
2. `specs/reorg/app-workplace-integration-move-plan.json`: 1 crate move + **86 `artifacts` entries**,
   applied by `oya-reorg-codemod-app apply`. Smallest real app product; creates `app/` for free.

Do **not** lead with `oya/application` (8 crates, 1.37 MB, fan-in 2, 102 non-crate files) even
though ADR-0615 names it — it is the most expensive app move, not the cheapest.

## 7. Biggest risk — the `artifacts` array

The move plan schema separates `moves` (crates, with in-file rewrites) from `artifacts`
(SLOs, catalogs, runbooks — content-preserving co-move). **`artifacts` is optional and defaults
to empty.** That is the defect surface, and it is visible in the landed plans:

- `ci-move-plan.json`: **46 moves, 1 artifact.** `intelligence-move-plan.json`: 15 moves, 15 artifacts.
- Today `ci/` has **0 SLO files**, while 18 other capabilities carry 279 under
  `<cap>/observability/slos/` (data 60, comms 48, storage 36, workflow 35, marketplace 34…).
  **I could NOT determine whether this is a move drop.** `origin/dev` is only **46 commits deep**
  (root commit `33134e055`, PR #1324) — all pre-#1324 history is unreachable locally, so
  `cloud/cloud-ci`'s prior SLO state is unverifiable from the tree. Do not bill it either way
  without checking the GitHub PR record.

This class already bit. *(PR numbers below came via `gh` from a sub-agent, single-sourced and not
hand-re-verified; the `origin/dev` tree facts are verified.)*
- **#735–#746** ran a crate-only codemod: **85 promotion-gating `slos/*.openslo.yaml` orphaned**
  across 8 capabilities. #747 added the `ArtifactMove` co-move; #748 backfilled. ADR-0562 §10.14
  is labelled "first doctrine-clean SLO co-move" — **moves 1–9 all shipped without it.**
- Catalog fallout: **#749** re-keyed 48 / deleted 7; **#751** removed 114 more dead records;
  **#752** built a born-blocking `catalog-liveness` gate. **#750** was a fix-of-a-fix.
- **Stale path refs survive the codemod and are live on `origin/dev` today** — it rewrites
  Cargo/BUCK/Rust idents but not config or doc path strings. Still pointing at the deleted
  `cloud/cloud-ci/gates/…`: `oya-deps.toml:24` (`drift_guard`, read by
  `ci/facade/dependency-automation`), **`cloud/cloud-iac/manifest.json` — 24 lines**,
  `.github/CONTRIBUTING.md:60`, `infra/ci/buckconfig/warm-cache-rw.buckconfig:6`,
  `toolchains/cache/defs.bzl:14`. #1223 fixed two at **push tier**, after `dev` went red.
- **#1337** found the codemod also drops **ADR doc-anchor path citations** — live
  `justification_ref` anchors. Its first fix was itself rejected on cross-model review:
  non-idempotent rewrite, missing left-boundary guard (`za/b` corrupted rewriting `a/b`), fail-open
  `read_dir`. Two of its three "fixes" were rejected as *weakening* merge authority.
- **Rename detection is unreliable here** (verified): #1335 produced 65 renames but **11 unpaired
  A + 9 unpaired D**, concentrated on `Cargo.toml`/`BUCK` whose content the move rewrites past the
  similarity threshold. Use ADR-0563's relabel engine, not `--find-renames`.

The app products are where the `artifacts` gap bites next: 1–3 crates but **81–121 non-crate
files and 5–13 SLO files each**. A plan that omits `artifacts` silently orphans them.

Mitigation: for every plan, assert `len(artifacts) == (files under old_path) − (.rs + Cargo.toml
+ BUCK + OWNERS)` before applying; grep the whole tree for the old path string after applying
(the codemod will not have rewritten config/doc occurrences); and update the
`specs/capability-registry.json` mapping entry by hand.

## Known gaps — not determined

- What content `policy/` is supposed to hold. No source dir exists anywhere.
- Baseline counts for `crate-name-prefix` and `crate-layer-suffix` (de-committed face, ADR-0616;
  requires running the accounting producer).
- Whether `ci/`'s zero SLO coverage is a move drop or pre-existing — history truncated at 46 commits.
- Whether `oya-deps.toml`'s `drift_guard` key is actually dereferenced at runtime (file is read by
  two `ci/facade` crates; the specific key was not traced).
- Whether live GitHub branch protection matches `.github/branch-protection.yaml`.
- No gate verifies plan `artifacts` completeness. I found no such check.
- No BUCK-breakage defect was found for the early moves, but truncated history means it cannot
  be ruled out.
- Ordering here is by fan-in and artifact surface only. Nothing was built or tested — buck2 was
  not run.
