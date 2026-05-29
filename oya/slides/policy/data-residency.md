---
doc_class: PolicyDoc
template_id: TPL-POLICY-DOC
microservice: slides
status: Accepted
date: 2026-05-17
owner_team: axis-workspace + ops-security + dpo-office
related_artifacts:
  - microservices/slides/multi-region.md
  - microservices/slides/dpia.md
  - microservices/slides/compliance.md
  - microservices/slides/iac/kustomize/overlays/
doc_status: published
---

# Data residency — slides µservice

This policy declares per-pack residency rules for deck content, audit ledgers, AI invocations, broadcast metadata, and chart-live-link refresh.

## Per-pack residency requirements

| Pack | Primary region | Secondary | Audit retention | Special-category | Tenant-tunable |
|---|---|---|---|---|---|
| kr | ap-seoul-1 | ap-tokyo-1 | 7y (전자문서법) | KR-FSS fintech overlay | up only within pack max |
| eu | eu-frankfurt-1 | eu-paris-1 | 7y default (GDPR Art. 30 4y minimum; longer for legal-obligation) | Art. 9 consent flag | up only |
| us | us-ashburn-1 | us-phoenix-1 | 7y default | CCPA opt-out | up only |
| us-healthcare | us-ashburn-1 (HIPAA) | us-phoenix-1 (HIPAA) | 6y (HIPAA §164.530(j)) | PHI redaction required for AI | up only |
| jp | ap-tokyo-1 | ap-osaka-1 | 7y | APPI cross-border consent | up only |
| sg | ap-singapore-1 | ap-melbourne-1 | 7y | PDPA consent | up only |
| au | ap-sydney-1 | ap-melbourne-1 | 7y | APP 8 cross-border | up only |
| in | ap-mumbai-1 | ap-hyderabad-1 | 7y | DPDPA consent | up only |
| br | sa-saopaulo-1 | sa-vinhedo-1 | 7y | LGPD consent | up only |
| ae | me-dubai-1 | me-jeddah-1 | 7y | UAE PDPL controller-processor agreement | up only |
| ksa | me-jeddah-1 | me-dubai-1 (GCC permitted) | 7y | KSA PDPL controller-processor agreement | up only |

## Cross-pack rules

- **Forbidden**: cross-pack collab; cross-pack chart-live-link; cross-pack broadcast viewer; cross-pack share-link redemption.
- **Allowed**: per-pack CDN edge replication of WASM bundles + signed theme/template galleries (no tenant deck content on CDN).
- **Allowed**: cross-pack public-read for unauthenticated reader bearing pack-pinned share-link signature, IF deck pack matches share-link pack.

## Region enforcement

- Per-pack VCN + namespace isolation.
- Per-pack overlay enforces ContentLocation header on all REST + WS responses.
- Cross-pack op admission refused at gate; audit row emitted.

## AI residency

- T0/T1/T2 AI invocations forward to foundry-runtime within tenant pack only.
- Cross-pack AI inference forbidden (foundry-runtime SDK enforces).
- Per-pack `ai_policy` overlay declares: T2 allowed/refused; PHI redaction required; provenance watermark required; Annex III override required.

## Broadcast residency

- LiveKit SFU nodes pack-pinned (via messenger).
- Broadcast viewers must be in same pack as broadcast deck OR bear pack-pinned public-share-link.
- Cross-pack broadcast viewer forbidden by default; per-pack override possible (e.g., eu pack with publish-to-public-link permit).

## Audit ledger residency

- Per-pack audit-chain ledger.
- Cross-pack audit emission forbidden; replication via cross-region (within pack) only.

## Retention

- Per-pack retention enforced; un-restorable past retention.
- Tenant may tune retention UP within pack max (e.g., us-healthcare may set 10y but not 5y).
- Cryptographic delete on retention expiry — Ed25519 seal of delete-event emitted.

## Tenant transparency

- Per-pack data-flow notice published in editor UI.
- Per-pack tenant T&C link.
- DPO sign-off required per pack.

## Verification

- `oya gate validate per-pack-residency --microservice slides` — verifies layout enforcement.
- Per-pack overlay test under `tests/e2e/per-pack-residency.rs`.
- Cross-pack admission refusal test under `tests/security/cross-pack-refusal.rs`.

## References

- `multi-region.md`.
- `dpia.md`.
- ADR-0117 per-pack residency.
- GDPR Art. 44.
- KR PIPA + 전자문서법.
- HIPAA §164.530(j).
- All 11 per-pack regulations.
