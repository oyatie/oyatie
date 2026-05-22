# IP-009 Healthcare Integration credential-sidecar-binding

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-009-credential-sidecar-binding.md
Batch: C healthcare-integration IP deepening
Status: implementation-plan-ready
Benchmarks displaced: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo references: microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/manifest.json; microservices/healthcare-integration/iac/openbao-policy.yaml; microservices/healthcare-integration/iac/local-openbao-policy.hcl; microservices/healthcare-integration/iac/secret-bindings.yaml; microservices/healthcare-integration/iac/local-secret-binding.yaml; microservices/healthcare-integration/threat-model.md; microservices/healthcare-integration/incident-response.md; microservices/healthcare-integration/capabilities/ehr-provenance-seal.yaml

## Objective
- IP-009-001: Bind healthcare-integration to the library-first credential sidecar pattern so EHR, HL7, FHIR, signing, marketplace, and audit export secrets never appear in service logs, proto metadata, policy payloads, or workflow state.
- IP-009-002: Preserve ADR-0296 by passing short-lived credential lease references to adapters rather than raw secret material.
- IP-009-003: Preserve ADR-0243 by requiring Cedar permit before lease acquisition and again before lease use when scope changes.
- IP-009-004: Preserve ADR-0244 by scoping all leases to tenant_id, principal_id, caller_service_id, home_cell, data_class, purpose, and capability.
- IP-009-005: Preserve ADR-0263 by emitting lease request, issue, use, deny, revoke, rotate, and quarantine audit events.
- IP-009-006: Preserve ADR-0253-amendment by requiring secure sidecar transport, strict TLS posture, and no fallback to plaintext loopback shortcuts.
- IP-009-007: Preserve ADR-0314 by including DealSet context for partner or marketplace credentials.
- IP-009-008: Preserve ADR-0297 by applying credential stuffing, scraping, replay, and bot-risk signals before lease issue.
- IP-009-009: Preserve ADR-0321 by displacing healthcare integration leaders with auditable secret isolation rather than integration-engine credential sprawl.
- IP-009-010: Keep this IP as a documentation/control deepening only; it does not edit OpenBao policy, IaC, or credential-binding files.

## Current thin content replacement
- IP-009-011: The previous file repeated generic capability rows and did not define lease types, sidecar contract, rotation behavior, threat controls, or benchmark displacement.
- IP-009-012: This rewrite uses iac/openbao-policy.yaml, iac/local-openbao-policy.hcl, iac/secret-bindings.yaml, and iac/local-secret-binding.yaml as the local evidence surfaces.
- IP-009-013: This rewrite treats threat-model.md and incident-response.md as required review surfaces for credential exposure and recovery.
- IP-009-014: This rewrite makes ehr-provenance-seal the first explicit signing-secret capability while still covering FHIR, HL7, consent, break-glass, and patient-match credentials.
- IP-009-015: This rewrite keeps raw vendor names out of secret paths except where a tenant-specific connector binding demands a provider class.

## Credential classes
- IP-009-016: FHIR API client credentials are scoped to tenant, source system, purpose, and allowed resource classes.
- IP-009-017: HL7 MLLP or gateway credentials are scoped to tenant, route, destination class, and ACK window.
- IP-009-018: OAuth refresh credentials are scoped to tenant, connector, delegated principal, and consent state.
- IP-009-019: mTLS client certificates are scoped to tenant, source system, environment, and home cell.
- IP-009-020: Webhook signing secrets are scoped to tenant, provider class, endpoint id, and rotation generation.
- IP-009-021: EHR provenance signing keys are scoped to tenant, transform id, source system, and evidence bundle class.
- IP-009-022: Audit export signing keys are scoped to tenant, regulator or auditor class, export batch, and pack.
- IP-009-023: Marketplace partner tokens are scoped to DealSet, provider tenant, buyer tenant, and settlement state.
- IP-009-024: Break-glass emergency credentials are scoped to emergency event, patient-scope bounds, expiry, and reviewer workflow.
- IP-009-025: Patient-match vendor credentials are scoped to matching job, candidate set, data class, and human review state.
- IP-009-026: Analytics export credentials are out of scope unless Health Catalyst displacement work requires evidence-only exports.
- IP-009-027: Developer, CI, and auditor credentials use separate policy scope and cannot access live PHI by default.
- IP-009-028: Credential classes must not collapse into a single provider-token bucket.
- IP-009-029: Credential classes must map to policy actions and audit event classes.
- IP-009-030: Credential classes must declare revocation triggers before implementation.

