# developer-sdk Ownership-Coherence Audit

- Date: 2026-05-20.
- Wave: 3.
- Batch: 3.2.
- Microservice: `developer-sdk`.
- Scope root: `microservices/developer-sdk/`.
- Audit owner: single-agent ownership lane.
- Deliverable: 1 of 3.
- Retired deliverable: capability-ladder deltas, removed by the 2026-05-20 no-ladder directive.
- Top-3 counterpart set: Stainless, Speakeasy, Fern.
- Required deployment contexts: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, `oyatie-as-cloud-provider`.
- Required IaC substrate: OpenTofu through cloud-iac handoff, not hand-rolled provisioning.
- Required language posture: Rust-strict backend and tooling, with approved frontend exceptions only.
- Required tenant-class posture for this audit: `demo_trial`, `paid`, `revenue_share`.
- Uniform quality posture: industry-leader-grade quality across tenant classes, not tier-stratified capability.
- Source discipline: each finding cites repo file lines, directive memory lines, chat history lines, or canonical docs.

## Citation Anchor Block

- Canonical sequence and batch discipline: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1732-2222`, `:2243-2625`, `:2648-3033`, `:3047-3319`, `:3831-4228`.
- Master-plan constraints: `specs/master-plan-sequencing.json:704-867`.
- Brief substance and anti-pattern bar: `docs/standards/brief-template.md:666-1185`, `:1511-1805`.
- Developer-sdk product correction: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:10-121`.
- Ownership-coherence and verification directives: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:10-92`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-53`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-20`.

## §1 Purpose

1. This audit tests whether `developer-sdk` has one coherent product identity under the current canonical direction.
2. The target identity is an SDK-generation service comparable to Stainless, Speakeasy, and Fern.
3. The target identity is not a generic developer portal, app marketplace, KYC onboarding product, payout ledger, or plugin-store substrate.
4. The developer-sdk-specific directive states that this microservice is an SDK generator like Stainless, not a portal, doc site, or CLI generator; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:10-13`.
5. The same directive defines input contracts as OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:16-18`.
6. The same directive defines the expected output SDK languages as TypeScript, Python, Go, Java, Kotlin, Swift, Rust, C#/.NET, C, and C++; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:18-30`.
7. The same directive rejects Ruby, PHP, and Elixir output as deliberate non-goals; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:18-30`.
8. The same directive requires idiomatic naming, type mapping, pagination, retries, streaming, idempotency, telemetry, error taxonomy, mocks, multipart, and serialization behavior; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:34-47`.
9. The same directive requires publication through package registries and release channels, not only local code generation; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:48-60`.
10. The same directive names Stainless, Speakeasy, and Fern as the top-3 counterpart set; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:78-84`.
11. Chat history repeats that Stainless, Speakeasy, and Fern are the developer-sdk counterpart set, and that Stripe SDK, Twilio helper libraries, and Auth0 SDK are SDK outputs rather than SDK generators; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:15754`.
12. Chat history also records that the capability-ladder retirement deltas deliverable was dropped from Wave 3 Batch 3.2 onward; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16020-16032`.
13. The audit therefore measures the existing artifacts against SDK-generator ownership coherence, not against marketplace, payout, or KYC feature breadth.
14. The audit also measures whether the service is ready for all six deployment contexts named by the master plan.
15. The audit checks OpenTofu readiness because ADR-0328 D-16 requires OpenTofu, per-context IaC, signed modules, and no Terraform Cloud/local state/manual console drift; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2625`.
16. The audit checks OS readiness because ADR-0328 D-17 requires a per-microservice `supported-oses.json` manifest for services with binaries, host agents, CLIs, controllers, or native bundles; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2648-3033`.
17. The audit checks language readiness because ADR-0328 D-18 makes backend runtime, CLI, validation, codegen, scripting, and CI Rust-only unless an exception is explicitly approved; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3047-3319`.
18. The audit checks OCI Always Free readiness because the master plan requires `iac/oci-guest/always-free/` and identifies the old `demo_trial_oci_profile` wording as stale; see `specs/master-plan-sequencing.json:857-867`.
19. The audit checks tenant-class retirement migration readiness because the no-capability-ladders directive records that the user said "we don't have tiers" and that tier artifacts are being retired; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_ladder_2026_05_20.md:10-44`.
20. The audit checks tenant-class adoption because this batch replaces feature tiers with tenant-class and billing semantics.
21. The audit treats the current prompt's three-class model, `demo_trial`, `paid`, and `revenue_share`, as the active target for this deliverable.
22. The audit notes that a later memory file records a two-class variant with revenue share as a billing component; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:10-12`.
23. That memory drift is not used to override the current prompt, but it is material evidence that tenant-class wording needs one authoritative contract before Wave 15J completion.
24. The stop condition for this audit is three landed reports, each meeting the stated line floor, with no capability-ladder retirement deltas deliverable added.
25. The stop condition also requires an inventory count, chat-history match count, severity counts, tenant-class retirement migration count, and verification evidence in the orchestrator report.

## §2 Inventory

### §2.1 Inventory Summary

1. Files seen under `microservices/developer-sdk/`: 137.
2. Existing lines inventoried across those files: 15,681.
3. Top-level product docs seen: PRD, architecture, phase plan, backfill replay, capacity, compliance, cost, DPIA, failure modes, incident response, manifestation, migration, multi-region, performance, SDK plan, and threat model.
4. Contracts seen: OpenAPI 3.2, AsyncAPI 3.1, and proto3 files.
5. ADRs seen: seven `ADR-SDK-*` files.
6. Implementation plans seen: fifteen `IP-*` files.
7. SLOs seen: nine OpenSLO files.
8. IaC files seen: Helm-oriented files under `iac/helm/`.
9. Canonical context IaC directories seen: none for `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, or `iac/oyatie-iaas/`.
10. OCI Always Free IaC directory seen: none for `iac/oci-guest/always-free/`.
11. OS support manifest seen: no `microservices/developer-sdk/supported-oses.json`.
12. Source directory seen: no `microservices/developer-sdk/src/`.
13. Test directory seen: no `microservices/developer-sdk/tests/`.
14. Forbidden source extension scan result: no first-party files found with prohibited backend source extensions inside the microservice path.
15. Tier-specific directory seen: `capability-ladders/`, which is a Wave 15J retirement candidate by directive.

### §2.2 Complete File Inventory

