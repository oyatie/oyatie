---
audit_class: microservice_ownership_coherence
batch: wave-3-batch-3.2
microservice: drive
service_path: microservices/drive
audit_date: 2026-05-20
auditor: codex
counterparts: [Google Drive, Dropbox, Microsoft OneDrive]
deliverable_count: 3
retired_deliverable: capability-tier-deltas-vs-counterparts-2026-05-20.md
status: landed
---

# Drive µservice Ownership-Coherence Audit — 2026-05-20

## Header

Audit target: `microservices/drive/`.

Deployable-context presumption: all six canonical deployment contexts remain in scope unless evidence under this path proves a valid exception.

Counterpart bar: Google Drive, Dropbox, Microsoft OneDrive.

Audit doctrine: uniform industry-leader quality across `demo_trial`, `paid`, and `revenue_share` tenant classes.

Retired doctrine: capability-tier deltas are not authored for this batch.

Source anchor 1: ADR-0328 requires one audit owner per µservice and a file-read-before-write discipline in §D-4, then extends the audit to dimensions 6-9 in §D-20.

Source anchor 2: ADR-0328 §D-15 requires explicit evidence for `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.

Source anchor 3: `specs/master-plan-sequencing.json:704-742` names the six deployment contexts and maps them to context-specific IaC targets.

Source anchor 4: `specs/master-plan-sequencing.json:747-756` makes OpenTofu the canonical IaC substrate.

Source anchor 5: `specs/master-plan-sequencing.json:817-854` makes Rust the backend/runtime language with narrow frontend exceptions.

Source anchor 6: `docs/standards/brief-template.md:666-1125` defines the required brief sections for API, SLO, integration, and implementation substance.

Source anchor 7: `docs/standards/brief-template.md:1520-1722` defines µservice ownership and agent-class anchors.

Source anchor 8: `docs/standards/brief-template.md:1722-1782` rejects scaffold-only bodies, line-count padding, and scripted substantive docs.

Source anchor 9: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_automation_risk_classes_2026_05_20.md:10-24` records the directive that the old feature-tier model is retired.

Source anchor 10: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:139-142` records that Wave 3 Batch 3.2 drops the fourth tier-delta deliverable and uses a single performance target set.

Source anchor 11: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:10-14` assigns one owner to one µservice and forbids fragmented ownership.

