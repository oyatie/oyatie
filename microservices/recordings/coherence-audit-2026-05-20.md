# recordings µservice ownership-coherence audit - 2026-05-20

- µservice: `recordings`
- Scope path: `/Users/jasonlee/oyatie/microservices/recordings/`
- Audit owner: single-agent audit lane for Wave 3 Batch 3.2.
- Deliverable status: substantive audit deliverable 1 of 3.
- Counterpart bar: Zoom Cloud Recording / Gong.io / Otter.ai.
- Deployment assumption under audit: all six canonical contexts unless evidence proves an explicit N/A.
- Tier-retirement directive: retired feature-tier language is evidence of a Wave 15J documentation gap, not a model to extend.
- Tenant-class target under this dispatch: `demo_trial`, `paid`, `revenue_share`.
- Primary local evidence read: 129 service files, 18,848 service lines reported by `wc -l`.
- Chat-history evidence read: recordings dispatch, prior Wave 2 recordings gapfill, top-3 counterpart queue rows, and mid-wave tier-retirement directive.

## §1 Purpose

1. This audit determines whether `microservices/recordings/` is internally coherent, externally competitive, and aligned with the current Oyatie canonical direction.
2. The service is product-critical because `PRD.md:37-48` defines it as the centralized audit-grade recording store for Meet, Messenger calls, Live streams, manual uploads, screen capture, retention, legal hold, eDiscovery, transcript redaction, and export.
3. The service is also substrate-critical because `PRD.md:186-217` maps it across `media-capture`, `transcription`, `redaction`, `retention`, `search`, `playback`, `sharing`, `legal-hold`, `ediscovery`, and `export-packaging` bounded contexts.
4. The expected benchmark surface is not just storage and playback. The prompt-specified industry union includes Zoom Cloud Recording, Gong.io, and Otter.ai.
5. The existing service docs partially compare against Zoom and Otter, but `competitor-parity-matrix.md:15-37` does not cover Gong.io as a first-class counterpart.
6. The prior Wave 2 chat-history task explicitly says recordings was gapfilled against "Zoom Cloud Recordings + Microsoft Stream" and introduced a per-service `capability-tiers/` surface with retired feature-tier language, per chat `8f603fc7...jsonl:10775-10777`.
7. The live Batch 3.2 adjustment says the fourth capability-tier-deltas deliverable is dropped and performance benchmarks must use one industry-leader target set with deployment-context overlays, per chat `8f603fc7...jsonl:16521`.
8. The audit therefore treats older tiered artifacts as findings, not as reusable scaffolding.
9. The canonical multi-context bar requires all six deployment contexts to be evaluated: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`, per `specs/master-plan-sequencing.json:704-746`.
10. The canonical IaC bar requires OpenTofu and forbids Terraform/Pulumi/CloudFormation as current implementation substrates, per `specs/master-plan-sequencing.json:747-776` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2241-2640`.
11. The canonical OS bar requires the supported-OS matrix and per-service evidence, per `specs/master-plan-sequencing.json:777-816` and memory `feedback_os_support_matrix_2026_05_20.md:10-78`.
12. The canonical language bar requires Rust backend and the explicit frontend allowlist, per `specs/master-plan-sequencing.json:817-856` and memory `feedback_rust_strict_only_no_python_2026_05_20.md:10-60`.
13. The canonical OCI profile bar requires an OCI Always Free profile module for applicable guest-on-OCI/demo-trial deployment, per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3666-3697` and memory `feedback_oci_always_free_maximization_2026_05_20.md:10-86`.
14. The current no-tier doctrine is explicit: no demo_trial/paid/paid/compliance_pack-bound paid capability tiers, per memory `feedback_no_capability_tracks_2026_05_20.md:3-55`.
15. The current tenant replacement model under this task is three classes: `demo_trial`, `paid`, and `revenue_share`; prior memory had a two-class form, but this dispatch supersedes it by naming `revenue_share` as a tenant class.
16. The audit's stop condition is an evidence-backed decision surface: what is coherent, what is missing, what must be retired, and what remains open.

## §2 Inventory

### §2.1 Complete file inventory

1. `microservices/recordings/ARCHITECTURE.md`
2. `microservices/recordings/GA-READINESS.md`
3. `microservices/recordings/PHASE-01-RECORDINGS-FOUNDATION.md`
4. `microservices/recordings/PRD.md`
5. `microservices/recordings/benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md`
6. `microservices/recordings/capability-tiers/tier-deltas-and-pricing.md`
7. `microservices/recordings/capability-tiers/tier-matrix.md`
8. `microservices/recordings/capacity-model.md`
9. `microservices/recordings/compliance.md`
10. `microservices/recordings/competitor-parity-matrix.md`
11. `microservices/recordings/contracts/asyncapi/recordings-events.yaml`
12. `microservices/recordings/contracts/openapi/recordings.yaml`
13. `microservices/recordings/contracts/proto/recordings.proto`
14. `microservices/recordings/cost-budget.md`
15. `microservices/recordings/decisions/ADR-RECORDINGS-0001-transcription-and-diarization-substrate.md`
16. `microservices/recordings/decisions/ADR-RECORDINGS-0002-retention-legal-hold-ediscovery.md`
17. `microservices/recordings/decisions/ADR-RECORDINGS-0003-playback-cdn-and-watermarking.md`
18. `microservices/recordings/decisions/ADR-RECORDINGS-0004-redaction-rendering-and-evidence-chain.md`
19. `microservices/recordings/decisions/ADR-RECORDINGS-0005-search-index-and-transcript-privacy.md`
20. `microservices/recordings/decisions/ADR-RECORDINGS-0006-summary-action-items-and-translation-pack.md`
21. `microservices/recordings/decisions/ADR-RECORDINGS-0007-source-ingest-contract.md`
22. `microservices/recordings/decisions/ADR-MS-recordings-2026-05-19.md`
23. `microservices/recordings/decisions/README.md`
24. `microservices/recordings/dpia.md`
25. `microservices/recordings/faqs/compliance-officer-faq.md`
26. `microservices/recordings/failure-modes.md`
27. `microservices/recordings/iac/helm/recordings/Chart.yaml`
28. `microservices/recordings/iac/helm/recordings/templates/deployment.yaml`
29. `microservices/recordings/iac/helm/recordings/templates/hpa.yaml`
30. `microservices/recordings/iac/helm/recordings/templates/networkpolicy.yaml`
31. `microservices/recordings/iac/helm/recordings/templates/pdb.yaml`
32. `microservices/recordings/iac/helm/recordings/templates/prometheusrule.yaml`
33. `microservices/recordings/iac/helm/recordings/templates/service.yaml`
34. `microservices/recordings/iac/helm/recordings/templates/servicemonitor.yaml`
35. `microservices/recordings/iac/helm/recordings/values.yaml`
36. `microservices/recordings/iac/kustomize/base/kustomization.yaml`
37. `microservices/recordings/iac/kustomize/overlays/pack-eu/kustomization.yaml`
38. `microservices/recordings/iac/kustomize/overlays/pack-kr/kustomization.yaml`
39. `microservices/recordings/iac/kustomize/overlays/pack-us-financial/kustomization.yaml`
40. `microservices/recordings/iac/kustomize/overlays/pack-us-healthcare/kustomization.yaml`
41. `microservices/recordings/iac/terraform/grafana-rbac.tf`
42. `microservices/recordings/implementation-plans/IP-001-iac-bootstrap.md`
43. `microservices/recordings/implementation-plans/IP-002-contracts.md`
44. `microservices/recordings/implementation-plans/IP-003-ingest-service.md`
45. `microservices/recordings/implementation-plans/IP-004-storage-router.md`
46. `microservices/recordings/implementation-plans/IP-005-transcription-diarization.md`
47. `microservices/recordings/implementation-plans/IP-006-transcript-search.md`
48. `microservices/recordings/implementation-plans/IP-007-playback-api.md`
49. `microservices/recordings/implementation-plans/IP-008-sharing-links.md`
50. `microservices/recordings/implementation-plans/IP-009-legal-hold.md`
51. `microservices/recordings/implementation-plans/IP-010-retention-worker.md`
52. `microservices/recordings/implementation-plans/IP-011-redaction-renderer.md`
53. `microservices/recordings/implementation-plans/IP-012-export-packager.md`
54. `microservices/recordings/implementation-plans/IP-013-compliance-packs.md`
55. `microservices/recordings/implementation-plans/IP-014-observability.md`
56. `microservices/recordings/implementation-plans/IP-015-ga-hardening.md`
57. `microservices/recordings/implementation-plans/IP-journey-j102-ad-video-variant-performance-analytics.md`
58. `microservices/recordings/implementation-plans/IP-journey-j103-multi-party-attribution-for-ads.md`
59. `microservices/recordings/implementation-plans/IP-journey-j104-partner-retail-media-exchange.md`
60. `microservices/recordings/implementation-plans/IP-journey-j105-shared-audience-clean-room.md`
61. `microservices/recordings/implementation-plans/IP-journey-j106-programmatic-ad-inventory.md`
62. `microservices/recordings/implementation-plans/IP-journey-j107-customer-match-audience-sync.md`
63. `microservices/recordings/implementation-plans/IP-journey-j108-lookalike-audience-modeling.md`
64. `microservices/recordings/implementation-plans/IP-journey-j109-sponsored-search-keyword-marketplace.md`
65. `microservices/recordings/implementation-plans/IP-journey-j110-retargeting-campaign-journey.md`
66. `microservices/recordings/implementation-plans/IP-journey-j111-affiliate-publisher-network.md`
67. `microservices/recordings/implementation-plans/IP-journey-j112-influencer-campaign-management.md`
68. `microservices/recordings/implementation-plans/IP-journey-j113-brand-lift-study-workflow.md`
69. `microservices/recordings/implementation-plans/IP-journey-j114-conversion-lift-experiment.md`
70. `microservices/recordings/implementation-plans/IP-journey-j115-ad-fraud-detection-workflow.md`
71. `microservices/recordings/implementation-plans/IP-journey-j90-communication-privacy-settings.md`
72. `microservices/recordings/implementation-plans/IP-journey-j91-creator-economy-onboarding.md`
73. `microservices/recordings/implementation-plans/IP-journey-j92-creator-payout-setup.md`
74. `microservices/recordings/implementation-plans/IP-journey-j93-creator-income-tax-withholding.md`
75. `microservices/recordings/implementation-plans/IP-journey-j94-digital-product-storefront.md`
76. `microservices/recordings/implementation-plans/IP-journey-j95-affiliate-link-disclosure.md`
77. `microservices/recordings/implementation-plans/IP-journey-j96-brand-collaboration-contracting.md`
78. `microservices/recordings/implementation-plans/IP-journey-j97-crowdfunding-campaign-launch.md`
79. `microservices/recordings/implementation-plans/IP-journey-j98-fan-subscription-memberships.md`
80. `microservices/recordings/implementation-plans/IP-journey-j99-live-shopping-event.md`
81. `microservices/recordings/incident-response.md`
82. `microservices/recordings/manifest.json`
83. `microservices/recordings/migration-playbooks/from-zoom-cloud-recordings.md`
84. `microservices/recordings/multi-region.md`
85. `microservices/recordings/onboarding/compliance-officer-first-week.md`
86. `microservices/recordings/reference-implementations/ingest-and-search-rust-sdk.md`
87. `microservices/recordings/runbooks/cross-region-replay-failover.md`
88. `microservices/recordings/runbooks/ediscovery-export-timeout.md`
89. `microservices/recordings/runbooks/expiring-share-link-abuse.md`
90. `microservices/recordings/runbooks/legal-hold-override-drift.md`
91. `microservices/recordings/runbooks/playback-cdn-cache-cascade.md`
92. `microservices/recordings/runbooks/redaction-render-queue-backlog.md`
93. `microservices/recordings/runbooks/retention-delete-backlog.md`
94. `microservices/recordings/runbooks/transcript-search-index-lag.md`
95. `microservices/recordings/service-lifecycle.md`
96. `microservices/recordings/slos/ediscovery-export-mp4-p99.openslo.yaml`
97. `microservices/recordings/slos/ediscovery-export-transcript-pdf-p99.openslo.yaml`
98. `microservices/recordings/slos/legal-hold-chain-correctness.openslo.yaml`
99. `microservices/recordings/slos/legal-hold-engagement-p99.openslo.yaml`
100. `microservices/recordings/slos/playback-start-p99.openslo.yaml`
101. `microservices/recordings/slos/recording-list-p99.openslo.yaml`
102. `microservices/recordings/slos/redaction-render-p99.openslo.yaml`
103. `microservices/recordings/slos/retention-policy-correctness.openslo.yaml`
104. `microservices/recordings/slos/transcript-render-p99.openslo.yaml`
105. `microservices/recordings/slos/transcript-search-p99.openslo.yaml`
106. `microservices/recordings/threat-model.md`
107. `microservices/recordings/tutorials/legal-hold-engage-and-ediscovery-export.md`
108. `microservices/recordings/ux/UIS-recordings-admin-console.md`
109. `microservices/recordings/ux/UIS-recordings-compliance.md`
110. `microservices/recordings/ux/UIS-recordings-data-subject.md`
111. `microservices/recordings/ux/UIS-recordings-legal.md`
112. `microservices/recordings/ux/UIS-recordings-operator.md`
113. `microservices/recordings/ux/UIS-recordings-viewer.md`
114. `microservices/recordings/workflow-bindings/WF-recording-ingested-index-transcribe.md`
115. `microservices/recordings/workflow-bindings/WF-recording-retention-delete.md`
116. `microservices/recordings/workflows/WF-annotations-produce-clip.md`
117. `microservices/recordings/workflows/WF-compliance-purge-audit.md`
118. `microservices/recordings/workflows/WF-dsr-delete-cascade.md`
119. `microservices/recordings/workflows/WF-ediscovery-export.md`
120. `microservices/recordings/workflows/WF-legal-hold-engage.md`
121. `microservices/recordings/workflows/WF-live-meeting-recording-ingest.md`
122. `microservices/recordings/workflows/WF-manual-upload-ingest.md`
123. `microservices/recordings/workflows/WF-playback-share-link.md`
124. `microservices/recordings/workflows/WF-redaction-render.md`
125. `microservices/recordings/workflows/WF-retention-expiry.md`
126. `microservices/recordings/workflows/WF-search-transcript.md`
127. `microservices/recordings/workflows/WF-summary-action-items.md`
128. `microservices/recordings/workflows/WF-transcript-translation.md`
129. `microservices/recordings/workflows/WF-webhook-recording-source.md`

### §2.2 Inventory completeness notes

1. The inventory command returned exactly 129 files under `microservices/recordings/`.
2. The service has no root `README.md`; the only `README.md` found by the required check is `microservices/recordings/decisions/README.md`.
3. The service has no `supported-oses.json` at the µservice root or the searched first two levels.
4. The service has no `cross-microservice-handoffs.md` at the µservice root or the searched first two levels.
5. The service has `PRD.md`, `ARCHITECTURE.md`, `manifest.json`, contracts, SLOs, capacity, compliance, DPIA, threat-model, runbooks, workflows, onboarding, FAQ, tutorial, benchmark, migration, and reference-implementation artifacts.
6. The service has substantial runtime-intent artifacts, but the canonical deployment-control artifacts are incomplete because no `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/oci-guest/always-free/`, `iac/on-prem/`, `iac/colo/`, or `iac/oyatie-iaas/` module exists.
7. The IaC files present are Helm, Kustomize, and a Terraform-named Grafana RBAC file under `iac/terraform/grafana-rbac.tf`.
8. The forbidden-language source scan for `*.py`, `*.js`, `*.ts`, `*.rb`, `*.go`, `*.java`, `*.scala`, `*.groovy`, `*.php`, and `*.fs` returned no files.
9. The tenant-class search returned one operational phrase, `runbooks/playback-cdn-cache-cascade.md:45`, using "non-paid tenants"; it did not find `tenant_class`, `demo_trial`, or `revenue_share`.
10. The retired tier-term search returned 148 matching lines for demo_trial/paid/paid/compliance_pack-bound paid across the µservice path.

### §2.3 Artifact families read

1. Product definition: `PRD.md:37-48` establishes centralized recording, transcript, redaction, retention, legal-hold, eDiscovery, and export purpose.
2. Product outcomes: `PRD.md:73-99` maps compliance officer, host, viewer, legal reviewer, and operations outcomes.
3. Functional requirements: `PRD.md:102-120` covers ingest, transcript generation, search, playback, sharing, legal hold, retention, redaction, export, admin reporting, and workflow events.
4. Nonfunctional targets: `PRD.md:126-136` covers playback, transcript, search, retention, and export latency/availability.
5. Security/compliance: `PRD.md:140-169` covers encryption, mTLS, SPIFFE, Cedar, consent, audit, HIPAA, SEC 17a-4, EU AI Act, GDPR, DSR, and legal-hold evidence.
6. Architecture source warning: `ARCHITECTURE.md:3` says the file was created by a Wave-3-C anchor sweep and that stub sections must be expanded during content-pass review.
7. Architecture dependencies: `ARCHITECTURE.md:40-46` connects recordings to tenancy, identity, policy-engine, observability, audit-chain, cloud-secrets, cell, and cloud-iac.
8. Manifest scope: `manifest.json:1-45` supplies service metadata, bounded contexts, and contracts.
9. Manifest runtime pins: `manifest.json:233-236` pins Rust 1.83 and container base `cgr.dev/chainguard/rust:latest`.
10. Manifest invariants: `manifest.json:269-274` requires tenant-scoped data, no raw cross-region replication without policy, preview redaction before share, immutable legal hold, and retention worker disablement under hold.
11. OpenAPI contract: `contracts/openapi/recordings.yaml:147-401` exposes recording list/get/playback/transcript/redaction/search/share-link/legal-hold/export/manual-ingest surfaces.
12. AsyncAPI contract: `contracts/asyncapi/recordings-events.yaml:18-214` emits or consumes ingest, publish, playback, share, redaction, deletion, transcript, translation, summary, legal hold, eDiscovery, and retention events.
13. Proto contract: `contracts/proto/recordings.proto:18-127` defines recording, source kind, ingest request, legal hold, and service RPC surfaces.
14. SLO family: `slos/playback-start-p99.openslo.yaml:5-16`, `slos/transcript-search-p99.openslo.yaml:5-17`, and `slos/legal-hold-chain-correctness.openslo.yaml:5-31` provide measurable service objectives.
15. Capacity model: `capacity-model.md:17-28` names demand drivers and `capacity-model.md:112-120` provides an estimated cost model per 1,000 media hours.
16. Existing competitor matrix: `competitor-parity-matrix.md:15-37` compares many products but does not include Gong.io as one of the primary union counterparts.
17. Existing benchmark: `benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md:9-15` defines a workload and hardware profile, but uses retired feature-tier headings in `benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md:19-30`.
18. Existing reference implementation: `reference-implementations/ingest-and-search-rust-sdk.md:1-120` gives a Rust-oriented ingest/search example, but line 92 includes retired feature-tier language.

## §3 Nine-Dimension Audit

### §3.1 Dimension 1 - Product purpose and µservice boundary

1. Judgment: mostly coherent.
2. Evidence: `PRD.md:37-48` defines a clear product surface: centralized audit-grade recording store, durable media retention, transcript indexing, redaction, legal hold, eDiscovery, and export.
3. Evidence: `PRD.md:55-59` positions recordings as a hero product and shared substrate, which is plausible because recordings serves Meet, Messenger calls, Live, manual upload, and screen capture.
4. Evidence: `PRD.md:186-217` splits the domain into practical bounded contexts rather than treating recordings as one storage bucket.
5. Evidence: `contracts/openapi/recordings.yaml:147-401` maps the domain into workable API operations.
6. Evidence: `contracts/asyncapi/recordings-events.yaml:18-214` maps workflow/event integration points.
7. Evidence: `decisions/ADR-RECORDINGS-0007-source-ingest-contract.md:31-56` gives a strong source-ingest contract across Meet, Messenger, Live, manual uploads, screen captures, and external webhooks.
8. Gap: `ARCHITECTURE.md:3` states the architecture document began as an anchor sweep and still needs content-pass expansion.
9. Gap: `ARCHITECTURE.md:23`, `ARCHITECTURE.md:85`, and repeated section markers use generic "tier product" boilerplate that obscures the actual service purpose.
10. Gap: `manifest.json:46-65` uses T0/T1/T2 capability buckets; those are not the retired marketing tiers, but they still need clear separation from the retired feature-tier model because Wave 15J will scrub ambiguous tier language.
11. Gap: the existing first-class competitor framing is not synchronized with the current dispatch: PRD and local parity docs cover Zoom/Otter/Microsoft/Rev/Descript, while this audit's counterpart bar includes Gong.io.
12. Risk: without a refreshed architecture pass, future implementers may optimize for generic media storage rather than the stronger product thesis of compliance-grade recording intelligence.

### §3.2 Dimension 2 - Artifact inventory and documentation depth

1. Judgment: broad artifact coverage, uneven depth.
2. Evidence: 129 files and 18,848 current lines under `microservices/recordings/` show that this is not an empty stub.
3. Evidence: the required artifact classes are well represented: PRD, architecture, ADRs, implementation plans, contracts, SLOs, runbooks, workflows, DPIA, compliance, threat model, capacity model, cost budget, benchmark, migration, FAQ, onboarding, tutorial, and reference implementation.
4. Evidence: `implementation-plans/IP-001-iac-bootstrap.md` through `implementation-plans/IP-015-ga-hardening.md` cover the core service implementation sequence.
5. Evidence: `workflow-bindings/WF-recording-ingested-index-transcribe.md` and `workflow-bindings/WF-recording-retention-delete.md` bind recordings into larger workflows.
6. Evidence: runbooks such as `runbooks/legal-hold-override-drift.md`, `runbooks/redaction-render-queue-backlog.md`, and `runbooks/transcript-search-index-lag.md` give operational surfaces for known failure classes.
7. Gap: root `README.md` is absent, despite the prompt requiring README/PRD/ARCHITECTURE review and despite service onboarding needs.
8. Gap: `cross-microservice-handoffs.md` is absent, despite the service depending on Meet, Messenger, Live, audit-chain, identity, tenancy, policy-engine, docs, cloud-iac, and observability, per `manifest.json:385-403`.
9. Gap: `supported-oses.json` is absent, which blocks the canonical OS evidence surface.
10. Gap: `ARCHITECTURE.md:3` is explicit that the architecture file is not yet a fully expanded substance-pass document.
11. Gap: the Wave 2 gapfill artifacts are now partially doctrinally stale because the no-tier directive retired the feature-tier model after those docs were authored, per chat `8f603fc7...jsonl:10775-10777` and `8f603fc7...jsonl:16521`.
12. Risk: readers can find a lot of text, but several canonical control surfaces are missing or stale in ways that matter for deployability and compliance.

### §3.3 Dimension 3 - Competitive product coherence

1. Judgment: strong compliance-recording surface, incomplete union coverage against current top-3 counterparts.
2. Zoom Cloud Recording counterpart surface: Zoom stores, streams, downloads, and processes cloud recordings, supports multiple layouts, and creates browser-streamable recording files, per Zoom support lines `https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0062627&trk=s-bl:2-18`.
3. Zoom AI surface: Zoom Smart Recording generates summaries, highlights, action items, chapters, and speaker insights from transcripts, per Zoom technical library `https://library.zoom.com/zoom-workplace/artificial-intelligence/artificial-intelligence-bluepaper/ai-companion/ai-companion-features/zoom-recordings:108-129`.
4. Gong.io counterpart surface: Gong emphasizes automatic recording/transcription, keyword/topic detection, sentiment/talk-ratio analysis, deal/pipeline tracking, CRM integrations, coaching, and compliance monitoring, per Gong `https://www.gong.io/conversation-intelligence:32-44`.
5. Gong.io operational surface: Gong records via native provider integration or assistant participant and requires consent-aware settings, per Gong help `https://help.gong.io/docs/understanding-call-recording:73-120`.
6. Otter.ai counterpart surface: Otter AI Chat works during or after meetings, and OtterPilot auto-joins meetings, produces live notes, captures slides, and sends summaries, per Otter help/blog `https://help.otter.ai/hc/en-us/articles/360047872833-Otter-ai-features:45-52` and `https://otter.ai/blog/otter-surpasses-1-billion-meetings-transcribed-and-launches-otterpilot-tm-the-smart-ai-meeting-assistant-to-eliminate-note-taking-and-automate-meeting-summaries:87-105`.
7. Local coverage strength: `PRD.md:102-120` covers ingest, transcript, search, playback, sharing, legal holds, retention, redaction, export, reporting, and workflow events.
8. Local coverage strength: `ADR-RECORDINGS-0001-transcription-and-diarization-substrate.md:68-94` chooses Whisper-large-v3 plus pyannote with deterministic fallback and compliance controls.
9. Local coverage strength: `ADR-RECORDINGS-0002-retention-legal-hold-ediscovery.md:68-123` gives compliance pack retention and eDiscovery bundle policy.
10. Local coverage strength: `ADR-RECORDINGS-0004-redaction-rendering-and-evidence-chain.md` covers redaction rendering and evidence chain, matching or exceeding common meeting assistant products.
11. Union gap: Gong-style revenue-intelligence analytics are not first-class in current artifacts; local docs have compliance/search/summary coverage but not deal-risk, pipeline, CRM-change, objection, competitor mention, talk-ratio, or coaching scorecards as product commitments.
12. Union gap: Otter-style live collaborative transcript comments/highlights/action items are partially covered through summary/action workflows but not fully visible in `contracts/openapi/recordings.yaml:147-401`.
13. Union gap: Zoom-style recording layout variants and 150-file live-session constraints are not modeled in local contracts; local source-kind coverage is strong but layout/file-fragment governance is not explicit.
14. Union gap: existing benchmark and parity docs do not include Gong.io as a top-3 counterpart, so the service's current competitive map is stale against this dispatch.
15. Risk: the current service can credibly target compliance-grade recording, but the union-coverage bar requires a clearer distinction between recordings-as-archive, recordings-as-meeting-assistant, and recordings-as-revenue-intelligence substrate.

