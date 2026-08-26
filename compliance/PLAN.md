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
  -> L3c-S empty/scanner package structure + dependencies + Cargo.lock
  -> L3c-C bounded semantic contract
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

Create five package identities with scanners and empty semantic streams:

```text
compliance/ports/draft/pack-source/Cargo.toml
compliance/ports/draft/pack-source/BUCK
compliance/ports/draft/pack-source/build.rs
compliance/ports/draft/pack-source/src/lib.rs
compliance/ports/draft/cas/Cargo.toml
compliance/ports/draft/cas/BUCK
compliance/ports/draft/cas/build.rs
compliance/ports/draft/cas/src/lib.rs
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
targets. No tracked generated or hand-maintained membership file exists.

Exact Cargo and Buck dependency directions are frozen here:

```text
data-classification
  -> compliance-cas-draft
  -> compliance-evidence-domain
compliance-pack-source-draft
  -> compliance-evidence-domain
  -> compliance-pack-auth-awslc-draft
compliance-cas-draft
  -> compliance-evidence-domain
  -> compliance-cas-app
compliance-evidence-domain
  -> compliance-cas-app
semver
  -> compliance-evidence-domain
aws-lc-rs
  -> compliance-pack-auth-awslc-draft
```

`pack-source` has no dependencies. The core does not depend on the crypto
adapter. `data-classification`, `semver`, and `aws-lc-rs` already exist in the
workspace/lock; the lock delta adds exactly five local package blocks/edges and
no dependency version. Root workspace globs discover the packages; root
`Cargo.toml` is unchanged.

Build/test closure is all five packages under Cargo and Buck plus the exact
`data-classification` provider target. There are no reverse consumers outside
this set. Required reviewers are Compliance, Data, root Packs ownership,
Secrets/IAM security, Pipeline/Buck, and architecture. This is D-29 because it
binds an agreed Data port and freezes the future cryptographic dependency seam.

Success: both graphs expose the same five empty/scanner packages and dependency
edges, every handwritten file is at or below 300 lines, the exact lock delta is
fresh, and no product type, parser, limit, state transition, or route exists.

Failure: behavior/tests land, dependency direction differs, core imports
`aws-lc-rs`, a draft port gains an external consumer, a manual inventory is
tracked, or the lockfile contains unrelated churn.

Rollback: remove the five unrouted packages and exact lock blocks.

Fault evidence: scanner-only disposable add/rename/remove/non-Rust fixtures
prove Cargo/Buck membership parity without editing any stable root.

## L3c-C — Freeze the bounded semantic contract

Class: behavioral contract; no manifests, build files, lockfile, route, or
production adapter.

Unique-file envelope:

```text
compliance/ports/draft/pack-source/src/items/a_envelope.rs
compliance/ports/draft/pack-source/src/items/b_key_resolution.rs
compliance/ports/draft/pack-source/src/test_items/a_contract.rs
compliance/ports/draft/cas/src/items/a_requests.rs
compliance/ports/draft/cas/src/items/b_responses.rs
compliance/ports/draft/cas/src/items/c_errors.rs
compliance/ports/draft/cas/src/test_items/a_contract.rs
compliance/core/evidence-domain/src/items/a_identifiers.rs
compliance/core/evidence-domain/src/items/b_catalog_binding.rs
compliance/core/evidence-domain/src/items/c_projection_manifest.rs
compliance/core/evidence-domain/src/items/d_registry.rs
compliance/core/evidence-domain/src/items/e_limits.rs
compliance/core/evidence-domain/src/test_items/a_contract.rs
compliance/core/evidence-domain/src/test_items/b_limits.rs
compliance/facade/cas-app/src/items/a_service_contract.rs
compliance/facade/cas-app/src/test_items/a_service_contract.rs
```

Freeze the records and typed errors from `SPEC.md`, exact
`data-classification` identity, all numeric/default/hard ceilings, canonical
SemVer, SHA-256 label, v1 binary signing preimage/domain, 32-byte trusted-key
receipt, 64-byte Ed25519 signature, key-resolution refusal, and immutable
registry record. This slice defines an injected verification receipt but does
not perform cryptography. Protobuf is not added and the facade remains an
owner-local semantic trait.

Cargo/Buck closure is `pack-source`, `cas`, `evidence-domain`, and `cas-app`;
the empty auth adapter must still build so dependency drift is visible. No
external reverse consumer is allowed. Required reviewers are Compliance, Data,
Packs, Secrets/IAM security, Audit/Policy contract owners, and architecture.
Every source/test file is at or below 300 lines.

Success: independent golden encoders produce byte-identical signing preimages;
limit and limit-plus-one fixtures return stable errors before allocation/state;
the same exact classification value crosses port/core/facade; both build graphs
run every scanner member.

Failure: protobuf/JSON/YAML bytes become the preimage, a self-supplied key is
trusted, a bound is configurable above its maximum, a second classification
type appears, behavior imports the crypto adapter, or Cargo/Buck membership
differs.

Rollback: remove only these scanner-discovered files; the empty packages remain
unrouted.

Fault evidence: truncation, duplicate/trailing fields, noncanonical ids/SemVer,
unknown enums, overflow, every exact-limit/limit-plus-one pair, changed
fingerprints, and cross-namespace references.

## L3d-P — Cryptographic verification and pack admission oracle

Class: behavioral admission; may run beside L3d-R after L3c-C.

```text
compliance/adapters/draft/pack-auth-awslc/src/items/a_verifier.rs
compliance/adapters/draft/pack-auth-awslc/src/test_items/a_verifier.rs
compliance/core/evidence-domain/src/items/f_pack_admission.rs
compliance/core/evidence-domain/src/test_items/c_pack_admission.rs
```

The adapter uses the already-admitted `aws-lc-rs` edge for SHA-256 and Ed25519
verification. It accepts only a key-resolution result bound to namespace, key
id/generation, key digest, validity/revocation generation, and trusted Cell
interval. The engine enforces canonical framing, bounds, compiler-receipt shape,
catalog compare-and-swap, and lower/equal-conflicting/stale refusal. Tests use a
fake resolver; this is not a production Secrets adapter or pack filesystem
loader.

Exact build closure is `pack-source`, `pack-auth-awslc`, `evidence-domain`, and
their dependency targets under both graphs. There are no reverse consumers.
Required reviewers are Compliance, Packs, Secrets/IAM security, Policy for the
compiler-receipt boundary, and architecture. No manifest/lock/root/generated
path changes; every file is at or below 300 lines.

Success: golden valid envelopes admit deterministically; malformed, oversized,
unknown, unsigned, digest/signature-mismatched, untrusted/revoked/expired-key,
unsupported-schema, stale, and conflicting versions mutate nothing.

Failure: another algorithm/preimage is accepted, parser work exceeds a hard
bound, Policy evaluation appears, or a test key becomes production trust.

Rollback: remove the four unique files; the contract/scaffold remains unrouted.

Fault evidence: bit flips across every preimage field/signature/payload,
key-generation races, revocation between resolve/admit, limit-plus-one, stale
catalog generation, and concurrent conflicting admission.

## L3d-R — Data-class registry state machine

Class: behavioral registry; may run beside L3d-P after L3c-C.

```text
compliance/core/evidence-domain/src/items/g_classification_registry.rs
compliance/core/evidence-domain/src/test_items/d_classification_registry.rs
```

Implement `ABSENT -> PREPARED -> ACTIVE -> SUPERSEDED|REVOKED` with immutable
history, exact Data classification values, canonical labels/aliases,
applicability/evidence obligations, source pack/schema/digest, idempotency, and
entry/registry compare-and-swap generations. No alternate enum/parser/wrapper
is allowed. The 32-alias/64-byte/4,096-entry hard bounds apply.

Cargo/Buck closure is `data-classification`, `cas`, and `evidence-domain`; no
external reverse consumer or manifest/lock change. Required reviewers are
Compliance, Data, Packs, Policy, and architecture. Both files are at or below
300 lines.

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
compliance/core/evidence-domain/src/items/h_binding.rs
compliance/core/evidence-domain/src/items/i_projection.rs
compliance/core/evidence-domain/src/items/j_evidence_manifest.rs
compliance/core/evidence-domain/src/test_items/e_binding.rs
compliance/core/evidence-domain/src/test_items/f_projection.rs
compliance/core/evidence-domain/src/test_items/g_evidence_manifest.rs
```

