---
doc_status: published
id: ADR-0719
title: "EaC north star: serving vs control, proto IR, cell-local authz, packs not an EU world-floor"
status: Accepted
planning_impact: true
deciders: founder
date: 2026-08-21
door: two-way
owner: council-architecture
supersedes: []
superseded_by: []
amends: [ADR-0701, ADR-0702, ADR-0704, ADR-0705, ADR-0708, ADR-0716]
amended_by: []
depends_on: [ADR-0615, ADR-0701, ADR-0702, ADR-0704, ADR-0705]
related: [ADR-0243, ADR-0280, ADR-0354, ADR-0049]
milestone: W0
deliverables:
  - id: ADR-0719-D1
    description: "Record serving-path vs control-path split as live law: 10^8-class user/Check traffic is in-cell RAM snapshots; writes, IR apply, packs, and cluster objects are a journaled control plane. k8s/ports: etcd is the v1 adapter, owned journal is the destination adapter; cluster-object API is never the etcd API. Time is our TrueTime: interval API always; cell clock port; data consumes it; storage does not use clock as identity."
    exit_criteria: "This ADR is Accepted; CLAUDE.md live apex list cites it; no implement PR treats etcd or a Kubernetes object store as the Check/IR/tuple store."
    verified_by: "presubmit"
  - id: ADR-0719-D2
    description: "Record EaC as one protobuf IR plus per-plane reconcilers behind one Connect/H3 gateway contract (N cell frontends). No wrap language. JSON is not a product codec. Gateway TLS port crates: hybrid ML-KEM public default, classical dying, ECH in tree. Visibility is after trusted terminate + audit export, not on-path QUIC MITM. No Istio. No standing gRPC. No firewall/ cap."
    exit_criteria: "New public product surfaces and IR apply/preview/watch are authored from the Rust/proto contract SSOT; Helm/Tofu/CUE are not sources of desired state."
    verified_by: "presubmit"
  - id: ADR-0719-D3
    description: "Record compliance as jurisdiction packs. EU is not the world baseline. KR (and others) are not GDPR subsets. ReBAC and snapshots stay in the certified cell."
    exit_criteria: "Pack overlays are the only place jurisdiction law is specialized; no implement PR assumes EU-only identity, retention, or global ACL replication."
    verified_by: "presubmit"
  - id: ADR-0719-D8
    description: "Closed directory set for repo root and capability/app/<product>/ roots. A name exists only if a compiler, test, PDP, SLO controller, or reconciler loads it (or it is OWNERS/README/BUCK/app PRD). Census files and wrap languages are not children."
    exit_criteria: "ADR-0701 Status cites this D-8; new cap/app children outside the set are born-blocking without grandfathering catalog.yaml or dual cedar+policy; layout-allowlist PRs match this set."
    verified_by: "presubmit"
  - id: ADR-0719-D9
    description: "The merge-blocking CI context is named presubmit (Google TAP-shaped). New workflow and required-context names do not use an oya- prefix. Today's oya-ci-required string is a rename target, not the destination name."
    exit_criteria: "This ADR uses presubmit as verified_by; no new ADR or workflow is named oya-ci-*; the live GitHub required context rename is a follow-through PR that updates branch protection in the same change."
    verified_by: "presubmit"
  - id: ADR-0719-D10
    description: "Hyperscaler pipeline names: presubmit (merge-blocking, graph-aware), postsubmit (on merge to dev), nightly, weekly, promotion rungs dev-staging-canary-production, release train bundling. One required context. No oya- prefix. No per-capability required GitHub checks."
    exit_criteria: "This ADR defines those cadences; new workflows use those names; oya-ci-required remains a rename target with branch protection in the same follow-through change."
    verified_by: "presubmit"
  - id: ADR-0719-D11
    description: "Cloud-provider placement: the registered capabilities ARE the cloud. Repo root holds only directory names plus meta (build/third-party; base/ only when admitted) and app/. No kernel/ or os/ rungs — fleet node is Linux + compute agent, not Talos/kube. Each capability owns one engine (core), ports, adapters, facade. 2+ compose in app/. No cloud/ folder."
    exit_criteria: "This table is the placement reading; new engines go in an existing cap or a §7 registry split, not a new root dump; app/ is composition only."
    verified_by: "presubmit"
  - id: ADR-0719-D13
    description: "Fleet is stripped-minimum Linux on Cloud Hypervisor and/or Firecracker. Asterinas/Hermit are not plant today; reconsider only with a measured five-field ADR. Not Talos/kube as the cloud OS. Delete kernel/ and os/."
    exit_criteria: "kernel/ and os/ are absent from the tree and from the capability-registry meta_directories; AGENTS.md/CLAUDE.md no longer list them as production rungs; port-engine remains under build/."
    verified_by: "presubmit"
  - id: ADR-0719-D14
    description: "Per-capability is/is-not/burn for the cloud-provider set. Nested leftover service trees burn. Product engines (payments, ledger, SaaS apps) are out of this set — later discussion."
    exit_criteria: "This table is the reading for reorg; no PR parks a second engine inside a cap or treats a k8s/Talos port as that cap's core."
    verified_by: "presubmit"
  - id: ADR-0719-D15
    description: "Cloud-provider purpose, in-scope, and out-of-scope for each registered engine. This set is IaaS/PaaS/control plane only. Tenant SaaS (HR, payroll, community, Slack-superset, SAP-class ledger/payments products) is app/, not a capability charter."
    exit_criteria: "Reorg and new crates match these in/out lists; no cap charter absorbs an app product; no app/ grows a cloud engine."
    verified_by: "presubmit"
  - id: ADR-0719-D16
    description: "console/ is not a cloud-provider capability. Discard the ops-dashboard-control-center pilot. git rm; no empty scaffold; do not park in app/ops-console. Token broker is iam. Operator actions stay on each cap facade. A future UI is app/ after the apps discussion."
    exit_criteria: "console/ is absent from the tree and from the closed capability registry; ADR-0701 Status cites D-16; layout allowlists do not re-admit console/."
    verified_by: "presubmit"
  - id: ADR-0719-D17
    description: "Presubmit is cargo fmt/clippy/test plus a short closed set of admission engines. Census gates, Helm/OpenAPI/OpenSLO parity, docs-coverage, frozen counts, min_expected_*, and expected_total pins are deleted, not trimmed."
    exit_criteria: "ci/facade and governance/check contain only the D-17 keep set; cedar-deploy-parity and scan-root-liveness are gone; no new gate is a path/count freeze."
    verified_by: "presubmit"
  - id: ADR-0719-D18
    description: "pipeline/ is one execute engine: TAP internally (tenant #0) and Cloud Build sold are two facades. GHA disjoint adapter. JSON/governance check fleets are not the product. ci/ is a retired path."
    exit_criteria: "ADR tables use pipeline/, bus/, notify/; workflow/ and comms/ trees absent; rust-first exclude_prefixes includes .github/scripts/; GHA YAML is not a face of pipeline/."
    verified_by: "presubmit"
  - id: ADR-0719-D19
    description: "Every repo-root name is DO or DON'T, and HAVE or HAVE NOT: DONE, BUILD, REMOVE, or STAY GONE. No new cloud-* crates. REMOVE is delete/rewrite in charter, not a move to another cap."
    exit_criteria: "New crates are DO+HAVE-NOT (BUILD) or DO+HAVE (DONE); PRs that add DON'T names or rehome REMOVE dumps fail review."
    verified_by: "presubmit"
