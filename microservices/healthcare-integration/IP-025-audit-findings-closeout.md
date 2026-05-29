# IP-025 Healthcare Integration audit findings closeout

Service: healthcare-integration
ChangeSet scope: microservices/healthcare-integration/IP-025-audit-findings-closeout.md
Doc class: Implementation Plan
Batch: C healthcare-integration IP deepening
Status: authoring-ready
Owner: axis-healthcare-integration
Binding ADRs: ADR-0105, ADR-0131, ADR-0242, ADR-0243, ADR-0244, ADR-0246, ADR-0253-amendment, ADR-0257, ADR-0258, ADR-0263, ADR-0294, ADR-0296, ADR-0297, ADR-0314, ADR-0321
Repo-local references: microservices/healthcare-integration/AUDIT-FINDINGS-2026-05-21.json; microservices/healthcare-integration/PRD.md; microservices/healthcare-integration/ARCHITECTURE.md; microservices/healthcare-integration/competitor-parity-matrix.md; microservices/healthcare-integration/compliance.md; microservices/healthcare-integration/threat-model.md; microservices/healthcare-integration/dpia.md; microservices/healthcare-integration/manifest.json; docs/standards/documentation-rigor.md
Capability references: capabilities/fhir-read.yaml; capabilities/hl7-route.yaml; capabilities/break-glass-authorize.yaml; capabilities/consent-sync.yaml; capabilities/ehr-provenance-seal.yaml; capabilities/patient-match-review.yaml
Benchmarks displaced: Redox, Rhapsody, InterSystems IRIS for Health, Lyniate/Corepoint, Mirth Connect, NextGate, Health Catalyst

## Objective
- Close the documented audit set findings without editing ADR-0321 or widening scope outside this IP.
- Replace thin checklist prose with buildable closeout mechanics for healthcare-integration artifact promotion.
- Bind every closeout row to tenant scope, Cedar default-deny, audit-chain evidence, DealSet settlement, and ADR-0321 operating-bar evidence.
- Treat the current audit JSON as evidence input, not as a blanket pass.
- Preserve the flat microservice layout required by ADR-0131.
- Keep clinical interoperability, consent, break-glass, MPI review, and provenance under this service boundary.
- Exclude code generation, schema generation, migrations, runtime code, and ADR edits from this implementation plan.

## Finding inventory
- Finding healthcare-integration-doc-set-001 is closed only when the artifact roster is reachable and each artifact cites local evidence.
- Finding healthcare-integration-transport-001 is closed only when contract and IaC references prove HTTP/3, ECH, PQC, and downgrade behavior.
- Finding healthcare-integration-marketplace-001 is closed only when DealSet settlement evidence is attached to every vendor-facing flow.
- The audit JSON status `closed-by-additive-artifacts` requires this IP to name the additive evidence set and rejection conditions.
- The closeout must prove that Redox-style connector breadth is displaced by tenant-scoped evidence ownership.
- The closeout must prove that Rhapsody-style interface-engine routing is displaced by policy-first workflow custody.
- The closeout must prove that InterSystems IRIS for Health-style platform breadth is displaced by flat service boundaries and ADR-traceable contracts.
- The closeout must prove that Lyniate/Corepoint-style interface operations are displaced by signed audit-chain closeout packets.
- The closeout must prove that Mirth Connect-style channel scripting is displaced by declarative contracts, Cedar gates, and replay evidence.
- The closeout must prove that NextGate-style MPI remediation is displaced by human adjudication with policy and provenance.
- The closeout must prove that Health Catalyst-style analytics evidence is displaced by operational evidence packets tied to each clinical exchange.

## ADR bindings
- ADR-0105 controls the layer names used by every cited catalog and implementation boundary.
- ADR-0131 controls the flat microservice layout and forbids suite folders.
- ADR-0242 controls the reserved Oyatie tenant doctrine and prevents vendor labels from becoming ownership roots.
- ADR-0243 controls Cedar as the universal gate for closeout-changing actions.
- ADR-0244 controls tenant_id, principal_id, audience_type, sub-scope, and home-cell scoping.
- ADR-0246 controls policy-engine substrate interaction and library-first policy dispatch assumptions.
- ADR-0253-amendment controls HTTP/3, h3-alt-svc, ECH, PQC, and transport downgrade disclosure.
- ADR-0257 controls ontology object-type versioning and deprecation handshakes.
- ADR-0258 controls public contract versioning and deprecation cadence.
- ADR-0263 controls audit-chain event naming, retention, and signed evidence.
- ADR-0294 controls fragment soak windows before policy enforcement.
- ADR-0296 controls library-first credential sidecar behavior and short-lived handles.
- ADR-0297 controls policy and bootstrap surface consistency for runtime admission.
- ADR-0314 controls DealSet settlement for marketplace obligations.
- ADR-0321 controls the second-pass operating-bar buildout and must remain read-only in this slice.