## Secret reference shape
- IP-009-031: Secret references use tenant-scoped paths like secret/<tenant_id>/healthcare-integration/<credential_class>/<binding_id>.
- IP-009-032: Secret references include secret_version or rotation_generation.
- IP-009-033: Secret references include home_cell and residency class.
- IP-009-034: Secret references include data_class and allowed capability.
- IP-009-035: Secret references include source_system_id or provider_binding_id when vendor-bound.
- IP-009-036: Secret references include dealset_ref when marketplace-governed.
- IP-009-037: Secret references include emergency_event_id when break-glass-scoped.
- IP-009-038: Secret references include audit_export_id when used for regulator or auditor export.
- IP-009-039: Secret references include created_by, approved_by, and policy_decision_ref for high-risk classes.
- IP-009-040: Secret references never contain patient identifiers.
- IP-009-041: Secret references never contain raw token material.
- IP-009-042: Secret references never appear in public API responses unless reduced to evidence ids.
- IP-009-043: Secret references must be stable enough for audit review but revocable without code changes.
- IP-009-044: Secret references must be serializable into audit events without leaking provider secrets.
- IP-009-045: Secret references must remain compatible with manifest cell eligibility and pack overlays.

## Lease acquisition
- IP-009-046: Lease acquisition starts after tenant, principal, capability, data_class, purpose, and policy decision are validated.
- IP-009-047: Lease acquisition requires caller workload identity from service mesh or runtime identity.
- IP-009-048: Lease acquisition requires request trace context and idempotency key for mutation flows.
- IP-009-049: Lease acquisition requires pack overlay evaluation for HIPAA, GDPR, EU-MDR, GxP, and related packs.
- IP-009-050: Lease acquisition requires data residency validation before sidecar selects a local secret backend.
- IP-009-051: Lease acquisition requires DealSet settlement validation for marketplace partner credentials.
- IP-009-052: Lease acquisition requires abuse-risk screening for suspicious source, replay, or stuffing patterns.
- IP-009-053: Lease acquisition for break-glass requires emergency reason, expiry, scope bounds, and reviewer workflow.
- IP-009-054: Lease acquisition for signing keys requires provenance event reference and transform id.
- IP-009-055: Lease acquisition for audit export requires recipient class, export batch, and regulatory pack.
- IP-009-056: Lease acquisition returns lease_ref, lease_scope, expires_at, key_generation, and audit_event_ref.
- IP-009-057: Lease acquisition does not return raw secret material to the domain or application layer.
- IP-009-058: Lease acquisition may return adapter-local delivery handles only to the adapter boundary.
- IP-009-059: Lease acquisition denies scope widening relative to the policy permit.
- IP-009-060: Lease acquisition emits both sidecar audit and healthcare-integration audit-chain events.

## Lease use
- IP-009-061: Lease use occurs only inside adapter or worker boundary code.
- IP-009-062: Lease use checks lease_ref, lease_scope, caller workload identity, and operation id.
- IP-009-063: Lease use checks tenant and source_system_id against the original policy decision.
- IP-009-064: Lease use checks expiry before every vendor call.
- IP-009-065: Lease use checks revocation state before retry.
- IP-009-066: Lease use for FHIR calls is limited to declared resource types and purpose.
- IP-009-067: Lease use for HL7 route calls is limited to declared route and destination class.
- IP-009-068: Lease use for consent calls is limited to consent source and consent purpose.
- IP-009-069: Lease use for provenance signing is limited to hash signing, not arbitrary payload signing.
- IP-009-070: Lease use for break-glass expires at the emergency event expiry.
- IP-009-071: Lease use for audit export expires at export batch completion.
- IP-009-072: Lease use failure emits denial evidence without exposing provider response bodies.
- IP-009-073: Lease use success links vendor call evidence to audit_chain_ref.
- IP-009-074: Lease use must never be logged with token, secret, certificate, private key, or authorization header values.
- IP-009-075: Lease use must be replay-safe: replay reuses prior lease evidence or requests a new lease under current policy.

