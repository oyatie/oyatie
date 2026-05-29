# workplace-integration µservice ownership-coherence audit
Audit date: 2026-05-20
Execution date: 2026-05-21
Target µservice: `microservices/workplace-integration/`
Owner lane audited: `axis-workplace-integration`
Audit owner: single-agent solo audit
Deliverable set: three reports only
Retired deliverable: no capability-adoption-deltas report authored
Top-3 counterpart set: Slack App Directory / Microsoft Teams App Store / Zapier Integrations
Initial deployable-context assumption: all six contexts unless evidence proves N/A
Final audit verdict: REVISE
P0 findings: 0
P1 findings: 8
P2 findings: 8
P3 findings: 2
tenant_class-retirement candidates found: 29 direct demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating references
Tenant-class adoption state: absent in current µservice artifacts

## Header citation anchors
- Canonical sequence source: `specs/master-plan-sequencing.json:704-745` names the six deployment contexts and their OpenTofu target paths.
- Canonical IaC source: `specs/master-plan-sequencing.json:747-775` sets OpenTofu as the engine and forbids Terraform, `null_resource`, local-exec, SSH provisioners, hand-edited state, and unsigned modules.
- Canonical OS source: `specs/master-plan-sequencing.json:777-815` requires the supported-OS matrix and per-µservice manifest coverage.
- Canonical language source: `specs/master-plan-sequencing.json:817-856` requires Rust backend posture with only bounded non-Rust configuration/contract formats.
- Canonical OCI source: `specs/master-plan-sequencing.json:857-868` requires the OCI Always Free profile module at `iac/oci-guest/always-free/`, while line 865 is itself stale tenant_class vocabulary to retire.
- Canonical audit protocol source: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3830-4165` defines dimensions 6-9, severity mapping, and evidence requirements.
- Substance-bar source: `docs/standards/brief-template.md:392-396` permits tools for counts but not scripted substantive bodies; `docs/standards/brief-template.md:1727-1806` names scaffold and line-count padding anti-patterns.
- tenant_class-retirement source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_tenant_class_adoption_2026_05_20.md:10-24` records the retirement of demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating capability availability.
- Tenant-class replacement source: current batch directive requires `{demo_trial, paid, revenue_share}` tenant classes; `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:139-142` confirms Wave 3 Batch 3.2 drops tenant_class deltas and uses single benchmark targets.
- Ownership-coherence source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:10-14` states contradictions in design docs lead to the wrong product.
- Verification source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-12` says line count is not enough; actual deliverables must be verified.
- Substance source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-18` rejects thin scaffold and variable-swapped boilerplate.

## §1 Purpose
1. This audit asks whether `workplace-integration` is internally coherent, externally aligned, and buildable from its µservice-local artifacts.
2. The audit does not remediate product docs, contracts, IaC, or code beyond landing the three requested audit deliverables.
3. The audit treats `microservices/workplace-integration/` as the entire ownership boundary.
4. The audit searched the chat-history dump for `workplace-integration` before authoring.
5. The chat-history search found 145 matching lines in `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl`.
6. The earliest direct product brief found in chat line 1355 named a "Workplace Integration Layer" for clocking-in, approvals, e-signing, meetings, and expense reports.
7. The expanded chat prompt at line 1444 directed a cross-cutting PRD orchestrated through Workflow Engine and delivered via Mail, Messenger, Calendar, and Meet.
8. The rolling audit queue at chat line 16311 assigned this µservice's counterpart set as Slack App Directory, Microsoft Teams App Store, and Zapier Integrations.
9. The current µservice README defines a narrower surface: workplace agreement, e-sign, roster, clock-in geofence, informed consent, closing package, and DLP evidence at `README.md:14-16`.
10. The PRD repeats the same narrowed scope at `PRD.md:17-22`.
11. The PRD endpoint table covers e-sign, offer letters, engagement agreements, roster bindings, clock events, and DLP traces at `PRD.md:54-63`.
12. The current product center of gravity is therefore regulated workforce agreements and evidence, not a general app-discovery or no-code integration marketplace.
13. That difference matters because the assigned counterpart set is marketplace/integration-platform shaped, not HRIS/vendor-migration shaped.
14. A coherent outcome can still exist if the µservice is explicitly scoped as the workplace-domain adapter into those integration surfaces.
15. Current artifacts do not yet make that bridge explicit.
16. The audit therefore evaluates two axes at once: the actual local product, and the required counterpart union bar.
17. The service has a large artifact set: 135 files and 22,660 total lines under the µservice path.
18. The line mass is not the same as substance because multiple 36-line operational docs share a common scaffold.
19. The most serious implementation blockers are not missing prose volume; they are contradictory contract/event mappings and missing canonical infrastructure posture.
20. The service has Rust source and Cargo metadata, but only a documentation scaffold in `src/lib.rs`.
21. The Rust source forbids unsafe code at `src/lib.rs:1`.
22. The Rust source declares versions and layer constants at `src/lib.rs:3-22`.
23. The Rust source contains two unit tests for version and layer declarations at `src/lib.rs:39-56`.
24. No application behavior beyond scaffold declarations is implemented in the inspected source.
25. The service has no `tests/` directory in the inventory.
26. The service does have contracts, Cedar policies, OpenSLO files, dashboards, and runbooks.
27. Those supporting artifacts are not mutually coherent in several key places.
28. The OpenAPI surface repeats `WorkplaceESignSessionCreated` as the audit event for every mutation route at `contracts/openapi-v1.yaml:28`, `:52`, `:76`, `:100`, `:124`, `:148`, and `:172`.
29. The AsyncAPI surface defines distinct events for the same conceptual lifecycle at `contracts/asyncapi-v1.yaml:18-52`.
30. The capabilities directory also assigns different capability events, but some are shifted relative to capability names at `capabilities/esign-initiate.yaml:27-30`, `capabilities/esign-sign.yaml:27-30`, and `capabilities/offer-generate.yaml:27-30`.
31. The OpenSLO files also show shifted metric wiring: clock availability reads DLP trace metrics at `slos/clock-attestation-availability.openslo.yaml:19-26`.
32. Signature latency reads offer-generation metrics at `slos/signature-capture-latency.openslo.yaml:19-26`.
33. Offer-generation latency reads roster-bind metrics at `slos/offer-generation-latency.openslo.yaml:19-26`.
34. Roster-binding accuracy reads clock-attest metrics at `slos/roster-binding-accuracy.openslo.yaml:19-26`.
35. Those shifts are not cosmetic because observability is the evidence plane for contractual SLOs and incident response.
36. The IaC directory has no per-context subdirectories.
37. The only IaC directory printed by the directory scan was `microservices/workplace-integration/iac`.
38. The IaC directory contains `terraform-main.tf` and `terraform-variables.tf`.
39. Both Terraform-named files contain `null_resource`, at `iac/terraform-main.tf:6` and `iac/terraform-variables.tf:6`.
40. This directly conflicts with OpenTofu canonical policy at `specs/master-plan-sequencing.json:747-775` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3897-3939`.
41. The service lacks `supported-oses.json` or `supported_oses.json` by inventory search.
42. The service manifest has no `supported_oses` field in `manifest.json:1-176`.
43. The service manifest has no `deployment_contexts` field in `manifest.json:1-176`.
44. The service manifest has no `tenant_class`, `demo_trial`, `paid`, or `revenue_share` semantics in `manifest.json:1-176`.
45. A repo-local search for tenant-class terms inside this µservice returned zero matches.
46. A repo-local search for forbidden language source files returned zero files.
47. The Rust-strict dimension therefore passes for source-file presence, with a caveat about proto generator options.
48. The proto file includes Java and Go package options at `contracts/workplace-integration-v1.proto:5-7`.
49. Those options are contract metadata, not authored Java or Go application logic, but should carry generated-client provenance if retained.
50. The audit verdict is REVISE because canonical infrastructure and contract coherency are not safe to implement from as-is.

## §2 Complete inventory
Inventory command: `find microservices/workplace-integration -type f | sort`
Inventory count: 135 files
Inventory line total: 22,660 lines
Read strategy: full inventory plus substantial reads of PRD, architecture, README, manifest, contracts, SLOs, policies, capability availability, benchmark, FAQ, onboarding, migration, tutorial, representative root docs, IaC, source, and canonical constraint files.

