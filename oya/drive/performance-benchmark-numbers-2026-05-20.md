---
audit_class: microservice_performance_benchmark_numbers
batch: wave-3-batch-3.2
microservice: drive
service_path: microservices/drive
audit_date: 2026-05-20
counterparts: [Google Drive, Dropbox, Microsoft OneDrive]
status: landed
---

# Drive Performance Benchmark Numbers — 2026-05-20

## Header

Benchmark scope: file storage, upload, download, sync, search, preview, share-link generation, DLP/virus scan correctness, WORM correctness, quota ceilings, and deployability overlays.

Target model: one industry-leader target set for all tenant classes, with deployment-context and tenant-class overlays only where infrastructure or contract shape changes caps.

No feature-schema rows are used.

Methodology disclosure: public SaaS providers rarely publish p50, p95, and p99 request latency for drive products. This document separates official public limits from local benchmark estimates and marks estimates explicitly.

Methodology disclosure: public numbers are treated as hard or soft counterpart ceilings, not as proof of actual latency behavior under every tenant workload.

Methodology disclosure: local Oyatie targets are requirements derived from `PRD.md`, OpenSLO files, and industry-leader comparison, not measured production results.

Methodology disclosure: local historical benchmark file `benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md` contains useful comparator estimates, but it also contains retired commercial vocabulary and is treated as historical input only.

Citation anchor 1: Google Drive API usage limits define project/user quota units, egress threshold, per-method units, 750 GB/day upload, and 5 TB maximum file size at `https://developers.google.com/workspace/drive/api/guides/limits` lines 951-989.

Citation anchor 2: Google Drive Help says all other files can be stored up to 5 TB and lists previewable common file types at `https://support.google.com/drive/answer/37603?hl=en` lines 33-65 and 68-103.

Citation anchor 3: Dropbox Help says the maximum uploadable Dropbox file is 2 TB, browser uploads above 375 GB can be unreliable, and the exact byte ceiling is 2,199,019,061,248 bytes at `https://help.dropbox.com/sync/upload-limitations` lines 60-84.

Citation anchor 4: Microsoft SharePoint limits say OneDrive/SharePoint file upload limit is 250 GB, path limit is 400 decoded characters, each list/library can hold up to 30 million files/folders, and sync optimum is no more than 300,000 files per library at `https://learn.microsoft.com/en-us/office365/servicedescriptions/sharepoint-online-service-description/sharepoint-online-limits` lines 61-70 and 139-145.

Citation anchor 5: Microsoft Graph upload-session docs say upload sessions split files into sequential byte ranges, each request must be less than 60 MiB, byte ranges must be multiples of 320 KiB, resumable transfers should be used for files larger than 10 MiB, and recommended fragment size is 5-10 MiB at `https://learn.microsoft.com/en-us/graph/api/driveitem-createuploadsession?view=graph-rest-1.0` lines 144-147 and 335-337.

Local anchor 1: Drive PRD performance targets are defined at `microservices/drive/PRD.md:70-85`.

Local anchor 2: Drive PRD scale targets are defined at `microservices/drive/PRD.md:310-353`.

Local anchor 3: Drive file-list SLO target is defined at `microservices/drive/slos/file-list-latency.openslo.yaml:16-39`.

Local anchor 4: Drive upload SLO target is defined at `microservices/drive/slos/upload-multipart-throughput.openslo.yaml:16-38`.

Local anchor 5: Drive download SLO target is defined at `microservices/drive/slos/download-first-byte-latency.openslo.yaml:16-39`.

Local anchor 6: Drive search SLO target is defined at `microservices/drive/slos/search-latency.openslo.yaml:16-39`.

Local anchor 7: Drive preview SLO target is defined at `microservices/drive/slos/preview-render-latency.openslo.yaml:16-39`.

Local anchor 8: Drive sync SLO target is defined at `microservices/drive/slos/sync-delta-latency.openslo.yaml:16-38`.

Local anchor 9: Drive share-link SLO target is defined at `microservices/drive/slos/share-link-generation-latency.openslo.yaml:16-39`.

Local anchor 10: Drive correctness SLOs are defined at `microservices/drive/slos/dlp-scan-correctness.openslo.yaml:16-44` and `microservices/drive/slos/immutability-tier-correctness.openslo.yaml:16-45`.

## §1 Methodology

