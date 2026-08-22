# compliance-evidence-collector

Compliance evidence collector tier per ADR-0209. In-house pipeline replacing Drata / Vanta /
Tugboat Logic / AuditBoard / ServiceNow GRC.

## Frameworks

- **SOC 2 Type II** — AICPA Trust Services Criteria.
- **GDPR** — Art. 12 DSAR automation.
- **HIPAA** — minimum-necessary access logs + BAA inventory.
- **PCI-DSS 4.0** — when payments in scope.

## Collector inventory

Per `values.yaml`:

- `ciArtifactHash` (every 15min poll)
- `deployReceipt` (event-driven from ADR-0181)
- `accessReviewSnapshot` (weekly)
- `backupRestoreDrillReceipt` (quarterly)
- `vulnScanReport` (event-driven from Trivy)
- `penTestReport` (manual upload, annual)
- `dsarCompletionRecord` (event-driven from dsr-usecase)
- `baaInventoryEntry` (quarterly)
- `minimumNecessaryAccessLog` (continuous)

## Storage

SeaweedFS filer (per ADR-0145); per-framework bucket; audit-chain seal hex per artifact.

## DSAR

- Statutory SLA: 30 days (GDPR Art. 12).
- Target: 5 days.
- Endpoint: `/api/v1/dsar/{export|delete|rectify}`.
- Auth: Zitadel passwordless.

## Cross-tenant isolation

Kernel guard rejects DSAR assembly when subject `tenant_id` ≠ request `tenant_id`.

## Auditor portal

first-party portal module (per ADR-0394). Per-engagement auditor identity; access expires on engagement close.

## Cross-references

- ADR-0209 — compliance evidence automation.
- `docs/standards/compliance-evidence-automation.md` — canonical standard.
