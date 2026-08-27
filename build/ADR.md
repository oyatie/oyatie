---
doc_class: Owner-ADR
owner: build
status: Accepted
date: 2026-08-27
inherits:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
---

# Build decisions in force

This file specializes ADR-0719 for `build/`. It records the inherited Build
charter and the first declaration-integrity destination. It does not claim that
the reconciler, image factory, or proposed repository-semantic system has
landed.

<current_state>

## Evidence at `8489b29bce609b8ee3a3e5874f1d3013672d20c9`

| Surface | What exists | Maturity |
|---|---|---|
| Root `Cargo.toml` and `Cargo.lock` | Workspace package declarations and the locked Rust dependency solution | Live Cargo authority; shared, serialized files |
| `reindeer.toml` | Reindeer reads the root manifest, refuses unresolved build-script fixups, emits `third-party/BUCK`, and names an owned overlay under deleted `ci/` | Active input with a dead post-generation reference |
| `third-party/fixups/**` | 66 package-local fixup files, including native/build-script decisions | Real generation inputs, but not yet covered by one provenance inventory or conformance receipt |
| `third-party/BUCK` | Checked generated dependency rules plus historical semantic mutations | Consumed by Buck2; its header names a different deleted shell wrapper, so clean reproduction is not proved |
| `build/port-engine/**` | Fourteen Rust packages for the named source-port engine | Implementation-bearing but frozen by ADR-0719; not a dependency or repository transformation engine |
| `build/toolchains/**`, `build/images/**` | Buck toolchain/cache declarations and one distroless image recipe | Partial Build assets, not a qualified image factory |
| `build/evidence/**`, `build/REORG-DRAIN.md` | Historical migration narratives | Residue; not runtime, generation, or admission evidence |

No owned Rust entrypoint currently regenerates `third-party/BUCK` atomically,
proves two clean runs byte-identical, emits a source-bound receipt, or supplies
the graph freshness step. Bare Reindeer output and the checked file are not a
proved round trip.

</current_state>

<charter>

## Decision: Build owns build inputs and produced machine artifacts

- **achieves:** one meta owner for toolchains, reproducible image/kernel inputs,
  pinned build tools, and declaration translation without creating a tenant
  product or a second execution plane.
- **origin:** ADR-0719 assigns toolchains, host/guest images, Cloud
  Hypervisor/Firecracker/kernel pins, and the frozen port engine to `build/`,
  while Pipeline owns graph execution and Compute owns fleet agents.
- **rule:** `build/` MUST own the reproducibility contracts and owned tooling
  that turn admitted source declarations into build-consumable artifacts. It
  MUST NOT own capability engines, price/rate logic, fleet placement or agents,
  CI/CD scheduling, merge/review state, Storage, or a user-facing CLI.
- **ensure:** dependency and package review rejects Pipeline/GitHub concepts,
  cloud capability cores, pricing, fleet control, and persistent artifact bytes
  from Build core; cross-owner effects use explicit ports and sold facades.
- **overturn_when:** a founder-accepted owner decision reallocates a named
  responsibility and amends every affected owner in the same protected wave.

</charter>

<declaration_authority>

## Decision: Cargo declares Rust packages; Buck2 executes the build graph

- **achieves:** one Rust package/dependency truth and one configured execution
  graph, with a deterministic adapter between them.
- **origin:** the workspace already resolves dependencies through Cargo and
  generates Buck declarations through Reindeer; ADR-0719 rejects parallel
  package graphs and keeps Buck2 as the local build/affected-set graph.
- **rule:** workspace and package `Cargo.toml` files plus root `Cargo.lock` MUST
  remain the Rust package/dependency declarations. Reindeer MUST remain a
  bounded Cargo-to-Buck adapter, and Buck2 MUST remain configured-target and
  execution authority. Generated `third-party/BUCK` MUST NOT become a separately
  edited package graph.
- **ensure:** reconciliation derives every third-party rule from the exact
  manifest, lock, Reindeer configuration, fixups, platform set, and pinned
  generator; review rejects independent dependency declarations and hand edits
  to generated output.
- **overturn_when:** a five-field owner decision replaces Cargo or Buck2 and
  proves a single authoritative declaration/execution path with deterministic
  migration and no dual-write interval.

</declaration_authority>

<reconciliation>

## Decision: one owned, fixup-first reconciliation transaction

- **achieves:** a clean checkout can regenerate the exact dependency rules or
  refuse without publishing a partial or semantically patched graph.
