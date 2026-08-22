---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: cloud-secrets
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-cloud-secrets + ops-security
deciders: council-architecture, ops-security, axis-cloud-secrets, council-privacy
methodology: STRIDE (Microsoft) + LINDDUN (privacy) + OWASP Top 10 (2021) + NIST SP 800-154 + NIST SP 800-57 (Key Management) + FIPS 140-3
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0120, ADR-0139, ADR-0131, ADR-0132, ADR-0133]
related_specs: [/specs/per-microservice-flat-layout.json]
review_cadence: quarterly + on every OpenBao or HSM architecture change
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1, CC6.2, CC6.3, CC6.6, CC6.7, CC6.8, CC7.1, CC7.2, CC7.3, CC7.4, CC7.5, CC8.1"
  - "ISO 27001:2022: A.5.7, A.5.10, A.5.14, A.5.15, A.5.16, A.5.17, A.5.18, A.5.23, A.5.26, A.5.31, A.5.32, A.5.33, A.8.2, A.8.3, A.8.5, A.8.7, A.8.10, A.8.11, A.8.12, A.8.15, A.8.16, A.8.20, A.8.21, A.8.23, A.8.24, A.8.25, A.8.26, A.8.27, A.8.28"
  - "GDPR Arts. 5, 25, 28, 30, 32, 33, 34"
  - "PCI-DSS v4.0 §3.5, §3.6, §3.7, §8.6, §10.5"
  - "NIST SP 800-57 Part 1 (Key Management — General)"
  - "FIPS 140-3 (Cryptographic Module Validation)"
suggested_frameworks_by_pack:
  pack-kr: ["KR-ISMS-P §2.7 (암호통제)", "KR PIPA Art. 29 (안전성 확보조치)", "KR PIPA Enforcement Decree Arts. 29-30", "KR-FSS sector guidance (encryption + audit retention)"]
  pack-us-healthcare: ["HIPAA §164.312(a)(2)(iv) Encryption and Decryption", "HIPAA §164.312(e)(2)(ii) Encryption (transmission)", "HIPAA §164.308(a)(4) Information Access Management", "HIPAA §164.308(a)(5)(ii)(D) Password Management"]
  pack-eu: ["GDPR Art. 32(1)(a) pseudonymisation + encryption", "GDPR Art. 25 data protection by design", "eIDAS 910/2014 Art. 24 (qualified signature)", "NIS2 Art. 21(2)(h) cryptography"]
  pack-jp: ["APPI Art. 23 (anzen kanri sochi) safety control measures"]
  pack-sg: ["PDPA 2012 §24 Protection Obligation", "MAS-TRM v2021 §9 Cryptography"]
  pack-au: ["Privacy Act APP 11.1 reasonable steps to protect", "APRA-CPS 234 §29-36 Information Security"]
  pack-in: ["DPDPA 2023 §8(5) reasonable security safeguards", "RBI Master Direction on IT Governance §6.4 Cryptographic Controls"]
  pack-br: ["LGPD Art. 46 + Art. 50 (security measures)", "BACEN Res. 4.893/2021 §29 Cryptography"]
  pack-ae: ["UAE PDPL Art. 20 Security of Personal Data"]
  pack-ksa: ["PDPL Royal Decree M/19/2021 Art. 19 security obligations", "SAMA Cybersecurity Framework §4.3.4 Cryptography Standards", "KSA NCA ECC-1:2018 Cryptography"]
doc_status: published
---

# Threat Model: cloud-secrets µservice

## Purpose

Identify, classify, and mitigate threats to the cloud-secrets µservice's confidentiality, integrity, availability, and privacy posture. The cloud-secrets substrate holds **every secret used by every other oyatie µservice and every tenant** — a compromise here cascades to every product. This document is the canonical security artifact reviewed by SOC 2 Type 2 examiners, ISO 27001 auditors, GDPR DPAs, HIPAA covered-entity counsel, PCI-DSS QSAs, and equivalent supervisory authorities in every active pack.

## Scope

### In-scope

All components introduced by ADR-0131 Cloud split and this PRD, deployed in a **dedicated cloud-secrets Kubernetes cluster** (decision confirmed 2026-05-17; internal isolation target_non_claim based on cell/blast-radius goals; AWS Secrets Manager, GCP Secret Manager, and HashiCorp HCP Vault are reference patterns, not evidence that Oyatie currently matches provider practice):

