---
doc_class: Owner-ADR
owner: flags
status: Accepted
date: 2026-08-26
inherits:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
---

# Flags decisions in force

This file specializes ADR-0719 for `flags/`. It records the destination and
migration constraints; it does not claim that a production Flags service has
landed.

<current_state>

## Evidence at L1a

| Surface | What exists | Maturity |
|---|---|---|
| `core/evaluation-domain` | Dependency-free Rust model, stable FNV-1a percentage bucketing, ordered targeting, typed variants, disabled/default behavior, safe resolution errors, and a synchronous source trait | Real deterministic kernel with 19 unit tests; its 292-line `src/lib.rs` carries a hand-maintained module index, and it has no complete admission limits, mutation authority, persistent source, snapshot distribution, facade, or measured performance evidence |
| `core/server` | Default config, no-op observability, empty REST/gRPC/OFREP/storage/tenant modules, and a re-export smoke test | Non-serving scaffold; no listener, protocol, storage, authorization, distribution, or readiness behavior |
| `cedar/` | Old CI, Argo/Jenkins, and cell-sharding policy fragments | Legacy cross-owner residue; not Flags mutation/evaluation policy |
| `iac/` | Hand-authored Kubernetes, Helm, Terraform, OpenBao, WAF, ECH, PQC, and secret manifests for an imagined evaluator fleet | Unconsumed deployment fiction for services and dependencies that do not exist |
| `observability/slos/` | Hand-authored OpenSLO files naming evaluator, experiment, kill-switch, and autosharding metrics | Design residue only; no producer or SLO-controller source proves the signals |
| `README.md` and root `BUCK` | A legacy feature-flags service narrative and a YAML corpus loader | Stale product/runtime claims; the loader does not make the described service real |

`evaluation-domain` is the only implementation-bearing surface in charter.
Existing tests prove pure function behavior, not network availability, durable
state, multi-cell propagation, authorization, kill-switch convergence, or SLOs.

</current_state>

<charter>

## Decision: runtime dynamic configuration, not experiments or deployment

- **achieves:** one owner for tenant-scoped runtime choices without turning a
  flag service into an analytics suite, policy engine, or release orchestrator.
- **origin:** the legacy tree combined deterministic evaluation with experiment
  statistics, cell topology, CI/CD, and hand-authored deployment narratives;
  ADR-0719 D-14/D-15 narrows the capability.
- **rule:** `flags/` MUST own versioned flag definitions, deterministic typed
  evaluation, ordered targeting, percentage assignment, emergency kill-switch
  precedence, and pack-gated overrides from verified C0 Cedar context. It MUST
  NOT own experiment design/statistical significance, code-deploy admission,
  cell topology/autosharding, a clock, the Cedar PDP, or application privilege
  modes.
- **ensure:** package and contract review rejects experiment/metric-attribution
  engines, Pipeline or Cell behavior, private time sources, and trusted-tenant
  switches; cross-owner effects use ports and sold facades.
- **overturn_when:** a founder-accepted owner decision reallocates a named
  behavior and amends every affected owner in the same change.

</charter>

<evaluation_contract>

## Decision: deterministic, bounded, and fail-closed evaluation

- **achieves:** identical admitted inputs produce identical outcomes at every
  cell while one malicious definition cannot create unbounded hot-path work.
- **origin:** the landed kernel is pure and uses a fixed bucketing algorithm,
  but accepts unbounded vectors, targeting keys, string/object variant payloads,
  and non-finite floats, and discovers some malformed references only when a
  subject happens to select them.
- **rule:** evaluation MUST be a total, side-effect-free function of an admitted
  definition snapshot, explicit evaluation context, and verified override
  context. Definition and context size MUST be bounded before the hot path,
  including targeting keys, every string/object payload dimension, and aggregate
  bytes; non-finite floats MUST be rejected. Referenced variants, rollout
  weights, identifiers, and operator shapes MUST validate completely. No
  evaluation may read network, storage, wall clock, RNG, or mutable global
  state. Invalid or unavailable authority MUST return a typed error and the
  registered safe fallback, never an optimistic variant or panic.
