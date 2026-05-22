# IP-009: Critical Result Escalation

Status: Reconciled
Date: 2026-05-21

## Goal

Close the loop on critical lab/pathology results.

## Scope

- Critical lab value detection.
- Pathology urgent finding escalation.
- Notification routing, acknowledgement, timeout escalation, and evidence retention.

## Acceptance

- Critical-result notifications meet the OpenSLO target.
- Acknowledgement evidence includes principal, method, timestamp, and tenant/cell context.
- Imaging critical-result workflows are owned by the imaging microservice.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/diagnostics/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/diagnostics/implementation-plans/IP-009-critical-result-escalation.md:18` - - Critical-result notifications meet the OpenSLO target..
