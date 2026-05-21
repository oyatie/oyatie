# Forms Ownership-Coherence Audit - 2026-05-20

Audit owner: single-agent audit lane for `microservices/forms/`.
Target microservice: `forms`.
Deployable-context presumption: all six canonical contexts unless this audit finds a documented reason to narrow scope.
Industry counterparts: Google Forms, Typeform, SurveyMonkey.
Deliverable set: this coherence audit, the feature parity matrix, and the performance benchmark numbers document.
Retired deliverable: no tenant-class-deltas document is authored because the tier system is retired by the 2026-05-20 directive.
Write scope: only `microservices/forms/` received new files.
Audit date basis: 2026-05-20 wave directive, executed on 2026-05-21.

## 1. Purpose

This audit evaluates whether the forms microservice is internally coherent, aligned with canonical direction, and comparable to the stated industry-counterpart union surface.
The service purpose in the PRD is to manage typed form definitions, response capture, and survey distribution for customer-facing and internal journeys, with cross-service integrations for sheets, drive, mail, messenger, workflow, analytics, payments, signatures, and AI-assisted form generation (`microservices/forms/PRD.md:30-68`).
The counterpart comparison target is intentionally narrower than the broad competitor list in the local parity artifact: this batch uses Google Forms, Typeform, and SurveyMonkey (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16311`).
The audit is read-only with respect to existing artifacts; findings are cataloged for follow-up remediation rather than repaired in-place.
The audit follows the microservice ownership directive: one agent owns one service and writes deliverables only under that service path (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:10-18`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:75-83`).
The audit uses evidence citations for each finding and avoids completion-by-line-count, matching the verification directive (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-12`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-18`).
The audit uses the 2026-05-20 tier-retirement amendment: demo_trial, paid, paid, and paid compliance_pack are not new planning surfaces; existing references are Wave 15J retirement candidates (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_2026_05_20.md:10-24`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_2026_05_20.md:28-45`).
The replacement commercialization model for this batch is three tenant classes: `demo_trial`, `paid`, and `revenue_share`, while quality remains uniformly industry-leader-grade across all classes.
The canonical multi-context rule requires six contexts to be explicit, with any non-applicable context justified by reason, missing primitives, customer impact, owner, and revisit gate (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-1736`, `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2079-2087`).
The canonical IaC rule requires OpenTofu, not Terraform, and requires per-context modules under the microservice `iac/` tree (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2241-2249`, `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2275-2294`).
The canonical OS rule requires a service-level supported OS manifest and explicit coverage across Linux, Windows, and Apple platforms (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3948-3999`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:10-31`).
The canonical language rule keeps backend implementation Rust-strict, with non-Rust limited to sanctioned config, contract, policy, documentation, and approved frontend surfaces (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4011-4080`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:10-18`).
The OCI Always Free rule requires a per-service `iac/oci-guest/always-free/` module with budget outputs and a zero-cost demo/trial posture (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3514-3517`, `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3679-3700`).
The substance bar requires docs that a junior engineer can build from, with concrete numbers, contracts, runbooks, and acceptance checks rather than scaffold text (`docs/standards/brief-template.md:1431-1471`, `docs/standards/brief-template.md:1727-1751`).
The present service is rich in authored docs and contracts, but the canonical substrate surfaces are incomplete.
The strongest positive signal is breadth: PRD, architecture, ADRs, IPs, contracts, SLOs, policy, runbooks, parity, capacity, cost, compliance, DPIA, and tutorials are all present in some form.
The strongest negative signal is mismatch: the service has substantial feature ambition but lacks the canonical deployment, OS, OpenTofu, tenant_class, and tier-retirement alignment needed for Wave 3 ownership coherence.
The audit conclusion is not a product rejection; it is a gate state: forms is feature-rich on paper but not canonically coherent enough to claim ready all-context service ownership.

## 2. Inventory