1. `microservices/workplace-integration/ARCHITECTURE.md`
2. `microservices/workplace-integration/AUDIT-FINDINGS-2026-05-20.json`
3. `microservices/workplace-integration/CHANGELOG.md`
4. `microservices/workplace-integration/Cargo.lock`
5. `microservices/workplace-integration/Cargo.toml`
6. `microservices/workplace-integration/IP-journey-j109-esign-roster-binding.md`
7. `microservices/workplace-integration/IP-journey-j110-esign-roster-binding.md`
8. `microservices/workplace-integration/IP-journey-j112-esign-roster-binding.md`
9. `microservices/workplace-integration/IP-journey-j113-esign-roster-binding.md`
10. `microservices/workplace-integration/IP-journey-j114-esign-roster-binding.md`
11. `microservices/workplace-integration/IP-journey-j121-esign-closing-package.md`
12. `microservices/workplace-integration/IP-journey-j132-offer-letter-esign-per-jurisdiction.md`
13. `microservices/workplace-integration/IP-journey-j134-engagement-agreement-and-staffing-aware-offer.md`
14. `microservices/workplace-integration/IP-journey-j140-internal-audit-dlp-egress-cross-tenant-trace.md`
15. `microservices/workplace-integration/IP-journey-j37-clock-in-geofence.md`
16. `microservices/workplace-integration/IP-journey-j38-e-sign-session.md`
17. `microservices/workplace-integration/IP-journey-j51-e-sign-on-po.md`
18. `microservices/workplace-integration/IP-journey-j54-e-signature.md`
19. `microservices/workplace-integration/IP-journey-j56-offer-e-sign.md`
20. `microservices/workplace-integration/IP-journey-j63-informed-consent.md`
21. `microservices/workplace-integration/IP-journey-j70-e-sign.md`
22. `microservices/workplace-integration/PHASE-01-DOC-SET-CLOSURE.md`
23. `microservices/workplace-integration/PRD.md`
24. `microservices/workplace-integration/README.md`
25. `microservices/workplace-integration/backfill-replay.md`
26. `microservices/workplace-integration/benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md`
27. `microservices/workplace-integration/capabilities/clock-attest.yaml`
28. `microservices/workplace-integration/capabilities/dlp-trace-seal.yaml`
29. `microservices/workplace-integration/capabilities/esign-initiate.yaml`
30. `microservices/workplace-integration/capabilities/esign-sign.yaml`
31. `microservices/workplace-integration/capabilities/offer-generate.yaml`
32. `microservices/workplace-integration/capabilities/roster-bind.yaml`
33. `microservices/workplace-integration/ADR-0330 and ADR-0331 tenant_class model`
34. `microservices/workplace-integration/capacity-model.md`
35. `microservices/workplace-integration/catalog/oya-workplace-integration-adapter.yaml`
36. `microservices/workplace-integration/catalog/oya-workplace-integration-api.yaml`
37. `microservices/workplace-integration/catalog/oya-workplace-integration-application.yaml`
38. `microservices/workplace-integration/catalog/oya-workplace-integration-domain.yaml`
39. `microservices/workplace-integration/catalog/oya-workplace-integration-events.yaml`
40. `microservices/workplace-integration/catalog/oya-workplace-integration-iac.yaml`
41. `microservices/workplace-integration/catalog/oya-workplace-integration-kernel.yaml`
42. `microservices/workplace-integration/catalog/oya-workplace-integration-observability.yaml`
43. `microservices/workplace-integration/catalog/oya-workplace-integration-policy.yaml`
44. `microservices/workplace-integration/catalog/oya-workplace-integration-rest.yaml`
45. `microservices/workplace-integration/catalog/oya-workplace-integration-sdk.yaml`
46. `microservices/workplace-integration/catalog/oya-workplace-integration-usecase.yaml`
47. `microservices/workplace-integration/catalog/oya-workplace-integration-worker.yaml`
48. `microservices/workplace-integration/competitor-parity-matrix.md`
49. `microservices/workplace-integration/compliance.md`
50. `microservices/workplace-integration/contracts/asyncapi-v1.yaml`
51. `microservices/workplace-integration/contracts/openapi-v1.yaml`
52. `microservices/workplace-integration/contracts/workplace-integration-v1.proto`
53. `microservices/workplace-integration/cost-budget.md`
54. `microservices/workplace-integration/dashboards/audit-evidence.json`
55. `microservices/workplace-integration/dashboards/policy-deny-rate.json`
56. `microservices/workplace-integration/dashboards/replay-health.json`
57. `microservices/workplace-integration/dashboards/service-overview.json`
58. `microservices/workplace-integration/dashboards/tenant-slo-burn.json`
59. `microservices/workplace-integration/decisions/ADR-WPI-001-clock-in-geofence-with-tolerance-vs-strict-vs-flexible-tenant_class-matrix.md`
60. `microservices/workplace-integration/dpia.md`
61. `microservices/workplace-integration/failure-modes.md`
62. `microservices/workplace-integration/faqs/hris-engineer-faq.md`
63. `microservices/workplace-integration/iac/grafana-datasource.yaml`
64. `microservices/workplace-integration/iac/kubernetes-deployment.yaml`
65. `microservices/workplace-integration/iac/kubernetes-service.yaml`
66. `microservices/workplace-integration/iac/kustomization.yaml`
67. `microservices/workplace-integration/iac/network-policy.yaml`
68. `microservices/workplace-integration/iac/openbao-policy.yaml`
69. `microservices/workplace-integration/iac/otel-collector.yaml`
70. `microservices/workplace-integration/iac/pqc-tls-profile.yaml`
71. `microservices/workplace-integration/iac/region-failover.yaml`
72. `microservices/workplace-integration/iac/secret-bindings.yaml`
73. `microservices/workplace-integration/iac/terraform-main.tf`
74. `microservices/workplace-integration/iac/terraform-variables.tf`
75. `microservices/workplace-integration/incident-response.md`
76. `microservices/workplace-integration/ip/IP-001-agreement-kernel.md`
77. `microservices/workplace-integration/ip/IP-002-esign-session-domain.md`
78. `microservices/workplace-integration/ip/IP-003-signature-proof-usecase.md`
79. `microservices/workplace-integration/ip/IP-004-offer-letter-rest-api.md`
80. `microservices/workplace-integration/ip/IP-005-jurisdiction-clause-engine.md`
81. `microservices/workplace-integration/ip/IP-006-roster-binding-domain.md`
82. `microservices/workplace-integration/ip/IP-007-clock-geofence-adapter.md`
83. `microservices/workplace-integration/ip/IP-008-informed-consent-domain.md`
84. `microservices/workplace-integration/ip/IP-009-closing-package-worker.md`
85. `microservices/workplace-integration/ip/IP-010-engagement-agreement-usecase.md`
86. `microservices/workplace-integration/ip/IP-011-program-identity-port.md`
87. `microservices/workplace-integration/ip/IP-012-office-barrier-policy.md`
88. `microservices/workplace-integration/ip/IP-013-dlp-trace-domain.md`
89. `microservices/workplace-integration/ip/IP-014-audit-chain-seal-usecase.md`
90. `microservices/workplace-integration/ip/IP-015-async-esign-events.md`
91. `microservices/workplace-integration/ip/IP-016-grpc-roster-reader.md`
92. `microservices/workplace-integration/ip/IP-017-cedar-default-deny-pack.md`
93. `microservices/workplace-integration/ip/IP-018-identity-provisioning-port.md`
94. `microservices/workplace-integration/ip/IP-019-drive-archive-adapter.md`
95. `microservices/workplace-integration/ip/IP-020-mail-delivery-adapter.md`
96. `microservices/workplace-integration/ip/IP-021-multi-region-replay.md`
97. `microservices/workplace-integration/ip/IP-022-sovereign-pack-overlay.md`
98. `microservices/workplace-integration/ip/IP-023-dashboard-and-slo-pack.md`
99. `microservices/workplace-integration/ip/IP-024-catalog-and-manifest-pack.md`
100. `microservices/workplace-integration/ip/IP-025-load-and-failure-fixtures.md`
101. `microservices/workplace-integration/manifest.json`
102. `microservices/workplace-integration/migration-playbooks/from-rippling-and-gusto.md`
103. `microservices/workplace-integration/multi-region.md`
104. `microservices/workplace-integration/onboarding/hris-engineer-first-week.md`
105. `microservices/workplace-integration/policies/clock-attest.cedar`
106. `microservices/workplace-integration/policies/dlp-trace-seal.cedar`
107. `microservices/workplace-integration/policies/esign-initiate.cedar`
108. `microservices/workplace-integration/policies/esign-sign.cedar`
109. `microservices/workplace-integration/policies/offer-generate.cedar`
110. `microservices/workplace-integration/policies/roster-bind.cedar`
111. `microservices/workplace-integration/reference-implementations/hire-onboard-payroll-rust-sdk.md`
112. `microservices/workplace-integration/runbooks/clock-geofence-dispute.md`
113. `microservices/workplace-integration/runbooks/clock-in-geofence-failure-cascade.md`
114. `microservices/workplace-integration/runbooks/closing-package-archive-failure.md`
115. `microservices/workplace-integration/runbooks/dlp-egress-trace-replay.md`
116. `microservices/workplace-integration/runbooks/e-sign-session-corruption-recovery.md`
117. `microservices/workplace-integration/runbooks/engagement-agreement-dual-signature.md`
118. `microservices/workplace-integration/runbooks/esign-session-stalled.md`
119. `microservices/workplace-integration/runbooks/offer-generation-clause-drift.md`
120. `microservices/workplace-integration/runbooks/office-barrier-deny-spike.md`
121. `microservices/workplace-integration/runbooks/program-identity-auto-revoke.md`
122. `microservices/workplace-integration/runbooks/roster-binding-revocation.md`
123. `microservices/workplace-integration/runbooks/shift-schedule-conflict-resolution.md`
124. `microservices/workplace-integration/runbooks/signature-proof-mismatch.md`
125. `microservices/workplace-integration/scorecards/overrides.json`
126. `microservices/workplace-integration/sdk-plan.md`
127. `microservices/workplace-integration/slos/clock-attestation-availability.openslo.yaml`
128. `microservices/workplace-integration/slos/dlp-trace-seal-fidelity.openslo.yaml`
129. `microservices/workplace-integration/slos/esign-initiate-availability.openslo.yaml`
130. `microservices/workplace-integration/slos/offer-generation-latency.openslo.yaml`
131. `microservices/workplace-integration/slos/roster-binding-accuracy.openslo.yaml`
132. `microservices/workplace-integration/slos/signature-capture-latency.openslo.yaml`
133. `microservices/workplace-integration/src/lib.rs`
134. `microservices/workplace-integration/threat-model.md`
135. `microservices/workplace-integration/tutorials/hire-onboard-clock-in-payroll-cycle.md`

