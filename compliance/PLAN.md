---
doc_class: Owner-PLAN
owner: compliance
status: Active
date: 2026-08-26
---

# Compliance remaining work

<baseline>

## L3a evidence

Compliance is not feature-ready.

- Seven workspace libraries exist. All seven are outside the terminal
  evidence-engine/CaS shape. DLP, DSR, eDiscovery, retention-DSR, trust portal,
  and the DSR use-case are explicitly burned by ADR-0719. The seventh,
  `core/retention`, evaluates holds and emits record-delete, KMS-shred, and
  cold-storage-purge permits; it is retention execution, not projection.
- `core/retention` also imports `data/core/data-boundary-kernel` directly rather
  than the agreed `data-classification` port. Because it has no lawful terminal
  Compliance role, the smallest correction is burn, not a temporary port hop,
  split, rename, or copied rehome.
- Nine handwritten Rust files total 9,401 lines and all exceed 300 lines. Files
  scheduled for terminal deletion are not split first.
- The seven packages pass 61 tests at base
  `2ed6af0b7ce0f48d071561c07a7489c0501c30f2`; that proves current behavior, not
  owner fitness or production service.
- Buck cannot parse root `compliance/BUCK` because it loads deleted
  `//governance/corpus/extract:yaml_facts.bzl`; package targets also name stale
  `//libs/data-boundary-kernel` labels.
- Root `packs/` has 30 tracked entries but no Compliance Rust loader. The Cedar,
  deployment/IaC, and 13 handwritten OpenSLO files are likewise unconsumed.
- No external Rust, Cargo, or Buck consumer exists for any current Compliance
  package identity. Internal edges do not justify rehome.

The executable dependency chain is:

```text
L3a owner law
  -> L3b terminal burn
  -> L3c-S empty/scanner package structure + dependency ports + Cell edge
  -> L3c-C bounded semantic and dependency-port contracts
       -> L3d-P pack authentication/admission
       -> L3d-R data-class registry
            -> L3d-B bind/project/evidence oracle
                 -> L3d-F unrouted facade oracle
                      -> L4 decision-gated production slices
```

L3d-P and L3d-R can run concurrently after L3c-C because their unique files are
disjoint. L3d-B consumes both. Structure, behavior, external-owner integration,
and promotion evidence never share one slice.

</baseline>

<sequence>

## L3a — Owner law and truthful inventory

Class: documentation/decision; this pull request.

Changed paths are only `compliance/{ADR,PRD,SPEC,PLAN,README}.md`. Success is
agreement across the five owner surfaces on current maturity, terminal burns,
contract, limits, signing, registry, sequenced production gates, success,
failure, rollback, and stage-available fault evidence. Failure is presenting a
target or future campaign as landed behavior.

## L3b — Terminal off-charter burn

Class: structural deletion/package-lock hop; no rehome or behavior.

Delete all seven current package cones, exactly 23 tracked files at the L3a
base:

```text
compliance/core/dlp/**
compliance/core/dsr/**
compliance/core/ediscovery/**
compliance/core/retention/**
compliance/core/retention-dsr/**
compliance/core/trust-portal/**
compliance/ports/dsr-usecase/**
```

Also delete the unconsumed/stale owner artifacts:

```text
compliance/BUCK
compliance/cedar/policies.cedar
compliance/iac/**
compliance/observability/slos/**
Cargo.lock
```

The last path means an exact generated lockfile delta, not deletion of the
lockfile. At the L3a base `iac/**` has 12 files and
`observability/slos/**` has 13 files. Preserve owner law, `OWNERS`, and root
`packs/**`.

Root `Cargo.toml` is unchanged. Cargo must remove exactly the seven local
Compliance package blocks and edges, with no third-party version churn. The
direct `data-boundary-kernel` edge disappears with the terminal package; no
replacement classification type or retention evaluator lands. A fresh reverse
search is a precondition in case the tree advances.

Logical closure after deletion is workspace metadata plus repository layout;
there is intentionally no Compliance Rust target. `buck2 targets
//compliance/...` must stop seeing the stale root loader. Required reviewers are
Compliance, Data for removal of the prohibited provider edge, and architecture.
No external consumer owner is required unless the precondition search finds a
new consumer, in which case this envelope stops and becomes D-29.

Success: every enumerated identity/artifact is absent, root packs and owner law
remain, Cargo metadata/lock freshness and Buck parsing close, and git history is
the only retained oracle.

Failure: retention behavior is renamed/re-homed/copied, any current package
still resolves, a consumer breaks, root packs are deleted, or the lockfile has
unrelated movement.

Rollback: revert the exact deletions and lock delta. No route, state, or format
changes exist.

Fault evidence: inject one reference to each deleted identity and the stale Buck
loader and prove analysis fails; the pre-burn 61-test receipt remains historical
evidence only.

## L3c-S — Structural CaS scaffold and dependency admission

Class: structure only; depends on L3b and is the next sole `Cargo.lock` writer.

Create eleven package identities with scanners and empty semantic streams:

```text
compliance/ports/draft/pack-source/Cargo.toml
compliance/ports/draft/pack-source/BUCK
compliance/ports/draft/pack-source/build.rs
compliance/ports/draft/pack-source/src/lib.rs
compliance/ports/draft/pack-auth/Cargo.toml
compliance/ports/draft/pack-auth/BUCK
compliance/ports/draft/pack-auth/build.rs
compliance/ports/draft/pack-auth/src/lib.rs
compliance/ports/draft/cas/Cargo.toml
compliance/ports/draft/cas/BUCK
compliance/ports/draft/cas/build.rs
compliance/ports/draft/cas/src/lib.rs
compliance/ports/draft/policy-client/Cargo.toml
compliance/ports/draft/policy-client/BUCK
compliance/ports/draft/policy-client/build.rs
compliance/ports/draft/policy-client/src/lib.rs
compliance/ports/draft/evidence-source/Cargo.toml
compliance/ports/draft/evidence-source/BUCK
compliance/ports/draft/evidence-source/build.rs
compliance/ports/draft/evidence-source/src/lib.rs
compliance/ports/draft/audit-sink/Cargo.toml
compliance/ports/draft/audit-sink/BUCK
compliance/ports/draft/audit-sink/build.rs
compliance/ports/draft/audit-sink/src/lib.rs
compliance/ports/draft/projection-target/Cargo.toml
compliance/ports/draft/projection-target/BUCK
compliance/ports/draft/projection-target/build.rs
compliance/ports/draft/projection-target/src/lib.rs
compliance/ports/draft/export-store/Cargo.toml
compliance/ports/draft/export-store/BUCK
compliance/ports/draft/export-store/build.rs
compliance/ports/draft/export-store/src/lib.rs
compliance/core/evidence-domain/Cargo.toml
compliance/core/evidence-domain/BUCK
compliance/core/evidence-domain/build.rs
compliance/core/evidence-domain/src/lib.rs
compliance/adapters/draft/pack-auth-awslc/Cargo.toml
compliance/adapters/draft/pack-auth-awslc/BUCK
compliance/adapters/draft/pack-auth-awslc/build.rs
compliance/adapters/draft/pack-auth-awslc/src/lib.rs
compliance/facade/cas-app/Cargo.toml
compliance/facade/cas-app/BUCK
compliance/facade/cas-app/build.rs
compliance/facade/cas-app/src/lib.rs
Cargo.lock
```

Every standard-library-only `build.rs` sorts `src/items/*.rs` and
`src/test_items/*.rs`, emits directory-level `rerun-if-changed`, tolerates the
structural stage's missing/empty directories, and writes only
`lib.generated.rs` and `tests.generated.rs` under `OUT_DIR`. Each stable
`src/lib.rs` contains the fixed includes and no semantic record, validation, or
test. Each Buck file stages the same two globs, creates the synthetic manifest
directories, runs `buildscript_run`, and supplies its `OUT_DIR` to library/test
targets. No tracked generated or hand-maintained membership file exists. This
exact sorted, missing/empty-tolerant, directory-rerun, fixed-output, Cargo/Buck-
parity rule is the **L3c-S D-41 scanner contract** inherited by every later
new Compliance package. Each structural family proves disposable Rust add,
rename, removal, and non-Rust canaries in both graphs before behavior lands.

The five added local port package names are
`compliance-policy-client-draft`, `compliance-evidence-source-draft`,
`compliance-audit-sink-draft`, `compliance-projection-target-draft`, and
`compliance-export-store-draft`. Their path leaf, Cargo package, rustc crate,
and Buck target identities follow D-30 exactly.

Exact Cargo and Buck dependency directions are frozen here; each line reads
`consumer <- provider`:

```text
compliance-evidence-domain <- compliance-pack-source-draft
compliance-evidence-domain <- compliance-pack-auth-draft
compliance-evidence-domain <- compliance-cas-draft
compliance-evidence-domain <- compliance-policy-client-draft
compliance-evidence-domain <- compliance-evidence-source-draft
compliance-evidence-domain <- compliance-audit-sink-draft
compliance-evidence-domain <- compliance-projection-target-draft
compliance-evidence-domain <- compliance-export-store-draft
compliance-evidence-domain <- data-classification
compliance-evidence-domain <- cell-clock-api
compliance-evidence-domain <- semver
compliance-pack-auth-draft <- cell-clock-api
compliance-pack-auth-awslc-draft <- compliance-pack-auth-draft
compliance-pack-auth-awslc-draft <- aws-lc-rs
compliance-cas-app <- compliance-cas-draft
compliance-cas-app <- compliance-evidence-domain
compliance-cas-app <- compliance-pack-auth-awslc-draft
compliance-cas-app <- compliance-policy-client-draft
compliance-cas-app <- compliance-evidence-source-draft
compliance-cas-app <- compliance-audit-sink-draft
compliance-cas-app <- compliance-projection-target-draft
compliance-cas-app <- compliance-export-store-draft
compliance-cas-app <- cell-clock-api
```

`pack-source` and `pack-auth` are distinct owner-local ports: the former
supplies bounded bytes, while the latter owns the `TrustedKeyResolver` and
`PackAuthenticator` boundaries. The aws-lc adapter implements
`PackAuthenticator` and consumes an injected `TrustedKeyResolver`; the core
depends on the auth port and invokes the authenticator, never the crypto
adapter. `cas-app` is the composition boundary and is the only edge that
selects `pack-auth-awslc`. The other five owner-local ports freeze the use cases
required by L3 fake composition: Policy compiler/decision evidence, verified
Audit reads, pre-ACK Audit writes, generation-bound target application, and
immutable export storage. They have no provider adapter yet.

