---
doc_class: ArchitectureWalkthrough
doc_id: WT-keystone-bundle-intern
status: Draft
date: 2026-05-20
owner_team: council-architecture + axis-messenger + axis-design
audience: intern-readable
related_adrs:
  - ADR-0009-cell-architecture-per-tenant-per-region.md
  - ADR-0028-audit-chain-merkle-sealed.md
  - ADR-0044-service-mesh-and-mtls.md
  - ADR-0099-data-class-registry.md
  - ADR-0105-thirteen-layer-canonical-enum.md
  - ADR-0128-hyperscaler-architecture-invariants.md
  - ADR-0131-per-microservice-flat-layout.md
  - ADR-0148-service-mesh-cilium-ambient-layered.md
  - ADR-0149-api-gateway-vs-service-mesh-separation.md
  - ADR-0150-cedar-policy-engine.md
  - ADR-0183-policy-engine-separation-cedar-app-authz-kyverno-admission.md
  - ADR-0188-passkey-webauthn-substrate.md
  - ADR-0211-in-house-tech-stack-preference.md
  - ADR-0238-dual-context-isolation-invariant.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0241-dr-business-continuity-portfolio-policy.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0247-self-hosting-self-modification-doctrine.md
  - ADR-0248-amazon-shape-cellular-architecture.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0252-time-coordination-distributed-consistency.md
  - ADR-0253-network-topology-edge-service-mesh.md
  - ADR-MSGR-0001-livekit-webrtc-canonical.md
  - ADR-MSGR-0002-mls-e2e-canonical.md
  - ADR-MSGR-0003-matrix-activitypub-federation.md
related_prds:
  - microservices/messenger/PRD.md
related_standards:
  - docs/standards/messenger-e2e-encryption-mls.md
related_user_stories:
  - docs/user-stories/b2c-consumer-surfaces.md (M-01)
intent: >
  Single canonical end-to-end worked example showing the entire 14-ADR
  keystone bundle, the Messenger PRD, the MLS E2E standard, and the B2C
  user-stories compendium in motion. Walks Alice's first personal MLS-E2E
  DM to Bob from UI tap to delivery, citing the exact ADR, spec, PRD
  section, and standard clause that governs every step. Intended as the
  one document a brand-new intern can read and recompose the architecture
  from.
---

# Keystone-Bundle Intern Walkthrough — Alice sends her first personal MLS-E2E DM to Bob

> Read this once and you understand the platform.
>
> Every step cites the artifact that governs it. Every citation is
> testable: open the cited section in the cited file and you will find the
> mechanism described there. Where two artifacts overlap, the one closest
> to the source of truth is cited (e.g., for cipher-suite selection, the
> MLS standard is cited, not the PRD; for tenant routing, ADR-0242 +
> ADR-0244 are cited, not the Messenger PRD).
>
> The walkthrough is intentionally long-form. Where redundancy improves
> intern comprehension, it is preserved. Skim once for shape; re-read
> after touching each cited artifact.

---

## 0. How to read this document

The walkthrough has six sections:

1. **Setup** — who Alice and Bob are, what cells they live in, what
   devices they hold, what tenants they belong to.
2. **Step-by-step walkthrough** — 70+ numbered steps from Alice tapping
   the app icon to her receiving Bob's heart-react. Each step cites the
   exact artifact (ADR §, PRD §, standard §, story §) that governs it.
3. **Architecture diagram** — ASCII art of the full path.
4. **What an intern learns** — checklist mapping the walkthrough back to
   every keystone ADR and every Messenger BC.
5. **Failure scenarios** — ten things that could go wrong at any step,
   and the architectural primitive that recovers from each.
6. **Comparison vs Slack / iMessage / WhatsApp** — why oyatie's path is
   hyperscaler-grade, not just feature-parity.

Conventions:

- **`tenant-alice-12345`** and **`tenant-bob-67890`** are the canonical
  per-user personal tenant IDs (per ADR-0242 D-1 + ADR-0244, every B2C
  consumer is a principal under a personal-tier tenant). They are
  reserved-namespace-checked at creation (ADR-0242 D-6).
- **`cell-us-west-2-a`** and **`cell-us-east-1-c`** are Tier 3 data plane
  cells (ADR-0248 D-4). Tier numbering is from ADR-0248.
- Cedar gates are cited as `<gate-name>` referencing ADR-0243 D-3
  catalogue or the per-µservice Cedar fragment file.
- HLC timestamps are cited as `<hlc>` referencing ADR-0252 D-1.
- All times are intern-readable approximate budgets, not contractual
  SLOs (PRD §10.1 carries the contractual numbers).

---

## 1. Setup

### 1.1 Alice's tenant

| Property | Value | Source |
|---|---|---|
| `tenant_id` | `tenant-alice-12345` | Tenancy migration; ADR-0242 D-1 reserved-namespace check passed because `tenant-alice-12345` does not match the `^oyatie[-_.]/i` regex per ADR-0242 D-6 |
| `audience_type` | `B2C-consumer` | ADR-0244 tenant model (replaces the retired ADR-0221 §M-04 µservice `audience` field per ADR-0242 D-3) |
| `parent_tenant_id` | `null` | Root personal tenant per ADR-0244 D-2 |
| `home_cell` | `cell-us-west-2-a` | Tier 3 data plane cell (ADR-0248 D-4); assigned by shuffle-sharding at signup (ADR-0248 D-7) |
| `dr_cell` | `cell-us-west-2-b` | ADR-0241 D-1 T2 active-passive cross-region pair (Messenger PRD §10.2 declares message-send as T1 for SLO but the home/dr pair semantics come from ADR-0241) |
| `jurisdiction.primary` | `US-CA` | Alice's signup IP geo + verified phone country |
| `sovereign_cloud_pack` | `pack-us-default` | ADR-0240 D-1 default US pack (not a sovereign overlay) |
| `compliance_packs_active` | `[pack-us-default, pack-gdpr-baseline]` | ADR-0251 D-3 baseline packs always active |
| `dr_tier` (for messenger) | `T1` (< 5 min RTO, 0 RPO) | ADR-0241 D-1 + Messenger PRD §10.2 |
| Authoritative identity | Zitadel `personal-idp` (passkey-bound) | Messenger PRD §2.1; ADR-0188 passkey-WebAuthn substrate |
| SPIFFE service principal | `spiffe://oyatie.example/tenant/alice-12345/device/iphone-15-pro-uuid` | ADR-0253 D-7 SPIFFE/SPIRE workload identity |

### 1.2 Bob's tenant

| Property | Value | Source |
|---|---|---|
| `tenant_id` | `tenant-bob-67890` | Same primitive as Alice |
| `audience_type` | `B2C-consumer` | ADR-0244 |
| `home_cell` | `cell-us-east-1-c` | **Different cell from Alice — this is the entire point of the walkthrough; we exercise cross-cell delivery (ADR-0248 D-4 cross-cell traffic permits)** |
| `dr_cell` | `cell-us-east-1-d` | ADR-0241 D-1 |
| `jurisdiction.primary` | `US-NY` | Bob lives in NYC (B2C user-stories §2.2 persona) |
| `sovereign_cloud_pack` | `pack-us-default` | ADR-0240 D-1 |
| Authoritative identity | Zitadel personal-idp | Messenger PRD §2.1 |
| SPIFFE service principal | `spiffe://oyatie.example/tenant/bob-67890/device/iphone-14-uuid` | ADR-0253 D-7 |

### 1.3 Devices

- Alice: iPhone 15 Pro (Secure Enclave; iOS 17.5; Messenger app v1.0.0).
  Per B2C user-stories §2.1 (Alice persona) she also has an M3 MacBook
  Pro and an iPad Pro, but the walkthrough only exercises the iPhone for
  send-side clarity. Her other devices appear as additional LeafNodes in
  her own multi-device group per MLS standard §7.3.
- Bob: iPhone 14 (Secure Enclave; iOS 17.4; Messenger app v1.0.0). Per
  B2C user-stories §2.2 (Bob persona) he also has a Dell XPS 13 work
  laptop and an iPad; out of scope for this walkthrough.

### 1.4 Pre-existing state (before step 1)

The following has already happened (off-stage). Each item is cited
because it is precondition for the walkthrough:

- Both tenants are admitted by `microservices/tenancy/` (ADR-0242 D-5
  bootstrap step 4, then per-user admission per ADR-0244).
- Both users have completed passkey enrolment via WebAuthn Level 3
  (ADR-0188).
- Each device has generated its long-term Ed25519 signing key in the
  Secure Enclave (MLS standard §4.2, iOS row).