- **origin:** `reindeer.toml` and `third-party/BUCK` name two different deleted
  wrappers; prior generation depended on post-hoc text changes for native
  build rules, and unresolved local build scripts currently stop a clean run.
- **rule:** declaration reconciliation MUST be an owned Rust transaction. It
  MUST run one explicitly pinned Reindeer with locked, offline inputs and an
  explicit environment; encode package/native exceptions in reviewed fixups;
  generate twice; require byte equality; validate the result; and publish by
  same-directory atomic replacement only after every check passes. It MUST
  clear unapproved tool variables, perform no network access, invoke no shell
  wrapper, make no textual semantic overlay, and never hand-edit generated
  BUCK output.
- **ensure:** tests inject generator failure, mismatched double runs, malformed
  fixups, staged-write failure, pre-rename interruption, and directory-sync
  failure; failures before replacement retain the prior output, and observers
  always see either the prior or new complete bytes. Every failure is typed,
  and a deterministic receipt binds all inputs, tools, output, and publication
  outcome without claiming uncertain durability.
- **overturn_when:** Reindeer natively supplies equivalent fixup expressivity,
  hermetic double-run verification, validated atomic publication, and the same
  provenance receipt, allowing the owned transaction to shrink without losing
  a property.

</reconciliation>

<port_engine>

## Decision: keep the source port engine frozen and separate

- **achieves:** declaration integrity cannot silently turn a narrowly named
  source-language port into a universal build, graph, or codemod service.
- **origin:** ADR-0719 freezes `build/port-engine` until a named owned corpus is
  accepted and explicitly rejects staffing it as a Kubernetes/Borg program.
- **rule:** declaration reconciliation MUST NOT reuse, extend, or depend on
  `build/port-engine`. Port-engine behavior and its fourteen package faces MUST
  remain frozen during the declaration-integrity sequence.
- **ensure:** changed-path and dependency review refuse port-engine files or
  edges in every declaration-reconciliation PR; its existing Cargo/Buck targets
  remain unchanged.
- **overturn_when:** a founder-accepted decision names an owned source corpus,
  unfreezes the engine for that corpus, and still keeps dependency declaration
  reconciliation a separate concern.

</port_engine>

<owner_boundaries>

## Decision: Build publishes artifacts; other owners decide orchestration and bytes

- **achieves:** declaration work remains reusable and provider-neutral without
  importing repository review, queue, storage, or forge semantics.
- **origin:** Pipeline owns graph/queue/schedule and the current Git adapter;
  Storage is the designated artifact-byte capability. Neither owner has adopted
  the later cross-owner semantic extensions.
- **rule:** Build core MUST consume explicit immutable input descriptors and
  produce deterministic declaration artifacts/receipts. It MUST NOT depend on
  Pipeline's draft repository port, model pull requests or GitHub, orchestrate
  merge state, or persist blobs in another owner's implementation. Shared
  contracts MUST be adopted by all affected owners before implementation.
- **ensure:** dependency review keeps Build free of Pipeline core/draft and
  forge types; initial reconciliation publishes to a caller-supplied local
  filesystem adapter, while future storage or scheduling integration waits for
  separately reviewed owner contracts.
- **overturn_when:** all affected owners accept a versioned replacement
  contract and prove that dependency direction, provider neutrality, and
  independent operability remain intact.

</owner_boundaries>

<unadopted_proposals>

## Proposal requiring explicit adoption — nonbinding

Status: **PROPOSED; NOT ADOPTED**.

The repository-evolution design proposes placing compilation-unit contracts,
language extractor adapters, semantic fact definitions, conformance engines,
and deterministic transformation recipes with Build. It also proposes that
Pipeline consume typed repository inputs and orchestrate codemod campaigns,
and that immutable blobs eventually use Storage behind a port.

This file records those questions so they cannot be mistaken for inherited
law. They are outside the binding Build charter and outside implementation
until the user explicitly adopts the placement, each affected owner records its
side, and architecture review accepts the shared contracts. No current lane may
use this section as dependency authority.

</unadopted_proposals>

## Rejected destinations

- A second Rust package graph beside Cargo/Reindeer.
- A second configured execution graph beside Buck2.
- A shell, Python, or Node regeneration wrapper.
- A text-replacement overlay that mutates generated BUCK semantics.
- A user-facing dependency CLI or a CI scheduler inside Build.
- Reusing the frozen port engine for dependencies, repository graphs, or
  codemods.
- Storing generated BUCK, receipts, AST/HIR dumps, or semantic indexes as a
  manually maintained evidence corpus.
