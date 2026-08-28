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
| `build/toolchains/**`, `build/images/**` | Buck toolchain/cache declarations and one distroless image recipe; both still name Rust 1.97.1 while the root toolchain/workspace and hosted jobs require 1.98.0; several standards also narrate 1.97.1 as current | Partial, internally drifted Build assets and documentation; not a qualified image factory |
| Dependency/security update automation | `Cargo.lock`, `deny.toml`, and a nonblocking weekly cargo-deny action exist; the documented root `deps.toml`, owned bump bot, and owned supply-chain audit gate do not | Partial observation with no owned update actuator or closed campaign |
| `build/evidence/**`, `build/REORG-DRAIN.md` | Historical migration narratives | Residue; not runtime, generation, or admission evidence |

No owned Rust entrypoint currently regenerates `third-party/BUCK` atomically,
proves two clean runs byte-identical, emits a source-bound receipt, or supplies
a consumer-neutral freshness contract. Bare Reindeer output and the checked
file are not a proved round trip.

</current_state>

<charter>

## Decision: Build owns its named build inputs and declaration artifacts

- **achieves:** one meta owner for the Build responsibilities named by ADR-0719
  without treating every generated machine artifact as Build property or
  creating a second execution plane.
- **origin:** ADR-0719 assigns toolchains, host/guest images, Cloud
  Hypervisor/Firecracker/kernel pins, and the frozen port engine to `build/`,
  while Pipeline owns graph execution and Compute owns fleet agents.
- **rule:** `build/` MUST own the reproducibility contracts and owned tooling
  for pinned build toolchains, the host/guest image and kernel inputs assigned
  by ADR-0719, Cargo/Reindeer-to-Buck declaration artifacts, and separately
  adopted reusable repository analysis/transformation machinery. Artifacts
  outside that closed list remain with their capability owner. Build MUST NOT
  own capability engines, price/rate logic, fleet placement or agents, CI/CD
  scheduling, merge/review state, Storage, or a user-facing CLI.
- **ensure:** dependency and package review rejects unlisted generated-artifact
  ownership, Pipeline/GitHub concepts, cloud capability cores, pricing, fleet
  control, and persistent artifact bytes from Build core; cross-owner effects
  use explicit ports and sold facades.
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

<toolchain_dependency_lifecycle>

## Decision: MSRV, production stable, and preview channels are separate

- **achieves:** fast uptake of compiler, Cargo, dependency, and security fixes
  without silently dropping supported consumers or letting a floating nightly
  enter production provenance.
- **origin:** the root toolchain and `rust-version` currently both name 1.98.0
  even though they express different contracts; Buck/image pins still name
  1.97.1; the 2026-08-27 local nightly moved to rustc 1.100.0-nightly commit
  `bff8e12ff`; and 1.96.1 included Cargo CVE fixes while 1.97.1 fixed a compiler
  miscompilation, changes that cannot wait for an MSRV policy cycle.
- **rule:** Build MUST model the declared MSRV, qualified production stable,
  beta candidate, and exact dated nightly observation as separate identities.
  Production builds MUST track the latest qualified stable patch; an MSRV move
  MUST be a deliberate consumer-compatibility change with its own acceptance.
  Every Rust/Cargo/rustfmt/Clippy release and dependency candidate MUST carry a
  consuming-owner-supplied or accepted `ADOPT`, `BENCHMARK`, `DEFER`, or
  `REJECT` disposition bound to graph impact and evidence. Build MAY emit a
  nonbinding recommendation but MUST NOT select another owner's semantic
  adoption. Build MUST bind tool binaries/components,
  targets, LLVM, dependency sources/checksums, and vulnerability/yank/advisory
  facts, but MUST NOT assign CVE IDs, operate embargo/disclosure, claim CNA
  authority, decide product semantics, or orchestrate protected campaigns.
- **ensure:** qualification compiles and tests the declared MSRV separately
  from latest stable, runs beta and pinned-nightly differential shadow lanes,
  inventories every pin surface, requires an owner disposition for every
  release-note item, regenerates dependency/build declarations, maps
  CVE/GHSA/OSV/RustSec aliases with CNA provenance to affected graph closure,
  and emits one reversible candidate plus receipts. Missing,
  withdrawn, conflicting, newly published, or unverifiable facts fail closed.
- **overturn_when:** a user-adopted five-field decision replaces the channel or
  compatibility policy and proves equal security latency, provenance,
  consumer support, rollback, and complete release-feature accounting.

</toolchain_dependency_lifecycle>

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
  generate twice; require byte equality; validate the result; and publish only
  through a declared, qualified filesystem capability profile. Publication
  MUST hold an exclusive destination lease or use a genuine compare-and-swap
  primitive, use directory-relative no-follow operations and same-directory
  atomic replacement, and refuse an unsupported profile. Reconciliation MUST
  clear unapproved tool variables, perform no network access, invoke no shell
  wrapper, make no textual semantic overlay, and never hand-edit generated
  BUCK output.
