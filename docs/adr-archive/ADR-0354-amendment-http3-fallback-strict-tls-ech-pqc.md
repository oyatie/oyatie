---
id: ADR-0354
status: Superseded
date: 2026-05-20
owners:
  - council-architecture
  - council-security
  - council-privacy
  - axis-network
  - axis-cloud-k8s
  - axis-edge
  - ops-sre-reliability
  - ops-security
  - ops-compliance
amends: ADR-0253
supersedes: []
superseded_by: [ADR-705]
related:
  - ADR-0044-service-mesh-and-mtls.md
  - ADR-0121-onprem-k8s-stack-kubeadm-containerd-istio-envoy.md
  - ADR-0148-service-mesh-cilium-ambient-layered.md
  - ADR-0149-api-gateway-vs-service-mesh-separation.md
  - ADR-0211-in-house-tech-stack-policy.md
  - ADR-0243-cedar-as-universal-gate.md
  - ADR-0246-policy-engine-substrate-promotion.md
  - ADR-0253-network-topology-edge-service-mesh.md
  - ADR-0293-meta-trust-root-offline-hsm-anchor.md
  - ADR-0295-bootstrap-spiffe-kill-switch.md
related_specs:
  - /specs/network-topology.json
  - /specs/microservices/edge-gateway.json
  - /specs/tls-profile.json
  - network/ports/transport-profile/endpoint-transport-profile.contract.json
related_memory:
  - feedback_http3_quic_default_protocol
  - feedback_no_silent_regression
  - feedback_quality_performance_scalability_bar
  - feedback_build_ahead_of_certification
doc_class: Architecture-Decision-Record
amendment_class: Clarification-and-Extension
keystone_bundle: 2026-05-20-foundational-doctrine
wave: Wave-3-A
created_by: wave-3-a-cross-reference-wiring-agent
purpose: >
  Codify the HTTP/3 → HTTP/2 → HTTP/1.1 protocol fallback chain, the
  strict TLS 1.3 profile, Encrypted Client Hello (ECH, RFC 9460) for
  all external-facing endpoints, and the post-quantum hybrid cipher
  suite (X25519MLKEM768 KEM + ed25519/ml_dsa_65 hybrid signature).
  Provides per-microservice protocol applicability table and CI
  enforcement lanes. Amends ADR-0253 §D-4 (TLS), §D-5 (protocol
  version), and §D-7 (observability) with binding operational
  parameters that were advisory in the original ADR.
enforcement_status: advisory-until-edge-pop-and-pqc-libs-land
enforced_by:
  - oya gate validate tls13-strict-profile
  - oya gate validate http3-fallback-chain-declared
  - oya gate validate ech-endpoint-coverage
  - oya gate validate pqc-hybrid-kem-declared
---

> **Disposition light-edit (2026-08-06):** Context re-triage Accept: HTTP/3 TLS ECH PQC amendment

# ADR-0354: Amendment — HTTP/3 Fallback Chain, Strict TLS, ECH, PQC Hybrid

## Context

