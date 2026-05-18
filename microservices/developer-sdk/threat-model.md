---
doc_class: ThreatModel
title: "Threat Model"
microservice: developer-sdk
status: Accepted
owner_team: axis-ecosystem
date: 2026-05-18
related_adrs: [ADR-0213, ADR-0131]
doc_status: published
---

# Threat Model


## Methodology

STRIDE per asset; SLSA L3 attestation per artifact; Cedar policy gating per declared capability; Wasmtime sandbox isolation per ADR-0147.

## Assets

| Asset | Owner | Sensitivity |
|---|---|---|
| Plugin manifest | publisher | INTERNAL_ONLY (vetting), PUBLIC (post-publish) |
| Plugin Wasmtime artifact | publisher | INTERNAL_ONLY |
| Tenant grant set | tenant operator | INTERNAL_ONLY (tenant-scoped) |
| Per-installation Cedar policy | governance µservice | INTERNAL_ONLY |
| Per-plugin audit trail | audit-chain µservice | AUDIT_RECORD |
| Subscription billing aggregate | finops-portal | FINANCIAL |
| Developer signing keys | OpenBao | CRYPTO_KEY |
| Developer bank account | tenancy (developer pack) | PII_FINANCIAL |

## STRIDE per asset (excerpt)

### T-I-01 — Cross-tenant plugin install (Information Disclosure)
**Threat**: Tenant A's installation surfaces in Tenant B's plugin list.
**Mitigation**: Cedar policy tenant-scope enforced at every read; Postgres row-level scoping by tenant_id; CI lane `tenant-isolation` (BLOCKER).

### T-T-02 — Tampered plugin artifact (Tampering)
**Threat**: Plugin artifact swapped between submission and execution.
**Mitigation**: Cosign signature verification at install + every fetch; hash pinned in installation record.

### T-E-03 — Wasmtime sandbox escape (Elevation of Privilege)
**Threat**: Plugin escapes Wasmtime sandbox; touches host or other tenant data.
**Mitigation**: ADR-0147 + ADR-0200 baseline; seccomp profile; syscall trace; isolation validator in vetting pipeline; runtime escape → kill-switch + revoke.

### T-D-04 — Plugin DoS via rate-limit bypass (DoS)
**Threat**: Plugin saturates tenant compute by ignoring declared rate limit.
**Mitigation**: Per-installation Valkey token bucket; bypass attempt audit-logged; SLO oya-plugin-app-store-per-plugin-rate-limit-correctness BLOCKER day 1.

### T-R-05 — Vetting decision repudiation (Repudiation)
**Threat**: Vetting reviewer denies having approved a malicious plugin.
**Mitigation**: Every vetting transition audit-chain sealed; reviewer Ed25519-signs decision.

### T-S-06 — Spoofed plugin publisher (Spoofing)
**Threat**: Adversary publishes plugin claiming to be a trusted developer.
**Mitigation**: KYC + signing-key chain-of-trust from developer-sdk; only signing keys issued to KYC-verified developers can submit.

## Compliance threats (per pack)

- pack-kr — PIPA cross-border violation if KR-tenant data flows to non-KR plugin → Cedar denial.
- pack-eu — GDPR Article 28 absence on developer onboarding → onboarding blocked until DPA accepted.
- pack-us-healthcare — PHI touched by non-BAA plugin → vetting reject.

## Out-of-scope threats

- Tenant-vs-tenant attacks at the network layer (owned by Cilium NetworkPolicy + ambient ztunnel).
- Substrate compromise (OpenBao, Postgres) — owned by cloud-secrets / cloud-iac µservices.