Implement deterministic in-memory compare-and-swap oracles for binding,
target-ready projections, generation-bound acknowledgements, verified Audit-
reference coverage, manifest completion, and export admission. Target adapters
are fakes and Compliance never executes retention. The hard fan-out, page,
queue, idempotency, evidence-reference, and export limits apply.

Cargo/Buck closure is `pack-source`, `cas`, `evidence-domain`, and
`data-classification`; the facade contract also compiles as the only local
reverse consumer. Required reviewers are Compliance, Audit, Data, Storage,
Policy, Packs, and architecture. No manifest/lock/build/root/generated changes;
all files are at or below 300 lines.

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
queue bounds, and map typed engine results. Do not add protobuf, Connect,
gateway registration, persistence, or production adapters.

Cargo/Buck closure is `cas-app`, `cas`, `evidence-domain`, and all transitive
local contracts. There are no external reverse consumers. Required reviewers
are Compliance, IAM/Policy, Audit, Packs, and architecture. All files are at or
below 300 lines; manifests/lock/build/root/generated paths are read-only.

Success: tenant zero and ordinary tenants share one contract; forged/expired/
wrong-audience Policy evidence, Audit outage, changed fingerprints, tenant
mismatch, pagination drift, and overload fail before disclosure/mutation.

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

1. **L4a-S structure/lock:** after Gateway and API-version owners accept the
   package/version/retirement envelope, create
   `compliance/facade/cas-proto/{Cargo.toml,BUCK,build.rs,src/lib.rs}` and the
   sole `Cargo.lock` delta. Admit exact dependencies on `cas`, `prost`, `tonic`,
   and `tonic-prost-build`; no schema/handler/route behavior lands.