- Each device has bound its long-term key to the user's passkey via
  signed attestation (MLS standard §4.3; `DeviceIdentityAttested` audit
  event has been emitted to the tenant's audit stream per ADR-0242 D-4).
- Each device has published a fresh batch of ≥ 100 KeyPackages signed
  under the device's long-term key to the cell-local KeyPackage
  registry (MLS standard §4.7; rotation cadence §4.8).
- Both tenants' Cedar fragments are loaded in their home-cell policy
  engine evaluator (ADR-0243 D-3 + ADR-0246 D-7 hot cache; bundle
  pulled from Tier 2 every 30s per ADR-0248 D-3 constant-work).
- Alice's address book contains Bob's contact (`@bob`) — synced via the
  contact-resolution path of the identity µservice (out of scope; cited
  here only to explain how Alice's client knows Bob exists).
- Both tenants' home cells are Tier 3 data plane cells whose substrate
  stacks are running healthy (ADR-0248 D-4 + Messenger PRD §10.2 SLO).
- The bootstrap cell has long since self-retired (ADR-0248 D-2 step 6,
  ADR-0242 D-5 step 10). Tier 1 bootstrap is not on the request path.

### 1.5 What Alice is about to do

Alice wants to send Bob:

- **A photo** — a JPEG of her dog (3 MB, taken on her iPhone camera).
- **A text caption** — "Just adopted him! Meet Mango."

She has never messaged Bob before. **The MLS group for the Alice↔Bob 1:1
DM does not yet exist.** Step-by-step it will be created on-demand.

---

## 2. Step-by-step walkthrough

### Phase A — Client launch + session

#### Step 1 — Alice taps the Messenger app icon on her iOS home screen

- **Source.** Messenger PRD §1 (the Personal Messenger surface);
  Messenger PRD §3.13 (parity scorecard's "iOS native app" row);
  B2C user-stories §1.3 (100ms response budget; "any UI gesture …
  MUST produce a visible UI response within 100ms").
- **What happens.** iOS schedules the Messenger app launch. The dock
  icon depresses; the system shows the launch screen (an oyatie
  silhouette + Mango-orange gradient) within 50 ms.
- **Why it matters architecturally.** This is the *only* step that
  happens entirely outside oyatie's substrate; everything past this is
  ours.

#### Step 2 — iOS launches the Messenger client process

- **Source.** Messenger PRD §3.10 (multi-platform; iOS native); B2C
  user-stories §1.3 (Offline-first read paths — "Messenger read paths
  work offline" — the local SQLCipher store comes up before network).
- **What happens.** iOS forks the Messenger app. The app's
  cold-start path:
  1. Initialise the local crypto core (`mls-rs` per MLS standard §16).
  2. Open the local SQLCipher database (encrypted with a device-bound
     key in the Keychain).
  3. Hydrate the message-list view from local cache for instant render
     (B2C user-stories §1.3 optimistic UI).
  4. Begin background session validation in parallel with UI render.
- **Why it matters architecturally.** The split between local-hydrate
  and network-validation is what makes the app feel native; it also
  means a network outage doesn't gate the UI.

#### Step 3 — Client validates Alice's session

- **Source.** ADR-0188 (passkey-WebAuthn substrate); Messenger PRD
  §10.4 ("All authenticated writes carry Zitadel JWT"); ADR-0253 D-7
  SPIFFE workload identity binding; ADR-0242 D-2 sub-scope inheritance
  (Alice's principal is `tenant-alice-12345.user`).
- **What happens.**
  1. The app reads the current OIDC refresh token from the Keychain.
  2. If the access token is expired or near-expiry, the client posts
     the refresh token to the cell-local Zitadel endpoint
     (`https://idp.<region>.oyatie.example/oauth2/token`).
  3. Zitadel issues a fresh JWT bound to the device's passkey
     (ADR-0188 acr levels: this is `acr=substantial`, the default for
     warm-launch DM send).
  4. The client also re-asserts the device's SPIFFE SVID (a short-lived
     X.509 cert) via the SPIRE Agent embedded in the Cilium-ambient
     mesh on the device-edge POP side. The SVID's identity is
     `spiffe://oyatie.example/tenant/alice-12345/device/iphone-15-pro-uuid`
     per ADR-0253 D-7.
- **Why it matters architecturally.** Cedar (next step) requires a
  principal claim. The principal IS the SPIFFE SVID + the OIDC subject;
  they are bound at issuance.

#### Step 4 — Client opens HTTP/3/QUIC connection to nearest edge POP

- **Source.** ADR-0253 D-5 (HTTP/3/QUIC client-side default);
  ADR-0253 D-1 (Anycast apex + GeoDNS); Messenger PRD §10.1
  ("Default protocol: HTTP/3 / QUIC at the edge"); MLS standard §6.5.1
  (WebSocket over mTLS; HTTP/3 carrier).
- **What happens.**
  1. The client resolves `api.oyatie.example` via the GeoDNS anycast
     apex; Alice in California resolves to the Cloudflare anycast POP
     in San Jose (~5 ms RTT).
  2. The client opens a QUIC connection (UDP 443) to the POP. QUIC's
     0-RTT resumption (RFC 9001) kicks in because Alice connected
     within the past 24 h; first byte at ~12 ms.
  3. The QUIC session multiplexes HTTPS request/response *and* the
     persistent WebSocket-over-HTTP/3 upgrade for realtime delivery
     (Messenger PRD §10.1 last line; ADR-MSGR-0001 §transport).
- **Why it matters architecturally.** HTTP/3 absorbs Alice's mobile
  network's micro-jitter (LTE handoff to 5G); a TCP-based path would
  re-handshake on every network change. QUIC connection migration
  (RFC 9000 §9) preserves the session.

#### Step 5 — Edge POP runs DDoS / WAF / bot checks

- **Source.** ADR-0253 D-2 (edge layer = CDN + DDoS + WAF + bot
  mitigation); ADR-0253 D-2's stated rationale (the October 2023 HTTP/2
  Rapid Reset attack absorbed at edge before backbone). Implemented as
  a Cloudflare Worker today; migrates to self-hosted Pingora by Year
  3+ per ADR-0253 D-17.
- **What happens.**
  1. Cloudflare's edge runs:
     - Anti-DDoS (volumetric + L7 inspection)
     - WAF rules (OWASP Top 10 + oyatie-custom)
     - Bot management (cf-bot scoring)
     - TLS 1.3 termination (with ML-KEM-768 hybrid key exchange where
       Alice's client supports it; ADR-0253 D-9 + MLS standard §3.5)
  2. Alice's request passes (she's a real iPhone with a valid TLS
     session resumption token; bot score is "clean human").
- **Why it matters architecturally.** None of these checks live in the
  cell. The cell never sees malicious traffic at scale; D-2 says edge
  absorbs volumetric attacks before they reach backbone uplinks.

#### Step 6 — TLS 1.3 handshake completes

- **Source.** ADR-0253 D-3 (TLS 1.3 only; no TLS 1.2 fallback);
  ADR-0253 D-9 (post-quantum hybrid KEX: X25519 + ML-KEM-768);
  MLS standard §3.5 (same PQ posture inside MLS).
- **What happens.** TLS 1.3 handshake completes with cipher suite
  `TLS_AES_128_GCM_SHA256` and key share `X25519MLKEM768` (the hybrid
  group). The negotiated 0-RTT data carries the OIDC JWT.
- **Why it matters architecturally.** Two cryptographic layers, two
  reasons:
  - TLS 1.3 secures the *transport*; the POP sees plaintext HTTP for
    routing.
  - MLS (later in the walkthrough) secures the *message content*; the
    POP and the cell never see plaintext message content.

#### Step 7 — Edge routes request to Alice's home cell

- **Source.** ADR-0248 D-11 (cell routing — `home_cell` lookup);
  ADR-0248 D-7 (shuffle sharding); ADR-0244 (tenant as scoping
  primitive: the JWT's `tenant_id` claim is the routing key);
  ADR-0242 D-7 (`oyatie` itself is a tenant — same code path, no
  special-case).
- **What happens.**
  1. The edge worker reads the JWT's `tenant_id` claim:
     `tenant-alice-12345`.
  2. It consults the cell-routing cache (last refreshed 12s ago from
     the Tier 2 control plane's `tenant_cell_bindings` table per
     ADR-0248 D-3 constant-work cadence).
  3. The cache says `tenant-alice-12345 → cell-us-west-2-a`. The edge
     proxies the QUIC stream to the cell's ingress (a per-cell
     Cloudflare Tunnel + ztunnel pair).
- **Why it matters architecturally.** This is the cellular blast-radius
  boundary. If `cell-us-west-2-a` were down, the edge would route to
  `cell-us-west-2-b` (Alice's `dr_cell`, per ADR-0241 D-1 active-passive
  cross-region pair). The edge has the routing table cached so cell
  failure does not propagate to edge failure (ADR-0248 D-8 static
  stability — 24h target).

#### Step 8 — Cilium ambient mesh validates SPIFFE workload identity

- **Source.** ADR-0148 (Cilium ambient + Istio Ambient layered service
  mesh); ADR-0253 D-7 (SPIFFE/SPIRE workload identity);
  ADR-0044 (mTLS canonical wire posture); ADR-0145 D-3 (mediator-free
  east-west under mTLS).
- **What happens.**
  1. The cell ingress (Envoy waypoint) validates Alice's SPIFFE SVID
     against the SPIRE trust bundle for `cell-us-west-2-a`.
  2. Cilium's L3/L4 layer enforces NetworkPolicy: only `messenger-api`
     pods can receive this traffic (per Kyverno admission policy from
     ADR-0183).
  3. The L7 waypoint extracts the JWT, attaches the principal claim to
     the request context.
- **Why it matters architecturally.** The principal claim is what
  Cedar evaluates against (step 10). Without SPIFFE workload identity
  injected at the mesh layer, Cedar would have to trust the app code
  to assert its principal — exactly the bypass class that ADR-0243's
  prior portfolio state §"Bypass-path temptation" warns against.

#### Step 9 — Messenger cell-local Deployment accepts the request

- **Source.** ADR-0248 D-4 (Tier 3 cell complete substrate stack;
  `messenger` listed); ADR-0131 (per-µservice flat layout);
  Messenger PRD §11 (`direct-messaging` BC is the entry point);
  ADR-0245 (substrate-vs-product layering: messenger is a product
  calling substrates).
- **What happens.** The Envoy waypoint forwards to the
  `messenger-api` Service in `cell-us-west-2-a`. Three pods sit behind
  the Service (HPA scales on CPU + queue depth per Messenger PRD
  §10.3). One pod accepts the request. The pod's Rust binary is the
  `oya-messenger-direct-messaging-api` crate (BC `direct-messaging` per
  Messenger PRD §11).
- **Why it matters architecturally.** The messenger µservice has 18
  bounded contexts (PRD §11). The API pod is the BC entry; everything
  past this is the BC's internal layering (api → usecase → domain →
  kernel → adapter, per ADR-0105 13-layer enum).

#### Step 10 — Policy engine Cedar evaluation: "may Alice open the DM compose surface?"

- **Source.** ADR-0243 D-2 (Cedar is the source of truth for every
  gate); ADR-0246 D-7 (per-cell evaluator with hot cache, < 1 ms p99);
  ADR-0243 D-3 catalogue (the entry for "App-tier authorisation —
  Messenger compose-DM action"); Messenger PRD §10.4 ("Cedar evaluated
  at every action").
- **What happens.**
  1. The `messenger-api` pod constructs a Cedar request:
     ```text
     principal: User::"tenant-alice-12345.user"
     action: Messenger::Action::"OpenComposeDM"
     resource: Messenger::DirectConversation::"new"
     context: {
       hlc: <hlc>,
       device_id: iphone-15-pro-uuid,
       acr: "substantial",
       compliance_packs: ["pack-us-default", "pack-gdpr-baseline"]
     }
     ```
  2. The cell-local Cedar evaluator pod (DaemonSet per ADR-0246 D-7)
     evaluates against:
     - The base personal-messenger permit fragment
       (`microservices/messenger/policy/cedar/personal.cedar`,
       PRD §2.2).
     - The default-deny fragment (ADR-0243 D-3 invariant 6).
     - Alice's tenant overlay (none active beyond pack-gdpr-baseline).
  3. Result: `Permit`. Returned in ~0.4 ms (hot cache hit).
- **Why it matters architecturally.** Cedar evaluated *before any
  business logic runs.* No "we'll check later if this is allowed"
  pattern. Every gate, every time, per ADR-0243 D-2.

### Phase B — Composing and addressing

#### Step 11 — Alice's UI renders the compose-DM surface

- **Source.** B2C user-stories §1.3 (100ms response budget; optimistic
  UI); Messenger PRD §3.4 sticker/emoji panel feature row; Messenger
  PRD §7.1 strive ("native feel; no AI uncanny valley").
- **What happens.** The "New DM" picker appears. Alice's recent
  contacts populate from local cache (Bob is in her contact list, see
  §1.4). The compose-DM surface is one tap away from sending.

#### Step 12 — Alice taps Bob's name

- **Source.** B2C user-stories M-01 story; Messenger PRD §6 user story
  P-1 (parallel canonical example for groups; the DM-of-2 case is the
  base case of P-1).
- **What happens.** Alice's client highlights Bob's contact;
  preview-loads Bob's display name + avatar (already locally cached
  from prior contact sync).

#### Step 13 — Client checks: "do we already have an MLS group with Bob?"

- **Source.** MLS standard §5.1.1 (1:1 group is an MLS group of 2-N
  LeafNodes spanning Alice's devices + Bob's devices); MLS standard
  §4.7 (KeyPackage publication); Messenger PRD §11 BC `e2e-encryption`
  (KeyPackage registry).
- **What happens.** Alice's client queries its local SQLCipher store
  for an MLS group whose `participants` set is `{tenant-alice-12345,
  tenant-bob-67890}`. None exists. The client decides to bootstrap one.

#### Step 14 — Client requests Bob's KeyPackages from the messenger API

- **Source.** MLS standard §5.1.1 step 1; ADR-0145 D-1 (mTLS+gRPC
  east-west); ADR-0248 D-4 (per-cell `e2e-encryption` BC).
- **What happens.** Alice's client issues:
  ```http
  GET /v1/messenger/users/tenant-bob-67890/devices
  Authorization: Bearer <JWT>
  ```
  via the QUIC stream opened in step 4.

#### Step 15 — Cedar gate: "may Alice fetch Bob's KeyPackages?"

- **Source.** ADR-0243 D-3 catalogue (entry: "Webhook subscription
  eligibility (which events can a tenant subscribe to)" — analogous
  pattern for key-package fetch); Cedar fragment
  `microservices/messenger/policy/cedar/personal.cedar`; ADR-0244
  (cross-tenant principals require explicit permit).
- **What happens.** Cedar evaluates:
  - Principal: Alice.
  - Action: `Messenger::Action::"FetchKeyPackagesFor"`.
  - Resource: `User::"tenant-bob-67890"`.
  - Bob has not blocked Alice (the `BlockList` Cedar fragment is
    empty for the pair).
  - Result: `Permit`.

#### Step 16 — Messenger API forwards to identity µservice (cross-cell)

- **Source.** ADR-0248 D-11 (cross-cell traffic permits); ADR-0145 D-1
  (direct sibling-µservice gRPC under mTLS); ADR-0253 D-13 (inter-cell
  mesh via SPIRE federation); ADR-0149 (north-south vs east-west
  boundary).
- **What happens.**
  1. Bob's KeyPackages live in *his* home cell (`cell-us-east-1-c`)
     per the per-cell KeyPackage registry schema in MLS standard §4.7.
  2. The Messenger API in `cell-us-west-2-a` issues a cross-cell gRPC
     call to `identity-api` in `cell-us-east-1-c`. The call's SPIFFE
     SVID identifies the calling workload as
     `spiffe://oyatie.example/cell/us-west-2-a/messenger-api`.
  3. SPIRE federation across the trust bundle for both cells
     (ADR-0253 D-7 + D-13) accepts the SVID.
  4. Cilium NetworkPolicy in `cell-us-east-1-c` permits the
     `messenger-api → identity-api` edge.
- **Why it matters architecturally.** This is the first cross-cell hop.
  Note: the call goes µservice-to-µservice cross-cell, *not*
  cell-to-cell-then-µservice; the cell boundary is a network failure
  domain, not a service boundary.

#### Step 17 — Identity µservice returns Bob's KeyPackages

- **Source.** MLS standard §4.7 KeyPackage publication + §4.10
  validation.
- **What happens.** Identity returns:
  ```json
  {
    "user_id": "tenant-bob-67890",
    "devices": [
      {
        "device_id": "iphone-14-uuid",
        "latest_keypackage": "<MLS-encoded bytes>",
        "platform": "ios",
        "passkey_attestation": "<verified attestation>"
      }
    ]
  }
  ```
  Bob has only one device on file. The KeyPackage is signed by Bob's
  iPhone's long-term Ed25519 key (MLS standard §4.7).

#### Step 18 — Client verifies Bob's KeyPackage signature

- **Source.** MLS standard §4.10 (KeyPackage validation checklist:
  signature, credential, extensions, lifetime); MLS standard §4.6
  (safety numbers — the client will surface "Bob's safety number is X"
  next time Alice opens settings, but does not gate the send).
- **What happens.** Alice's `mls-rs` verifies:
  - Ed25519 signature on `KeyPackageTBS`.
  - Ed25519 signature on the embedded `LeafNodeTBS`.
  - `oyatie_passkey_attestation` extension verifies (Bob's passkey ↔
    Bob's device key chain is intact).
  - Lifetime: KeyPackage's `not_after` is 90 days in the future.
- All four pass.

#### Step 19 — Client constructs the MLS group locally

- **Source.** MLS standard §5.1.1 step 2 (the `mls-rs` API call);
  MLS standard §3.1 (cipher suite 0x0001 default); Messenger PRD §11
  BC `direct-messaging` (`Group` / `MlsLeafNode` entities).
- **What happens.** Alice's `mls-rs` creates the group:
  ```rust
  let group_id = GroupId::generate(); // 32 random bytes
  let mut group = MlsGroup::new(
      group_id,
      GroupConfig::builder()
          .cipher_suite(CipherSuite::Mls128X25519Aes128GcmSha256Ed25519)
          .extensions(vec![
              Extension::oyatie_group_metadata(GroupMetadata {
                  kind: GroupKind::DirectMessage,
                  participants: vec!["tenant-alice-12345", "tenant-bob-67890"],
              }),
          ])
          .build(),
      &alice_signing_key,
  )?;
  ```

#### Step 20 — Client adds Bob's device and Alice's own other devices

- **Source.** MLS standard §5.1.1 step 3-4; MLS standard §7.1 (every
  device is a distinct MLS member); MLS standard §7.3 (per-user device
  group).
- **What happens.** Alice's client calls `group.add_member(bob_keypackage)`
  for each of Bob's devices (1 in our example). It also calls
  `group.add_member(alice_macbook_keypackage)` and
  `group.add_member(alice_ipad_keypackage)` for her own additional
  devices so they receive copies. The group is now epoch 0 with 4
  LeafNodes.

#### Step 21 — Client generates Commit + Welcome messages

- **Source.** MLS standard §5.1.1 step 3; MLS standard §2.3 (Commit
  per epoch; Welcome carries group state for new members).
- **What happens.** `mls-rs` produces:
  - One Commit (signed by Alice's iPhone LeafNode; advances to epoch 1).
  - One Welcome per recipient device (1 for Bob's iPhone, 1 each for
    Alice's MacBook + iPad).

### Phase C — Photo upload (in parallel with group creation)

#### Step 22 — Alice selects the dog photo from her camera roll

- **Source.** Messenger PRD §3.6 (Files, photos, video, drive feature
  row); B2C user-stories §1.3 (optimistic UI); Messenger PRD §3.4
  feature row "Send photo from gallery".
- **What happens.** Alice taps the paperclip icon; iOS photo picker
  appears; Alice picks "Mango.jpg" (3 MB).

#### Step 23 — Client generates per-attachment AES-256-GCM key

- **Source.** MLS standard §13.1 (per-attachment AES-256-GCM);
  MLS standard §13.2 (key wrapping via MLS message); Messenger PRD
  §11 BC `attachments`.
- **What happens.** Client generates 32 random bytes + 12 random nonce
  bytes; encrypts the JPEG inline:
  ```rust
  let attachment_key = OsRng.gen::<[u8; 32]>();
  let nonce = OsRng.gen::<[u8; 12]>();
  let ciphertext = AesGcm256::encrypt(&attachment_key, &nonce, &jpeg_bytes, &aad)?;
  ```

#### Step 24 — Client generates a thumbnail and encrypts separately

- **Source.** MLS standard §13.4 (thumbnail preview; distinct key).
- **What happens.** Client downscales the JPEG to 256x256 (~20 KB),
  generates a new key + nonce, encrypts.

#### Step 25 — Client requests an upload URL via tus.io resumable upload

- **Source.** Messenger PRD §10.1 (file-attachment upload init budget);
  Messenger PRD AC-11 (resumable across forced network drop);
  Messenger PRD §11 BC `attachments` (tus.io multipart, SeaweedFS
  backend); ADR-0211 (in-house SeaweedFS).
- **What happens.**
  ```http
  POST /v1/messenger/attachments/init
  Content-Length: 0
  Tus-Resumable: 1.0.0
  Upload-Length: 3145728
  Upload-Metadata: filename <base64>
  ```
  Returns 201 with `Location: /v1/messenger/attachments/<blob-id>` and
  a Tus-Resumable upload session.

#### Step 26 — Cedar gate: "may Alice upload to SeaweedFS for this conversation?"

- **Source.** ADR-0243 D-3 catalogue (per-action permit); per-tenant
  quota fragment.
- **What happens.** Cedar evaluates:
  - Principal: Alice.
  - Action: `Messenger::Action::"UploadAttachment"`.
  - Resource: `Messenger::AttachmentBlob::"new"`.
  - Context includes `attachment_size: 3145728`, quota check.
  - Result: `Permit`.

#### Step 27 — Client streams ciphertext via QUIC to messenger API which forwards to SeaweedFS

- **Source.** MLS standard §13.3 (blob storage in SeaweedFS;
  server is blind to content); ADR-0211 (in-house SeaweedFS as the
  object store).
- **What happens.** Client streams the 3 MB ciphertext (plus separate
  20 KB thumbnail ciphertext) over the QUIC stream. SeaweedFS in
  `cell-us-west-2-a` stores both blobs; returns blob URLs.

#### Step 28 — Audit chain emission: attachment stored

- **Source.** ADR-0242 D-4 (audit chain emission per state-changing
  action); ADR-0028 (Merkle-sealed audit chain); MLS standard §6.7
  (audit events record hashes + sizes, not plaintext).
- **What happens.** The messenger API appends an `AttachmentStored`
  event to the tenant audit stream:
  ```json
  {
    "event_type": "attachment_stored",
    "tenant_id": "tenant-alice-12345",
    "blob_id_hash": "<sha256>",
    "blob_size": 3145728,
    "content_type_hint": "image/jpeg",
    "hlc_timestamp": "<hlc>",
    "audit_chain_seq": 1932847,
    "merkle_seal_signature": "<ed25519>"
  }
  ```
- **Why it matters architecturally.** Even though the server never sees
  the photo, it records that a 3 MB blob attributed to Alice's tenant
  passed through. This is the tamper-evident record useful for billing,
  capacity planning, and DSAR cascade (ADR-0242 D-4).

### Phase D — Sending the MLS message

#### Step 29 — Client composes the MLS message payload (text + file_ref)

- **Source.** MLS standard §8.1 (send pipeline); MLS standard §13.2
  (key wrapping via MLS); Messenger PRD §11 BC `direct-messaging`
  (`DirectMessage` entity).
- **What happens.** Alice's client builds:
  ```json
  {
    "type": "composite",
    "parts": [
      { "type": "text", "content": { "body": "Just adopted him! Meet Mango.", "format": "markdown" } },
      { "type": "file_ref",
        "blob_url": "https://blob.us-west-2.oyatie.example/v1/blobs/<id>",
        "blob_size": 3145728,
        "content_type": "image/jpeg",
        "encryption": {
          "aead": "AES-256-GCM",
          "key": "<base64 32 bytes>",
          "nonce": "<base64 12 bytes>",
          "aad_metadata_hash": "<base64 32 bytes>"
        },
        "preview": { "thumbnail_blob_url": "...", "thumbnail_encryption": { ... } },
        "metadata": { "filename": "Mango.jpg", "dimensions": [4032, 3024] }
      }
    ],
    "metadata": { "client_id": "msg-uuid-local-1", "client_timestamp": "<iso>" }
  }
  ```

#### Step 30 — Client encrypts as MLS application message

- **Source.** MLS standard §8.1; RFC 9420 §6 (PrivateMessage frame).
- **What happens.**
  ```rust
  let payload_bytes = serde_cbor::to_vec(&payload)?;
  let mls_ciphertext = group.encrypt_application_message(&payload_bytes)?;
  ```
  The ciphertext is AES-128-GCM with the current epoch's application
  secret (MLS standard §3.1).

#### Step 31 — Client attaches HLC timestamp

- **Source.** ADR-0252 D-1 (HLC as canonical clock); ADR-0252 D-4
  (caller-supplied idempotency keys per state-changing action);
  ADR-0252 D-5 (no wall clock for ordering decisions).
- **What happens.** Client reads its local HLC primitive (per
  `oya-shared-time-kernel` per ADR-0252's enforcement preconditions);
  attaches `hlc_timestamp = (physical_now, logical_counter)` to the
  outer frame. Also attaches `idempotency_key = uuid_v4()` so the
  server can dedupe on retry.

#### Step 32 — Client POSTs the MLS group creation + initial message

- **Source.** MLS standard §5.1.1 step 5 + §8.2; Messenger PRD §13.5
  (Protocol surfaces: REST 3.2 + WebSocket).
- **What happens.** Client issues:
  ```http
  POST /v1/messenger/mls/groups
  Authorization: Bearer <JWT>
  Idempotency-Key: <uuid>
  Content-Type: application/json
  body: {
    group_id: <bytes>,
    initial_commit: <mls Commit>,
    welcomes: [
      { recipient_device_id: "iphone-14-uuid", welcome: <bytes> },
      { recipient_device_id: "alice-macbook-uuid", welcome: <bytes> },
      { recipient_device_id: "alice-ipad-uuid", welcome: <bytes> }
    ],
    initial_message: { mls_ciphertext: <bytes>, hlc_timestamp: <bytes> },
    tenant_scope: "personal",
    group_kind: "dm"
  }
  ```

#### Step 33 — Cedar gate: "may Alice create an MLS group + send first message?"

- **Source.** ADR-0243 D-3 catalogue + Messenger Cedar fragment;
  ADR-0246 D-7 (cell-local evaluator < 1 ms).
- **What happens.** Cedar evaluates:
  - Action: `Messenger::Action::"CreateMlsGroup"` AND
    `Messenger::Action::"SendDM"`.
  - Resource: the new DirectConversation.
  - Context: dual-context-isolation check (ADR-0238 +
    Messenger PRD §10.4) — both principals are B2C-personal scope, so
    the `dual-context-isolation.cedar` fragment permits.
  - Result: `Permit`.

#### Step 34 — Server validates MLS message wrapper

- **Source.** MLS standard §6.2 (anti-spam + anti-corruption
  validations: KeyPackage signature, Commit signature, epoch
  monotonicity, group existence, member presence, message size,
  rate limit).
- **What happens.** Server checks:
  - Commit's outer Ed25519 signature verifies under Alice's iPhone
    LeafNode signature key.
  - Epoch 1 ≥ current group epoch (group does not yet exist; epoch is
    fresh).
  - Initial message size (3 MB ciphertext including embedded file_ref
    keys + headers) is under the 10 MB cap (MLS standard §6.2 item 6).
  - Per-device rate limit token-bucket allows the message
    (Messenger PRD §10.4 "message send ≤ 240/min/member").
- All pass. Server **does not decrypt** the ciphertext.

#### Step 35 — Server persists MLS group metadata and message

- **Source.** MLS standard §6.3 (encrypted message storage schema);
  Messenger PRD §10.7 (storage backends per BC — Postgres + Citus
  for `direct-messaging`); ADR-0211 (in-house tech stack).
- **What happens.** Server writes to per-cell Postgres + Citus:
  ```sql
  INSERT INTO mls_group_metadata (group_id, tenant_id, group_kind, current_epoch, cipher_suite, members, ...) VALUES (...);
  INSERT INTO mls_message (message_id, group_id, tenant_id, epoch, sender_leaf_index, content_type, payload, payload_hash, hlc_timestamp, ...) VALUES (...);
  ```
  `tenant_id` is dual-keyed: Alice's tenant is the writer, but the
  group is *shared between two tenants*. The schema's tenant_id is the
  group's creating tenant per MLS standard §6.3; cross-tenant access is
  Cedar-gated on read.

#### Step 36 — HLC advance for ordering

- **Source.** ADR-0252 D-1 (HLC primitive); MLS standard §6.4
  (ordering by `(group_id, epoch, hlc_timestamp, sender_leaf_index)`).
- **What happens.** The cell's HLC primitive merges Alice's observed
  HLC with the cell's local clock; advances the logical counter if
  necessary; persists the merged HLC on the row. This is the canonical
  ordering key for any later catch-up.

#### Step 37 — Audit chain emission: MlsMessageRelayed

- **Source.** ADR-0242 D-4 (audit chain emission, uniform); MLS
  standard §6.7 (encrypted-message-hash only, never plaintext);
  ADR-0028 (Merkle-sealed audit chain); ADR-0252 D-5 (HLC on every
  audit row).
- **What happens.** Server appends to **Alice's tenant audit stream**:
  ```json
  {
    "event_type": "mls_message_relayed",
    "tenant_id": "tenant-alice-12345",
    "group_id_hash": "<sha256>",
    "epoch": 1,
    "payload_hash": "<sha256-of-mls-ciphertext>",
    "payload_size": 3192847,
    "sender_device_id_hash": "<sha256>",
    "recipient_count": 3,
    "hlc_timestamp": "<hlc>",
    "audit_chain_seq": 1932848,
    "merkle_seal_signature": "<ed25519>"
  }
  ```
  Per ADR-0242 D-4 the same Merkle-sealed format applies whether the
  sender is a B2C consumer or the `oyatie` corp tenant; no carve-out.

#### Step 38 — Parallel audit emission to Bob's tenant audit stream

- **Source.** ADR-0242 D-4 (each tenant's audit chain receives events
  attributable to it; cross-tenant events emit to both); ADR-0246 D-3
  catalogue ("Audit-stream selection (which event emits to which
  stream)" is a Cedar-gated policy decision, NOT hard-coded code).
- **What happens.** A Cedar fragment
  `messenger/policy/cedar/audit-stream-selection.cedar` (ADR-0246
  scope) decides:
  - Alice's stream gets the full event (as above).
  - Bob's stream gets a parallel event because the message is
    addressed to him; this is the cross-tenant delivery-side record.
  - Both streams seal the events independently with their own Ed25519
    chain keys (ADR-0028 inheritance).
- **Why it matters architecturally.** This is the key uniform-machinery
  insight from ADR-0242: there is no "DM stream" separate from
  "platform stream." Every tenant has its own stream; events emit
  per-tenant; rollups happen at query time.

#### Step 39 — Server returns 201 to Alice with delivery receipt

- **Source.** MLS standard §8.2 step 6 (`DeliveryReceipt` returned to
  sender); Messenger PRD §10.1 (p99 ≤ 100 ms for 1:1 in-region).
- **What happens.**
  ```json
  {
    "message_id": "<uuid>",
    "epoch": 1,
    "accepted_at": "<hlc>",
    "recipient_count": 3
  }
  ```
  Alice's UI flips the message bubble from "sending..." to "sent ✓"
  (Messenger PRD §10.1 — accepting client UX is identical to WhatsApp
  /iMessage / Signal patterns).

### Phase E — Fan-out to Alice's other devices (intra-cell)

#### Step 40 — Server enqueues delivery tasks

- **Source.** MLS standard §8.2 step 5 (fan-out); Messenger PRD §11
  BC `direct-messaging` (`RealtimeBroadcaster` port impl over
  WebSocket adapter).
- **What happens.** Server enqueues three `MlsDeliveryTask`s — one
  each for `iphone-14-uuid` (Bob), `alice-macbook-uuid`, and
  `alice-ipad-uuid`.

#### Step 41 — Alice's MacBook is connected via WebSocket; receives Welcome immediately

- **Source.** MLS standard §6.5.1 (WebSocket online delivery);
  Messenger PRD §10.1 (presence + WebSocket).
- **What happens.** The cell's WebSocket gateway pushes the Welcome
  frame to Alice's MacBook. `mls-rs` on the MacBook calls
  `MlsGroup::join_from_welcome(welcome)?`, derives epoch 1's secrets
  locally. The MacBook now holds the same group state as the iPhone.

#### Step 42 — Alice's MacBook receives the initial message frame

- **Source.** MLS standard §6.5.1 + §8.3 (receive flow).
- **What happens.** Right after the Welcome, the server pushes the
  initial application message frame to the MacBook. `mls-rs` decrypts;
  the MacBook UI shows the message in the Alice↔Bob conversation
  (multi-device sync per MLS standard §7).

#### Step 43 — Alice's iPad is offline; gets push notification

- **Source.** MLS standard §6.5.2 (offline: push + queue); MLS standard
  §6.5.2 push payload constraints (4 KB cap; hashes only).
- **What happens.** Server emits an APNs push:
  ```json
  {
    "aps": { "alert": { "loc-key": "MESSENGER_NEW_MESSAGE" }, "mutable-content": 1, "content-available": 1 },
    "oyatie": {
      "kind": "mls_message",
      "tenant_id_hash": "<sha256>",
      "group_id_hash": "<sha256>",
      "message_id": "<uuid>",
      "epoch": 1,
      "size": 3192847
    }
  }
  ```
  The iPad's Notification Service Extension wakes up on receipt,
  fetches the full Welcome + message via authenticated REST, decrypts.

### Phase F — Cross-cell delivery to Bob

#### Step 44 — Server routes Bob's Welcome + message to `cell-us-east-1-c`

- **Source.** ADR-0248 D-11 (cross-cell traffic permits); ADR-0145 D-1
  (mTLS+gRPC east-west); ADR-0253 D-13 (inter-cell mesh tunnel via
  SPIRE federation); MLS standard §6.8 (cell + multi-region: ciphertext
  is eligible for cross-region replication within tenant's allowed
  pack residency).
- **What happens.**
  1. The messenger API in `cell-us-west-2-a` issues a gRPC stream to
     `messenger-delivery` in `cell-us-east-1-c`.
  2. SPIFFE SVIDs at both ends verify; SPIRE federation accepts.
  3. Cedar gate on `cell-us-east-1-c` re-validates: "may
     `cell-us-west-2-a` deliver to Bob in this cell?" (ADR-0243
     entry "Cross-cell traffic permits"). Result: `Permit` (both packs
     are `pack-us-default`; intra-pack cross-cell traffic is allowed
     per ADR-0240 D-1).
  4. `cell-us-east-1-c` persists the inbound delivery task in its
     per-cell Postgres + Citus.

#### Step 45 — Cross-cell audit chain emission

- **Source.** ADR-0242 D-4 (uniform audit); ADR-0028 (per-event Merkle
  seal); MLS standard §6.7.
- **What happens.** `cell-us-east-1-c` emits a
  `MlsMessageReceivedCrossCell` event to Bob's tenant audit stream.
  Both cells now hold sealed events for the same logical hop; tamper
  detection would catch any inconsistency.

#### Step 46 — Bob's iPhone is online with WebSocket; receives Welcome

- **Source.** MLS standard §6.5.1; MLS standard §5.1.1 step 7
  (recipient devices process Welcome on receive).
- **What happens.** Server pushes the Welcome frame over the QUIC
  WebSocket. Bob's `mls-rs` calls:
  ```rust
  let mut group = MlsGroup::join_from_welcome(welcome)?;
  ```
  Bob's iPhone now holds the same group state as Alice's iPhone at
  epoch 1.

#### Step 47 — Bob's iPhone receives the application message frame

- **Source.** MLS standard §8.3 (receive flow); MLS standard §6.4
  (ordering by HLC).
- **What happens.** Bob's client decrypts the MLS frame:
  ```rust
  let plaintext_bytes = group.decrypt_application_message(&frame.payload)?;
  let payload: Composite = serde_cbor::from_slice(&plaintext_bytes)?;
  ```
  Bob's UI gets a new-message signal.

#### Step 48 — iOS displays banner notification

- **Source.** Messenger PRD §10 NFR push notification flow;
  MLS standard §6.5.2 (push payload shape — sender name from local
  contact book, not server).
- **What happens.** iOS shows the banner: "Alice — Just adopted him!
  Meet Mango. [📷]". Sender name comes from Bob's local contact book
  (Bob already has Alice in his contacts in our setup, §1.4). The
  banner is composed *on-device* using the locally decrypted plaintext;
  the server never sent the sender name in the push payload.

### Phase G — Bob views the photo

#### Step 49 — Bob taps the banner

- **Source.** B2C user-stories §1.3 (100ms response budget); Messenger
  PRD §3.6 (Files, photos feature row).
- **What happens.** iOS deep-links into the Messenger app, opens the
  Alice↔Bob conversation, scrolls to the new message.

#### Step 50 — Client renders the thumbnail (already locally decrypted)

- **Source.** MLS standard §13.4 (thumbnail preview).
- **What happens.** The 256x256 thumbnail was included in the MLS
  message; client decrypted it inline at step 47. UI renders the
  thumbnail immediately while the full blob loads.

#### Step 51 — Client fetches the full encrypted blob

- **Source.** MLS standard §13.3 (blob storage in SeaweedFS;
  cross-cell fetch); ADR-0211 (in-house SeaweedFS); ADR-0253 D-13
  (cross-cell mesh).
- **What happens.**
  1. Client issues:
     ```http
     GET https://blob.us-west-2.oyatie.example/v1/blobs/<id>
     Authorization: Bearer <JWT>
     ```
  2. This is a **cross-region read** (Bob in us-east-1 reading a blob
     in us-west-2). The edge POP routes to `cell-us-west-2-a`'s blob
     gateway; Cedar gates check Bob has permission to read this blob
     (the MLS message giving him the key + blob URL is the permit
     basis — Cedar fragment `attachment-access.cedar`).
  3. Server streams ciphertext.

#### Step 52 — Client decrypts photo with the embedded attachment key

- **Source.** MLS standard §13.1.
- **What happens.**
  ```rust
  let jpeg_bytes = AesGcm256::decrypt(&attachment_key, &nonce, &ciphertext, &aad)?;
  ```
  Client decodes the JPEG; UI shows Mango in full resolution.

#### Step 53 — Bob taps the photo to fullscreen

- **Source.** Messenger PRD §3.6 (photo gallery viewer); B2C
  user-stories §1.3 (accessibility — alt-text expected on every
  image).
- **What happens.** Standard iOS photo-viewer transition. Bob smiles.

### Phase H — Bob reacts with a heart

#### Step 54 — Bob long-presses the message; emoji picker appears

- **Source.** Messenger PRD §3.2 (reactions — emoji feature row; "tap
  to react" UX); Messenger PRD §11 BC `direct-messaging`
  (`MessageReactionAdded` event).
- **What happens.** Standard iOS context menu with the quick-reaction
  picker.

#### Step 55 — Bob taps the ❤️ emoji

- **Source.** Messenger PRD §3.2 (reactions); MLS standard §8.6 / §8.7
  pattern (control messages — reactions are application-layer control
  messages, encrypted as ordinary MLS payloads).

#### Step 56 — Bob's client constructs reaction control message

- **Source.** Messenger PRD §13.1 (Workflow events produced —
  `MessageReactionAdded`); MLS standard §6.6 (read-receipt style
  control messages; same pattern).
- **What happens.**
  ```json
  {
    "type": "reaction",
    "reacts_to_message_id": "<uuid-of-Alice-message>",
    "emoji": "❤️",
    "action": "add",
    "metadata": { "client_id": "react-uuid-local", "client_hlc": "<hlc>" }
  }
  ```

#### Step 57 — Client encrypts the reaction as MLS application message

- **Source.** MLS standard §8.1 (same encrypt path as text); MLS
  standard §3.1 (cipher suite 0x0001).
- **What happens.** `group.encrypt_application_message(reaction_bytes)`
  in `mls-rs`. Same epoch (still epoch 1; no member-change has
  triggered a Commit).

#### Step 58 — Client POSTs the reaction to Bob's home cell

- **Source.** MLS standard §8.2 server-side flow; Messenger PRD §13.5
  Protocol surfaces.
- **What happens.**
  ```http
  POST /v1/messenger/mls/groups/<group_id>/messages
  Authorization: Bearer <JWT>
  Idempotency-Key: <uuid>
  body: { mls_ciphertext: <bytes>, hlc_timestamp: <bytes> }
  ```

#### Step 59 — Cedar gate on Bob's cell

- **Source.** ADR-0243 D-2 (every gate, every time).
- **What happens.** Cedar evaluates `Messenger::Action::"SendReaction"`
  with Bob as principal, the existing group as resource. `Permit`.

#### Step 60 — Server validates and persists

- **Source.** MLS standard §6.2 + §6.3.
- **What happens.** Server in `cell-us-east-1-c` validates the outer
  signature (Bob's iPhone LeafNode), epoch 1 matches current epoch,
  persists in Bob's home cell's mls_message table. The reaction is
  semantically content-addressed to Alice's message but the storage
  row is in Bob's cell — Bob initiated.

#### Step 61 — Audit chain emission for reaction

- **Source.** ADR-0242 D-4; MLS standard §6.7.
- **What happens.** `MessageReactionAdded` event sealed to both Bob's
  and Alice's tenant audit streams. Same Merkle-sealed format.

#### Step 62 — Cross-cell delivery: reaction fan-out to Alice's cell

- **Source.** ADR-0248 D-11; ADR-0253 D-13.
- **What happens.** Same mechanism as step 44, in reverse:
  `cell-us-east-1-c` → `cell-us-west-2-a` gRPC; SPIFFE + SPIRE
  federation accepts; Cedar permits.

#### Step 63 — Alice's iPhone (still online) receives reaction frame

- **Source.** MLS standard §6.5.1 + §8.3.
- **What happens.** Server pushes via WebSocket. Alice's `mls-rs`
  decrypts; the UI:
  - Updates the message bubble with the ❤️ reaction badge.
  - Shows a brief animation (Messenger PRD §7.1 strive — native feel).

#### Step 64 — Alice's MacBook and iPad also receive (multi-device sync)

- **Source.** MLS standard §7 (multi-device sync); MLS standard §6.5.1
  (online) and §6.5.2 (offline push for iPad).
- **What happens.** All three of Alice's devices show the heart on her
  message simultaneously.

#### Step 65 — Alice receives a subtle notification

- **Source.** Messenger PRD §11 BC `notifications` (mention inbox +
  push); Messenger PRD §10.4 ("reactions ≤ 600/min/member" rate limit
  context: reactions are noticeable but not push-noisy by default).
- **What happens.** A small heart icon appears in Alice's notification
  inbox (in-app); no audible push on iOS unless Alice has enabled
  reaction-push (off by default for her).

### Phase I — Closing receipts and observability

#### Step 66 — Bob's client emits a read-receipt control message

- **Source.** MLS standard §6.6 (read receipts opt-in, encrypted).
- **What happens.** Because Bob has read receipts enabled (1:1 default
  is on per MLS standard §6.6), his client encrypts and posts:
  ```json
  { "kind": "read_receipt", "message_id": "<uuid>", "read_at": "<hlc>" }
  ```
  Same MLS application-message path. Server fans out to Alice's
  devices. Alice's UI flips the message bubble check-marks from
  "delivered (✓✓)" to "read (✓✓ blue)".

#### Step 67 — Server-side delivery state cleanup

- **Source.** MLS standard §6.3 (retention: until delivered to all
  recipients + 7 days grace).
- **What happens.** Server marks each device's `delivered_to` set as
  complete for both the original message and the reaction. Sets
  `expires_at = now + 7d`. A background worker (per ADR-0252 D-7
  per-cell cron with jitter) will reap rows past their TTL.

#### Step 68 — Observability metric emission

- **Source.** ADR-0148 §observability layer integration; Messenger PRD
  §10.2 OpenSLO reports; ADR-0253 D-7 mesh-emitted spans;
  microservices/observability per ADR-0130 + ADR-0131.
- **What happens.** Throughout the flow, the Cilium ambient mesh +
  Envoy waypoints emitted:
  - W3C Trace Context spans linking all hops (Alice's iPhone → edge →
    `cell-us-west-2-a` messenger-api → identity → SeaweedFS → audit
    chain → `cell-us-east-1-c` messenger-delivery → Bob).
  - RED metrics (rate, errors, duration) on every span.
  - The per-cell OpenSLO budget burn for the `message-send` SLO
    advanced by 0% (within the p99 budget).

#### Step 69 — FinOps cost attribution

- **Source.** ADR-0242 D-4 (cost attribution; per-action; per-deepest
  sub-scope); ADR-0174 (FinOps + sustainability tagging).
- **What happens.** The cost-attribution event chain charges:
  - `tenant-alice-12345` for: outbound 3 MB blob upload, MLS group
    create, MLS message send, audit-chain emission ×2, cross-cell
    bandwidth (50/50 split with Bob since the message is mutual).
  - `tenant-bob-67890` for: inbound blob fetch, reaction send,
    audit-chain emission ×2, his cell's storage of the reaction.
- Because both are personal-tier B2C tenants on a free-tier
  Cedar permit, neither sees a bill; the costs roll up to
  `oyatie-corp` per ADR-0242 D-4 ("`oyatie.*` operational services …
  Cost attribution …").

#### Step 70 — Background HLC heartbeat closes the conversation

- **Source.** ADR-0252 D-1 (HLC); MLS standard §10.1 (forward secrecy
  via per-epoch commit); MLS standard §5.4 (Update key — proactive
  rekey cadence; monthly per §10).
- **What happens.** No active action; the group sits at epoch 1 until
  a member-change or scheduled update. Forward secrecy is in force:
  even if Alice's iPhone is later seized and unlocked, the previous
  epoch's secrets cannot be derived from the current state (MLS
  standard §10.1).

### Phase J — Extras (intern bonus reading)

#### Step 71 — Key-transparency log consistency check (next launch)

- **Source.** Messenger PRD §4.2 ("CONIKS-class transparency log");
  MLS standard §17 (threat model: equivocation detection).
- **What happens (later).** On Alice's next app launch, the client
  re-fetches Bob's KeyPackage transparency proof and checks it against
  the gossiped log root. A server that equivocated (gave Alice a
  different KeyPackage than it gave others for Bob) is detectable
  here. No alert fires today; the architecture would surface it
  client-side via "Bob's safety number changed."

#### Step 72 — Sealed-sender future (M02)

- **Source.** Messenger PRD §4.2 final paragraph ("Sealed sender")
  and ADR-MSGR-0002 §metadata-minimisation.
- **What happens.** At M02 the outer frame will be re-encrypted to the
  recipient device's wrap key so the server doesn't see sender
  identity. This walkthrough exercises the M01 baseline; M02 changes
  step 32 + step 34's wrapper but leaves all other steps unchanged.

#### Step 73 — DSAR readiness

- **Source.** ADR-0242 D-4 (uniform DSAR cascade);
  Messenger PRD FR-57 + FR-58.
- **What happens (if Alice ever requests).** Alice's `GDPR Art. 17`
  request would tombstone her `author_id` across both tenant audit
  streams, purge ciphertext + attachment blobs, retain Merkle-sealed
  audit rows for legal floor. Bob's audit row mentioning Alice's
  pseudonym is rewritten via the same primitive. Because every audit
  emission carries `payload_hash` + `tenant_id` and not plaintext, the
  cascade is mechanical.

---

## 3. Architecture diagram

```
                                                  PLANETARY APEX
   Alice's iPhone (us-west, CA)                 [Anycast DNS + GeoDNS]                 Bob's iPhone (us-east, NY)
   --------------------------                   per ADR-0253 D-1                      ------------------------
        |                                                                                       |
        | (1) tap icon                                                                          | (49) tap banner
        | (2) cold-start                                                                        |
        | (3) OIDC/SPIFFE                                                                       |
        | (4) HTTP/3 / QUIC                                                                     |
        v                                                                                       v
+----------------------------------+                                              +----------------------------------+
| Cloudflare POP (San Jose)        |                                              | Cloudflare POP (New York)        |
|  - DDoS / WAF / Bot (ADR-0253 D-2)|                                             |  - DDoS / WAF / Bot              |
|  - TLS 1.3 + X25519MLKEM768       |  ADR-0253 D-3 + D-9 (PQ hybrid)             |  - TLS 1.3 + X25519MLKEM768      |
+----------------+-----------------+                                              +----------------+----------------+
                 |                                                                                |
                 | (7) route by tenant→home_cell                                                  | (49) deep-link
                 |     ADR-0248 D-11 + D-7 shuffle-sharding                                       |
                 v                                                                                v
   ============================== cell-us-west-2-a (Tier 3) ============================       cell-us-east-1-c (Tier 3)
   |                                                                                 |       |                                  |
   |   Cilium ambient L3/L4 + Istio Ambient L7 waypoint   (ADR-0148, ADR-0253 D-7)   |       |   same substrate stack           |
   |   SPIFFE SVID validation                                                        |       |                                  |
   |                                                                                 |       |                                  |
   |   +----------------+    Cedar evaluator    +-------------------+                |       |   +----------------+             |
   |   | messenger-api  |<--(0.4ms hot cache)-->| policy-engine     |                |       |   | messenger-     |             |
   |   |  (BC: direct-  |                       |  (ADR-0246 D-7)   |                |       |   |  delivery-api  |             |
   |   |   messaging,   |                       |  Cedar fragments  |                |       |   |                |             |
   |   |   attachments, |                       |   per ADR-0243    |                |       |   +-------+--------+             |
   |   |   e2e-encrypt) |                       +-------------------+                |       |           |                      |
   |   +-------+--------+                                                            |       |           |                      |
   |           |                                                                     |       |           |                      |
   |           +--(step 16, 44, 62)----- cross-cell gRPC via SPIRE federation -------------------------->+                      |
   |           |        ADR-0145 D-1 mTLS + ADR-0253 D-13                            |       |           |                      |
   |           v                                                                     |       |           v                      |
   |   +----------------+                                                            |       |   +----------------+             |
   |   | SeaweedFS      |  <-- encrypted blob storage (ADR-0211 in-house)            |       |   | identity / KP  |             |
   |   |  (attachments) |      MLS standard §13.3                                    |       |   |  registry      |             |
   |   +----------------+                                                            |       |   +----------------+             |
   |                                                                                 |       |                                  |
   |   +----------------+      +----------------+      +----------------+            |       |   +----------------+             |
   |   | Postgres+Citus |      | Valkey         |      | audit-chain    |            |       |   | audit-chain    |             |
   |   |  mls_message,  |      | (presence,     |      |  Merkle-sealed |            |       |   | (Bob's stream) |             |
   |   |  mls_group_md  |      |  realtime,     |      |  (ADR-0028)    |            |       |   |                |             |
   |   +----------------+      |  delivery state)      | per ADR-0242   |            |       |   +----------------+             |
   |                           +----------------+      |   D-4 uniform  |            |       |                                  |
   |                                                   +----------------+            |       |                                  |
   |                                                                                 |       |                                  |
   ===================================================================================       ====================================
              ^                                                                                       ^
              | (Tier 2 control plane cells publish snapshots every 30s)                              |
              | per ADR-0248 D-3 constant-work                                                        |
              |                                                                                       |
              +---<-- Tier 2 control-plane cells (tenancy / identity / policy-engine /     ---->-----+
                       audit-chain / cell / governance / compliance / observability /
                       cloud-iac / consent-graph / intelligence-control-plane)
                       per ADR-0248 D-3 — 2-3 Tier 2 cells per region, active-active

                                                                                              [Tier 1 bootstrap cell — self-retired
                                                                                               long before this walkthrough; per ADR-0248 D-2]

Cryptographic layering (top to bottom):
   1. MLS application encryption  (RFC 9420 cipher suite 0x0001)  — ends at recipient device
   2. TLS 1.3 (X25519+MLKEM768)                                    — ends at edge POP / inter-mesh
   3. SRTP/DTLS for media (not used in this walkthrough; same epoch key)
   4. AES-256-GCM per-attachment                                   — ends at recipient device
   5. Server-side audit-chain Ed25519 + Merkle seal                — at audit chain
```

---

## 4. What an intern learns from this walkthrough

### 4.1 Every keystone ADR has been applied at least once

| ADR | Where exercised | Step(s) |
|---|---|---|
| ADR-0242 (oyatie-is-a-tenant) | Same machinery applies to oyatie-corp as Alice/Bob; reserved namespace check at signup; uniform DSAR + audit + cost machinery | 1.1, 1.2, 37, 38, 69, 73 |
| ADR-0243 (Cedar as universal gate) | Every gate evaluated through Cedar — compose, fetch KeyPackages, create group, send, upload, fetch blob, react, cross-cell traffic | 10, 15, 26, 33, 44, 51, 59 |
| ADR-0244 (tenant as universal scoping primitive) | Tenant ID is the routing + audit + Cedar principal key end-to-end | 1.1, 1.2, 7 |
| ADR-0245 (substrate vs product layering) | Messenger calls Identity, Policy-Engine, Audit-Chain, SeaweedFS as substrate µservices; never cross-product imports | 16, 17, 27, 37, 38 |
| ADR-0246 (policy-engine substrate promotion) | Per-cell policy engine evaluator with hot cache < 1 ms; bundles pulled from Tier 2 every 30 s | 10, 15, 26, 33, 38, 44, 51, 59 |
| ADR-0247 (self-hosting/self-modification) | Implicit: every CI lane that authored these crates and Cedar fragments goes through the agentic foundry pipeline (out of scope for this walkthrough but referenced) | (cited only) |
| ADR-0248 (Amazon-shape cellular architecture) | Tier 3 data plane cells (Alice's + Bob's home cells); Tier 2 control plane snapshot pull; bootstrap cell long retired; shuffle-sharding; cross-cell traffic permit | 1.1, 1.2, 7, 9, 16, 44, 62 |
| ADR-0251 (compliance pack cell certification levels) | Pack-us-default + pack-gdpr-baseline overlay on Alice's tenant | 1.1 |
| ADR-0252 (HLC + idempotency keys + no distributed locks) | HLC on every audit row; idempotency key on every state-changing POST; ordering by HLC | 31, 32, 36, 37, 67, 70 |
| ADR-0253 (network topology — Anycast + edge + Cilium ambient + SPIFFE + HTTP/3 + PQ hybrid) | HTTP/3 client; Cloudflare edge POP; Cilium ambient mesh; SPIRE federation cross-cell; X25519MLKEM768 hybrid KEX | 4, 5, 6, 7, 8, 16, 44, 62 |
| ADR-0241 (DR + business continuity) | Alice's `dr_cell` = us-west-2-b; messenger is T1 (< 5 min RTO); failure scenarios §5 use this | 1.1, 1.2 (and §5) |
| ADR-0148 (Cilium ambient + Istio ambient layered mesh) | Layer 3/4 + Layer 7 mesh decisions on every cell boundary | 8, 16, 44, 62 |
| ADR-0188 (passkey-WebAuthn substrate) | JWT acr level + passkey attestation chain → MLS device key | 3, 18 |
| ADR-0211 (in-house tech stack) | SeaweedFS as the encrypted-blob store | 27, 51 |
| ADR-0238 (dual-context isolation invariant) | Cedar dual-context-isolation check on every send | 33 |
| ADR-0028 (Merkle-sealed audit chain) | Every audit event Ed25519-signed + Merkle-chained | 28, 37, 38, 45, 61 |
| ADR-MSGR-0001 (LiveKit + WebRTC canonical) | Not exercised in DM walkthrough (would cover voice); referenced for huddles parity | (cited only) |
| ADR-MSGR-0002 (MLS as E2E canonical) | The entire MLS layer | 13–47 inclusive |
| ADR-MSGR-0003 (Matrix + ActivityPub federation) | Not exercised (intra-oyatie); referenced for future federation | (cited only) |

### 4.2 Every Messenger PRD BC has been touched (by at least one step)

| BC | Step(s) | Notes |
|---|---|---|
| `direct-messaging` | 9, 13, 19, 29, 32, 35 | Primary BC for this walkthrough |
| `e2e-encryption` | 13–21, 30, 34 | KeyPackage fetch + MLS group create + encrypt |
| `attachments` | 22–28, 51, 52 | Photo upload + cross-region fetch + decrypt |
| `multi-device-sync` | 20, 41–43, 64 | Alice's three devices, Bob's one |
| `notifications` | 43, 48, 65 | APNs push; in-app reaction badge |
| `presence-status` | (passive — Bob is "online" when notification arrives) | Implicit but cited |
| `archive-retention` | 67 | TTL set on delivered rows |
| `search` | (out of scope) | Cited in step 73 for DSAR |
| `dlp` | (out of scope for B2C) | DLP is B2B-only per Messenger PRD §5.11 |
| `federation` | (out of scope) | Both endpoints are intra-oyatie |
| `workflow-triggers` | (out of scope for B2C personal DM) | Trigger only on B2B per PRD §5.4 |
| `voice-calls` / `video-calls` / `huddles` | (out of scope; cited) | Referenced as future-extension |
| `stickers-emoji` | 55 (reaction emoji is the stock Unicode set) | One reaction sent |
| `threads` / `channels` | (out of scope for 1:1 DM) | Referenced in §4.4 |
| `group-messaging` | (out of scope for 1:1) | Same code path scaled |

Across all 18 BCs, the **direct-messaging + e2e-encryption +
attachments + multi-device-sync + notifications** quintet is the
minimum vertical slice exercised. PRD §11 invariants — port-in-kernel,
inward-only flow, no cross-product imports — are satisfied because
messenger never reaches into mail, calendar, drive, community, etc.;
substrates are called via gRPC SDK.

### 4.3 Cross-cell traffic pattern visible

- **Step 16** — Alice's cell calls Bob's identity µservice in his cell.
- **Step 44** — Alice's cell delivers MLS Welcome + message to Bob's
  cell.
- **Step 51** — Bob's client fetches the blob from Alice's cell.
- **Step 62** — Bob's cell delivers reaction to Alice's cell.

Each crossing is a separately Cedar-gated, separately mTLS-attested,
separately SPIFFE-validated, separately audit-emitted hop. **No
implicit cross-cell calls; every crossing is policy and identity
gated.** (ADR-0248 D-11.)

### 4.4 Cedar-as-universal-gate visible

Cedar evaluations in this walkthrough:

| Step | Action | Result | Time |
|---|---|---|---|
| 10 | OpenComposeDM | Permit | ~0.4 ms |
| 15 | FetchKeyPackagesFor | Permit | ~0.4 ms |
| 26 | UploadAttachment | Permit | ~0.4 ms |
| 33 | CreateMlsGroup + SendDM + DualContextIsolation | Permit | ~0.6 ms (compound) |
| 38 | AuditStreamSelection (decides which streams receive) | Both streams | ~0.4 ms |
| 44 | CrossCellTrafficPermit (us-west-2-a → us-east-1-c) | Permit | ~0.4 ms |
| 51 | AttachmentAccess (Bob fetching Alice's blob) | Permit | ~0.4 ms |
| 59 | SendReaction | Permit | ~0.4 ms |
| 62 | CrossCellTrafficPermit (us-east-1-c → us-west-2-a) | Permit | ~0.4 ms |

**Nine Cedar evaluations.** Zero policy decisions in code. (ADR-0243
D-2.)

### 4.5 MLS E2EE visible

- **Group creation** with TreeKEM (MLS std §5.1 + §2.3): epoch 0 → 1.
- **Application messages** encrypted with epoch 1's application secret
  (MLS std §3.1, cipher suite 0x0001 = MLS_128_DHKEMX25519_AES128GCM_-
  SHA256_Ed25519).
- **Welcome messages** carry epoch 1 state to new joiners (Bob's
  iPhone, Alice's other devices).
- **Per-attachment AES-256-GCM** key wrapped in the MLS application
  message — server is blind to photo (MLS std §13.1–§13.3).
- **Forward secrecy** via per-epoch commit (MLS std §10.1).
- **Post-compromise security** ready on next epoch rotation (MLS std
  §10.2).
- **Server is an untrusted relay** — cannot decrypt; can observe size,
  membership, frequency, timing (MLS std §6.1).

### 4.6 Per-tenant isolation visible

- Alice's audit stream and Bob's audit stream are separate; the same
  logical event emits to both (step 37 + 38). Tenant-scoped queries
  on either stream see only that tenant's view (ADR-0242 D-4).
- The MLS group spans both tenants' devices but the group metadata
  schema's `tenant_id` (the creating tenant) plus per-row Cedar
  permits scope read access.
- Cost attribution is per-tenant (step 69).

### 4.7 Audit chain visible

Seven audit events emitted across the walkthrough:

| Step | Event | Streams |
|---|---|---|
| 28 | AttachmentStored | Alice's |
| 37 | MlsMessageRelayed (initial) | Alice's |
| 38 | MlsMessageReceivedCrossTenant (initial) | Bob's |
| 45 | MlsMessageReceivedCrossCell (initial) | Bob's (cell receipt) |
| 61 | MessageReactionAdded | Bob's and Alice's |
| 67 | DeliveryAck / RetentionScheduled | Both (system-internal) |

Every event Merkle-sealed (ADR-0028); each event carries `hlc_timestamp`
(ADR-0252 D-1); none carry plaintext (MLS std §6.7).

### 4.8 Observability emission visible

W3C Trace Context spans propagate from edge → cell ingress → API →
substrate calls → cross-cell → recipient cell → recipient device.
RED metrics per span. OpenSLO budget burn computed per Messenger PRD
§10.2's monthly availability target.

### 4.9 HTTP/3 + Cilium + SPIFFE visible

- HTTP/3 at the client edge (steps 4–6).
- Cilium ambient L3/L4 + Istio ambient L7 waypoints on every
  intra-cell hop (step 8 + 16 + 44 + 62).
- SPIFFE SVID on every workload; SPIRE federation on every cross-cell
  hop (steps 3, 8, 16, 44, 62).

---

## 5. Failure scenarios — what could go wrong, and how the architecture recovers

Ten failure injections, each tagged with the responsible primitive
and the ADR that codifies the recovery.

### 5.1 Alice's home cell (`cell-us-west-2-a`) goes down between step 7 and step 8

- **Source.** ADR-0241 D-1 (T1 DR tier: < 5 min RTO, 0 RPO); ADR-0248
  D-8 (static stability; 24 h target).
- **Recovery.** The edge POP's cell-routing cache, refreshed every
  12 s, observes the cell health check failing. Within ~30 s, the
  cache flips: `tenant-alice-12345 → cell-us-west-2-b` (Alice's
  `dr_cell`). New connections from Alice route to us-west-2-b, which
  has been kept warm via active-passive replication (ADR-0241 D-4). The
  bootstrap-cell escape hatch is *not* used (the bootstrap cell
  retired long ago, ADR-0248 D-2).
- **What Alice sees.** A 30 s spinner; reconnect; her UI re-syncs from
  us-west-2-b's Postgres replica. Her in-flight message (steps 31–32)
  was idempotency-keyed (ADR-0252 D-4), so a retry on the new cell is
  safe to dedupe.

### 5.2 Cedar fragment registry temporarily slow

- **Source.** ADR-0243 D-7 (in-cell cache + DaemonSet evaluator);
  ADR-0246 D-7 (per-cell evaluator pulls bundles every 30 s; cache
  fallback for up to 24 h staleness).
- **Recovery.** Even if the Tier 2 control plane's policy-engine bundle
  publisher is degraded, every Tier 3 cell holds the last good bundle
  in memory and on local disk. The 24 h static-stability budget covers
  this.
- **What Alice sees.** Nothing. Evaluations continue at < 1 ms.

### 5.3 MLS Commit fails mid-flight (network drop during step 32)

- **Source.** ADR-0252 D-4 (caller-supplied idempotency keys);
  Messenger PRD §6 story P-1 edge case ("MLS commit fails (network
  drop during commit): client retries with exponential backoff").
- **Recovery.** The client's `Idempotency-Key: <uuid>` lets the server
  dedupe. Client retries with exponential backoff. If the Commit was
  already accepted (server returned 201 but the response was dropped),
  the retry receives 200 OK with the same delivery receipt.

### 5.4 Network partition between Alice's cell and Bob's cell

- **Source.** ADR-0248 D-8 (static stability — cells must function
  with last-known state); ADR-0252 D-3 (saga-based cross-µservice
  coordination per ADR-0222, never distributed locks).
- **Recovery.** The cross-cell delivery task (step 44) is enqueued in
  Alice's cell with an at-least-once delivery semantic. The retry
  saga (per ADR-0222) replays from the outbox every 10 s. When the
  partition heals, delivery completes. Bob sees the message late but
  intact.
- **What Bob sees.** The push notification may arrive when the
  partition heals; the message itself is exactly what Alice sent.

### 5.5 Bob's iPhone is offline for 3 days

- **Source.** MLS standard §6.5.2 (offline: push + queue); MLS
  standard §6.3 (Welcome retention 30 days; PrivateMessage retention
  until delivered + 7 days grace).
- **Recovery.** Server holds the Welcome + initial message + reaction.
  When Bob re-connects, he pulls them via authenticated REST. MLS's
  epoch ordering ensures he processes in order.

### 5.6 Bob's iPhone's MLS state corrupts (app data loss)

- **Source.** MLS standard §5.7 (Group state recovery); MLS standard
  §4.7–§4.8 (KeyPackage publication; the cell-local registry has
  ≥ 100 unused KPs per device).
- **Recovery.** Bob's reinstall publishes new KeyPackages signed under
  a freshly-generated long-term key (HW-backed in Secure Enclave); his
  passkey rebinds. He re-joins existing groups via a member-add
  Commit from any other member (here, Alice). The audit chain records
  the device re-pair as `DeviceIdentityAttested` (per MLS std §4.3
  step 5).

### 5.7 SeaweedFS in Alice's cell loses a blob (silent corruption)

- **Source.** ADR-0211 (in-house tech stack; SeaweedFS replication
  policy); ADR-0028 (audit-chain seal on `AttachmentStored`).
- **Recovery.** SeaweedFS replicates per volume (3x default).
  Bob's fetch (step 51) may go to a healthy replica. If all three
  replicas were lost (catastrophic), the audit-chain's `payload_hash`
  in step 28 surfaces the discrepancy at the next integrity scan; the
  client receives a 410 Gone and the message UI shows "attachment
  unavailable." (No silent data loss — INV-IDEMPOTENCY guarantees
  reproducibility via the sealed audit chain.)

### 5.8 Audit chain Merkle seal verification fails on a periodic audit

- **Source.** ADR-0028 (per-period Merkle seal; per ADR-0242 D-4 the
  chain is replicated per sovereign overlay); ADR-0247 §self-modification
  doctrine (chain tamper triggers incident-response workflow).
- **Recovery.** A failed seal verification triggers `oyatie.security.
  incident-response` (T1 per ADR-0241 D-1). The tamper window is
  bounded by the verification cadence (per ADR-0028, every 5 min for
  active streams). Investigation begins immediately. The replicated
  copies in other sovereign overlays give a triangulation source.

### 5.9 Cloudflare edge POP global outage

- **Source.** ADR-0253 D-2 (edge migration path — Year 3+ self-hosted
  Pingora POPs); ADR-0241 D-6 ("Provider failure" semi-annual drill).
- **Recovery (today).** Anycast DNS routes around the affected POPs;
  Cloudflare is itself multi-POP. Genuine global Cloudflare outages
  (rare but observed in 2024-2025) require BGP-anycast fallback to
  AWS Route 53 + ALB, which Year 1 implements as the explicit
  second-provider escape hatch (ADR-0253 D-2 final paragraph).
- **Recovery (Year 3+).** Self-hosted Pingora POPs in dedicated
  dev-tools-cells per ADR-0253 D-17.

### 5.10 KeyPackage exhaustion (Bob's KP registry runs out of unused KPs)

- **Source.** MLS standard §4.7 (registry maintains ≥ 100 unused KPs
  per device); §4.8 (low-water alert at < 20).
- **Recovery.** Server worker emits a push to Bob's device requesting
  fresh KP publication. If Bob's device is offline beyond the
  emergency threshold, Alice's group-create (step 14) receives a 503
  with `Retry-After`; Alice's UI surfaces "Bob hasn't been online
  recently; your message will be queued and delivered when he comes
  back." (Reflects iMessage / WhatsApp UX for stale recipients.)

### Bonus failure 5.11 — Cell-pair correlated failure (us-west-2-a AND us-west-2-b both down)

- **Source.** ADR-0241 D-6 ("Multi-cell catastrophic loss"; tabletop
  semi-annual); ADR-0248 D-7 (shuffle sharding bounds blast radius).
- **Recovery.** Active-active cross-region: tenant's traffic re-routes
  to its cross-region warm pair (per ADR-0241 D-4 replication shape).
  Shuffle sharding ensures that even with 2 cells gone, the platform
  as a whole degrades by a small fraction; Alice's tenant is one of
  N tenants in the cell-pair, not all.

### Bonus failure 5.12 — Alice's device clock skew exceeds HLC uncertainty budget

- **Source.** ADR-0252 D-1 (HLC uncertainty bound); ADR-0252 D-7
  (per-cell cron with jitter).
- **Recovery.** The HLC primitive merges client-observed time with
  cell-observed time; if the client's wall clock is ≥ 60 s off (a
  configurable uncertainty bound), the server uses its own HLC
  reading authoritatively. The message still orders correctly relative
  to other messages in the group because the server's HLC is the
  causal source of truth (ADR-0252 D-1).

### Bonus failure 5.13 — Alice tries to send a personal DM that crosses into a Professional channel context

- **Source.** ADR-0238 (dual-context-isolation invariant);
  Messenger PRD §2.3 ("kernel-isolated"); Cedar fragment
  `policy/dual-context-isolation.cedar`.
- **Recovery.** Cedar refuses at step 33. The Cedar `forbid` rule
  forbids any send where `principal.audience_type == "B2C-consumer"`
  and `resource.parent_tenant.audience_type == "B2B-tenant"` (or
  vice-versa). Alice's UI shows "This message can't be sent across
  workspace boundaries." No data crossed.

---

## 6. Comparison — this walkthrough vs Slack / iMessage / WhatsApp

Why oyatie's path is hyperscaler-grade, not just feature-parity.

### 6.1 vs Slack

| Concern | Slack | oyatie |
|---|---|---|
| E2E posture | None for messages (TLS only). Slack admins + Salesforce ops can read message bodies. | MLS RFC 9420 (group-native FS + PCS). Server is an untrusted relay (MLS std §6.1). |
| Tenant isolation | Per-tenant Postgres shard. Single audit log (shared substrate). | Per-tenant audit stream + per-cell isolation + Cedar overlays. Both tenants in this walkthrough get their own Merkle-sealed audit (steps 37 + 38). |
| Cell architecture | Roughly per-region; not Hamilton-cellular per se. | Tier 0–3 (4-tier model per ADR-0248); shuffle sharding; static stability. |
| Policy posture | OPA in some places; mostly application-code authz. | Cedar at every gate, every time (ADR-0243). |
| HTTP/3 | Partial. | Default (ADR-0253 D-5; Messenger PRD §10.1). |
| Post-quantum | Roadmap. | X25519MLKEM768 hybrid in TLS 1.3 today (ADR-0253 D-9). |
| Audit chain | Append-only logs; not Merkle-sealed. | Merkle-sealed + Ed25519-signed (ADR-0028); per-period seal verification (failure 5.8). |
| Cross-cell traffic | Implicit. | Cedar-gated + SPIFFE-attested per hop (steps 16, 44, 62). |
| Federation | Slack (Slack-to-Slack only). | Matrix r0.6.1 + ActivityPub + Slack-adapter (Messenger PRD §4.8). |

Slack is excellent at the consumer-product layer; oyatie matches it
and adds the hyperscaler substrate underneath.

### 6.2 vs iMessage

| Concern | iMessage | oyatie |
|---|---|---|
| E2E posture | Signal-Protocol-derived; PQ3 (Kyber + ECDH) since iOS 17.2. | MLS RFC 9420; IETF-standardised; group-native FS + PCS. Hybrid PQ planned 2026 Q3-Q4 (MLS std §3.5.3). |
| Group scale | ~32 active (iMessage); ~1000 cap (FaceTime). | 10k per MLS group (Messenger PRD AC-03). |
| Multi-device | Proprietary; Apple-ID-mediated; per-device E2E since iOS 17.2 with Contact Key Verification. | MLS multi-device via distinct LeafNodes per device (MLS std §7); same code path as multi-user. |
| Cross-vendor | Closed; falls back to SMS for non-Apple. | Matrix federation + ActivityPub + Slack-(PRD §4.8). |
| Cell architecture | Apple internal; not publicly documented. | Hamilton-cellular per ADR-0248; publicly documented at this level. |
| Policy posture | Apple-internal; implicit. | Cedar at every gate (ADR-0243). |
| Audit chain | None visible to users; Apple-internal. | Per-tenant Merkle-sealed; DSAR Article 17 + 20 ready (ADR-0242 D-4). |
| Compliance posture | Apple GA. | Per-tenant compliance pack overlay (ADR-0251); pack-kr / pack-eu / pack-us-healthcare etc. |

iMessage is the gold standard for consumer 1:1 quality; oyatie matches
it on the wire (per MLS std §17 threat model) and exceeds it on
multi-tenancy, federation, and audit visibility.

### 6.3 vs WhatsApp

| Concern | WhatsApp | oyatie |
|---|---|---|
| E2E posture | Signal Protocol since 2016. | MLS RFC 9420. |
| Group scale | 1024 cap. | 10k cap (PRD AC-03). |
| Multi-device | Sesame + Signal Sender Keys; 4-device cap. | MLS multi-device via distinct LeafNodes; no fixed device cap. |
| Cross-cell | Implicit (Meta data centres). | Cedar-gated + SPIFFE-attested per hop. |
| Policy posture | Meta-internal. | Cedar at every gate. |
| Federation | None. | Matrix + ActivityPub + Slack-(PRD §4.8). |
| Backup | Cloud key backup; recoverable with phone PIN. | Per-user passphrase-encrypted (Argon2id) or passkey-bound; recoverable only by user (MLS std §11.3). |
| Audit chain | Meta-internal. | Per-tenant Merkle-sealed. |
| Sovereign cloud | Single Meta data centres. | Per-pack overlay (ADR-0240); pack-kr CSAP, pack-eu GAIA-X, pack-ksa NDMO. |

WhatsApp has scale; oyatie has scale **plus** standardisation,
federation, and per-tenant sovereign overlays.

### 6.4 What "hyperscaler-grade" means in this walkthrough

The walkthrough demonstrates seven properties that distinguish a
hyperscaler-grade path from a feature-parity path:

1. **Constant work at the control plane.** Tier 2 cells publish
   policy / topology / identity bundles every 30 s; Tier 3 cells pull.
   No per-change push; no per-event control-plane round trip. (ADR-0248
   D-3; demonstrated by Cedar evaluator hot cache step 10/15/26/etc.)

2. **Static stability.** Tier 3 cells can serve user traffic for 24 h
   with their last-pulled bundle; the control plane can be unavailable
   without user impact. (ADR-0248 D-8; demonstrated by failure 5.2.)

3. **Shuffle sharding.** Tenant→cell binding is shuffle-sharded; a
   single cell's failure affects a bounded fraction of tenants.
   (ADR-0248 D-7; demonstrated by §1.1/§1.2 cell assignment +
   failure 5.11.)

4. **Caller-supplied idempotency keys + HLC ordering.** Every state-
   changing POST carries an idempotency key; every audit row carries
   HLC. No distributed locks. (ADR-0252; demonstrated by steps 31, 36,
   67 + failure 5.3.)

5. **Universal Cedar gating.** No policy in code; every gate is a Cedar
   evaluation; per-cell evaluator with hot cache < 1 ms p99. (ADR-0243
   + ADR-0246; demonstrated by 9 evaluations in §4.4.)

6. **Audit chain as first-class substrate.** Every state-changing
   action emits to a per-tenant Merkle-sealed stream; same primitive
   for `oyatie-corp` as for `tenant-alice-12345`. (ADR-0242 D-4 +
   ADR-0028; demonstrated by 7 audit emissions in §4.7.)

7. **MLS for E2E that scales to groups + multi-device + PQ-ready.** Not
   a Signal-Protocol bolt-on; the IETF-standardised path with public
   formal analysis. (ADR-MSGR-0002 + MLS std §2; demonstrated by the
   entire MLS-bearing portion of phases B–H.)

---

## 7. Closing — what to do after reading this

1. **Reread the cited sections.** Every citation in this document is a
   pointer to a specific paragraph in a specific file. Open the file,
   read the section, and you will find the mechanism described.
2. **Run the walkthrough mentally on a different scenario.** Try
   "Carol creates a 250-person group with Alice + 249 strangers." The
   shape is identical to this walkthrough; the differences are at MLS
   std §5.1.2 (group create) and Messenger PRD AC-02 (1k-member
   fan-out).
3. **Pick one BC and read the PRD §11 row + the corresponding crates.**
   The crates are at `microservices/messenger/src/<bc>-<layer>/` per
   ADR-0131 per-µservice flat layout.
4. **Open a Cedar fragment and read it.** They're at
   `microservices/messenger/policy/cedar/{personal,work,internal}.cedar`
   per Messenger PRD §2.2. They are short; intern-readable.
5. **Read the corresponding user story for your BC.** They're in
   `docs/user-stories/b2c-consumer-surfaces.md` (B2C) or the upcoming
   work-stories compendium (B2B).

The walkthrough is the rubric. Every architectural decision in oyatie
must compose with it. When you propose a new feature, ask yourself:
*can this walkthrough still happen end-to-end after my change?* If
yes, you're aligned with the keystone bundle. If no, you've either
found a real flaw in the architecture (file an ADR) or you're missing
a piece of the doctrine (re-read the cited ADRs).

---

## 8. Appendix — citation index

The walkthrough cites the following artifacts. This is the intern's
required-reading list, in approximate order of importance for understanding
the walkthrough:

1. `docs/decisions/ADR-0702-identity-authz-live-apex.md` (uniform
   machinery; reserved namespace; bootstrap sequence)
2. `docs/decisions/ADR-0700-ci-admission-live-apex.md` (every gate is
   Cedar; coverage CI lane)
3. `docs/decisions/ADR-0700-ci-admission-live-apex.md`
   (Tier 0–3; shuffle sharding; static stability; constant work)
4. `docs/decisions/ADR-0708-platform-foundations-live-apex.md`
   (Anycast + edge POPs + Cilium ambient + SPIFFE + HTTP/3 + PQ hybrid)
5. `docs/decisions/ADR-0709-general-live-apex.md`
   (HLC; idempotency keys; no distributed locks)
6. `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
   (per-cell evaluator; bundle pull cadence)
7. `docs/decisions/ADR-0702-identity-authz-live-apex.md`
8. `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
9. `docs/decisions/ADR-0704-k8s-port-live-apex.md`
   (DR tiers; drill cadence)
10. `docs/decisions/ADR-0708-platform-foundations-live-apex.md`
11. `docs/decisions/ADR-0247-self-hosting-self-modification-doctrine.md`
12. `docs/decisions/ADR-0700-ci-admission-live-apex.md`
13. `docs/decisions/ADR-0701-monorepo-capability-live-apex.md`
14. `docs/adr-archive/ADR-0211-in-house-tech-stack-policy.md`
15. `docs/adr-archive/ADR-0238-connect-super-app-expansion.md`
16. `docs/adr-archive/ADR-0028-cloud-microservice-architecture.md`
17. `microservices/messenger/PRD.md` (especially §1, §2, §4.2, §6, §10,
    §11, §13)
18. `docs/standards/messenger-e2e-encryption-mls.md` (especially §3–§8,
    §13)
19. `docs/user-stories/b2c-consumer-surfaces.md` (§1, §2.1, §2.2, M-01)

Total artifacts: 19. Total cited steps: 73. Total Cedar evaluations: 9.
Total audit-chain emissions: 7. Total cross-cell hops: 4. Total MLS
epochs advanced: 1 (group creation).

> *End of walkthrough.*