### §3.4 Dimension 4 - Canonical-direction alignment

1. Judgment: materially non-conformant on deployment-control doctrine and stale on tier/tenant-class doctrine.
2. Canonical source: ADR-0328 D-15 requires each µservice to state supported deployment contexts or explicit N/A reasons and forbids context support claims without matching `iac/<context>/` modules, per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2079-2084` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2195-2199`.
3. Canonical source: master plan names the six deployment contexts and their expected IaC targets, per `specs/master-plan-sequencing.json:704-746`.
4. Local evidence: `manifest.json:385-403` lists dependencies but does not declare `deployment_contexts` or per-context N/A reasoning.
5. Local evidence: present IaC paths are Helm/Kustomize plus `iac/terraform/grafana-rbac.tf`, not canonical context directories.
6. Canonical source: ADR-0328 D-16 requires OpenTofu and forbids Terraform, Pulumi, CloudFormation, Terraform Cloud, and imperative provisioners as active IaC paths, per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2309` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2464-2492`.
7. Local violation: `implementation-plans/IP-001-iac-bootstrap.md:26` calls out Terraform-managed Grafana RBAC and CloudFront/self-host CDN work.
8. Local violation: `implementation-plans/IP-001-iac-bootstrap.md:45` targets `iac/terraform/grafana-rbac.tf`.
9. Local violation: `implementation-plans/IP-001-iac-bootstrap.md:54` uses a `terraform -chdir=... validate` command rather than `tofu`.
10. Local violation: `iac/terraform/grafana-rbac.tf:1-8` has a Terraform block and Grafana provider configuration.
11. Canonical source: OS support matrix requires Tier-1 OS coverage and manifest evidence, per `specs/master-plan-sequencing.json:777-816` and memory `feedback_os_support_matrix_2026_05_20.md:10-78`.
12. Local gap: no `supported-oses.json` exists under the service path.
13. Canonical source: Rust-strict backend and frontend allowlist are required, per `specs/master-plan-sequencing.json:817-856`.
14. Local evidence: forbidden-language extension scan returned no files, so the service path has no direct Python/JavaScript/TypeScript/Ruby/Go/Java/Scala/Groovy/PHP/F# implementation files.
15. Local caveat: `contracts/proto/recordings.proto:10-12` includes `go_package`, but this is proto generated-client metadata rather than a Go implementation file; if retained, it needs a comment or generated-client policy reference so Rust-strict reviewers do not misclassify it.
16. Canonical source: OCI Always Free must be a provider profile with explicit budget and module surface, not provider-specific architecture, per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3493-3524` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3666-3697`.
17. Local gap: no `iac/oci-guest/always-free/` module exists.
18. Local gap: capacity model has useful scaling numbers in `capacity-model.md:17-120`, but it does not express demo-trial OCI Always Free caps separately from paid/revenue-share scaling.
19. Canonical source: no-tier memory says the feature-tier system is retired and must not be extended, per `feedback_no_capability_tracks_2026_05_20.md:3-55`.
20. Local gap: current service docs contain 148 matching retired tier-term lines, cataloged below.

