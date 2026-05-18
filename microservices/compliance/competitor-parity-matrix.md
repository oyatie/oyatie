---
microservice: compliance
doc: CompetitorParityMatrix
status: Drafting
authority_tier: 3
owner: axis-compliance
date: 2026-05-18
related_adrs: [ADR-0173, ADR-0209]
---

# Compliance — Competitor Parity Matrix

## Compared offerings

| Vendor | Tier | Pricing model |
|---|---|---|
| Drata | SaaS continuous-evidence | $25k-$100k/yr + per-employee |
| Vanta | SaaS continuous-evidence | $20k-$80k/yr + per-employee |
| Tugboat Logic (now part of OneTrust) | SaaS GRC | $30k-$100k/yr |
| AuditBoard | SaaS GRC | $50k-$200k/yr |
| ServiceNow GRC | Enterprise GRC | $200k+/yr |

## Parity matrix (oyatie compliance µservice vs commercial)

| Feature | Drata | Vanta | Tugboat | AuditBoard | ServiceNow GRC | **Oyatie compliance** |
|---|---|---|---|---|---|---|
| Continuous evidence collection | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (IP-001..IP-011) |
| Pre-mapped SOC 2 controls | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (IP-002 policy/soc2-control-mapping.json) |
| Pre-mapped ISO 27001 | ✓ | ✓ | ✓ | ✓ | ✓ | (Phase 1.5; iso-27001-annex-a-coverage.json) |
| Pre-mapped HIPAA | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (compliance.md + IP-004) |
| Pre-mapped PCI-DSS | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (substrate ready; payments-µservice gated) |
| GDPR DSAR automation | partial | partial | partial | ✗ | partial | ✓ (IP-003) |
| Auditor read-only portal | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (IP-007, Backstage plugin) |
| Tamper-evident audit chain | ✗ (opaque vendor hash) | ✗ | ✗ | ✗ | ✗ | ✓ (cosign keyless OIDC; verifiable) |
| Multi-cloud agent collectors | ✓ (AWS / Azure / GCP) | ✓ | ✓ | ✓ | ✓ | ✓ (cell-aware per ADR-0153) |
| Multi-region evidence storage | ✗ (SaaS US-centric) | ✗ | ✗ | ✗ | partial | ✓ (per-pack overlay per ADR-0179) |
| Self-hosted evidence storage | ✗ | ✗ | ✗ | ✗ | partial | ✓ (SeaweedFS on operator cluster) |
| Sovereignty preserved (evidence stays in operator-controlled cluster) | ✗ | ✗ | ✗ | ✗ | depends | ✓ |
| Per-tenant isolation | partial | partial | ✗ | ✗ | ✓ | ✓ (kernel invariant) |
| Per-pack regulatory overlay (PIPA / PDPL / APPI) | ✗ | ✗ | ✗ | ✗ | partial | ✓ (IP-015) |
| OpenAPI + AsyncAPI contract | ✗ | partial | ✗ | ✗ | partial | ✓ |
| Open source kernel | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ (oya-shared-compliance-evidence-kernel) |
| Pricing | $25k-$200k/yr | $20k-$80k/yr | $30k-$100k/yr | $50k-$200k/yr | $200k+/yr | **engineering cost only** |

## Differentiation summary

- **Direct auditor relationship** — no SaaS proxy.
- **Verifiable tamper evidence** — cosign keyless OIDC + Sigstore Rekor; auditor verifies independently.
- **Sovereignty** — evidence never leaves operator-controlled cluster; matches EU / KR / UAE sovereignty packs.
- **Per-tenant kernel-level isolation** — not just per-tenant filtering; kernel-level invariant.
- **Per-pack regulatory overlay** — KR PIPA + UAE PDPL + JP APPI built-in; commercial vendors lag on non-US jurisdictions.
- **Open source kernel** — `oya-shared-compliance-evidence-kernel` is auditable.
- **Cost stable** — no per-employee fee; storage cost scales linearly with evidence volume.

## What we DON'T do (intentional)

- We don't sell GRC risk-register UI; that's a separate (commercial-tier) capability.
- We don't provide automated policy authoring; legal team owns policy text.
- We don't claim to replace human auditors; we provide the substrate they audit against.

## References

- ADR-0173 — vendor lock-in avoidance.
- ADR-0209 — substrate authority (in-house build doctrine).
