# IP-009 Data Pipeline credential sidecar binding

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-009-credential-sidecar-binding.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Bind connector credentials to sidecar-issued, tenant-scoped leases.
- Prevent connector workers from owning durable source secrets.
- Keep replay, drift sampling, and lineage repair from expanding secret access.
- Tie every credential use to Cedar decision, DealSet state, and audit event.
- Support source APIs, warehouse targets, signing keys, and sample export keys.
- Treat Fivetran and Airbyte Cloud connector setup as usability pressure.
- Treat Hevo and Stitch lightweight secret setup as fast-onboarding pressure.
- Treat Matillion and Talend Cloud as transform credential pressure.
- Treat Informatica IICS as governed credential rotation pressure.
- Treat Estuary Flow as streaming capture lease pressure.

## Local references
- `microservices/data-pipeline/iac/local-secret-binding.yaml` binds local sidecar secret shape.
- `microservices/data-pipeline/iac/secret-bindings.yaml` binds companion secret shape.
- `microservices/data-pipeline/iac/local-openbao-policy.hcl` constrains OpenBao access.
- `microservices/data-pipeline/iac/openbao-policy.yaml` constrains shared OpenBao policy.
- `microservices/data-pipeline/runbooks/local-source-credential-expiry.md` defines expiry response.
- `microservices/data-pipeline/runbooks/secret-rotation-failure.md` defines rotation response.
- `microservices/data-pipeline/policy/data-residency.md` constrains secret cell locality.
- `microservices/data-pipeline/policies/local-ingest-source-scope.cedar` gates source use.
- `microservices/data-pipeline/capabilities/connector-run-start.yaml` requires tenant fields.
- `microservices/data-pipeline/threat-model.md` records credential threats.

## Credential classes
- Source API credential is leased per connector run.
- Source CDC credential is leased per capture worker.
- Destination warehouse credential is leased per landing worker.
- Transform signing credential is leased per transform run.
- Lineage export credential is leased per graph export.
- Dead-letter sample credential is leased per custody review.
- Replay credential is leased per replay window.
- Audit export credential is leased per evidence packet.
- DealSet license credential is leased per connector license check.
- Pack overlay key is leased per regulated export.
- Provider webhook secret is leased per callback validation.
- Operator break-glass credential is out of normal path.

## Secret reference model
- Secret references use `${openbao:secret/<tenant_id>/data-pipeline/<credential>}` shape.
- Secret references include connector id.
- Secret references include source object id where applicable.
- Secret references include home cell.
- Secret references include data class.
- Secret references include lease purpose.
- Secret references include expiry.
- Secret references include rotation generation.
- Secret references include DealSet license id when applicable.
- Secret references include pack overlay id when regulated.
- Secret references exclude raw secret material.
- Secret references are audit-safe.

## Command deltas
- Connector run start requests a source credential lease.
- Schema drift sample requests a sample custody lease.
- Transform approval requests no source credential.
- Transform run requests transform and destination leases.
- Lineage reconciliation requests graph export lease only when exporting.
- Dead-letter replay requests source or destination lease only after approval.
- Watermark advance requests no source credential unless provider verification is required.
- Audit export requests evidence encryption lease.
- DealSet connector check requests marketplace credential lease.
- Credential rotation command requires tenant and connector scope.
- Credential revoke command requires incident or rotation reason.
- Credential lease renewal requires active worker lease.

## Event deltas
- `credential.lease_requested` records requested secret class.
- `credential.lease_granted` records sidecar lease id.
- `credential.lease_denied` records Cedar denial.
- `credential.lease_expired` records normal expiry.
- `credential.lease_revoked` records revocation reason.
- `credential.rotation_started` records rotation generation.
- `credential.rotation_completed` records new generation.
- `credential.rotation_failed` records runbook link.
- `credential.sidecar_unavailable` records degraded state.
- `credential.break_glass_denied` records normal-path refusal.
- Events include secret reference hash.
- Events never include raw secret values.

## Proto deltas
- `SecretLeaseRequest` includes tenant scope.
- `SecretLeaseRequest` includes credential class.
- `SecretLeaseRequest` includes connector id.
- `SecretLeaseRequest` includes lease purpose.
- `SecretLeaseRequest` includes requested ttl.
- `SecretLeaseRequest` includes Cedar decision receipt.
- `SecretLeaseResponse` includes lease id.
- `SecretLeaseResponse` includes expiry.
- `SecretLeaseResponse` includes sidecar id.
- `SecretLeaseResponse` includes rotation generation.
- `SecretLeaseRevokeRequest` includes revocation reason.
- `SecretLeaseRenewRequest` includes active worker lease.

## Cedar facts
- `credential_class` is a policy fact.
- `secret_reference_hash` is a policy fact.
- `connector_id` is a policy fact.
- `source_object_id` is a policy fact.
- `lease_purpose` is a policy fact.
- `requested_ttl` is a policy fact.
- `rotation_generation` is a policy fact.
- `dealset_license_state` is a policy fact.
- `pack_overlay_state` is a policy fact.
- `worker_lease_state` is a policy fact.
- `sidecar_identity` is a policy fact.
- `home_cell` is a policy fact.