### §3.4.T Tier retirement candidates

1. Count: 148 matching lines for retired demo_trial/paid/paid/compliance_pack-bound paid terms under `microservices/recordings/`.
2. Severity default: P2 documentation gap unless a line is wired into executable behavior.
3. `capability-tiers/tier-deltas-and-pricing.md:18` - retired term reference.
4. `capability-tiers/tier-deltas-and-pricing.md:19` - retired term reference.
5. `capability-tiers/tier-deltas-and-pricing.md:20` - retired term reference.
6. `capability-tiers/tier-deltas-and-pricing.md:21` - retired term reference.
7. `capability-tiers/tier-deltas-and-pricing.md:22` - retired term reference.
8. `capability-tiers/tier-deltas-and-pricing.md:23` - retired term reference.
9. `capability-tiers/tier-deltas-and-pricing.md:24` - retired term reference.
10. `capability-tiers/tier-deltas-and-pricing.md:25` - retired term reference.
11. `capability-tiers/tier-deltas-and-pricing.md:36` - retired term reference.
12. `capability-tiers/tier-deltas-and-pricing.md:37` - retired term reference.
13. `capability-tiers/tier-deltas-and-pricing.md:38` - retired term reference.
14. `capability-tiers/tier-deltas-and-pricing.md:39` - retired term reference.
15. `capability-tiers/tier-deltas-and-pricing.md:43` - retired term reference.
16. `capability-tiers/tier-deltas-and-pricing.md:47` - retired term reference.
17. `capability-tiers/tier-deltas-and-pricing.md:49` - retired term reference.
18. `capability-tiers/tier-deltas-and-pricing.md:84` - retired term reference.
19. `capability-tiers/tier-deltas-and-pricing.md:85` - retired term reference.
20. `capability-tiers/tier-deltas-and-pricing.md:87` - retired term reference.
21. `capability-tiers/tier-deltas-and-pricing.md:91` - retired term reference.
22. `capability-tiers/tier-deltas-and-pricing.md:93` - retired term reference.
23. `capability-tiers/tier-deltas-and-pricing.md:126` - retired term reference.
24. `capability-tiers/tier-deltas-and-pricing.md:127` - retired term reference.
25. `capability-tiers/tier-deltas-and-pricing.md:128` - retired term reference.
26. `capability-tiers/tier-deltas-and-pricing.md:129` - retired term reference.
27. `capability-tiers/tier-deltas-and-pricing.md:131` - retired term reference.
28. `capability-tiers/tier-deltas-and-pricing.md:135` - retired term reference.
29. `capability-tiers/tier-deltas-and-pricing.md:138` - retired term reference.
30. `capability-tiers/tier-deltas-and-pricing.md:168` - retired term reference.
31. `capability-tiers/tier-deltas-and-pricing.md:169` - retired term reference.
32. `capability-tiers/tier-deltas-and-pricing.md:171` - retired term reference.
33. `capability-tiers/tier-deltas-and-pricing.md:175` - retired term reference.
34. `capability-tiers/tier-deltas-and-pricing.md:205` - retired term reference.
35. `capability-tiers/tier-deltas-and-pricing.md:206` - retired term reference.
36. `capability-tiers/tier-deltas-and-pricing.md:210` - retired term reference.
37. `capability-tiers/tier-deltas-and-pricing.md:211` - retired term reference.
38. `capability-tiers/tier-deltas-and-pricing.md:212` - retired term reference.
39. `capability-tiers/tier-deltas-and-pricing.md:213` - retired term reference.
40. `capability-tiers/tier-deltas-and-pricing.md:214` - retired term reference.
41. `capability-tiers/tier-deltas-and-pricing.md:215` - retired term reference.
42. `capability-tiers/tier-deltas-and-pricing.md:216` - retired term reference.
43. `capability-tiers/tier-deltas-and-pricing.md:217` - retired term reference.
44. `capability-tiers/tier-deltas-and-pricing.md:218` - retired term reference.
45. `capability-tiers/tier-deltas-and-pricing.md:219` - retired term reference.
46. `capability-tiers/tier-deltas-and-pricing.md:220` - retired term reference.
47. `capability-tiers/tier-deltas-and-pricing.md:221` - retired term reference.
48. `capability-tiers/tier-deltas-and-pricing.md:222` - retired term reference.
49. `capability-tiers/tier-deltas-and-pricing.md:223` - retired term reference.
50. `capability-tiers/tier-deltas-and-pricing.md:224` - retired term reference.
51. `capability-tiers/tier-deltas-and-pricing.md:232` - retired term reference.
52. `capability-tiers/tier-deltas-and-pricing.md:236` - retired term reference.
53. `capability-tiers/tier-deltas-and-pricing.md:237` - retired term reference.
54. `capability-tiers/tier-deltas-and-pricing.md:238` - retired term reference.
55. `capability-tiers/tier-deltas-and-pricing.md:239` - retired term reference.
56. `capability-tiers/tier-deltas-and-pricing.md:240` - retired term reference.
57. `capability-tiers/tier-deltas-and-pricing.md:241` - retired term reference.
58. `capability-tiers/tier-deltas-and-pricing.md:242` - retired term reference.
59. `capability-tiers/tier-deltas-and-pricing.md:243` - retired term reference.
60. `capability-tiers/tier-deltas-and-pricing.md:244` - retired term reference.
61. `capability-tiers/tier-deltas-and-pricing.md:245` - retired term reference.
62. `capability-tiers/tier-deltas-and-pricing.md:246` - retired term reference.
63. `capability-tiers/tier-deltas-and-pricing.md:247` - retired term reference.
64. `capability-tiers/tier-deltas-and-pricing.md:248` - retired term reference.
65. `capability-tiers/tier-deltas-and-pricing.md:249` - retired term reference.
66. `capability-tiers/tier-deltas-and-pricing.md:250` - retired term reference.
67. `capability-tiers/tier-deltas-and-pricing.md:251` - retired term reference.
68. `capability-tiers/tier-deltas-and-pricing.md:252` - retired term reference.
69. `capability-tiers/tier-deltas-and-pricing.md:253` - retired term reference.
70. `capability-tiers/tier-deltas-and-pricing.md:254` - retired term reference.
71. `capability-tiers/tier-deltas-and-pricing.md:255` - retired term reference.
72. `capability-tiers/tier-deltas-and-pricing.md:256` - retired term reference.
73. `capability-tiers/tier-deltas-and-pricing.md:257` - retired term reference.
74. `capability-tiers/tier-deltas-and-pricing.md:258` - retired term reference.
75. `capability-tiers/tier-deltas-and-pricing.md:259` - retired term reference.
76. `capability-tiers/tier-deltas-and-pricing.md:265` - retired term reference.
77. `capability-tiers/tier-deltas-and-pricing.md:266` - retired term reference.
78. `capability-tiers/tier-deltas-and-pricing.md:267` - retired term reference.
79. `capability-tiers/tier-deltas-and-pricing.md:268` - retired term reference.
80. `capability-tiers/tier-deltas-and-pricing.md:269` - retired term reference.
81. `capability-tiers/tier-deltas-and-pricing.md:270` - retired term reference.
82. `capability-tiers/tier-deltas-and-pricing.md:271` - retired term reference.
83. `capability-tiers/tier-deltas-and-pricing.md:272` - retired term reference.
84. `capability-tiers/tier-deltas-and-pricing.md:273` - retired term reference.
85. `capability-tiers/tier-deltas-and-pricing.md:274` - retired term reference.
86. `capability-tiers/tier-deltas-and-pricing.md:275` - retired term reference.
87. `capability-tiers/tier-deltas-and-pricing.md:276` - retired term reference.
88. `capability-tiers/tier-deltas-and-pricing.md:277` - retired term reference.
89. `capability-tiers/tier-deltas-and-pricing.md:278` - retired term reference.
90. `capability-tiers/tier-deltas-and-pricing.md:279` - retired term reference.
91. `capability-tiers/tier-deltas-and-pricing.md:280` - retired term reference.
92. `capability-tiers/tier-deltas-and-pricing.md:281` - retired term reference.
93. `capability-tiers/tier-deltas-and-pricing.md:282` - retired term reference.
94. `capability-tiers/tier-deltas-and-pricing.md:283` - retired term reference.
95. `capability-tiers/tier-deltas-and-pricing.md:284` - retired term reference.
96. `capability-tiers/tier-deltas-and-pricing.md:285` - retired term reference.
97. `capability-tiers/tier-deltas-and-pricing.md:286` - retired term reference.
98. `capability-tiers/tier-deltas-and-pricing.md:287` - retired term reference.
99. `capability-tiers/tier-deltas-and-pricing.md:288` - retired term reference.
100. `capability-tiers/tier-deltas-and-pricing.md:289` - retired term reference.
101. `capability-tiers/tier-deltas-and-pricing.md:296` - retired term reference.
102. `capability-tiers/tier-deltas-and-pricing.md:297` - retired term reference.
103. `capability-tiers/tier-deltas-and-pricing.md:298` - retired term reference.
104. `capability-tiers/tier-deltas-and-pricing.md:299` - retired term reference.
105. `capability-tiers/tier-deltas-and-pricing.md:300` - retired term reference.
106. `capability-tiers/tier-deltas-and-pricing.md:301` - retired term reference.
107. `capability-tiers/tier-deltas-and-pricing.md:302` - retired term reference.
108. `capability-tiers/tier-deltas-and-pricing.md:303` - retired term reference.
109. `capability-tiers/tier-deltas-and-pricing.md:304` - retired term reference.
110. `capability-tiers/tier-deltas-and-pricing.md:305` - retired term reference.
111. `capability-tiers/tier-deltas-and-pricing.md:306` - retired term reference.
112. `capability-tiers/tier-deltas-and-pricing.md:307` - retired term reference.
113. `capability-tiers/tier-deltas-and-pricing.md:308` - retired term reference.
114. `capability-tiers/tier-deltas-and-pricing.md:309` - retired term reference.
115. `capability-tiers/tier-deltas-and-pricing.md:310` - retired term reference.
116. `capability-tiers/tier-deltas-and-pricing.md:311` - retired term reference.
117. `capability-tiers/tier-deltas-and-pricing.md:312` - retired term reference.
118. `capability-tiers/tier-deltas-and-pricing.md:313` - retired term reference.
119. `capability-tiers/tier-deltas-and-pricing.md:314` - retired term reference.
120. `capability-tiers/tier-deltas-and-pricing.md:315` - retired term reference.
121. `capability-tiers/tier-deltas-and-pricing.md:316` - retired term reference.
122. `capability-tiers/tier-deltas-and-pricing.md:317` - retired term reference.
123. `capability-tiers/tier-deltas-and-pricing.md:318` - retired term reference.
124. `capability-tiers/tier-deltas-and-pricing.md:324` - retired term reference.
125. `capability-tiers/tier-deltas-and-pricing.md:325` - retired term reference.
126. `capability-tiers/tier-deltas-and-pricing.md:326` - retired term reference.
127. `capability-tiers/tier-matrix.md:13` - retired term reference.
128. `capability-tiers/tier-matrix.md:19` - retired term reference.
129. `capability-tiers/tier-matrix.md:20` - retired term reference.
130. `capability-tiers/tier-matrix.md:44` - retired term reference.
131. `capability-tiers/tier-matrix.md:46` - retired term reference.
132. `capability-tiers/tier-matrix.md:71` - retired term reference.
133. `capability-tiers/tier-matrix.md:73` - retired term reference.
134. `capability-tiers/tier-matrix.md:97` - retired term reference.
135. `capability-tiers/tier-matrix.md:101` - retired term reference.
136. `capability-tiers/tier-matrix.md:103` - retired term reference.
137. `capability-tiers/tier-matrix.md:115` - retired term reference.
138. `capability-tiers/tier-matrix.md:122` - retired term reference.
139. `capability-tiers/tier-matrix.md:128` - retired term reference.
140. `benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md:13` - retired term reference.
141. `benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md:21` - retired term reference.
142. `benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md:22` - retired term reference.
143. `benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md:32` - retired term reference.
144. `benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md:54` - retired term reference.
145. `benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md:55` - retired term reference.
146. `benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md:61` - retired term reference.
147. `benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md:79` - retired term reference.
148. `benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md:101` - retired term reference.
149. `reference-implementations/ingest-and-search-rust-sdk.md:92` - retired term reference.
150. `faqs/compliance-officer-faq.md:94` - retired term reference.
151. `tutorials/legal-hold-engage-and-ediscovery-export.md:15` - retired term reference.
152. Additional ambiguous tier vocabulary should be scrubbed separately even when it is not one of the four retired terms.
153. Ambiguous non-matching examples include `PRD.md:78`, `PRD.md:205`, `manifest.json:48-60`, `manifest.json:326-330`, `manifest.json:358`, `manifest.json:406`, `ARCHITECTURE.md:23`, `ARCHITECTURE.md:573`, `ARCHITECTURE.md:636`, `ARCHITECTURE.md:697`, `slos/retention-policy-correctness.openslo.yaml:24`, `decisions/ADR-RECORDINGS-0002-retention-legal-hold-ediscovery.md:38`, and `decisions/ADR-RECORDINGS-0002-retention-legal-hold-ediscovery.md:199`.
154. These ambiguous examples are not counted in the 148 retired-term total, but they should be handled during Wave 15J so "tier" stops meaning multiple incompatible things.

