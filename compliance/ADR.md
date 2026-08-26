---
doc_class: Owner-ADR
owner: compliance
status: Accepted
date: 2026-08-26
inherits:
  - docs/decisions/ADR-0719-eac-serving-control-north-star.md
---

# Compliance decisions in force

This file specializes ADR-0719 for `compliance/`. It records current owner law
and the destination contract. It is not evidence that the CaS facade or
evidence engine has landed.

<current_state>

## Evidence at L3a

| Surface | What exists | Maturity |
|---|---|---|
| Rust packages | Seven library packages: retention plus DLP, DSR, eDiscovery, retention-DSR, trust portal, and a DSR use-case port | Typed in-memory validation only; no service listener or durable authority |
| File shape | Nine handwritten Rust files total 9,401 lines; every one exceeds the 300-line budget | Structural preparation is required before feature work |
| Retention | `core/retention` evaluates legal holds and returns record-delete, KMS-shred, and cold-storage-purge permits while importing `data/core/data-boundary-kernel` directly | Off-charter execution authority and prohibited core edge; terminal burn with the other Workspace packages |
| Cargo | All seven packages are workspace members; their 61 unit/integration tests pass at the L3a base | Test evidence for existing behavior only |
| Buck | Package targets use stale deleted `//libs/data-boundary-kernel` labels and root `compliance/BUCK` loads deleted `//governance/corpus/extract:yaml_facts.bzl` | `buck2 targets //compliance/...` fails before target analysis |
| Packs | Root `packs/` contains 30 tracked entries: 24 Markdown files, three YAML files, two JSON files, and `OWNERS`; no Compliance Rust loader consumes them | Documentation/scaffold data, not the Cedar+IR CaC destination |
| Deployment and SLO files | Handwritten Helm, Kubernetes, Terraform, OpenBao, Cedar, and 13 OpenSLO files describe collectors, DSAR, portals, and unrelated platform behavior | No reconciler, facade, or engine consumes them; they are not deployment or SLO evidence |

There is no current evidence engine, CaS bind/project/export facade, pack
catalog authority, signed pack parser, audit adapter, retention projection
publisher, network endpoint, production SLO, or horizontal-scale behavior.

</current_state>

<boundary>

## Decision: evidence engine and CaS facade only

- **achieves:** one cloud owner for compliance evidence and pack control-plane
  projection without turning the capability into a suite of legal products.
- **origin:** the current tree implements DLP, DSR, eDiscovery, retention-DSR,
  and trust-portal models, while ADR-0719 D-14/D-19 assigns Compliance only the
  evidence engine and Compliance-as-a-Service facade.
- **rule:** `compliance/` MUST own the evidence engine and CaS catalog, bind,
  list, preview/project, and export facade. It MUST NOT own a DLP engine, DSR
  workflow or API, eDiscovery product, trust portal, Merkle log, pack source
  data, or a second policy decision point. `core/dlp`, `core/dsr`,
  `core/ediscovery`, `core/retention-dsr`, `core/trust-portal`, and
  `ports/dsr-usecase` MUST burn without rehome.
- **ensure:** dependency and path review admits only evidence, catalog,
  binding, projection, export, and data-class-registry behavior; the plan names
  the exact burn and rejects replacement packages with equivalent product
  semantics.
- **overturn_when:** a founder-accepted owner-boundary decision adds a cloud
  capability, names its public contract and operators, and updates every
  affected owner in the same change.

</boundary>

<authority>

## Decision: Audit records events; Compliance proves coverage

- **achieves:** tenant-exportable compliance evidence without a fifth copy of
  records or a competing tamper-evident log.
- **origin:** current artifacts describe an evidence collector and audit-chain
  seal as though Compliance owns collection, storage, and the Merkle record.
- **rule:** `audit/` MUST remain the authoritative tamper-evident event record.
  Compliance MUST consume verified Audit references, evaluate evidence
  coverage against a bound pack revision, and publish versioned manifests and
  exports. It MUST NOT seal a second Merkle log, fabricate missing events, or
  acknowledge an export as complete when required evidence is absent.
- **ensure:** every manifest binds tenant, pack/binding generation, policy and
  schema revisions, source Audit cursor/range, item digests, gaps, and export
  generation; tests prove a missing, foreign-tenant, unverified, or reordered
  source cannot produce a complete receipt.