1. Benchmark dimension: metadata latency.
2. Workload: list a folder with 1,000 entries.
3. Local source: `PRD.md:70-85` and `slos/file-list-latency.openslo.yaml:16-39`.
4. Canonical metric: p50, p95, and p99 latency.
5. Benchmark dimension: upload throughput.
6. Workload: 1 GB multipart upload and 10 GB large-file upload.
7. Local source: `PRD.md:70-85`, `slos/upload-multipart-throughput.openslo.yaml:16-38`, and `iac/helm/values.yaml:45-60`.
8. Canonical metric: completion time, MiB/s, retry behavior, chunk size, maximum object size.
9. Benchmark dimension: download latency.
10. Workload: warm CDN first byte, cold cache first byte, and range download.
11. Local source: `PRD.md:70-85` and `slos/download-first-byte-latency.openslo.yaml:16-39`.
12. Canonical metric: first-byte p50, p95, p99.
13. Benchmark dimension: delta sync.
14. Workload: 100 changed files and a 1 GB office-document churn sample.
15. Local source: `PRD.md:70-85`, `slos/sync-delta-latency.openslo.yaml:16-38`, and `decisions/ADR-DRIVE-0002-content-defined-chunking-and-delta-sync.md:57-74`.
16. Canonical metric: p99 session completion and bytes uploaded after edit.
17. Benchmark dimension: search.
18. Workload: 1 million file corpus with per-tenant full-text index.
19. Local source: `PRD.md:70-85` and `slos/search-latency.openslo.yaml:16-39`.
20. Canonical metric: p99 query latency.
21. Benchmark dimension: preview.
22. Workload: image, PDF first page, Office first page, and video keyframe.
23. Local source: `PRD.md:70-85` and `slos/preview-render-latency.openslo.yaml:16-39`.
24. Canonical metric: p99 render latency and sandbox resource cap.
25. Benchmark dimension: share-link mint.
26. Workload: Ed25519 signed link with optional Argon2id password.
27. Local source: `slos/share-link-generation-latency.openslo.yaml:16-39` and `decisions/ADR-DRIVE-0003-share-link-security-model.md:62-100`.
28. Canonical metric: p99 mint latency.
29. Benchmark dimension: correctness.
30. Workload: upload promotion before scan, legal-hold purge attempt, and WORM-retention purge attempt.
31. Local source: `slos/dlp-scan-correctness.openslo.yaml:16-44` and `slos/immutability-tier-correctness.openslo.yaml:16-45`.
32. Canonical metric: correctness ratio, with zero-tolerance violation handling.
33. Benchmark dimension: scale ceiling.
34. Workload: per-cell tenant count, active users, file count, bytes stored, downloads, uploads, and sync sessions.
35. Local source: `PRD.md:326-353` and `capacity-model.md:30-67`.
36. Canonical metric: baseline cell and maximum cell scale.
37. OS disclosure: drive docs do not yet provide `supported_oses.json`; OS-specific performance overlays are therefore requirements, not verified results.
38. Architecture disclosure: current IaC evidence is Helm/Kustomize; OpenTofu six-context substrate is not yet present.
39. Deployment-context disclosure: all six canonical contexts are treated as in scope, but only Kubernetes packaging evidence exists locally.
40. Tenant-class disclosure: this benchmark uses one quality target set; `demo_trial` gets hard usage caps, `paid` scales with contract and payment, and `revenue_share` scales under at-cost or zero-margin substrate controls.

## §2 Counterpart Numbers

### §2.1 Google Drive Numbers

1. Google metric G-01: Drive API per-minute project limit is 1,000,000 quota units; source: Google Drive API limits lines 951-956.
2. Google metric G-02: Drive API per-minute user-per-project limit is 325,000 quota units; source: Google Drive API limits lines 951-956.
3. Google metric G-03: Drive API per-day project egress threshold before charges is 1 TB; source: Google Drive API limits lines 951-956.
4. Google metric G-04: Drive API daily billing threshold is 400,000,000 quota units; source: Google Drive API limits lines 966-968.
5. Google metric G-05: `files.get` consumes 5 quota units; source: Google Drive API limits lines 973-980.
6. Google metric G-06: `files.list` consumes 100 quota units; source: Google Drive API limits lines 973-980.
7. Google metric G-07: `files.download` consumes 200 quota units; source: Google Drive API limits lines 973-980.
8. Google metric G-08: `files.update` consumes 50 quota units; source: Google Drive API limits lines 973-980.
9. Google metric G-09: Workspace users can upload 750 GB/day across My Drive and shared drives; source: Google Drive API limits lines 983-989.
10. Google metric G-10: maximum uploaded file size is 5 TB; source: Google Drive API limits lines 983-989 and Google Drive Help lines 63-65.
11. Google metric G-11: maximum Google Docs converted text document is 50 MB; source: Google Drive Help lines 37-40.
12. Google metric G-12: Google Sheets supports up to 10 million cells or 18,278 columns; source: Google Drive Help lines 42-47.
13. Google metric G-13: Google Slides imported presentation size is up to 100 MB; source: Google Drive Help lines 48-50.
14. Google metric G-14: common preview file families include archives, audio, images, text, video, Adobe files, Microsoft files, and Apple editor files; source: Google Drive Help lines 68-103.
15. Google metric G-15: latency is not publicly published as p50/p95/p99; local comparator estimate is historical only from `benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:24`, `:38`, `:51`, `:65`, and `:79`.

