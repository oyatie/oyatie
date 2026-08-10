---
doc_class: CompetitorParity
microservice: cloud-secrets
status: Accepted
date: 2026-05-17
owner_team: axis-cloud-secrets + gtm-customer-success
review_cadence: quarterly
doc_status: published
---

# Competitor Parity Matrix: cloud-secrets µservice

Parity dimensions are derived from competitor product documentation (canonical source for each). Each row marks oyatie's current posture, target posture, and gap to close.

## Dimensions

- D1: Per-region / per-pack residency
- D2: HSM-backed root key (KEK)
- D3: HSM FIPS 140-3 Level 3+
- D4: KV v2 (versioned secrets)
- D5: Transit engine (encryption-as-a-service)
- D6: PKI engine (certificate issuance)
- D7: KMIP support
- D8: Per-tenant namespace isolation
- D9: Per-µservice / per-application scope policies
- D10: Automatic key rotation
- D11: Cascade rotation (dependent re-wrap)
- D12: Revocation push (server-sent invalidation)
- D13: encryption-key BYOK (Bring Your Own Key; ADR-0251 §D-10)
- D14: encryption-key BYOK wrapped under platform KEK-of-KEKs (ADR-0251 §D-10)
- D15: HSM attestation (daily verified)
- D16: Audit log (signed / sealed)
- D17: Audit log Merkle / cryptographic non-repudiation
- D18: Per-region audit instance (residency)
- D19: SDK (multi-language)
- D20: SDK enforces `Secret<T>` wrapper
- D21: SDK enforces cache TTL ceiling
- D22: SDK enforces no-log on resolved value
- D23: Mechanical raw-secret-leak prevention (CI BLOCKER)
- D24: mTLS workload identity (SPIFFE)
- D25: OIDC + MFA + JIT for admin
- D26: 4-eye break-glass approval
- D27: HA / Raft consensus
- D28: DR-pair failover
- D29: Cryptographic erasure on tenant offboard
- D30: OSS / self-hostable / no vendor lock

## Matrix

| Dimension | HashiCorp Vault | AWS Secrets Manager + KMS | GCP Secret Manager + KMS | Azure Key Vault | OCI Vault | 1Password | Doppler | Infisical | Akeyless | **oyatie cloud-secrets (M01)** | **oyatie target** |
|---|---|---|---|---|---|---|---|---|---|---|---|
| D1 | per-region | per-region | per-region | per-region | per-region | global SaaS | global SaaS | self-host opt | global SaaS | **per-pack** | per-pack |
| D2 | Vault Enterprise+HSM | KMS-CMK | Cloud KMS HSM tier | Managed HSM | OCI Cloud-HSM | partial | partial | partial | DFC fragments | **HSM-backed** | HSM-backed |
| D3 | yes (paid) | FIPS 140-2 L3 (KMS); 140-3 L3 (CloudHSM) | FIPS 140-2 L3 | FIPS 140-3 L3 (Managed HSM) | FIPS 140-3 L3 | n/a | n/a | n/a | partial | **FIPS 140-3 L3** | L3 |
| D4 | yes | versioning | versioning | versioning | versioning | versioning | versioning | versioning | versioning | **yes (KV v2 via OpenBao)** | yes |
| D5 | yes (Transit) | KMS encrypt/decrypt | KMS encrypt/decrypt | encrypt/decrypt | encrypt/decrypt | no | no | partial | DFC | **yes (Transit via OpenBao)** | yes |
| D6 | yes (PKI) | ACM-Private CA | CAS | yes | yes | no | no | no | no | **yes (PKI via OpenBao)** | yes |
| D7 | yes (Vault Ent) | partial | partial | yes | partial | no | no | no | partial | **yes (via OpenBao)** | yes |
| D8 | yes (Vault Ent namespaces) | per-account | per-project | per-vault | per-vault | per-vault | per-project | partial | yes | **per-tenant namespace** | per-tenant |
| D9 | yes (policies) | IAM | IAM | RBAC | IAM | per-service tokens | per-service tokens | per-app | yes | **per-µservice scope** | per-µservice |
| D10 | yes | yes | partial (manual) | partial (manual) | yes | partial | yes (webhook) | partial | yes | **yes (scheduler)** | yes |
| D11 | partial | n/a | n/a | n/a | n/a | n/a | n/a | n/a | n/a | **yes (cascade DAG)** | yes |
| D12 | partial (lease revoke) | n/a | n/a | n/a | n/a | partial | n/a | n/a | partial | **yes (SSE push)** | yes |
| D13 | yes (Vault Ent) | yes | yes | yes | yes | n/a | n/a | n/a | yes | **yes** | yes |
| D14 | partial | yes (CMK wraps) | yes | yes | yes | n/a | n/a | n/a | partial | **yes (KEK-of-KEKs)** | yes |
| D15 | partial | partial (vendor) | partial (vendor) | yes (Attestation) | partial (vendor) | n/a | n/a | n/a | partial | **daily verified** | daily |
| D16 | yes (audit devices) | CloudTrail | Cloud Audit Logs | Azure Monitor | OCI Audit | yes | yes | yes | yes | **yes (audit-chain)** | yes |
| D17 | no (linear log) | no | no | no | no | no | no | no | partial (DFC) | **Merkle + Ed25519** | Merkle + Ed25519 |
| D18 | per-region | per-region | per-region | per-region | per-region | per-region | no | partial | global | **per-pack** | per-pack |
| D19 | yes (multi-lang) | yes | yes | yes | yes | yes | yes | yes | yes | **Rust + TS + Python (M01)** | + Go + Java subsequent-to-M01-completion |
| D20 | partial | partial | partial | partial | partial | partial | partial | partial | partial | **enforced** | enforced |
| D21 | optional | optional | optional | optional | optional | optional | optional | optional | optional | **mandatory ≤60s** | mandatory ≤60s |
| D22 | not enforced | not enforced | not enforced | not enforced | not enforced | not enforced | not enforced | not enforced | not enforced | **LEAN-A11 + scrubbed logs** | enforced |
| D23 | no | no | no | no | no | no | no | no | no | **LEAN-A11 BLOCKER** | BLOCKER |
| D24 | yes (SPIFFE via integration) | IAM roles | service-account | managed identity | OCI workload-id | partial | partial | partial | partial | **SPIFFE + mTLS** | SPIFFE |
| D25 | yes | IAM + MFA | IAM + MFA | RBAC + MFA | IAM + MFA | yes | yes | yes | yes | **OIDC + MFA + JIT** | OIDC + MFA + JIT |
| D26 | yes (Sentinel) | partial (IAM conditions) | partial | partial | partial | n/a | n/a | n/a | yes | **OpenBao Sentinel 4-eye** | 4-eye |
| D27 | yes (Raft) | managed | managed | managed | managed | managed | managed | yes | yes | **5-node Raft per pack** | Raft |
| D28 | yes (PR) | yes | yes | yes | yes | n/a | n/a | partial | yes | **DR-pair per residency rules** | DR-pair |
| D29 | partial | partial | partial | partial | partial | partial | partial | partial | partial | **cascade DEK destruction** | yes |
| D30 | OSS-BSL since 2023 (downgrade) | proprietary | proprietary | proprietary | proprietary | proprietary | proprietary | OSS | proprietary | **Apache-2.0 (OpenBao)** | Apache-2.0 |

