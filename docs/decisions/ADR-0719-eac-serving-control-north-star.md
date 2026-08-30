---
doc_status: published
id: ADR-0719
title: "Transitional migration input: EaC serving/control authoring record"
status: Superseded
planning_impact: false
deciders: founder
date: 2026-08-21
door: two-way
owner: oyatie
supersedes: []
superseded_by: []
amends: []
amended_by: []
depends_on: []
related: []
milestone: W0
deliverables: []
---

# ADR-0719: Transitional migration input — EaC serving/control authoring record

## Status

**Accepted at authoring; frozen deletion-bound migration DATA now.** This
numbered record is superseded and non-operative migration input, not current
authority, current owner truth, or a normal agent reading surface.
Root `AGENTS.md` plus semantic native facts are current. The sequential filename and
identifier survive solely as deletion-bound provenance while every source
claim is classified; this amendment does not claim that projection, retained-
reference retirement, or deletion is complete.

For this frozen transition, only the status boundary above governs how the body
may be consumed. Every later MUST, decision, deliverable, and reference in this body is non-operative migration DATA until classified.
Numbered identifiers and this path are provenance only, never current control.

## Context

The platform was sketched as Everything-as-Code (EaC) plus Cedar and Zanzibar-pattern ReBAC,
then challenged at hundreds of millions of RPS, against AWS’s closed etcd-journal, and against
Eurozone (and KR) law. Three mistakes were available: (1) put serving state in etcd/CRDs or a
remote PDP; (2) treat JSON/REST/Helm as the product because they are familiar; (3) copy GDPR
as a global floor and copy Zanzibar’s worldwide ACL replica.

Hyperscalers split **serving** (memory, local, constant-work) from **control** (journal,
lower RPS). Zanzibar 2019: >10M client QPS, ~200M cache lookups/s, Spanner behind the
watch, not etcd on `Check`. EKS rebuilt etcd (Raft → internal journal, BoltDB → tmpfs) to
sell **one 100k-node Kubernetes**. GKE used Spanner behind the apiserver watch cache. We
sell a **cloud** sharded by **cells**, not a 100k-node logo.

## Decision

### D-1 — Serving vs control

- **Serving path** (product RPC + `Check`, 10^8-class fleet): Maglev-class anycast
  **per cell**, then the **one** L7 gateway **contract** (Connect/H3; many
  frontends), then **in-process PEP**, then **in-process PDP** on a **cell-local
  RAM snapshot** (compiled Cedar + ReBAC shards + product cache). Lookups ≫ RPCs. RPC
  `Check` is miss / recent-zookie / cross-cell only. No sidecar hop.
- **Control path** (IR apply, pack sign, tuple write, schedule, SLO, billing close):
  journaled, sharded, 10^3–10^5 class. First-party IR may live in git. Tenant IR lives on
  the gateway API.
- **Scheduler** = kube-scheduler (pods) + cell placement (tenants). Not an EaC scheduler.
- **Orchestrator** = `workflow` for business sagas. Apply order is `iac`’s object graph.
- **Monitor** = `observability` + `audit` + `iac` drift. No `eac-monitor`.
- **Audit on serving:** two classes. Privileged / admin / payroll-approve / policy-publish
  persist evidence **before ACK**. Other `Check`s may be async/sampled. Silent drop under
  load on privileged class is **platform-forbidden**. DORA/NIS2/CSAP retention
  overlays live in packs (D-6): a pack may **raise** the privileged class, never
  lower it.

**MUST (serving store)**

- **achieves:** 10^8-class `Check` and product RPC without a consensus box on the hit path.
- **origin:** etcd/Raft and remote PDP-RPC were drawn as if they were Borg/Zanzibar.
- **rule:** serving state is cell-local memory snapshots; consensus/journal is off the hit path.
- **ensure:** no PR stores tuples, IR, or tenant documents as etcd keys or product CRDs.
- **overturn_when:** a measured serving path needs a different store AND a replacement ADR
  with five fields lands same-wave.

**Time — our TrueTime, two layers, one API.** Hyperscalers who **are** the
cloud install clocks in the DC. Software-only HLC is what you do when you
**rent** someone else’s DC. We are the operator, so the **destination plant
is GNSS + holdover atomic + PTP**. We do **not** wait for that hardware to
exist before the API exists. Google TrueTime, AWS ClockBound, and Meta
fbclock all return an **interval**; the antenna only **narrows** it.

- **API (always, from v1).** `Now() → Interval { earliest, latest }` plus
  a hybrid logical counter inside the interval (Cockroach HLC shape).
  Never a product `time.Now()` point. This **is** our TrueTime. Bound width
  is **measured from the wired adapter**, not a brochure number. Do not
  publish Spanner-class ε until GNSS/atomic is live **and** a measured
  bound exists.
- **Port (`cell/ports`).** Clock source is a port. Adapters (closed):
  `ntp` (chrony, v1 default, wide ε), `ptp_phc` (NIC / hypervisor PHC —
  AWS PHC, Azure VMIC), `gnss_atomic` (OCP Time Card + GNSS antenna +
  holdover atomic; Meta 2022 / Google GPS+atomic plant). Same API on
  every adapter. Without hardware the interval is wide; with hardware it
  is tight. Callers do not branch.
- **Not a product flag.** Adapter is **cell IR at bring-up** (like which
  NTP peers), not `flags/` and not a runtime kill-switch that silently
  shrinks ε under in-flight timestamps. Swapping NTP→PTP→GNSS is a
  **control-plane cell mutation**. In-flight intervals keep the bound they
  were issued with. `flags/` kill switches do not select the clock.
- **Physical plant (`cell/` core + adapters).** Google TrueTime = GPS +
  atomic in the DC. AWS Time Sync = satellite **and atomic clocks in each
  region**, PTP PHC, ClockBound. Azure = hypervisor PTP. Meta 2020 NTP;
  **2022 PTP** GNSS antenna + atomic Time Card (OCP) + NIC PHC + ptp4u;
  still Window of Uncertainty (~100× better commit-wait vs NTP). v1 plant
  is NTP/chrony. Skipping GNSS/PTP *forever* is not Meta/AWS/Google. Not
  a `time/` cap. Sold Time Sync is a later `cell` facade.
- **Use.** `data/` **consumes** cell `Now() → Interval`. It does not own
  a second clock. Linearizability is **cell-local** via Raft/Paxos.
  **FoundationDB versionstamps** are an OLTP **commit ordinal** inside an
  engine, not a second TrueTime and not a replacement for `Now()`.
  `storage/` identity is digest / generation / CAS key. `Last-Modified` is
  metadata, not a consistency primitive — TrueTime-on-PUT would kill the
  cheap object store. Cross-cell: no global commit timestamp. Serving
  `Check` does not commit-wait. **Commit-wait is not implied by the
  interval API.** v1 OLTP does not wait NTP ε. `data/` keeps a
  `commit_wait` **adapter crate** (IR off unless a measured ε SLO says
  wait is cheaper than restart). Deleting that crate is born-blocking.

**MUST (time: our TrueTime, ported plant)**

- **achieves:** one interval API with or without atomic clocks; plant can
  mature NTP→PTP→GNSS without a second clock product.
- **origin:** first draft treated “no hardware” as identity; Meta/AWS/Google
  as operators deploy GNSS/atomic + PTP and still expose WOU/ClockBound;
  a flags switch would change ε under live timestamps.
- **rule:** TrueTime API is always an interval; clock source is a `cell/`
  port with ntp / ptp_phc / gnss_atomic adapters; v1 is ntp; GNSS/atomic
  is destination plant not a missing API; no `time/` cap; no `flags/` clock
  switch; no global commit time; `data/` consumes the interval;
  versionstamps are commit ordinals not a second clock; `storage/` does
  not use wall time as identity; Check does not commit-wait; v1 OLTP
  does not commit-wait NTP ε; no ε claim without measured plant.
- **ensure:** new clock code implements the port; no PR requires GPS for
  v1 merge; no PR forbids the GNSS adapter; no PR adds `Now() → Instant`
  as the product clock; no PR wires clock selection through `flags/`;
  no PR puts TrueTime into object identity.
- **overturn_when:** a measured GNSS/atomic plant exists AND a five-field
  ADR publishes ε (the API does not wait for that ADR).

### D-2 — k8s / etcd / “AWS journal”

`k8s/` remains the managed-cluster product (ADR-0704 as scoped by D-13). Durable
store is **cluster objects in a cell**, served from an apiserver **watch cache**.
The store is a **port** (`k8s/ports`):

| Adapter | Role |
|---|---|
| `etcd` | v1 for **sold** kube SKU only (upstream). Cell-sized. Not the fleet store. |
| `owned_journal` | Destination. Log vs memory, no single Raft leader on the hot path, MIT/Apache, cell-local. |

The **cluster-object API is never the etcd API**. Product records, tuples, and
IR are never etcd keys (D-1). Do **not** adopt Amazon’s closed EKS journal.
Steal the split, not the binary. Do **not** rewrite etcd before cells exist
(opportunity cost: cells are the scale lever). Do **not** leave a door that
blesses etcd as destination. Deleting the `owned_journal` adapter name is
born-blocking.

Admission (VAP/CEL+PSA) remains the live rule cited from ADR-0704 / ADR-0700.
Proposed 0710-range ids are not `depends_on`.

### D-3 — EaC shape

EaC is **one protobuf IR** (Rust contract → Protobuf Editions) plus **per-plane
reconcilers**. Unifier: platform ∧ pack ∧ app ∧ tenant. Helm, Tofu, CUE, Timoni, Haskell,
and Kyverno/Kubewarden are **not** sources of desired state. Helm may exist only as an
`iac` **adapter** that renders third-party charts into objects **validated against the IR**.

There is no mega `EaService` that owns policy + infra + SLOs. Plane APIs stay plane APIs.
A thin IR `validate` / `unify` / `preview` / `apply` / `watch` fans out. The SDK is
generated from the IDL. Console is a client of the same methods. Apps (`app/<product>/`)
are **IR modules** on the cloud, not private Helm worlds.

**MUST (no wrap language, no JSON product codec)**

- **achieves:** one evolution story; JSON/YAML/Helm dual stacks cannot become the product.
- **origin:** public REST+JSON and Helm-as-source were compatibility fossils and corpus.
- **rule:** product and IR contracts are protobuf; JSON is not a product codec; wrap
  languages are not EaC.
- **ensure:** new surfaces take types from the Rust/proto SSOT; gates do not require JSON
  censuses (`manifest.json`, scorecard dumps) as existence proofs.
- **overturn_when:** a pack or Data-Act export format is separately Accepted as
  **control-plane dump**, not as the serving API.

Leftover REST/JSON public shapes are **reorg_now deletion**, not a supported codec.
Temporarily breaking callers is accepted; a dual REST+proto product API is not.

### D-4 — Protocol (no pre-2016 compatibility as destination)

**One contract, two transports.** Protobuf IDL is the SSOT. The RPC is
**Connect-class HTTP** (no gRPC trailers). Same contract north-south and
east-west. `Check` on the hit path is **in-process** (D-1) — no QUIC, no
TCP, no RPC. Cross-cell Check, control apply, and watches use Connect.

**Transport follows the plant.** DC NICs, kernel, Maglev, and in-cell
path are **TCP-optimized** (TSO/GRO/RSS, DCTCP-class). Hyperscalers put
QUIC on the **public door** (GFE/Cloudflare); east-west stays TCP
(gRPC/H2-over-TCP in their fossil; we still do **Connect**, not gRPC).

| Leg | Transport | Why |
|---|---|---|
| Hit-path Check | in-process | D-1 |
| East-west (in-cell / cross-cell RPC) | **TCP** + TLS (SPIFFE). Connect on H2 (or H1 where needed). | Hardware offload. UDP/QUIC east-west is a tax. |
| North-south public door | **HTTP/3 (QUIC)** + ECH/PQC. Connect. WebTransport watches. | Internet, ECH, one public contract. H2 same Connect if the path cannot UDP. |

Forcing H3 east-west because the public door is H3 is the same class as
forcing gRPC because a mesh “automates HTTP/2.” UEC/RDMA is a later
`network/` adapter, not a reason to skip TCP v1.

**Why DCs run gRPC east-west — and what that is not.** They wanted
**binary protobuf**, HTTP/2 multiplex/streaming, deadlines, generated
stubs on **TCP**. That is not “gRPC so we can skip TLS.” Google still
encrypts in-DC (**ALTS**, cheaper handshake than public TLS, still
identity-bound). Plaintext-on-the-trusted-LAN is the old model; we do
not take it. **Connect is already binary protobuf.** gRPC vs Connect is
the **envelope** (trailers, `grpc-status`), not the codec. A standing
gRPC east-west stack is a second SDK. Same-host hit path is in-process
(no TLS). Same-host cross-process may be unix/vsock. Cross-host: TCP +
SPIFFE mTLS. If handshake CPU is the issue, that is **kTLS / ALTS-class
adapter on the TLS port**, not gRPC and not plaintext.

**gRPC leftover still deletes.** No public gRPC-Web. No new east-west
gRPC services. Overturn only with a measured EW path where the **gRPC
envelope** (not protobuf, not TCP) beats Connect **and** a five-field
ADR same-wave.

**One door, many frontends.** One gateway **codebase and proto**. Maglev-class
anycast **per cell**. Not one global VIP. Not a second REST/gRPC connector.

**No mesh product.** Istio/Linkerd/sidecar is not how we run mTLS. SPIFFE mTLS
is a **library** plus `gateway/` terminate and `network/` dataplane. A sidecar
hop on serving or product RPC is forbidden. A sold “ASM-class” SKU, if it
exists, is a `network/` facade IR module — it must not become our own Check
path. Deleting the mesh-not-identity rule is born-blocking.

**gRPC is leftover, not a mode.** Public gRPC-Web is not an operator-UI
protocol. East-west gRPC is not the product because a mesh “automates HTTP/2.”
Existing gRPC surfaces are **reorg_now deletion**, same class as leftover REST.
No standing transcode. FlatBuffers remain a measured adapter, never a second
SSOT.

Identity on the channel: SPIFFE mTLS east-west; passkeys at L2; **step-up to
L3/L4** (KR 본인인증 / eIDAS EUDI / passport+liveness per interview D58) via
Cedar `acr_required`.

**TLS port on `gateway/` (v1, not prose).** Closed adapters live as crates.
Deleting an adapter without a five-field ADR is born-blocking (same anti-forget
as GNSS on `cell/`):

| Adapter | Role |
|---|---|
| `tls13_hybrid_mlkem` | **v1 default on the public door** (X25519 + ML-KEM). |
| `tls13_x25519` | Classical-only. Dying adapter, not the destination. |
| `ech` | Encrypted Client Hello on the public door. IR `ech=on` when the cell has a cover hostname; `ech=off` does not mean the crate is absent. |

East-west SPIFFE is **TCP TLS** (same port crates; hybrid ML-KEM when the
stack speaks it). ECH is a **public-door** adapter (SNI privacy), not an
in-cell SPIFFE feature. ADR-0354 remains the crypto apex this amends.

**Enterprise visibility = Zero Trust endpoint isolation, not a QUIC
firewall.** A 2019 NGFW gets “visibility” by blocking UDP/443 or MITMing
TLS (custom CA, read SNI). QUIC + ECH are designed so an **on-path** box
cannot do that. Disabling ECH, blocking H3, or decrypting QUIC in the
middle weakens the product so a Palo Alto can keep its business model.
We do not sell that.

Inspection happens at **endpoints**. The path is opaque.

```
[Zero Trust: endpoint isolation]

  Client  -- inspect here --========================>  Server
  (device agent / ZTNA      encrypted H3 + ECH + PQC   (we terminate:
   client the user chose)    no SNI steal, no DPI       Cedar, WAF, audit)
```

Two trusted ends, not a middlebox:

- **Client end:** device posture + pre-encrypt inspect (their agent), or
  an **explicit** ZTNA hop they chose (`gateway/` `explicit_proxy` crate).
  That hop is a client they authenticated to, then a new H3/PQC/ECH leg
  to origin. It is not transparent UDP MITM.
- **Server end:** we terminate. After decrypt: Cedar, WAF, quota,
  structured events into `audit/` (tenant-exportable). This is GFE, not
  a NGFW on the wire.

What they need is **who talked to what, whether the call was allowed,
and whether data left**. That is endpoint evidence + our audit export.
It is not a copy of ciphertext on the path.

| Observer | Visibility (does not weaken H3/ECH/PQC) | Forbidden |
|---|---|---|
| Us (operator) | We **terminate**. After decrypt: Cedar, WAF, quota, structured access events into `audit/`. | On-path QUIC DPI of tenant traffic we did not terminate. |
| Tenant admin (workloads in a cell) | `network/` flow logs + SPIFFE identity; `audit/` export to their SIEM; WAF events after **our** terminate. | A `firewall/` cap. Sidecar MITM. Security groups that **block** UDP/443. |
| Employee device / corp SASE | **Explicit-proxy / ZTNA adapter** on `gateway/`: they **choose** us as the hop; both legs are real TLS/H3 (PQC+ECH still on). Or **Connect on H2** (same contract, TCP) if their office blocks UDP. Or private connectivity so the NGFW is not on path. Endpoint agents they run see pre-encrypt — their device, not our wire. | Transparent QUIC MITM. “Enterprise mode” that turns ECH off so SNI is visible to a middlebox. A second public API for “inspected” clients. |

**How the three SASE patterns map (apply, do not become a NGFW).**

1. **Endpoint / browser inspect (DLP and malware before encrypt).** Applies as
   the **client end**, not as `gateway/` core. Chrome Enterprise / Island /
   CrowdStrike-class: scan on the device, then wrap in QUIC/TLS, path stays
   E2E. We compose with that; we do not intercept after encrypt.

   **We do not ship an Island-class browser as a cloud requirement.** AWS
   does not; Google can because they already own Chrome. Device health is
   a **port**, not a Chromium fork: `iam/` `device_attestation` crate
   (closed adapters: passkey/WebAuthn, MDM/Intune, Chrome-Enterprise
   attestation, SPIFFE workload). Cedar reads those as context
   (`acr_required`). First-party web apps (when the apps discussion
   adds them) run **in the customer’s browser** as tenant #0, same APIs.
   A sold enterprise-browser `app/` exists only if that discussion
   explicitly adds it — deleting the **attestation port** is
   born-blocking; not building Chromium is not a gap.

2. **Handshake metadata and behavior (JA4/JA4S, volume, timing, dest
   reputation).** Applies at **our terminate and dataplane** as abuse/bot
   **signal**, never as the PDP and never as DLP. `gateway/` `fingerprint`
   crate (JA4-class on the ClientHello we actually terminate);
   `network/` `quic_metadata` + `flow_log`; dest reputation is **egress**
   of tenant workloads, not SNI-steal on inbound. ECH makes outer-hello
   fingerprints weaker — do not “fix” that by turning ECH off. ML-on-flow
   is `observability/` + gateway WAF, not `intelligence/detection`
   (GuardDuty stays out). Authz remains Cedar.

3. **Identity-aware proxy for inbound workloads.** Applies: this **is**
   north-south `gateway/` + `iam/` + in-process `policy/` Check. Validate
   identity, device health, context, then the backend sees an already
   authorized call. It does **not** apply as a sidecar / ephemeral proxy
   per microservice (that is Istio). East-west stays SPIFFE library +
   in-process PEP. A sold “Cloud IAP” SKU is a `gateway/` **facade** on
   the same contract, not a second door and not a mesh.

**Connect transport:** one RPC contract. H3 is the door default. **H2 is
the same Connect framing** when the path cannot carry UDP — not a second
SDK, not REST, not “disable QUIC in the product.” Our `network/` default
**allows** UDP/443.

Closed adapter crates (delete = born-blocking):

| Crate | Cap | Role |
|---|---|---|
| `waf` | `gateway/` | L7 inspect **after** decrypt. |
| `explicit_proxy` | `gateway/` | ZTNA / explicit hop. Trusted terminate, not transparent MITM. |
| `fingerprint` | `gateway/` | JA4-class handshake signal. Not Cedar. Not DLP. |
| `device_attestation` | `iam/` | Device/workload posture into Cedar. Not a browser. |
| `flow_log` | `network/` | 5-tuple + identity. No payload. |
| `quic_metadata` | `network/` | Version / connection-ID. Not payload decrypt. |

No `firewall/` capability (ADR-0132). Marketplace NGFW appliances plug
into `network/` as plugins, they do not become our door.

`Check` is never a public method. Product RPCs are. PEPs call the in-cell PDP.

**MUST (one Connect wire; TLS adapters in tree; visibility at terminate)**

- **achieves:** one generated SDK; PQC/ECH cannot vanish into chat; no sidecar
  tax; enterprise audit without turning off cryptography.
- **origin:** “gRPC+mesh vs protobuf+HTTP for middleboxes” is a tenant-on-AWS
  playbook. “Inspect QUIC like a NGFW” is the same playbook applied to H3:
  visibility by weakening. Prose-only PQC/ECH/WAF is forgotten.
- **rule:** one Connect contract (protobuf **is** the binary); public
  door **H3/QUIC**; east-west **TCP** + SPIFFE mTLS (Connect on H2);
  Check in-process; no plaintext-LAN; no standing gRPC EW (envelope ≠
  codec); TLS CPU → kTLS/ALTS-class adapter not gRPC; no QUIC-EW because
  the door is H3; no Istio; ECH public-door only; no on-path QUIC MITM.
- **ensure:** new RPCs generate Connect; new TLS/ECH/WAF/proxy/fingerprint/flow-log
  code implements those ports; layout/registry cannot drop those adapter
  names without OVERRULE; no new gRPC service crates; no PR that turns ECH
  off as “enterprise visibility”; no PR adds a sidecar IAP.
- **overturn_when:** a measured serving path needs a different RPC **and** a
  five-field ADR lands same-wave; TLS suite change is a new adapter plus IR,
  not a second door.

### D-5 — Authz placement (ADR-0615 stands, topology corrected)

`policy/` is the Cedar PDP + ReBAC tuple plane (G-face distribute, C0 in-cell snapshot).
`iam/` emits the principal. Stale snapshot denies or routes; never silent allow.

**ReBAC is cell-local.** Zanzibar’s “replicate all ACLs worldwide” is rejected: tuples and
group memberships are personal data; global replica fails residency (E18, ADR-0049, GDPR
transfers, KR CSAP). Cross-cell `Check` is an explicit Cedar’d hop.

Subject IDs in tuples, logs, and meters are **opaque handles** under the subject DEK
(interview D10). Erasure is crypto-shred + unlink. Legal retention (D11) is pack data and
blocks shred for that class only. Merkle+TSA (D86) hashes the control log; it does not
store PII and does not sync-append every `Check`.

### D-6 — Packs, not “EU = world”

- **achieves:** sell KR and EU (and later US/others) without a fake global GDPR floor.
- **origin:** “strictest common subset = EU” fails KR (CSAP, 본인인증, RRN, e-tax, K-GAAP
  retention) and fails CN localization; DORA/AI Act/eIDAS are not universal.
- **rule:** jurisdiction law is **pack overlay** on the same IR/Cedar/ReBAC/cells.
  **EU is not a country.** v1 **namespaces:** `us`, `eu`, `jp`, `kr`.
  **Packages** inside a namespace are granular (instrument/program), not
  the whole namespace. Binding is a **union** of package ids. Structural
  controls that help many regimes live in the platform; the rest lives in
  packages.
- **ensure:** no PR encodes EU-only identity or worldwide ACL replica as default; cell
  placement refuses a pack that exceeds the cell’s certification (E18); no PR
  invents `packs/kr-eu` combinatoric ids; extra jurisdiction dirs beyond
  us/eu/jp/kr are not v1.
- **overturn_when:** a single jurisdiction is the only remaining market AND a replacement
  ADR says so.

EU next-decade instruments (GDPR tightening, DORA, NIS2, AI Act, Data Act, CRA, eIDAS 2)
are **inputs to `packs/eu`**, not a reason to delete KR-shaped L3 or legal-retention
classes. Data Act switching is a **control-plane dump** of the event log and proofs, not a
JSON public API. Member states (DE, FR, …) bind **`eu/*` packages**
(at least `eu/gdpr` when GDPR attaches) until a member-state package
exists. They are not `packs/eu` as one blob.

**Granular packages, projected, then unioned.** A namespace is not one
blob. Packages are the unit (`eu/gdpr`, `eu/dora`, `kr/pipa`,
`kr/csap`, `us/hipaa`). Selecting a package is **not** blanket
application to the tenant.

Each package **projects** onto dimensions of the request. IR declares
the projection; Cedar is scoped to it. Dimensions are **not a closed
product catalog**. If a regulation attaches to it, it must be
projectable. They **map onto Cedar’s tuple** so we do not grow a second
policy language:

| Dimension (examples) | Cedar | Example package |
|---|---|---|
| **client** / subject / device | Principal | `eu/gdpr` on customer C, not every principal |
| **action** / RPC / business transaction | Action | `eu/dora` on an operational posting, not a profile `Get` |
| **resource** / record / field | Resource | `kr/pipa` on RRN; `us/hipaa` on a clinical row |
| **routing** / cell / hop / home-cell | Context | `kr/csap` on the cell; cross-cell Check; no silent replica |
| **purpose, acr, sector, time, …** | Context | `acr_required`; DORA ICT third-party; retention window |

A Check **unions** packages whose projection **matches this**
(Principal, Action, Resource, Context). Selecting `packs/eu` does not
blanket DORA onto every dimension. HIPAA does not attach to an
anonymous US download. New dimension = pack IR + Cedar attribute, not a
new cap and not a markdown folder.

**Placement:** packages that project onto **cell** or **record** still
need `required ⊆ certified_for` for those bytes. Contradiction on the
same bytes fail-closes or splits. Never “strictest common subset.”
Never `packs/kr-eu`.

**A in B, customer C from D.** KR company in JP, German customer:
profile Check unions client-projected `{eu/gdpr, …}`; a JP payroll
posting unions transaction/record `{jp/appi}` only. Same tenant, different
projections.

Empty per-instrument directories are not v1. Projection lives in Cedar+IR.