Source anchor 12: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-31` requires scope, ADR, and hyperscaler-quality verification, not line-count self-report.

Source anchor 13: chat history `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290` queues `drive` with Google Drive, Dropbox, and Microsoft OneDrive.

Source anchor 14: chat history `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16386` records a prior rolling drive audit dispatch.

## §1 Purpose

1. This audit checks whether the `drive` µservice has a coherent product definition, coherent ownership boundaries, and enough deployability evidence to satisfy current Oyatie direction.
2. The actual product described in `microservices/drive/PRD.md:20-28` is a tenant-facing file and object storage product with folders, multipart upload, range download, FastCDC delta sync, share links, permissions, search, preview, DLP, virus scanning, encryption, WORM, quotas, and migration support.
3. The PRD names both personal and professional contexts as first-class surfaces at `microservices/drive/PRD.md:25-27`.
4. The µservice is not merely a cloud-object-store wrapper; it combines human drive workflows with compliance storage, cross-tenant sharing, sync, search, and evidence retention.
5. The counterpart set is confirmed locally by `microservices/drive/PRD.md:281-283`, which names Google Drive, Dropbox, and OneDrive as the first three competitors.
6. The same counterpart set is confirmed externally by chat queue evidence at `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290`.
7. The audit therefore evaluates product breadth against file storage, collaboration, sync, search, preview, compliance, API, migration, and operations surfaces.
8. The audit also evaluates canonical alignment against the five cross-cutting 2026-05-20 constraints: multi-context deployment, OpenTofu-only IaC, OS support, Rust-strict implementation policy, and OCI Always Free profile maximization.
9. The audit explicitly treats old feature-tier terms as retirement candidates, not as a current design model.
10. The audit does not author a capability-tier-deltas deliverable.
11. The stop condition is three landed reports under `microservices/drive/`, fresh line-count verification, a tier-reference verification, and a clear report block at the bottom of this file.

## §2 Inventory

### §2.1 Inventory Method

1. Complete tree inventory command: `find microservices/drive -type f | sort`.
2. Inventory file count: 175 files.
3. Tree line count command: `find microservices/drive -type f -print0 | xargs -0 wc -l`.
4. Tree line count observed before authoring: 40,982 lines.
5. Primary documents read or sampled substantially: `PRD.md`, `ARCHITECTURE.md`, `manifest.json`, all ADR filenames, all implementation-plan filenames, OpenAPI, AsyncAPI, proto, all OpenSLO files, capability-tenant_class adoption record, compliance, DPIA, cost, capacity, failure modes, incident response, multi-region, backfill, migration, onboarding, tutorial, benchmark, FAQ, runbooks, policy, and IaC.
6. Code-sample scan: no `src/` directory is present under `microservices/drive/`.
7. Test-code scan: no `tests/` directory is present under `microservices/drive/`; test intent exists under `test-plans/`.
8. Forbidden-language scan over `*.py`, `*.js`, `*.ts`, `*.rb`, `*.go`, and `*.java` returned no files under the drive path.
9. OpenTofu scan did not find context modules under the canonical six context directories.
10. IaC evidence currently exists as Helm and Kustomize, with concrete paths cited below.

### §2.2 Complete File Inventory

1. `microservices/drive/ARCHITECTURE.md`
2. `microservices/drive/AUDIT-FINDINGS-2026-05-18.json`
3. `microservices/drive/IP-001-iac-bootstrap.md`
4. `microservices/drive/IP-002-file-store-kernel.md`
5. `microservices/drive/IP-003-file-store-adapters.md`
6. `microservices/drive/IP-004-file-store-rest-worker-sdk-app.md`
7. `microservices/drive/IP-005-folder-hierarchy.md`
8. `microservices/drive/IP-006-upload.md`
9. `microservices/drive/IP-007-download.md`
10. `microservices/drive/IP-008-sync.md`
11. `microservices/drive/IP-009-share-link.md`
12. `microservices/drive/IP-010-permissions.md`
13. `microservices/drive/IP-011-search-index.md`
14. `microservices/drive/IP-012-preview.md`
15. `microservices/drive/IP-013-dlp-virus-scan.md`
16. `microservices/drive/IP-014-immutability-tier.md`
17. `microservices/drive/IP-015-hg-drive-registration.md`
18. `microservices/drive/IP-journey-j04-shelter-evidence-vault.md`
19. `microservices/drive/IP-journey-j06-source-document-vault.md`
20. `microservices/drive/IP-journey-j07-estate-data-export.md`
21. `microservices/drive/IP-journey-j100-pack-rollout-first-action.md`
22. `microservices/drive/IP-journey-j102-evidence-vault.md`
23. `microservices/drive/IP-journey-j105-evidence-vault.md`
24. `microservices/drive/IP-journey-j11-offline-file-journal.md`
25. `microservices/drive/IP-journey-j123-shared-asset-vault.md`
26. `microservices/drive/IP-journey-j125-deal-room-and-records-transfer.md`
27. `microservices/drive/IP-journey-j127-drive-transfer-of-ownership.md`
28. `microservices/drive/IP-journey-j133-work-drive-transfer-and-archival.md`
29. `microservices/drive/IP-journey-j136-plan-docs-and-aca-archive.md`
30. `microservices/drive/IP-journey-j140-internal-audit-dlp-egress-drive-protect.md`
31. `microservices/drive/IP-journey-j142-work-drive-classification-and-readonly.md`
32. `microservices/drive/IP-journey-j143-export-with-dlp-scrub-cross-tenant.md`
33. `microservices/drive/IP-journey-j17-encrypted-evidence-locker.md`
34. `microservices/drive/IP-journey-j26-photo-backup-album.md`
35. `microservices/drive/IP-journey-j34-channel-file-share.md`
36. `microservices/drive/IP-journey-j38-contract-record-archive.md`
37. `microservices/drive/IP-journey-j39-archive-folder.md`
38. `microservices/drive/IP-journey-j45-lab-result-vault.md`
39. `microservices/drive/IP-journey-j51-po-archival.md`
40. `microservices/drive/IP-journey-j52-label-and-receipt-archive.md`
41. `microservices/drive/IP-journey-j54-contract-archive.md`
42. `microservices/drive/IP-journey-j55-evidence-pack.md`
43. `microservices/drive/IP-journey-j57-starter-pack.md`
44. `microservices/drive/IP-journey-j59-ownership-transfer.md`
45. `microservices/drive/IP-journey-j61-imaging-archive.md`
46. `microservices/drive/IP-journey-j64-imaging-share.md`
47. `microservices/drive/IP-journey-j65-file-export.md`
48. `microservices/drive/IP-journey-j66-filing-archive.md`
49. `microservices/drive/IP-journey-j67-record-production.md`
50. `microservices/drive/IP-journey-j68-evidence-pack.md`
51. `microservices/drive/IP-journey-j70-contract-archive.md`
52. `microservices/drive/IP-journey-j74-plugin-file-actions.md`
53. `microservices/drive/IP-journey-j80-document-storage-boundary.md`
54. `microservices/drive/IP-journey-j85-document-storage-boundary.md`
55. `microservices/drive/IP-journey-j91-us-msb-mtl-overlay.md`
56. `microservices/drive/IP-journey-j92-br-lgpd-us-parent-dsar.md`
57. `microservices/drive/IP-journey-j93-in-dpdpa-rbi-overlay.md`
58. `microservices/drive/IP-journey-j94-sox404-public-company-controls.md`
59. `microservices/drive/IP-journey-j95-iso27001-soc2-annual-audit.md`
60. `microservices/drive/IP-journey-j96-ksa-uae-mena-onboarding.md`
61. `microservices/drive/IP-journey-j97-sg-pdpa-mas-tenant.md`
62. `microservices/drive/IP-journey-j98-au-privacy-apra-cps234.md`
63. `microservices/drive/IP-journey-j99-multi-pack-conflict-resolution.md`
64. `microservices/drive/PHASE-01-DRIVE-FOUNDATION.md`
65. `microservices/drive/PRD.md`
66. `microservices/drive/backfill-replay.md`
67. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md`
68. `microservices/drive/capabilities/T0-suggest.yaml`
69. `microservices/drive/capabilities/T1-assist.yaml`
70. `microservices/drive/capabilities/T2-auto.yaml`
71. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md`
72. `microservices/drive/capacity-model.md`
73. `microservices/drive/catalog/oya-drive-dlp-virus-scan-adapter-clamav.yaml`
74. `microservices/drive/catalog/oya-drive-dlp-virus-scan-adapter-opswat.yaml`
75. `microservices/drive/catalog/oya-drive-file-store-adapter-garage.yaml`
76. `microservices/drive/catalog/oya-drive-file-store-adapter-postgres.yaml`
77. `microservices/drive/catalog/oya-drive-file-store-adapter-s3.yaml`
78. `microservices/drive/catalog/oya-drive-file-store-adapter-seaweedfs.yaml`
79. `microservices/drive/catalog/oya-drive-file-store-app.yaml`
80. `microservices/drive/catalog/oya-drive-file-store-domain.yaml`
81. `microservices/drive/catalog/oya-drive-file-store-kernel.yaml`
82. `microservices/drive/catalog/oya-drive-file-store-rest.yaml`
83. `microservices/drive/catalog/oya-drive-file-store-sdk.yaml`
84. `microservices/drive/catalog/oya-drive-file-store-usecase.yaml`
85. `microservices/drive/catalog/oya-drive-file-store-worker.yaml`
86. `microservices/drive/catalog/oya-drive-folder-hierarchy-kernel.yaml`
87. `microservices/drive/catalog/oya-drive-immutability-tier-kernel.yaml`
88. `microservices/drive/catalog/oya-drive-permissions-kernel.yaml`
89. `microservices/drive/catalog/oya-drive-preview-adapter-libreoffice.yaml`
90. `microservices/drive/catalog/oya-drive-preview-adapter-libvips.yaml`
91. `microservices/drive/catalog/oya-drive-search-index-adapter-meilisearch.yaml`
92. `microservices/drive/catalog/oya-drive-search-index-adapter-tika.yaml`
93. `microservices/drive/catalog/oya-drive-share-link-kernel.yaml`
94. `microservices/drive/catalog/oya-drive-sync-kernel.yaml`
95. `microservices/drive/catalog/oya-drive-upload-adapter-valkey.yaml`
96. `microservices/drive/catalog/oya-drive-upload-kernel.yaml`
97. `microservices/drive/competitor-parity-matrix.md`
98. `microservices/drive/compliance.md`
99. `microservices/drive/contracts/asyncapi/drive-events.yaml`
100. `microservices/drive/contracts/openapi/drive.yaml`
101. `microservices/drive/contracts/proto/drive.proto`
102. `microservices/drive/cost-budget.md`
103. `microservices/drive/dashboards/security-dlp.json`
104. `microservices/drive/dashboards/storage-and-bandwidth.json`
105. `microservices/drive/dashboards/sync-pipeline.json`
106. `microservices/drive/decisions/ADR-DRIVE-0001-object-storage-substrate-selection.md`
107. `microservices/drive/decisions/ADR-DRIVE-0002-content-defined-chunking-and-delta-sync.md`
108. `microservices/drive/decisions/ADR-DRIVE-0003-share-link-security-model.md`
109. `microservices/drive/decisions/ADR-DRIVE-0004-encryption-at-rest-and-e2e.md`
110. `microservices/drive/decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md`
111. `microservices/drive/decisions/ADR-DRIVE-0006-immutability-and-worm-policy.md`
112. `microservices/drive/decisions/ADR-DRIVE-001-tenant-cmk-kek-dek-envelope-encryption.md`
113. `microservices/drive/decisions/README.md`
114. `microservices/drive/deprecation-notice.md`
115. `microservices/drive/dpia.md`
116. `microservices/drive/failure-modes.md`
117. `microservices/drive/faqs/drive-engineer-faq.md`
118. `microservices/drive/iac/helm/Chart.yaml`
119. `microservices/drive/iac/helm/templates/deployment.yaml`
120. `microservices/drive/iac/helm/templates/hpa.yaml`
121. `microservices/drive/iac/helm/templates/networkpolicy.yaml`
122. `microservices/drive/iac/helm/templates/pdb.yaml`
123. `microservices/drive/iac/helm/templates/prometheusrule.yaml`
124. `microservices/drive/iac/helm/templates/service.yaml`
125. `microservices/drive/iac/helm/templates/servicemonitor.yaml`
126. `microservices/drive/iac/helm/values.yaml`
127. `microservices/drive/iac/kustomize/base/kustomization.yaml`
128. `microservices/drive/iac/kustomize/base/namespace.yaml`
129. `microservices/drive/iac/kustomize/base/serviceaccount.yaml`
130. `microservices/drive/iac/kustomize/overlays/pack-eu/kustomization.yaml`
131. `microservices/drive/iac/kustomize/overlays/pack-eu/values-pack-eu.yaml`
132. `microservices/drive/iac/kustomize/overlays/pack-kr/kustomization.yaml`
133. `microservices/drive/iac/kustomize/overlays/pack-kr/values-pack-kr.yaml`
134. `microservices/drive/incident-response.md`
135. `microservices/drive/manifest.json`
136. `microservices/drive/migration-from-connect.md`
137. `microservices/drive/migration-playbooks/from-google-drive.md`
138. `microservices/drive/multi-region.md`
139. `microservices/drive/onboarding/drive-engineer-first-week.md`
140. `microservices/drive/packs/EU-AI-Act.md`
141. `microservices/drive/packs/GDPR.md`
142. `microservices/drive/packs/HIPAA.md`
143. `microservices/drive/packs/KR-PIPA.md`
144. `microservices/drive/packs/SOC2.md`
145. `microservices/drive/policy/auditor-scope.cedar`
146. `microservices/drive/policy/ci-scope.cedar`
147. `microservices/drive/policy/data-residency.md`
148. `microservices/drive/policy/dual-context-isolation.md`
149. `microservices/drive/policy/public-read.cedar`
150. `microservices/drive/policy/tenant-scope.cedar`
151. `microservices/drive/reference-implementations/upload-encrypted-file-rust-sdk.md`
152. `microservices/drive/runbooks/dlp-quarantine-release.md`
153. `microservices/drive/runbooks/immutability-tier-violation.md`
154. `microservices/drive/runbooks/object-storage-degraded.md`
155. `microservices/drive/runbooks/share-link-takeover-incident.md`
156. `microservices/drive/runbooks/sync-conflict-resolution.md`
157. `microservices/drive/runbooks/upload-multipart-stuck.md`
158. `microservices/drive/runbooks/virus-scan-rollback.md`
159. `microservices/drive/scorecards/overrides.json`
160. `microservices/drive/sdk-plan.md`
161. `microservices/drive/security/threat-model.md`
162. `microservices/drive/slos/dlp-scan-correctness.openslo.yaml`
163. `microservices/drive/slos/download-first-byte-latency.openslo.yaml`
164. `microservices/drive/slos/file-list-latency.openslo.yaml`
165. `microservices/drive/slos/immutability-tier-correctness.openslo.yaml`
166. `microservices/drive/slos/preview-render-latency.openslo.yaml`
167. `microservices/drive/slos/search-latency.openslo.yaml`
168. `microservices/drive/slos/share-link-generation-latency.openslo.yaml`
169. `microservices/drive/slos/sync-delta-latency.openslo.yaml`
170. `microservices/drive/slos/upload-multipart-throughput.openslo.yaml`
171. `microservices/drive/test-plans/contract-test-strategy.md`
172. `microservices/drive/test-plans/integration-test-strategy.md`
173. `microservices/drive/test-plans/unit-test-strategy.md`
174. `microservices/drive/threat-model.md`
175. `microservices/drive/tutorials/upload-file-with-cmk-rotation.md`

### §2.3 Artifact Coverage Notes

1. Product definition is anchored in `PRD.md`, `ARCHITECTURE.md`, `manifest.json`, ADRs, IPs, and contracts.
2. Operational coverage includes `capacity-model.md`, `failure-modes.md`, `incident-response.md`, runbooks, OpenSLO files, dashboards, and cost budget.
3. Compliance coverage includes `compliance.md`, `dpia.md`, packs, data residency policy, dual-context policy, and threat models.
4. Integration coverage includes OpenAPI 3.2.0 at `microservices/drive/contracts/openapi/drive.yaml:1`.
5. Event coverage includes AsyncAPI 3.1.0 at `microservices/drive/contracts/asyncapi/drive-events.yaml:1`.
6. gRPC coverage includes `proto3` at `microservices/drive/contracts/proto/drive.proto:1`.
7. IaC coverage is concentrated in Helm and Kustomize, as shown by `microservices/drive/IP-001-iac-bootstrap.md:16-24`.
8. `microservices/drive/iac/helm/Chart.yaml:1-20` proves a Helm chart exists and identifies the drive bounded contexts.
9. `microservices/drive/iac/helm/values.yaml:21-139` sizes the workload pods and service ports for several bounded contexts.
10. No root-level `README.md` is present in the inventory, even though the audit packet asked to read one when present.

## §3 9-Dimension Audit

### §3.1 Dimension 1 — Internal Product Coherence

1. Product purpose is coherent: `PRD.md:20-28` describes a file/object storage product with sync, sharing, search, preview, DLP, encryption, WORM, quotas, and migration.
2. The functional requirement list is broad and internally consistent: `PRD.md:40-67` defines FR-01 through FR-23 across storage, folders, upload, download, share links, permissions, search, preview, scanning, immutability, quotas, APIs, SDKs, and migration.
3. The same product surface appears in the OpenAPI description at `contracts/openapi/drive.yaml:6-11`.
4. REST endpoints cover files, folders, upload sessions, download, sync, share links, permissions, search, preview, scan verdicts, and immutability at `contracts/openapi/drive.yaml:64-542`.
5. gRPC services mirror the REST surface with `FileStore`, `FolderHierarchy`, `Upload`, `Download`, `Sync`, `ShareLinkService`, `Permissions`, `Search`, `Preview`, `Scan`, and `Immutability` at `contracts/proto/drive.proto:99-380`.
6. AsyncAPI channels mirror the workflow/event surface for lifecycle, access, share, permissions, sync, scan, immutability, legal hold, and quota at `contracts/asyncapi/drive-events.yaml:29-90`.
7. The architecture file repeats the same bounded-context split in its cross-service links and transport sections, including tenancy, identity, policy-engine, observability, audit-chain, cloud-secrets, cell, and cloud-iac dependencies at `ARCHITECTURE.md:40-46`.
8. Positive finding: drive has a stronger product model than a generic storage service because it explicitly includes preview, DLP, share-link, sync, compliance retention, and cross-tenant policies.
9. Coherence risk: the architecture file carries an anchor-sweep warning that sections must be expanded during content-pass review at `ARCHITECTURE.md:1-3`.
10. Coherence risk: that warning does not invalidate the content, but it means reviewers should treat repetitive sections as needing a later precision pass.

### §3.2 Dimension 2 — Ownership Boundaries

1. The manifest owns many bounded contexts inside one µservice, with file-store, folder-hierarchy, upload, download, sync, share-link, permissions, search-index, preview, dlp-virus-scan, immutability-tier, migration, quota, SDK, and API entries in `manifest.json:1-131`.
2. The PRD assigns drive as the owner of object/file storage semantics, not connect or workspace, at `PRD.md:20-28`.
3. Migration evidence confirms the replacement of legacy `oya-drive-*` surfaces by `oya-drive-*` at `migration-from-connect.md:15-33`.
4. Migration evidence says the old bundled `oya-drive-domain` crate split into specific drive bounded contexts at `migration-from-connect.md:33-63`.
5. The deprecation notice names the old `oya-drive-*` crate family and points to the new µservice ownership at `deprecation-notice.md:16-48`.
6. Ownership is strong for object-store, folder, upload, download, sync, share, permissions, search, preview, scan, immutability, and migration.
7. Ownership is less clear for cross-service OCR handoff because `IP-011-search-index.md:17-20` names a foundry-runtime OCR handoff while `manifest.json:473-492` lists `foundry`, not `foundry-runtime`, as a dependency.
8. That naming drift is a cross-microservice dependency coherence issue, not a product-purpose issue.
9. Drive should retain product ownership for file content and metadata, and consume OCR/extraction or workflow services only through explicit contracts.
10. Current docs mostly follow that boundary because contracts and events keep file lifecycle, search, scan, and share activities under drive-owned namespaces.

### §3.3 Dimension 3 — Substance Versus Scaffold

1. Substance is high in the PRD: `PRD.md:70-112` sets latency, throughput, security, audit, availability, and residency requirements.
2. Substance is high in capacity planning: `capacity-model.md:18-45` defines per-tenant and per-cell object, file, byte, and operation scale.
3. Substance is high in cost planning: `cost-budget.md:20-58` defines per-tenant and per-cell cost envelopes.
4. Substance is high in compliance: `compliance.md:18-48` maps GDPR, SOC 2, ISO 27001, HIPAA, SEC/FINRA, APPI, KR PIPA, PDPA, LGPD, DPDP, NIS2, DORA, and EU AI Act pack overlays.
5. Substance is high in DPIA risk modeling: `dpia.md:124-168` lists drive-specific risks and mitigations.
6. Substance is high in failure-mode coverage: `failure-modes.md:18-167` enumerates storage, metadata, upload, sync, share-link, permissions, search, preview, scan, immutability, quota, audit, key, and residency failure modes.
7. Substance is high in OpenSLO coverage: `slos/file-list-latency.openslo.yaml:16-39`, `slos/upload-multipart-throughput.openslo.yaml:16-38`, `slos/download-first-byte-latency.openslo.yaml:16-39`, `slos/search-latency.openslo.yaml:16-39`, `slos/preview-render-latency.openslo.yaml:16-39`, `slos/sync-delta-latency.openslo.yaml:16-38`, `slos/share-link-generation-latency.openslo.yaml:16-39`, `slos/dlp-scan-correctness.openslo.yaml:16-44`, and `slos/immutability-tier-correctness.openslo.yaml:16-45`.
8. Substance risk exists where implementation-plan bodies are formulaic journey expansions, especially long journey files such as `IP-journey-j17-encrypted-evidence-locker.md`, which contain repeated numbered slice-detail headings.
9. Substance risk also exists because the architecture file self-identifies as anchor-sweep output at `ARCHITECTURE.md:1-3`.
10. Overall dimension result: PASS with P2 cleanup risk, because the path contains real service-specific detail even though some generated sections need a later deslop pass.

### §3.4 Dimension 4 — Canonical-Direction Alignment

1. Multi-context support is not adequately evidenced under the drive path.
2. Canonical contexts are required by `specs/master-plan-sequencing.json:704-742`.
3. The current drive IaC plan is Helm plus Kustomize at `IP-001-iac-bootstrap.md:16-24`.
4. The concrete file targets for that plan are Helm and Kustomize only at `IP-001-iac-bootstrap.md:30-47`.
5. The architecture file also lists IaC transport evidence as Helm/Kustomize manifests at `ARCHITECTURE.md:518` and `ARCHITECTURE.md:578`.
6. OpenTofu is required by `specs/master-plan-sequencing.json:747-756`.
7. The OpenTofu-only doctrine is reinforced by `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md:10-35`.
8. The drive path has no context-specific OpenTofu modules for `oyatie-public-cloud`, `guest-on-aws`, `oci-guest`, `oci-guest/always-free`, `on-prem`, `colo`, or `oyatie-iaas`.
9. OS support is not adequately evidenced because the drive path has no `supported_oses.json` or equivalent manifest.
10. The OS doctrine requires Talos, RHEL, Oracle Linux, SLES, Ubuntu, Debian, Rocky, AlmaLinux, CentOS Stream, Amazon Linux, Flatcar, Photon, and macOS Apple Silicon M5+ coverage per `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:10-78`.
11. Rust-strict alignment is currently clean at the file-extension level: the forbidden-language scan found no `*.py`, `*.js`, `*.ts`, `*.rb`, `*.go`, or `*.java` files under this path.
12. Rust-strict doctrine is anchored at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:10-60`.
13. OCI Always Free profile evidence is missing because no `microservices/drive/iac/oci-guest/always-free/` directory is present.
14. OCI Always Free maximization is required by `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md:10-82`.
15. Canonical-direction result: REVISE, with P1 gaps for multi-context OpenTofu deployability and P2 gaps for OS-support and tenant-class semantics.

