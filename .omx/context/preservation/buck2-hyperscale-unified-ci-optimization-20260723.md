# Buck2 hyperscale unified CI/CD optimization evidence packet

Date: 2026-07-23

Status: `NON_AUTHORITY`

Planning state: `HOLD(Planning)`

Dispatch state: `NO_DISPATCH`

Decision state: `PREPARATORY_EVIDENCE_ONLY`

This packet is a read-only research and current-state consolidation. It is not
an ADR, an accepted plan, a roadmap, a C13 council receipt, a C14 dissent
receipt, a C15 exit receipt, implementation authority, merge authority, or
permission to change the protected CI contract.

The controlling rule remains:

> Preserve one unified CI/CD build/test process and one canonical blocking
> `oya-ci-required` result. Deterministic graph, hermeticity, cache, remote
> execution, and scheduling improvements come first. AI/ML may later provide
> bounded advisory evidence inside that process; it is neither a separate lane
> nor an authority source.

No implementation, migration, selective-admission cutover, NativeLink rollout,
Buck2 Change Detector adoption, or roadmap wave may dispatch from this packet.
Stage-1 must independently clear its controlling authority and evidence gates
before planning can exit `HOLD(Planning)`.

## 1. Evidence boundary and receipt

### 1.1 Evidence classes

- `LOCAL_OBSERVATION`: byte-bound inspection of the named local file on
  2026-07-23. The root checkout was not assumed clean and is not treated as a
  protected-branch oracle.
- `UPSTREAM_PRIMARY`: the named Buck2 or Buck2 Change Detector official
  documentation, repository, release, commit, workflow, or issue.
- `RESEARCH_PRIMARY`: the named first-party engineering report, research
  publication, or specification.
- `DERIVED_PROPOSAL`: an engineering inference in this packet. It has no
  authority or dispatch effect.
- `UNKNOWN`: a fact not established by the evidence freeze. Unknowns fail
  closed anywhere they could omit required work or produce a green admission
  result.

### 1.2 Local snapshot receipt

The root checkout reported:

- commit: `c52bdb09ea337de103b05317de0c120f2b7a3e45`
- tree: `e4352254e5bf63411c8b4068db3d19210dc661f0`

These object identifiers describe the checkout observation only. They are not a
claim about the current protected `dev` object, GitHub ruleset state, or a clean
working tree. The exact inspected bytes are instead bound below:

| Input | Lines | SHA-256 |
| --- | ---: | --- |
| `.buckconfig` | 28 | `d939662c2522a9243ccf589d4821a1887eab82ad1721bd09c142dc63b512032a` |
| `.github/workflows/oya-ci-required.yml` | 450 | `184ed223547a7e37cc475ac60139a4819a9a542ed43c45efa266d040a4df6462` |
| `infra/ci/buck2-affected-gate.sh` | 137 | `11f8183e3ad0a29b91a184964c0683f2a1606f1f336f695e1efa1bb271d3e661` |
| `docs/decisions/ADR-0392-buck2-canonical-build-graph.md` | 95 | `c3d5a09517668b83e0d1c1c8ad3f28b867a6a42572f8e3bfab75febc1c95d2bb` |
| `docs/decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md` | 406 | `2d12cb19368801b533fb3b6713d642bd51011fe006907f9119161dc54e2e0832` |

Any change to an input byte, protected predecessor, Buck2 binary, prelude,
toolchain, cell configuration, target graph, selector, or policy creates a new
evidence generation. Findings in this packet must not be silently carried
forward.

## 2. Executive finding

The evidence supports a strong direction but not an implementation dispatch:

1. Keep GitHub Actions as the single live CI orchestrator and
   `oya-ci-required` as the single protected fan-in identity unless qualified
   authority explicitly amends that contract.
2. Treat Buck2 as the canonical build graph and use its native graph,
   configuration, toolchain, query, observability, cache, and execution
   capabilities instead of translating Cargo-workspace or Bazel-repository
   concepts into it.
3. Do not treat the legacy workflow comment about graph pollution as a current
   finding; obtain fresh, pinned base-and-candidate graph receipts and prove
   affected-set correctness before selective execution can affect admission.
4. Build exact base-and-candidate graph snapshots, not a candidate-only
   filename heuristic, before attempting million-line selective CI.
5. Introduce remote CAS/execution only after hermeticity, tenant/trust
   isolation, cold-versus-warm equivalence, and provenance are proven.
6. Preserve full-universe execution as the automatic fallback for unknown,
   stale, unsupported, or failed selection states.
7. Let AI/ML rank, forecast, generate candidates, or triage only within
   deterministic safety envelopes. It may not decide that a target is
   unnecessary, suppress a failure, quarantine a test without governed
   disposition, or make `oya-ci-required` green.

Current result: `BLOCKED_NOT_READY_FOR_SELECTIVE_ADMISSION`.

Reason: the current affected driver is explicitly advisory; its workflow carries
a stale legacy comment about a `.claire/worktrees` package, but this packet has
no fresh pinned evidence that the current whole graph is polluted. Current
selection loses change semantics such as rename/delete/copy, no protected
base/diff graph receipt exists, no live remote CAS/RBE receipt exists, and the
Stage-1 qualified-human authority chain is incomplete.

## 3. Current local CI/CD facts

### 3.1 Controlling shape

`ADR-0515` and `.github/workflows/oya-ci-required.yml` encode one GitHub Actions
pipeline and one zero-build fan-in job named `oya-ci-required`. The fan-in
depends on:

