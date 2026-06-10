---
id: ADR-0536
title: "Hyperscaler-grounded substrate decision matrix (FD-001 + cloud substrate) — sixteen normative domain decisions, each with cited hyperscaler precedent and rejected anti-patterns"
status: Proposed
planning_impact: false
deciders: founder
date: 2026-06-09
door: one-way
owner: founder
supersedes: []
superseded_by: []
depends_on: [ADR-0328, ADR-0510, ADR-0516]
amends: []
related: [ADR-0131, ADR-0132, ADR-0243, ADR-0328, ADR-0341, ADR-0348, ADR-0393, ADR-0476, ADR-0510, ADR-0513, ADR-0516, ADR-0517, ADR-0518, ADR-0519, ADR-0520, ADR-0521, ADR-0522, ADR-0523, ADR-0524, ADR-0525, ADR-0526, ADR-0527, ADR-0528, ADR-0529, ADR-0530, ADR-0531, ADR-0532, ADR-0533, ADR-0534, ADR-0535, ADR-0537]
related_specs:
  - /specs/masterplan.json
  - /specs/master-plan-sequencing.json
milestone: W0
---

# ADR-0536: Hyperscaler-grounded substrate decision matrix (FD-001 + cloud substrate)

## Status

**Proposed — 2026-06-09 (decision matrix authored for founder sign-off; door: one-way).**

Founder sign-off is the one-way door; this ADR stays Proposed until the founder rules. The matrix
locks one normative decision per researched substrate domain so that parallel lanes can start after
contract lock (founder directive 2026-06-09: contract lock first, then parallel lanes; not MVP —
hyperscaler-grade now). ADR-0328 remains the canonical sequence authority; ADR-0537 sequences the
dogfood bring-up of the substrate this matrix decides.

## Context

The sixteen-domain substrate research wave (identity, authorization, cells, console shell, control
plane, observability, delivery fabric, KMS, network/DNS, data, storage/CAS, compute, messaging,
metering/billing, gateway/SSOT, audit) concluded with strong per-domain convergence across the
surveyed hyperscalers (AWS, Google, Microsoft, Meta, plus Stripe for ledger doctrine). The founder
directives in force require proven hyperscaler patterns reimplemented in Rust with cited precedent,
the full RBAC+ABAC+PBAC spectrum, cloud-native K8s-native operation, and the whole stack owned in
Rust (kuberos kernel → cloud-os → cloud-k8s → cloud services → oyatie products; upstream
k8s/Talos remain ADR-0510 transitional implementations behind stable interfaces, per the
ADR-0520 owned-substrate doctrine).

What was missing was one ratified matrix: cross-domain couplings (KMS keying ↔ storage crypto-shred,
gateway Check/Report ↔ metering pipeline, cell router ↔ DNS static stability, console shell ↔ CLI
retirement) cannot be decided lane-by-lane without contract mismatch. The microservice layout
authority is ADR-0131 as amended by ADR-0512 (services under `{oya,cloud}/<service>/`), with the
ADR-0132 no-grouping policy; the delivery-fabric canon is ADR-0516..0535.

## Decision

Adopt the following sixteen normative domain decisions as the substrate contract for FD-001 and the
cloud substrate. Precedent is cited inline; each domain names its rejected anti-patterns.

### D-1 Identity provider (IdP)

**Decision.** Single-homed write control plane + cell-replicated offline-verify authentication data
plane: all identity writes (principal, credential, policy-binding mutations) commit in one home
partition, while every cell holds a signed, replicated, offline-verifiable snapshot of
authentication material so authn keeps serving when the write plane is down. Identity domains are
first-class, with a primordial operator domain that bootstraps the operator estate and sealed FIDO2
hardware break-glass credentials held offline (root ceremony, ADR-0537 step 0). Passkeys
(WebAuthn/FIDO2) are the v1 primary human credential per ADR-0476; passwords are not a launch
surface. Revocation is event-driven CAEP-style: a shared-signals revocation event stream invalidates
sessions ahead of TTL expiry rather than shrinking TTLs.

**Precedent.** AWS IAM (single-region write control plane, globally replicated verification data
plane); Microsoft Entra ID (partitioned write masters + stateless regional verify, continuous access
evaluation for CAEP-style revocation); OCI Identity Domains (domain model + Default/primordial
domain).