The exact Cell provider is Cargo package `cell-clock-api` at
`cell/ports/clock` and Buck target `//cell/ports/clock:cell-clock-api`.
`compliance/ports/draft/pack-auth`, `compliance/core/evidence-domain`, and
`compliance/facade/cas-app` name that same dependency in both manifests and
Buck targets. The Rust identities are exactly `cell_clock_api::Clock` and
`cell_clock_api::Interval`; no local clock trait, point-time wrapper, or
trusted-time DTO is admitted.

`data-classification`, `cell-clock-api`, `semver`, and `aws-lc-rs` already
exist in the workspace/lock. The lock delta adds exactly eleven local package
blocks and their edges, but no Data/Cell block or dependency version. Root
workspace globs discover the packages; root `Cargo.toml` is unchanged.

Build/test closure is all eleven packages under Cargo and Buck plus the exact
`data-classification` and `cell-clock-api` provider targets. There are no
reverse consumers outside this set. Required reviewers are Compliance, Data,
Cell, root Packs ownership, Policy, Audit, Storage, Secrets/IAM security,
Pipeline/Buck, and architecture. This is D-29 because it binds agreed Data and
Cell ports and freezes future provider-adapter seams.

Success: both graphs expose the same eleven empty/scanner packages and
dependency edges, every handwritten file is at or below 300 lines, the exact
lock delta is fresh, and no product type, parser, limit, state transition, or
route exists.

Failure: behavior/tests land, dependency direction differs, core imports
`aws-lc-rs`, a draft port gains an external consumer, a manual inventory is
tracked, or the lockfile contains unrelated churn.

Rollback: remove the eleven unrouted packages and exact lock blocks.

Fault evidence: scanner-only disposable add/rename/remove/non-Rust fixtures
prove Cargo/Buck membership parity without editing any stable root.

## L3c-C — Freeze the bounded semantic contract

Class: behavioral contract; no manifests, build files, lockfile, route, or
production adapter.

Unique-file envelope:

```text
compliance/ports/draft/pack-source/src/items/a_envelope.rs
compliance/ports/draft/pack-source/src/test_items/a_contract.rs
compliance/ports/draft/pack-auth/src/items/a_contract.rs
compliance/ports/draft/pack-auth/src/items/b_key_resolution.rs
compliance/ports/draft/pack-auth/src/items/c_receipt.rs
compliance/ports/draft/pack-auth/src/items/d_frame.rs
compliance/ports/draft/pack-auth/src/test_items/a_contract.rs
compliance/ports/draft/pack-auth/src/test_items/b_golden_frame.rs
compliance/ports/draft/cas/src/items/a_requests.rs
compliance/ports/draft/cas/src/items/b_responses.rs
compliance/ports/draft/cas/src/items/c_errors.rs
compliance/ports/draft/cas/src/test_items/a_contract.rs
compliance/ports/draft/policy-client/src/items/a_contract.rs
compliance/ports/draft/policy-client/src/test_items/a_contract.rs
compliance/ports/draft/evidence-source/src/items/a_contract.rs
compliance/ports/draft/evidence-source/src/test_items/a_contract.rs
compliance/ports/draft/audit-sink/src/items/a_contract.rs
compliance/ports/draft/audit-sink/src/test_items/a_contract.rs
compliance/ports/draft/projection-target/src/items/a_contract.rs
compliance/ports/draft/projection-target/src/test_items/a_contract.rs
compliance/ports/draft/export-store/src/items/a_contract.rs
compliance/ports/draft/export-store/src/test_items/a_contract.rs
compliance/core/evidence-domain/src/items/a_identifiers.rs
compliance/core/evidence-domain/src/items/b_catalog_binding.rs
compliance/core/evidence-domain/src/items/c_projection_manifest.rs
compliance/core/evidence-domain/src/items/d_registry.rs
compliance/core/evidence-domain/src/items/e_limits.rs
compliance/core/evidence-domain/src/items/f_time_validity.rs
compliance/core/evidence-domain/src/test_items/a_contract.rs
compliance/core/evidence-domain/src/test_items/b_limits.rs
compliance/core/evidence-domain/src/test_items/c_time_validity.rs
compliance/facade/cas-app/src/items/a_service_contract.rs
compliance/facade/cas-app/src/test_items/a_service_contract.rs
```

Freeze the records and typed errors from `SPEC.md`, exact
`data-classification` identity, all numeric/default/hard ceilings, canonical
SemVer, the complete tagged v1 frame/type/tag/enum/collection grammar,
millisecond interval endpoints, SHA-256 payload/key/preimage derivations,
32-byte trusted-key contract, 64-byte Ed25519 signature, key-resolution
refusal, and immutable registry record. `pack-auth` defines both
`TrustedKeyResolver` and `PackAuthenticator`; only an authenticator invocation
can produce the receipt the core accepts. A caller-supplied receipt is not a
request field. The five dependency ports freeze Policy compiler/decision
evidence, verified Audit source references, durable pre-ACK Audit receipts,
generation-bound projection acknowledgements, and immutable export receipts.
L3 tests implement fakes only against these traits; core/facade packages may
not hide substitute traits.

The time contract accepts the exact `cell_clock_api::Interval` obtained by
`cas-app` from an injected `cell_clock_api::Clock`; no request or caller receipt
contains trusted time. Signed Unix-millisecond endpoints convert to
`SystemTime` with checked signed add/subtract from `UNIX_EPOCH`. Conversion
overflow, `earliest > latest`, or any interval not wholly contained by both the
pack and resolved-key windows is a typed refusal before mutation. For each
inclusive-lower/exclusive-upper window `[not_before, not_after)`, acceptance is
exactly `not_before <= interval.earliest && interval.latest < not_after`.
Touching or straddling the exclusive upper boundary, straddling the lower
boundary, choosing a midpoint, truncating uncertainty, or replacing the Cell
type with a local DTO is invalid. This slice does not perform cryptography.
Protobuf is not added and the facade remains an owner-local semantic trait.