- an 18-entry Cargo gate matrix;
- freshness;
- registry drift;
- the cloud-CI firewall;
- generated-output diff policy;
- Buck2;
- app-shell code generation; and
- PR reviewer evidence.

The workflow runs for `push` to `dev`, `pull_request` to `dev`,
`merge_group`, and manual dispatch. Its fan-out is surface-all and its fan-in
fails unless every named required job succeeds.

This packet does not propose a second CI authority, a second required status, or
an “AI CI lane.”

### 3.2 Repeated work

The inspected workflow independently performs full checkout, Buck2
installation, and/or generated-face materialization across several jobs. The
18 Cargo matrix legs each:

1. check out full history;
2. install Buck2;
3. materialize cloud-CI generated faces; and
4. run one serialized Cargo crate test.

The producer job uploads `accounting-faces`, but the inspected workflow contains
no `download-artifact` consumer for that artifact. It is evidence, not a shared
job input. Several other jobs rematerialize independently.

The Cargo matrix and the binding Buck2 lane also execute overlapping gate
behavior. That redundancy is currently a safety bridge. It must not be removed
until target, verdict, environment, and corpus parity are measured against exact
candidate objects.

### 3.3 Buck2 path today

The binding Buck2 step executes:

```text
buck2 test //cloud/cloud-ci/...
```

The subsequent affected-set driver invokes
`infra/ci/buck2-affected-gate.sh`, but is marked `continue-on-error: true`.
The workflow contains a legacy comment referring to
`.claire/worktrees/.../oya-payroll-run-usecase` and a whole-graph query. That
comment is not fresh, pinned evidence that the current graph contains that
package or that a query is polluted. No current graph-pollution claim is made
here. The affected path remains advisory because `continue-on-error: true` is
an observed workflow control, not because the legacy comment is accepted as
proof.

The affected script:

- calculates a Git merge base;
- uses `git diff --name-only`;
- classifies Rust and selected Buck graph files;
- runs Buck2 `owner()` queries;
- expands `rdeps(//..., owners)`;
- fails on a missing base/head, query error, missing owner, or empty closure; and
- builds then tests the resulting target list.

Useful fail-closed behavior already exists, but the selection semantics are not
yet sufficient for hyperscale admission:

- `--name-only` loses add/delete/rename/copy/type-change semantics;
- candidate-only `owner()` and `rdeps()` cannot recover targets deleted from the
  candidate graph;
- classification is narrower than every graph-, configuration-, toolchain-,
  policy-, generator-, and external-cell-affecting input;
- one default configuration cannot prove all required platform/configuration
  variants; and
- there is no immutable selector/graph/plan receipt joined into the fan-in.

### 3.4 Cache and execution state

The Buck2 job restores `buck-out` through `actions/cache` with a key derived
from OS, `.buckconfig`, `toolchains/BUCK`, and `Cargo.lock`. This is an
interim runner-level artifact cache.

The inspected `.buckconfig` contains no remote-execution or shared-CAS endpoint.
No current observation in this packet establishes a live NativeLink deployment,
cross-runner action-cache hit, tenant isolation, cache provenance, eviction
behavior, or cold-versus-warm equivalence. `ADR-0392` names NativeLink as a
target architecture; that does not establish operational adoption.

Therefore all cache/RBE performance, capacity, hit-rate, and correctness claims
remain `UNPROVEN`.

### 3.5 Generated faces

Generated faces remain producer-owned. Optimization must never turn
`*.generated.json` into a hand-edited merge surface. The safe sequence is:

```text
canonical source -> pinned materializer -> immutable output digest
-> declared Buck2 input -> gate verdict -> receipt
```

Materialization deduplication is permissible only after consumers prove they
read one candidate-bound immutable artifact and registry-drift/freshness checks
retain their independent failure semantics.

## 4. Buck2-native operating model

### 4.1 Cells are configuration and trust boundaries

Buck2 has one project rooted at the outer `.buckconfig`. Its `[cells]` section
maps directory trees into cells. Cells are not Bazel repository rules, and they
are not Cargo workspaces. The deprecated `[repositories]` spelling is an alias
for cells, not a repository-rule engine.

Official grounding:

- key concepts:
  https://buck2.build/docs/concepts/key_concepts/
- `.buckconfig`, cells, aliases, and deprecated `repositories`:
  https://buck2.build/docs/concepts/buckconfig/
- external cells:
  https://buck2.build/docs/users/advanced/external_cells/

The observed cell map is:

```text
root        = .
prelude     = prelude
toolchains  = toolchains
none        = none
third-party = third-party
```

It also aliases compatibility names to `prelude` or `none`, uses the bundled
prelude as an external cell, assigns the default target platform detector to
the active cells, and selects `prelude//platforms:default` as the execution
platform.

Derived topology rule:

- **cell**: independently configured namespace, prelude/toolchain boundary,
  external-origin boundary, or trust/ownership boundary;
- **package**: graph loading and ownership unit containing a `BUCK` file;
- **target**: independently buildable, testable, cacheable, visible unit;
- **provider/artifact**: typed dependency interface and concrete produced
  output;
- **configuration**: target platform, modifiers, constraints, transitions, and
  execution platform applied to a target.

Do not create one cell per crate, service, team, or directory merely to imitate
Cargo workspaces. Product ownership should normally be expressed through
packages, targets, visibility, metadata, and policy. Add a cell only when the
configuration or trust boundary is genuinely independent and its cross-cell
cost is justified.

### 4.2 Capabilities to exploit deliberately