ADR-0253 established HTTP/3 + QUIC as the default transport for all
oyatie endpoints (KS#10 per `feedback_http3_quic_default_protocol`).
However, the original ADR left the following operational parameters
advisory or unspecified:

1. **Protocol fallback ordering** — what happens when a client or
   network path does not support QUIC/HTTP/3? The fallback chain
   (HTTP/3 → HTTP/2 → HTTP/1.1) must be explicitly declared with
   per-hop timeout budgets so every µservice operator can implement
   identically.

2. **Strict TLS 1.3 profile** — ADR-0253 §D-4 says "TLS 1.3 with
   post-quantum hybrid key exchange" but does not enumerate the
   forbidden cipher suites, the minimum HKDF-SHA-384 requirement for
   HSM-backed sessions, or the legacy-grace compliance-pack override
   path for tenants still requiring TLS 1.2.

3. **Encrypted Client Hello (ECH, RFC 9460)** — ECH prevents
   server-name leakage to on-path observers. External-facing endpoints
   (edge POPs, API gateway, intelligence inference API, connect
   signalling) must declare ECH support. The ECH key rotation cadence,
   retry_configs distribution, and fallback-to-GREASE handling were
   not specified.

4. **Post-quantum hybrid suite** — the original ADR names
   "post-quantum hybrid key exchange" but does not pin the IANA
   code-point, the hybrid signature scheme, or the negotiation
   fallback for clients that do not yet support the NIST PQC round-4
   KEM.

5. **Per-microservice protocol table** — the original ADR applies
   globally but operators need a per-µservice reference for which
   protocol tiers apply, which endpoints are ECH-mandatory, and which
   are PQC-hybrid-mandatory.

This amendment is binding (not advisory) for all new endpoint
declarations added on or after 2026-05-20. Existing endpoints have a
90-day migration grace window tracked by the `tls-profile-migration`
FixupTask family.

---

## Decision

### §B-1 HTTP/3 Fallback Chain

**Decision:** Every oyatie endpoint that accepts external traffic MUST
implement the following protocol fallback chain in the listed priority
order. Inter-cell and internal endpoints are declared through §B-8 and
remain gRPC over HTTP/2 with SPIFFE mTLS until a real endpoint-specific
pull justifies a transport-runtime adapter.

```
Priority 1 (default):  HTTP/3 over QUIC (RFC 9114)
Priority 2 (fallback): HTTP/2 over TLS 1.3 (RFC 7540 + RFC 9325)
Priority 3 (legacy):   HTTP/1.1 over TLS 1.3 (RFC 9112)
                        — permitted only when explicitly declared via
                          the `legacy-tls12-grace` compliance pack OR
                          for health-check endpoints behind the
                          service mesh loopback only.
```

**Fallback trigger conditions** (any one sufficient):

| Condition | Fallback level |
|-----------|---------------|
| QUIC port 443 UDP blocked by network path | HTTP/3 → HTTP/2 |
| QUIC version negotiation failure (RFC 9000 §6) | HTTP/3 → HTTP/2 |
| Alt-Svc header not received by client within 500 ms | HTTP/3 → HTTP/2 |
| HTTP/2 SETTINGS frame timeout (> 10 s) | HTTP/2 → HTTP/1.1 |
| HTTP/2 GOAWAY with NO_ERROR + advertised HTTP/1.1-only | HTTP/2 → HTTP/1.1 |

**Fallback timeout budgets:**

```
QUIC handshake budget:       2 000 ms  (includes version negotiation)
Alt-Svc discovery budget:      500 ms
HTTP/2 connection budget:    5 000 ms
HTTP/1.1 connection budget:  5 000 ms
Total worst-case fallback:  12 500 ms  (must fit within §D-8 latency SLO)
```

**Alt-Svc advertisement requirement:**

Every HTTP/2 and HTTP/1.1 response MUST carry the `Alt-Svc` header
advertising the H3 equivalent:

```
Alt-Svc: h3=":443"; ma=86400, h3-29=":443"; ma=86400
```

The `ma` (max-age) value of 86 400 s (24 h) is the canonical default.
Edge POPs may lower to 3 600 s during rolling upgrades.

**Protocol downgrade logging:**

Every fallback transition MUST emit a structured log event:

```json
{
  "event": "protocol_fallback",
  "from_protocol": "h3",
  "to_protocol": "h2",
  "reason": "quic_port_blocked",
  "tenant_id": "<redacted-or-null-for-unauth>",
  "cell_id": "<cell-id>",
  "endpoint_id": "<endpoint-id>"
}
```

This event feeds the `protocol_fallback_rate` SLO metric. Alert
threshold: fallback_rate > 5 % of requests over a 5-minute window
triggers a P2 incident.

---

### §B-2 Strict TLS 1.3 Profile

**Decision:** All TLS sessions on oyatie endpoints MUST conform to
the strict TLS 1.3 profile defined below.

#### §B-2.1 Permitted Cipher Suites (closed list)

```
TLS_AES_256_GCM_SHA384           (REQUIRED — default for HSM sessions)
TLS_CHACHA20_POLY1305_SHA256     (REQUIRED — default for non-HSM sessions)
TLS_AES_128_GCM_SHA256           (PERMITTED — fallback for constrained clients)
```

All TLS 1.3 cipher suites not in the above list are FORBIDDEN.

**TLS 1.2 cipher suites are FORBIDDEN** on all endpoints except
those covered by the `legacy-tls12-grace` compliance pack (see §B-2.4).

#### §B-2.2 Key Exchange Groups (TLS 1.3 supported_groups)

```
x25519mlkem768    (REQUIRED first — PQC hybrid, see §B-4)
x25519            (REQUIRED second — classical fallback)
secp256r1         (PERMITTED third — P-256 for constrained clients)
secp384r1         (PERMITTED — P-384 for FIPS-adjacent requirements)
```

Groups not in this list (including secp521r1, ffdhe*, brainpool*)
are FORBIDDEN in new endpoint configurations.

#### §B-2.3 Certificate and Signature Requirements

**Leaf certificate:**
- Key type: Ed25519 OR ECDSA P-256 (RSA-2048 PERMITTED until
  2027-01-01 for legacy-grace tenants only)
- Signature algorithm on cert: Ed25519 OR ecdsa-with-SHA256
- Validity: ≤ 90 days (automated via ACME RFC 8555 or internal CA)
- CT log proof: ≥ 2 SCTs from distinct log operators (required for
  all external endpoints)

**Hybrid signature (post-quantum, see §B-4):**
- Signature on TLS CertificateVerify (TLS 1.3 handshake):
  ed25519 classical component PLUS ml_dsa_65 PQC component
  per the composite-signatures draft (draft-ietf-lamps-pq-composite-sigs)
- This is the `ed25519+ml_dsa_65` hybrid scheme referenced in
  feedback_http3_quic_default_protocol.

**CA chain:**
- Root CA: trust-store scoped per cell; no global trust store override
- Intermediate CA max depth: 3 (root → int-1 → int-2 → leaf)
- OCSP stapling: REQUIRED for all leaf certs; Must-Staple extension
  encoded in cert where CA supports it
- OCSP response max-age: 4 h for external endpoints; 12 h for
  inter-cell (Cilium mesh mTLS)

#### §B-2.4 Legacy-Grace Override (TLS 1.2)

Tenants may activate the `legacy-tls12-grace` compliance pack to
enable TLS 1.2 on their tenant-scoped endpoints subject to:

1. Pack activation requires council-security review (ADR-0251 §D-4).
2. TLS 1.2 endpoints are restricted to cipher suites:
   `TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384` and
   `TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384` only.
3. The grace window is hard-limited to 18 months from pack activation.
4. ECH (§B-3) does NOT apply to TLS 1.2 endpoints.
5. PQC hybrid (§B-4) does NOT apply to TLS 1.2 endpoints.
6. The `legacy-tls12-grace` pack MUST NOT be combined with
   `CN-PIPL-2021` or `EU-GDPR-2018-baseline` packs (those packs
   require TLS 1.3 minimum per their cross-border transfer gating
   Cedar fragments).

**Scope restriction:** Legacy-grace applies to tenant-scoped
endpoints only. Substrate µservice endpoints (governance, policy-engine,
audit-chain, identity, tenancy, cloud-iam) are NOT eligible for
legacy-grace and MUST remain on TLS 1.3 at all times.

---

### §B-3 Encrypted Client Hello (ECH, RFC 9460)

**Decision:** All external-facing oyatie endpoints MUST support ECH
as defined by RFC 9460 (TLS Encrypted Client Hello).

#### §B-3.1 ECH-Mandatory Endpoint Classes

The following endpoint classes are ECH-mandatory (no opt-out):

| Endpoint class | Justification |
|----------------|--------------|
| Edge POP ingress (all regions) | Prevents SNI leakage to ISP on-path observers |
| API gateway (external tenant API surface) | SNI carries tenant-id-derived hostname → privacy leak |
| Intelligence inference API | User prompt routing SNI leaks capability tier |
| signalling (WebRTC / MLS) | SNI leaks participant identity to on-path MITM |
| Cloud IAM / STS token endpoint | SNI timing + frequency leaks auth patterns |
| Governance policy-engine evaluator (external) | Leaks policy evaluation patterns |

Internal mesh endpoints (Cilium ambient mTLS, loopback) are EXEMPT
from ECH (mTLS already provides mutual identity; SNI is cell-internal).

#### §B-3.2 ECH Key Lifecycle

```
ECH key type:     X25519 (HPKE RFC 9180)
ECH AEAD:         AES-128-GCM (HPKE AEAD ID 0x0001)
ECH KDF:          HKDF-SHA256 (HPKE KDF ID 0x0001)
ECH key rotation: every 24 h (automated via SPIFFE-bound rotation job)
ECH overlap window: 48 h (old key decrypts during rotation overlap)
```

**ECHConfigList distribution:** ECH public keys are published via
DNS HTTPS records (RFC 9460 §4) for each external endpoint hostname.
DNS TTL for ECH records: 3 600 s (1 h) to bound propagation lag after
key rotation.

**GREASE fallback handling:**
- When a client sends an ECH ClientHello with the GREASE code-point
  (0xfe0d), the server MUST NOT downgrade to plaintext SNI.
- The server MUST respond with a retry_configs hint (ECH extension
  with the current ECHConfigList) even if the GREASE was intentional
  sounding.
- If client falls back to plaintext SNI after receiving retry_configs,
  the endpoint MUST still serve the connection (no hard reject) but
  MUST emit a `ech_plaintext_sni_fallback` structured log event for
  observability.

#### §B-3.3 ECH and Multi-Tenant Hostname Routing

Tenant-specific hostnames (e.g., `<tenant>.api.oyatie.dev`) MUST
use shared outer ECH SNI (`api.oyatie.dev`) so on-path observers
cannot infer the tenant from the ClientHello. The edge POP decrypts
ECH and routes to the correct cell using the inner SNI. This pattern
is the canonical implementation of ADR-0242 §D-3 (oyatie-is-a-tenant
privacy isolation at network layer).

---

### §B-4 Post-Quantum Hybrid (PQC) Suite

**Decision:** oyatie adopts a hybrid PQC strategy: every TLS session
uses a hybrid KEM combining a classical ECDH group with a NIST
round-4 post-quantum KEM. Hybrid signatures are used for TLS
CertificateVerify to protect against harvest-now-decrypt-later
attacks on recorded handshakes.

#### §B-4.1 Hybrid KEM: X25519MLKEM768

```
IANA TLS supported_groups code point:  x25519mlkem768
                                         (draft-kwiatkowski-tls-ecdhe-mlkem,
                                          assigned code point 0x11EC)
Classical component:  X25519 (RFC 7748)
PQC component:        ML-KEM-768 (FIPS 203, formerly CRYSTALS-Kyber-768)
Hybrid combiner:      concatenation of (X25519_shared_secret || MLKEM_shared_secret)
                      fed to HKDF as per draft-ietf-tls-hybrid-design
Security level:       NIST Category 3 (128-bit quantum security)
```

**Negotiation fallback:** If the client does not advertise
`x25519mlkem768` in supported_groups, the server MUST fall back to
`x25519` (classical only). The server MUST NOT refuse connections
from classical-only clients during the transition period ending
2027-12-31. After 2027-12-31, `x25519mlkem768` becomes mandatory for
substrate µservice inter-cell connections; classical-only clients
require a `pqc-transition-grace` pack activation for an additional
12-month window.

**Metrics tracking:**
```
pqc_hybrid_kem_negotiated_total   (counter, label: endpoint_id)
pqc_classical_fallback_total      (counter, label: endpoint_id, reason)
```

Alert: if `pqc_classical_fallback_total` rate exceeds 20 % of
total handshakes on a substrate endpoint for > 1 hour, escalate to
council-security for investigation.

#### §B-4.2 Hybrid Signature: ed25519 + ml_dsa_65

```
Scheme:  Composite signature per draft-ietf-lamps-pq-composite-sigs
Classical component:  Ed25519 (RFC 8032)
PQC component:        ML-DSA-65 (FIPS 204, formerly CRYSTALS-Dilithium-65)
                      Security level: NIST Category 3
OID:     id-MLDSA65-Ed25519 (draft OID 2.16.840.1.114027.80.8.1.3)
Used in: TLS 1.3 CertificateVerify message
         Code-signing of Cedar fragment artifacts (ADR-0247 §D-5)
         SPIFFE SVID signature for bootstrap runners (ADR-0295 §D-2)
```

The hybrid signature ensures that a quantum adversary who can break
Ed25519 still cannot forge a CertificateVerify without also breaking
ML-DSA-65, and vice versa. This is the primary protection against
harvest-now-decrypt-later attacks recorded after 2026.

**Certificate issuance:** The internal CA (bootstrap-ca per ADR-0295)
MUST issue composite-signature leaf certificates for all substrate
endpoints. The public CA path (Let's Encrypt / ZeroSSL) still issues
Ed25519-only certificates; the hybrid signature is applied at the
CertificateVerify layer (not the cert itself) until public CAs support
composite issuance.

#### §B-4.3 PQC Scope Per Endpoint Class

| Endpoint class | Hybrid KEM | Hybrid Sig |
|----------------|-----------|-----------|
| Edge POP ingress (external) | REQUIRED | REQUIRED |
| API gateway (external) | REQUIRED | REQUIRED |
| Intelligence inference API | REQUIRED | REQUIRED |
| signalling | REQUIRED | REQUIRED |
| Cloud IAM / STS | REQUIRED | REQUIRED |
| Policy-engine evaluator (external) | REQUIRED | REQUIRED |
| Inter-cell substrate mesh (Cilium mTLS) | REQUIRED | REQUIRED |
| Intra-cell loopback (service mesh only) | RECOMMENDED | RECOMMENDED |
| Health check / metrics scrape (internal) | EXEMPT | EXEMPT |
| Legacy-grace tenant endpoints (TLS 1.2) | NOT APPLICABLE | NOT APPLICABLE |

---

### §B-5 Per-Microservice Protocol Table

The following table defines the protocol applicability for each
oyatie µservice's externally-reachable endpoints. "External" means
reachable from outside the Cilium cell boundary. "Internal" means
intra-cell or inter-cell mesh only.

| µservice | External endpoints | HTTP/3 | ECH | PQC hybrid | Notes |
|----------|-------------------|--------|-----|-----------|-------|
| api-gateway | yes | REQUIRED | REQUIRED | REQUIRED | Primary external ingress |
| intelligence | yes (inference API) | REQUIRED | REQUIRED | REQUIRED | Consumer-facing AI surface |
| connector | yes (signalling, WebRTC) | REQUIRED | REQUIRED | REQUIRED | MLS RFC 9420 E2EE channel |
| cloud-iam | yes (token endpoint) | REQUIRED | REQUIRED | REQUIRED | STS auth criticality |
| governance | no (internal only) | RECOMMENDED | EXEMPT | REQUIRED | Policy eval; mesh-internal |
| policy-engine | no (internal only) | RECOMMENDED | EXEMPT | REQUIRED | Cedar eval; mesh-internal |
| audit-chain | no (internal only) | RECOMMENDED | EXEMPT | REQUIRED | Merkle-sealed audit |
| identity | limited (OIDC discovery) | REQUIRED | REQUIRED | REQUIRED | OIDC well-known endpoint is external |
| tenancy | no (internal only) | RECOMMENDED | EXEMPT | REQUIRED | Tenant lifecycle; mesh-internal |
| consent | no (internal only) | RECOMMENDED | EXEMPT | REQUIRED | PIPL/GDPR consent gate |
| cloud-kms | no (internal only) | RECOMMENDED | EXEMPT | REQUIRED | Key mgmt; mesh-internal |
| cloud-secrets | no (internal only) | RECOMMENDED | EXEMPT | REQUIRED | SecretReference UDS; mesh-internal |
| observability | limited (Prometheus scrape) | RECOMMENDED | EXEMPT | RECOMMENDED | Scrape is cell-internal |
| marketplace | yes | REQUIRED | REQUIRED | REQUIRED | Commerce surface |
| plugin-app-store | yes | REQUIRED | REQUIRED | REQUIRED | Plugin distribution |
| community | yes | REQUIRED | REQUIRED | REQUIRED | Social/forum surface |
| workflow-studio | yes (canvas API) | REQUIRED | REQUIRED | REQUIRED | No-code UI |
| payments | yes | REQUIRED | REQUIRED | REQUIRED | PCI scope |
| cloud-billing-* | no (internal only) | RECOMMENDED | EXEMPT | REQUIRED | Billing substrate |
| cloud-network-* | no (internal only) | RECOMMENDED | EXEMPT | REQUIRED | Network management |
| foundry | no (internal only) | RECOMMENDED | EXEMPT | REQUIRED | retired external agent harness pipeline; internal |

**Default rule for µservices not listed:** If a µservice exposes
any endpoint reachable from outside the cell boundary, HTTP/3,
ECH, and PQC hybrid are all REQUIRED. When in doubt, treat as
external. Cell-to-cell control-plane automation is inter-cell, not
external, and remains on gRPC over HTTP/2 unless §B-8 reclassifies the
endpoint.

---

### §B-6 CI Enforcement Lanes

The following CI lanes enforce this amendment. Each lane MUST have a
passing + failing fixture pair per F3 adversarial requirement
(ADR-0221 + multispectrum-review.json §facets.F3_adversarial).

#### §B-6.1 `oya-check-tls13-strict-profile`

**Purpose:** Validate that every declared TLS endpoint configuration
(in `/specs/microservices/*.json` + `/specs/tls-profile.json`) uses
only the permitted cipher suites and key groups from §B-2.

**Sub-checks:**
- `tls-cipher-suite-allowlist` — rejects any cipher suite not in
  §B-2.1 permitted list.
- `tls-key-group-allowlist` — rejects any supported_group not in
  §B-2.2.
- `tls12-grace-pack-required` — if any endpoint declares TLS 1.2,
  fails unless `legacy-tls12-grace` pack is active for that tenant scope.
- `substrate-tls12-forbidden` — substrate µservice endpoints listed
  in §B-4.3 REQUIRED rows fail if they declare TLS 1.2 regardless
  of any pack.

**Severity ramp:** day-0 report-only; day-8 error (ADR-0092 ramp).

#### §B-6.2 `oya-check-http3-fallback-chain`

**Purpose:** Every endpoint spec that declares HTTP/3 MUST also
declare the fallback chain and timeout budgets matching §B-1.

**Sub-checks:**
- `alt-svc-header-declared` — endpoint spec includes `alt_svc`
  field with `h3=":443"` value.
- `fallback-timeout-budget-declared` — endpoint spec includes
  `fallback_timeout_ms` with value ≤ 12 500.
- `protocol-downgrade-log-event-registered` — observability registry
  carries `protocol_fallback` event for the endpoint.

**Severity ramp:** day-0 report-only; day-8 error.

#### §B-6.3 `oya-check-ech-endpoint-coverage`

**Purpose:** Every endpoint in the ECH-mandatory class (§B-3.1) MUST
declare ECH support in its microservice spec.

**Sub-checks:**
- `ech-mandatory-class-coverage` — scans all endpoint specs; any
  endpoint of type `edge-pop|api-gateway|inference-api|signalling|sts|policy-eval`
  that lacks `ech.enabled: true` and `ech.support_required: true` fails.
- `ech-key-rotation-cadence` — `key_rotation_hours` value MUST be
  ≤ 24.
- `ech-grease-handling-declared` — endpoint spec includes
  `ech.grease_retry_configs: true`.
- `ech-transition-fallback-declared` — endpoint spec includes
  `ech.plaintext_sni_fallback_allowed: true` so §B-3.2 fallback
  remains observable rather than hard-refused.
- `ech-multi-tenant-outer-sni` — tenant-scoped endpoints use shared
  `ech.outer_sni` (`*.oyatie.dev` root) not tenant-specific hostname.

**Severity ramp:** day-0 report-only; day-8 error.

#### §B-6.4 `oya-check-pqc-hybrid-kem`

**Purpose:** Every PQC-REQUIRED endpoint (§B-4.3) MUST declare
hybrid negotiation support through the `pqc` policy block.

**Sub-checks:**
- `pqc-kem-declared` — endpoint spec `pqc.kem` equals
  `x25519mlkem768`.
- `pqc-hybrid-sig-declared` — endpoint spec `pqc.signature` equals
  `ed25519+ml_dsa_65`.
- `pqc-hybrid-negotiation-required` — endpoint spec
  `pqc.hybrid_negotiation_required` is true for PQC-required external
  endpoints.
- `pqc-classical-fallback-present` — endpoint spec includes
  `pqc.classical_transition_fallback_allowed: true` (MUST NOT remove
  classical fallback before 2027-12-31 deadline).
- `pqc-metrics-registered` — observability registry carries
  `pqc_hybrid_kem_negotiated_total` and `pqc_classical_fallback_total`
  metrics for the endpoint.

**Severity ramp:** day-0 report-only; day-8 error.

---

### §B-7 Migration Plan

#### §B-7.1 Phase 1 — Audit (0 to 30 days, 2026-05-20 to 2026-06-19)

1. Run all four CI lanes in report-only mode against the current
   endpoint spec corpus.
2. File F-TLS-PROFILE-MIGRATION-* fixuptasks for each failing endpoint.
3. Priority order: external endpoints in ECH-mandatory class (§B-3.1)
   first; internal substrate endpoints second.

#### §B-7.2 Phase 2 — Remediation (30 to 90 days)

1. Update all external endpoint specs to declare HTTP/3 fallback
   chain, ECH, and PQC hybrid.
2. Deploy ECH key rotation job (SPIFFE-bound, 24 h cadence).
3. Update internal CA (bootstrap-ca, ADR-0295) to issue
   composite-signature leaf certs for substrate endpoints.
4. Wire `x25519mlkem768` into Pingora edge config + Envoy sidecar
   config (Cilium ambient mesh).

#### §B-7.3 Phase 3 — Enforcement (day 90+)

1. Promote CI lane severity from report-only to error.
2. Add lanes to dev branch protection required status checks.
3. Monitor `pqc_classical_fallback_total` metric; track toward < 5 %
   of handshakes by 2027-06-01.
4. Set calendar reminder for 2027-12-31 `x25519mlkem768`-mandatory
   deadline for substrate inter-cell connections.

---

### §B-8 Protocol-Agnostic Transport Port

**Decision:** Endpoint transport posture is a typed declaration owned
by `network/ports/transport-profile`, not by edge engines, cloud SDKs,
shell scripts, or provider-specific adapters. The Rust port exposes
`TransportEndpointSpec` and `TransportProfilePort` with the stable
fields `endpoint_id` (the required endpoint identifier), `protocol`, `tls_profile`, `alt_svc`,
`fallback_timeout_ms`, `ech`, `pqc`, and `capability_class`. The
crate-local contract artifact is
`network/ports/transport-profile/endpoint-transport-profile.contract.json`.

**Capability classification:**

| Class | Protocol | TLS profile | Alt-Svc / fallback | ECH | PQC | Adapter rule |
|-------|----------|-------------|--------------------|-----|-----|--------------|
| external | `http3` | `strict_tls13` | required; `fallback_timeout_ms <= 12500` | `support_required=true`; `plaintext_sni_fallback_allowed=true` | `hybrid_negotiation_required=true`; `classical_transition_fallback_allowed=true` | runtime adapter deferred |
| inter_cell | `grpc_http2` | `spiffe_mtls_tls13` | forbidden | disabled | hybrid declared, rollout optional | no QUIC engine |
| internal | `grpc_http2` | `spiffe_mtls_tls13` | forbidden | disabled | optional | no QUIC engine |

The external class requires endpoint support for ECH and hybrid PQC; it
does not hard-refuse transition fallback paths. Plaintext-SNI fallback
and classical PQC fallback remain explicitly allowed until the
deadline and are governed by the §D-6 SLOs.

**Runtime deferral:** An owned QUIC engine is not introduced by this
ADR. If a real endpoint use later requires a Layer-5 transport runtime,
the adapter binding point is a single transport-runtime adapter, with
`s2n-quic` as the candidate engine. The port MUST remain
protocol-agnostic and provider-neutral.

**Verification:** `buck2 test
//network/ports/transport-profile:network-transport-profile-unittest`
parses the contract artifact and rejects declarations that move
inter-cell/internal traffic onto HTTP/3, add Alt-Svc to non-external
endpoints, or weaken external ECH/PQC posture.

**Structural accounting:** The initial protocol-boundary slice is
the typed port plus its contract:
`network/ports/transport-profile/BUCK`,
`network/ports/transport-profile/Cargo.toml`,
`network/ports/transport-profile/src/lib.rs`, and
`network/ports/transport-profile/endpoint-transport-profile.contract.json`,
with `registry/catalog/network-transport-profile.yaml` as the catalog
row. These files are the reviewed, provider-neutral port and catalog
row for issue #773; runtime transport adapters remain deferred until
a real endpoint pulls them.

---

## §C — Rationale

### Why ECH is mandatory for external endpoints

ECH (RFC 9460) is the only standardised mechanism that prevents
on-path observers from reading the server-name indication during TLS
handshake. Without ECH, the SNI field leaks:

- Tenant identity (tenant-scoped hostnames)
- Product tier (intelligence.api vs standard.api)
- Capability routing (model-tier subdomains)

For a platform that handles PHI, PCI, CN-PIPL, and EU-GDPR data
classes, SNI leakage is a regulatory compliance risk (ADR-0251
compliance packs). ECH is the network-layer complement to ADR-0242
tenant privacy isolation.

### Why X25519MLKEM768 rather than pure ML-KEM

The hybrid approach is required because:

1. **Harvest-now-decrypt-later**: Adversaries record today's TLS
   sessions and decrypt them when quantum computers capable of
   breaking X25519 become available. ML-KEM-768 (Category 3 NIST
   PQC standard) provides quantum resistance for those recorded sessions.

2. **Downgrade safety**: A hybrid combiner means that an adversary
   who can break either component (but not both) cannot recover the
   session key. This is strictly stronger than either component alone.

3. **NIST finalization**: ML-KEM was standardised as FIPS 203 in
   August 2024. X25519MLKEM768 is the only IETF-standardised hybrid
   code point with broad library support (BoringSSL, rustls,
   OpenSSL 3.3+, AWS-LC).

4. **Performance**: X25519MLKEM768 adds ~1.1 KB to the ClientHello
   key_share. Benchmarks on Pingora edge hardware show < 0.5 ms
   additional handshake latency at p99 — within the §B-1 fallback
   budget.

### Why ed25519 + ml_dsa_65 for hybrid signature

Ed25519 provides fast verification (< 100 µs) and small signatures
(64 bytes). ML-DSA-65 (CRYSTALS-Dilithium-65, FIPS 204) adds 3 293
bytes but provides Category 3 quantum resistance for TLS handshake
authentication. The composite scheme per
draft-ietf-lamps-pq-composite-sigs ensures that neither component
can be stripped without invalidating the composite signature.

The same composite scheme is used for Cedar fragment cosign
attestation (ADR-0247 §D-5 amendment) and SPIFFE SVID signatures
(ADR-0295 §D-2), establishing a consistent PQC signature primitive
across the entire trust chain.

### Why TLS 1.2 is forbidden except via compliance pack

TLS 1.2 permits cipher suites with forward-secrecy gaps (RSA key
exchange, CBC mode, SHA-1 PRF). Even with the two permitted suites in
§B-2.4, TLS 1.2 is fundamentally weaker than TLS 1.3 because it
lacks the improved key schedule, the 0-RTT resumption security model,
and mandatory forward secrecy for all handshakes. The compliance pack
mechanism provides an audited escape hatch for legacy clients without
creating a blanket TLS 1.2 surface on substrate endpoints.

---

## §D — Operational Parameters (Normative)

### §D-1 TLS 1.3 Session Parameters

| Parameter | Value | Authority |
|-----------|-------|-----------|
| Session tickets | disabled for substrate µservices; enabled (max 24 h) for edge | ADR-0044 §D-3 |
| 0-RTT (early data) | disabled on all endpoints | Security: replay attack risk |
| SNI extension | required; ECH outer-SNI for external; inner-SNI carries real hostname | §B-3 |
| Max TLS record size | 16 384 bytes (RFC 8449 record_size_limit = 16384) | Performance |
| HKDF variant for HSM sessions | HKDF-SHA-384 (higher strength for HSM-backed private keys) | §B-2.1 |
| Renegotiation | forbidden (TLS 1.3 has no renegotiation) | Protocol |

### §D-2 QUIC Transport Parameters

| Parameter | Value |
|-----------|-------|
| QUIC version | RFC 9000 v1 (0x00000001) primary; RFC 9369 v2 (0x6b3343cf) negotiated |
| QUIC initial max_data | 10 MB |
| QUIC initial max_stream_data_bidi_local | 1 MB |
| QUIC max_idle_timeout | 30 000 ms |
| QUIC stateless reset | enabled; token rotated every 1 h |
| QUIC connection migration | enabled for client-to-edge; disabled for inter-cell |
| QUIC ACK frequency (RFC 9406) | enabled; min_ack_delay = 1 ms |

### §D-3 ECH Operational Parameters

| Parameter | Value |
|-----------|-------|
| ECH version | draft-ietf-tls-esni-18 (RFC 9460 when published) |
| HPKE suite | DHKEM(X25519, HKDF-SHA256) + HKDF-SHA256 + AES-128-GCM |
| ECH key rotation | 24 h automated; 48 h overlap for zero-downtime |
| ECHConfigList DNS record type | HTTPS (RFC 9460 §4) + SVCB |
| DNS TTL for ECH records | 3 600 s |
| Inner SNI | real tenant hostname (encrypted) |
| Outer SNI | shared service hostname (e.g., api.oyatie.dev) |

### §D-4 PQC KEM Parameters

| Parameter | Value |
|-----------|-------|
| KEM algorithm | ML-KEM-768 (FIPS 203) |
| Classical component | X25519 (RFC 7748) |
| Hybrid scheme | X25519MLKEM768 (IANA code point 0x11EC) |
| Combiner | Concatenation KDF per draft-ietf-tls-hybrid-design |
| Mandatory from | 2026-05-20 for new external endpoints; 2027-12-31 for all inter-cell |

### §D-5 PQC Signature Parameters

| Parameter | Value |
|-----------|-------|
| Classical component | Ed25519 (RFC 8032) |
| PQC component | ML-DSA-65 (FIPS 204, security level 3) |
| Composite OID | id-MLDSA65-Ed25519 (draft-ietf-lamps-pq-composite-sigs) |
| Signature size | Ed25519: 64 bytes; ML-DSA-65: 3 293 bytes; composite: 3 357 bytes |
| Used for | TLS CertificateVerify + Cedar fragment cosign + SPIFFE SVID |

### §D-6 Observability SLOs

| Metric | SLO target | Alert threshold |
|--------|-----------|----------------|
| `protocol_fallback_rate` (h3→h2) | < 2 % | > 5 % for > 5 min → P2 |
| `protocol_fallback_rate` (h2→h1.1) | < 0.5 % | > 2 % for > 5 min → P1 |
| `ech_plaintext_sni_fallback_rate` | < 0.1 % | > 1 % for > 5 min → P2 |
| `pqc_classical_fallback_rate` | < 5 % | > 20 % for > 1 h → P2 + council-security |
| `tls_handshake_p99_ms` (external) | < 80 ms | > 150 ms for > 5 min → P2 |
| `tls_handshake_p99_ms` (inter-cell) | < 15 ms | > 40 ms for > 5 min → P2 |

### §D-7 Spec Field Additions

Every endpoint declaration governed by this amendment MUST serialize
the `TransportEndpointSpec` shape from
`network/ports/transport-profile/endpoint-transport-profile.contract.json`.
External endpoint declarations use this canonical JSON shape:

<!-- transport-profile-external-example:start -->
```json
{
  "endpoint_id": "api-gateway-public",
  "capability_class": "external",
  "protocol": "http3",
  "fallback_protocol": "http2",
  "tls_profile": "strict_tls13",
  "alt_svc": "h3=\":443\"; ma=86400",
  "fallback_timeout_ms": 500,
  "ech": {
    "enabled": true,
    "support_required": true,
    "plaintext_sni_fallback_allowed": true,
    "key_rotation_hours": 24,
    "grease_retry_configs": true,
    "outer_sni": "api.oyatie.dev"
  },
  "pqc": {
    "enabled": true,
    "hybrid_negotiation_required": true,
    "kem": "x25519mlkem768",
    "signature": "ed25519+ml_dsa_65",
    "supported_groups": [
      "x25519mlkem768",
      "x25519"
    ],
    "classical_transition_fallback_allowed": true
  }
}
```
<!-- transport-profile-external-example:end -->

For inter-cell and internal endpoints, the same serialized shape is
used with `capability_class: "inter_cell"` or `"internal"`,
`protocol: "grpc_http2"`, `tls_profile: "spiffe_mtls_tls13"`,
`fallback_protocol: null`, `alt_svc: null`, `fallback_timeout_ms: null`, and
`ech.enabled: false`. Inter-cell endpoints declare hybrid PQC posture
with `pqc.enabled: true`; internal endpoints may set `pqc.enabled:
false` unless a substrate-specific ADR raises the class.

---

## §E — Alternatives Considered

### E-1 Pure ML-KEM (no classical hybrid)

**Rejected.** ML-KEM-768 alone would be sufficient for quantum
resistance, but breaks all current TLS stacks that do not yet
support pure-PQC negotiation. The hybrid approach allows incremental
deployment while maintaining quantum resistance. The downgrade-safety
argument (§C) also favours the hybrid over either component alone.

### E-2 NTRU or Classic McEliece as PQC KEM

**Rejected.** ML-KEM (CRYSTALS-Kyber) was chosen by NIST for FIPS
203. NTRU was considered but not standardised in the final NIST PQC
round-4 selection. Classic McEliece provides stronger quantum security
margins but produces unacceptably large public keys (1 MB+), making
it impractical for TLS handshakes within the latency SLO in §D-6.

### E-3 ECH as optional (tenant opt-in)

**Rejected.** Making ECH optional creates a split-world where some
tenants leak SNI and others do not, complicating the privacy audit
surface. The blanket mandate for external endpoints is operationally
simpler and aligns with ADR-0242 (oyatie-is-a-tenant) privacy
isolation. The compliance cost is low: ECH is now supported by all
major TLS libraries and CDN providers.

### E-4 Defer PQC until post-quantum threat is materialised

**Rejected.** ADR-0250 (build ahead of certification) and the
`feedback_build_ahead_of_certification` doctrine explicitly require
building the certified shape day one, never retrofitting compliance.
Harvest-now-decrypt-later attacks are already occurring against
recorded TLS sessions. Deploying PQC hybrid now costs one extra
ClientHello key_share (< 0.5 ms p99 latency); retrofitting later
requires re-keying every stored session credential.

### E-5 HTTP/2 as primary protocol (defer HTTP/3)

**Rejected.** HTTP/3 (QUIC) eliminates head-of-line blocking at the
transport layer, which is the primary latency bottleneck for
multi-stream API responses (intelligence inference streaming,
workflow event streams, audit-chain batch ingestion). The Alt-Svc
fallback mechanism in §B-1 provides safe degradation without
sacrificing HTTP/3 benefits for clients that support it.

---

## Consequences

### F-1 Impact on ADR-0295 Bootstrap SPIFFE

ADR-0295 §D-2 specifies that bootstrap runners are bound via SPIFFE
SVIDs. This amendment extends the SVID signature requirement: all
bootstrap-runner SVIDs MUST use the `ed25519+ml_dsa_65` composite
signature defined in §B-4.2. This ensures the SPIFFE trust chain is
quantum-resistant from day one of the bootstrap window.

### F-2 Impact on ADR-0247 Cedar Fragment Cosign

ADR-0247 §D-5 (Wave-3-A amendment) requires cosign attestation per
Cedar fragment artifact. This amendment specifies that the cosign
signature MUST use the `ed25519+ml_dsa_65` composite scheme from
§B-4.2. The bootstrap-ca (ADR-0295) MUST issue composite-signature
certificates for the cosign signing key.

### F-3 Impact on ADR-0293 Meta-Trust-Root

ADR-0293 defines the offline HSM trust anchor. The HSM's signing key
for the meta-trust-root witness signatures MUST use ML-DSA-87
(FIPS 204 security level 5) rather than ML-DSA-65, given the
higher criticality of the offline root. ML-DSA-87 is not mandated
broadly (§B-4.2 uses level 3) but the HSM-offline root requires the
highest available quantum security margin.

### F-4 Impact on MLS (RFC 9420)

The connect µservice uses MLS for E2EE group messaging. MLS key
packages contain the leaf node's HPKE public key for QUIC-based
delivery. When delivering MLS messages over the connect signalling
channel, the QUIC connection MUST use `x25519mlkem768` to protect
the MLS key package delivery itself. The MLS internal ratchet tree
is not affected by this amendment (MLS post-quantum support is
tracked separately).

### F-5 Impact on Cloud KMS encryption-BYOK

Cloud KMS handles tenant encryption-BYOK key material. The KMS API endpoint is
internal-only (mesh mTLS). The `pqc` block in the KMS endpoint spec
is still REQUIRED (per §B-5 table) to protect key-material transport
against future harvest-now-decrypt-later attacks on inter-cell KMS
traffic.

---

## §G — Naming Justification

Per `feedback_naming_justification` doctrine: every new name must
carry a one-line justification.

| New name / identifier | Justification (v4 BNF + 12-layer compliance) |
|-----------------------|----------------------------------------------|
| `oya-check-tls13-strict-profile` | `oya` prefix + `check` layer (CI lane) + `tls13-strict-profile` concept (kebab-case, ASCII lowercase) — ADR-0056 BNF v4.1 compliant |
| `oya-check-http3-fallback-chain` | `oya` prefix + `check` layer + `http3-fallback-chain` concept — ADR-0056 compliant |
| `oya-check-ech-endpoint-coverage` | `oya` prefix + `check` layer + `ech-endpoint-coverage` concept — ADR-0056 compliant |
| `oya-check-pqc-hybrid-kem` | `oya` prefix + `check` layer + `pqc-hybrid-kem` concept — ADR-0056 compliant |
| `x25519mlkem768` | IANA-registered TLS supported_group name — no oyatie-internal name introduced |
| `ed25519+ml_dsa_65` | Composite signature scheme per draft-ietf-lamps-pq-composite-sigs; `+` separator is the IETF draft convention |
| `legacy-tls12-grace` | compliance pack id — `legacy-tls12-grace` is kebab-case; matches ADR-0251 pack-id BNF |
| `pqc-transition-grace` | compliance pack id — `pqc-transition-grace` is kebab-case; matches ADR-0251 pack-id BNF |
| `protocol_fallback` | structured log event name — snake_case per ADR-0153 observability naming convention |
| `pqc_hybrid_kem_negotiated_total` | Prometheus counter name — snake_case + `_total` suffix per OpenMetrics convention |
| `pqc_classical_fallback_total` | Prometheus counter name — snake_case + `_total` suffix per OpenMetrics convention |
| `network-transport-profile` | `network` domain prefix + `transport-profile` typed endpoint contract; follows existing network port crate naming |
| `TransportEndpointSpec` | typed declaration for one endpoint's protocol posture without naming a runtime engine |
| `TransportProfilePort` | provider-neutral port returning endpoint transport declarations to future adapters |

---

## Change Log

| Date | Author | Change |
|------|--------|--------|
| 2026-06-26 | waveB-iac-transport-773-agent | Adds §B-8 typed protocol-agnostic transport port and contract artifact for issue #773; clarifies external HTTP/3 versus inter-cell/internal gRPC over HTTP/2 and defers QUIC engine adapters. |
| 2026-05-20 | wave-3-a-cross-reference-wiring-agent | Initial creation. Amends ADR-0253 §D-4/§D-5/§D-7. Codifies HTTP/3→HTTP/2→HTTP/1.1 fallback chain (§B-1), strict TLS 1.3 profile (§B-2), ECH RFC 9460 (§B-3), PQC hybrid X25519MLKEM768 + ed25519+ml_dsa_65 (§B-4), per-µservice protocol table (§B-5), 4 CI enforcement lanes (§B-6), migration plan (§B-7). Cross-cutting impact noted for ADR-0293/0295/0247/Connect/KMS (§F). |