### §3.4.C Tenant-class adoption gaps

1. Judgment: tenant-class adoption is missing.
2. Required model for this dispatch: `demo_trial`, `paid`, and `revenue_share`.
3. Local search result: no `tenant_class` string exists under `microservices/recordings/`.
4. Local search result: no `demo_trial` string exists under `microservices/recordings/`.
5. Local search result: no `revenue_share` string exists under `microservices/recordings/`.
6. Local search result: one line, `runbooks/playback-cdn-cache-cascade.md:45`, uses "non-paid tenants" in a degraded-mode action.
7. Local search result: "paid" appears elsewhere only inside unrelated "Prepaid Payment Instruments" content in creator-economy journey documents, so those lines do not implement tenant-class semantics.
8. Gap: the service does not define how a demo-trial tenant maps onto the OCI Always Free profile for recordings storage, transcript minutes, legal-hold disablement, compliance-pack exclusion, export caps, or playback/CDN quotas.
9. Gap: the service does not define how a paid tenant maps onto per-seat licensing plus usage-based storage/transcription/export billing.
10. Gap: the service does not define how a revenue-share tenant maps onto at-cost or zero-margin substrate for creator, marketplace, B2C, embedded SaaS, or affiliate use cases.
11. Gap: the existing `capability-tiers/` documents cannot be mechanically renamed into tenant-class docs because the retired feature-tier model stratified product quality, while the tenant-class doctrine keeps a uniform industry-leader quality bar.
12. Risk: without tenant-class semantics, recordings cannot answer whether a demo-trial user can transcribe a long meeting, whether a revenue-share tenant receives compliance export at cost, or whether paid tenants can buy higher storage/throughput without changing feature quality.