---

# ADR-0719: EaC north star — serving vs control, proto IR, packs

## Status

**Accepted** (founder 2026-08-21). Amends the live identity/authz, k8s-port, product-protocol,
and platform-foundations apexes. Does not archive them.

Chat/session architecture is not law. This file is.

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
Today’s markdown under `packs/` is KEEP+WORK, not the job.

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

A directory or file is allowed only if something that is **not a census gate** loads it,
or it is `OWNERS` / short `README.md` / `BUCK` / `app/<product>/PRD.md`. Git history is
the audit log. Do not invent a destination for leftovers; many must **not exist**.

**Repo root (closed).** Workspace: `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`,
`rustfmt.toml`, `deny.toml`, `reindeer.toml`, `.buckconfig`, `.buckroot`, `.cargo/`.
GitHub: `.github/`, `.gitignore`, `.gitattributes`. Hubs: `README.md`, `LICENSE`,
`OWNERS`, `AGENTS.md`, `CLAUDE.md`. Meta: `build/`, `third-party/`. `base/` is **not**
pre-created; it appears only when the first crate admitted under the ≥3-caps-below-all
rule. `governance/` is a **capability** (checks + `capability-registry.json`).
**No `kernel/` and no `os/` rungs.** Fleet is **stripped-minimum Linux**
on **Cloud Hypervisor** and/or **Firecracker** (`compute/`). Not Talos,
not kubelet, **not Asterinas/Hermit today**. Do not vendor or leave an
evaluation tree. Reconsider Linux replacement only per D-13
`overturn_when`. In-tree Talos/Asterinas output is **deleted**.
`os/ports/kernel-abi` dies with `os/`.
Composition: `app/`. One directory per **registered** capability (including `policy/`
the **engine**). Jurisdiction: `packs/<id>/` one versioned bundle the engines load.
`docs/` = ADRs + operating contract. **No catch-all `specs/`.** Machine contracts live
next to the evaluator (gate `*-policy.json`, Cedar, IR proto). Agent entry is
`AGENTS.md` / `CLAUDE.md`. `HANDOFF.md` is **deleted** (founder 2026-06-08 exception
withdrawn); it was a redirect, not law.

**Not repo-root:** `oya/`, `cloud/`, `libs/`, `infra/`, `tools/`, `toolchains/`
(reorg_now); root `Makefile`; root `Dockerfile*` (→ `build/`); `oya-*.toml`; tracked
agent dirs; `evidence/` dumps; `benchmarks/`; `scripts/`; `tasks/`/`plan/` as required
corpora; `catalog.yaml` trees; root `contracts/` YAML as IDL; root `registry/`;
root `specs/` as a JSON law corpus (delete with the gates that load contradicting
files; do not scatter into cap/app). `HANDOFF.md`. `kernel/`. `os/`.