## Rotation and revocation
- IP-009-076: Rotation triggers include scheduled rotation, tenant offboarding, provider breach, employee departure, compromised lease, pack upgrade, and policy fragment rollback.
- IP-009-077: Revocation triggers include policy denial, consent revocation, break-glass closeout, DealSet cancellation, abuse lockout, and incident response quarantine.
- IP-009-078: Rotation must produce old_generation, new_generation, affected bindings, and validation evidence.
- IP-009-079: Revocation must produce revocation_event_ref, affected lease ids, and blocked capability list.
- IP-009-080: Rotation cannot allow dual-valid generations beyond the declared overlap window.
- IP-009-081: Revocation takes precedence over retry and replay.
- IP-009-082: Break-glass revocation happens automatically at expiry and again after review closeout.
- IP-009-083: Marketplace partner token revocation follows DealSet state changes.
- IP-009-084: FHIR OAuth refresh rotation requires consent state revalidation.
- IP-009-085: HL7 route secret rotation requires ACK-window drainage or safe NACK behavior.
- IP-009-086: Provenance signing key rotation requires verification continuity for prior evidence bundles.
- IP-009-087: Audit export key rotation requires auditor/regulator export validation.
- IP-009-088: Revoked lease use returns PERMISSION_DENIED-equivalent adapter error and audit event.
- IP-009-089: Rotation failures fail closed for writes and allow only safe status reads.
- IP-009-090: Rotation and revocation evidence feeds incident-response.md.

## Sidecar deployment
- IP-009-091: Sidecar runs with least privilege and cannot read secrets outside healthcare-integration tenant scopes.
- IP-009-092: Sidecar is cell-local and respects manifest cross_cell_replication rules.
- IP-009-093: Sidecar policy is defined by openbao-policy.yaml and local-openbao-policy.hcl surfaces.
- IP-009-094: Sidecar binding is declared in secret-bindings.yaml and local-secret-binding.yaml surfaces.
- IP-009-095: Sidecar transport uses strict TLS and workload identity.
- IP-009-096: Sidecar does not expose a public ingress.
- IP-009-097: Sidecar does not expose debug endpoints with secret metadata.
- IP-009-098: Sidecar emits metrics without secret path or raw tenant labels.
- IP-009-099: Sidecar logs redact secret_ref beyond stable evidence id.
- IP-009-100: Sidecar can be disabled per credential class without disabling the entire service.
- IP-009-101: Sidecar readiness fails if policy, OpenBao, audit, or workload identity prerequisites are unavailable.
- IP-009-102: Sidecar liveness does not prove secret access; readiness must validate scoped capability.
- IP-009-103: Sidecar admission must block pods missing secret binding annotations.
- IP-009-104: Sidecar service account must not grant cross-microservice secret access.
- IP-009-105: Sidecar deployment evidence is reviewed alongside local-secret-binding and network policy evidence.

## Threat controls
- IP-009-106: Threat control covers token exfiltration by preventing raw secrets from crossing adapter boundary.
- IP-009-107: Threat control covers confused deputy by binding lease to caller workload identity.
- IP-009-108: Threat control covers cross-tenant secret path traversal by canonical tenant-scoped paths.
- IP-009-109: Threat control covers replay by binding lease use to operation id and policy decision.
- IP-009-110: Threat control covers credential stuffing by integrating ADR-0297 risk signals before lease issue.
- IP-009-111: Threat control covers policy bypass by requiring policy decision ids at acquisition.
- IP-009-112: Threat control covers audit evasion by blocking high-risk operations during audit-chain outage.
- IP-009-113: Threat control covers secret sprawl by separating credential classes and provider bindings.
- IP-009-114: Threat control covers stale vendor access by revoking on consent, DealSet, tenant, or pack state change.
- IP-009-115: Threat control covers CI misuse by separate CI-scope policy and no live PHI default access.
- IP-009-116: Threat control covers auditor overreach by separate auditor-scope policy and export class bounds.
- IP-009-117: Threat control covers signing misuse by restricting provenance signing to hashes and evidence bundles.
- IP-009-118: Threat control covers break-glass abuse by expiry, scope, review workflow, and post-event audit.
- IP-009-119: Threat control covers provider breach by quarantine and replay-safe credential generation.
- IP-009-120: Threat controls must be reflected in threat-model.md and incident-response.md during implementation.