Inventory method: `rg --files microservices/forms | sort` plus recursive file count.
Inventory file count: 145 files under `microservices/forms/`.
Inventory line count surface: 20,257 total lines under `microservices/forms/` from `find microservices/forms -type f -print0 | xargs -0 wc -l`.
Key absence check: top-level `README.md` is absent under `microservices/forms/`, while PRD and ARCHITECTURE exist.
Key absence check: `supported-oses.json` is absent under `microservices/forms/`.
Key absence check: canonical context modules `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/oci-guest/always-free/`, `iac/on-prem/`, `iac/colo/`, and `iac/oyatie-iaas/` are absent.
Key presence check: non-canonical `iac/terraform/` is present.
Key presence check: no application-language files matching `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, or `.fs` were found under `microservices/forms/`.
Key search check: no `tenant_class`, `demo_trial`, or `revenue_share` terms were found under `microservices/forms/`; incidental `paid` occurrences are ordinary prose, not tenant-class semantics.

### 2.1 Complete File Inventory

001. `microservices/forms/ARCHITECTURE.md` - architecture document; contains anchor-sweep scaffold language and unknown bindings.
002. `microservices/forms/AUDIT-FINDINGS-2026-05-18.json` - prior audit findings data.
003. `microservices/forms/IP-001-layer-a-postgres-valkey-meilisearch-clamav-waf-cdn-captcha-iac.md` - implementation plan for layer A substrate.
004. `microservices/forms/IP-002-form-field-section-response-domain-kernel.md` - implementation plan for forms domain kernel.
005. `microservices/forms/IP-003-conditional-logic-engine-cel.md` - implementation plan for conditional logic.
006. `microservices/forms/IP-004-validation-engine.md` - implementation plan for validation.
007. `microservices/forms/IP-005-versioning-and-changeset-binding.md` - implementation plan for version binding.
008. `microservices/forms/IP-006-postgres-citus-adapter-with-column-encryption.md` - implementation plan for encrypted persistence.
009. `microservices/forms/IP-007-valkey-adapter.md` - implementation plan for cache adapter.
010. `microservices/forms/IP-008-meilisearch-adapter.md` - implementation plan for search adapter.
011. `microservices/forms/IP-009-captcha-adapter.md` - implementation plan for anti-abuse adapter.
012. `microservices/forms/IP-010-form-builder-leptos-wasm.md` - implementation plan for approved web frontend technology.
013. `microservices/forms/IP-011-form-renderer.md` - implementation plan for renderer.
014. `microservices/forms/IP-012-response-collector-rest.md` - implementation plan for REST collector.
015. `microservices/forms/IP-013-bulk-distribute-worker.md` - implementation plan for distribution worker.
016. `microservices/forms/IP-014-export-worker.md` - implementation plan for exports.
017. `microservices/forms/IP-015-hg-forms-registration.md` - implementation plan for registry/registration.
018. `microservices/forms/IP-journey-j100-pack-rollout-first-action.md` - journey implementation plan.
019. `microservices/forms/IP-journey-j136-per-jurisdiction-enrollment-forms.md` - journey implementation plan.
020. `microservices/forms/IP-journey-j54-quote-request.md` - journey implementation plan.
021. `microservices/forms/IP-journey-j58-self-assessment.md` - journey implementation plan.
022. `microservices/forms/IP-journey-j60-role-scope-intake.md` - journey implementation plan.
023. `microservices/forms/IP-journey-j61-patient-intake.md` - journey implementation plan.
024. `microservices/forms/IP-journey-j63-eligibility-interest.md` - journey implementation plan.
025. `microservices/forms/IP-journey-j91-us-msb-mtl-overlay.md` - journey implementation plan.
026. `microservices/forms/IP-journey-j92-br-lgpd-us-parent-dsar.md` - journey implementation plan.
027. `microservices/forms/IP-journey-j93-in-dpdpa-rbi-overlay.md` - journey implementation plan.
028. `microservices/forms/IP-journey-j94-sox404-public-company-controls.md` - journey implementation plan.
029. `microservices/forms/IP-journey-j95-iso27001-soc2-annual-audit.md` - journey implementation plan.
030. `microservices/forms/IP-journey-j96-ksa-uae-mena-onboarding.md` - journey implementation plan.
031. `microservices/forms/IP-journey-j97-sg-pdpa-mas-tenant.md` - journey implementation plan.
032. `microservices/forms/IP-journey-j98-au-privacy-apra-cps234.md` - journey implementation plan.
033. `microservices/forms/IP-journey-j99-multi-pack-conflict-resolution.md` - journey implementation plan.
034. `microservices/forms/PHASE-01-FORMS-FOUNDATION.md` - phase plan.
035. `microservices/forms/PRD.md` - primary product requirements document.
036. `microservices/forms/backfill-replay.md` - replay and backfill guidance.
037. `microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md` - prior counterpart benchmark artifact with retired tier terminology.
038. `microservices/forms/capabilities/T0-suggest.yaml` - capability definition using T0 vocabulary.
039. `microservices/forms/capabilities/T1-assist.yaml` - capability definition using T1 vocabulary.
040. `microservices/forms/capabilities/T2-auto.yaml` - capability definition using T2 vocabulary.
041. `microservices/forms/tenant-class/tier-matrix.md` - retired tier matrix; direct Wave 15J candidate.
042. `microservices/forms/capacity-model.md` - capacity model.
043. `microservices/forms/catalog/oya-forms-bulk-distribute-worker.yaml` - service catalog entry.
044. `microservices/forms/catalog/oya-forms-captcha-adapter.yaml` - service catalog entry.
045. `microservices/forms/catalog/oya-forms-conditional-logic-domain.yaml` - service catalog entry.
046. `microservices/forms/catalog/oya-forms-crypto-domain.yaml` - service catalog entry.
047. `microservices/forms/catalog/oya-forms-domain.yaml` - service catalog entry.
048. `microservices/forms/catalog/oya-forms-export-worker.yaml` - service catalog entry.
049. `microservices/forms/catalog/oya-forms-form-builder-leptos-wasm.yaml` - service catalog entry.
050. `microservices/forms/catalog/oya-forms-form-renderer.yaml` - service catalog entry.
051. `microservices/forms/catalog/oya-forms-meilisearch-adapter.yaml` - service catalog entry.
052. `microservices/forms/catalog/oya-forms-postgres-adapter.yaml` - service catalog entry.
053. `microservices/forms/catalog/oya-forms-valkey-adapter.yaml` - service catalog entry.
054. `microservices/forms/catalog/oya-forms-response-collector-rest.yaml` - service catalog entry.
055. `microservices/forms/catalog/oya-forms-validation-domain.yaml` - service catalog entry.
056. `microservices/forms/catalog/oya-forms-version-domain.yaml` - service catalog entry.
057. `microservices/forms/competitor-parity-matrix.md` - local parity matrix; broader than this batch target.
058. `microservices/forms/compliance.md` - compliance artifact.
059. `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml` - AsyncAPI contract.
060. `microservices/forms/contracts/openapi/forms.openapi.yaml` - OpenAPI contract.
061. `microservices/forms/contracts/proto/forms.proto` - protobuf contract.
062. `microservices/forms/cost-budget.md` - cost artifact.
063. `microservices/forms/dashboards/ai-form-build-quality.json` - dashboard artifact.
064. `microservices/forms/dashboards/embed-and-distribution.json` - dashboard artifact.
065. `microservices/forms/dashboards/response-pipeline.json` - dashboard artifact.
066. `microservices/forms/decisions/ADR-FORMS-0001-form-definition-schema.md` - ADR.
067. `microservices/forms/decisions/ADR-FORMS-0002-captcha-and-anti-spam.md` - ADR.
068. `microservices/forms/decisions/ADR-FORMS-0003-pii-column-encryption-and-residency.md` - ADR.
069. `microservices/forms/decisions/ADR-FORMS-0004-conditional-logic-and-branching-engine.md` - ADR.
070. `microservices/forms/decisions/ADR-FORMS-0005-ai-form-build-bounds.md` - ADR with tenant-class vocabulary.
071. `microservices/forms/decisions/ADR-FORMS-0006-e-signature-conformance.md` - ADR with tenant-tier vocabulary.
072. `microservices/forms/decisions/ADR-FRM-001-logic-jump-evaluator-with-conditional-cedar-permit-per-question.md` - ADR.
073. `microservices/forms/decisions/README.md` - ADR index.
074. `microservices/forms/dpia.md` - privacy impact assessment.
075. `microservices/forms/failure-modes.md` - failure-mode artifact.
076. `microservices/forms/faqs/forms-engineer-faq.md` - engineer FAQ with retired tier vocabulary and broken runbook reference.
077. `microservices/forms/iac/helm/captcha-sidecar/Chart.yaml` - Helm chart.
078. `microservices/forms/iac/helm/captcha-sidecar/values.yaml` - Helm values.
079. `microservices/forms/iac/helm/form-cdn/Chart.yaml` - Helm chart.
080. `microservices/forms/iac/helm/form-cdn/values.yaml` - Helm values.
081. `microservices/forms/iac/helm/form-rest/Chart.yaml` - Helm chart.
082. `microservices/forms/iac/helm/form-rest/templates/prometheusrule.yaml` - Helm template.
083. `microservices/forms/iac/helm/form-rest/values.yaml` - Helm values.
084. `microservices/forms/iac/helm/form-waf/Chart.yaml` - Helm chart.
085. `microservices/forms/iac/helm/form-waf/values.yaml` - Helm values.
086. `microservices/forms/iac/helm/response-cache-valkey/Chart.yaml` - Helm chart.
087. `microservices/forms/iac/helm/response-cache-valkey/values.yaml` - Helm values.
088. `microservices/forms/iac/helm/response-search-meilisearch/Chart.yaml` - Helm chart.
089. `microservices/forms/iac/helm/response-search-meilisearch/templates/deployment.yaml` - Helm template.
090. `microservices/forms/iac/helm/response-search-meilisearch/templates/networkpolicy.yaml` - Helm template.
091. `microservices/forms/iac/helm/response-search-meilisearch/values.yaml` - Helm values.
092. `microservices/forms/iac/helm/response-store-postgres/Chart.yaml` - Helm chart.
093. `microservices/forms/iac/helm/response-store-postgres/values.yaml` - Helm values.
094. `microservices/forms/iac/helm/upload-scan-clamav/Chart.yaml` - Helm chart.
095. `microservices/forms/iac/helm/upload-scan-clamav/values.yaml` - Helm values.
096. `microservices/forms/iac/kustomize/base/cdn-edge-config.yaml` - Kustomize resource.
097. `microservices/forms/iac/kustomize/base/kustomization.yaml` - Kustomize base.
098. `microservices/forms/iac/kustomize/base/namespace.yaml` - Kustomize resource.
099. `microservices/forms/iac/kustomize/base/openbao-secret-references.yaml` - Kustomize resource.
100. `microservices/forms/iac/kustomize/base/service-mesh-tenant-headers.yaml` - Kustomize resource.
101. `microservices/forms/iac/kustomize/overlays/pack-ae/kustomization.yaml` - pack overlay.
102. `microservices/forms/iac/kustomize/overlays/pack-au/kustomization.yaml` - pack overlay.
103. `microservices/forms/iac/kustomize/overlays/pack-br/kustomization.yaml` - pack overlay.
104. `microservices/forms/iac/kustomize/overlays/pack-eu/kustomization.yaml` - pack overlay.
105. `microservices/forms/iac/kustomize/overlays/pack-in/kustomization.yaml` - pack overlay.
106. `microservices/forms/iac/kustomize/overlays/pack-jp/kustomization.yaml` - pack overlay.
107. `microservices/forms/iac/kustomize/overlays/pack-kr/kustomization.yaml` - pack overlay.
108. `microservices/forms/iac/kustomize/overlays/pack-ksa/kustomization.yaml` - pack overlay.
109. `microservices/forms/iac/kustomize/overlays/pack-sg/kustomization.yaml` - pack overlay.
110. `microservices/forms/iac/kustomize/overlays/pack-us-healthcare/kustomization.yaml` - pack overlay.
111. `microservices/forms/iac/kustomize/overlays/pack-us/kustomization.yaml` - pack overlay.
112. `microservices/forms/iac/terraform/cdn-edge-config.tf` - forbidden Terraform surface.
113. `microservices/forms/iac/terraform/template-marketplace-publishers.tf` - forbidden Terraform surface.
114. `microservices/forms/incident-response.md` - incident response artifact.
115. `microservices/forms/manifest.json` - service manifest.
116. `microservices/forms/migration-playbooks/from-google-forms-and-typeform.md` - migration playbook with retired tier vocabulary.
117. `microservices/forms/multi-region.md` - multi-region artifact.
118. `microservices/forms/onboarding/forms-engineer-first-week.md` - onboarding guide.
119. `microservices/forms/policy/auditor-scope.cedar` - Cedar policy.
120. `microservices/forms/policy/ci-scope.cedar` - Cedar policy.
121. `microservices/forms/policy/data-residency.md` - policy doc.
122. `microservices/forms/policy/dual-context.md` - policy doc.
123. `microservices/forms/policy/public-read.cedar` - Cedar policy.
124. `microservices/forms/policy/tenant-scope.cedar` - Cedar policy.
125. `microservices/forms/reference-implementations/submit-form-and-export-rust-sdk.md` - Rust SDK reference implementation.
126. `microservices/forms/runbooks/ai-form-build-rollback.md` - runbook.
127. `microservices/forms/runbooks/captcha-degraded.md` - runbook.
128. `microservices/forms/runbooks/embed-iframe-csp-incident.md` - runbook.
129. `microservices/forms/runbooks/export-pipeline-failure.md` - runbook.
130. `microservices/forms/runbooks/pii-leak-incident-p0.md` - runbook.
131. `microservices/forms/runbooks/response-store-corruption.md` - runbook.
132. `microservices/forms/runbooks/spam-flood-throttle.md` - runbook.
133. `microservices/forms/scorecards/overrides.json` - scorecard override.
134. `microservices/forms/sdk-plan.md` - SDK plan.
135. `microservices/forms/slos/accessibility-wcag-correctness.openslo.yaml` - SLO artifact.
136. `microservices/forms/slos/ai-form-build-latency.openslo.yaml` - SLO artifact.
137. `microservices/forms/slos/analytics-render-latency.openslo.yaml` - SLO artifact.
138. `microservices/forms/slos/bulk-distribute-latency.openslo.yaml` - SLO artifact.
139. `microservices/forms/slos/export-csv-latency.openslo.yaml` - SLO artifact.
140. `microservices/forms/slos/field-validate-latency.openslo.yaml` - SLO artifact.
141. `microservices/forms/slos/form-render-latency.openslo.yaml` - SLO artifact.
142. `microservices/forms/slos/pii-encryption-correctness.openslo.yaml` - SLO artifact.
143. `microservices/forms/slos/submission-latency.openslo.yaml` - SLO artifact.
144. `microservices/forms/threat-model.md` - threat model.
145. `microservices/forms/tutorials/build-multi-page-survey-with-logic-jump-payment-warehouse.md` - tutorial with retired tier vocabulary.

## 3. Nine-Dimension Audit

### 3.1 Dimension 1 - Product Purpose and Counterpart Fit

Assessment: partial pass.
The PRD states the core purpose clearly: typed form definitions, response capture, and survey distribution (`microservices/forms/PRD.md:30-38`).
The functional surface includes builder, logic jumps, validation, file upload, signatures, payments, quiz scoring, analytics, export, embed, distribution, workflow triggers, and AI build (`microservices/forms/PRD.md:44-68`).
The local competitor matrix already recognizes Google Forms, Typeform, and SurveyMonkey as relevant forms/survey counterparts, though it also includes Jotform, Tally, Paperform, Formstack, Qualtrics, Wufoo, and Airtable Forms (`microservices/forms/competitor-parity-matrix.md:15-30`).
The batch target narrows the comparison to Google Forms, Typeform, and SurveyMonkey, matching chat-history direction (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16311`).
Forms has a plausible product identity: it is not only a survey editor; it is an Oyatie-governed intake, consent, response, and workflow capture service.
Forms also claims advanced surfaces beyond the three counterparts: per-question data classes, pack-bound signatures, policy evaluation, workflow events, ontology projections, and audit-chain anchoring (`microservices/forms/PRD.md:156-193`).
This advanced purpose is coherent with Oyatie's governed-platform ambition, but it increases the evidence burden for contracts, policy, deployment, and compliance surfaces.
The present docs over-index on feature breadth and under-index on canonical substrate proof.
The counterpart union does not require Oyatie to copy every consumer UX detail; it requires no headline gap against the aggregate industry surface.
Current local artifacts show many union capabilities as intended, not implemented.
Finding severity for product fit: P2 because purpose is strong but evidence is uneven.

