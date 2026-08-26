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
  Compliance MUST consume verified Audit references through its owner-local
  `evidence-source` port, obtain privileged pre-ACK evidence through
  `audit-sink`, evaluate coverage against a bound pack revision, and publish
  versioned manifests and exports. It MUST NOT seal a second Merkle log,
  fabricate missing events, or acknowledge an export as complete when required
  evidence is absent.
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
  the facade wire contract, not the signing preimage. Validity MUST consume the
  exact `cell_clock_api::Interval` obtained from an injected
  `cell_clock_api::Clock`; requests MUST NOT assert trusted time and Compliance
  MUST NOT define a point-clock or interval DTO. A pack/key window is valid
  only when the whole Cell interval is contained by its inclusive-lower,
  exclusive-upper Unix-millisecond bounds after checked conversion. A copied
  signer-revocation generation MUST NOT authorize catalog commit. Immediately
  before catalog compare-and-swap, Compliance MUST invoke commit authorization
  through `pack-auth`; the production Secrets adapter MUST serialize that
  operation with key revocation under one per-key authority. A successful
  receipt binds the verified pack, expected catalog generation, Policy and
  Audit receipts, Cell interval, and a monotonic key-use ordinal. A revocation
  ordered first returns the stable typed refusal in `SPEC.md` and permits no
  catalog mutation; a key-use authorization ordered first remains valid only
  for that exact idempotent commit and is persisted with it.
- **ensure:** a production encoder and an independently implemented test
  encoder share only the frozen input record and match exact preimage, payload
  digest, key digest, preimage digest, public key, and signature golden bytes;
  field/header bit flips plus byte, depth, count, identifier, fan-out, and queue
  limit-plus-one cases fail closed. Cargo/Buck bind the exact Cell package and
  type identities; pre-epoch/overflow conversion, reversed uncertainty, and
  intervals before/on/across either boundary fail before mutation. Deterministic
  race tests force both resolve/revoke/commit orderings and reject a stale,
  expired, mismatched, replayed, or unavailable fence. Crypto and trusted-key
  adapters require Packs, Secrets/IAM security, Cell, and architecture review.
- **overturn_when:** an accepted Packs/Security/Compliance decision replaces
  the algorithm or canonicalization while preserving deterministic identity,
  domain separation, key provenance, bounded work, and rollback refusal.

</pack_integrity>

<request_and_pagination_integrity>

## Decision: server-derived identities and durable page sessions

- **achieves:** one reproducible authority/idempotency identity per semantic
  mutation and tenant-safe pagination that survives restore without trusting
  caller bytes.
- **origin:** the catalog/registry fence names `descriptor_digest` without
  deriving it, `request_fingerprint` is only a width, and the public page token
  binds several fields only in prose. Independent implementations could hash
  different requests or accept a forged, stale, or cross-tenant cursor.
- **rule:** An immutable catalog descriptor MUST store `descriptor_digest` as
  the exact `verified_preimage_digest` of the canonical signed Pack v1 frame;
  no second descriptor encoder or caller-supplied digest is authoritative.
  Every privileged transition MUST use the server-derived, versioned,
  domain-separated request frame and exhaustive operation table frozen in
  `SPEC.md`. That v1 table MUST separately identify pack admit/revoke/
  supersede, registry prepare/activate/supersede/revoke, binding activate/
  supersede/revoke, projection publication, authenticated target
  acknowledgement, and export admission. A binding successor or revocation
  MUST use a fresh idempotency identity rather than inherit activation;
  acknowledgement MUST authenticate and authorize the target principal under
  a fresh identity and bind the verified provider receipt plus the stored
  projection-publication fingerprint. Non-request state-machine steps MUST use
  the frozen inherited-transition record and generation fence and MUST NOT
  treat an initiating fingerprint as bearer authority.
  Callers MUST NOT supply a trusted fingerprint; Policy, Audit, Secrets commit
  authorization, idempotency state, and durable outcomes MUST bind the same
  computed 32-byte value. CaS v1 pagination MUST use a 32-byte server-minted
  durable random handle, not a self-contained bearer assertion. Its
  `pagination-session` record MUST bind verified tenant/principal, list kind,
  immutable snapshot ordinal and generations, normalized filter, exclusive
  cursor, fixed page size, trusted expiry, and schema version. The session MUST
  join the catalog-store transaction/snapshot/restore root. Production handle
  minting MUST occur only through the accepted Data-backed pagination adapter
  and its reviewed CSPRNG; lookup MUST return one redacted result for forged,
  unknown, and foreign-tenant handles. There is no pagination signing key to
  rotate in v1. Replacing the durable handle with a stateless authenticated
  token requires a separately accepted D-29 Secrets key-generation, rotation,
  revocation, outage, and compatibility contract before any wire change.