## §2.1 Artifact-class coverage inventory
1. Root product docs are present: PRD, ARCHITECTURE, README, compliance, capacity, cost, failure modes, incident response, DPIA, threat model, SDK plan, multi-region, and replay docs.
2. Contract docs are present: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3.
3. Policy docs are present: six Cedar policy files for e-sign, offer, roster, clock, and DLP trace actions.
4. SLO docs are present: six OpenSLO files.
5. Runbook docs are present: thirteen operational runbooks.
6. Journey docs are present: sixteen root journey IP files and twenty-five implementation-plan files under `ip/`.
7. Capability docs are present: six capability YAML files.
8. Marketplace/counterpart docs are present but misaligned: `competitor-parity-matrix.md` uses Workday/DocuSign/SAP/FINRA precedent at `competitor-parity-matrix.md:16-20`.
9. Existing benchmark docs are present but use Rippling, Gusto, Workday, Justworks, and Deel, not the assigned Slack/Teams/Zapier set at `benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:1-5`.
10. tenant_class docs are present but retired: `ADR-0330 and ADR-0331 tenant_class model` is entirely based on the retired ladder at `ADR-0330 and ADR-0331 tenant_class model:10-72`.
11. Onboarding docs are present and include retired tenant_class assumptions at `onboarding/hris-engineer-first-week.md:12` and `:24`.
12. Tutorial docs are present and include retired tenant_class assumptions at `tutorials/hire-onboard-clock-in-payroll-cycle.md:3-8`.
13. Migration docs are present and include retired tenant_class CLI flags at `migration-playbooks/from-rippling-and-gusto.md:38-43`.
14. Source code is present but minimal at `src/lib.rs:1-56`.
15. A dedicated `tests/` directory is absent from the inventory.
16. A dedicated `cross-microservice-handoffs.md` file is absent from the inventory despite ten manifest dependencies at `manifest.json:47-58`.
17. Per-context IaC subdirectories are absent from the inventory.
18. OCI Always Free subprofile IaC is absent from the inventory.
19. OS support manifest is absent from the inventory.
20. Tenant-class behavior document is absent from the inventory.

## §3 Nine-dimension audit

### §3.1 Dimension 1 — Internal coherence
1. Verdict: REVISE.
2. The core README and PRD agree on a narrowed regulated workplace-evidence substrate at `README.md:16` and `PRD.md:17-22`.
3. The chat history originally described a broader workplace integration layer covering clocking, approvals, e-signing, meetings, expense reports, and delivery through Mail/Messenger/Calendar/Meet at chat lines 1355 and 1444.
4. The architecture document lists dependencies on mail, drive, workflow-engine, community, compliance, audit-chain, marketplace, payments, and tenancy at `ARCHITECTURE.md:37-47`.
5. The manifest repeats the dependency list at `manifest.json:47-58`.
6. No local cross-microservice handoff file records ownership boundaries for those dependencies.
7. The current local docs therefore describe "uses typed contract only" boundaries, but do not prove reciprocal handoffs.
8. The PRD says every state transition emits metrics, traces, logs, and audit-chain events at `PRD.md:65-73`.
9. The OpenAPI file repeats one audit event for all seven mutation routes at `contracts/openapi-v1.yaml:28`, `:52`, `:76`, `:100`, `:124`, `:148`, and `:172`.
10. The AsyncAPI file defines seven distinct events at `contracts/asyncapi-v1.yaml:18-52`.
11. The architecture event table also defines those seven distinct events at `ARCHITECTURE.md:104-113`.
12. The OpenAPI mapping is therefore internally inconsistent with the event contract and architecture event table.
13. The SLO files are internally shifted: clock availability points at DLP metrics, DLP fidelity points at e-sign-initiate metrics, and signature latency points at offer metrics.
14. These shifts appear in `slos/clock-attestation-availability.openslo.yaml:19-26`, `slos/dlp-trace-seal-fidelity.openslo.yaml:19-26`, and `slos/signature-capture-latency.openslo.yaml:19-26`.
15. The architecture document includes repeated route/journey mismatches near the end of the file.
16. Example: `ARCHITECTURE.md:1475-1476` says "j37 Clock In Geofence" enters through `/workplace/esign/sessions/{session_id}/sign`.
17. Example: `ARCHITECTURE.md:1483-1484` says "j38 E Sign Session" enters through `/workplace/offer-letters`.
18. Example: `ARCHITECTURE.md:1507-1508` says "j56 Offer E Sign" enters through `/workplace/clock-events`.
19. These are not only wording issues because the route determines contracts, Cedar actions, audit event class, SLO, and runbook ownership.
20. The ADR for clock-in geofence is locally substantive and names wage-risk concerns, privacy concerns, and geofence policy at `decisions/ADR-WPI-001-clock-in-geofence-with-tolerance-vs-strict-vs-flexible-tenant_class-matrix.md:21-52`.
21. The ADR also uses generic policy "tenant_class" terminology throughout the decision at `decisions/ADR-WPI-001-clock-in-geofence-with-tolerance-vs-strict-vs-flexible-tenant_class-matrix.md:55-75`.
22. That policy-tenant_class vocabulary is distinct from demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating, but it is currently ambiguous because the rest of the corpus is retiring tenant_class terminology.
23. Recommended interpretation: rename ADR policy tiers to `clock_policy_profile` or another non-tenant_class term in Wave 15J or a µservice-local cleanup.
24. The FAQ claims the µservice can run payroll by tenant tenant_class at `faqs/hris-engineer-faq.md:14-22`.
25. The PRD says replacing payments, treasury, identity, audit-chain, workflow-engine, mail, drive, or compliance is out of scope at `PRD.md:97-100`.
26. The PRD does not explicitly exclude payroll ownership in the out-of-scope list at `PRD.md:97-100`, while the FAQ says native payroll engine lives in service crates at `faqs/hris-engineer-faq.md:22`.
27. That is a scope-risk contradiction because `payroll` is a separate microservice in the workspace.
28. The service needs a single product truth: either payroll calculation is out of scope and this service exports evidence, or it owns bounded workforce payroll adapters with explicit cross-service contracts.
29. Current docs contain both claims.
30. Dimension 1 finding count contribution: P1 findings F-WPI-001, F-WPI-004, F-WPI-005, F-WPI-006, and F-WPI-008.