### 3.2 Dimension 2 - Artifact Completeness and Internal Surface Coverage

Assessment: partial pass with missing mandatory surface.
Positive evidence: PRD, architecture, manifest, ADRs, IPs, contracts, OpenSLO files, policies, runbooks, dashboards, capacity, cost, DPIA, compliance, failure modes, onboarding, migration, tutorial, and reference implementation are present.
Negative evidence: top-level `README.md` is absent from the microservice path, even though the batch explicitly asked for `PRD.md / ARCHITECTURE.md / README.md`.
Negative evidence: the PRD links to `/specs/microservices/forms.json`, but `specs/microservices/forms.json` is absent in the live checkout (`microservices/forms/PRD.md:218`).
Negative evidence: the PRD references `policy/embed-csp.md`, but that file is absent in the forms path (`microservices/forms/PRD.md:50`).
Negative evidence: the FAQ references `runbooks/warehouse-export-lag.md`, but that runbook is absent (`microservices/forms/faqs/forms-engineer-faq.md:75`).
Negative evidence: the PRD references `legal/ai-act-conformity.md`, but no such forms-local legal artifact exists (`microservices/forms/PRD.md:201`).
The architecture document still says it was created by a Wave-3-C anchor sweep and that stub sections should be expanded in a content-pass review (`microservices/forms/ARCHITECTURE.md:3`).
The architecture document lists `forms.unknown` for state/event binding, which is not a junior-startable architecture surface (`microservices/forms/ARCHITECTURE.md:35`).
The architecture document includes tenant-scoping placeholders such as `forms.unknown` and planned table/event names, which indicate scaffold residue (`microservices/forms/ARCHITECTURE.md:133-149`).
The manifest is detailed but has `ontology_projections: []`, while the PRD claims ontology writes for `FormTemplate`, `QuestionNode`, `ResponseMetric`, and `LeadSignal` (`microservices/forms/manifest.json:303`, `microservices/forms/PRD.md:187-193`).
The service has enough documentation breadth to support an audit, but not enough coherence to pass a build-from-docs standard.
Finding severity for artifact completeness: P1 for missing canonical surfaces and scaffold language in the primary architecture doc.

### 3.3 Dimension 3 - Contract and API Coherence

Assessment: partial pass.
OpenAPI is present at `contracts/openapi/forms.openapi.yaml` and declares OpenAPI 3.2.0 (`microservices/forms/contracts/openapi/forms.openapi.yaml:1`).
AsyncAPI is present at `contracts/asyncapi/forms.asyncapi.yaml` and declares AsyncAPI 3.1.0 (`microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:1`).
The PRD says OpenAPI 3.2.0 and AsyncAPI 3.0, so AsyncAPI version prose has drifted from the contract file (`microservices/forms/PRD.md:136`, `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:1`).
The AsyncAPI description also says AsyncAPI 3.0 while the file declares 3.1.0, creating an internal contradiction inside the same artifact (`microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:1`, `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:6`).
The OpenAPI contract exposes `/ai-build` and requires `prompt` plus `tier`, continuing T0/T1/T2 tenant-class vocabulary in the API body (`microservices/forms/contracts/openapi/forms.openapi.yaml:263-285`, `microservices/forms/contracts/openapi/forms.openapi.yaml:541-545`).
The AsyncAPI contract also carries `tier` in AI build-related message payloads (`microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:200-207`).
The protobuf contract declares `CapabilityTier` and uses it in `AiBuildDraftRequest` (`microservices/forms/contracts/proto/forms.proto:74-80`, `microservices/forms/contracts/proto/forms.proto:193-197`).
The protobuf file includes Go and Java package options (`microservices/forms/contracts/proto/forms.proto:16-18`); this is acceptable only as schema metadata for generated clients, not as backend implementation direction under the Rust-strict policy.
The contracts have enough shape to represent the service, but the tier vocabulary is now a Wave 15J API retirement candidate.
The contracts do not express the replacement `tenant_class` model anywhere.
Finding severity for contract coherence: P2 because contracts exist but carry retired vocabulary and version drift.

### 3.4 Dimension 4 - Canonical-Direction Alignment