1. Pre-write inventory contains 137 existing files; the three audit reports authored by this batch are excluded from this list and are reported in the orchestrator block.
2. `microservices/developer-sdk/ARCHITECTURE.md`.
3. `microservices/developer-sdk/IP-journey-j100-pack-rollout-first-action.md`.
4. `microservices/developer-sdk/IP-journey-j41-sandbox-deploy.md`.
5. `microservices/developer-sdk/IP-journey-j91-us-msb-mtl-overlay.md`.
6. `microservices/developer-sdk/IP-journey-j92-br-lgpd-us-parent-dsar.md`.
7. `microservices/developer-sdk/IP-journey-j93-in-dpdpa-rbi-overlay.md`.
8. `microservices/developer-sdk/IP-journey-j94-sox404-public-company-controls.md`.
9. `microservices/developer-sdk/IP-journey-j95-iso27001-soc2-annual-audit.md`.
10. `microservices/developer-sdk/IP-journey-j96-ksa-uae-mena-onboarding.md`.
11. `microservices/developer-sdk/IP-journey-j97-sg-pdpa-mas-tenant.md`.
12. `microservices/developer-sdk/IP-journey-j98-au-privacy-apra-cps234.md`.
13. `microservices/developer-sdk/IP-journey-j99-multi-pack-conflict-resolution.md`.
14. `microservices/developer-sdk/PHASE-01-DEVELOPER-SDK-SUBSTRATE.md`.
15. `microservices/developer-sdk/PRD.md`.
16. `microservices/developer-sdk/backfill-replay.md`.
17. `microservices/developer-sdk/benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md`.
18. `microservices/developer-sdk/capabilities/developer-onboard.yaml`.
19. `microservices/developer-sdk/capabilities/developer-payout-settle.yaml`.
20. `microservices/developer-sdk/capabilities/developer-sandbox-reset.yaml`.
21. `microservices/developer-sdk/capability-ladders/tier-matrix.md`.
22. `microservices/developer-sdk/capacity-model.md`.
23. `microservices/developer-sdk/catalog/oya-developer-sdk-api-contracts-registry-kernel.yaml`.
24. `microservices/developer-sdk/catalog/oya-developer-sdk-dev-portal-app.yaml`.
25. `microservices/developer-sdk/catalog/oya-developer-sdk-dev-portal-rest.yaml`.
26. `microservices/developer-sdk/catalog/oya-developer-sdk-developer-onboarding-adapter-postgres.yaml`.
27. `microservices/developer-sdk/catalog/oya-developer-sdk-developer-onboarding-domain.yaml`.
28. `microservices/developer-sdk/catalog/oya-developer-sdk-developer-onboarding-kernel.yaml`.
29. `microservices/developer-sdk/catalog/oya-developer-sdk-developer-onboarding-rest.yaml`.
30. `microservices/developer-sdk/catalog/oya-developer-sdk-developer-onboarding-usecase.yaml`.
31. `microservices/developer-sdk/catalog/oya-developer-sdk-payout-adapter-ach.yaml`.
32. `microservices/developer-sdk/catalog/oya-developer-sdk-payout-adapter-kftc.yaml`.
33. `microservices/developer-sdk/catalog/oya-developer-sdk-payout-adapter-sepa.yaml`.
34. `microservices/developer-sdk/catalog/oya-developer-sdk-payout-worker.yaml`.
35. `microservices/developer-sdk/catalog/oya-developer-sdk-sandbox-provisioner-adapter-tenancy.yaml`.
36. `microservices/developer-sdk/catalog/oya-developer-sdk-sandbox-provisioner-worker.yaml`.
37. `microservices/developer-sdk/catalog/oya-developer-sdk-sdk-codegen-worker.yaml`.
38. `microservices/developer-sdk/catalog/oya-developer-sdk-signing-key-issuance-adapter-openbao.yaml`.
39. `microservices/developer-sdk/catalog/oya-developer-sdk-signing-key-issuance-kernel.yaml`.
40. `microservices/developer-sdk/catalog/oya-developer-sdk-tax-form-usecase.yaml`.
41. `microservices/developer-sdk/competitor-parity-matrix.md`.
42. `microservices/developer-sdk/compliance.md`.
43. `microservices/developer-sdk/contracts/asyncapi/developer-sdk-events.yaml`.
44. `microservices/developer-sdk/contracts/openapi/developer-sdk.yaml`.
45. `microservices/developer-sdk/contracts/openapi/oya-ecosystem.yaml`.
46. `microservices/developer-sdk/contracts/proto/developer-sdk.proto`.
47. `microservices/developer-sdk/cost-budget.md`.
48. `microservices/developer-sdk/cross-microservice-handoffs.md`.
49. `microservices/developer-sdk/dashboards/onboarding-funnel.json`.
50. `microservices/developer-sdk/dashboards/payout-health.json`.
51. `microservices/developer-sdk/dashboards/sdk-codegen-health.json`.
52. `microservices/developer-sdk/decisions/ADR-SDK-0001-ed25519-signing-keys-via-openbao-transit-engine-only;-privat.md`.
53. `microservices/developer-sdk/decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md`.
54. `microservices/developer-sdk/decisions/ADR-SDK-0003-per-developer-sandbox-tenant-via-tenancy-µservice's-sandbox-.md`.
55. `microservices/developer-sdk/decisions/ADR-SDK-0004-payout-substrate-uses-iso-20022-pain.001-for-sepa-and-nacha-.md`.
56. `microservices/developer-sdk/decisions/ADR-SDK-0005-tax-form-emission-triggered-at-year-end-regenerated-on-deman.md`.
57. `microservices/developer-sdk/decisions/ADR-SDK-0006-kyc-pipeline-in-house;-no-external-kyc-saas-(onfido-persona-.md`.
58. `microservices/developer-sdk/decisions/ADR-SDK-0007-dev-portal-as-backstage-extension-not-standalone-app.md`.
59. `microservices/developer-sdk/deprecation-plan.md`.
60. `microservices/developer-sdk/dpia.md`.
61. `microservices/developer-sdk/evidence-emission.md`.
62. `microservices/developer-sdk/failure-modes.md`.
63. `microservices/developer-sdk/faqs/sdk-engineer-faq.md`.
64. `microservices/developer-sdk/iac/helm/backstage/Chart.yaml`.
65. `microservices/developer-sdk/iac/helm/backstage/values.yaml`.
66. `microservices/developer-sdk/iac/helm/developer-sdk-rest/Chart.yaml`.
67. `microservices/developer-sdk/iac/helm/developer-sdk-rest/values.yaml`.
68. `microservices/developer-sdk/iac/helm/developer-sdk-worker/Chart.yaml`.
69. `microservices/developer-sdk/iac/helm/developer-sdk-worker/values.yaml`.
70. `microservices/developer-sdk/iac/helm/openbao/Chart.yaml`.
71. `microservices/developer-sdk/iac/helm/openbao/values.yaml`.
72. `microservices/developer-sdk/iac/helm/package-registry-cargo/Chart.yaml`.
73. `microservices/developer-sdk/iac/helm/package-registry-cargo/values.yaml`.
74. `microservices/developer-sdk/iac/helm/package-registry-npm/Chart.yaml`.
75. `microservices/developer-sdk/iac/helm/package-registry-npm/values.yaml`.
76. `microservices/developer-sdk/iac/helm/package-registry-nuget/Chart.yaml`.
77. `microservices/developer-sdk/iac/helm/package-registry-nuget/values.yaml`.
78. `microservices/developer-sdk/iac/helm/package-registry-pypi/Chart.yaml`.
79. `microservices/developer-sdk/iac/helm/package-registry-pypi/values.yaml`.
80. `microservices/developer-sdk/iac/helm/postgres/Chart.yaml`.
81. `microservices/developer-sdk/iac/helm/postgres/values.yaml`.
82. `microservices/developer-sdk/implementation-plans/IP-001-layer-a-postgres-openbao-backstage-iac.md`.
83. `microservices/developer-sdk/implementation-plans/IP-002-developer-onboarding-kernel-domain.md`.
84. `microservices/developer-sdk/implementation-plans/IP-003-developer-onboarding-usecase-api-adapter-rest-app.md`.
85. `microservices/developer-sdk/implementation-plans/IP-004-signing-key-issuance-openbao.md`.
86. `microservices/developer-sdk/implementation-plans/IP-005-api-contracts-registry.md`.
87. `microservices/developer-sdk/implementation-plans/IP-006-sdk-codegen-ts-rust-swift-kotlin-csharp-python.md`.
88. `microservices/developer-sdk/implementation-plans/IP-007-sandbox-provisioner-tenant-on-demand.md`.
89. `microservices/developer-sdk/implementation-plans/IP-008-dev-portal-backstage-extension.md`.
90. `microservices/developer-sdk/implementation-plans/IP-009-dev-portal-app-submission-flow.md`.
91. `microservices/developer-sdk/implementation-plans/IP-010-payout-ach-sepa-kftc-fedwire.md`.
92. `microservices/developer-sdk/implementation-plans/IP-011-tax-form-1099-vat-moss-kr-vat.md`.
93. `microservices/developer-sdk/implementation-plans/IP-012-package-registry-vendored.md`.
94. `microservices/developer-sdk/implementation-plans/IP-013-observability-slo-manifests.md`.
95. `microservices/developer-sdk/implementation-plans/IP-014-branch-protection-and-hyperscaler-gates.md`.
96. `microservices/developer-sdk/implementation-plans/IP-015-stripe-connect-parity-end-to-end-drill.md`.
97. `microservices/developer-sdk/incident-response.md`.
98. `microservices/developer-sdk/manifest.json`.
99. `microservices/developer-sdk/migration-playbooks/from-stripe-and-twilio-sdks.md`.
100. `microservices/developer-sdk/multi-region.md`.
101. `microservices/developer-sdk/onboarding/sdk-engineer-first-week.md`.
102. `microservices/developer-sdk/packs/eu/manifest.json`.
103. `microservices/developer-sdk/packs/eu/policy-overlay.cedar`.
104. `microservices/developer-sdk/packs/kr/manifest.json`.
105. `microservices/developer-sdk/packs/kr/policy-overlay.cedar`.
106. `microservices/developer-sdk/packs/us-financial/manifest.json`.
107. `microservices/developer-sdk/packs/us-financial/policy-overlay.cedar`.
108. `microservices/developer-sdk/packs/us-healthcare/manifest.json`.
109. `microservices/developer-sdk/packs/us-healthcare/policy-overlay.cedar`.
110. `microservices/developer-sdk/packs/us-public-sector/manifest.json`.
111. `microservices/developer-sdk/packs/us-public-sector/policy-overlay.cedar`.
112. `microservices/developer-sdk/performance-bench.md`.
113. `microservices/developer-sdk/policy/admin-scope.cedar`.
114. `microservices/developer-sdk/policy/developer-scope.cedar`.
115. `microservices/developer-sdk/policy/payout-scope.cedar`.
116. `microservices/developer-sdk/policy/public-read.cedar`.
117. `microservices/developer-sdk/reference-implementations/multi-language-canary-rust-sdk.md`.
118. `microservices/developer-sdk/runbooks/codegen-pipeline-non-deterministic.md`.
119. `microservices/developer-sdk/runbooks/dev-portal-down.md`.
120. `microservices/developer-sdk/runbooks/developer-revocation-cascade-stuck.md`.
121. `microservices/developer-sdk/runbooks/kyc-pipeline-stuck.md`.
122. `microservices/developer-sdk/runbooks/payout-settlement-mismatch.md`.
123. `microservices/developer-sdk/runbooks/sandbox-provision-slow.md`.
124. `microservices/developer-sdk/runbooks/signing-key-issuance-timeout.md`.
125. `microservices/developer-sdk/runbooks/tax-form-emission-mismatch.md`.
126. `microservices/developer-sdk/scorecards/overrides.json`.
127. `microservices/developer-sdk/sdk-plan.md`.
128. `microservices/developer-sdk/slos/codegen-availability.openslo.yaml`.
129. `microservices/developer-sdk/slos/onboarding-completion-availability.openslo.yaml`.
130. `microservices/developer-sdk/slos/payout-settlement-availability.openslo.yaml`.
131. `microservices/developer-sdk/slos/payout-settlement-correctness.openslo.yaml`.
132. `microservices/developer-sdk/slos/portal-availability.openslo.yaml`.
133. `microservices/developer-sdk/slos/sandbox-provision-latency.openslo.yaml`.
134. `microservices/developer-sdk/slos/sandbox-reset-latency.openslo.yaml`.
135. `microservices/developer-sdk/slos/signing-key-issuance-latency.openslo.yaml`.
136. `microservices/developer-sdk/slos/tax-form-emission-correctness.openslo.yaml`.
137. `microservices/developer-sdk/threat-model.md`.
138. `microservices/developer-sdk/tutorials/generate-publish-and-verify-rust-typescript-python-sdks.md`.
139. No hidden source or test file was found inside the target path during the recursive file scan.
140. No file outside `microservices/developer-sdk/` was modified for this audit.

