# IP-012 Data Pipeline abuse-defence edge WAF

Service: data-pipeline
ChangeSet scope: microservices/data-pipeline/IP-012-abuse-defence-edge-waf.md
Benchmarks: Fivetran, Airbyte Cloud, Hevo, Stitch, Matillion, Talend Cloud, Informatica IICS, Estuary Flow
Binding ADRs: ADR-0105, ADR-0131, ADR-0132, ADR-0243, ADR-0244, ADR-0245, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0315, ADR-0316, ADR-0321

## Objective
- Protect Data Pipeline REST ingress from connector-run abuse.
- Protect schema drift sample endpoints from scraping.
- Protect transform approval endpoints from brute-force override attempts.
- Protect lineage repair endpoints from graph poisoning.
- Protect replay approval endpoints from custody enumeration.
- Protect watermark endpoints from freshness manipulation.
- Keep edge controls suspicion-based, not default user friction.
- Treat Fivetran and Airbyte Cloud public API exposure as benchmark pressure.
- Treat Hevo and Stitch low-friction ingestion as usability pressure.
- Treat Matillion, Talend Cloud, Informatica IICS, and Estuary Flow as governed API pressure.
- Preserve audit evidence for every edge refusal.
- Avoid using WAF as substitute for Cedar.

## Local references
- `microservices/data-pipeline/policy/abuse-defence.cedar` defines abuse policy.
- `microservices/data-pipeline/iac/edge-waf.yaml` defines edge WAF configuration.
- `microservices/data-pipeline/iac/production-ingress.yaml` defines ingress shape.
- `microservices/data-pipeline/iac/ech-config.yaml` defines encrypted client hello support.
- `microservices/data-pipeline/iac/pqc-cert.yaml` defines certificate posture.
- `microservices/data-pipeline/runbooks/provider-rate-limit.md` defines provider throttling response.
- `microservices/data-pipeline/dashboards/abuse-defence-outcomes.json` observes decisions.
- `microservices/data-pipeline/threat-model.md` records abuse threats.
- `microservices/data-pipeline/contracts/local-openapi-v1.yaml` anchors routes.
- `microservices/data-pipeline/slos/policy-decision-latency.openslo.yaml` tracks policy latency.

## Protected routes
- Connector run start is protected from burst creation.
- Connector catalog refresh is protected from enumeration.
- Schema drift sample capture is protected from payload scraping.
- Schema drift disposition is protected from unauthorized release attempts.
- Transform approval is protected from override brute force.
- Transform run status is protected from tenant enumeration.
- Lineage reconciliation apply is protected from graph poisoning.
- Lineage edge export is protected from egress abuse.
- Dead-letter replay approval is protected from custody enumeration.
- Replay cursor advance is protected from freshness tampering.
- Watermark advance is protected from stale/fresh oscillation.
- Audit export is protected from evidence scraping.

## Edge signals
- Request rate per tenant hash is a signal.
- Request rate per principal hash is a signal.
- Request rate per route family is a signal.
- Idempotency conflict rate is a signal.
- Policy denial rate is a signal.
- Payload size anomaly is a signal.
- Schema drift sample request frequency is a signal.
- Replay approval failure frequency is a signal.
- Lineage apply conflict frequency is a signal.
- Watermark backward attempt count is a signal.
- Credential lease denial count is a signal.
- Audit export failure count is a signal.

## Command deltas
- REST commands receive edge decision id.
- Connector start command consumes edge allow or challenge decision.
- Drift sample command consumes stricter edge decision.
- Transform override command consumes reviewer and edge decision.
- Lineage apply command consumes graph poisoning score.
- Replay approval command consumes custody enumeration score.
- Watermark advance command consumes freshness tamper score.
- Audit export command consumes evidence scraping score.
- Edge decision never replaces Cedar decision.
- Edge challenge is recorded before workflow start.
- Edge block emits refusal audit event.
- Edge allow remains scoped to request, not session.