| Capability | Hyperscale use | Guardrail |
| --- | --- | --- |
| Cells, aliases, external cells | Stable configuration, prelude, toolchain, third-party, and trust boundaries | No cell-per-package sprawl; pin external origin |
| Providers and declared artifacts | Typed, analyzable dependency and output contracts | No ambient undeclared filesystem/network inputs |
| `visibility` and `within_view` | Enforce architectural direction and reduce accidental graph coupling | Narrow by default; exceptions receipt-bound |
| Constraints, `select()`, modifiers, transitions | Represent platform/configuration variants | Selection must cover every required variant |
| Toolchains and execution platforms | Reproducible compiler/linker/runner selection | Toolchain and platform fingerprints enter every receipt |
| Depfiles | Precise incremental action inputs | Audit under-declaration with clean/cold comparisons |
| Remote execution and CAS | Cross-runner locality and parallel execution | Tenant/trust isolation, provenance, cold canaries |
| `dynamic_output_new` and anonymous targets | Runtime-discovered or generated subgraphs when static declaration is impossible | Use only where necessary; version and test graph expansion |
| Content-based paths | Reduce path-instability effects where supported | Treat feature/version changes as graph-generation changes |
| `uquery`, `cquery`, `aquery` | Unconfigured, configured, and action-graph diagnosis | Do not treat experimental output formats as stable APIs |
| BXL | Repository-specific analysis and controlled orchestration | Pin scripts and schema their machine-readable output |
| Build reports, event logs, critical path | Exact observability and performance attribution | Retain raw log plus normalized receipt; event format may evolve |
| Starlark lint/typecheck/profile | Keep build logic safe and fast | Gate build-definition changes like product code |
| Isolation directories | Intentionally separate Buck2 daemon/output state | Limit cardinality; avoid destroying daemon/cache locality |

Official grounding:

- architecture:
  https://buck2.build/docs/concepts/architecture/
- visibility:
  https://buck2.build/docs/concepts/visibility/
- configurations:
  https://buck2.build/docs/concepts/configurations/
- modifiers:
  https://buck2.build/docs/concepts/modifiers/
- configuration transitions:
  https://buck2.build/docs/rule_authors/configuration_transitions/
- toolchains:
  https://buck2.build/docs/rule_authors/writing_toolchains/
- depfiles:
  https://buck2.build/docs/rule_authors/dep_files/
- remote execution:
  https://buck2.build/docs/users/remote_execution/
- BXL:
  https://buck2.build/docs/bxl/
- query language:
  https://buck2.build/docs/concepts/buck_query_language/
- build reports:
  https://buck2.build/docs/users/build_observability/build_report/
- logs:
  https://buck2.build/docs/users/build_observability/logging/
- isolation directories:
  https://buck2.build/docs/concepts/isolation_dir/

### 4.3 Version discipline

At the evidence freeze, the official Buck2 releases surface exposed dated
prereleases and no stable release tag. The latest dated OSS prerelease observed
in the gathered upstream evidence was 2026-07-15.

Source:

https://github.com/facebook/buck2/releases

Consequences:

- pin the exact Buck2 asset digest, not only a moving release label;
- bind the Buck2 commit/version, prelude identity, rule definitions, and
  `.buckconfig` digest into graph and execution receipts;
- rehearse upgrades in a distinct generation with base/candidate graph diff,
  clean/cold build, warm build, target/verdict parity, and rollback;
- never infer support from Meta-internal deployment if the OSS release and
  bundled prelude do not expose it; and
- treat experimental query or log surfaces as versioned inputs behind an
  adapter, not as repository-wide stable contracts.

## 5. Buck2 Change Detector assessment

### 5.1 What it is

The official project performs target determination from changed files and
base/diff Buck graphs. Its published surfaces include:

- `targets`: changed-file-to-target support;
- `btd`: Buck target determination; and
- `supertd`: a higher-level target-determination surface.

The caller still performs the actual build and test. The detector is not a CI
or admission system.

Primary sources:

- repository:
  https://github.com/facebookincubator/buck2-change-detector
- repository README at the evaluated source:
  https://github.com/facebookincubator/buck2-change-detector/blob/7a632871f376cb04bc007829c8dd7d536078ae4a/README.md
- `btd` README at the evaluated source:
  https://github.com/facebookincubator/buck2-change-detector/blob/7a632871f376cb04bc007829c8dd7d536078ae4a/btd/README.md

### 5.2 Evaluated source state

The gathered upstream evidence identified:

- repository head:
  `7a632871f376cb04bc007829c8dd7d536078ae4a`;
- head date: 2026-07-20;
- head change: adds `--vcs git`;
- public moving `latest` prerelease: points to earlier source
  `e8de756...`, before the Git support;
- commit-verification observation: the evaluated GitHub commit page did not
  present a verified signature; and
- release workflow: upstream publishes build artifacts, but Oyatie would still
  need an exact source pin, independent build, checksum, provenance, and
  license/security review.

Primary sources:

- evaluated commit:
  https://github.com/facebookincubator/buck2-change-detector/commit/7a632871f376cb04bc007829c8dd7d536078ae4a
- moving prerelease:
  https://github.com/facebookincubator/buck2-change-detector/releases/tag/latest
- release workflow:
  https://github.com/facebookincubator/buck2-change-detector/blob/7a632871f376cb04bc007829c8dd7d536078ae4a/.github/workflows/release_btd.yml