### §2.2 Dropbox Numbers

1. Dropbox metric D-01: maximum Dropbox uploaded file size is 2 TB; source: Dropbox Help lines 60-63.
2. Dropbox metric D-02: exact maximum file size is 2,199,019,061,248 bytes; source: Dropbox Help lines 69-73.
3. Dropbox metric D-03: web browser uploads larger than 375 GB can time out or be interrupted; source: Dropbox Help lines 62-64.
4. Dropbox metric D-04: Dropbox recommends the desktop app or API for large uploads; source: Dropbox Help lines 62-84.
5. Dropbox metric D-05: Dropbox stops syncing when an account is over its storage limit; source: Dropbox Help lines 69-73.
6. Dropbox metric D-06: Dropbox path naming guidance says use fewer than 260 characters in file or folder paths; source: Dropbox Help lines 88-94.
7. Dropbox metric D-07: Dropbox excludes certain file types such as symlinks, `.lnk`, and web-based files; source: Dropbox Help lines 75-79.
8. Dropbox metric D-08: local historical estimate for single-stream upload is 380 MiB/s; source: `benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:32-40`.
9. Dropbox metric D-09: local historical estimate for parallel upload is 1,800 MiB/s; source: `benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:32-40`.
10. Dropbox metric D-10: local historical estimate for changed bytes on 1 GB office-document churn is 18 MiB; source: `benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:46-54`.
11. Dropbox metric D-11: local historical estimate for preview p99 is 4,800 ms; source: `benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:59-67`.
12. Dropbox metric D-12: local historical estimate for cross-region read lag p95 is 18 seconds; source: `benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:73-81`.
13. Dropbox metric D-13: local historical estimate uses Dropbox Business Advanced pricing at 50k seats; source: `benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:87-96`.
14. Dropbox metric D-14: public p50/p95/p99 API latencies are not published in the source set reviewed.
15. Dropbox metric D-15: public sync conflict latency is not published in the source set reviewed.

### §2.3 Microsoft OneDrive Numbers

1. OneDrive metric M-01: file upload limit is 250 GB for OneDrive, SharePoint document libraries, Teams Files tab, and Viva Engage conversations; source: Microsoft SharePoint limits lines 59-62.
2. OneDrive metric M-02: autogenerated ZIP download limit is 20 GB; source: Microsoft SharePoint limits lines 61-66.
3. OneDrive metric M-03: decoded path limit is 400 characters; source: Microsoft SharePoint limits lines 65-66 and Microsoft Support lines 335-348.
4. OneDrive metric M-04: list or library can have up to 30 million files and folders; source: Microsoft SharePoint limits lines 69-70.
5. OneDrive metric M-05: permissions inheritance cannot be broken once a list/library/folder contains more than 100,000 items; source: Microsoft SharePoint limits lines 69-70.
6. OneDrive metric M-06: unique permissions supported limit is 50,000, with a recommended general limit of 5,000; source: Microsoft Support lines 260-263.
7. OneDrive metric M-07: one device can sync one personal account and nine work or school accounts; source: Microsoft Support lines 274-276.
8. OneDrive metric M-08: OneNote notebooks in OneDrive/SharePoint are limited to 2 GB; source: Microsoft Support lines 287-295.
9. OneDrive metric M-09: thumbnails are not generated for images larger than 100 MB; source: Microsoft Support lines 354-360.
10. OneDrive metric M-10: PDF previews are not generated for files larger than 100 MB; source: Microsoft Support lines 354-360.
11. OneDrive metric M-11: OneDrive website can copy up to 2,500 files at one time; source: Microsoft Support lines 363-370.
12. OneDrive metric M-12: optimum sync recommendation is no more than 300,000 items across cloud storage; source: Microsoft Support lines 363-372 and Microsoft SharePoint limits lines 139-141.
13. OneDrive metric M-13: upload sessions can split files into ranges, with each request less than 60 MiB; source: Microsoft Graph lines 144-147.
14. OneDrive metric M-14: upload-session byte ranges must be multiples of 320 KiB; source: Microsoft Graph lines 144-149 and 335-337.
15. OneDrive metric M-15: recommended upload-session fragment size is 5-10 MiB for stable high-speed connections; source: Microsoft Graph lines 335-337.