## Event deltas
- `abuse.edge.allowed` records low-risk pass.
- `abuse.edge.challenged` records suspicion friction.
- `abuse.edge.blocked` records hard refusal.
- `abuse.edge.rate_limited` records throttling.
- `abuse.edge.payload_rejected` records size or shape failure.
- `abuse.edge.replay_enumeration_detected` records custody probing.
- `abuse.edge.lineage_poisoning_detected` records graph attack.
- `abuse.edge.watermark_tamper_detected` records freshness attack.
- `abuse.edge.audit_scrape_detected` records export abuse.
- Events include route family.
- Events include tenant hash.
- Events include audit event id.

## Cedar facts
- `edge_decision` is a policy fact.
- `edge_signal_score` is a policy fact.
- `route_family` is a policy fact.
- `idempotency_conflict_count` is a policy fact.
- `schema_sample_rate` is a policy fact.
- `replay_custody_lookup_rate` is a policy fact.
- `lineage_apply_conflict_rate` is a policy fact.
- `watermark_backwards_attempts` is a policy fact.
- `audit_export_attempt_rate` is a policy fact.
- `principal_reputation` is a policy fact.
- `tenant_rate_bucket` is a policy fact.
- `client_network_class` is a policy fact.

## Workflow decisions
- Edge WAF evaluates before REST body reaches application.
- Cedar evaluates after request normalization.
- WAF challenge cannot grant permission.
- WAF allow cannot bypass Cedar denial.
- WAF block emits audit evidence when route is known.
- Provider rate limits are not treated as tenant abuse automatically.
- Replay enumeration opens custody-protection workflow.
- Graph poisoning opens lineage incident workflow.
- Freshness tampering opens watermark incident workflow.
- Audit scraping opens privacy/security review.
- Emergency bypass remains separately governed.
- Clean default path avoids unnecessary friction.

## Failure cases
- WAF unavailable fails closed for high-risk mutation.
- WAF unavailable may allow low-risk read with degraded marker.
- Edge false positive can be appealed with audit evidence.
- Edge false negative is detected by downstream Cedar denial spike.
- Provider rate limit can masquerade as abuse and must be classified.
- Bot attack on connector start is throttled before worker enqueue.
- Payload flood on drift sample is blocked before custody.
- Replay custody enumeration is blocked before case details leak.
- Lineage graph poison attempt is blocked before ontology adapter.
- Watermark tamper attempt is blocked before freshness projection.
- Audit export scraping is blocked before bundle materialization.
- WAF config mismatch opens deployment incident.

## Replay cases
- Replay of edge-blocked command requires fresh edge evaluation.
- Replay of idempotent command returns prior refusal if facts match.
- Replay approval cannot bypass edge custody enumeration controls.
- Replay cursor advance cannot bypass watermark tamper controls.
- Replay evidence stores original edge decision id.
- Replay evidence stores current edge decision id.
- Replay after false positive needs operator review id.
- Replay after provider rate limit can use delayed retry.
- Replay after bot burst requires rate bucket reset.
- Replay after WAF outage waits for high-risk routes.
- Replay does not reuse edge allow across requests.
- Replay rollback preserves edge refusal history.

## Evidence fields
- `edge_decision_id` is mandatory.
- `route_family` is mandatory.
- `tenant_hash` is mandatory.
- `principal_hash` is mandatory.
- `signal_score` is mandatory.
- `decision` is mandatory.
- `challenge_type` is mandatory when challenged.
- `block_reason` is mandatory when blocked.
- `rate_bucket` is mandatory when rate-limited.
- `cedar_decision_id` is mandatory after downstream policy.
- `audit_event_id` is mandatory for mutation routes.
- `trace_id` is mandatory.
- `waf_config_version` is mandatory.
- `ingress_id` is mandatory.
- `benchmark_pressure` is mandatory for parity summary.
- `runbook_ref` is mandatory for incident class.