| Layer-A (adopted OSS / vendor) | Layer-B (oyatie-owned) |
|---|---|
| OpenBao 2.x LTS (Vault open-source fork) | `cloud-secrets-secret-reference-resolver-*` (9 crates) |
| Patroni-HA Postgres (OpenBao storage backend) | `cloud-secrets-openbao-operator-*` (6 crates) |
| OCI Cloud-HSM (FIPS 140-3 Level 3 HSM partitions) | `cloud-secrets-key-rotation-scheduler-*` (7 crates) |
| Thales Luna HSM (pack-kr regulated tenants) | `cloud-secrets-hsm-integration-*` (5 crates) |
| cert-manager (mTLS issuance) | `cloud-secrets-per-tenant-namespace-controller-*` (6 crates) |
| SPIRE (SPIFFE identity issuance to µservice consumers) | `cloud-secrets-audit-emitter-*` (5 crates) |
| | SecretReference URI spec at `microservices/cloud-secrets/contracts/proto/cloud-secrets.proto` |
| | LEAN-A11 raw-secret-emission lane configuration |

### Out-of-scope

- Threats to the underlying Kubernetes cluster, container runtime, or hyperscaler IaaS layer — owned by `cloud-k8s` µservice's threat model.
- Threats to the `audit-chain` µservice itself — owned by `audit-chain` threat-model (this doc inherits as upstream).
- Threats to GitHub Actions runners — owned by `governance` µservice.
- Threats to tenant client-side handling of resolved secrets after they cross the SDK boundary — tenant responsibility per DPA.
- Bominal-side secret manager counterpart — separate Bominal threat-model.

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────┐
│                                                                            │
│   Tenant operators (HSM attestation review, encryption-key BYOK upload)     │
│         │ (OIDC + MFA + JIT short-lived token from OpenBao)                │
│         ▼                                                                  │
│  ┌─ Public ingress (Envoy/Istio gateway, mTLS-only) ─────────────────────┐ │
│  │  - TLS termination + WAF + DDoS                                       │ │
│  │  - mTLS-only for admin endpoints; OIDC bearer for tenant reads        │ │
│  │  - SecretReference resolution is in-cluster ONLY; no public endpoint  │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────│─────────────────────────────────────────────┘
                               ▼