Do not install the moving `latest` prerelease for a Git pilot and assume it
contains the evaluated Git behavior. A pilot must pin the exact source commit,
build it independently, record toolchain/dependency lock and output digest, and
retain source-to-binary provenance.

### 5.3 Known correctness gaps

Two upstream open issues are admission-relevant:

- issue #12 reports missed toolchain/PACKAGE-class changes:
  https://github.com/facebookincubator/buck2-change-detector/issues/12
- issue #9 reports missed symlink changes:
  https://github.com/facebookincubator/buck2-change-detector/issues/9

External-cell behavior across Oyatie's exact topology is not established by the
gathered documentation. These gaps preclude use as the sole producer of
`oya-ci-required`.

Verdict:

`ADVISORY_SHADOW_PILOT_ONLY`

Permissible preparation after Stage-1 planning authority, not before:

- reproduce upstream tests from the exact pin;
- add Oyatie-specific delete/rename/copy/symlink/PACKAGE/toolchain/cell/
  configuration/external-cell regressions;
- compare its target set against a deterministic full-run oracle;
- record false negatives, false positives, runtime, and graph-size behavior;
- run it in shadow without reducing executed targets; and
- fail over to the complete universe on every unknown or error.

Not permissible:

- using its empty output as proof that no work is affected;
- allowing it to suppress an always-run gate;
- letting it select only the candidate graph;
- relying on the moving prerelease tag;
- treating its own test suite as Oyatie admission proof; or
- making it a required status independent of the unified pipeline.

## 6. Million-line monorepo graph and snapshot logistics

### 6.1 Canonical graph identity

At millions of lines and many configurations, the repository path list is not
the dependency graph. The minimum deterministic model is a content-addressed
Merkle hierarchy:

```text
repository generation
  -> cell configuration roots
    -> package definition roots
      -> configured target roots
        -> action/input/output roots
```

The repository-generation root must bind at least:

- exact protected base commit and tree;
- exact projected candidate commit/tree or synthetic merge-group tree;
- Buck2 binary digest and version;
- bundled or pinned prelude identity;
- every `.buckconfig` and cell alias/external-cell mapping;
- all Starlark build definitions and extensions;
- all toolchain and execution-platform identities;
- target platform, constraints, modifiers, and transitions;
- third-party graph and dependency-lock identities;
- generated-source/materializer identities;
- selector binary/source and policy version;
- always-run registry and graph-boundary policy; and
- schema/canonicalization version for every emitted receipt.

Any mismatch produces a new generation. Receipts from different generations
must not join.

### 6.2 Change capture

Selection input must use a NUL-safe, rename/copy-aware name-status representation
against the exact protected merge base or merge-train predecessor. The canonical
change record must retain:

- old path;
- new path;
- change kind;
- similarity score where applicable;
- file mode/type change;
- symlink target change;
- submodule/external-source change;
- content digest before and after; and
- whether each path exists in the base and candidate graph.

Deletes and renames require base-graph ownership. Additions require candidate
ownership. Copies may require both. Configuration, toolchain, prelude, cell,
policy, generator, and external-source changes can invalidate graph regions
that have no source-file owner.

### 6.3 Sound affected closure

For every required configuration/platform tuple, the candidate affected set is
the union of:

1. direct base-graph owners;
2. direct candidate-graph owners;
3. reverse dependencies in the base graph;
4. reverse dependencies in the candidate graph;
5. generated-source and code-generation consumers;
6. toolchain, platform, transition, modifier, and configuration dependents;
7. external-cell and third-party dependents;
8. policy, security, governance, selector-integrity, and evidence gates;
9. tests associated through declared metadata or audited mapping; and
10. the immutable always-run registry.

Every inclusion and exclusion needs a reason code. “No owner” is not a safe
exclusion for a graph-affecting path.

### 6.4 Partition and storage

Recommended derived storage model:

- shard graph snapshots by cell, then package, target, and configuration;
- content-address every shard and build an immutable root manifest;
- deduplicate unchanged base/candidate shards;
- keep adjacency indexes for forward and reverse edges;
- retain typed cross-cell edges explicitly;
- stream target sets through manifests or argfiles, not command-line expansion;
- cap query memory through partitioned joins and spill-safe storage;
- version the graph schema and retain migration/rollback readers; and
- garbage-collect only after receipt retention and merge-train reachability
  permit it.

No graph database or file format is selected by this packet.

### 6.5 Merge-train projection

For a merge train `P1, P2, ... Pn`, candidate `Pn` must be evaluated against
the projected tree after `P1..P(n-1)`, not against the old protected base.

```text
protected base
  -> projected P1
  -> projected P1+P2
  -> ...
  -> projected P1+...+Pn
```

Each prefix has its own graph root and receipt. If a predecessor is changed,
removed, reordered, or fails admission:

- every dependent projected tree changes;
- all downstream selector, plan, execution, review, and fan-in receipts become
  stale;
- graph/test computation may rerun in parallel for the new projections; but
- landing remains serial against the actual protected predecessor.

Never trim Git history or approximate the predecessor merely to save compute.

### 6.6 Scheduling and sharding

Once the sound target universe is known:

- shard by declared resource class, historical duration, platform, and
  exclusivity constraints;
- use critical-path-aware scheduling for graph bottlenecks;
- keep deterministic shard manifests so retries run the same work;
- separate infrastructure retries from test retries in evidence;
- reserve capacity for always-run and full-fallback jobs;
- reuse CAS objects, not mutable shared workspaces;
- isolate tenant/trust domains in CAS namespaces and execution credentials; and
- cap fanout by the minimum capacity of runners, CAS, network, reviewers,
  appliers, and the protected admission slot.