### §3.2 Dimension 2 — Outbound cross-references
1. Verdict: REVISE.
2. The manifest dependency list names ten dependent µservices at `manifest.json:47-58`.
3. The architecture list names the same ten dependencies at `ARCHITECTURE.md:37-47`.
4. The PRD says the service is not a replacement for payments, treasury, identity, audit-chain, workflow-engine, mail, drive, or compliance at `PRD.md:97-100`.
5. The service still lacks `cross-microservice-handoffs.md` in the inventory.
6. The ownership-coherence directive expects cross-service handoffs to be verified across µservice-local artifacts at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:47-58`.
7. The missing handoff file is especially material because this service touches HR, payroll-like data, identity, mail, drive archival, workflow orchestration, audit-chain, marketplace, payments, and tenancy.
8. The PRD points to ADR-0320, ADR-0244, ADR-0243, and ADR-0263 at `PRD.md:21-22`.
9. The architecture and manifest cite ADR-0105, ADR-0131, ADR-0243, ADR-0244, ADR-0263, ADR-0319, and ADR-0320 at `ARCHITECTURE.md:7-11` and `manifest.json:7-16`.
10. The local ADR cites root ADRs including tenant scoping, Cedar, audit-chain, data class, cross-region, idempotency, and observability at `decisions/ADR-WPI-001-clock-in-geofence-with-tolerance-vs-strict-vs-flexible-tenant_class-matrix.md:7-16`.
11. Those outbound ADR references are mostly coherent.
12. The benchmark file uses a different counterpart family at `benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:1-5`.
13. The chat queue assigns Slack App Directory, Microsoft Teams App Store, and Zapier Integrations at chat line 16311.
14. The existing competitor parity matrix uses Workday HCM, DocuSign, SAP SuccessFactors, and FINRA information-barrier supervision at `competitor-parity-matrix.md:16-20`.
15. Those are not wrong references for the current HR/e-sign surface, but they are not the requested Wave 3 Batch 3.2 counterpart set.
16. The service needs a counterpart-family decision: HR lifecycle system, app/integration marketplace, or a hybrid workplace-domain integration adapter.
17. Current docs cite all three families without reconciling them.
18. Dimension 2 finding count contribution: P1 finding F-WPI-008 and P2 finding F-WPI-011.

### §3.3 Dimension 3 — Substance bar
1. Verdict: PARTIAL.
2. The PRD and ARCHITECTURE files each exceed 1,500 lines and include endpoint, persona, event, and dependency material.
3. Volume alone is not sufficient under `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-12`.
4. The README is concise and accurately points to contracts, policies, runbooks, dashboards, SLOs, and IPs at `README.md:18-25`.
5. The capacity model has a real purpose line at `capacity-model.md:16-20`, but the rest of the file is a 36-line scaffold also shared by cost, failure modes, incident response, and DPIA.
6. The cost budget purpose line at `cost-budget.md:16-20` is specific, but it contains no actual per-million operation cost numbers in the inspected 36-line file.
7. The failure-modes file purpose line at `failure-modes.md:16-20` names denied/deferred/replaying/quarantined/compensated/revoked states, but it does not enumerate route-specific failure trees.
8. The incident-response file purpose line at `incident-response.md:16-20` names alert paths and runbooks, but the file lacks concrete escalation timings, severity routing, or decision tree content.
9. The DPIA purpose line at `dpia.md:16-20` names personal data and DSAR, but the inspected file lacks data categories, retention bases, lawful bases, processor/subprocessor matrix, and residual-risk decisions.
10. The scaffold repetition across these root docs is exactly the anti-pattern described in `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-18`.
11. The local ADR is more substantive than those root operational docs because it names regulatory pressures and alternatives at `decisions/ADR-WPI-001-clock-in-geofence-with-tolerance-vs-strict-vs-flexible-tenant_class-matrix.md:21-138`.
12. The FAQ is substantive for HRIS/payroll/e-sign questions at `faqs/hris-engineer-faq.md:7-184`.
13. The migration playbook is operationally concrete for Rippling/Gusto migrations at `migration-playbooks/from-rippling-and-gusto.md:1-130`.
14. The tutorial is concrete enough for a dev-cell happy path at `tutorials/hire-onboard-clock-in-payroll-cycle.md:1-130`.
15. The benchmark document contains numeric claims and competitor comparisons, but it is tied to the retired tenant_class model and an obsolete counterpart set at `benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:1-112`.
16. The source file is intentionally scaffold-like, not product implementation, at `src/lib.rs:24-37`.
17. The tests in `src/lib.rs:39-56` verify declarations only.
18. No behavior tests exist for clock attempts, signature proof, roster binding, DLP trace sealing, tenant-class caps, or deployment contexts.
19. The service has enough docs to infer intended scope, but not enough coherent implementation detail to safely hand to an engineer for all claimed surfaces.
20. Dimension 3 finding count contribution: P2 findings F-WPI-010, F-WPI-012, F-WPI-013, and F-WPI-018.

### §3.4 Dimension 4 — Canonical-direction alignment
1. Verdict: REVISE.
2. Multi-context doctrine requires every µservice PRD/ADR/IP to specify supported deployment contexts; absence is an audit finding under `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_multi_context_provider_agnostic_2026_05_20.md:32-38`.
3. The master sequence names six contexts at `specs/master-plan-sequencing.json:704-745`.
4. The service manifest does not contain deployment context declarations in `manifest.json:1-176`.
5. OpenTofu doctrine requires zero handroll and no Terraform binary references at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md:10-17`.
6. This service has Terraform-named HCL files and `null_resource` usage at `iac/terraform-main.tf:1-13` and `iac/terraform-variables.tf:1-13`.
7. OS doctrine requires a µservice-level OS support manifest at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:56-76`.
8. This service has no such manifest.
9. Rust-strict doctrine forbids authored Python, JavaScript, TypeScript, Ruby, Java, Go, and similar backend logic at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:51-60`.
10. This service has no forbidden language source files by extension scan.
11. OCI doctrine requires OCI Always Free maximization for the OCI profile and an Always Free module at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md:65-80`.
12. This service has no `iac/oci-guest/always-free/` path.
13. tenant_class-retirement doctrine retires the demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating ladder at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_tenant_class_adoption_2026_05_20.md:10-24`.
14. This service has 29 direct demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating references.
15. Tenant-class doctrine for this batch is the three-class replacement model in the user directive: `demo_trial`, `paid`, and `revenue_share`.
16. The local search for `tenant_class`, `demo_trial`, `revenue_share`, `per-seat`, `per_seat`, and usage billing semantics returned zero matches under this µservice path.
17. The current docs still frame capability differentiation through retired feature tiers rather than uniform quality plus tenant-class deployment/billing overlays.
18. Dimension 4 finding count contribution: P1 findings F-WPI-002, F-WPI-003, F-WPI-007 and P2 findings F-WPI-009, F-WPI-014, F-WPI-015.

#### §3.4.T tenant_class retirement candidates
Default classification: Wave 15J retirement candidate, P2 documentation gap, unless the same line also participates in a higher-severity contradiction.
1. `onboarding/hris-engineer-first-week.md:12` references "paid with compliance_pack gating healthcare e-sign requirements."
2. `onboarding/hris-engineer-first-week.md:24` uses `TENANT_CLASS=paid with per_seat billing_component`.
3. `migration-playbooks/from-rippling-and-gusto.md:41` uses `--tenant-class paid with per_seat billing_component`.
4. `benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:10` labels workplace-integration as `(paid with per_usage billing_component)`.
5. `benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:21` labels e-sign features as `(paid with per_usage billing_component)`, `(paid with compliance_pack gating)`, and `(paid with per_usage billing_component)`.
6. `benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:33` labels payroll coverage as `(paid with per_usage billing_component)`.
7. `benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:44` labels payroll latency as `(paid with per_usage billing_component)`.
8. `benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:55` labels compliance packs as `(paid with compliance_pack gating)`.
9. `benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:66` labels TCO as `(paid with per_usage billing_component)`.
10. `benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:90` says EU AI Act is native at paid with compliance_pack gating.
11. `benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:92` says tenant_class-based promotion starts demo_trial and grows to paid with compliance_pack gating.
12. `tutorials/hire-onboard-clock-in-payroll-cycle.md:3` says the tutorial is for a paid with per_seat billing_component tenant.
13. `tutorials/hire-onboard-clock-in-payroll-cycle.md:8` uses `TENANT_CLASS=paid with per_seat billing_component`.
14. `ADR-0330 and ADR-0331 tenant_class model:10` has heading `tenant_class demo_trial`.
15. `ADR-0330 and ADR-0331 tenant_class model:26` has heading `tenant_class paid with per_seat billing_component`.
16. `ADR-0330 and ADR-0331 tenant_class model:42` has heading `tenant_class paid with per_usage billing_component`.
17. `ADR-0330 and ADR-0331 tenant_class model:58` has heading `tenant_class paid with compliance_pack gating`.
18. `ADR-0330 and ADR-0331 tenant_class model:63` says features are "full paid with per_usage billing_component" plus additional controls.
19. `ADR-0330 and ADR-0331 tenant_class model:87` says EU AI Act pack is active at "paid with per_usage billing_component".
20. `faqs/hris-engineer-faq.md:17` says demo_trial has no native payroll.
21. `faqs/hris-engineer-faq.md:18` says paid with per_seat billing_component has native US and UK payroll.
22. `faqs/hris-engineer-faq.md:19` says paid with per_usage billing_component has native multi-country payroll.
23. `faqs/hris-engineer-faq.md:20` says paid with compliance_pack gating has regulator-cleared partners.
24. `faqs/hris-engineer-faq.md:35` says KR/Japan/India/South Africa signature support is at paid with per_usage billing_component.
25. `faqs/hris-engineer-faq.md:46` says demo_trial uses two clock signals and paid with per_seat billing_component adds a third.
26. `faqs/hris-engineer-faq.md:56` says KR visa checks are sovereign tenants only and paid with compliance_pack gating.
27. `faqs/hris-engineer-faq.md:94` says paid with compliance_pack gating healthcare tenants may need HIPAA retention.
28. `faqs/hris-engineer-faq.md:114` says paid with per_seat billing_component can use legacy e-sign partners.
29. `faqs/hris-engineer-faq.md:162` says on-prem self-hosting is paid with compliance_pack gating-only.