### §3.5 Dimension 5 - Contracts, workflows, and cross-service handoffs

1. Judgment: strong contract depth, missing explicit handoff register.
2. Evidence: `contracts/openapi/recordings.yaml:11-13` describes tenant header, SPIFFE mTLS, and Cedar policy expectations.
3. Evidence: `contracts/openapi/recordings.yaml:31-70` models recording and transcript fields.
4. Evidence: `contracts/openapi/recordings.yaml:83-98` models legal holds.
5. Evidence: `contracts/openapi/recordings.yaml:117-133` models eDiscovery and export responses.
6. Evidence: `contracts/openapi/recordings.yaml:147-401` includes the core operation surface.
7. Evidence: `contracts/asyncapi/recordings-events.yaml:18-214` gives event coverage for the workflow substrate.
8. Evidence: `contracts/proto/recordings.proto:57-75` models ingest requests with tenant, source, recording, and idempotency fields.
9. Evidence: `PRD.md:278-308` names workflow events produced and consumed by recordings.
10. Evidence: `ADR-RECORDINGS-0007-source-ingest-contract.md:60-81` defines proto, REST, and AsyncAPI as the canonical ingest contract surface.
11. Gap: no `cross-microservice-handoffs.md` exists, despite service dependencies named in `manifest.json:385-403`.
12. Gap: `ADR-RECORDINGS-0007-source-ingest-contract.md:133-140` allows a direct-call exception in limited hot paths; this exception needs an explicit cross-service handoff and replay guarantee because the canonical direction favors Workflow/event-bus mediation.
13. Gap: Gong-style CRM and sales-engagement handoffs are not represented in the current counterpart union even though Gong is a required counterpart for this audit.
14. Gap: Otter-style calendar auto-join and AI Notetaker removal semantics are not explicit in current contracts.
15. Gap: Zoom-style cloud recording file/layout inventory is not explicit in contracts.
16. Risk: the contracts are strong for legal/compliance recording, but absent handoff documentation makes it hard to prove cross-service ownership without reverse-engineering every contract and workflow file.

