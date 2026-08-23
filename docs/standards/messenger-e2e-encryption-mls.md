---
doc_class: Standard
doc_id: STD-messenger-e2e-encryption-mls
microservice: messenger
status: Draft
date: 2026-05-20
owner_team: axis-messenger
council_reviewers:
  - council-privacy
  - council-security
  - council-architecture
  - ops-compliance
related_adrs:
  - ADR-0188-passkey-webauthn-substrate.md
  - ADR-0211-in-house-tech-stack-policy.md
  - ADR-0240-sovereign-cloud-per-regional-pack.md
  - ADR-0242-oyatie-is-a-tenant-doctrine.md
  - ADR-0251-compliance-pack-cell-certification-levels.md
  - ADR-0252-time-coordination-distributed-consistency.md
  - ADR-0253-network-topology-edge-service-mesh.md
related_specs:
  - /specs/microservices/messenger.json
related_ips:
  - microservices/meet/IP-012-e2e-encryption-mls.md
canonical_protocol: MLS (IETF RFC 9420, July 2023)
canonical_implementation: mls-rs (awslabs/mls-rs, Rust)
implementation_status: design-phase
threat_model_owner: council-security
---

# Messenger End-to-End Encryption Design — MLS (RFC 9420)

> **Audience.** Implementers (Rust + native mobile + WASM) building the
> `messenger` µservice's E2E surface. Reviewers (council-privacy,
> council-security, ops-compliance) approving the cryptographic posture.
> Auditors verifying the design against GDPR Article 32, HIPAA Security
> Rule §164.312, PCI DSS Requirement 4, FedRAMP-Moderate SC-13/SC-28, and
> KR-PIPA Article 24.

> **Status.** Draft, 2026-05-20. Pending council-privacy + council-security review. Implementation gated on this
> standard's acceptance + the companion implementation plans referenced
> in §16.

---

## Table of contents

1. Purpose + Scope
2. Why MLS over Signal Protocol
3. Cryptographic primitives
4. Identity + Key Management
5. Group lifecycle
6. Server's role (delivery service)
7. Multi-device sync
8. Message lifecycle
9. Federation
10. Forward secrecy + Post-compromise security
11. Encrypted backups
12. Voice + Video E2E
13. File + Photo + Video attachment encryption
14. Sticker + GIF + emoji
15. Group call (Meet integration)
16. Implementation choices
17. Threat model
18. Compliance considerations
19. Operational considerations
20. References
21. Appendices

---

## 1. Purpose + Scope

### 1.1 Purpose