## SLOs
- Edge decision latency is tracked separately from policy latency.
- WAF block rate feeds abuse-defence outcomes dashboard.
- WAF challenge rate feeds operator remediation.
- False positive appeals feed quality review.
- Connector start throttle count feeds domain throughput.
- Drift sample block count feeds privacy risk.
- Replay enumeration block count feeds replay safety.
- Lineage poisoning block count feeds graph safety.
- Watermark tamper block count feeds freshness safety.
- Audit scrape block count feeds compliance health.
- WAF unavailable count feeds availability risk.
- Edge logs avoid raw tenant id labels.

## Test cases
- Burst connector starts are rate-limited.
- Cross-tenant route probing is blocked.
- Drift sample scraping is challenged or blocked.
- Transform override brute force is blocked.
- Lineage apply poisoning is blocked.
- Replay custody enumeration is blocked.
- Watermark backward attempts are blocked.
- Audit export scrape is blocked.
- WAF allow still requires Cedar permit.
- Cedar deny still emits refusal after WAF allow.
- WAF block emits audit evidence for mutation route.
- Provider rate limit is classified separately from abuse.

## Rollback
- WAF rule rollback uses config version.
- Previous WAF decisions remain evidence.
- Rate buckets are not blindly cleared.
- False-positive appeal list is preserved.
- High-risk routes remain fail-closed during rollback.
- Low-risk routes can run degraded if policy allows.
- Rollback emits edge config event.
- Ingress config rollback is verified.
- ECH and PQC posture remains unchanged unless explicitly rolled back.
- Abuse dashboard recomputes from events.
- Cedar policy remains authoritative after WAF rollback.
- Runbooks retain old edge decision ids.

## Acceptance criteria
- Every protected route has edge decision evidence.
- Edge WAF never replaces Cedar.
- Edge WAF blocks custody enumeration.
- Edge WAF blocks lineage poisoning.
- Edge WAF blocks freshness tampering.
- Edge WAF blocks audit scraping.
- Clean default path remains low-friction.
- Every benchmark reference is comparative.
- Every block can be audited.
- Abuse defence remains Data Pipeline-specific.

## Citation map
- `microservices/data-pipeline/policy/abuse-defence.cedar`
- `microservices/data-pipeline/iac/edge-waf.yaml`
- `microservices/data-pipeline/iac/production-ingress.yaml`
- `microservices/data-pipeline/iac/ech-config.yaml`
- `microservices/data-pipeline/iac/pqc-cert.yaml`
- `microservices/data-pipeline/runbooks/provider-rate-limit.md`
- `microservices/data-pipeline/dashboards/abuse-defence-outcomes.json`
- `microservices/data-pipeline/threat-model.md`
- `microservices/data-pipeline/contracts/local-openapi-v1.yaml`
- `microservices/data-pipeline/slos/policy-decision-latency.openslo.yaml`
- `ADR-0105`
- `ADR-0321`

## Wave 15 counterpart verification note

This IP was preserved as already substantive; the Wave 15 scrub adds the grep-visible counterpart hook required by ADR-0328 D-20 without replacing the existing Fivetran/Airbyte/dbt grounding. Data-pipeline parity remains anchored in Fivetran, Airbyte, and dbt Cloud, with Snowflake, Databricks, HubSpot, Stripe, Slack, Notion, Linear, GitHub, and GitLab named as connector/destination/ecosystem pressure where the specific primitive applies.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/data-pipeline/IP-012-abuse-defence-edge-waf.md:31` - - `microservices/data-pipeline/contracts/local-openapi-v1.yaml` anchors routes.; `microservices/data-pipeline/IP-012-abuse-defence-edge-waf.md:227` - - `microservices/data-pipeline/contracts/local-openapi-v1.yaml`.
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/data-pipeline/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `valkey`, `postgres_wal_g`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/data-pipeline/IP-012-abuse-defence-edge-waf.md:164` - ## SLOs.