Large affected closures must scale out or fall back to the full universe. They
must not be silently truncated.

## 7. One unified CI/CD process

The proposed internal topology, still non-authoritative, is:

```text
exact candidate event
  -> authority/config snapshot
  -> deterministic base + candidate graphs
  -> sound affected-universe proposal
  -> always-run union
  -> deterministic shard plan
  -> build/test/materialize fan-out
  -> evidence and provenance join
  -> one oya-ci-required fan-in
```

Full and selective execution are modes inside the same canonical process. They
must share:

- the same protected candidate identity;
- the same gate implementations;
- the same hermetic toolchains and execution platforms;
- the same generated-source boundary;
- the same evidence schema;
- the same failure semantics;
- the same protected fan-in identity; and
- the same post-merge verification obligations.

The process may decide to execute the full universe. It may not decide to
create a weaker second CI.

## 8. Deterministic-first promotion sequence

Every phase below remains `PREPARED_NOT_DISPATCHED` until controlling Stage-1
authority authorizes roadmap planning.

### P0 — Authority and invariant freeze

Required before code work:

- ratified authority amendment if selective admission changes the content of
  `oya-ci-required`;
- exact always-run registry;
- exact supported change/configuration/cell classes;
- exact fallback policy;
- exact receipt and retention contract;
- exact protected-base/merge-group semantics;
- security, custody, legal/JCR, veto, and affected-party dispositions; and
- rollback and incident authority.

### P1 — Current graph correctness

- obtain fresh, pinned base-and-candidate graph receipts before any disposition
  of the alleged stale worktree graph pollution; the legacy workflow comment is
  insufficient evidence on its own;
- make every tracked source/build definition resolve through the intended cell
  and package ownership model;
- prove clean checkout graph loading;
- add regression tests for add/delete/rename/copy/symlink/type changes;
- add cell, external-cell, prelude, toolchain, PACKAGE/BUCK, configuration,
  modifier, transition, generated-source, and policy invalidation tests; and
- keep full `//cloud/cloud-ci/...` binding execution.

### P2 — Exact receipt substrate

- materialize base and candidate graph snapshots;
- emit canonical selector, plan, execution, parity, and fan-in receipts;
- bind all receipts to exact objects, policies, tools, and configurations;
- independently validate canonicalization and hash joins; and
- prove stale/mixed-generation receipts fail closed.

### P3 — Shadow selection

- compute affected sets but execute the complete universe;
- compare the proposed set against full-run failures and realized dependencies;
- retain every false-negative candidate as a blocking incident;
- compare the current script, any Buck2-native adapter, and optionally the
  pinned Change Detector without allowing any to reduce work; and
- measure compute overhead and graph snapshot scalability.

### P4 — Dual execution

- execute both selected and full universes for representative changes;
- compare target lists, configured variants, verdicts, logs, and artifacts;
- inject detector, graph, cache, CAS, network, and runner failures;
- prove automatic full fallback; and
- run warm/cold and local/remote equivalence.

### P5 — Stratified promotion

Promotion may be considered independently by cell and change class only after:

- zero observed false negatives;
- every supported class has complete corpus coverage;
- every unsupported or unknown class deterministically falls back;
- receipts are complete and independently replayable;
- the full oracle remains scheduled; and
- qualified authority accepts the residual risk.

### P6 — Selective admission candidate

Only an accepted authority amendment may allow the selected set to influence
the content of the single required context. Initial promotion must preserve:

- always-run gates;
- scheduled full-universe runs;
- all-platform/configuration coverage;
- cold-cache canaries;
- random and risk-weighted full comparisons;
- incident-triggered automatic reversion to full mode; and
- a one-step rollback to the previous canonical pipeline generation.

### P7 — Remote CAS/RBE expansion

NativeLink or any alternative enters only after:

- hermetic input/output proof;
- tenant/trust isolation;
- source-to-action and action-to-output provenance;
- authenticated worker identity;
- capacity, queue, and eviction measurements;
- poisoning and cross-tenant negative tests;
- cold equals warm and local equals remote comparisons;
- observability and rollback rehearsals; and
- qualified operational custody.

No speedup number is accepted until measured against exact comparable
generations.

## 9. Fail-closed fallback matrix

The complete required target/configuration universe is the automatic fallback
for:

- selector process error, timeout, crash, or incompatible version;
- empty result for a non-empty or graph-affecting change;
- unresolved owner;
- missing base or candidate graph shard;
- graph schema mismatch;
- protected-base or merge-train movement;
- receipt hash/signature/canonicalization failure;
- unknown change kind;
- symlink, submodule, external-cell, or generated-source uncertainty;
- `.buckconfig`, prelude, Starlark, cell, toolchain, platform, constraint,
  modifier, transition, or execution-platform change without a proven rule;
- query or action-graph API incompatibility;
- detector disagreement;
- cache/CAS integrity incident;
- missing always-run registry entry;
- unrecognized policy or authority generation;
- target-set size over a proven safe selector threshold; and
- any explicit safety invariant violation.

If the full fallback cannot execute, `oya-ci-required` is red. Capacity
exhaustion is not permission to omit work.

## 10. Exact evidence receipts

Machine-readable receipts should use a canonical serialization and
cryptographic digests. The repository does not yet select the serialization or
signature scheme through this packet.

### 10.1 Candidate event receipt