Define the **end-to-end encryption (E2EE) substrate** for the `messenger`
µservice's **personal-context (B2C)** surface, grounded on the IETF
Messaging Layer Security protocol (MLS) defined in
**RFC 9420 — *The Messaging Layer Security (MLS) Protocol*** published
July 2023 ([datatracker.ietf.org/doc/rfc9420/](https://datatracker.ietf.org/doc/rfc9420/)).
MLS provides forward secrecy + post-compromise security + tree-based
group key agreement for groups from 2 to thousands of members on a
single standardised wire format.

The standard is implementation-ready: an engineer with cryptography
knowledge should be able to read this document, the cited RFCs, and the
`mls-rs` crate documentation, and produce a production-grade
implementation. Where this document defers a choice, the deferral is
explicit and the deferral-resolution owner is named.

### 1.2 Scope — what's in

| Surface | E2E Posture | Why |
|---|---|---|
| Personal 1:1 DMs | MLS, mandatory | privacy contract per Messenger PRD; competitive parity with Signal / iMessage / WhatsApp |
| Personal group chats (≤ 5000) | MLS, mandatory | group-native MLS; same code path as 1:1 (group of size 2) |
| Personal voice + video (1:1) | MLS for signaling + key agreement, DTLS-SRTP for media (keys from MLS epoch) | per IP-012-e2e-encryption-mls in Meet µservice |
| Personal voice + video (group call) | MLS for signaling, DTLS-SRTP for media, per-participant TreeKEM key share | matches Meet IP-012 |
| File + photo + video attachments (personal) | Per-attachment AES-256-GCM; key wrapped in MLS message | server stores ciphertext only |
| Stickers + custom emoji (personal) | Same as attachments | server stores ciphertext only |
| Backups (personal) | Per-user backup encryption key derived from passkey or encryption-key BYOK (ADR-0251 §D-10) | recovery without server access to plaintext |
| Cross-device sync (personal) | Each device is its own MLS member; per-user "device group" duplicates messages across devices | Signal-style multi-device, MLS-implemented |

### 1.3 Scope — what's out

| Surface | Posture | Why |
|---|---|---|
| Professional-context (B2B) channels | Server-side envelope encryption (tenant-DEK per Bominal ADR-0111) + Cedar policy + four-eyes admin disclosure per Bominal ADR-0215 | DLP, eDiscovery, compliance (HIPAA admin access), retention all require structured server-side visibility |
| Audit-chain entries | Merkle-sealed + Ed25519-signed but NOT E2E-encrypted | Audit chain is shared substrate; tamper-evidence is the property, not confidentiality from the platform owner |
| Search index (personal) | Client-side encrypted search index per user (per-device) | server cannot index plaintext; client builds local index from decrypted material |
| Push notification body | Encrypted MLS payload + minimal unencrypted envelope (sender display name if user opts in; otherwise opaque) | per APNs / FCM constraint that requires server-routable envelope |
| Account recovery without passkey + without backup | Manual identity-proofing per `identity-account-recovery` runbook (ADR-0188 path) | accepting irrecoverable data loss on full-credential loss is the privacy guarantee |

### 1.4 Personal vs Professional split

Per the Messenger PRD (§Tenant Value, §Security):

- **Personal (B2C) DMs** carry no tenant-admin disclosure path. The
  server stores ciphertext; the platform owner (oyatie, per ADR-0242
  oyatie-is-a-tenant doctrine) operates the server as one tenant among
  many and has the same access posture (none, on personal-context
  ciphertext) as any other tenant operator.
- **Professional (B2B) channels** use server-side envelope encryption
  (tenant-DEK from `microservices/cloud-secrets/` OpenBao) so that DLP
  scanners, eDiscovery exports, four-eyes admin disclosure, HIPAA
  audit, and Cedar policy filters can operate on cleartext under
  appropriate authorisation. The split honours the **dual-context
  isolation invariant** (parallel ADR-0238): a personal DM cannot be
  re-routed into a professional channel by any client, server, or
  admin action.

This standard documents only the **personal** path. The professional
path is documented separately in `docs/standards/messenger-server-side-encryption-tenant-dek.md`
(not in this PR).

### 1.5 Relationship to Meet IP-012

The **Meet µservice's IP-012-e2e-encryption-mls** establishes
MLS+SFrame for opt-in E2E meetings (recording / transcription denied
by Cedar when `e2e=true`). This standard adopts the same MLS cipher
suite, the same `mls-rs` crate, the same epoch advancement cadence
(monthly recommended per RFC 9420 §11.6), and the same hybrid PQ
migration roadmap. The shared posture means **clients reuse the
cryptographic core across Messenger and Meet**, reducing surface area.

Meet's W3C Insertable Streams (SFrame) layer is **specific to
WebRTC media** — it's not used by Messenger for text. Messenger's
file/photo/video attachment encryption is a separate AEAD layer
described in §13.

---

## 2. Why MLS over Signal Protocol

### 2.1 Signal Protocol summary

Signal Protocol (sometimes called "Double Ratchet" after its core
algorithm) is the de facto E2EE standard since 2013, deployed by
Signal Messenger, WhatsApp (Meta, since 2016), Facebook Messenger
(Meta, secret conversations 2016, default rollout 2023), Google
Messages RCS (since 2020), and Skype (2018). Its primitives:

- **X3DH (Extended Triple Diffie-Hellman, Marlinspike & Perrin 2016)**
  — initial key agreement using a long-term identity key, a signed
  pre-key, and a one-time pre-key.
- **Double Ratchet (Marlinspike & Perrin 2016)** — symmetric ratchet
  per message + Diffie-Hellman ratchet per round-trip, giving forward
  secrecy and post-compromise security on pairwise sessions.
- **Sender Keys** — for group chats, each sender derives a sender
  chain per group; recipients hold a copy of each sender's chain.
- **Sesame** — multi-device session management layered atop X3DH.

Signal Protocol has strong, public security analysis (Cohn-Gordon
et al., "A Formal Security Analysis of the Signal Messaging
Protocol", JoC 2020).

### 2.2 Signal Protocol limitations

| Limitation | Impact at oyatie scale |
|---|---|
| Pairwise sessions, not group-native | Group fan-out is O(N) re-encryptions per message; cost grows with group size; epoch advancement on member-change is bespoke per implementation |
| Sender Keys are a separate protocol with weaker PCS than Double Ratchet | A compromised sender key remains valid until the next rekey; PCS lags behind 1:1 |
| Not IETF-standardised | Wire format and key derivation are documented as Signal whitepapers, not RFCs; cross-implementation interop is by convention, not by reference test vector |
| Multi-device is bolted on (Sesame) | Per-device fan-out doubles or triples message volume; UX gotchas (e.g. cross-device read receipts) are implementation-specific |
| Group max ≈ 1024 (WhatsApp limit; Signal raised to 1000 in 2023) | Above ~1000, performance degrades sharply; large communities push into Discord/Telegram alternatives |
| Post-quantum migration is per-implementation | Signal added PQXDH (Kyber + X25519 hybrid) in Sep 2023 (Signal blog) but as a Signal-specific extension; other Signal-Protocol-using vendors must follow independently |

### 2.3 MLS summary (RFC 9420)

MLS (Messaging Layer Security) is an **IETF-standardised** group key
agreement protocol, published as **RFC 9420** in July 2023 after a
six-year IETF MLS Working Group (charter 2017, framework draft 2018,
protocol draft 2019, last call 2022, RFC 2023). Designed from day
one for groups of arbitrary size with forward secrecy + post-compromise
security in **logarithmic** cost.

Key constructions:

- **TreeKEM (Tree-based Key Encapsulation Mechanism)** — every group
  member is a leaf of a left-balanced binary tree; the root holds the
  group secret. Member add/remove requires O(log N) HPKE
  encryptions, not O(N). Mathematical details: Bhargavan et al.,
  "TreeKEM: Asynchronous Decentralized Key Management for Large
  Dynamic Groups" (HotPETs 2018) and Alwen et al., "Key Agreement
  with Logarithmic Communication Cost" (CRYPTO 2021).
- **Epoch advancement** — every group state change (add, remove,
  update, commit) advances the epoch; secrets from prior epochs
  cannot be derived from the current state, giving forward secrecy.
- **HPKE (RFC 9180, Hybrid Public Key Encryption)** — wire-format
  building block for asymmetric encryption.
- **Welcome messages** — new members receive an encrypted blob
  containing the current group state; they can join without
  pre-distributing keys.
- **Commit messages** — group-state-changing operations are batched
  into a Commit; one Commit per epoch.
- **LeafNode + KeyPackage** — durable identity binding +
  pre-published asynchronous group-join material.

### 2.4 Industry adoption (as of 2026-05)

| Vendor | Product | MLS Deployment | Source |
|---|---|---|---|
| Cisco | Webex (Meetings + Messaging) | Production since 2023 ("MLS for Webex E2E meetings"); messaging E2E added 2024 | Cisco "Cisco Webex Zero-Trust Security" whitepaper (2024); Cisco MLS interop demo at IETF 117 (2023) |
| AWS | AWS Wickr (acquired 2021) | Migrated from Signal-derived to MLS in 2023; awslabs/mls-rs is the implementation | AWS blog "AWS Wickr architecture" (2023); awslabs/mls-rs GitHub README |
| Wire (Wire Swiss GmbH) | Wire Messenger | MLS rolled out as default in 2024; replaces Proteus (Signal-derived) | Wire blog "MLS migration complete" (2024); Wire MLS spec at github.com/wireapp/proteus-mls |
| RingCentral | RingCentral Messaging | MLS adopted 2024 for compliance with EU sovereign-cloud requirements | RingCentral product update 2024 |
| Phoenix R&D | Phoenix Messenger (open-source MLS reference) | MLS-only by design | github.com/phnx-im (Phoenix Initiative) |
| Akamai | Akamai Edge Workers (E2E session for edge functions) | MLS adopted 2025 (per Akamai Edge Live 2025) | Akamai Edge Live 2025 keynote |
| Mozilla | Thunderbird Matrix bridge | MLS for Matrix E2E (Element's MLS migration) | Element blog "MLS in Matrix" (2024) |
| Element / Matrix.org | Element / Synapse | MLS as Matrix's next-gen E2E (replacing Olm/Megolm); rolling deployment 2024-2025 | matrix.org "MLS in Matrix" blog post (2024); MSC4244 (Matrix Spec Change) |
| Google | Google Messages (RCS E2E) | Planning MLS adoption per Google I/O 2024 RCS Universal Profile 3.0 hint; still Signal Protocol in production as of 2026-05 | Google I/O 2024 RCS session |
| Apple | iMessage Contact Key Verification | Signal-Protocol-derived; PQ3 (Kyber+ECDH) since iOS 17.2 (Dec 2023); no public MLS plans | Apple Security Engineering blog "Advancing iMessage security" (Feb 2024); Apple iMessage PQ3 whitepaper |
| Meta | WhatsApp + Messenger | Signal Protocol; no public MLS plans as of 2026-05 | Meta Engineering blog (ongoing) |

The pattern: enterprise + sovereign-cloud vendors converged on MLS in
2023-2025 for the standardisation, group-native, and PCS-by-default
properties. Consumer vendors that started with Signal Protocol have
substantial migration cost and have not announced MLS adoption yet.

### 2.5 Why oyatie picks MLS

The decision matrix below records the trade-off explicitly.

| Criterion | Signal Protocol | MLS | Winner |
|---|---|---|---|
| Standardisation | Signal whitepaper | IETF RFC 9420 | MLS |
| Group-native | Sender Keys (bolted on) | TreeKEM (built in) | MLS |
| Cost at group size N | O(N) per message | O(log N) per epoch | MLS |
| Forward secrecy | Yes (double ratchet) | Yes (epoch advance) | tie |
| Post-compromise security | Yes, on rekey | Yes, on rekey | tie |
| Cross-implementation interop | Convention | RFC test vectors + interop matrix | MLS |
| Open-source Rust crate | `libsignal` (Signal-owned) | `mls-rs` (AWS Apache 2.0); `openmls` (Phoenix MPL-2.0); `mlspp` (Cisco BSD-3) | MLS (multiple, no single-vendor lock-in) |
| Post-quantum extension path | PQXDH (Signal-specific) | draft-ietf-mls-pq (IETF process); MLS extensions framework (draft-ietf-mls-extensions) | MLS (in process, IETF-blessed) |
| Industry adoption (2026) | WhatsApp, Signal, FB Messenger, Google Messages, Skype | Webex, Wickr, Wire, Matrix, RingCentral; Element MLS-default 2025 | tie (different cohorts) |
| Maturity of formal analysis | 7+ years (Cohn-Gordon 2017 → 2020) | 5+ years (Alwen 2020 → 2024) | Signal (slightly more mature) |
| Audit ecosystem | NCC Group 2016, Trail of Bits 2023 | Cure53 2023 (mls-rs); Cryspen 2024 (openmls) | tie |
| Compliance with sovereign-cloud (ADR-0240) | No published profile | RFC 9421 architecture is sovereign-cloud-compatible | MLS |
| Upgrade path | Bespoke per vendor | Versioned extension framework | MLS |

**Decision (this standard):** MLS. The standardisation and
group-native cost characteristics are decisive at oyatie's target
scale (groups up to 5000; 100M+ concurrent users in steady state).
The forward secrecy / PCS / formal-analysis criteria are tied. The
post-quantum migration path is decided by an IETF process not by a
single vendor.

This decision is consistent with the choice already made for the Meet
µservice (IP-012); using the same protocol across Messenger + Meet
means a single audited cryptographic core.

### 2.6 What we're not getting from MLS

Honesty about the gaps:

1. **MLS doesn't standardise the delivery service.** RFC 9420
   specifies the cryptographic protocol; RFC 9421 (Architecture)
   describes the role of the delivery service (DS) and authentication
   service (AS) abstractly. The wire format between client and DS is
   per-vendor. This standard documents oyatie's choice (§6).
2. **MLS doesn't define federation.** draft-ietf-mls-federation
   (in-progress, IETF MLS WG) sketches federation semantics but isn't
   stable. Federation strategy is §9.
3. **MLS doesn't define backup / recovery.** Per-user backup
   encryption is layered on top by application designers. §11.
4. **MLS doesn't define identity verification UX.** Safety numbers,
   contact key verification, etc., are application-layer. §4.6.
5. **MLS is text-message-shaped.** Voice/video media keys derive from
   MLS epoch but the media path is DTLS-SRTP (RFC 5764). §12.

---

## 3. Cryptographic primitives

### 3.1 Cipher suite

MLS cipher suites are defined in RFC 9420 §17.1. A cipher suite is
identified by a 16-bit code and consists of (HPKE-KEM, HPKE-KDF,
HPKE-AEAD, Signature, Hash). The IANA registry is at
[iana.org/assignments/mls/mls.xhtml](https://www.iana.org/assignments/mls/mls.xhtml).

**Default cipher suite (oyatie):**

```
MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519  (suite ID = 0x0001)
```

Components:

| Component | Algorithm | RFC | Rationale |
|---|---|---|---|
| KEM | DHKEM(X25519, HKDF-SHA-256) | RFC 9180 §7.1.2 | curve25519 = standard, hardware-acceleratable, no patent issues |
| KDF | HKDF-SHA-256 | RFC 5869 | de facto KDF; hardware-acceleratable; sufficient at 128-bit security |
| AEAD | AES-128-GCM | NIST SP 800-38D | hardware AES-NI on Intel/AMD; ARMv8 AES instructions on mobile; meets FIPS 140-3 |
| Signature | Ed25519 | RFC 8032 | deterministic, side-channel-resistant, smaller signatures than ECDSA, faster than RSA |
| Hash | SHA-256 | FIPS 180-4 | hardware-accelerated (SHA-NI on x86; ARMv8 SHA instructions) |

Why cipher suite 0x0001 over alternatives:

- **vs MLS_128_DHKEMP256_AES128GCM_SHA256_P256 (0x0002)** — X25519 has
  no patent encumbrance, simpler implementation, well-audited
  libraries (curve25519-dalek; libsodium). P-256 is preferred only
  for FIPS-restricted deployments (see §3.2.4).
- **vs MLS_256_DHKEMX448_AES256GCM_SHA512_Ed448 (0x0007)** — 256-bit
  security is overkill for messaging plaintext lifetimes (forward
  secrecy collapses the value of long-term cryptanalysis). The
  performance cost (X448 ~3x slower than X25519; AES-256 ~25%
  slower than AES-128) is not justified outside PQ migration.
- **vs MLS_128_DHKEMX25519_CHACHA20POLY1305_SHA256_Ed25519 (0x0003)**
  — ChaCha20-Poly1305 is the better choice on devices lacking AES-NI
  (some older ARM cores; some IoT). Listed as the fallback (§3.4).

### 3.2 Cipher-suite negotiation

#### 3.2.1 Per-group cipher suite

A cipher suite is fixed at group creation per RFC 9420 §10. Mixed-suite
groups are not supported. Cipher-suite upgrade requires creating a
new group and migrating members (§3.6).

#### 3.2.2 Supported suites (initial deployment)

| Suite ID | Suite name | Status |
|---|---|---|
| 0x0001 | MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519 | **default**; all clients MUST support |
| 0x0003 | MLS_128_DHKEMX25519_CHACHA20POLY1305_SHA256_Ed25519 | **fallback**; required on devices without AES-NI |
| 0x0007 | MLS_256_DHKEMX448_AES256GCM_SHA512_Ed448 | optional; for sovereign-cloud-pack-KR + pack-KSA + pack-US-Government where regulators require 256-bit symmetric |
| `0x0A0F` (pending IANA assignment) | MLS_HYBRID_X25519_MLKEM768_AES128GCM_SHA256_Ed25519_MLDSA65 | **post-quantum migration**; see §3.5 |

Clients announce supported suites in their KeyPackage's
`capabilities.cipher_suites`. Group creator picks the first suite
that all initial members support; subsequent member additions must
support the group's suite or they're refused.

#### 3.2.3 Per-pack overrides

Per ADR-0240 (sovereign-cloud-per-regional-pack), regional packs may
override the default suite for tenants in that pack:

| Pack | Default suite | Required by |
|---|---|---|
| pack-default (US, EU non-government, JP, AE) | 0x0001 | this standard |
| pack-kr (CSAP-certified) | 0x0001 (CSAP 3.1 accepts X25519 + AES-128 since 2024); 0x0007 optional for government tenants | KR MSIT CSAP guide v3.1 |
| pack-ksa (NDMO + SDAIA) | 0x0001 (NDMO accepts NIST-approved primitives); 0x0007 for sovereign data | SDAIA Cloud Computing Framework v1.0 |
| pack-us-government (FedRAMP-Moderate + IL5) | 0x0002 (P-256) MANDATORY for FedRAMP-High; 0x0001 acceptable for FedRAMP-Moderate per NIST SP 800-186 (FIPS-186-5 admitting Ed25519 since 2023) | NIST SP 800-186, FIPS 186-5 |
| pack-eu-sovereign (GAIA-X) | 0x0001 (GAIA-X accepts FIPS-186-5 algorithms) | GAIA-X cryptographic policy v2 |

#### 3.2.4 FIPS 140-3 mode

For tenants requiring FIPS 140-3 validated cryptographic modules:

- AES-128-GCM, SHA-256, HKDF-SHA-256, ECDSA-P-256, ECDH-P-256, HMAC
  are all FIPS-186-5 approved.
- Ed25519, X25519 became FIPS-186-5 approved in February 2023 (NIST
  SP 800-186, "Recommendations for Discrete Logarithm-based
  Cryptography: Elliptic Curve Domain Parameters").
- The cryptographic primitive provider in FIPS mode is **AWS-LC**
  (FIPS 140-3 validated module since 2024; certificate #4759 in the
  NIST CMVP database). `mls-rs` supports an AWS-LC backend; selection
  is per-tenant via OpenBao-stored policy.

### 3.3 KDF (Key Derivation Function)

HKDF-SHA-256 (RFC 5869) is used throughout MLS:

- **HKDF-Extract** — combines an input keying material (IKM) and a
  salt into a pseudo-random key (PRK).
- **HKDF-Expand** — derives output keys from the PRK and a context
  label.
- **MLS-specific labels** per RFC 9420 §8 — every derivation is
  domain-separated by a label like `"MLS 1.0 epoch secret"`,
  `"MLS 1.0 commit secret"`, etc.

Implementation: `hkdf` crate (v0.12+, rust-crypto); audited by
NCC Group 2022. Constant-time.

### 3.4 AEAD (Authenticated Encryption with Associated Data)

**Default:** AES-128-GCM per NIST SP 800-38D.

- 96-bit nonce (RFC 9420 §9 specifies nonce derivation).
- 128-bit authentication tag.
- Hardware-accelerated via AES-NI / ARMv8 AES instructions.
- Per-message nonce is derived from the AEAD key + the generation
  counter (RFC 9420 §9.1); nonce reuse is impossible by construction
  unless the implementation is buggy.

**Fallback:** ChaCha20-Poly1305 per RFC 8439.

- Used on devices lacking AES-NI (older ARM cores, some Android
  devices).
- Performance: comparable to AES-128-GCM with AES-NI; faster
  without.
- Implementation: `chacha20poly1305` crate (rust-crypto).

**Selection algorithm at client:**

```rust
fn select_aead_for_device() -> AeadSuite {
    if cpu_supports_aes_ni() || cpu_supports_armv8_aes() {
        AeadSuite::Aes128Gcm
    } else {
        AeadSuite::ChaCha20Poly1305
    }
}
```

The selection is reported in the client's KeyPackage capabilities;
the group's chosen suite must intersect every member's capabilities.

### 3.5 Post-quantum migration

#### 3.5.1 Threat model: "harvest now, decrypt later"

Adversaries with long-term storage capability (nation-state SIGINT)
are presumed to collect today's ciphertext and store it for future
decryption when cryptographically-relevant quantum computers (CRQC)
become available. Per NIST estimates, CRQC capable of breaking 256-bit
elliptic curves may emerge between 2030 and 2040; conservative
deployments assume the lower bound.

The "harvest now, decrypt later" threat means:

- Today's plaintext sent under classical-only crypto is at risk *now*
  if it has a confidentiality lifetime extending past CRQC arrival.
- Forward secrecy partially mitigates this — each epoch's secret is
  destroyed — but the *initial key agreement* in a classical-only
  cipher suite is the long-lived weakness.

#### 3.5.2 Hybrid cipher suite

The IETF MLS WG drafts `draft-ietf-mls-pq` (rev 04 as of 2025-Q3)
defines a **hybrid** cipher suite combining classical and PQ primitives.
Hybrid means both algorithms run in parallel; the resulting key is the
concatenation; an adversary must break both to derive the key.

**Target suite (oyatie):**

```
MLS_HYBRID_X25519_MLKEM768_AES128GCM_SHA256_Ed25519_MLDSA65
```

Components:

| Component | Classical | PQ | Combined |
|---|---|---|---|
| KEM | X25519 | ML-KEM-768 (FIPS 203, NIST 2024) | concatenate shared secrets, HKDF into final KEM output |
| Signature | Ed25519 | ML-DSA-65 (FIPS 204, NIST 2024) | dual-sign; verifier checks both |
| KDF | HKDF-SHA-256 | n/a | unchanged |
| AEAD | AES-128-GCM | n/a (symmetric is PQ-resistant at 128-bit per Grover's quadratic-speedup mitigation; doubled key size optional) | unchanged |

**ML-KEM (Module-Lattice-based KEM):**
- Standardised as FIPS 203 in August 2024.
- ML-KEM-768 = NIST level III (128-bit classical-equivalent security
  post-quantum); the recommended size.
- Public key ~1.2 KB; ciphertext ~1.1 KB.
- Implementation: `mlkem` crate (RustCrypto, OSS); `aws-lc` (FIPS).
- Audited: NCC Group 2024 (mlkem reference); Trail of Bits 2024 (AWS-LC).

**ML-DSA (Module-Lattice Digital Signature Algorithm):**
- Standardised as FIPS 204 in August 2024.
- ML-DSA-65 = NIST level III.
- Public key ~2 KB; signature ~3.3 KB.
- Implementation: `mldsa` crate (RustCrypto); `aws-lc` (FIPS).

#### 3.5.3 Migration plan

Per ADR-0253 (Network Topology + Service Mesh) §"Year 2 PQ rollout"
and matching the Meet IP-012 PQ trajectory:

| Year | State | Action |
|---|---|---|
| **Y1 (2026)** | Classical-only (cipher suite 0x0001) | Production deploy; observe |
| **Y1.5 (2026 Q3-Q4)** | Hybrid available as opt-in for sovereign tenants | Cipher suite registration; client + server `mls-rs` upgrade to MLS-PQ draft; performance benchmarking |
| **Y2 (2027)** | Hybrid default for new groups; classical migration tool ships | New 1:1 and group creates pick hybrid; existing groups upgrade via cipher-suite-migration (creates new group, copies members + permits, deprecates old group after 30d) |
| **Y3 (2028)** | Hybrid is the only supported cipher suite for new groups; classical-only refused | Classical-only groups read-only after 90 days; archived to backup |
| **Y4+ (2029+)** | PQ-only when ML-KEM proves stable; classical removed from suite | Subject to IETF MLS WG progress + NIST guidance refresh |

The migration is gated on:

1. `mls-rs` crate adding hybrid-suite support (currently behind a
   feature flag; expected GA in late 2026).
2. IETF `draft-ietf-mls-pq` becoming RFC (estimated 2027 Q1-Q2).
3. Cure53 / NCC audit of the hybrid implementation.

### 3.6 Cipher-suite migration semantics

Migrating an existing group from one suite to another is **not
supported in-place** by RFC 9420. The procedure:

1. Group founder (or any member, with all-member consent) initiates
   a migration.
2. A new group is created with the target cipher suite.
3. All members of the old group are added via Welcome to the new
   group.
4. Recent message history (last 30 days, configurable) is
   client-side re-encrypted under the new group's keys and posted
   to the new group as "historical-archive" messages (separate from
   live messages).
5. The old group is marked deprecated; senders post to new group;
   readers may continue to read old group ciphertext for 90 days
   (read-only).
6. After 90 days, old group ciphertext is purged from the server
   (per backup retention).

Migrations emit a `MlsCipherSuiteMigrationStarted` and
`MlsCipherSuiteMigrationCompleted` event into the audit chain.

---

## 4. Identity + Key Management

### 4.1 User-level identity vs device-level identity

oyatie's E2E model uses **device-level identity** (each device has its
own MLS signing key + KeyPackage), not user-level identity.

Why device-level:

- A user's devices have different security postures (phone with
  biometric vs. desktop with Touch ID vs. browser). Treating them as
  the same identity attaches all devices' blast radius to any one
  device's compromise.
- MLS member-removal is the PCS primitive. Removing *a compromised
  device* without removing the user is only expressible if devices
  are distinct members.
- The Signal Sesame multi-device model is similar; iMessage's per-
  device E2E (since 2023 Contact Key Verification) is similar.

**Identity stack:**

```
+-------------------------------------------+
| User Account (Zitadel; per-tenant)        |
|  - email, display name, profile pic       |
|  - passkey-bound (ADR-0188)               |
|  - tenant_id (oyatie-* for personal)      |
+-------------------------------------------+
                  |
   one-to-many (user owns N devices)
                  v
+-------------------------------------------+
| Device (per physical device)              |
|  - device_id (UUID)                       |
|  - platform (iOS / Android / web / ...)   |
|  - long-term Ed25519 signing key (in     |
|    Secure Enclave / TPM / Keystore)       |
|  - device-binding signature by user's    |
|    passkey (ADR-0188)                     |
+-------------------------------------------+
                  |
   one-to-many (device has KeyPackages)
                  v
+-------------------------------------------+
| KeyPackage                                |
|  - one-shot async-join material           |
|  - includes leaf node + HPKE init key    |
|  - signed by device's long-term key       |
|  - lifetime 90 days; rotated proactively  |
+-------------------------------------------+
                  |
                  v
+-------------------------------------------+
| Group Membership (LeafNode)               |
|  - device acts as MLS member of a group  |
|  - epoch-specific keys derived per-epoch  |
+-------------------------------------------+
```

### 4.2 Long-term signing key

Each device generates an Ed25519 key pair at first launch. The
private key never leaves the device's hardware-backed secure storage:

| Platform | Storage | Hardware backing | Extraction resistance |
|---|---|---|---|
| iOS / iPadOS | Keychain (`kSecAttrAccessibleWhenUnlockedThisDeviceOnly`, `kSecAttrSynchronizable=false`); key generated in Secure Enclave when device has SE | Secure Enclave (A7+, all modern iOS) | Strong; Apple-published threat model treats SE extraction as state-actor-level |
| Android | Android Keystore (`KeyProperties.PURPOSE_SIGN`; `setUserAuthenticationRequired(true)`); StrongBox where available | TEE (TrustZone) or StrongBox (Pixel 3+, Samsung Knox) | StrongBox: state-actor-level; TEE: malware-resistant |
| macOS | Keychain (`kSecAttrAccessibleWhenUnlockedThisDeviceOnly`); Secure Enclave on Apple Silicon + T2 Macs | Secure Enclave (T2; M1+) | Strong on M-series; moderate on Intel |
| Windows | Platform Crypto Provider (TPM-backed; CNG NCRYPT_PLATFORM_KEY_STORAGE_PROVIDER) | TPM 2.0 (Windows 11 baseline) | Strong with TPM; weak without (refuse to operate without TPM 2.0 per ADR-0188 alignment) |
| Linux | TPM 2.0 (tpm2-tss) + libsecret (D-Bus Secret Service for cached references) | TPM 2.0 where available | Strong with TPM; weak without |
| Web (browser) | WebAuthn-derived seed; CryptoKey API with `extractable=false`; sealed to origin | Per-browser: Chrome+Edge use OS-backed if available; Firefox uses software | Moderate; mitigated by KeyPackage rotation + short web-session lifetime |

For the web client specifically, the Ed25519 key is derived from a
WebAuthn assertion at first launch and stored as a `CryptoKey` with
`extractable=false` in IndexedDB. The key cannot be exported by JS; it
can only be used to sign via `crypto.subtle.sign()`. On browser data
clear, the key is destroyed; the next launch generates a new device
identity (and the user must re-add the device to existing groups).

The Linux desktop client refuses to run on machines without a TPM 2.0
(per ADR-0188 §"Browser support matrix" extension to native).

### 4.3 Identity binding to passkey (ADR-0188)

Per ADR-0188, the **passkey** (WebAuthn Level 3 credential) is the
phishing-resistant first-factor identity primitive. The MLS
device-level identity is **bound** to the passkey at device pairing.

Binding semantics:

1. User signs in with passkey on a new device.
2. Device generates fresh Ed25519 long-term key in HSM-backed storage.
3. Device requests user's passkey to sign an attestation:
   ```
   attestation_payload = {
     device_id: <uuid>,
     device_public_key: <ed25519-pub>,
     timestamp: <utc>,
     user_account_id: <zitadel-subject>,
     pairing_method: "passkey-webauthn-l3"
   }
   ```
4. The user's passkey (WebAuthn assertion) signs the attestation.
5. Server (identity µservice) verifies passkey assertion + records
   the `DeviceIdentityAttested` event in the audit chain.
6. The signed attestation is appended to the device's first
   KeyPackage.

This means **any party verifying a device's identity** can chain
`device public key → passkey assertion → Zitadel account → tenant`
without needing to trust the server's claim about the binding.

If the user has multiple passkeys (e.g., iCloud-synced Passkey +
hardware YubiKey for `acr=critical`), the attestation can be signed
by either; both signatures are recorded for redundancy.

### 4.4 LeafNode structure

Per RFC 9420 §7.2, a LeafNode is the data structure representing a
group member:

```
struct {
    HPKEPublicKey encryption_key;        // ephemeral per-epoch HPKE init
    SignaturePublicKey signature_key;    // device's long-term Ed25519 pub
    Credential credential;                // identity binding (see §4.5)
    Capabilities capabilities;            // supported suites, extensions, proposals
    LeafNodeSource leaf_node_source;     // KeyPackage / Update / Commit
    Extension extensions<V>;              // application-specific
    /* SignWithLabel(., "LeafNodeTBS", LeafNodeTBS) */
    opaque signature<V>;                  // signed by device's long-term key
} LeafNode;
```

oyatie-specific extensions in the `extensions<V>` field:

| Extension | Purpose | Content |
|---|---|---|
| `oyatie_device_metadata` | Device platform + version | `{ platform: "ios", os_version: "17.5", app_version: "1.0.0", device_model: "iPhone15,3" }` |
| `oyatie_passkey_attestation` | Passkey-bound attestation (§4.3) | The signed payload from §4.3 step 3-4 |
| `oyatie_tenant_scope` | Tenant identity context | `{ tenant_id: "tenant-customer-xyz" }` (always personal scope; verified by Cedar) |
| `oyatie_compliance_pack` | Regional pack the device operates in | `{ pack: "kr" }` (informs server-side residency enforcement) |

The signature on the LeafNode is verified by every group member
before accepting an add or update.

### 4.5 Credential

The MLS `Credential` field per RFC 9420 §5.3 binds the LeafNode's
signature key to an external identity. RFC 9420 defines two
credential types: `basic` (raw identifier bytes) and `x509` (X.509
certificate chain).

oyatie uses **`basic`** credential type with the identifier being
the Zitadel account subject (a stable UUID):

```
credential = Credential {
    credential_type: 0x0001  // basic
    identity: <Zitadel account subject UUID, 16 bytes>
}
```

The binding from the Zitadel subject to a passkey + device is
established by the `oyatie_passkey_attestation` extension (§4.4) +
the audit-chain event from §4.3 step 5.

Why basic, not x509:

- x509 credentials require an oyatie-operated CA + cert lifecycle. The
  passkey + audit-chain binding achieves the same property with less
  infrastructure.
- x509 credentials embed a full cert chain in every LeafNode, inflating
  Welcome messages (which carry every member's LeafNode).
- Future migration to x509 is supported by the credential type field;
  if a regulatory pack demands x509 (e.g., a national PKI), the device
  pairing flow appends an x509 attestation.

### 4.6 Safety numbers + contact verification

Users can verify they're talking to the right person (vs. a
server-mediated MitM, see threat model §17) via **safety numbers**:

- The MLS group's tree hash is a stable, public function of all
  members' LeafNodes.
- For 1:1 conversations, the safety number is derived as:
  ```
  safety_number = SHA-256(
    canonical_encoding(alice_long_term_pub) ||
    canonical_encoding(bob_long_term_pub)
  )[:30 bytes] -> base32 -> 60-character string
  ```
- Displayed in groups of 5 characters: `ABCDE FGHIJ KLMNO PQRST UVWXY 23456 789AB CDEFG HIJKL MNOPQ RSTUV WXYZ2`.
- Encoded as a QR code that the other party can scan in-person.
- Comparison is byte-for-byte; mismatch indicates MitM.

For group conversations, MLS's tree hash is exposed in the UI as
"group fingerprint." A change in fingerprint after a member-add
indicates the add was legitimate; a change without a known
member-event is a red flag.

This UX mirrors:
- Signal's safety numbers (since 2017).
- iMessage Contact Key Verification (since iOS 17.2, Dec 2023).
- Wire's group fingerprints.

### 4.7 KeyPackage publication

A KeyPackage (RFC 9420 §10) is a pre-published bundle that allows
asynchronous addition to a group without round-trip negotiation:

```
struct {
    ProtocolVersion version;
    CipherSuite cipher_suite;
    HPKEPublicKey init_key;     // one-shot HPKE init for Welcome encryption
    LeafNode leaf_node;          // the device's LeafNode
    Extension extensions<V>;
    /* SignWithLabel(., "KeyPackageTBS", KeyPackageTBS) */
    opaque signature<V>;
} KeyPackage;
```

Each device publishes a **KeyPackage bundle** (≥ 100 KeyPackages per
push) to the cell-local KeyPackage registry. When another device
wants to add this device to a group:

1. Server (delivery service) issues `GetKeyPackage(device_id)`.
2. Registry returns one unused KeyPackage and marks it consumed.
3. The added party uses the KeyPackage to create a Welcome.
4. The receiving device unwraps the Welcome with the corresponding
   init_key private (which it generated locally).

Registry storage:

- Per-cell KeyPackage table:
  ```sql
  CREATE TABLE key_package_registry (
      id UUID PRIMARY KEY,
      tenant_id TEXT NOT NULL,        -- always "oyatie-*" or personal scope
      account_id UUID NOT NULL,
      device_id UUID NOT NULL,
      cipher_suite SMALLINT NOT NULL, -- 0x0001 / 0x0003 / etc.
      key_package BYTEA NOT NULL,     -- MLS-encoded
      lifetime_not_after TIMESTAMPTZ NOT NULL,
      consumed_at TIMESTAMPTZ NULL,
      consumed_by_account_id UUID NULL,
      created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
      CONSTRAINT key_package_unique UNIQUE (id),
      CONSTRAINT key_package_one_shot CHECK (
          consumed_at IS NULL OR consumed_by_account_id IS NOT NULL
      )
  );

  CREATE INDEX idx_kp_unused
      ON key_package_registry (account_id, device_id, cipher_suite)
      WHERE consumed_at IS NULL;
  ```

- Bundle target: maintain ≥ 100 unused KeyPackages per device per
  cipher suite. Server background worker monitors low-water mark and
  notifies the device to publish more (via a push or on next
  reconnect).

- Lifetime: 90 days from generation. Past expiry, KeyPackages are
  refused.

- Server validates KeyPackage signatures on receipt (anti-spam +
  anti-corruption). Invalid signatures rejected with audit event.

### 4.8 KeyPackage rotation cadence

| Trigger | Action |
|---|---|
| Routine, per device | Publish a fresh batch of 100 KeyPackages every 30 days (lifetime 90 days; 60-day overlap) |
| Low-water (< 20 unused) | Publish immediately |
| Long-term signing key rotation | Publish fresh KeyPackages signed under the new key; mark old KPs consumed |
| Compromise detected | Mark all KPs consumed; publish fresh batch under a fresh long-term key; out-of-band notify groups |
| App update with new MLS protocol version | Publish fresh KPs declaring new version in `capabilities` |

The rotation cadence is enforced by a server-side worker
(`messenger-mls-keypackage-rotation-worker`) that issues push
notifications to devices that fall behind.

### 4.9 HSM-backed signing for high-security tenants

For tenants designated as `high-security` (per ADR-0240 sovereign-cloud
+ ADR-0251 compliance-pack-cell-certification-levels), the device's
long-term signing key is held in an HSM rather than the device's
local secure storage:

- Per-tenant cloud HSM: AWS CloudHSM, Azure Managed HSM, Google Cloud
  HSM, or domestic equivalents (KT Cloud HSM for pack-KR; STC HSM for
  pack-KSA).
- Device communicates with the HSM via a per-tenant TLS-mTLS tunnel.
- Signing operations are gated by the user's passkey + a per-tenant
  policy in OpenBao.
- Latency cost: ~50-150ms additional per signing operation; acceptable
  for KeyPackage rotation and Commit signing.

HSM-backed signing applies to:
- Tenants on FedRAMP-High packs.
- Tenants on KR-FSS (Korean financial services) pack.
- Tenants on PCI-DSS-Level-1 packs.
- Tenants explicitly opting in via tenant settings (charged per
  cloud-HSM cost in FinOps).

Personal-context users (the focus of this standard) typically don't
use HSM-backed signing; the device's hardware secure storage is the
floor. The mechanism is documented here for cross-reference because
the same `mls-rs` code path is used.

### 4.10 KeyPackage validation

Before accepting a KeyPackage, the server validates:

1. **Protocol version** — supported (currently MLS 1.0 only).
2. **Cipher suite** — supported (default + fallback + optional list).
3. **Signature** — Ed25519 signature on `KeyPackageTBS` verifies under
   `leaf_node.signature_key`.
4. **LeafNode signature** — Ed25519 signature on `LeafNodeTBS`
   verifies.
5. **Credential** — the identity field is a valid Zitadel account
   subject that the publishing device is authorised to publish for
   (verified via Cedar policy against the OIDC session).
6. **Extension** — required extensions (`oyatie_device_metadata`,
   `oyatie_passkey_attestation`, `oyatie_tenant_scope`,
   `oyatie_compliance_pack`) are present and well-formed.
7. **Passkey attestation** — the signed attestation in
   `oyatie_passkey_attestation` is verifiable against a registered
   passkey for the account.
8. **Lifetime** — `lifetime_not_before <= now < lifetime_not_after`.

Failed validation emits an `MlsKeyPackageValidationFailed` audit event
and refuses the publish; the device receives the specific failure
reason for client-side correction.

---

## 5. Group lifecycle

### 5.1 Group creation

#### 5.1.1 1:1 (direct conversation)

A 1:1 MLS group is created when User A first messages User B:

1. **Client A** queries Server for User B's account_id and an active
   KeyPackage for each of User B's devices:
   ```
   GET /v1/messenger/users/{userB}/devices
   → returns: [{ device_id, latest_keypackage }, ...]
   ```
   Per ADR-0145 inter-microservice-communication-reform, the call uses
   gRPC with mTLS.

2. **Client A creates the MLS group** locally via `mls-rs`:
   ```rust
   use mls_rs::{Client, MlsGroup, GroupConfig};

   let group_id = GroupId::generate();
   let config = GroupConfig::builder()
       .cipher_suite(CipherSuite::Mls128X25519Aes128GcmSha256Ed25519)
       .extensions(vec![
           Extension::oyatie_group_metadata(GroupMetadata {
               kind: GroupKind::DirectMessage,
               participants: vec![user_a, user_b],
           }),
       ])
       .build();

   let mut group = MlsGroup::new(group_id, config, &client_a.signing_key)?;
   ```

3. **Client A adds User B's devices** to the group:
   ```rust
   for device_kp in user_b_devices {
       group.add_member(device_kp)?;
   }
   let commit = group.commit()?;
   let welcomes = group.welcome_messages()?;
   ```

4. **Client A also adds its own other devices** (per §7):
   ```rust
   for own_device_kp in user_a_other_devices {
       group.add_member(own_device_kp)?;
   }
   ```

5. **Client A sends to server:**
   ```
   POST /v1/messenger/mls/groups
   body: {
       group_id: <bytes>,
       initial_commit: <mls Commit>,
       welcomes: [
           { recipient: device_id_b1, welcome: <bytes> },
           { recipient: device_id_b2, welcome: <bytes> },
           { recipient: device_id_a2, welcome: <bytes> },
           ...
       ],
       tenant_scope: "personal",
       group_kind: "dm"
   }
   ```

6. **Server** validates the Commit's signature (signed by Client A's
   LeafNode), stores the group metadata, and routes Welcomes to each
   recipient device's pending queue.

7. **Recipient devices**, on next reconnect or push, fetch their
   Welcome, process it via `MlsGroup::join_from_welcome(welcome)?`,
   and now have the group's initial epoch secret.

#### 5.1.2 Group chat

Same as 1:1 but with more initial members. Max group size per RFC
9420 is approximately 5000 (the practical limit depends on Welcome
message size — every leaf node adds ~200-500 bytes to Welcome). At
~5000, Welcome ≈ 1-2 MB; acceptable on broadband, marginal on mobile
data.

For super-large groups (≥ 1000 members), the group creator should
use a staged add approach:

- Initial group: creator + first ~100 members.
- Subsequent batches of ~500 members added per epoch.
- This bounds the worst-case Welcome size to ~500 leaves.

The Messenger PRD targets up to ~5000-member groups for personal
context (community + family groups; not channels).

### 5.2 Add member

```rust
let new_member_kp = fetch_keypackage(new_user_device_id).await?;
group.add_member(new_member_kp)?;
let commit = group.commit()?;
let welcome = group.welcome_message_for(new_member_device_id)?;

// Send commit to existing members; welcome to new member
deliver_commit_to_existing_members(commit).await?;
deliver_welcome_to_new_member(new_member_device_id, welcome).await?;
```

**Epoch advances** by 1. Existing members process the Commit; they
arrive at the new epoch secret. The new member processes the Welcome;
they also arrive at the new epoch secret (via a different derivation
path; both arrive at the same key).

**Forward secrecy property:** Messages encrypted in epoch N are
undecryptable from the state of epoch N+1 (provided members
correctly discard epoch-N secrets, which `mls-rs` does by default).

**Welcome message contains:**
- The current group state (every member's LeafNode).
- The init_secret (a secret used to derive the epoch secret).
- Group context (group_id, epoch, tree_hash, extensions).
- All encrypted to the new member's KeyPackage HPKE init_key.

Server is **not** in the cryptographic loop; it only relays messages.

### 5.3 Remove member

```rust
group.remove_member(member_to_remove)?;
let commit = group.commit()?;

deliver_commit_to_all_members(commit).await?;
```

**Epoch advances** by 1. Remaining members process the Commit, derive
the new epoch secret. The removed member does **not** receive the
Commit; even if they did, they could not decrypt (the removal Commit
proposes a new `path_secret` for the subtree containing the removed
leaf, and the removed leaf's path cannot derive it).

**Post-compromise security property:** A compromised member who is
removed has zero forward access. Once epoch N+1 is reached, the
removed member cannot decrypt any message from N+1 onward, even if
they retained all their pre-removal keys.

**UX consideration:** the removed user should see "you were removed
from this group" on their next session. The server delivers a
`GroupMembershipChangedNotification` to the removed device,
unencrypted (because they can't decrypt the group's payload anymore).
This notification is not authenticated against the group's MLS state
(since the removed device can't verify the new state); it's a server-
side claim. The client SHOULD show "this group is no longer
accessible" rather than the original group contents.

### 5.4 Update key

A member rotates their LeafNode keys periodically to advance forward
secrecy:

```rust
group.update_own_leaf()?;
let commit = group.commit()?;
deliver_commit_to_all_members(commit).await?;
```

Triggers:

- **Time-based:** every 7 days per device, jittered.
- **Message-count-based:** every 1000 sent messages, jittered.
- **Significant event:** app launch after backgrounded > 24h; device
  unlock after lock > 4h; restoration from backup.
- **Proactive:** any member can request an update (via "rekey now"
  in advanced settings).

Updates are cheap (O(log N) HPKE encryptions per the TreeKEM path).
At a group size of 1000, an update is ~10 HPKE operations + a Commit
of ~3-5 KB.

### 5.5 Epoch synchronisation

In practice, multiple members may attempt Commits at the same epoch.
RFC 9420 §12.1 specifies that **at most one Commit per epoch is
applied**; the others are rejected. The Delivery Service enforces
ordering:

1. Server receives Commits from members A, B, C concurrently for
   epoch N → N+1.
2. Server applies the first Commit it sees (say A's) and rejects
   B's and C's with a "stale epoch" error.
3. B and C re-process: they fetch A's Commit, advance their local
   state to epoch N+1, and may re-issue their proposals (which now
   target epoch N+1 → N+2).

The server's role here is **ordering enforcement only**; the server
doesn't choose semantically which Commit wins (any winner is correct
because the protocol is conflict-free given a serialised order).

**Ordering primitive:** per ADR-0252 (Time Coordination + Distributed
Consistency), the server uses a Hybrid Logical Clock (HLC) to order
Commits within a group:

```
commit_order = (hlc_timestamp, server_node_id, monotonic_counter)
```

The HLC is causally consistent (a Commit's HLC > any HLC the issuer
had observed). This is sufficient because MLS's epoch is a coarse
counter; the server only needs to break ties.

### 5.6 Group state synchronisation

Each client maintains its own MLS state for each group it's a member
of. State synchronisation between clients of the same user (multi-
device) is via:

1. **Per-message epoch tag** — every message carries its epoch number;
   recipients verify their local state matches.
2. **Out-of-order delivery** — if a device receives a message at epoch
   N but is at epoch N-2, it pauses delivery and requests the
   intermediate Commits from the server.
3. **Periodic state attestation** — every 24 hours, each device emits
   its current epoch + tree_hash to the server; server cross-checks
   against the canonical state and flags mismatches.

Server is **not** the source of truth for cryptographic state — every
member of a group is — but the server is the source of truth for
*ordering* (which Commit was accepted at which point in the message
stream).

### 5.7 Group state recovery

If a device's local MLS state is corrupted or lost (uninstall,
storage failure, OS reset), the device must rejoin every group it was
in. The rejoin flow:

1. Device generates a fresh long-term signing key + KeyPackages,
   re-binds to the user's passkey, publishes KeyPackages.
2. Server enumerates the user's group memberships (group_id list).
3. For each group, server emits a notification to one other member of
   the group (preferably another of the user's devices; otherwise
   the longest-membership peer) to re-add the recovering device.
4. The other member calls `group.add_member(new_kp)` and sends a
   Welcome.
5. The recovering device processes the Welcome and is back in the
   group.

**Trade-off:** message history before the rejoin is not recoverable
on the recovering device unless backups (§11) are restored. The user
may see a gap in their history; this is honest about the FS+PCS
property of MLS.

The "longest-membership peer" rule is to optimise for the peer most
likely to have stable state; the peer is selected from the LeafNode
metadata recorded in the group's epoch-0 KeyPackage history (which is
public to all members).

---

## 6. Server's role (delivery service)

### 6.1 Trust model: untrusted relay

The server (oyatie's `messenger` µservice running as one tenant under
the `oyatie` org per ADR-0242) is an **untrusted relay** for personal-
context E2E messages:

- **Cannot decrypt** any group's MLS payload (it doesn't have any
  group's epoch secret, init_secret, or leaf private keys).
- **Cannot impersonate** any user (devices' long-term keys are in
  HW-backed storage; the server doesn't have them).
- **Can observe** metadata: who messages whom (group membership), at
  what time, message size, frequency.
- **Can deny service** (drop messages, refuse to deliver Welcomes,
  delay Commits).
- **Can collude with one member to MitM new joiners** (a compromised
  server + one compromised member could forge group membership; the
  safety-number UX, §4.6, mitigates by requiring out-of-band
  verification).

This trust model is the floor; oyatie operationally treats the server
as a trustworthy entity (it's an `oyatie.*` principal subject to
Cedar policy, audit chain, DR, compliance) but the design does not
require trust.

### 6.2 Server validations (anti-spam + anti-corruption)

For each MLS message passing through the server:

1. **KeyPackage signature** (on publish) — server verifies Ed25519
   signature on KeyPackage. Refuses invalid signatures.
2. **Commit signature** — server verifies the Commit's outer signature
   is by a known group member's LeafNode. Refuses unknown signers.
3. **Epoch monotonicity** — server refuses Commits with epoch ≤
   current group epoch.
4. **Group existence** — server refuses messages for groups that
   don't exist or are deleted.
5. **Member presence** — server refuses messages from non-members
   (i.e., a member who was removed in a prior epoch).
6. **Message size** — refuses messages > 10 MB (configurable per
   pack; default mirrors WhatsApp's media-message size).
7. **Rate limit** — per-device + per-tenant rate limit (token-bucket;
   bypass requires `acr=critical` per ADR-0188 step-up).

The server does **not** validate the content (which is encrypted); it
validates the wrapper.

### 6.3 Encrypted message storage

Server stores three classes of MLS content:

| Class | Lifetime | Retention basis | Storage |
|---|---|---|---|
| **PrivateMessage** (encrypted application data) | Until delivered to all recipients + 7 days grace | Required for offline-device delivery | Postgres + S3 (large attachments — encrypted blob; see §13) |
| **Welcome** | Until consumed by recipient + 30 days | Required for new-device join | Postgres |
| **Commit** | Until consumed by all members of the epoch + 90 days | Required for state catch-up + audit-chain reproducibility | Postgres + audit chain |

Schema (Postgres, per-cell):

```sql
CREATE TABLE mls_message (
    message_id UUID PRIMARY KEY,
    group_id BYTEA NOT NULL,
    tenant_id TEXT NOT NULL,
    epoch BIGINT NOT NULL,
    sender_leaf_index INTEGER NOT NULL,  -- not the sender's identity; just the tree position
    content_type SMALLINT NOT NULL,      -- 0x01 application; 0x02 welcome; 0x03 commit
    payload BYTEA NOT NULL,              -- MLS-encoded wire format
    payload_hash BYTEA NOT NULL,         -- SHA-256 for audit chain
    hlc_timestamp BYTEA NOT NULL,        -- per ADR-0252
    delivered_to JSONB NOT NULL DEFAULT '[]',  -- list of device_ids that ack'd receipt
    pending_for JSONB NOT NULL DEFAULT '[]',   -- list of device_ids still to deliver
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NULL,         -- set when fully delivered
    INDEX idx_group_epoch_hlc (group_id, epoch, hlc_timestamp),
    INDEX idx_pending (tenant_id, pending_for) WHERE pending_for != '[]'
);

CREATE TABLE mls_group_metadata (
    group_id BYTEA PRIMARY KEY,
    tenant_id TEXT NOT NULL,
    group_kind TEXT NOT NULL,            -- "dm" | "group" | "voice_call" | "video_call"
    current_epoch BIGINT NOT NULL,
    cipher_suite SMALLINT NOT NULL,
    members JSONB NOT NULL,               -- [{ leaf_index, account_id, device_id, keypackage_ref }, ...]
    extensions JSONB NOT NULL,            -- application extensions
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_commit_at TIMESTAMPTZ NULL,
    deleted_at TIMESTAMPTZ NULL
);
```

The `payload` column holds the **encrypted** MLS message; the server
never decrypts it.

### 6.4 Ordering

Per ADR-0252 (Time Coordination + Distributed Consistency), the
server orders messages by **Hybrid Logical Clock** within a group:

```
ordering_key = (group_id, epoch, hlc_timestamp, sender_leaf_index)
```

- **Within an epoch:** messages are totally ordered by HLC.
- **Across epochs:** epoch number is the higher-order sort key.
- **Causal consistency:** every message's HLC ≥ all HLCs the sender
  has previously observed in the group.

Clients (recipients) display messages in this order. If a client
receives messages out-of-order (due to push-notification jitter), it
re-sorts before display.

### 6.5 Delivery

Two delivery paths:

#### 6.5.1 Online: WebSocket

When a device is connected via WebSocket (per Messenger PRD §
Performance, RFC 6455 over mTLS), the server pushes messages
immediately:

```
Server → Client: MLSMessageDeliveryFrame {
    message_id: UUID,
    group_id: bytes,
    epoch: u64,
    payload: bytes (MLS-encoded),
    hlc_timestamp: bytes,
    sender_leaf_index: u32
}

Client → Server: MLSMessageDeliveryAck {
    message_id: UUID,
    received_at: HLC
}
```

#### 6.5.2 Offline: push notification + queue

When a device is offline, the server queues the message and emits a
push notification (APNs for iOS / macOS; FCM for Android +
WebPush on web; Windows Push Notification Service on Windows).

Push notification payload constraints:

- **APNs:** max 4 KB.
- **FCM:** max 4 KB; up to 4 KB of `data` payload.
- **WebPush:** max 4 KB.

The full MLS message can exceed 4 KB (especially for attachments +
Commits at high group size); the push **does not contain the full
message**.

Push payload shape:

```json
{
  "aps": {
    "alert": {
      "loc-key": "MESSENGER_NEW_MESSAGE",
      "loc-args": []
    },
    "sound": "default",
    "badge": 1,
    "mutable-content": 1,
    "content-available": 1
  },
  "oyatie": {
    "kind": "mls_message",
    "tenant_id_hash": "<sha256-of-tenant-id>",
    "group_id_hash": "<sha256-of-group-id>",
    "message_id": "<uuid>",
    "epoch": 42,
    "size": 1024
  }
}
```

- **`alert.loc-key`** ensures the user-facing notification shows
  "New message" (or localised equivalent) via a per-locale string
  lookup; the actual sender/content is not in the push.
- The `oyatie.*` extension fields use **hashes** of identifiers so
  Apple/Google can't aggregate per-tenant or per-group statistics.

When the OS shows the notification, the **Notification Service
Extension** (iOS) / **Service Worker** (web) wakes up, fetches the
full message from the server via authenticated REST call:

```
GET /v1/messenger/mls/messages/{message_id}
Authorization: Bearer <device-token>
```

Decrypts via the local MLS state, and updates the user-facing
notification with:

- Sender display name (from the user's contact list — local
  knowledge, not from server).
- Message preview (first ~80 chars of plaintext, if user has enabled
  "show preview").
- Sound + badge.

If the device is offline (e.g., low battery, airplane mode), the
notification stays as the generic "New message"; full content is
fetched on next online.

### 6.6 Read receipts

Read receipts are **opt-in** and **encrypted**:

1. Recipient client decrypts an inbound message, displays it.
2. If recipient has read-receipts enabled (default: on for 1:1;
   off for groups ≥ 10):
   a. Recipient client constructs a read-receipt control message
      inside the MLS group:
      ```
      { kind: "read_receipt", message_id: <uuid>, read_at: <HLC> }
      ```
   b. Encrypts the control message as a normal MLS application
      message.
   c. Sends to server.
3. Server delivers the control message to all group members.
4. Sender's client receives, decrypts, updates UI.

Server **cannot** see the read-receipt content because it's
encrypted as MLS payload. Server-observable metadata: a control
message of size ~100 bytes was sent at time T from device X to group G.

For 1:1 personal DMs, read receipts are the user-expected default.
For groups ≥ 10, fan-out cost grows (every member's receipt → every
other member); default off, configurable per-group.

### 6.7 Audit chain emission (encrypted-message-hash only)

Per ADR-0242 §D-4, every state-changing action emits to the audit
chain. For E2E messages, the audit event records:

```json
{
  "event_type": "mls_message_relayed",
  "tenant_id": "<tenant>",
  "group_id_hash": "<sha256>",
  "epoch": 42,
  "payload_hash": "<sha256-of-payload>",
  "payload_size": 1024,
  "sender_device_id_hash": "<sha256>",
  "recipient_count": 5,
  "hlc_timestamp": "<hlc>",
  "audit_chain_seq": 17483291,
  "merkle_seal_signature": "<ed25519>"
}
```

The audit chain **does not** record plaintext content; it records
hashes + sizes + counts. This is the tamper-evident record useful for:

- Legal compliance (proof that a message of size X passed through
  the server at time T; the message's plaintext is unknown to the
  server, but the event chain is integrity-protected).
- Forensic investigation (if a device is compromised and the user
  reports it, the user's audit trail enumerates which messages the
  device sent + received; the user can correlate with their own
  local cleartext to identify suspicious activity).
- Internal accounting (message counts per tenant for billing,
  capacity planning).

The audit chain payload is the chain's data class `AUDIT`; per
ADR-0242, the chain is replicated per sovereign-cloud-overlay (§3.2.3).

### 6.8 Cell + multi-region

Per ADR-0009 (cell architecture) and ADR-0049 (cross-region
replication), the `messenger` µservice deploys per-cell + per-region.

**Per-cell scope for E2E content:**

- A user's group lives in their home-cell.
- Messages are stored in the home-cell's Postgres + audit chain.
- If a user is travelling (their device is geographically distant),
  the WebSocket connection may terminate in a different cell's edge
  gateway, but the group's authoritative store remains in the home
  cell.
- Cross-cell traffic for E2E payloads uses the inter-cell mesh
  tunnel (ADR-0044 + ADR-0148); ciphertext only.

**Multi-region replication:**

- E2E ciphertext is **eligible for cross-region replication** within
  the tenant's allowed pack residency.
- Cross-pack replication is **denied** by Cedar policy when the
  tenant's pack has `prohibited_egress` constraints (ADR-0240 §D-1).
- The ciphertext is opaque, but the metadata (tenant_id,
  sender/recipient identifiers) is in the data class registry as
  `BEHAVIORAL_TENANT_PRODUCT_PERSONAL`; this metadata is subject to
  residency rules.

**Pack-KR example:**

A KR personal user's messages:

- Group lives in pack-kr cell.
- Postgres replicates within pack-kr (Seoul + Busan DR).
- Audit chain replicates within pack-kr.
- Push notification routing: server emits to APNs/FCM endpoints,
  which are global (Apple/Google operate; KR PIPA does not require
  push providers be domestic, but the push payload contains hashes
  only, no PII).
- Cross-cell-into-pack-eu traffic for the same group: **denied** if
  the metadata class is `pack-bound`.

### 6.9 Server-side abuse handling

Personal messages are E2E-encrypted; the server cannot scan content
for abuse (spam, illegal material, threats). Mitigations:

#### 6.9.1 User-side reporting

When a user reports a message:

1. Client uploads the **plaintext** message + metadata to a per-tenant
   abuse-review queue under the user's consent.
2. Reviewers (oyatie's Trust & Safety team, operating as
   `oyatie.trust-safety.*` principals per ADR-0242) examine the
   report.
3. If actionable, the offending device is sanctioned (account
   suspension; in egregious cases, law enforcement referral per local
   jurisdiction's MLAT process).
4. The plaintext upload is recorded in the audit chain as a
   `UserReportedMessage` event; retention follows the abuse-review
   retention policy (longer than ordinary message retention, e.g.,
   3 years for safety review).

The user's consent to upload plaintext is the privacy-preserving
mechanism: the platform never reads cleartext without user opt-in.

#### 6.9.2 Metadata-based abuse signals

Server can observe (without reading content):

- Message-rate anomalies (one device sending to thousands of new
  recipients per hour = likely spam botnet).
- Group-creation spikes (one user creating thousands of groups = mass
  spam).
- Network signals (TLS fingerprint, IP reputation).

These trigger rate-limiting, captcha, or temporary suspension at the
delivery service; the cryptographic protocol is untouched.

#### 6.9.3 CSAM detection: out-of-band only

Per Apple's iCloud CSAM detection controversy (2021-2022) and the
prevailing 2024-2026 industry consensus, **client-side or server-side
scanning of E2E content is not deployed**. CSAM detection is via:

- Operating-system-level scanning (Apple's NeuralHash on photos
  *before* they enter Messages; Google's similar on Android Photos)
  — out of oyatie's scope.
- User reporting (§6.9.1).
- Industry-shared hash databases (NCMEC PhotoDNA, CSAM hash sets) —
  applied at user-side upload to oyatie blob storage (§13) on file
  attach, with the user's awareness in the Privacy Policy.

The latter detail: when a user attaches a photo, the client
optionally computes a perceptual hash and compares against NCMEC's
hash set locally; if matched, the upload is refused. This is done
client-side; the server never sees the cleartext or the hash unless
matched.

---

## 7. Multi-device sync

### 7.1 Each device is a distinct MLS member

Per §4.1, each user device has its own MLS identity. A user with N
devices in a 1:1 conversation with another user (with M devices) is
in a group of size N + M.

Example:

- Alice has phone + tablet + desktop + web = 4 devices.
- Bob has phone + desktop = 2 devices.
- Their "1:1" conversation is a group of 6 MLS members.

Trade-off:

- **Pro:** per-device PCS. If Alice's tablet is stolen, Alice removes
  the tablet from all groups; the thief has zero forward access.
- **Pro:** simpler protocol (no Sesame-style multi-device extension).
- **Con:** message size multiplies (the same ciphertext is delivered
  to N+M devices, each decryption is independent).
- **Con:** group size is users * devices, hitting RFC 9420's ~5000
  practical limit faster.

### 7.2 Adding a new device

When a user adds a new device (e.g., gets a new phone):

1. **New device** signs in via passkey (ADR-0188) on the new device.
2. New device generates fresh Ed25519 signing key, publishes initial
   KeyPackages.
3. Server emits a `DeviceAdded` notification to all the user's existing
   devices.
4. One of the existing devices (the user's primary, by user policy or
   freshest-online algorithm) initiates a fan-out add:
   a. Enumerate every group the user is in (via local state).
   b. For each group, fetch the new device's KeyPackage from server.
   c. Call `group.add_member(new_device_kp)?; group.commit()?`.
   d. Send Commit to the group + Welcome to the new device.

5. New device receives Welcomes for every group, processes them, and
   has full group membership.

**Failure mode:** if the user has no other online device when adding
the new device, the new device cannot be added to existing groups.
Resolution: the new device displays "Waiting for another device to
add you to your groups; please open the app on your phone to
complete sync."

For users with **only one device** (e.g., new sign-up; replaced lost
phone with no backup), there are no existing groups to sync, so this
is a non-issue at first sign-up. For lost-and-replaced phone scenarios,
the user is in the "device recovery" path: §5.7.

### 7.3 Per-user device group

In addition to per-conversation groups, each user has a **personal
device group** — an MLS group whose members are all of the user's own
devices. This group is used for:

- **Cross-device sync of metadata** — contact list updates, read
  cursor positions across devices, settings changes — all flow as
  MLS application messages in the device group.
- **Encrypted cloud backup key material** (§11).
- **Device-pairing handshake** for new devices.

The device group is created at the user's first device sign-in; new
devices are added as new MLS members.

Why a separate group:

- Isolation: metadata sync between devices is operationally separate
  from message conversations.
- Smaller group: only the user's devices (typically 2-6), so
  operations are cheap.
- Distinct cryptographic state: a compromise of one conversation
  group doesn't compromise the device group (and vice versa).

### 7.4 Cross-platform clients

Same MLS protocol; different platform integrations:

| Platform | Client architecture | Storage | UX surface |
|---|---|---|---|
| iOS / iPadOS | Native (Swift) with `mls-rs` via FFI (C ABI); UI in SwiftUI | Keychain (signing key) + Core Data (group state) | Native iOS Messages-like UX |
| Android | Native (Kotlin) with `mls-rs` via JNI; UI in Jetpack Compose | Android Keystore (signing key) + Room (group state) | Native Material 3 UX |
| macOS | Native (Swift) sharing iOS codebase; AppKit shell | Keychain + Core Data | Native macOS UX |
| Windows | Native (Rust) with `mls-rs` direct; UI in Tauri (web tech) | TPM 2.0 (signing key) + SQLite (group state) | Native-feeling shell |
| Linux | Native (Rust) with `mls-rs` direct; UI in Tauri | TPM 2.0 + SQLite | Native-feeling shell |
| Web | Rust → WASM with `mls-rs`; UI in React+TypeScript | WebAuthn CryptoKey (signing key, non-extractable) + IndexedDB (group state) | Same as native shell, browser-hosted |

The cryptographic core is **`mls-rs`** in all cases; the FFI
boundary varies. This ensures cryptographic behaviour is identical
across platforms (no platform-specific re-implementation =
no platform-specific cryptographic bugs).

---

## 8. Message lifecycle

### 8.1 Send

```
User types message in client UI
  ↓
Client serialises message payload (per AsyncAPI 2.6 schema in
  /microservices/messenger/contracts/asyncapi/personal-messages.yaml):
  {
    type: "text",
    content: { body: "hello", format: "markdown" },
    metadata: { client_id: "msg-uuid-local", timestamp: <client-time> }
  }
  ↓
Client encrypts as MLS application message in the conversation's MLS group:
  let mls_ciphertext = group.encrypt_application_message(payload_bytes)?;
  ↓
Client signs the outer MLS frame with device's long-term key (per RFC 9420 §6)
  ↓
Client sends to server via WebSocket (online) or HTTPS POST (push-only mode):
  POST /v1/messenger/mls/groups/{group_id}/messages
  body: { mls_ciphertext, hlc_timestamp }
  ↓
Server validates (§6.2)
  ↓
Server persists, updates HLC ordering (§6.4)
  ↓
Server emits to audit chain (§6.7)
  ↓
Server fans out to recipients (§6.5)
```

### 8.2 Server-side flow

```rust
// microservices/messenger/src/mls_delivery.rs (sketch)

pub async fn handle_inbound_mls_message(
    msg: InboundMlsMessage,
    sender_ctx: &SenderContext,
    cell: &CellContext,
) -> Result<DeliveryReceipt, DeliveryError> {
    // 1. Validate
    let group = cell.mls_groups
        .lookup(&msg.group_id)
        .await?;
    if group.tenant_id != sender_ctx.tenant_id {
        return Err(DeliveryError::CrossTenantRefused);
    }
    if !group.members.iter().any(|m| m.device_id == sender_ctx.device_id) {
        return Err(DeliveryError::SenderNotInGroup);
    }
    if msg.epoch < group.current_epoch {
        return Err(DeliveryError::StaleEpoch);
    }

    // 2. Verify outer signature
    let leaf_node = group.leaf_node_for(&sender_ctx.device_id)?;
    verify_mls_outer_signature(&msg, &leaf_node.signature_key)?;

    // 3. Persist
    let stored = cell.mls_message_store
        .persist(MlsMessageRecord {
            message_id: Uuid::new_v4(),
            group_id: msg.group_id.clone(),
            epoch: msg.epoch,
            sender_leaf_index: msg.sender_leaf_index,
            payload: msg.ciphertext.clone(),
            payload_hash: sha256(&msg.ciphertext),
            hlc_timestamp: cell.hlc.advance(sender_ctx.hlc_observed),
            content_type: msg.content_type,
            ..Default::default()
        })
        .await?;

    // 4. Audit chain emission
    cell.audit_chain.append(AuditEvent::MlsMessageRelayed {
        tenant_id: sender_ctx.tenant_id.clone(),
        group_id_hash: sha256(&msg.group_id),
        epoch: msg.epoch,
        payload_hash: stored.payload_hash.clone(),
        payload_size: msg.ciphertext.len(),
        sender_device_id_hash: sha256(&sender_ctx.device_id),
        recipient_count: group.members.len() as u32,
        hlc_timestamp: stored.hlc_timestamp.clone(),
    }).await?;

    // 5. Fan-out to recipients
    let recipients: Vec<_> = group.members
        .iter()
        .filter(|m| m.device_id != sender_ctx.device_id)
        .collect();
    for recipient in &recipients {
        cell.delivery_queue.enqueue(MlsDeliveryTask {
            message_id: stored.message_id,
            recipient_device_id: recipient.device_id.clone(),
            payload: stored.payload.clone(),
            // ...
        }).await?;
    }

    // 6. Return delivery receipt to sender
    Ok(DeliveryReceipt {
        message_id: stored.message_id,
        epoch: msg.epoch,
        accepted_at: stored.hlc_timestamp,
        recipient_count: recipients.len() as u32,
    })
}
```

### 8.3 Receive

```
Client receives MLSMessageDeliveryFrame from server
  ↓
Client validates HLC ordering: ensure received HLC > last HLC for this group
  ↓
Client decrypts via mls-rs:
  let plaintext = group.decrypt_application_message(&frame.payload)?;
  ↓
Client parses payload by content type:
  - "text" → text message
  - "file_ref" → attachment (see §13)
  - "control" → read receipt, edit, delete, etc. (see §8.4–§8.7)
  ↓
Client updates local message store (encrypted at rest per device platform)
  ↓
Client signals UI: new message arrived
  ↓
Client sends MLSMessageDeliveryAck to server
  ↓
Server marks message as delivered to this device; if all recipient devices
  have ack'd, schedules message for deletion (per retention)
```

### 8.4 Delivery receipt

After client successfully decrypts + persists locally, it sends an
acknowledgement to the server:

```
POST /v1/messenger/mls/messages/{message_id}/delivery-ack
Authorization: Bearer <device-token>
body: { received_at: <HLC>, decrypt_status: "ok" | "failed" }
```

If `decrypt_status == "failed"`, the client provides:
- Failure reason (epoch mismatch, signature invalid, payload corrupt).
- Current local epoch.
- Suggested recovery action (request Commit catch-up, request re-add).

The server emits a `MlsDecryptFailed` audit event (with hashes only,
not content) for forensic analysis.

The sender's client receives a per-recipient delivery report (via
encrypted control message in the same MLS group):

```
{
  type: "delivery_status",
  message_id: "msg-uuid",
  recipient_device_id: "device-uuid",
  status: "delivered" | "read",
  at: "<HLC>"
}
```

UX surface: the sender sees "✓ delivered" / "✓✓ delivered to all" /
"✓✓ read" badges per common messenger conventions (WhatsApp /
iMessage / Signal patterns).

### 8.5 Read receipt

Identical to §6.6:

- Client decrypts message + displays in UI.
- If user has read receipts enabled, client emits an encrypted
  `read_receipt` control message in the MLS group.
- Server fans out to other members; sender's client updates UI to
  "read" status.

### 8.6 Edit

MLS doesn't define "edit a previous message" — every MLS message is
final. To support edits, oyatie layers an application-level protocol:

1. User clicks "edit" on a recently-sent message (within the edit
   window, e.g., 24 hours after send).
2. Client constructs an edit control message:
   ```json
   {
     "type": "edit",
     "edits_message_id": "msg-uuid-original",
     "new_content": { "body": "hello world", "format": "markdown" },
     "edited_at": "<HLC>"
   }
   ```
3. Client encrypts as a normal MLS application message + sends.
4. Recipients decrypt the edit message and **update their local
   display** of the original message (replacing the body, showing
   "edited" indicator).
5. The original message ciphertext is **not modified** on the server
   (it's immutable); the edit is a separate ciphertext referencing
   the original.

Edit limits:
- Original message must have been sent by the editor.
- Edit window: 24 hours after original send (configurable per pack;
  some jurisdictions require shorter, e.g., HIPAA contexts).
- Edits beyond the window are refused client-side.

### 8.7 Delete

Two variants:

#### 8.7.1 Delete for self (local)

User removes a message from their local view; the message remains in
other recipients' views and on the server (until retention TTL).

- Client-side only.
- No protocol message.
- Server retains the ciphertext (audit chain + delivery state).

#### 8.7.2 Delete for all (tombstone)

User retracts a message from everyone's view. Within an edit window
(longer than edit, e.g., 7 days):

1. Client emits a delete control message:
   ```json
   {
     "type": "delete",
     "deletes_message_id": "msg-uuid-original",
     "deleted_at": "<HLC>"
   }
   ```
2. Recipients receive, update UI to "[message deleted]" placeholder.
3. Server's audit chain records the delete event.
4. Server **retains the original ciphertext** until retention TTL
   expires (not crypto-shredded by the delete event) but marks it as
   "tombstoned" — meaning new connections will not receive it on
   catch-up.
5. After retention TTL, the original ciphertext is purged from
   server storage; the tombstone marker persists in the audit chain.

Delete-for-all does not give cryptographic erasure on recipients'
devices (the recipient's local-decrypted plaintext may have been
backed up, screenshotted, or kept by a malicious client). It is a
social convention + UI affordance, not a cryptographic guarantee.

UX is explicit: "Deleting for everyone removes this message from
their conversation view, but they may have already seen it."

---

## 9. Federation

### 9.1 Current state (2026-05)

Cross-server MLS federation is **in IETF working-group draft**
(`draft-ietf-mls-federation`, current rev as of 2025) but not yet
RFC. The draft proposes:

- **Federated Authentication Service (AS):** trusted directory service
  for resolving user@domain → KeyPackage.
- **Federated Delivery Service:** server-to-server protocol for
  routing Welcome / Commit / message between domains.
- **Cross-domain group lifecycle:** Add member from domain X to a
  group on domain Y; Commit fans out via inter-server protocol.

Until federation is RFC-stable, oyatie's federation strategy is:

| Path | Status | Trust model |
|---|---|---|
| oyatie ↔ oyatie federation across packs / cells | Internal (single-domain MLS at metadata level; cell-routing only) | Single trust domain (oyatie); cross-cell traffic per ADR-0044 + ADR-0148 |
| oyatie ↔ Matrix federation | Bridge (see §9.2) | Bridge is a trusted intermediary; ciphertext re-encrypted at bridge |
| oyatie ↔ Signal / iMessage / WhatsApp | Not supported initially | Not supported; closed protocols |
| oyatie ↔ XMPP / SimpleX | Bridge (see §9.3); deprioritised | Bridge |

### 9.2 Matrix bridge

The Messenger PRD pins Matrix Client-Server r0.6.1 + Server-Server
r0.1.4 (LTS) as the federated transport. Within Matrix, E2E is
historically Olm/Megolm (Matrix's Signal-Protocol-derived layer);
Matrix is migrating to MLS per matrix.org's "MLS in Matrix" MSC4244
(rolling 2024-2025).

oyatie's Matrix bridge:

1. **Bridge as authenticated server** — oyatie operates a Matrix
   homeserver-equivalent for the `@*:oyatie.example.com` domain.
2. **MLS-to-MLS bridge (target)** — when both ends use MLS, the
   bridge relays the MLS messages directly. The bridge does NOT
   decrypt; it forwards. (This requires Matrix's MLS migration to
   complete.)
3. **Decrypt-and-re-encrypt bridge (interim)** — for Matrix endpoints
   still on Olm/Megolm, the bridge:
   a. Holds keys for both protocols.
   b. Receives MLS from oyatie's side; decrypts; re-encrypts as
      Olm/Megolm for Matrix recipients.
   c. The bridge is a trust-boundary; users of federated chats are
      shown an explicit "federated chat: encryption ends at the
      bridge" UX indicator.
4. **Per-room federation policy** — users can opt in to federation
   per room; default for personal rooms is "oyatie-only."

The bridge runs as an `oyatie.messenger.federation-bridge` principal
under ADR-0242 sub-scope semantics; its activity is audited.

### 9.3 Other federations (deprioritised)

XMPP, SimpleX, and Briar federations are deprioritised until the
core MLS standard + Matrix bridge are stable. Future ADRs will
address as user demand emerges.

### 9.4 Future: native MLS federation when draft-ietf-mls-federation
finalises

When the federation draft becomes RFC, oyatie will:

1. Implement the federated AS + DS interfaces in `messenger`
   µservice + `identity` µservice.
2. Migrate the Matrix bridge to MLS-native federation (no
   re-encryption needed).
3. Publish oyatie's federation policy (which domains are accepted,
   under what trust criteria).

---

## 10. Forward secrecy + Post-compromise security

### 10.1 Forward secrecy (FS)

**Property:** ciphertext from epoch N cannot be decrypted given the
state of any epoch M > N.

**Mechanism (MLS):**
- Each epoch's secrets are derived from the previous epoch's secrets
  via a one-way KDF chain.
- On epoch advance (any Commit), the prior epoch's secrets are
  zeroized by `mls-rs` (as documented in the crate's secret-handling
  guide).
- Even an adversary with the current state cannot derive prior
  epoch keys; the KDF chain is one-way.

**Practical FS lifetime:**
- Epoch advances on every group change (add, remove, update).
- Routine `update_own_leaf` advances epochs every 7 days per device.
- For a group of 5 active members, expect 5+ epochs per week from
  routine updates alone.
- A single epoch's ciphertext exposure is bounded to the time between
  that epoch's Commit and the next one — typically hours.

### 10.2 Post-compromise security (PCS)

**Property:** after a compromised member is **removed** and the
group advances at least one epoch, the compromised member has zero
forward access.

**Mechanism (MLS):**
- Removal Commit proposes a new `path_secret` for the subtree
  containing the removed leaf.
- All other members process the Commit, derive the new path secret;
  the removed member cannot derive it (their leaf private key is
  excluded from the new path).
- Subsequent messages encrypt under the new epoch secret, which the
  removed member cannot reach.

**Important caveat — "self-healing":** Even if a member's state is
compromised but they are NOT removed (e.g., the user doesn't know
they were compromised), the next `update_own_leaf` operation by that
member (or any other) advances the epoch, and the compromise is
self-healed — provided the new leaf private key is uncompromised.

This "self-healing on update" property is core to MLS's value:
membership rotation + key updates provide PCS without explicit
removal. It's the reason updates are scheduled aggressively (every
7 days per device).

### 10.3 Compromise indicators

What triggers a compromise-response workflow:

| Indicator | Source | Response |
|---|---|---|
| User reports device stolen or lost | Account dashboard | Server initiates: revoke OIDC tokens; refuse device's KeyPackage publishes; emit removal proposal to all groups |
| Device's OS reports unauthorised access (e.g., iOS Keychain corruption alert) | Device-local | Client emits "self-compromise" event; rotates keys; user prompted to verify |
| Anomalous behaviour from device (e.g., sending high-volume to new recipients) | Server-side metadata analytics | Server flags; rate-limits; emits notification to other user devices |
| User manually rotates keys via "Reset encryption" in app settings | User-initiated | Client emits rotation; all groups get a new LeafNode |
| AAGUID metadata reveals attestation revoked (FIDO-MDS3 status: revoked) | Server-side FIDO-MDS3 refresh | Server refuses new KeyPackages from that AAGUID; user notified to use a different authenticator |
| Tenant-administered force-rotate (e.g., security policy) | Tenant admin (`oyatie.security.*` principal) | Server forces rotation; all groups receive removal+re-add for the affected device |

### 10.4 Self-test: verify FS + PCS in CI

The Messenger µservice's `tests/e2e/forward_secrecy.rs` and
`tests/e2e/post_compromise_security.rs` test cases:

```rust
#[tokio::test]
async fn forward_secrecy_prior_epoch_unreadable() {
    let mut alice = MlsTestClient::new("alice").await;
    let mut bob = MlsTestClient::new("bob").await;
    let group = alice.create_group_with(&bob).await;

    // Alice sends a message at epoch 1
    let msg1 = group.send_application("hello at epoch 1").await;
    let snapshot_epoch1 = alice.snapshot_state(&group);

    // Advance epoch: Alice rotates her leaf
    group.alice_update_leaf().await;
    assert_eq!(group.current_epoch(), 2);

    // Capture alice's epoch-2 state
    let snapshot_epoch2 = alice.snapshot_state(&group);

    // Attempt to decrypt msg1 with epoch-2 state ONLY (no epoch-1 keys)
    let attempted = decrypt_with_state_only(&msg1, &snapshot_epoch2);
    assert!(attempted.is_err(), "Forward secrecy violated: epoch-1 msg readable from epoch-2 state");
}

#[tokio::test]
async fn post_compromise_security_removed_member_no_forward_access() {
    let mut alice = MlsTestClient::new("alice").await;
    let mut bob = MlsTestClient::new("bob").await;
    let mut charlie = MlsTestClient::new("charlie").await;
    let group = alice.create_group_with_members(&[&bob, &charlie]).await;

    // Charlie is compromised — we capture all of charlie's state
    let charlie_compromised_state = charlie.full_state_snapshot();

    // Alice removes charlie
    group.remove_member(&charlie).await;
    assert_eq!(group.current_epoch(), 2);

    // Alice + Bob exchange messages
    let msg = group.send_application_from(&alice, "secret after removal").await;

    // Attempt to decrypt with charlie's pre-removal state
    let attempted = try_decrypt_with_compromised_state(&msg, &charlie_compromised_state);
    assert!(attempted.is_err(), "PCS violated: removed member retained forward access");
}
```

These tests run in CI on every commit to `microservices/messenger/`.

---

## 11. Encrypted backups

### 11.1 Why backups

MLS gives forward secrecy by destroying prior epoch keys. If a user
loses their last device, message history before the loss is
cryptographically unrecoverable from the server (server has only
ciphertext; the keys are gone with the device).

Most users expect recoverable history. The solution is **client-side-
encrypted backups** stored server-side, with backup keys derived from
**user-known material** (passkey, encryption-key BYOK per ADR-0251 §D-10).

### 11.2 Backup contents

Per-user backup includes:

| Item | Purpose | Size estimate |
|---|---|---|
| Device list | Resume which devices exist | ~10 KB |
| Group membership list (per group: group_id, members, current epoch) | Resume which groups the user is in | ~100 KB (typical user) |
| Recent message history (per group: last 30 days) | Resume conversation context | ~10-100 MB |
| Attachment references (per attachment: blob URL, AEAD key) | Resume attachment access | ~10 KB |
| User settings (notification preferences, contact aliases) | Resume customisation | ~10 KB |

The backup excludes:
- MLS private keys (per-device; not portable; new device generates
  new keys).
- Other-user's private keys (we never have these).
- Plaintext message bodies (encrypted with the backup key).

### 11.3 Backup key derivation

Two modes:

#### 11.3.1 Passkey-derived backup key

The user's passkey signs a deterministic challenge (per WebAuthn
`largeBlob` extension or `prf` extension):

```
backup_seed = WebAuthn.prf.evaluate("oyatie-messenger-backup-key-v1")
backup_key = HKDF-SHA-256(backup_seed, salt="oyatie-msgr-bkup-v1", info="backup", L=32)
```

The passkey's PRF capability (per WebAuthn Level 3) provides a
deterministic 32-byte output for the same input across re-evaluations
on the same authenticator. The output never leaves the authenticator
in a form usable by the relying party except through the PRF
extension.

The backup key:
- Is never stored on the server.
- Is regenerable on any device with passkey access.
- Is lost permanently if all passkeys are lost.

#### 11.3.2 encryption-key BYOK (Bring Your Own Key; ADR-0251 §D-10)

For users who want explicit control, encryption-key BYOK lets them (ADR-0251 §D-10):

1. Generate a 32-byte random seed locally.
2. Store the seed in their own key management (1Password, Bitwarden,
   YubiKey, hardware token).
3. Use the seed as the backup key directly.

UX: at backup setup, user is offered "passkey-derived" (default) or
"encryption-key BYOK" (advanced; ADR-0251 §D-10). encryption-key BYOK requires the user to confirm they understand
they're responsible for the seed; oyatie cannot recover it.

### 11.4 Backup format

```
struct OyatieMessengerBackup {
    version: u16,                       // 0x0001
    user_account_id: UUID,
    backup_created_at: HLC,
    backup_key_derivation: {
        kind: "passkey-prf" | "byok",
        prf_input: bytes (if passkey-prf),
    },
    ciphertext_aead: u16,               // 0x0001 = AES-256-GCM
    nonce: [u8; 12],
    encrypted_payload: bytes,           // gzip(serialize(BackupContents))
    signature: bytes,                   // Ed25519 over the above by the user's primary device's signing key
}
```

The `encrypted_payload` is AES-256-GCM with key = `backup_key`,
nonce = random 96-bit, AAD = the metadata fields.

### 11.5 Backup storage

Backups are stored in **SeaweedFS** (per cell, per ADR-0211 in-house
tech-stack policy):

- Per-user directory: `/messenger-backups/<tenant>/<account-id>/`.
- Versioned: each backup creation produces a new immutable object;
  prior versions retained for 30 days.
- Encryption-at-rest: SeaweedFS layer adds AES-256 envelope
  encryption with per-cell DEK (defense in depth; the backup is
  already client-encrypted).
- Replication: per-cell + per-pack per ADR-0049 residency rules.

### 11.6 Backup cadence

- **First backup:** offered to the user at first multi-device sync.
- **Incremental backups:** every 24 hours per device (incremental
  payload only — deltas since last full backup; full backup every 7
  days).
- **On-demand:** user can trigger a backup via "Back up now" in
  settings.

Each backup emits a `MessengerBackupCreated` audit event with the
backup blob's content hash + size.

### 11.7 Backup restoration

When a user installs the app on a new device:

1. User signs in with passkey.
2. Server lists available backups (by account_id).
3. User chooses "Restore from backup" (or "Start fresh").
4. New device fetches the latest backup blob from SeaweedFS.
5. New device derives backup_key via passkey-PRF or prompts for encryption-key BYOK
   seed.
6. New device decrypts the backup payload, restores:
   a. Local message history (re-creates encrypted-at-rest in the
      device's secure storage).
   b. Contact list, settings.
   c. **Group membership knowledge** — but the device still needs to
      be re-added to each group's MLS state by another of the user's
      online devices (§5.7) OR by another group member, because the
      restored backup contains *historical* group state, not the
      *current* group epoch keys.
7. Once re-added to groups, the device participates normally.

The "must be re-added to groups" point is the honest trade-off: the
backup recovers the user-visible state but not the cryptographic
keys (because the user's prior device's MLS keys are not in the
backup — that would violate forward secrecy from the prior epochs).

### 11.8 Cross-jurisdiction backup

Per ADR-0240, backups respect data residency:

- pack-kr user's backups stored in pack-kr SeaweedFS only.
- pack-eu user's backups stored in pack-eu SeaweedFS only.
- Cross-pack travel: a pack-kr user travelling to EU still backs up
  to pack-kr (home pack); the EU SeaweedFS replicas may temporarily
  cache for read latency but the canonical store is home-pack.

---

## 12. Voice + Video E2E

### 12.1 Overview

Voice + video calls between personal users are E2E-encrypted with
**media keys derived from the MLS epoch**. The MLS group is used for:

- **Signaling**: call setup (SDP offer/answer), ICE candidate exchange,
  call control (mute/unmute, hold/resume), call termination.
- **Media key derivation**: a per-call symmetric key is derived from
  the MLS group's current epoch secret + a call-specific salt.

The actual media path (RTP audio + video packets) is **DTLS-SRTP**
(RFC 5764) with the master key derived as above. This means:

- Signaling is MLS-encrypted (only group members can read).
- Media is SRTP-encrypted (only the call participants — who are a
  subset of the group — can decrypt audio/video).
- The SFU (Selective Forwarding Unit, if used for group calls) is
  untrusted; the SFU forwards encrypted RTP packets without ever
  having keys.

This design mirrors Meet IP-012's W3C Insertable Streams approach
(SFrame), but applied at the SRTP layer for native clients.

### 12.2 Call setup (1:1)

```
[Alice's device]                          [Server]                          [Bob's device]
       |                                     |                                     |
       |--- POST /voice-call/init ---------->|                                     |
       |   (Alice → Bob, video=true)         |                                     |
       |                                     |--- push notification --------------|
       |                                     |   (kind: incoming_call)             |
       |                                     |                                     |
       |                                     |<-- WebSocket connect ---------------|
       |                                     |                                     |
       |--- MLS msg in group: call-offer --->|--- forward MLS msg --------------->|
       |   { type: "call_offer",             |                                     |
       |     sdp: <SDP offer>,               |                                     |
       |     call_id: <uuid>,                |                                     |
       |     dtls_fingerprint: <hex>,        |                                     |
       |     media_key_salt: <bytes> }       |                                     |
       |                                     |                                     |
       |                                     |<-- MLS msg: call-answer ------------|
       |<-- forward MLS msg -----------------|   { type: "call_answer",            |
       |                                     |     sdp: <SDP answer>,              |
       |                                     |     dtls_fingerprint: <hex> }       |
       |                                     |                                     |
       |--- ICE candidates (MLS msg) ------->|--- forward (MLS msg) ------------->|
       |                                     |                                     |
       |<-- ICE candidates (MLS msg) --------|<-- ICE candidates (MLS msg) --------|
       |                                     |                                     |
       |   ============================ DTLS-SRTP handshake ===========================
       |   <----- direct or via TURN relay (encrypted) ------>
       |   <----- media flows over SRTP ------>
       |                                                                              |
```

Notes:

- The SDP includes DTLS fingerprints; both parties verify the
  fingerprints match expected values (within the MLS-signed message)
  to defeat MitM at the DTLS layer.
- ICE candidates are exchanged as MLS application messages (encrypted
  within the group).
- The media-key derivation:
  ```
  media_master_key = HKDF-Expand(
      epoch_secret_of_current_mls_group,
      info = "oyatie-voice-call" || call_id || media_key_salt,
      L = 32
  )
  ```
  This master key is fed into the DTLS-SRTP key extractor (RFC 5764
  §5.1) for the SRTP master key.

### 12.3 Group calls

For group voice/video calls (3+ participants), the design uses an
SFU (Selective Forwarding Unit). The SFU is per-cell, runs the
LiveKit server (per Meet's IP-014-huddles-livekit-signaling).

Per-participant keys via TreeKEM:

1. Call is initiated within an existing MLS group.
2. The MLS group's current epoch secret is the master key source.
3. Each participant derives a per-participant SRTP key:
   ```
   per_participant_srtp_key = HKDF-Expand(
       epoch_secret,
       info = "oyatie-group-call-participant" || call_id || participant_leaf_index,
       L = 32
   )
   ```
4. The SFU receives SRTP packets, forwards them to recipients
   without decryption (it can route based on RTP header info — SSRC,
   timestamp, sequence number — none of which require decryption).
5. Recipients decrypt with the per-participant key derived for the
   sender.

This is structurally identical to Meet's IP-012 SFrame approach but
operates at the SRTP layer rather than the WebRTC encoded-transform
layer; native clients use SRTP (faster) while browser clients use
WebRTC Insertable Streams.

### 12.4 Call recording

For personal calls, recording is **disabled by default**. Per
Meet IP-012, recording during an E2E call is denied by Cedar policy
because the recording would either:

- Capture the cleartext at the recording client (requiring decryption
  → no longer untrusted-server property).
- Capture the SRTP ciphertext (un-replayable for human review).

User-initiated local-only recording (one party records on their own
device, with all participants notified) is permitted with consent
prompts; the recording is stored on the recording user's device and
not transmitted to the server.

### 12.5 Call quality metrics

The SFU emits metrics (jitter, packet loss, bandwidth) without
seeing media content; these metrics are useful for QoS but don't
compromise privacy.

---

## 13. File + Photo + Video attachment encryption

### 13.1 Per-attachment AES-256-GCM

Attachments (any binary blob — photos, videos, documents, voice
notes) are encrypted with a **per-attachment AES-256-GCM** key:

```
attachment_key = random_32_bytes()
nonce = random_12_bytes()
ciphertext, tag = AES-256-GCM-Encrypt(
    key = attachment_key,
    nonce = nonce,
    plaintext = attachment_bytes,
    aad = attachment_metadata_hash
)
```

The attachment_key is generated **client-side** at upload time;
distinct for each attachment.

### 13.2 Key wrapping via MLS message

The attachment_key is delivered to recipients by including it in an
MLS-encrypted message:

```json
{
  "type": "file_ref",
  "blob_url": "https://blob.oyatie.example.com/v1/blobs/<id>",
  "blob_size": 1048576,
  "content_type": "image/jpeg",
  "encryption": {
    "aead": "AES-256-GCM",
    "key": "<base64 32 bytes>",
    "nonce": "<base64 12 bytes>",
    "aad_metadata_hash": "<base64 32 bytes>"
  },
  "preview": {
    "thumbnail_blob_url": "https://blob.oyatie.example.com/v1/blobs/<thumb-id>",
    "thumbnail_encryption": { ... }
  },
  "metadata": {
    "filename": "photo.jpg",
    "dimensions": [1920, 1080]
  }
}
```

The entire structure is encrypted as an MLS application message;
recipients decrypt to obtain the `attachment_key`, then fetch the
blob from SeaweedFS and decrypt it.

### 13.3 Blob storage in SeaweedFS

The encrypted blob is stored in SeaweedFS (per ADR-0211 in-house
storage):

- Server receives ciphertext + metadata via authenticated upload.
- Server stores in SeaweedFS volume; returns blob URL.
- Server is **blind** to content (cipher is opaque).
- Retention: by default, 90 days after last access; configurable.

The server may compute the SHA-256 of the **ciphertext** for
deduplication (same encrypted bytes = same blob). It cannot
deduplicate by plaintext because keys differ per upload.

### 13.4 Thumbnail preview

For images + videos, the client generates a small thumbnail (say
256×256, ~20 KB), encrypts with a separate per-thumbnail key, and
uploads as a separate blob. The MLS message includes both blob refs.

Recipients show the thumbnail (decryptable quickly) while
downloading the full blob lazily. The thumbnail key is distinct
from the full-blob key so the full blob can be revoked / purged
while leaving the thumbnail visible (or vice versa).

### 13.5 Streaming media

For videos > a threshold (say 10 MB), the client uses chunked
encryption + streaming:

- Source video is split into chunks of ~1 MB.
- Each chunk encrypted with the same `attachment_key` but a
  per-chunk nonce derived from the base nonce + chunk index.
- Server stores chunks as a manifest + N chunks.
- Recipients can play the video by fetching chunks sequentially,
  decrypting each chunk inline (HLS-style — encrypted HLS).

### 13.6 Voice notes

Voice notes are recorded as Opus codec (RFC 6716), encoded at
24 kbps. A 60-second voice note is ~180 KB. Encryption is identical
to other attachments.

### 13.7 Attachment limits

| Limit | Value | Rationale |
|---|---|---|
| Max single attachment size | 100 MB (configurable to 5 GB per Messenger PRD FR-04 for professional) | Personal-context default; PR-04 covers professional |
| Max attachments per message | 10 | UX limit; avoid mega-messages |
| Allowed MIME types | All; client-side validation only | Server cannot inspect content |
| Virus scanning | NOT performed server-side for personal | Server cannot decrypt; client-side perceptual-hash check (§6.9.3) on upload |

### 13.8 Attachment deletion

When a user deletes a message containing an attachment (§8.7.2 delete-
for-all), the server purges the blob from SeaweedFS within 24 hours
(eventual deletion, per blob storage backend's GC cadence). The blob
key in recipients' MLS history is also tombstoned but not crypto-
shredded (client behaviour to retain decrypted preview is allowed).

For attachments that were never explicitly deleted, the retention TTL
(per pack policy + user preference) governs purge.

---

## 14. Sticker + GIF + emoji

### 14.1 Stock emoji (Unicode)

Unicode emoji (Unicode 16, October 2024) are characters in the
message text; they're encrypted as part of the text body. No
special handling.

### 14.2 Custom stickers

Users can purchase or create custom sticker packs. Each sticker is a
small image (typically 256×256 to 512×512, transparent PNG or WebP).

- Stickers are uploaded as attachments (§13).
- Sticker packs are managed per-user; the user "owns" the unencrypted
  rights to use the sticker.
- When sending a sticker to a contact, the client treats it as a
  small attachment — encrypts the image with a per-message
  attachment key, sends.

For frequently-used stickers, the recipient's client caches the
decrypted blob locally; future receives of the same sticker (re-sent
in the same group) are decrypted but reference the cache for display.

### 14.3 GIFs

Two paths:

**Path A — Encrypted GIF (default):** the GIF is uploaded as an
attachment, identical to images.

**Path B — External GIF reference (when user picks from a public
library like Giphy):**

```json
{
  "type": "external_gif_ref",
  "provider": "giphy",
  "gif_id": "abc123",
  "gif_url": "https://media.giphy.com/...",
  "preview_hash": "<sha256>",
  "dimensions": [480, 270]
}
```

- The GIF URL points to a public CDN (Giphy).
- The recipient's client fetches the GIF from Giphy directly (with
  the user's consent — Giphy sees the recipient's IP + the GIF ID).
- The preview_hash is computed by the sender client (over the actual
  GIF bytes) and included in the MLS message; recipient verifies
  the fetched bytes hash matches, defeating Giphy-side replacement.
- Privacy: the *sender* is not exposed to Giphy (the sender never
  re-fetched; they have a local copy). The *recipients* are exposed
  to Giphy (each recipient fetches independently).

User UX explicitly notes: "Sharing public GIFs may expose recipient
IP addresses to the GIF provider."

### 14.4 Reactions

Emoji reactions on messages (👍 ❤️ 😂) are MLS-encrypted control
messages:

```json
{
  "type": "reaction",
  "reacts_to_message_id": "<uuid>",
  "emoji": "👍",
  "added_at": "<HLC>"
}
```

Each reaction is a normal MLS application message; same encryption.

Reactions can be removed (a "remove_reaction" control message; same
shape with negative semantic).

---

## 15. Group call (Meet integration)

Per the Messenger PRD §Open Questions Q2 ("Voice / video signaling:
own µservice or messenger BC?") and the existing Meet µservice's
IP-012 + IP-014, **group voice/video calls reuse the Meet µservice's
LiveKit-based infrastructure**.

### 15.1 Integration shape

When personal users in a group decide to start a voice/video call:

1. Within the MLS group, one user initiates a call:
   ```
   POST /v1/messenger/groups/{group_id}/call/start
   ```
2. `messenger` µservice forwards to the `meet` µservice (per
   ADR-0145 inter-microservice-communication-reform via gRPC):
   ```
   meet.CreateCallRequest {
       group_id: ...,
       initiator_device_id: ...,
       call_kind: "voice" | "video",
       e2e_mode: true,
       mls_group_state_ref: ...
   }
   ```
3. `meet` provisions an SFU instance (per cell).
4. `messenger` emits an MLS control message in the group: "call
   started, join URL: ...".
5. Each participating client connects to the SFU; cryptographic keys
   are derived from the MLS epoch (§12).

### 15.2 Cedar policy: recording disabled in E2E

Per Meet IP-012's Cedar policy (which this standard inherits):

```cedar
forbid (
  principal,
  action in [
    Action::"start_recording",
    Action::"start_transcription",
    Action::"start_ai_summary"
  ],
  resource is MeetingInstance
) when {
  resource has e2e_mode &&
  resource.e2e_mode == true
};
```

Personal calls are always `e2e_mode = true`; recording, transcription,
and AI summary are denied.

---

## 16. Implementation choices

### 16.1 Cryptographic library: mls-rs

**Choice: `mls-rs` (github.com/awslabs/mls-rs, Apache 2.0).**

Justification (per ADR-0211 in-house tech-stack policy applied to MLS):

| Criterion | Verdict |
|---|---|
| Open-source | Yes (Apache 2.0); permissive |
| Maturity | Production at AWS Wickr since 2023; used by Cisco Webex via internal port |
| Audit | Cure53 in 2023 (mls-rs); no critical findings |
| RFC conformance | Passes IETF MLS interop matrix (mls-rs vs openmls vs mlspp) |
| Active maintenance | Weekly commits as of 2026-05; AWS-funded maintainers |
| Rust-native | Yes; no FFI inside the core |
| WASM-compilable | Yes (proven by `mls-rs` examples) |
| FIPS support | Yes via AWS-LC backend (2024+) |
| Hybrid-PQ roadmap | Tracked; behind a feature flag as of 2025 |

Alternative considered: **`openmls`** (github.com/openmls/openmls,
MPL-2.0). Strengths: academic provenance (TU Darmstadt + INRIA);
formally-verified components (HACL*-derived crypto). Weaknesses:
slower release cadence than mls-rs; FIPS support more nascent.

**Decision:** mls-rs primary; openmls as a contingency option. The
`messenger-mls-adapter` BC isolates the choice behind a trait
boundary so a future swap is mechanical (per ADR-0211 §"Phase 2"
posture).

### 16.2 Server-side stack

Per ADR-0211 in-house tech-stack policy + per ADR-0145 inter-µservice
communication reform:

| Component | Choice | Rationale |
|---|---|---|
| Programming language | Rust (1.80+) | per ADR-0211; performance, safety |
| Async runtime | tokio | de facto Rust async runtime |
| HTTP / gRPC server | axum (HTTP) + tonic (gRPC) | per ADR-0145; tower-based, composable |
| WebSocket | tokio-tungstenite | RFC 6455 compliant |
| Postgres client | sqlx | compile-time-checked queries |
| Postgres | Postgres 17 LTS | per portfolio standard |
| KV / cache | Valkey | OSS Redis fork; per portfolio standard |
| Object storage | SeaweedFS | per ADR-0211 in-house |
| Audit chain | `microservices/audit-chain/` | per ADR-0028 inheritance |
| Identity | `microservices/identity/` (Zitadel) | per ADR-0187 |
| Authorisation | Cedar | per ADR-0150 + ADR-0246 |
| Secrets management | OpenBao | per `microservices/cloud-secrets/` |
| Observability | OpenTelemetry → Tempo + Loki + Prometheus | per portfolio standard |
| HLC | Per ADR-0252 (Time Coordination) | inherited |

### 16.3 Client-side stack

| Platform | Language | UI framework | Native bridge to mls-rs |
|---|---|---|---|
| iOS / iPadOS | Swift | SwiftUI | C-ABI FFI via `cxx` or `swift-bridge` |
| Android | Kotlin | Jetpack Compose | JNI |
| macOS | Swift | SwiftUI (cross-platform with iOS) | C-ABI FFI |
| Windows | Rust + TypeScript | Tauri 2.x (web shell) | direct |
| Linux | Rust + TypeScript | Tauri 2.x | direct |
| Web | TypeScript | React 19 | mls-rs compiled to WASM; wasm-bindgen |

The cryptographic core (`mls-rs`) is shared across all platforms;
platform-specific code is only the UI + storage + push integration.

### 16.4 Cell-local components

Per ADR-0131 per-microservice flat layout, the `messenger` µservice
adds these BCs for E2E:

| BC | Purpose |
|---|---|
| `e2e-mls-group-store` | Group metadata + member list + epoch tracking |
| `e2e-mls-message-relay` | Ciphertext storage + delivery + ordering |
| `e2e-mls-keypackage-registry` | Per-device KeyPackage publication + lookup |
| `e2e-mls-welcome-router` | Welcome message routing on add-member |
| `e2e-mls-commit-router` | Commit message fan-out + epoch sync |
| `e2e-mls-backup-store` | Encrypted backup management |
| `e2e-mls-audit-emitter` | Audit chain emission for E2E events |

Each BC follows the 13-layer canonical structure (ADR-0105) with
crates: `kernel`, `domain`, `usecase`, `api`, `adapter`,
`adapter-postgres`, `adapter-redis`, `adapter-websocket`,
`adapter-mls-rs`, `rest`, `worker`, `sdk`, `app`.

### 16.5 Per-platform secure storage

| Platform | Secure storage API | Wraps |
|---|---|---|
| iOS / iPadOS / macOS | Keychain (Security framework); `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` | Long-term Ed25519 private; group state DEK |
| Android | Android Keystore (KeyProperties); StrongBox when available | Long-term Ed25519 private; Room DB encryption key |
| Windows | Platform Crypto Provider (TPM-backed); DPAPI for non-key data | Long-term Ed25519 private; SQLite DEK |
| Linux | TPM 2.0 via tpm2-tss; libsecret for cached references | Long-term Ed25519 private; SQLite DEK |
| Web | WebAuthn CryptoKey (non-extractable); IndexedDB sealed under origin | Per-session ephemeral signing; never long-term |

Group state at rest is encrypted with a device-scoped DEK derived
from the user's passkey + a salt; the DEK is held in OS secure
storage, not the message store directly.

### 16.6 In-process testing

`tests/e2e/` contains:

- `mls_handshake_smoke.rs` — basic 2-member group creation.
- `mls_group_scale.rs` — 1000-member group creation + 10k messages.
- `mls_multi_device.rs` — user with 4 devices joins group.
- `mls_member_remove.rs` — PCS on member-remove.
- `mls_offline_delivery.rs` — offline-online catch-up.
- `mls_backup_restore.rs` — backup creation + restoration on new device.
- `mls_attachment.rs` — attachment encrypt + upload + decrypt cycle.
- `mls_call_signaling.rs` — voice call signaling via MLS group.

Acceptance criteria (must run green in CI per Messenger PRD AC matrix):

- `cargo nextest run -p messenger-e2e-mls-*` exits 0.
- `cargo run -p dev-cli -- gate validate cedar-coverage --microservice messenger` exits 0.
- `cargo run -p dev-cli -- gate validate mls-conformance --microservice messenger` exits 0 (new lane; checks against IETF MLS interop matrix).

---

## 17. Threat model

### 17.1 Trust assumptions

| Entity | Trust |
|---|---|
| End device (the user's phone, laptop, etc.) | Trusted: must operate correctly; if compromised, blast radius is limited via PCS |
| User's passkey authenticator | Trusted: WebAuthn Level 3 hardware-bound (ADR-0188) |
| oyatie server (`messenger` µservice + cells) | Untrusted relay: stores ciphertext, observes metadata, cannot decrypt |
| Other group members | Honest-but-curious: assumed to follow protocol; a compromised member is removed via PCS |
| Network adversary (ISP, WiFi attacker, nation-state SIGINT on the wire) | Untrusted: TLS + MLS together provide secrecy + integrity |
| Push provider (APNs, FCM) | Untrusted: sees only hashes and counts, no plaintext |
| External CDN for GIFs (Giphy) | Untrusted: sees IP + GIF ID per fetch; user is warned |
| Federation peers (Matrix bridges) | Trusted to a documented degree; users warned at bridge |
| Authentication service (Zitadel) | Trusted: per ADR-0187; provides identity, not cryptographic keys |
| AAGUID issuer (FIDO Alliance MDS3) | Trusted within FIDO Alliance's trust framework |

### 17.2 Threats considered

#### 17.2.1 Network adversary

**Goal:** read plaintext, modify messages, replay messages, impersonate.

**Mitigation:**
- TLS 1.3 (per ADR-0145) + mTLS on WebSocket (per PRD).
- MLS provides E2E secrecy + integrity above TLS; the network adversary
  sees only TLS-wrapped MLS ciphertext.
- Replay protection via MLS sequence numbers (RFC 9420 §9.1) +
  server-side HLC enforcement.
- Impersonation requires the device's long-term signing key (HW-bound).

#### 17.2.2 Compromised server

**Goal:** read plaintext, deliver fake messages, MitM new members.

**Mitigation:**
- Server has no keys; cannot decrypt.
- Server-forged messages would fail recipient's signature verification
  (signature is by the sender's long-term key).
- Server-forged Welcome messages: a fabricated Welcome could "add" a
  fake device to a group. Mitigation: the addee verifies their own
  passkey-attestation extension is signed by a user-recognised
  identity; the safety-number / device-fingerprint UX (§4.6) warns
  users on unexpected device additions.
- Server denying service (drop messages): users see "message
  pending"; server-side SLO + alerting flag denial.

#### 17.2.3 Compromised device

**Goal:** ongoing access to user's messages.

**Mitigation:**
- User removes the device from all groups + revokes the long-term
  key.
- After removal + one epoch advance, the device has zero forward
  access (PCS).
- Server emits a `DeviceCompromiseReported` audit event; per-tenant
  policy may force-rotate all of the user's KeyPackages.

#### 17.2.4 Compromised member (one of N in a group)

**Goal:** read all group messages.

**Mitigation:**
- Group's other members remove the compromised member.
- After removal + epoch advance, the compromised member has zero
  forward access.
- The compromised member's keys to retroactive messages (epochs
  before removal) are unaffected — the protocol cannot un-share what
  was already shared.

#### 17.2.5 Quantum adversary ("harvest now, decrypt later")

**Goal:** capture today's ciphertext, decrypt in 10-20 years when
CRQC exists.

**Mitigation:**
- Hybrid PQ cipher suite (§3.5) deployed by 2027.
- Messages whose confidentiality has elapsed (e.g., a casual chat
  about lunch from 2026) have no value in 2040.
- Messages with long confidentiality lifetimes (e.g., personal
  health records, legal correspondence) should be on the hybrid suite
  before 2030.

#### 17.2.6 Side-channel attacks

**Goal:** infer plaintext from observed timing, traffic patterns,
power consumption.

**Mitigation:**
- **Timing:** mls-rs uses constant-time primitives (curve25519-dalek,
  HACL\*-derived AES); no early-exit on signature verify; no
  branch-on-secret patterns. Audited by Cure53 2023.
- **Traffic analysis:** message-size padding optional (round up to
  next multiple of 256 bytes); cover traffic NOT implemented in v1
  (cost/benefit not justified for personal messaging at scale; would
  be a v2 feature).
- **Power:** out of oyatie's threat model (device-level mitigation
  is the OS/SoC's responsibility).

#### 17.2.7 Metadata leakage

**Goal:** infer who talks to whom, when, how often, from server
metadata.

**Mitigation:**
- Server logs are encrypted at rest.
- Per-jurisdiction retention bounds (§18).
- Server metadata is unavoidably visible to the server operator
  (oyatie); the privacy contract is "we won't read your messages; we
  do know who you talk to."
- Sealed Sender (Signal's pattern of hiding sender identity from
  server) is considered for v2 but not v1 because it complicates
  abuse-mitigation (you can't rate-limit a sender you can't identify).

#### 17.2.8 Coercion / legal compulsion

**Goal:** government compels oyatie to reveal user data.

**Mitigation:**
- For personal context, the cleartext is unavailable to oyatie;
  compulsion can only yield ciphertext + metadata.
- Per ADR-0240, sovereign-cloud overlays place data under the
  jurisdiction of the regional regulator; cross-jurisdiction compulsion
  is per MLAT.
- Transparency reports: oyatie publishes quarterly transparency
  reports listing legal demands received + responses.

### 17.3 Threats explicitly out of scope

- Compromise of the user's OS / kernel (an OS-level rootkit can read
  cleartext from RAM; out of scope for application-layer crypto).
- Physical coercion of the user (rubber-hose cryptanalysis).
- Camera / microphone capture by a different app on the device.
- Account recovery without passkey + without backup (deliberate data
  loss as a privacy guarantee).

### 17.4 Threat-model evolution

This threat model is reviewed:

- Quarterly by council-security.
- On every major MLS protocol update (RFC errata, new extension RFC).
- On every reported security incident (per `microservices/messenger/runbooks/incident-response.md`).

---

## 18. Compliance considerations

### 18.1 GDPR

**Article 32 (Security of processing):** "Implement appropriate
technical and organisational measures to ensure a level of security
appropriate to the risk."

Satisfaction:
- E2E encryption with MLS (RFC 9420) is the industry-leading technical
  measure for confidentiality.
- Forward secrecy + post-compromise security exceed Article 32's
  baseline.
- Audit chain provides integrity-of-processing evidence.

**Article 17 (Right to erasure):** subject to Bominal-inherited DSAR
cascade per ADR-0242 §D-4 (DSAR machinery uniform across `oyatie` and
customer tenants).

- Personal messages: subject can request deletion; ciphertext is
  purged from server; the subject's own backups they hold remain
  (the subject controls their copies).
- Counterparties' copies of messages cannot be erased (cryptographic
  reality; cannot un-send).
- DSAR response within 30 days per Article 12.

**Article 25 (Data protection by design and by default):**
- Privacy-by-design: encryption defaults to on; cannot be disabled
  for personal context.
- Privacy-by-default: read receipts default off for large groups;
  metadata minimisation in push payloads.

### 18.2 HIPAA

**Security Rule §164.312(a)(2)(iv) (encryption):** "Implement a
mechanism to encrypt electronic protected health information (ePHI)
whenever deemed appropriate."

For personal-context messaging (Messenger PRD §1.4 establishes
personal users carry no PHI by intent), HIPAA's encryption rule does
not strictly apply. However:

- E2E with AES-128-GCM is HIPAA-compliant if PHI inadvertently
  appears (e.g., user sends a photo of a prescription).
- For HIPAA-pack tenants (professional context, deliberately PHI-
  enabled), the server-side encryption pathway is used (out of scope
  for this standard, but cross-referenced).

### 18.3 PCI DSS

**Requirement 4 (Protect cardholder data in transit):** "Render PAN
unreadable anywhere it is stored or transmitted."

For personal messaging, PCI scope is N/A; users may informally share
card data but the messenger does not handle payment processing.

If a payment-flow integration (future) routes payment through
Messenger, the payment data is encrypted at MLS + does not persist
in cleartext on the server.

### 18.4 FedRAMP-Moderate

**SC-13 (Cryptographic protection):** "Implement cryptographic
mechanisms approved by FIPS 140 standards."

- AES-128-GCM, SHA-256, HKDF-SHA-256 are FIPS-186-5 approved.
- Ed25519 + X25519 are FIPS-186-5 approved as of 2023.
- The AWS-LC FIPS module is the cryptographic provider in FedRAMP
  contexts.

**SC-28 (Protection of information at rest):** "Protect the
confidentiality and integrity of information at rest."

- Server-side blob storage (SeaweedFS) is encrypted at rest via
  per-cell DEK (defense in depth).
- Client-side message store is encrypted via OS-native secure
  storage (Keychain, Keystore, TPM).

### 18.5 KR-PIPA Article 24

**Personal data protection technical measures:** "Personal data
controllers shall implement technical, administrative, and physical
measures to prevent loss, theft, alteration, damage, or disclosure
of personal data."

- E2E encryption per RFC 9420 satisfies the "encryption-at-the-source"
  bar.
- Per ADR-0240 pack-kr overlay, processing is on CSAP-certified
  domestic substrate.
- KR PIPA Article 29 (cross-border transfer) is respected by
  pack-residency enforcement.

### 18.6 EU AI Act (where relevant)

The messenger µservice itself is not an AI system. Future AI features
(e.g., on-device transcription of voice notes) would be subject to
EU AI Act Article 6 risk classification per ADR-0144.

### 18.7 Cross-jurisdiction key custody

**Per-tenant encryption-key BYOK (ADR-0251 §D-10):** tenants may bring their own backup encryption
key (§11.3.2). The key never leaves the user's possession; the server
cannot help recover it.

**No key escrow for personal:** oyatie operates no escrow service
for personal-user keys. The privacy guarantee is "if you lose all
your devices + your passkey + your encryption-key BYOK, your historical messages
are unrecoverable."

**For regulated tenants (KR-FSS, financial-services):** key custody
may be required by regulator; per-pack overlay (§3.2.3) allows for
HSM-backed signing where the HSM is under tenant control + regulator
audit. For personal users, key custody is the user's responsibility.

### 18.8 Audit packets for regulators

Per ADR-0240 §D-6 audit + regulator evidence:

Quarterly evidence packet for messenger E2E (data class
`MessengerE2EEvidence`):

- Total MLS messages relayed per pack (counts only, no content).
- Cryptographic primitive inventory (which suites in use; FIPS
  module certificate number; mls-rs version).
- Key rotation cadence statistics (mean / p99 KeyPackage rotation
  intervals).
- Compromise incidents (counts + categories; aggregated; no PII).
- DSAR responses (counts + SLA adherence).

The packet is signed by the audit-chain key, emitted via
`microservices/audit-chain/` to the regulator endpoint per pack.

---

## 19. Operational considerations

### 19.1 Key lifecycle

| Phase | Action | Cadence |
|---|---|---|
| Genesis | New device generates long-term Ed25519 key + first KeyPackage batch | Once per device |
| Pre-publish | Maintain ≥ 100 unused KeyPackages | Continuous monitor |
| Rotation | New KeyPackage batch | Every 30 days (lifetime 90 days; 60-day overlap) |
| Update (LeafNode) | `update_own_leaf` Commit in each group | Per device, every 7 days jittered |
| Compromise rotation | Mark all KPs consumed; new long-term key; remove device from all groups | On demand |
| Device retirement | Remove from all groups; archive KPs | On user action |

### 19.2 User onboarding (first device)

```
1. User downloads app on first device.
2. App prompts: sign in with passkey or create account.
3. New account creation:
   a. User chooses display name.
   b. User registers passkey (per ADR-0188 conditional UI flow).
   c. App generates Ed25519 long-term key in HW secure storage.
   d. App publishes initial KeyPackage batch (100 KPs).
   e. App creates the user's personal device group (only this device).
4. Onboarding complete. App displays "Start a conversation."
```

### 19.3 Device pairing (additional device)

```
1. User downloads app on new device.
2. App prompts: sign in with passkey.
3. New device generates Ed25519 long-term key + publishes KPs.
4. Server emits DeviceAdded notification to all of the user's existing
   online devices.
5. User confirms on existing device: "Pair new device [model X] from
   [location] now?"
6. Existing device adds new device to:
   a. The personal device group.
   b. Every active conversation group (via Welcome).
7. New device receives Welcomes; processes; gains group membership.
8. Optional: restore backup (per §11.7).
```

If no existing device is online, the new device displays "Waiting to
be added by another of your devices." After 24 hours, the user can
override and start fresh (losing message history; offered to restore
from backup if available).

### 19.4 User offboarding (account deletion)

```
1. User initiates "Delete my account" in settings.
2. App displays warning: "This permanently deletes your account and
   all conversations from your devices. Contacts may retain copies
   of messages you sent them. This action cannot be undone."
3. User confirms with passkey assertion.
4. Server:
   a. Removes user from every group (each removal emits a Commit; PCS
      kicks in).
   b. Purges all of user's KeyPackages from registry.
   c. Purges user's encrypted backups from SeaweedFS.
   d. Purges user's metadata from messenger µservice (with audit
      chain tombstone retained per regulatory requirement).
   e. Emits `AccountDeleted` event.
5. After 30 days (grace period for change-of-mind reversal), final
   purge of audit-chain references (subject's identifier replaced
   with hash; Merkle proof retained for tamper detection per
   ADR-0242 §B).
```

### 19.5 Disaster recovery

**Per ADR-0241 (DR + business continuity portfolio policy):**

- The `messenger` µservice declares `dr_tier = T2` (< 1h RTO,
  bounded data loss).
- E2E ciphertext is replicated within the home cell + cross-cell
  within pack (RPO ~5 min).
- KeyPackage registry replicated similarly.
- Audit chain replicated independently (T1: < 5 min RTO, RPO = 0).

**Disaster scenarios:**

| Scenario | Recovery |
|---|---|
| Home cell loss (datacentre fire, regional power) | Failover to DR cell in same pack; users reconnect; messages from last 5 minutes may be lost; client retries pending sends |
| KeyPackage registry corruption | Restored from cross-cell replica; missing KPs cause "no KeyPackages for device" errors on add; users re-publish on next online |
| Audit chain seal compromise | Per ADR-0028 inheritance; rolling re-seal; gap reported to ops-compliance |
| User's all devices lost + backup lost | Cryptographic data loss is permanent; user accepts at backup-disabled setup |
| Catastrophic provider failure (cloud-substrate offline for sovereign pack) | Per ADR-0240 §D-7; stateless workloads cut over; stateful workloads follow DR; sovereign-data may be brown-out until primary recovers |

**Drills:** quarterly DR drill per ADR-0241 includes:
- Failover messenger µservice from home to DR cell.
- Restore backup of a test user to a new device.
- Verify message-send latency p99 < 100ms post-failover.

### 19.6 Backup format spec

Beyond §11.4's shape, the full backup format spec lives at
`microservices/messenger/contracts/asyncapi/backup-format.yaml`:

```yaml
asyncapi: 2.6.0
info:
  title: Messenger Backup Format
  version: 1.0.0
channels:
  backup-blob:
    publish:
      message:
        $ref: '#/components/messages/OyatieMessengerBackup'
components:
  messages:
    OyatieMessengerBackup:
      payload:
        type: object
        required: [version, user_account_id, backup_created_at, backup_key_derivation, ciphertext_aead, nonce, encrypted_payload, signature]
        properties:
          version: { type: integer, enum: [1] }
          user_account_id: { type: string, format: uuid }
          backup_created_at: { type: string, description: "HLC string" }
          backup_key_derivation:
            type: object
            oneOf:
              - properties:
                  kind: { type: string, enum: ["passkey-prf"] }
                  prf_input: { type: string, format: byte }
              - properties:
                  kind: { type: string, enum: ["byok"] }
          ciphertext_aead: { type: integer, enum: [1] }  # AES-256-GCM
          nonce: { type: string, format: byte, minLength: 16, maxLength: 16 }  # base64 12 bytes
          encrypted_payload: { type: string, format: byte }
          signature: { type: string, format: byte }
```

The decrypted payload schema is documented at
`microservices/messenger/contracts/openapi/backup-payload.yaml`.

### 19.7 Observability

| Metric | Source | Use |
|---|---|---|
| `messenger.mls.message.relay.count` | server | per-pack volume |
| `messenger.mls.message.size.bytes` (histogram) | server | capacity planning |
| `messenger.mls.delivery.latency.ms` (histogram p50/p99/p999) | server | SLO tracking |
| `messenger.mls.epoch.advance.count` | server | rotation activity |
| `messenger.mls.keypackage.consumed.count` | server | KP registry health |
| `messenger.mls.keypackage.low_water_alert.count` | server | KP rotation cadence health |
| `messenger.mls.welcome.delivery.latency.ms` | server | add-member health |
| `messenger.mls.decrypt.failure.count` | server (from client ack) | forensic |
| `messenger.mls.signature.verify.failure.count` | server | abuse signal |
| `messenger.mls.backup.created.count` | server | backup adoption |
| `messenger.mls.backup.restored.count` | server | recovery rate |

Per ADR-0252 (Time Coordination) all metrics carry HLC timestamps
for cross-cell correlation.

Per OpenSLO requirement (ADR-0139), the following SLOs are authored
at `microservices/messenger/slos/`:

| SLO | Target | Burn-rate alert |
|---|---|---|
| `mls-delivery-success` | 99.95% / 30d | 14.4× over 1h |
| `mls-delivery-latency-p99` | ≤ 100ms within region | p99 > 100ms over 5m |
| `mls-keypackage-availability` | ≥ 100 KPs per device per suite | low-water hit on any device > 1h |
| `mls-decrypt-success` | ≥ 99.99% (client-side) | failure rate > 0.01% over 1h |

### 19.8 Performance targets

Per Messenger PRD §Performance (table inherited):

- Message-send p99 ≤ 100ms (within region).
- Read-receipt fan-out p99 ≤ 150ms.
- @mention resolution p99 ≤ 250ms.
- File-attachment upload init p99 ≤ 300ms.

E2E-specific additions:

| Op | p50 | p99 | p999 |
|---|---|---|---|
| MLS handshake (2 members) | 50ms | 200ms | 500ms |
| MLS handshake (100 members) | 200ms | 800ms | 2s |
| MLS group commit (100 members) | 30ms | 100ms | 250ms |
| MLS application encrypt (1KB) | 0.5ms | 2ms | 5ms |
| MLS application decrypt (1KB) | 0.5ms | 2ms | 5ms |
| KeyPackage validate (server-side) | 1ms | 5ms | 15ms |

These targets are validated against mls-rs's benchmark set +
oyatie's integration tests.

### 19.9 Capacity model

Per cell, target steady-state:

| Resource | Baseline | Max | Scale-out trigger |
|---|---|---|---|
| Active MLS groups | 10M | 100M | postgres write IOPS > 70% |
| KeyPackages stored | 1B (100 per device × 10M devices) | 10B | shard by tenant_id + device_id |
| Messages/sec relayed | 5k | 50k | gateway CPU > 70% |
| Welcomes/sec | 100 | 1000 | similar |
| Commits/sec | 500 | 5000 | similar |
| Backup storage | 10 TB | 1 PB | SeaweedFS capacity alert |

Sharding strategies:

- MLS groups: by `(tenant_id, group_id)` mod N.
- KeyPackages: by `(tenant_id, account_id, device_id)`.
- Backups: by `(tenant_id, account_id)`.

---

## 20. References

### 20.1 IETF + W3C standards

- **RFC 9420 — *The Messaging Layer Security (MLS) Protocol*** (Barnes, Beurdouche, Robert, Millican, Omara, Cohn-Gordon; July 2023). [datatracker.ietf.org/doc/rfc9420/](https://datatracker.ietf.org/doc/rfc9420/)
- **RFC 9421 — *The Messaging Layer Security (MLS) Architecture*** (Omara, Beurdouche, Barnes, Millican, Robert; February 2024). [datatracker.ietf.org/doc/rfc9421/](https://datatracker.ietf.org/doc/rfc9421/)
- **draft-ietf-mls-extensions** — *MLS Extensions* (in-progress IETF MLS WG; multiple drafts as of 2025). [datatracker.ietf.org/wg/mls/documents/](https://datatracker.ietf.org/wg/mls/documents/)
- **draft-ietf-mls-federation** — *Federation Architecture for MLS* (in-progress IETF MLS WG, 2024-2025).
- **draft-ietf-mls-pq** — *Post-Quantum MLS Cipher Suites* (in-progress, 2024-2025).
- **RFC 9180 — *Hybrid Public Key Encryption (HPKE)*** (Barnes, Bhargavan, Lipp, Wood; February 2022).
- **RFC 8032 — *Edwards-Curve Digital Signature Algorithm (EdDSA)*** (Josefsson, Liusvaara; January 2017).
- **RFC 5869 — *HMAC-based Extract-and-Expand Key Derivation Function (HKDF)*** (Krawczyk, Eronen; May 2010).
- **RFC 8439 — *ChaCha20 and Poly1305 for IETF Protocols*** (Nir, Langley; June 2018).
- **RFC 5764 — *Datagram Transport Layer Security (DTLS) Extension to Establish Keys for the Secure Real-time Transport Protocol (SRTP)*** (McGrew, Rescorla; May 2010).
- **RFC 6716 — *Definition of the Opus Audio Codec*** (Valin, Vos, Terriberry; September 2012).
- **RFC 6455 — *The WebSocket Protocol*** (Fette, Melnikov; December 2011).
- **RFC 6238 — *TOTP: Time-Based One-Time Password Algorithm*** (M'Raihi et al.; May 2011).
- **W3C WebAuthn Level 3 — *Web Authentication: An API for accessing Public Key Credentials Level 3*** (W3C Recommendation, April 2024). [w3.org/TR/webauthn-3/](https://www.w3.org/TR/webauthn-3/)
- **W3C Insertable Streams — *WebRTC Encoded Transform*** (W3C Editor's Draft). [w3c.github.io/webrtc-encoded-transform/](https://w3c.github.io/webrtc-encoded-transform/)

### 20.2 Academic + cryptographic literature

- Bhargavan, Beurdouche, Naldurg. **"Formal modelling and verification of MLS in F\*"** (CRYPTO Tools workshop 2020).
- Alwen et al. **"Continuous Key Agreement with Reduced Bandwidth"** — Asynchronous decentralized key management (CRYPTO 2021).
- Cohn-Gordon, Cremers, Dowling, Garratt, Stebila. **"A Formal Security Analysis of the Signal Messaging Protocol"** (Journal of Cryptology 2020).
- Marlinspike & Perrin. **"The X3DH Key Agreement Protocol"** (Signal whitepaper, 2016).
- Marlinspike & Perrin. **"The Double Ratchet Algorithm"** (Signal whitepaper, 2016).
- Cremers, Hale, Kohbrok. **"The Complexities of Healing in Secure Group Messaging: Why Cross-Group Effects Matter"** (USENIX Security 2021).
- Verma, Pedrosa, Korupolu, Oppenheimer, Tune, Wilkes. **"Borg, Omega, and Kubernetes"** (CACM 2016) — for tenant model context.

### 20.3 Implementations

- **`mls-rs`** — github.com/awslabs/mls-rs (Apache 2.0). Production at AWS Wickr; selected for oyatie.
- **`openmls`** — github.com/openmls/openmls (MPL-2.0). Phoenix Initiative / TU Darmstadt; reference implementation.
- **`mlspp`** — github.com/cisco/mlspp (BSD-3-Clause). Cisco; production at Webex.
- **`libsignal`** — github.com/signalapp/libsignal (AGPL-3.0). Signal Protocol; not used by oyatie but referenced for comparison.
- **`webauthn-rs`** — github.com/kanidm/webauthn-rs (Apache-2.0/MPL-2.0). Per ADR-0188; relying-party WebAuthn implementation.
- **`hkdf`** crate (RustCrypto, Apache-2.0/MIT). Used by mls-rs.
- **`curve25519-dalek`** crate (RustCrypto / dalek-cryptography, BSD-3). X25519 + Ed25519 implementations.
- **`aws-lc-rs`** — github.com/aws/aws-lc-rs. FIPS 140-3 validated module; certificate #4759.

### 20.4 Industry deployments

- **AWS Wickr** — "How Wickr leverages MLS for secure group messaging" (AWS blog, 2023). [aws.amazon.com/blogs/security/](https://aws.amazon.com/blogs/security/).
- **Cisco Webex** — "Cisco Webex Zero-Trust Security" whitepaper (Cisco, 2024).
- **Wire** — "MLS migration complete" (Wire blog, 2024); Wire MLS spec at github.com/wireapp/.
- **Apple iMessage Contact Key Verification** — "Advancing iMessage security: iMessage Contact Key Verification" (Apple Security Engineering blog, Feb 2024).
- **Apple iMessage PQ3** — "iMessage with PQ3: The new state of the art in quantum-secure messaging at scale" (Apple Security Engineering blog, Feb 2024).
- **Google Messages RCS** — "RCS Universal Profile 3.0" hints at Google I/O 2024.
- **Meta WhatsApp** — "How WhatsApp uses Signal Protocol" (Meta Engineering blog, ongoing).
- **Element / Matrix** — "MLS in Matrix" blog (matrix.org, 2024); MSC4244.
- **Signal PQXDH** — "Quantum Resistance and the Signal Protocol" (Signal blog, Sep 2023).

### 20.5 NIST + Regulatory standards

- **NIST FIPS 203** — *Module-Lattice-Based Key-Encapsulation Mechanism Standard* (ML-KEM; August 2024).
- **NIST FIPS 204** — *Module-Lattice-Based Digital Signature Standard* (ML-DSA; August 2024).
- **NIST FIPS 205** — *Stateless Hash-Based Digital Signature Standard* (SLH-DSA; August 2024; alternate PQ signature).
- **NIST FIPS 186-5** — *Digital Signature Standard* (February 2023; admits Ed25519, EdDSA).
- **NIST SP 800-38D** — *Recommendation for Block Cipher Modes of Operation: Galois/Counter Mode (GCM) and GMAC* (2007 with 2024 update).
- **NIST SP 800-63B** — *Digital Identity Guidelines* (December 2024 revision).
- **NIST SP 800-186** — *Recommendations for Discrete Logarithm-based Cryptography* (February 2023).
- **GDPR** — Regulation (EU) 2016/679; Articles 5, 12, 17, 25, 32.
- **HIPAA Security Rule** — 45 CFR §164.312.
- **PCI DSS v4.0** — Requirement 4.
- **FedRAMP-Moderate** — NIST SP 800-53 Rev 5; SC-13, SC-28.
- **KR PIPA** — Personal Information Protection Act; Articles 24, 29, 36.
- **KR CSAP v3.1** — Cloud Security Assurance Program (MSIT).
- **SDAIA Cloud Computing Framework v1.0** (KSA).
- **GAIA-X Cryptographic Policy v2** (EU).
- **EU AI Act** (Regulation 2024/1689); Article 6 risk classification.

### 20.6 Oyatie internal references

- **`microservices/messenger/PRD.md`** — Messenger product requirements.
- **`microservices/meet/IP-012-e2e-encryption-mls.md`** — Meet E2E implementation plan (shared MLS approach).
- **`microservices/meet/IP-014-huddles-livekit-signaling.md`** — LiveKit signaling for group calls.
- **ADR-0009** — Cell architecture per-tenant per-region.
- **ADR-0028** — Audit-chain Merkle + Ed25519 (Bominal-inherited).
- **ADR-0044** — Inter-cell mesh tunnel.
- **ADR-0049** — Cross-region replication + residency.
- **ADR-0105** — 13-layer canonical enum.
- **ADR-0131** — Per-microservice flat layout.
- **ADR-0139** — Agentic SLO-gated promotion.
- **ADR-0144** — EU AI Act graduated-risk tier model.
- **ADR-0145** — Inter-microservice communication reform.
- **ADR-0148** — Inter-provider mesh tunnel security.
- **ADR-0150** — Cedar app authorisation.
- **ADR-0174** — Sustainability tag.
- **ADR-0187** — Canonical OIDC IdP Zitadel.
- **ADR-0188** — Passkey/WebAuthn Level 3 substrate.
- **ADR-0211** — In-house tech-stack policy.
- **ADR-0238** (parallel) — Dual-context isolation for Messenger.
- **ADR-0240** — Sovereign cloud per regional pack.
- **ADR-0241** — DR + business-continuity portfolio policy.
- **ADR-0242** — `oyatie`-is-a-tenant doctrine.
- **ADR-0243** — Cedar as universal gate.
- **ADR-0246** — Policy-engine substrate promotion.
- **ADR-0251** — Compliance-pack cell certification levels.
- **ADR-0252** — Time coordination + distributed consistency.
- **ADR-0253** — Network topology + edge service mesh.
- **Bominal ADR-0028** — Audit-chain inheritance.
- **Bominal ADR-0111** — Ciphertext property type + envelope encryption.
- **Bominal ADR-0208** — dual-context unified channel hub.
- **Bominal ADR-0215** — retention legal-hold dual-context.

---

## 21. Appendices

### Appendix A: Wire-format examples

#### A.1 KeyPackage (MLS-encoded; hex dump format)

```
Field                          Type           Hex / Description
-----------------------------  -------------  ----------------------------------------------------------------
ProtocolVersion                u16            0x0001 (MLS 1.0)
CipherSuite                    u16            0x0001 (DHKEMX25519_AES128GCM_SHA256_Ed25519)
HPKEPublicKey init_key         opaque<V>      32 bytes (X25519 public; e.g., 0x06fdb6da...)
LeafNode leaf_node:
  encryption_key (HPKEPublicKey) opaque<V>    32 bytes (X25519 public)
  signature_key (SignaturePublicKey) opaque<V> 32 bytes (Ed25519 public; long-term)
  Credential credential:
    credential_type            u16            0x0001 (basic)
    identity                   opaque<V>      16 bytes (Zitadel account UUID)
  Capabilities capabilities:
    versions                   u16-array      [0x0001]
    cipher_suites              u16-array      [0x0001, 0x0003]  // default + ChaCha fallback
    extensions                 u16-array      [0xCA01..0xCA04]   // oyatie ext IDs
    proposals                  u16-array      [...]
    credentials                u16-array      [0x0001]            // basic only
  LeafNodeSource source        u8             0x01 (key_package)
  Extension extensions<V>:
    oyatie_device_metadata (ext_type 0xCA01):
      platform: "ios"
      os_version: "17.5"
      app_version: "1.0.0"
      device_model: "iPhone15,3"
    oyatie_passkey_attestation (ext_type 0xCA02):
      passkey_assertion: <CBOR-encoded WebAuthn assertion>
      attestation_payload: <signed payload>
    oyatie_tenant_scope (ext_type 0xCA03):
      tenant_id: "personal-account-xyz"
    oyatie_compliance_pack (ext_type 0xCA04):
      pack: "kr"
  signature                    opaque<V>      64 bytes (Ed25519 sig over LeafNodeTBS)
Extension extensions<V>        ...            (KeyPackage-level extensions; usually empty)
signature                      opaque<V>      64 bytes (Ed25519 sig over KeyPackageTBS by leaf signature_key)
```

Typical KeyPackage size: ~400-600 bytes.

#### A.2 Welcome message structure

```
struct Welcome {
    CipherSuite cipher_suite;
    EncryptedGroupSecrets secrets<V>;       // one per new member
    opaque encrypted_group_info<V>;          // encrypted with group info key
}

struct EncryptedGroupSecrets {
    KeyPackageRef new_member;
    HPKECiphertext encrypted_group_secrets; // contains init_secret + path_secret
}

struct GroupInfo {
    GroupContext group_context;              // group_id, epoch, tree_hash, confirmed_transcript_hash
    Extension extensions<V>;
    MAC confirmation_tag;
    uint32 signer;
    opaque signature<V>;
}
```

Typical Welcome size for a 2-member 1:1 group: ~1 KB.

#### A.3 Commit message structure

```
struct PublicMessage {
    FramedContent content;                    // CommitContent
    FramedContentAuthData auth;              // signature, optional confirmation_tag
    optional<MAC> membership_tag;            // for PublicMessage in epoch
}

struct FramedContent {
    opaque group_id<V>;
    uint64 epoch;
    Sender sender;                            // by leaf_index
    opaque authenticated_data<V>;
    ContentType content_type;                 // 0x03 = commit
    select (content_type) {
        case commit: Commit commit;
    };
}

struct Commit {
    ProposalOrRef proposals<V>;               // add / remove / update / etc.
    optional<UpdatePath> path;                // tree-key-update path
}
```

Typical Commit size at group size 10: ~3-5 KB; at group size 100:
~5-15 KB.

### Appendix B: Sample `mls-rs` group lifecycle code

```rust
//! microservices/messenger/src/e2e_mls_lifecycle.rs (illustrative)

use mls_rs::{
    Client, ClientBuilder, GroupConfig, MlsGroup, ProtocolVersion,
    crypto::CipherSuite,
    identity::{BasicCredential, SigningIdentity},
    storage::{InMemoryGroupStateStorage, InMemoryKeyPackageStorage},
};
use mls_rs_crypto_rustcrypto::RustCryptoProvider;

const SUITE: CipherSuite = CipherSuite::CURVE25519_AES128;

pub async fn create_client(
    account_id: uuid::Uuid,
    long_term_signing_key: ed25519_dalek::SigningKey,
) -> anyhow::Result<Client<...>> {
    let crypto = RustCryptoProvider::default();
    let basic_cred = BasicCredential::new(account_id.as_bytes().to_vec());

    let signing_id = SigningIdentity::new(
        basic_cred.into_credential(),
        long_term_signing_key.public_key_bytes().to_vec(),
    );

    let client = ClientBuilder::new()
        .crypto_provider(crypto)
        .identity_provider(BasicIdentityProvider::new())
        .signing_identity(signing_id, long_term_signing_key, SUITE)
        .group_state_storage(InMemoryGroupStateStorage::new())  // production: PostgresGroupStorage
        .key_package_repo(InMemoryKeyPackageStorage::new())     // production: PostgresKpRepo
        .build();
    Ok(client)
}

pub async fn create_group(
    client: &Client<...>,
    group_id: Vec<u8>,
) -> anyhow::Result<MlsGroup<...>> {
    let mut group = client.create_group(
        Some(group_id),
        Default::default(),
        Default::default(),
    )?;
    Ok(group)
}

pub async fn add_member(
    client: &Client<...>,
    group: &mut MlsGroup<...>,
    new_member_kp_bytes: Vec<u8>,
) -> anyhow::Result<(CommitOutput, Welcome)> {
    let kp = client.cipher_suite_provider(SUITE)?
        .key_package_from_bytes(&new_member_kp_bytes)?;
    let mut commit = group.commit_builder();
    commit.add_member(kp)?;
    let output = commit.build()?;
    let welcome = output.welcome_messages
        .first()
        .ok_or_else(|| anyhow::anyhow!("expected at least one welcome"))?
        .clone();
    group.apply_pending_commit()?;
    Ok((output, welcome))
}

pub async fn send_application(
    group: &mut MlsGroup<...>,
    plaintext: &[u8],
) -> anyhow::Result<Vec<u8>> {
    let app_msg = group.encrypt_application_message(plaintext, vec![])?;
    Ok(app_msg.to_bytes()?)
}

pub async fn receive_application(
    group: &mut MlsGroup<...>,
    ciphertext: &[u8],
) -> anyhow::Result<Vec<u8>> {
    let received = group.process_incoming_message(MlsMessage::from_bytes(ciphertext)?)?;
    match received {
        ReceivedMessage::ApplicationMessage(app) => Ok(app.data().to_vec()),
        other => anyhow::bail!("unexpected message type: {:?}", other),
    }
}

pub async fn remove_member(
    client: &Client<...>,
    group: &mut MlsGroup<...>,
    leaf_index_to_remove: u32,
) -> anyhow::Result<CommitOutput> {
    let mut commit = group.commit_builder();
    commit.remove_member(LeafIndex(leaf_index_to_remove))?;
    let output = commit.build()?;
    group.apply_pending_commit()?;
    Ok(output)
}

pub async fn update_own_leaf(
    group: &mut MlsGroup<...>,
) -> anyhow::Result<CommitOutput> {
    let mut commit = group.commit_builder();
    let output = commit.build()?;
    group.apply_pending_commit()?;
    Ok(output)
}
```

The full implementation lives in `microservices/messenger/src/`; this
sketch shows the major call patterns.

### Appendix C: Open questions

| # | Question | Owner | Resolution target |
|---|---|---|---|
| Q1 | Sealed Sender for personal context (Signal-style sender obfuscation) | council-security | post-v1 ADR |
| Q2 | Per-message cover-traffic padding to a fixed size band | council-security | post-v1 ADR |
| Q3 | Federation with non-Matrix targets (e.g., Wire, Element-Web direct) | axis-messenger + council-privacy | per-tenant opt-in ADR |
| Q4 | E2E for cross-tenant messages (where Bob is on tenant-A and Alice is on oyatie tenant) | council-architecture | requires federation finalisation |
| Q5 | Sealed-server-pattern (server doesn't learn user identifiers in MLS routing) | council-security | post-PQ-migration consideration |
| Q6 | Per-pack PQ rollout schedule deviations (some packs may go PQ earlier or later) | council-privacy + ops-compliance | per-pack ADR |
| Q7 | Hardware-attestation-only mode (refuse non-MDS3-attested authenticators for high-security tenants) | ops-security | pack-specific ADR |
| Q8 | Multi-account on one device (Signal-style profile switching) | UX team + axis-messenger | UX research deliverable |
| Q9 | Conversation export (user-initiated export of plaintext history into a portable archive) | council-privacy | GDPR portability response |
| Q10 | Per-message disappearing-messages (Signal-style auto-delete after time) | UX team | v1.1 feature |

---

*End of standard. Implementation tracked in subsequent IPs under
`microservices/messenger/`.*