### §3.6 Dimension 6 - Deployment-context support

1. Judgment: P1 conformance gap.
2. Canonical source: ADR-0328 D-15 says public cloud context maps to `iac/oyatie-public-cloud/`, guest AWS maps to `iac/guest-on-aws/`, guest OCI maps to `iac/oci-guest/`, on-prem maps to `iac/on-prem/`, colo maps to `iac/colo/`, and Oyatie cloud provider maps to `iac/oyatie-iaas/`, per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1749-1994`.
3. Canonical source: ADR-0328 D-15 also says recordings is in the class where on-prem/colo must receive service-local review rather than silent omission, per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2116-2125`.
4. Local evidence: current `iac/` contains only Helm, Kustomize, and `iac/terraform/grafana-rbac.tf`.
5. Local gap: no `iac/oyatie-public-cloud/` directory exists.
6. Local gap: no `iac/guest-on-aws/` directory exists.
7. Local gap: no `iac/oci-guest/` directory exists.
8. Local gap: no `iac/oci-guest/always-free/` directory exists.
9. Local gap: no `iac/on-prem/` directory exists.
10. Local gap: no `iac/colo/` directory exists.
11. Local gap: no `iac/oyatie-iaas/` directory exists.
12. Local gap: no service-local N/A manifest explains why any context is excluded.
13. Local evidence: `multi-region.md` exists, but multi-region posture is not equivalent to six-context deployability.
14. Local evidence: `cost-budget.md` and `capacity-model.md` provide operating assumptions, but not per-context provisioning modules.
15. Risk: deployability is currently claimed by product posture more than by canonical control-surface evidence.
16. Finding: P1, because missing context modules are explicitly called a violation by ADR-0328 D-15.

### §3.7 Dimension 7 - OpenTofu IaC substrate