## Benchmark displacement
- IP-009-121: Redox is displaced by tenant-bound credential leases with audit proof instead of connector-level secret possession.
- IP-009-122: Rhapsody is displaced by sidecar-scoped leases rather than route-engine credential stores.
- IP-009-123: InterSystems IRIS for Health is displaced by externalized secret governance rather than platform database credential centrality.
- IP-009-124: Lyniate/Corepoint is displaced by revocable, policy-scoped leases rather than channel credentials.
- IP-009-125: Mirth Connect is displaced by removing secret access from transform scripts and local channel state.
- IP-009-126: NextGate is displaced by patient-match credentials that stay bound to review jobs and human decisions.
- IP-009-127: Health Catalyst is displaced by export credentials tied to evidence bundles and pack overlays before analytics flows.
- IP-009-128: Epic parity pressure is handled by scoped FHIR OAuth and mTLS credential handling.
- IP-009-129: Cerner parity pressure is handled by route-scoped HL7 and FHIR credentials with ACK evidence.
- IP-009-130: Veeva parity pressure is handled by GxP-grade provenance signing and audit export controls.

## Implementation steps
- IP-009-131: Inventory all healthcare-integration secret bindings and classify them by credential class.
- IP-009-132: Define SecretReference, CredentialLeaseRequest, CredentialLease, LeaseUseRequest, and LeaseRevocation evidence shapes.
- IP-009-133: Add policy decision id and permit scope to every lease request.
- IP-009-134: Add DealSet and settlement state to marketplace-bound credential requests.
- IP-009-135: Add break-glass expiry and review workflow to emergency credential requests.
- IP-009-136: Add source-system and transform references to provenance signing leases.
- IP-009-137: Add adapter-bound lease use checks for workload identity and operation id.
- IP-009-138: Add lease revocation on consent, tenant, DealSet, pack, and policy rollback events.
- IP-009-139: Add rotation evidence fixture for each credential class.
- IP-009-140: Add redaction tests that scan structured logs and error payloads for secret-shaped values.
- IP-009-141: Add sidecar readiness tests for policy, OpenBao, audit, and workload identity prerequisites.
- IP-009-142: Add negative tests for missing policy decision, scope widening, expired lease, and revoked lease.
- IP-009-143: Add incident drill for provider breach and secret quarantine.
- IP-009-144: Add SLO and dashboard evidence for lease issue latency and denial rate.
- IP-009-145: Add runbook references for rotation failure, compromised credential, and break-glass review.

## Tests and evidence
- IP-009-146: Unit evidence: raw secret material never appears outside adapter boundary mocks.
- IP-009-147: Unit evidence: missing policy_decision_ref denies lease acquisition.
- IP-009-148: Unit evidence: scope widening denies lease use.
- IP-009-149: Unit evidence: expired lease denies vendor call.
- IP-009-150: Unit evidence: revoked lease denies retry.
- IP-009-151: Unit evidence: DealSet cancellation revokes marketplace credential.
- IP-009-152: Unit evidence: consent revocation invalidates FHIR credential use.
- IP-009-153: Unit evidence: break-glass expiry revokes emergency credential.
- IP-009-154: Integration evidence: OpenBao policy path matches secret binding path.
- IP-009-155: Integration evidence: sidecar readiness fails when audit-chain evidence cannot be emitted.
- IP-009-156: Integration evidence: sidecar cannot read another tenant secret path.
- IP-009-157: Integration evidence: provenance signing lease signs hash only.
- IP-009-158: Observability evidence: metrics omit raw tenant id and secret path.
- IP-009-159: Incident evidence: compromised credential drill produces quarantine and rotation events.
- IP-009-160: Replay evidence: replay cannot reuse stale lease outside current policy.

## Rollback
- IP-009-161: If sidecar binding fails, disable vendor-bound methods while preserving status and evidence-only reads.
- IP-009-162: If OpenBao policy denies legitimate scope, pin affected credential class to prior policy generation.
- IP-009-163: If raw secret exposure is detected, quarantine affected leases and rotate credential generation.
- IP-009-164: If metrics leak secret labels, disable metric export and preserve audit-chain evidence.
- IP-009-165: If DealSet credential binding fails, block commercial partner flows.
- IP-009-166: If break-glass lease closeout fails, disable new emergency leases until review workflow is repaired.
- IP-009-167: If provenance signing rotation fails, stop new evidence bundle signing and keep verification for prior bundles.
- IP-009-168: If HL7 route rotation interrupts ACK windows, switch route to safe NACK and drain.
- IP-009-169: If FHIR OAuth refresh fails, deny vendor calls and retain local projection reads allowed by policy.
- IP-009-170: Rollback evidence must include credential class, tenant scope, lease ids, policy decisions, audit ids, and rotation generation.

