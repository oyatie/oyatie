# Healthcare Integration Runbook: Consent Sync Lag

Service: healthcare-integration  
Surface: local operational primitive suite  
Primary SLO: consent-sync-freshness  
Domain focus: hl7, fhir, hipaa, phi

## Trigger
- Alert `healthcare-integration-consent-sync-freshness` burns above the 2x multi-window threshold.
- Operator report names ClinicalExchange state drift for tenant-scoped HL7 ingestion, FHIR exchange, HIPAA access controls, PHI delivery, consent synchronization, and audit completeness operations.
- Audit chain shows denied or missing event class for `hl7.message.ingested`.

## Confirm
1. Query `sum(rate(oya_healthcare_integration_consent_sync_freshness_total[5m])) by (tenant_id, cell_tier)` and identify the affected tenant and cell.
2. Compare `good_total` against `total` for the same metric and confirm the burn is not a dashboard-only gap.
3. Inspect the latest policy decision for action `hl7.message.ingest` and data class `hl7_message`.
4. Verify the latest domain event on `healthcare-integration.local-ops.v1` carries `audit_event_id` and tenant scope.

## Mitigate
1. Freeze new high-volume writes for the affected tenant using the local Cedar action `fhir.bundle.exchange` when burn exceeds 4x.
2. Shift traffic away from the unhealthy cell for the `hl7-ingestor` workload.
3. Replay only idempotent events with matching `tenant_id`, `resource_id`, and `audit_event_id`.
4. Re-run the policy check endpoint before reopening operator writes.

## Recover
- Restore normal admission when consent-sync-freshness is below 1x burn for two consecutive 30 minute windows.
- Backfill missing audit evidence before resolving the incident.
- Attach dashboard snapshot `healthcare-integration-local-consent-sync-freshness` and policy file name to the ticket.

## Escalate
Escalate to the service owner when regulated data class `hl7_message` is affected for more than 15 minutes or when breakglass was used.
