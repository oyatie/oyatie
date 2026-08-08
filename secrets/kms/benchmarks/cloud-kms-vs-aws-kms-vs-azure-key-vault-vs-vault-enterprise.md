# `cloud-kms` µservice — Benchmark vs AWS KMS, GCP Cloud KMS, Azure Key Vault, HashiCorp Vault Enterprise

> Measured 2026-04-25 to 2026-05-13 across 3 trial windows × 4 workloads (DEK-only, sign-only, mixed, PQC-only).
> All vendors over HTTPS/HTTP-2. `cloud-kms` runs HTTP/3 (QUIC) by default per ADR-0253. Pricing from each vendor's public sheet
> on 2026-05-13.

## DEK issuance latency (`GenerateDataKey` equivalent, 32-byte AES key)

| Surface | p50 | p95 | p99 | Cold-start |
| --- | --- | --- | --- | --- |
| `cloud-kms` (paid, HSM-cached) | **2.8 ms** | **3.9 ms** | 6.2 ms | 0 ms (warm pool) |
| `cloud-kms` (paid, AWS CloudHSM) | 5.1 ms | 7.8 ms | 12.4 ms | 14 ms |
| AWS KMS (`GenerateDataKey`) | 6.4 ms | 11.2 ms | 19.8 ms | n/a |
| GCP Cloud KMS (`AsymmetricSign` proxy) | 8.2 ms | 14.6 ms | 26.4 ms | n/a |
| Azure Key Vault (`getSecret` + envelope) | 11.8 ms | 22.4 ms | 38.6 ms | n/a |
| HashiCorp Vault Enterprise (transit + HSM seal) | 4.6 ms | 7.1 ms | 11.2 ms | 22 ms |

## Sign latency (RSA-3072 PKCS#1 v1.5)

| Surface | p50 | p95 | p99 |
| --- | --- | --- | --- |
| `cloud-kms` (paid, Thales Luna 7) | **4.2 ms** | **5.8 ms** | 8.4 ms |
| `cloud-kms` (paid, CloudHSM) | 7.9 ms | 11.6 ms | 17.8 ms |
| AWS KMS Sign | 12.6 ms | 22.4 ms | 38.2 ms |
| GCP Cloud KMS Sign | 14.2 ms | 26.8 ms | 42.6 ms |
| Azure Key Vault Sign | 18.4 ms | 32.6 ms | 54.2 ms |
| Vault Enterprise (transit RSA-3072) | 9.4 ms | 14.6 ms | 22.4 ms |

## Sign latency (ECDSA P-256)

| Surface | p50 | p95 | p99 |
| --- | --- | --- | --- |
| `cloud-kms` (paid) | **1.6 ms** | **2.2 ms** | 3.4 ms |
| AWS KMS | 6.8 ms | 12.4 ms | 19.8 ms |
| GCP Cloud KMS | 7.4 ms | 13.6 ms | 22.4 ms |
| Azure Key Vault | 9.2 ms | 16.8 ms | 28.4 ms |
| Vault Enterprise | 4.2 ms | 6.4 ms | 9.8 ms |

## Sign latency (ML-DSA-65, FIPS 204 PQC)

| Surface | p50 | p95 | p99 |
| --- | --- | --- | --- |
| `cloud-kms` (paid) | **2.8 ms** | **3.9 ms** | 5.6 ms |
| AWS KMS | not supported (as of 2026-05) | — | — |
| GCP Cloud KMS | preview only; ~22 ms | ~38 ms | ~62 ms |
| Azure Key Vault | preview; ~28 ms | ~46 ms | ~74 ms |
| Vault Enterprise | preview (enterprise+; ~12 ms) | ~18 ms | ~28 ms |

## HSM compliance + custody surface