#### §3.4.T — Tier Retirement Candidates

Default classification: Wave 15J retirement candidate, P2 documentation gap, because the current audit doctrine retires feature tiers and replaces them with tenant-class semantics.

1. `microservices/drive/onboarding/drive-engineer-first-week.md:12` mentions demo_trial.
2. `microservices/drive/onboarding/drive-engineer-first-week.md:27` mentions demo_trial.
3. `microservices/drive/onboarding/drive-engineer-first-week.md:68` mentions paid.
4. `microservices/drive/onboarding/drive-engineer-first-week.md:269` mentions paid and demo_trial.
5. `microservices/drive/onboarding/drive-engineer-first-week.md:289` mentions compliance_pack and demo_trial.
6. `microservices/drive/onboarding/drive-engineer-first-week.md:323` mentions demo_trial.
7. `microservices/drive/onboarding/drive-engineer-first-week.md:330` mentions paid, paid, and compliance_pack.
8. `microservices/drive/migration-playbooks/from-google-drive.md:26` mentions compliance_pack.
9. `microservices/drive/migration-playbooks/from-google-drive.md:32` mentions paid.
10. `microservices/drive/migration-playbooks/from-google-drive.md:117` mentions paid.
11. `microservices/drive/migration-playbooks/from-google-drive.md:174` mentions paid.
12. `microservices/drive/migration-playbooks/from-google-drive.md:222` mentions paid.
13. `microservices/drive/tutorials/upload-file-with-cmk-rotation.md:16` mentions paid.
14. `microservices/drive/tutorials/upload-file-with-cmk-rotation.md:215` mentions paid.
15. `microservices/drive/tutorials/upload-file-with-cmk-rotation.md:300` mentions paid.
16. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:13` mentions paid.
17. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:21` mentions paid.
18. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:22` mentions paid.
19. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:23` mentions compliance_pack.
20. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:30` mentions compliance_pack.
21. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:36` mentions paid.
22. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:37` mentions paid.
23. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:50` mentions paid.
24. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:63` mentions paid.
25. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:64` mentions paid.
26. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:71` mentions paid.
27. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:77` mentions paid.
28. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:78` mentions paid.
29. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:85` mentions paid.
30. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:91` mentions paid.
31. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:92` mentions paid.
32. `microservices/drive/benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:100` mentions paid and paid.
33. `microservices/drive/faqs/drive-engineer-faq.md:134` mentions demo_trial and paid.
34. `microservices/drive/faqs/drive-engineer-faq.md:135` mentions paid.
35. `microservices/drive/faqs/drive-engineer-faq.md:136` mentions compliance_pack.
36. `microservices/drive/faqs/drive-engineer-faq.md:140` mentions compliance_pack.
37. `microservices/drive/faqs/drive-engineer-faq.md:151` mentions compliance_pack.
38. `microservices/drive/faqs/drive-engineer-faq.md:155` mentions paid.
39. `microservices/drive/faqs/drive-engineer-faq.md:181` mentions paid.
40. `microservices/drive/faqs/drive-engineer-faq.md:195` mentions compliance_pack and paid.
41. `microservices/drive/faqs/drive-engineer-faq.md:207` mentions paid.
42. `microservices/drive/reference-implementations/upload-encrypted-file-rust-sdk.md:193` mentions paid-tier.
43. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:15` mentions demo_trial.
44. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:22` mentions demo_trial.
45. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:59` mentions paid.
46. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:61` mentions demo_trial.
47. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:93` mentions paid.
48. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:95` mentions paid.
49. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:121` mentions paid.
50. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:125` mentions compliance_pack.
51. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:127` mentions paid.
52. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:142` mentions compliance_pack.
53. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:147` mentions paid.
54. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:150` mentions paid.
55. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:165` mentions demo_trial, paid, paid, and compliance_pack.
56. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:172` mentions compliance_pack.
57. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:173` mentions paid.
58. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:174` mentions paid.
59. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:176` mentions paid.
60. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:177` mentions paid.
61. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:178` mentions compliance_pack.
62. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:179` mentions paid.
63. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:180` mentions paid.
64. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:181` mentions compliance_pack.
65. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:182` mentions compliance_pack.
66. `microservices/drive/tenant-class-adoption/tenant-class-adoption-record.md:184` mentions paid.

#### §3.4.C — Tenant-Class Adoption Gaps

1. Required current model for this audit: `demo_trial`, `paid`, and `revenue_share`.
2. The drive path does not express those three values as current tenant-class semantics.
3. The only explicit `tenant_class` policy evidence found is `microservices/drive/policy/ci-scope.cedar:20`, which refuses `production` for CI.
4. The same file permits `synthetic` and `dev` at `microservices/drive/policy/ci-scope.cedar:47-48`.
5. The same file references synthetic cleanup behavior at `microservices/drive/policy/ci-scope.cedar:71`.
6. `microservices/drive/IP-journey-j123-shared-asset-vault.md:40` contains a plain-language revenue-share phrase, but not a current tenant-class control model.
7. The gap is semantic and operational: `demo_trial` needs OCI Always Free caps, hard usage limits, best-effort SLO language, and no compliance/BYOK promise.
8. `paid` needs per-seat plus usage metering, any-context deployment semantics, contractual SLO language, compliance pack eligibility, and BYOK eligibility.
9. `revenue_share` needs at-cost or zero-margin substrate controls, gross-revenue percentage accounting hooks, and limits that prevent substrate subsidy leakage.
10. Current drive docs still describe cost and quota in tenant/generic pack terms rather than the current tenant-class model; see `cost-budget.md:16-18` and `cost-budget.md:60-67`.

### §3.5 Dimension 5 — Industry Counterpart Parity

1. The PRD identifies Google Drive, Dropbox, and OneDrive as direct counterpart anchors at `PRD.md:281-283`.
2. `competitor-parity-matrix.md:18-24` confirms those three as the first three entries in a wider competitor set.
3. Google Drive sets the bar for Workspace search, sharing, OCR, pooled storage, web preview, and global infrastructure.
4. Dropbox sets the bar for sync efficiency, selective sync, file history, large-file desktop ergonomics, and developer-upload guidance.
5. Microsoft OneDrive sets the bar for Office integration, SharePoint document libraries, co-authoring, compliance coupling, and Files On-Demand.
6. Drive reaches parity in planned surface breadth: storage, hierarchy, permissions, sharing, sync, search, preview, compliance, APIs, and migration.
7. Drive claims a differentiator in FastCDC/LBFS delta sync, with the design anchored at `decisions/ADR-DRIVE-0002-content-defined-chunking-and-delta-sync.md:57-74`.
8. Drive claims a differentiator in Cedar-gated cross-tenant sharing, reflected in `competitor-parity-matrix.md:65-69`.
9. Drive claims a differentiator in WORM object-lock for a drive-class product, reflected in `competitor-parity-matrix.md:96-102`.
10. Drive has parity risk in smart/on-demand sync because the local competitor matrix records smart sync as later roadmap rather than current GA at `competitor-parity-matrix.md:75-78`.
11. Drive has parity risk in Office co-authoring because the PRD emphasizes preview and storage, but counterpart OneDrive integrates directly with Microsoft 365 co-authoring.
12. Drive has parity risk in consumer-grade desktop/mobile UX evidence because artifacts focus on backend/service contracts and not client UX implementation.
13. Result: REVISE for evidence maturity, not product ambition.

### §3.6 Dimension 6 — Multi-Context Deployment

1. Canonical six-context support is expected unless excluded with explicit evidence.
2. The drive docs do not provide a `deployment_contexts` manifest mapping.
3. The current IaC file targets are Helm/Kustomize only at `IP-001-iac-bootstrap.md:30-47`.
4. `specs/master-plan-sequencing.json:704-742` requires context identities and context-specific deployment targets.
5. ADR-0328 §D-15 treats all six contexts as supported by default for Phase 0-2 unless a service-specific exception is recorded.
6. The path has no context directory for `oyatie-public-cloud`.
7. The path has no context directory for `guest-on-aws`.
8. The path has no context directory for `oci-guest`.
9. The path has no context directory for `oci-guest/always-free`.
10. The path has no context directory for `on-prem`.
11. The path has no context directory for `colo`.
12. The path has no context directory for `oyatie-iaas`.
13. Drive's product naturally needs all six contexts because file storage is a tenant-facing core service and because the PRD explicitly needs pack, residency, and migration behavior.
14. Dimension result: BLOCK until OpenTofu context evidence exists or a narrowly justified context exception is authored.

### §3.7 Dimension 7 — OpenTofu IaC

1. The OpenTofu requirement is canonical in `specs/master-plan-sequencing.json:747-756`.
2. The local IaC plan says Helm and Kustomize in `IP-001-iac-bootstrap.md:16-24`.
3. The chart itself is Helm API v2 in `iac/helm/Chart.yaml:1-15`.
4. Helm values define Kubernetes workload sizing and substrate dependencies at `iac/helm/values.yaml:21-139`.
5. Kustomize directories exist for base, pack-eu, and pack-kr in inventory rows 127-133 above.
6. Helm and Kustomize may remain useful Kubernetes packaging surfaces, but they do not satisfy the OpenTofu context substrate requirement by themselves.
7. There is no local evidence of `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, or a context README under the six canonical context directories.
8. No Terraform, Pulumi, or CloudFormation files were found in the drive path, so the gap is absence rather than competing forbidden IaC.
9. Dimension result: P1 deployability gap.
10. Remediation should be context-specific OpenTofu modules that call the Helm/Kustomize packaging only after provider-neutral substrate resources exist.

