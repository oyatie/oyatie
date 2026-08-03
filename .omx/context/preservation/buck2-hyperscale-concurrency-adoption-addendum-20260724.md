# Buck2 hyperscale concurrency adoption-gate addendum

Date: 2026-07-24

Status: `NON_AUTHORITY`

Planning state: `HOLD(Planning)`

Dispatch state: `NO_DISPATCH`

Decision state: `PREPARATORY_EVIDENCE_ONLY`

This addendum turns the requested hyperscaler patterns into explicit future
admission gates. It is not an ADR, roadmap, implementation plan, work queue,
lease, merge instruction, C13 council receipt, C14 dissent receipt, C15 exit
receipt, or permission to change protected CI.

No authoritative live Stage-1 epoch exists. The required fail-closed epoch
ceiling remains `HOLD_EPOCH_OPEN`, and the effective planning state remains
`HOLD(Planning)`. Nothing below dispatches before one qualified Stage-1 PASS
authorizes entry into roadmap planning.

## 1. Bound evidence

This addendum supplements, but does not mutate:

- `buck2-hyperscale-unified-ci-optimization-20260723.md`
  - SHA-256:
    `88da35392efec3f7fc7810463253b562b10f16b41022ac62ce8ef02f8f98267c`
- `buck2-hyperscale-unified-ci-optimization-20260723.upstream-evidence.json`
  - SHA-256:
    `c54c683a2c692ba89d598eb8128b88d07e40acf1d2804193ca5666ff327d0531`

The Bun migration report was retrieved read-only from
<https://bun.com/blog/bun-in-rust> on 2026-07-24. The observed response-byte
SHA-256 was
`3d2a919434e7f61d4246c9dd0aa0401d5168a3aa023d8f8ab65b1da0febfafd8`.
That digest records a mutable web response observation, not immutable source
provenance or Oyatie adoption authority.

## 2. Effective-concurrency contract

Agent count is not the optimization target. Verified throughput is.

For a representative workload and an exact repository generation:

\[
S(N) \le
\frac{1}{s + \frac{1-s}{N} + \kappa(N)}
\]

where:

- \(N\) is the number of concurrent execution or agent lanes;
- \(s\) is the measured serialized fraction, including shared contracts,
  protected-parent rebinding, generated-face production, integration,
  review/fix closure, and protected landing;
- \(\kappa(N)\) is measured contention and rework, including cache misses,
  CPU, memory, IOPS, network, fixture, browser/simulator, database, review,
  conflict, invalidation, flaky retry, and failed-consolidation costs.

No speedup claim is accepted from nominal job count, agent count, changed lines,
or test-count reduction. A concurrency generation must record at least:

- exact base, candidate, projected merge-group, Buck2, prelude, toolchain,
  platform, cell-map, selector, and policy identities;
- \(N\), queue time, wall time, critical path, compute-minutes, target count,
  affected/full ratio, CPU, memory, disk, IOPS, network, retries,
  cancellations, and infrastructure-red rate;
- local/cold, local/warm, CAS-warm, remote-execution, affected, and full
  classifications;
- cache lookup/hit/miss and trust domain;
- review queue dwell, conflict/rework count, and protected fan-in dwell; and
- correctness parity, output-digest parity, false-negative count, and escape
  rate.

The smallest future promotion rule, still subject to qualified planning:

- increase concurrency only when the lower 95% confidence bound shows at least
  5% incremental verified-throughput improvement;
- do not accept a p95 end-to-end latency regression greater than 5%;
- accept zero new correctness failures, false-negative selections,
  mixed-generation receipts, generated-face hand edits, or cache-integrity
  failures;
- do not accept an infrastructure-red or retry-rate increase greater than
  0.5 percentage points;
- require zero OOM events, 100% disk preflight success, and byte-identical cache
  canaries; and
- on missing telemetry or a failed criterion, fall back to the last proven
  \(N\), ultimately \(N=1\).

The numerical thresholds are a `DERIVED_PROPOSAL`, not an authority fact. A
future qualified planning process may tighten them, but may not weaken the
zero-false-negative, zero-false-green, provenance, or fail-closed invariants.

## 3. Hyperscaler-proven operating pattern

### 3.1 Graph-aware selection

Selection must compare immutable base and candidate graph generations. It must
not infer impact from candidate-only filenames or an arbitrary reverse-depth
cap.

