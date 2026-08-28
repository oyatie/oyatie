---
doc_class: Owner-ADR
owner: build
status: Accepted
date: 2026-08-28
inherits:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
---

# Build decisions in force

This file specializes ADR-0719 for `build/`. It records the inherited Build
charter and the first declaration-integrity destination. It does not claim that
the reconciler, image factory, or proposed repository-semantic system has
landed.

<current_state>

## Evidence rechecked at `bef729c9af9653d8057fb46d8fa61e18bb4528a4`

| Surface | What exists | Maturity |
|---|---|---|
| Root `Cargo.toml` and `Cargo.lock` | Workspace package declarations and the locked Rust dependency solution | Live Cargo authority; shared, serialized files |
| `reindeer.toml` | Reindeer reads the root manifest, refuses unresolved build-script fixups, emits `third-party/BUCK`, and names an owned overlay under deleted `ci/` | Active input with a dead post-generation reference |
| `third-party/fixups/**` | 70 package-local fixup files: 66 inherited plus four run-only scanner bindings | Real generation inputs, but not yet covered by one provenance inventory or conformance receipt |
| `third-party/BUCK` | Checked generated dependency rules plus historical semantic mutations | Consumed by Buck2; its header names a different deleted shell wrapper, so clean reproduction is not proved |
| `build/dependency-declarations/**` | Six Reindeer transaction packages: pure-core home, two ports, two adapters, and a facade | Structural only; core/adapters have no behavior and the facade refuses service |
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
- **rule:** `build/` MUST own reproducibility contracts/tooling for pinned build
  toolchains, ADR-0719 host/guest image and kernel inputs, Cargo/Reindeer-to-Buck
  artifacts, and adopted reusable repository transformation machinery. Other
  artifacts stay with their owner. Build MUST NOT own capability engines,
  price/rate logic, fleet agents, CI/CD/review state, Storage, or a user-facing
  CLI. Production capabilities MUST be versioned APIs, declarative resources,
  and reconcilers; local diagnostic CLIs are retirement-marked scaffolding.
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

- **achieves:** a clean checkout regenerates exact dependency rules or refuses, never publishing a partial, patched, or self-asserted graph.
- **origin:** `reindeer.toml` and `third-party/BUCK` name different deleted wrappers; prior generation text-patched native rules.
  Reindeer constructs an ordered typed rule set before rendering, while file-only checks repeat errors.
- **rule:** declaration reconciliation MUST be an owned Rust transaction. It
  MUST run one pinned Reindeer with locked/offline exact inputs, binary/toolchain
  provenance, closed environment/sandbox, and reviewed fixups. Before
  qualification a reviewed producer-side patch/API against that exact source,
  bound by patch/fork/source/binary digests, MUST return one invocation's typed
  graph and bytes rendered from it; generator code owns the type, upstreaming is
  optional, and Build owns rollback. Full
  canonical DTO fields determine equality/digest—not private state, a second
  invocation, text reconstruction, `Rule::PartialEq`, or sort keys—and duplicate,
  colliding, unknown, or lossy graphs refuse. Two clean independent runs MUST be
  byte/graph identical. `ReindeerRuleGraphV1` is the primary semantic seam; a
  distinct Build port around exact-pinned maintained `starlark_syntax` MUST make
  its bounded projection the independent full-equality cross-check;
  no caller-authored expected graph or Oyatie reverse parser qualifies. Buck2 is
  consumer/configured authority: every promoted generator/parser/grammar/platform
  tuple, including Buck2 source/binary/toolchain/cell/config/prelude, MUST pass
  representative consumer qualification. Publication uses
  only qualified lease-or-CAS, no-follow, same-directory atomic capabilities;
  network, shell wrappers, text overlays, and generated-file hand edits refuse.
- **ensure:** qualification binds every input, tool, producer API/graph/renderer,
  parser, environment/sandbox, grammar/platform/Buck2 profile, output and
  receipt. Red-first tests inject dirty roots, undeclared state, duplicate/sort-
  key collisions, full-field/parser/byte disagreement, lossy/unknown syntax,
  malformed fixups, unsupported publication and every stage/rename/sync fault.
- **overturn_when:** Reindeer natively supplies equivalent fixup expressivity,
  independent maintained-parser cross-check, hermetic double-run proof,
  configured Buck2 evidence, validated atomic publication, and the same
  provenance receipt, allowing the transaction to shrink without lost proof.

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
  contracts MUST be adopted by all affected owners before implementation. Its
  surface is API/declarative-resource/reconciler first; diagnostics never become
  a durable CLI or merge authority.
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
  owner-supplied fixtures; completed manual migrations may become bounded
  adversarial fixtures, never implementation or automated-campaign proof.