### §3.8 Dimension 8 — OS Support

1. OS support doctrine is mandatory per `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:10-78`.
2. No `supported_oses.json` appears in the complete file inventory.
3. No equivalent manifest was found in `manifest.json`.
4. `manifest.json:293-296` pins Rust and observability substrate versions, but not supported operating systems.
5. Drive has Kubernetes-focused packaging evidence, but Kubernetes packaging does not prove node OS support across Talos, RHEL, Oracle Linux, SLES, Ubuntu, Debian, Rocky, AlmaLinux, CentOS Stream, Amazon Linux, Flatcar, Photon, and macOS Apple Silicon M5+.
6. The missing OS matrix matters for drive because preview sandboxing, object-store backends, kernel/runtime classes, filesystem cache behavior, and Desktop sync clients all have OS-specific constraints.
7. This is a P2 documentation/control-surface gap unless a build lane already owns OS manifests elsewhere.
8. Dimension result: REVISE.

### §3.9 Dimension 9 — Rust-Strict Policy and Forbidden-Language Scan

1. Backend/runtime policy is Rust-first by `specs/master-plan-sequencing.json:817-854`.
2. The explicit memory doctrine forbids Python, JavaScript, TypeScript, Ruby, Go, Java, Scala, Groovy, PHP, and F# for backend/runtime/test/codegen work at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:51-60`.
3. The forbidden-extension scan found no matching code files in `microservices/drive/`.
4. The local contracts are YAML and proto, which are allowed contract formats under the doctrine.
5. The local Cedar policies are policy artifacts, not forbidden application runtimes.
6. The local dashboard JSON files are observability configuration artifacts.
7. The Rust SDK reference implementation is documentation, not runtime code, but its sample direction is aligned with Rust-first intent.
8. Dimension result: PASS for file-extension hygiene.
9. Residual risk: absence of actual `src/` code means this audit cannot prove implementation-level Rust coverage.
10. Residual risk: absence of executable tests means this audit cannot prove Rust test harness coverage.