## Acceptance criteria
- IP-009-171: No healthcare-integration layer outside adapter/sidecar receives raw secret material.
- IP-009-172: Every lease request includes tenant, principal, capability, purpose, data_class, home_cell, and policy decision.
- IP-009-173: Every vendor-bound lease includes source_system_id or provider_binding_id.
- IP-009-174: Every commercial partner lease includes DealSet context.
- IP-009-175: Every break-glass lease includes emergency event, scope, expiry, and review workflow.
- IP-009-176: Every provenance signing lease is limited to evidence hash signing.
- IP-009-177: Every lease issue, use, denial, revoke, and rotate emits audit evidence.
- IP-009-178: Every secret path is tenant scoped and cell aware.
- IP-009-179: Every lease enforces expiry and revocation before retry.
- IP-009-180: Every credential class has rotation and revocation triggers.
- IP-009-181: Every credential class has threat-model coverage.
- IP-009-182: Every sidecar metric avoids raw tenant and secret path labels.
- IP-009-183: Every sidecar error response is redacted.
- IP-009-184: Every CI and auditor access path uses scoped policy.
- IP-009-185: Every benchmark displacement claim maps to a concrete lease, sidecar, rotation, or audit control.
- IP-009-186: ADR-0296 behavior is explicit, testable, and rollback-ready.
- IP-009-187: ADR-0321 remains cited as doctrine and is not edited by this IP.
- IP-009-188: Implementation can proceed without broadening this batch beyond the assigned IP.
- IP-009-189: Verification can be done through policy tests, sidecar tests, log redaction tests, and incident drills.
- IP-009-190: The plan does not require editing any non-assigned file in this batch.

## Citation summary
- IP-009-191: PRD.md supplies tenant, purpose, data_class, idempotency, pack, and audit-chain requirements.
- IP-009-192: ARCHITECTURE.md supplies adapter boundaries, dependencies, and key compromise failure mode.
- IP-009-193: manifest.json supplies binding ADRs, cell eligibility, packs, benchmarks, and dependency list.
- IP-009-194: iac/openbao-policy.yaml anchors OpenBao policy evidence.
- IP-009-195: iac/local-openbao-policy.hcl anchors local OpenBao behavior.
- IP-009-196: iac/secret-bindings.yaml anchors production secret binding evidence.
- IP-009-197: iac/local-secret-binding.yaml anchors local secret binding evidence.
- IP-009-198: threat-model.md anchors secret exfiltration, confused deputy, replay, and path traversal review.
- IP-009-199: incident-response.md anchors compromise, quarantine, rotation, and post-incident evidence.
- IP-009-200: capabilities/ehr-provenance-seal.yaml anchors provenance signing and marketplace settlement expectations.
- IP-009-201: policy/clinical-interoperability-authorization.cedar anchors lease authorization.
- IP-009-202: policy/abuse-defence.cedar anchors credential abuse risk evaluation.
- IP-009-203: policy/data-residency.md anchors cell-local secret resolution.
- IP-009-204: ADR-0296 anchors library-first credential sidecar doctrine.
- IP-009-205: ADR-0321 remains cited as existing B2B leader coverage doctrine only; this IP does not edit ADR-0321.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `14400s` RTO p99 and `900s` RPO p99.
- Applicable compliance pack floor: `KR-PIPA-2023-amendment` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=14400`, `rpo_p99_seconds=900`, `multi_region_required=false`, `drill_cadence_required=semi-annual`).
- Multi-region active-active posture: `false` (not pack-mandated by the selected floor and IP evidence).
- backup_substrate: `valkey`, `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-009-credential-sidecar-binding.md:42` - - IP-009-027: Developer, CI, and auditor credentials use separate policy scope and cannot access live PHI by default.; `microservices/healthcare-integration/IP-009-credential-sidecar-binding.md:142` - - IP-009-115: Threat control covers CI misuse by separate CI-scope policy and no live PHI default access..