- **overturn_when:** a user-adopted, five-field cross-owner decision reallocates
  the provider/consumer boundary and every affected owner lands its side in the
  same protected wave.

</migration_provider_boundary>

<source_declaration_integrity>

## Decision: one corpus-free first-party source-declaration relation

- **achieves:** Cargo and BUCK declaration changes converge through one
  deterministic, canonically owner-grouped repair contract without a configured-graph
  oracle, repository census, or manual label repair.
- **origin:** ADR-0719 D-17 adopted one Build-owned engine after recurring
  first-party labels survived stale until weekly Buck smoke; the existing
  reconciliation program covers generated third-party declarations only.
- **rule:** Build MUST own one versioned, unconfigured first-party source
  grammar and normalized complete-HEAD relation. Maintained Cargo and Starlark
  syntax dependencies MUST sit behind parser ports; Build MUST NOT hand-write a
  parser, interpret candidate Starlark, invoke SCM, or resolve ownership. The
  core MUST consume immutable caller-supplied snapshot bytes and ownership
  facts, fail closed on incomplete, unknown, unmapped, malformed, or ambiguous
  extraction, and emit sorted violations plus deterministic non-mutating
  output: exactly one canonical `DeclarationRepairSetV1` per evaluation,
  including zero actions/groups. It MUST bind engine/snapshot/profile/caller
  owner-authority/ownership-fact provenance; complete semantic reads and
  `semantic_writes`; their exact proposed-path projection; and digest-or-absence
  plus exact owner-or-absence on every bound path. `OwnerExpectation::Absent` is
  valid only for non-write reads. `semantic_writes` is sole action authority:
  one concrete-owner `Replacement` per proposed path and no others; each
  `Replacement` alone binds its path's complete present/absent postimage and
  canonical digest. The set binds typed postconditions, exact group-output
  digests, and a whole-set digest/identity over every other canonical field.
  Canonically ordered groups are exactly non-empty groups induced by distinct
  replacement owners; each replacement/path occurs once, writes are disjoint,
  and zero actions have zero groups.
  Absent-owner writes; empty/extraneous/missing/duplicate/ambiguous/wrong-owner/
  cross-owner/incomplete/overlapping groups; or any semantic/owner precondition
  mismatch refuse. Snapshot identity is provenance, not a global lock; a
  disjoint successor remains applicable only while every bound precondition
  matches. Generated `third-party/BUCK` remains exclusively in Reindeer. Reindeer
  qualification MUST precede a new parser dependency or package-graph change.
- **ensure:** an owner design freezes participating target/dependency kinds,
  admitted syntax, package/port placement, parser identities and supply-chain
  review before implementation. Pure relation tests precede adapters;
  adversarial and out-of-required-path differential qualification cover both
  declaration triggers, legitimate target subsets, every modeled Cargo kind,
  every admitted/refused grammar form, full-HEAD evaluation, deterministic
  V1 provenance, path-bound preconditions, postimages/postconditions/digests,
  exact owner grouping/zero behavior/refusals, disjoint-successor behavior, and
  forbidden effects. Only the protected qualification harness runs exact
  `cargo metadata --offline --locked --no-deps --format-version 1` and
  non-building `buck2 uquery`; the engine and required check never do. Profile
  identity changes requalify, and activation waits for zero legacy violations
  without a baseline or allowlist. Any Pipeline consumption is a separately
  adopted owner contract; Build neither applies nor regroups repairs.
- **overturn_when:** a founder-accepted five-field amendment preserves one
  protected verdict, a complete fail-closed first-party relation, deterministic
  preconditioned repairs, no frozen corpus or second compile plane, and the
  third-party/Reindeer boundary.

</source_declaration_integrity>

## Rejected destinations

- A second Rust package graph or configured execution graph beside Cargo/Buck2.
- A shell/Python/Node wrapper or text overlay that mutates generated semantics.
- A user-facing dependency CLI or a CI scheduler inside Build.
- A hand-written Cargo/Starlark parser, candidate interpreter, or Build-owned
  SCM/ownership resolver; reuse of the frozen port engine for dependency graphs.
- Storing generated BUCK, receipts, AST/HIR dumps, or semantic indexes as a manually maintained corpus.