## §4 Findings Table

| ID | Severity | Dimension | Finding | Evidence | Required Action |
|---|---:|---|---|---|---|
| DRIVE-AUD-001 | P1 | 6,7 | All-six deployability is claimed by canonical default but not evidenced under drive. | `specs/master-plan-sequencing.json:704-742`; `IP-001-iac-bootstrap.md:30-47`; `ARCHITECTURE.md:518`; `ARCHITECTURE.md:578` | Add context-specific OpenTofu modules or record a narrow exception. |
| DRIVE-AUD-002 | P1 | 7 | OpenTofu substrate is absent; IaC is Helm/Kustomize-only. | `specs/master-plan-sequencing.json:747-756`; `iac/helm/Chart.yaml:1-20`; `iac/helm/values.yaml:21-139` | Create OpenTofu modules for canonical contexts and wire Helm as a deploy step. |
| DRIVE-AUD-003 | P2 | 8 | OS support matrix is missing. | `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:10-78`; inventory rows 1-175 | Add `supported_oses.json` or an equivalent machine-readable manifest. |
| DRIVE-AUD-004 | P2 | 4 | Tenant-class semantics are not adopted. | `policy/ci-scope.cedar:20`; `policy/ci-scope.cedar:47-48`; `policy/ci-scope.cedar:71` | Replace synthetic/prod-only policy vocabulary with current tenant-class semantics where product behavior depends on commercial class. |
| DRIVE-AUD-005 | P2 | 4 | 66 retired vocabulary references remain. | §3.4.T candidate list; `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_automation_risk_classes_2026_05_20.md:10-24` | Retire or rewrite those references during Wave 15J cleanup. |
| DRIVE-AUD-006 | P2 | 3 | Architecture file still identifies itself as anchor-sweep output. | `ARCHITECTURE.md:1-3` | Run a precision cleanup pass that preserves substance and removes generated sweep warnings. |
| DRIVE-AUD-007 | P2 | 2 | Foundry dependency naming drifts between `foundry-runtime` and `foundry`. | `IP-011-search-index.md:17-20`; `manifest.json:473-492` | Normalize the dependency name and contract boundary. |
| DRIVE-AUD-008 | P2 | 3 | Root README is missing. | inventory rows 1-175; requested artifact set includes README when present | Add a root README or explicitly fold onboarding into a canonical index. |
| DRIVE-AUD-009 | P2 | 3,9 | No executable `src/` or `tests/` tree exists under drive. | inventory rows 1-175; `PRD.md:355-375` acceptance criteria expect runnable evidence | Add code/test pointers or ensure generated crates are represented by repo-local machine-readable manifests. |
| DRIVE-AUD-010 | P2 | 3 | DPIA status is not closed because sign-off rows are pending. | `dpia.md:170-177` | Close or clearly gate privacy, security, DPO, and SRE sign-offs before claiming compliance readiness. |
| DRIVE-AUD-011 | P3 | 5 | Smart/on-demand sync is a roadmap item, while counterparts ship it. | `competitor-parity-matrix.md:75-78` | Promote smart/on-demand sync evidence or keep it as a clearly dated parity gap. |
| DRIVE-AUD-012 | P3 | 3 | Cost budget still says it justifies per-tier pricing. | `cost-budget.md:16-18` | Rewrite cost-budget vocabulary to tenant classes and usage economics. |