Cargo/Buck closure is all eleven L3c-S packages plus the exact
`data-classification` and `cell-clock-api` targets; the empty auth adapter must
still build so dependency drift is visible. No external reverse consumer is
allowed. Required reviewers are Compliance, Data, Cell, Packs, Secrets/IAM
security, Audit/Policy contract owners, Storage, and architecture. Every
source/test file is at or below 300 lines.

Success: the production frame encoder and a test-only reference encoder share
only the frozen input record, not framing helpers or constants, and produce the
same byte-identical preimage and frozen expected byte array. The contract also
freezes expected payload/key/preimage digest, public-key, and Ed25519-signature
byte arrays for one deterministic seed; L3d-P performs the crypto comparison.
Every tag/type/length/value byte has a one-byte corruption fixture;
limit and limit-plus-one fixtures return stable errors before allocation/state;
the exact classification and Cell interval types cross port/core/facade; every
dependency fake implements a frozen local port; both build graphs run every
scanner member.

Failure: protobuf/JSON/YAML bytes become the preimage, a self-supplied key is
trusted, a bound is configurable above its maximum, a second classification
type appears, behavior imports the crypto adapter, or Cargo/Buck membership
differs. A local clock type, caller-supplied trusted time, unchecked Unix-ms
conversion, midpoint decision, or boundary-straddling acceptance also fails.

Rollback: remove only these scanner-discovered files; the empty packages remain
unrouted.

Fault evidence: truncation, duplicate/trailing fields, noncanonical ids/SemVer,
unknown enums, overflow, every exact-limit/limit-plus-one pair, changed
fingerprints, cross-namespace references, pre-epoch/overflowing timestamps,
reversed/widened intervals, and intervals just below/on/across both boundaries.

## L3d-P — Cryptographic verification and pack admission oracle

Class: behavioral admission; may run beside L3d-R after L3c-C.

```text
compliance/adapters/draft/pack-auth-awslc/src/items/a_verifier.rs
compliance/adapters/draft/pack-auth-awslc/src/test_items/a_verifier.rs
compliance/core/evidence-domain/src/items/g_pack_admission.rs
compliance/core/evidence-domain/src/test_items/d_pack_admission.rs
compliance/facade/cas-app/src/items/d_pack_auth_composition.rs
compliance/facade/cas-app/src/test_items/d_pack_auth_composition.rs
```

The adapter uses the already-admitted `aws-lc-rs` edge for SHA-256 and Ed25519
verification and implements the owner-local `PackAuthenticator` trait. It
invokes an injected `TrustedKeyResolver` from the same port and accepts only a
result bound to namespace, key id/generation, key digest,
validity/revocation generation, and the exact trusted
`cell_clock_api::Interval` supplied by composition. The engine
invokes that port, validates the returned request/preimage/key binding, and
enforces canonical framing, bounds, the L3c-C full-interval validity predicate,
compiler-receipt shape through `policy-client`, catalog compare-and-swap, and
lower/equal-conflicting/stale refusal. The facade composition selects the
aws-lc implementation and obtains time from an injected `cell_clock_api::Clock`
but remains unrouted and has no production resolver until L4c. No request can
inject a receipt or time value. Tests may inject fake authenticator, resolver,
Policy client, or Clock implementations only through their frozen ports. This
is not a production Secrets adapter or pack filesystem loader.

Exact build closure is `pack-source`, `pack-auth`, `pack-auth-awslc`,
`policy-client`, `evidence-domain`, `cas`, `cas-app`, `cell-clock-api`, and
their dependency targets under both graphs. There are no reverse consumers.
Required reviewers are Compliance, Cell, Packs, Secrets/IAM security, Policy
for the compiler-receipt boundary, and architecture. No manifest/lock/root/
generated path changes; every file is at or below 300 lines.

Success: aws-lc digest/verification results for the production frame and the
independent reference frame match every frozen golden digest/public-key/
signature byte; golden valid envelopes admit deterministically. Malformed,
oversized, unknown, unsigned, digest/signature-mismatched,
untrusted/revoked/expired-key, unsupported-schema, stale, and conflicting
versions mutate nothing. Checked pre-epoch conversion and intervals before,
on, or straddling either pack/key boundary follow the frozen refusal matrix.

Failure: another algorithm/preimage is accepted, parser work exceeds a hard
bound, Policy evaluation appears, or a test key becomes production trust.

Rollback: remove the six unique files; the contract/scaffold remains unrouted.

Fault evidence: bit flips across every preimage field/signature/payload,
key-generation races, revocation between resolve/admit, limit-plus-one, stale
catalog generation, and concurrent conflicting admission.

## L3d-R — Data-class registry state machine

Class: behavioral registry; may run beside L3d-P after L3c-C.

```text
compliance/core/evidence-domain/src/items/h_classification_registry.rs
compliance/core/evidence-domain/src/test_items/e_classification_registry.rs
```

Implement `ABSENT -> PREPARED -> ACTIVE -> SUPERSEDED|REVOKED` with immutable
history, exact Data classification values, canonical labels/aliases,
applicability/evidence obligations, source pack/schema/digest, idempotency, and
entry/registry compare-and-swap generations. No alternate enum/parser/wrapper
is allowed. The 32-alias/64-byte/4,096-entry hard bounds apply.