## Key Differentiators (oyatie advantages)

1. **D11 Cascade rotation** — only HashiCorp partial; oyatie's cascade DAG is automatic, with topological-order rotation and revocation propagation.
2. **D17 Audit-chain Merkle + Ed25519** — none of the competitors offer cryptographic non-repudiation; oyatie inherits Bominal ADR-0028 audit-chain.
3. **D20-D23 SDK-enforced safety contract** — competitors offer SDKs but don't enforce `Secret<T>` wrapper, cache TTL ceiling, no-log, or mechanical raw-secret prevention; oyatie enforces all four at SDK + CI layers.
4. **D30 OSS substrate** — OpenBao's BSL re-license (2023) is the rationale for OpenBao fork; oyatie commits to Apache-2.0 OSS-only forever.
5. **D18 Per-pack audit residency** — each pack has its own audit-chain instance; cross-pack audit replication forbidden. Competitors typically have per-region audit but not per-pack regulatory binding.
6. **D26 OpenBao Sentinel 4-eye break-glass** — strongest in industry (only HashiCorp and Akeyless approach this).

## Gaps to Close (oyatie behind)

1. **D2/D5/D6 — Transit + PKI maturity**: OpenBao's Transit + PKI engines have ~10 years of features; oyatie inherits via OpenBao but operational maturity is still nascent. Mitigation: pin to OpenBao 2.x LTS; track feature parity quarterly.
2. **D19 — Go + Java SDK**: scheduled-for-distinct-tracked-work to subsequent-to-M01-completion. Mitigation: schedule for M02/M03.
3. **D7 — KMIP**: OpenBao inherits HashiCorp's partial KMIP support; not native. Mitigation: gateway pattern via separate KMIP proxy if needed.

## Anti-Differentiators (deliberately not pursued)

- **DFC (Distributed Fragments Cryptography)** (Akeyless): adds zero-knowledge but complexity outweighs benefit for our use cases; HSM model preferred.
- **Global SaaS**: deliberately not — per-pack residency invariant.
- **Per-environment secret config UX** (Doppler, Infisical): the SecretReference URI model carries the equivalent (path-based scoping); avoids divergent metaphor.

## Verification

```bash
cargo run -p oya-dev-cli -- gate validate competitor-parity-coverage --microservice cloud-secrets
```

Quarterly review:
- Update competitor versions / new entrants.
- Re-assess gaps + subsequent-to-M01-completion timeline.
- Sign-off by gtm-customer-success + axis-cloud-secrets leads.

## References

- HashiCorp Vault Enterprise docs (canonical for D1-D17, D26)
- AWS Secrets Manager + KMS + CloudHSM docs
- GCP Secret Manager + Cloud KMS docs
- Azure Key Vault + Managed HSM docs
- OCI Vault + Cloud-HSM docs
- 1Password docs
- Doppler docs
- Infisical OSS docs
- Akeyless docs
- OpenBao project (LF Edge)
- `secrets/PRD.md` §"Competitive Benchmark"