| Surface | FIPS 140-3 | Common Criteria | Operator-quorum custody | Cryptoshred SLO | XKS / external HSM |
| --- | --- | --- | --- | --- | --- |
| `cloud-kms` (paid/paid/paid) | L2 / L3 / L3 | — / — / EAL 4+ | M-of-N at paid | 30 min / 5 min / 60 s | ✅ (AWS XKS + PKCS#11) |
| AWS KMS | L2 (multi-tenant) / L3 (CloudHSM) | — | ❌ | N/A (24h scheduled deletion) | ✅ (XKS) |
| GCP Cloud KMS | L2 / L3 (HSM ring) | — | ❌ | N/A (24h schedule) | partial (EKM) |
| Azure Key Vault | L2 / L3 (Managed HSM) | — | partial (Managed HSM) | N/A | partial (BYOK) |
| Vault Enterprise | L2 / L3 (HSM seal) | varies by HSM | ✅ (Auto-Unseal quorum) | configurable | ✅ |

## TCO at 50,000 DEK/sec sustained, 5 M signs/day, mid-market scope

| Surface | License | HSM | API calls | Storage | Total monthly | Annual |
| --- | --- | --- | --- | --- | --- | --- |
| `cloud-kms` (paid) | included | $3,200 | included | included | **$4,800** | **$57,600** |
| AWS KMS | $1/CMK + $0.03/10k req | $0 (multi-tenant) | $6,500 | $0 | $7,300 | $87,600 |
| AWS KMS + CloudHSM | $1.60/HSM/h × 3 HSMs | $3,460 | $0 | $0 | $4,500 + ops staff | n/a |
| GCP Cloud KMS | $0.03/10k req + $1/CMK | $1.50/key-hour (HSM) | $6,800 | $0 | $7,800 | $93,600 |
| Azure Key Vault Managed HSM | $3,600/HSM/mo × 3 | $10,800 | $0 (incl) | $0 | $11,500 | $138,000 |
| Vault Enterprise (Plus) | $5,200 | + HSM cost | $0 | $0 | $9,400 (with Thales) | $112,800 |

`cloud-kms` (paid) is **34 % below AWS KMS API model** and **48 % below Vault Enterprise + Thales** at this scale. Below paid
the gap narrows; above paid the cost crossover is at ~150k DEK/sec.

## Where vendors still win

1. **Public sign-up.** AWS KMS / Azure Key Vault / GCP KMS available on any account; `cloud-kms` requires tenant provisioning.
2. **AWS-native integration breadth.** AWS KMS is wired into 200+ AWS services; `cloud-kms` integrates with AWS via XKS only.
3. **Vault dynamic secrets.** Vault Enterprise issues dynamic database creds, AWS STS, etc.; `cloud-kms` defers to `cloud-secrets`.
4. **Marketplace HSM catalog.** Vault's HSM-seal supports a long list of vendor HSMs out-of-the-box; `cloud-kms` ships Marvell + Thales + Utimaco at v1.

## Where `cloud-kms` wins

1. **DEK issuance ≤ 4 ms p95** — 2-5× faster than AWS KMS at paid tenant_class.
2. **PQC GA** — ML-KEM/ML-DSA at paid; vendors still in preview.
3. **Cryptoshredding as a first-class operation** — vendors offer "scheduled deletion" (24h+), not immediate cryptoshred with HSM attestation.
4. **BLAKE3 audit chain** — tamper-evident; vendor audit logs are append-only.
5. **M-of-N operator quorum at paid** — Utimaco HSM operator cards; vendors usually require external tooling.
6. **AAD-mandatory at v0.42+** — refuses DEKs without AAD binding; AWS KMS makes it optional.
7. **HTTP/3 (QUIC) RPC** — ADR-0253; vendors are HTTP/2.
8. **Per-tenant compliance pack overlays** — flip PCI/HIPAA/EU-AI-Act rules per tenant.

## Reproducibility

```bash
make benchmarks.cloud-kms.run \
  VENDORS="cloud-kms,aws-kms,gcp-kms,azure-key-vault,vault-enterprise" \
  WORKLOADS="dek-only,sign-only,mixed,pqc-only" \
  TRIALS=3
```

Evidence: `.foundry/evidence/benchmarks/cloud-kms/2026-05-13T16:42:18Z/`.
