# REORG PROGRAM — drive to completion, no deferrals

**Founder directive 2026-07-30:** finish the capability-first reorg to the northstar hyperscaler
monorepo shape, de-brand where applicable, **defer nothing**, end on a **clean slate**. This
REVERSES the 2026-07-26 freeze at 45%. The freeze memory's own exit condition was met (all four
sanctioned enforcement items shipped or dropped on evidence), so this resumes migration, not
machinery.

This file is the durable spine. It outlives any session. Update it as waves land.

---

## 0. The northstar shape (ADR-0562 §1, as amended by ADR-0615)

```
kernel/        rung 0: kuberos no_std kernel + sysroot (own excluded workspace, ADR-0512 carve-out)
os/            rung 1: cloud-os node OS (Talos-class)
base/          Google //base: ADMISSION-GATED — >=3 capability consumers AND strictly below all
               of them in the ADR-0280 DAG. NOT a util/ junk-drawer.
governance/    meta, off the runtime ladder: ADRs, specs, policy-as-data, registry, masterplan
build/         meta, off the ladder: buck2 prelude, toolchains, reindeer, CI engines. Zero crates.
third-party/   meta, off the ladder: reindeer-vendored sources. TOP-LEVEL per ADR-0615.
<capability>/  THE PRIMARY AXIS — path = namespace = buck2 label root
  core/          the engine we RUN (substrate face)
  ports/         capability traits (the stable seam)
  adapters/      transient-infra impls; vanish at owned-stack cutover
  facade/        the multi-tenant surface we SELL — reaches core/ ONLY through ports/
app/<product>/ composition ring: wires 2+ capabilities for a tenant
```

Precedent cited by the ADR: Google `//base //net //storage //compute`, Meta fbcode, Azure
`sdk/<service>/`, AWS service-as-boundary. All root the tree by WHAT a system is. Tier-first
(`substrate/ product/ service-cell/`) was considered and REJECTED.

**De-brand is part of the shape:** no `oya-`/`oya_` package prefixes, no path doubling
(`cloud/cloud-*`). Path is the namespace, so the brand in the leaf is redundant.

---

## 1. Measured scope (origin/dev @96da99d14, 2026-07-30 — re-measure, don't trust)