Cargo/Buck closure is `data-classification`, `cell-clock-api`, `pack-source`,
`pack-auth`, `cas`, `evidence-domain`, and the empty auth adapter; no external
reverse consumer or manifest/lock change. Required reviewers are Compliance,
Data, Cell, Packs, Policy, and architecture. Both files are at or below 300
lines.

Success: replay converges; stale/lower/equal-conflicting generations, duplicate
or shadowing aliases, unknown classifications, invalid intervals, source-digest
drift, and cross-tenant scope mutate nothing; historical generations remain
addressable.

Failure: type identity forks, aliases are ambiguous, old history is rewritten,
or a projection can omit its registry generation.

Rollback: remove the two unique files; no external registry route/state exists.

Fault evidence: concurrent prepare/activate/revoke, stale CAS, alias collision,
pack supersession/revocation, idempotency fingerprint reuse, and exact bound
edges.

## L3d-B — Binding, projection, and evidence oracle

Class: behavioral engine; requires L3d-P and L3d-R.

```text
compliance/core/evidence-domain/src/items/i_binding.rs
compliance/core/evidence-domain/src/items/j_projection.rs
compliance/core/evidence-domain/src/items/k_evidence_manifest.rs
compliance/core/evidence-domain/src/test_items/f_binding.rs
compliance/core/evidence-domain/src/test_items/g_projection.rs
compliance/core/evidence-domain/src/test_items/h_evidence_manifest.rs
```

Implement deterministic in-memory compare-and-swap oracles for binding,
target-ready projections, generation-bound acknowledgements, verified Audit-
reference coverage, manifest completion, and export admission. Test-only fakes
implement the already frozen `policy-client`, `evidence-source`, `audit-sink`,
`projection-target`, and `export-store` ports; production code depends only on
those traits and Compliance never executes retention. The hard fan-out, page,
queue, idempotency, evidence-reference, and export limits apply.

Cargo/Buck closure is `pack-source`, `pack-auth`, `pack-auth-awslc`, `cas`,
all five dependency ports, `evidence-domain`, `data-classification`, and
`cell-clock-api`; `cas-app` also compiles as the only local reverse consumer.
Required reviewers are Compliance, Audit, Data, Storage, Cell, Policy, Packs,
and architecture. No manifest/lock/build/root/generated changes; all files are
at or below 300 lines.

Success: same inputs replay byte-identically; duplicate receipts converge;
stale/skipped/reordered/incomplete/foreign-tenant input remains visible and
cannot complete a target or manifest generation.

Failure: Compliance erases/holds data, fabricates Audit evidence, reports a gap
complete, applies the wrong tenant/generation, exceeds a bound, or claims
durability from memory.

Rollback: remove the six unique files. No production state or route exists.

Fault evidence available here is deterministic loss/duplication/reorder of fake
receipts, concurrent bind/revoke, missing Audit ranges, cross-tenant ids,
bounded overload, and replay. Process death, WAL/snapshot corruption, quorum,
and restore are explicitly unavailable until L4b.

## L3d-F — Unrouted CaS facade oracle

Class: behavioral application facade; requires the L3d engine contracts.

```text
compliance/facade/cas-app/src/items/b_authz.rs
compliance/facade/cas-app/src/items/c_handlers.rs
compliance/facade/cas-app/src/test_items/b_authz.rs
compliance/facade/cas-app/src/test_items/c_handlers.rs
```

Implement default-deny handlers against fake ports: authenticate, verify Policy
provenance, bind tenant/idempotency/admission context, enforce all request/page/
queue bounds, obtain the exact interval from an injected
`cell_clock_api::Clock`, and map typed engine results. The Policy and Audit
fakes implement the L3c-C ports; no handler-local trait or caller-supplied
decision, receipt, or trusted time is allowed. Do not add protobuf, Connect,
gateway registration, persistence, or production adapters.

Cargo/Buck closure is `cas-app`, `cas`, `evidence-domain`, and all transitive
local contracts plus `cell-clock-api`. There are no external reverse consumers.
Required reviewers are Compliance, Cell, IAM/Policy, Audit, Packs, and
architecture. All files are at or below 300 lines; manifests/lock/build/root/
generated paths are read-only.

Success: tenant zero and ordinary tenants share one contract; forged/expired/
wrong-audience Policy evidence, Audit outage, changed fingerprints, tenant
mismatch, pagination drift, clock widening/boundary straddling, and overload
fail before disclosure/mutation.

Failure: a private route, caller-asserted authorization, permit/forbid response,
unbounded work, or external owner import lands.

Rollback: remove the four unique files. The product remains unavailable.

Fault evidence: default-deny contract cases, cancellation/retry, tenant/page
token swaps, exact bound edges, fake dependency outage, and queue saturation.

</sequence>

<production_sequence>

## L4 — Decision-gated production path

L3 ends at an unrouted, in-memory oracle. The following are real remaining
work, not deferred acceptance evidence for L3.

### L4a — Protobuf/Connect and gateway publication

