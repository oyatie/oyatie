# IP-014 Data Pipeline marketplace DealSet settlement

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-014-marketplace-dealset-settlement.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Bind licensed connector use to marketplace DealSet settlement.
- Prevent connector runs from bypassing commercial scope.
- Tie transform, lineage, replay, and audit export to the connector license context when derived from licensed sources.
- Keep DealSet settlement separate from policy permit, but required before licensed connector side effects.
- Treat Fivetran and Airbyte Cloud connector catalogs as commercial pressure.
- Treat Hevo and Stitch low-cost connector setup as pricing pressure.
- Treat Matillion and Talend Cloud package licensing as transform pressure.
- Treat Informatica IICS as enterprise contract pressure.
- Treat Estuary Flow as streaming connector license pressure.
- Preserve ADR-0314 as the commercial authority.

## Local references
- `microservices/data-pipeline/capabilities/dealset-connector-license.yaml` is the primary capability.
- `microservices/data-pipeline/capabilities/connector-run-start.yaml` consumes license state.
- `microservices/data-pipeline/capabilities/transform-job-approve.yaml` may inherit license state.
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml` may inherit license state.
- `microservices/data-pipeline/cost-budget.md` records commercial cost dimensions.
- `microservices/data-pipeline/runbooks/dealset-connector-hold.md` defines hold response.
- `microservices/data-pipeline/runbooks/provider-rate-limit.md` separates provider throttling from license hold.
- `microservices/data-pipeline/competitor-parity-matrix.md` records benchmark context.
- `microservices/data-pipeline/contracts/local-openapi-v1.yaml` carries license refs.
- `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` emits license events.

## Settlement states
- `unknown` blocks licensed connector mutation.
- `pending` blocks connector worker start.
- `active` permits connector use when Cedar also permits.
- `suspended` blocks connector use.
- `expired` blocks connector use.
- `over_quota` blocks or throttles by policy.
- `trial` permits only trial-scoped connectors.
- `disputed` blocks new runs and allows evidence export.
- `terminated` blocks all new licensed side effects.
- `grandfathered` requires explicit contract metadata.
- `migration_only` permits export and replay but not new ingest.
- `audit_only` permits evidence read but not data movement.

## Command deltas
- Connector run start requires DealSet decision for licensed connector.
- Connector catalog import records license class.
- Schema drift release checks whether new fields change license scope.
- Transform approval records inherited connector license id.
- Lineage reconciliation records licensed source visibility.
- Replay approval checks original and current license state.
- Replay cursor advance records license state at replay time.
- Watermark advance checks license state when source connector is licensed.
- Audit export records license state but hides commercial details cross-tenant.
- Cost attribution records license dimension.
- Capacity admission can throttle over-quota licensed connector runs.
- SDK generation exposes license-state error codes.

## Event deltas
- `dealset.connector.checked` records license check.
- `dealset.connector.active` records active state.
- `dealset.connector.pending` records pending state.
- `dealset.connector.suspended` records suspension.
- `dealset.connector.expired` records expiry.
- `dealset.connector.over_quota` records quota block.
- `dealset.connector.migration_only` records migration restriction.
- `dealset.connector.audit_only` records audit restriction.
- `dealset.connector.scope_changed` records drift-derived scope change.
- `dealset.connector.hold_opened` records operator hold.
- Events include license id hash.
- Events include connector id.

## Proto deltas
- `DealSetDecisionRef` carries decision id.
- `DealSetDecisionRef` carries license state.
- `DealSetDecisionRef` carries connector id.
- `DealSetDecisionRef` carries scope hash.
- `DealSetDecisionRef` carries evaluated at.
- `DealSetDecisionRef` carries expires at.
- Connector run request embeds DealSet decision ref.
- Transform approval request embeds inherited DealSet decision ref.
- Replay approval request embeds original and current decision refs.
- Audit export request embeds redacted DealSet decision ref.
- Proto rejects licensed connector mutation without decision ref.
- Proto rejects expired decision ref.

## Cedar facts
- `dealset_license_state` is a policy fact.
- `dealset_scope_hash` is a policy fact.
- `connector_license_class` is a policy fact.
- `connector_id` is a policy fact.
- `source_object_id` is a policy fact.
- `tenant_contract_scope` is a policy fact.
- `quota_state` is a policy fact.
- `migration_only` is a policy fact.
- `audit_only` is a policy fact.
- `license_expires_at` is a policy fact.
- `commercial_details_visibility` is a policy fact.
- `derived_data_license_scope` is a policy fact.

## Workflow decisions
- DealSet check runs after tenant validation and before connector worker.
- Cedar still denies if DealSet permits but policy forbids.
- DealSet still blocks if Cedar permits but license is inactive.
- Schema drift can open license scope review.
- Transform approval inherits source license context.
- Lineage visibility is limited by license scope.
- Replay compares license state at failure and retry.
- Audit export redacts commercial terms unless auditor scope permits.
- Migration-only state allows export for exit path.
- Audit-only state allows evidence but not new data movement.
- Over-quota state can open operator remediation.
- License state changes emit audit evidence.

## Failure cases
- DealSet service unavailable blocks licensed connector mutation.
- Unknown license blocks connector start.
- Pending license opens hold runbook.
- Suspended license freezes connector runs.
- Expired license blocks new ingest.
- Over-quota license blocks or throttles by policy.
- Scope hash mismatch opens license review.
- Schema drift adds licensed field and opens hold.
- Replay under terminated license is denied unless migration-only permits.
- Audit export under disputed license is allowed only by auditor scope.
- Commercial detail leak is a compliance incident.
- Provider rate limit is not license denial.

## Replay cases
- Replay stores original DealSet decision id.
- Replay evaluates current DealSet decision id.
- Replay denied if current state is suspended.
- Replay allowed for migration-only export when policy permits.
- Replay denied for new ingest under migration-only.
- Replay under over-quota state follows quota policy.
- Replay under audit-only state cannot move cursor.
- Replay evidence includes license scope hash.
- Replay rollback preserves license decisions.
- Dead-letter custody redacts commercial terms.
- Cursor advance records current license state.
- Derived transform replay records inherited license state.

## Evidence fields
- `dealset_decision_id` is mandatory for licensed connector.
- `license_state` is mandatory.
- `license_scope_hash` is mandatory.
- `connector_id` is mandatory.
- `tenant_id` is mandatory.
- `evaluated_at` is mandatory.
- `expires_at` is mandatory.
- `quota_state` is mandatory when quota applies.
- `migration_only` is mandatory when applicable.
- `audit_only` is mandatory when applicable.
- `scope_change_reason` is mandatory on change.
- `cedar_decision_id` is mandatory.
- `audit_event_id` is mandatory.
- `cost_attribution_id` is mandatory when cost applies.
- `benchmark_pressure` is mandatory for parity summary.
- `runbook_ref` is mandatory for holds.

## SLOs
- DealSet check latency is tracked separately from policy latency.
- License hold count feeds operator remediation.
- Suspended connector run count feeds domain throughput.
- Over-quota count feeds cost dashboard.
- Scope-change review age feeds compliance pack health.
- Migration-only replay age feeds replay freshness risk.
- Audit-only evidence export latency feeds audit emission lag.
- License service outage feeds availability risk for licensed connectors.
- DealSet decision cache hit rate is monitored.
- Commercial detail redaction failures feed compliance incidents.
- Provider rate limit metrics remain separate.
- Benchmark parity summary records licensed connector coverage.

## Test cases
- Licensed connector run rejects missing DealSet decision.
- Active DealSet plus Cedar permit allows connector start.
- Active DealSet plus Cedar deny blocks connector start.
- Suspended DealSet blocks connector start.
- Expired DealSet blocks connector start.
- Migration-only permits export but blocks new ingest.
- Audit-only permits evidence read but blocks cursor advance.
- Schema drift scope change opens DealSet hold.
- Replay compares original and current license state.
- Commercial details are redacted without auditor scope.
- Over-quota state opens remediation.
- Provider rate limit is not treated as license failure.

## Rollback
- DealSet rollback restores prior decision cache version.
- Historical DealSet decisions remain immutable.
- Connector runs started under invalid decision freeze.
- Replay windows under invalid decision freeze.
- Transform approvals under invalid inherited decision require reapproval.
- Lineage visibility under invalid decision is recomputed.
- Cost attribution under invalid decision is corrected.
- Audit exports include original and rollback decision refs.
- Hold runbook closes only after active decision.
- Commercial terms remain redacted during rollback.
- Rollback emits dealset binding rollback event.
- Contract tests verify decision ref compatibility.

## Acceptance criteria
- Licensed connector mutation always has DealSet decision.
- DealSet permit never bypasses Cedar.
- Cedar permit never bypasses inactive license.
- Replay compares original and current DealSet state.
- Schema drift can trigger scope review.
- Transform cost carries license dimension.
- Audit export redacts commercial terms.
- Migration-only and audit-only states are explicit.
- Every benchmark reference is comparative.
- DealSet settlement remains Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/capabilities/dealset-connector-license.yaml`
- `microservices/data-pipeline/capabilities/connector-run-start.yaml`
- `microservices/data-pipeline/capabilities/transform-job-approve.yaml`
- `microservices/data-pipeline/capabilities/replay-cursor-advance.yaml`
- `microservices/data-pipeline/cost-budget.md`
- `microservices/data-pipeline/runbooks/dealset-connector-hold.md`
- `microservices/data-pipeline/runbooks/provider-rate-limit.md`
- `microservices/data-pipeline/competitor-parity-matrix.md`
- `microservices/data-pipeline/contracts/local-openapi-v1.yaml`
- `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml`
- `ADR-0314`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-014-marketplace-dealset-settlement.md:29` - - `microservices/data-pipeline/contracts/local-openapi-v1.yaml` carries license refs.; `microservices/data-pipeline/IP-014-marketplace-dealset-settlement.md:30` - - `microservices/data-pipeline/contracts/local-asyncapi-v1.yaml` emits license events..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `valkey`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-014-marketplace-dealset-settlement.md:162` - ## SLOs.

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: eligible for non-urgent batch, replay, export, backfill, package, or analytics work when error budget and pack recovery bounds permit deferral.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/data-pipeline/IP-014-marketplace-dealset-settlement.md:14` - - Treat Hevo and Stitch low-cost connector setup as pricing pressure.; `microservices/data-pipeline/IP-014-marketplace-dealset-settlement.md:25` - - `microservices/data-pipeline/cost-budget.md` records commercial cost dimensions..