- repository identity;
- protected base commit/tree;
- candidate or merge-group projected commit/tree;
- merge base and predecessor generation;
- complete NUL-safe change manifest digest;
- event kind and immutable event identifier; and
- workflow/run/attempt identity.

### 10.2 Graph receipt

- base and candidate graph root digests;
- cell/package/target/configuration shard manifests;
- Buck2 binary, prelude, rule, and Starlark digests;
- `.buckconfig`, cell alias, and external-cell mappings;
- toolchain and execution-platform fingerprints;
- dependency-lock and third-party graph digests;
- generator/materializer identities; and
- graph schema and adapter versions.

### 10.3 Selection receipt

- selector identity and source/binary digest;
- policy and always-run registry digest;
- supported/unsupported/unknown classification;
- direct owners from base and candidate;
- base and candidate reverse closures;
- configuration/platform expansion;
- generated/policy/toolchain/cell expansions;
- included targets with reason codes;
- excluded targets with reason codes;
- fallback decision and reason; and
- runtime/resource measurements.

### 10.4 Plan and shard receipt

- ordered deterministic target manifest;
- shard manifests and digests;
- platform/resource/exclusivity assignment;
- scheduling-policy digest;
- expected versus actual shard count;
- retry policy;
- required capacity reservation; and
- full-universe oracle linkage.

### 10.5 Execution receipt

- exact target and configuration;
- action/input/output digests;
- toolchain/execution worker identity;
- local/remote and cold/warm classification;
- cache lookup/result with trust domain;
- exit status, normalized finding, and raw-log digest;
- retry/flake classification without suppressing the original result;
- Buck2 build report and event-log digests; and
- artifact/provenance attestations.

### 10.6 Parity receipt

- selected versus full target-set relation;
- selected versus full configured-target relation;
- selected versus full verdict relation;
- Cargo versus Buck2 verdict relation while both remain active;
- local versus remote relation;
- cold versus warm relation;
- detector disagreement;
- false-negative count;
- false-positive count; and
- exact corpus and time window.

### 10.7 Fan-in receipt

- all required child receipt digests;
- missing, duplicate, stale, or mixed-generation rejection evidence;
- always-run completeness proof;
- fallback result;
- single canonical `oya-ci-required` verdict;
- protected admission-envelope response; and
- post-merge replay/rollout/rollback/observability linkage.

Raw logs, manifests, and artifacts must be retained where their digest can be
resolved under the applicable custody policy. A summary without the bound raw
evidence is not an exact receipt.

## 11. AI/ML evidence inside the unified process

There is no “AI-assisted CI lane.” AI/ML is an optional advisory technique
inside the one unified CI/CD build/test process and stays subordinate to the
deterministic graph and admission contract.

### 11.1 Permissible bounded uses

- rank already eligible targets to improve time-to-first-failure;
- estimate duration and resource class for deterministic sharding;
- identify suspicious selector disagreements for human/engineering review;
- cluster failures and suggest likely owners without changing verdicts;
- forecast flaky-test probability while preserving the original failure;
- propose test cases, fuzz seeds, or mutation targets that must compile, run,
  repeat, and pass ordinary review;
- identify likely missing dependency edges for deterministic confirmation; and
- prioritize which full-oracle strata receive extra capacity.

### 11.2 Prohibited authority uses

AI/ML may not:

- remove a target from the sound deterministic affected set;
- declare an unknown safe;
- make an empty detector result authoritative;
- suppress, reinterpret, or silently quarantine a failure;
- waive an always-run gate;
- invent human identity, qualification, independence, or approval;
- produce a legal/JCR, veto, affected-party, custody, or operations authority
  receipt;
- attest provenance or signature validity by assertion;
- choose whether `oya-ci-required` is green; or
- replace scheduled full-universe and cold-cache oracles.

### 11.3 Research evidence and limits

Primary research and engineering reports show useful potential but do not
transfer a safety claim to Oyatie:

- Meta reported Predictive Test Selection running roughly one third of tests
  while capturing more than 99.9% of problematic changes in its measured
  environment:
  https://engineering.fb.com/2018/11/21/developer-tools/predictive-test-selection/
- Google studied techniques for regression testing in continuous integration:
  https://research.google/pubs/techniques-for-improving-regression-testing-in-continuous-integration-development-environments/
- Google studied speculative testing with transition prediction:
  https://research.google/pubs/speculative-testing-at-google-with-transition-prediction/
- Google also assessed limits and trade-offs of transition-based selection:
  https://research.google/pubs/assessing-transition-based-test-selection-algorithms-at-google/
- Google described automatic root-cause localization for flaky tests:
  https://research.google/pubs/de-flake-your-tests-automatically-locating-root-causes-of-flaky-tests-in-code-at-google/
- Meta described probabilistic flakiness:
  https://engineering.fb.com/2020/12/10/developer-tools/probabilistic-flakiness/
- Google published smart scheduling work for scalable build service:
  https://research.google/pubs/scalable-build-service-system-with-smart-scheduling-service/
- TestGen-LLM research:
  https://arxiv.org/abs/2402.09171
- ACH/LLM test-generation research:
  https://arxiv.org/abs/2501.12862
- Google SafeRevert research:
  https://research.google/pubs/saferevert-when-can-breaking-changes-be-automatically-reverted/
- Microsoft Conan batch-failure diagnosis:
  https://www.microsoft.com/en-us/research/publication/conan-diagnosing-batch-failures-for-cloud-systems/