## §3 Oyatie Target Numbers

### §3.1 Single Industry-Leader Target Set

1. Target O-001: file-list folder latency p50 <= 40 ms.
2. Source: `PRD.md:70-85`.
3. Target O-002: file-list folder latency p95 <= 100 ms.
4. Source: derived from p99 <= 150 ms and cache-hit design in `slos/file-list-latency.openslo.yaml:16-39`.
5. Target O-003: file-list folder latency p99 <= 150 ms for <=1,000 entries.
6. Source: `PRD.md:70-85`; `slos/file-list-latency.openslo.yaml:16-39`.
7. Deployment overlay: `oyatie-public-cloud` target unchanged with elastic metadata pool.
8. Deployment overlay: `guest-on-aws` target unchanged if OpenTofu provisions equivalent Postgres, Valkey, and HPA capacity.
9. Deployment overlay: `guest-on-oci` target unchanged for paid and revenue-share tenants; demo_trial gets lower concurrency cap, not lower quality target.
10. Deployment overlay: `on-prem` target conditional on tenant-provided SSD metadata storage and enough memory for cache.
11. Deployment overlay: `colo` target conditional on storage/network procurement.
12. Deployment overlay: `oyatie-as-cloud-provider` target unchanged if internal substrate matches public-cloud cell design.
13. Tenant overlay: `demo_trial` can be capped to smaller file count and lower request concurrency.
14. Tenant overlay: `paid` can scale with contract and usage charges.
15. Tenant overlay: `revenue_share` can scale under at-cost or zero-margin substrate with gross-revenue accounting.

16. Target O-004: 1 GB multipart upload p99 <= 90 seconds.
17. Source: `PRD.md:70-85`; `slos/upload-multipart-throughput.openslo.yaml:16-38`.
18. Target O-005: effective 1 GB upload throughput >= 11.4 MiB/s at p99 floor.
19. Source: computed from 1 GB / 90 seconds.
20. Target O-006: 10 GB parallel upload target >= 1,800 MiB/s on unconstrained production cells.
21. Source: set to meet or exceed Dropbox local historical estimate in `benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:32-40`, but refreshed measurement required.
22. Target O-007: maximum object size >= 5 TiB.
23. Source: `iac/helm/values.yaml:45-60`, matching Google Drive 5 TB ceiling and exceeding Microsoft 250 GB.
24. Deployment overlay: OCI Always Free profile caps sustained throughput and total storage before it changes the quality target.
25. Deployment overlay: OCI Always Free profile should cap demo_trial storage to fit 200 GB block storage plus allowed object storage envelope.
26. Deployment overlay: on-prem and colo depend on tenant uplink bandwidth and local object-store design.
27. Tenant overlay: `demo_trial` hard-caps daily upload and storage.
28. Tenant overlay: `paid` scales upload volume with per-seat and usage charges.
29. Tenant overlay: `revenue_share` scales upload volume only while gross-revenue share covers substrate cost.

30. Target O-008: warm CDN download first-byte p50 <= 35 ms.
31. Source: target set below p99 budget in `slos/download-first-byte-latency.openslo.yaml:16-39`.
32. Target O-009: warm CDN download first-byte p95 <= 75 ms.
33. Source: target set below p99 budget in `slos/download-first-byte-latency.openslo.yaml:16-39`.
34. Target O-010: warm CDN download first-byte p99 <= 100 ms.
35. Source: `PRD.md:70-85`; `slos/download-first-byte-latency.openslo.yaml:16-39`.
36. Target O-011: cold object-store miss first-byte p99 <= 500 ms.
37. Source: `PRD.md:70-85`; `slos/download-first-byte-latency.openslo.yaml:16-20`.
38. Deployment overlay: public cloud and Oyatie cloud-provider contexts use elastic edge caches.
39. Deployment overlay: guest-on-aws and guest-on-oci use provider egress and local CDN choices.
40. Deployment overlay: on-prem and colo use tenant/provider egress contracts.
41. Tenant overlay: `demo_trial` can cap egress GiB/day and share-link deliveries.
42. Tenant overlay: `paid` and `revenue_share` preserve the same latency target when capacity is funded.