**Rejected.** Multi-master writable identity (split-brain on the root of trust); authn that dials
the control plane per request (control-plane outage becomes a fleet-wide authn outage); password-
first credential stacks; TTL-only revocation.

### D-2 Authorization

**Decision.** Cedar is the single policy language and the PDP is embedded in-process in every
service — an authorization decision never takes a network hop. A central policy store compiles,
signs, and pushes content-addressed policy bundles to every PDP; bundles are signature-verified
before load. RBAC + ABAC + PBAC ship as the full suite from FD-001 (founder directive). A structural
`forbid` tenant-isolation invariant (deny unless `principal.tenant == resource.tenant`) is authored
as non-bypassable policy. The hand-rolled oya-policy-cedar evaluator
(`oya/policy/crates/oya-policy-cedar-*`) is retired in favor of the upstream formally-verified
`cedar-policy` crate embedded behind the existing policy API.

**Precedent.** Cedar / Amazon Verified Permissions (embedded, formally verified evaluator + central
policy store); Google Zanzibar (isolation is structural, not conventional); ADR-0243 already names
Cedar the universal gate.

**Rejected.** A central PDP service on the request path (latency + availability coupling);
hand-rolled policy evaluators (a correctness burden Cedar discharges with formal verification);
RBAC-only v1; unsigned policy distribution.

### D-3 Cells

**Decision.** Cluster-per-cell cellular topology: each cell is its own K8s cluster with hard caps
declared in TPS, tenant count, and GB, and live load capped at ≤70% of the maximum verified in test.
The cell router is thin and static-stable: tenant→cell routing data is a signed, replicated snapshot
servable with the control plane down, and the router carries no business logic. Composes with the
ADR-0341 cellular promotion gates and ADR-0348 autosharding/auto-rebalance/dynamic-sharding.

**Precedent.** AWS cell-based architecture whitepaper (cell caps, thin router, tested maximums);
Azure deployment stamps.

**Rejected.** One shared mega-cluster (unbounded blast radius); a smart router that accretes
business logic (it becomes the new monolith and the new single point of failure); theoretical cell
caps never verified under test load.

### D-4 Console shell

**Decision.** One platform-owned production Leptos shell (ADR-0393) composed at build time via
buck2: every product surface compiles into the one shell artifact. No iframes; no JS module
federation. The shell is the sole token broker — product surfaces never hold raw tokens. A
design-system merge gate keeps every surface on the one design system. The console replaces ALL
operator CLIs (founder directive 2026-06-09: authority = cloud-ci gates; operations = console +
API; ADR-0537 §4 carries the retirement doctrine).

**Precedent.** AWS Management Console unified shell; Azure Portal extension-model lessons; Google
Cloud Console — including Google's retirement of iframe-composed console surfaces for performance
and UX-integrity reasons.

**Rejected.** Per-product consoles; iframe composition (Google retired it); runtime module
federation (version skew, no Rust/WASM story); per-surface token handling; any new operator CLI.

### D-5 Control plane

**Decision.** Every resource provider implements one uniform resource-provider contract (uniform
create/read/update/delete/list plus a uniform error taxonomy), exercised by a shared Rust
contract-test crate that gates CI for every provider. Long-running operations follow AIP-151: a
durable operation ledger records every mutation; mutations are idempotent via client-supplied UUID
idempotency keys. Actuation is K8s-native — declared state reconciled by operators/reconcilers,
never imperative scripts.

**Precedent.** Azure ARM resource-provider contract; Google AIP (AIP-151 long-running operations);
AWS Cloud Control API (uniform CRUDL over heterogeneous services).

**Rejected.** Per-service bespoke API shapes; synchronous long-running mutations; mutations without
a durable operation record; imperative actuation paths that bypass reconcilers.

### D-6 Observability

**Decision.** OpenSLO documents at `{oya,cloud}/<service>/slos/*.openslo.yaml` are the single
codegen source: recording rules, dashboards, and multiwindow multi-burn-rate alerts are generated,
never hand-authored, and burn-rate alerts drive automatic rollback of the offending deploy. One wide
event per unit of work (a single high-dimensional structured event) is the canonical telemetry
primitive. Cardinality caps are enforced at ingestion.

**Precedent.** Google SRE Workbook (multiwindow multi-burn-rate alerting); Google Monarch
(cardinality limits as a survival property); AWS Builders' Library (instrumentation doctrine);
wide-event practice per Meta/Honeycomb lineage.

