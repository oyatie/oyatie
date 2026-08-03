### 1. Repair affected-set soundness before using it to remove full work
**Change**
The current Rust successor is already binding; do not “flip” the historical shell driver. Fix these defects first:
- `ci/facade/affected-target-set/src/lib.rs:365-378` treats deleted non-Rust files as irrelevant. First safe version: every deletion, rename, copy, type change, and submodule change selects FULL. Later, query both isolated base and head graphs and union their owners.
- Replace the narrow owner-required set at `affected-set-policy.json:21-25`. Every changed path must map to a Buck owner or an explicit synthetic dependency; otherwise FULL.
- `src/main.rs:823-836` returns green after a PR FULL build without running full tests. Add a full test-health ratchet: run base/head full test reports and block new failures. A full fallback that only builds is “checking less,” not an optimization.
- Move merge-base evaluation out of the candidate filesystem. Workflow `:641-642` creates ignored candidate JSON, then `git checkout` at `:816-820` leaves it in place; broad `**/*.json` BUCK globs can consume it. Use a clean detached merge-base worktree and merge-base-owned Rust materialization.
- Replace workflow shell/Python baseline handling at `:671-835` with the owned Rust selector/baseline component. Validate commit, toolchain, universe digest, report completeness, and provenance—not merely a non-empty `results` object.
- Retire unused `infra/ci/buck2-affected-gate.sh`; do not extend it.
The safe selector is:
`selected = always_run ∪ owners(base,head,diff) ∪ rdeps(universe, owners ∪ synthetic_dependencies)`
Any merge-base, diff, owner, graph, policy, or query uncertainty sets `selected = universe`. Buildfiles, Buck config, prelude, toolchains, dependencies, workflow/setup, selector, materializer, generator, and global-corpus changes also select FULL. Push, merge-group, and dispatch remain full.
Only after this is proven should the selector replace the unconditional `buck2 test //ci/...`. Whole-tree scanners remain `always_run` until their implicit inputs become declared graph or synthetic edges.
**Saving**
No saving may be credited until shadow scoped/full runs show zero target or verdict mismatches. Thereafter, approximately **50–80 runner-seconds** on eligible small PRs; FULL-trigger changes save zero. Current end-to-end wall impact is near zero while the 501/497-second jobs dominate.
**Rigor preserved because**
Every changed path is accounted for; the scoped set is mechanically a superset; every uncertainty executes the exact full build-and-test universe. Shadow comparison validates the proof but does not replace fail-closed derivation.
---
### 2. Replace archive warmth and repeated setup with a hermetic image plus Buck2-native CAS
**Change**
- First make toolchains actual immutable inputs. `toolchains/BUCK:8-28` currently resolves ambient rustup and `/usr/bin/clang`; those compiler bytes are outside the declared graph.
- Introduce a digest-pinned, attested declarative CI image containing the exact Buck2 binary, Rust toolchain/components, C/C++ toolchain, and PostgreSQL client. Its successor belongs under the debranded `ci/` capability.
- Retire `infra/ci/install-buck2.sh` and the repeated installer/rustup steps. A composite action alone merely deduplicates YAML and saves essentially no runtime.
- After hermetic closure, replace the multi-gigabyte archive blocks at workflow `:463-470`, `:629-636`, and `:883-889` with authenticated NativeLink/CAS compile-action reuse across warm-eligible jobs.
- Do **not** add `buck-out` archive restores to the other jobs. The current archive is documented as approximately 5.78 GiB compressed and 12–15 GiB restored (`:437-440`).
- Never cache live-test verdicts. Tests still execute; CAS supplies only content-addressed build artifacts.
**Saving**
- At 20–40 seconds per 41 matrix legs: **13.7–27.3 runner-minutes**.
- Every 10 seconds removed from 50 repeated setups saves another **8.3 runner-minutes**.
- Likely **1–3 minutes critical-path improvement** if compilation/setup materially contributes to the 497/501-second jobs; step telemetry must confirm.
**Rigor preserved because**
Action identity must include image/platform digest, Buck2 revision, prelude, Rust/C++ toolchains, graph configuration, and declared sources. Fork/untrusted execution remains cold; trusted post-merge is the writer. Misses recompute locally, digest mismatches quarantine and fail, and a binding cold-versus-warm byte-digest canary disables all warm reads on any mismatch.
The repository already encodes that contract, but `specs/cache-warm-license.json:6-7` is still false. Warm CAS must not activate before the first green canary.
**Current slow-and-weak configuration:** the pipeline pays for multi-gigabyte warmth while telemetry declares `CACHE_MODE=bypass` (`oya-ci-required.yml:504-511`), and ambient compiler identity is not rigorously closed. This is a false-green risk, not a proven observed false green.
---
### 3. Eliminate duplicate gate execution with an exact, surface-all gate fleet
**Change**
The 41 matrix legs (`:114-196`) run named CI test pairs, while `buck2 test //ci/...` (`:489-497`) runs the CI test universe again.
Replace both with an owned Rust gate-fleet scheduler under `ci/facade/gate-fleet`:
- Query and hash the exact existing `//ci/...` test-target universe.
- Compare it against the registered matrix target union; refuse cutover if they differ.
- Partition the exact union into approximately 8 weighted shards.
- Execute every target despite sibling failures and emit per-gate result packets/annotations.
- Make the fan-in red if any target is missing, duplicated, unexecuted, or failed.
- Dual-run old and new topology until target lists and verdicts are identical.
**Saving**
The supplied ten 95–99-second legs plus the 148-second catalog leg already represent a lower bound of roughly **18.5 runner-minutes** of duplicated cost. Going from 41 to eight shards also removes 33 setup/startup sequences.
Current wall saving may be nearly zero because `215s producer + 148s catalog = 363s`, below the 501-second Postgres critical path. It becomes important after Postgres/freshness optimization.
**Rigor preserved because**
The exact same test target union executes, with surface-all failure reporting and mechanically enforced coverage. No matrix leg is deleted merely because it appears redundant.
---
### 4. Optimize materialization internally; preserve independent attestations
**Change**
Producer reuse already exists: `producer-regen` creates the artifact at `:50-84`, and matrix legs download it at `:176-181`.
Further optimize the owned-Rust materializer by:
- Scanning git/history once and sharing the immutable scan across emitters.
- Running byte-independent emitters concurrently.
- Separating candidate-face generation from merge-base frozen-baseline generation.
- Letting mere-reader jobs consume a candidate-SHA-bound artifact while independently producing private frozen baselines.
- Retaining independent regeneration in registry-drift (`:238-263`) and baseline-ratchet (`:269-300`).
Reject “materialize once and feed every job.” A detector consuming the artifact it attests is self-referential and weaker.
**Saving**
Freshness + registry-drift + producer total 1,102 supplied seconds. A profiling-confirmed 25–40% materializer reduction would save approximately **275–441 runner-seconds**, potentially **124–199 seconds** from the freshness critical path.
**Rigor preserved because**
Every emitted byte is compared against the existing implementation during qualification; artifacts bind candidate SHA, generator/policy digests, path list, sizes, and content hashes. Missing or mismatched metadata is red. Regeneration/determinism detectors remain independent.
---
### 5. Optimize live Postgres execution without skipping or sharing coverage
**Change**
For both lanes (`:930-1065`, `:1067-1186`):
- Replace mutable `postgres:16` (`:936`, `:1073`) with a reviewed immutable digest. No digest is present locally, so this audit will not invent one.
- Remove repeated mutable `apt-get` client installation (`:971-976`, `:1108-1113`) via the pinned CI image.
- Replace inline bootstrap shell with owned Rust while preserving every role, grant, migration, schema, and provenance field.
- Submit the existing six and four targets in one Buck2 invocation per lane while retaining `--local-only`, `--num-threads 1`, and `RUST_TEST_THREADS=1`.
- A later concurrency experiment may allocate a fresh isolated database per target and run targets concurrently, but only after serial-versus-parallel verdict parity is proven.
Reject:
- Cross-job service-container reuse: it sacrifices isolation and turns two parallel lanes into a shared-state failure domain.
- Smaller fixtures: reduced coverage.
- Skipping the jobs for “unrelated” diffs: under the literal no-skipped-check constraint, live environment behavior can change independently of the source cone. Give this proposal zero credit.
**Saving**
CAS, prewarmed client/toolchain, and fewer Buck2/bootstrap round trips should target **120–360 combined runner-seconds**, with perhaps **60–200 seconds** off the 501-second critical lane if profiling confirms setup/compilation dominance.
**Rigor preserved because**
All ten test targets, fixtures, roles, migrations, environment variables, and fresh live-database executions remain. No live verdict is cached.
---
### 6. Benchmark larger runners only after CAS and topology fixes
**Change**
Benchmark identical digest-pinned x86_64 images on larger runners for compile-bound full Buck2, affected FULL, freshness, and materializer shards. Do not blanket-upsize the matrix or Postgres lanes.
**Saving**
Plausible **20–45% cold compile wall reduction**. Cost is favorable only when:
`new_duration / old_duration < old_price / new_price`
**Rigor preserved because**
Image, architecture, toolchain, target set, test concurrency, and output digests remain identical. Reject any runner change whose cold differential produces a different artifact or verdict manifest.
---
### 7. Apply setup/CAS improvements to docs drift, but preserve all checks
**Change**
For `docs-graph-drift.yml`:
- Replace the executable cache and shell installer at `:63-76` with the same digest-pinned image/CAS setup.
- Keep materialization, generator build, regeneration test, and golden test at `:72-87`.
- Move the generator to its debranded capability home when reorganization authority provides that target; do not introduce a compatibility alias with deprecated naming.
**Saving**
Approximately **20–60 seconds per triggered run**; zero required-context wall impact because this workflow is feedback-only and path-filtered.
**Rigor preserved because**
The exact materialization, build, regeneration, and golden-test sequence remains. Do not remove the explicit build until graph-level target equivalence proves it redundant.
## Hostile QA result
| Scenario | Verdict |
|---|---|
| Revision/timing consistency | PASS after selecting local `origin/dev` |
| Deleted/non-Rust affected inputs | **FAIL — selector hole** |
| Uncertain FULL fallback | **FAIL — full tests omitted** |
| Merge-base isolation | **FAIL — candidate ignored files can survive checkout** |
| Cold/warm bit identity | BLOCKED — warm license false and ambient toolchain incomplete |
| Matrix artifact reuse | PASS; already optimized |
| Blanket materialization reuse | REJECTED; weakens independent detection |
| Postgres skipping/fixture reduction | REJECTED; violates hard constraint |
**ULTRAQA COMPLETE, read-only:** no files or runtime state were changed. Optimization cutover should stop at the three P0 affected-set/baseline defects; the safe high-value implementation order is hermetic image/CAS → exact gate-fleet collapse → owned-Rust materialization/Postgres optimization → runner sizing.
258,352
Verdict: the productized CI is under-optimized—51 runner executions duplicate substantial work, while three rigor defects make affected scoping and warm caching unsafe to credit today.
### Audit-base correction
The dirty cwd is at `c52bdb09e`; local `origin/dev@d7568c731` is 424 commits ahead and is the only local revision matching the supplied Postgres jobs/timings. References below therefore use the local `origin/dev:` blob.
| Fact | Verified productized state |
|---|---|
| Jobs | 11 YAML jobs, expanding to **51 runner executions** because the gate matrix has 41 entries (`oya-ci-required.yml:125-165`) |
| Buck2 setup | 10 call sites, expanding to **50 installs** |
| Rust setup | Roughly **50 normal `rustup toolchain install` executions**, plus a conditional merge-base install |
| Materialization | **5 owned-Rust executions**; matrix reuse is already implemented |
| `buck-out` | 2 restores, 1 trusted-dev save |
| Binding scope | `buck2 test //ci/...` (`:489-497`) plus a binding workspace affected-set job (`:543-870`) |
| Stale `.claire` graph | Historical; removed on 2026-06-09 and absent from both HEAD and `origin/dev` |
| Old scripts | `buck2-affected-gate.sh` survives but is unused; the shell materializer is already gone; `install-buck2.sh` remains heavily used |
The supplied timings are treated as measured inputs; local git has no workflow-duration logs.
## Ranked optimization plan
### 1. Repair affected-set soundness before using it to remove full work
**Change**
The current Rust successor is already binding; do not “flip” the historical shell driver. Fix these defects first:
- `ci/facade/affected-target-set/src/lib.rs:365-378` treats deleted non-Rust files as irrelevant. First safe version: every deletion, rename, copy, type change, and submodule change selects FULL. Later, query both isolated base and head graphs and union their owners.
- Replace the narrow owner-required set at `affected-set-policy.json:21-25`. Every changed path must map to a Buck owner or an explicit synthetic dependency; otherwise FULL.
- `src/main.rs:823-836` returns green after a PR FULL build without running full tests. Add a full test-health ratchet: run base/head full test reports and block new failures. A full fallback that only builds is “checking less,” not an optimization.
- Move merge-base evaluation out of the candidate filesystem. Workflow `:641-642` creates ignored candidate JSON, then `git checkout` at `:816-820` leaves it in place; broad `**/*.json` BUCK globs can consume it. Use a clean detached merge-base worktree and merge-base-owned Rust materialization.
- Replace workflow shell/Python baseline handling at `:671-835` with the owned Rust selector/baseline component. Validate commit, toolchain, universe digest, report completeness, and provenance—not merely a non-empty `results` object.
- Retire unused `infra/ci/buck2-affected-gate.sh`; do not extend it.
The safe selector is:
`selected = always_run ∪ owners(base,head,diff) ∪ rdeps(universe, owners ∪ synthetic_dependencies)`
Any merge-base, diff, owner, graph, policy, or query uncertainty sets `selected = universe`. Buildfiles, Buck config, prelude, toolchains, dependencies, workflow/setup, selector, materializer, generator, and global-corpus changes also select FULL. Push, merge-group, and dispatch remain full.
Only after this is proven should the selector replace the unconditional `buck2 test //ci/...`. Whole-tree scanners remain `always_run` until their implicit inputs become declared graph or synthetic edges.
**Saving**
No saving may be credited until shadow scoped/full runs show zero target or verdict mismatches. Thereafter, approximately **50–80 runner-seconds** on eligible small PRs; FULL-trigger changes save zero. Current end-to-end wall impact is near zero while the 501/497-second jobs dominate.
**Rigor preserved because**
Every changed path is accounted for; the scoped set is mechanically a superset; every uncertainty executes the exact full build-and-test universe. Shadow comparison validates the proof but does not replace fail-closed derivation.
---
### 2. Replace archive warmth and repeated setup with a hermetic image plus Buck2-native CAS
**Change**
- First make toolchains actual immutable inputs. `toolchains/BUCK:8-28` currently resolves ambient rustup and `/usr/bin/clang`; those compiler bytes are outside the declared graph.
- Introduce a digest-pinned, attested declarative CI image containing the exact Buck2 binary, Rust toolchain/components, C/C++ toolchain, and PostgreSQL client. Its successor belongs under the debranded `ci/` capability.
- Retire `infra/ci/install-buck2.sh` and the repeated installer/rustup steps. A composite action alone merely deduplicates YAML and saves essentially no runtime.
- After hermetic closure, replace the multi-gigabyte archive blocks at workflow `:463-470`, `:629-636`, and `:883-889` with authenticated NativeLink/CAS compile-action reuse across warm-eligible jobs.
- Do **not** add `buck-out` archive restores to the other jobs. The current archive is documented as approximately 5.78 GiB compressed and 12–15 GiB restored (`:437-440`).
- Never cache live-test verdicts. Tests still execute; CAS supplies only content-addressed build artifacts.
**Saving**
- At 20–40 seconds per 41 matrix legs: **13.7–27.3 runner-minutes**.
- Every 10 seconds removed from 50 repeated setups saves another **8.3 runner-minutes**.
- Likely **1–3 minutes critical-path improvement** if compilation/setup materially contributes to the 497/501-second jobs; step telemetry must confirm.
**Rigor preserved because**
Action identity must include image/platform digest, Buck2 revision, prelude, Rust/C++ toolchains, graph configuration, and declared sources. Fork/untrusted execution remains cold; trusted post-merge is the writer. Misses recompute locally, digest mismatches quarantine and fail, and a binding cold-versus-warm byte-digest canary disables all warm reads on any mismatch.
The repository already encodes that contract, but `specs/cache-warm-license.json:6-7` is still false. Warm CAS must not activate before the first green canary.
**Current slow-and-weak configuration:** the pipeline pays for multi-gigabyte warmth while telemetry declares `CACHE_MODE=bypass` (`oya-ci-required.yml:504-511`), and ambient compiler identity is not rigorously closed. This is a false-green risk, not a proven observed false green.
---
### 3. Eliminate duplicate gate execution with an exact, surface-all gate fleet
**Change**
The 41 matrix legs (`:114-196`) run named CI test pairs, while `buck2 test //ci/...` (`:489-497`) runs the CI test universe again.
Replace both with an owned Rust gate-fleet scheduler under `ci/facade/gate-fleet`:
- Query and hash the exact existing `//ci/...` test-target universe.
- Compare it against the registered matrix target union; refuse cutover if they differ.
- Partition the exact union into approximately 8 weighted shards.
- Execute every target despite sibling failures and emit per-gate result packets/annotations.
- Make the fan-in red if any target is missing, duplicated, unexecuted, or failed.
- Dual-run old and new topology until target lists and verdicts are identical.
**Saving**
The supplied ten 95–99-second legs plus the 148-second catalog leg already represent a lower bound of roughly **18.5 runner-minutes** of duplicated cost. Going from 41 to eight shards also removes 33 setup/startup sequences.
Current wall saving may be nearly zero because `215s producer + 148s catalog = 363s`, below the 501-second Postgres critical path. It becomes important after Postgres/freshness optimization.
**Rigor preserved because**
The exact same test target union executes, with surface-all failure reporting and mechanically enforced coverage. No matrix leg is deleted merely because it appears redundant.
---
### 4. Optimize materialization internally; preserve independent attestations
**Change**
Producer reuse already exists: `producer-regen` creates the artifact at `:50-84`, and matrix legs download it at `:176-181`.
Further optimize the owned-Rust materializer by:
- Scanning git/history once and sharing the immutable scan across emitters.
- Running byte-independent emitters concurrently.
- Separating candidate-face generation from merge-base frozen-baseline generation.
- Letting mere-reader jobs consume a candidate-SHA-bound artifact while independently producing private frozen baselines.
- Retaining independent regeneration in registry-drift (`:238-263`) and baseline-ratchet (`:269-300`).
Reject “materialize once and feed every job.” A detector consuming the artifact it attests is self-referential and weaker.
**Saving**
Freshness + registry-drift + producer total 1,102 supplied seconds. A profiling-confirmed 25–40% materializer reduction would save approximately **275–441 runner-seconds**, potentially **124–199 seconds** from the freshness critical path.
**Rigor preserved because**
Every emitted byte is compared against the existing implementation during qualification; artifacts bind candidate SHA, generator/policy digests, path list, sizes, and content hashes. Missing or mismatched metadata is red. Regeneration/determinism detectors remain independent.
---
### 5. Optimize live Postgres execution without skipping or sharing coverage
**Change**
For both lanes (`:930-1065`, `:1067-1186`):
- Replace mutable `postgres:16` (`:936`, `:1073`) with a reviewed immutable digest. No digest is present locally, so this audit will not invent one.
- Remove repeated mutable `apt-get` client installation (`:971-976`, `:1108-1113`) via the pinned CI image.
- Replace inline bootstrap shell with owned Rust while preserving every role, grant, migration, schema, and provenance field.
- Submit the existing six and four targets in one Buck2 invocation per lane while retaining `--local-only`, `--num-threads 1`, and `RUST_TEST_THREADS=1`.
- A later concurrency experiment may allocate a fresh isolated database per target and run targets concurrently, but only after serial-versus-parallel verdict parity is proven.
Reject:
- Cross-job service-container reuse: it sacrifices isolation and turns two parallel lanes into a shared-state failure domain.
- Smaller fixtures: reduced coverage.
- Skipping the jobs for “unrelated” diffs: under the literal no-skipped-check constraint, live environment behavior can change independently of the source cone. Give this proposal zero credit.
**Saving**
CAS, prewarmed client/toolchain, and fewer Buck2/bootstrap round trips should target **120–360 combined runner-seconds**, with perhaps **60–200 seconds** off the 501-second critical lane if profiling confirms setup/compilation dominance.
**Rigor preserved because**
All ten test targets, fixtures, roles, migrations, environment variables, and fresh live-database executions remain. No live verdict is cached.
---
### 6. Benchmark larger runners only after CAS and topology fixes
**Change**
Benchmark identical digest-pinned x86_64 images on larger runners for compile-bound full Buck2, affected FULL, freshness, and materializer shards. Do not blanket-upsize the matrix or Postgres lanes.
**Saving**
Plausible **20–45% cold compile wall reduction**. Cost is favorable only when:
`new_duration / old_duration < old_price / new_price`
**Rigor preserved because**
Image, architecture, toolchain, target set, test concurrency, and output digests remain identical. Reject any runner change whose cold differential produces a different artifact or verdict manifest.
---
### 7. Apply setup/CAS improvements to docs drift, but preserve all checks
**Change**
For `docs-graph-drift.yml`:
- Replace the executable cache and shell installer at `:63-76` with the same digest-pinned image/CAS setup.
- Keep materialization, generator build, regeneration test, and golden test at `:72-87`.
- Move the generator to its debranded capability home when reorganization authority provides that target; do not introduce a compatibility alias with deprecated naming.
**Saving**
Approximately **20–60 seconds per triggered run**; zero required-context wall impact because this workflow is feedback-only and path-filtered.
**Rigor preserved because**
The exact materialization, build, regeneration, and golden-test sequence remains. Do not remove the explicit build until graph-level target equivalence proves it redundant.
## Hostile QA result
| Scenario | Verdict |
|---|---|
| Revision/timing consistency | PASS after selecting local `origin/dev` |
| Deleted/non-Rust affected inputs | **FAIL — selector hole** |
| Uncertain FULL fallback | **FAIL — full tests omitted** |
| Merge-base isolation | **FAIL — candidate ignored files can survive checkout** |
| Cold/warm bit identity | BLOCKED — warm license false and ambient toolchain incomplete |
| Matrix artifact reuse | PASS; already optimized |
| Blanket materialization reuse | REJECTED; weakens independent detection |
| Postgres skipping/fixture reduction | REJECTED; violates hard constraint |
**ULTRAQA COMPLETE, read-only:** no files or runtime state were changed. Optimization cutover should stop at the three P0 affected-set/baseline defects; the safe high-value implementation order is hermetic image/CAS → exact gate-fleet collapse → owned-Rust materialization/Postgres optimization → runner sizing.