43. Target O-012: search p99 <= 400 ms over a 1 million file corpus.
44. Source: `PRD.md:70-85`; `slos/search-latency.openslo.yaml:16-39`.
45. Target O-013: search p50 <= 120 ms over a 1 million file corpus.
46. Source: derived target under the p99 envelope.
47. Target O-014: search p95 <= 250 ms over a 1 million file corpus.
48. Source: derived target under the p99 envelope.
49. Deployment overlay: all contexts need local per-tenant index placement in the same residency boundary.
50. Tenant overlay: `demo_trial` can cap indexed file count, extracted text size, and query rps.
51. Tenant overlay: `paid` and `revenue_share` scale index shards with usage.

52. Target O-015: preview image/PDF/Office p99 <= 1,000 ms.
53. Source: `slos/preview-render-latency.openslo.yaml:16-39`.
54. Target O-016: video preview keyframe p99 <= 8,000 ms.
55. Source: `PRD.md:70-85`.
56. Target O-017: preview sandbox timeout <= 30 seconds for Office and PDF.
57. Source: `decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md:75-84`.
58. Target O-018: preview queue backpressure begins at >300 queued renders per tenant.
59. Source: `decisions/ADR-DRIVE-0005-preview-pipeline-sandboxing.md:82-85`.
60. Deployment overlay: OCI Always Free profile needs aggressive preview concurrency caps.
61. Deployment overlay: on-prem and colo must prove gVisor or a documented equivalent before enabling untrusted preview.
62. Tenant overlay: `demo_trial` caps concurrent previews and total renders.
63. Tenant overlay: `paid` and `revenue_share` can buy or justify larger render pools.

64. Target O-019: sync delta p99 <= 30 seconds for 100 changed files.
65. Source: `PRD.md:70-85`; `slos/sync-delta-latency.openslo.yaml:16-38`.
66. Target O-020: office-document churn upload bytes <= 18 MiB for 1 GB changed-file sample.
67. Source: meets Dropbox local historical estimate at `benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:46-54`.
68. Target O-021: sync bandwidth saving >= 98.2% on the 1 GB churn workload.
69. Source: equals or beats Dropbox local historical estimate at `benchmarks/gdrive-onedrive-dropbox-box-vs-oyatie.md:46-54`.
70. Target O-022: chunk average size 8 MiB, min 4 MiB, max 16 MiB.
71. Source: `iac/helm/values.yaml:54-60`.
72. Deployment overlay: client network quality dominates end-to-end sync, so context targets need last-mile disclosure.
73. Tenant overlay: `demo_trial` caps concurrent sync sessions and daily changed bytes.
74. Tenant overlay: `paid` and `revenue_share` scale sync workers with usage funding.

75. Target O-023: share-link mint p99 <= 50 ms.
76. Source: `PRD.md:70-85`; `slos/share-link-generation-latency.openslo.yaml:16-39`.
77. Target O-024: share-link signing key rotation every 30 days.
78. Source: `decisions/ADR-DRIVE-0003-share-link-security-model.md:68-70`.
79. Target O-025: share-link max view cap supports up to 10,000,000 views per link.
80. Source: `decisions/ADR-DRIVE-0003-share-link-security-model.md:89-95`.
81. Target O-026: share-link max TTL <= 1 year.
82. Source: `iac/helm/values.yaml:105-114`.
83. Deployment overlay: all contexts need OpenBao and audit-chain availability before public share-link enablement.
84. Tenant overlay: `demo_trial` caps public links, view counts, and daily egress.
85. Tenant overlay: `paid` and `revenue_share` can raise caps while retaining the same p99 target.