## §5 Open Questions

1. Should drive support all six deployment contexts immediately, or is there a documented reason to stage one or more contexts later?
2. Which file owns the canonical machine-readable `deployment_contexts` declaration for drive: `manifest.json`, a new context manifest, or a generated OpenTofu registry?
3. Should Helm/Kustomize remain the Kubernetes application packaging layer behind OpenTofu, or should OpenTofu render all per-context deployment primitives directly?
4. Where should the drive OS support matrix live so it can be validated automatically with other µservices?
5. Should the old `tenant-class-adoption/` directory be removed during Wave 15J, or retained temporarily as a migration map with an explicit deprecation banner?
6. What exact usage caps define `demo_trial` for drive: bytes stored, objects, daily upload, API requests, preview renders, DLP scans, or all of the above?
7. What exact paid-tenant unit economics should drive expose to billing: seats, stored GiB-month, egress GiB, preview renders, DLP scans, share-link deliveries, and object-lock retention GiB-month?
8. How should `revenue_share` tenants prove gross revenue so drive can enforce at-cost substrate economics without subsidizing unbounded storage?
9. Should the `foundry-runtime` dependency in search/index IPs be renamed to the canonical service in `manifest.json`, or should `manifest.json` add a separate runtime dependency?
10. Should the root README be human-facing, machine-indexed, or both?
11. Which source should own drive desktop/mobile client requirements so OneDrive/Dropbox parity is not reduced to backend-only evidence?
12. Should the DPIA sign-off pending rows block deployment, or only regulated pack launch?
13. Are the local benchmark numbers in `benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:17-100` retained as historical evidence after tier retirement, or should the file be replaced wholesale by tenant-class overlays?
14. Should WORM wording stay as a storage-mode phrase, while old commercial tier wording is retired?
15. Should `immutability-tier` bounded-context names be renamed to `immutability` to avoid confusion with retired commercial feature tiers?