## Workflow decisions
- Connector workflow requests lease after Cedar source permit.
- Lease ttl is shorter than connector worker lease.
- Replay workflow revalidates credential lease after approval.
- Drift sampling uses separate sample lease.
- Transform approval does not grant credential access.
- Transform worker receives credential through sidecar only.
- Lineage reconciliation does not expose source credentials.
- Audit export leases encryption key only after auditor scope.
- Rotation workflow drains active leases before generation switch.
- Revocation workflow freezes affected connector runs.
- Sidecar outage fails closed for source mutation.
- Secret metadata can be logged; secret material cannot.

## Failure cases
- Sidecar unavailable blocks connector worker start.
- Lease expired blocks replay continuation.
- Lease tenant mismatch blocks secret retrieval.
- Lease connector mismatch blocks source call.
- Lease purpose mismatch blocks transform use.
- Lease ttl too long is denied.
- Rotation generation mismatch opens rotation failure runbook.
- Revoked lease stops worker at next checkpoint.
- Raw secret in logs is a security incident.
- DealSet stale state blocks licensed connector secret.
- Pack overlay mismatch blocks regulated export key.
- Audit-chain outage blocks lease grant for high-risk operations.

## Replay cases
- Replay cannot reuse original source credential lease.
- Replay requests a fresh lease after custody approval.
- Replay binds lease to replay window id.
- Replay binds lease to dead-letter case id.
- Replay revokes lease after cursor outcome.
- Replay failure preserves lease id in evidence.
- Replay rollback does not resurrect expired lease.
- Replay of credential-denied item remains blocked.
- Replay of rotation-failed item waits for rotation completion.
- Replay of provider-rate-limit item may delay lease request.
- Replay evidence records original and current secret generation.
- Replay never stores raw secret in dead-letter custody.

## Evidence fields
- `tenant_id` is mandatory.
- `connector_id` is mandatory.
- `credential_class` is mandatory.
- `secret_reference_hash` is mandatory.
- `lease_id` is mandatory after grant.
- `sidecar_id` is mandatory after grant.
- `requested_ttl` is mandatory.
- `granted_ttl` is mandatory.
- `expires_at` is mandatory.
- `rotation_generation` is mandatory.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `worker_lease_id` is mandatory when worker-bound.
- `dealset_decision_id` is mandatory when licensed.
- `pack_overlay_id` is mandatory when regulated.
- `revocation_reason` is mandatory on revoke.

## SLOs
- Credential lease latency is tracked separately from provider latency.
- Credential expiry incidents link local-source-credential-expiry runbook.
- Rotation failure incidents link secret-rotation-failure runbook.
- Lease denial spikes feed policy decision dashboard.
- Sidecar unavailable feeds operating-bar overview.
- Revoked active lease count feeds incident dashboard.
- Expired replay lease count feeds replay freshness risk.
- Transform lease wait contributes to transform latency.
- Connector lease wait contributes to ingest freshness.
- Audit export lease wait contributes to audit emission lag.
- Secret access never adds raw tenant id metric labels.
- Lease ttl distribution is monitored for drift.

## Test cases
- Lease request rejects missing tenant.
- Lease request rejects missing credential class.
- Lease request rejects stale Cedar decision.
- Lease request rejects DealSet stale state.
- Lease request rejects pack mismatch.
- Worker cannot read secret after lease expiry.
- Replay cannot reuse old lease.
- Transform approval does not create source lease.
- Rotation revokes old generation after drain.
- Raw secret never appears in event payload.
- Sidecar outage fails closed.
- Revoke command records reason.

## Rollback
- Roll back secret binding by generation.
- Preserve lease grant audit events.
- Revoke leases from rejected generation.
- Restart connector workers only after new lease.
- Freeze replay windows using rejected generation.
- Re-run transform workers only after new lease.
- Keep DealSet decisions immutable.
- Keep pack overlay decisions immutable.
- Emit credential binding rollback event.
- Link rotation rollback to secret-rotation-failure runbook.
- Do not delete secret reference hashes.
- Verify sidecar policy after rollback.

## Acceptance criteria
- No worker receives raw durable secret material.
- Every credential lease is tenant-scoped.
- Every credential lease has Cedar receipt.
- Every licensed connector lease has DealSet decision.
- Every regulated export lease has pack overlay.
- Every replay lease is fresh after custody approval.
- Every rotation has generation evidence.
- Every revocation has reason evidence.
- Every benchmark reference is comparative.
- Credential sidecar binding remains Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/iac/local-secret-binding.yaml`
- `microservices/data-pipeline/iac/secret-bindings.yaml`
- `microservices/data-pipeline/iac/local-openbao-policy.hcl`
- `microservices/data-pipeline/iac/openbao-policy.yaml`
- `microservices/data-pipeline/runbooks/local-source-credential-expiry.md`
- `microservices/data-pipeline/runbooks/secret-rotation-failure.md`
- `microservices/data-pipeline/policy/data-residency.md`
- `microservices/data-pipeline/policies/local-ingest-source-scope.cedar`
- `microservices/data-pipeline/capabilities/connector-run-start.yaml`
- `microservices/data-pipeline/threat-model.md`
- `ADR-0105`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-009-credential-sidecar-binding.md:176` - ## SLOs.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-009-credential-sidecar-binding.md:186` - - Audit export lease wait contributes to audit emission lag..