**Capability root and `app/<product>/` (closed children):** `core/`, `ports/`,
`adapters/`, `facade/`, `cedar/` (**.cedar unique to this cap** — schema and resource
policies the PDP compiles; platform templates live under the `policy/` **capability**,
not stamped here), `observability/slos/` (**generated OpenSLO from IR only** — hand-authored OpenSLO,
including “unique” files, is debt; clones already forbidden), `iac/` (IR the reconciler
applies, not Helm/Tofu source), `OWNERS`,
short `README.md`, `BUCK`. `app/<product>/` may add `PRD.md`. No cap-root `contracts/`
(IDL is Rust → proto on `ports/` / generated faces). No cap-root `policy/` (that name
is the engine at repo root). Nested leftover service trees become faces or go.

**Must not exist at cap/app root:** `manifest.json` census, `catalog.yaml`, `IPs/`,
`IP-journey-*.md`, `AUDIT-FINDINGS-*.json`, `REMEDIATION-NOTES-*.md`, `scorecards/`,
`dashboards/*.json`, `dpia/`, `decisions/` copies, `capabilities/*.yaml` essays,
stamped runbooks, stamped `tenant-scope.cedar` copies, dual ARCH files, placeholder
READMEs. Delete the gate with the files.

**Public door:** proto/H3 is the product. REST/JSON leftover is **deleted**, not
transcoded as a standing codec. No new public REST shapes. Console/SDK/gates that
still speak REST may go red until they speak proto — that break is in-scope hygiene.

**MUST (closed children)**

- **achieves:** engine vs data names do not collide; N copies of platform Cedar/SLO
  cannot reappear; org law has one tree gates already load.
- **origin:** naming `policy/` for both the PDP and per-cap Cedar followed the live
  tree; OpenSLO-as-authoring and REST transcode are Helm-shaped dual stacks; dual
  `cedar/`+`policy/` allowlists encoded the collision.
- **rule:** cap-root `cedar/` only; `policy/` is the capability; SLO source is IR;
  no `specs/` catch-all; `ports/` is the contract face; extras, REST/JSON product
  surfaces, and `HANDOFF.md` deleted, not grandfathered. Temporarily breaking live
  callers/gates is accepted. Leaving anti-pattern debt is not.
- **ensure:** layout allowlists match this set; no immortal `IPs/`; no both `cedar/` and
  `policy/` as cap children.
- **overturn_when:** a child is loaded by a compiler/PDP/SLO/reconciler AND a five-field
  amendment lands same-wave.

## Consequences

- Implementers read this plus ADR-0701/0702/0704/0705/0615. Do not re-derive from chat.
- `policy/` extraction and IR proto are implementation follow-through, not optional sketch.
- Admission remains VAP/CEL+PSA as cited from ADR-0704 / ADR-0700. Proposed
  0710-range ids are not `depends_on`.
- Merge-blocking CI is **presubmit** only (D-10). No second protected
  `merge-admission-required` context.
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

**Our fleet:** **stripped-to-minimum Linux** guests, scheduled by
**`compute/` + `cell/`**, running on:

| Adapter | Class | Analog |
|---|---|---|
| **Firecracker** | microVM / functions / dense isolation | Lambda, Fargate microVM |
| **Cloud Hypervisor** | VM | rust VMM, virtio, full-ish guests |

Both are KVM. Not QEMU as identity. Not kubelet. The **agent** on the
guest (or host) is ours — Borglet analog. `compute/` already has three
reconcilers; Firecracker ≈ functions, Cloud Hypervisor ≈ VM.
k8s-on-compute remains an **optional sold SKU**, not how we operate.

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
  code is CH/Firecracker adapters under `compute/`.
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

No new `scripts/check.sh` / pre-push product. Rust-first: a three-line git hook may call `cargo fmt --check` on staged `*.rs`. Do not resurrect `oya verify` / `dev-cli`.

**MUST (nextest is the proof)**

- **achieves:** one compile+test signal; no double compile; no libtest dual; PR hermetic.
- **origin:** blog four-tier put `cargo check` and network `cargo-audit` on the merge path; `cargo-dist` assumes a CLI product; mutants-on-everything is not a nightly.
- **rule:** nextest is the only compile+test proof in presubmit/postsubmit/nightly unit lanes; `cargo check` and `cargo test` are not CI; release binary is CD; advisory fetch on PR is vendored; one license/ban engine; no cargo-dist; no crates.io semver gate.
- **ensure:** required workflow invokes nextest, not libtest; no `cargo check` job; no win/mac per-PR smoke; deny/audit are not two network tools on the PR.
- **overturn_when:** a five-field ADR names a different runner that still compiles once and stays hermetic.

Do **not** add one required GitHub check per capability (skipped-check failures, queue
combinatorics). **One** protected context: `presubmit`. `merge-admission-required`
is not a second protected check. Lane isolation is **worktrees + non-overlapping
paths**, not 24 contexts. Do not resurrect merge-base **count** baselines as
“affected set.”