#### §3.4.C Tenant-class adoption gaps
1. Current expected model for this batch: `demo_trial`, `paid`, and `revenue_share`.
2. Current artifact state: no direct `tenant_class` field or tenant-class enum was found under `microservices/workplace-integration/`.
3. Current artifact state: no `demo_trial` reference was found under `microservices/workplace-integration/`.
4. Current artifact state: no `revenue_share` reference was found under `microservices/workplace-integration/`.
5. Current artifact state: no per-seat or usage billing semantics were found under `microservices/workplace-integration/`.
6. The manifest declares `"tenant_class": "product"` at `manifest.json:6`, but not tenant-class semantics.
7. The capability YAMLs declare `tenant_class: T0/T1/T2/T3` at `capabilities/clock-attest.yaml:4`, `capabilities/dlp-trace-seal.yaml:4`, `capabilities/esign-initiate.yaml:4`, `capabilities/esign-sign.yaml:4`, `capabilities/offer-generate.yaml:4`, and `capabilities/roster-bind.yaml:4`.
8. Those T0/T1/T2/T3 values look like capability criticality labels, not customer feature tiers, but they need explicit naming to avoid conflict with tenant_class-retirement doctrine.
9. Required adoption gap: define how `demo_trial` usage caps apply to e-sign sessions, clock events, DLP traces, roster bindings, and offer-letter generation.
10. Required adoption gap: define how `paid` tenants scale by contract, SLO, compliance packs, BYOK, and per-seat/usage billing.
11. Required adoption gap: define how `revenue_share` tenants meter marketplace-seller, B2C, embedded-SaaS, affiliate, or reseller workloads when this µservice emits workplace events.
12. Required adoption gap: define whether the tenant class is visible only via IAM principal claims or also in audit event metadata.
13. Required adoption gap: define how `demo_trial` uses the OCI Always Free profile without the retired "OCI Always Free demo_trial" wording.
14. Required adoption gap: define whether this service has an own usage meter or only emits raw usage events to cloud-billing.
15. Required adoption gap: define contract tests for cap-hit behavior under `demo_trial` and no-cap scaling under `paid` and `revenue_share`.

### §3.5 Dimension 5 — Industry-counterpart parity
1. Verdict: REVISE.
2. Assigned counterpart set: Slack App Directory, Microsoft Teams App Store, Zapier Integrations.
3. Slack App Directory/Marketplace provides discovery, install governance, permissions, app home/messages/shortcuts/commands, app messages, agents/assistants, pricing disclosures, and security/compliance review surfaces; Slack's help page says users can browse more than 2,000 apps in the Slack Marketplace at https://slack.com/help/articles/360001537467-Guide-to-apps-in-Slack lines 47-60.
4. Slack distribution docs distinguish undistributed, unlisted distributed, and listed Marketplace apps and require OAuth/SSL/onboarding for distribution at https://docs.slack.dev/app-management/distribution/ lines 64-105.
5. Microsoft Teams app store provides app discovery, category browse, app install contexts, admin allow/block governance, tabs, webhooks/connectors, messaging extensions, meeting extensions, bots, cards/task modules, and activity feeds; see https://learn.microsoft.com/pl-pl/microsoftTeams/apps-in-teams lines 84-119.
6. Microsoft Teams admin center provides allow/block, custom app upload, app assignment, compliance/certification evidence, support info, and app catalog export at https://learn.microsoft.com/en-us/microsoftteams/manage-apps lines 32-50 and 112-132.
7. Zapier Integrations provide authentication, triggers, actions, searches, public API integration, Platform UI, and Platform CLI; see https://docs.zapier.com/integrations/quickstart/how-zapier-works lines 130-147.
8. Zapier recommended integration docs require foundational triggers, actions, searches, and search-or-create surfaces by app category at https://docs.zapier.com/integrations/quickstart/recommended-triggers-and-actions lines 162-178.
9. The current workplace-integration docs do not own a public app directory, install approval workflow, app package submission, third-party app security review, app-scoped OAuth installation flow, or Zap-style trigger/action/search catalog.
10. The current docs do have domain workflows that could become integration events: e-sign session created, signature captured, offer generated, agreement bound, roster binding granted, clock event attested, and DLP trace sealed at `contracts/asyncapi-v1.yaml:18-52`.
11. The current docs have a workplace-domain API surface at `contracts/openapi-v1.yaml:21-189`.
12. Parity gap is therefore not "missing e-sign"; it is "missing integration marketplace/app surface."
13. Existing `benchmarks/` and `competitor-parity-matrix.md` compare the service to HRIS/e-sign vendors, not to the required app/integration counterparts.
14. If the intended product is a regulated HRIS/workplace substrate, the counterpart set should be revised by the orchestrator.
15. If the counterpart set is binding, the µservice must expand or delegate an app-directory/integration-adapter layer.
16. The feature-parity matrix in the companion deliverable treats current docs as a domain service and lists additive surfaces needed to meet the Slack/Teams/Zapier union.
17. Dimension 5 finding count contribution: P1 finding F-WPI-001 and P2 finding F-WPI-011.

### §3.6 Dimension 6 — Multi-context deployment
1. Verdict: FAIL.
2. Canonical context ids are `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider` at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3854-3871`.
3. The master sequence maps those contexts to IaC targets at `specs/master-plan-sequencing.json:704-745`.
4. `manifest.json:1-176` has no `deployment_contexts` key.
5. `README.md:1-53` has no context support section.
6. `PRD.md:1-130` has no explicit six-context deployment support table in the inspected front matter and functional sections.
7. `ARCHITECTURE.md:1-180` names tenancy, region, and cell facts, but not all six deployment contexts.
8. `faqs/hris-engineer-faq.md:151-163` mentions sovereign and on-prem in a retired tenant_class context, not six-context canonical support.
9. The `iac/` directory scan found only the root `iac` directory and no per-context subdirectories.
10. Required context path `iac/oyatie-public-cloud/` is absent.
11. Required context path `iac/guest-on-aws/` is absent.
12. Required context path `iac/oci-guest/` is absent.
13. Required context path `iac/on-prem/` is absent.
14. Required context path `iac/colo/` is absent.
15. Required context path `iac/oyatie-iaas/` is absent.
16. Required OCI Always Free subpath `iac/oci-guest/always-free/` is absent.
17. The service therefore cannot honestly claim all six deployable contexts from current artifacts.
18. No N/A rationale is present for any context.
19. Under ADR-0328 severity mapping, a non-P0 µservice with false deployment support is P1 at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4106-4111`.
20. Dimension 6 finding count contribution: P1 finding F-WPI-002.

### §3.7 Dimension 7 — OpenTofu IaC
1. Verdict: FAIL.
2. Canonical engine is OpenTofu at `specs/master-plan-sequencing.json:747-756`.
3. Forbidden patterns include `null_resource`, `local-exec`, SSH provisioners, hand-edited tfstate, and unsigned modules at `specs/master-plan-sequencing.json:767-774`.
4. ADR-0328 forbids Terraform as engine or binary at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3897-3901`.
5. ADR-0328 requires `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, README, signing evidence, backend evidence, and cloud-iac orchestration at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3908-3939`.
6. Current file `iac/terraform-main.tf` is Terraform-named.
7. `iac/terraform-main.tf:3` mentions `terraform-main.tf`.
8. `iac/terraform-main.tf:6` declares `resource "null_resource"`.
9. `iac/terraform-main.tf:10` encodes `workplace-integration.iac.terraform-main`.
10. Current file `iac/terraform-variables.tf` is Terraform-named.
11. `iac/terraform-variables.tf:3` mentions `terraform-variables.tf`.
12. `iac/terraform-variables.tf:6` declares `resource "null_resource"`.
13. `iac/terraform-variables.tf:10` encodes `workplace-integration.iac.terraform-variables`.
14. There is no `versions.tf` file in any context path.
15. There is no `outputs.tf` file in any context path.
16. There is no context-specific state backend.
17. There is no module signing evidence.
18. There is no cloud-iac orchestration reference for tenant onboarding.
19. There is no OCI Always Free module.
20. Dimension 7 finding count contribution: P1 finding F-WPI-003.

### §3.8 Dimension 8 — OS support
1. Verdict: FAIL.
2. Master sequence requires supported OS declarations and marks per-µservice manifest coverage required at `specs/master-plan-sequencing.json:777-815`.
3. ADR-0328 requires `supported-oses.json` at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3950-3954`.
4. ADR-0328 requires Talos, RHEL, Oracle Linux, SLES, Ubuntu, Debian, Rocky, Alma, CentOS Stream, Amazon Linux, Flatcar, Photon, and macOS Apple Silicon M5+ posture at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3954-3979`.
5. ADR-0328 requires explicit out-of-scope entries and architecture matrix at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3981-4000`.
6. File search found no `supported-oses.json` or `supported_oses.json` under the µservice.
7. `manifest.json:1-176` has no OS matrix.
8. `README.md:1-53` has no OS matrix.
9. `Cargo.toml` exists, but the audit did not find package/build matrix docs for the thirteen primary OS entries.
10. The source file's Rust-only scaffold is portable in principle, but documentation does not declare portability or packaging.
11. No Talos profile is declared.
12. No Oracle Linux/UEK profile is declared.
13. No Amazon Linux guest-on-AWS profile is declared.
14. No macOS Apple Silicon M5+ support or exclusion statement is declared.
15. No explicit Intel macOS/pre-M5/FreeBSD/OpenBSD/Windows Server/Solaris out-of-scope list is present.
16. Dimension 8 finding count contribution: P1 finding F-WPI-007.