1. Judgment: P1 conformance gap.
2. Canonical source: OpenTofu is the approved IaC substrate and Terraform is superseded/forbidden as an active implementation surface, per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2243-2248`.
3. Canonical source: required context module files include `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and `README.md`, per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2296-2309`.
4. Canonical source: forbidden patterns include Terraform Cloud, `terraform` binary usage, Pulumi, CloudFormation, `null_resource`, `local-exec`, `remote-exec`, and SSH provisioners, per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2464-2492`.
5. Local violation: `implementation-plans/IP-001-iac-bootstrap.md:26` says "Terraform-managed Grafana RBAC".
6. Local violation: `implementation-plans/IP-001-iac-bootstrap.md:45` targets `iac/terraform/grafana-rbac.tf`.
7. Local violation: `implementation-plans/IP-001-iac-bootstrap.md:54` runs `terraform -chdir=microservices/recordings/iac/tofu validate`.
8. Local violation: `iac/terraform/grafana-rbac.tf:1-8` begins with a Terraform block and provider requirements.
9. Local partial evidence: `implementation-plans/IP-001-iac-bootstrap.md:16` and `implementation-plans/IP-001-iac-bootstrap.md:32` mention OpenTofu, so the intended direction is visible.
10. Gap: the actual `iac/tofu` path referenced by the validation command is absent from the inventory.
11. Gap: there is no OpenTofu module per canonical context.
12. Gap: there is no OpenTofu state-backend posture per canonical context.
13. Risk: current IaC evidence cannot pass the D-16 gate without either migration or an explicit evidence-bound retirement note.
14. Finding: P1 because the service contains active Terraform-named implementation files and commands.

### §3.8 Dimension 8 - OS support and packaging

1. Judgment: P2 documentation/control-surface gap.
2. Canonical source: supported OS matrix includes Talos, RHEL, Oracle Linux, SUSE Linux Enterprise Server, Ubuntu LTS, Debian, Rocky, Alma, CentOS Stream, Amazon Linux, Flatcar, Photon OS, macOS Apple Silicon M5+, and architecture overlays, per `specs/master-plan-sequencing.json:777-816`.
3. Canonical source: memory `feedback_os_support_matrix_2026_05_20.md:56-72` requires manifest shape for OS support.
4. Local evidence: no `supported-oses.json` exists in the service inventory.
5. Local evidence: Helm and Kustomize artifacts exist, so Kubernetes deployment packaging is partially represented.
6. Local evidence: no per-OS systemd unit, Talos machine config, package metadata, OCI image build matrix, or host requirement document was found under the service path.
7. Local evidence: `manifest.json:233-236` pins Rust 1.83 and a Chainguard Rust image, but that does not express the 13+2+6 OS support bar.
8. Local evidence: `PRD.md:126-136` covers service performance NFRs but not host OS constraints.
9. Local evidence: `capacity-model.md:83-90` discusses Whisper/pyannote GPU throughput, which creates hardware/driver implications that must be reconciled with supported OS/arch matrices.
10. Gap: no M5+ macOS development/build qualification is stated.
11. Gap: no s390x/ppc64le exclusion or N/A rationale is stated.
12. Gap: no distinction exists between server runtime OS, local development OS, and frontend client OS for recordings UI surfaces.
13. Risk: OS portability claims cannot be audited from current service-local artifacts.
14. Finding: P2 because the missing manifest is a canonical documentation/control-surface gap, while direct runtime implementation evidence is not present under this service path.

### §3.9 Dimension 9 - Rust-strict language policy

1. Judgment: current file extension scan is clean, but implementation evidence is incomplete.
2. Canonical source: memory `feedback_rust_strict_only_no_python_2026_05_20.md:10-18` requires Rust backend and forbids Python/Node/Go/Java/Ruby/PHP/F#/Scala/Groovy as backend/substrate implementation languages.
3. Canonical source: memory `feedback_rust_strict_only_no_python_2026_05_20.md:38-49` allows OpenTofu, YAML, JSON, Cedar, OpenAPI, AsyncAPI, Proto, OpenSLO, Markdown, Swift, Kotlin, WinUI 3, and Leptos/WASM SSR with selective hydration as appropriate.
4. Local verification: forbidden-extension scan returned no files for `*.py`, `*.js`, `*.ts`, `*.rb`, `*.go`, `*.java`, `*.scala`, `*.groovy`, `*.php`, or `*.fs`.
5. Local evidence: `manifest.json:233-236` pins Rust 1.83, suggesting backend intent is Rust.
6. Local evidence: `reference-implementations/ingest-and-search-rust-sdk.md` is Rust-oriented, which aligns with Rust-strict direction for examples.
7. Local caveat: `contracts/proto/recordings.proto:10-12` includes a `go_package` option; this is not a Go implementation file, but future cleanup should explain generated-client metadata under the allowed proto surface.
8. Gap: no `src/` directory was found under `microservices/recordings/`, so the service-local audit cannot inspect actual Rust application source.
9. Gap: no service-local `Cargo.toml` was found under the target path.
10. Gap: implementation plans reference Rust services, but the service-local path is still documentation-heavy rather than code-complete.
11. Risk: a clean forbidden-extension scan should not be misread as a buildability pass.
12. Finding: P2 implementation-evidence gap, not a direct forbidden-language violation.

## §4 Findings Table

| ID | Sev | Finding | Evidence | Required disposition |
|---|---:|---|---|---|
| REC-AUD-001 | P1 | Six deployment contexts are not evidenced by canonical `iac/<context>/` modules or N/A manifest. | `specs/master-plan-sequencing.json:704-746`; `docs/decisions/ADR-0328...:1749-1994`; local IaC inventory. | Add/validate context modules or explicit N/A reasons before claiming all-context deployability. |
| REC-AUD-002 | P1 | Active Terraform-named IaC and commands violate OpenTofu-only doctrine. | `implementation-plans/IP-001-iac-bootstrap.md:26`; `implementation-plans/IP-001-iac-bootstrap.md:45`; `implementation-plans/IP-001-iac-bootstrap.md:54`; `iac/terraform/grafana-rbac.tf:1-8`. | Migrate to OpenTofu paths/commands or retire the old artifact with provenance. |
| REC-AUD-003 | P1 | OCI Always Free profile is missing as a demo-trial infrastructure module. | `docs/decisions/ADR-0328...:3666-3697`; no `iac/oci-guest/always-free/` in inventory. | Add OCI Always Free module and caps, or state N/A with canonical rationale. |
| REC-AUD-004 | P2 | Supported OS manifest is absent. | `specs/master-plan-sequencing.json:777-816`; no `supported-oses.json` in inventory. | Add service-local OS/arch support matrix. |
| REC-AUD-005 | P2 | Retired feature-tier terms remain in 148 matching lines. | §3.4.T complete candidate list; memory `feedback_no_capability_tracks_2026_05_20.md:3-55`. | Wave 15J retirement scrub. |
| REC-AUD-006 | P2 | Tenant-class semantics are not implemented in service artifacts. | §3.4.C; only `runbooks/playback-cdn-cache-cascade.md:45` contains a weak "non-paid" phrase. | Add `tenant_class` semantics for `demo_trial`, `paid`, and `revenue_share`. |
| REC-AUD-007 | P2 | Counterpart set is stale because Gong.io is not first-class in current parity docs. | `competitor-parity-matrix.md:15-37`; chat `8f603fc7...jsonl:16290-16311` names Zoom/Gong/Otter. | Refresh parity and benchmark docs around the current top-3 union. |
| REC-AUD-008 | P2 | Architecture doc declares itself an anchor-sweep artifact needing expansion. | `ARCHITECTURE.md:3`. | Expand architecture around current service-specific deployment and runtime boundaries. |
| REC-AUD-009 | P2 | Root README and cross-service handoff register are missing. | Inventory notes; `manifest.json:385-403` dependency list. | Add root README and explicit handoff register. |
| REC-AUD-010 | P2 | Service-local Rust implementation evidence is not present under the target path. | No `src/` or `Cargo.toml` found in inventory; `manifest.json:233-236` only states intent. | Add or link canonical Rust implementation evidence. |
| REC-AUD-011 | P2 | Gong-style revenue-intelligence features are not committed in contracts. | Gong source `https://www.gong.io/conversation-intelligence:32-44`; local `contracts/openapi/recordings.yaml:147-401`. | Decide whether recordings owns revenue conversation analytics or hands them to another µservice. |
| REC-AUD-012 | P3 | Proto `go_package` metadata could be misread under Rust-strict policy. | `contracts/proto/recordings.proto:10-12`. | Add generated-client policy note if proto target generation remains multi-language. |
| REC-AUD-013 | P3 | Existing benchmark methodology uses retired segmentation and does not disclose current tenant-class overlays. | `benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md:9-30`; chat `8f603fc7...jsonl:16521`. | Replace with single target set plus deployment-context and tenant-class overlays. |
| REC-AUD-014 | P3 | Direct-call hot-path exception needs replay/ownership guardrail. | `ADR-RECORDINGS-0007-source-ingest-contract.md:133-140`. | Add explicit exception conditions, replay path, and audit events in handoff docs. |