1. **L4a-S canonical proto/codegen structure:** after Gateway, API
   compatibility, and the repository Connect-codegen owner accept the
   version/retirement and generated-artifact contract, create
   `compliance/facade/proto/compliance/cas/v1/BUCK`, update only
   `compliance/facade/cas-app/{Cargo.toml,BUCK,build.rs}`, and apply the sole
   exact `Cargo.lock` delta. The existing app build script gains the accepted
   Connect generator/runtime edge, tolerates the structurally absent schema,
   preserves its L3c-S sorted `src/items` and `src/test_items` inputs and
   `lib.generated.rs` / `tests.generated.rs` outputs, and adds only
   `connect.generated.rs` under the same stable `OUT_DIR`. Buck stages the same
   scanner inputs plus the accepted proto input and produces the same three
   outputs. No second Rust/proto crate, RPC schema, handler, route, or
   handwritten generated file lands. This stage is non-dispatchable until the
   D-30 receipt records exact Cargo packages, Buck labels, all three outputs,
   Rust add/rename/remove/non-Rust plus proto add/remove/change canary parity,
   reverse closure, and compatibility/removal.
2. **L4a-C frozen schema:** add only
   `compliance/facade/proto/compliance/cas/v1/cas.proto`, with protobuf package
   `compliance.cas.v1`. The accepted repository compatibility checker consumes
   that exact file through the L4a-S Buck target. This file is the one wire source of truth and transports the
   L3c semantics and limits; it is never the pack-signing preimage. Protobuf
   API review freezes field identities, reserved numbers, pagination,
   idempotency, typed errors, compatibility, and retirement before behavior.
3. **L4a-A Connect application binding:** add only
   `compliance/facade/cas-app/src/items/e_connect_service.rs` and
   `compliance/facade/cas-app/src/test_items/e_connect_service.rs`. The existing
   `compliance-cas-app` scanner binds accepted generated Connect types to the
   frozen semantic handlers. Public ingress is HTTP/3 and east-west ingress is
   HTTP/2 through the canonical Connect contract. No `tonic`, standing gRPC
   service/status/trailers, second semantic model, manifest, or lock delta is
   allowed.
4. **L4a-G disabled registration:** after L4a-A, a separately dispatched D-29
   Gateway/IAM structural lane may register the accepted schema, service
   identity, and version metadata only in an explicit disabled state. Its
   receipt names every foreign file, reverse closure, generated input, default-
   deny Policy action, mTLS/quota/audit requirements, compatibility, and
   reviewers. Disabled means no listener/VIP/path can select the handler and an
   accidental invocation fails closed before disclosure or mutation. This
   Compliance envelope grants no foreign path and the registration cannot
   advertise service availability.
5. **L4a-R route activation:** this provider-owned D-29 behavior lane is not
   dispatchable after L4a-G alone. It joins on L4b-R durable recovery/restore
   evidence and every required L4c adapter behavior row: pack source, trusted-
   key resolution, Policy client, Audit evidence source, pre-ACK Audit sink,
   all three projection targets, and Storage export. The join also requires
   exact `cell_clock_api::Clock` composition and the L3c-C interval-boundary
   refusal campaign. Only then may its separately accepted exact Gateway/IAM
   files change disabled registration to admitted traffic with default-deny
   authn/Policy, mTLS, quota, pre-ACK Audit, version negotiation, rollback, and
   adapter-outage tests. Any missing/unavailable dependency keeps the route
   disabled; no fake or in-memory authority can satisfy the join. Activation
   makes only a declared pre-production cell eligible for fault/SLO trials;
   L4d-O remains the separate gate for production tenant promotion.

Required reviewers: Compliance, Gateway, IAM/Policy, Audit, API compatibility,
Connect/codegen owner, security, and architecture. Promotion fails on a second
semantic model, JSON SSOT, gRPC/tonic substitute, private tenant-zero route, or
auth/audit after handler logic. L4a-S, L4a-C, L4a-A, and disabled L4a-G are
ordered; L4a-R waits for the L4b/L4c/Cell join. None shares a structure,
behavior, or foreign-owner write.

### L4b — Durable records, recovery, and restore

Data's planned `data/ports/draft/records` is not yet a sold cross-owner port and
its own plan forbids draft external consumers. Compliance therefore does not
duplicate that records contract or import a Data core. Data and architecture
must first promote an accepted engine-neutral provider at
`data/ports/records` (`data-records`) with exact Cargo/Buck targets, durable
compare-and-swap/idempotency/snapshot/restore semantics, reverse closure, and a
D-29 external-consumer decision. Until then every L4b stage is
non-dispatchable.

1. **L4b-S local persistence seam:** after that provider decision, create
   exactly the two four-file roots
   `compliance/ports/draft/catalog-store/{Cargo.toml,BUCK,build.rs,src/lib.rs}`
   (`compliance-catalog-store-draft`) and
   `compliance/adapters/draft/catalog-store-data/{Cargo.toml,BUCK,build.rs,src/lib.rs}`
   (`compliance-catalog-store-data-draft`), update only
   `compliance/core/evidence-domain/{Cargo.toml,BUCK}` and
   `compliance/facade/cas-app/{Cargo.toml,BUCK}`, and take the sole exact
   `Cargo.lock` delta. `catalog-store` is a Compliance-shaped repository for
   catalog/binding/projection/manifest/idempotency compare-and-swap state, not
   another generic records contract. Under both graphs the exact directions
   are `evidence-domain <- catalog-store`, `catalog-store-data <-
   catalog-store + data-records`, and `cas-app <- catalog-store-data`. Both new
   roots inherit the full L3c-S D-41 scanner contract with fixed
   `lib.generated.rs` / `tests.generated.rs` outputs, equivalent Buck
   `buildscript_run` inputs, and disposable add/rename/remove/non-Rust parity
   evidence. The lock adds exactly two local blocks/edges and no provider or
   third-party version.