- **ensure:** golden vectors run across supported architectures/toolchains;
  property and fuzz campaigns cover every exact boundary, maximum+one, aggregate
  byte cap, NaN, and positive/negative infinity; workload tests prove the
  declared work bound and zero deterministic replay mismatch.
- **overturn_when:** a versioned evaluation algorithm proves equivalent replay,
  bounded work, safe fallback, and migration behavior, with dual-version
  comparison before old vectors retire.

</evaluation_contract>

<stable_item_membership>

## Decision: stabilize evaluation membership before behavior

- **achieves:** bounded Flags changes become unique item files while the crate
  root, public API, and Cargo/Buck source membership remain stable.
- **origin:** `core/evaluation-domain/src/lib.rs` is already 292 lines and names
  four modules manually. D-35 requires the split when this crate is next worked,
  and D-41 forbids replacing it with a tracked or hand-maintained index.
- **rule:** L1d.0 MUST be a behavior-preserving structural slice that installs an
  owned package-root `build.rs`, sorts declared `src/items/*.rs` and
  `src/test_items/*.rs`, and writes membership only to `OUT_DIR`. `src/lib.rs`
  MUST retain stable generated `include!` lines; current module and root re-export
  paths MUST remain source compatible through that slice, all 19 vectors MUST
  remain unchanged, and no tracked generated or manual per-item `mod` inventory
  may remain. Later structural port promotion follows its own closed migration.
- **ensure:** Buck's `buildscript_run` stages the same globbed directories and
  executes the same scanner as Cargo; generated source order/content is parity-
  checked, and an add/rename/remove canary compiles through both graphs without
  a parent-index edit. A compile fixture compares every current public path.
- **overturn_when:** rustc gains deterministic directory membership without a
  generated index, or a five-field owner decision provides a smaller owned
  mechanism that preserves stable parents, public paths, and Cargo/Buck parity.

</stable_item_membership>

<authority_and_distribution>

## Decision: durable control authority, immutable cell-local snapshots

- **achieves:** independently scalable mutation and evaluation planes with no
  remote dependency on ordinary evaluation and no split-brain definition truth.
- **origin:** a pure kernel alone does not define who may mutate a flag, which
  generation is authoritative, or how evaluators recover from stale or corrupt
  state.
- **rule:** flag mutation MUST commit through one authoritative, versioned
  control-plane record with idempotency and pre-ack audit/outbox evidence;
  adapters MUST distribute signed immutable snapshots or ordered deltas.
  Evaluators MUST atomically publish one verified generation and evaluate from
  cell-local memory. Flags MUST NOT own Cell placement or fetch a pack/PDP on the
  evaluation hot path.
- **ensure:** receipts bind tenant, flag, definition generation, policy revision,
  snapshot digest, idempotency key, and audit evidence; partition/reorder/corrupt
  snapshot tests prove monotonic publication, last-known-safe behavior, and
  explicit staleness instead of two active generations.
- **overturn_when:** a specified alternative proves equal single-authority,
  audit, replay, partition, recovery, and hot-path independence properties.

</authority_and_distribution>

<policy_and_isolation>

## Decision: Policy decides permission; Flags consumes verified context

- **achieves:** tenant targeting and jurisdiction overrides without embedding a
  second PDP or trusting caller-supplied attributes as authorization.
- **origin:** ADR-0719 places Cedar evaluation in `policy/` and says Flags pack
  gates arrive through C0 context, while the current domain accepts arbitrary
  targeting attributes with no proof boundary.
- **rule:** mutation and override admission MUST require verified principal,
  tenant, action, resource, policy revision, audience, expiry, and request
  binding from the normal IAM/Policy path. Pack-gated overrides MUST consume
  verified C0 Cedar context; Flags MUST NOT fetch pack content or reinterpret an
  arbitrary evaluation attribute as policy proof. Every tenant uses the same
  path, including first-party workloads.
- **ensure:** forged, expired, replayed, wrong-audience, and cross-tenant proofs
  fail before commit or snapshot publication; tests separate ordinary targeting
  attributes from unforgeable override authority; telemetry contains no raw
  subject attributes.
- **overturn_when:** an accepted Policy or Packs contract replaces C0 context
  and preserves one PDP, one install authority, request binding, and fail-closed
  tenant isolation.