- **ensure:** independent production/reference encoders share only typed input
  and match descriptor/fingerprint golden bytes for all thirteen v1 operation
  codes across field permutations, normalization, N/N+1, every byte
  corruption, changed semantic fields, and operation-code swaps. Binding
  activation/supersession/revocation and target acknowledgement additionally
  prove exact replay, another-key duplicate, stale generation, forged receipt,
  target/principal/tenant swap, publication-identity swap, and inherited-step
  actor/fingerprint/generation mismatch. Handle exact/limit-plus-one, random
  collision, corruption, tenant/principal swap, filter/page drift, expiry
  uncertainty, snapshot retention, process death, and restore tests disclose
  no cursor or foreign state and never resume a different snapshot.
- **overturn_when:** an independently reviewed identity/session design has one
  canonical server derivation, equal cross-version verification, tenant-safe
  unforgeability, bounded work, complete restore coverage, and an explicit
  migration/rotation path.

</request_and_pagination_integrity>

<trusted_clock_production>

## Decision: Cell proves production time; Compliance only consumes it

- **achieves:** expired or not-yet-valid packs cannot be admitted because an
  unsynchronized host still returns a plausible narrow interval.
- **origin:** the live `cell-clock-api` preserves the correct `Interval` type,
  but its current `NtpClock` computes `SystemTime::now() +/- 250 ms`, cannot
  report read/freshness failure, and binds NTP without observing chrony, source
  age, rollback, or measured uncertainty. Boundary tests over that value do not
  establish a production clock plant.
- **rule:** L3 MAY retain the exact Cell port/type and fakes for deterministic
  interval semantics, but no L4 process boot, listener publication, route
  activation, or tenant promotion is dispatchable until a Cell-owned D-29
  owner-law and adapter receipt freezes the production clock contract. That
  receipt MUST name the exact port/result/error identity, provider adapter path
  and Cargo/Buck closure, cell-IR NTP/PTP/GNSS selection, measured uncertainty
  and source generation, maximum bound age, rollback/regression detection,
  startup and mid-request source-loss behavior, and the interval or typed read
  result which necessarily makes Compliance refuse. `cas-app` MUST consume the
  Cell-selected adapter, continuously join its freshness/health to readiness,
  withdraw readiness and new admission before using a stale/lost source, and
  drain bounded in-flight work. The current infallible static-uncertainty NTP
  implementation MUST NOT satisfy that gate. Compliance MUST NOT create a
  private clock, plant adapter, freshness flag, or fallback to process time.
- **ensure:** exact Cargo/Buck reverse closure and named Cell/Compliance/SRE/
  security reviewers precede composition. Cold start and live campaigns remove
  NTP/chrony, stale the last measurement, regress system/source time, widen the
  bound, change source generation, and lose PTP/GNSS; each produces the accepted
  typed refusal, readiness withdrawal, no catalog/registry/binding mutation,
  and deterministic recovery only after a fresh measured receipt.
- **overturn_when:** Cell accepts another production-time contract that exposes
  measured uncertainty/freshness and fail-closed source loss with equal
  readiness, rollback, portability, and fault evidence.

</trusted_clock_production>

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
  work complete without a verified receipt through its owner-local
  `projection-target` port.
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
  aliases, applicability, and evidence obligations. Before prepare or activate,
  it MUST resolve an authoritative immutable `ADMITTED` catalog descriptor and
  bind its exact pack id, version, content and descriptor digests, schema
  revision, admitted catalog generation, and current pack-head generation into
  the registry entry plus its Policy and Audit receipts. The registry CAS MUST
  fence that pack head against concurrent revoke or supersede; a losing race
  fails closed, and a later catalog transition creates durable reconciliation
  work rather than silently leaving an active registry generation usable. It
  MUST consume the agreed value contract and MUST NOT copy or wrap those Rust
  types. Moving the defining value package requires a separate D-29 provider
  migration with Data and every consumer; L3 work MUST NOT hide that transfer.
- **ensure:** one value crosses registry, projection, and consumer tests with
  identical type/error/parser identity; deterministic prepare/activate versus
  revoke/supersede races produce the stable typed outcomes in `SPEC.md`; restore
  replays immutable registry history and catalog-to-registry reconciliation;
  dependency review rejects a second enum or label parser in Compliance.
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
  gRPC/tonic contract or transport MUST NOT be introduced. Every pack admit/
  revoke/supersede, registry prepare/activate/supersede/revoke, binding
  activate/supersede/revoke, projection publication, authenticated target
  acknowledgement, and evidence-export admission MUST authenticate, obtain a
  verified Policy decision bound to the exact transition and expected
  generation, bind tenant or pack-namespace authority plus idempotency identity,
  enforce bounded admission, and persist a transition-bound Audit receipt
  before the authoritative compare-and-swap and acknowledgement. Preview MUST
  disclose no foreign-tenant catalog or binding state. Gateway registration
  MUST remain traffic-disabled
  until durable catalog authority and restore are proven and the required
  Pack/Secrets, Policy, Audit, projection-target, export, and Cell dependencies
  have production adapters/composition plus fail-closed outage evidence.