Transfer rule: published percentages are comparator evidence, not Oyatie
acceptance thresholds. Oyatie must validate every technique against its exact
graph, change distribution, failure corpus, configurations, and cost of false
negatives. Safety metrics remain absolute even when efficiency metrics are
statistical.

## 12. Security, provenance, and supply chain

The minimum supply-chain floor is:

- immutable source and dependency pins;
- independently verified binary digests;
- authenticated build worker and workflow identity;
- hermetic declared inputs;
- isolated tenant/trust cache namespaces;
- content-addressed outputs;
- signed or equivalently authenticated provenance;
- complete source-to-binary and binary-to-verdict linkage;
- retention and revocation policy;
- incident-triggered cache invalidation and full fallback; and
- replayable evidence.

SLSA Build Track grounding:

https://slsa.dev/spec/v1.2/build-track-basics

Neither a GitHub release asset, a cache hit, a Buck2 action digest, nor an AI
assessment is sufficient alone to prove provenance.

## 13. Measures that optimize truth, not activity

Primary safety metrics:

- false-negative affected selections: exactly `0`;
- unknown/error states that fail open: exactly `0`;
- always-run omissions: exactly `0`;
- stale or mixed-generation receipts accepted: exactly `0`;
- hand-edited generated faces: exactly `0`;
- cache poisoning or cross-tenant object disclosure: exactly `0`;
- admission greens without complete evidence: exactly `0`;
- skipped/deleted tests used to obtain green: exactly `0`; and
- unresolved protected-behavior regressions: exactly `0`.

Efficiency metrics, measured only after safety:

- graph snapshot latency and incremental reuse;
- query and selector latency by repository size/change class;
- affected/full target ratio;
- false-positive ratio;
- cold and warm critical-path duration;
- local and remote execution ratio;
- cache hit rate by trust domain and action class;
- CAS ingress/egress and storage churn;
- queue wait, execution time, and fan-in dwell;
- time to first actionable failure;
- materialization reuse;
- duplicate Cargo/Buck work;
- retry and flaky-test cost;
- full-fallback rate and cause;
- merge-train invalidation/recompute cost; and
- verified protected candidates per admission slot.

Do not optimize agent count, job count, changed lines, or nominal test-count
reduction independently of verified throughput and escape rate.

## 14. Sixteen-lens council analysis

This section applies the proposed L01-L16 roster as a preparatory analysis. It
is not C13 satisfaction. No author of this packet is asserted to be a qualified,
independent human reviewer, and no row is an authority receipt.

| Lens | Preparatory finding | Planning-exit evidence still required |
| --- | --- | --- |
| L01 product and user value | Faster truthful feedback improves delivery; false-green selection directly harms users | Product-value hypothesis, user-impact measures, regression ceiling |
| L02 intuitive no-code UX and accessibility | CI internals must remain invisible to no-code users while console diagnostics stay understandable and accessible | User-story/browser/accessibility evidence for operator surfaces |
| L03 ontology, data, and temporal semantics | Base/candidate/projected-tree identities and graph generations need explicit temporal semantics | Canonical identity/time model and stale-generation rejection |
| L04 workflow automation and compensation | Automatic full fallback and rollback are compensating workflows, not optional scripts | Rehearsed failure, retry, rollback, and compensation receipts |
| L05 architecture, modularity, and hyperscale | Cells should be stable config/trust boundaries; packages and targets carry product modularity | Graph scale tests, boundary fitness, configuration completeness |
| L06 cloud, platform, and enterprise infrastructure | RE/CAS can unlock cross-runner locality but creates capacity and tenancy obligations | Live topology, isolation, capacity, disaster recovery, cost evidence |
| L07 developer, build, release, and supply chain | Buck2-native graph/query/toolchain/provenance should replace duplicated heuristic work only after parity | Exact tool pins, hermeticity, parity, provenance, upgrade/rollback |
| L08 reliability, operations, and observability | Full fallback, build reports, logs, critical path, and deterministic shards are mandatory | SLOs, dashboards, alerts, runbooks, fault injection, on-call acceptance |
| L09 security, identity, and abuse resistance | Shared caches and remote workers add poisoning, impersonation, and data-exposure risk | Threat model, authenticated identities, namespace isolation, negative tests |
| L10 privacy, residency, and data governance | CAS/logs may contain source, secrets, customer fixtures, and regional data | Classification, residency, retention, deletion, access, audit disposition |
| L11 legal, regulatory, and JCR | Third-party detector, Buck2 releases, generated tests, logs, and RBE need license/JCR review | Exact-source legal/JCR receipt bound to evaluated versions and uses |
| L12 affected-party safety and ethics | Selective testing shifts escape risk to users, operators, maintainers, and regulated parties | Affected-party analysis, safety thresholds, appeal/override path |
| L13 economics, FinOps, and business viability | Savings may be erased by graph storage, CAS egress, duplicate execution, or false-green incidents | Measured TCO, sensitivity, capacity, incident-cost, break-even evidence |
| L14 interoperability, supply chain, and ecosystem | GitHub Actions, Buck2, NativeLink/REAPI, Rust/Cargo, and external cells must interoperate without hidden authority | Versioned interface tests, portability, export/recovery, vendor-exit proof |
| L15 maintainability, evolvability, and deprecation | Moving tags, experimental formats, bespoke adapters, and dual Cargo/Buck bridges require lifecycle control | Ownership, compatibility policy, deprecation triggers, migration/rollback |
| L16 evidence, audit, governance, and dissent | Exact receipts and independent full oracles are the core safety mechanism | Independent audit, qualified council, fresh dissent, cold-reader exit |