2. **L4a-C behavior:** add only
   `compliance/facade/cas-proto/proto/compliance-cas-v1.proto`, scanner items,
   and contract tests. Protobuf transports the L3c semantics and limits; it is
   not the pack signing preimage.
3. **L4a-R route:** a separately dispatched D-29 Gateway/IAM lane resolves and
   names the exact gateway registration files, default-deny Policy action,
   mTLS, quota, audit, version negotiation, and retirement behavior. No route
   change is authorized until that file-level envelope and reviewers exist.

Required reviewers: Compliance, Gateway, IAM/Policy, Audit, API compatibility,
security, and architecture. Promotion fails on a second semantic model, JSON
SSOT, private tenant-zero route, or auth/audit after handler logic.

### L4b — Durable records, recovery, and restore

The persistence provider is decision-blocked. Data has no accepted sold records
port at the L3a base, and Compliance cannot consume a Data core or Storage draft
port. Data/provider-owner and architecture must first accept one records
contract covering compare-and-swap generations, idempotency, snapshots, point-
in-time restore, encryption references, quorum durability receipts, and exit.

After that decision, a structural lock-writing lane may create only
`compliance/ports/draft/records/**` and
`compliance/adapters/draft/records-data/**`; behavior follows in unique
scanner-discovered files. The final exact provider and reverse-consumer paths
must be resolved from that accepted contract before dispatch. Required
reviewers are Compliance, Data, Cell, Secrets, Audit, security, and
architecture.

Only L4b can claim acknowledged durability or execute process-death,
durable-barrier, corrupt-WAL/snapshot, quorum-loss, point-in-time restore,
N/N+1 format, downgrade-barrier, and cell-loss campaigns. Success requires RPO
0 inside the declared tolerance and independently verified restore; failure
keeps the service unrouted.

### L4c — Production owner adapters

Each adapter is its own D-29 structural-dependency hop followed by a
unique-file behavior hop; lock-writing hops serialize. Required adapter seams
are:

| Adapter | Compliance destination | Provider/review owners |
|---|---|---|
| Root pack source | `compliance/adapters/draft/pack-source-repository/**` | Packs, Compliance, security, architecture |
| Trusted pack keys | `compliance/adapters/draft/pack-keys-secrets/**` | Secrets/IAM security, Packs, Compliance, architecture |
| Policy compiler/decision | `compliance/adapters/draft/policy/**` | IAM/Policy, Compliance, architecture |
| Audit evidence/query/emission | `compliance/adapters/draft/audit/**` | Audit, Compliance, security, architecture |
| Audit retention projection | `compliance/adapters/draft/projection-audit/**` | Audit, Compliance, architecture |
| Data retention projection | `compliance/adapters/draft/projection-data/**` | Data, Compliance, architecture |
| Storage retention/export | `compliance/adapters/draft/projection-storage/**` | Storage, Compliance, architecture |
| Trusted Cell interval | `compliance/adapters/draft/cell-clock/**` | Cell, Compliance, architecture |
| Tenant identity/PDP | `compliance/adapters/draft/iam/**` | IAM/Policy, Compliance, security, architecture |

No row authorizes a provider-core dependency. Before dispatch, each row must
name the accepted provider port, exact Cargo/Buck directions, full target and
reverse-consumer closure, exact file set, compatibility/removal policy, and
reviewers. Target receipts remain generation-bound; Compliance does not execute
owner workflows.

### L4d — Generated SLOs, operations, and promotion

After a routed durable service exists, land structure then behavior for
`compliance/observability/slo-ir/**`. Its owned Rust IR is the only hand-edited
SLO source; the repository materializer creates generated OpenSLO faces. Do not
restore the 13 handwritten files burned in L3b.

Add bounded-cell capacity/admission profiles, metrics/traces/logs, queue and
unit-cost telemetry, snapshot/restore runbooks as reconciled declarative state,
mixed-version upgrade/rollback barriers, repair/reconciliation, regional
evacuation, and recurring fault campaigns. Exact Observability, Cell,
Pipeline/materializer, and deployment paths/reviewers are resolved as D-29/D-30
envelopes before dispatch.

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
  package-add/delete lane. L3c-C is behavior-only and lock-free.
- L3d-P and L3d-R commute after L3c-C. L3d-B joins them; L3d-F follows the
  engine contract. Their scanner-discovered file sets are disjoint and no lane
  edits a parent root, manifest, build file, or lockfile.
- Every L4 structural dependency/package hop is a sole lock writer. Behavior
  follows after the structure merges. Cross-owner work is never inferred from
  a Compliance path allowance.
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