## §6 Remediation Map

1. Remediation R-001 maps to finding DRIVE-AUD-001.
2. R-001 target result: drive has machine-readable six-context deployment evidence.
3. R-001 first artifact: a manifest field or context registry that names `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.
4. R-001 second artifact: context-specific OpenTofu directories matching the canonical context names.
5. R-001 validation: each context has `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and a context README.
6. R-001 risk: drive stores tenant content, so context drift can violate residency, durability, and recovery commitments.
7. R-001 stop condition: every context either has a deployable module or a dated exception signed by architecture and SRE.
8. Remediation R-002 maps to finding DRIVE-AUD-002.
9. R-002 target result: OpenTofu owns substrate and invokes Kubernetes packaging only after substrate exists.
10. R-002 first artifact: `iac/oyatie-public-cloud/` module for public cloud substrate.
11. R-002 second artifact: `iac/guest-on-aws/` module for AWS guest substrate.
12. R-002 third artifact: `iac/oci-guest/` module for OCI guest substrate.
13. R-002 fourth artifact: `iac/oci-guest/always-free/` profile for demo_trial infrastructure caps.
14. R-002 fifth artifact: `iac/on-prem/` module for customer-owned facility substrate.
15. R-002 sixth artifact: `iac/colo/` module for colocation substrate.
16. R-002 seventh artifact: `iac/oyatie-iaas/` module for Oyatie-as-cloud-provider substrate.
17. R-002 validation: no Terraform, Pulumi, CloudFormation, shell-driven provisioning, or SSH bootstrap is introduced.
18. R-002 stop condition: `oya vcs verify` or the successor gate can prove OpenTofu plan evidence for every context.
19. Remediation R-003 maps to finding DRIVE-AUD-003.
20. R-003 target result: drive has OS support as a machine-readable control surface.
21. R-003 first artifact: `supported_oses.json` under `microservices/drive/` or a manifest-embedded equivalent.
22. R-003 required OS coverage: Talos, RHEL, Oracle Linux, SLES, Ubuntu, Debian, Rocky, AlmaLinux, CentOS Stream, Amazon Linux, Flatcar, Photon, and macOS Apple Silicon M5+ where client or packaging applies.
23. R-003 required architecture coverage: arm64, x86_64, and context-specific exceptions.
24. R-003 validation: preview sandbox, object store, OpenBao path, Postgres client, Valkey client, and filesystem sync assumptions are each mapped to supported OS families.
25. R-003 stop condition: unsupported OS claims are removed or explicitly scoped.
26. Remediation R-004 maps to finding DRIVE-AUD-004.
27. R-004 target result: tenant-class behavior is explicit.
28. R-004 first artifact: a `tenant_classes` block that names `demo_trial`, `paid`, and `revenue_share`.
29. R-004 demo_trial caps: stored bytes, file count, daily upload, daily egress, preview renders, scan operations, share-link count, sync sessions, and retention duration.
30. R-004 paid controls: per-seat license, usage billing dimensions, contractual SLO eligibility, compliance pack eligibility, BYOK eligibility, and any-context eligibility.
31. R-004 revenue_share controls: gross-revenue percentage, at-cost substrate policy, zero-margin guardrails, storage cap fallback, and audit evidence for revenue calculation.
32. R-004 validation: no quality downgrade by tenant class.
33. R-004 stop condition: quota, cost, SLO, onboarding, and policy docs use tenant-class vocabulary consistently.
34. Remediation R-005 maps to finding DRIVE-AUD-005.
35. R-005 target result: old feature-vocabulary is retired from drive.
36. R-005 first artifact: rewritten onboarding, migration, tutorial, benchmark, FAQ, reference implementation, and capability-tier docs.
37. R-005 second artifact: the `tenant-class-adoption/` directory is removed or converted into a deprecation bridge with no current design authority.
38. R-005 validation: `rg -n "demo_trial|paid|paid|compliance_pack" microservices/drive` returns only archived-retirement evidence or zero hits.
39. R-005 stop condition: no current drive deliverable can be read as quality-stratified by commercial feature tier.
40. Remediation R-006 maps to finding DRIVE-AUD-006.
41. R-006 target result: architecture reads as a deliberate design document rather than anchor-sweep output.
42. R-006 first artifact: remove the self-identifying sweep warning after each section has a domain-specific assertion, evidence citation, and open risk.
43. R-006 validation: sections that repeat generic patterns are either condensed or replaced with drive-specific architecture decisions.
44. R-006 stop condition: architecture can be reviewed without consulting generated-sweep provenance.
45. Remediation R-007 maps to finding DRIVE-AUD-007.
46. R-007 target result: OCR/search dependency naming is coherent.
47. R-007 first artifact: choose `foundry`, `foundry-runtime`, or another canonical service id and use it consistently.
48. R-007 validation: `manifest.json`, `IP-011-search-index.md`, contracts, and event docs all use the same owner name.
49. R-007 stop condition: search/OCR handoff can be traced to one service owner and one contract.
50. Remediation R-008 maps to finding DRIVE-AUD-008.
51. R-008 target result: a root entry point exists for humans and machines.
52. R-008 first artifact: `README.md` or a generated index that points to PRD, architecture, manifest, ADRs, IPs, contracts, SLOs, runbooks, policies, and parity artifacts.
53. R-008 validation: first-week onboarding can start from the index without guessing the canonical file order.
54. R-008 stop condition: inventory, ownership, and current blockers are discoverable in under one minute.
55. Remediation R-009 maps to finding DRIVE-AUD-009.
56. R-009 target result: executable implementation and test evidence are linked or present.
57. R-009 first artifact: Rust crate paths under drive or manifest pointers to generated crate locations.
58. R-009 second artifact: unit, integration, contract, and e2e test pointers for each bounded context.
59. R-009 validation: `cargo nextest`, contract tests, and smoke tests can be invoked from documented commands.
60. R-009 stop condition: the audit can cite runnable evidence, not only planning documents.
61. Remediation R-010 maps to finding DRIVE-AUD-010.
62. R-010 target result: DPIA status matches sign-off reality.
63. R-010 first artifact: privacy, security, DPO, and SRE sign-off rows are either completed or explicitly marked as launch blockers.
64. R-010 validation: regulated pack enablement refuses until sign-off state is green.
65. R-010 stop condition: compliance docs no longer mix accepted status with pending approval ambiguity.
66. Remediation R-011 maps to finding DRIVE-AUD-011.
67. R-011 target result: smart/on-demand sync is scoped, scheduled, or explicitly excluded.
68. R-011 first artifact: client sync specification with local stubs, hydration trigger, offline behavior, conflict UI, and OS constraints.
69. R-011 validation: desktop and mobile client tests prove sync behavior across at least one supported OS per client family.
70. R-011 stop condition: Dropbox and OneDrive parity status can be judged from evidence, not roadmap glyphs.
71. Remediation R-012 maps to finding DRIVE-AUD-012.
72. R-012 target result: cost budget uses tenant-class and usage economics.
73. R-012 first artifact: replace legacy commercial-tier pricing vocabulary with `demo_trial`, `paid`, and `revenue_share` overlays.
74. R-012 validation: cost alarms map to stored bytes, egress bytes, scan operations, preview renders, KMS/OpenBao calls, metadata shards, and audit-chain seal cost.
75. R-012 stop condition: finance, billing, and SRE can compute cost exposure for each tenant class without reading retired doctrine.