### §3.9 Dimension 9 — Rust-strict language policy
1. Verdict: PASS WITH CAVEAT.
2. Master sequence requires backend strict Rust and forbids Python, JavaScript application logic, TypeScript application logic, Ruby, Perl, PHP, Java, Scala, Groovy, Go, and F# at `specs/master-plan-sequencing.json:817-856`.
3. ADR-0328 allows `.tf`, `.cedar`, `.yaml`, `.json`, `.proto`, OpenAPI, AsyncAPI, OpenSLO, SQL, and Markdown at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4011-4032`.
4. Extension scan found no `*.py`, `*.js`, `*.ts`, `*.rb`, `*.go`, `*.java`, `*.scala`, `*.groovy`, `*.php`, or `*.fs` files under this µservice.
5. Rust source exists at `src/lib.rs:1-56`.
6. `src/lib.rs:1` forbids unsafe code.
7. `src/lib.rs:3-8` declares service and contract constants.
8. `src/lib.rs:9-22` declares the local layer enum slice.
9. `src/lib.rs:39-56` has two unit tests for declarations.
10. The proto file includes `option java_multiple_files`, `java_package`, and `go_package` at `contracts/workplace-integration-v1.proto:5-7`.
11. These proto options are not authored Java or Go source files.
12. The caveat is provenance: generated Java/Go SDK output would need an explicit generated-client surface and no backend runtime dependency.
13. The current audit does not classify those proto options as a language violation.
14. Dimension 9 finding count contribution: P3 finding F-WPI-017.

## §4 Findings table
| ID | Severity | Dimension(s) | Finding | Evidence | Required correction shape |
|---|---:|---|---|---|---|
| F-WPI-001 | P1 | 1,5 | Product surface is split between the current HR/workplace agreement substrate and the assigned Slack/Teams/Zapier integration-marketplace bar. | README scope at `README.md:16`; PRD scope at `PRD.md:17-22`; chat cross-cutting workplace layer at chat lines 1355 and 1444; assigned counterpart set at chat line 16311. | Decide whether this service is a workplace-domain substrate with integration adapters or a broader workplace app/integration marketplace; update PRD, architecture, benchmark, and parity docs accordingly. |
| F-WPI-002 | P1 | 4,6 | Six deployment contexts are not declared, and no per-context IaC directories or N/A rationales exist. | Canonical contexts at `specs/master-plan-sequencing.json:704-745`; manifest has no context fields at `manifest.json:1-176`; IaC directory scan only found root `iac/`; ADR requires paths at `docs/decisions/ADR-0328...md:3854-3871`. | Add service-local deployment-context manifest and OpenTofu modules or explicit N/A rationale for each context. |
| F-WPI-003 | P1 | 4,7 | IaC violates OpenTofu-only doctrine through Terraform-named files and `null_resource`. | `iac/terraform-main.tf:1-13`; `iac/terraform-variables.tf:1-13`; canonical forbidden patterns at `specs/master-plan-sequencing.json:747-775`; ADR `null_resource` ban at `docs/decisions/ADR-0328...md:3928-3939`. | Replace flat Terraform-named scaffolds with signed OpenTofu modules under canonical context paths, with versions, outputs, backend, and cloud-iac orchestration. |
| F-WPI-004 | P1 | 1,3 | OpenAPI maps every mutation route to `WorkplaceESignSessionCreated`, contradicting AsyncAPI and architecture event model. | OpenAPI audit events at `contracts/openapi-v1.yaml:28`, `:52`, `:76`, `:100`, `:124`, `:148`, `:172`; AsyncAPI event channels at `contracts/asyncapi-v1.yaml:18-52`; architecture event table at `ARCHITECTURE.md:104-113`. | Align each OpenAPI route to the event it actually emits and add contract tests for route-event mapping. |
| F-WPI-005 | P1 | 1,3 | OpenSLO files are metric-shifted across routes, so evidence would measure the wrong operation. | Clock availability reads DLP metrics at `slos/clock-attestation-availability.openslo.yaml:19-26`; DLP fidelity reads e-sign-initiate metrics at `slos/dlp-trace-seal-fidelity.openslo.yaml:19-26`; signature latency reads offer metrics at `slos/signature-capture-latency.openslo.yaml:19-26`; offer latency reads roster metrics at `slos/offer-generation-latency.openslo.yaml:19-26`; roster accuracy reads clock metrics at `slos/roster-binding-accuracy.openslo.yaml:19-26`. | Rewire each SLO to its matching capability metric and add validation that SLO name, route, capability metric, and runbook agree. |
| F-WPI-006 | P1 | 1,3 | Architecture route-to-journey mappings drift near the end of the file, likely from repeated template rotation. | `ARCHITECTURE.md:1475-1476` maps clock-in geofence to e-sign sign route; `ARCHITECTURE.md:1483-1484` maps e-sign session to offer letters; `ARCHITECTURE.md:1507-1508` maps offer e-sign to clock events. | Rebuild the journey-to-route table from the OpenAPI and journey IPs; delete rotated rows. |
| F-WPI-007 | P1 | 4,8 | Supported-OS manifest is absent. | Required OS manifest at `specs/master-plan-sequencing.json:777-815`; ADR check at `docs/decisions/ADR-0328...md:3950-4000`; no `supported-oses.json`; manifest has no `supported_oses` at `manifest.json:1-176`. | Add supported-OS manifest with required OS, arch, packaging, CI, and out-of-scope entries. |
| F-WPI-008 | P1 | 1,2 | Cross-microservice handoff artifact is missing despite ten dependencies and broad workflow claims. | Dependencies in `manifest.json:47-58` and `ARCHITECTURE.md:37-47`; no `cross-microservice-handoffs.md` in inventory; handoff expectation at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:47-58`. | Add service-local handoff matrix for identity, mail, drive, workflow-engine, community, compliance, audit-chain, marketplace, payments, and tenancy. |
| F-WPI-009 | P2 | 4 | 29 direct retired demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating references remain. | Full candidate list in §3.4.T; retirement doctrine at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_tenant_class_adoption_2026_05_20.md:10-24`. | Wave 15J scrub: replace feature-tenant_class claims with uniform quality plus tenant-class usage/cost/compliance overlays. |
| F-WPI-010 | P2 | 4 | Tenant-class semantics are absent. | No tenant-class search hits; manifest lacks tenant-class at `manifest.json:1-176`; batch model requires `demo_trial`, `paid`, and `revenue_share`. | Add tenant-class behavior section and tests for demo caps, paid scale, and revenue-share cost/settlement path. |
| F-WPI-011 | P2 | 5 | Existing benchmark and competitor docs use HRIS/e-sign vendors, not assigned Slack/Teams/Zapier counterparts. | Benchmark title and source scope at `benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:1-5`; parity matrix precedent at `competitor-parity-matrix.md:16-20`; assigned queue at chat line 16311. | Retain HRIS evidence if product scope needs it, but add required integration-marketplace counterpart matrix and benchmark. |
| F-WPI-012 | P2 | 3 | Operational root docs have scaffold shape and thin service-specific details. | `capacity-model.md:16-36`; `cost-budget.md:16-36`; `failure-modes.md:16-36`; `incident-response.md:16-36`; `dpia.md:16-36`; anti-scaffold memory at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-18`. | Replace scaffolds with concrete capacity numbers, cost envelopes, failure trees, incident routing, DPIA data map, and residual risks. |
| F-WPI-013 | P2 | 3,9 | Implementation source is only a declaration scaffold and has no behavior tests beyond constants. | `src/lib.rs:1-56`; no `tests/` directory in inventory. | Add Rust implementation plan and tests for workplace events, idempotency, Cedar denial, tenant caps, and audit evidence. |
| F-WPI-014 | P2 | 4 | OCI Always Free profile is absent from service IaC. | Required module at `specs/master-plan-sequencing.json:857-868`; OCI memory requires per-service Always Free limits at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md:65-80`; no `iac/oci-guest/always-free/`. | Add `iac/oci-guest/always-free/` OpenTofu module or explicit N/A with resource math. |
| F-WPI-015 | P2 | 4 | Capability metadata uses ambiguous `tenant_class` labels that are not demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating but can collide with retirement work. | `manifest.json:6`; capability `tenant_class` fields at all six capability YAML `:4`; local ADR policy tenant_class fields at `decisions/ADR-WPI-001...md:55-75`. | Rename or document these as criticality/profile labels, not customer feature tiers. |
| F-WPI-016 | P2 | 1,3 | FAQ claims payroll ownership that can conflict with the separate payroll µservice and PRD out-of-scope posture. | FAQ payroll claims at `faqs/hris-engineer-faq.md:14-22`; PRD out-of-scope list at `PRD.md:97-100`; tutorial payroll flow at `tutorials/hire-onboard-clock-in-payroll-cycle.md:1-9`. | Clarify whether workplace-integration emits payroll evidence/export rows or owns payroll calculation adapters, and hand off to payroll µservice where needed. |
| F-WPI-017 | P3 | 9 | Proto includes Java/Go package options without generated-client provenance. | `contracts/workplace-integration-v1.proto:5-7`; Rust-strict distinction for generated clients at `docs/decisions/ADR-0328...md:4082-4083`. | Add generation provenance or remove unused non-Rust package options. |
| F-WPI-018 | P3 | 3 | OpenAPI server URL is an example-shaped endpoint. | `contracts/openapi-v1.yaml:18-20` uses `https://api.oyatie.example/workplace-integration/v1`. | Replace with canonical environment URL patterns or document that it is an example-only contract server. |