**Rejected.** Hand-authored alert thresholds; per-service bespoke metric taxonomies; unbounded label
cardinality; logs-first debugging as the primary signal.

### D-7 Delivery fabric

**Decision.** Presubmit carries an explicit latency SLO; postsubmit failures trigger auto-bisect and
auto-revert. Merge admission starts pessimistic — every PR is tested against projected merge state
before admission (ADR-0513 cloud-ci/oya-ci Tide) — and relaxes only with measured evidence. Code
review is the last human gate: everything after approval is automated. New gates roll out
shadow → warn → enforce.

**Precedent.** Google TAP (presubmit latency budget, culprit-finding auto-bisect); Meta Landcastle
(auto-revert at scale); Amazon Apollo + pipelines (code-review-last-human-gate, fully automated
promotion).

**Rejected.** Optimistic merge first; human operational gates after review; born-enforcing gates
with no shadow data.

### D-8 KMS

**Decision.** Adopt the AWS KMS domain model: keys live in HSM-backed domains and key material never
leaves the crypto boundary. The one-way-door property is carried by the Rust type system plus a
separate crypto-enclave process — key-material types cannot cross the process boundary. Rotation is
version rotation: a new key version encrypts forward and existing ciphertext is never re-encrypted.
Data-plane static stability comes from bounded-TTL data keys: envelope encryption with cached,
TTL-bounded DEKs keeps the data plane serving while the KMS control plane is down. Per-tenant KEKs
make crypto-shred (destroy the KEK ⇒ tenant data unrecoverable) the deletion primitive. OpenBao is
the ADR-0510 transitional implementation behind the owned interface.

**Precedent.** AWS KMS (domains, version rotation without re-encryption, envelope encryption); GCP
Cloud KMS/Keystore; Azure Managed HSM.

**Rejected.** Re-encrypt-on-rotate; per-request KMS calls on the data path; shared cross-tenant
keys (no crypto-shred); committing to a software-only root of trust (HSM procurement is Open
Question OQ-5).

### D-9 Network/DNS

**Decision.** Bespoke Rust authoritative DNS on Route 53 doctrine: shuffle-shard-of-4 name-server
assignment per zone; the data plane serves signed zone snapshots when the control plane is dead; a
minimum-answer floor guarantees a best-known answer rather than SERVFAIL. L4 load balancing is a
Katran-class Rust dataplane (aya eBPF/XDP); L7 is a GFE-class Rust proxy fleet. Strict
config-compiler/dataplane split: the dataplane consumes compiled, verified config artifacts, never
raw operator intent.

**Precedent.** AWS Route 53 (shuffle sharding, static-stable serving plane); Meta Katran (eBPF/XDP
L4); Google GFE + Maglev (L7 fleet, consistent hashing); Google Andromeda (config-compiler split).

**Rejected.** BIND/CoreDNS as the authoritative serving plane; userspace-only L4 in v1; a dataplane
that parses operator intent; DNS serving coupled to control-plane liveness.

### D-10 Data

**Decision.** One owned Rust SQL interface (oya-data) is the only persistence API services may
link; a CockroachDB-class proven distributed SQL engine is the ADR-0510 transitional implementation
behind it. The W5 bespoke engine adopts the architecture all five surveyed systems converge on:
multi-Raft with leader-per-range, hybrid logical clocks behind a `ClockSource` trait (so
TrueTime-class hardware slots in without API change), and LSM storage.

**Precedent.** Google Spanner, AWS DynamoDB, CockroachDB, TiKV, Meta ZippyDB — 5/5 convergence on
ranged consensus + LSM; HLC per CockroachDB/TiKV practice.

**Rejected.** Services linking a vendor SQL client directly; bespoke-storage-first sequencing (it
blocks every lane on the hardest component, against ADR-0520); wall-clock ordering;
single-writer-Postgres-forever. W5 numeric cutover triggers are Open Question OQ-4.

### D-11 Storage/CAS

**Decision.** Four-plane object storage (control, metadata, data, background/repair), with metadata
in oya-data using Tectonic-style keyspace layering. Strong read-after-write consistency from the
first commit. Durability is 3x chain replication on write with background re-encode to LRC erasure
coding. Content addressing is BLAKE3, scoped within each tenant's KEK boundary — no cross-tenant
dedup, preserving D-8 crypto-shred and avoiding cross-tenant side channels. Object-Lock-style WORM
compliance mode ships at launch because the audit digest chain (D-16) anchors into it.