- **overturn_when:** an Audit-owner and architecture decision replaces the
  evidence boundary while preserving one tamper authority, tenant export, gap
  detection, and independently verifiable provenance.

</authority>

<packs_and_policy>

## Decision: CaC is input; Policy is the only Check

- **achieves:** granular jurisdiction/program projection without pack fetches
  or a second evaluator on the serving path.
- **origin:** ADR-0719 keeps root `packs/` as signed Cedar+IR data, assigns
  serving evaluation to `policy/`, and limits CaS to control-plane
  bind/list/preview/evidence behavior.
- **rule:** Compliance MUST ingest packs through a bounded, verified source
  port and MUST bind immutable pack id, semantic version, schema revision,
  content digest, signature provenance, plane, and projection dimensions.
  CaS MAY preview which packages and obligations apply, but it MUST NOT return
  a product authorization decision, compile a private PDP, fetch packs during
  `Check`, encode pack ids as ReBAC tuples, or own root `packs/` data.
- **ensure:** malformed, unsigned, digest-mismatched, unsupported-schema,
  unknown-plane/dimension, combinatoric-namespace, and stale-version inputs
  fail before candidate or binding mutation; traces prove request-time policy
  Check reads only the Policy snapshot.
- **overturn_when:** a joint Compliance/Policy/Packs decision proves a smaller
  boundary with one PDP, no serving-path fetch, signed provenance, rollback
  refusal, and the same first-/third-party contract.

</packs_and_policy>

<pack_integrity>

## Decision: canonical signed envelopes, verified off the serving path

- **achieves:** deterministic pack identity and rollback refusal without
  signing ambiguous protobuf/JSON/YAML bytes or trusting a self-supplied key.
- **origin:** “signed pack” and “bounded parser” are not implementable contracts
  unless the preimage, digest, signature, key resolution, and hard ceilings are
  frozen before admission behavior lands.
- **rule:** Pack v1 MUST use the complete tagged binary grammar, field/type/enum
  codes, widths, collection framing, inclusive/exclusive millisecond interval,
  and digest derivations frozen in `SPEC.md`, with SHA-256 and Ed25519 under the
  `oyatie.compliance.pack.v1` domain. The engine MUST invoke verification
  through its owner-local `pack-auth` port; it MUST NOT accept a
  caller-constructed verification receipt. The port MUST resolve a trusted key
  by namespace, key id, and key generation and reject unknown, ambiguous,
  revoked, expired, or self-asserted keys before catalog mutation. Protobuf is
  the facade wire contract, not the signing preimage.
- **ensure:** a production encoder and an independently implemented test
  encoder share only the frozen input record and match exact preimage, payload
  digest, key digest, preimage digest, public key, and signature golden bytes;
  field/header bit flips plus byte, depth, count, identifier, fan-out, and queue
  limit-plus-one cases fail closed. Crypto and trusted-key adapters require
  Packs, Secrets/IAM security, and architecture review.
- **overturn_when:** an accepted Packs/Security/Compliance decision replaces
  the algorithm or canonicalization while preserving deterministic identity,
  domain separation, key provenance, bounded work, and rollback refusal.

</pack_integrity>

<projection>

## Decision: project obligations; do not execute owner workflows

- **achieves:** one pack binding can drive retention and evidence behavior in
  each owning engine without Compliance becoming their data plane.
- **origin:** ADR-0719 assigns retention execution to `audit/`, `data/`, and
  `storage/`, while current DSR packages orchestrate and prove erasure inside
  Compliance.
- **rule:** CaS MUST publish immutable, versioned projections for the target
  owner. A retention projection MUST name tenant, target capability, pack and
  binding generation, classification selector, obligation, effective interval,
  source digest, and supersession rule. Compliance MUST NOT directly erase,
  hold, move, or duplicate Audit/Data/Storage state and MUST NOT report their
  work complete without a verified target receipt.
- **ensure:** conformance uses fake Audit/Data/Storage consumers to prove
  idempotent apply, stale-generation refusal, gap visibility, and no
  cross-tenant target; production adapters require separate provider-owner and
  architecture review.