2. **L4b-C persistence behavior:** add only
   `compliance/ports/draft/catalog-store/src/items/a_contract.rs`,
   `compliance/ports/draft/catalog-store/src/test_items/a_contract.rs`,
   `compliance/adapters/draft/catalog-store-data/src/items/a_store.rs`,
   `compliance/adapters/draft/catalog-store-data/src/test_items/a_store.rs`,
   `compliance/core/evidence-domain/src/items/l_durable_repository.rs`,
   `compliance/core/evidence-domain/src/test_items/i_durable_repository.rs`,
   `compliance/facade/cas-app/src/items/f_durable_composition.rs`, and
   `compliance/facade/cas-app/src/test_items/f_durable_composition.rs`.
   Contract suites prove atomic generations, idempotency, durable receipt
   binding, encryption references, and refusal before the routed facade.
3. **L4b-R recovery:** add only
   `compliance/adapters/draft/catalog-store-data/src/items/b_snapshot_restore.rs`,
   `compliance/adapters/draft/catalog-store-data/src/test_items/b_snapshot_restore.rs`,
   `compliance/core/evidence-domain/src/items/m_recovery.rs`,
   `compliance/core/evidence-domain/src/test_items/j_recovery.rs`,
   `compliance/facade/cas-app/src/items/g_recovery_gate.rs`, and
   `compliance/facade/cas-app/src/test_items/g_recovery_gate.rs`. It proves
   process-death/durable-barrier, corrupt WAL/snapshot quarantine, quorum loss,
   point-in-time restore, N/N+1 format, downgrade barriers, and cell loss.

Required reviewers are Compliance, Data, Cell, Secrets, Audit, security, and
architecture. Success requires RPO 0 within the declared tolerance and an
independently verified restore; failure or an unpromoted/draft provider keeps
the service unrouted. Each stage is independently revertible; L4b-S removes
only the two packages/graph delta, while later rollback removes only its unique
scanner files.

### L4c — Production owner adapters

L4c begins only after each provider accepts a sold port/target and reverse
closure. The ports and adapters are not aliases for provider contracts:
Compliance ports express its required use cases, and adapters translate an
accepted provider port into them. Draft ports remain owner-local.

The five owner-local use-case ports and their contracts already exist from
L3c-S/C before any oracle behavior. L4c neither recreates them nor moves traits
out of core/facade after the fact.

1. **L4c-A-S adapter structure:** each following row is a separate sole-lock
   D-29/D-30 slice after L3d-F. It creates exactly the named four-file package
   root, updates only `compliance/facade/cas-app/{Cargo.toml,BUCK}`, and applies
   its exact `Cargo.lock` block. Dependency direction is always `adapter <-
   named local port + accepted provider port`, then `cas-app <- adapter` in
   Cargo and Buck. Every new adapter root inherits the complete L3c-S D-41
   scanner contract: fixed `lib.generated.rs` / `tests.generated.rs`, missing/
   empty tolerance, directory rerun, equivalent Buck `buildscript_run`, and
   per-row disposable Rust add/rename/remove/non-Rust parity evidence. A row
   adds exactly one local lock block plus its accepted provider edges and no
   unrelated dependency version.

   | Local port | Exact adapter package root | App prefix | Provider decision/review owners |
   |---|---|---:|---|
   | `pack-source` | `compliance/adapters/draft/pack-source-repository` | `h_pack_source_repository` | Repository/Packs, Compliance, security, architecture |
   | `pack-auth` (`TrustedKeyResolver`) | `compliance/adapters/draft/pack-auth-secrets` | `i_pack_auth_secrets` | Secrets, Packs, Compliance, security, architecture |
   | `policy-client` | `compliance/adapters/draft/policy-client-policy` | `j_policy_client` | IAM/Policy, Compliance, architecture |
   | `evidence-source` | `compliance/adapters/draft/evidence-source-audit` | `k_evidence_source` | Audit, Compliance, security, architecture |
   | `audit-sink` | `compliance/adapters/draft/audit-sink-audit` | `l_audit_sink` | Audit, Compliance, security, architecture |
   | `projection-target` | `compliance/adapters/draft/projection-target-audit` | `m_projection_audit` | Audit, Compliance, architecture |
   | `projection-target` | `compliance/adapters/draft/projection-target-data` | `n_projection_data` | Data, Compliance, architecture |
   | `projection-target` | `compliance/adapters/draft/projection-target-storage` | `o_projection_storage` | Storage, Compliance, architecture |
   | `export-store` | `compliance/adapters/draft/export-store-storage` | `p_export_storage` | Storage, Compliance, security, architecture |

   A row is non-dispatchable until its receipt records the provider's exact
   Cargo/Buck target, target/reverse closure, compatibility/removal policy, and
   named reviewers. No table row grants a foreign write or provider-core edge.
2. **L4c-A-C adapter behavior:** after a row's L4c-A-S structure merges, add only
   scanner-discovered `<adapter>/src/items/a_adapter.rs`,
   `<adapter>/src/test_items/a_contract.rs`, and the exact
   `compliance/facade/cas-app/src/items/<app-prefix>.rs` and
   `compliance/facade/cas-app/src/test_items/<app-prefix>.rs` from that row. Its
   provider conformance suite must prove tenant/generation/receipt binding,
   idempotent replay, loss/duplication/reorder, timeout, and fail-closed outage.