**Precedent.** AWS S3 (ShardStore; strong consistency retrofit of 2020 — fourteen years after
launch — is the cautionary tale); Google Colossus; Azure WAS (stream/partition layering); Meta
Tectonic (metadata in a keyspace-layered KV store).

**Rejected.** Eventual consistency at launch; global cross-tenant dedup; LRC-on-write; WORM as a
later bolt-on.

### D-12 Compute

**Decision.** One shared fleet with entitlements: capacity is granted as entitlements against the
shared fleet, never as per-team clusters. Workload isolation is a Cedar-enforced ladder: first-party
trusted services run hardened runc; anything tenant-influenced (tenant code, tenant-supplied
templates, model-generated code) runs in Firecracker microVMs; placement onto a ladder rung is
itself a Cedar policy decision (D-2). Nodes run a zero-SSH immutable OS — no shell, no SSH, API-only
host management (cloud-os in the owned stack, ADR-0537 §3).

**Precedent.** Google Borg (shared cells over per-team clusters); Meta Twine (entitlements); AWS
Nitro + Firecracker (microVM isolation for multi-tenant code); Bottlerocket (zero-SSH immutable node
OS). The 20–30% stranded-capacity tax of per-team clusters is the measured industry rationale.

**Rejected.** Per-team clusters; SSH-driven node operations; mutable node images; runc for
tenant-influenced code. Firecracker adopt-vs-reimplement is Open Question OQ-6.

### D-13 Messaging

**Decision.** Apache Pulsar is the validated launch-primary broker, consumed only through a thin
owned Rust client interface (ADR-0510 transitional-behind-interface). The product surface is the
queue/stream/bus trichotomy shipped as three single-concern surfaces (ADR-0132) over ONE substrate.
The delivery contract is at-least-once transport + transactional outbox at producers = effectively-
once processing; ordering is promised per-key only.

**Precedent.** AWS SQS (queue semantics, at-least-once doctrine); Google Pub/Sub (stream semantics);
Meta FOQS (disaggregated queue over shared storage); Apache Pulsar (segmented BookKeeper storage
that separates compute from storage).

**Rejected.** Kafka-first launch (partition-coupled storage, operational rebalance burden); global
"exactly-once" delivery promises; one kitchen-sink messaging API (violates ADR-0132); services
speaking the raw broker protocol.

### D-14 Metering/Billing

**Decision.** Usage is a pipeline, not a query: metering events flow dedup → rate → aggregate →
invoice; operational databases are never aggregated at query time. The dedup key is
`(tenant, resource, dimension, usage_hour)`. The internal cost/usage schema is FOCUS 1.2 from day
one. Three-clock doctrine: accrual per-second, rating hourly, invoicing monthly. The price book is
versioned and immutable — a price change is a new version, never a mutation. Line items are
append-only with restatement-then-freeze. The subledger is double-entry: every monetary movement
posts balanced debits = credits. KR VAT is native at launch.

**Precedent.** AWS Cost and Usage Report pipeline doctrine; Azure metered billing (idempotent usage
ingestion); Stripe Ledger (double-entry, append-only, immutable postings); FinOps FOCUS 1.2.

**Rejected.** Query-time billing; mutable line items; floating-point money; single-entry ledgers;
tax as an afterthought. v1 pricing dimensions are Open Question OQ-3.

### D-15 Gateway/SSOT

**Decision.** Reimplement the Smithy ARCHITECTURE in Rust: a typed API model with traits and
emitters is the single source of truth; OpenAPI is emitted, never authored. The gateway is a Cedar
PEP — every request is authorized by the embedded PDP (D-2). Rate limiting is two-stage: a local
token bucket per gateway instance plus asynchronous global budget reconciliation. Quota and metering
share ONE Check/Report substrate: Check admits, Report meters, and the same pipe feeds D-14.

**Precedent.** AWS Smithy (model + traits + emitters); Google AIP + Service Control (Check/Report);
Microsoft TypeSpec; Envoy (local + global rate-limit staging).

**Rejected.** Hand-authored OpenAPI as the source of truth; authorization logic in handlers;
synchronous global rate limiting on the hot path; separate quota and metering pipelines.

### D-16 Audit