86. Target O-027: DLP/virus scan correctness = 100%.
87. Source: `slos/dlp-scan-correctness.openslo.yaml:16-44`.
88. Target O-028: WORM/legal-hold correctness = 100%.
89. Source: `slos/immutability-tier-correctness.openslo.yaml:16-45`.
90. Target O-029: upload promotion without scan verdict = zero tolerated cases.
91. Source: `slos/dlp-scan-correctness.openslo.yaml:16-21`.
92. Target O-030: legal-hold purge before release = zero tolerated cases.
93. Source: `slos/immutability-tier-correctness.openslo.yaml:16-22`.
94. Deployment overlay: no context may enable compliance packs until correctness tests pass.
95. Tenant overlay: `demo_trial` has no compliance-pack promise.
96. Tenant overlay: `paid` and `revenue_share` may enable compliance packs only with correctness evidence.

97. Target O-031: baseline cell tenant count >= 50,000 active tenants.
98. Source: `PRD.md:326-353`; `capacity-model.md:30-45`.
99. Target O-032: maximum planned cell tenant count >= 500,000 active tenants.
100. Source: `PRD.md:326-353`.
101. Target O-033: baseline cell file count >= 1 billion files.
102. Source: `PRD.md:326-353`.
103. Target O-034: maximum planned cell file count >= 10 billion files.
104. Source: `PRD.md:326-353`.
105. Target O-035: baseline stored bytes >= 5 PB.
106. Source: `PRD.md:326-353`.
107. Target O-036: maximum planned stored bytes >= 50 PB.
108. Source: `PRD.md:326-353`.
109. Target O-037: baseline download ops >= 5,000 rps.
110. Source: `PRD.md:326-353`.
111. Target O-038: maximum download ops >= 50,000 rps.
112. Source: `PRD.md:326-353`.
113. Target O-039: baseline upload ops >= 500 rps.
114. Source: `PRD.md:326-353`.
115. Target O-040: maximum upload ops >= 5,000 rps.
116. Source: `PRD.md:326-353`.

### §3.2 Deployment-Context Overlay Summary

1. `oyatie-public-cloud`: target full elastic profile; no target degradation.
2. `oyatie-public-cloud`: must prove OpenTofu substrate before the target can be called deployable.
3. `guest-on-aws`: target full profile if provisioned storage, network, cache, OpenBao, and object-store backends match sizing.
4. `guest-on-aws`: egress cost and provider bandwidth should cap usage, not latency objectives.
5. `guest-on-oci`: target full profile for funded tenants.
6. `guest-on-oci`: OCI Always Free profile is demo_trial infrastructure and caps throughput, storage, preview, and sync concurrency.
7. `on-prem`: target full profile only when tenant supplies SSD-backed metadata, object-store capacity, and required sandbox runtime.
8. `colo`: target full profile only when facility uplink, storage, and hardware isolate noisy tenants.
9. `oyatie-as-cloud-provider`: target full profile if Oyatie substrate exposes the same cell abstractions as public-cloud deployments.
10. All contexts: compliance-pack enablement requires DLP and WORM correctness evidence.
11. All contexts: preview enablement requires sandbox evidence.
12. All contexts: current drive docs are not yet deployability-complete because OpenTofu context modules are missing.

### §3.3 Tenant-Class Overlay Summary

1. `demo_trial`: same latency targets when inside caps.
2. `demo_trial`: hard cap storage, daily upload, daily egress, file count, share-link count, preview render count, DLP scan count, and sync sessions.
3. `demo_trial`: OCI Always Free profile is the default infrastructure shape.
4. `demo_trial`: best-effort SLO language is allowed, but product correctness targets do not degrade.
5. `demo_trial`: no compliance pack promise and no BYOK promise.
6. `paid`: same latency targets with contractual SLO.
7. `paid`: per-seat plus usage-based billing funds scale-out.
8. `paid`: any deployment context is allowed when OpenTofu and OS gates pass.
9. `paid`: compliance packs and BYOK are allowed when validated.
10. `revenue_share`: same latency targets inside funded substrate envelope.
11. `revenue_share`: storage and egress scale under gross-revenue share economics.
12. `revenue_share`: substrate should run at cost or zero margin, with explicit accounting to prevent unbounded subsidy.
13. `revenue_share`: compliance packs and BYOK are allowed only if the revenue-share contract funds them.
14. All classes: no class receives a lower quality implementation.
15. All classes: only caps, billing, deployment-context eligibility, and contractual envelope differ.

## §4 Comparison Narrative