**CaC in, CaS out.** Packs are **Compliance-as-Code**: versioned Cedar+IR
packages, not markdown. They are **consumed as Compliance-as-a-Service**
on the public contract (same Connect/H3 door): bind, project, evaluate
which packages apply to this client / transaction / record / cell,
export evidence. **First-party apps and third-party apps** (marketplace
plugins) consume **that same CaS** as tenant #0 — no private pack path,
no in-process PDP only we can call. CaS is a **facade of `compliance/`
+ `policy/` Check**, not a `packs/` capability and not a second door.
`packs/` remains data the engines load.

**Where it belongs, and what it must not do.** `packs/` is **not** a
capability. It is a **versioned CaC bundle** at repo root (like
`third-party/`: data engines load, no `core/`). It stays at root so
**one** bundle feeds three engines without those engines owning each
other’s data:

| Job | Owner | Packs do **not** |
|---|---|---|
| Evaluate Check | `policy/` (compiled Cedar in-cell) | No PDP, no pack-algebra, no fetch on the hit path |
| ReBAC tuples | `policy/` | No `eu/gdpr` as a Zanzibar group |
| Principals, attestation | `iam/` | No IdP, no passkeys |
| Cell certify / place | `cell/` (`certified_for`) | No tenant CRM, no second placer |
| Evidence, CaS catalog/bind | `compliance/` facade | No Merkle log (`audit/`) |
| Tenant lifecycle | `tenancy/` | No “the tenant is pack X” as the only law |
| Marketplace plugins | `marketplace/` (signed plugins) | Jurisdiction packages ≠ `vpack_` vertical plugin ids — different word |
| i18n FTL, sticker packs | apps / copy | Not `packs/` |

Folding the bundle under `policy/` makes `cell/` and `compliance/`
depend on the PDP’s tree. Folding under `compliance/` makes Check
compile from the evidence product. **Root `packs/` is the zip SSOT.**
Today’s Markdown under `packs/` is frozen transition input to delete, not the
job. Current pack truth is Cedar + typed IR consumed by the engines.

**Realization (adversarial — do not build a second PDP).**

Hyperscalers do **not** recompute “GDPR ∪ APPI ∪ PIPA” as a custom
algebra on every RPC. They split:

| Plane | AWS / Google | Us |
|---|---|---|
| **Control** | Org policy, Assured Workloads, VPC-SC, region | Cell `certified_for` + pack IR placement. CSAP, DORA, FedRAMP-class packages live **here**. They are not a per-payment Check. |
| **Serving** | IAM / Cedar on the call | In-process `policy/` Check. Pack **Cedar fragments** are already in the cell snapshot. |
| **Evidence** | Artifact / Audit | `compliance/` CaS catalog, bind, export — **not** a DIY evaluator |

**Challenges that fail if we ignore them:**

1. **“Union” ≠ Cedar permit-OR.** Regulatory composition is **conjunction
   of forbids/obligations**. A custom unioner beside Cedar is a second
   PDP. Compile enabled fragments into the snapshot; Cedar’s forbid
   wins. Do not implement pack-algebra on the hit path.
2. **DORA/CSAP on a profile `Get` is a category error.** Package IR has
   `plane: serving | control`. Control packages do not enter every
   Check union.
3. **“Any dimension” without schema is README.** A projection dimension
   **is** a Cedar schema attribute (principal/action/resource/context)
   already on the request. New dimension = schema+IR version, not a
   free string.
4. **CaS “evaluate packs” for apps is a second Check.** First-party and
   third-party apps are **PEPs**: they call product RPCs that already
   Check. CaS is bind/list/preview/evidence. Plugins do not re-implement
   GDPR.
5. **Fetching `packs/` on Check** violates D-1. Snapshot is compiled at
   pack bind / cell certify. Request-time only supplies context.
6. **Zanzibar tuples are not pack ids.** Do not encode `eu/gdpr` as a
   ReBAC group. Packs overlay policy; tuples stay relationships.

**MUST (packs: granular union, not a country stamp)**

- **achieves:** A-in-B-with-C-from-D without GDPR-floor, N² ids, or DORA
  for every EU tenant.
- **origin:** markdown country trees; EU as a country; one pack per
  tenant; combinatoric `kr-eu`; namespace implied every instrument.
- **rule:** v1 namespaces `us`, `eu`, `jp`, `kr`; EU is Union law not a
  country; packages are granular CaC (Cedar+IR) in root `packs/` **data**;
  each **projects** via Cedar schema (P/A/R/Ctx); Check is `policy/`;
  placement is `cell/`; CaS is `compliance/` facade; `iam/` `tenancy/`
  `audit/` `marketplace/` are not pack engines; no pack-algebra, no pack
  fetch on Check; `plane: serving | control`; apps are PEPs; no
  `packs/<a>-<b>` ids; `packs/` is not a cap.
- **ensure:** no new namespace outside {us,eu,jp,kr} without a five-field
  pack ADR; no PR that blanket-applies a namespace; no PR that loads
  `packs/` on Check; no PR that adds a pack-unioner beside Cedar; no
  private pack API for `app/`; control-plane packages are not serving
  Check predicates; no PR that puts a PDP, placer, or CaS core under
  `packs/`; marketplace `vpack_` is not a jurisdiction package.
- **overturn_when:** a five-field ADR adds a namespace or member-state
  package with a loader same-wave.

### D-7 — 3-year no-regret / regret

**Keep even if implementations swap:** cells; serving RAM vs control journal; Cedar+ReBAC
in-cell; proto IDL; in-process `Check`; k8s store behind a port; apps as IR modules;
two-class evidence; shreddable principals; eIDAS/KR step-up; owned cell journal; one
gateway (one contract, N cell frontends); packs; TrueTime interval API with a
cell clock port (ntp/ptp/gnss adapters); Connect-class one wire; gateway TLS
port with hybrid-ML-KEM + ECH crates; Zero Trust endpoint isolation (not
on-path QUIC MITM); Connect contract with H3 north-south and TCP east-west.

**Regret:** etcd as product DB; AWS closed journal; JSON as product codec; Helm-as-source;
Kyverno/Kubewarden as default; one global cluster; worldwide ACL replica; silent drop of
privileged evidence; passkeys as L3; unpublished binary lock-in; EU-as-only-baseline;
cap-root census files; dual `cedar/`+`policy/` children; Istio/Linkerd as our
mTLS identity; standing gRPC east-west; PQC/ECH as ADR prose with no crates;
on-path QUIC MITM or ECH-off “enterprise mode”; a `firewall/` cap; `ci/` and
`messaging/` as a live capability name; new `cloud-*` crates or a `cloud/` root.

### D-8 — Repo root and capability / app root (amends ADR-0701)

A capability/app directory or file is allowed only when a compiler, test, PDP,
SLO controller, reconciler, Cargo, Buck, or ownership enforcement consumes it.
Those native artifacts hold current owner truth. SCM history is the historical
record and is accessed only through a separate explicit opt-in historical
lookup; a current view never traverses or mixes it. Git commit/tree identity is
the current SCM adapter. Do not invent a tracked prose destination for leftovers.

**Repo root (closed).** Workspace: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
`rustfmt.toml`, `deny.toml`, `reindeer.toml`, `.buckconfig`, `.buckroot`, `.cargo/`.
GitHub: `.github/`, `.gitignore`, `.gitattributes`. Compatibility: `README.md`,
`AGENTS.md`, `CLAUDE.md`; these are the **only** destination tracked Markdown.
Root `LICENSE` and `OWNERS` remain non-Markdown control files. Meta: `build/`,
`third-party/`. `base/` is **not**
pre-created; it appears only when the first crate admitted under the ≥3-caps-below-all
rule. `governance/` is gone (D-17). **No `kernel/` and no `os/` rungs** (D-13). Fleet is
stripped Linux on Cloud Hypervisor and/or Firecracker (`compute/`).
Composition: `app/`. One directory per capability (including BUILD `policy/`).
`packs/` = install authority (D-24). No destination `docs/`, Markdown template
tree, or catch-all `specs/`. Root compatibility entry is `AGENTS.md` /
`CLAUDE.md`; owner knowledge is discovered from semantic native surfaces (D-36).

**Not repo-root in the destination:** `contracts/`, `plan/`, `tasks/`, `scripts/`,
`specs/`, `registry/`, `evidence/`, `governance/`, `docs/`, Markdown templates,
`oya/`, `cloud/`.
**Removed this wave (not shrink-only):** `libs/`, `tools/`, `infra/`, `kernel/`,
`os/`, `contracts/`, `plan/`, `tasks/`, `scripts/`. Last leg is **gone**, not
tolerated. This ADR is retained only for the frozen transition and becomes
SCM-only history when the D-36 migration closes, reachable only through an
explicit historical lookup and never mixed into a current view; that temporary
location does not re-admit `docs/` to the destination grammar.

**Capability and `app/<product>/` — identical closed children.** Caps and apps
use the **same** four faces. Hyperscaler analog: one API (ports), one engine
(core), N backends (adapters), one serving process (facade). Google Stubby
service + impls; AWS control-plane vs data-plane as engines, not dump folders.

| Child | Belongs | Does not belong |
|---|---|---|
| `core/` | Domain + use cases. No IO. | sqlx, HTTP, S3, Helm, IPs |
| `ports/` | Traits: blob, records, mailbox, clock, pack-id, … | Adapter impls, proto dumps as SSOT |
| `adapters/` | One impl per backend: sqlite, postgres, s3, oyatie, imap, onprem, stripe | Business rules |
| `facade/` | The process / Connect surface you run | Cloud SKU implementation |
| `cedar/` | This cap/app’s Cedar only | Platform templates (those wait `policy/` cap) |
| `observability/slos/` | Generated from IR | Hand OpenSLO, dashboards JSON |
| `iac/` | IR the reconciler applies | Helm/Tofu/charts as source |
| `OWNERS`, `BUCK` | Ownership and build inputs | Markdown, prose indexes |

**This shape does not change.** Owner PRs fill these children with content.
They do **not** add faces, rename faces, insert `plan/`/`tasks/`/`crates/`/
`internal/`/`domain/`, or invent a per-team taxonomy. A new child is a D-8
five-field amendment (escalated, D-29), not an OWNERS courtesy.

**Inside each face (Cargo defaults; same for every cap and every `app/<product>/`).**
One crate per directory. Only `Cargo.toml`, `src/`, `tests/`, `OWNERS`, `BUCK`.
No nested IPs, catalog.yaml, Helm, `plan/`, `tasks/`. Do not flatten `src/`
(matklad / rust-analyzer). Do not require Clean-Architecture `domain/` or
`use_case/` — those are not Cargo or google3.

| Path | What is in it | Not in it |
|---|---|---|
| `core/<engine>/` | Lib crate: `src/lib.rs` + snake_case modules. No IO. Calls **port crates**. | sqlx, HTTP, proto, adapter types |
| `ports/<port>/` | **Agreed** traits (`owner-port`). Dir leaf = `<port>`. | Impl, proto SSOT |
| `ports/draft/<port>/` | **Unagreed** (`owner-port-draft`). Other owners must not `path =`. | Sold proto |
| `adapters/<port>-<backend>/` | One backend of an agreed port. Dir leaf = `<port>-<backend>`. | Domain rules |
| `adapters/draft/<port>-<backend>/` | Draft impl; `git mv` with the port | — |
| `facade/<surface>-app/` | Service surface. Roots at `src/main.rs` once its listener is attached, `src/lib.rs` until then (D-30 amendment 2026-08-30). Handlers call core. | sqlite, business novels |
| `facade/proto/<owner>/<api>/v1/` | Sold proto (AIP-191): dir **is** the proto package; files `snake_case.proto` | Draft names, `v1.proto` as filename |
| `cedar/` | This owner's Cedar only | Platform templates |
| `observability/slos/` | Generated from IR | Hand YAML novels |
| `iac/` | IR this engine needs in a cell | Helm/Tofu as source |

Do not add a fifth face. Do not pre-create empty module dirs.

Grammar (no `oyatie-`, no `cloud-` prefix):

```
owner     := <capability> | <product>          # kebab, registered / D-22
engine    := kebab                             # lifecycle, evaluate, journal
port      := kebab                             # blob, records, mailbox, clock
backend   := sqlite|postgres|s3|oyatie|imap|smtp|jmap|stripe|onprem|…
surface   := kebab                             # optional facade qualifier

core crate      := owner "-" engine
ports crate     := owner "-" port                 # AGREED only (D-28)
draft port crate:= owner "-" port "-draft"        # UNAGREED; path ports/draft/<port>/
adapters crate  := owner "-" port "-" backend     # agreed port
draft adapter   := owner "-" port "-" backend "-draft"
facade crate    := owner ["-" surface] "-app"
```

Examples: `storage-blob`, `storage-blob-sqlite`, `storage-blob-s3`,
`foundry-blob-draft` (local, rename freely), `foundry-ontology`,
`payroll-run-app`, `iam-pdp`. One primary `core/` engine per cap/app unless a
§7 split names a second. `app/foundry/{pages,grid}` fold into these faces on
the Foundry BUILD PR — they are not a third layout. Draft names are **illegal**
in sold proto packages. Promotion is `git mv` of `ports/draft/<port>/` onto the
provider's `ports/<port>/` after D-28/D-29 review — not a copy.

**Must not exist at cap/app root:** `manifest.json` census, `catalog.yaml`, `IPs/`,
`IP-journey-*.md`, `AUDIT-FINDINGS-*.json`, `REMEDIATION-NOTES-*.md`, `scorecards/`,
`dashboards/*.json`, `dpia/`, `decisions/` copies, `capabilities/*.yaml` essays,
stamped runbooks, stamped `tenant-scope.cedar` copies, dual ARCH files, placeholder
or explanatory Markdown. The prohibition applies recursively to every `*.md`
under a capability or `app/<product>/`, not only to these former root names.

**Public door:** proto/H3 is the product. REST/JSON leftover is **deleted**, not
transcoded as a standing codec. No new public REST shapes. Console/SDK/gates that
still speak REST may go red until they speak proto — that break is in-scope hygiene.

**MUST (closed children)**

- **achieves:** engine vs data names do not collide; N copies of platform Cedar/SLO
  or prose authority cannot reappear; current truth stays on semantic native
  surfaces.
- **origin:** naming `policy/` for both the PDP and per-cap Cedar followed the live
  tree; OpenSLO-as-authoring, owner quartets, g3doc, and REST transcode created
  parallel stacks that drifted from the artifacts consumers execute.
- **rule:** cap and `app/<product>/` share this child set; the set and inner Cargo
  layout **do not change** per owner (D-29/D-30); cap-root `cedar/` only; `policy/`
  is the capability; SLO source is IR; no `specs/` catch-all; `ports/` is the
  contract face (draft vs agreed: D-28); no capability/app Markdown; extras and
  REST/JSON product surfaces are deleted, not grandfathered. Temporarily breaking
  live callers/gates is accepted. Leaving anti-pattern debt is not.
- **ensure:** the separate Pipeline compatibility lane later matches this set,
  rejects all capability/app Markdown, and applies the three-root-Markdown
  allowlist; no both `cedar/` and `policy/` as cap children; owner PRs that add a
  new child or invented inner taxonomy fail. This ADR-only amendment does not
  claim those checks are live.
- **overturn_when:** a child is loaded by a compiler/PDP/SLO/reconciler AND a
  five-field amendment lands same-wave.

## Consequences

- During the frozen transition, migration lanes read this amendment as source
  input. Ordinary implementation consumes the owner's semantic native surfaces;
  after migration, this ADR is SCM-only provenance reachable only through an
  explicit historical lookup, never a current-truth index or current-view input.
- `policy/` extraction and IR proto are implementation follow-through, not optional sketch.
- Admission remains VAP/CEL+PSA as cited from ADR-0704 / ADR-0700. Proposed
  0710-range ids are not `depends_on`.
- Merge-blocking CI is **presubmit** only (D-10). No second protected
  `presubmit` context.
- Node OS/kernel: D-13. Do not re-create `kernel/` or `os/` as empty rungs.

### D-13 — Stripped Linux on Cloud Hypervisor / Firecracker

**Decision.** Hyperscalers do **not** run Talos or Kubernetes as the
cloud OS. Google: Linux + **Borglet**. Meta: Linux + **Twine**. AWS:
**Nitro** + **Firecracker** (Lambda/microVM) / KVM VMs — not Talos.
Talos is a **Kubernetes node** OS. GKE nodes are COS/Ubuntu, not Talos.

**Honest constraint today: Asterinas is not a working node kernel for
us.** Delete `kernel/`. Do not vendor it. Do not leave an evaluation
tree as a destination. **Reconsider** replacing Linux **only when**
Asterinas **or Hermit** (or an equally measured unikernel/kernel) is
mature **and** a five-field ADR cuts over with plant evidence. That is
not a vacant `kernel/` rung and not a pin “in case.”

**How the plant actually runs** (Talos is not in this picture):

```
cell/ places load
        │
        v
compute/ agent (Borglet analog)  ──host Linux (KVM)──
        │                              │
        ├─ Cloud Hypervisor  ──►  stripped Linux guest (VM)
        └─ Firecracker       ──►  stripped Linux guest (microVM / function)
```

| Piece | What we do | Who owns it |
|---|---|---|
| **Host** | Minimum Linux **with KVM**. Runs the agent + VMM processes. Not Talos, not kubelet. | `compute/` + `build/` (host image) |
| **VMM** | **Cloud Hypervisor** = VM class (virtio, full-ish machine). **Firecracker** = microVM/functions (jailer, dense). Both KVM. Not QEMU as identity. | `compute/` adapters |
| **Guest kernel** | **Stripped-to-minimum upstream Linux** (Firecracker-class Kconfig / tiny guest). One kernel family, two configs (host vs guest). We **consume** stable LTS; we do not fork a distro under `os/`. | `build/` produces the artifacts |
| **Guest userspace** | Bare min to run the workload + our agent/protocol (virtio-vsock). Not a distro, not Talos machined. | `compute/` + `build/` |
| **Agent** | Ours, Borglet analog: place, health, attach net/disk. Host-side talks CH/FC APIs. | `compute/` core |

**Sold kube SKU** (optional): kubelet runs **inside a guest**, as a tenant. It is not the host agent.

**gVisor and Kata — considered, not the fleet VMM.**

| Tech | What it is | Hyperscaler | Us |
|---|---|---|---|
| **gVisor** | Userspace kernel (Sentry) sandboxing **containers** | GKE gVisor, Cloud Run, App Engine — **not** Borg’s VMM | Optional **`compute/` adapter** for a container-isolation SKU (Cloud Run analog). Not the host. Not a replacement for CH/FC. |
| **Kata Containers** | VM-per-pod via CRI (QEMU/CH/FC underneath) | Kube **runtime class**, not Twine/Borg/Nitro | Allowed only as a **sold `k8s/` runtime class** (guest). Using Kata as our fleet scheduler **is** CRI-on-kube. |

Fleet plant stays **CH + Firecracker + stripped Linux**. Kata sitting on Firecracker is still kube CRI. gVisor is a **sandbox**, not a hypervisor. Empty `gvisor/` / `kata/` dirs are STAY GONE. New adapter crates under `compute/` only when that SKU is sold.

**What we build vs do not:**

| Build (Rust) | Do **not** build |
|---|---|
| **Fleet orchestration** — `cell/` placement + `compute/` agent + CH/FC reconcilers. This **is** our Borg/Twine. Own proto, own agent, own VMM APIs. | A **Kubernetes engine** (apiserver, kubelet-as-host, kube-rs as Borg, etcd as the fleet store). Google did not Borg-on-kube. |
| Sold **`k8s/`** SKU **if** customers want a kube API: wrap **upstream** kube (EKS/AKS pattern) — hosting, quota, SLA, CAPI. | A Go→Rust kubernetes.git port as a prerequisite to operate. |

Two products only if we **sell** kube. One **operations** plane either way: CH/FC + stripped Linux.

**kube-rs / k8s-openapi:** Kubernetes **client** libraries (OpenAPI types for
Pod/Job/…). They talk to a **kube-apiserver**. That is **not Borg.** Do
**not** use them as the fleet scheduler (`compute/` / `cell/`). Using
them for “our Borg” **is** running Kubernetes. Allowed only as
**`k8s/` adapters** talking to **upstream** kube for the **sold SKU**.
Pipeline must not spawn `batch/v1 Job`s as the worker plant (that is
Prow-on-kube). Workers are CH/FC guests via `compute/`.

`build/` ships: host image, guest kernel, CH/FC binaries (pinned). `storage/` holds images. `pipeline/` builds them when the graph says so. No `os/` farm. No Talos machine API.

- **Delete `kernel/`.** Asterinas is gone. No third-party pin “in case.”
- **Delete `os/`.** Talos farm is not the fleet OS.
- **`os/ports/kernel-abi` dies with `os/`.** We are not porting a
  second kernel ABI until we own one — and that is **not** Asterinas.
- **Talos** at most a **`k8s/` adapter** for a sold kube worker image —
  optional. `infra/talos/` burns or that adapter. Not operations.

**Hyperscale operation does not use Kubernetes.** Google’s fleet is **Borg**
(still; kube ~5k-node class is not Borg). Meta’s fleet is **Twine**. AWS’s
fleet is not EKS; they **sell** EKS by wrapping **upstream** kube and never
needed a Go→Rust apiserver. Our fleet scheduler is **`compute/` + `cell/`**
(Borg/Twine analog). A `build/port-engine` **kubernetes.git port is not
required to run the cloud.** It is a SKU choice: if we sell GKE-class, **EKS
pattern** (upstream apiserver + our hosting/quota/SLA) is enough. Porting
kube-apiserver does **not** give multi-tenant CP hosting, upgrades, quota,
SLA, CAPI, cell placement, or cluster billing. Google still has **GKE** after
writing Kubernetes. Collapsing “core = port, facade = managed” is the
dual-stack we killed for Talos.

`build/port-engine` stays as a **generic** generator for when we **own the
node OS** (D-13), not as a standing kubernetes-port program. Do not staff a
kube port because the tree once had one.

- **Do not add a second empty capability** for “the port.” Empty `k8s-port/` is a
  magnet. `build/port-engine` **is** the port (build meta, ADR-0704). Generated
  apiserver output is **not** checked in (same as `os/`). When we **run** an owned
  apiserver, it plugs in as **`k8s/adapters`** (alongside CAPI / upstream), not as
  a replacement for the product `core/`.
- **`k8s/` capability = managed cluster product only.** Today’s 18 crates already
  are that: cluster lifecycle, control-plane **hosting**, tenant quota, SLO, CAPI
  adapter. Registry charter “owned CP core + managed facade” is **scoped-superseded**
  by this paragraph. Nested leftover `k8s/managed-*` dirs **burn** — they duplicate
  faces.
- **`k8s/` dump burns (D-8).** IPs, AUDIT-FINDINGS, dual ARCH, dashboards, DPIA,
  `manifest.json`, scorecards, capability `PRD.md`, Helm/Tofu/kustomize **source**,
  cap-root `policy/` vs `cedar/`, extra `slos/`.