</policy_and_isolation>

<interfaces>

## Decision: Connect is canonical; compatibility stays at adapters

- **achieves:** one semantic contract while supporting standard client
  ecosystems without maintaining REST and gRPC as separate truths.
- **origin:** the retired server advertises REST, gRPC, and OFREP modules despite
  implementing none; ADR-0719 selects the platform Connect facade and treats
  OFREP/OpenFeature as compatibility, not authority.
- **rule:** the sold Flags facade MUST be one versioned protobuf contract over
  Connect/H3 with evaluation and control-plane surfaces separated by policy and
  capacity. OpenFeature/OFREP and language SDK behavior MUST be adapters derived
  from that semantic contract; an independent REST/gRPC server pair or OFREP
  metadata model MUST NOT exist.
- **ensure:** one conformance suite exercises the native facade and retained
  adapters; error/reason/generation semantics remain identical; deleting an
  adapter leaves core state and the canonical contract unchanged.
- **overturn_when:** measured client compatibility requires another sold wire
  surface and a same-wave decision specifies its shared semantics, versioning,
  authorization, and retirement.

</interfaces>

<readiness>

## Decision: readiness and SLO claims follow executable evidence

- **achieves:** operators never route critical evaluation traffic to an empty
  shell or mistake hand-authored YAML for observed reliability.
- **origin:** the legacy README and OpenSLO corpus claim SDKs, endpoints, fleet
  topology, availability, and propagation behavior absent from the code.
- **rule:** Flags MUST advertise only capabilities exercised by the selected
  runtime and its live signals. Readiness MUST require an admitted local
  snapshot, verified policy/distribution authority, healthy facade runtime, and
  supported schema/generation; SLO outputs MUST be generated from owner IR only
  after the named metrics exist and the Observability controller consumes them.
- **ensure:** health tests refuse on empty, corrupt, rolled-back, or unsupported
  snapshots and missing authority; repository tests reject hand-authored
  OpenSLO/deployment claims with no producer; promotion cites measured load and
  fault results.
- **overturn_when:** a replacement readiness/SLO producer is live, consumed,
  and proves equally fail-closed capability and signal provenance.

</readiness>

<migration>

## Decision: remove false surfaces before adding behavior

- **achieves:** a small, reviewable Flags base whose next behavior cannot
  accidentally depend on retired server or paper infrastructure.
- **origin:** ADR-0719 explicitly keeps `evaluation-domain`, removes the bundled
  REST/gRPC server and cap-root dump, and separates structural from behavioral
  work.
- **rule:** migration MUST land as L1b server retirement, L1c consumer-confirmed
  residue cleanup, L1d.0 stable-index/file-budget preparation, L1d bounded
  definition admission, L1e.0 structural port-face promotion, L1e.1 dependency
  admission, L1e.2 structural adapter-face creation, L1e.3 durable-adapter
  behavior, L1e.4 kill/C0 authority, then L1f facade and distribution.
  Structural lanes MUST remain separate from behavior, preserve the kernel's
  current public surface where promised, and add no runtime claims.
- **ensure:** `PLAN.md` fixes the sequence and changed-path envelopes; each lane
  has before/after tests, protected review, explicit rollback, and no generated
  hand edits.
- **overturn_when:** independently reviewed evidence shows two adjacent steps
  cannot be separated and a replacement sequence retains the same behavior,
  rollback, and false-readiness boundaries.

</migration>

## Rejected destinations

- A bundled placeholder server kept because it has a Cargo target.
- Experiment analytics or statistical significance inside Flags.
- REST, gRPC, and OFREP as independent sources of truth.
- Per-evaluation network, storage, PDP, pack, clock, or RNG calls.
- Caller attributes treated as authorization or installed-pack proof.
- Hand-authored Helm, Terraform, Kubernetes, Cedar, or OpenSLO files presented
  as deployed capability.
- A feature flag that grants first-party/trusted-tenant privilege or bypasses
  IAM, policy, quota, audit, or metering.
- A tracked generated module index, hand-maintained parent `mod` list, or
  Cargo-only item scanner whose Buck membership differs.
- Port moves, crate/face creation, or lockfile mutation mixed with adapter,
  authority, or evaluation behavior.