## §7 Final Audit Classification

1. Product coherence classification: PASS.
2. Ownership-boundary classification: PASS with dependency-name cleanup.
3. Substance classification: PASS with generated-section cleanup.
4. Canonical-direction classification: REVISE.
5. Industry-parity classification: REVISE.
6. Multi-context classification: BLOCK.
7. OpenTofu classification: BLOCK.
8. OS-support classification: REVISE.
9. Rust-strict classification: PASS for file hygiene, unproven for implementation because source files are absent.
10. Overall readiness classification: REVISE before any production-readiness claim.
11. P0 finding count: zero.
12. P1 finding count: two.
13. P2 finding count: eight.
14. P3 finding count: two.
15. Tier-retirement candidate count: sixty-six.
16. Tenant-class adoption gap: yes.
17. Top counterpart confirmation: Google Drive, Dropbox, and Microsoft OneDrive.
18. Five cross-cutting constraints evaluated: yes.
19. Halt-cleanly condition invoked: no, because all deliverables could be authored and verified.
20. Next gate: context OpenTofu plus tenant-class control surfaces before drive can claim current masterplan alignment.

<!-- ORCHESTRATOR REPORT
  µservice: drive
  deliverables_landed:
    - /Users/jasonlee/oyatie/microservices/drive/coherence-audit-2026-05-20.md: 651 lines
    - /Users/jasonlee/oyatie/microservices/drive/feature-parity-matrix-2026-05-20.md: 407 lines
    - /Users/jasonlee/oyatie/microservices/drive/performance-benchmark-numbers-2026-05-20.md: 395 lines
  inventory_files_seen: 175
  inventory_lines_read: 40982
  chat_history_matches_processed: 4
  findings_p0: 0
  findings_p1: 2
  findings_p2: 8
  findings_p3: 2
  tier_retirement_candidates_found: 66
  tier_retirement_candidate_cites: onboarding/drive-engineer-first-week.md:12,27,68,269,289,323,330
  tier_retirement_candidate_cites: migration-playbooks/from-google-drive.md:26,32,117,174,222
  tier_retirement_candidate_cites: tutorials/upload-file-with-cmk-rotation.md:16,215,300
  tier_retirement_candidate_cites: benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:13,21,22,23,30,36,37,50,63,64,71,77,78,85,91,92,100
  tier_retirement_candidate_cites: faqs/drive-engineer-faq.md:134,135,136,140,151,155,181,195,207
  tier_retirement_candidate_cites: reference-implementations/upload-encrypted-file-rust-sdk.md:193
  tier_retirement_candidate_cites: tenant-class-adoption/tenant-class-adoption-record.md:15,22,59,61,93,95,121,125,127,142,147,150,165,172,173,174,176,177,178,179,180,181,182,184
  tenant_class_adoption_gaps: yes; only ci-scope production/synthetic/dev exists, not demo_trial/paid/revenue_share product semantics
  top_3_counterparts_confirmed: Google Drive / Dropbox / Microsoft OneDrive
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1453
-->