- **Not this cap:** fleet node OS (**compute/** agent on Linux), mesh/DNS
  (**network**), public door (**gateway**), cell topology (**cell**), SPIFFE
  (**secrets**). GKE-class **uses** those; it does not own the fleet.

**MUST (managed cluster ≠ k8s port)**

- **achieves:** porting Kubernetes cannot be mistaken for finishing the cloud
  product; no generated apiserver tree as `k8s/core`.
- **origin:** 0704 port-engine + 0562 coarse “k8s = owned CP + managed facade”
  mixed two jobs hyperscalers split (GKE vs kubernetes.git).
- **rule:** hyperscale **operation** is `compute/`+`cell/`, not Kubernetes;
  `k8s/` is a **sold** GKE/EKS-class SKU wrapping **upstream** kube unless
  a later ADR owns an apiserver; no kubernetes port-engine as an operations
  requirement.
- **ensure:** no in-tree generated kube API farm; no empty port capability dir;
  nested `managed-*` deleted in favor of existing faces.
- **overturn_when:** we cut over production to an owned apiserver **and** a
  five-field ADR moves implementation from adapter to `core/` same-wave.

**Destinations (so delete is not an orphan):** nothing in `infra/` **needs** `os/`.
`infra/talos/` is **upstream Talos machine state** — it moves with the `infra/`
`reorg_now` burn into **`k8s/adapters`** (cluster consume) / **`iac/`** (desired
state), not into an `os/` keep.

| In `os/` today | Outside consumers | Destination |
|---|---|---|
| Talos domain farm (`machined`, `siderolink`, `kubelet-domain`, …) | none (self-only) | **Delete.** Port-engine regenerates if we own the node. |
| `os-trustd-domain` | `iam` SVID adapters + `iam/facade/cloud-pdp-app` | **`secrets/`** (trustd = SPIFFE issuance). Rehome those three crates’ deps **in the same change as `os/` delete**. |
| `os/core/kubernetes-domain` templates (PSA/namespace emit) | `governance/psa-exception-registry.json` | **`k8s/`** (it is apiserver/namespace law, not an OS). |
| `os/ports/kernel-abi` | none worth keeping | **Delete** with `os/`. |
| `os/harness/*` | workspace glob | **Delete** with `os/` or fold into `k8s/` only if it tests k8s, not Talos-port. |
| `infra/talos/**` | local kube bring-up | **`k8s/adapters`** only if we sell that worker image; else **delete**. Not fleet OS. |
| `kernel/**` Asterinas | nested workspace only | **Delete.** No pin. Reconsider only via D-13 overturn_when. |

Census JSON (`registry/graph`, manifests) **regenerates or dies** with the dirs. It is
not a destination.

**MUST (one kernel story)**

- **achieves:** fleet is Linux+agent like Borg/Twine/Nitro; Talos/kube are not
  the cloud OS; no empty rungs.
- **origin:** kuberos deleted; Talos port farm remained as if Sidero were the
  hyperscaler node story.
- **rule:** in-tree `kernel/` and `os/` are gone; fleet is **stripped
  Linux** on **Cloud Hypervisor and/or Firecracker**; agent is
  `compute/`; Asterinas/Hermit are **not** plant today and not a git
  pin; Talos/kube are not operations; sold `k8s/` may wrap upstream kube.
- **ensure:** registry has no `kernel/` or `os/`; no Asterinas/Hermit
  evaluation tree; no PR treats Talos as the fleet OS; new VM/microVM
  code is CH/Firecracker adapters under `compute/`; guest/host kernels
  are `build/` artifacts from upstream Linux, not an `os/` capability;
  no `kube`/`k8s-openapi` in `compute/` or `cell/` core (sold `k8s/`
  adapters only).
- **overturn_when:** Asterinas **or Hermit** (or equal) is measured
  mature **and** a five-field ADR replaces Linux as the guest kernel
  same-wave, with CH/Firecracker still the VMM unless that ADR also
  changes the VMM.

**MUST (ADR override)**

- **achieves:** agents cannot treat a higher number as silent OVERRULE.
- **origin:** `HANDOFF.md` carried this; the file is deleted.
- **rule:** a newer ADR controls an earlier Accepted ADR only when it is Accepted
  and carries explicit `amends` or `supersedes` (reciprocal lifecycle edge).
- **ensure:** 0701/this ADR `amends` edges exist; number-only citations are not
  implement authority.
- **overturn_when:** a replacement lifecycle ADR with five fields lands same-wave.

### D-10 — Pipeline cadences (names, not brand)

Hyperscaler shape: Google **presubmit / postsubmit / continuous / release**; one
admission, graph-aware work; promotion is not “the test job.”

| Cadence | When | What | Blocks merge? |
|---|---|---|---|
| **presubmit** | every PR / merge-queue group | `cargo fmt --all --check`; clippy `-D warnings` (blocking once the tree is clippy-clean; advisory is debt); **`cargo nextest run --locked --workspace --no-fail-fast`** (this **is** compile proof — do not also `cargo check`); D-8 path-set; license; rust-first; generated-not-hand-edited; Cedar compile of **touched** `cedar/`. **linux/amd64 only.** | **Yes.** One required context. |
| **postsubmit** | merge to `dev` | Same nextest workspace (or remainder not proven on an affected set); start of promotion **into staging** only via the promotion pipeline | No (already merged). Failure is a **revert/block-next** signal, not a second required PR check. |
| **nightly** | schedule | arm64 nextest (D88-amend), fuzz, long E2E, soak | No |
| **weekly** | schedule | buck2 `build //...` honesty smoke (ADR-0716); hermetic graph | No |
| **promotion** | explicit rung | `dev` → `staging` → `canary` → `production`; predecessor check | N/A (branch protection on the rung) |
| **release / CD** | train (interview D63) | Bundle what’s on the promotion rung; **`cargo build --release`** (and buck2 if still local-honest) **here, not presubmit** | N/A |

**Cargo verbs (closed).** Hyperscaler TAP: cheap local, hermetic blocking presubmit, slow on a schedule, artifacts only on CD. `cargo nextest` is the only compile+test proof. `cargo check` is a local optional shortcut, never CI. `cargo test` (libtest) is not CI. `rustfmt` CLI is not CI — `cargo fmt` is. Windows/macOS per-PR smoke is not a cadence.

The “four-tier cargo blog” (fmt+clippy+check locally in <5s; nextest+doc-tests+deny+audit+semver on PR; mutants+machete+update+asan nightly; dist+bloat on CD) is the right **shape** and the wrong **tool list** for this repo.

| Tool | Local | Presubmit | Nightly / weekly | CD | Ruling |
|---|---|---|---|---|---|
| `cargo fmt` | touched `--check` | `--all --check` | — | — | **Keep.** Merge bar is `--all`. Local is not `--all` (this workspace is not <5s). |
| `cargo clippy -D warnings` | optional, touched | `--workspace --all-targets` (blocking when clean) | — | — | **Keep.** Local `--all-targets` is a lie on an 800-crate graph; it gets skipped. |
| `cargo check` | optional | **never** | **never** | **never** | **Drop from CI.** Nextest already compiles. A check job is a second compile. |
| `cargo nextest` | `-p` affected | `--workspace --locked --no-fail-fast` linux/amd64 | arm64 workspace; long / soak | — | **The proof.** Developers run affected; CI runs the workspace. |
| `cargo test` / `--doc` | **never** | **never** | **never** | **never** | **Drop.** libtest is not the runner. Doctests are `doctest = false` here; an empty `--doc` job is vacant. If rustdoc tests return, they are nightly until nextest covers them. |
| License / bans (`deny.toml`) | — | hermetic `cargo deny check licenses bans sources` **or** the owned license-policy — **one**, not both | — | — | **Keep one.** Dual deny+license-policy is bloat. Advisories are not this row. |
| Advisories | — | **vendored** snapshot (owned supply-chain-audit). **No network.** | Network `cargo audit` / refresh snapshot as a **report**, not a lockfile mutate | — | **PR is hermetic.** TAP does not fetch rustsec.org to merge. |
| `cargo-semver-checks` | — | **never** | — | — | **Drop.** Public surface is proto/H3 facades, not crates.io semver of internal crates. |
| `cargo-mutants` | — | **never** | **sampled** / per-crate, not 800 crates | — | **Not whole-workspace nightly.** That is weeks. |
| `cargo-machete` | — | **never** | weekly advisory | — | **Keep as signal.** Unused-dep cleanup is not merge-blocking. |
| `cargo update` | — | **never** | scratch report; human PR | — | **Cron must not rewrite Cargo.lock.** |
| ASan / TSan | — | **never** | subset of concurrent crates | — | **Keep as nightly subset.** Not the workspace. |
| `cargo build --release` | — | **never** | **never** | **yes** | **Keep.** |
| `cargo-dist` | — | — | — | **never** | **Drop.** CLI retirement; we ship images + promotion, not GitHub Release binaries. |
| `cargo-bloat` | — | — | optional size log | not a ship gate | **Metric, not CD.** |
| `buck2 build //...` | local hermeticity | **never** | weekly honesty | optional | Unchanged (ADR-0716). |
| Live Postgres nextest | — | dedicated jobs (service) | — | — | **Keep.** Not a laptop default. |
| Win/mac | — | **never** | — | — | **Drop.** D88-amend: amd64 per-PR, arm64 nightly. |

Local pre-push is **fmt on touched files**, not a 5-second fantasy that also runs clippy `--all-targets` and `cargo check`. A hook that takes minutes is a hook people `--no-verify`. Tests the author cares about are `cargo nextest run -p <crate>`.

No new `scripts/check.sh` / pre-push product. Rust-first: a three-line git hook may call `cargo fmt --check` on staged `*.rs`. Do not resurrect retired `./bin/oya verify` / `dev-cli`.

**MUST (nextest is the proof)**

- **achieves:** one compile+test signal; no double compile; no libtest dual; PR hermetic.
- **origin:** blog four-tier put `cargo check` and network `cargo-audit` on the merge path; `cargo-dist` assumes a CLI product; mutants-on-everything is not a nightly.
- **rule:** nextest is the only compile+test proof in presubmit/postsubmit/nightly unit lanes; `cargo check` and `cargo test` are not CI; release binary is CD; advisory fetch on PR is vendored; one license/ban engine; no cargo-dist; no crates.io semver gate.
- **ensure:** required workflow invokes nextest, not libtest; no `cargo check` job; no win/mac per-PR smoke; deny/audit are not two network tools on the PR.
- **overturn_when:** a five-field ADR names a different runner that still compiles once and stays hermetic.

Do **not** add one required GitHub check per capability (skipped-check failures, queue
combinatorics). **One** protected context: `presubmit`. `presubmit`
is not a second protected check. Lane isolation is **worktrees + non-overlapping
paths**, not 24 contexts. Do not resurrect merge-base **count** baselines as
“affected set.”

New workflow and context names: `presubmit`, `postsubmit`, `nightly`, `weekly`,
`promotion-predecessor`, `release`. No `oyatie-` prefix. Today’s `presubmit` is
the **presubmit** rename target (branch protection in the same change).

### D-11 — What the cloud is, and what each capability holds

The cloud is **not** a root `cloud/` tree and **not** a JSON catalog. It is the
**closed capability set** (registry). Repo root only **names** those directories
(plus `base/` when admitted, `build/`, `third-party/`, `app/`, and `packs/` as
already in D-8 — **not** `docs/`, `governance/`, `kernel/`, or `os/`). Everything a
tenant or operator calls is a **facade** of one capability or an
**`app/<product>/`** that wires 2+.

**Inside every capability (D-8 children only):** `core/` = engine we run;
`ports/` = traits + IDL; `adapters/` = transient AWS/OCI/etc.; `facade/` = sold
API; `cedar/` = this cap’s unique Cedar; `observability/slos/` generated from IR;
`iac/` = this cap’s IR desired state (what *this* engine needs in a cell). PEP
calls `policy/` in-process; it does not embed a second PDP.

**AWS/GCP analog → our cap** (coarse, for placement — not a product-mapping SKU list):

| Cloud-provider concern | Capability | core / facade split |
|---|---|---|
| Regions / cells / hard caps | **cell** | core: topology, router, rebalance. Not tenant CRM. |
| Accounts / orgs | **tenancy** | core: tenant lifecycle, home-cell. Enablement SKUs are IR apply, not a side console. |
| IAM users/roles/IdP | **iam** | core: principals, passkeys, SCIM, tenant-RBAC **store**, workload identity **consumption**. Does **not** evaluate Cedar. |
| IAM Access Analyzer / Zanzibar / Verified Permissions | **policy** | core: PDP + ReBAC tuple store (G + C0). Every other cap is a PEP. |
| KMS / Secrets Manager / SPIFFE | **secrets** | core: keys, secrets, **SVID issuance**. |
| CloudTrail | **audit** | core: Merkle log. Always on. Async seal on serving path (D-1). |
| CloudWatch / Monarch | **observability** | core: telemetry + SLO **controller**. Per-cap SLOs are IR → generated OpenSLO, not this cap’s YAML novel. |
| S3 / GCS / CAS | **storage** | core: **bytes** (object/CAS). Identity = digest/generation. Wall time is metadata, not TrueTime. Not SQL. |
| Spanner / Cockroach / Cloud SQL / BigQuery / Dataflow | **data** | core: **records**. Consumes cell TrueTime interval. Versionstamps = engine commit ordinal, not a second clock. **No `cloud-*` crates.** |
| EC2 / GCE / Functions | **compute** | **One** cap, **two reconcilers** (CH VM, Firecracker functions). GPU = VM SKU. gVisor = adapter. Sold kube is `k8s/`, not a third reconciler. |
| GKE / EKS control plane (sold) | **k8s** | core: managed cluster (lifecycle, CP host, quota, CAPI). Adapter: upstream or owned apiserver when we run it (D-13). Store: cluster objects only (D-2). |
| VPC / DNS / firewall / flow logs | **network** | core: dataplane, security groups (**allow** UDP/443), flow logs, QUIC metadata. Not Istio. Not a `firewall/` cap. |
| Front door / GFE / WAF / IAP | **gateway** | core: one Connect contract (H3 default, H2 same framing), Maglev **per cell**. TLS port + WAF-after-decrypt + explicit-proxy crates. Transcode is not a second API. |
| Pub/Sub / SQS | **bus** | owned queue + fan-out + seekable stream + outbox; per-key order. Kafka/Pulsar = adapters. Not `notify`. |
| Vertex / Bedrock | **intelligence** | core: inference + agent runtime + adapters. Not GuardDuty. Not a chat app. |
| Step Functions / Composer | **workflow** | core: engine. Studio is facade. Business sagas, **not** deploy orchestrator (D-1). |
| Cloud Build / TAP / CodePipeline | **pipeline** | **One** execute engine. Internal TAP = this repo as tenant #0. Sold Cloud Build = same engine, tenant graphs. GitHub is an adapter, not the product. JSON check fleets are not this cap. |
| CloudFormation / Config reconciler | **iac** | core: IR unifier + reconcilers. `<cap>/iac/` is **this** cap’s desired state; `iac/` the cap owns the **engine**. |
| Billing / Cost Explorer | **billing** | core: meter, rate, invoice, tax, FinOps. Sold-ness, not a drawer. |
| Marketplace | **marketplace** | core: signed plugins + Cedar install envelope + SKU **engine** (what exists). Rate/invoice is `billing/`. Not a `build/` price list. Not KYC/escrow. |
| Artifact / evidence packs | **compliance** | core: pack evidence, data-class registry. Consumes **audit**. Not the Merkle log. |
| SES / SNS / FCM (send) | **notify** | core: transactional email/SMS/push **send**. Not Gmail/Meet/Slack. |
| AppConfig / Feature flags | **flags** | core: flags, kill switches. Pack-gated overrides. |

**Meta (not sold as a tenant API, still in-repo):** `base/` only when admitted (≥3 caps, **non-domain** primitives; `TenantId`/`CellId` stay on their owner caps). `build/` toolchains/images/**frozen** port-engine (no destination corpus). `third-party/` vendored. `governance/` registry + off-ladder checks (D-17 default delete). No `kernel/` or `os/` rungs (D-13).

**`app/<product>/`:** composition only (hr, payroll, calendar, community, …). Wires 2+ of the table. **Does not** grow a cloud engine.

**`payments/` and `ledger/` are not this cloud set** (D-15). Do not park them in `billing/` or `oya/`. Product placement is a later discussion.

### D-14 — Each capability: is / is not / burns

Same split as `k8s/` (GKE product vs kube port). Nested leftover service dirs inside a cap **burn** (faces or `git rm`).

| Cap | **Is** (engine) | **Is not** | **Burns / move** |
|---|---|---|---|
| **cell** | Topology, physical **capacity**, which-cell router, rebalance, **clock port**. | GKE. Tenant CRM. Pack **loader**. `time/` cap. Schedulable remainder (`compute`). | Nested lifecycle/rebalancer dumps; `core/regional-pack`; `cloud-*` names. |
| **tenancy** | Tenant lifecycle, home-cell, org/account analog. Stores SKU **entitlement counts**; does **not** enforce them. | IdP (`iam`). PDP (`policy`). SKU catalog (`marketplace`). KYC/DSR/JWT. Numeric enforcer. | KYB/KYC, DSR cascade, nested PDP, JWT issuer, Citus/Helm, IP-journeys. |
| **iam** | Principals, passkeys, SCIM, role **store** (compiles to Cedar), federation **consume**, workload identity **consume**, **`device_attestation`**. Cognito-class user-token **issue** is a facade SKU if sold — not the kernel. | Cedar **eval** (`policy`). SVID **issue** (`secrets`). Zitadel-as-identity. SCIM creating tenants. Island browser. | PDP crates **move to `policy/`**. SVID issue **move to `secrets/`**. `consent-graph`, `cloud-iam/` dump, `tenant-rbac-*-evidence` farm. |
| **policy** | Cedar + ReBAC PDP, G-face + C0 snapshots. | IdP. Empty dir forever. | **Extract crates from iam now.** Cap-root `<other>/policy/*.cedar` → `<other>/cedar/`. |
| **secrets** | One cap, three facades: **KMS**, secret store, **SPIFFE/cert issue**. One crypto root. | PDP. OpenBao as identity. Nested `kms/` cap. | Nested `kms/` paperwork; OpenBao Helm as product; BYOK journey novels. |
| **audit** | Tamper-evident emit/seal/verify/query. Async on serving path. Chain TTL. CISO export. | Packs. Sync Merkle on Check. DPIA. Fifth retention store. | Journey `.cedar` novels; Helm HSM/postgres; scorecards. |
| **observability** | Telemetry + SLO **controller**. Not the billing meter. | Per-cap hand OpenSLO. SIEM as a 25th cap. Lab/diagnostics product. | Stamped OpenSLO. Nested `diagnostics/`. Helm Grafana/Loki/Mimir as identity. |
| **storage** | Durable **bytes**: object/CAS. Identity = digest/generation. S3 API + EBS-class **block** facade when sold. Pipeline CAS lives here. | SQL (`data`). Search. Drive/Meet/PACS **apps**. TrueTime as object identity. | `drive/`, `recordings/`, `imaging/` **REMOVE** (later `app/`). Census. `cloud-storage-*` OpenAPI. |
| **data** | Durable **records** engines: OLTP + OLAP + pipelines. **Consumes** cell TrueTime. Versionstamps = commit ordinal. `commit_wait` adapter (IR off on NTP ε). | S3/CAS (`storage`). **Ontology / Pages / Grid** (`app/foundry`). SERP. RAG (`intelligence` facade). BI **app**. `cloud-*`. A private `Now()`. Foundry-the-product. | Ontology kernel crates **move to `app/foundry`** when that product BUILDs (do not leave them in `data/core`). `search-*`; `data-cloud-*`. |
| **compute** | **One** cap, **two reconcilers**: CH **VM** + Firecracker **functions**. Agent is ours. GPU = VM SKU. gVisor = adapter. | GKE (`k8s/`). **`k8s-on-compute`**. Kata as Borg. Talos. QEMU as identity. One Raft. | Phrase `k8s-on-compute`. Splitting into 3 caps. |
| **k8s** | **Managed cluster product** (lifecycle, CP host, quota, SLO, CAPI). | kube-apiserver port (`build/port-engine` → adapter when we run it). Node OS. Mesh. | Dump + nested `managed-*`. |
| **network** | VPC, **private DNS**, L3/L4 dataplane, SG (allow UDP/443), `flow_log` + `quic_metadata`. Volumetric DDoS. TCP-optimized. DPU destination adapter. | Public door, public zone for the door, CDN, L7 WAF (`gateway`). Istio. `firewall/` cap. Cell picker. | Nested `dns/` dump; mesh as identity; `cloud-network-*`. |
| **gateway** | One Connect **contract**, Maglev **per cell**, TLS/ECH/WAF/IAP/fingerprint. **PEP only** (then `policy/` Check). Public DNS for the door + CDN/cache SKU. L7 bot/WAF. | Mesh. Second REST/gRPC API. Cedar **engine**. One global VIP. QUIC MITM. Connectors as the door. | Connector dump **REMOVE** then Connect door BUILD. `edge-cedar-eval`. |
| **bus** | Owned queue + fan-out bus + seekable stream + outbox; per-key order. Pub/Sub + SQS analog. | Sagas (`workflow`). Mailbox (`app/`). **Kafka/Pulsar as `core/`**. Human chat. SES (`notify`). | Crate names still `messaging-*` (KEEP+WORK rename). Kafka/Pulsar = **adapters** only. |
| **intelligence** | Vertex/Bedrock: **invoke**, endpoints, batch, quota. Hosted-agent **SKU**. GPUs **rented from `compute/`**. RAG = facade over `data/`. | Copilot UX. Chat CLI/SDK as core. GPU plant. GuardDuty. A vector store. | Claude/Codex/OpenAI-compat dumps; `authz-cedar-adapter`; CLIs. |
| **workflow** | Step Functions analog (rewrite). | Bus (`bus`). Forms/tasks/SaaS. Deploy (`pipeline`/`iac`). | **Purge current tree; rewrite.** Do not strangler. |
| **pipeline** | One execute engine (graph, queue, workers, controller). TAP / Cloud Build. | GHA as the product. **Foundry Pipeline Builder** (that UX is `app/foundry` calling `data/`). Prow/Tide as `core/`. JSON census. A root named `ci/`. | KEEP+WORK: today’s tree is not the product. Census/JSON gates REMOVE. Tide/webhook = GitHub adapter until cutover. |
| **iac** | IR unifier + reconcilers that call **cap Connect APIs** (`compute/`/`network/`/`storage/`). | Argo/Helm/Tofu **source**. kubectl as operations. Merge queue (`pipeline`). | `tofu/`, Argo app-of-apps, IP-GITOPS, `ports/rest`. |
| **billing** | **What you pay:** usage events from owning caps, rate, invoice, tax on platform SKUs, FOCUS/FinOps. Not scraped from observability. | Ledger (`ledger/`). Payments (`payments/`). SKU **catalog** (`marketplace`). | `accounting/`, `tax/` dump, `finops-portal/`, payment/subscription modules. |
| **marketplace** | **What exists:** signed plugins, Cedar install envelope, SKU attach. | List price / invoice (`billing/`). KYC/escrow/payout. App/dataset store. `build/` price view. | Escrow/deals/mediation dumps; `developer-sdk/` + `plugin-app-store/`. |
| **compliance** | **CaS** bind/project/export + evidence engine. Projects pack retention into `audit/`/`data/`/`storage/`. Does not store a fifth copy. | Merkle log. DLP core. Trust portal. eDiscovery product. Pack **data** (`packs/`). Second PDP. | `core/dlp`, `dsr`, `ediscovery`, `trust-portal`, DPIA clones. |
| **notify** | Transactional send (SES/SNS/FCM). | Email/SMS/push **send API**. | Mailbox/Meet/Messenger/contact-center (`app/` later). Current `comms/` dump **purged**. |
| **flags** | Deterministic **eval** (keep `evaluation-domain`), targeting, kill switch, pack-gated overrides. | Experiments product / p-value dashboards. OpenAPI+REST+gRPC dual. Clock adapter. OFREP as SSOT. | Cap-root dump (`catalog.yaml`, IPs, Helm, AUDIT-FINDINGS). REST/gRPC server dual. |
| **governance/** | Registry. Check crates **off ladder**; D-17 default **delete**. | Org JSON `specs/` corpus. Cloud product. | Census kernels (`no-template-stamping`, …). |
| **build/** | Toolchains, images, CH/FC+kernel **pins**. Port-engine **frozen** until a named owned corpus is Accepted. | Capability engines. Price list. Fleet agent. | `evidence/` essays. Staffing port-engine as Borg. |
| **third-party/** | Vendored pins when we need them. | Fake rungs (`kernel/`/`os/`). | Asterinas eval in `kernel/`. |
| **app/** | 2+ cap products. One shell `application`; Foundry is a **module**. V1: foundry, hr, payroll, accounting, **ledger**, payments, calendar, mail, messenger, community (shrunk). | A cloud engine. Ops console. D41 retirees. V1 treasury/FP&A/performance/learning. `ledger/` **cap**. `app/social`. | Absorbing D41; parking payments as a **cap**; `console/` pilot; empty SAP ghosts; community SecureDrop. |

**Missed before, now closed:** GKE vs kube-port (`k8s/`); Talos vs `os/`; PDP vs iam; trustd vs secrets; payments/ledger not billing; no empty `base/`/`kernel/`/`os/`/`k8s-port/`; census `ci` gates are not the delivery fabric. **D-20:** no `k8s-on-compute`; ontology not `data/core`; no gateway PDP; no `build/` price view.

### D-15 — Cloud-provider purpose and scope (not SaaS)

This set is what we **sell and run as a hyperscale cloud**. Analog: AWS/GCP/Azure **platforms**.  
**Out of every row below:** tenant SaaS products (HR, payroll, community, calendar, Slack-class UX, SAP-class accounting). Those **use** these engines via `app/<product>/`. A capability that ships a vertical product in `core/` is out of charter.

| Cap | Purpose | In scope | Out of scope |
|---|---|---|---|
| **cell** | Bound failure domains and place load. | Topology, physical capacity, which-cell router, rebalance, home-cell **admit**, clock port + adapters. TrueTime interval is the API. | GKE. Tenant CRM. Pack loader. Schedulable remainder (`compute`). Spanner ε as v1. `time/` cap. |
| **tenancy** | Tenant as the scoping primitive. | Create/suspend/delete, org/account tree, home-cell **bind**, store SKU entitlement **counts**. | IdP. PDP. Enforcing quotas. KYC. DSR execution. JWT. HR orgs. |
| **iam** | Prove **who** (and **device posture** as Cedar context). | Principals, passkeys, SCIM into an **existing** tenant, role **store**, federation **consume**, workload **consume**, `device_attestation`. User-token **issue** only as Cognito-class **facade SKU**. | Cedar **eval**. SVID **issue**. Creating tenants via SCIM. Zitadel kernel. Forking Chromium. |
| **policy** | Decide **may**. | Cedar PDP, ReBAC tuples, G-face distribute, C0 in-cell snapshot, in-process Check. | IdP. Writing every cap’s Cedar (caps own `<cap>/cedar/`). Global tuple replica. |
| **secrets** | Crypto root and issuance. | KMS, secret material, SPIFFE **issue**, cert **issue** when sold. | PDP. Embedding secrets in app products. |
| **audit** | Tamper-evident **record**. | Merkle log, seal of principal+tenant, privileged-path durability, **tenant-exportable** access events (the CISO feed). | Pack evidence (`compliance`). Sync seal on every Check. DPIA markdown. On-path packet capture as the audit product. |
| **observability** | See and SLO-gate the platform. | Metrics/logs/traces **substrate**, SLO **controller**, generated OpenSLO. | The **bill**. Hand OpenSLO. SIEM. Diagnostics/lab product. App analytics. |
| **storage** | Durable **bytes** (S3 / GCS / Colossus / CAS). | Object/CAS; S3 API; EBS-class block **when sold**. Pipeline CAS. Object DLP as SKU if sold. | Query engines. Drive/Meet/PACS apps. Search. Clock as identity. |
| **data** | Durable **records** infrastructure (sold **without** Foundry). | OLTP + OLAP + pipeline **engines**. Consumes cell `Now() → Interval`. Versionstamps = ordinal. `commit_wait` crate (IR off without measured ε). Vector search **facade SKU** if sold. | Bytes (`storage/`). Ontology / Pages / Grid / Workshop (`app/foundry`). TAP (`pipeline/`). SERP. RAG. BI app. `cloud-*`. Private `Now()`. |
| **compute** | Run **the fleet** (Borg/Twine/Nitro analog). | Two reconcilers: **CH VM** + **Firecracker functions**. Agent. GPU SKU. gVisor adapter. | GKE as fleet. **`k8s-on-compute`**. Talos. Kata as Borg. Asterinas today. QEMU as identity. GPU plant for intelligence. |
| **k8s** | **Sold** GKE/EKS/AKS-class SKU. | Cluster lifecycle, hosted CP, quota, SLA, CAPI, **upstream** kube adapter (EKS pattern). | Our Borg (`compute/`+`cell/`). A kubernetes.git port as operations. Node OS. Mesh. Public door. Empty `k8s-port/`. |
| **network** | Connect inside the cloud. | VPC, **private DNS**, TCP dataplane, SG, flow logs, volumetric DDoS, UDP/443 allowed. | Public door, public door DNS, CDN, L7 WAF (`gateway`). QUIC-EW. Istio. `firewall/`. Payload decrypt. Cell picker. |
| **gateway** | **One** north-south **contract**, many cell frontends. | Public H3/QUIC (H2 fallback). Maglev per cell. TLS/ECH/WAF/IAP. **PEP** then `policy/` Check. Public names + CDN SKU. L7 bot. | Mesh. Cedar engine. Connectors as door. REST/gRPC second API. One global VIP. QUIC MITM. Per-pod IAP. |
| **bus** | Move **events** (Pub/Sub / SQS / Service Bus). | Owned substrate: **queue** (competing consumers), **bus** (fan-out subscriptions), **stream** (seekable cursor); transactional **outbox**; at-least-once; per-key order. Serving `Check` never *is* a consume. | Sagas (`workflow`). Mailbox / chat (`app/`). **Kafka or Pulsar as `core/`**. SES send (`notify`). A root named `messaging/`. |
| **workflow** | Managed **sagas** (Step Functions / Cloud Workflows). | Rewrite: state machine, retries, timers, execution API; studio as authoring **facade**. | Bus (`bus`). Forms/tasks/SaaS. Deploy (`pipeline`/`iac`). Current tree (purged). |
| **intelligence** | Managed **inference** (Vertex / Bedrock). | Invoke, endpoints, batch, quota; hosted-agent **SKU**; adapters. GPUs from `compute/`. RAG facade over `data/`. | Copilot UX. Chat CLI/SDK core. GPU plant. GuardDuty. Vector store. Nested Cedar PDP. |
| **flags** | Dynamic config and kill switches. | Deterministic eval (`evaluation-domain`), targeting, kill switch. Pack gates via **C0 Cedar context**, not a pack fetch. Connect facade. | Experiment product. Clock adapter. REST/gRPC dual. OFREP as SSOT (adapter only). Helm. Cell topology. |
| **pipeline** | Productized execute (TAP internally, Cloud Build sold). | **One engine**, two facades: **polyglot** hermetic graph + queue (buck2 when CAS+RE live). Workers = `compute/`. Promotion graph execute. One required context `presubmit`. Tenant #0 is Rust-first; **customers are not**. | GHA as product. **Foundry Pipeline Builder** (`app/foundry` UX on `data/` engines). Cargo as sold runtime. JSON check product. Prow/Tide as core. A `ci/` root. |
| **iac** | Apply **desired state**. | IR unify/preview/apply/watch. Reconcilers call **cap APIs**, not kubectl. Helm/Tofu **adapters** only. | Merge queue. Sagas. Helm/Tofu as source. Argo as identity. |
| **billing** | Charge for **cloud use**. | Usage **events** from owning caps; rate; invoice; tax adapter; FOCUS. | Scraped metrics as the bill. Catalog of what exists (`marketplace`). Ledger. Card rails. |
| **marketplace** | Third-party **modules** on the cloud. | Signed plugins, Cedar install envelope, SKU **attach** (what exists). | Invoice (`billing`). KYC/escrow/payout. App/dataset store. `build/` price view. |
| **compliance** | Evidence **engine** + **CaS facade**. | Bind/project/export. Data-class registry. Projects pack retention into audit/data/storage. Apps are PEPs. | Merkle log. `packs/` data. DLP/eDiscovery/trust-portal products. Second PDP. Fetch-on-Check. |
| **notify** | Transactional **delivery** (SES / SNS / FCM). | Send email/SMS/push; bounce/complaint; DKIM/SPF/DMARC; optional inbound **to the bus** (SES-receive analog). | **Mailbox** (IMAP/JMAP/webmail), Meet, Messenger, calendar, contact-center — later `app/`. Emergency clinical. Current `comms/` tree (purged). |
| **packs/** (data, not a cap) | **CaC** — jurisdiction/program packages the engines load. | v1 namespaces `us`, `eu` (Union, **not a country**), `jp`, `kr`. Granular packages **projected** on any compliance dimension (Cedar Principal/Action/Resource/Context: client, action/txn, resource, routing/cell, purpose, acr, …). Check **unions** matching projections. CaS for first- and third-party apps. | A capability `core/`. Closed dimension catalog that cannot add routing/action. Blanket apply. Combinatoric ids. Markdown as CaC. Private pack path. |

**Not cloud-provider capabilities:** `payments` (money movement **product**), `ledger` (books **product**), `app/*` (SaaS), **`console/`** (D-16 — discarded pilot, not a shell engine). They must not live in `billing/` or in a cloud cap `core/`. If they ship, they are **product** placement (`app/` if 2+ cloud caps, or a later §7 **product** engine — not this cloud set).

**Dogfood.** First-party `app/<product>/` is a **tenant of this cloud** (Oyatie as tenant #0). It calls the **same** gateway, iam, policy Check, cells, storage, data, bus, workflow, billing meters, and packs as any customer. No private Helm tree, no in-process PDP that customers cannot call, no `iam/**` shortcut around `policy/`, no cap `core/` that exists only for our SaaS.

**MUST (cloud vs SaaS)**

- **achieves:** capability charters cannot absorb HR/Slack/SAP products; apps cannot grow a shadow cloud.
- **origin:** registry mixed `cloud/` + `oya/` seeds; drafts delivered IPs as if each cap were a SaaS; private paths would make dogfood fake.
- **rule:** D-15 in/out is the cloud-provider charter; apps only in `app/`; apps consume public/cloud APIs only; no cap `core/` owns a vertical product.
- **ensure:** new crates match in-scope; PRs that put payroll/studio-SaaS in `workflow/core` or add a private control plane for `app/` fail review.
- **overturn_when:** a §7 ADR explicitly adds a product engine **outside** this cloud set, with five fields.

**MUST (cloud lives in caps)**

- **achieves:** one place for each cloud concern; `cloud/` and `specs/cloud-*` cannot return.
- **origin:** `cloud/` was emptied; the cloud leaked into JSON specs and nested `oyatie-*` / `cloud-*` leftover dirs inside caps.
- **rule:** a cloud-provider engine occupies exactly one registered capability’s `core/`; sold single-cap surface is `facade/`; 2+ is `app/`; repo root does not hold IaaS dumps; **no new `cloud-*` crate, dir, or type name** (existing `CloudRegion` fossils burn with their dump, they are not a pattern).
- **ensure:** new engines get a registry row or a face, never `cloud/` or `libs/`; new crate names use the cap slug (`cell-clock-api`, not `cloud-clock`).
- **overturn_when:** a §7 split/merge ADR with five fields lands same-wave.

### D-19 — DO / DON'T × HAVE / HAVE NOT

Repo-root capabilities are the cloud provider. Every name is exactly one cell:

|  | **DO** (should exist) | **DON'T** (must never exist) |
|---|---|---|
| **HAVE** (on this branch) | **DONE** — keep. If dump/ports remain: **KEEP+WORK** | **REMOVE** — delete or rewrite **here**; do not move |
| **HAVE NOT** (absent) | **BUILD** — create in charter | **STAY GONE** — do not invent a home |

**DONE** is not “finished product.” KEEP+WORK = engine stays; nested dump is REMOVE; missing ports are BUILD. Law names the **DO** root (`bus/`, `pipeline/`), never the retired path.

No new `cloud-*` crates. REMOVE is not rehome.

**DONE** (DO + HAVE, engine matches charter)

- `cell/` engine + clock port (GNSS adapter still fail-closed — KEEP+WORK plant)
- `tenancy/` engine
- `iam/` who + `device_attestation` port
- `secrets/`, `audit/`, `observability/` engines
- `storage/` bytes; `data/` records (not `cloud-*`)
- `compute/` (one cap); `k8s/` managed-cluster **engine**; `network/` dataplane
- `bus/` kernels: queue + fan-out + stream + outbox (crate names `messaging-*` are KEEP+WORK rename, not a second slug)
- `intelligence/` inference
- `iac/` unifier; `billing/` platform meter; `marketplace/` plugin+SKU kernel; `compliance/`
- `flags/core/evaluation-domain`
- Meta: `build/`, `third-party/`, `packs/` (install authority), `app/` (composition); root compatibility Markdown is only `README.md`, `AGENTS.md`, `CLAUDE.md`

**KEEP+WORK** (DO + HAVE, additional REMOVE or BUILD inside)

- `iam/`: extract PDP → `policy/` (BUILD `policy/` same change); drop nested dumps; SVID issue → `secrets/`
- `k8s/`, `network/`: nested census REMOVE
- `data/`: ontology/Pages/Grid **product** trees REMOVE from core (OLTP/OLAP/pipeline engines stay); they BUILD under `app/foundry` — not `app/ontology`, not `data/`
- `intelligence/`: chat/CLI/SDK dumps REMOVE; invoke kernel stays
- `storage/`: `drive/` `recordings/` `imaging/` REMOVE
- `tenancy/`: KYC/DSR/JWT/nested PDP REMOVE
- `secrets/`: nested `kms/` dump REMOVE; one crypto root
- `audit/`: journey cedar novels REMOVE
- `observability/`: `diagnostics/` + Helm-as-identity REMOVE
- `billing/`: accounting/tax/portal/payment REMOVE
- `marketplace/`: escrow/deals/app-store dumps REMOVE
- `compliance/`: dlp/dsr/ediscovery/trust-portal REMOVE; BUILD CaS
- `iac/`: Argo/Tofu/Helm source REMOVE; reconcilers call cap APIs
- `flags/`: cap-root dump + REST/gRPC `server` REMOVE; eval stays
- `gateway/`: connector dump REMOVE; Connect door BUILD after; no Cedar engine
- `pipeline/`: path is `pipeline/`; contents are not the product (Prow/Tide/census). BUILD execute core; REMOVE JSON gates and Tide-as-core
- `bus/`: rename `messaging-*` crate ids; Connect facade BUILD
- `cell/`: PTP/GNSS bind when plant exists; `regional-pack` REMOVE
- `packs/`: KEEP us/eu/jp/kr; BUILD Cedar+IR loader; REMOVE markdown/OVH YAML and extra jurisdiction dirs (`au br cn in ksa mx`)
- `build/`: port-engine frozen; no price view
- `governance/check`: D-17 default delete

**BUILD** (DO + HAVE NOT)

- `policy/` — extract from `iam/` in the same change as the directory
- `gateway/` Connect/H3 door, TLS/WAF/IAP/fingerprint/ECH crates **after** connector REMOVE
- `workflow/` Step Functions — directory + `core/` in one PR
- `notify/` SES send — directory + `core/` in one PR
- `network/` `flow_log` + `quic_metadata`; `k8s/ports` `owned_journal`; `data/` `commit_wait`
- `base/` only when the ≥3-caps rule admits the first crate
- `app/foundry` — Palantir Foundry product (ontology + Pages + Grid + Workshop). No empty scaffold until that PR. Not a cap. Module of `app/application`.
- `app/application` — suite launchpad (from `oya/application`). Not console.
- `app/mail`, `app/messenger`, `app/payments`, `app/accounting`, `app/ledger` — v1 products; no empty scaffold until each PR

**REMOVE** (DON'T + HAVE)

- `gateway/` Workday/Slack/Salesforce/… connectors
- `flags/` dump (catalog.yaml, IPs, Helm, REST+gRPC server, experiment dashboards)
- Nested census; a root named `ci/` (retired; do not recreate)
- Repo-root leftovers **deleted in the destination:** `contracts/`, `plan/`, `tasks/`, `scripts/`, `libs/`, `infra/`, `tools/`, `kernel/`, `os/`, `oya/`, `evidence/`, `registry/`, `specs/`, `governance/`, `docs/`, and Markdown templates. Not shrink-only. The frozen transition retains this ADR only until D-36 closes.
- A root named `messaging/` (retired; do not recreate)
- `cloud-*` crates; cap-root IPs, AUDIT-FINDINGS, Helm source, OpenAPI product, `catalog.yaml`

**STAY GONE** (DON'T + HAVE NOT)

- `cloud/`, `console/`, `comms/`, `time/`, `firewall/`, `k8s-port/`, empty `kernel/`/`os/`/`policy/`/`workflow/`/`notify/` scaffolds
- Island-class browser as a cloud root; `payments/` and `ledger/` as **caps** (products, §7); `foundry/` as a **cap** (Palantir Foundry is `app/foundry`)
- Kafka as `bus/` core; GHA as `pipeline/` core; Istio as identity; on-path QUIC MITM
- New `cloud-*` names; EU-as-world-floor; EU as a country; combinatoric
  pack ids; REST+gRPC as a standing product
- Search/detection/GPU/CDN as **roots** (vector = `data/` facade SKU; GPU = `compute/` SKU; CDN = `gateway/` SKU; DLP object = `storage/` SKU; client DLP = endpoint)

Apps: D-22/D-23. `app/sheets` → `app/foundry/grid`. `app/global-trade` deleted.

**MUST (DONE / KEEP+WORK / BUILD / REMOVE / STAY GONE; no cloud-* debt)**

- **achieves:** placement is a 2×2; DONE is not finished; law never treats a retired slug as live.
- **origin:** calling the bus `messaging/` in the ADR recreated the alias; dumps “moved” instead of dying.
- **rule:** DO+HAVE = DONE or KEEP+WORK; DO+HAVE NOT = BUILD in charter; DON'T+HAVE = REMOVE here; DON'T+HAVE NOT = STAY GONE; capability id is `bus/` not `messaging/`; no new `cloud-*`; REMOVE is not rehome.
- **ensure:** new crates land under the DO name (`bus/`, `pipeline/`); PRs that add STAY GONE names, a `messaging/` root, or rehome REMOVE dumps fail review.
- **overturn_when:** a §7 split/merge changes DO/DON'T with five fields same-wave.

### D-20 — Charter reconciliation (founder default A, 2026-08-22)

Interview on remaining collisions. Unanswered picker; **A** is the recorded default.

**Two reconcilers, not three.** `compute/` = Cloud Hypervisor VMs + Firecracker functions. Kill **`k8s-on-compute`**. GPU is a VM SKU. gVisor is an adapter. Sold kube is `k8s/` placing nodes **as** compute VMs (kubelet in the guest). AWS does not put EKS inside EC2.

**Ontology is not `data/core`.** `data/` = OLTP + OLAP + pipelines (records engines). Palantir Foundry placement is **D-21** (`app/foundry`, not a generic later app, not `data/`). Vector search, if sold, is a `data/` **facade SKU**, not a `search/` root.

**Intelligence is Vertex, not Copilot.** Invoke + endpoints + batch + quota. Hosted-agent **SKU** allowed. GPUs rented from `compute/`. RAG is a facade over `data/`, not a store. Chat CLI/SDK dumps are not core.

**Price has two owners.** `marketplace/` = **what exists** (plugin/SKU attach). `billing/` = **what you pay** (rate, invoice, FOCUS). `build/` is **not** a price list.

**Meters are usage events.** Owning caps emit VM-hour / GB-month / invoke onto billing ingest / `bus/`. Observability may display; it is not the bill.

**IAM consumes federation.** User-token issue is a Cognito-class **`iam/` facade SKU** if sold — not the kernel. Workload SVID **issue** stays `secrets/`. SCIM does not create tenants. Role store compiles to Cedar; `iam/` never Checks. No Zitadel-as-identity.

**Storage facades are S3 and EBS-class.** `drive/`, `recordings/`, `imaging/` REMOVE (later `app/`). Pipeline CAS lives in `storage/`.

**Marketplace is plugins + SKU attach.** No escrow, KYC, payout, app/dataset store.

**Gateway is a PEP.** Authn, WAF, quota, then `policy/` Check. No Cedar engine in `gateway/` (nor in intelligence/tenancy nested PDPs).

**Quota.** Cell = physical capacity. Compute = schedulable remainder. Marketplace SKU attach stored on tenancy; **owning cap enforces** (network refuses VPC 6). Billing = money. Tenancy does not enforce.

**DNS / CDN / DDoS.** Private DNS = `network/`. Public door names + CDN/cache = `gateway/` SKU. Volumetric DDoS = `network/`; L7 WAF/bot = `gateway/`. No `cdn/` root.

**Retention cascade.** `tenancy` sets deleting → `data` erases records → `storage` erases bytes → `audit` keeps what the pack requires → `compliance` **projects** via CaS. No fifth store.

**`base/`:** domain IDs stay on the owner cap (`TenantId`, `CellId`). `base/` only for non-domain primitives if three caps share one.

**`build/` port-engine:** frozen until a named owned corpus is Accepted. Not staffed as Borg.

**`packs/`:** consumed only via `compliance` CaS + `policy` C0. `cell/` and `flags/` do not load packs.

**MUST (D-20 reconciliation)**

- **achieves:** stop over-claiming products inside cloud caps; one owner per collision.
- **origin:** adversarial pass found k8s-on-compute vs EKS wrap, ontology-in-data, copilot-in-intelligence, three price lists, gateway PDP, meter-from-metrics.
- **rule:** D-20 defaults above are live reading on conflict with earlier D-11/D-14/D-15 slogans; no `k8s-on-compute`; no ontology in `data/core`; no gateway Cedar engine; no `build/` price view.
- **ensure:** new crates and registry charters match this section; PRs that reintroduce those phrases fail review.
- **overturn_when:** a five-field ADR same-wave names a different owner for any row.

### D-21 — Foundry is `app/foundry`

**Foundry** occupies `app/foundry`: ontology (heart), Pages, Grid, Workshop,
Ontology Manager, Pipeline Builder UX. It sits on `data/` engines and
`storage/` bytes. A tenant can buy those engines without Foundry.

| Path | Occupant |
|---|---|
| `app/foundry` | Foundry product |
| `data/` | Records engines (OLTP/OLAP/pipelines) |
| `storage/` | Bytes |
| `pipeline/` | TAP / Cloud Build execute |
| `intelligence/` | Vertex / AIP on Foundry objects |

Further Foundry product truth amends semantic native artifacts under
`app/foundry/{core,ports,adapters,facade,cedar,observability/slos,iac}`;
proposals stay in the PR body or an external work system (D-36).

**MUST (Foundry occupant)**

- **achieves:** one product directory; ontology is its heart; engines stay sold without the app.
- **origin:** ontology was parked in `data/` and in a generic app; founder 2026-08-22 put it in Foundry.
- **rule:** `app/foundry` owns ontology + Pages + Grid + Workshop + Manager + Pipeline Builder UX; `data/` is records engines; TAP is `pipeline/`; AIP is `intelligence/`.
- **ensure:** new ontology crates land under `app/foundry`.
- **overturn_when:** a five-field amendment names a different Foundry home.

### D-22 — Apps: one shell; v1 People + Finance (shrunk)

Founder 2026-08-22: (1) **`app/application`** is the launchpad for the whole suite; **`app/foundry`** is a **module** in that shell (Palantir Workspace hosts Foundry — Foundry is not the only shell). D-16 stands: not `console/`, not `app/ops-console`. (2) Interview D1 Finance+People clusters stay the **shape**, but v1 **drops** FP&A, treasury, performance-management, learning-management. That is D1-A shrunk toward “B plus payments.”

**V1 product dirs** (BUILD when missing; no empty scaffolds; KEEP+WORK if HAVE):

| Dir | Role |
|---|---|
| `app/application` | **Shell / launchpad** (move from `oya/application`) |
| `app/foundry` | Foundry module **v1 full**: ontology + Pages + Grid + Workshop + Manager + Pipeline Builder **UX** — D-21/D-23 |
| `app/hr` | People |
| `app/payroll` | People |
| `app/accounting` | Finance **UI** (GL/close, statements, AR/AP). |
| `app/ledger` | Posting **engine** product (universal journal). Not a cap. Not `billing/`. |
| `app/payments` | Processor + execution (Stripe analog). **Lowest v1 priority.** Not `billing/`. Not a cap. |
| `app/calendar` | Events + availability. **Embeddable** in other modules (PTO, payroll run, …). |
| `app/mail` | **Mailbox**. Not notify. |
| `app/messenger` | **One dir**, dual-context: Slack-superset **professional** + Discord/WhatsApp-class **personal**. Meet inside. Not `app/social`. |
| `app/community` | **One dir**, dual-context: TeamBlind-class **professional** + Reddit-class **personal**. No SecureDrop v1. |
| `app/drive` | People files/folders/sharing. **Not** `storage/`. Maps to `storage/` via adapter. BUILD. |

**Not v1** (no dirs, not membership ghosts): `app/treasury`, `app/financial-planning`, `app/performance-management`, `app/learning-management`, and every registry SAP ghost (`crm`, `itsm`, `warehouse`, …). D1 can grow them later. **Do not create empty `app/<ghost>/`.**

**3A.** `app/ledger` = journal engine; `app/accounting` = accountant UI; cloud `billing/` invoices **cloud** SKUs (D37: Oyatie as tenant #0 later, then billing’s internal journal can die).

**4A.** Community KEEP+WORK **shrunk**: drop SecureDrop/whistleblower from v1.

**5A.** Personal network is the same `app/messenger` engine (deny-default dual-context). No second product dir. D32 stands (`app/social` dead).

**6A.** Drop SAP ghosts from membership. Live `oya/*` dumps that still exist stay mapped until REMOVE/move, then disappear — they are not a roadmap catalog.

**Dump map (founder 2026-08-22: refactor ≠ KEEP+WORK; REMOVE means delete).**

| Tree | Class | Action |
|---|---|---|
| `oya/application` → `app/application` | **REFACTOR** | `git mv` (done) |
| `oya/payments` → `app/payments` | **REFACTOR** | `git mv` (done) |
| `oya/docs` → `app/foundry/pages` | **REFACTOR** | `git mv` (done) |
| `app/sheets` → `app/foundry/grid` | **REFACTOR** | `git mv` (done) |
| `oya/intelligence` | **REMOVE** | deleted |
| `oya/governance` | **REMOVE** | deleted |
| `oya/global-trade`, `app/global-trade` | **REMOVE** | deleted |
| `storage/drive`, `storage/recordings`, `storage/imaging`, `storage/facade/{drive,recordings}` | **REMOVE** | deleted |
| `observability/diagnostics` | **REMOVE** | deleted |
| `iam/consent-graph` | **REMOVE** | deleted |
| `oya/` root | **REMOVE** | empty after moves; deleted |

**MUST (one shell; shrunk v1 People+Finance; ledger product; no ghosts)**

- **achieves:** one launchpad; Foundry is a module; v1 money/people set is small enough to staff; ledger is not billing and not a cap.
- **origin:** D23/D35 one shell vs D-16 no console; D1 vs founder drop of four modules; D15/D37 two ledgers; D38 personal messenger; census-like app membership lists.
- **rule:** `app/application` is the only shell; `app/foundry` is a module in it; v1 People = hr+payroll (employment + pay-run); v1 Finance = accounting + **ledger** + payments (payments last); community and messenger are each one dir dual-context; `app/drive` is an app; no empty ghost dirs; `ledger/` is not a cap.
- **ensure:** D-22 table + D-8 `app/` children are the roster (no JSON membership hub); no `console/` / `app/ops-console` / `app/social` / empty `app/crm`; Foundry PRs do not replace the launchpad.
- **overturn_when:** a five-field ADR same-wave changes the shell, v1 roster, or ledger home.

### D-23 — Apps are tenants; cloud is SKUs behind adapters

Founder 2026-08-22: dump **all** console (including application `tenant-admin-console`). Foundry v1 is the **full** suite. Identity pattern is the template: **cloud provider and tenant are separate**. First-party apps dogfood as tenant #0. They may consume cloud capabilities **only behind an adapter**. In-process `use <cloud>_core` from `app/` is vendor lock-in to ourselves.

**Proposal (adopted).** Every `app/*` is a tenant product. Every cloud cap is a sold SKU (`iam`, `storage`, `data`, `pipeline`, `notify`, `bus`, `secrets`, …). The only coupling is the **public** Connect/proto facade the cloud already sells. Cloud crates do not import `app/`. App crates do not import cloud `core/` / `ports/` as libraries. `app/*/adapters/` talk to cloud facades the way an external tenant would. Substituting an external S3/Stripe/IdP is the same port with a different adapter.

**Foundry (v1 all in).** Ontology, Pages, Grid, Workshop, Ontology Manager, Pipeline Builder **UX** do the product work **inside** `app/foundry`. They **settle** to cloud SKUs:

| Foundry work | Settles through adapter to |
|---|---|
| Object instances, datasets, OLAP, lineage facts | `data/` records engines |
| Files, attachments, workbook bytes | **blob port** → `storage/` or on-prem adapter (D-24) |
| Pipeline Builder *run this graph* | **`data/` dataset jobs** — not `pipeline/` TAP (D-24) |
| Who | `iam/` (federation; Foundry users are not cloud principals) |

Ontology kernels today under `data/core/` **move** in the Foundry BUILD PR and then call `data/` through the adapter — they do not stay as a cloud ontology engine and they do not become a `foundry/` cap.

**Console.** `console/` and tenant-admin-console dumps are **REMOVE**. Tenant policy/roles live in each product + cloud IAM, not an ops-dashboard cap and not a shell “control plane” product.

**Calendar.** `app/calendar` is the suite scheduling engine (events, availability, recurrence). Other modules **embed** it (HR PTO, payroll run calendar, Foundry due dates). They compose `app/calendar` as an app port — not `cell/` TrueTime, not a cloud cap. Meet stays in messenger.

**Mail vs `notify/`.** `app/mail` = **mailbox**. Cloud `notify/` = **notification delivery** (email send, SMS, push, messenger ping). Notify may use mail as a *channel* without being a mailbox. Apps raise notifications through the notify adapter.

**Messenger.** One dir. Professional = Slack-superset. Personal = Discord/WhatsApp-class. Deny-default dual-context (D38). Meet inside.

**Drive.** Cloud `storage/` = buckets/bytes (S3 analog). `app/drive` = people Drive (folders, sharing). Drive **maps** to storage through an adapter. Not a storage dump (`storage/drive` stays deleted). Not Foundry Pages (those are ontology documents).

**Payments (lowest v1).** `app/payments` is the processor + merchant product (Stripe analog) **and** a port for an *external* processor adapter. `billing/` remains cloud SKU invoices. Later: banking-API sync so payroll can payout through payments — still an **app** adapter, not `billing/core`. Do not staff payments ahead of ledger/hr/foundry.

**Community.** One dir. Professional = TeamBlind-class. Personal = Reddit-class. Dual-context. No SecureDrop v1.

**Packs.** OVERRULED by D-24: not one `packs/kr` novel. Overlay **content** per owner; `packs/` is thin **install** authority (pack-id).

**MUST (tenant apps; adapters; one pack engine)**

- **achieves:** first-party apps are customers of the cloud; Foundry is not a backdoor into `data/core`; packs are one CaC plane.
- **origin:** cloud/app fusion (ontology-in-data, Drive-in-storage, console-as-cap, payroll-law-in-iam).
- **rule:** `app/` ↔ cloud only via sold facades/adapters; no in-process cloud `core` from apps; Foundry v1 is the full module set and persists via data/blob/iam adapters; Foundry Pipeline Builder is **not** TAP; console dumps gone; calendar embeddable; mailbox ≠ notify; messenger and community each one dual-context dir; `app/drive` over the **blob port**; payments last in v1; packs = thin install authority + per-owner overlay content (D-24).
- **ensure:** new `app/` crates that `path =` a cloud `core/` or `ports/` fail review; tenant-admin-console files stay deleted; pack PRs do not add a second pack reconciler or a git JSON overlay SSOT.
- **overturn_when:** a five-field ADR same-wave allows in-process cloud cores for tenant #0 or a second pack engine.

### D-24 — Packs split; SQLite then cloud; TAP ≠ Foundry pipelines; mailbox + blob ports

Founder challenge 2026-08-22. D-23 stood except where this OVERRULES it.

#### Packs: per-capability content, central **install** authority

A single `packs/kr` novel with `cloud.*` and `app.*` slices **recouples** every cap PR (the JSON product we just burned). Per-cap overlays with **no** shared tenant→jurisdiction fact **split-brain** (storage US, payroll KR).

**Decouple.** Overlay **content** lives next to the owner that evaluates it: `storage/` KR residency Cedar, `app/payroll` KR filings, `iam/` proofing. Independent PRs. One overlay **format** (Cedar + typed proto config). Not N DSLs.

**Connect.** Thin cloud cap `packs/` is **install authority only**: tenant T (cell) has pack-id `kr@v3`. Runtime object, not a git census. Caps and apps ask `packs/` (adapter) “what pack is T on?” then load **their** overlay for that id. Default: one pack-id per tenant/cell (split-brain is explicit, not accidental).

IaaS-only tenants install a pack-id whose owners have only `cloud.*` overlays. App data-at-rest still needs the storage owner’s overlay (payroll cannot place bytes).

**Not:** resurrect `governance/capability-registry.json`. **Not:** `app/payroll/packs/` as a second reconciler.

#### Runtime state is not git plaintext

Native source artifacts such as proto, Cedar, Rust, Cargo, and Buck stay tracked.
Frozen ADR/README prose is D-36 migration input, not destination authority.
Instance data (objects, mail, drive files, pack **installs**, ledger rows) does
**not** stay in SCM. v1 adapters: **SQLite**. Destination adapters: `data/`
(records), blob port (bytes), on-prem. Ports exist on day 1 so SQLite is not a
data model. SQLite is not the D-1 serving path (10^8 checks stay RAM snapshots).

#### `pipeline/` is CI/CD only

OVERRULE D-23 “Pipeline Builder → `pipeline/` TAP.” Cloud `pipeline/` = TAP / Cloud Build / automation **of software**. Foundry Pipeline Builder = **dataset transforms** → `data/` job engine. Two English “pipelines”; **one slug** (`pipeline/`) is CI. Foundry does not embed TAP. TAP is not ontology-aware.

#### Mailbox port (not a casual `mail/` root)

`app/mail` is the client (Gmail analog). Store + ingress/egress is a **Mailbox port**: adapters IMAP, JMAP, SMTP, and Connect/protobuf on **one** store. v1 = SQLite mailbox (self-contained). Destination = sell hosting as facade of `data/`+blob+`network/` (MX/spam is the only reason to add a cloud `mail/` engine later). `notify/` stays delivery, not IMAP.

#### One blob port (Drive, Foundry, mail attachments)

On-prem / off-cloud / customer MinIO is **why** the port exists. Drive, Foundry bytes, and mail attachments share **one** blob port. Placement is pack-id + tenant config, not a per-app side door to `storage/core`. Our `storage/` is one adapter.

#### Challenges that did not overturn the rest of D-23

- **Foundry “all in v1”** is a company. Charter stays full-suite; **success** is ports + ontology + Pages/Grid actually persisting through SQLite adapters — not feature-parity Palantir on day one. Workshop/Manager/Pipeline Builder UX may be thin, not absent from the charter.
- **Calendar embed** must not become `hr` importing `calendar` core. Same adapter rule **between apps** (Connect), or we built a second cloud.
- **Dual-context** one dir is a lie if professional/personal are two codepaths. Shared engine or two binaries is an implementation choice; one **dir** and one dual-context policy stay.
- **Notify** is delivery. It does not own messenger threads. A “messenger ping” is notify → messenger adapter.
- **Payments last** vs payroll payout: v1 payroll may mark paid without a processor. Do not fake Stripe.
- **SQLite everywhere** is a trap if Check/ReBAC tuples only live there (D-1).

**MUST**

- **achieves:** caps/apps ship overlays without a god pack file; tenant jurisdiction is one fact; apps persist without git; CI ≠ Foundry data jobs; mail/drive can leave our storage.
- **origin:** D-23 one-file packs; Pipeline Builder mis-settled to TAP; plaintext/JSON as databases; Drive assumed our S3.
- **rule:** `packs/` = install authority (pack-id); overlay content per owner; runtime state via ports (SQLite v1 → data/blob/on-prem); `pipeline/` is CI/CD; Foundry Pipeline Builder → `data/`; Mailbox port (IMAP/JMAP/SMTP/Connect); one blob port for Drive/Foundry/mail; on-prem is an adapter.
- **ensure:** D-23 settle table matches this; no git JSON pack SSOT; no `path =` storage core from Drive/Foundry.
- **overturn_when:** a five-field ADR same-wave restores a monolithic pack file, TAP-as-Foundry-pipelines, or git as the object store.

### D-25 — Clean architecture: app core is portable; cloud is an adapter

Founder 2026-08-22 (clarified): maximize **business logic** behind ports. Cloud and apps are separate. **“Instantly portable” is not HA during our downtime.** It is **end of service** of a cloud capability (we retire the SKU, the tenant leaves, or they never bought it): the **app must keep serving** by pointing the same ports at **existing technologies** (Postgres, S3/MinIO, IMAP, Stripe, on-prem) **without a significant rewrite**.

**Shape (ADR-0562 faces, app edition).**

| Face | Holds | Must not hold |
|---|---|---|
| `app/<p>/core` | Domain + use cases (ontology rules, payroll calc, ledger postings, calendar recurrence, mail labels) | SQLite, HTTP, S3, IAM, `storage::`, `data::` |
| `app/<p>/ports` | Traits: records, blob, mailbox, identity, notify, pack-id, payments, calendar-embed | Adapter impls |
| `app/<p>/adapters` | SQLite v1; our-cloud Connect client; Postgres/S3/IMAP/Stripe/on-prem | Business rules |
| `app/<p>/facade` | That product’s UX/API | Cloud SKU implementation |

**One active adapter per port per tenant.** Not dual-write. Between apps (hr → calendar): **ports**, not `use calendar_core`.

**Portable means.** Core source does not change when the substrate does. Proof: the same Foundry/hr/payroll tests run against SQLite and against a commodity adapter (Postgres, S3 API). Rewriting ontology/payroll because we stopped selling `storage/` is a D-25 fail. Live outage behavior is **not** this decision (that is SLO/cell failover of whatever adapter is selected).

**MUST (portable app cores)**

- **achieves:** retiring or unbundling a cloud SKU does not kill tenant apps; they keep running on commodity adapters.
- **origin:** in-process cloud cores; “just call storage/”; Foundry would die if we EOL’d a cap.
- **rule:** app business logic only in `core`; every IO through a port; v1 SQLite adapter per durable port; our cloud is one adapter among Postgres/S3/IMAP/Stripe/on-prem; no `path =` cloud `core/`/`ports/` from `app/`; between-app composition is ports too. Not an HA-during-outage claim.
- **ensure:** new `app/<p>/core` crate that imports sqlx/reqwest/`storage-*`/`data-*`/`iam-*` fails review; each durable port has a non-Oyatie adapter path in charter (SQLite and/or commodity).
- **overturn_when:** a five-field ADR same-wave allows in-process cloud cores or makes an app require a living Oyatie SKU with no substitute adapter.

### D-26 — Do not add a trusted-tenant *mode* (premise rejected)

Founder 2026-08-22: maybe privileged access as a trusted tenant vs untrusted, to prove the cloud works — flag or otherwise. **Do not weaken or add a regret.**

**The premise is surplus.** D-23 already proves the cloud (apps call the sold facade). D-25 already proves portability (commodity adapters at SKU EOL). IAM already issues service principals. A named trusted/untrusted **mode in the app** is a second path. Second paths become skip-PDP, unmetered, extra proto, or `cfg` linking `storage/core`. That is a product regression.

Hyperscalers do not ship “trusted GCS for Gmail.” One API, identities, ACLs. Console is not an enum inside S3.

**Do not add:** feature flag; `TrustedTenant` in `app/*/core`; first-party quota class; skip-PDP; second cloud API; in-process cloud cores.

**Already exists (do not rebrand as trusted mode):** adapter injection (sqlite | cloud-client | s3 | …); IAM principal including first-party; cell-local VIP as **network**, not an app bit; customer-data access uses **that customer’s** grant.

**MUST**

- **achieves:** no regret mechanism that splits dogfood from customers or bypasses D-23/D-25.
- **origin:** “prove it works” invited a second path; second paths become the only path.
- **rule:** no trusted-tenant mode, flag, or skip-PDP. Proofs remain sold facade + commodity adapters. First-party is an IAM principal. Adapter injection is substrate choice, not privilege.
- **ensure:** review rejects `TrustedTenant`, `cfg(trusted)` cloud-core path-deps, first-party unmetered classes, and privileged extra APIs.
- **overturn_when:** a five-field ADR same-wave shows a missing capability that IAM+adapters cannot express AND names a fail-closed alternative that is still one proto.

### D-27 — Current views are untracked, semantic, and revision-keyed

D-36 controls current owner knowledge and replaces the former owner-docs model.
There is no tracked human-authoring tree in the destination. A human or agent
view is a **projection**, never a second authority.

**Native inputs.** The view reads the same artifacts that product and control
consumers read: Rust types and tests; Cargo and Buck declarations; port traits;
protobuf contracts; Cedar; reconciler IR; SLO-controller inputs and generated
outputs; and `OWNERS`. Current surfaces use semantic domain and operator names.
ADR numbers, D-n labels, migration sequence numbers, and former prose filenames
may appear only as transitional provenance, never as the name or lookup key for
a current check, contract, test, error, view, or workflow.

**Immutable revision binding.** The view request binds an opaque immutable
source revision supplied through an SCM port plus the view schema/generator
identity. The current Git adapter resolves that revision to verified commit and
tree bytes. A branch, tag, working tree, mutable `HEAD` label, timestamp, or
"latest" is not a durable identity. A view refuses if the exact revision bytes
or required native inputs are unavailable or mismatch. It may be cached outside
the repository under that full key; it is never tracked, checked in, or treated
as input to the native authority. Before any operational owner prose is deleted,
the exact-candidate view must be materialized from that immutable revision and
remain available for inspection without network access.

**Human work and history.** Proposals, rationale under consideration, sequence,
and acceptance discussion live in the PR body or an external work system. Once
landed, current truth is the native candidate and history is available only
through the SCM by a separate, explicitly requested historical lookup. A current
view never enables that lookup or mixes historical material. No archive,
changelog copy, Markdown tombstone, migration receipt, or generated view stays
in tree.

**Root compatibility only.** `/README.md`, `/AGENTS.md`, and `/CLAUDE.md` are
the complete destination Markdown set. They bootstrap humans and harnesses;
they do not duplicate owner contracts. Each of the two agent hubs independently
remains at most 300 physical lines **and** 32 KiB (32,768 UTF-8 bytes); either
ceiling can fail even when the other passes. All other tracked Markdown,
including under `docs/`, `templates/`, a capability, or `app/<product>/`, is
transition input to remove under D-36.

**MUST (immutable current views)**

- **achieves:** humans and agents can inspect exact-revision current truth
  without copying it into a prose authority or mixing it with history.
- **origin:** g3doc, quartets, indexes, archives, and numbered citations drifted
  from native artifacts and made stale context look current.
- **rule:** current views are semantic projections of native authority, bind an
  immutable SCM-neutral revision plus view identity, and remain untracked; Git
  commit/tree resolution is the current adapter, not the product contract;
  proposals are off-tree; SCM-only history requires a separate explicit opt-in
  historical lookup and never mixes into a current view.
- **ensure:** the separate Pipeline lane later qualifies exact-tip view tests,
  refusal of historical inputs on current-view requests, retained-reference
  checks, capability/app Markdown rejection, and the three-root-Markdown
  allowlist plus independent 300-physical-line and 32-KiB UTF-8-byte ceilings
  for each root agent hub. This amendment neither implements that adapter nor
  claims enforcement is live.
- **overturn_when:** measured evidence proves an exact-revision view cannot make
  native current truth operable and a bounded five-field replacement lands
  atomically without restoring a parallel tracked authority.

### D-28 — Shared contracts: draft vs agreed (ports + adapters)

Caps and apps bind **only** through ports and adapters (D-23/D-25). There is
**no** `contracts/` root and **no** `libs/ports`.

**Local / unagreed.** A team may invent a port under `ports/draft/<port>/`
(crate `owner-port-draft`). The word `draft` is in the **path and crate name**
so it cannot be mistaken for an agreed contract. Rename is `git mv`. Other
owners **must not** depend on it. Sold proto packages **must not** contain
`draft`.

**Shared / agreed.** The second owner that needs the same shape **cannot** copy
the draft. Reconcile onto **one** name on the **provider** (`storage-blob`,
`data-records`, mailbox, clock, …) using the `owner-port` grammar, plus the
sold proto `*.v1` if it is a facade. Promotion is `git mv` of the draft onto
the provider, not a fork. Google analog: AIP-181/185 — `v1alpha` is expected
to break; `v1` is the compatibility surface. We use `draft` internally so
rename stays cheap; customer-facing proto uses `v1` / `v2`, not `v1draft`.

**Escalated review** (D-29) is what turns draft → agreed. Observation that two
trees “look similar” is not agreement.

**MUST (contract stages)**

- **achieves:** unagreed shapes are grep-visible and cheap to rename; agreed
  names are the compatibility surface; no third contracts tree.
- **origin:** hexagonal ports proliferate incompatible `blob` traits; shared
  JSON/IDL dumps drifted; founder required distinguishable unagreed contracts.
- **rule:** bindings are ports+adapters; `ports/draft/` is local and not a
  dependency; a second consumer forces one agreed provider port + proto v1;
  no `contracts/` or `libs/`; draft illegal in sold proto.
- **ensure:** review rejects cross-owner `path =` into `ports/draft/`; agreed
  crate names match `owner-port`; layout rejects a `contracts/` root.
- **overturn_when:** a five-field ADR names a different staging that still
  makes unagreed contracts visually distinct and keeps shared names reviewed.

### D-29 — Amendment jurisdiction (owner-local vs external vs repo root)

**Same structure, different blast radius.** Every cap and every app uses the
D-8 children. Who may **amend** them is not the same.

| Blast | What | Who reviews | Analog |
|---|---|---|---|
| **Owner-local** | `core/`, `ports/draft/`, local `adapters/draft/`, `cedar/`, `iac/` that only this engine consumes, and owner-local tests/build declarations | That owner's `OWNERS` | Package OWNERS |
| **External contract** | Agreed `ports/<port>/`, adapters other owners consume, sold `facade/` proto, any crate another owner `path =`s | **This owner + every consuming owner + architecture** | Google API review / AIP; not a drive-by |
| **Repo root** | `README.md`, `AGENTS.md`, `CLAUDE.md`, D-8 structural allowlist, `rust-toolchain.toml`, workspace membership policy, required `presubmit` | Architecture (+ founder on law) | Central compatibility/structure; not a cap feature PR |

Owner-local includes amending this owner's semantic native artifacts and
`ports/draft/` content. It is **not** a license to add Markdown, change the
canonical children, inner crate files, crate grammar, or add a private planning
or decision tree. Proposals remain in the PR body or external work system.
Structure is the same for every owner and does not evolve per team. Root
compatibility/structure and agreed contracts do not land as a side effect of a
feature PR.

**MUST (jurisdiction)**

- **achieves:** teams move inside a frozen tree; they cannot silently bind the
  rest of the company, rewrite structural law, or fork layout.
- **origin:** local iteration blocked by org review; conversely, shared ports
  and root hubs were edited from a cap dump; teams invented prose and private
  taxonomies inside “their” root.
- **rule:** owner OWNERS for **native content** in canonical children that does
  not leak;
  the D-8 shape and D-30 grammar are not owner-amendable; escalated review for
  agreed ports/proto/facade and for root compatibility/structure; tracked owner
  Markdown stays forbidden.
- **ensure:** PRs touching agreed ports, sold proto, or root law name the extra
  reviewers; layout rejects non-canonical children and invented inner taxonomies
  or capability/app Markdown even on owner PRs.
- **overturn_when:** a five-field ADR replaces this split with an equally
  fail-closed OWNERS + API-review model.

### D-30 — Names and inner files: Cargo + google3 + AIP (not invented taxonomy)

The D-8 **tree is frozen**. Names *inside* that tree follow conventions that
already exist. Do not invent a per-repo dialect.

**Cargo package vs rustc crate (RFC 940, API guidelines C-CASE / RFC 430).**

| Thing | Convention | Source |
|---|---|---|
| `[package] name` | kebab-case, `[a-z0-9-]+`. Grammar above. No `-rs`, `-rust`, `oyatie-`, `cloud-` | RFC 940; C-CASE; AWS SDK `aws-sdk-s3`, tokio `tokio-util` |
| rustc crate / `use` path | hyphen → underscore. **Omit** `[lib] name` so Cargo does this | RFC 940 |
| Directory leaf | last grammar token: `blob`, `blob-sqlite`, `pdp-app` | google3 path identity; matklad: don't strip prefixes *from the Cargo name* |
| Cargo name | full `owner-port` / `owner-port-backend` / `owner-app` (Cargo's namespace is **flat**, so the prefix lives in the name) | matklad large workspaces; not a `crates/` dump — faces stay D-8 |
| Modules / files | `snake_case.rs`; `src/lib.rs` (lib), `src/main.rs` (facade bin, once attached — D-30 amendment) | RFC 430; Cargo book |
| Types / traits | `UpperCamelCase` (`BlobStore`, not `BLOBStore`) | RFC 430 |
| fns / methods | `snake_case`; getters are `blob()`, not `get_blob()` | C-GETTER |
| Consts / statics | `SCREAMING_SNAKE_CASE` | RFC 430 |
| Cargo features | additive; named `std` not `use-std` / `no-foo` | C-FEATURE |
| Tests | unit next to code; integration in `tests/` | Cargo book |
| Proto package | `owner.api.v1`; directory **matches** package; files `snake_case.proto` not `v1.proto` | AIP-191, AIP-185 |
| Proto messages / RPCs | `UpperCamelCase` | AIP-190 |

**Why not rust-analyzer's flat `crates/`.** That layout is right for a *single*
Cargo project (hir, hir_def, …). This repo is a google3-shaped capability
monorepo: path is identity (`storage/ports/blob`), Cargo names carry the
owner prefix because Cargo cannot express `storage::blob`. Fuchsia/GN and
google3 work the same way: hierarchical path + flat target/package name.

**Why not `domain/` / `use_case/`.** Those are Uncle Bob Clean Architecture
folders. They are not Cargo, not google3, not rustc, not Oxide/tokio. Core
is a lib crate; ports are sibling crates; adapters implement ports. Extra
module names are ordinary `snake_case` as the code needs them.

**MUST (established names)**

- **achieves:** one grammar humans and cargo already know; rename stays a
  `git mv` of a leaf that matches the last token; proto path = package.
- **origin:** D-8 needed inner files; an invented `domain/use_case/connect`
  taxonomy is not hyperscaler or Rust-workspace practice.
- **rule:** RFC 430/940 + dir leaf = last token + package = full grammar +
  AIP-191 proto under `facade/proto`; no `[lib] name` override; no
  `-rs`/`oyatie-`/`cloud-`; no required `domain/`/`use_case/`/`crates/`.
- **ensure:** new crate PRs match the table; proto dirs match `package`;
  layout still rejects extra faces.
- **overturn_when:** a five-field ADR cites a different *established*
  (Cargo or AIP or google3) convention and lands same-wave.

#### D-30 amendment (2026-08-30) — a facade roots at a library until its listener attaches

- **achieves:** a service surface can be edited while it is still staged,
  so a crate move can repoint its consumers. The Cargo convention this
  table cites is unchanged: a binary crate roots at `src/main.rs`.
- **origin:** the row read `Process: src/main.rs`, and admission enforced
  it on every touched `facade/` leaf. 31 of 54 facades have no
  `src/main.rs`; the `iam/facade/tenant-rbac-*` family records why in its
  own source — `deployed_listener_attached: false`, "does not start a
  listener". Those crates were unedittable: any change to such a manifest
  demanded inventing a binary the crate says must not exist yet. Because a
  moved crate must repoint its consumers, one staged facade anywhere in a
  dependency chain blocked the extraction; `shared-pdp-kernel` could not
  reach `policy/` for that reason alone.
- **rule:** a `facade/` leaf roots at `src/main.rs` when it has one and at
  `src/lib.rs` otherwise. Both are canonical; a touched leaf must present
  one as a regular blob. The relaxation is facade-only — a `core`, `ports`
  or `adapters` leaf that ships a binary instead of a library is still
  refused. It is also one-way: a facade that HAD `src/main.rs` at the
  merge base must still have it at head, so a running service cannot
  become a library by deleting its entry point.
- **ensure:** admission tests fix all three directions — a lib-rooted
  facade is admitted, a binary-rooted library face is refused, and
  deleting a facade's existing `src/main.rs` is refused. Both `autobins`
  and `autolib` guard a facade, since either may carry its canonical
  target. The ratchet's boundary is a fourth case it does NOT cover:
  renaming a facade's whole directory while dropping `src/main.rs`
  passes, because the change parser discards rename pairing and the
  destination leaf never had a binary at the merge base. That case is a
  reviewed crate move under D-8/D-41, not a silent demotion; closing it
  needs a rename-pairing contract the path-set does not carry.
- **overturn_when:** a five-field ADR shows that a staged surface and its
  attached listener should be different faces, so that `facade/` can mean
  a running process again without making staged crates unedittable.

#### Semantic operational names

- **achieves:** operators can understand a check, job, test, or failure without
  consulting a decision-number index.
- **origin:** decision identifiers leaked from provenance into workflow labels
  and diagnostics, turning historical numbering into the user interface.
- **rule:** executable, check, job, error, test, and code-facing names MUST be
  semantic. Decision identifiers remain valid provenance in citations, comments,
  and metadata. Historical ADR filenames, headings, and identifiers MUST NOT be
  renamed or renumbered merely for naming cleanup; legitimate ADR content
  amendments remain allowed.
- **ensure:** regression tests inspect workflow display names and emitted
  diagnostics for semantic wording while explicitly admitting ADR citations and
  decision-file paths; review preserves historical provenance without freezing
  legitimate content amendments.
- **overturn_when:** a recorded challenge demonstrably shows that an external
  protocol requires a stable numbered identifier and the same surface retains a
  semantic operator-facing label alongside that identifier.

### D-31 — Ephemeral out-of-tree sandbox (owner-writable only)

D-29 is policy. This is **physical**. A default implement/review worker
must not be able to edit `iam/` while dispatched to `storage/`.

**Hyperscaler analogs (use, don't cargo-cult).**

| Practice | What it actually does | We take |
|---|---|---|
| Google CitC / Piper workspace | Full *view*, overlay only of dirty files; OWNERS on submit | Out-of-tree workspace; **not** “hide the rest of google3 from reads” — CitC does not |
| git cone sparse-checkout + worktree | Working tree materializes only named dirs ([GitHub](https://github.blog/open-source/git/bring-your-monorepo-down-to-size-with-sparse-checkout/), git-scm) | Default cone = one owner |
| Bazel action sandbox | Process sees **declared inputs**; Linux namespaces / macOS `sandbox-exec`; undeclared writes die ([Bazel sandboxing](https://bazel.build/docs/sandboxing)) | Write-jail + declared read set |
| Docker AI / agent sandboxes | VM or Landlock/Seatbelt; workspace-write vs full-trust | Landlock (Linux) / Seatbelt (macOS) when the runtime has it |
| Codex/Claude worktrees | Isolated checkout per agent, destroy later | Ephemeral; not the human clone |

**Default worker sandbox (one owner).**

1. **Out of tree.** `git worktree add` under a temp path (`/tmp`, `/private/tmp`, …). Never the human clone (`integ/audit` or `~/Developer/oyatie`). Already required; this names why.
2. **Ephemeral.** Create on dispatch. Remove on PR merge, lane abort, or idle expiry. Leftover worktrees are bugs, not inventory.
3. **Writable cone.** Maximum is one `<cap>/` or `app/<product>/` named in the dispatch (D-29). Parallel subagents **narrow** that to disjoint **leaf crate** dirs (D-32). Plus the worktree's own `.git` / index so git works. One git index per worktree — two writers in one worktree are forbidden.
4. **Read-only declared inputs** (not “the whole company” as writes). Root compatibility (`AGENTS.md`, `CLAUDE.md`), `rust-toolchain.toml`, workspace `Cargo.toml`/`Cargo.lock`, and exact-revision native artifacts for agreed ports/facades this owner already consumes (D-28). A D-36 migration lane may additionally declare its frozen prose as migration data. Toolchain. `/tmp` for build scratch.
5. **OS write-jail** when the host can: Landlock path-beneath on Linux, Seatbelt `sandbox-exec` on macOS (Bazel Darwin sandbox, Anthropic/DeepSeek agent runtimes). Writable = owner cone + tmp + target dir. Everything else read-only or absent. The agent **cannot** widen this set.
6. **Local proof is buck2** (D-32 / ADR-0716). Not `cargo nextest --workspace`, not N cargo processes. Optional `cargo nextest -p <one crate>` is local feedback only. Full-graph cargo stays CI.

**Rejected (would regret).**

- **Hide other owners from all reads.** Cargo path-deps and D-28 reconciliation need to *see* the agreed provider port. Blind agents duplicate `blob` traits. CitC shows the repo; OWNERS bind submit. We jail **writes**.
- **A VCS ratchet product** (claim/verify/done). ADR-0363. Sandbox is dispatch plumbing, not a merge context.
- **JSON census of allowed paths** as a gate fleet. D-17. The cone is the dispatch argument, not a repo-wide list to freeze.
- **Agent self-service `sparse-checkout add iam/`.** Expanding the sandbox is a **new dispatch** (D-29 escalated), with extra writable cones named by the dispatcher.

**Escalated D-29 lane.** Writable cones listed explicitly (this owner + consuming owner, or repo-root files named). Still out-of-tree, still ephemeral, still not “the whole monorepo.” Coordinator/architecture that must survey many trees is a different profile — read-mostly, not a default implementer with a full write mount.

**MUST (owner sandbox)**

- **achieves:** a worker physically cannot mutate another owner's tree; D-29 is enforced by FS not by hope; parallel lanes do not share a checkout.
- **origin:** worktrees existed but were full clones; agents edited across caps; founder asked for ephemeral out-of-tree sandbox keyed to ownership.
- **rule:** default worker = ephemeral out-of-tree worktree; writable = dispatched owner only; OS write-jail when available; declared read-only inputs for law + existing path-deps; agent cannot expand the cone; D-29 extra cones are dispatcher-named; no ratchet product; no full-read hide.
- **ensure:** dispatch records the owner path; writes outside it fail or are review-blocking; worktree is deleted when the lane ends; human clone stays untouched.
- **overturn_when:** a five-field ADR names a stricter isolation (e.g. per-crate Bazel-style input set) that still lets Cargo and D-28 work, or a fail-closed alternative that still keeps writes owner-local.

### D-32 — Parallelism is a leaf crate + buck2 locally; cargo is the linearized merge

The cargo **workspace** is the merge graph (ADR-0716). It is not an isolation
mechanism. Two agents in one workspace still collide on:

| Shared file | Who may write it live | Why |
|---|---|---|
| A leaf crate's `src/`, `Cargo.toml` | **One** lane | Git merge of the same crate is the conflict you already feel |
| Root `Cargo.toml` members | **One** lane (D-29) | Adding a crate is org-graph |
| `Cargo.lock` | **One** lane, only when a crates.io / version pin changes | N cargo processes regenerate it; path-dep-only work must not touch it |
| One worktree `.git/index` | **One** process | Two subagents in one checkout serialize on `index.lock` even on disjoint files |

**Unit of parallel write (OVERRULED by D-39):** not the whole crate.
Crate → files/modules → one primary Item per file (D-35). N agents on
the **same crate** is the point. Same **file/Item** → assign disjoint
or stack that item; do **not** lock the crate.

**Local (N-way).** `buck2 build` / `buck2 test` of the dispatched targets.
Hermetic per action; no `Cargo.lock` rewrite; no workspace-wide cargo lock.
That is why buck2 exists here (ADR-0716). Reindeer still reads `Cargo.toml` as
manifest input — do not invent a second package graph.

**CI (1-way).** Merge queue / `presubmit` runs **one** `cargo nextest --workspace`.
By then lanes are linearized; one lockfile writer. Cargo at CI is therefore
**not** the N-parallelism problem. Dual cargo+buck2 merge proof stays forbidden.

**Do not:** per-agent `Cargo.lock`; N virtual workspaces; `cargo update` /
`generate-lockfile` inside owner sandboxes; `--workspace` cargo as the
subagent loop; two live writers on the same **file/Item** (D-39). Two
writers on the same **crate** (disjoint files) is the desired case.

**MUST (parallel crates, local buck2)**

- **achieves:** subagents actually run in parallel; lockfile and crate files
  have a single writer; local proof does not N-way cargo; CI cargo stays the
  one merge graph.
- **origin:** cargo workspace still shares `Cargo.lock` and crate files;
  splitting workspaces creates N lockfiles; founder: buck2 is the local
  resolution, CI cargo is fine once linearized.
- **rule:** parallel write unit = file/Item (D-39), not the crate;
  worktree per parallel process; local verify = buck2; `Cargo.lock` is
  single-writer; membership is directory glob/generated (D-39); CI cargo
  `--workspace` is merge-only. Do not crate-lock.
- **ensure:** dispatch names crate paths when an owner is split; review
  rejects lockfile diffs on path-dep-only changes; agents do not run
  workspace cargo locally as the parallel loop.
- **overturn_when:** pipeline **serves** buck2 as tenant #0 presubmit
  (ADR-0716 overturn_when) same-wave, or a five-field ADR names another
  single-writer lock story that still permits crate-parallel worktrees.

### D-33 — Structural Mutation Separation

The reorg (D-8 last-leg, dumps, `git mv` / `git rm`, crate grammar, debrand,
workspace membership) is **not** the same class of change as implementing
ontology or a blob adapter. Mixing them is how agents tangle layout with
behavior (empirical: majority of agent “refactors” land inside feature
commits and become unreviewable).

| Class | What moves | Lane |
|---|---|---|
| **Structural** | Faces, crate dirs, `git mv`/`git rm`, root `Cargo.toml` members, `Cargo.lock`, D-8 children, proto package path, agreed port promotion | One serialized (or explicitly stacked) reorg lane. Escalated (D-29). This is `#2221`. |
| **Behavioral** | `src/`, `tests/`, Cedar, IR, proto, and other native artifacts under an **existing** owner | Parallel leaf-crate worktrees (D-32). Frozen shape (D-8/D-30). |

`REFACTOR` = `git mv`. `REMOVE` = `git rm`. Neither is a feature PR. After
the structure wave, implement agents **do not** mutate the tree shape.
Draft→agreed port promotion is structural (D-28), not a side effect of a
Foundry feature.

**MUST (separate structure from behavior)**

- **achieves:** review can tell layout from logic; parallel agents do not
  fight the workspace graph; reorg can finish.
- **origin:** one PR that both moves `iam/` and changes PDP semantics;
  cargo lock + git mv + feature in the same index.
- **rule:** a lane is structural or behavioral, not both; implement
  dispatches do not add/rename/remove crates or edit `Cargo.lock` /
  root members; structure stays frozen while behavior fans out.
- **ensure:** review rejects mixed PRs; D-8 allowlist diffs are structural
  lanes only.
- **overturn_when:** a five-field ADR names a mechanical splitter that
  still keeps layout diffs independently revertible.

### D-34 — Cache and graphs (not a cargo lock-bypass, not an AST-merge product)

N worktrees need **shared compile reuse** without sharing cargo’s locks.

| Idea | Ruling | Why |
|---|---|---|
| Shared **read-only** cache (buck2 CAS / disk cache, Bazel remote/disk cache, cargo **registry** as fetch cache) | **Keep** | Hyperscaler default. Trusted writers (CI + local buck2); agents **read**. Do not let agents poison CAS. |
| Global target dir / `CARGO_TARGET_DIR` for all worktrees | **Reject as the parallel loop** | Cargo’s target lock serializes N agents; bypassing it corrupts artifacts ([cargo#16804](https://github.com/rust-lang/cargo/issues/16804)). |
| “Lock bypass” of cargo target-dir | **Reject** | The lock exists to prevent corruption. Bypass **git** `Cargo.lock` mutation instead (D-32). |
| `cargo --offline --locked` if cargo is used at all | **Keep** | No fetch, no lock rewrite. Offline mode is not a second workspace. |
| Path mapping so cache keys are not worktree-absolute (`SCCACHE_BASEDIRS`, Bazel execroot, buck2 action keys) | **Keep as cache impl** | Hits must be content-addressed, not `/tmp/wt-3/...` paths. |
| Aggregated AST patching (merge N agents’ edits in one file via AST) | **Reject as a product** | D-32 already forbids two writers on one crate. An AST-merge service is a new dual stack. Git merge of disjoint crates is enough. |
| **Build graph** (buck2) | **Keep — dispatcher** | Who depends on this target; local `buck2 test //owner/...`. |
| **Crate/code graph** (`cargo metadata --offline --locked`, rust-analyzer) | **Keep — dispatcher** | Concurrent-safe paths; D-28 consumers. Read-only. |
| **AST/HIR graph** (rust-analyzer) | **Keep as editor/query**, not a stored census | Do not freeze HIR dumps in git (D-17). |

Local N-way proof remains **buck2** (D-32). The cache sits **beside**
worktrees, not inside them. When CAS is served by `storage/` that is
pipeline/RE (ADR-0716 overturn), the same read-only rule holds.

**MUST (cache + graphs)**

- **achieves:** parallel worktrees reuse compiles; agents cannot take the
  cargo target lock or rewrite `Cargo.lock`; dispatch uses graphs we
  already have.
- **origin:** N cargo `target/` dirs; shared `CARGO_TARGET_DIR` deadlocks;
  founder listed cache, offline cargo, AST patch, graphs as candidates.
- **rule:** shared cache is content-addressed and read-only to agents;
  no shared cargo target-dir as the parallel engine; cargo if used is
  `--offline --locked`; dispatcher uses buck2 + metadata/r-a; no
  AST-merge product; no HIR/census files in git.
- **ensure:** path-only PRs have no lockfile diff; agent logs do not show
  `cargo update` / `generate-lockfile`; cache writers are not owner
  sandboxes.
- **overturn_when:** a five-field ADR shows a safe shared cargo cache that
  does not lock or corrupt, or an AST merge that is still crate-disjoint
  (then it is unnecessary).

### D-35 — File budget: 300 lines plus an independent 32 KiB agent-hub ceiling

For agents, a 2k-line hand-written native file is a context and conflict
magnet. The destination maximum remains **300 physical lines** for every
hand-written file. Prefer splitting earlier; 100 is not a gate. Count physical
lines without comment-stripping or a repository file-count census.

**Destination exemptions (closed).** `Cargo.lock`; `third-party/`; generated
artifacts such as `*.generated.*`, protobuf/Reindeer output, and controller
outputs; and vendored lock-step snapshots. The root compatibility set
`README.md`, `AGENTS.md`, `CLAUDE.md` is allowed by D-8, but it is not an
unbounded prose exemption: each agent hub independently remains at most 300
physical lines and 32 KiB (32,768 UTF-8 bytes). The byte ceiling is independent
of the line ceiling. The README follows the ordinary 300-line budget.

Existing ADRs, owner quartets, owner READMEs, and owner prose trees are frozen
transition input under D-36. Their current length is tolerated only so they can
be classified and deleted atomically. They may not be expanded or used as a
template for a new owner. Native over-budget files split inside the same leaf
crate when touched or in a dedicated structural lane.

**Enforcement state.** A touched-path budget and Markdown allowlist belong to
the separate Pipeline compatibility/enforcement lane. This ADR-only amendment
defines the target and does **not** claim either check is implemented, qualified,
or live.

**MUST (file budget)**

- **achieves:** native diffs stay reviewable and the three compatibility files
  cannot become a replacement wiki.
- **origin:** unbounded code and prose files became context magnets; broad
  exemptions preserved stale owner novels; file-count censuses are forbidden.
- **rule:** hand-written destination files are at most 300 physical lines;
  destination exemptions are generated, lock, third-party, and vendored
  lock-step artifacts; the three root Markdown files are allowed, but each agent
  hub independently remains at most 300 physical lines and 32 KiB (32,768 UTF-8
  bytes); frozen migration prose may only be deleted.
- **ensure:** the separate Pipeline lane later checks touched native files and
  the three-root-Markdown allowlist plus the independent per-agent-hub UTF-8
  byte ceiling without `expected_total`, a baseline, or a frozen corpus; this
  lane claims no live enforcement.
- **overturn_when:** a five-field amendment names a different bound that still
  fits agent context, keeps root compatibility bounded, and does not become a
  file-count freeze.

### D-36 — Current-only native owner knowledge; frozen atomic transition

This amendment replaces tracked owner prose with current truth on the artifacts
that compilers, tests, policy engines, controllers, reconcilers, build systems,
and ownership enforcement actually consume.

**Semantic native authority.** Each capability and `app/<product>/` expresses
its current contract through the applicable native surfaces:

| Concern | Current semantic surface |
|---|---|
| Domain behavior and invariants | Rust types/implementation plus exact tests and typed failures |
| Owner and cross-owner contracts | Port traits, protobuf packages/methods/messages, contract tests |
| Backend realization | Adapter types, bindings, conformance tests |
| Authorization | Cedar schema/policy and PDP tests |
| Desired state and reconciliation | Typed IR, reconciler inputs, observed-state/status types, reconciliation tests |
| Reliability | SLO-controller inputs, generated controller output, failure-injection tests |
| Build/dependency declaration | `Cargo.toml`, `BUCK`, generated admitted relations |
| Jurisdiction | `OWNERS` and ownership enforcement inputs |

Names on those surfaces describe the domain or operator action. Sequential
ADR, D-n, migration-wave, or ticket identifiers are transitional provenance
only; they may appear in SCM metadata or comments but must not be the current
surface, lookup key, check name, test name, error name, or workflow label.

**No tracked prose authority.** No `*.md` is tracked anywhere below a
capability or `app/<product>/`. Proposals, plans, rationale under review, and
remaining-work sequences live in the PR body or an external work system. A
human-readable current view follows D-27: it is derived on demand from the
native artifacts, keyed by an immutable SCM-neutral revision and view identity,
and never tracked. Historical truth is available only from SCM; the current Git
adapter uses verified commit/tree bytes. Historical lookup is a separate,
explicit opt-in request; a current view never enables it or mixes its results.

**Fail-closed transition.** The following order is mandatory:

1. Pipeline must first stop requiring `ADR.md`, `PRD.md`, `SPEC.md`, and
   `PLAN.md` in a **separate Pipeline compatibility lane**. That change and all
   later enforcement are not implemented or made live by this ADR-only lane.
   No new capability or `app/<product>/` owner may be created before that
   prerequisite lands; afterward, a new owner starts with native artifacts and
   no Markdown.
2. On adoption, every existing owner `ADR.md`, `PRD.md`, `SPEC.md`, `PLAN.md`,
   `README.md`, and `docs/**/*.md` becomes read-only migration input. No lane
   may add or rewrite that prose; the only admitted owner-prose diff is its
   atomic deletion in the owner migration.
3. The migration builds an ephemeral, off-tree claim ledger. Every source
   claim receives **exactly one** result: accepted-current, proposal/work,
   historical/rejected, or `Unknown`. Any ambiguity, unresolved conflict,
   duplicate classification, or absent classification yields `Unknown`.
   `Unknown` is fail-closed: it blocks native projection and prose deletion and
   must not be coerced into a successful class. Conflicting claims are
   challenged against native behavior, consumers, owner intent, and current
   acceptance until evidence supports one success result or they remain
   `Unknown`.
4. Accepted-current claims project exactly once onto the semantic native
   surfaces above. Proposal/work claims move to the PR body or external work
   system. Historical/rejected claims remain only in SCM history and require a
   separate explicit opt-in historical lookup; they never enter a current view.
   A prose sentence, generated Markdown copy, numbered citation, or retained
   archive is not a projection.
5. One owner migration candidate contains every required native change and the
   deletion of **all** that owner's Markdown. Exact-candidate tests exercise the
   compiler/test/PDP/SLO-controller/reconciler/Cargo/Buck/ownership consumers;
   an exact-tip derived view is regenerated from the same immutable revision and
   is available offline before deletion; retained-reference checks refuse any
   live reference to deleted prose. Every deletion also requires a
   failure-injection proof that a failed consumer, missing native input,
   revision mismatch, or unavailable offline view refuses the migration. The
   candidate refuses on a failed or missing proof, test failure, revision
   mismatch, incomplete native projection, unresolved retained reference,
   unavailable offline view, or any `Unknown` result.
6. Native projection and source deletion land atomically. No tombstone,
   migration matrix, receipt, changelog copy, archive, redirect, or in-tree view
   remains. If any step cannot complete, the source stays frozen and the owner
   remains unmigrated; partial deletion is forbidden.

**Pipeline boundary.** Required compatibility, transition, reference, exact-tip,
Markdown-allowlist, 300-physical-line, and independent 32-KiB UTF-8-byte checks
are a later Pipeline-owned implementation and qualification lane. This section
specifies their contract only. It does not claim those checks, the view adapter,
owner migrations, or enforcement are live.

**MUST (current-only native owner knowledge)**

- **achieves:** exact-revision current truth without prose duplication or
  historical context poisoning.
- **origin:** mandatory quartets, g3doc trees, indexes, archives, and numbered
  citations became stale parallel authority.
- **rule:** no tracked Markdown under capability/app roots; native artifacts are
  authority; proposals/work are off-tree; SCM-only history requires a separate
  explicit opt-in historical lookup and never mixes into a current view; derived
  current views are revision-keyed and untracked.
- **ensure:** Pipeline compatibility change, atomic owner migrations,
  `Unknown` classification/refusal without coercion, retained-reference checks,
  current-view refusal of historical inputs, exact-tip tests, per-deletion
  failure-injection proofs, offline-available immutable-revision-bound views,
  three-root-file allowlist, and independent per-agent-hub ceilings of <=300
  physical lines and 32 KiB (32,768 UTF-8 bytes). These are separate Pipeline
  follow-through and are not claimed live by this ADR-only amendment.
- **overturn_when:** measured evidence proves an irreducible tracked human
  contract cannot be represented by native authority or an off-tree view, and a
  bounded replacement lands atomically.

### D-37 — Shared native config is not a 300-line split; one fold

N agents colliding on `Cargo.toml` / `Cargo.lock` / YAML / JSON is the
problem D-32 named. Splitting those files like `lib.rs` does not work
(Cargo has one manifest; lock is one graph). **Different resolution.**

**Prefer delete or generate.** File-based config stays the closed minimum:
root `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `rustfmt.toml`,
`deny.toml`, per-crate `Cargo.toml` (travels with that crate’s writer).
No new JSON/YAML product. OpenSLO/faces stay generated. Cedar is per-owner
policy, not a global yaml.

**Denylist (implement agents: no in-place edit).** Root workspace
`Cargo.toml`, `Cargo.lock`, toolchain/fmt/deny, root `README.md`, `AGENTS.md`,
`CLAUDE.md`, and generated `*.json`/`*.yaml`. Crate-local `Cargo.toml` is
**not** on this list — it is that crate’s file (D-32). Frozen transition prose
may only be deleted by its D-36 migration lane.

**Additive path: uuid fragments, not clones of the whole file.** Copying
`Cargo.toml.<uuid>` and hoping to 3-way merge is still a novel. Agents
**add** a unique file, e.g. `Cargo.toml.d/<uuid>.toml`, that states only
the addition (`members = ["storage/adapters/blob-sqlite"]`). Paths never
collide across worktrees. Ephemeral: the fragment is not the long-term
SSOT.

**Fold is serial, once, on the receiving branch.** If each worktree
pre-commit *already* folds into `Cargo.toml` and deletes the fragment,
merge recreates the `Cargo.toml` conflict. So:

| Stage | What happens |
|---|---|
| Implement lane commit | Fragment only. Hook **rejects** in-place denylist edits. Validates fragment schema. Does **not** rewrite the canonical file. |
| Restack / merge_group / integration pre-commit | **One** Rust fold engine: apply all fragments (set-union of members; fail on key conflict), write canonical, `git rm` fragments, refresh `Cargo.lock` **once** (`--locked` path if no pin change; regenerate only if the fold added a crates.io dep). |

That step is “pre-commit” in the TAP sense (before the linearized snapshot
is proven), not N local hooks racing.

**Cannot fragment-merge:** root compatibility Markdown (single writer);
frozen migration prose (delete atomically, never merge); `Cargo.lock` body
(derive after fold); YAML/JSON novels (do not exist). Duplicate fragment keys
with different values **fail closed**. Proposals and plans remain off-tree.

**YAGNI bound (thought experiment).** After D-33 the common implement
lane never touches root `Cargo.toml`. Two humans on different laptops
already resolve via **merge queue + disjoint crate paths** (Google:
small packages so BUILD files rarely collide; TAP mid-air collisions
are semantic, caught on the linearized snapshot). UUID fragments are **YAGNI** if workspace `members` are **globs** over
D-8 faces (D-39). They are not a general clone-the-file protocol and
not a second VCS (`.delta`). Prefer glob; fragments only if a glob
cannot express the add.
Same-machine N agents: shared **read-only** buck2 cache (D-34),
separate worktrees, no shared cargo `target/` lock. Different
machines: git + merge_group is the bus; do not invent a fragment
message bus. Dispatcher assigns disjoint crates **before** spawn —
that is the waste-avoidance, not a smarter fold.

**MUST (fragments + one fold)**

- **achieves:** N agents add to shared config without touching the same
  git path; one mechanical fold; no yaml/json farm; lockfile has one
  writer at fold.
- **origin:** N-way `Cargo.toml`/`Cargo.lock` and root compatibility edits;
  300-line split does not apply; founder: uuid clones + pre-commit resolve.
- **rule:** keep file-config minimal; implement agents do not in-place
  edit the denylist; additions are uuid fragments; fold is one Rust
  serial step on the receiving branch; root compatibility stays
  single-writer; no Markdown fragment or prose sidecar; lock is regenerated
  once after fold; no whole-file uuid clones.
- **ensure:** review/presubmit reject in-place denylist diffs on
  implement PRs; fragment schema is closed; leftover fragments after
  fold fail; no `specs/` resurrection.
- **overturn_when:** a five-field ADR names another single-writer
  merge-driver that still keeps additions on unique paths and fails
  closed on semantic conflict.

### D-38 — Worktrees are not integration; star into `dev`; quarantine the identity

Worktrees stop two processes sharing one `.git/index`. They do **not**
make two diffs commute. Mesh-merging live worktrees (A into B into C
while agents still write) is how conflicts take hours and then grow:
each resolution is a moving target (trunk-based: the cost lives at a
**stale** merge boundary, not inside the branch).

Google/CitC: no git worktree mesh. CLs are patches on Piper HEAD; TAP
tests the projected submit; mid-air collisions are **semantic** and
caught on a linearized snapshot. GitHub/Trunk merge queues: test PR
against `dev` **plus** what is ahead in the queue — not against a
sibling feature branch. Hyperscaler rule: **never merge trunk into the
lane; never merge lanes into each other.** Replay the lane onto trunk
(rebase / queue).

**Topology (star, not mesh).**

```
lane worktree ──PR──► merge_group ──► origin/dev
lane worktree ──PR──►      ▲
lane worktree ──PR──►      │
```

Forbidden: `git merge other-agent-worktree`, `git merge origin/dev` into
a live implement branch, a “integration worktree” that sucks in five
lanes while they keep coding.

**When a conflict appears.** Constant-work / bulkhead: **quarantine that
identity** (the **file/Item**, D-39 — not the whole crate). Writers to
*that* path stop or restack **one** lane onto current `dev`. Other files
in the same crate **continue**. Global stop is the N-tax. Continuing to
implement on the conflicted **item** is how you get a second hour of
conflicts. If a git conflict happened, assignment was not path-disjoint
(failure), not “need a better merge.”

Worktrees stay **optional isolation** (same-machine index, D-31). If a
lane is sequential on one crate, one worktree is enough. Parallelism
is disjoint PRs into the queue, not N checkouts that later have to be
hand-folded.

**MUST (star integration)**

- **achieves:** no hour-long mesh merges; implementation does not halt
  globally; conflict debt cannot compound on a path still being written.
- **origin:** worktrees treated as the integration strategy; resolving
  while agents kept writing the same files.
- **rule:** integrate only lane → `dev` via merge_group; no live
  worktree↔worktree merges; no merge-trunk-into-lane; on conflict,
  pause writers to that identity only; other disjoint lanes continue.
- **ensure:** dispatch/review reject inter-lane merges; overlapping
  crate assignment is a dispatcher fail, not a merge-hero task.
- **overturn_when:** a five-field ADR names another integration topology
  that still forbids mesh merges of live writers and still tests the
  combination before `dev` moves.

### D-39 — Identity is a unique git path (file/Item), not a crate lock or `.delta`

**Adversarial on D-32/D-37/D-38.** “One writer per crate, others wait” is
a mutex with polling. “`.delta` / uuid clones” is a second version
control. Neither meets “conflict is structurally impossible under N
writers,” including **N on the same crate**.

Git conflicts iff two commits mutate the **same path** (or the same
hunk of that path). Therefore the only structural impossibility is:
**each parallel mutation owns a unique path.** Crate directories are too
big; uuid sidecars invent paths that are not the code.

**Decomposition (this is the grain).**

```
crate → module files → one primary Item per file
        (fn / struct / impl / trait / enum — RFC 430 names)
```

D-35 (≤300 lines) exists so one file ≈ one Item. N agents on crate P
= N files under P (`blob.rs`, `quota.rs`, …). `git merge` of those
paths cannot conflict.

**Parent lists must not be hand-edited.** `mod blob;` in `lib.rs` and
`members = ["storage/adapters/blob-sqlite"]` in root `Cargo.toml` are
the remaining mutexes. Make membership a **pure function of the
directory**:

- Workspace: Cargo globs over closed D-8 faces
  (`storage/adapters/*`, `app/*/core/*`, …). Adding a crate is
  `git add storage/adapters/foo/{Cargo.toml,src/lib.rs}` — unique
  paths; root toml does not change.
- Crate modules: Buck `srcs = glob(["src/**/*.rs"])` already. Cargo
  still wants `mod`. **Generate** `src/mods.generated.rs` (sorted
  `mod x;`) from the directory at the **one** merge_group snapshot
  (same class as generated faces). Agents never edit it. `lib.rs` is
  `include!("mods.generated.rs");` plus the crate’s tiny public API
  (itself one Item file if it grows).

No `.delta`. No uuid object store. The new file **is** the commit.

**Honest limit.** Two agents editing the same `fn foo` in the same file
**will** conflict. That is not solved by OT, mergiraf, or hope. Occupancy
is **that file/Item** on open PRs. Agent 2 is dispatched to a **different
Item/file in the same crate**, immediately — not parked on a crate lock.

Root compatibility Markdown stays one writer. Owner Markdown does not split
like code because D-36 forbids it; proposals remain off-tree and current truth
changes at semantic native paths.

**MUST (path identity)**

- **achieves:** N-way work on one crate without crate mutex, without a
  sidecar VCS; git conflicts become assignment bugs.
- **origin:** crate-lock and uuid-delta both failed the founder bar;
  crates already decompose into files and Items.
- **rule:** commute identity = unique git path at native module/Item grain;
  membership = directory glob + generated mod list;
  no `.delta`; no Markdown sidecars; no whole-crate lock; same-Item dual write
  refused at assign; N disjoint files in one crate is allowed and expected.
- **ensure:** workspace members globs match D-8; generated mod list not
  hand-edited; occupancy matches PR file paths; review rejects crate-
  wide locks and uuid-delta dumps.
- **overturn_when:** rustc/Cargo load modules by directory without
  generated `mod` (then drop the generator) or a five-field ADR names
  another unique-path membership that is not a second VCS.

### D-40 — Path-sets, mixed ops, off-tree proposals, cross-owner

**Is a session locked to an app/capability?** For **merge: no.** Commute
is path-sets (D-39). For **blast radius / sandbox: default yes** (D-31
cone). For **contracts: escalate** (D-29). Those are three different
knobs. Conflating them produced crate-locks and “agent owns `storage/`.”

An agent’s dispatch is a **path-set** `S` of AUTHORED paths (see the
D-40 amendment below). Spawn iff `S` is disjoint from every open PR’s
authored path-set (rename occupies `{old, new}`). The set
may list files in more than one cap **only** when the dispatcher names
them (found a correction; D-29). The agent still cannot self-widen.

**Mixed ops (all at once, N-way).** Git only cares about paths.

| A \ B | new unique file | edit F | delete F | `git mv` F→G |
|---|---|---|---|---|
| new unique file | commute | commute | commute | commute if dest ≠ new |
| edit F | | **no** (same Item) | **no** | **no** |
| delete F | | | idempotent / **no** | **no** |
| `git mv` F→G | | | | commute iff `{F,G}` ∩ `{F',G'}` = ∅ |

Refactor = `git mv` + edits of **callers**. Occupancy = old + new +
every caller file touched. If that set is huge, it is a **large-scale
change** (Google LSC / Rosie): **one** mechanical lane, or N agents
each given a **disjoint shard of caller files**. Do not N-way edit the
same caller. Do not run a feature lane on files an LSC already occupies.

Write + refactor + move + delete **in parallel** is allowed **and
expected** when those path-sets do not intersect. That is structurally
conflict-free. Same-path mix is an assignment bug, not a merge skill.

**Proposal vs implementation.** A proposal or plan in a PR body or external
work system has no tracked owner path and cannot become current authority by
narration. Accepted truth changes only by editing the applicable semantic native
surface and therefore occupies that native path. Two lanes changing the same
trait, proto, Cedar fragment, IR type, test, build declaration, or root
compatibility file conflict and serialize on that path; they do not create prose
sidecars. A correction in another cap is a new dispatch with those paths; the
original lane keeps its set.

**Scenarios (experiment).**

1. N writes of new Items in one crate — unique files, glob membership —
   commute.
2. N edits of disjoint files, plus one `git mv` of a third file —
   commute.
3. Edit F while another lane moves F — **forbidden at assign.**
4. Extract fn from F.rs into G.rs while another lane edits H.rs —
   commute; if they both edit F.rs — **no.**
5. Delete dead F.rs while someone still edits F.rs — **no.**
6. Mechanical rename of a trait across 40 files — LSC lane owns those
   40, or 40 shards with disjoint files; feature work on those files
   waits **on those files**, not on the capability.
7. Off-tree plan/review proceeds while N native crate paths implement — no
   tracked collision and no current-truth effect until native projection.
8. Two lanes edit the same port trait or root compatibility file — serialize on
   that path; do not split the decision into Markdown blocks.
9. Foundry agent finds a `storage` bug — dispatcher opens a **storage
   path-set** lane; Foundry lane does not absorb `storage/` (D-31 no
   self-widen). If it is a **shared port** shape, that is D-28/D-29,
   not a silent extra file.
10. Session “locked” to `app/foundry` as sandbox, but occupancy is
    `app/foundry/core/ontology/src/property.rs` — another Foundry agent
    on `pages.rs` runs now.

**MUST (path-sets)**

- **achieves:** mixed N ops without crate/cap locks; plans commute with
  code; cross-owner corrections are named dispatches; LSC does not
  collide with features on the same files.
- **origin:** cap-session lock and crate-wait were mutexes; founder
  asked mixed write/refactor/move/delete, propose/amend/correct, and
  whether cap lock is necessary.
- **rule:** occupancy = path-set (mv occupies both ends); disjoint
  sets commute including across caps; cap cone is default sandbox not
  merge law; proposals/work stay off-tree and current amendments occupy
  semantic native paths; LSC is one lane or file-sharded; no poll-lock;
  no self-widen; no `.delta` or Markdown sidecar.
- **ensure:** dispatch records the path-set; overlapping PR path-sets
  are not spawned; review rejects cap-wide locks and un-named cross-cap
  writes.
- **overturn_when:** a five-field ADR names another occupancy grain
  that still makes same-path dual write unspawnable without a sidecar
  VCS or a capability mutex.

#### D-40 amendment (2026-08-30) — the grain is AUTHORED paths

- **achieves:** structural lanes spawn concurrently again, while a
  same-path dual write over content a lane actually wrote stays
  unspawnable. This is the other occupancy grain D-40's `overturn_when`
  invites, not a relaxation of disjointness.
- **origin:** the grain was every changed path, including paths this
  repository had already declared concurrently editable. `.gitattributes`
  assigns `Cargo.lock` a structural `merge=cargo-lock` driver *because*
  "package sections can be added, removed, or version-replaced by
  independent branches". Every capability lane births or renames a crate,
  so every lane rewrote the lockfile, so every structural lane refused
  every other and the driver written to combine them never ran. Four
  lanes were wedged simultaneously on that one path; the gate made its
  own declared remedy unreachable.
- **rule:** the occupancy path-set is a lane's AUTHORED paths. A path
  carrying one of this repository's own merge drivers in `.gitattributes`
  is excluded: the driver is a standing statement that concurrent edits
  over it are expected. That is intent, not a guarantee — registration is
  per-clone and the lockfile driver exits 1 on same-package divergence —
  so what the exclusion buys is that the hard case fails loudly at merge
  instead of refusing both lanes at spawn. `merge=` alone does not
  qualify: git's own `merge=binary` declares a conflict rather than
  combining, so the drivers are an allowlist. Disjointness over the remaining paths is unchanged,
  and a shared authored path remains an assignment rename (D-41), never
  a queue. Occupancy does not reimplement gitattributes matching: a
  non-literal `merge=` pattern fails the gate closed rather than being
  guessed at.
- **ensure:** admission tests fix both directions — two lanes sharing
  only the lockfile both spawn; two lanes sharing one source file are
  still refused — and the facade call site is frozen so reverting to a
  raw all-paths admit fails a test rather than passing silently.
- **overturn_when:** a five-field ADR shows a `merge=` declaration that
  does not in fact make concurrent edits combine deterministically, so
  that excluding its path admits a collision the driver cannot resolve.


### D-41 — Simplest commute: stable indexes + unique files + queue

**Challenge.** D-37 fragments, D-39 committed generated `mods`, D-40
occupancy-before-spawn are still machinery. Compile-time `mod` generation
**committed at merge_group** fails locally (agents need to compile) or
**conflicts** if they commit it. Occupancy against GitHub is not
structural (two laptops can still `add src/put.rs`). Cap cones do not
make git commute.

**First principle.** Two commits conflict iff they change the same git
path. Therefore:

1. **Never edit the indexes.** Workspace `members` are **globs** over
   D-8 faces (Cargo already supports `crates/*`). Crate `lib.rs` has
   **one stable line** that includes modules by **scanning `src/items/`
   at compile time** (tiny owned `build.rs` / buck2 genrule writing
   `OUT_DIR`, not a tracked file). Agents never touch `lib.rs` or root
   `Cargo.toml` for adds.
2. **New work is a new unique file** (`src/items/quota.rs`, or a new
   globbed crate dir). The file **is** the commit. No `.delta`.
3. **Edit / delete / `git mv`** only those files. `git mv` needs a free
   destination. Callers you actually rewrite are extra paths — if that
   set is large, it is one mechanical LSC PR, not N overlapping edits.
4. **Integrate** PR → `origin/dev` → merge_group. Star, not mesh (D-38).
5. **Jail** writes to the dispatched files (narrower than a cap). Cap
   is a default hint, not a lock.
6. **Proposals** stay in the PR body or external work system and occupy no
   tracked owner path. Accepted truth occupies the semantic native path it
   changes; same-path writers serialize.

**Do not build:** occupancy JSON; uuid fragments; committed generated
mod lists; crate/cap poll-locks; automod from crates.io unless owned.

**Uncoordinated same-name create** (`put.rs` vs `put.rs`): Google TAP
mid-air — cheap, one file. Merge queue rebases the loser; they rename.
Do not uuid-name every module to prevent a rare event.

**How we achieve it (instruction → automation → presubmit).**

- Convention: new Items only under `src/items/` (or a new globbed crate
  directory). `lib.rs` / root members lists are denylist for implement
  PRs (same class as D-37 denylist, without fragments).
- One owned compile-time scanner per crate (or shared tiny codegen in
  `build/`) when a crate has more than `lib.rs`. Until the second file,
  YAGNI — stay in `lib.rs` under 300 lines.
- Workspace glob in the structural wave (`#2221` or immediately after).
- Write-jail = path-set. merge_group as today.
- Dispatcher *looks* at open PR files (no store). If overlap, pick
  another filename **now**, do not wait on a lock.

**MUST (stable index)**

- **achieves:** N mixed ops with no sidecar VCS, no mutex, no generated
  file in git; git conflicts only when two lanes actually share a path.
- **origin:** fragments and occupancy were solving coordination with
  product; founder asked simplest first-principles.
- **rule:** membership is glob + compile-time directory scan; new work
  is a new unique file; implementers do not edit parent indexes; jail
  the path-set; PR to `dev`; no `.delta`; no crate/cap lock; same-path
  dual write is an assign rename, not a queue.
- **ensure:** admission rejects implement diffs to root members lists
  and hand-maintained `mod` inventories; `src/items/*.rs` adds do not
  change `lib.rs`.
- **overturn_when:** rustc modules-from-directory lands and drops the
  scanner, or a five-field ADR shows a smaller rule that still keeps
  parent indexes stable.

### D-42 — Cross-harness: only git, draft PR, and presubmit travel

50 agents on five vendors do **not** share a dispatcher, Landlock,
CitC, or even a worktree. Cursor often shares one checkout; Claude
may lazy-worktree; Codex sandboxes; Grok uses isolated worktrees.
Antigravity will differ again. **Worktrees and sandboxes are
harness-private. They are not Oyatie law.** D-31 is a *local recipe*
for a Grok-class isolation, not `required_sequence` for every tool.

**What still holds (harness-agnostic).**

- Commute = disjoint git paths (D-41).
- Parent indexes stable; new work is a new file.
- Every lane opens a **draft PR against `origin/dev` as soon as it
  has a path** — that is occupancy every other harness can `gh pr
  list` / GitHub files API. No JSON board.
- `presubmit` / merge_group is the only combination test. Denylist
  and 300-line cap fire on the PR, not in the vendor.
- Instruction on **every** session hub (`AGENTS.md`, `CLAUDE.md`;
  Cursor/Codex overlays must point here, not fork law).

**What does not hold unless you stop believing the vendor.**

- Physical write-jail — optional, per harness. Assume the agent *can*
  edit `iam/` anyway; admission + review catch it.
- “Dispatcher assigned disjoint names before spawn” — there is no
  one dispatcher across Grok+Cursor. Two uncoordinated `put.rs`
  creates: merge_group rebase, rename. Waste is **one file**, not a
  crate. Accept it (TAP mid-air). Do not uuid-name modules.
- Ten Cursors in one clone without worktrees: `index.lock` / mixed
  diffs. That is a **local** problem: those ten need worktrees or
  they are one writer. Other harnesses on other machines are fine.
- 50× `cargo nextest --workspace`: laptop tax. Instruction: buck2
  `-p`/target. CI still one cargo snapshot. You cannot make Codex
  run buck2 if it ignores AGENTS — that is operator, not law.

**50-wide merge queue.** Semantic collisions (two PRs, different
files, tests red together) grow with N. That is why the queue
exists. Do not serialise spawn to avoid it.

**MUST (portable protocol)**

- **achieves:** five harnesses, one merge story; no vendor lock-in
  of isolation.
- **origin:** N agents on Grok/Claude/Codex/Cursor/Antigravity do
  not share a sandbox.
- **rule:** portable surface is git + draft PR on `origin/dev` +
  `presubmit`; law on every hub; do not depend on vendor jail or a
  cross-harness dispatcher; same-path create is rebase.
- **ensure:** denylist/file-budget fail regardless of authoring
  tool; hubs do not fork D-41/D-42.
- **overturn_when:** a five-field ADR names another portable
  occupancy that is still git (not a new sidecar VCS).

### D-43 — N-parallel delivery: launcher assigns paths; PRs are the pipeline

This is the acquire → plan → red → green → slop → CI → queue loop,
**N-wide**. It is not a Kanban JSON and not “agents browse a list.”

**Honest limit.** Thought-experiments on commute (D-38…D-42) are
law. The four-harness collision lab was **not** executed to merge.
Do not claim that loop is empirically green.

**Acquire is the launcher, once.** The accepted off-tree work package and
exact-revision native dependency/ownership surfaces name **output paths**
(`storage/adapters/blob-sqlite/src/items/put.rs`). A spawn script (human or
one coordinator process) gives each harness one path-set. The agent does not
poll issues, `gh pr list`, or an in-tree task board. Grabbing is `argv`, not a
mutex. Second spawn with an overlapping path is a launcher bug; presubmit
path-intersect is the later backstop (D-42).

**One PR per path-set** (usually one harness). Cross-harness =
separate PRs onto `dev`/`main`, never a Graphite stack across
vendors. Subagents inside one harness share that PR unless the
vendor invents branches by itself.

**Stages on that PR** (same path-set; do not open a second PR per
stage). Other PRs run the same stages **at the same time**.

| Stage | What | N-parallel? |
|---|---|---|
| Valid | Path in D-8 grammar; not denylist; not intersecting open PR files | Launcher + presubmit |
| Propose | PR body or external work system only; semantic current-surface names, with sequential ids permitted only as transitional provenance | Yes, different items |
| Red | Tests for **this** item as unique files | Yes |
| Review tests | PR review (≠ merge APPROVE) | Yes |
| Implement | Fill the item file | Yes |
| Review / de-slop | Same PR | Yes |
| Coverage | More tests in **this** crate only | Yes |
| Pipeline/CI review | **Only if** `.github/` or `pipeline/` ∈ path-set. Metrics: wall-clock, queue depth, duplicate jobs vs last `dev` presubmit — skip if the PR is `src/items/` | Rare; serialize if they touch the same workflow file |
| Diff + merge review | Required APPROVE | Per PR |
| presubmit | fmt/clippy/nextest (touched + workspace as today). Red = **this PR’s** process error. Do not stop other PRs | N in the queue |
| merge_group | Combination on advancing `origin` (D-38). Agent does not rebase-loop | One snapshot at a time, many PRs waiting |
| squash | Lands | |

**Local pre-push** is `cargo fmt` on staged `*.rs` only. It cannot
catch workspace nextest; claiming it should is false on this graph.
presubmit catches. If presubmit is red on fmt/denylist, the agent
skipped the cheap local step. If red on full nextest, that is the
merge proof, not a pre-push miss.

**CI must be green to merge.** If it is not, that PR is wrong — not
the factory. Other path-sets stay in the queue.

**Forbidden:** `tasks/` board; agent wait-until-unlock; one global
plan review gate; every implement PR reviewing CI throughput;
mesh-merge of stages across PRs; new ceremony markdown per stage.

**MUST (N-parallel loop)**

- **achieves:** many PRs in plan/red/implement/review at once;
  acquire without polling; CI red is local to a PR.
- **origin:** a 19-step serial ritual plus a grab-list would recreate
  locks and GooWiki.
- **rule:** launcher assigns unique path-sets from an accepted off-tree package
  plus exact-revision native surfaces; one PR
  carries plan→tests→impl→slop→coverage; pipeline/CI review only
  when those files are touched; presubmit/merge_group/squash;
  pre-push is fmt-touched only; no tracked design/plan Markdown, task board,
  or factory-stop on one red PR.
- **ensure:** path-intersect check; denylist; no `tasks/` directory;
  workflow diffs require CI-metric look, src-only PRs do not.
- **overturn_when:** a five-field ADR names a smaller loop that still
  assigns unique paths without agent-side wait.

### D-44 — Operator interview is the only intake; Product xor Program

**Who receives client directions?** Not implementers. Not the merge
queue. The **human operator** working with the orchestrator.

Client needs arrive **ambiguous and sometimes wrong**. Passing them
down the factory is how you get N agents building the wrong shape.

**Gate (fail closed, before any slice).**

1. Interview until the need is unambiguous (no TBD / empty acceptance).
2. Research the exact immutable revision's native artifacts and, when useful,
   an untracked D-27 view; cite semantic paths and the immutable revision.
3. Realistic evaluation: layout (D-8 faces), YAGNI, blast radius,
   what already exists. Dump-root asks (`plan/`, `libs/`, …) are
   **Rejected**, not clarified into existence.
4. Emit an **ephemeral off-tree artifact package** in the PR body or external
   work system; never add a planning/design Markdown path to the product tree.
5. Handoff is a function of paths: `app/` → **Product**; capability
   root → **Program**. Mixed packages split. No owner → no package.

Product/Program decompose the package into path-sets. Only then do
role hops exist.

**MUST (intake)**

- **achieves:** wrong or vague client text cannot occupy implement
  capacity; Product vs Program is deterministic.
- **origin:** prompt-to-implement skipped clarification and built
  dump folders; founder required interview artifacts and a Product
  or Program handoff.
- **rule:** operator+orchestrator interview, cite, verify; fail
  closed on ambiguity or forbidden shape; ephemeral package; Product
  xor Program; never raw text to implement.
- **ensure:** admission of an implement hop requires a packaged
  handoff; mixed app+cap packages fail; dump-root packages reject.
- **overturn_when:** a five-field ADR names another intake that still
  stops wrong/ambiguous needs before path occupancy.

### D-45 — Lanes are deterministic; orchestrator does not spawn or fold

D-43's occupancy (one PR per path-set) stands. The implication that
**one agent walks plan→red→implement→coverage** on that PR is
**OVERRULED**. That is a serial mega-agent. Ten implement-ready
disjoint slices bound to one implementer is a fold. The whole
pipeline waiting on that implementer is a bottleneck.

**Ready hops are a pure function** of (slices, completed roles,
in-flight, path occupancy). Anvil publishes them. Harnesses bind a
**fresh agent id** per claimed hop. The orchestrator does not spawn
vendor CLIs for those hops.

Roles form a **DAG with fan-out**: after Implement, review, coverage,
security, native-knowledge/view validation, and box tests become ready
**together**. Completing Implement on slice A unblocks A's successors **and** frees the
implement lane for slice B.

**MUST (lanes)**

- **achieves:** N-wide role trains; no fold; no spawn bottleneck in
  the orchestrator.
- **origin:** one implementer looping disjoint work; serial stage
  enum; founder: tasks and lanes are deterministic; 1-of-10 is
  wrong.
- **rule:** ready-hop count for a role is the number of disjoint
  schedulable slices in that role; no agent-id reuse; orchestrator
  publishes, launchers bind; fan-out after Implement; not one
  sequential pipeline.
- **ensure:** fold (k < N bound agents for N ready hops) fails
  closed; reused agent ids fail; tests prove fan-out cardinality.
- **overturn_when:** a five-field ADR names another scheduler that
  still forbids folding a lane onto one long-running agent.

### D-46 — Foreign-path need is a draft on *your* files, not a committee

When something breaks, or a downstream team needs a contract change
on a slice they **do not occupy**: they **do not write those files**
(D-40). They do not open a ticket. They do not wait on a lock.

They add `ports/draft/<port>/` or `adapters/draft/` **on their own
path-set** (D-28). That commutes with the owner. The owner receives
a `ContractAmend` hop when **their** paths are free. Agreement is
`git mv` of the draft onto the provider (D-28), under D-29
escalated review — not a standing committee.

A red presubmit / merge_group **quarantines that path-set** (D-38).
Other disjoint slices **continue**. Global stop is the N-tax.

**MUST (amend without bureaucracy)**

- **achieves:** N-parallelism survives contract lag and CI red;
  unagreed shapes stay grep-visible.
- **origin:** escalation boards and "please change your API" waits
  serialized the factory; founder asked minimum bureaucracy.
- **rule:** no occupancy ⇒ no write; consumer drafts on owned
  paths; owner amends on owner paths when free; red is local to
  the path-set; no `tasks/` board.
- **ensure:** cross-owner writes without occupancy fail; draft
  paths contain `draft`; one red PR is not a factory stop.
- **overturn_when:** a five-field ADR names another settlement that
  still keeps foreign writes unspawnable and does not add a queue
  product.

### D-16 — `console/` is not a capability; discard the pilot

The tree at `console/` is **ops-dashboard-control-center**: a Wave-15 internal
ops dashboard (incident-command, tenant-admin, pack-author, on-call-handoff, …).
That is a **pilot product**, not a cloud engine.

Hyperscaler analog: AWS Console / GCP Console is a **first-party web app** that
dogfoods IAM and the public APIs. It is not EC2. It is not a closed-registry row.
ADR-0615 “console SHELL is the substrate; ops leaves → `app/ops-console/`” mixed
the shell with the pilot and invented a parking lot.

**Burn.** `git rm -r console/`. No empty `console/` scaffold. Do **not** park the
pilot in `app/ops-console/` or `app/console/`. Token broker is **iam**. Operator
mutations stay on each capability’s **facade** (Cedar). A future tenant/operator
UI is `app/` **after** the apps discussion, as tenant #0, same public APIs.

**MUST (no console capability)**

- **achieves:** a dashboard pilot cannot occupy a cloud-provider root.
- **origin:** `console/` README is ops-dashboard-control-center; registry called it
  the Leptos shell / token broker.
- **rule:** `console/` is absent from the tree and the closed registry; no empty
  scaffold; no `app/ops-console` rehome of this pilot.
- **ensure:** membership/hygiene allowlists do not re-admit `console/`; new UI
  waits for the apps discussion.
- **overturn_when:** a §7 ADR adds a real sold shell as `app/` (or a new cap) with
  five fields same-wave — not a resurrection of this directory.

### D-17 — Presubmit is the graph, not a JSON product

**Executed (this wave):** `git rm` of `specs/` as a living-law corpus (including `root-hub-pointers.json`, `integ-branch-envelopes.json`, `masterplan.json`, `cedar-policy-schema.json`), `registry/`, `evidence/`, `governance/` (including `capability-registry.json` and `check/`), `ci.toml`, `pipeline/facade` census crates, Tide/GateRun/process-kit/webhook-gateway, and the `libs/check-*` + `libs/governance-*` fitness farm (except library kernels `check-cost-budget` and `governance-eval-domain`). D-8 unknown-root names live in `pipeline/core/admission`. Root compatibility entry is `AGENTS.md` + `CLAUDE.md`; D-36 makes semantic native owner surfaces current and freezes owning ADRs as migration input. No replacement JSON hub.

`pipeline/facade/*` (was `ci/facade`) and `governance/check/*` grew a **second
product**: JSON policy files, frozen path lists, FNV signatures, Helm/OpenAPI
parity. Hyperscaler TAP is **build + test the graph**. It is not ten crates
that observe the tree. Trimming one stale JSON keeps the anti-pattern.

**Merge proof (the TAP).** `cargo fmt --all --check`; clippy `-D warnings`
when clean; **`cargo nextest`** (compile+test once). That is the product’s
presubmit for tenant #0. Not `cargo check`. Not libtest. Not a JSON census.

**Still allowed as graph steps** (code in the execute graph, **not** a
governance JSON product). Default is delete. Keep only if it is a **pattern**
(new unauth HTTP, new GraphQL crate, new license, new non-Rust automation,
hand-edit of generated faces, D-8 unknown root name):

| Step | Why it is not census |
|---|---|
| One license/ban engine (`deny.toml` + weekly `cargo deny`, not a crate) | Legal. |
| D-8 unknown-name (`pipeline/core/admission`) | Closed root set. Fails on **unknown names**, never `expected_total`. |
| D-35 file budget on **touched** non-exempt files | Pattern: this file is too long. Not a frozen count of files. |
| First-party Cargo↔BUCK source-declaration conformance (one Build-owned, versioned engine) | Pattern: a changed source declaration is outside the admitted relation. Direct parsing of a closed grammar, never a frozen package/path census. |

**First-party Cargo↔BUCK source-declaration conformance — adopted 2026-08-28.**
This is exactly one narrow exception to the default-delete rule above: a
Build-owned, versioned, corpus-free engine. This amendment admits that engine;
it does not claim that implementation, Pipeline integration, qualification, or
enforcement has landed.

**Inputs, grammar, and relation.** Pipeline acquires immutable base and head
snapshot bytes through its SCM port, resolves owners, and supplies those bytes,
snapshot identities, and ownership facts to the Build engine. Git is
Pipeline's required current SCM adapter, not a Build-core dependency or the
only future SCM. The engine neither invokes Git nor resolves owners. Cargo and
BUCK syntax libraries are maintained Rust dependencies behind parser ports.
The engine never grows a hand-written Starlark parser or interpreter. The
closed grammar profile binds each parser's identity and version plus every
admitted prelude, macro, and rule contract identity.

A change to either declaration surface triggers evaluation of the complete
HEAD first-party declaration graph over one closed, versioned Cargo/BUCK
grammar and its unconfigured source IR. Base/head deltas are attribution and
repair-sharding inputs only, never the correctness boundary. Every admitted
first-party BUCK edge must resolve to a unique declared identity in the
admitted unconfigured source IR and be permitted by the applicable Cargo
declaration. Cargo-to-BUCK coverage applies only to the participating target
and dependency kinds explicitly modeled by the Build owner SPEC. It does not
require naive package-wide set equality: test and binary rules may legitimately
consume subsets. Normal, build, dev, optional, target-specific, and path
dependency semantics must map into that relation; an unsupported, unmapped,
malformed, or ambiguous shape refuses rather than returning a false green. This
is source-declaration conformance, not a claim about Buck2's configured graph
and not a Cargo or Buck2 compile proof. The first-party scope explicitly
excludes `third-party//` and generated `third-party/BUCK`; those remain in
Reindeer declaration reconciliation, not this engine.

The default grammar refuses unknown or unadmitted target- or
dependency-affecting loads, macros, mutations or reassignments, control flow,
comprehensions, selects or other configuration forms, expressions, label or
cell forms, and Cargo semantics. Each remains refused until the Build owner
SPEC explicitly admits and qualifies its exact source form. Simple loaded rule
symbols, constants, and list concatenation are no exception: they are
admissible only when that SPEC admits and qualifies them. Any later syntactic
admission remains an unconfigured source relation; it does not evaluate
configuration. Inability to prove that an unknown construct cannot influence
target identity or dependencies is itself a refusal.

**Outputs and effects.** The engine emits sorted typed violations and
exactly one canonical, deterministic, non-mutating `DeclarationRepairSetV1` per
evaluation; the set may contain zero repair actions. It binds the engine
identity, source-snapshot identity, admitted grammar-profile identity (including
parser, prelude, macro, and rule-contract identities), and the caller-supplied
owner-authority identity and ownership facts. It declares the complete semantic
read set, complete semantic write set, and complete proposed-write path set.
Every bound semantic read, semantic write, and proposed-write path carries a
digest-or-absence precondition and the exact expected owner identity or expected
owner absence. `OwnerExpectation::Absent` is valid as an ownership CAS fact only
for a non-write semantic read; its use on a semantic write, proposed-write
action, or proposed-write path refuses. Every proposed-write action and path
must otherwise resolve to exactly one concrete expected owner.
The set carries deterministic complete postimages and typed postconditions and
binds canonical digests for every postimage and owner-group output plus a
whole-set digest over the canonical encoding of every other field. That
whole-set digest is its canonical whole-set identity. Its owner groups are
exactly the non-empty groups induced by the distinct concrete expected owners of
its proposed-write actions, as bound from caller-supplied ownership facts.
Every repair action and proposed-write path appears in exactly one group, the
groups follow canonical owner order, their write sets are pairwise disjoint,
and a zero-action set has zero groups. Any of the following causes refusal: an
absent-owner proposed write; an empty, extraneous, missing, duplicate, ambiguous,
or wrong-owner group; cross-owner grouping; incomplete action or proposed-write
path coverage; or overlapping group writes. Snapshot identity is provenance,
not a repository-wide application lock. An unrelated disjoint successor commit
does not invalidate a repair when every declared semantic and owner-authority
precondition still matches. Any semantic read-set, write-set, proposed-write,
or owner-authority precondition mismatch refuses. Repeated evaluation of the
same immutable inputs produces byte-identical violations, grouping,
preconditions, postimages, postconditions, output digests, and whole-set
identity. The engine does not apply a repair, mutate the candidate, invoke
`buck2`, access the network, or spawn a shell or any other process. It creates
no frozen count, package/path list, census file, learned baseline, or gate fleet.

**Protected integration.** Pipeline later invokes the versioned engine from
ruleset-selected protected source inside the existing trusted layout admission
and feeds its verdict to the one existing `presubmit`. It does not add a
workflow fleet, required context, or standalone Cargo/Buck compile lane, and it
is not a second proof that either build system compiles the candidate. This
source-declaration relation does not revive the deleted
cross-artifact-agreement census. D-34's native Buck2,
`cargo metadata --offline --locked`, and rust-analyzer dispatcher graphs remain
unchanged; this engine neither replaces them nor stores their configured
results. Pipeline later validates each `DeclarationRepairSetV1` owner group and
maps it one-to-one to a canonical ChangeSet before application; it
never invents or regroups ownership. That orchestration is not Build behavior.

**Activation and qualification.** Initial enforcement activates only after
adversarial qualification proves both declaration-side triggers, complete HEAD
graph evaluation, delta-only attribution and repair sharding, legitimate target
subsets, every modeled dependency semantic, unique declared-identity
resolution, every admitted and refused grammar form, exactly one canonical
`DeclarationRepairSetV1` per evaluation including a zero-action set, complete
digest-or-absence and owner-or-absence preconditions on every bound semantic
read, semantic write, and proposed-write path, complete postimages and typed
postconditions, canonical postimage/output/whole-set digests and identity,
`OwnerExpectation::Absent` restricted to non-write reads, exactly one concrete
expected owner for every proposed-write action and path, absent-owner semantic
writes and proposed writes refusing, canonical ordering of exactly the non-empty
owner groups induced by distinct concrete expected owners of proposed-write
actions, exactly-once repair-action and proposed-write-path membership,
pairwise-disjoint writes, zero actions yielding zero groups, empty, extraneous,
missing, duplicate, ambiguous, wrong-owner, cross-owner, incomplete, and
overlapping-group refusal, mismatch refusal, and the no-side-effect boundary.
Out-of-presubmit differential qualification compares the engine with protected
`cargo metadata --offline --locked --no-deps --format-version 1` and
non-building `buck2 uquery`. Only the protected qualification harness invokes
those exact commands; the engine and required declaration check never do. A
change to the admitted grammar, Buck prelude, or rule contract or to a parser
or admitted macro identity/version produces a new grammar-profile identity and
mechanically requires requalification before enforcement resumes.

Qualification must also repair every violation the engine detects as legacy
drift on the then-current `dev`. There is no baseline, count, or violation
allowlist. A stale or missing label, duplicate declared identity, unsupported
relation or grammar form, false green, false failure for a legitimate subset,
nondeterministic result, incomplete read/write preconditions or postimages,
read/write mismatch acceptance, candidate mutation, process/network attempt,
unqualified grammar/prelude/rule-contract change, or unresolved legacy
violation blocks activation; after activation, the same declaration failures
refuse `presubmit`.

**Not needed** as crates, JSON dirs, or merge predicates: corpus-census,
planning-projection as a required check, cross-artifact-agreement as a
required check, endpoint-authorization-coverage (that is tests + PDP
fail-closed), graphql/crypto as standalone census crates (encode in
clippy/tests), `governance/check` fleet, `governance/corpus`, cap-root
`*-policy.json` path lists, Helm/OpenAPI/OpenSLO parity, Tide/webhook as
**product core**, Prow `GateRun` as Cloud Build.

`governance/` **is** needed for `capability-registry.json`, packs, and
Cedar the PDP compiles. It is **not** a CI product. `check/` crates that
are not a D-8 unknown-name step **REMOVE**.

**MUST (graph, not JSON gates)**

- **achieves:** merge is TAP execute and first-party source declarations stay
  conformant under one protected verdict, not a policy-file farm, configured
  graph oracle, or second compile plane.
- **origin:** census JSON and `governance/check` became the product while Cloud
  Build never shipped; recurring stale first-party labels survived until
  nonblocking weekly Buck smoke or manual repair without deterministic
  owner-sharded repairs.
- **rule:** presubmit is fmt + clippy + nextest plus the short pattern-step
  table. That table admits exactly one Build-owned, versioned, corpus-free
  first-party Cargo↔BUCK source-declaration conformance engine under the complete
  contract above: Pipeline supplies immutable snapshots and ownership facts,
  Build checks the complete HEAD graph with deltas used only for attribution
  and repair sharding, and Build emits typed violations plus exactly one neutral,
  canonical `DeclarationRepairSetV1` per evaluation, including a zero-action
  set, binding engine, source-snapshot, profile, and owner-authority provenance;
  complete digest-or-absence and exact owner-or-absence preconditions for every
  bound semantic read, semantic write, and proposed-write path;
  `OwnerExpectation::Absent` allowed only for non-write reads and refused on
  semantic writes or proposed writes; one concrete expected owner for every
  proposed-write action and path; deterministic complete postimages and typed
  postconditions; canonical
  postimage/output/whole-set digests and identity; and canonical ordering of
  exactly the non-empty owner groups induced by those concrete owners, with
  every repair action and proposed-write path in exactly one group,
  pairwise-disjoint writes, zero actions producing zero groups, and refusal of
  absent-owner proposed writes and empty, extraneous, missing, duplicate,
  ambiguous, wrong-owner, cross-owner, incomplete, or overlapping groups.
  Pipeline alone invokes the protected engine in existing trusted layout
  admission, feeds the one `presubmit`, and validates and maps each owner group
  one-to-one to its canonical ChangeSet without inventing or regrouping
  ownership.
  Path/count freeze JSON is not a gate; `governance/` is registry not CI; new
  check crates are born-blocking unless they are a tabled pattern step in the
  `pipeline/` graph.
- **ensure:** no new `*-policy.json` freeze, `governance/check/*` census crate,
  gate fleet, or GHA predicate this ADR deleted. Adversarial and protected
  differential qualification proves the closed grammar/profile, full-HEAD
  evaluation, first-party/Reindeer boundary, canonical `DeclarationRepairSetV1`
  provenance, read/write/proposed-write preconditions, postimages,
  postconditions, digests, identity, absent-owner semantic-write/proposed-write
  refusal, canonical non-empty induced owner groups, exact-once
  repair-action/proposed-write-path membership, zero-action/zero-group behavior,
  pairwise-disjoint writes, refusal of empty, extraneous, missing, duplicate,
  ambiguous, wrong-owner, cross-owner, incomplete, or overlapping groups,
  mismatch refusal, and forbidden side effects; profile identity changes
  mechanically requalify, and enforcement remains off until all detected legacy
  drift is repaired without an allowlist.
  Review preserves the D-34 dispatcher graphs and rejects Build-side Git/owner
  resolution, required-path tool invocation, a configured-graph claim, a second
  required context, or a second compile proof.
- **overturn_when:** one founder-accepted five-field amendment changes this
  combined TAP/declaration contract while proving one protected presubmit, no
  frozen corpus/count/fleet, no second compile plane or context, and equally
  fail-closed complete-HEAD declaration integrity with deterministic repairs.

### D-18 — `pipeline/` product vs GHA operator; purge `workflow/`; `.github/scripts` glue

**Product.** One execute engine: **graph + queue + schedule**. Analog:
Google **TAP** (internal) + **Cloud Build** (sold). Two **facades**, not
two codebases.

- **Internal:** this monorepo is **tenant #0**. D-10 cadences are that
  tenant’s graph. Same APIs a customer will call.
- **Sold:** tenant submits a graph; same schedule/queue.
- **Workers:** **`compute/` runs the work** (CH VM / Firecracker
  functions). `pipeline/` does not own a second cluster or a Prow job
  pool. Functions are one step type, not the only runner. Sold kube
  workers, if any, are `k8s/` nodes that **are** compute VMs.
- **Promotion / CD:** `pipeline/` **executes** the promotion graph
  (`dev` → `staging` → `canary` → `production`). `iac/` is desired
  state. `k8s/` is the cluster product. Images are `build/` artifacts in
  `storage/`. `cargo build --release` is a **CD graph step**, not
  presubmit.
- **Merge:** **one** required context: **presubmit**. GitHub merge queue
  is GitHub’s, via an adapter, then gone. No `presubmit`
  as a second protected check. No owned Gerrit/submit-queue in v1.
- **CAS + execute client:** When the cloud can **serve** `pipeline/`
  against `storage/` CAS and `compute/` RE, the execute client is
  **buck2** (hermetic **polyglot** action graph). The language set is
  **not closed**: C, C++, C#, Go, Java, Kotlin, Scala, Python, Ruby,
  JS/TS, Dart, Swift, Rust, Zig, Haskell, Assembly, R, SQL, Shell, and
  whatever else we need. Each is an **action + toolchain**, not a CI
  SKU. The sold graph IR is language-agnostic (D-3 protobuf). New
  language = `build/` toolchain (or tenant-provided tool in the graph),
  not a new cap and not a cargo/go/npm product. **Do not sell cargo.**
  **Do not** grow one SKU per language (the Actions-matrix anti-pattern).
  Tenant #0 is Rust-first **in this repo**; that is not the product
  contract. Cargo is not a second pipeline runtime. Keep cargo only
  where buck2 cannot for **this** repo (today: `Cargo.toml` + reindeer
  as crate **manifest** input to buck2; `cargo fmt` until buck2 owns
  format). rust-analyzer is not merge proof. Dual cargo+buck2 merge is
  forbidden.
  **v1** until that cloud is serving: cargo nextest is tenant #0
  presubmit (ADR-0716) because there is no live CAS+RE. **Overturn 0716
  same-wave** when pipeline **serves** the buck2 graph; cargo execute
  is then not needed. CAS up or weekly `buck2 build //...` alone does
  not overturn.
- **Not the product:** `.github/` GHA (adapter until the engine **runs**
  tenant #0); Tide/webhook-gateway as `core/`; Prow
  `GateRun` as `core/` (**REMOVE** — BUILD a clean graph+queue kernel;
  do not strangler GateRun into Cloud Build); JSON check fleets; a
  directory named `ci/`.

Today’s tree under `pipeline/` is **KEEP+WORK** at the **path** only.
Contents are not the product. BUILD `pipeline/core/` graph+queue.
REMOVE census, Tide-as-core, webhook-as-core, GateRun-as-core.

**MUST (one pipeline engine; GHA ≠ product)**

- **achieves:** sold TAP/Cloud Build cannot be Actions YAML or a JSON
  gate farm; internal and sold cannot drift into two pipelines.
- **origin:** `ci/` mixed census, GHA glue, Prow, and the name of a
  product we did not build.
- **rule:** capability is `pipeline/`; one execute engine (graph+queue);
  workers are `compute/`; promotion graphs are `pipeline/` execute, `iac/`
  desired state, `k8s/` the cluster; one required context `presubmit`;
  GateRun/Tide are not core; tenant #0 and customers share the engine;
  GHA is a temporary adapter; JSON/census/`governance/check` fleets are
  not the product; no root named `ci/`.
- **ensure:** no new crates under `ci/`; no new GHA glue outside
  `.github/scripts/` and workflow YAML; git mv of dumps does not make
  them `pipeline/core`.
- **overturn_when:** `pipeline/` runs tenant #0 and GHA is deleted
  same-wave, or a five-field ADR names a different sold slug.

**Operator GHA.** `.github/workflows` is a **temporary** merge path for *this*
monorepo. Completely **disjoint** from `pipeline/`: no YAML copied into cap
`core/`; no claim that GHA **is** Cloud Build. Cutover is when `pipeline/` can
run this repo’s nextest graph; until then GHA stays.

**Glue languages.** Rust-first still owns `scripts/`, `tools/`, `infra/`, and every
capability. **Exception:** self-contained files under **`.github/scripts/`** may be
shell, JS/MJS, Python, **Go, or any other language**. That keeps GHA glue out of
the product tree.

Self-contained means: no repo-root `go.mod` / `package.json` / `requirements.txt`;
if a module file exists it lives **only** under `.github/scripts/`; not a Cargo
workspace member; no shared crate with a capability; no `npm install` / `pip install`
/ `go get` on the presubmit path (pin a binary or a file in that directory). The
exception **dies with GHA**. It is not a license to grow `scripts/` or `tools/`.

**`bus/`.** Capability root is **`bus/`**. What goes here: owned **queue**
(competing consumers), **fan-out bus**, **seekable stream**, plus
transactional **outbox**. At-least-once; per-key order only. Analog:
Google Pub/Sub, AWS SQS+SNS, Azure Service Bus. Kafka/Pulsar are
**adapters** (or a sold protocol facade crate — not `core/`). Not the
serving path, not the outbox store, not mailbox, not sagas, not SES.
D-1: Check/IR/tuples are not a broker log. A directory named
`messaging/` is **REMOVE**, not an alias.

**MUST (owned bus, not Kafka)**

- **achieves:** async fabric matches cell-local serving; no consumer-group on Check.
- **origin:** Kafka-as-default is industry cargo-cult; hyperscalers sell MSK, they
  do not run Kafka as S3/IAM’s bus.
- **rule:** `bus/` core is the owned queue/bus/stream + outbox; Kafka/Pulsar only
  as `adapters/` or a sold facade crate; serving traffic is not a Kafka consume.
- **ensure:** no new `core/` crate named kafka/pulsar; workflow Kafka dumps stay
  deleted (D-18).
- **overturn_when:** a five-field ADR sells Kafka-protocol as `core/` with
  measured serving-path evidence.

**`notify/` not `comms/`.** Mailbox, Meet, messenger, calendar are **`app/`**
(D-22/D-23). Cloud `notify/` is **notification delivery** (email send, SMS,
push, messenger ping) — SES/SNS/FCM analog, not a mailbox. Apps raise
notifications through a notify adapter; mail as a *channel* is not `app/mail`.
**Purge** `comms/`. Rewrite `notify/` from that charter.

**`workflow/`.** Current tree is n8n/SaaS/bus/forms/tasks, not Step Functions.
**Purge** (`git rm -r workflow/`). Keep the **registry row** as the rewrite
destination. No empty scaffold. Do **not** strangler event-bus crates into
`bus/` from this junk — `bus/` already has kernels. Rewrite the
saga engine from the D-15 charter (proto/H3, studio as facade). Forms/tasks wait
for the apps discussion.

**MUST (GHA ≠ pipeline product)** — see D-18 one-engine MUST above. `workflow/`
implementation is gone pending rewrite; no dump resurrection.

## Rejected alternatives

- AWS EKS etcd journal as our store.
- A kubernetes.git port-engine as a hyperscale **operations** requirement.
  Google runs Borg, Meta Twine, AWS sells EKS without writing kube. Our
  fleet is `compute/`+`cell/`. Sold `k8s/` wraps **upstream**.
- Talos (Sidero) as the fleet node OS. Hyperscalers do not run Talos.
  Talos is a kube-node OS; at most a sold-SKU worker image.
- Asterinas or Hermit as **today’s** node kernel or a vacant `kernel/`
  rung “until it matures.” QEMU as the compute identity. Kubernetes as
  the cloud OS. Talos as the fleet OS.
- kube-rs / k8s-openapi as the fleet (Borg) control plane. Those
  libraries are a kube-apiserver client. Sold `k8s/` adapters only.
- Kata Containers as the fleet runtime (that is CRI/kube). gVisor as
  the fleet VMM (it is a container sandbox). Empty `kata/` / `gvisor/`
  roots. **`k8s-on-compute` as a compute reconciler** (sold kube is `k8s/`).
- Ontology kernel in `data/core`. Copilot/CLI as `intelligence/core`.
  Drive/Meet/PACS as storage facades. `build/` as a price list.
  Gateway (or nested cap) Cedar **engine**. Observability as the bill.
- Palantir Foundry as `data/` or as `intelligence/`. A `foundry/` **capability**
  root. Reviving intelligence/RAG “foundry.” `app/ontology` as a sibling of
  Foundry. D41-retiring docs/sheets/Pages/Grid.
- CUE+Timoni or Haskell as EaC wrap.
- Public JSON/REST as the destination codec.
- Standing gRPC (public or east-west) because a mesh automates HTTP/2,
  because middleboxes break HTTP/2, or because “binary/gRPC skips TLS.”
  Protobuf is already binary on Connect. TLS CPU is kTLS/ALTS-class, not
  plaintext and not a second RPC envelope. Leftover gRPC deletes.
- Istio/Linkerd/sidecar as SPIFFE identity.
- PQC/ECH only in prose (no `gateway/` TLS adapter crates). Classical TLS as
  the destination suite. ECH with no crate.
- On-path QUIC MITM, blocking UDP/443 on the **public** door, or turning
  ECH off so a NGFW can read SNI. Forcing QUIC/H3 **east-west** because
  the public door is H3 (DC plant is TCP-optimized). A `firewall/` cap.
- Forking Chromium / shipping Island-class browser as a cloud v1
  requirement so that endpoint DLP exists. Attestation is the port;
  their browser is the client.
- New `cloud-*` crates, `cloud/` root, or moving a dump into another cap
  because the current home is wrong. Wrong-home burns or rewrites; it
  does not change address.
- Zanzibar global tuple replica.
- EU GDPR as the sole compliance floor. EU as a country pack id. Combinatoric
  `packs/kr-eu` (or A×B×C×D pack ids) instead of a set on the record.
  Treating the tenant’s home country as the only pack.
- A pack-union engine beside Cedar. Fetching packs on Check. CaS as a
  DIY PDP for apps. DORA/CSAP as a per-RPC serving predicate. Encoding
  pack ids as ReBAC groups. Projection dimensions as free strings
  outside the Cedar schema.
- Mega EaC orchestrator / remote PDP on the hit path.
- Sync Merkle on every `Check`.
- Cap-root `catalog.yaml` / `manifest.json` as allowlist debt.
- Cap-root `policy/` for Cedar files (collides with the `policy/` engine) or dual `cedar/`+`policy/` children.
- Rehoming AUDIT-FINDINGS, IPs, scorecards, DPIA essays into `docs/` instead of deleting.
- Cap-root `contracts/` or root `contracts/*.yaml` as IDL SSOT.
- Hand-authored OpenSLO (clones or “unique”) as SLO source of truth.
- Standing REST/JSON transcode “until SDK is ready” (that is dual-stack debt).
- Hand-authored OpenSLO as a W0 source of truth.
- Branding the merge-blocking CI `ci-*` or adding one required check per capability.
- Keeping `specs/` as a JSON org-law tree (even “thin”).
- Keeping `HANDOFF.md` as a fourth root hub.
- Keeping `console/` as a shell/token-broker capability, or rehoming the
  ops-dashboard pilot to `app/ops-console/`.
- Trimming `cedar-deploy-parity-policy.json` (or any census JSON) instead of
  deleting the gate. Hand-maintained path lists, FNV signatures, `min_expected_*`,
  and `expected_total` are the same anti-pattern.
- `cargo check` as a CI job (nextest already compiles). `cargo test` (libtest) as
  the merge runner. Windows/macOS per-PR smoke. `rustfmt` CLI instead of `cargo fmt`.
- `cargo test --doc` as a vacant merge job; `cargo-semver-checks` on internal crates;
  network `cargo-audit` on presubmit; `cargo-dist` GitHub Release CLIs; whole-repo
  `cargo-mutants` nightly; a cron that runs `cargo update` into `Cargo.lock`; a
  local hook that runs clippy `--all-targets` and claims “<5 seconds.”
- **mold** on GitHub presubmit. Rust 1.90+ already uses bundled `rust-lld` on
  `x86_64-unknown-linux-gnu`; CI already has debuginfo off. Mold is a local Linux
  optional, not a merge-path pin.
- Treating `.github/workflows` as the `pipeline/` product, or copying GHA YAML
  into a capability `core/`.
- A second protected GitHub context (`presubmit`) or per-cap
  checks. Prow `GateRun` / Tide as `pipeline/core`. A pipeline-owned worker
  cluster beside `compute/`. `iac/` as the CD engine.
- Dual cargo+buck2 merge proof. Switching tenant #0 to buck2 because CAS
  exists but pipeline does not yet **serve** that graph. Cargo as a
  standing second pipeline runtime after cutover. Cargo-as-destination TAP.
  A sold per-language CI (cargo / go test / npm / mvn as products).
  Requiring tenant graphs to be Cargo.toml. A closed language enum that
  refuses Haskell/Zig/SQL/Shell/… without a new cap.
- Strangler-moving `workflow/` event-bus/saas/forms into `bus/` or `app/`
  instead of purge+rewrite.
- Keeping the slug `messaging/` as a live capability name (collides with
  `comms`). Law and registry are `bus/`.
- Allowing shell/Python/Go under `scripts/` or `tools/` because `.github/scripts`
  is allowed. The exception is prefix-exact.
- Keeping `comms/` as a cloud cap (mailbox/Meet/messenger/calendar). Those are
  apps. Cloud send is `notify/`.
- Kafka (or Pulsar) as the `bus/` engine / serving consume path.
- `intelligence/detection/` as this cap’s core (GuardDuty ≠ Vertex). Copilot UX in `intelligence/core`.
- Marketplace as KYC/escrow/app-store; strangler of `developer-sdk/` into billing.
- `data/core/cloud-*` crates or a `cloud-data/` nest. Search/SERP/RAG in `data/core`.
- Putting Spanner/Cockroach/RDS in `storage/`. Putting BigQuery in `storage/`. A `search/` capability for Google Search.
- Claiming Spanner TrueTime / zero-ε without a measured GNSS/atomic plant.
  Waiting for GNSS hardware before the interval API exists. A product
  `Now() → Instant`. Selecting the clock adapter through `flags/`.
  Forbidding PTP/GNSS adapters because v1 is NTP. A `time/` capability.
  Commit-wait on the `Check` path. Commit-wait of NTP ε on every v1
  OLTP write. Treating Cockroach-on-AWS as the operator plant. Two
  APIs (software clock vs hardware clock).

## Appendix — considerations (not implement authority)

Record of why D-8/D-9/D-10 landed. Not additional MUST.

- **`specs/`:** Live tree ~376 files / ~359 JSON. Gates load `product-protocol-contract.json`
  (REST required, Connect forbidden) and `per-microservice-flat-layout.json`
  (`microservices/` + required PRDs). That contradicts D-3/D-4/D-8. A catch-all
  `specs/` refilled once; shrinking it still leaves a magnet. Hyperscaler: policy
  next to the evaluator; ADRs for decisions. **Do not copy `specs/` into cap/app.**
- **`cedar/` vs cap-root `policy/`:** Inventory had ~359 files under `<cap>/policy/`.
  That collides with the `policy/` **capability** (PDP). Engine vs data names must
  not collide (`observability/` vs `observability/slos/`). Platform Cedar lives in
  the engine; caps hold unique fragments only. Stamped `tenant-scope.cedar` is the
  runbook-clone bug.
- **OpenSLO:** “Controller eats YAML” is how Helm survived. SLO source is IR;
  OpenSLO YAML is generated. Hand files (clone or unique) are debt.
- **REST transcode:** Google ESP still JSON-encodes proto. We still reject a
  **standing** dual API. Leftover REST is deleted; temporary caller red is hygiene.
- **`HANDOFF.md`:** Thin redirect (2026-06-08 exception). Content is a read-order
  already in `AGENTS.md`/`CLAUDE.md`. Deleted so a fourth hub cannot accrete status.
  **Pending founder:** the sentence “a higher ADR number does not override an earlier
  Accepted ADR by itself; only explicit `amends`/`supersedes`” — admit into this ADR
  if that is still wanted as law (it is not copied here as MUST).
- **presubmit name:** TAP/presubmit vs postsubmit. Not `presubmit`.
- **One context vs per-cap CI:** Central **admission** is hyperscaler; central
  **full-repo JSON census** is the conflict source. Per-cap **required** checks are
  the skip-fail anti-pattern.
- **etcd / AWS journal:** EKS journal is closed, etcd-API-preserving, mega-cluster.
  We cell-shard; steal log-vs-memory, not the binary.
- **EU world-floor:** KR CSAP/본인인증/e-tax is not a GDPR subset. Packs overlay.