- **ensure:** forged/expired decisions, wrong audience, tenant mismatch,
  reused idempotency keys with changed fingerprints, Audit outage, stale
  catalog/registry/binding generations, and a receipt for another transition
  fail before mutation or disclosure; tenant #0 runs the same tests. Contract
  tests enumerate every privileged catalog, registry, binding, projection,
  target-acknowledgement, and export edge plus forbidden cross-operation
  receipt/idempotency replay. Route admission checks the immutable durability/
  restore/adapter join and cannot accept an in-memory fake as production
  authority.
- **overturn_when:** an independently reviewed contract supplies equivalent
  authentication, authorization, audit, isolation, compatibility, and
  retirement properties.

</interfaces_and_security>

<durable_authority>

## Decision: every acknowledged control state shares one durable root set

- **achieves:** restore can reproduce catalog and registry authority, accepted
  export work, and every idempotent outcome without accepting a lower generation
  or forgetting work that was acknowledged before process death.
- **origin:** the first persistence contract named catalog, binding,
  projection, manifest, and idempotency state, but omitted registry history,
  catalog-to-registry reconciliation, and durable export jobs even though the
  facade can acknowledge registry transitions and export admission.
- **rule:** the `catalog-store` port MUST durably persist immutable catalog
  history/current pack heads, immutable registry history/current generation,
  bindings, projections and acknowledgements, manifests/evidence cursors,
  export admission/job/publication state, transition idempotency outcomes, and
  catalog-to-registry reconciliation work, plus live pagination-session records
  and the immutable snapshot ordinals they retain, bounded canonical transition
  frames, pending/terminal operation slots, and inherited internal-step
  identities. Snapshots, point-in-time restore, process replay, and schema
  upgrades MUST cover the same root set. An affected registry generation MUST
  remain unavailable to bind/project/export while its source-pack transition is
  unreconciled; an unrestored page session MUST never restart at another
  snapshot.
- **ensure:** death at every catalog/registry/export transaction boundary,
  corrupt or partial snapshots, queued/running export replay, supersede/revoke
  during registry activation, and N/N+1 restore prove one monotonic result with
  no lost acknowledged job, resurrected source, duplicate output publication,
  or lowered generation before route eligibility.
- **overturn_when:** an independently reviewed persistence design proves an
  equivalent atomic root and recovery join with fewer records while preserving
  RPO 0, immutable history, export replay, and fail-closed reconciliation.

</durable_authority>

<process_boundary>

## Decision: `cas-app` is a fail-closed process

- **achieves:** one runnable D-8 facade whose boot, readiness, drain, and crash
  behavior can be tested independently from Gateway routing.
- **origin:** a library-only handler crate or in-process Gateway plugin has no
  process boundary, cannot prove production composition, and contradicts the
  canonical `facade/<surface>-app` shape.
- **rule:** `compliance/facade/cas-app` MUST contain `src/main.rs` and build as
  the CaS process; `src/lib.rs` MAY retain testable handlers and composition.
  The process MUST consume declarative cell configuration, compose only the
  accepted durable store, Pack/Secrets, Policy, Audit, projection, export,
  pagination-session, Cell, and Connect adapters, and refuse readiness/listener
  publication when any mandatory dependency, restore fence, or Cell production-
  time receipt is absent/stale. It MUST NOT become an in-process Gateway plugin,
  accept CLI authority, or let process existence satisfy route activation.
- **ensure:** L3c-S admits only a compiler shell and empty process-test target;
  the separate content-only L3c-B stage installs and executes the typed
  `ProcessBootError::Uncomposed` refusal while freezing the package/build face.
  Later cold start, malformed configuration, missing adapter, failed restore,
  bind error, dependency loss, drain, cancellation, and process-death tests
  prove that an unready process cannot receive admitted traffic or acknowledge
  mutations.
- **overturn_when:** a founder-accepted D-8 amendment replaces the process with
  another failure-isolated facade and updates Gateway, Compliance, and the
  canonical tree contract in the same change.

</process_boundary>

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
  from the content-only uncomposed boot refusal, contract, admission, registry,
  engine, and facade behavior. The structural `src/main.rs` MUST remain a
  dependency-free compiler shell with no specified or tested boot outcome, and
  its execution MUST NOT be acceptance evidence. The oracle stages MUST remain
  unrouted; production proto, persistence, owner adapters, restore, and
  generated-SLO promotion MUST each have an explicit later gate.
  Every owner-local dependency port and the exact Cell edge MUST exist before
  L3 behavior uses a fake. Every new L4 package MUST inherit the D-41 scanner/
  Buck parity contract, and adding Connect codegen MUST preserve the existing
  library and test scanner outputs.
- **ensure:** `PLAN.md` fixes the path/build envelope plus success, failure,
  rollback, and stage-available fault evidence for each hop; no current
  retention type is rehomed or copied; D-41 scanners make later behavior
  unique-file additions; one lock writer and exact Cargo/Buck closures hold at
  every structural stage.
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