┌─ Dedicated cloud-secrets cluster ──────────────────────────────────────────┐
│                                                                            │
│  Trust boundary 1: External → Cluster admin ingress (admin endpoints only) │
│                                                                            │
│  ┌─ openbao-operator-rest (admin) ─┐    ┌─ Auditor JIT read API ─┐         │
│  │  OIDC + Cedar; ops-security only │    │  time-boxed; OIDC+MFA  │         │
│  └──────────────────────────────────┘    └────────────────────────┘         │
│             │                                                              │
│  Trust boundary 2: In-cluster µservice → cloud-secrets (SDK + SPIFFE)      │
│             │                                                              │
│  ┌─ Resolver SDK (in consumer µservice process) ────────────────────────┐  │
│  │  - SPIFFE workload-identity attests to consumer µservice            │  │
│  │  - mTLS to OpenBao via cert-manager-issued workload cert            │  │
│  │  - Tenant context derived from inbound request (tenancy µservice)   │  │
│  │  - In-process LRU + TTL ≤60s; never logs resolved value             │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│             │                                                              │
│  ┌─ OpenBao cluster (5-node Raft, per-pack) ────────────────────────────┐  │
│  │  - Per-tenant namespace                                              │  │
│  │  - Per-µservice scope policies                                       │  │
│  │  - KV v2 + Transit + PKI + Kubernetes auth                           │  │
│  │  - Auto-unseal via HSM-protected KEK                                 │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│             │                                                              │
│  Trust boundary 3: OpenBao → HSM (PKCS#11 / KMIP)                          │
│             │                                                              │
│  ┌─ HSM partition (OCI Cloud-HSM / Thales Luna; FIPS 140-3 Level 3) ───┐  │
│  │  - KEK never leaves HSM partition                                    │  │
│  │  - Signing operations only; no key export                            │  │
│  │  - Attestation report every 24h                                      │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│             │                                                              │
│  Trust boundary 4: OpenBao → Postgres-HA (storage backend)                 │
│             │                                                              │
│  ┌─ Patroni-HA Postgres (per-pack) ─────────────────────────────────────┐  │
│  │  - Encrypted at rest (LUKS + per-pack KEK)                           │  │
│  │  - mTLS between OpenBao and Postgres                                 │  │
│  │  - Backups encrypted with per-pack BCDR KEK                          │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│             │                                                              │
│  Trust boundary 5: audit-emitter → audit-chain µservice                    │
│             │                                                              │
│  ┌─ Audit-chain bridge (Ed25519 signed events) ─────────────────────────┐  │
│  │  - Append-only; cryptographically sealed                             │  │
│  │  - Per-pack residency-pinned audit-chain instance                    │  │
│  └──────────────────────────────────────────────────────────────────────┘  │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

Five trust boundaries:
1. **External → Cluster admin ingress** (TLS + WAF + DDoS; admin endpoints only).
2. **In-cluster µservice → cloud-secrets (SDK + SPIFFE)** (mTLS; per-µservice tenant-scoped policy).
3. **OpenBao → HSM** (PKCS#11 / KMIP; KEK never exits).
4. **OpenBao → Postgres-HA** (mTLS; LUKS-at-rest).
5. **audit-emitter → audit-chain** (Ed25519 signed events).

## Assets & Data Classification

Per Bominal ADR-0028 (audit-chain + data-class taxonomy) and the `check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Raw secret values (KV v2 entries) | `SECRET` | Critical | per rotation policy (30d API keys, 90d signing keys, 365d KEK) | OpenBao KV (per-pack) |
| KEK (Key Encryption Key) material | `SECRET` (sub-class: ROOT_KEY) | Critical+ | 365d rotation; HSM-resident | HSM partition (per-pack) |
| DEK (Data Encryption Key) material | `SECRET` (sub-class: TENANT_DEK) | Critical | rotated on KEK rotation cascade | OpenBao Transit (per-tenant) |
| Tenant identifiers (used as OpenBao namespace path) | `SENSITIVE_PIPA_ART23` (re-identification potential) | High | per-tenant lifecycle | OpenBao namespace metadata + Postgres |
| Secret-access audit events | `AUDIT` | High | per-pack legal cadence (KR ≥1y, HIPAA 6y, PCI-DSS ≥1y) | audit-chain µservice |
| Rotation policy definitions | `INTERNAL_ONLY` | Low | append-only git history + OpenBao policy store | repo + OpenBao |
| HSM attestation reports | `AUDIT` | High | 7y (regulatory) | audit-chain |
| Per-µservice SPIFFE workload certs | `SECRET` (short-lived; ≤24h) | Critical | 24h TTL | cert-manager |
| Cedar policy fragments | `INTERNAL_ONLY` (policy text) | Medium | git history | `microservices/cloud-secrets/policy/*.cedar` |
| OpenBao operator service-account tokens | `SECRET` | Critical | per-deployment rotation | OpenBao (bootstrap via auto-unseal) |
| BYOK uploads from tenants | `SECRET` (sub-class: TENANT_BYOK) | Critical | per tenant DPA; wrapped under our KEK-of-KEKs | OpenBao + HSM |
| Cache contents (in-process resolver) | `SECRET` (transient in-memory) | Critical | TTL ≤60s; flushed on revocation push | consumer µservice process memory |
| OpenBao audit-device log | `AUDIT` | High | append-only; bridged to audit-chain | OpenBao audit-device file + audit-chain |
| Revocation-push events | `AUDIT` + `INTERNAL_ONLY` | Medium | 90d | OpenBao + audit-chain |

## Actors

| Actor | Trust level | Authentication | Capability |
|---|---|---|---|
| In-cluster µservice consumer (Resolver SDK) | Semi-trusted internal | SPIFFE workload identity + mTLS | Resolve secrets within its tenant + scope; never list; never write |
| openbao-operator controller | Trusted internal | Kubernetes ServiceAccount + auto-unseal HSM access | Manage OpenBao cluster lifecycle |
| key-rotation-scheduler worker | Trusted internal | SPIFFE identity + OpenBao policy `rotate` | Initiate rotation; trigger cascade; emit audit |
| per-tenant-namespace-controller | Trusted internal | SPIFFE identity + OpenBao policy `namespace-admin` | Provision + seal tenant namespaces |
| audit-emitter | Trusted internal | SPIFFE identity + audit-chain bridge cert | Append audit events; cannot read or modify existing |
| Tenant operator (encryption-key BYOK upload, attestation review; ADR-0251 §D-10) | Untrusted external | OIDC + MFA + JIT short-lived token | Upload encryption-key BYOK to own tenant namespace; review own HSM attestation; never read other tenants |
| ops-security (human) | Trusted internal | OIDC + MFA + JIT elevation + ops-security group | Admin OpenBao; rotate KEK; quarantine namespace; cannot read raw secrets without break-glass |
| External auditor (SOC 2 / ISO / PCI-DSS QSA) | Read-only external on time-boxed window | OIDC + MFA + JIT token | Read audit-chain events; read policy + IaC; cannot read raw secrets |
| Reviewer agent (pr-review lane) | Trusted internal | OIDC-bound CI identity | Read OpenBao policy + IaC for PR review; cannot resolve secrets |
| Attacker — opportunistic | Untrusted | none | Scan + low-skill; assume always present |
| Attacker — targeted (nation-state, financially motivated) | Untrusted | none | Sophisticated; supply-chain awareness; HSM-extraction attempts; insider-recruitment |
| Insider — accidental | Trusted internal | OIDC + MFA | Misconfigure policy, rotation cadence, namespace (mitigated by PR-review + LEAN gates + 4-eye admin operations) |
| Insider — malicious | Trusted internal | OIDC + MFA | Worst-case for confidentiality; mitigated by least-privilege + audit-chain non-repudiation + separation-of-duties + 4-eye break-glass |

## STRIDE Threat Catalog

Each threat carries: ID; category; asset; description; likelihood (L/M/H); impact (L/M/H); risk score; mitigations (concrete); owner; residual risk; framework controls satisfied.

### Spoofing (S)

**T-S-01 — Attacker forges SPIFFE workload identity to resolve another µservice's secrets**
- Asset: SecretReference resolution path
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - SPIFFE issuance gated on Kubernetes ServiceAccount attestation + node attestation (SPIRE node-attestor).
  - mTLS between SDK and OpenBao validates SPIFFE SVID; OpenBao Kubernetes auth method validates Service Account JWT against API server.
  - Per-µservice policy scopes secrets to `secret/<tenant>/<microservice>/*`; SPIFFE ID embedded in token; cross-µservice read denied even with valid mTLS.
  - Audit-emit `spiffe_identity_mismatch_attempt` on any deviation.
- Owner: ops-security
- Residual: L (SPIRE compromise + Kubernetes API compromise required)
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.5.17, A.8.2, A.8.3; GDPR Art. 32(1)(b); PCI-DSS §8.6

**T-S-02 — Attacker impersonates audit-emitter to suppress / fake audit events**
- Asset: audit-chain bridge path
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - audit-chain validates Ed25519 signatures from a fixed allowlist of emitter SPIFFE identities.
  - audit-chain is append-only with Merkle tree; tampering detected at audit-export.
  - OpenBao audit-device runs locally + bridge replays from local audit-device file (not in-memory); reconciliation cron compares local vs bridged.
  - Audit-emit on signature-mismatch.
- Owner: axis-cloud-secrets + axis-governance
- Residual: L (Ed25519 signing-key extraction + audit-chain compromise required)
- Frameworks: SOC 2 CC7.1, CC7.4; ISO 27001 A.5.10, A.5.15, A.8.15, A.8.16; HIPAA §164.312(b)

**T-S-03 — Attacker forges tenant identity to resolve another tenant's secrets**
- Asset: tenant namespace isolation
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Tenant context carried in inbound request (from tenancy µservice); SDK validates via signed tenant token.
  - OpenBao namespace policies are strict-match on tenant_id; cross-namespace reads denied.
  - Cedar policy at `microservices/cloud-secrets/policy/tenant-scope.cedar` enforces `principal.tenant_id == resource.tenant_id`.
  - Per-tenant DEK ensures even bypass of policy yields ciphertext only.
- Owner: ops-security + axis-cloud-secrets
- Residual: L (multiple-layer bypass required)
- Frameworks: SOC 2 CC6.1, CC6.2; ISO 27001 A.5.15, A.8.3; GDPR Art. 32; pack-kr PIPA Art. 29; HIPAA §164.308(a)(4)

### Tampering (T)

**T-T-01 — Attacker modifies rotation policy to delay rotation indefinitely**
- Asset: rotation policy definitions
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Rotation policies stored in OpenBao under `secret/policy/rotation/*` with write requiring 4-eye approval (OpenBao Sentinel policy).
  - Policy versions append-only; git history mirrors in repo (LEAN-A11 forbids raw secrets but rotation policy is INTERNAL_ONLY).
  - `cloud-secrets-key-rotation-scheduler-worker` emits `RotationOverdue` event after T+1 day past SLA.
  - Audit-emit on every policy mutation.
- Owner: axis-cloud-secrets
- Residual: L
- Frameworks: SOC 2 CC6.1, CC7.1; ISO 27001 A.5.17; PCI-DSS §3.6.4

**T-T-02 — Attacker tampers with cached secret in-process to inject malicious value**
- Asset: resolver in-process cache
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - Cache values held in `zeroize::Zeroizing<String>`; bounded TTL ≤60s.
  - Cache key includes HMAC over `(tenant_id, secret_path, version)`; deserialisation validates HMAC.
  - Resolved values used only via `Secret<T>` newtype that disables `Debug`/`Display` and zeroises on drop.
  - Tampering implies process-memory write access — attacker can read any in-process secret anyway; mitigation is process-isolation + minimal cache TTL.
- Owner: axis-cloud-secrets
- Residual: M (process-memory write is the inherent threat model boundary; addressed by least-privilege containers + AppArmor/SELinux confinement)
- Frameworks: SOC 2 CC6.1; ISO 27001 A.8.12, A.8.24; PCI-DSS §3.5.1

**T-T-03 — Attacker tampers with the SecretReference URI in config to point to a different secret**
- Asset: SecretReference URI in deployed config
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - Per-µservice scope policy restricts secrets to `secret/<tenant>/<microservice>/*`; URI pointing outside the scope is denied at OpenBao.
  - PR-review of config changes + LEAN-A11 lane (secret-emission lane treats URI changes as scope changes; reviewer alerted).
  - Audit-emit unauthorised-path-attempt.
- Owner: axis-cloud-secrets + axis-governance
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.3; ISO 27001 A.5.15, A.8.3

### Repudiation (R)

**T-R-01 — Privileged operator denies having read a SECRET-class entry**
- Asset: audit-chain `SecretAccessed` events
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - OpenBao audit-device + audit-emitter bridge emit `SecretAccessed{principal_spiffe_id, secret_path_hash, accessed_at, request_id}` on every read.
  - audit-chain is append-only + Merkle-sealed; non-repudiation guaranteed cryptographically.
  - Break-glass reads ALSO audit-emit (with elevated-context flag).
  - Audit retention per pack legal cadence.
- Owner: axis-governance
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.28, A.8.15, A.8.16; GDPR Art. 30; HIPAA §164.312(b); PCI-DSS §10.2

**T-R-02 — Tenant denies having uploaded encryption-key BYOK that was used to encrypt their data (ADR-0251 §D-10)**
- Asset: encryption-key BYOK upload events (ADR-0251 §D-10)
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - encryption-key BYOK upload requires tenant OIDC + MFA + JIT short-lived token; signed receipt issued to tenant (ADR-0251 §D-10).
  - `KekAttested` audit-chain event records the encryption-key BYOK upload with tenant_id + KEK-of-KEKs SHA + timestamp.
- Owner: axis-cloud-secrets + council-privacy
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.5.28; GDPR Art. 30; eIDAS Art. 24

### Information Disclosure (I)

**T-I-01 — Raw secret committed to the git repo (the user-flagged directive)**
- Asset: repo + commit history
- Likelihood: H (without controls) → L (with LEAN-A11 BLOCKER) / Impact: Critical / Risk: **Critical → L**
- Mitigations:
  - LEAN-A11 `check-raw-secret-emission` lane BLOCKER: gitleaks + tartufo + oyatie custom patterns (Stripe sk_, AWS AKIA, GCP private key, GitHub PAT ghp_, OpenBao token hvb., HSM PKCS#11 PIN, etc.) refuse any PR introducing a credential-shaped string.
  - PreReceive Git hook (defence-in-depth) on `oya-vcs`-managed branches.
  - Quarterly retroactive scan via `cloud-secrets-secret-leak-scanner` (cron); any hit triggers Sev-1 incident.
  - Tenant DPA forbids tenant-supplied secret material in PR comments/chat.
- Owner: axis-cloud-secrets + axis-governance
- Residual: L (false-negative on novel patterns; mitigated by quarterly pattern update + reviewer-agent vigilance)
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.15, A.5.17, A.5.18; GDPR Art. 32(1)(a); HIPAA §164.312(a)(2)(iv); PCI-DSS §3.5.1, §3.5.2; KR PIPA Art. 29

**T-I-02 — Raw secret leaked in chat transcript / agent checkpoint (the user-flagged directive)**
- Asset: chat transcripts + .omc/state checkpoints + session ledger
- Likelihood: H (without controls) → L (with controls) / Impact: Critical / Risk: **Critical → L**
- Mitigations:
  - Agent-side redaction: every transcript pass through a redaction filter that masks credential-shaped strings before writing to disk.
  - `.omc/state/` filesystem confinement: writeable only by agent process; no public exposure.
  - Per-session OIDC: tenant-touching agents authenticate; raw secrets never returned to agent context (SDK returns opaque `Secret<T>` wrapper; agent prompts shown only the SecretReference URI).
  - Periodic scan: `omc-state-leak-scanner` against `.omc/state/sessions/*` (cron + on-commit if state files are tracked, which they should not be — `.gitignore`).
- Owner: axis-cloud-secrets + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.6; ISO 27001 A.5.15, A.5.34, A.8.12; GDPR Art. 32; HIPAA §164.312(a)(2)(iv); PCI-DSS §3.5.1

**T-I-03 — Side-channel timing attack reveals secret length or content from cache-hit timing**
- Asset: resolver SDK timing
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Constant-time comparison for cache-key match (`subtle::ConstantTimeEq`).
  - Cache lookup time padded to a noise budget so cache-hit vs cache-miss is indistinguishable at p99.
  - mTLS encrypts wire traffic; timing only observable from same-process attacker (already privileged).
- Owner: axis-cloud-secrets
- Residual: L
- Frameworks: ISO 27001 A.8.24; NIST SP 800-57

**T-I-04 — HSM partition compromise via supply chain (firmware backdoor)**
- Asset: HSM KEK material
- Likelihood: L / Impact: Critical / Risk: **H**
- Mitigations:
  - HSM vendor selection: FIPS 140-3 Level 3 minimum (OCI Cloud-HSM + Thales Luna both qualify).
  - Attestation report every 24h; cryptographically signed; deviation triggers Sev-1.
  - KEK ceremony performed in-person with 4-eye witness; key custodian rotation.
  - Multi-vendor strategy: pack-kr uses Thales Luna; other packs default OCI Cloud-HSM; reduces single-vendor compromise blast radius.
  - HSM partition has bounded operation rate (anti-replay); audit on every signing op.
- Owner: ops-security + ops-finance
- Residual: M (residual supply-chain risk; mitigated by multi-vendor + attestation)
- Frameworks: SOC 2 CC6.6, CC6.7; ISO 27001 A.5.19, A.5.20, A.5.23, A.8.12, A.8.30; FIPS 140-3; PCI-DSS §3.6.1; KR-FSS sector

**T-I-05 — Postgres backend backup leak exposes encrypted KV blobs**
- Asset: Postgres backups
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - OpenBao KV encrypted at rest with per-pack KEK (HSM-wrapped); Postgres sees ciphertext only.
  - LUKS-encrypted Postgres volumes; backups encrypted with per-pack BCDR KEK.
  - Backup storage in pack-pinned object storage with bucket policies forbidding cross-region replication.
- Owner: ops-sre + ops-security
- Residual: L (backup decrypt requires KEK; HSM mitigations apply)
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.17, A.5.30, A.8.12, A.8.13, A.8.24; PCI-DSS §3.6.4

**T-I-06 — Resolved secret logged accidentally by consumer µservice (third-party logging library)**
- Asset: consumer µservice logs
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - SDK returns `Secret<T>` newtype with `Debug` returning `"[REDACTED]"`; `Display` not implemented.
  - SDK refuses to return raw `String`; consumer must use scoped `with_secret(|s| ...)` callback that zeroises after use.
  - `check-secret-newtype-leak` LEAN lane scans for `format!("{:?}", secret)` and `.to_string()` on `Secret<T>` types.
  - Loki ingest scans for credential-shaped strings; matched lines redacted + Sev-2 incident.
- Owner: axis-cloud-secrets + axis-observability
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.7; ISO 27001 A.5.15, A.8.12, A.8.16; GDPR Art. 32(1)(a); HIPAA §164.312(a)(2)(iv); PCI-DSS §3.5.1

### Denial of Service (D)

**T-D-01 — Resolver SDK consumer storm overwhelms OpenBao**
- Asset: OpenBao availability
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - SDK in-process LRU cache with TTL ≤60s reduces backend load >95% under steady state.
  - OpenBao quota policies per-tenant + per-µservice; rate-limit at policy layer.
  - HPA on OpenBao read replicas (read-only Raft followers serve KV reads).
  - Circuit breaker in SDK: on consecutive OpenBao failures, serve from cache + alarm; tenant SLO error budget consumed.
- Owner: axis-cloud-secrets + ops-sre
- Residual: M
- Frameworks: SOC 2 A1.1, A1.2; ISO 27001 A.5.30, A.8.6, A.8.32; PCI-DSS §6.4.1

**T-D-02 — HSM partition operation queue saturated**
- Asset: HSM signing availability
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - HSM operation rate bounded per partition (signing ops ≤1000/s per partition); HPA-ish capacity-planning: add partitions when steady-state >70%.
  - Application caches PKI-issued certs (e.g., 24h TTL for workload certs) reduces HSM ops.
  - DEK cache reduces KEK signing for routine en/decrypt.
  - Saturation triggers Sev-2 + capacity-planning playbook.
- Owner: ops-security + ops-sre
- Residual: M
- Frameworks: SOC 2 A1.1; ISO 27001 A.8.6; PCI-DSS §6.4.1

**T-D-03 — Audit emission backpressure stalls SecretAccessed events**
- Asset: audit-chain bridge
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - audit-emitter writes to local append-only file first (durable), then bridges to audit-chain; backpressure backlogs locally without blocking resolution.
  - Backlog > 60s triggers Sev-2 page (axis-governance).
  - Per ADR-0028, audit-chain throughput target 100k events/s/cluster; per-pack horizontal scale.
- Owner: axis-governance + axis-cloud-secrets
- Residual: L
- Frameworks: SOC 2 CC7.1, CC7.2; ISO 27001 A.5.30, A.8.15; HIPAA §164.308(a)(1)(ii)(D)

**T-D-04 — Rotation cascade storm — many secrets due simultaneously**
- Asset: rotation scheduler throughput
- Likelihood: M / Impact: M / Risk: **M**
- Mitigations:
  - rotation-scheduler uses jittered schedule (±10% on rotation cadence) to spread load.
  - Per-pack rate-limit: max 100 concurrent rotations.
  - Stuck-rotation detection: any rotation past T+1 day SLA emits `RotationOverdue` and pages.
- Owner: axis-cloud-secrets
- Residual: L
- Frameworks: SOC 2 A1.1; ISO 27001 A.5.17

**T-D-05 — Namespace controller crash leaves tenant onboarding stalled**
- Asset: per-tenant-namespace-controller
- Likelihood: L / Impact: M / Risk: **L**
- Mitigations:
  - Controller is HA (2+ replicas, leader-election via Kubernetes lease).
  - Reconciliation is idempotent; restart resumes from last-applied state.
  - `NamespaceProvisioningStuck` event after T+10min on a `TenantRegistered` without controller progress.
- Owner: axis-cloud-secrets
- Residual: L
- Frameworks: SOC 2 A1.1; ISO 27001 A.8.6

### Elevation of Privilege (E)

**T-E-01 — Consumer µservice escapes its per-µservice scope to read another µservice's secrets**
- Asset: per-µservice scope policy
- Likelihood: L / Impact: H / Risk: **M**
- Mitigations:
  - OpenBao policy scopes secrets to `secret/<tenant>/<microservice>/*`; cross-µservice path denied.
  - Cedar policy at `policy/tenant-scope.cedar` defence-in-depth.
  - Audit-emit on cross-scope-attempt.
  - Reviewer-agent flags policy widenings in PR review.
- Owner: ops-security + axis-cloud-secrets
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.3; ISO 27001 A.5.15, A.8.3; GDPR Art. 32

**T-E-02 — Privileged operator break-glass without 4-eye approval**
- Asset: break-glass admin operations
- Likelihood: L / Impact: Critical / Risk: **H**
- Mitigations:
  - Break-glass requires OpenBao Sentinel policy `4_eye_approval`: 2 ops-security approvers; auto-expiry 1h.
  - JIT elevation through OpenBao identity provider; audit-emit every break-glass approval + use.
  - Quarterly review of break-glass use; pattern detection (e.g., same approver pair).
- Owner: ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC6.3; ISO 27001 A.5.15, A.5.16, A.8.2, A.8.3; HIPAA §164.308(a)(4); PCI-DSS §7.1, §7.2

**T-E-03 — OpenBao policy mis-author grants tenant operator unintended scope**
- Asset: OpenBao policy authoring
- Likelihood: M / Impact: H / Risk: **H**
- Mitigations:
  - Policies authored as code in `microservices/cloud-secrets/policy/openbao/*.hcl`; PR-reviewed.
  - Policy tests under `tests/policy/` validate scope per-policy.
  - LEAN-A12 `check-openbao-policy-scope` lane refuses policies granting >intended scope.
- Owner: axis-cloud-secrets + axis-governance
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15; PCI-DSS §7.2

## LINDDUN Threat Catalog (Privacy)

| ID | Category | Threat | Mitigation | Frameworks |
|---|---|---|---|---|
| L-L-01 | Linkability | Tenant identifiers in OpenBao namespace paths + audit events permit linkage across audit time-series | Salted-hash tenant_id (`tenant:<sha256(tenant_id+salt)[..16]>`); never log raw IDs; rotate salt per pack annually | GDPR Art. 25; KR PIPA Art. 23; HIPAA §164.514(b) |
| L-I-01 | Identifiability | Audit emission could embed tenant raw IDs | audit-chain receives only salted-hash tenant IDs; raw mapping in OpenBao tenant-resolver | GDPR Art. 25; KR PIPA Art. 23 |
| L-N-01 | Non-repudiation gap | Tenant denies encryption-key BYOK upload (ADR-0251 §D-10) | Signed receipt at upload time; `KekAttested` audit event with KEK-of-KEKs SHA | eIDAS Art. 24; GDPR Art. 30 |
| L-D-01 | Detectability | Probing whether a secret path exists reveals tenant + secret-name presence | 404 returned with constant-time response shape; audit-emit probe-attempt | ISO 27001 A.8.24 |
| L-DI-01 | Disclosure of information | Resolved value disclosed via verbose error message | Errors return opaque codes; never echo secret value or path | GDPR Art. 32; HIPAA §164.312(e) |
| L-U-01 | Unawareness | Tenant unaware of when their secret was accessed | Per-pack legal cadence: tenant audit export available; tenant DPA promises access transparency | GDPR Art. 15; KR PIPA Art. 35 |
| L-NC-01 | Non-compliance | Pack-specific regulator requires data sovereignty inspection | Per-pack OpenBao + HSM enables in-pack inspection; cross-pack data movement forbidden | KR PIPA Art. 28; GDPR Art. 44 |

## Mitigations Summary Matrix

| Mitigation | Threats addressed | CI lane / control |
|---|---|---|
| LEAN-A11 `check-raw-secret-emission` | T-I-01, T-I-02 | `cargo run -p dev-cli -- gate validate lean-a11` |
| SPIFFE workload identity | T-S-01 | SPIRE node-attestor + Kubernetes auth |
| Per-tenant OpenBao namespace | T-S-03, T-E-01 | OpenBao policy + Cedar |
| Per-µservice scope policy | T-S-01, T-E-01, T-E-03 | LEAN-A12 + OpenBao policy tests |
| HSM partition (FIPS 140-3 Level 3) | T-I-04 | HSM attestation cron + audit-chain seal |
| `Secret<T>` newtype + zeroize | T-I-06 | `check-secret-newtype-leak` lane |
| In-process LRU + TTL ≤60s | T-D-01 | SDK design |
| audit-chain Merkle + Ed25519 | T-R-01, T-R-02, T-S-02 | per Bominal ADR-0028 |
| 4-eye break-glass | T-E-02 | OpenBao Sentinel policy `4_eye_approval` |
| Rotation jitter ±10% | T-D-04 | scheduler design |
| Postgres LUKS + backup encryption | T-I-05 | IP-001 IaC |
| Local audit-device file (durable) | T-D-03 | OpenBao audit-device config |

## Review + Drill Schedule

| Drill | Frequency | Owner | Acceptance |
|---|---|---|---|
| Emergency-revoke chaos | monthly | ops-security | revocation propagates to 100 consumers within p99 ≤5s |
| HSM attestation verification | daily (automated) | ops-security | attestation passes; deviation Sev-1 |
| LEAN-A11 false-negative-rate review | quarterly | axis-governance | < 0.1% on a 1k-line seeded corpus |
| Break-glass review | quarterly | ops-security + ops-legal | every break-glass justified + post-mortem |
| Cross-pack write attempt drill | quarterly | ops-sre | attempt rejected + alert fires |
| Threat-model review | quarterly + on architecture change | council-architecture + ops-security | doc updated; new threats catalogued |

## References

- `microservices/cloud-secrets/PRD.md`
- `microservices/cloud-secrets/dpia.md`
- `microservices/cloud-secrets/policy/{tenant-scope,ci-scope,auditor-scope,public-read}.cedar`
- `microservices/cloud-secrets/policy/secret-isolation.md`
- `microservices/cloud-secrets/policy/data-residency.md`
- `microservices/cloud-secrets/incident-response.md`
- `microservices/cloud-secrets/runbooks/secret-leak-detected.md`
- `microservices/cloud-secrets/runbooks/hsm-key-rotation.md`
- ADR-0028 (audit-chain + data-class taxonomy, Bominal-inherited)
- ADR-0131 (Cloud split)
- NIST SP 800-57 Part 1 (Key Management)
- FIPS 140-3 (Cryptographic Module Validation)
- OpenBao → OpenBao migration notes (OpenBao governance)
- KR PIPA Art. 29 + Enforcement Decree Art. 30
- HIPAA 45 CFR §164.312
- GDPR Arts. 25, 28, 30, 32
- PCI-DSS v4.0 §3.5, §3.6