## §5 Open questions
1. Is workplace-integration intended to remain a regulated HR/workplace substrate, or must it become the workplace-specific app/integration marketplace layer against Slack/Teams/Zapier?
2. If it remains a workplace substrate, should Slack/Teams/Zapier be treated as delivery/integration channels rather than direct product peers?
3. If Slack/Teams/Zapier are binding peers, which µservice owns app submission, app listing, app install approval, OAuth scopes, third-party risk review, and integration directory search?
4. Does `marketplace` own generic app-directory mechanics while `workplace-integration` owns only workplace-domain triggers/actions?
5. Does `workflow-engine` own Zap-style no-code flow execution while `workplace-integration` owns domain events only?
6. Does `messenger` own Slack/Teams-style conversational surfaces while this service owns the domain payloads?
7. Does `mail` own e-sign delivery, or does this service own signed-document notification templates?
8. Does `drive` own signed-document archive, or does this service own evidence package retention?
9. Does `payroll` own all payroll calculations, or can this service own narrowly scoped payroll export rows?
10. Should `ClockGeofenceTierMatrix` be renamed to `ClockGeofencePolicyProfileMatrix` to avoid tenant_class-retirement ambiguity?
11. Which tenant-class source of truth should this service read: cloud-billing, tenancy, identity claims, or all three through gateway claims?
12. What are the exact `demo_trial` caps for e-sign sessions, clock events, DLP trace seals, roster bindings, and offer-letter generation?
13. What is the `revenue_share` business event for this service: per hire, per signed workplace document, per marketplace sale, per active worker, or no direct event?
14. Should the service publish usage events to cloud-billing for every mutating route?
15. Should the OCI Always Free profile include all six routes, or can DLP trace sealing be disabled for demo tenants for compliance reasons?
16. Which OSes require service-specific packaging versus container-only deployment?
17. Are the proto Java/Go package options retained only for future generated SDKs, or should Rust-only generated bindings be the only current target?
18. Should existing HRIS competitor material be moved to an appendix once Slack/Teams/Zapier parity docs land?
19. Should the flat operational docs be replaced with route-specific runbook/capacity/cost/DPIA sections, or split into smaller route-owned artifacts?
20. What evidence source will prove cross-microservice reciprocal handoffs without touching other µservices during this audit?

## §6 Evidence ledger for remediation planning
1. E-001: Service identity is explicit in `manifest.json:2-4`; remediation should preserve the `workplace-integration` slug and not split a new service from this audit alone.
2. E-002: README scope at `README.md:16` frames workplace agreements, e-sign, roster, and workforce integration; Slack/Teams/Zapier work should be integrated through this purpose or reassigned by a later owner decision.
3. E-003: PRD problem statement at `PRD.md:17-22` is regulated-workplace centered; this is compatible with app-directory peers only if the directory surface is for regulated workplace workflows.
4. E-004: Chat line 1355 describes clocking-in, approvals, e-signing, meetings, expense reports, and workflow orchestration; this is broader than current PRD route coverage.
5. E-005: Chat line 1444 describes Workflow Engine plus Mail/Messenger/Calendar/Meet UX; current manifest dependencies include workflow-engine and mail/drive/community style services but no ownership split.
6. E-006: Chat line 16311 assigns Slack App Directory, Microsoft Teams App Store, and Zapier Integrations as the comparison set; existing benchmark docs do not answer that comparison.
7. E-007: OpenAPI route `/workplace/esign/sessions` exists at `contracts/openapi-v1.yaml:21-31`; it can become a Zapier create action or Slack/Teams modal-backed command.
8. E-008: OpenAPI route `/workplace/esign/sessions/{session_id}/sign` exists at `contracts/openapi-v1.yaml:45-55`; it needs human-channel flow ownership before Slack/Teams parity can be claimed.
9. E-009: OpenAPI route `/workplace/offer-letters` exists at `contracts/openapi-v1.yaml:69-79`; this route maps naturally to HR workflow automation but not to app-directory installation.
10. E-010: OpenAPI route `/workplace/engagement-agreements` exists at `contracts/openapi-v1.yaml:93-103`; certification and e-sign compliance evidence should be attached before marketplace publication.
11. E-011: OpenAPI route `/workplace/roster-bindings` exists at `contracts/openapi-v1.yaml:117-127`; Teams admin controls and Slack workspace controls would need roster-to-channel policy.
12. E-012: OpenAPI route `/workplace/clock-events` exists at `contracts/openapi-v1.yaml:141-151`; location-sensitive clock actions require extra privacy disclosure in Slack/Teams/Zapier listings.
13. E-013: OpenAPI route `/workplace/dlp-traces` exists at `contracts/openapi-v1.yaml:165-175`; DLP trace sealing is evidence infrastructure, not a user-facing app-directory feature by itself.
14. E-014: AsyncAPI channel `workplace.esign.session.created.v1` at `contracts/asyncapi-v1.yaml:18-22` supplies a candidate external trigger.
15. E-015: AsyncAPI channel `workplace.signature.captured.v1` at `contracts/asyncapi-v1.yaml:23-27` supplies the strongest Zapier-style "signed document" trigger.
16. E-016: AsyncAPI channel `workplace.offer.generated.v1` at `contracts/asyncapi-v1.yaml:28-32` supports downstream onboarding automations.
17. E-017: AsyncAPI channel `workplace.agreement.bound.v1` at `contracts/asyncapi-v1.yaml:33-37` supports compliance and HRIS synchronization.
18. E-018: AsyncAPI channel `workplace.roster.binding.granted.v1` at `contracts/asyncapi-v1.yaml:38-42` supports channel/role provisioning but needs identity/tenancy handoff.
19. E-019: AsyncAPI channel `workplace.clock.event.attested.v1` at `contracts/asyncapi-v1.yaml:43-47` supports shift and payroll evidence, but payroll calculation remains outside current PRD scope.
20. E-020: AsyncAPI channel `workplace.dlp.trace.sealed.v1` at `contracts/asyncapi-v1.yaml:48-52` supports audit and compliance workflows.
21. E-021: AsyncAPI schemas require `tenant_id`, `sub_scope_path`, `event_id`, `occurred_at`, and `audit_chain_ref` at `contracts/asyncapi-v1.yaml:127-231`; this is strong internal evidence but not sufficient external integration contract.
22. E-022: The OpenAPI repeated `x-audit-event` value at `contracts/openapi-v1.yaml:28` starts a pattern repeated on every route; route-event conformance tests should be the first contract remediation.
23. E-023: Architecture event table at `ARCHITECTURE.md:104-113` has the correct event vocabulary; it should become the source for OpenAPI and SLO generation.
24. E-024: Architecture dependency list at `ARCHITECTURE.md:37-47` names ten dependencies; a handoff document is required before any integration-marketplace claim is safe.
25. E-025: `cross-microservice-handoffs.md` is absent from the 135-file inventory; this is the most direct ownership-coherence gap.
26. E-026: `capacity-model.md:16-36` is too generic to support benchmark claims; remediation should add per-route capacity and per-context overlays.
27. E-027: `cost-budget.md:16-36` does not express OCI Always Free profile limits or tenant-class cost behavior.
28. E-028: `failure-modes.md:16-36` does not enumerate Slack, Teams, Zapier, webhook, OAuth, or event replay failure modes.
29. E-029: `incident-response.md:16-36` does not name app store, marketplace, or external integration incident playbooks.
30. E-030: `dpia.md:16-36` does not enumerate geofence, signature proof, DLP trace, or app-directory metadata data categories.
31. E-031: `compliance.md` is present in inventory and should be cross-linked to Teams certification and Slack security review evidence during remediation.
32. E-032: `security.md` is present in inventory and should be tied to OAuth scope, app install, and webhook secret handling before external publication.
33. E-033: `threat-model.md` is present in inventory and should include Zapier REST Hook, Slack event retry, and Teams bot throttling threat paths.
34. E-034: `privacy.md` is present in inventory and should explicitly disclose app-directory channel data flows for workplace actions.
35. E-035: `data-retention.md` is present in inventory and should distinguish signed-document retention from app integration telemetry retention.
36. E-036: `manifest.json:47-58` lists dependencies but does not classify which dependency owns marketplace, workflow execution, channel delivery, billing, or tenant-class enforcement.
37. E-037: `manifest.json:59-69` lists contract files; contract conformance remediation should stay service-local.
38. E-038: `manifest.json:70-151` maps journey implementation plans; the count is useful, but route-to-journey drift in architecture proves the map needs validation.
39. E-039: `manifest.json:152-167` names capabilities and SLOs; these links should be validated after SLO metric rewiring.
40. E-040: `manifest.json:6` uses the key `tenant_class`; this is not a direct demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating reference but should be renamed or documented during tenant_class retirement.
41. E-041: Capability YAML files use `tenant_class: T0/T1/T2/T3` at line 4; keep only if these are criticality levels and not commercial capability availability.
42. E-042: ADR-WPI-001 title at `decisions/ADR-WPI-001-clock-in-geofence-with-tolerance-vs-strict-vs-flexible-tenant_class-matrix.md:3` contains `tenant_class adoption matrix`; rename to policy profile matrix during Wave 15J cleanup.
43. E-043: ADR-WPI-001 uses `strict_site`, `tolerant_site`, and `flexible_work` at `decisions/ADR-WPI-001...md:55-75`; these are policy profiles and should not become commercial tiers.
44. E-044: ADR-WPI-001 data shapes use `tenant_class` at `decisions/ADR-WPI-001...md:164-165`; renaming this field reduces future ambiguity.
45. E-045: `onboarding/hris-engineer-first-week.md:24` uses `TENANT_CLASS=paid with per_seat billing_component`; this should become tenant-class plus policy-profile setup if the workflow remains.
46. E-046: `migration-playbooks/from-rippling-and-gusto.md:41` uses `--tenant-class paid with per_seat billing_component`; migration CLI examples must be rewritten with tenant-class and policy profile flags.
47. E-047: `tutorials/hire-onboard-clock-in-payroll-cycle.md:8` uses `TENANT_CLASS=paid with per_seat billing_component`; tutorial remediation should keep scenario caps separate from quality claims.
48. E-048: `faqs/hris-engineer-faq.md:17-20` defines demo_trial/paid with per_seat billing_component/paid with per_usage billing_component/paid with compliance_pack gating behavior; this is a direct Wave 15J retirement candidate.
49. E-049: `ADR-0330 and ADR-0331 tenant_class model:10-58` is the strongest retired-tenant_class artifact and should be removed or replaced by tenant-class and policy-profile material.
50. E-050: The benchmark file uses wrong comparison vendors at `benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:1-5`; keep as secondary HRIS evidence only if product owner confirms.
51. E-051: The same benchmark file uses retired tenant_class labels at lines `10`, `21`, `33`, `44`, `55`, `66`, `90`, and `92`; do not reuse its structure for the new benchmark report.
52. E-052: `competitor-parity-matrix.md:16-20` cites Workday, DocuSign, SAP, and FINRA; it does not satisfy the assigned union-coverage bar.
53. E-053: The six canonical contexts are named at `specs/master-plan-sequencing.json:704-745`; the µservice lacks all six declared targets.
54. E-054: OpenTofu is canonical at `specs/master-plan-sequencing.json:747-775`; flat `iac/terraform-*.tf` files should be rewritten even if `.tf` extension remains normal for OpenTofu.
55. E-055: `iac/terraform-main.tf:6` and `iac/terraform-variables.tf:6` use `null_resource`; ADR-0328 treats that as forbidden handroll IaC.
56. E-056: OCI Always Free profile is canonical at `specs/master-plan-sequencing.json:857-868`; no `iac/oci-guest/always-free/` directory exists.
57. E-057: The OS manifest requirement is explicit at `specs/master-plan-sequencing.json:777-815`; no service-local `supported-oses.json` was found.
58. E-058: Rust backend policy is explicit at `specs/master-plan-sequencing.json:817-856`; the extension scan found no forbidden source files under this µservice.
59. E-059: `src/lib.rs:1` forbids unsafe code; this is good but not enough implementation evidence.
60. E-060: `src/lib.rs:39-56` tests only declarations; behavior, idempotency, Cedar, SLO, tenant-class, and contract tests remain absent.
61. E-061: No `tests/` directory exists; regression coverage must be added before behavior-changing cleanup in future implementation work.
62. E-062: `contracts/workplace-integration-v1.proto:5-7` includes Java/Go package options; generated-client provenance should be documented before SDK output is generated.
63. E-063: Slack docs state apps can be built by Slack, third parties, or the workspace team and browsed from a directory; this maps to marketplace ownership, not just workplace domain events.
64. E-064: Slack docs state app Home, shortcuts, slash commands, messages, and app DMs are app surfaces; current µservice has none of those surface definitions.
65. E-065: Slack Events API requires HTTP 2xx within three seconds; current PRD p95/p99 numbers do not prove external event acknowledgement conformance.
66. E-066: Slack Events API permits 30,000 events per workspace-app per 60 minutes; capacity docs need this external ingress envelope.
67. E-067: Teams docs enumerate tabs, webhooks/connectors, messaging extensions, meeting extensions, bots, cards, task modules, and activity feeds; current µservice has none of those package artifacts.
68. E-068: Teams docs define admin-managed app availability, certification, compliance, security, permissions, pricing, and setup metadata; current service docs do not model app package governance.
69. E-069: Teams bot docs impose per-thread and per-tenant throttles; current service lacks backoff/retry language for Teams.
70. E-070: Zapier docs require triggers, actions, searches, and search-or-create patterns; current contracts can map to those patterns but do not expose a Zapier catalog.
71. E-071: Zapier polling checks endpoints every one to fifteen minutes; current SLO docs do not model polling freshness.
72. E-072: Zapier REST Hooks require subscribe/unsubscribe and target URL handshake; current OpenAPI has no subscription endpoints.
73. E-073: Zapier webhook response includes `X-Hook-Secret`; current security docs need secret verification and rotation before Zapier parity.
74. E-074: User directive retires capability-adoption deltas for this batch; no fourth deliverable should be created.
75. E-075: User directive requires three tenant classes; current service has no `demo_trial`, `paid`, or `revenue_share` semantics.
76. E-076: User directive keeps quality uniform across tenant classes; remediation must avoid quality-downscoping for demo tenants.
77. E-077: User directive names all six contexts unless audit finds otherwise; this audit found no evidence to remove a context from scope.
78. E-078: `README.md:18-25` lists local artifact families; this helps future remediation route changes without touching other microservices.
79. E-079: `ARCHITECTURE.md:115-124` includes the intended API map; compare it with OpenAPI before editing route tables.
80. E-080: `ARCHITECTURE.md:1475-1508` contains rotated route-to-journey mappings; generated or copied tables should be treated as suspect until validated.
81. E-081: `slos/esign-initiate-availability.openslo.yaml:19-26` reads sign metrics; all SLOs should be checked as a set, not one-off edited.
82. E-082: `slos/signature-capture-latency.openslo.yaml:19-26` reads offer metrics; latency targets in the new benchmark report should not rely on current SLO wiring.
83. E-083: `slos/offer-generation-latency.openslo.yaml:19-26` reads roster metrics; service observability cannot be certified until corrected.
84. E-084: `slos/roster-binding-accuracy.openslo.yaml:19-26` reads clock metrics; accuracy claims are currently unauditable.
85. E-085: `slos/clock-attestation-availability.openslo.yaml:19-26` reads DLP metrics; clock availability claims are currently unauditable.
86. E-086: `slos/dlp-trace-seal-fidelity.openslo.yaml:19-26` reads e-sign-initiate metrics; DLP trace fidelity claims are currently unauditable.
87. E-087: `contracts/openapi-v1.yaml:18-20` uses an example server URL; external marketplace publication needs canonical public, guest, and private endpoint patterns.
88. E-088: `PRD.md:90-100` names compliance and out-of-scope boundaries; marketplace remediation should not silently expand into payroll calculation or full HRIS ownership.
89. E-089: `faqs/hris-engineer-faq.md:14-22` claims payroll-related behavior; this should be converted to evidence handoff language or moved to payroll ownership.
90. E-090: The audit halt condition was not triggered because the three requested deliverables can be completed service-locally with available evidence.