### §2.3 Read Coverage Notes

1. `PRD.md` was read in full and is 194 lines.
2. `ARCHITECTURE.md` was read to substantial depth and is 754 lines.
3. `manifest.json` was read through the product, capability, dependency, benchmark, and classification blocks.
4. All contract files were read through their service/path/schema sections.
5. All ADR file names were inventoried and the key ADRs governing codegen, sandboxing, portal architecture, payout, and risk scoring were inspected.
6. All implementation plan file names were inventoried and representative plans covering substrate, codegen, portal, registry, observability, and security were inspected.
7. All SLO file names were inventoried, and the codegen, portal, signing-key, sandbox, catalog, webhook, and payout SLOs were considered as product-surface evidence.
8. The `capability-ladders/tier-matrix.md` file was read because it is a direct Wave 15J retirement candidate.
9. The benchmark and parity files were read because the counterpart set was a known risk.
10. The tutorial was read because it contains command, language, registry, and tier claims.
11. The FAQ was read because it contains several direct tier references and long-tail language claims.
12. Chat history was searched for `developer-sdk`, `Stainless`, `Speakeasy`, `Fern`, `tenant_class`, and tenant-class retirement migration language.
13. Chat-history matches processed: 139.
14. The audit did not use scripting to author substantive report content.
15. Shell commands were used only for inventory, search, line-count, and verification evidence.

## §3 9-Dimension Audit

### §3.1 Dimension 1 — Product Purpose and Ownership Boundary

1. Finding: P1 ownership drift exists between the canonical SDK-generator target and the current artifact corpus.
2. Canonical evidence: the developer-sdk memory says the service is an SDK generator like Stainless, not a portal, doc site, or CLI generator; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:10-13`.
3. Canonical evidence: the same file says the service does not own runtime frontends, doc site authoring, tutorials, or customer credentials; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:70-76`.
4. Repo evidence: `PRD.md:21-25` defines the product as onboarding, signing, sandboxing, codegen, marketplace, and payout/tax foundation.
5. Repo evidence: `PRD.md:31-38` frames outcomes around onboarding, plugin submission, sandboxing, revenue share, payout, and Stripe-Connect-parity substrate, which is broader than generator ownership.
6. Repo evidence: `PHASE-01-DEVELOPER-SDK-SUBSTRATE.md:37` frames the phase as Stripe-Connect-parity onboarding plus OpenBao, contracts, six SDKs, Backstage portal, payout, and tax.
7. Repo evidence: `manifest.json:10-18` says doctrine inherits Stripe Connect, Apple Developer, Backstage, and OpenAPI Generator.
8. Repo evidence: `ARCHITECTURE.md:445-457` centers deployment shape on REST, workers, Backstage, package registries, OpenBao, and Postgres.
9. Repo evidence: `contracts/openapi/developer-sdk.yaml:7-9` describes developer onboarding, signing keys, plugin catalog, payout, and tax surfaces.
10. Repo evidence: `contracts/proto/developer-sdk.proto:126-133` exposes SignupDeveloper, signing key, sandbox, and payout methods rather than SDK-generation methods.
11. Repo evidence: `competitor-parity-matrix.md:15-27` compares Apple App Store, VS Code Marketplace, AWS Marketplace, Shopify, Stripe Connect, and Salesforce AppExchange rather than SDK-generator products.
12. Repo evidence: `performance-bench.md:36-40` uses Apple App Store, Stripe Connect, and VS Code Marketplace baselines.
13. The service contains a real codegen strand, but it is not the dominant ownership story.
14. `PRD.md:33` calls out six official SDK families.
15. `implementation-plans/IP-006-sdk-codegen-ts-rust-swift-kotlin-csharp-python.md:16-20` scopes a six-family codegen plan.
16. `decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:47-80` and `decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:207-235` are generator-specific evidence for deterministic codegen and verification gates.
17. The issue is not absence of SDK-generation intent.
18. The issue is that SDK generation is embedded inside a larger developer-marketplace-and-payout product.
19. The current corpus therefore cannot yet support a clean single-owner microservice boundary.
20. Recommended ownership correction: make `developer-sdk` own contract ingestion, generator configuration, deterministic code generation, generated SDK verification, generated docs/snippets as artifacts, package publishing workflows, and SDK lifecycle evidence.
21. Recommended boundary correction: move or explicitly hand off developer onboarding, KYC, payout ledger, tax package, marketplace vetting, plugin entitlement, and generic portal runtime to the owning services.
22. Severity: P1 because the product identity affects contracts, implementation plans, benchmarks, parity comparisons, and deployment scope.

### §3.2 Dimension 2 — Contract and API Coherence