| root | crates | note |
|---|---|---|
| `oya/` | 190 | was 210; payments −20 landed (PR #1451) |
| `libs/` | 185 | only **14** clear the base/ >=3-consumer bar; 64 have zero dependents |
| `tools/` | 30 | mix of build/ meta, capability-owned, and the 17 unwired gate apps |
| `cloud/` | 21 | path doubling `cloud/cloud-*` — de-brand target |
| **total legacy** | **426** | of 918 packages |

**431 of 918 packages carry an `oya-` / `oya_` prefix** (oya 210, libs 185, tools 29, cloud 7).

**The crates are ~12% of the work.** `oya/` alone is 8306 tracked files; only 955 are crate
layouts. The rest: `iac` 1692, `catalog` 1619, `runbooks` 974, `policy` 487, `slos` 423,
`capabilities` 378, `dashboards` 315, `contracts` 307. **Any plan sized on crate count
under-scopes by ~8x.** The satellite moves with its capability — it is not out of scope.

`app/`, `base/`, `policy/`, `governance/` (as a code root) DO NOT YET EXIST on dev.

---

## 2. Preconditions that genuinely BLOCK moves

1. **Registry amendment.** `specs/capability-registry.json` is `closed: true`. `infra/` (17
   subdirs) is in neither `capabilities` nor `meta_directories`, and `oya/application` (8 crates)
   + `oya/workplace-integration` (1) are named nowhere. Nothing can move into or out of an
   unregistered root while the registry is closed. Also repair the **11 dead**
   `absorbs_current_dirs` prefixes (86 live of 97 — the widely-cited "69 of 73 dead" is FALSE).
2. **Four scan-root-class arrays** still exclude capability/`app` roots, so a moved crate silently
   exits enforcement (the `app` gap in both authz gates is ALREADY FIXED — do not re-fix):
   - `service-tier-metadata/tier-field-coverage-policy.json` `.governed_service_roots` = `[cloud,oya]`
     with `min_expected_service_manifests: 95` (90 service manifests exist: 69 oya + 21 cloud) —
     vacating `oya/` drops it to 21 and the gate goes RED. Blocking, not a false green.
   - `layer-dependency-acyclicity/...policy.json` `.service_roots` = `[cloud,oya]`; its
     `.crate_root_globs` (30) lack `app/*/*`, `base/*/*`, `kernel/*/*`, `third-party`.
   - `automation-language-policy/rust-first-automation-policy.json`
     `.scan.cli_package_authority.roots` = `[cloud,os,infra,tools]` — `oya/` unscanned for new
     `-cli` packages.
   - `module-membership/capability-membership-policy.json` `.scan_roots`(33) + `.meta_directories`(6)
     lack `third-party` (ADR-0615 promoted it).
3. **138 path-keyed occurrences of `oya/...` across 14 gate JSONs** that the codemod does NOT
   rewrite: `cedar-deploy-parity-policy.json` 67 (fail-closed on missing authored Cedar),
   `dto-authz-trust-policy.json` 41 (keys are `path#fn:hash`), `tier-dependency-acyclicity-baseline`
   8, `contract-slice-policy.json` 7 (+4 slice files), `port-placement-baseline` 2,
   `authz-coverage-policy` 2, `operator-secret-bootstrap-policy` 1.
4. **Workspace globs.** Root `Cargo.toml` has exactly two `oya` globs (`oya/*/crates/oya-*`,
   `oya/office/oya-*`). Each new capability destination needs its own `<cap>/*/*` line. A missing
   glob **silently drops the crate from the workspace while census gates stay green** — this
   already happened on the first de-brand move.
5. **Born-accounting (ADR-0555)** for every new artifact: justification naming the EXACT
   repo-relative path in a decision record, an `OWNERS` file, and a
   `specs/reachability-registry.json` prefix. Measured: one new file cost all three, and the
   `OWNERS` was invisible to the producer until `git add`.

---

## 3. Mechanics that constrain execution (not preferences)

- **USE THE CODEMOD.** `oya-reorg-codemod-app` + a move-plan under `specs/reorg/`. Hand-rolled
  moves survived **1-of-8** then **0-of-7** adversarial review. The codemod rewrites Cargo.toml,
  BUCK, `.rs` idents, ADR anchors, workspace members and `Cargo.lock`; it does NOT touch the
  14 gate JSONs in §2.3 — those are hand-carried per wave.
- **MOVES ARE STRICTLY SERIAL BY MECHANISM.** `manifest.rs::select_active_move_plan` hard-errors
  `MultipleMovePlans` on more than one non-landed plan, and that path runs inside the required
  gate. At most ONE plan may be in flight.
- **PR shape (Bun).** A migration with an intermediate state ships as ONE PR; independent moves
  split, serial one-move-per-PR. Never `git stash`/`reset`; `git mv` so renames are detected.
- **Oracle.** Policy-only waves: run the gate on merge-base and candidate, assert identical
  finding sets and unchanged `min_expected_*` floors. Move waves: `buck2 test //...` verdicts
  identical before/after — now actually runnable, since the owned Talos fleet executes 44 gate
  legs (it executed **zero** for the ~24h before 2026-07-30).
- **facade-core-layering** is keyed by cargo PACKAGE NAME (not path), so moves do not invalidate
  its 35-frozen baseline. BUT landing crates into `<cap>/facade/` + `<cap>/core/` without a
  `ports/` seam creates a NEW violation and blocks. Founder ruling: **do not defer those crates —
  give them the ports seam as part of the move (REFACTOR, not MOVE).**

---

## 4. Disposition doctrine

Quadrichotomy **MOVE | REFACTOR | REWRITE | DELETE**, founder-stated repeatedly: *"it's not a
simple move."* Independent re-classification measured **27% disagreement** (31% on MOVE rows), so
every non-obvious row needs an adversarial second opinion, not a single pass.

**Two probes that are INVALID — both produced wrong answers this session:**
- Counting `fn` to detect a husk. A `-domain`/`-kernel` crate is types and traits; zero `fn` is
  its expected shape. `oya-payments-charge-kernel` reads as a 47-line empty husk and carries 30
  dependency edges.
- Reverse-dependency count to detect an orphaned **binary**. Nothing depends on an entrypoint.
  The right axis for an `-app` is **invocation** (workflow / Makefile / BUCK test / CI config).

**The valid husk test is CLUSTER CLOSURE:** count consumers OUTSIDE the candidate subtree. The
payments deletion was justified because external consumers = 0 while internal edges = many.

---

## 5. Wave order (no deferrals; each wave leaves the tree green)

| # | wave | why here |
|---|---|---|
| 0 | **DONE** — payments closed cluster deleted (PR #1451): 20 crates, 20 catalog rows, 20 lock entries; 102 spec files kept | verified closed |
| 1 | **STOP ACCRUAL** — freeze the legacy-root crate census shrink-only in module-membership | delivers the founder's stated *reason*; no new crate can be born in a legacy root |
| 2 | **Kill the scan-root drift class** — derive `crate_root_globs` instead of hand-maintaining 30; fix the four §2.2 arrays | provably zero-delta only while destinations are empty — this is the one moment |
| 3 | **Registry amendment** — admit `infra/`, `app/`, `base/`; repair the 11 dead prefixes | closed registry blocks every later wave |
| 4 | **Wire the 17 unwired gate apps** (see §6) | they are enforcement built and switched off; wiring before moving means the moves are policed |
| 5 | `oya/intelligence` → `intelligence/{core,ports,adapters}` (88 crates, 42% of oya/) | largest single coherent block |
| 6 | Birth `app/` + land the product surfaces (35 named in `app_products_note`) | composition ring |
| 7 | `oya/` remainder + `cloud/` de-doubling (`cloud/cloud-*` → `cloud/*`) | |
| 8 | `libs/` 185: 14 → `base/` (admission-tested), 171 → owning capability | hardest disposition |
| 9 | `tools/` 30: build/ meta vs capability vs delete | |
| 8b | **`infra/` distribution** (founder 2026-07-31: in scope) — 17 subdirs to registered roots: gitops/talos/capi/sidero-metal → `iac/`; arc/ci/ci-webhook-gateway/nativelink → `ci/`; external-secrets/kms → `secrets/`; registry/seaweedfs → `storage/`; observability → `observability/`; cilium/cloudflare → `network/`; kyverno → `policy/` | carries the 35-ADR path-amendment tail (14 of 17 dirs are named by path in ADRs) |
| 8c | **`specs/` → `governance/`** (founder 2026-07-31: in scope) — ADR-0562 §114 already declares this: the registry's *"eventual home is `governance/capability-registry.json` after the reorg; held at `specs/` until the `governance/` top-level dir exists"* | `governance/` is already a registered meta_dir; this is executing a decision, not making one |
| 10 | **De-brand sweep**: 431 `oya-`/`oya_` package prefixes → bare names | LAST and STOP-THE-WORLD, incl. `oya-ci-required` → `ci-required` + `ci-advisory` (renames the single required protected context = merge authority) |

---

## 6. The 17 unwired gate apps (founder: WIRE them, don't delete)

45 governance crates have zero reverse-deps. **17 are `-app` binaries in `tools/` and are NEVER
INVOKED** — not by the required workflow, Makefile, BUCK test, or any CI config. They are
enforcement that was built and switched off. All wrap the LIVE
`libs/oya-governance-lifecycle-kernel` (9 dependents) and read a config from
`specs/lifecycle-configs/`, all 9 of which exist.

Wire highest-value first: **`adr-status-lifecycle`**. Its lane config declares stages
`proposed|accepted|superseded|archived` while the canonical
`oya-governance-adr-shape-kernel::VALID_STATUSES` is
`Proposed|Accepted|Amended|Superseded|Deprecated|Rejected` — a **THIRD** ADR status vocabulary
(dev-cli's `ADR_STATUSES` was the second; it omitted `Amended`+`Rejected` and carried a phantom
`Retracted`, fixed in PR #1450). Reconcile the vocabulary FIRST, then wire — otherwise wiring
fails instantly against the corpus.

Corpus evidence the lane would catch: 440 ADRs containing `OK`, `completed-locally`,
`Proposed | Soaking | Active | Sunset`, 34 lowercase `accepted`, 31 lowercase `proposed`.

Runtime note: the app refuses a default clock —
`--trusted-now YYYY-MM-DD` is REQUIRED ("refuses stale default or environment-supplied clocks").

The other 28 orphans are libs with real implementations (300–1254 LOC). Each needs per-crate
**supersession proof** against a live `ci/facade/*` gate — a name-similarity match is NOT proof.

---

## 7. Open founder rulings

- **Drive model: `storage/facade/drive` SURVIVES** (ruled 2026-07-30). `oya/office`'s independent
  `oya-office-*` Drive equivalent converges onto it.
- **facade crates: do NOT defer** (ruled 2026-07-30) — refactor to add the `ports/` seam.
- PG 16 → 18.4 for the CNPG rewire of the two live-Postgres gates: pin a 16 cluster, or accept 18
  and validate the four migrations as part of the work.

---

## 8. Pipeline friction resolution (founder 2026-07-31)

**The bar for everything in the pipeline: global · canonical · portable · productized ·
comprehensive.** The pipeline IS a product; a fix that only works on this repo is not a fix.

**Measured friction:** `gate · affected-set` cost **25m42s** on an infra PR. Cause is NOT
"workflows are unmapped" — `.github/**` is well mapped to 21 gate targets. Cause is that
`infra/**` and `specs/*.json` appear in NO `synthetic_dependencies` row, and ADR-0554 round-6
defect 2 escalates any unmapped path to FULL (correctly, fail-closed).

**Do NOT hand-add rows for `infra/**` or `specs/**`.** Founder caught this: both are reorg targets
(neither is a capability nor a meta-dir in the CLOSED registry), so a hand-written row is a row
that must be rewritten mid-program — migration debt, the exact thing this program exists to stop.

**Resolution — derive, don't enumerate.** Make input-declaration a repo-agnostic contract: every
tool that consumes repo content declares its input classes, and the engine seeds from those
declarations plus the build graph's own `owner()`. `synthetic_dependencies` becomes a *derived
projection*, which survives `infra/` → `iac/` and `specs/` → `governance/` with no edit.
(Deriving from this repo's `ci/facade/*` `scan_roots` spelling would fail *portable*.)

**Guards: data-driven, not flag-driven.** The inline-shell ratchet is shrink-only with
born-blocking growth, so fixing a BROKEN step (the unauthenticated `git fetch` on a PRIVATE repo)
was blocked at "line count grew from 1 to 2" and only landed after being golfed to one line. The
gate cannot tell a safety assertion from a risk. Founder ruling: **fund the productized path** —
a preflight product evaluating declared preconditions as DATA, following the existing
`oya-cloud-ci-runner-disk-reclaim-bin` shape (FRIC-017 pipeline-glue(b)). Adding a guard becomes a
row, not a flag and not a shell line. A second preflight mechanism would fail *canonical*.

**ADR verdict:** no ADR blocks best practice. ADR-0554's fail-closed rule is correct; only its data
is incomplete. The one real tension was the shell ratchet's incentive, and the policy already names
the sanctioned escape (productize to Rust) — it was simply unfunded.

One-pager: `docs/ideas/pipeline-friction-derived-seeds-and-productized-guards.md` (on the
ci/flip branch).


## 9. Wave 1 STATUS (workflow wf_fd35dc28-796, 2026-07-31)

**W1 stop-accrual is BUILT and both oracle directions are PROVEN.** Branch `reorg/w1-stop-accrual`,
commit `e77179eb8`, worktree `scratchpad/wt-w1`. NOT pushed, NOT a PR yet.

Design choice worth keeping: it **extends `ci/facade/module-membership`**, which is ALREADY a
required `oya-ci-required` matrix leg — rather than adding a sibling gate needing workflow wiring +
gate_registration + catalog row + OWNERS + reachability. That recipe is exactly what produced the
repo's 17 built-but-never-invoked gate apps.

- Census is **producer-emitted** (`--emit-legacy-freeze`), never hand-typed.
- The producer **REFUSES to grow** the census without `--allow-new` (exit 2, naming each grown dir).
  This closes the obvious laundering reflex — "the gate went red, regenerate the baseline" — which
  would otherwise defeat the entire wave.
- **RED direction proven**: synthesized a crate under all four legacy roots -> 4x
  `MEM-NEW-LEGACY-ROOT-CRATE`, gate leg FAILED, producer exit 2.
- **THE MEASURED ACCRUAL HOLE**: `oya/identity/crates/accrual-probe` and
  `cloud/cloud-iam/crates/accrual-probe` emitted NO `MEM-NEW-UNMAPPED-CRATE` — they map cleanly to
  registered capabilities, so the pre-existing gate was GREEN on them. Only the freeze catches
  them. That is the hole, closed and demonstrated.

**BLOCKER BEFORE LANDING: the census is STALE.** It was emitted at `96da99d14`, which still
contained `oya/payments` — PR #1451 has since merged and deleted those 20 crates. Re-run
`--emit-legacy-freeze` against current dev and commit the shrunken census (shrink is always
allowed; growth is not). Verified counts on dev today: oya 190, libs 185, tools 30, cloud 21 =
**426** (the workflow's 445 was correct for its own base).


## 10. Waves 1-3 FULL STATUS + findings (wf_fd35dc28-796) — READ BEFORE RESUMING

| wave | branch @ commit | review verdict |
|---|---|---|
| W1 stop-accrual | `reorg/w1-stop-accrual` @ `e77179eb8` | **LANDABLE** (3 findings) — needs census regen, see §9 |
| W2 scan-root derivation | `reorg/w2-scan-root-derivation` @ `8b19f63` | **NEEDS_REWORK** (4 findings) |
| W3 registry amendment | `reorg/w3-registry-amendment` @ `4efaa437f` | **NEEDS_REWORK** (4 findings) |

All three are UNPUSHED worktree branches. W3 minted **ADR-0631**.

### W2 blockers worth keeping (do NOT rediscover)

- **`rust-first-automation` `scan.cli_package_authority.roots` CANNOT be derived in W2.** Three roots
  already hold `-cli` packages, so admitting them turns a born-blocking dimension RED:
  `marketplace/facade/dev-cli` -> `marketplace-dev-cli`, `tenancy/ports/cli` -> `tenancy-cli`,
  `oya/intelligence/crates/oya-codeview-cli`. **Both escape hatches are correctly forbidden** and the
  agent backed out of each after trying: (a) `exclude_prefixes` FAILS
  `live_scan_scope_does_not_narrow_the_immutable_merge_base_configuration` (shrink-only ceiling);
  (b) recording the misses in `gate-coverage-baseline.json` breaks
  `gate_coverage_baseline_is_born_empty_and_wellformed`. **Admission belongs to the W10
  CLI-retirement/de-brand wave** that renames those packages. The array is already shrink-PROOF.
- `libs/` likewise NOT added (4 more `-cli`: `oya-shared-{architecture,bounded-contexts,semver,supply-chain}-check-cli`); `libs/` dissolves in W8 anyway.
- **SATELLITE DRIFT — this will bite W5/W7.** The already-migrated capability roots (iam 68 crates,
  intelligence 43, workflow 48, ...) carry **ZERO `manifest.json`**. The crates moved; the service
  manifest, `slos/` and `catalog/` did NOT. So widening `governed_service_roots` is necessary but
  NOT sufficient — the satellite must actually move with the capability, or those services silently
  have no tier manifest at all. This is the §1 "crates are only 12% of the work" claim, confirmed
  from the other direction.

### W3 findings worth keeping

- **The dead-prefix repair has NO DETECTOR and can rot again.** `ci/facade/scan-root-liveness`'s
  collector walks `ci/facade/*/*.json` ONLY (`tests/scan_root_liveness.rs:199-204`), so
  `specs/capability-registry.json` is structurally outside its universe, and `absorbs_current_dirs`
  is not a `coverage_bearing_key`. Fixing needs BOTH a data edit AND a collector change. Recurrence
  prevention -> belongs with W2, recorded as a Known residual in ADR-0631.
- `module-membership`'s `non_crate_top_level_dirs` (15 entries) is still hand-maintained: `infra` is
  now registry-authorized but the other 14 (bin, benchmarks, contracts, docs, evidence, packs, plan,
  registry, scripts, specs, tasks, templates, third-party, toolchains) are not. W2-class work.
- **infra/'s 16 subdirs are NOT dispositioned — deliberately, and recorded in-registry not dropped.**
  Measured cost: **26 ADR files carry verbatim `infra/<subdir>` path anchors, 10 of them
  Accepted-class**. **`cilium` and `observability` carry ZERO anchors and are the CHEAPEST FIRST
  MOVERS.** Start the infra/ distribution there.
- Local gate lanes need the scm-facts materializer pre-step or 8 tests fail spuriously.

## 11. cloud-ci-firewall — rename + de-brand, KEEP in place (founder flagged 2026-07-31)

It has THREE names for one thing: check `cloud-ci-firewall`, crate `ci/facade/baseline-ratchet`,
test file `tests/firewall.rs`, and target `oya-cloud-ci-firewall-signoff-fixer-unittest`.

- **"firewall" mis-describes it** — it is a merge-base BASELINE RATCHET (blocks new debt, grandfathers
  existing), not a network control. The failing test is `firewall_is_green_on_the_live_corpus_with_the_baseline`.
- **De-brand**: `oya-cloud-ci-firewall-signoff-fixer` carries BOTH the `oya-` prefix AND `cloud-ci-`
  doubling, inside a crate already under `ci/`. Path is namespace -> `ci/facade/baseline-ratchet:signoff-fixer`.
- **Location is CORRECT** — `ci` is a registered capability, `facade/` a valid face. Rename != move.
- **Do NOT remove.** It is one of the few controls that IS wired and IS enforcing; it caught the
  born-accounting violation on a new file this session.
- **SEQUENCING**: the check name is a REQUIRED PROTECTED CONTEXT, same class as `oya-ci-required`.
  Renaming retitles a branch-protection requirement -> belongs in the STOP-THE-WORLD W10 de-brand
  wave, never as a drive-by, or merges block on a context name that no longer exists.

## 12. Session 2026-07-31 (post-compact) — CI unblocked, W1 landed, infra/ interrogated

### Merged this session
- **#1450** required lane → `oya-arm64` + pwsh baked. **#1451** payments cluster deleted.
- **#1452** every `upload-artifact` bounded with `retention-days`.
- **#1453** `check-substrates` → `oya-arm64` (it was the 3rd hosted job, red-with-no-verdict for days).

### Open PRs
- **#1454** W1 STOP ACCRUAL — census regen, 425 crates. *(see corrections below)*
- **#1455** gitops ARC `valuesFile:` → `valueFiles:`.

### CI unblock — root cause was NOT a gate
`producer-regen` step 5 `Upload regenerated faces` hit `Artifact storage quota has been hit`.
All 44 gate legs + `affected-set` + `buck2` declare `needs: producer-regen`, so they were
**skipped**. The required context went red with **zero gates evaluated**.

Mechanics that cost wrong conclusions first — see memory `ci-quota-exhaustion-blanks-all-gates`:
- Deleting the bytes does **not** restore capacity; GitHub recalculates usage on a **6–12 h**
  cycle. Measured 2.34 GB → 0.23 GB and the upload still failed 20 min later.
- `size_in_bytes` is unstable while a delete drains — three probes disagreed. **Ground truth is
  whether the upload step succeeds**, not any API sum.
- `DELETE /actions/caches?key=` with a TRUNCATED key returns 200 + a result-shaped body.
  Both the delete and the usage endpoint lag, in opposite directions.

**Freed 6.92 GB of Actions cache**: one entry keyed `buck-out-Linux-<hash>` — the pre-#1450 key
shape with **no arch segment**. Live key is `buck-out-${{ runner.os }}-${{ runner.arch }}-…`, so it
could never be hit again while squatting 69% of the 10 GB limit against a ~5.78 GiB buck-out.
**Arch-qualifying a cache key orphans its old entries; they need an explicit reap.**

### W1 — two corrections found while re-deriving
1. Census was stale (emitted at `96da99d1`, before #1451). Re-emitted at HEAD: **445 → 425**,
   20 payments names burned down. Had it landed unregenerated it would have fired
   `MEM-STALE-LEGACY-ROOT-BASELINE` **on its own freeze commit**.
2. **`_provenance.crates_total` REMOVED.** It made the block assert its own contradiction:
   `frozen_at` stayed pinned to the freeze commit while `crates_total` tracked the CURRENT tree,
   so the pair read "at commit 96da99d1 there were 425 crates" when there were 445. Nothing read
   it; it duplicates `crates.len()`. Deleted, not repaired. `_provenance` is now carried forward
   untouched — the fixed-anchor shape every sibling policy's `frozen_at`/`frozen_at_ref` uses.

**Census is 425, NOT 426** — `oya` 190, `libs` 185, `tools` 30, `cloud` **20**.
`cloud/cloud-kernel/Cargo.toml` is `[workspace]`-only with no `[package]`, so it is not a crate.
An earlier note recording 426/cloud-21 counted Cargo.toml FILES and caught a nested workspace root.
Verified idempotent: re-running the producer reports `0 burned down`.

### infra/ is not mis-located — it is UNTRUE
Founder reminder 2026-07-31: **`microservices/*`, `oya-*`, `oya/*`, `cloud/*`, `cloud-*`, `infra/`
are ALL deprecating / reorg targets.** So nothing may be repointed *into* `infra/`.

Measured against the live cluster and the tree:
- **`infra/cilium/`** — 5 files, **zero Cilium resources**; all plain `networking.k8s.io/v1
  NetworkPolicy`. Live cluster runs **`kube-flannel`**, **0 Cilium CRDs**. Flannel does not
  implement NetworkPolicy, so all 5 "cell boundary" controls enforce **nothing**.
  `infra/gitops/values.yaml:14` declares a Cilium chart that was never applied.
- **`infra/observability/`** — manifest is real (VictoriaMetrics + OTel + Grafana, ns
  `oya-observability`). Its GitOps app is broken 3 ways: `path:` → **0 files**, declares ns
  `observability` (mismatch), and the cluster has **no observability namespace at all**.

**FOUNDER RULINGS 2026-07-31:**
- **cilium → "Keep intent, make it enforceable."** Cell-boundary intent IS northstar and the
  nativelink CAS netpol depends on the pattern. Install a NetworkPolicy-enforcing CNI, de-brand
  the dir off the vendor name, add a detector that fails when a declared netpol cannot be enforced.
- **gitops → "Build the gate first, let it tell us."** Do not hand-fix declarations one at a time.

### The gate to build (next substantial unit)
**`ci/facade/gitops-declaration-integrity/`** — every GitOps app declaration must resolve:
`path:` exists, and values keys are ones the template actually reads.

Full sweep of all 18 apps found **4 defects**:

| app | defect |
|---|---|
| `arc` | `valuesFile:` — key template never reads *(fixed #1455)* |
| `oya-arm64` | `valuesFile:` — same *(fixed #1455)* |
| `cloud-intelligence` | `path: microservices/cloud-intelligence/k8s` → **0 files** |
| `observability` | `path: microservices/observability/iac/k8s/helm` → **0 files** |

The two remaining need a **northstar destination**, not a repoint into another deprecating root.

**Registration is part of done.** `ci/facade/baseline-ratchet/tests/gate_registration.rs`
`gate_crate_dirs()` enumerates EVERY directory under `ci/facade/`, so a new gate is automatically
required to register in the workflow, branch-protection, agent contract, PR template and oya-ci
config. That mechanism is what prevents another unwired gate — budget for it.
Structural template: `ci/facade/hook-wiring/` (BUCK + Cargo.toml + src/lib.rs + tests/). `ci/OWNERS`
covers the tree, so a per-gate OWNERS is optional.

### Verification lesson that repeated
The `valuesFile` defect was in `infra/gitops/values.yaml`, wired to the fleet **I** declared in
ADR-0630 — whose D6 claims "verified field-for-field against the live CR at zero drift." That check
compared the VALUES FILE to the LIVE CR and never checked that the app consumes the values file.
**Comparing two things that agree proves nothing about a third that reads neither.** Proof came
from `helm template` before/after, not from reading the key name.

### CI artifact policy — CLOSED as a class (2026-07-31)
Comprehensive sweep: **10 distinct artifact families, all with live producers, ZERO dead.**
Reaped in total: `oya-pr-review-rollup` (1397 + **219 the out-of-band reap MISSED**),
`accounting-faces` (656), `oya-vcs-provider-execution-proof` (**977**, from the ADR-0363-retired
VCS platform). Do not trust a reap's own claim — the first one reported 1397 rollups deleted and
219 survived; verify with a producer sweep, not the actor's report.

Remaining volume is now entirely structural: 5735 artifacts across the five `if: always()`
families, which **PR #1456** converts to `if: failure()`.
Taxonomy (hyperscaler/SLSA): consumed-in-run → unconditional; ratchet anchor → long retention;
failure triage → `if: failure()`; recurring number → metrics substrate, NOT artifact storage.
Exception kept: `affected-set-operator-artifacts` stays `always()` — an EMPTY affected set is a
FALSE GREEN, invisible to a `failure()` upload because the run did not fail.

**Cache 0% = same root cause.** buck-out has ONE writer (push-to-dev, inside
`gate-affected-target-set`, which `needs: producer-regen`). Quota broke producer-regen → sole
writer skipped → nothing saved; #1450's arch-qualified key invalidated the one old entry.
Expect the first green dev push to be slow (~55-60 min cold full-graph) to repopulate.

### Open PRs at session end
#1454 W1 STOP ACCRUAL (census 425) · #1455 gitops valuesFile → valueFiles · #1456 artifact policy.
All three verified locally; all three blocked on the GitHub quota recalculation, not on their diffs.

## 13. buck2 RE / cache — ROOT-CAUSED 2026-07-31 (read before touching cache work)

**buck2 DOES support REAPI cache-only (CAS+AC, NO scheduler).** Explicit match arm in the executor
builder. Verified against buck2 source at commit 1560aca2 — the byte-exact commit of our installed
binary. The scheduler/worker tiers of ADR-0612 are NOT needed for the cache win; conflating cache
with RE is why this never started.

**Three independent defects, any ONE of which alone yields the measured zero-upload symptom:**
1. `remote_enabled` / `remote_cache_enabled` / `allow_cache_uploads` are **NOT `.buckconfig` keys**.
   They are `CommandExecutorConfig()` Starlark params on the EXECUTION PLATFORM. `[buck2_re_client]`
   configures only the client CONNECTION. **Two disjoint config planes** — setting either alone is
   inert. This single misunderstanding explains every previous dead end.
2. `prelude//platforms:default` hardcodes `remote_enabled = False`; in OSS `remote_cache_enabled`
   DEFAULTS to `remote_enabled` -> False -> `Executor::Local` -> hardwired `NoOpCacheUploader`,
   behavior DISCARDED. Zero uploads, no error, no warning. Fix: own exec platform with
   `remote_cache_enabled = True` explicit.
3. `[buck2_re_client]` is absent from `DaemonStartupConfig`, the ONLY thing the daemon-reuse check
   compares. Editing `.buckconfig.local` with a live daemon does NOT restart it — it serves stale
   empty RE config forever. **`buck2 killall` is mandatory after any RE config change**; any
   measurement taken without it is invalid.

Corrected: nativelink DOES expose `capabilities` on both listeners; buck2's OSS client reads only
`cache_capabilities`, never `execution_capabilities` — so scheduler-less satisfies the connect path.
Point `engine_address` at the writer listener. Also: address scheme must be grpc://; http(s):// is
hard-rejected. `tls_client_cert` must be ONE PEM holding BOTH chain and key. No SNI override key —
the address host must match a SAN on the server cert.

**ADR corrections:** ADR-0612 (Proposed, 564 lines) IS the RE-phase ADR — RE needs it IMPLEMENTED,
not a new ADR; zero of its 5 named artifacts exist on dev. ADR-0525 D3 says NOTHING about a 3-tier
split (0 hits for tier/scheduler/worker) — that is a founder decision in
docs/ideas/nativelink-remote-cache-first.md, and ADR-0556 D3 is what splits cache-only from RE.

**Governance blockers (each independently fatal):** `root_buckconfig_stays_dark` is a LIVE required
matrix leg; `buckconfig_local_is_ignored_and_untracked` bans the only file that can wire RE;
`remote_enabled = False` is a hardcoded literal in `toolchains/cache/defs.bzl` rule impl (not an
attr) so enabling is a Starlark edit.

**RESOLVED this week, unnoticed:** ADR-0612 OQ1 ratified "there is no RE phase until the runner can
route to the cluster." The oya-arm64 Talos fleet (#1450) resolves *.svc.cluster.local natively.

### The canary never RAN (corrects an agent finding)
An agent reported the cold canary "fails at the cold build step". FALSE — the job has **steps=0**
across five consecutive runs: it failed ADMISSION on `runs-on: ubuntu-latest`, never entering its
body. "Our cache integrity is broken" vs "our cache integrity is UNMEASURED" are different claims;
only the second is true. PR #1457 flips it plus docs-graph-drift (found only by sweeping every
`runs-on` in every workflow, not by triage).

### Two defects recorded, deliberately not fixed
- **De-brand**: `oya-arm64` is the ARC `runnerScaleSetName` and therefore the runner label; 11
  hardcoded refs on dev. Belongs in the de-brand move queue — renaming means destroying and
  re-registering the pool that holds merge authority.
- **ADR-0630 D2 overstates**: claims an amd64 box "never edits a workflow". False as built — the
  arch-specific name is hardcoded 11 times. Needs an amendment, not a silent contradiction.

Idea one-pager: `.omc/ultragoal/ideas/derivation-resolution-not-artifact-transport.md`

## 14. buck2 config is SILENTLY PERMISSIVE — the class behind every cache dead end

buck2 NEVER errors on config it does not understand. Three confirmed instances:
1. `[buck2_re_client]` via `--config` — parsed against an EMPTY config-args slice at daemon init;
   silently inert. Also absent from `DaemonStartupConfig`, so editing `.buckconfig.local` with a
   live daemon does not restart it — it serves stale empty RE config forever. `buck2 killall` is
   MANDATORY after any RE config change; a measurement without it is invalid.
2. `remote_cache_enabled` set on `[buck2_re_client]` — WRONG PLANE. It is a
   `CommandExecutorConfig()` Starlark param on the EXECUTION PLATFORM. Silently resolves False ->
   `Executor::Local` -> `NoOpCacheUploader` / DISCARDED. Zero uploads, no error, no warning.
3. `[cache]` (the HTTP-cache section from buck2 issue #459) — **DOES NOT EXIST**. Verified against
   our exact binary's source (commit 1560aca2): zero hits for `http_cache`/`HttpCache` tree-wide,
   and the recognized-section list has no `cache`. **There is NO HTTP cache shortcut** — the only
   remote transport is gRPC REAPI (`remote_execution/oss/re_grpc/`). A JFrog/S3/SeaweedFS-style
   HTTP cache is not an option, so mTLS gRPC to NativeLink is the ONLY path.

**Rule: verify buck2 config from the DAEMON side (DaemonStartupConfig / daemon argv), never from
the fact that config was accepted.**

## 15. install-buck2.sh does NOT use the baked buck2 (measured)

The ambient-buck2 branch is the `*)` UNSUPPORTED-HOST fallback, gated on
`OYA_CI_ALLOW_AMBIENT_BUCK2=1` — it is NOT a general switch. On linux-aarch64 the script takes the
normal path and DOWNLOADS from GitHub releases unless the digest-pinned COMPRESSED ASSET is present
at `${BUCK2_INSTALL_DIR}/sha256-<digest>/`.

Consequence: **every job in oya-ci-required downloads buck2 on every run** despite it being baked —
the lane has no buck2 asset cache (only docs-graph-drift had one). The bake buys nothing for buck2
(it does for rustc, because RUSTUP_HOME is set and rustup finds it).

LIKELY FIX, unverified: set `BUCK2_INSTALL_DIR=/opt/buck2` at workflow level so the fast path finds
the baked asset. Requires confirming the image RETAINS the `.zst` asset (the Dockerfile does not
delete it, but that was not verified against a pulled image). Do NOT ship on reasoning alone.

NEAR-MISS: I nearly removed docs-graph-drift's buck2 cache as "redundant against the baked image".
It is NOT redundant — removing it ADDS a download per run. Caught only by reading install-buck2.sh
instead of trusting the inference. The edit also corrupted the step structure (a step named
"Cache Buck2 official prebuilt" ended up carrying the materialize step's `run:` body) and was
reverted. #1457 stays the verified one-line flip.

RECORDED, not fixed: docs-graph-drift pins `toolchain: 1.97.1` as a LITERAL via
`dtolnay/rust-toolchain`, duplicating rust-toolchain.toml — a second source of truth for the pinned
channel, and the lane's only third-party action.

## 16. Artifact accounting CLOSED (2026-07-31)
233 MB -> **35 MB** (7% of the 500 MB allowance). Reaped: 195 dead intra-run `generated-faces`
(they pre-dated #1452's retention-days:1 and carried the 90-day default), plus the earlier
dead-producer families. Full sweep: **10 families, ALL with live producers, ZERO dead.**

Count and bytes point at DIFFERENT culprits — do not conflate them again:
- by BYTES: `generated-faces` was 87% (199 x ~1 MB) — an intra-run handoff.
- by COUNT: the `always()` triage families are 93% but total ~4.6 MB. So #1456 cuts ~65% of COUNT
  and ~2% of BYTES.
Diagnostic that proves the driver is TRIGGER CONDITION, not retention: `build-health-baseline` has
the LONGEST retention (90d) and is one of the SMALLEST families (247), because it is correctly
conditioned to push-on-dev.

Packages do NOT consume the pool: both (`console-web`, `console-app`) are PUBLIC, and GitHub does
not bill storage for public packages. So there is NO overage to bill and NO billing dependency —
the founder was right that this never needed billing clearance.

## 17. REORG RE-SCOPED + THE REAL BLOCKER (2026-07-31, read before ANY move)

### The remaining work is 4 crate moves, not 22 capabilities
59 of 63 live legacy dirs hold **ZERO crates** — the crates already moved; 4284 files of PHASE-2
SATELLITE (docs/SLOs/contracts/manifests/runbooks) remain. Only 4 dirs still hold crates:
  oya/ci-tide 3 · oya/ci-controller 4 · oya/ci-webhook-gateway 5 · **oya/intelligence 88**
The "22 capabilities, serial, days of work" framing came from the STALE ledger and was wrong.
This only became visible after #1465 made the ledger honest — the measurement was worth more
than the migration.

> ⚠️ **THE PARAGRAPH ABOVE IS WRONG — superseded by the re-measurement in §18.** It undercounted
> by ~100x. It measured only a subset of legacy dirs; the true figure on dev @0c7e0f3a3 is
> **425 crates across 339 capability dirs in 5 legacy roots**. Kept, not deleted, because the
> error is instructive: an under-scope this large came from trusting a partial scan, and it is
> the same class as the stale ledger it was correcting. **Re-measure; do not trust either number
> without re-running the probe.**

### 🔴 BLOCKER — artifact-only move plans WEDGE THE ENTIRE REPO
`manifest.rs:129` `plan_is_landed = !old_crate_dirs.is_empty() && …` and `main.rs` built the probe
from `plan.moves` ONLY. An artifact-only plan (`moves: []` — the satellite shape) yields an EMPTY
probe -> **never landed -> ACTIVE FOREVER**. Two committed => `MultipleMovePlans`, raised from
**step 1 of the UNIVERSAL materializer** (`generated-artifact-freshness/src/lib.rs:607`),
**fail-closed**, on every CI leg and every local gate lane => **every subsequent PR wedged**.
`model.rs:119-128` already blesses the artifact-only shape and its own comment admits the guard was
"left keyed on `moves` alone". `manifest.rs:322` PINS the bug.
**A 10-way parallel satellite fan-out would have created 10 permanent landmines.** The oracle phase
cost one agent and prevented it. FIX IN FLIGHT: branch `fix/reorg-artifact-only-plan-never-lands`
(commit 14291dd1f) — caller fix + `manifest::plan_probe_paths` seam. **NEEDS A RED TEST + GATE RUN.**

### Parallel-by-destination is the WRONG AXIS
My premise "zero crates => touches none of the four playbook files" is TRUE and IRRELEVANT. Those
four are crate-keyed. The files that actually gate a satellite move are five OTHERS, all single
files edited regardless of destination:
  root `Cargo.toml` `[workspace].exclude` — **68/68 movers**, 737 entries program-wide (19 for
    cloud/tenancy alone: `<cap>/*/*` member globs match every non-crate subdir, cargo then errors)
  `specs/microservice-tier-classification.json` 60/68 · `specs/reachability-registry.json` 44/68
  `specs/capability-registry.json` 62/68 · plus `specs/reorg/` as a single-ACTIVE-plan mutex
Grouping by destination separates NONE of them.

### THE ANSWER: shard the registries — contention is a symptom, not a constraint
Hyperscaler pattern is **derive, don't declare**. Google has no global ownership registry
(per-directory OWNERS); Bazel has no global target list (per-package BUILD, graph derived by
walking). A global file hand-edited on every move IS a mutex by construction.
Per-file test: **is this fact derivable from the tree?**
  derivable -> DELETE the file, derive it (`absorbs_current_dirs` = "which legacy dirs still exist";
    the FS knows. `microservice-tier-classification` RESTATES the per-service manifest.json 60/68)
  human judgement -> SHARD beside its subject (`<cap>/.facts/`, existing `manifest.json`)
  gate needs totality -> GENERATE the projection from shards, machine-written, never hand-edited
  frozen ratchet -> LEAVE ALONE (baselines/signoff are legitimately global, NOT edited per-move,
    cause zero contention — do not shard these)
**This repo already solved this pattern**: ADR-0613 de-committed 7 derivation faces for exactly this
reason and the registries were never migrated. Matches our own
`optimal-monorepo-shape-cellular-hub-aware` doctrine (per-cell nested workspace + sharded
`<cap>/.facts/`). SHARD FIRST -> the 59 moves parallelize by construction, no oracle needed.

### Other findings from the oracle
- The playbook claims `specs/reorg/move-manifest.generated.json` is "the ONLY committed face a move
  updates". It is **NOT tracked** — de-committed, materializer-produced. Following it literally
  commits a generated face.
- `tenancy/` destination IS northstar-conforming (core/47 adapters/17 ports/10 facade/5 + OWNERS) —
  a correct exemplar to conform to.
- OWNERS breadth cap: `oya-ci.toml max_paths_per_owners_file = 2000`. Exceeding it SILENTLY refuses
  coverage and leaves the tree `unowned`. tenancy goes 85 -> 296 (fine); check larger satellites.

---

## 18. SESSION 2026-07-31 (late) — contention attacked at the root; re-measured scope

### Re-measured scope on dev @0c7e0f3a3 (probe: `scratchpad/scope.rb`, re-run it)
A "crate" = a `Cargo.toml` carrying `[package]` (a bare `[workspace]` root is NOT a crate — that
single distinction is what made the earlier 425-vs-426 census disagree).

| root | capability dirs | crates | tracked files |
|---|---|---|---|
| `oya/` | 82 | 190 | 8246 |
| `cloud/` | 21 | 20 | 1477 |
| `libs/` | 185 | 185 | 690 |
| `tools/` | 34 | 30 | 167 |
| `infra/` | 17 | 0 | 83 |
| **total** | **339** | **425** | **10663** |

48 of 82 `oya/` caps and 20 of 21 `cloud/` caps hold ZERO crates (satellite-only). But **34 `oya/`
caps DO hold crates**, plus every one of the 185 `libs/` dirs — so §17's "only 4 dirs hold crates"
was wrong by ~100x. `oya/intelligence` (88) remains the single largest.

### ✅ BLOCKER CLEARED — `plan_is_landed` landed as #1466
All SEVEN committed plans in `specs/reorg/` verify as **LANDED** (probe: `scratchpad/plan_state.rb`,
which mirrors `manifest.rs` exactly: landed iff probe non-empty AND every old path absent at the
merge-base). **0 ACTIVE plans**, so the single-plan guard is satisfied and the repo is not wedged.
Re-run that probe before committing any new plan.

### ✅ MUTEX #1 REMOVED — root `Cargo.toml` (#1471)
The **68/68** mover. ADR-0538 globbed membership so adding a CRATE needs no edit, and named the
class ("100%-by-construction merge-conflict class"). But membership was enumerated ONE GLOB PER
CAPABILITY (24 of them), so adding a CAPABILITY still edited it — the mutex had only moved up a
level. And `<cap>/*/*` matched every satellite dir, so each capability ALSO needed an `exclude`
entry (23 of 26 were `<cap>/observability`-shaped). **Two edits to one shared file per move.**

Replaced with four shape globs — `*/core/*`, `*/ports/*`, `*/adapters/*`, `*/facade/*` — that
describe ADR-0562 §1's layout. members 30→12, exclude 26→3.
- **Proven behaviour-preserving**: `cargo metadata` resolves the IDENTICAL member set, 876→876
  (sorted `manifest_path` set equality, not just count).
- **Proven to remove the contention, both directions**: a probe capability with a crate AND a
  satellite dir needs ZERO manifest edits under the new globs (877 members, crate present); under
  the OLD pattern the same capability needs a members entry AND an exclude entry, and without the
  latter cargo hard-fails `failed to load manifest ... referenced via `probecap/*/*``.

### MUTEX #2 IN FLIGHT — `specs/microservice-tier-classification.json` (60/68 movers)
**Fully derivable, and the ADRs already say so.** Measured (`scratchpad/tier_derive2.rb`):
101 rows, all 101 referenced manifests exist, and **all 101 restate a `tier` the per-service
`manifest.json` already carries** — plus `service_count` / `tier_distribution`, which are pure
recomputations of the rows. **No Rust/shell/YAML reads its content**; only generated inventories
and specs/doc pointers reference it by path.
ADR-0562 (~L2772) already calls it *"the generated projection"*; ADR-0245 lists it as
*"NEW — derived from §D-3"*. So this IMPLEMENTS an existing decision (cf.
`reorg-must-implement-existing-adr-cluster`), following the ADR-0595/0597/0613 de-commit strangler.
⚠️ Watch for the irony trap: `specs/reachability-registry.json` REGISTERS this path, so a careless
de-commit trades one global-registry edit for another. Check before landing.

### 🔎 First probe was WRONG-SHAPED — a reminder, not a footnote
My first derivability probe keyed on service NAME and reported `0 matching, 101 missing`, which
reads as "not derivable at all". The registry is keyed by PATH. Corrected, it is 101/101 —
the exact opposite conclusion. Same class as `absence-of-proxy-is-not-absence-of-thing`:
**a zero result from a probe is evidence about the probe until the probe is validated.**

### Remaining mutexes (unattacked)
`specs/capability-registry.json` 62/68 — 23 readers, a CLOSED registry = human judgement, so
  SHARD beside its subject rather than derive.
`specs/reachability-registry.json` 44/68 — 17 readers, 124 `registered` entries.
`specs/reorg/` — single-ACTIVE-plan mutex; now empty of active plans, cost is per-move not global.

### MUTEX #3 MEASURED — `specs/capability-registry.json` (62/68 movers): SHARD, do not derive
Per-capability keys are `name`, `charter`, `seed_domains`, `absorbs_current_dirs`, `dag_node`.
**Only `absorbs_current_dirs` is a per-move field**; the other four are stable human judgement set
once at capability creation. So the contention is ONE field, not the file.

Measured on dev @0c7e0f3a3: 24 capabilities, 23 with absorb entries, **86 entries, 0 stale**
(#1465's burn-down is holding). Remaining load is spread — billing 8, iam 7, comms 7, data 6,
k8s 6, storage 5, workflow 5 — which is exactly why sharding decouples: a move into `billing`
would touch only billing's shard, so two concurrent moves to different capabilities never collide.

**NOT derivable, unlike the tier projection.** `absorbs_current_dirs` declares *"this legacy dir
BELONGS to this capability"* — a mapping the filesystem cannot know. Only its LIVENESS (does the
dir still exist) is derivable, and that is already gated. So the doctrine answer is SHARD beside
the subject (`<cap>/.facts/` or the capability's own manifest), then GENERATE the totality
projection the gates read.

⚠️ **Bigger blast radius than the tier file — 3 gate crates read this one for real** (not just by
path): `ci/facade/module-membership`, `ci/facade/crate-registration`,
`ci/facade/cross-artifact-agreement` (incl. `src/registry_policy_sync.rs`). Compare the tier
projection, which has ZERO code readers. Sequence tier FIRST; it is the cheap, fully-derivable win
that proves the pattern before touching a registry three gates depend on.

### Why "drive to completion" is a program, not a session
425 crates + 10663 tracked files remain across 339 capability dirs. The binding constraint is not
throughput, it is that **moves are serialized by shared-file contention** — the strangler playbook
mandates SERIAL moves for exactly this reason. So the highest-leverage work is removing the
contention (mutexes #1–#3), after which the moves parallelize by construction and the remaining
volume becomes schedulable. Attacking the 425 before the mutexes would serialize the whole program
behind five files.

### 🔴 THE ACTUAL BLOCKER TO "COMPLETION": 81% of legacy dirs have NO DECLARED DESTINATION
Measured on dev @0c7e0f3a3 (longest-prefix match of every live `<root>/<dir>` against the union of
all `absorbs_current_dirs`):

| | count |
|---|---|
| live legacy capability-level dirs | **338** |
| ABSORBED — some capability claims them | **63** (18.6%) |
| **ORPHAN — no capability claims them** | **275** (81.4%) |

Orphans by root: `libs/` 185 · `oya/` 37 · `tools/` 34 · `infra/` 16 · `cloud/` 3.

**Read this correctly.** `libs/` 185 are individual CRATES, not capability dirs — they need a
DISPOSITION decision (base/ admission vs capability assignment vs DELETE; memory records only 14
clear the base/ >=3-consumer bar and 64 have zero dependents). The other **90** (`infra` 16,
`cloud` 3, `oya` 37, `tools` 34) are genuine capability-level dirs with no declared home.

**This is a DECISION problem, not an execution problem.** The move-plan machinery requires a
destination and 81% of the tree has none, so "drive the reorg to completion" cannot be discharged
by executing moves — the destinations must be ruled first. `specs/capability-registry.json` is
`closed: true`, so adding 275 absorb entries is a founder-level governance act, not an agent one.
§2 precondition #1 anticipated this but understated it as "infra/ (17 subdirs)".

### 🔎 THE DETECTOR IS ONE-DIRECTIONAL — that is why the gap was invisible
`module-membership`'s `live_registry_absorbs_dirs_all_resolve` (added by #1465) checks
**entries -> dirs**: every absorb entry must name a directory that exists. Nothing checks
**dirs -> entries**: that every live legacy dir is claimed by some capability. So a dir with no
destination is silently fine, forever, and the reorg's true remaining scope is invisible to CI.

Proof it already drifted: `specs/microservice-tier-classification.json` says
`cloud/cloud-iac -> capability "iac"`, while `iac.absorbs_current_dirs` is `["iac"]` — it does NOT
include `cloud/cloud-iac`, which is still tracked (235 files). Two hand-maintained registries
state overlapping facts and disagree in exactly one place, and no gate can see it.
**The class-fix is the missing direction of an existing detector**, per
`friction-is-process-failure-productize`.

### Tier projection: derivable, but from TWO sources — my "fully derivable" was too strong
Reproduction probe (`scratchpad/tier_reproduce.rb`) against all 101 rows:
| field | result |
|---|---|
| `tier`, `tier_subtype`, `dr_tier` | **101/101 identical** — pure restatement |
| `manifest` | 101/101 (it is the path) |
| `substrate_dag_position` | 55/55 present agree; 46 absent (products legitimately have none) |
| `service` | 90/101 = `dirname(manifest)`; **11 are NESTED** (`oya/developer-sdk/packs/eu/…` -> `oya/developer-sdk`), so the rule is "walk to the service root", not dirname |
| `capability` | **0/101 in the manifest** — must come from `capability-registry.absorbs_current_dirs` (64 agree, **0 disagree**, 37 no-entry of which 36 are legitimately empty) |

So it IS derivable and de-committing is right, but the derivation is real logic over two registries
with two edge-case rules — not the trivial manifest read I first claimed. **Do not ship it as a
one-liner.**

### Tier-projection de-commit: 4 BLOCKERS found before writing any code (wf_6754f68f-d61)
Do not attempt this as a quick win. Each of these turns a naive de-commit into a false green:

1. **The one-way door is a NO-OP unless the path matches a `generated_path_rules` entry.**
   `generated_artifact_not_tracked_path_is_tracked` — the ADR-0595 guard that makes RE-committing a
   de-committed face a hard RED — computes `not_tracked_paths.intersection(&tracked_generated)`,
   and `tracked_generated` only contains paths that matched a RULE. Declare the face
   `not-tracked-in-git` without a matching rule and the protection silently disarms. Textbook
   `dark-wiring-that-greens`.
2. **`canonical-json` scans the UNTRACKED materialized file** — genuinely `specs/`-specific, and no
   prior de-commit hit it because ADR-0595/0597/0613 all operated under `ci/facade/`. That gate
   declares `governed_roots: ["specs"]` and its collector is a read-only FILESYSTEM WALK, so a
   materialized-but-untracked file under `specs/` IS scanned.
3. **`CONTROLLER_MATERIALIZED_ARTIFACT_PATHS` without a regenerator in `regenerate_all_faces` is an
   instant CI RED**, not a silent skip — the freshness gate requires every de-committed face name to
   appear in the regenerated set. (Fail-closed, so this one is safe-by-design; just do both.)
4. **`serde_json::to_string_pretty` is NOT byte-deterministic across cargo and buck2 here.**
   Reindeer unions the serde_json `preserve_order` feature ON under buck2, so parsed object key
   order follows FILE order there and SORTED order under cargo. The projection embeds whole
   `substrate_dag_position` objects, so a naive emitter produces different bytes in the two
   toolchains — a byte-parity gate would flip depending on which build ran it. Cf.
   [[buck2-config-evidence-admissibility]]: cargo and buck2 are not interchangeable oracles.

**Precondition now MET:** `capability` reproduces 101/101 by pure reverse lookup once
`cloud/cloud-iac` is absorbed (PR #1472). Field-by-field derivation status is in the table above.

### 🔴 LAST NON-QUOTA BLOCKER TO A GREEN REQUIRED LANE: buck2 test actions have NO rustup env
Four tests fail in CI and PASS locally on macOS — the classic pass-local/fail-CI shape. One root
cause, two symptoms:
  `core-dependency-isolation` (2)  "rustup could not choose a version of cargo to run, because one
                                    wasn't specified explicitly, and no default is configured"
  `scm-facts-snapshot` (2)         "rustup is not installed at '/home/runner/.cargo'"

Both are tests that SHELL OUT to `cargo`/`rustup` from inside a buck2 action
(`cargo_metadata_validator` in core-dependency-isolation; `preprovision_historical_p2_toolchain`
in scm-facts-snapshot). `grep -rn 'RUSTUP_HOME|CARGO_HOME' toolchains/ .buckconfig* prelude/`
returns **NOTHING**, so buck2 sanitizes the action environment and the child falls back to
`$HOME/.cargo` — which the runner image never populates (it sets `RUSTUP_HOME=/opt/rust/rustup`,
`CARGO_HOME=/opt/rust/cargo`). It passes on a dev Mac only because a personal rustup default exists.

**Same class the image already hit once**, per its own Dockerfile comment: *"the build-time check
passed for exactly that reason while the shipped image was broken for every other caller"* — a
toolchain that resolves only in a directory that HAS a `rust-toolchain.toml`. Note
`core-dependency-isolation` builds a TEMP fixture workspace with no `rust-toolchain.toml`, so even
a cwd-based resolution fails there.

**NOT FIXED — needs its own investigation, do not guess.** The obvious move (hardcode
`/opt/rust/cargo` into `rust_test` env) breaks local macOS, and the prelude is not vendored so
buck2's test-env semantics cannot be settled by reading source in-tree. Guessing here ships a
false green on the binding hermetic lane. Relevant: [[toolchain-is-ambient-not-hermetic]] — this
is that memory's prediction coming true in CI.

### 🔴 VERIFIED DEFECT: `docs/MASTERPLAN.md` is a TRACKED file the materializer silently overwrites
The universal materializer writes NINE outputs. Eight are untracked AND gitignored. **`docs/MASTERPLAN.md`
is TRACKED, NOT ignored, and has ZERO rows in `registry/generated-artifact-control-plane.json`** —
the materializer writes one committed file that no control-plane row governs.

**Proven, not inferred.** Appended a probe line to the file, ran the materializer, and it was
silently reverted:
```
after edit:        md5=b1a6f2d2…  git-dirty=1
after materialize: md5=4d3d4a51…  git-dirty=0     <-- the edit is GONE
```
Written at `generated-artifact-freshness/src/lib.rs:635` -> `:2439-2466`; path const at
`cross-artifact-agreement/src/projection_rederivation.rs:65`.

**Latent, not yet firing.** It is byte-stable today, which is the only reason no one has noticed.
The moment masterplan inputs change, any contributor running the materializer — which the local
gate-lane instructions REQUIRE before several gates — silently rewrites or reverts a committed doc.
`generated-output-diff-policy` cannot catch it because catching it requires a control-plane row,
and there isn't one. Exactly [[committed-projection-faces-decommit-adr0613]]: *"masterplan/
product-graph faces are merge surfaces a mis-invoked materializer corrupts silently."* Note
`docs/architecture/product-graph.html` — its sibling in that memory — WAS de-committed
(`.gitignore:132`); MASTERPLAN.md was left behind.

**NOT fixed here — it is a governance call, not an agent one.** Two options with different
consequences: add a `hand-curated-committed`-style row (keeps the doc linkable, but then the
materializer must stop writing it), or de-commit it per ADR-0613 (consistent with product-graph.html,
but breaks every human link to a tracked path). Founder/ADR decision.

---

## 19. DESTINATIONS ARE DERIVABLE — the rules, and the two traps (2026-07-31)

Founder challenge: *"can't you figure it out based on hyperscaler monorepo best practices?"* — YES.
Destination is a FUNCTION of measured facts, not a taste call. Probe: `scratchpad/destinations.rb`.

### The mechanical rules (each cites doctrine the repo already adopted)
| rule | condition | disposition |
|---|---|---|
| R1 | zero dependents AND zero deps | DELETE candidate |
| R2 | zero dependents | DELETE candidate ⚠️ see TRAP 2 |
| R3 | consumed by exactly ONE capability | MOVE into that capability |
| R4 | consumed by >=3 capabilities **AND strictly below all of them in the ADR-0280 DAG** | `base/` (Google //base bar) |
| R5 | consumed by 2 capabilities | semantic call |
| R6 | consumed only by other LEGACY crates | follow the chain upward |

Measured over 405 legacy workspace crates: R6 124 · R2 105 · R3 82 · R1 78 · R5 9 · R4 7.

### 🪤 TRAP 1 — a RETIRING MONOLITH silently becomes everyone's destination
R3 naively sends **68 crates to `marketplace`**, because their single consumer is
`marketplace/facade/dev-cli` — which has **77 first-party deps, 67 of them `oya-check-*`**, and
whose OWN manifest says *"retirement-marked per founder CLI-retirement directive (2026-06-09);
zero merge authority"*. **A dying, zero-authority monolith must never define ownership.** Obeying
the raw edge would have rebuilt the junk drawer ADR-0562 §6 exists to prevent, under a new name.
**Rule: exclude retirement-marked consumers from the ownership graph before classifying.**

### 🪤 TRAP 2 — the cargo graph is the WRONG oracle for DELETE
183 crates show zero cargo dependents, but a crate is alive if a BUCK target, a workflow, a gate
matrix, or a `[[bin]]` reaches it — none of which is a Cargo edge. **DELETE requires a buck2 + CI
probe; cargo-dependents alone is not a liveness predicate.** Cf.
[[absence-of-proxy-is-not-absence-of-thing]]. Under adversarial check in wf_99f1513a-def.

### 📐 CLI AUDIT (founder: "API based not CLI based, universal, productized for CI")
Probe `scratchpad/cli_audit.rb` over all 876 packages:
- **121 crates ship an executable**; **43 are FAT CLI surfaces** (parse argv + >120-line main);
  **24 have NO library at all** — logic trapped behind argv, not API-callable.
- **`ci/` = 23 bins, 23 of which parse argv, only 3 thin.** *The gate fleet itself is CLI-shaped.*
- `tools/` = 30 bins, **19 with no lib** — the worst root.
- `libs/` still carries four crates literally named `*-check-cli`.

**So the gate fleet is the biggest violator of the directive it exists to enforce.** This is the
`ci/` inversion measured the same day: **facade=52, ports=1, adapters=1, core=0** — every gate is a
facade with no kernel, which is backwards from the stated rule that a facade reaches core only
through ports.

### ✅ THE DERIVED ANSWER for the 67 check crates (satisfies all three constraints at once)
They must NOT become 67 more `ci/facade` CLIs. Northstar + clean architecture gives:
```
ci/core/<check>      pure kernel, ZERO I/O:  fn evaluate(&Policy, &Facts) -> Verdict   <- API
ci/ports/            the seam: Collector / Reporter traits
ci/adapters/         filesystem + git collectors (ALL the I/O lives here)
ci/facade/           ONE engine that runs every gate — not 52 argv parsers
```
- **API-based**: `evaluate()` is callable in-process; no argv, no shell.
- **Universal**: a pure kernel over (Policy, Facts) carries no oyatie path — the pack-shaped
  contract several gate policies already claim ("an adopting repo repoints registry_path").
- **Productized for CI**: one engine + N kernels is a product; 52 CLIs are a script collection.
The sampled checker (`libs/oya-check-a11y-discipline`) is already PURE — no `std::fs`, no
`std::process` — so for many of the 67 this is a MOVE into `ci/core/`, not a rewrite.

**De-brand on arrival**: `libs/oya-check-a11y-discipline` -> `ci/core/a11y-discipline`.

---

## 20. FULL MOVE AUDIT + REPO-WIDE SHAPE AUDIT (2026-07-31)

### Did we naively MOVE where a REFACTOR was warranted? YES.
Audit of all **117 crates relocated by landed move plans** (`scratchpad/move_audit.rb`):

| finding | count | note |
|---|---|---|
| **NO_OWNERS** | **40** | `os/` had exactly ONE OWNERS file for 41 relocated crates |
| SHELLS_OUT (`Command::new`) | 7 | all `ci/facade/*`; needs per-case ADR-0523 ledger judgment |
| **IMPURE_CORE** | **3** | `os/core/{apid-domain,init-app,trustd-domain}` |
| **LAYER_SUFFIX_MISMATCH** | **3** | `os/core/init-app` (-app in core), `os/core/proto-api` (-api in core), `os/harness/difftest-app` |
| **NO_PORTS_SEAM** | **2** | `intelligence` (core30/adapters10/ports0), `messaging` (core5/adapters1/ports0) |
| STRANDED_SATELLITE | 1 | `cloud/cloud-os`, 1 file |
| moved INTO a facade→core violation | **0** | ✅ the codemod did not manufacture layering breakage |

**The clearest case:** `os-move-plan` relocated 41 crates as a bulk move with no shape work.
`os/core/trustd-domain/src/persistence.rs` does `use std::fs::{self, OpenOptions}` — a *domain*
crate in `core/` doing FILE PERSISTENCE. That is an adapter concern in the pure layer.
**Fix landed:** `os/OWNERS` (one hierarchical file; coverage is additive — the policy's own comment
cites `registry/catalog/OWNERS` covering 886 paths). 557 paths under `os/`, cap is 2000. 40 → 0.

### 🪤 THE PORTS TRAP — fixing NO_PORTS_SEAM wedges CI unless done atomically
`facade-core-layering/src/lib.rs:179`: `let has_ports = !dir_names(&cap_dir.join(ports))?.is_empty();`
The gate picks its violation CODE at scan time from whether `ports/` is non-empty. Verified:
`facade_core_no_ports_layer` (5) contains `intelligence-app` + `intelligence-worker`;
`facade_core_direct_dep` (30) contains **zero** intelligence entries.
**So the instant ANY crate lands in `intelligence/ports/`, `has_ports` flips true and those two
already-frozen edges re-emit under the OTHER code — which does not baseline them ⇒ 2 NEW
violations ⇒ RED.** Creating the seam must land WITH the `facade/{app,worker}` rewire in the same PR.
(`messaging` is likely exempt: the gate `continue`s when a capability has no `facade/` dir.)

### Repo-wide shape audit (all top-level dirs, not just moved crates)
Registry: 24 capabilities + 7 meta dirs (`app base build governance kernel os third-party`).
- **12 unregistered top-level dirs, ALL with zero crates** — `docs specs registry evidence scripts
  templates tasks packs plan contracts benchmarks toolchains`. Mostly benign data/doc roots.
- **`toolchains/` is real drift**: the `build/` charter explicitly claims *"buck2 prelude,
  toolchains, reindeer, CI engines"*, yet `toolchains/` sits at top level. Should be `build/toolchains/`.
- **`ci` — 53 crates, layers `ports+adapters+facade`, NO core.** The gate fleet is all facade.
- **`gateway` — 10 crates, `adapters` ONLY, no core.** Ten connectors (adp, epic-fhir, gusto,
  netsuite) implementing ports for a core that does not exist — the INVERSE of intelligence
  (core with no seam vs adapters with nothing to adapt to).

### ⚠️ SIX PROBE ERRORS IN ONE SESSION — the instrument, not the finding
Every one was the same failure: asserting a measurement before validating the instrument.
1. tier-registry derivability keyed on service NAME (registry is keyed by PATH) → "0 derivable", actually 101/101.
2. liveness "named in a BUCK file" → matched the crate's OWN `name =` declaration.
3. liveness "named in a policy JSON" → matched only `*.generated.json` INVENTORIES.
4. liveness "hand-authored policy" → matched `capability-membership-policy.json`'s **legacy-debt census** (being in it is the OPPOSITE of liveness).
5. IMPURE_CORE → matched `std::process::id()` (reads own PID) as I/O; 6 → 3.
6. shape audit → `meta_directories` entries are objects keyed `"dir"` WITH a trailing slash; my extractor read `"name"`, got nils, collapsed to one → falsely reported `os` (41 crates) and `kernel` as unregistered.

**RULE: a probe returning a suspiciously round, total, or alarming number is measuring the wrong
thing until proven otherwise.** 183/183 · 100% · 16-unregistered — each looked like a finding,
each was a broken instrument. This is the same argument as `within_view`: encode the check in the
build system, because a constraint that depends on the operator remembering will eventually not be.

---

## 21. CONSOLIDATED BACKLOG → PARALLELIZABLE LANES (2026-07-31)

**Lane rule: two items share a lane iff they touch the SAME FILE.** Grouping by topic is what
produced the wrong-axis parallel plan earlier; grouping by contention is what actually parallelizes.

### 🔴 LANE A — MOVES. STRICTLY SERIAL, repo-wide mutex.
`MultipleMovePlans` is raised from step 1 of the universal materializer, fail-closed, on every CI
leg. At most ONE active plan exists at a time. Everything here queues behind everything else here.
| id | item | size | note |
|---|---|---|---|
| A1 | `oya/ci-tide` → `ci/{core,adapters,facade}` | 10 files, 3 crates | PAVED-ROAD PROOF. Already correctly layered (pure kernel + reqwest adapter + tokio app), ZERO satellite, ZERO consumers ⇒ zero blast radius |
| A2 | `os/` layer fixes: `init-app`, `proto-api`, `difftest-app` | 3 crates | LAYER_SUFFIX_MISMATCH; these are moves |
| A3 | `intelligence` (+`cloud/cloud-intelligence`+`oya/detection`) | 88 crates + 452 satellite | NOT a move — MOVE(13 pure kernels) + REFACTOR(ports seam) + REWRITE(3 CLI adapters) |
| A4 | `toolchains/` → `build/toolchains/` | 4 files | `build/` charter already claims it |

### LANE B — THE CI WORKFLOW FILE. Serial within (one file), parallel with A/C/D/F/G.
| id | item |
|---|---|
| B1 | buck2 test actions carry NO rustup env ⇒ 4 tests fail CI, pass locally. **Last non-quota blocker to a green required lane.** |
| B2 | remote-cache enablement: owned execution platform + named opt-in + fail-closed overlay assert |
| B3 | move the 2 required Postgres lanes off GitHub-hosted runners (needs D2) |

### LANE C — ADR CORPUS. Internally parallel by ADR; touches nothing in A/B/D/E/F.
| id | item |
|---|---|
| C1 | 66 Accepted ADRs with ≥3 dead path anchors. **ADR-0562 first — 167 dead of 807 refs, and it is the shape authority every reorg decision cites.** ADR-0258 is 44/45 dead (effectively fully stale) |
| C2 | 429 citations of PROPOSED ADRs by Accepted ones. Triage, don't sweep: ratify the load-bearing (ADR-0009 cell-architecture ×31, ADR-0003 audit-chain ×23, ADR-0111 merge-queue ×16 **and named in root CLAUDE.md**) vs de-cite the dead |
| C3 | ADR-0184 says Citus+Patroni; live ARC config assumes CNPG. **Amend before deploying D2, not after** |
| C4 | Record that ADR-0044 (Istio/Envoy) is PROPOSED — the gateway TIER (0157/0182) is ratified, the implementation never was |

### LANE D — INFRA DEPLOYMENT. New dirs under `infra/`; highly parallel.
| id | item | blocked? |
|---|---|---|
| **D1** | **BACKUP (ADR-0197 Accepted, unimplemented).** `nativelink-cas`, `seaweedfs`, `registry` all on `storageClassName: local-path` NODE-LOCAL PVCs with zero backup. **UNBLOCKED, cheap, only item where delay risks irreversible loss** | no |
| D2 | Postgres/CNPG — also resolves the dangling `oya-pg-rw` + `oya-pg-superuser` refs in ARC values that point at a cluster with ZERO manifests | needs C3 |
| D3 | API gateway / L7 ingress / LB — today the ONLY ingress is a cloudflared tunnel to the apiserver VIP | needs impl ruling |
| D4 | Event bus (ADR-0557) — its anchor ADR-0397 is PROPOSED and self-describes as a "RECONSTRUCTION" | needs C2 |

### ⚠️ LANE E — NEW DETECTORS. **Has its own hidden mutex.**
Each detector is a new crate (parallel), BUT adding a gate also edits the gate matrix in
`oya-ci-required.yml` (**Lane B's file**) AND `capability-membership-policy.json`. So Lane E is
serialized on the gate matrix unless that line is sharded first. *This is the same class of
contention the whole session has been about — worth fixing before running the lane.*
| id | detector | closes |
|---|---|---|
| E1 | dirs → entries (every live legacy dir must be claimed) | 275 orphans invisible |
| E2 | a `ports/` crate must define ≥1 trait | storage/object-api, workflow/event-bus-api, k8s/cluster-lifecycle-api are DTO crates wearing a port's name |
| E3 | every `.gitattributes` merge driver resolves to an executable | 4 declared, 1 registered, pointing at a DEAD buck-out path |
| E4 | ADR path anchors must resolve | 66 stale Accepted ADRs |
| E5 | an Accepted ADR may not cite a Proposed one as authority | 429 citations |
| E6 | no generated file may be git-tracked | `docs/MASTERPLAN.md` is tracked AND silently overwritten |

### LANE F — PORTS / REFACTOR. Parallel ACROSS capabilities, serial within one.
| id | item |
|---|---|
| F1-F3 | add real traits to `storage/ports/object-api`, `workflow/ports/event-bus-api`, `k8s/ports/cluster-lifecycle-api` (0 traits each today) |
| F4 | `compute/adapters/{aws,oci}` have NO ports layer — direct cloud-provider lock-in |
| F5 | `intelligence` ports seam ⚠️ **WEDGE TRAP** — `facade-core-layering:179` picks its violation CODE from whether `ports/` is non-empty, so the FIRST ports crate re-emits 2 frozen edges under an unbaselined code ⇒ RED. Seam + facade rewire MUST land in the same PR |
| F6 | `gateway` bundles two concerns (API-edge + 10 SaaS connectors) — rule BEFORE moving, or the absorb list cements it |

### LANE G — REPO HYGIENE. Parallel with everything.
| id | item |
|---|---|
| G1 | automate merge-driver registration to a stable path + assert it |
| G2 | `docs/MASTERPLAN.md`: add a control-plane row OR de-commit (governance call) |
| G3 | restore a fail-closed freshness test for the de-committed faces — byte-parity-to-committed was RETIRED and replaced by regenerate-twice, which proves determinism, NOT correctness |

### Dispatch order (by what unblocks the most, not by ambition)
1. **D1 backup** — unblocked, cheap, irreversible-loss risk.
2. **B1 toolchain env** — last non-quota blocker to a green required lane.
3. **A1 ci-tide** — proves the paved road at zero blast radius.
4. **C3 + D2** — amendment then Postgres; removes rented state from merge authority.
5. Everything else parallel by lane.

---

## 22. COMPACTION HANDOFF — 2026-07-31 late

### Landed today (7 PRs)
#1463 disk-reclaim · #1464 affected-set · #1465 absorbs burn-down · #1466 plan_is_landed ·
**#1467** faces-artifact coupling · **#1469** faces upload FATAL (corrects #1467) ·
**#1470** GATE-1 topology + comment-prose counting · **#1471** root Cargo.toml SHAPE GLOBS
(mutex #1 removed, 876=876 identical member set) · **#1472** cloud/cloud-iac absorb.
**OPEN: #1473** `os/OWNERS` (40 unowned relocated crates → 0, accountability gate Pass 2).
dev @628ffa709.

### In flight at compaction (nothing committed by any of them)
| run | state | produces |
|---|---|---|
| `wf_07cd9429-932` ralplan | 12/13 results, ~4 of 5 iterations | full-northstar consensus plan, `pending approval` |
| `wf_3e4b959a-f04` stack audit | just started | per-component hyperscaler pattern/anti-pattern + transitional-appropriateness |
| `wf_18a9e839-868` lane dispatch | just started, **worktree-isolated** | G1 merge-drivers · C1 ADR-0562 anchors · B1 toolchain env · A1 ci-tide plan |

### ⚠️ TWO FILTERS TO APPLY BEFORE MERGING ANY LANE OUTPUT
1. **Did it VERIFY or merely assert?** `IMPLEMENTED_VERIFIED` with no command output is a red flag.
2. **Does it land WIRED?** I told the G1 agent not to edit the gate matrix — **that instruction was
   WRONG and produces dark wiring** (a gate crate outside the matrix runs never and passes always).
   The repo already carries 17 unwired gate apps + 59 unwired check kernels. **A gate lands wired or
   it does not land**; if that serializes on the gate matrix with Lane B, pay the contention.
   (Bun rule: the "temporary" intermediate IS the dark wiring.)

### The single most urgent unblocked item
**D1 BACKUP.** ADR-0197 is Accepted and unimplemented while `nativelink-cas`, `seaweedfs` and
`registry` all sit on `storageClassName: local-path` NODE-LOCAL PVCs. The CAS, the container
registry and the object store are one node-loss from gone. Nothing blocks this. Founder had not
yet given an explicit go at compaction time — it deploys to a cluster SHARED with the console
project, so confirm ownership before any cluster-scoped apply.

### Governance defects found today (new class — see §20 and Lane C)
- **429 citations of PROPOSED ADRs by ACCEPTED ones.** ADR-0009 cell-architecture ×31,
  ADR-0003 audit-chain ×23, **ADR-0111 merge-queue ×16 AND named in root CLAUDE.md** as a current
  substrate ADR. Unratified work is load-bearing across the corpus.
- **66 of 186 Accepted ADRs carry ≥3 dead path anchors.** ADR-0562 (the shape authority) is
  167 dead of 807; ADR-0258 is 44/45 — effectively fully stale.
- **ADR-0044 (Istio ambient + Envoy gateway) is `proposed`, not Accepted** — I cited it as doctrine
  and was wrong. The gateway TIER (ADR-0157/0182) IS ratified; the IMPLEMENTATION never was, so
  nothing commits us to rented mesh infra. **ALWAYS check status + superseded_by + newer
  contradictions before citing an ADR.**

### Corrections I issued against my own earlier claims (do not re-inherit the originals)
- "5 layering gates violate the rule they enforce" — **FALSE**, retracted. I conflated `std::fs` in a
  gate's collector (sanctioned: "Hermetic: filesystem reads only") with the DEP-EDGE rule the gate
  enforces. What survives: **35 facade packages** violate it, frozen shrink-only.
- "spine §17: only 4 dirs hold crates" — wrong for total scope (425 crates remain) but **RIGHT for
  the movable subset** (4 dirs hold the 100 crates that have a declared destination). Both true,
  different questions.
- "generation is the answer to committed-derived-files" — too glib. The real rule is
  **generator = truth, committed copy = CACHE, legitimate only WITH a fail-closed freshness test**
  (Bazel `diff_test`; Google checks in `descriptor.pb.cc` for bootstrap). The pain we paid came
  from generating **OUT-OF-GRAPH into the worktree**, not from generation. And `Cargo.lock` MUST
  stay committed — google3 has no lockfile, so it offers no counter-precedent.