New workflow and context names: `presubmit`, `postsubmit`, `nightly`, `weekly`,
`promotion-predecessor`, `release`. No `oya-` prefix. Today’s `oya-ci-required` is
the **presubmit** rename target (branch protection in the same change).

### D-11 — What the cloud is, and what each capability holds

The cloud is **not** a root `cloud/` tree and **not** a JSON catalog. It is the
**closed capability set** (registry). Repo root only **names** those directories
(plus `base/` when admitted, `build/`, `third-party/`, `app/`, `packs/`, `docs/`,
`governance/` as already in D-8 — **not** `kernel/` or `os/`). Everything a
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
| EC2 / GCE / Functions / GKE-on-VMs | **compute** | **One** cap, **three reconcilers** (VM, k8s-on-compute, functions). Facades, not three caps, not one Raft. |
| GKE / EKS control plane (sold) | **k8s** | core: managed cluster (lifecycle, CP host, quota, CAPI). Adapter: upstream or owned apiserver when we run it (D-13). Store: cluster objects only (D-2). |
| VPC / DNS / firewall / flow logs | **network** | core: dataplane, security groups (**allow** UDP/443), flow logs, QUIC metadata. Not Istio. Not a `firewall/` cap. |
| Front door / GFE / WAF / IAP | **gateway** | core: one Connect contract (H3 default, H2 same framing), Maglev **per cell**. TLS port + WAF-after-decrypt + explicit-proxy crates. Transcode is not a second API. |
| Pub/Sub / SQS | **bus** | owned queue + fan-out + seekable stream + outbox; per-key order. Kafka/Pulsar = adapters. Not `notify`. |
| Vertex / Bedrock | **intelligence** | core: inference + agent runtime + adapters. Not GuardDuty. Not a chat app. |
| Step Functions / Composer | **workflow** | core: engine. Studio is facade. Business sagas, **not** deploy orchestrator (D-1). |
| Cloud Build / TAP / CodePipeline | **pipeline** | **One** execute engine. Internal TAP = this repo as tenant #0. Sold Cloud Build = same engine, tenant graphs. GitHub is an adapter, not the product. JSON check fleets are not this cap. |
| CloudFormation / Config reconciler | **iac** | core: IR unifier + reconcilers. `<cap>/iac/` is **this** cap’s desired state; `iac/` the cap owns the **engine**. |
| Billing / Cost Explorer | **billing** | core: meter, rate, invoice, tax, FinOps. Sold-ness, not a drawer. |
| Marketplace | **marketplace** | core: signed plugins + Cedar install envelope + SKU **engine**. Price list is `build/` view. Not KYC/escrow. |
| Artifact / evidence packs | **compliance** | core: pack evidence, data-class registry. Consumes **audit**. Not the Merkle log. |
| SES / SNS / FCM (send) | **notify** | core: transactional email/SMS/push **send**. Not Gmail/Meet/Slack. |
| AppConfig / Feature flags | **flags** | core: flags, kill switches. Pack-gated overrides. |

**Meta (not sold as a tenant API, still in-repo):** `base/` only when admitted (≥3 caps, below all); `build/` toolchains/images/port-engine; `third-party/` vendored; `governance/` registry + check crates (off the runtime ladder). No `kernel/` or `os/` rungs (D-13).

**`app/<product>/`:** composition only (hr, payroll, calendar, community, …). Wires 2+ of the table. **Does not** grow a cloud engine.

**`payments/` and `ledger/` are not this cloud set** (D-15). Do not park them in `billing/` or `oya/`. Product placement is a later discussion.

### D-14 — Each capability: is / is not / burns

Same split as `k8s/` (GKE product vs kube port). Nested leftover service dirs inside a cap **burn** (faces or `git rm`).