## Repo evidence map
- Evidence source 001: AUDIT-FINDINGS-2026-05-21.json supplies finding ids and declared closeout statuses.
- Evidence source 002: PRD.md supplies product boundary, target users, NFRs, and follow-up buildout rows.
- Evidence source 003: ARCHITECTURE.md supplies ADR-0105 layers, bounded contexts, and failure modes.
- Evidence source 004: competitor-parity-matrix.md supplies benchmark parity rows and audit floor language.
- Evidence source 005: compliance.md supplies pack impact and regulated evidence obligations.
- Evidence source 006: threat-model.md supplies control mapping and abuse/failure assumptions.
- Evidence source 007: dpia.md supplies privacy impact and regulated data-class handling.
- Evidence source 008: manifest.json supplies service tier, dependencies, ownership, and layer declarations.
- Evidence source 009: contracts/openapi-v1.yaml supplies public REST command/query proof.
- Evidence source 010: contracts/asyncapi-v1.yaml supplies async event surface proof.
- Evidence source 011: contracts/healthcare-integration-v1.proto supplies internal gRPC shape proof.
- Evidence source 012: iac/ech-config.yaml supplies ECH reference.
- Evidence source 013: iac/pqc-cert.yaml supplies PQC certificate reference.
- Evidence source 014: capabilities/*.yaml supplies capability binding ADRs and tenant required fields.
- Evidence source 015: runbooks/*.md supplies operational response references.

## Closeout workstream A - doc set closure
- A01: Confirm each required healthcare artifact remains in microservices/healthcare-integration, not a suite root.
- A02: Link closeout rows to PRD.md, ARCHITECTURE.md, compliance.md, threat-model.md, and dpia.md.
- A03: Require every cited artifact to include tenant scope or explicitly explain why it is not tenant-scoped.
- A04: Require every cited artifact to preserve healthcare-integration as a product-tier microservice.
- A05: Reject any closeout packet that points only to generated scaffold prose.
- A06: Reject any closeout packet that uses benchmark names as canonical object names.
- A07: Require local evidence for capability records, contracts, policies, runbooks, dashboards, SLOs, catalog records, IaC, and scorecards.
- A08: Require an owner field for unresolved follow-ups.
- A09: Require a promotion gate name for unresolved follow-ups.
- A10: Require a rollback route for each closeout claim.
- A11: Require the current audit finding id in each closeout packet.
- A12: Require ADR-0321 mention as authority without editing ADR-0321.
- A13: Require line-item review for healthcare-integration-doc-set-001.
- A14: Require line-item review for healthcare-integration-transport-001.
- A15: Require line-item review for healthcare-integration-marketplace-001.
- A16: Require final evidence packet export to include path, hash, reviewer, and timestamp fields.
- A17: Require cross-service references to remain API/event only.
- A18: Require `oyatie` reserved namespace references to stay under ADR-0242 authority.
- A19: Require stale evidence to be marked `superseded`, not deleted.
- A20: Require closeout dashboard rows to include finding id, artifact path, owner, and gate.

## Closeout workstream B - transport closure
- B01: Confirm openapi-v1.yaml declares tenant, principal, audience, purpose, data_class, idempotency, and trace context.
- B02: Confirm asyncapi-v1.yaml declares event classes for accepted, rejected, replayed, exported, and reviewed flows.
- B03: Confirm proto surfaces retain ADR-0105 layer separation and do not embed policy logic.
- B04: Confirm h3-alt-svc behavior is referenced through ADR-0253-amendment.
- B05: Confirm ECH and PQC evidence points to local IaC files.
- B06: Confirm downgrade to HTTP/2 or HTTP/1.1 emits trace and audit evidence.
- B07: Confirm transport closeout rejects silent downgrade.
- B08: Confirm transport closeout rejects PHI in URL path or query parameters.
- B09: Confirm transport closeout rejects raw tenant_id as high-cardinality metric label.
- B10: Confirm transport closeout includes certificate rotation and revocation evidence.
- B11: Confirm transport closeout includes source-system credential sidecar references.
- B12: Confirm transport closeout includes OpenBao reference shape where applicable.
- B13: Confirm transport closeout includes edge WAF and network policy references.
- B14: Confirm transport closeout includes regional cell behavior.
- B15: Confirm transport closeout includes break-glass no-friction exception rules.
- B16: Confirm transport closeout includes replay idempotency.
- B17: Confirm transport closeout includes contract version and deprecation notes.
- B18: Confirm transport closeout includes local runbook references for latency burn.
- B19: Confirm transport closeout includes local dashboard references.
- B20: Confirm transport closeout includes fail-closed mutation behavior when audit evidence cannot be written.

## Closeout workstream C - marketplace and settlement closure
- C01: Confirm DealSet settlement is named for every vendor-sourced integration flow.
- C02: Confirm marketplace obligations do not bypass Cedar evaluation.
- C03: Confirm marketplace obligations do not bypass consent segmentation.
- C04: Confirm provider billing evidence uses tenant, vendor, route, data class, and workflow run.
- C05: Confirm settlement holds are reversible with audit-chain evidence.
- C06: Confirm settlement release requires successful ACK or export seal where applicable.
- C07: Confirm settlement disputes link to clinical evidence without exposing PHI to billing operators.
- C08: Confirm pack overlays can tighten retention and export rules.
- C09: Confirm source vendor credentials remain sidecar-held.
- C10: Confirm marketplace closeout rejects raw vendor dashboards as authoritative evidence.
- C11: Confirm marketplace closeout rejects patient identifiers in billing ledger rows.
- C12: Confirm marketplace closeout includes hold release runbook references.
- C13: Confirm marketplace closeout includes cost-budget references.
- C14: Confirm marketplace closeout includes capacity-model references.
- C15: Confirm marketplace closeout includes audit event class names.
- C16: Confirm marketplace closeout includes refund and reversal behavior.
- C17: Confirm marketplace closeout includes tenant admin visibility.
- C18: Confirm marketplace closeout includes auditor visibility.
- C19: Confirm marketplace closeout includes support-operator limitations.
- C20: Confirm marketplace closeout includes benchmark displacement proof.

## Closeout-Specific Benchmark Displacement
- Displacement claim: this IP is about audit-findings closeout, so benchmark evidence is accepted only when each vendor comparison is tied to a finding, local artifact path, reviewer state, and closeout decision.
- Non-generic rule: a vendor row without route, consent, MPI, provenance, settlement, audit, or rollback proof remains a REVISE outcome even if the prose reads complete.
- Redox displacement: closeout evidence is tenant-scoped, signed, and policy-gated instead of connector-status centered.
- Redox displacement proof: require local route, consent, MPI, and provenance files to prove end-to-end ownership.
- Rhapsody displacement: route closeout includes custody and ACK evidence instead of opaque interface-engine state.
- Rhapsody displacement proof: require workflow run, route version, retry budget, and audit event reference.
- InterSystems IRIS for Health displacement: service boundary remains flat and ADR-traceable instead of platform-suite coupled.
- InterSystems IRIS for Health proof: require PRD, architecture, manifest, and contract cross-reference.
- Lyniate/Corepoint displacement: operator remediation is runbook and audit-chain driven instead of channel console driven.
- Lyniate/Corepoint proof: require local runbook, dashboard, and incident-response references.
- Mirth displacement: no channel script is the source of truth; contracts, Cedar, and replay evidence are.
- Mirth proof: require OpenAPI, AsyncAPI, proto, Cedar, and replay references.
- NextGate displacement: MPI duplicate closeout requires adjudication, match score rationale, and provenance.
- NextGate proof: require patient-match-review capability, runbook, and IP-029 reference.
- Health Catalyst displacement: analytics quality is not enough; operational custody and audit-chain closeout are mandatory.
- Health Catalyst proof: require event, metric, trace, log, and regulator-export evidence.

## Failure modes
- F01: Missing artifact path results in REVISE, not partial approval.
- F02: Artifact path exists but lacks ADR references results in REVISE.
- F03: Artifact path exists but lacks tenant scope results in REVISE unless explicitly non-tenant operational evidence.
- F04: Transport claim lacks local IaC evidence results in REVISE.
- F05: DealSet claim lacks settlement event evidence results in REVISE.
- F06: Benchmark claim lacks displacement wording results in REVISE.
- F07: Audit finding status conflicts with local evidence results in BLOCKED-FINDING.
- F08: Reviewer cannot resolve a local reference results in DOC-LINK-REVISE.
- F09: Closeout packet attempts to edit ADR-0321 results in SCOPE-REJECT.
- F10: Closeout packet touches any file outside assigned IPs in this batch results in SCOPE-REJECT for this slice.

## Capacity and performance notes
- Capacity note 001: closeout packet generation is offline documentation work and must not define runtime throughput.
- Capacity note 002: runtime-facing closeout claims defer to existing SLO files.
- Capacity note 003: audit export must budget for tenant, cell, vendor, route, and data-class partitions.
- Capacity note 004: audit export must avoid one global queue for all healthcare tenants.
- Capacity note 005: closeout dashboard cardinality must use finding id, class, owner, and status, not raw patient ids.
- Capacity note 006: closeout review latency target is one business day for ordinary findings.
- Capacity note 007: break-glass evidence review target is same shift for critical findings.
- Capacity note 008: failed evidence resolution opens remediation rather than blocking unrelated settled rows.
- Capacity note 009: replay validation may run asynchronously but must expose progress.
- Capacity note 010: regulator export packaging must be idempotent.

## Observability and evidence
- Emit `oya.healthcare.integration.audit.finding.closeout.requested`.
- Emit `oya.healthcare.integration.audit.finding.closeout.accepted`.
- Emit `oya.healthcare.integration.audit.finding.closeout.rejected`.
- Emit `oya.healthcare.integration.audit.finding.closeout.superseded`.
- Metric `healthcare_integration_audit_closeout_total` dimensions: finding_id, status, owner_team, cell, pack.
- Metric `healthcare_integration_audit_reference_missing_total` dimensions: artifact_class, owner_team, status.
- Trace span `healthcare.audit_closeout.evaluate` carries finding_id, tenant_scope_required, and evidence_count.
- Log schema includes event_id, finding_id, artifact_path, reviewer_id, decision_id, and audit_event_id.
- Audit evidence retains tenant id inside signed evidence, not high-cardinality metrics.
- Dashboard references: dashboards/local-audit-completeness.json and dashboards/compliance-pack-health.json.
- Runbook references: runbooks/local-audit-completeness-gap.md and incident-response.md.
- Scorecard reference: scorecards/overrides.json.

## Implementation steps
- Step 001: Treat AUDIT-FINDINGS-2026-05-21.json as the input finding ledger.
- Step 002: Build one closeout packet per finding id.
- Step 003: Attach local artifact references for each closeout claim.
- Step 004: Attach ADR binding list to each packet.
- Step 005: Attach benchmark displacement paragraph to each packet.
- Step 006: Attach transport evidence for healthcare-integration-transport-001.
- Step 007: Attach DealSet evidence for healthcare-integration-marketplace-001.
- Step 008: Attach doc roster evidence for healthcare-integration-doc-set-001.
- Step 009: Run link-resolution verification for referenced local paths.
- Step 010: Run scope, benchmark, citation-reference, and healthcare evidence verification for this IP.
- Step 011: Reject rows that rely on external vendor consoles.
- Step 012: Reject rows with unresolved future-work markers or scaffold filler language.
- Step 013: Preserve failed rows as explicit remediation tasks.
- Step 014: Produce reviewer-ready acceptance evidence.
- Step 015: Do not run oya vcs verify, done, or promote from this slice.

## Tests and evidence
- Test 001: `wc -l` on this IP must report at least 200 lines.
- Test 002: reference scan must show all binding ADR ids in this IP.
- Test 003: reference scan must show all seven displacement benchmarks in this IP.
- Test 004: reference scan must show AUDIT-FINDINGS-2026-05-21.json.
- Test 005: reference scan must show PRD.md, ARCHITECTURE.md, compliance.md, threat-model.md, and dpia.md.
- Test 006: reference scan must show contract, capability, runbook, dashboard, IaC, and scorecard local references.
- Test 007: review must confirm ADR-0321 was not edited.
- Test 008: review must confirm only assigned IP files changed.
- Test 009: review must confirm no shell-generated content files were created.
- Test 010: review must confirm no oya vcs verify, done, or promote command was run.

## Rollback
- Rollback 001: revert this IP file if closeout language conflicts with authoritative ADRs.
- Rollback 002: mark the affected finding as REVISE instead of deleting evidence.
- Rollback 003: keep previous audit JSON intact and attach superseding evidence.
- Rollback 004: open a follow-up IP if runtime artifacts must change.
- Rollback 005: do not mutate ADR-0321 as rollback.
- Rollback 006: do not remove existing service artifacts as rollback.
- Rollback 007: do not weaken tenant, Cedar, or DealSet constraints as rollback.
- Rollback 008: preserve audit event references for failed closeout attempts.

## Acceptance criteria
- AC01: This IP contains at least 200 lines.
- AC02: This IP cites repo-local audit, PRD, architecture, compliance, threat model, DPIA, manifest, capability, contract, runbook, dashboard, IaC, and scorecard evidence.
- AC03: This IP cites the current healthcare-integration ADR set without editing ADR-0321.
- AC04: This IP explicitly displaces Redox.
- AC05: This IP explicitly displaces Rhapsody.
- AC06: This IP explicitly displaces InterSystems IRIS for Health.
- AC07: This IP explicitly displaces Lyniate/Corepoint.
- AC08: This IP explicitly displaces Mirth Connect.
- AC09: This IP explicitly displaces NextGate.
- AC10: This IP explicitly displaces Health Catalyst.
- AC11: This IP rejects thin or template-stamped closeout evidence.
- AC12: This IP keeps scope to documentation planning for closeout mechanics.
- AC13: This IP names failure modes for missing evidence, stale evidence, transport gaps, settlement gaps, and ADR scope drift.
- AC14: This IP provides verification steps limited to assigned files.
- AC15: This IP is ready for a later implementation slice to produce machine-checkable closeout packets.

## Wave 15 grep-visible counterpart anchor
- Counterpart baseline: Salesforce Health Cloud, ServiceNow healthcare workflows, GitHub evidence review, and Slack incident collaboration are grep-visible Wave 15 verification anchors; native healthcare displacement remains Redox, Rhapsody, InterSystems IRIS for Health, Mirth Connect, Epic, Cerner, and NextGen-style clinical integration.

## API Versioning (per ADR-0342)

- Binding ADR: ADR-0342.
- Carrier: public API date version `2026-05-21` via header `Oyatie-Version`, URL prefix `/v/2026-05-21/`, and proto3 envelope field tag `8001` (`oyatie_version`).
- Initial declared_version: `2026-05-21`; no earlier shipped API date is declared in this IP or its µservice manifest.
- Support window: keep N=3 public versions available for at least 180 days after deprecation.
- Surface evidence: `microservices/healthcare-integration/IP-025-audit-findings-closeout.md:62` - - Evidence source 009: contracts/openapi-v1.yaml supplies public REST command/query proof.; `microservices/healthcare-integration/IP-025-audit-findings-closeout.md:63` - - Evidence source 010: contracts/asyncapi-v1.yaml supplies async event surface proof..
- Internal-mesh exemption: ADR-0145 direct internal gRPC remains unaffected; the version carriers bind only public OpenAPI, AsyncAPI, and externally exposed proto3 surfaces.

## DR posture (per ADR-0343)

- Binding ADR: ADR-0343.
- Numeric target source: `microservices/healthcare-integration/manifest.json#dr` is not declared; using the applicable compliance-pack floor until the D-2 manifest DR block lands.
- RTO/RPO target: `3600s` RTO p99 and `300s` RPO p99.
- Applicable compliance pack floor: `HIPAA-2024` from `specs/compliance-pack-floors.json` (`rto_p99_seconds=3600`, `rpo_p99_seconds=300`, `multi_region_required=true`, `drill_cadence_required=quarterly`).
- Multi-region active-active posture: `true` (required by the selected floor and IP evidence).
- backup_substrate: `postgres_wal_g`, `iceberg_snapshot`, `clickhouse_iceberg_layered`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Surface evidence: `microservices/healthcare-integration/IP-025-audit-findings-closeout.md:77` - - A07: Require local evidence for capability records, contracts, policies, runbooks, dashboards, SLOs, catalog records, IaC, and scorecards.; `microservices/healthcare-integration/IP-025-audit-findings-closeout.md:100` - - B08: Confirm transport closeout rejects PHI in URL path or query parameters..

## Sustainability emission (per ADR-0344)

- Binding ADR: ADR-0344.
- Per-call audit row emission: every audit event this IP introduces or mutates must include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` alongside `provider` and `region`.
- Workload signal: derive cost/carbon/energy from the IP-owned call, event, connector, transform, document, image, or notification operation named in the evidence below.
- Carbon-aware scheduling eligibility: excluded from deferral for synchronous clinical or critical-care paths; carbon-aware placement can apply only to offline replay, export, archive, or backfill work when pack recovery bounds remain satisfied.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Surface evidence: `microservices/healthcare-integration/IP-025-audit-findings-closeout.md:127` - - C13: Confirm marketplace closeout includes cost-budget references..