## §5 Open Questions

1. Should Gong-style revenue-intelligence analytics belong inside `recordings`, or should recordings emit normalized conversation facts to an analytics/intelligence µservice that owns deal-risk and CRM-derived views?
2. What is the authoritative tenant-class schema file for `demo_trial`, `paid`, and `revenue_share`, and should recordings declare it in `manifest.json` or a separate machine-readable policy artifact?
3. Should demo-trial recordings allow any legal-hold or eDiscovery workflow, or should those workflows be disabled with explicit upgrade/error semantics because compliance packs are excluded from demo-trial tenant class?
4. What is the hard cap for demo-trial transcription minutes, retained media hours, share-link bandwidth, and export bundle size under the OCI Always Free profile?
5. Should revenue-share tenants receive at-cost compliance storage/export, or should compliance packs still require paid contractual SLO terms because legal obligations create support exposure?
6. Should `go_package` stay in proto contracts as generated-client metadata, or should generated-client options be moved into generator config to avoid Rust-strict confusion?
7. Should the current Terraform Grafana RBAC file be migrated into an OpenTofu module, replaced by Helm/GitOps RBAC, or retired as stale Wave 1/2 residue?
8. What is the canonical handoff between recordings and Meet for native recording layout, pause/resume, file-fragment count, and host/co-host consent semantics?
9. What is the canonical handoff between recordings and Messenger/Live for call capture, chat transcript capture, captions, viewer replay, and retention?
10. What is the canonical handoff between recordings and policy-engine for per-pack retention override, DSR deletion, legal-hold immutability, and share-link redaction preview?
11. Is `ARCHITECTURE.md` expected to be rewritten from the anchor-sweep template, or should a new machine-readable architecture artifact supersede it under the markdown-retirement policy?
12. Which deployment contexts are truly in scope for GPU-heavy transcription: all six with local acceleration options, or all six with transcription offloaded through a policy-controlled worker pool?
13. Should on-prem and colo deployments include local-only transcription models, external managed speech API fallback, or both under customer policy?
14. Should the capacity model define separate ingestion, transcription, search, playback, export, redaction, and retention SLO caps by deployment context?
15. Should the root README be retired in favor of a machine-readable manifest expansion, given the broader project direction away from prose-only control surfaces?

### §5.1 Immediate audit disposition

1. Gate decision: recordings is not ready to claim canonical all-context deployability.
2. Gate reason: six context modules are missing, per the local IaC inventory and canonical requirements in `specs/master-plan-sequencing.json:704-746`.
3. Gate reason: there is no N/A manifest for contexts where recordings should not deploy.
4. Gate decision: recordings is not ready to claim OpenTofu-only compliance.
5. Gate reason: active local files and plans still reference Terraform, per `implementation-plans/IP-001-iac-bootstrap.md:26`, `implementation-plans/IP-001-iac-bootstrap.md:45`, `implementation-plans/IP-001-iac-bootstrap.md:54`, and `iac/terraform/grafana-rbac.tf:1-8`.
6. Gate decision: recordings is not ready to claim OCI Always Free profile readiness for demo-trial tenants.
7. Gate reason: no `iac/oci-guest/always-free/` module exists and no service-local caps define demo-trial retained media, transcript minutes, share links, or export behavior.
8. Gate decision: recordings is not ready to claim supported-OS matrix coverage.
9. Gate reason: no `supported-oses.json` exists even though canonical OS support is defined in `specs/master-plan-sequencing.json:777-816`.
10. Gate decision: recordings is clean on forbidden implementation-file extensions inside the service path.
11. Gate reason: scan returned no files for forbidden Python, JavaScript, TypeScript, Ruby, Go, Java, Scala, Groovy, PHP, or F# extensions.
12. Gate caveat: absence of forbidden files is not equivalent to buildability because no service-local `src/` or `Cargo.toml` was present in the inventory.
13. Gate decision: recordings has strong product substance around compliance-grade recordings.
14. Gate reason: `PRD.md:37-48`, `PRD.md:102-120`, `contracts/openapi/recordings.yaml:147-401`, and `contracts/asyncapi/recordings-events.yaml:18-214` are concrete and service-specific.
15. Gate decision: recordings has stale competitive framing.
16. Gate reason: the current dispatch and chat history identify Zoom Cloud Recording, Gong.io, and Otter.ai, while local parity artifacts still emphasize Microsoft Stream and omit Gong as a top-3 counterpart.
17. Gate decision: recordings has a major Wave 15J documentation cleanup requirement.
18. Gate reason: 148 matching lines contain retired feature-tier terms, and the current no-tier memory forbids extending that model.
19. Gate decision: recordings has no tenant-class adoption.
20. Gate reason: `tenant_class`, `demo_trial`, and `revenue_share` do not occur under the service path, and the only semantically adjacent line is `runbooks/playback-cdn-cache-cascade.md:45`.
21. Gate decision: legal-hold and eDiscovery design is a relative strength.
22. Gate reason: `ADR-RECORDINGS-0002-retention-legal-hold-ediscovery.md:88-123` and legal-hold SLO files define correctness-sensitive behavior.
23. Gate decision: architecture needs a content pass.
24. Gate reason: `ARCHITECTURE.md:3` explicitly marks the file as anchor-sweep output requiring expansion.
25. Stop condition: this audit should not halt-cleanly as blocked because all three deliverables can be completed with current evidence.
26. Stop condition: this audit should report P1/P2/P3 findings rather than attempt remediation, because the user requested audit deliverables and no commits.
27. Stop condition: no service files outside `microservices/recordings/` should be modified by this audit.
28. Stop condition: the fourth tier-deltas deliverable remains retired and must not be authored.
29. Stop condition: line-count verification must pass for all three deliverables before final report.
30. Stop condition: the orchestrator report must be appended after final line counts are known.

<!-- ORCHESTRATOR REPORT
  µservice: recordings
  deliverables_landed:
    - /Users/jasonlee/oyatie/microservices/recordings/coherence-audit-2026-05-20.md: 622 lines
    - /Users/jasonlee/oyatie/microservices/recordings/feature-parity-matrix-2026-05-20.md: 403 lines
    - /Users/jasonlee/oyatie/microservices/recordings/performance-benchmark-numbers-2026-05-20.md: 310 lines
  inventory_files_seen: 129
  inventory_lines_read: 18848
  chat_history_matches_processed: 6
  findings_p0: 0
  findings_p1: 3
  findings_p2: 8
  findings_p3: 3
  tier_retirement_candidates_found: 148 matching lines; capability-tiers/tier-deltas-and-pricing.md:18-25,36-39,43,47,49,84,85,87,91,93,126-129,131,135,138,168-169,171,175,205-206,210-224,232,236-259,265-289,296-318,324-326; capability-tiers/tier-matrix.md:13,19-20,44,46,71,73,97,101,103,115,122,128; benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md:13,21-22,32,54-55,61,79,101; reference-implementations/ingest-and-search-rust-sdk.md:92; faqs/compliance-officer-faq.md:94; tutorials/legal-hold-engage-and-ediscovery-export.md:15
  tenant_class_adoption_gaps: yes - no tenant_class/demo_trial/revenue_share semantics found; only runbooks/playback-cdn-cache-cascade.md:45 says non-paid tenants
  top_3_counterparts_confirmed: Zoom Cloud Recording / Gong.io / Otter.ai
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1335
-->