The graph receipt binds:

- base and candidate commits and trees;
- projected merge-group predecessor;
- Buck2 binary, prelude, rules, BXL/query adapter, and schema versions;
- cells, packages, configured targets, action graph, platforms, modifiers,
  transitions, toolchains, and execution platforms;
- generated inputs, policy roots, always-run registry, and external cells; and
- add, delete, rename, copy, type, mode, symlink, submodule, and external-source
  change semantics.

Buck2 Change Detector may be evaluated as one pinned selector implementation.
It is not admission authority. It runs shadow-first against the full oracle,
and any incomplete, unsupported, unparseable, stale, or disagreeing result
falls back to the complete required universe.

### 3.2 Packages and targets are parallelism boundaries

Cells are reserved for genuine repository configuration, prelude, toolchain,
external-origin, ownership, or trust boundaries.

Product modularity and work queues use packages, targets, providers, declared
artifacts, configurations, visibility, and ownership metadata. Oyatie must not
create a cell per crate, service, team, directory, or agent merely to increase
nominal concurrency.

### 3.3 Distributed execution and shared CAS

Remote CAS and execution enter only after:

- hermetic declared input/output proof;
- authenticated worker and workflow identity;
- isolated tenant/trust namespaces and credentials;
- source-to-action and action-to-output provenance;
- cold/warm and local/remote output-digest, exit-status, and verdict equality;
- poisoning, disclosure, eviction, outage, retry, and rollback tests;
- measured capacity, queue, hit rate, ingress/egress, and cost; and
- qualified operational custody.

CAS activation precedes remote execution. A cache hit is evidence reuse, not a
test waiver or authority receipt.

### 3.4 Bounded resource scheduling

The scheduler treats these as separate finite pools:

- CPU;
- memory;
- disk and IOPS;
- network and CAS ingress/egress;
- database and external-service fixtures;
- browsers, simulators, devices, and GPUs;
- signing, provenance, and protected fan-in capacity;
- reviewers, fixers, and conflict-resolution capacity; and
- the single protected landing slot.

Fan-out is capped by the smallest saturated pool, not the number of available
agents. Always-run, full-fallback, security, and recovery capacity is reserved
before best-effort fan-out.

### 3.5 Deterministic cost-balanced sharding

After the sound target universe is fixed:

- shard by platform, configuration, declared resource class, exclusivity,
  historical duration, and critical-path position;
- use stable content-addressed shard manifests and reason codes;
- keep retries on the same manifest unless an explicit new generation is
  created;
- distinguish product, test, infrastructure, environment, and policy failure;
- stream large manifests rather than expand command lines; and
- never silently truncate an oversized closure—scale it out or run the full
  universe.

Historical or learned estimates may order already required work. They may not
remove targets or make `oya-ci-required` green.

### 3.6 Merge-train rebinding

Every candidate is evaluated against the exact projected predecessor:

```text
protected dev
  -> projected P1
  -> projected P1+P2
  -> projected P1+P2+P3
```

Predecessor movement invalidates downstream graph, selection, plan, execution,
review, and fan-in receipts. Digest-identical actions may be reused through
CAS, but receipt identity is still regenerated for the new projected tree.
Landing remains serialized against the actual protected predecessor unless a
future proven bounded batch contract is separately authorized.

### 3.7 Build observability

Every execution retains the Buck2 build report, event log, action/target
results, critical path, cache result, resource measurements, raw-log digests,
and normalized failure classification.

Build observability must support:

- exact target/configuration reproduction;
- product versus infrastructure versus environment attribution;
- cache and remote-execution parity;
- selector/full-oracle comparison;
- postsubmit culprit attribution and safe revert evidence;
- SLO, capacity, cost, and queue analysis; and
- a context-free evidence consumer that can resolve every digest.

## 4. Bun-derived worktree and consolidation pattern

The transferable lesson is not “rewrite everything in Rust at once.” It is to
turn a mechanically large migration into a deterministic queue of mostly
independent, behavior-locked units.

The future Oyatie gate is:

1. Freeze one immutable base generation and a language-independent behavior
   corpus.
2. Precompute work queues by package/target/crate, dependency closure,
   configuration, platform, resource class, and expected cost.