Cross-lens result: no lens supports immediate implementation dispatch. The
architecture is promising only if the safety, authority, evidence, operations,
custody, and affected-party conditions advance together.

## 15. Stage-1 relationship and stop rule

This optimization packet may inform later comparator and planning work only
after the controlling Stage-1 chronology permits it. It does not independently
satisfy:

- controlling ADR chronology or ratification;
- parser/IR closure;
- corpus/archive/freshness;
- decision-population closure;
- comparator authority;
- legal/JCR;
- affected-party review;
- operations;
- custody;
- veto;
- pilot;
- immutable successor;
- the qualified sixteen-lens council;
- fresh dissent;
- context-free exit; or
- protected admission.

The correct current stop state is:

`HOLD(Planning) / PREPARATORY_EVIDENCE_ONLY / NO_DISPATCH`

A qualified `PASS` would require independently authenticated, object-bound,
fresh receipts for every controlling gate plus successful protected admission.
Autonomous engineering authorization cannot manufacture human qualification,
identity, independence, trust-root facts, legal standing, custodial authority,
veto authority, operational authority, or protected-server facts.

## 16. Source index

### Reproducible upstream evidence manifest

The byte-integrity-bound primary-source rows for the Buck2 and Buck2 Change
Detector claims are in
`buck2-hyperscale-unified-ci-optimization-20260723.upstream-evidence.json`.
It records source URLs, immutable full SHAs where available, retrieval times,
captured-byte SHA-256 values, and a re-verification command. The manifest was
captured on 2026-07-24 after this packet's nominal date; it is a refresh of
upstream evidence only, remains `NON_AUTHORITY`, and does not alter
`HOLD(Planning)` or `NO_DISPATCH`. Mutable releases/issues are explicitly
labeled as snapshot-only, not immutable proof.

### Buck2 official sources

- https://buck2.build/docs/concepts/key_concepts/
- https://buck2.build/docs/concepts/buckconfig/
- https://github.com/facebook/buck2/releases
- https://buck2.build/docs/concepts/architecture/
- https://buck2.build/docs/rule_authors/writing_toolchains/
- https://buck2.build/docs/concepts/configurations/
- https://buck2.build/docs/concepts/modifiers/
- https://buck2.build/docs/rule_authors/configuration_transitions/
- https://buck2.build/docs/concepts/visibility/
- https://buck2.build/docs/users/remote_execution/
- https://buck2.build/docs/rule_authors/dep_files/
- https://buck2.build/docs/bxl/
- https://buck2.build/docs/concepts/buck_query_language/
- https://buck2.build/docs/users/build_observability/build_report/
- https://buck2.build/docs/users/build_observability/logging/
- https://buck2.build/docs/concepts/isolation_dir/
- https://buck2.build/docs/users/advanced/external_cells/

### Buck2 Change Detector official sources

- https://github.com/facebookincubator/buck2-change-detector
- https://github.com/facebookincubator/buck2-change-detector/blob/7a632871f376cb04bc007829c8dd7d536078ae4a/README.md
- https://github.com/facebookincubator/buck2-change-detector/blob/7a632871f376cb04bc007829c8dd7d536078ae4a/btd/README.md
- https://github.com/facebookincubator/buck2-change-detector/commit/7a632871f376cb04bc007829c8dd7d536078ae4a
- https://github.com/facebookincubator/buck2-change-detector/releases/tag/latest
- https://github.com/facebookincubator/buck2-change-detector/issues/12
- https://github.com/facebookincubator/buck2-change-detector/issues/9
- https://github.com/facebookincubator/buck2-change-detector/blob/7a632871f376cb04bc007829c8dd7d536078ae4a/.github/workflows/release_btd.yml

### CI optimization, ML advisory, diagnosis, and provenance sources

- https://engineering.fb.com/2018/11/21/developer-tools/predictive-test-selection/
- https://research.google/pubs/techniques-for-improving-regression-testing-in-continuous-integration-development-environments/
- https://research.google/pubs/speculative-testing-at-google-with-transition-prediction/
- https://research.google/pubs/assessing-transition-based-test-selection-algorithms-at-google/
- https://research.google/pubs/de-flake-your-tests-automatically-locating-root-causes-of-flaky-tests-in-code-at-google/
- https://engineering.fb.com/2020/12/10/developer-tools/probabilistic-flakiness/
- https://research.google/pubs/scalable-build-service-system-with-smart-scheduling-service/
- https://arxiv.org/abs/2402.09171
- https://arxiv.org/abs/2501.12862
- https://research.google/pubs/saferevert-when-can-breaking-changes-be-automatically-reverted/
- https://www.microsoft.com/en-us/research/publication/conan-diagnosing-batch-failures-for-cloud-systems/
- https://slsa.dev/spec/v1.2/build-track-basics

## 17. Packet self-limitations

- Upstream source observations are frozen to 2026-07-23 and require a fresh
  version/release/issue audit before any later plan or pilot.
- The root checkout was not used as a protected-state oracle.
- No GitHub branch-protection, merge-queue, live Actions, runner, CAS, cluster,
  NativeLink, or production observation is claimed here.
- No benchmark was run, so no latency, capacity, cost, cache-hit, or speedup
  claim is made.
- No detector binary was installed or executed.
- No generated face, source worktree, authority document, workflow, or build
  definition was edited.
- The sixteen-lens table is preparatory analysis only, not independent council
  satisfaction.
- All implementation and roadmap dispatch remain blocked by `HOLD(Planning)`.