Assessment: fail until remediated.
Canonical direction requires six deployment contexts and explicit service-level context posture (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-1736`, `specs/master-plan-sequencing.json:704-745`).
Forms has no canonical per-context OpenTofu module directories for the six contexts.
The only `iac/` children found are Helm, Kustomize, and Terraform, which are not the canonical context module layout.
Canonical direction requires OpenTofu and forbids Terraform as an active engine (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2241-2249`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md:10-17`).
Forms contains `iac/terraform/cdn-edge-config.tf` with a `terraform {}` block and `required_version = ">= 1.6.0"` (`microservices/forms/iac/terraform/cdn-edge-config.tf:5-6`).
Forms contains a second Terraform file for marketplace publishers (`microservices/forms/iac/terraform/template-marketplace-publishers.tf:1-31`).
Canonical direction requires OCI Always Free modeling through `iac/oci-guest/always-free/` and budget outputs (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3679-3700`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md:65-82`).
Forms has no `iac/oci-guest/always-free/` directory.
Canonical direction requires a service-level supported OS manifest (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3948-3999`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:56-76`).
Forms has no `supported-oses.json`.
Canonical direction requires Rust-strict backend surfaces (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4011-4080`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:51-60`).
The extension scan found no `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, or `.fs` files under forms, so the file tree passes the obvious forbidden-language scan.
The service still needs Rust implementation evidence; absence of forbidden languages is not the same as completion.
Canonical direction requires substance over scaffold; the architecture document's anchor-sweep language and `unknown` bindings violate that bar (`microservices/forms/ARCHITECTURE.md:3`, `microservices/forms/ARCHITECTURE.md:35`).
This dimension is the primary blocker for ownership coherence.

#### 3.4.T Tier Retirement Candidates

Tier retirement rule: direct demo_trial, paid, paid, and paid compliance_pack references are Wave 15J retirement candidates by default severity P2.
Direct candidate count: 34 direct demo_trial/paid/paid/paid compliance_pack references under `microservices/forms/`.
Candidate 01: `microservices/forms/tenant-class/tier-matrix.md:13` uses demo_trial in a retired capability matrix heading.
Candidate 02: `microservices/forms/tenant-class/tier-matrix.md:52` uses paid in a retired capability matrix heading.
Candidate 03: `microservices/forms/tenant-class/tier-matrix.md:54` compares new content to demo_trial.
Candidate 04: `microservices/forms/tenant-class/tier-matrix.md:62` uses demo_trial in field-type scope.
Candidate 05: `microservices/forms/tenant-class/tier-matrix.md:92` uses paid in a retired capability matrix heading.
Candidate 06: `microservices/forms/tenant-class/tier-matrix.md:94` compares new content to paid.
Candidate 07: `microservices/forms/tenant-class/tier-matrix.md:126` uses paid in cost delta.
Candidate 08: `microservices/forms/tenant-class/tier-matrix.md:130` uses paid compliance_pack in a retired capability matrix heading.
Candidate 09: `microservices/forms/tenant-class/tier-matrix.md:132` compares new content to paid.
Candidate 10: `microservices/forms/tenant-class/tier-matrix.md:145` uses paid in latency posture.
Candidate 11: `microservices/forms/tenant-class/tier-matrix.md:147` uses paid in SLO posture.
Candidate 12: `microservices/forms/tenant-class/tier-matrix.md:160` uses demo_trial, paid, paid, and paid compliance_pack in migration chain prose.
Candidate 13: `microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:13` uses paid for hardware.
Candidate 14: `microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:21` uses paid in benchmark row.
Candidate 15: `microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:31` uses paid in interpretation.
Candidate 16: `microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:33` uses paid in target statement.
Candidate 17: `microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:39` uses paid in benchmark row.
Candidate 18: `microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:47` uses paid in interpretation.
Candidate 19: `microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:53` uses paid in benchmark row.
Candidate 20: `microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:91` uses paid in TCO row.
Candidate 21: `microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:92` uses paid in TCO row.
Candidate 22: `microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:111` uses `oyatie-paid` in benchmark command.
Candidate 23: `microservices/forms/faqs/forms-engineer-faq.md:40` uses demo_trial in anti-abuse policy.
Candidate 24: `microservices/forms/faqs/forms-engineer-faq.md:41` uses paid in anti-abuse policy.
Candidate 25: `microservices/forms/faqs/forms-engineer-faq.md:63` uses paid and paid in warehouse-export SLOs.
Candidate 26: `microservices/forms/faqs/forms-engineer-faq.md:92` uses paid in upload limits.
Candidate 27: `microservices/forms/faqs/forms-engineer-faq.md:93` uses paid in upload limits.
Candidate 28: `microservices/forms/faqs/forms-engineer-faq.md:94` uses paid compliance_pack in pack-specific upload limits.
Candidate 29: `microservices/forms/faqs/forms-engineer-faq.md:100` uses paid in lead-scoring scope.
Candidate 30: `microservices/forms/faqs/forms-engineer-faq.md:111` uses paid compliance_pack in pack-bound forms.
Candidate 31: `microservices/forms/faqs/forms-engineer-faq.md:119` uses paid compliance_pack in DSAR and consent-withdrawal scope.
Candidate 32: `microservices/forms/migration-playbooks/from-google-forms-and-typeform.md:111` uses paid in calculator migration.
Candidate 33: `microservices/forms/migration-playbooks/from-google-forms-and-typeform.md:132` uses paid in quiz migration.
Candidate 34: `microservices/forms/migration-playbooks/from-google-forms-and-typeform.md:248` uses paid compliance_pack in HIPAA migration.
Additional retirement-adjacent vocabulary exists outside the direct demo_trial/paid/paid/paid compliance_pack scan: PRD, manifest, OpenAPI, AsyncAPI, protobuf, ADRs, cost-budget, DPIA, and policy files carry T0/T1/T2, Tier-G, tenant-tier, or generic `tier` semantics (`microservices/forms/PRD.md:42-68`, `microservices/forms/manifest.json:46-65`, `microservices/forms/contracts/proto/forms.proto:74-80`).
Those adjacent terms should be handled by the same Wave 15J rewrite because the replacement model is tenant_class plus uniform quality, not tenant_class model.

#### 3.4.C Tenant-Class Adoption Gaps

Tenant-class rule for this batch: `demo_trial`, `paid`, and `revenue_share` are the replacement commercialization classes.
Search result: no `tenant_class` term appears under `microservices/forms/`.
Search result: no `demo_trial` term appears under `microservices/forms/`.
Search result: no `revenue_share` term appears under `microservices/forms/`.
Search result: `paid` appears only as ordinary prose, not as the canonical tenant class.
The current docs express tier semantics through tenant_class model, tenant-tier tables, and lettered tier gates (`microservices/forms/decisions/ADR-FORMS-0006-e-signature-conformance.md:60-73`, `microservices/forms/manifest.json:311-315`).
Gap: forms has no canonical tenant_class field in manifest, contracts, policy, pricing, cost budget, capacity model, or deployment overlays.
Gap: forms has no demo/trial usage cap language tied to OCI Always Free profile outputs.
Gap: forms has no paid tenant class language that separates per-seat licensing from usage-based scaling.
Gap: forms has no revenue_share tenant class language for marketplace, B2C operator, embedded SaaS reseller, or affiliate partner deployments.
Finding severity: P2, because tenant-class adoption is a documentation and contract model gap rather than a live code runtime failure.

### 3.5 Dimension 5 - Cross-Microservice Boundaries and Dependency Coherence

Assessment: partial pass with boundary drift.
The PRD lists dependencies and boundaries with sheets, drive, mail, messenger, workflow-engine, analytics, identity, tenancy, storage, AI, catalog, payments, signatures, and public-content surfaces (`microservices/forms/PRD.md:156-173`).
The manifest lists dependencies on workflow-engine, audit-chain, sheets, governance, cell, identity, tenancy, observability, network, intelligence, ontology, detection, and cloud-iac (`microservices/forms/manifest.json:366-379`).
The architecture document lists a different dependency set: identity, tenancy, policy-engine, observability, audit-chain, and cloud-secrets (`microservices/forms/ARCHITECTURE.md:197-208`).
The PRD references foundry-runtime and foundry-providers for AI form build (`microservices/forms/PRD.md:164`, `microservices/forms/PRD.md:173`).
The manifest dependency list does not include foundry-runtime or foundry-providers (`microservices/forms/manifest.json:366-379`).
The PRD and ADR-FORMS-0006 reference fintech/payment behavior (`microservices/forms/PRD.md:53`, `microservices/forms/decisions/ADR-FORMS-0006-e-signature-conformance.md:167`).
The manifest dependency list does not include fintech (`microservices/forms/manifest.json:366-379`).
The architecture document's dependency set does not include sheets, drive, mail, messenger, foundry, fintech, or social-public distribution surfaces despite PRD scope (`microservices/forms/ARCHITECTURE.md:197-208`, `microservices/forms/PRD.md:156-173`).
The service boundary is conceptually plausible, but the dependency registry is inconsistent across PRD, manifest, and architecture.
Finding severity: P2 because the issue is documentation and registry drift with integration consequences.

### 3.6 Dimension 6 - Compliance, Privacy, Security, and Abuse Posture

Assessment: partial pass.
Forms has compliance and DPIA artifacts, and the PRD contains privacy/compliance requirements for consent, PII classification, DSAR, audit-chain, and retention (`microservices/forms/PRD.md:69-101`, `microservices/forms/dpia.md:65-97`).
Forms includes Cedar policy files for auditor scope, CI scope, public read, and tenant scope.
The DPIA identifies a tier mismatch risk, which is now also a Wave 15J modeling risk (`microservices/forms/dpia.md:75`, `microservices/forms/dpia.md:95`).
The compliance document contains many `tier product` metadata references and lettered tier gates; those need a follow-up model rewrite after tier retirement (`microservices/forms/compliance.md:96`, `microservices/forms/compliance.md:106`).
The PRD requires file uploads, malware scanning, captcha, public embeds, signed forms, payment bridge, DSAR, and AI-assisted build, each of which is security-sensitive (`microservices/forms/PRD.md:50-67`, `microservices/forms/PRD.md:85-101`).
The runbook set covers captcha degradation, iframe CSP incidents, export pipeline failure, PII leak, response-store corruption, and spam flood.
The PRD references `policy/embed-csp.md`, but that policy file is absent, leaving embed security partly ungrounded (`microservices/forms/PRD.md:50`).
The OpenAPI and AsyncAPI contracts include AI build operations with tiered capability vocabulary, but no tenant_class authorization model (`microservices/forms/contracts/openapi/forms.openapi.yaml:263-285`, `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:200-207`).
Security posture is directionally strong but not canonically complete.
Finding severity: P2 because missing policy artifacts and tiered authorization language affect security design clarity.

### 3.7 Dimension 7 - Operational Readiness, SLOs, Runbooks, and Failure Modes

Assessment: partial pass.
OpenSLO artifacts exist for accessibility, AI build latency, analytics rendering, bulk distribution, CSV export, field validation, form rendering, PII encryption correctness, and submission latency.
The PRD states measurable performance targets: render, validation, submission, analytics, bulk distribution, export, AI build, and upload scanning (`microservices/forms/PRD.md:103-115`).
The runbook set is broad enough to cover multiple operational hazards.
The FAQ references a warehouse-export lag runbook that does not exist (`microservices/forms/faqs/forms-engineer-faq.md:75`).
The architecture document claims SLO/dashboard/runbook/IaC evidence surfaces are present, but canonical context IaC evidence is not present (`microservices/forms/ARCHITECTURE.md:68-69`).
The benchmark document claims achieved numbers on a retired tiered hardware profile, which is no longer a valid reporting shape for this batch (`microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:13-33`).
The benchmark document gives useful dimensions but not canonical tenant_class or deployment-context overlays.
Incident response is present, but the missing OpenTofu context modules mean operational readiness cannot be validated for all six deployment contexts.
Finding severity: P2 for runbook reference breakage and benchmark/reporting model drift; P1 for deployment-context readiness.

### 3.8 Dimension 8 - OS Support Matrix

Assessment: fail until remediated.
Canonical direction requires the OS support matrix across Linux, Windows, and Apple platforms (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3948-3999`, `specs/master-plan-sequencing.json:777-815`).
The OS memory requires per-microservice OS manifests and treats missing OS coverage as a P1 gap (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:56-76`).
No `supported-oses.json` exists under `microservices/forms/`.
The service has no evidence of Linux distribution coverage, Windows Server/Desktop coverage, Apple platform coverage, architecture coverage, or explicit exclusions.
Forms includes Leptos/WASM web planning (`microservices/forms/IP-010-form-builder-leptos-wasm.md`) and Rust SDK reference material (`microservices/forms/reference-implementations/submit-form-and-export-rust-sdk.md`), but those do not substitute for an OS manifest.
The absence is especially important because forms includes public rendering, file upload scanning, ClamAV, Valkey, Postgres, Meilisearch, WAF, CDN, and export workers, all of which may have OS-specific packaging and operations constraints.
Finding severity: P1.

### 3.9 Dimension 9 - Rust-Strict Implementation and Language Policy

Assessment: partial pass.
The forbidden-language extension scan found no `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, or `.fs` files under `microservices/forms/`.
The file tree uses authorized document/config/contract/policy forms such as Markdown, YAML, JSON, OpenAPI, AsyncAPI, proto, Cedar, Helm, Kustomize, and OpenSLO.
The PRD and IPs point toward Rust backend and Leptos/WASM frontend surfaces, which fits the canonical language direction (`microservices/forms/IP-010-form-builder-leptos-wasm.md`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:38-49`).
The protobuf package options for Go and Java are not implementation files, but they need an explicit generated-client/provenance note to avoid accidental backend-language drift (`microservices/forms/contracts/proto/forms.proto:16-18`).
There is no `src/` directory under `microservices/forms/`, so this audit cannot prove Rust implementation completeness from service-local source files.
The canonical build command expectation is Rust-based and should be expressed in implementation plans and future code gates (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:64`).
Finding severity: P3 for generated-client provenance note; no forbidden language violation found.

## 4. Findings Table

| ID | Severity | Finding | Evidence | Required follow-up |
| --- | --- | --- | --- | --- |
| FORMS-COH-001 | P1 | Missing canonical six-context OpenTofu module layout. | `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-1736`, `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2275-2294`; inventory shows only Helm, Kustomize, Terraform. | Create context-scoped OpenTofu modules or justified N/A records for every context. |
| FORMS-COH-002 | P1 | Forbidden Terraform surface is active under service IaC. | `microservices/forms/iac/terraform/cdn-edge-config.tf:5-6`, `microservices/forms/iac/terraform/template-marketplace-publishers.tf:1-31`; OpenTofu-only rule at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md:10-17`. | Retire Terraform files or migrate their intent into OpenTofu modules. |
| FORMS-COH-003 | P1 | OCI Always Free profile is missing. | Required at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3679-3700`; absent `microservices/forms/iac/oci-guest/always-free/`. | Add demo_trial OCI Always Free profile with budget outputs. |
| FORMS-COH-004 | P1 | OS support manifest is absent. | Required at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3948-3999`; absent `microservices/forms/supported-oses.json`. | Add service-level OS support manifest. |
| FORMS-COH-005 | P1 | Primary architecture doc contains scaffold/unknown state. | `microservices/forms/ARCHITECTURE.md:3`, `microservices/forms/ARCHITECTURE.md:35`, `microservices/forms/ARCHITECTURE.md:133-149`. | Replace scaffold residues with concrete state, event, table, tenant, and deployment bindings. |
| FORMS-COH-006 | P2 | Tier retirement candidates remain throughout the service. | Direct candidates listed in Section 3.4.T; tier retirement rule at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_2026_05_20.md:10-24`. | Rewrite to tenant_class plus uniform quality model during Wave 15J. |
| FORMS-COH-007 | P2 | Tenant-class model is not adopted. | No `tenant_class`, `demo_trial`, or `revenue_share` under forms; replacement direction from user prompt and `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:101-112`. | Add tenant_class semantics to manifest, contracts, policy, capacity, cost, and deployment overlays. |
| FORMS-COH-008 | P2 | Top-level README is missing. | Batch required PRD/ARCHITECTURE/README; `microservices/forms/PRD.md` and `ARCHITECTURE.md` exist, `README.md` does not. | Add README after canonical surface decisions are settled. |
| FORMS-COH-009 | P2 | Referenced microservice spec is absent. | `microservices/forms/PRD.md:218` references `/specs/microservices/forms.json`; `specs/microservices/forms.json` absent. | Add or correct the spec pointer. |
| FORMS-COH-010 | P2 | Referenced embed CSP policy is absent. | `microservices/forms/PRD.md:50`; absent `microservices/forms/policy/embed-csp.md`. | Add embed CSP policy or correct the reference. |
| FORMS-COH-011 | P2 | Referenced warehouse-export lag runbook is absent. | `microservices/forms/faqs/forms-engineer-faq.md:75`; absent `microservices/forms/runbooks/warehouse-export-lag.md`. | Add runbook or align FAQ to existing export pipeline runbook. |
| FORMS-COH-012 | P2 | Referenced AI Act conformity file is absent. | `microservices/forms/PRD.md:201`; absent `microservices/forms/legal/ai-act-conformity.md`. | Add legal artifact or move reference to canonical compliance source. |
| FORMS-COH-013 | P2 | Contract version prose drift exists. | `microservices/forms/PRD.md:136`, `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:1`, `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:6`. | Align PRD and AsyncAPI description with declared contract version. |
| FORMS-COH-014 | P2 | Dependency registry differs across PRD, manifest, and architecture. | `microservices/forms/PRD.md:156-173`, `microservices/forms/manifest.json:366-379`, `microservices/forms/ARCHITECTURE.md:197-208`. | Reconcile dependency authority and handoff ownership. |
| FORMS-COH-015 | P2 | Ontology projections are claimed in PRD but empty in manifest. | `microservices/forms/PRD.md:187-193`, `microservices/forms/manifest.json:303`. | Add projections or remove unsupported PRD claim. |
| FORMS-COH-016 | P2 | Accepted PRD still contains unresolved open questions. | `microservices/forms/PRD.md:203-214`. | Resolve or downgrade open questions before implementation handoff. |
| FORMS-COH-017 | P2 | Benchmark artifact uses retired tiered reporting. | `microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:13-33`, `microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:91-111`. | Replace with single industry-leader target plus context and tenant_class overlays. |
| FORMS-COH-018 | P2 | API and event contracts carry tenant-class field names. | `microservices/forms/contracts/openapi/forms.openapi.yaml:263-285`, `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:200-207`, `microservices/forms/contracts/proto/forms.proto:74-80`. | Migrate semantics to tenant_class and policy entitlements without feature-quality tiers. |
| FORMS-COH-019 | P3 | Valkey prose/pin mismatch exists in PRD. | `microservices/forms/PRD.md:145`, `microservices/forms/PRD.md:150`. | Align dependency version prose with actual pin. |
| FORMS-COH-020 | P3 | Proto Go/Java options need generated-client provenance note. | `microservices/forms/contracts/proto/forms.proto:16-18`; Rust-strict rule at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:51-60`. | Document these as generated-client metadata only. |
| FORMS-COH-021 | P3 | Local parity artifact includes more counterparts than this batch target. | `microservices/forms/competitor-parity-matrix.md:15-30`; batch target is Google Forms, Typeform, SurveyMonkey. | Keep broad market matrix if useful, but tag batch-specific matrices clearly. |

### 4.1 Required-Surface Evidence Register

ER-001. Canonical source read: ADR-0328 D-15 makes six-context explicitness mandatory, so missing context directories are not optional (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-1736`).
ER-002. Canonical source read: ADR-0328 D-15 lists the public-cloud context path as `iac/oyatie-public-cloud/`; forms lacks that path (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1749-1750`).
ER-003. Canonical source read: ADR-0328 D-15 lists the AWS guest context path as `iac/guest-on-aws/`; forms lacks that path (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1796-1797`).
ER-004. Canonical source read: ADR-0328 D-15 lists the OCI guest context path as `iac/oci-guest/`; forms lacks that path (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1846-1848`).
ER-005. Canonical source read: ADR-0328 D-15 lists the on-prem context path as `iac/on-prem/`; forms lacks that path (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1894-1895`).
ER-006. Canonical source read: ADR-0328 D-15 lists the colo context path as `iac/colo/`; forms lacks that path (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1944-1945`).
ER-007. Canonical source read: ADR-0328 D-15 lists the Oyatie provider context path as `iac/oyatie-iaas/`; forms lacks that path (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1993-1994`).
ER-008. Canonical source read: unsupported context claims require reason, missing primitives, customer impact, remediation owner, and revisit gate; forms has no such N/A matrix (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2079-2087`).
ER-009. Canonical source read: `cloud-iac` plus OpenTofu must exist for every supported context; forms has no context OpenTofu evidence (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2095-2099`).
ER-010. Canonical source read: Phase 0/1/2 services default to all six contexts; forms audit found no reasoned exception (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2116-2125`).
ER-011. Canonical source read: ADR-0328 D-16 says OpenTofu, not Terraform; forms contains active Terraform files (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2241-2249`).
ER-012. Canonical source read: ADR-0328 D-16 requires per-context module files including `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and `README.md`; forms has none under canonical context dirs (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2296-2308`).
ER-013. Canonical source read: ADR-0328 D-16 forbids Terraform binary, Terraform Cloud, Pulumi, CloudFormation, ARM/Bicep, shell durable infra, and manual console paths (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2464-2498`).
ER-014. Canonical source read: OCI Always Free profile has 4 OCPU and 24 GB memory total; forms has no profile to translate that cap into demo_trial operations (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3514-3517`).
ER-015. Canonical source read: OCI Always Free service module must include tenant_class and budget outputs; forms has no such module (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3679-3700`).
ER-016. Canonical source read: OCI Always Free audit stop condition includes `iac/oci-guest/always-free/`; forms lacks it (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3808-3823`).
ER-017. Canonical source read: OS support dimension requires a manifest and coverage matrix; forms lacks `supported-oses.json` (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3948-3999`).
ER-018. Canonical source read: Rust-only language dimension allows config, contracts, policies, docs, and approved frontend surfaces; forms file extensions fit that allowance (`docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4011-4080`).
ER-019. Master-plan source read: six deployment contexts are represented in `specs/master-plan-sequencing.json`; forms lacks service-local realization (`specs/master-plan-sequencing.json:704-745`).
ER-020. Master-plan source read: OpenTofu engine and forbidden engines are represented in `specs/master-plan-sequencing.json`; forms violates this with Terraform files (`specs/master-plan-sequencing.json:747-775`).
ER-021. Master-plan source read: service-level OS manifest is required in `specs/master-plan-sequencing.json`; forms lacks it (`specs/master-plan-sequencing.json:777-815`).
ER-022. Master-plan source read: Rust backend and frontend allowlist are represented in `specs/master-plan-sequencing.json`; forms has no forbidden app-language files (`specs/master-plan-sequencing.json:817-855`).
ER-023. Master-plan source read: OCI Always Free profile path is represented in `specs/master-plan-sequencing.json`; forms lacks it (`specs/master-plan-sequencing.json:857-867`).
ER-024. Brief-template source read: multi-context anchor requires context declarations and `tofu` tenant onboarding (`docs/standards/brief-template.md:666-700`).
ER-025. Brief-template source read: OpenTofu anchor requires module paths and forbids other engines (`docs/standards/brief-template.md:809-850`).
ER-026. Brief-template source read: OS support anchor requires supported-oses manifest and architecture matrix (`docs/standards/brief-template.md:967-1004`).
ER-027. Brief-template source read: language-policy anchor requires Rust backend and approved frontend allowlist (`docs/standards/brief-template.md:1125-1163`).
ER-028. Brief-template source read: substance-bar test requires concrete docs, not padding (`docs/standards/brief-template.md:1431-1471`).
ER-029. Brief-template source read: line count as completion is an anti-pattern (`docs/standards/brief-template.md:1727-1751`).
ER-030. Memory source read: multi-context absence in PRD/ADR/IP is a P1 gap, matching this audit's IaC and context findings (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_multi_context_provider_agnostic_2026_05_20.md:32-38`).
ER-031. Memory source read: zero-handroll OpenTofu-only doctrine requires per-service context modules (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md:20-35`).
ER-032. Memory source read: OS-support doctrine requires a service-level OS manifest and treats missing coverage as P1 (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:56-76`).
ER-033. Memory source read: Rust-strict doctrine defines the forbidden language extension scan used in this audit (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:51-60`).
ER-034. Memory source read: OCI Always Free maximization requires per-service module and capacity tests (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md:65-82`).
ER-035. Memory source read: no-tenant-class directive establishes tier retirement as current doctrine (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_2026_05_20.md:10-24`).
ER-036. Memory source read: tenant-class replacement work is explicitly tied to Wave 15J and batch 3.2 dropping tier delta output (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:90-99`, `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:139-142`).
ER-037. Memory source read: microservice ownership directive requires PRD, architecture, ADRs, IPs, contracts, SLOs, tiers, handoffs, capacity, failure, incident, cost, DPIA, compliance, and guides to be inspected (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:18-45`).
ER-038. Memory source read: chat history search is required for this audit (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:47`).
ER-039. Memory source read: deliverables must stay inside the service path (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:75-83`).
ER-040. Chat history read: user directed audit-only cataloging of contradictions and gaps, with remediation deferred (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:14043`).
ER-041. Chat history read: assistant summarized a five-dimension audit including industry-counterpart parity, matching current batch framing (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:14049`).
ER-042. Chat history read: older Wave-4 forms gapfill used a broader competitor set and included tier artifacts now retired (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:10950`).
ER-043. Chat history read: older completion note said forms artifacts were authored with tier-bound surfaces, explaining local tier residue (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:11169`).
ER-044. Chat history read: rolling audit queue names forms with Google Forms, Typeform, and SurveyMonkey, matching the assigned counterpart set (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290`).
ER-045. Chat history read: later queue evidence repeats forms with the same top-three counterparts (`/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16311`).
ER-046. PRD read: product purpose is typed form definitions, response capture, and survey distribution (`microservices/forms/PRD.md:30-38`).
ER-047. PRD read: functional requirements cover builder, logic, validation, file upload, signatures, payments, analytics, export, distribution, and AI build (`microservices/forms/PRD.md:44-68`).
ER-048. PRD read: acceptance criteria include public renderer, uploads, signatures, payments, AI build, privacy, and audit-chain controls (`microservices/forms/PRD.md:72-101`).
ER-049. PRD read: performance target numbers are concrete enough to seed the performance deliverable (`microservices/forms/PRD.md:103-115`).
ER-050. PRD read: protocols say AsyncAPI 3.0 while the actual AsyncAPI declares 3.1.0 (`microservices/forms/PRD.md:136`, `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:1`).
ER-051. PRD read: Valkey prose and pin mismatch create a minor version-coherence gap (`microservices/forms/PRD.md:145`, `microservices/forms/PRD.md:150`).
ER-052. PRD read: cross-service boundaries mention foundry and fintech surfaces missing from manifest dependencies (`microservices/forms/PRD.md:164`, `microservices/forms/PRD.md:173`, `microservices/forms/PRD.md:53`).
ER-053. PRD read: workflow events still include tier metadata (`microservices/forms/PRD.md:175-185`).
ER-054. PRD read: ontology writes are claimed but not mirrored by manifest projections (`microservices/forms/PRD.md:187-193`, `microservices/forms/manifest.json:303`).
ER-055. PRD read: open questions remain in an accepted service artifact (`microservices/forms/PRD.md:203-214`).
ER-056. Architecture read: document self-identifies as anchor-sweep output needing content-pass expansion (`microservices/forms/ARCHITECTURE.md:3`).
ER-057. Architecture read: bounded context and state/event binding are unresolved in primary architecture (`microservices/forms/ARCHITECTURE.md:22-35`).
ER-058. Architecture read: architecture claims evidence surfaces are present, but canonical IaC evidence is not present (`microservices/forms/ARCHITECTURE.md:68-69`).
ER-059. Architecture read: tenant-scoping section includes placeholder names and `forms.unknown` (`microservices/forms/ARCHITECTURE.md:133-149`).
ER-060. Architecture read: declared dependency set differs from PRD and manifest (`microservices/forms/ARCHITECTURE.md:197-208`).
ER-061. Manifest read: capabilities use T0/T1/T2 vocabulary (`microservices/forms/manifest.json:46-65`).
ER-062. Manifest read: SLO path registry is present and useful (`microservices/forms/manifest.json:66-120`).
ER-063. Manifest read: IP registry is present and broad (`microservices/forms/manifest.json:122-212`).
ER-064. Manifest read: LTS pins exist, which is a positive dependency-control signal (`microservices/forms/manifest.json:227-234`).
ER-065. Manifest read: `tenant_class` explicitly records T0/T1/T2 (`microservices/forms/manifest.json:311-315`).
ER-066. Manifest read: `criticality_tier` also keeps tier vocabulary (`microservices/forms/manifest.json:382`).
ER-067. Contract read: OpenAPI declares 3.2.0 and is present as a structured contract (`microservices/forms/contracts/openapi/forms.openapi.yaml:1`).
ER-068. Contract read: OpenAPI AI build requires prompt and tier, requiring Wave 15J API modeling (`microservices/forms/contracts/openapi/forms.openapi.yaml:263-285`).
ER-069. Contract read: OpenAPI schema carries `tier` field (`microservices/forms/contracts/openapi/forms.openapi.yaml:541-545`).
ER-070. Contract read: AsyncAPI declares 3.1.0 but description says 3.0 (`microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:1`, `microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:6`).
ER-071. Contract read: AsyncAPI AI build payload carries tier (`microservices/forms/contracts/asyncapi/forms.asyncapi.yaml:200-207`).
ER-072. Contract read: protobuf defines `CapabilityTier` and uses it in AI build request (`microservices/forms/contracts/proto/forms.proto:74-80`, `microservices/forms/contracts/proto/forms.proto:193-197`).
ER-073. Contract read: protobuf Go/Java options are metadata requiring provenance note (`microservices/forms/contracts/proto/forms.proto:16-18`).
ER-074. ADR read: ADR-FORMS-0005 title and body explicitly model tenant_class model (`microservices/forms/decisions/ADR-FORMS-0005-ai-form-build-bounds.md:3`, `microservices/forms/decisions/ADR-FORMS-0005-ai-form-build-bounds.md:53-57`).
ER-075. ADR read: ADR-FORMS-0005 downstream event records tier metadata (`microservices/forms/decisions/ADR-FORMS-0005-ai-form-build-bounds.md:188-189`).
ER-076. ADR read: ADR-FORMS-0006 title and body model tenant-tier mapping (`microservices/forms/decisions/ADR-FORMS-0006-e-signature-conformance.md:3`, `microservices/forms/decisions/ADR-FORMS-0006-e-signature-conformance.md:60-73`).
ER-077. ADR read: ADR-FORMS-0006 references fintech/payment integration, increasing manifest dependency pressure (`microservices/forms/decisions/ADR-FORMS-0006-e-signature-conformance.md:167`).
ER-078. ADR index read: decisions README keeps tier and tenant-tier vocabulary (`microservices/forms/decisions/README.md:24-25`).
ER-079. Benchmark read: local benchmark uses retired hardware profile terminology (`microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:13`).
ER-080. Benchmark read: local benchmark has useful dimensions but its target framing is retired (`microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:21-33`).
ER-081. Benchmark read: local benchmark TCO lines use retired terms (`microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:91-92`).
ER-082. Benchmark read: local benchmark command includes `--tier oyatie-paid` (`microservices/forms/benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:111`).
ER-083. FAQ read: anti-abuse posture is tiered and needs rewrite (`microservices/forms/faqs/forms-engineer-faq.md:40-41`).
ER-084. FAQ read: export SLO answer is tiered and needs rewrite (`microservices/forms/faqs/forms-engineer-faq.md:63`).
ER-085. FAQ read: upload limits are tiered and need tenant_class usage caps (`microservices/forms/faqs/forms-engineer-faq.md:92-94`).
ER-086. FAQ read: lead-scoring and pack-bound answers use retired terms (`microservices/forms/faqs/forms-engineer-faq.md:100`, `microservices/forms/faqs/forms-engineer-faq.md:111`).
ER-087. Migration read: calculator and quiz mappings use retired terms (`microservices/forms/migration-playbooks/from-google-forms-and-typeform.md:111`, `microservices/forms/migration-playbooks/from-google-forms-and-typeform.md:132`).
ER-088. Migration read: HIPAA migration uses retired terms (`microservices/forms/migration-playbooks/from-google-forms-and-typeform.md:248`).
ER-089. Tutorial read: prerequisite line uses retired terms (`microservices/forms/tutorials/build-multi-page-survey-with-logic-jump-payment-warehouse.md:15`).
ER-090. Cost read: cost model contains tier-2 pricing and Tier-G upsell language (`microservices/forms/cost-budget.md:47`, `microservices/forms/cost-budget.md:59`).
ER-091. Cost read: cost tags include `tenant_class`, requiring model replacement (`microservices/forms/cost-budget.md:68`).
ER-092. DPIA read: risk register names tier mismatch (`microservices/forms/dpia.md:75`).
ER-093. DPIA read: mitigation table names tier validation (`microservices/forms/dpia.md:95`).
ER-094. Compliance read: e-signature/payment conformance uses lettered tier gates (`microservices/forms/compliance.md:96`, `microservices/forms/compliance.md:106`).
ER-095. Policy read: tenant scope uses `tenant_class-gated` language (`microservices/forms/policy/tenant-scope.cedar:149`).
ER-096. Policy read: tenant scope uses `production-tier` language (`microservices/forms/policy/tenant-scope.cedar:305`).
ER-097. Policy read: dual-context policy carries `tier=T2` (`microservices/forms/policy/dual-context.md:118`).
ER-098. IaC read: Terraform CDN edge config hardcodes active Terraform syntax (`microservices/forms/iac/terraform/cdn-edge-config.tf:5-6`).
ER-099. IaC read: Terraform CDN edge config includes OCI provider resource shape outside canonical OpenTofu context module (`microservices/forms/iac/terraform/cdn-edge-config.tf:8-10`, `microservices/forms/iac/terraform/cdn-edge-config.tf:39-56`).
ER-100. IaC read: marketplace publisher Terraform file belongs to the same forbidden IaC family (`microservices/forms/iac/terraform/template-marketplace-publishers.tf:1-31`).
ER-101. IaC read: Helm and Kustomize files exist, but they do not replace required OpenTofu context modules.
ER-102. SLO read: OpenSLO files cover core latency and correctness dimensions; this is a positive evidence surface.
ER-103. Runbook read: seven runbooks exist and cover AI rollback, captcha, embed CSP incident, export failure, PII leak, corruption, and spam flood.
ER-104. Broken-reference check: FAQ points to warehouse-export lag runbook that is absent (`microservices/forms/faqs/forms-engineer-faq.md:75`).
ER-105. Broken-reference check: PRD points to embed CSP policy that is absent (`microservices/forms/PRD.md:50`).
ER-106. Broken-reference check: PRD points to AI Act conformity file that is absent (`microservices/forms/PRD.md:201`).
ER-107. Broken-reference check: PRD points to service spec JSON that is absent (`microservices/forms/PRD.md:218`).
ER-108. Rust scan result: no forbidden app-language files were found under forms.
ER-109. Tenant-class scan result: no canonical tenant_class terms were found under forms.
ER-110. Inventory result: all 145 files were listed and categorized in Section 2.1.
ER-111. Product comparison result: local product purpose maps to forms/survey/intake category and is not a category mismatch.
ER-112. Counterpart result: Google Forms covers low-friction creation, sharing, quiz, charts, and Sheets export.
ER-113. Counterpart result: Typeform covers engagement, logic, payments, file uploads, API, webhooks, and response limits.
ER-114. Counterpart result: SurveyMonkey covers templates, question bank, advanced logic, quotas, audience, and enterprise response governance.
ER-115. Union result: Oyatie forms is ahead in governed deployment ambition but behind in canonical deployment evidence.
ER-116. Union result: Oyatie forms is ahead in per-question policy/data-class ambition but behind in registry coherence.
ER-117. Union result: Oyatie forms lacks local Question Bank and audience-panel evidence against SurveyMonkey.
ER-118. Union result: Oyatie forms lacks partial-response and score-release detail against Typeform and Google Forms.
ER-119. Substrate result: service-local docs cannot claim all six deployable contexts today.
ER-120. Substrate result: service-local docs cannot claim OCI Always Free demo_trial readiness today.
ER-121. Substrate result: service-local docs cannot claim OS matrix support today.
ER-122. Substrate result: service-local docs cannot claim OpenTofu-only IaC today.
ER-123. Model result: service-local docs cannot claim post-tier tenant_class adoption today.
ER-124. Severity result: no P0 issue was found because this is a documentation/coherence audit, not evidence of live production data loss.
ER-125. Severity result: five P1 findings block canonical ownership coherence.
ER-126. Severity result: thirteen P2 findings identify model, artifact, dependency, and contract gaps.
ER-127. Severity result: three P3 findings identify smaller consistency/provenance issues.
ER-128. Completion result: the audit produced exactly three deliverables and no tier-deltas file.
ER-129. Scope result: no other microservice path was modified.
ER-130. Git result: no commit is made per execution rule.

## 5. Open Questions

1. Which file becomes authoritative for the forms dependency registry: PRD, manifest, architecture, or a generated service registry?
2. Should the missing `/specs/microservices/forms.json` be created as a machine-readable mirror of `manifest.json`, or should the PRD pointer be retired?
3. Should `foundry-runtime` and `foundry-providers` be direct forms dependencies, or should AI form build route through a higher-level intelligence service?
4. Should `fintech` be a direct forms dependency for payment fields, or should payment fields emit workflow events handled by a payment-owned service?
5. What is the canonical replacement for T0/T1/T2 in forms contracts: tenant_class, policy entitlement, capability flag, or all three with separate semantics?
6. Does `demo_trial` allow payment/signature/file-upload fields with capped usage, or are those enabled uniformly but constrained by usage and risk controls?
7. What exact demo_trial usage caps should forms enforce under the OCI Always Free profile: submissions per month, file bytes per tenant, AI build count, export bytes, and concurrent responders?
8. What paid tenant-class scaling model should forms publish: per-seat admin seats, per-response usage, per-file storage, per-AI-build usage, or mixed meters?
9. What revenue_share forms model is expected for marketplace sellers, B2C operators, embedded SaaS resellers, and affiliate partners?
10. Should the retired `tenant-class/tier-matrix.md` be deleted during Wave 15J or rewritten as a tenant_class and entitlement matrix?
11. How should the benchmark claims be revalidated after removing the retired hardware profile terms?
12. Which SLO artifacts are contractual versus internal error-budget objectives?
13. Where should embed CSP policy live if not `microservices/forms/policy/embed-csp.md`?
14. Should the missing warehouse-export lag runbook be a new runbook or a subsection of `runbooks/export-pipeline-failure.md`?
15. Does forms need a service-local legal directory for AI Act conformance, or should the compliance document link to shared legal authority?
16. Should architecture use concrete table/event names from the domain IPs, or should those be generated from code once Rust implementation lands?
17. Should the protobuf Go and Java options be retained for external SDK generation after the Stainless SDK directive, or moved behind a generated-client policy?
18. What formal evidence proves forms can run in each of the six deployment contexts after OpenTofu modules are added?
19. Which customer journeys depend on forms as a blocking intake service, and do those journeys require higher priority for the forms deployment-context remediation?
20. Is the forms service allowed to keep broad competitor research beyond Google Forms, Typeform, and SurveyMonkey as long as batch-specific deliverables use the assigned top three?

## 6. Audit Closeout State

No P0 finding was identified in the forms path during this audit.
The blocking coherence issues are P1: canonical deployment context modules, OpenTofu-only IaC, OCI Always Free profile, supported OS manifest, and primary architecture scaffold residue.
The broadest P2 issue is Wave 15J tier retirement: 34 direct demo_trial/paid/paid/paid compliance_pack references plus many adjacent T0/T1/T2 and tenant-tier fields remain in service artifacts.
The tenant_class model is absent and must be added before forms can express the post-tier commercial model.
The Rust-strict forbidden-language scan passed at file-extension level, but this does not prove Rust implementation completeness.
The counterpart product target is confirmed as Google Forms, Typeform, and SurveyMonkey for this batch.
The service has enough product substance to be worth remediating; the work is coherence alignment, not a blank restart.

### 6.1 Completion Criteria Check

CC-001. Required deliverable count is three, and this audit intentionally excludes the retired tier-deltas deliverable.
CC-002. Required target path is `microservices/forms/`, and all authored files are under that path.
CC-003. Required counterpart set is Google Forms, Typeform, and SurveyMonkey, and both parity/performance deliverables use that set.
CC-004. Required inventory was complete at 145 files, and Section 2.1 lists every path found.
CC-005. Required PRD review was completed, with findings tied to purpose, functional requirements, performance targets, cross-service dependencies, and open questions.
CC-006. Required architecture review was completed, with scaffold and unknown-binding findings cited.
CC-007. Required ADR review was completed, with tier and tenant-tier findings cited in ADR-FORMS-0005 and ADR-FORMS-0006.
CC-008. Required implementation-plan review was completed at inventory level, with no implementation file edits performed.
CC-009. Required contract review was completed for OpenAPI, AsyncAPI, and protobuf.
CC-010. Required SLO review was completed at surface level, with performance targets carried into Deliverable 3.
CC-011. Required tenant-class read was completed, and direct retired demo_trial/paid/paid/paid compliance_pack candidates were cataloged.
CC-012. Required capacity, failure, incident, cost, DPIA, and compliance surfaces were inspected for coherence signals.
CC-013. Required benchmarks, FAQs, onboarding, migration, reference implementation, tutorials, and runbooks were inspected.
CC-014. Required IaC review found Helm, Kustomize, and Terraform, but no canonical context OpenTofu modules.
CC-015. Required source-language scan found no forbidden app-language files under forms.
CC-016. Required chat-history search processed forms-specific messages and counterpart assignment evidence.
CC-017. Required product-purpose comparison found a strong forms/survey/intake fit against the assigned counterpart family.
CC-018. Required multi-context dimension was evaluated and failed for missing context modules.
CC-019. Required OpenTofu dimension was evaluated and failed because active Terraform files are present.
CC-020. Required OS support dimension was evaluated and failed because `supported-oses.json` is absent.
CC-021. Required Rust-strict dimension was evaluated and passed only at forbidden-extension scan level.
CC-022. Required OCI Always Free dimension was evaluated and failed because the profile module is absent.
CC-023. Required tier-retirement audit found 34 direct candidates plus adjacent tier vocabulary.
CC-024. Required tenant-class audit found no canonical tenant_class adoption.
CC-025. Required provenance style was applied with file:line, memory file, chat-history, and public-source citations.
CC-026. Required no-commit rule was followed.
CC-027. Required no-other-microservice rule was followed.
CC-028. Required no tier scaffolding in new content was followed; tier terms appear only as findings or source citations.
CC-029. Required halt-cleanly condition was not invoked because the deliverables could be completed.
CC-030. Remaining risk: line-count floors prove size only; substance is represented by cited findings, matrices, and target registers.

<!-- ORCHESTRATOR REPORT
  µservice: forms
  deliverables_landed:
    - microservices/forms/coherence-audit-2026-05-20.md (622 lines)
    - microservices/forms/feature-parity-matrix-2026-05-20.md (411 lines)
    - microservices/forms/performance-benchmark-numbers-2026-05-20.md (303 lines)
  inventory_files_seen: 145
  inventory_lines_read: 20257
  chat_history_matches_processed: 6
  findings_p0: 0
  findings_p1: 5
  findings_p2: 13
  findings_p3: 3
  tier_retirement_candidates_found: 34; citations: tenant-class/tier-matrix.md:13,52,54,62,92,94,126,130,132,145,147,160; benchmarks/forms-vs-google-forms-vs-typeform-vs-jotform-vs-surveymonkey.md:13,21,31,33,39,47,53,91,92,111; faqs/forms-engineer-faq.md:40,41,63,92,93,94,100,111,119; migration-playbooks/from-google-forms-and-typeform.md:111,132,248; tutorials/build-multi-page-survey-with-logic-jump-payment-warehouse.md:15
  tenant_class_adoption_gaps: yes; no tenant_class/demo_trial/revenue_share semantics found under microservices/forms/
  top_3_counterparts_confirmed: Google Forms / Typeform / SurveyMonkey
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1336
-->