- **ensure:** tests inject generator failure, mismatched double runs, malformed
  fixups, unsupported capability profiles, lease/CAS conflict, staged-write
  failure, pre-rename interruption, and directory-sync failure. On qualified
  profiles, failures before replacement retain the prior output and observers
  see either prior or new complete bytes. Every failure is typed; generation
  identity and publication-attempt receipt bind all inputs, tools, profiles,
  output, and actual success, typed failure, or indeterminate replacement and
  durability state without claiming uncertain durability.
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

<migration_provider_boundary>

## Decision: Build provides reusable migration machinery, not migration intent

- **achieves:** one reusable analysis/transformation provider while product
  owners retain domain correctness and Pipeline retains campaign blast radius.
- **origin:** the user's 2026-08-27 repository-evolution direction assigns
  semantic intent, postconditions, and acceptance to consuming product owners;
  reusable machinery to Build; and campaign/review orchestration to Pipeline.
- **rule:** Build MUST own reusable repository analysis and deterministic
  transformation machinery, and MUST require caller-supplied semantic intent,
  postconditions, and acceptance evidence. It MUST NOT own product acceptance,
  campaign execution or protected-review orchestration, production database or
  customer-data migration, traffic shifting, service deployment, or runtime
  cell evacuation.
- **ensure:** future contracts keep domain postconditions and campaign state out
  of Build core; conformance tests exercise provider-neutral machinery against
  owner-supplied fixtures; completed manual migrations become gold-corpus input
  but never implementation or automated-campaign proof.
- **overturn_when:** a user-adopted, five-field cross-owner decision reallocates
  the provider/consumer boundary and every affected owner lands its side in the
  same protected wave.

</migration_provider_boundary>

<source_declaration_integrity>

## Decision: one corpus-free first-party source-declaration relation

- **achieves:** one deterministic, versioned, corpus-free, unconfigured relation between participating first-party local-path Cargo dependencies and checked first-party BUCK labels, never a configured graph or census.
- **origin:** ADR-0719 D17a admits first-party stale-label prevention distinct from Reindeer-generated `third-party/BUCK`.
- **rule:** Build MUST evaluate complete HEAD from immutable inputs every time; base/delta facts only attribute and shard repairs. Only profiled normal/build/dev/optional/target-specific first-party local-path Cargo and checked-BUCK target/dependency pairs participate bidirectionally.
  Registry, Git, and other third-party Cargo edges, `third-party//`, and generated `third-party/BUCK` are excluded. The maintained `toml` and exactly pinned Meta `starlark_syntax` dependencies MUST sit behind Build parser ports and a closed, bounded grammar/profile.
  The core MUST emit sorted typed violations and deterministic, non-mutating `DeclarationRepairSetV1` values binding snapshot/profile provenance, complete semantic read/write facts and digest-or-absence preconditions, deterministic complete postimages, typed postconditions, postimage digests, an output digest, and a whole-set digest,
  deterministic caller-owner grouping, and pairwise-disjoint write sets; ambiguous ownership or overlap MUST refuse. Build MUST NOT invoke SCM/Git, discover owners, apply or mutate repairs, orchestrate campaigns/protected reviews, invoke Cargo, Buck2, a shell, any process, or a candidate executable, access a network,
  store a graph corpus/baseline/count/path inventory, interpret configured Starlark, or claim configured-graph/compile authority.
- **ensure:** adversarial and protected differential qualification cover valid binary/test subsets, both one-sided drift directions, every admitted/refused grammar form, deterministic replay, and every forbidden effect.
  Later-disjoint application is allowed only while every precondition matches; activation waits for full deterministic legacy repair without a baseline, and every profile-identity change requires requalification.
- **overturn_when:** a founder-accepted five-field replacement retains an equally fail-closed complete-HEAD relation and every ownership/effect boundary.

</source_declaration_integrity>

<unadopted_proposals>

## Contract details requiring explicit adoption — nonbinding

Status: **PROPOSED DETAILS; NOT ADOPTED**.

The provider/consumer boundary and only D17a's first-party declaration relation
are adopted. General compilation-unit, semantic-fact, conformance, codemod,
recipe, repository-input, campaign, Storage/Data, and cross-owner schemas remain
proposed details and decisions for their affected owners.

This file records those questions so the adopted provider boundary cannot be
mistaken for approval of any remaining schema or interface. Those details remain
outside implementation until the user adopts them, each affected owner records
its side, and architecture review accepts the shared contracts. No current lane
may use this section as dependency authority.

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