**Decision.** One audit crate emits CloudEvents-enveloped, AuditLog-shaped payloads from tower
middleware — services cannot choose whether to emit. Asymmetric defaults: the admin/management event
stream is always-on with no kill switch, and a CI lint refuses any code path that could disable it;
data-plane events are policy-opt-in. Integrity is a signed digest chain anchored into CAS WORM
storage (D-11). Operator access to tenant data emits Access-Transparency-class events.

**Precedent.** AWS CloudTrail (management events always on; digest-file integrity chain); GCP Cloud
Audit Logs (Admin Activity stream cannot be disabled; AuditLog payload shape); Google Access
Transparency.

**Rejected.** App-level optional audit logging; deletable or mutable audit storage; symmetric
defaults; a kill-switchable admin stream. Retention posture is Open Question OQ-2.

## Drivers

- Founder directives 2026-06-09: contract lock before parallel lanes; proven hyperscaler patterns
  reimplemented in Rust with cited precedent; RBAC+ABAC+PBAC full spectrum; cloud-native K8s-native
  operation with the whole stack owned in Rust; not MVP — hyperscaler-grade now.
- The sixteen-domain research wave converged per-domain across AWS/Google/Microsoft/Meta/Stripe;
  the residual risk is cross-domain contract mismatch, which only a single matrix removes.
- ADR-0510/ADR-0520 transitional doctrine: proven substrates may serve behind stable owned
  interfaces, so no domain decision blocks another lane's start.

## Alternatives considered

- **Sixteen separate per-domain ADRs with no matrix** — rejected: the cross-domain couplings
  (KMS ↔ storage crypto-shred, gateway Check/Report ↔ billing pipeline, cells ↔ DNS static
  stability, shell ↔ CLI retirement) deadlock parallel lanes on contract mismatch.
- **Adopt managed/third-party services as the destination** — rejected: contradicts the owned-stack
  doctrine; third-party substrates are ADR-0510 transitional implementations behind stable
  interfaces only.
- **Let implementation teams decide per-lane** — rejected: ADR-0328 is the canonical sequence
  authority and the founder directive is contract lock before parallel lanes.

## Open questions (founder decision points)

- **OQ-1 Cell-boundary GTM** — whether cells surface as a sellable isolation SKU (dedicated-cell
  tier) or remain a purely internal scaling unit (D-3).
- **OQ-2 Audit retention posture** — default retention windows and tenant-configurable extensions
  for the D-16 streams.
- **OQ-3 Pricing dimensions** — which metered dimensions ship in the v1 immutable price book
  (D-14).
- **OQ-4 W5 numeric cutover triggers** — the measured thresholds that trigger the bespoke oya-data
  engine cutover (D-10; ADR-0510 trigger discipline).
- **OQ-5 HSM procurement** — which hardware root of trust backs the KMS domains (D-8).
- **OQ-6 Firecracker adopt-vs-reimplement confirmation** — adopt Firecracker as an ADR-0510
  transitional implementation versus a Rust reimplementation timeline (D-12).
- **OQ-7 Policy-Zones logging-vs-enforce for messenger/mail at KR launch** — whether
  Policy-Zones-style data-flow controls run in logging or enforcing mode for messenger/mail at the
  KR launch.

## Consequences

On founder sign-off (door: one-way) this matrix becomes the binding substrate contract: each domain
decision is the conformance bar for its service family under `{oya,cloud}/<service>/` (ADR-0131 as
amended by ADR-0512; ADR-0132 single-concern + flat), and parallel lanes start against locked
contracts. ADR-0537 sequences the circular-dependency-free bring-up of exactly these substrates and
carries the buck2 tier-dependency lint plus the all-CLI-retirement doctrine. Gate wiring per domain
is sequenced under ADR-0328 batch discipline after sign-off (born-advisory, then shadow → warn →
enforce per D-7). The hand-rolled oya-policy-cedar evaluator retirement (D-2) and the operator-CLI
retirement (D-4) are each one-way doors once ruled. The seven Open Questions above are carried to
founder; none blocks lane start because every affected decision degrades to its transitional
ADR-0510 posture until ruled.

---
*Proposed 2026-06-09 (decision matrix for founder sign-off; door:one-way). Source: the sixteen-domain
substrate research wave + founder directives 2026-06-09. Companion: ADR-0537 (dogfood bootstrap order
+ Rust-owned stack doctrine). Sequence authority: ADR-0328.*