| Cap | **Is** (engine) | **Is not** | **Burns / move** |
|---|---|---|---|
| **cell** | Topology, hard caps, router, rebalance, **clock port** (ntp / ptp_phc / gnss_atomic). Borg/GCP-zone analog. | GKE product (`k8s/`). Tenant CRM. A `time/` cap. Clock via `flags/`. | Census, leftover lifecycle dirs once faces exist. |
| **tenancy** | Tenant lifecycle, home-cell, org/account analog. | IdP (`iam`). PDP (`policy`). SKU catalog (`marketplace`). | Enablement side-effects; JSON tenant novels. |
| **iam** | Principals, passkeys, SCIM, role **store**, workload identity **consume**, **`device_attestation` port**. | Cedar **eval** (`policy`). SVID **issue** (`secrets`). An Island-class browser. | PDP crates **move to `policy/`**. trustd deps **move to `secrets/`** with `os/` delete. |
| **policy** | Cedar + ReBAC PDP, G-face + C0 snapshots. | IdP. Empty dir forever. | **Extract crates from iam now.** Cap-root `<other>/policy/*.cedar` → `<other>/cedar/`. |
| **secrets** | KMS, secret material, **SPIFFE issue**. | PDP. Cert spam as YAML. | Absorb `os-trustd-domain` consumers. |
| **audit** | Tamper-evident log. Always on. | Compliance packs (`compliance`). Sync Merkle on every Check. | DPIA essays, scorecards. |
| **observability** | Telemetry + SLO **controller**. | Per-cap hand OpenSLO. SIEM as a 25th cap. | Stamped OpenSLO. Detection is **not** this cap and **not** intelligence core. |
| **storage** | Durable **bytes**: object/CAS. Identity = digest/generation. | SQL/Spanner/Cockroach (`data`). Search. Analytics query. TrueTime as object identity. | Imaging leftover; census. Block/file = facades **when sold**, no empty dirs. |
| **data** | Durable **records**. **Consumes** cell TrueTime. Versionstamps = commit ordinal, not a second clock. `commit_wait` adapter crate (IR off on NTP ε). | S3/CAS (`storage`). Google Search / SERP. RAG. BI **app**. `cloud-*` names. A private `Now()`. | Nested dumps + `search-*` + `data-cloud-*` **purged**. ClickHouse/Postgres = **adapters**. |
| **compute** | **One** cap: VM + k8s-on-compute + functions as **three reconcilers / facades**. | GKE product (`k8s/`). GPU = facade when sold. One Raft for all three. | Splitting into 3 caps. |
| **k8s** | **Managed cluster product** (lifecycle, CP host, quota, SLO, CAPI). | kube-apiserver port (`build/port-engine` → adapter when we run it). Node OS. Mesh. | Dump + nested `managed-*`. |
| **network** | VPC/DNS/**dataplane**, security groups (allow UDP/443), `flow_log` + `quic_metadata` crates. Not Istio. | Public API door (`gateway`). A `firewall/` cap. Sidecar mesh as **our** identity. On-path QUIC decrypt. | Census. Istio/Linkerd as default. |
| **gateway** | One Connect **contract** (H3 default, H2 same framing), N cell frontends, TLS + `waf` + `explicit_proxy` + `fingerprint` crates. Cloud IAP is this facade. | Mesh (`network`). Second REST/gRPC API. One global VIP. Transparent QUIC MITM. ECH-off “enterprise mode.” Per-pod IAP sidecar. Browser DLP as this cap’s core. | REST/gRPC dual-stack; connector leftover if it’s a second door. |
| **bus** | Owned queue + fan-out bus + seekable stream + outbox; per-key order. Pub/Sub + SQS analog. | Sagas (`workflow`). Mailbox (`app/`). **Kafka/Pulsar as `core/`**. Human chat. SES (`notify`). | Crate names still `messaging-*` (KEEP+WORK rename). Kafka/Pulsar = **adapters** only. |
| **intelligence** | Vertex/Bedrock: inference + agent runtime. | Provider adapters, eval/proof, invoke facade. | GuardDuty (`detection/` **purged**). Chat copilot **app**. CLIs. Census YAML. |
| **workflow** | Step Functions analog (rewrite). | Bus (`bus`). Forms/tasks/SaaS. Deploy (`pipeline`/`iac`). | **Purge current tree; rewrite.** Do not strangler. |
| **pipeline** | One execute engine (graph, queue, workers, controller). | `.github/` GHA as the product. Prow/Tide as `core/`. JSON policy crates as the product. A root named `ci/`. | KEEP+WORK: today’s tree is not the product. Census/JSON gates REMOVE. Tide/webhook = GitHub adapter until cutover. |
| **iac** | IR unifier + reconcilers. | Argo-SHA observer as the engine. Helm/Tofu **source**. | Observer; `<cap>/iac` Helm dumps. |
| **billing** | Meter, rate, invoice, tax, FinOps. | Ledger books (`ledger/`). Payments rails (`payments/`). | Nested accounting/tax leftover dirs. |
| **marketplace** | Signed plugins, install envelope, SKU engine. | Price list (`build/` view). KYC/escrow/payout. App store UX. | **Purge** `developer-sdk/` + `plugin-app-store/`. |
| **compliance** | Pack evidence, data-class registry, **CaS facade** (bind/project/export). | Merkle log (`audit`). Cloned `dpia.md`. Private pack API. | Those clones. |
| **notify** | Transactional send (SES/SNS/FCM). | Email/SMS/push **send API**. | Mailbox/Meet/Messenger/contact-center (`app/` later). Current `comms/` dump **purged**. |
| **flags** | Deterministic **eval** (keep `evaluation-domain`), targeting, kill switch, pack-gated overrides. | Experiments product / p-value dashboards. OpenAPI+REST+gRPC dual. Clock adapter. OFREP as SSOT. | Cap-root dump (`catalog.yaml`, IPs, Helm, AUDIT-FINDINGS). REST/gRPC server dual. |
| **governance/** | Registry + check **crates** (off ladder). | Org JSON `specs/` corpus. | Specs catch-all. |
| **build/** | Toolchains, images, **port-engine**, SKU **view**. | Capability engines. | — |
| **third-party/** | Vendored pins when we need them. | Fake rungs (`kernel/`/`os/`). | Asterinas eval in `kernel/`. |
| **app/** | 2+ cap products. | A cloud engine. | Absorbing D41 retirees; parking payments; **do not absorb the `console/` pilot**. |

**Missed before, now closed:** GKE vs kube-port (`k8s/`); Talos vs `os/`; PDP vs iam; trustd vs secrets; payments/ledger not billing; no empty `base/`/`kernel/`/`os/`/`k8s-port/`; census `ci` gates are not the delivery fabric.

### D-15 — Cloud-provider purpose and scope (not SaaS)

This set is what we **sell and run as a hyperscale cloud**. Analog: AWS/GCP/Azure **platforms**.  
**Out of every row below:** tenant SaaS products (HR, payroll, community, calendar, Slack-class UX, SAP-class accounting). Those **use** these engines via `app/<product>/`. A capability that ships a vertical product in `core/` is out of charter.

| Cap | Purpose | In scope | Out of scope |
|---|---|---|---|
| **cell** | Bound failure domains and place load. | Topology, hard caps, router, rebalance, home-cell, **clock port** + adapters (`ntp` v1, `ptp_phc`, `gnss_atomic` destination). TrueTime interval is the API; plant only sets ε. | GKE product. Tenant CRM. Spanner ε as a v1 claim. Clock selection via `flags/`. A `time/` cap. |
| **tenancy** | Tenant as the scoping primitive. | Create/suspend/delete tenant, home-cell binding, org/account tree. | IdP (`iam`). Authz eval (`policy`). Marketplace SKUs. HR orgs. |
| **iam** | Prove **who** (and **device posture** as Cedar context). | Principals, credentials, passkeys, SCIM, role **store**, workload identity **consume**, `device_attestation` (WebAuthn / MDM / Chrome-Enterprise / SPIFFE adapters). | Cedar **eval** (`policy`). SVID **issue** (`secrets`). Tenant lifecycle (`tenancy`). Forking Chromium. |
| **policy** | Decide **may**. | Cedar PDP, ReBAC tuples, G-face distribute, C0 in-cell snapshot, in-process Check. | IdP. Writing every cap’s Cedar (caps own `<cap>/cedar/`). Global tuple replica. |
| **secrets** | Crypto root and issuance. | KMS, secret material, SPIFFE **issue**, cert **issue** when sold. | PDP. Embedding secrets in app products. |
| **audit** | Tamper-evident **record**. | Merkle log, seal of principal+tenant, privileged-path durability, **tenant-exportable** access events (the CISO feed). | Pack evidence (`compliance`). Sync seal on every Check. DPIA markdown. On-path packet capture as the audit product. |
| **observability** | See and SLO-gate the platform. | Metrics/logs/traces **substrate**, SLO **controller**, generated OpenSLO apply. | Hand OpenSLO novels. SIEM as a 25th cap. App product analytics. |
| **storage** | Durable **bytes** (S3 / GCS / Colossus / CAS). | Object/CAS; drive/recordings as byte **facades**. Identity = digest/generation. | Any **query engine**. Spanner/Cockroach/RDS. BigQuery. Search. Clock as object identity. |
| **data** | Durable **records** + query engines. | OLTP/OLAP/pipelines/ontology. Consumes cell `Now() → Interval`. Versionstamps as engine ordinal. `commit_wait` crate present, IR off without measured ε. | Bytes (`storage`). **BI product** (`app/`). **Web search / SERP**. **RAG** (`intelligence` facade if sold). **`cloud-*` crate names.** A second TrueTime. |
| **compute** | Run **the fleet** (Borg/Twine/Nitro analog). | Stripped Linux on **Cloud Hypervisor** (VM) and **Firecracker** (microVM/functions). Agent is ours. Optional k8s-on-compute **SKU**. GPU as facade when sold. | GKE as the fleet. Talos as the fleet OS. Asterinas/Hermit **today**. QEMU as identity. One Raft. |
| **k8s** | **Sold** GKE/EKS/AKS-class SKU. | Cluster lifecycle, hosted CP, quota, SLA, CAPI, **upstream** kube adapter (EKS pattern). | Our Borg (`compute/`+`cell/`). A kubernetes.git port as operations. Node OS. Mesh. Public door. Empty `k8s-port/`. |
| **network** | Connect inside the cloud. | VPC, DNS, **TCP-optimized** dataplane; UDP/443 **north-south**; `flow_log`, `quic_metadata`. | Public door (`gateway`). QUIC as the in-cell RPC plant. Istio. `firewall/` cap. Block public QUIC. Payload decrypt. |
| **gateway** | **One** north-south **contract**, many cell frontends. | Public **H3/QUIC**; H2 if path cannot UDP. East-west is **not** this door’s plant (TCP in-cell). TLS/ECH/WAF/IAP/fingerprint as above. | Mesh. Second gRPC/REST door. QUIC-mandatory east-west. One global VIP. Transparent QUIC MITM. ECH-off enterprise mode. Per-pod IAP. |
| **bus** | Move **events** (Pub/Sub / SQS / Service Bus). | Owned substrate: **queue** (competing consumers), **bus** (fan-out subscriptions), **stream** (seekable cursor); transactional **outbox**; at-least-once; per-key order. Serving `Check` never *is* a consume. | Sagas (`workflow`). Mailbox / chat (`app/`). **Kafka or Pulsar as `core/`**. SES send (`notify`). A root named `messaging/`. |
| **workflow** | Managed **sagas** (Step Functions / Cloud Workflows). | Rewrite: state machine, retries, timers, execution API; studio as authoring **facade**. | Bus (`bus`). Forms/tasks/SaaS. Deploy (`pipeline`/`iac`). Current tree (purged). |
| **intelligence** | Managed **inference + agent runtime** (Vertex / Bedrock). | Model adapters, eval, invoke facade, quota. | `detection/` (GuardDuty — later product). Copilot **app**. CLIs. Cap-root YAML essays. |
| **flags** | Dynamic config and kill switches. | Deterministic eval (`evaluation-domain`), targeting, kill switch, pack overlays. Connect facade. | App roadmaps. Census catalogs. **Clock adapter**. A/B experiment **product**. REST/gRPC/OpenAPI dual. OFREP as SSOT (OpenFeature may be an adapter). Helm source. |
| **pipeline** | Productized execute (TAP internally, Cloud Build sold). | **One engine**, two facades: **polyglot** hermetic graph + queue (buck2 when CAS+RE live). Workers = `compute/`. Promotion graph execute. One required context `presubmit`. Tenant #0 is Rust-first; **customers are not**. | GHA as product. Cargo as sold runtime. Per-language CI SKUs. JSON check product. Prow/Tide as core. Owned worker cluster. `iac/` as CD engine. Second protected check. A `ci/` root. |
| **iac** | Apply **desired state**. | IR unify/preview/apply/watch, reconcilers, Helm **adapter only**. | Merge queue (`pipeline`). Business sagas (`workflow`). Helm/Tofu as source. |
| **billing** | Charge for **cloud use**. | Meter, rate, invoice, tax on **platform SKUs**, FinOps attribution. | Card rails as a bank (`payments` product). Universal accounting books (`ledger` product). |
| **marketplace** | Third-party **modules** on the cloud. | Signed plugins, Cedar envelope at install, SKU **engine**. | Price list (`build/` view). KYC/escrow/SEPA/tax. Developer portal **app**. `developer-sdk/` + `plugin-app-store/` dumps (purged). |
| **compliance** | Evidence **engine** + **CaS facade**. | Pack catalog, projection bind, evidence export. First-party and third-party apps consume this + `policy/` Check. | Merkle log (`audit`). Jurisdiction **data** (`packs/`). Cloned DPIA. A private pack API for `app/`. |
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
- **origin:** `cloud/` was emptied; the cloud leaked into JSON specs and nested `oya-*` / `cloud-*` leftover dirs inside caps.
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
- Meta: `docs/`, `governance/`, `build/`, `third-party/`, `packs/` (v1: us, eu, jp, kr), `app/` (composition only)

**KEEP+WORK** (DO + HAVE, additional REMOVE or BUILD inside)

- `iam/`: extract PDP → `policy/` (BUILD `policy/` same change)
- `k8s/`, `data/`, `network/`: nested census REMOVE
- `flags/`: cap-root dump + REST/gRPC `server` REMOVE; eval stays
- `gateway/`: connector dump REMOVE; Connect door BUILD after
- `pipeline/`: path is `pipeline/`; contents are not the product (Prow/Tide/census). BUILD execute core; REMOVE JSON gates and Tide-as-core
- `bus/`: rename `messaging-*` crate ids; Connect facade BUILD
- `cell/`: PTP/GNSS bind when plant exists (adapters already named)
- `packs/`: KEEP us/eu/jp/kr; BUILD Cedar+IR loader; REMOVE markdown/OVH YAML and extra jurisdiction dirs (`au br cn in ksa mx`)

**BUILD** (DO + HAVE NOT)

- `policy/` — extract from `iam/` in the same change as the directory
- `gateway/` Connect/H3 door, TLS/WAF/IAP/fingerprint/ECH crates **after** connector REMOVE
- `workflow/` Step Functions — directory + `core/` in one PR
- `notify/` SES send — directory + `core/` in one PR
- `network/` `flow_log` + `quic_metadata`; `k8s/ports` `owned_journal`; `data/` `commit_wait`
- `base/` only when the ≥3-caps rule admits the first crate

**REMOVE** (DON'T + HAVE)

- `gateway/` Workday/Slack/Salesforce/… connectors
- `flags/` dump (catalog.yaml, IPs, Helm, REST+gRPC server, experiment dashboards)
- Nested census; a root named `ci/` (retired; do not recreate)
- Repo-root leftovers: `oya/`, `libs/`, `infra/`, `tools/`, `toolchains/`, `benchmarks/`, `evidence/`, `contracts/`, `registry/`, `scripts/`, `plan/`, `tasks/`, `specs/`, `kernel/`, `os/`
- A root named `messaging/` (retired; do not recreate)
- `cloud-*` crates; cap-root IPs, AUDIT-FINDINGS, Helm source, OpenAPI product, `catalog.yaml`

**STAY GONE** (DON'T + HAVE NOT)

- `cloud/`, `console/`, `comms/`, `time/`, `firewall/`, `k8s-port/`, empty `kernel/`/`os/`/`policy/`/`workflow/`/`notify/` scaffolds
- Island-class browser as a cloud root; `payments/` and `ledger/` as **caps** (products, §7)
- Kafka as `bus/` core; GHA as `pipeline/` core; Istio as identity; on-path QUIC MITM
- New `cloud-*` names; EU-as-world-floor; EU as a country; combinatoric
  pack ids; REST+gRPC as a standing product
- Search/detection/GPU/CDN as **roots** (facades of `data/`/`compute/`/`network/` if sold)

`app/hr` `app/payroll` `app/calendar` `app/community` `app/sheets` `app/global-trade` are HAVE and **not caps**. Apps ADR after the cloud set is settled.

**MUST (DONE / KEEP+WORK / BUILD / REMOVE / STAY GONE; no cloud-* debt)**

- **achieves:** placement is a 2×2; DONE is not finished; law never treats a retired slug as live.
- **origin:** calling the bus `messaging/` in the ADR recreated the alias; dumps “moved” instead of dying.
- **rule:** DO+HAVE = DONE or KEEP+WORK; DO+HAVE NOT = BUILD in charter; DON'T+HAVE = REMOVE here; DON'T+HAVE NOT = STAY GONE; capability id is `bus/` not `messaging/`; no new `cloud-*`; REMOVE is not rehome.
- **ensure:** new crates land under the DO name (`bus/`, `pipeline/`); PRs that add STAY GONE names, a `messaging/` root, or rehome REMOVE dumps fail review.
- **overturn_when:** a §7 split/merge changes DO/DON'T with five fields same-wave.

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
| One license/ban engine (`deny.toml` **or** one crate, not both) | Legal. |
| Generated-face freshness | Generated files are not merge surfaces. |
| Rust-first (`automation-language-policy`) | Founder invariant. `.github/scripts/` excluded. |
| D-8 unknown-name (one engine, not two of hygiene + membership + frozen lists) | Closed root set. Fails on **unknown names**, never `expected_total`. |
| `affected-target-set` | This **is** TAP graph. Belongs in `pipeline/` **core**, not a JSON gate. |

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

- **achieves:** merge is TAP execute, not a policy-file farm.
- **origin:** census JSON and `governance/check` became the product; Cloud
  Build never shipped.
- **rule:** presubmit is fmt + clippy + nextest plus the short pattern-step
  table; path/count freeze JSON is not a gate; `governance/` is registry
  not CI; new check crates are born-blocking unless they are a pattern
  step in the `pipeline/` graph.
- **ensure:** no new `*-policy.json` freeze; no new `governance/check/*`
  census crate; GHA must not grow predicates this ADR deleted.
- **overturn_when:** a five-field ADR adds one engine that evaluates
  IR/Cedar/cargo graph without a frozen corpus.

### D-18 — `pipeline/` product vs GHA operator; purge `workflow/`; `.github/scripts` glue

**Product.** One execute engine: **graph + queue + schedule**. Analog:
Google **TAP** (internal) + **Cloud Build** (sold). Two **facades**, not
two codebases.

- **Internal:** this monorepo is **tenant #0**. D-10 cadences are that
  tenant’s graph. Same APIs a customer will call.
- **Sold:** tenant submits a graph; same schedule/queue.
- **Workers:** **`compute/` runs the work** (VM / k8s-on-compute /
  functions). `pipeline/` does not own a second cluster or a Prow job
  pool. Functions are one step type, not the only runner.
- **Promotion / CD:** `pipeline/` **executes** the promotion graph
  (`dev` → `staging` → `canary` → `production`). `iac/` is desired
  state. `k8s/` is the cluster product. Images are `build/` artifacts in
  `storage/`. `cargo build --release` is a **CD graph step**, not
  presubmit.
- **Merge:** **one** required context: **presubmit**. GitHub merge queue
  is GitHub’s, via an adapter, then gone. No `merge-admission-required`
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

**`notify/` not `comms/`.** `comms/core` is mailbox, Meet, messenger, calendar,
address book — Workspace/Slack, not a cloud send API. Those are **later `app/`**
products (dogfood tenant #0). The cloud primitive is transactional **send**
(SES/SNS/FCM): destination slug **`notify/`**. **Purge** `comms/`. Do not
strangler mail/meet/messenger into `app/` from this dump. Rewrite `notify/` from
the send charter.

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
- Branding the merge-blocking CI `oya-ci-*` or adding one required check per capability.
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
- A second protected GitHub context (`merge-admission-required`) or per-cap
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
- **presubmit name:** TAP/presubmit vs postsubmit. Not `oya-ci-required`.
- **One context vs per-cap CI:** Central **admission** is hyperscaler; central
  **full-repo JSON census** is the conflict source. Per-cap **required** checks are
  the skip-fail anti-pattern.
- **etcd / AWS journal:** EKS journal is closed, etcd-API-preserving, mega-cluster.
  We cell-shard; steal log-vs-memory, not the binary.
- **EU world-floor:** KR CSAP/본인인증/e-tax is not a GDPR subset. Packs overlay.
