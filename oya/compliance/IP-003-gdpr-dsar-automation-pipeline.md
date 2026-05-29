---
microservice: compliance
ip: IP-003
title: GDPR DSAR automation pipeline (export + delete + rectify + 5-day target SLA)
status: Drafting
authority_tier: 3
owner: axis-compliance
co_owners: [axis-security]
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0209]
---

# IP-003 — GDPR DSAR automation pipeline

## Purpose

Stand up the Data Subject Access Request (DSAR) lifecycle per GDPR Art. 12-22. The pipeline accepts a subject's request, walks the Ontology projection across every µservice that holds the subject's data (per ADR-0145), assembles the export / executes the cascade / applies the rectification, emits the `dsar-completion-record` artifact, and tracks the 30-day statutory SLA + 5-day internal target.

## Acceptance criteria

1. Endpoints live: `POST /api/v1/dsar/export`, `POST /api/v1/dsar/delete`, `POST /api/v1/dsar/rectify`, `GET /api/v1/dsar/{id}`.
2. Subject identity verified via Zitadel passwordless flow.
3. Ontology projection traversal (per ADR-0145) carries `tenant_id` at every hop; kernel rejects cross-tenant assembly.
4. Export output is encrypted zip (AES-256-GCM); symmetric key delivered out-of-band via subject's verified contact.
5. DSAR completion record artifact emitted with audit-chain seal hex.
6. Status endpoint reports elapsed-days + statutory SLA + internal target.
7. Backlog-protection circuit-breaker: reject new DSAR intake when backlog > 100.
8. Per-tenant rate limit: 10 DSARs / tenant / day.
9. ≥ 8 integration tests: subject-verify, export end-to-end, delete cascade, rectify field, cross-tenant reject, rate-limit reject, backlog-circuit-break, SLA tracking.

## Lifecycle

```
                                                ┌─────────────────────────┐
[Subject]→passwordless OIDC→[POST /dsar/X]──────►│ Open DSAR request       │──[EVT-DSAR-REQUEST-OPENED]
                                                │ Validate tenant_id      │
                                                │ Validate subject id     │
                                                └────────────┬────────────┘
                                                             ▼
                                                ┌─────────────────────────┐
                                                │ Ontology projection     │── per-µservice fan-out
                                                │   walk per tenant_id    │── outbox subscriber
                                                └────────────┬────────────┘
                                                             ▼
                                  ┌──────────────────────────┴──────────────────────────┐
                                  ▼                          ▼                          ▼
                            [EXPORT path]              [DELETE path]             [RECTIFY path]
                            zip + encrypt              cascade                    field update
                            deliver key                emit tombstones            emit revised
                                                                                    record
                                  └──────────────────────────┬──────────────────────────┘
                                                             ▼
                                                ┌─────────────────────────┐
                                                │ Emit dsar-completion-   │── [EVT-DSAR-REQUEST-CLOSED]
                                                │   record artifact       │── audit-chain seal hex
                                                └─────────────────────────┘
```

## Cross-tenant isolation invariant — DSAR-specific

DSAR is the riskiest entry point for cross-tenant leak. Belt-and-suspenders:

1. **API guard:** REST handler asserts `request.principal.tenant_id == request.subject.tenant_id` (rejects 403 otherwise).
2. **Domain guard:** Ontology walk takes `tenant_id` as a typed parameter; downstream `oya-ontology-domain` filters by `tenant_id` at every projection step.
3. **Kernel guard:** `oya-shared-compliance-evidence-kernel::coverage_gaps` filters artifacts by `tenant_id` (already verified in kernel tests).
4. **Cedar guard:** `dsar:exec` capability requires `principal.tenant_id == resource.tenant_id`.
5. **Integration test:** `tests/cross_tenant_dsar.rs` builds two tenants A + B with identical subject pseudonym `subj_42`; opens DSAR for tenant A; asserts response contains only tenant A's data + zero tenant B records.

## Export format

JSON-LD per GDPR Art. 20 (right to data portability). Encrypted zip:

```json
{
  "@context": "https://oya.dev/dsar/v1",
  "subject_pseudonym": "subj_42",
  "tenant_id": "tenant_a",
  "exported_at": "2026-05-18T12:00:00Z",
  "records": [
    {"@type": "WorkflowRun", "id": "run_123", "data": {...}},
    {"@type": "AuditEvent", "id": "evt_abc", "data": {...}}
  ],
  "audit_chain_seal_hex": "a1b2c3..."
}
```

## SLA tracker

A scheduler job runs every 6 hours:

- Lists open DSARs.
- Computes elapsed-days.
- Pages on-call at 25 days (statutory: 30; target buffer 5).
- Flags Sev-2 at 28 days; Sev-1 at 30 days (statutory breach risk).
- Updates Grafana dashboard `dashboards/dsar-sla.json`.

## Risk + mitigation

- **Risk:** Ontology projection misses a µservice → incomplete export. **Mitigation:** `oya-check-ontology-projection-coverage` advisory gate validates per-µservice DSAR fan-in.
- **Risk:** subject identity spoofing. **Mitigation:** Zitadel passwordless + email-link verification + audit-chain seal on EVT-DSAR-REQUEST-OPENED.
- **Risk:** cascade-delete races with running workflows. **Mitigation:** delete sets tombstone first; workflow runner respects tombstone before next step.

## Acceptance evidence

`evidence/ip-003-dsar-pipeline-acceptance.json`.

## Cross-references

- ADR-0145 — Ontology projection + audit-chain seal.
- ADR-0209 — substrate authority.
- IP-001 — collector bootstrap.
- IP-008 — PII scrubber (DSAR export redaction).
- IP-011 — cross-µservice evidence fan-in (Ontology walk).