- **overturn_when:** every affected owner accepts another single projection
  protocol that preserves independent authority, replay, fencing, and evidence
  provenance.

</projection>

<classification_registry>

## Decision: registry state without a type fork

- **achieves:** pack-versioned data-class catalog and evidence coverage while
  existing callers retain one Rust classification identity.
- **origin:** ADR-0719 assigns the data-class registry to Compliance, but the
  current exact classification values and parsers are exposed by the agreed
  `data-classification` compatibility port.
- **rule:** Compliance MUST own registry entries that bind classification value,
  pack/schema revision, aliases, applicability, and evidence obligations. It
  MUST consume the agreed value contract and MUST NOT copy or wrap those Rust
  types. Moving the defining value package requires a separate D-29 provider
  migration with Data and every consumer; L3 work MUST NOT hide that transfer.
- **ensure:** one value crosses registry, projection, and consumer tests with
  identical type/error/parser identity; dependency review rejects a second
  enum or label parser in Compliance.
- **overturn_when:** a cross-owner contract amendment names another registry
  and value owner and migrates all Cargo/Buck consumers without dual identity.

</classification_registry>

<interfaces_and_security>

## Decision: one tenant contract, fail closed

- **achieves:** first- and third-party consumers use the same CaS behavior with
  no tenant-zero shortcut or unaudited control mutation.
- **origin:** the current tree has no facade; its DSR port trusts locally
  configured credentials and its Helm artifacts advertise private endpoints.
- **rule:** the sold CaS contract MUST have one protobuf source of truth at
  `compliance/facade/proto/compliance/cas/v1/` with package
  `compliance.cas.v1`, served through the normal Connect gateway. A standing
  gRPC/tonic contract or transport MUST NOT be introduced. Every bind,
  projection publication, and evidence export
  MUST authenticate, obtain a verified Policy decision, bind tenant and
  idempotency identity, enforce bounded admission, and persist required Audit
  evidence before acknowledgement. Preview MUST disclose no foreign-tenant
  catalog or binding state.
- **ensure:** forged/expired decisions, wrong audience, tenant mismatch,
  reused idempotency keys with changed fingerprints, Audit outage, and stale
  binding generations fail before mutation or disclosure; tenant #0 runs the
  same tests.
- **overturn_when:** an independently reviewed contract supplies equivalent
  authentication, authorization, audit, isolation, compatibility, and
  retirement properties.

</interfaces_and_security>

<migration>

## Decision: structural truth, burn, then behavior

- **achieves:** a lawful CaS seed without preserving off-charter products or
  mixing deletion, lockfile movement, and new behavior in one change.
- **origin:** every current Rust file exceeds the budget, Buck cannot parse the
  owner root, and all seven packages are outside the terminal CaS/evidence
  engine shape; the former retained retention crate itself executes purge and
  hold decisions through a prohibited Data-core edge.
- **rule:** Compliance MUST first burn all seven current packages and
  unconsumed artifacts in one exact lock-writing structural deletion. It MUST
  then land empty/scanner CaS package structure and dependencies separately
  from contract, admission, registry, engine, and facade behavior. The oracle
  stages MUST remain unrouted; production proto, persistence, owner adapters,
  restore, and generated-SLO promotion MUST each have an explicit later gate.
- **ensure:** `PLAN.md` fixes the path/build envelope and success/failure for
  each hop; no current retention type is rehomed or copied; D-41 scanners make
  later behavior unique-file additions; one lock writer and exact Cargo/Buck
  closures hold at every structural stage.
- **overturn_when:** an independently reviewed dependency graph proves another
  order is smaller while preserving behavior, lock serialization, review
  jurisdiction, and rollback.

</migration>

## Rejected destinations

- Compliance as a DLP, DSR, eDiscovery, trust-portal, breach-notification, or
  generic GRC application suite.
- A retention/hold/purge evaluator retained or renamed inside Compliance.
- A second Merkle log, retention store, policy evaluator, or pack-algebra
  runtime.
- Root `packs/` copied under Compliance or fetched on the serving Check path.
- Markdown, Helm, Terraform, or handwritten OpenSLO presented as a running CaS
  service.
- An external GRC vendor or database as the permanent product identity.