3. Assign one isolated worktree and branch to one bounded lane.
4. Start each behavior change or port with RED regression evidence. Keep Rust
   unit tests beside private implementation and integration tests in separate
   public-API test targets.
5. Prohibit shared mutable workspaces, global Cargo/Buck state, hand-edited
   generated faces, and cross-lane opportunistic edits.
6. Use distinct writer, independent review, fixer, and verifier roles. Review
   capacity is scheduled like compute capacity.
7. Consolidate only clean, signed, exact-head candidates. Rebind each candidate
   to its projected predecessor and rerun the affected/full safety contract.
8. Preserve behaviors where Oyatie is already ahead. Comparator evidence may
   reveal improvements; it never permits a regression merely to resemble Bun,
   Palantir, or another baseline.
9. Record every conflict, rework cycle, invalidated receipt, review dwell, and
   failed consolidation in \(\kappa(N)\).
10. Keep the full behavior suite, no-skipped-test rule, resource limits, and
    multi-platform evidence as non-negotiable convergence gates.

This model permits discovery, regression authoring, bounded implementation,
review, verification, and later migration waves to overlap only when their
evidence dependencies are independent. It does not turn dependency edges into
parallel tasks.

## 5. Unified CI safety floor

All optimizations remain modes inside one canonical process and one protected
`oya-ci-required` result:

```text
exact candidate or merge-group event
  -> authority/configuration snapshot
  -> immutable base and candidate graphs
  -> sound affected universe
  -> always-run union
  -> resource-aware deterministic shard plan
  -> local/remote build, test, and materialization fan-out
  -> provenance, review, and parity evidence join
  -> one fail-closed oya-ci-required fan-in
```

Mandatory fallbacks:

- unknown, stale, unsupported, empty, malformed, or disagreeing selection:
  `FULL`;
- missing graph, receipt, report, trust, provenance, review, or required lane:
  `RED`;
- cache/CAS/RE integrity or isolation uncertainty: local cold execution or
  `RED`;
- protected-base or merge-group movement: invalidate and recompute;
- capacity exhaustion that prevents the full safety floor: `RED`; and
- any attempt to hand-edit generated faces: `RED`.

AI/ML may rank already required work, forecast duration, cluster failures, or
propose tests for ordinary deterministic review. It may not suppress a target,
waive a gate, invent authority, attest provenance, approve a PR, or determine
the protected verdict.

## 6. Sixteen-lens future admission questions

This is preparatory analysis, not C13 satisfaction:

| Lens cluster | Required future proof |
| --- | --- |
| Product, UX, accessibility | Faster actionable feedback without exposing build-system complexity to no-code users |
| Ontology, temporal semantics | Exact base/candidate/projected-tree, graph-generation, receipt, and stale-state semantics |
| Workflow and compensation | Automatic full fallback, invalidation, retry, rollback, and safe-revert rehearsals |
| Architecture and hyperscale | Package/target boundaries, genuine cell boundaries, graph scale, and critical-path evidence |
| Cloud, reliability, operations | Capacity, isolation, SLOs, alerts, runbooks, fault injection, disaster recovery, and custody |
| Developer/build/supply chain | Hermetic toolchains, signed provenance, exact pins, review admission, and upgrade/rollback |
| Security, privacy, legal/JCR | Cache poisoning, worker identity, secrets, residency, retention, licenses, and qualified dispositions |
| Affected parties and ethics | False-negative risk, appeal/override, fairness across languages/teams, and user/operator impact |
| Economics and FinOps | Verified throughput, TCO, CAS/RE egress, capacity, incident cost, and break-even sensitivity |
| Interoperability and ecosystem | GitHub, Buck2, REAPI, Rust/Cargo, external cells, portable receipts, and vendor exit |
| Maintainability and evidence | Ownership, deprecation, format adapters, context-free replay, independent council, and fresh dissent |

No row is an independent qualified lens receipt.

## 7. Stop rule

Current result:

`authoritative_live_epoch_exists=false /
required_fail_closed_epoch_state=HOLD_EPOCH_OPEN / HOLD(Planning) /
PREPARATORY_EVIDENCE_ONLY / NO_DISPATCH`

The patterns above are now explicit and testable as future planning inputs.
They are not adopted in protected infrastructure until the controlling Stage-1
authority, admission, council, dissent, and context-free exit gates authorize
roadmap planning and a later accepted plan separately authorizes implementation.