Trusted time uses the L3c-S exact `cell-clock-api` Cargo/Buck edge directly;
`cas-app` injects `cell_clock_api::Clock` and passes its exact `Interval`
through the frozen validity contract. It does not invent a `cell-clock`
adapter, point clock, or local DTO. Tenant identity and PDP evidence arrive
through the canonical Gateway/Policy contract; Compliance does not add an
`iam` adapter. Target receipts remain generation-bound, and Compliance never
executes an owner workflow.

### L4d — Generated SLOs, operations, and promotion

Handwritten Rust never lives under `observability/slos`. Sequence is:

1. **L4d-S canonical IR port:** after Observability and Pipeline accept the
   provider contract, create exactly
   `compliance/ports/slo/{Cargo.toml,BUCK,build.rs,src/lib.rs}` as package
   `compliance-slo`, plus the sole `Cargo.lock` block. Its D-30 receipt freezes
   the exact accepted materializer target and reverse closure; this stage is
   non-dispatchable before that decision. It contains structure only and
   inherits the complete L3c-S D-41 contract: sorted missing/empty-tolerant
   item/test-item scans, directory rerun, fixed `lib.generated.rs` /
   `tests.generated.rs`, equivalent Buck staging/`buildscript_run`, and
   disposable add/rename/remove/non-Rust membership parity. The lock adds one
   local block plus only the accepted materializer edge.
2. **L4d-C bounded IR behavior:** add only

   ```text
   compliance/ports/slo/src/items/a_objectives.rs
   compliance/ports/slo/src/items/b_indicators.rs
   compliance/ports/slo/src/items/c_budgets.rs
   compliance/ports/slo/src/items/d_promotion.rs
   compliance/ports/slo/src/test_items/a_contract.rs
   compliance/ports/slo/src/test_items/b_bounds.rs
   ```

   The IR is the sole hand-edited Compliance SLO source and freezes bounded-cell
   capacity/admission profiles, metric identity, units/windows, queue/unit-cost
   signals, recovery objectives, and promotion gates. No materialized file is
   edited here.
3. **L4d-M materialization:** a separately dispatched D-29 Pipeline/
   Observability lane changes only its accepted provider-owned integration
   files and consumes `compliance-slo`. The repository materializer, never an
   author, writes exactly
   `compliance/observability/slos/{cas-availability,cas-latency,projection-freshness,evidence-coverage,durability-recovery,tenant-isolation}.generated.openslo.yaml`.
   Cargo/Buck freshness and generator canaries prove those six outputs derive
   from the IR. The 13 handwritten files burned in L3b stay absent.
4. **L4d-O operations/promotion:** separately accepted Cell/Observability/
   deployment envelopes add metrics/traces/logs, reconciled snapshot/restore
   runbooks, mixed-version upgrade/rollback barriers, repair/reconciliation,
   regional evacuation, and recurring fault campaigns. Exact foreign files and
   reviewers are named by those D-29 receipts; this plan grants none.

Production promotion requires all PRD SLOs measured at the sold facade under
steady/noisy/fault load; signed pack/key rotation; registry replay; projection
and evidence convergence; restore; mixed-version upgrade; and every external
adapter outage. Until then no CaS availability, durability, evidence coverage,
certification, or compliance-conformance claim is valid.

</production_sequence>

<coordination>

## Lane and lock rules

- L3b is next. It is terminal deletion, not retention refactoring.
- L3c-S follows L3b and owns `Cargo.lock`; it serializes with every monorepo
  package-add/delete lane. It creates every L3 dependency port and the exact
  Cell edge before behavior. L3c-C freezes contracts across the eleven-package
  closure and is behavior-only and lock-free.
- L3d-P and L3d-R commute after L3c-C. L3d-B joins them; L3d-F follows the
  engine contract. Their scanner-discovered file sets are disjoint and no lane
  edits a parent root, manifest, build file, or lockfile.
- Every L4 structural dependency/package hop is a sole lock writer. L4a's proto
  and Connect-codegen structure owns its exact app build/lock delta; schema,
  app binding, and disabled Gateway registration follow as three distinct
  behavior/foreign-owner hops. L4b structure precedes persistence behavior and
  recovery. L4c provider adapters serialize only their structure/lock hops and
  then fan out on unique behavior files because their local ports already froze
  in L3. L4a-R activation waits at the join of L4b-R, every mandatory L4c
  adapter behavior, and Cell interval evidence; it never precedes them. L4d IR
  structure precedes IR behavior, provider-owned materialization, and
  operations. Every new L4 Rust package explicitly inherits the L3c-S D-41
  scanner/Buck/canary contract; L4a codegen preserves the two existing scanner
  outputs while adding its third. Cross-owner work is never inferred from a
  Compliance path allowance.
- Any root `packs/`, Data, Audit, Storage, Cell, IAM/Policy, Secrets, Gateway,
  Observability, deployment, protobuf-root, generated, or foreign consumer
  write needs a separately dispatched exact D-29/D-30 envelope and named
  reviewers.
- Moving `data-classification` identity is not Compliance work. L3 consumes the
  agreed port; a provider migration remains Data-led across all consumers.

## Next dispatch

Dispatch L3b with the exact deletion set, current reverse-consumer proof, seven-
block lock delta, Cargo metadata/lock freshness, Buck parse evidence, protected
path admission, signed commit, and independent Compliance/Data/architecture
review. Do not split or preserve any current Compliance Rust package.

</coordination>