<!-- ORCHESTRATOR REPORT
  µservice: workplace-integration
  deliverables_landed:
    - microservices/workplace-integration/coherence-audit-2026-05-20.md (651 lines)
    - microservices/workplace-integration/feature-parity-matrix-2026-05-20.md (421 lines)
    - microservices/workplace-integration/performance-benchmark-numbers-2026-05-20.md (311 lines)
  inventory_files_seen: 135
  inventory_lines_read: 22660
  chat_history_matches_processed: 145
  findings_p0: 0
  findings_p1: 8
  findings_p2: 8
  findings_p3: 2
  tier_retirement_candidates_found: 29
  tier_retirement_candidate_cites_1: onboarding/hris-engineer-first-week.md:12; onboarding/hris-engineer-first-week.md:24; migration-playbooks/from-rippling-and-gusto.md:41; benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:10; benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:21; benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:33
  tier_retirement_candidate_cites_2: benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:44; benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:55; benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:66; benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:90; benchmarks/workplace-integration-vs-rippling-vs-gusto-vs-workday-vs-justworks-vs-deel.md:92; tutorials/hire-onboard-clock-in-payroll-cycle.md:3
  tier_retirement_candidate_cites_3: tutorials/hire-onboard-clock-in-payroll-cycle.md:8; ADR-0330 and ADR-0331 tenant_class model:10; ADR-0330 and ADR-0331 tenant_class model:26; ADR-0330 and ADR-0331 tenant_class model:42; ADR-0330 and ADR-0331 tenant_class model:58; ADR-0330 and ADR-0331 tenant_class model:63; ADR-0330 and ADR-0331 tenant_class model:87
  tier_retirement_candidate_cites_4: faqs/hris-engineer-faq.md:17; faqs/hris-engineer-faq.md:18; faqs/hris-engineer-faq.md:19; faqs/hris-engineer-faq.md:20; faqs/hris-engineer-faq.md:35; faqs/hris-engineer-faq.md:46; faqs/hris-engineer-faq.md:56
  tier_retirement_candidate_cites_5: faqs/hris-engineer-faq.md:94; faqs/hris-engineer-faq.md:114; faqs/hris-engineer-faq.md:162
  tenant_class_adoption_gaps: yes; no tenant_class/demo_trial/paid/revenue_share semantics found under microservices/workplace-integration
  top_3_counterparts_confirmed: Slack App Directory / Microsoft Teams App Store / Zapier Integrations
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1383
-->