1. Finding: P1 contract surface does not expose the canonical generator control plane.
2. Canonical evidence: the developer-sdk memory says the audit API should include `POST /sdks/generate`, `/publish`, versions, configurations, docs, and fixtures; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:92-100`.
3. Repo evidence: `contracts/openapi/developer-sdk.yaml:1-3` correctly declares OpenAPI 3.2.0.
4. Repo evidence: `contracts/asyncapi/developer-sdk-events.yaml:1-3` correctly declares AsyncAPI 3.1.0.
5. Repo evidence: `contracts/proto/developer-sdk.proto:1-4` declares proto3 syntax and a developer SDK package.
6. Repo evidence: `contracts/openapi/developer-sdk.yaml:147-171` centers SDK artifact read/download behavior and not a generator run creation endpoint.
7. Repo evidence: `contracts/openapi/developer-sdk.yaml:161` defines language enum values for TypeScript, Rust, Swift, Kotlin, C#, and Python only.
8. Repo evidence: `contracts/openapi/developer-sdk.yaml:302` repeats the six-language enum.
9. Repo evidence: `contracts/proto/developer-sdk.proto:126-133` has no GenerateSdk, PublishSdk, ConfigureSdk, CreateFixture, or GetSdkDocs operation.
10. Repo evidence: `contracts/asyncapi/developer-sdk-events.yaml:12-40` emits developer, plugin, sandbox, SDK published, payout, and tax events.
11. Repo evidence: `contracts/asyncapi/developer-sdk-events.yaml:121` repeats the six-family language enum.
12. Repo evidence: `ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:155-165` describes `/v1/sdk/codegen/runs...`, which is closer to generator ownership but not aligned to the current directive's `/sdks/generate` shape.
13. Repo evidence: `ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:47-80` captures deterministic two-run codegen decisions that should be retained.
14. Repo evidence: `cross-microservice-handoffs.md:22-37` lists inbound API handoffs but lacks the canonical generate/publish/configuration/fixture lifecycle.
15. Contract drift exists in both direction and vocabulary.
16. Direction drift: contracts prioritize onboarding, keys, sandbox, plugin, payout, and tax.
17. Vocabulary drift: codegen appears as `/v1/sdk/codegen/runs`, not as the canonical `/sdks/generate` and related surface.
18. Language drift: the current contracts emit six output families, while the canonical direction requires ten output SDK families.
19. Publication drift: `contracts/openapi/developer-sdk.yaml` does not model registry-specific release policies, provenance bundles, or semver channels at the expected depth.
20. Fixture drift: current contracts do not model canonical contract fixtures, generated-client compile fixtures, or cross-language behavioral fixtures.
21. Documentation drift: current contracts do not model generated docs and snippets as first-class outputs.
22. Severity: P1 because contracts are the main µservice boundary and currently point implementers at the wrong product.

### §3.3 Dimension 3 — Implementation Plan, Data, and Workflow Coherence

1. Finding: P1 implementation plans overfit non-generator workflows.
2. Repo evidence: `implementation-plans/IP-001-layer-a-postgres-openbao-backstage-iac.md:27-29` sets up Postgres, OpenBao, Backstage, REST, worker, package-registry, and policy paths.
3. Repo evidence: `implementation-plans/IP-001-layer-a-postgres-openbao-backstage-iac.md:39-41` creates Helm chart files.
4. Repo evidence: `implementation-plans/IP-001-layer-a-postgres-openbao-backstage-iac.md:49` says Helm charts deploy Postgres, OpenBao, Backstage, REST API, worker, and package registries.
5. Repo evidence: `implementation-plans/IP-001-layer-a-postgres-openbao-backstage-iac.md:92-94` verifies through Helm install commands.
6. Repo evidence: `implementation-plans/IP-002-developer-onboarding-kernel-domain.md` is a KYC plan, which is outside the SDK-generator target unless it is a handoff consumer.
7. Repo evidence: `implementation-plans/IP-004-signing-key-issuance-openbao.md` is a credential plan, which conflicts with the canonical note that developer-sdk does not own customer credentials.
8. Repo evidence: `implementation-plans/IP-005-api-contracts-registry.md` is a plugin registry plan, not an SDK-generator plan.
9. Repo evidence: `implementation-plans/IP-009-dev-portal-app-submission-flow.md` is a trust review plan, not an SDK-generator plan.
10. Repo evidence: `implementation-plans/IP-006-sdk-codegen-ts-rust-swift-kotlin-csharp-python.md:16-20` is the best-aligned plan but still caps output at six families.
11. Repo evidence: `implementation-plans/IP-008-dev-portal-backstage-extension.md` is a portal plan, while the canonical directive says developer-sdk is not a portal/doc site owner.
12. Repo evidence: `implementation-plans/IP-007-sandbox-provisioner-tenant-on-demand.md` is sandbox infrastructure, not generator output.
13. Repo evidence: `implementation-plans/IP-010-payout-ach-sepa-kftc-fedwire.md` is payout and tax ownership, outside generator scope.
14. Repo evidence: `implementation-plans/IP-009-dev-portal-app-submission-flow.md` is plugin vetting/review ownership, outside generator scope.
15. Repo evidence: `implementation-plans/IP-012-package-registry-vendored.md:20` does align to publication channels but is registry-infrastructure oriented rather than generated-package release governance.
16. Repo evidence: `implementation-plans/IP-012-package-registry-vendored.md:40-43` references `iac/registry/*/Chart.yaml`, but the actual inventory has package-registry charts under `iac/helm/package-registry-*`.
17. Repo evidence: `implementation-plans/IP-013-observability-slo-manifests.md` can be retained if metrics are generator-specific.
18. Repo evidence: `implementation-plans/IP-014-branch-protection-and-hyperscaler-gates.md` can be retained only after credential ownership is re-scoped.
19. Repo evidence: `implementation-plans/IP-015-stripe-connect-parity-end-to-end-drill.md` should be reworked to generator workloads.
20. The implementation backlog contains useful pieces, but its ownership map is too broad.
21. The codegen implementation plan should become the core spine rather than one plan among many unrelated product lanes.
22. The data model should center API specifications, generation configurations, SDK targets, generator runs, reproducibility evidence, package publication, and fixture results.
23. Current data plans center developer identity, signing keys, plugin manifests, sandboxes, payout ledger, and tax packages.
24. Recommended correction: split non-generator plans into handoff dependencies or other microservice backlogs, then author a coherent generator-first IP set.
25. Severity: P1 because implementation plans are likely to direct future agents toward building the wrong service.

### §3.4 Dimension 4 — Canonical-Direction Alignment

1. Finding: P1 canonical-direction gaps exist across deployment contexts, IaC, OS manifest, language posture, OCI profile, tier retirement, and tenant classes.
2. Canonical evidence: ADR-0328 D-15 requires six deployment contexts and context support cannot be claimed in prose without matching `iac/<context>/` or explicit N/A manifest; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1732-2222`.
3. Canonical evidence: ADR-0328 D-16 requires OpenTofu and forbids Terraform Cloud/local disk state/manual console handoff patterns; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2625`.
4. Canonical evidence: ADR-0328 D-17 requires a per-microservice OS manifest; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2907-2947`.
5. Canonical evidence: ADR-0328 D-18 requires Rust-strict backend, CLI, validation, codegen, scripting, and CI; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3047-3319`.
6. Canonical evidence: the master plan lists six deployment contexts and target IaC paths; see `specs/master-plan-sequencing.json:704-745`.
7. Canonical evidence: the master plan lists OpenTofu substrate requirements and forbidden substrate patterns; see `specs/master-plan-sequencing.json:747-775`.
8. Canonical evidence: the master plan lists OS and language policy requirements; see `specs/master-plan-sequencing.json:777-855`.
9. Canonical evidence: the master plan requires `iac/oci-guest/always-free/` and marks `demo_trial_oci_profile` as stale; see `specs/master-plan-sequencing.json:857-867`.
10. Repo evidence: the only IaC subtree found is `microservices/developer-sdk/iac/helm/`.
11. Repo evidence: `ARCHITECTURE.md:392` identifies transport evidence as Helm paths.
12. Repo evidence: `ARCHITECTURE.md:445-457` describes Kubernetes, Kata, Helm manifests, and no OpenTofu context directories.
13. Repo evidence: `implementation-plans/IP-001-layer-a-postgres-openbao-backstage-iac.md:92-94` verifies by Helm install.
14. Repo evidence: no `microservices/developer-sdk/supported-oses.json` exists.
15. Repo evidence: `manifest.json:194-204` pins Node and Python dependencies as part of the service dependency story.
16. Repo evidence: `decisions/ADR-SDK-0007-dev-portal-as-backstage-extension-not-standalone-app.md:47-50` frames the portal around Backstage-style React plugin composition.
17. Repo evidence: `decisions/ADR-SDK-0007-dev-portal-as-backstage-extension-not-standalone-app.md:132` includes a Node-related operational path.
18. Repo evidence: no first-party prohibited source files were found under the microservice path, which is a positive implementation signal.
19. Repo evidence: the no-source-file result does not resolve documentation-level language drift because multiple docs still prescribe disallowed runtime surfaces without explicit exception boundaries.
20. Severity: P1 for deployment/IaC/language product blockers and P2 for missing manifests when the underlying implementation may still be portable.

#### §3.4.T Tier Retirement Candidates

1. Classification rule: every direct retired four-label ladder reference below is a Wave 15J retirement candidate unless another canonical document explicitly preserves it; no preserving document was found in this microservice path.
2. Default severity: P2 documentation gap because the retired language is a documentation/control-surface mismatch rather than direct runtime breakage.
3. `tutorials/generate-publish-and-verify-rust-typescript-python-sdks.md:10` uses `tenant_class=demo_trial`; retire and replace with `tenant_class=demo_trial` only if the tutorial remains in scope.
4. `benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md:11` uses tenant_class paid and compliance_pack-bound paid benchmark rows; retire and replace with single industry-leader target plus deployment-context overlays.
5. `benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md:12` uses compliance_pack-bound paid benchmark rows; retire.
6. `benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md:82` says compliance_pack-bound paid environments; retire.
7. `capability-ladders/tier-matrix.md:11` uses retired four-label ladder headings; retire the file or replace it with tenant-class/billing semantics after the shared Wave 15J scrub.
8. `capability-ladders/tier-matrix.md:24` uses `Tier::tenant_class demo_trial`; retire.
9. `capability-ladders/tier-matrix.md:28` uses tenant_class paid; retire.
10. `capability-ladders/tier-matrix.md:33` uses tenant_class demo_trial; retire.
11. `capability-ladders/tier-matrix.md:43` uses tenant_class paid; retire.
12. `capability-ladders/tier-matrix.md:45` uses tenant_class paid; retire.
13. `capability-ladders/tier-matrix.md:50` uses tenant_class paid; retire.
14. `capability-ladders/tier-matrix.md:60` uses tenant_class paid; retire.
15. `capability-ladders/tier-matrix.md:62` uses compliance_pack-bound paid; retire.
16. `capability-ladders/tier-matrix.md:67` uses tenant_class paid; retire.
17. `capability-ladders/tier-matrix.md:77` uses compliance_pack-bound paid; retire.
18. `capability-ladders/tier-matrix.md:93` uses retired four-label ladder; retire.
19. `capability-ladders/tier-matrix.md:95` uses compliance_pack-bound paid; retire.
20. `faqs/sdk-engineer-faq.md:28` uses tenant_class paid/tenant_class paid/tenant_class demo_trial in promotion guidance; retire.
21. `faqs/sdk-engineer-faq.md:29` uses tenant_class paid/tenant_class paid/tenant_class demo_trial in promotion guidance; retire.
22. `faqs/sdk-engineer-faq.md:66` uses tenant_class paid for WASM acceleration; retire.
23. `faqs/sdk-engineer-faq.md:83` uses tenant_class demo_trial in language lifetime guidance; retire.
24. `faqs/sdk-engineer-faq.md:84` uses tenant_class paid in language lifetime guidance; retire.
25. `faqs/sdk-engineer-faq.md:85` uses tenant_class paid in language lifetime guidance; retire.
26. `faqs/sdk-engineer-faq.md:86` uses compliance_pack-bound paid in language lifetime guidance; retire.
27. `faqs/sdk-engineer-faq.md:101` uses compliance_pack-bound paid for custom fork policy; retire.
28. `faqs/sdk-engineer-faq.md:114` uses tenant_class paid for FIDO2 behavior; retire.
29. `faqs/sdk-engineer-faq.md:163` uses retired four-label ladder in long-tail language guidance; retire.
30. Direct retired-name candidate count: 28 cited lines.
31. Additional tier-word candidates without the retired names exist and should be included in the Wave 15J sweep.
32. `PRD.md:8` labels the service `tier: external-facing`; reword to product criticality or exposure class.
33. `PRD.md:82-85` uses hyperscaler tier wording; rework to context and capacity class.
34. `manifest.json:9` labels the service `tier`.
35. `manifest.json:145-163` uses T1/T2/T3 capability labels.
36. `manifest.json:274-278` has a `tenant_classs` block.
37. `manifest.json:294-327` uses tier classification and criticality tier metadata.
38. `ARCHITECTURE.md:23` labels the service tier.
39. `ARCHITECTURE.md:450` refers to Tier 0/1 paths and lower tiers.
40. `ARCHITECTURE.md:574` refers to tenant_class.
41. `cost-budget.md:19` uses billing_components contract.
42. `cost-budget.md:38` uses non-paid tier.
43. `onboarding/sdk-engineer-first-week.md:11` and `:110` use tier framing.
44. `decisions/ADR-SDK-0003-per-developer-sandbox-tenant-via-tenancy-µservice's-sandbox-.md:48` uses tenant class `sandbox`, which is not a retired tier name but is also not the target class set in this prompt.
45. `decisions/ADR-SDK-0006-kyc-pipeline-in-house;-no-external-kyc-saas-(onfido-persona-.md:48`, `:59`, `:152`, `:194`, and `:223` use risk-tier language; these may be legitimate risk severity buckets, but they need a non-capability-ladder glossary to avoid collision.
46. The retired-name list should not be converted into a new four-level tenant ladder.
47. OCI Always Free must be expressed as the OCI Always Free profile or demo-trial infrastructure profile, not as a capability ladder.

#### §3.4.C Tenant-Class Adoption Gaps

1. Finding: P2 tenant-class adoption gap exists.
2. Active target for this audit: `demo_trial`, `paid`, and `revenue_share` as tenant classes, per the user prompt.
3. Drift note: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:10-12` records a later two-class model with revenue share as a billing component, so this area needs one final authoritative machine-readable contract.
4. Repo evidence: `decisions/ADR-SDK-0003-per-developer-sandbox-tenant-via-tenancy-µservice's-sandbox-.md:48` uses `tenant_class="sandbox"`.
5. Repo evidence: `decisions/ADR-SDK-0003-per-developer-sandbox-tenant-via-tenancy-µservice's-sandbox-.md:153` says issued credentials are always scoped to sandbox tenant class.
6. Repo evidence: `decisions/ADR-SDK-0003-per-developer-sandbox-tenant-via-tenancy-µservice's-sandbox-.md:205` tests `tenant_class="sandbox"`.
7. Repo evidence: `contracts/openapi/developer-sdk.yaml:323` includes `revenue_share` as a payout ledger kind, not as a tenant class.
8. Repo evidence: `cost-budget.md:19` references paid billing components, not `paid` tenant_class semantics.
9. Repo evidence: `cost-budget.md:38` references non-paid billing components, not `demo_trial` semantics.
10. Repo evidence: no `demo_trial` string was found in the microservice path.
11. Repo evidence: no first-class `paid` tenant_class contract was found in the microservice path.
12. Repo evidence: no first-class `revenue_share` tenant_class contract was found in the microservice path.
13. The current service can describe sandbox behavior, payout revenue-share entries, and paid/non-paid cost split.
14. It cannot yet describe the replacement tenant-class model as a product contract.
15. Recommended correction: create one tenant-class vocabulary in the manifest or service contract after the shared tenant-class authority is settled.
16. Recommended correction: express demo-trial caps through usage quotas, OCI Always Free profile constraints, and best-effort SLO language.
17. Recommended correction: express paid scale through per-seat and per-usage billing, contractual SLOs, compliance pack flags, and BYOK permission.
18. Recommended correction: express revenue-share handling either as a tenant class per this prompt or as a paid billing component if the later memory is promoted.
19. Severity: P2 because tenant-class absence affects docs and control surfaces but can be corrected without changing runtime code yet.

### §3.5 Dimension 5 — Multi-Context Deployability

1. Finding: P1 multi-context readiness is not substantiated.
2. Canonical evidence: ADR-0328 D-15 defines all six contexts and target IaC dirs; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1738-1994`.
3. Canonical evidence: ADR-0328 D-15 requires any N/A context to specify reason, missing primitives, customer impact, remediation owner, and revisit gate; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2079-2084`.
4. Canonical evidence: the multi-context memory says every µservice PRD, ADR, and IP must specify contexts and absence is an audit finding; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_multi_context_provider_agnostic_2026_05_20.md:32-38`.
5. Repo evidence: no `iac/oyatie-public-cloud/` directory exists.
6. Repo evidence: no `iac/guest-on-aws/` directory exists.
7. Repo evidence: no `iac/oci-guest/` directory exists.
8. Repo evidence: no `iac/on-prem/` directory exists.
9. Repo evidence: no `iac/colo/` directory exists.
10. Repo evidence: no `iac/oyatie-iaas/` directory exists.
11. Repo evidence: `ARCHITECTURE.md:321-332` says cell eligibility is not declared in manifest and conservative defaults apply.
12. Repo evidence: `ARCHITECTURE.md:445-457` describes deployment through Helm and Kubernetes rather than six context modules.
13. Repo evidence: `PRD.md:127-157` lists broad context capabilities but does not provide canonical context IaC evidence.
14. Repo evidence: `multi-region.md` exists but does not substitute for the six context IaC matrix.
15. The service may eventually run in all six contexts because an SDK generator is not inherently provider-specific.
16. The current artifacts do not prove it.
17. The absence of N/A manifests means no context can be safely marked unsupported.
18. The absence of OpenTofu context modules means no context can be safely marked supported.
19. Recommended correction: declare all six contexts in a manifest and land context-specific OpenTofu modules or explicit N/A decisions.
20. Recommended correction: treat external package registries as per-context egress dependencies with DNS, credentials, audit, and fallback controls.
21. Recommended correction: separate generated SDK publication endpoints from developer identity or payout endpoints when context constraints differ.
22. Severity: P1 because canonical context support is a Wave 3 audit dimension and currently unproven.

### §3.6 Dimension 6 — OpenTofu IaC and Provisioning Discipline

1. Finding: P1 OpenTofu substrate gap exists.
2. Canonical evidence: ADR-0328 D-16 states that OpenTofu is the IaC engine and Terraform spelling is not canonical; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2249`.
3. Canonical evidence: ADR-0328 D-16 requires `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and `README.md` per context; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2296-2309`.
4. Canonical evidence: ADR-0328 D-16 requires module signing with sigstore/cosign evidence; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2357-2368`.
5. Canonical evidence: ADR-0328 D-16 says cloud-iac owns IaC orchestration and µservices must not invent separate provisioning CLIs; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2405-2434`.
6. Canonical evidence: ADR-0328 D-16 requires tenant onboarding through `tofu init`, `tofu plan`, and `tofu apply`; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2436-2456`.
7. Repo evidence: `iac/helm/` exists, but no canonical OpenTofu context directories exist.
8. Repo evidence: `iac/helm/developer-sdk-rest/Chart.yaml` and related Helm files are deploy manifests, not OpenTofu modules.
9. Repo evidence: `ARCHITECTURE.md:392` points to Helm paths as transport evidence.
10. Repo evidence: `ARCHITECTURE.md:445-457` describes Helm manifests, Kubernetes, OpenBao, Postgres, and registries.
11. Repo evidence: `implementation-plans/IP-001-layer-a-postgres-openbao-backstage-iac.md:39-41` directs chart creation.
12. Repo evidence: `implementation-plans/IP-001-layer-a-postgres-openbao-backstage-iac.md:92-94` validates with Helm install commands.
13. Repo evidence: `implementation-plans/IP-012-package-registry-vendored.md:40-43` references chart paths for registries.
14. Positive evidence: no Pulumi, CloudFormation, or Terraform Cloud provisioning path was observed in the target path.
15. Positive evidence: the current gap is mostly missing OpenTofu structure, not an observed competing IaC engine.
16. Negative evidence: Helm-only deployment does not satisfy per-context infrastructure ownership.
17. Negative evidence: no OCI Always Free OpenTofu profile exists.
18. Negative evidence: no plan/apply evidence model exists for tenant onboarding.
19. Recommended correction: keep Helm as Kubernetes packaging only when called by OpenTofu/cloud-iac, not as the canonical provisioning contract.
20. Recommended correction: add per-context OpenTofu modules that provision registry credentials, object storage, build workers, signing materials, queues, DB dependencies, egress policy, observability, and rate limits.
21. Recommended correction: add signed module evidence and CI verification for `tofu init`, `tofu fmt`, `tofu validate`, `tofu plan`, and signed artifact attestations.
22. Severity: P1 because context readiness and tenant onboarding cannot be proven without OpenTofu modules.

### §3.7 Dimension 7 — OS and Runtime Support

1. Finding: P2 OS support manifest is absent.
2. Canonical evidence: ADR-0328 D-17 requires `microservices/<name>/supported-oses.json`; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2907-2927`.
3. Canonical evidence: ADR-0328 D-17 defines required manifest row fields; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2930-2947`.
4. Canonical evidence: ADR-0328 D-17 defines Tier 1 OSes including Talos, RHEL, Oracle Linux, SLES, Ubuntu, Debian, Rocky, AlmaLinux, CentOS Stream, Amazon Linux, Flatcar, Photon, and macOS Apple Silicon M5+; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2659-2807`.
5. Canonical evidence: ADR-0328 D-17 calls out developer-sdk as a service that may support macOS M5+ while excluding Intel macOS; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2995-2996`.
6. Repo evidence: no `microservices/developer-sdk/supported-oses.json` exists.
7. Repo evidence: `sdk-plan.md:15-24` lists SDK language families but does not define OS support.
8. Repo evidence: `tutorials/generate-publish-and-verify-rust-typescript-python-sdks.md:23-28` shows generated SDK commands but not host OS validation.
9. Repo evidence: `benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md:95-104` has a reproducibility block but not an OS matrix.
10. Repo evidence: `ARCHITECTURE.md:445-457` says Kubernetes/Kata deployment but does not map host OS support.
11. Generated SDK support has a separate OS question from service runtime support.
12. Rust generator runtime support needs a server OS matrix for controllers and workers.
13. Generated SDK artifacts need language-specific compatibility matrices where package ecosystems require them.
14. Native output families such as Swift, Kotlin, C#/.NET, C, C++, and Rust require OS/architecture constraints for compile fixtures.
15. macOS Apple Silicon M5+ is material for generated SDK fixture testing and local developer tooling.
16. Intel macOS and M1-M4 should be explicitly out of scope if the canonical OS policy remains unchanged.
17. Recommended correction: add `supported-oses.json` with service runtime rows, generated SDK fixture rows, and explicit out-of-scope rows.
18. Recommended correction: include `target_triple`, `min_kernel_or_os_version`, `artifact_kind`, `ci_job`, and `last_verified_at` fields per ADR.
19. Recommended correction: express generated SDK compatibility without turning output languages into backend implementation languages.
20. Severity: P2 because the service has no implementation source yet, but the manifest absence blocks canonical compliance.

### §3.8 Dimension 8 — Rust-Strict Language Policy and Generated SDK Boundary

1. Finding: P1 language boundary is unclear in docs, even though no prohibited first-party source files were found.
2. Canonical evidence: ADR-0328 D-18 requires backend runtime, CLI, validation, codegen, scripting, and CI in Rust; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3047-3067`.
3. Canonical evidence: ADR-0328 D-18 allows `.tf`, `.cedar`, `.yaml`, `.json`, `.proto`, `.openslo.yaml`, `.sql`, and `.md`; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3085-3107`.
4. Canonical evidence: ADR-0328 D-18 defines Swift, Kotlin, WinUI 3, and Leptos frontend exceptions; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3119-3188`.
5. Canonical evidence: the Rust-strict memory says SDK clients may be generated outputs when provenance is recorded; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:51-60`.
6. Repo evidence: forbidden source extension scan found no `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, or `.fs` files under the service path.
7. Repo evidence: `manifest.json:194-204` pins Node and Python dependency versions in the microservice dependency model.
8. Repo evidence: `decisions/ADR-SDK-0007-dev-portal-as-backstage-extension-not-standalone-app.md:47-50` adopts a Backstage-style React plugin composition.
9. Repo evidence: `decisions/ADR-SDK-0007-dev-portal-as-backstage-extension-not-standalone-app.md:132` includes a Node operational path.
10. Repo evidence: `tutorials/generate-publish-and-verify-rust-typescript-python-sdks.md:91-145` shows generated SDK package-manager verification using language ecosystem tooling.
11. Repo evidence: `migration-playbooks/from-stripe-and-twilio-sdks.md:25-57` shows migration examples across several client ecosystems.
12. The generated SDK output boundary is legitimate if it is documented as generated artifact output, not first-party backend implementation.
13. The portal runtime boundary is not legitimate as currently written because it points to Backstage/React rather than the approved web frontend path.
14. The service should not own a generic developer portal runtime if the canonical developer-sdk identity remains generator-only.
15. The code generator itself should be Rust.
16. Generator templates can emit target-language SDK artifacts, but provenance, regeneration command, fixture hash, and publication attestation must be part of the generated-output boundary.
17. Tutorial examples can mention consumer-side SDK verification but should not look like first-party implementation instructions.
18. Recommended correction: replace Backstage/React portal ownership with generated docs/snippet artifacts or hand off portal UI to an approved frontend service.
19. Recommended correction: move Node/Python pins out of first-party dependency posture unless they are generated-client test harness tools with explicit exception/provenance boundaries.
20. Recommended correction: document every generated language artifact as generated output, not implementation language.
21. Severity: P1 because language-policy ambiguity can lead future implementation toward disallowed runtime choices.

### §3.9 Dimension 9 — Evidence, SLOs, Benchmarks, and Operational Readiness

1. Finding: P2 evidence corpus is broad but not generator-specific enough.
2. Canonical evidence: the verification directive says not to trust line count and to verify scope, quality, and hyperscaler substance; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-31`.
3. Canonical evidence: the docs-substance directive says scaffold and padding do not meet the bar; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-20`.
4. Repo evidence: `ARCHITECTURE.md:3` says the file was created by an anchor sweep and that stub sections should be expanded during review.
5. Repo evidence: `performance-bench.md:19-28` benchmarks catalog install, plugin vetting, sandbox reset, payout processing, and codegen, mixing generator and non-generator workloads.
6. Repo evidence: `performance-bench.md:36-40` uses Apple App Store, Stripe Connect, and VS Code Marketplace baselines rather than Stainless/Speakeasy/Fern.
7. Repo evidence: `benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md:1` uses Stripe, Twilio, Auth0, and AWS SDK v3 instead of generator counterparts.
8. Repo evidence: `benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md:22-41` measures generated SDK cold import and first-call overhead, which is useful but not enough for generator service performance.
9. Repo evidence: `capacity-model.md:15-24` models catalog, install, vetting, sandbox, payout, and signing traffic, not generator runs and package publishing at leader depth.
10. Repo evidence: `failure-modes.md:15-73` emphasizes Cedar, Postgres, Valkey, Wasmtime, Cosign, Trivy, OpenBao, bank rail, KYC, and audit failures.
11. Repo evidence: `slos/codegen-availability.openslo.yaml` exists and is directly relevant.
12. Repo evidence: `slos/portal-availability.openslo.yaml`, `slos/payout-settlement-availability.openslo.yaml`, and `slos/sandbox-reset-latency.openslo.yaml` may belong to other ownership boundaries after re-scope.
13. Repo evidence: `evidence-emission.md` exists and should be reoriented around generator run evidence.
14. Repo evidence: `incident-response.md:15-21` defines severity language, but the incident classes are not yet generator-first.
15. Repo evidence: `dpia.md:21-25` maps PII and payout flows, which may be mostly out-of-scope after generator re-scope.
16. Repo evidence: `dpia.md:46-48` references specific regions rather than all six deployment contexts.
17. A generator-first performance model should measure spec parse latency, validation latency, generation queue latency, per-language build time, generated package compile/test pass rates, publication latency, docs/snippet generation latency, and fixture replay latency.
18. A generator-first SLO model should isolate synchronous API latency from asynchronous generation completion time.
19. A generator-first capacity model should use spec size, endpoint count, schema count, target language count, template complexity, publication fanout, and fixture count.
20. A generator-first incident model should prioritize bad SDK generation, breaking package publication, credential leakage in generated code, incorrect retry/idempotency behavior, registry outage, signing failure, and unsafe template regression.
21. Recommended correction: preserve useful evidence machinery but realign metrics, SLOs, drills, and runbooks to SDK generator ownership.
22. Severity: P2 because evidence files exist, but they are not yet pointed at the right product surface.

## §4 Findings Table

| ID | Severity | Finding | Evidence | Required remediation |
| --- | --- | --- | --- | --- |
| SDK-AUD-001 | P1 | Product ownership is split across SDK generation, developer onboarding, marketplace/plugin review, sandboxing, KYC, payout, tax, and portal ownership. | `PRD.md:21-25`, `PHASE-01-DEVELOPER-SDK-SUBSTRATE.md:37`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:10-13` | Re-scope developer-sdk to SDK generator lifecycle and hand off non-generator ownership. |
| SDK-AUD-002 | P1 | Contract surface lacks canonical generate/publish/configuration/docs/fixture APIs. | `contracts/openapi/developer-sdk.yaml:147-171`, `contracts/proto/developer-sdk.proto:126-133`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:92-100` | Replace onboarding-first contract with generator-first API surface. |
| SDK-AUD-003 | P1 | Output language matrix is incomplete and inconsistent with canonical SDK output set. | `contracts/openapi/developer-sdk.yaml:161`, `contracts/asyncapi/developer-sdk-events.yaml:121`, `implementation-plans/IP-006-sdk-codegen-ts-rust-swift-kotlin-csharp-python.md:16-20`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:18-30` | Normalize target outputs to the current ten-family set and drop obsolete long-tail claims. |
| SDK-AUD-004 | P1 | Six deployment contexts are not backed by per-context IaC or N/A decisions. | `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1738-1994`, `specs/master-plan-sequencing.json:704-745`, no canonical context dirs found | Land context manifests and OpenTofu modules or explicit N/A records. |
| SDK-AUD-005 | P1 | OpenTofu substrate is absent; Helm-only artifacts do not satisfy IaC doctrine. | `ARCHITECTURE.md:392`, `implementation-plans/IP-001-layer-a-postgres-openbao-backstage-iac.md:92-94`, `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2625` | Introduce cloud-iac/OpenTofu context modules and treat Helm as packaging only. |
| SDK-AUD-006 | P1 | Portal and runtime docs point toward Backstage/React/Node surfaces that conflict with Rust-strict and approved frontend posture. | `manifest.json:194-204`, `decisions/ADR-SDK-0007-dev-portal-as-backstage-extension-not-standalone-app.md:47-50`, `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3047-3319` | Remove portal ownership or re-scope to approved frontend/generator artifacts. |
| SDK-AUD-007 | P1 | Existing parity and benchmark docs use wrong counterpart set. | `competitor-parity-matrix.md:15-27`, `benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md:1`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:15754` | Rebuild parity and benchmarks around Stainless, Speakeasy, and Fern. |
| SDK-AUD-008 | P2 | OS support manifest is missing. | no `microservices/developer-sdk/supported-oses.json`, `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2907-2947` | Add `supported-oses.json` with runtime and generated artifact fixture rows. |
| SDK-AUD-009 | P2 | OCI Always Free profile is not represented. | no `iac/oci-guest/always-free/`, `specs/master-plan-sequencing.json:857-867`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md:65-80` | Add OCI Always Free profile or explicit demo-trial constraint record. |
| SDK-AUD-010 | P2 | Tenant-class retirement migration candidates remain in tier matrix, FAQ, benchmark, tutorial, manifest, PRD, architecture, and cost docs. | §3.4.T cited lines | Retire four-level capability language and replace with tenant-class/billing/context semantics. |
| SDK-AUD-011 | P2 | Tenant-class model is not adopted; only sandbox and payout-kind fragments appear. | `decisions/ADR-SDK-0003-per-developer-sandbox-tenant-via-tenancy-µservice's-sandbox-.md:48`, `contracts/openapi/developer-sdk.yaml:323`, §3.4.C | Add one authoritative tenant-class contract after tenant-class doctrine is finalized. |
| SDK-AUD-012 | P2 | Cross-handoff paths disagree with actual repository paths. | `cross-microservice-handoffs.md:16-17`, actual `contracts/proto/developer-sdk.proto`, actual `policy/` directory | Correct path references and add checks that paths exist. |
| SDK-AUD-013 | P2 | Policy inventory claims an auditor-scope Cedar file that was not found. | `cross-microservice-handoffs.md:154-167`, inventory `policy/admin-scope.cedar`, `policy/developer-scope.cedar`, `policy/payout-scope.cedar`, `policy/public-read.cedar` | Add missing policy or correct the handoff ledger. |
| SDK-AUD-014 | P2 | Evidence and SLO files are substantial but not generator-first. | `performance-bench.md:19-28`, `capacity-model.md:15-24`, `failure-modes.md:15-73` | Rebase operational evidence on generator workloads and SDK artifact risks. |
| SDK-AUD-015 | P2 | `ARCHITECTURE.md` still carries anchor-sweep/scaffold language. | `ARCHITECTURE.md:3`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-20` | Replace scaffold notes with complete implementation-facing architecture text. |
| SDK-AUD-016 | P3 | No first-party prohibited source files were found, which reduces immediate implementation-language risk. | forbidden extension scan result, `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3085-3107` | Preserve this by keeping generator runtime Rust and generated SDKs provenance-scoped. |
| SDK-AUD-017 | P3 | Existing ADR-SDK-0002 contains useful deterministic codegen decisions worth preserving. | `decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:47-80`, `:207-235` | Retain deterministic generation and verification gates while expanding output surface. |

## §5 Open Questions

1. Which artifact becomes the authoritative tenant-class contract for developer-sdk after the prompt-level three-class model and later two-class memory model are reconciled?
2. Which existing microservice owns developer onboarding, KYC, signing-key custody, sandbox lifecycle, plugin marketplace, payout ledger, and tax packages after developer-sdk is narrowed to SDK generation?
3. Should generated docs and snippets remain developer-sdk outputs, or should a separate documentation service own rendering and publishing?
4. Should generated SDK publication to external registries be owned by developer-sdk directly, or mediated through a shared release/publication service?
5. What is the canonical API path naming for generator runs: `/sdks/generate` from the directive or `/v1/sdk/codegen/runs` from ADR-SDK-0002?
6. Which ten output SDK families are mandatory at GA, and which can be staged behind explicit launch gates without becoming capability ladders?
7. What generated SDK compatibility matrix is required for C and C++ package managers, given the Rust-strict first-party runtime posture?
8. Which OS rows belong to the service runtime, and which belong to generated SDK fixture testing?
9. What is the first acceptable OCI Always Free profile workload for developer-sdk: spec validation only, single-language generation, or full multi-language package publication?
10. What external registry credentials can developer-sdk hold, and which credentials must be delegated to a secret-management service?
11. Should the Backstage portal plan be retired outright, or converted into a handoff document for an approved frontend owner?
12. Should old marketplace and payout journeys be moved out of this microservice or retained as consumer journeys that call other services?
13. What evidence does cloud-iac require before developer-sdk can claim all six deployment contexts?
14. What generator workload defines industry-leader parity: endpoint count, schema count, language count, fixture count, publication fanout, or a composite benchmark?
15. What should happen to `capability-ladders/tier-matrix.md` during Wave 15J: delete, archive with retirement ADR, or transform into a tenant-class limits file?

## §5.1 Required Remediation Sequence

1. First remediation: freeze new development against the current broad developer-program surface until product ownership is narrowed.
2. Evidence basis: canonical developer-sdk direction rejects portal, doc site, CLI generator, runtime frontend, tutorial, and customer-credential ownership; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:10-13` and `:70-76`.
3. First acceptance test: PRD purpose names SDK generation, generated SDK verification, and package publication as the primary service purpose.
4. First non-acceptance signal: PRD still centers KYC, payout, tax, plugin marketplace, or generic portal runtime after the rewrite.
5. Second remediation: replace the counterpart set in local parity and benchmark docs.
6. Evidence basis: chat history says Stainless, Speakeasy, and Fern are the counterpart set, not Stripe SDK, Twilio helper libraries, or Auth0 SDK; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:15754`.
7. Second acceptance test: parity docs compare generator-platform features, not downstream SDK consumer libraries or app marketplaces.
8. Second non-acceptance signal: benchmark or parity docs still use `benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md:1` as a target comparison frame.
9. Third remediation: rewrite the OpenAPI control plane around generator lifecycle resources.
10. Evidence basis: canonical API surface includes `/sdks/generate`, `/publish`, versions, configurations, docs, and fixtures; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:92-100`.
11. Third acceptance test: every generator run has a request, status, artifacts, fixtures, publication, provenance, and error model.
12. Third non-acceptance signal: proto and OpenAPI contracts still expose onboarding, sandbox, and payout as first-class developer-sdk methods.
13. Fourth remediation: normalize output SDK families.
14. Evidence basis: directive requires TypeScript, Python, Go, Java, Kotlin, Swift, Rust, C#/.NET, C, and C++; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:18-30`.
15. Fourth acceptance test: contract enums, SDK plan, implementation plans, and AsyncAPI event payloads use one aligned output-family registry.
16. Fourth non-acceptance signal: `implementation-plans/IP-006-sdk-codegen-ts-rust-swift-kotlin-csharp-python.md:16-20` remains the sole language authority with only six families.
17. Fifth remediation: codify generated-output provenance.
18. Evidence basis: Rust-strict memory allows SDK clients as generated outputs with provenance while forbidding first-party backend implementation drift; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:51-60`.
19. Fifth acceptance test: every generated SDK artifact records input spec digest, generator version, template version, fixture result, package digest, signing proof, and publication target.
20. Fifth non-acceptance signal: tutorials or migration playbooks read as first-party implementation instructions rather than generated-client examples.
21. Sixth remediation: retire portal runtime ownership from developer-sdk.
22. Evidence basis: `decisions/ADR-SDK-0007-dev-portal-as-backstage-extension-not-standalone-app.md:47-50` adopts Backstage-style React composition, while canonical frontend policy allows Leptos web and native frontend lanes under controlled exceptions.
23. Sixth acceptance test: developer-sdk owns generated docs and snippets as artifacts; another approved frontend owner renders them.
24. Sixth non-acceptance signal: developer-sdk continues to own Backstage deployment as a service runtime component.
25. Seventh remediation: split non-generator implementation plans into handoffs.
26. Evidence basis: current IPs include KYC, signing-key custody, sandbox provisioning, plugin trust scoring, payout ledger, tax package, and vetting console plans.
27. Seventh acceptance test: each non-generator IP has a named owning service or explicit archival decision.
28. Seventh non-acceptance signal: non-generator IPs remain active under developer-sdk without boundary language.
29. Eighth remediation: create six-context OpenTofu scaffolds or N/A records.
30. Evidence basis: ADR-0328 D-15 and D-16 require context IaC and OpenTofu evidence; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1738-1994` and `:2243-2625`.
31. Eighth acceptance test: each required context has `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, README, and signed plan evidence or a complete N/A record.
32. Eighth non-acceptance signal: Helm remains the only deployment evidence under `iac/helm/`.
33. Ninth remediation: create the OCI Always Free profile.
34. Evidence basis: master plan requires `iac/oci-guest/always-free/`; see `specs/master-plan-sequencing.json:857-867`.
35. Ninth acceptance test: demo-trial infrastructure caps are expressed as workload, worker, storage, queue, and publication limits.
36. Ninth non-acceptance signal: any new document names the OCI profile through retired capability-ladder vocabulary.
37. Tenth remediation: add `supported-oses.json`.
38. Evidence basis: ADR-0328 D-17 requires per-microservice OS manifests; see `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2907-2947`.
39. Tenth acceptance test: runtime, worker, local CLI if any, and generated SDK fixture rows are distinct.
40. Tenth non-acceptance signal: generated SDK support claims remain detached from OS and architecture fixture evidence.
41. Eleventh remediation: settle tenant_class semantics.
42. Evidence basis: this prompt requires `demo_trial`, `paid`, and `revenue_share`, while the later memory records a two-class variant; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:10-12`.
43. Eleventh acceptance test: one machine-readable contract defines valid tenant_class values and billing components.
44. Eleventh non-acceptance signal: `sandbox` remains the only tenant_class value in developer-sdk docs.
45. Twelfth remediation: execute Wave 15J tier retirement inside this microservice.
46. Evidence basis: the no-ladder memory records the retirement directive; see `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_capability_ladder_2026_05_20.md:10-44`.
47. Twelfth acceptance test: retired capability-ladder names, T1/T2/T3 capability classes, and tier metadata are removed or transformed into tenant-class, deployment-context, usage-cap, or product-criticality fields.
48. Twelfth non-acceptance signal: `capability-ladders/tier-matrix.md` remains active without retirement status.
49. Thirteenth remediation: replace benchmark files with generator-first measurement plans.
50. Evidence basis: current benchmark docs use marketplace and downstream SDK baselines; see `performance-bench.md:19-40` and `benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md:1`.
51. Thirteenth acceptance test: benchmarks measure contract ingestion, validation, generation, fixtures, docs, package publication, rollback, and provenance.
52. Thirteenth non-acceptance signal: benchmark targets remain tied to catalog, payout, sandbox, or old comparison sets.
53. Fourteenth remediation: rebase SLOs onto generator workloads.
54. Evidence basis: `slos/codegen-availability.openslo.yaml` is relevant, while portal, payout, sandbox, and catalog SLOs may belong to other ownership boundaries.
55. Fourteenth acceptance test: generator SLOs separate synchronous API latency from asynchronous completion time and publication proof.
56. Fourteenth non-acceptance signal: SLOs continue to imply developer-sdk owns payout or marketplace uptime.
57. Fifteenth remediation: fix path contradictions in handoff docs.
58. Evidence basis: `cross-microservice-handoffs.md:16-17` names `developer_sdk.proto` and `policies/`, while the inventory shows `contracts/proto/developer-sdk.proto` and `policy/`.
59. Fifteenth acceptance test: every cited path in handoff docs exists or is marked as an intentional future path with owner.
60. Fifteenth non-acceptance signal: handoff docs continue to cite files absent from the microservice tree.
61. Sixteenth remediation: preserve useful deterministic codegen decisions from ADR-SDK-0002.
62. Evidence basis: `decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:47-80` and `:207-235` contain reusable determinism and verification gates.
63. Sixteenth acceptance test: the rewritten generator ADR keeps deterministic rerun, fixture, compile, and publication gates while updating product scope and output-family count.
64. Sixteenth non-acceptance signal: useful verification logic is deleted only because the old six-family framing is stale.
65. Final remediation checkpoint: do not start implementation until product boundary, contracts, context IaC, OS manifest, tenant_class contract, and tier retirement decisions are consistent.
66. Final evidence checkpoint: every remaining claim should cite a repo path, canonical doc path, memory file, or chat-history line.
67. Final stop condition: developer-sdk can be described in one sentence as a Rust-backed SDK generator with generated artifacts, fixtures, publication, provenance, and context-aware deployment evidence.

<!-- ORCHESTRATOR REPORT
  µservice: developer-sdk
  deliverables_landed: microservices/developer-sdk/coherence-audit-2026-05-20.md: 659 lines; microservices/developer-sdk/feature-parity-matrix-2026-05-20.md: 475 lines; microservices/developer-sdk/performance-benchmark-numbers-2026-05-20.md: 369 lines
  inventory_files_seen: 137
  inventory_lines_read: 15681
  chat_history_matches_processed: 139
  findings_p0: 0
  findings_p1: 7
  findings_p2: 8
  findings_p3: 2
  tier_retirement_candidates_found: 28 direct retired-name lines: tutorials/generate-publish-and-verify-rust-typescript-python-sdks.md:10; benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md:11,12,82; capability-ladders/tier-matrix.md:11,24,28,33,43,45,50,60,62,67,77,93,95; faqs/sdk-engineer-faq.md:28,29,66,83,84,85,86,101,114,163
  tenant_class_adoption_gaps: yes; no first-class demo_trial/paid/revenue_share tenant_class contract, with only sandbox tenant_class and revenue_share payout kind fragments found
  top_3_counterparts_confirmed: Stainless / Speakeasy / Fern
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1503
-->