1. Headline comparison C-001: maximum object size.
2. Google Drive: 5 TB official.
3. Dropbox: 2 TB official.
4. OneDrive: 250 GB official.
5. Oyatie target: 5 TiB maximum multipart file size in local values.
6. Verdict: parity with Google Drive, ahead of Dropbox and OneDrive, subject to implementation proof.

7. Headline comparison C-002: daily upload cap.
8. Google Drive: 750 GB/day official.
9. Dropbox: governed by storage quota, with large web uploads above 375 GB less reliable.
10. OneDrive: no single daily cap in cited sources, but 250 GB per-file maximum.
11. Oyatie target: no universal paid cap; demo_trial hard caps; paid and revenue_share scale with economics.
12. Verdict: target ahead for funded tenants, intentionally capped for demo_trial.

13. Headline comparison C-003: upload chunk/request sizing.
14. Google Drive: public API quota units govern calls; file-size ceiling is 5 TB.
15. Dropbox: public help recommends desktop app/API for very large files.
16. OneDrive: each upload-session request less than 60 MiB, multiple of 320 KiB, recommended 5-10 MiB.
17. Oyatie target: FastCDC chunks average 8 MiB, min 4 MiB, max 16 MiB.
18. Verdict: target aligns with OneDrive recommended fragment range while supporting content-defined boundaries.

19. Headline comparison C-004: metadata list latency.
20. Counterparts: no public p99 numbers in cited sources.
21. Oyatie target: p99 <= 150 ms for 1,000 folder entries.
22. Verdict: target is aggressive and must be proven with Rust implementation and cell tests.

23. Headline comparison C-005: search latency.
24. Counterparts: no public p99 numbers in cited sources.
25. Oyatie target: p99 <= 400 ms over 1 million files.
26. Verdict: target is credible only if Meilisearch/Tika shards remain tenant-local and OCR handoff is normalized.

27. Headline comparison C-006: preview ceilings.
28. Google Drive: supports many preview file families.
29. OneDrive: thumbnails/PDF previews are not generated for files larger than 100 MB.
30. Oyatie target: preview image/PDF/Office p99 <= 1,000 ms, with video p99 <= 8,000 ms.
31. Verdict: target is ahead on latency ambition and sandbox posture, but preview-size caps need explicit file-type overlays.

32. Headline comparison C-007: sync item scale.
33. OneDrive: optimum sync recommendation is no more than 300,000 items across cloud storage.
34. Dropbox: public help does not publish p99 sync limits in cited source.
35. Google Drive: public API limits publish quotas but not desktop sync p99.
36. Oyatie target: baseline cell 1 billion files and sync p99 <= 30 seconds for 100 changed files.
37. Verdict: ahead in backend target scale, unproven in client sync ergonomics.

38. Headline comparison C-008: path length.
39. Dropbox: fewer than 260 characters in file/folder paths.
40. OneDrive: 400 decoded characters, with synced OS path constraints.
41. Google Drive: no same path ceiling cited in official docs used here.
42. Oyatie target: not yet specified in drive docs.
43. Verdict: catch-up; drive must set path/name limits before client launch.

44. Headline comparison C-009: correctness.
45. Counterparts: public docs describe product behavior but do not publish correctness SLO ratios for scan and legal hold.
46. Oyatie target: 100% DLP/virus and WORM correctness.
47. Verdict: target is stronger and should remain zero-tolerance.

48. Headline comparison C-010: context deployability.
49. Counterparts: SaaS providers hide infrastructure contexts from customers.
50. Oyatie target: all six contexts with OpenTofu substrate.
51. Current evidence: missing under drive.
52. Verdict: catch-up to Oyatie doctrine before production claim.

53. Headline comparison C-011: tenant-class caps.
54. Counterparts: plan/account limits vary by provider.
55. Oyatie target: same quality across classes, caps only where economics require.
56. Current evidence: missing `demo_trial`, `paid`, and `revenue_share` semantics in drive.
57. Verdict: catch-up to 2026-05-20 tenant-class doctrine.

58. Headline comparison C-012: overall performance posture.
59. Google Drive leads in global infrastructure and 5 TB maximum file size.
60. Dropbox leads in visible sync ergonomics and large desktop upload guidance.
61. Microsoft OneDrive leads in Office/SharePoint integration and published enterprise scale limits.
62. Oyatie targets lead in tenant custody, WORM correctness, DLP correctness, provider-neutral deployment, and transparent delta-sync design.
63. Oyatie is not yet evidence-complete for six-context deployment, executable Rust implementation, OS support, and client UX performance.
