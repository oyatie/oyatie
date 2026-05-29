# design-collaboration Performance Benchmark Numbers - 2026-05-20

Audit owner: single-agent µservice audit for `design-collaboration`.
Target path: `microservices/design-collaboration/`.
Counterpart set: Figma, Adobe XD, InVision.
Methodology status: public vendor performance SLOs for design-collaboration workflows are limited; this report separates published numbers from estimated engineering targets.
Segmentation status: one industry-leader target set is used, with deployment-context overlays and tenant_class overlays.
Tenant classes used: `demo_trial`, `paid`. Paid billing components referenced: `revenue_share`, `per_seat`, `per_usage`.
No retired commercial-segmentation deltas deliverable is authored.
No retired metal-label headings, rows, or product segmentation are used.

## Citation Anchor Block

1. Figma REST rate-limit source: `https://developers.figma.com/docs/rest-api/rate-limits/` lines 71-138.
2. Figma file endpoint source: `https://developers.figma.com/docs/rest-api/file-endpoints/` lines 110-166 and 348-376.
3. Figma webhook source: `https://developers.figma.com/docs/rest-api/webhooks/` lines 74-123.
4. Figma Dev Mode source: `https://help.figma.com/hc/en-us/articles/15023124644247-Guide-to-Dev-Mode` lines 87-232.
5. Adobe XD maintenance and cloud-document source: `https://helpx.adobe.com/support/xd.html` lines 82-104.
6. Adobe XD coediting source: `https://helpx.adobe.com/in/xd/help/collaborate-coedit-designs.html` lines 149-184.
7. Adobe XD design-spec/share source: `https://helpx.adobe.com/xd/help/publish-design-specs.html` lines 159-197.
8. InVision discontinuation source: `https://miro.com/blog/future-miro-freehand/` lines 50-54.
9. InVision Freehand acquisition source: `https://miro.com/newsroom/miro-acquires-freehand-app-from-invision/` lines 117-121.
10. Oyatie ADR target source: `decisions/ADR-DC-001-creative-artifact-operation-log-and-token-promotion-gate.md:56-82` and `decisions/ADR-DC-001-creative-artifact-operation-log-and-token-promotion-gate.md:220-227`.
11. Oyatie SLO source: `slos/availability.openslo.yaml:26-28`, `slos/read-latency.openslo.yaml:26-28`, `slos/write-latency.openslo.yaml:26-28`, and `slos/policy-decision-latency.openslo.yaml:26-28`.
12. Oyatie implementation source: `src/usecase/mod.rs:69-158` and `src/adapter/mod.rs:33-52`.

## §1 Methodology

1. Benchmark dimensions: latency p50, p95, p99, throughput, concurrency, replay, event delivery, API request ceilings, webhook fanout, export throughput, storage envelope, tenant cap, and context portability.
2. Workload W1: design-file open for a medium artifact with metadata, permissions, and audit event.
3. Workload W2: design-file open for a large artifact with asset references and permission evaluation.
4. Workload W3: comment sync, including comment creation, thread update, resolution, audit, and fanout event.
5. Workload W4: version save, including operation-log append, checkpoint decision, and read-after-write visibility.
6. Workload W5: token promotion, including governance policy, review approval, audit, and package reference update.
7. Workload W6: prototype share, including link creation, access-mode decision, and reviewer notification.
8. Workload W7: handoff export, including asset packaging, platform unit conversion, and immutable evidence bundle.
9. Workload W8: asset preview render, including transform, cache lookup, and egress accounting.
10. Workload W9: replay recovery, including checkpoint load and operation-log suffix replay.
11. Workload W10: policy decision latency for a single user action.
12. Operating systems: current target has no supported-OS manifest, so OS-specific numbers are target requirements rather than measured results.
13. Architectures: current target has no architecture matrix, so x86_64 and arm64 expectations are stated as targets, not measured results.
14. Deployment context disclosure: all six canonical contexts are considered because no service-specific N/A evidence exists.
15. Deployment contexts: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.
16. Tenant-class disclosure: demo_trial constrains usage and infrastructure profile; paid scales with license, usage, and any contracted revenue_share component.
17. Published-number rule: a number is labeled source-backed only when it appears in a cited public source.
18. Estimated-number rule: a number is labeled estimated when derived from public feature constraints, common interactive UX tolerances, or Oyatie ADR/SLO intent.
19. Confidence rule: high confidence means public source or repo SLO; medium confidence means target derived from repo ADR/SLO plus industry workflow need; low confidence means no public vendor SLO and estimate is used for audit target setting.
20. Measurement caveat: the audit did not run a live benchmark against counterpart products.
21. Measurement caveat: the audit did not run a production load test against Oyatie because the service currently lacks full context deployment modules.
22. Targeting principle: the Oyatie target should meet or beat active Figma-like experience for governed design collaboration, while also beating Adobe XD and InVision on portability and shutdown resilience.
23. Targeting principle: no product-quality reduction is allowed by tenant class; tenant class constrains caps and cost envelope, not quality bar.
24. Targeting principle: demo_trial may have lower total throughput ceilings because of OCI Always Free profile caps, but each accepted operation should still hit the same latency SLO when under cap.
25. Targeting principle: paid tenants get elastic scaling and contractual SLOs by deployment context.
26. Targeting principle: paid tenants with a revenue_share billing component use at-cost capacity and can scale with business volume after admission control and margin policy pass.
27. Current Oyatie measured status: not measured in this audit.
28. Current Oyatie implementation status: only file open, comment resolve, and token promote are implemented in Rust source.
29. Current Oyatie SLO status: multiple SLO files exist, but their labels require canonical cleanup.
30. Current Oyatie benchmark readiness: blocked by contract/source mismatch, missing context modules, and missing tenant_class fields.

## §2 Counterpart Numbers

### §2.1 Figma Numbers

1. Figma number F-01: file JSON endpoint exists as `GET /v1/files/:key`; source: Figma file endpoint docs lines 110-120.
2. Figma number F-02: file node retrieval supports comma-separated node IDs; source: Figma file endpoint docs lines 122-127.
3. Figma number F-03: file endpoint supports depth control with a positive integer; source: Figma file endpoint docs lines 129-132.
4. Figma number F-04: file image export endpoint exists as `GET /v1/files/:key/images`; source: Figma file endpoint docs lines 348-365.
5. Figma number F-05: file metadata endpoint exists and returns metadata for a file key; source: Figma file endpoint docs lines 370-376.
6. Figma number F-06: current REST rate-limit update took effect November 17, 2025; source: Figma rate-limit docs lines 71-75.
7. Figma number F-07: file and image endpoints for Dev/Full seats show published ceilings of 10/min, 15/min, and 20/min across plan classes in the public table; source: Figma rate-limit docs lines 97-108.
8. Figma number F-08: comments, dev resources, discovery, image fills, projects, variables, version history, and webhooks show published Dev/Full ceilings of 25/min, 50/min, and 100/min; source: Figma rate-limit docs lines 108-123.
9. Figma number F-09: activity logs, components/styles, developer logs, file metadata, library analytics, payments, users, and variable writes show published Dev/Full ceilings of 50/min, 100/min, and 150/min; source: Figma rate-limit docs lines 124-138.
10. Figma number F-10: View/Collab seats may be limited to 6 file/image requests per month; source: Figma rate-limit docs lines 100-107.
11. Figma number F-11: Figma REST rate limiting uses a leaky bucket algorithm; source: Figma rate-limit docs lines 140-142.
12. Figma number F-12: failed webhook delivery is retried 3 times; source: Figma webhook docs lines 118-121.
13. Figma number F-13: webhook retry schedule is 5 minutes, 30 minutes, and 3 hours after consecutive failures; source: Figma webhook docs lines 118-121.
14. Figma number F-14: webhook attachment limits are 20 per team, 5 per project, and 3 per file; source: Figma webhook docs lines 84-88.
15. Figma number F-15: file-context webhook totals by plan class are 150, 300, and 600; source: Figma webhook docs lines 89-93.
16. Figma number F-16: Dev Mode requires a paid plan and a Full or Dev seat; source: Figma Dev Mode docs lines 85-90.
17. Figma number F-17: Dev Mode supports version comparison in the inspect panel; source: Figma Dev Mode docs lines 155-160.
18. Figma number F-18: Dev Mode exposes downloadable asset formats PNG, JPG, SVG, and PDF; source: Figma Dev Mode docs lines 219-232.
19. Figma estimate F-19: active design-file open UX target should stay under 1.5 seconds p95 for medium files; source basis: Oyatie ADR open-file target at ADR lines 220-227 and Figma live-product expectation.
20. Figma estimate F-20: comment sync target should stay under 500 ms p95; source basis: Oyatie ADR comment target at ADR lines 220-227 and Figma webhook/comment surfaces.

### §2.2 Adobe XD Numbers

1. Adobe XD number A-01: XD is in maintenance mode; source: Adobe support page lines 82-84.
2. Adobe XD number A-02: ongoing development and new features are not being shipped; source: Adobe support page lines 82-84.
3. Adobe XD number A-03: cloud documents can be saved and accessed from any online device; source: Adobe support page lines 101-104.
4. Adobe XD number A-04: offline edits to cloud documents are paused to avoid conflicts before reconnection; source: Adobe coediting docs lines 149-153.
5. Adobe XD number A-05: coediting supports simultaneous access and edit invitations by Adobe ID; source: Adobe coediting docs lines 154-161.
6. Adobe XD number A-06: real-time edits are visible while collaborators navigate and modify a design; source: Adobe coediting docs lines 162-175.
7. Adobe XD number A-07: live cursor behavior is documented for collaborators; source: Adobe coediting docs lines 177-184.
8. Adobe XD number A-08: design-spec links can be public; source: Adobe design-spec docs lines 159-163.
9. Adobe XD number A-09: design-spec links can be secure private links; source: Adobe design-spec docs lines 163-164.
10. Adobe XD number A-10: design-spec links can be password links; source: Adobe design-spec docs lines 164-165.
11. Adobe XD number A-11: Share for Development output supports iOS, Web, and Android settings; source: Adobe design-spec docs lines 172-179.
12. Adobe XD number A-12: iOS export assets include 1x, 2x, and 3x; source: Adobe design-spec docs lines 176-179.
13. Adobe XD number A-13: Web export assets include 1x and 2x; source: Adobe design-spec docs lines 176-179.
14. Adobe XD number A-14: link update can preserve existing link identity for updated prototypes/specs; source: Adobe design-spec docs lines 185-196.
15. Adobe XD estimate A-15: maintenance-mode new-feature velocity is 0 new product feature families per release cycle; source basis: Adobe support page lines 82-84.
16. Adobe XD estimate A-16: current benchmark role is compatibility/migration and not active live parity expansion; source basis: Adobe support page lines 82-84.
17. Adobe XD estimate A-17: cloud-document conflict avoidance target for Oyatie should be 0 accepted offline writes without replay validation; source basis: Adobe coediting pause behavior lines 149-153.
18. Adobe XD estimate A-18: design-spec generation target for Oyatie should emit platform units for at least 3 platform families; source basis: Adobe design-spec docs lines 172-179.
19. Adobe XD estimate A-19: share-link access mode target for Oyatie should support at least 3 modes; source basis: Adobe public/private/password docs lines 159-165.
20. Adobe XD estimate A-20: prototype/spec update target for Oyatie should preserve existing comments and conversations during link refresh; source basis: Adobe share docs lines 243-250.

### §2.3 InVision Numbers

1. InVision number I-01: Miro states InVision design products including Prototype and DSM were discontinued effective December 31, 2024; source: Miro blog lines 50-54.
2. InVision number I-02: current version of Freehand was no longer available effective December 31, 2024; source: Miro blog lines 50-54.
3. InVision number I-03: Miro announced acquisition of Freehand from InVision on November 6, 2023; source: Miro newsroom lines 117-121.
4. InVision number I-04: Freehand acquisition included technology assets; source: Miro newsroom lines 117-121.
5. InVision number I-05: Freehand acquisition included brand assets; source: Miro newsroom lines 117-121.
6. InVision number I-06: Freehand acquisition included customer relationships; source: Miro newsroom lines 117-121.
7. InVision number I-07: Freehand acquisition included talent associated with Freehand; source: Miro newsroom lines 117-121.
8. InVision number I-08: Miro planned Freehand major enhancements completed by Summer 2024; source: Miro blog lines 52-54.
9. InVision number I-09: Miro planned data migration services for Freehand customers available Summer 2024; source: Miro blog lines 53-54.
10. InVision estimate I-10: active InVision design-collaboration throughput is 0 accepted new production operations after shutdown; source basis: Miro discontinuation source lines 50-54.
11. InVision estimate I-11: historical prototype migration target for Oyatie should support 100 percent export-attempt audit coverage because source-service shutdown risk is proven; source basis: Miro discontinuation source lines 50-54.
12. InVision estimate I-12: historical DSM migration target for Oyatie should support component and token package import with immutable evidence; source basis: InVision DSM discontinuation in Miro source lines 50-54.
13. InVision estimate I-13: legacy Freehand migration target for Oyatie should support board/image/comment attachment import or explicit unsupported-item evidence; source basis: Miro acquisition source lines 117-121.
14. InVision estimate I-14: shutdown resilience target for Oyatie should include export-before-delete workflows with operator-visible progress; source basis: discontinuation and migration timing in Miro blog lines 50-54.
15. InVision estimate I-15: benchmark role for InVision is catch-up on migration safety, not active live collaboration speed; source basis: discontinued-service status.

## §3 Oyatie Target Numbers - Single Industry-Leader Target Set

1. Target O-01 design-file open p50: 150 ms canonical target for cached medium artifact.
2. Target O-01 overlay oyatie-public-cloud: 150 ms p50 under steady regional capacity.
3. Target O-01 overlay guest-on-aws: 175 ms p50 due customer-network variance.
4. Target O-01 overlay guest-on-oci: 175 ms p50 under paid capacity, 220 ms p50 under OCI Always Free profile.
5. Target O-01 overlay on-prem: 250 ms p50 unless customer storage is certified faster.
6. Target O-01 overlay colo: 200 ms p50 with local cache.
7. Target O-01 overlay oyatie-as-cloud-provider: 150 ms p50 with managed cell placement.
8. Target O-02 design-file open p95: 900 ms canonical target for medium artifact.
9. Target O-02 overlay oyatie-public-cloud: 900 ms p95 with autoscaled cache and storage.
10. Target O-02 overlay guest-on-aws: 1100 ms p95 due customer account variance.
11. Target O-02 overlay guest-on-oci: 1100 ms p95 paid, 1500 ms p95 OCI Always Free profile cap.
12. Target O-02 overlay on-prem: 1500 ms p95 unless local storage certificate permits tighter target.
13. Target O-02 overlay colo: 1200 ms p95.
14. Target O-02 overlay oyatie-as-cloud-provider: 900 ms p95.
15. Target O-03 design-file open p99: 1800 ms canonical target for medium artifact.
16. Target O-03 overlay oyatie-public-cloud: 1800 ms p99.
17. Target O-03 overlay guest-on-aws: 2200 ms p99.
18. Target O-03 overlay guest-on-oci: 2200 ms p99 paid, 3000 ms p99 OCI Always Free profile.
19. Target O-03 overlay on-prem: 3500 ms p99.
20. Target O-03 overlay colo: 2500 ms p99.
21. Target O-03 overlay oyatie-as-cloud-provider: 1800 ms p99.
22. Target O-04 comment sync p50: 75 ms canonical target.
23. Target O-04 deployment overlay: all managed contexts hold 75 ms p50; on-prem and OCI Always Free profile may use 120 ms p50.
24. Target O-05 comment sync p95: 300 ms canonical target.
25. Target O-05 deployment overlay: managed contexts hold 300 ms p95; on-prem/colo/guest contexts permit 500 ms p95 if network evidence supports it.
26. Target O-06 comment sync p99: 750 ms canonical target.
27. Target O-06 deployment overlay: guest and customer-owned contexts permit 1000 ms p99 when external identity or network round trips dominate.
28. Target O-07 version save p50: 100 ms canonical target for operation-log append and acknowledgement.
29. Target O-07 deployment overlay: OCI Always Free profile cap permits 150 ms p50 under capped concurrency.
30. Target O-08 version save p95: 350 ms canonical target.
31. Target O-08 deployment overlay: customer-owned storage contexts permit 750 ms p95 unless storage certification proves lower.
32. Target O-09 version save p99: 900 ms canonical target.
33. Target O-09 deployment overlay: on-prem unoptimized storage permits 1500 ms p99 but must emit facility constraint evidence.
34. Target O-10 token promotion p50: 120 ms canonical target.
35. Target O-10 deployment overlay: all contexts should hold 120 ms p50 because token promotion is governance-heavy but payload-light.
36. Target O-11 token promotion p95: 500 ms canonical target.
37. Target O-11 deployment overlay: external compliance-pack calls may extend to 900 ms p95 with audit cause.
38. Target O-12 token promotion p99: 1200 ms canonical target.
39. Target O-12 deployment overlay: customer-owned compliance service integrations may extend to 2000 ms p99 with explicit dependency label.
40. Target O-13 prototype share p50: 200 ms canonical target for link creation without render.
41. Target O-13 deployment overlay: OCI Always Free profile permits 300 ms p50 and capped creation count.
42. Target O-14 prototype share p95: 750 ms canonical target.
43. Target O-14 deployment overlay: all managed contexts hold 750 ms p95; on-prem external mail/webhook may extend to 1500 ms.
44. Target O-15 prototype share p99: 2000 ms canonical target.
45. Target O-15 deployment overlay: customer-owned mail/webhook dependencies may extend to 3000 ms with reason label.
46. Target O-16 handoff export p50: 800 ms for small export package.
47. Target O-16 deployment overlay: OCI Always Free profile allows 1500 ms p50 and hard export-size cap.
48. Target O-17 handoff export p95: 5000 ms for medium package.
49. Target O-17 deployment overlay: on-prem and guest contexts may vary with local asset store; maximum target remains 8000 ms with facility evidence.
50. Target O-18 handoff export p99: 15000 ms for large package.
51. Target O-18 deployment overlay: large exports must be async beyond 15000 ms and report progress rather than blocking a request.
52. Target O-19 asset preview render p50: 250 ms for cached preview.
53. Target O-19 deployment overlay: all contexts should meet 250 ms p50 for cached preview.
54. Target O-20 asset preview render p95: 2000 ms for uncached preview.
55. Target O-20 deployment overlay: OCI Always Free profile caps concurrent renders and keeps accepted renders at 2000 ms p95.
56. Target O-21 policy decision p50: 10 ms.
57. Target O-21 deployment overlay: all contexts must hold 10 ms p50 for local policy path.
58. Target O-22 policy decision p95: 40 ms.
59. Target O-22 deployment overlay: externalized policy calls are not accepted for latency-critical path unless cached.
60. Target O-23 policy decision p99: 100 ms.
61. Target O-23 deployment overlay: p99 above 100 ms is incident-worthy unless caused by documented customer-owned dependency.
62. Target O-24 replay recovery p50: 500 ms from checkpoint plus small suffix.
63. Target O-24 deployment overlay: customer-owned storage may allow 1000 ms p50 with storage evidence.
64. Target O-25 replay recovery p95: 2000 ms.
65. Target O-25 deployment overlay: matches ADR replay p95 under 2 seconds for bounded suffix.
66. Target O-26 replay recovery p99: 5000 ms.
67. Target O-26 deployment overlay: p99 above 5000 ms requires checkpoint compaction or incident review.
68. Target O-27 operation-log checkpoint cadence: every 500 operations or 5 minutes, whichever comes first.
69. Target O-27 source: ADR lines 56-82.
70. Target O-28 max operation payload: 256 KiB.
71. Target O-28 source: ADR lines 56-82.
72. Target O-29 accepted request throughput paid managed cell: 500 rps mixed read/comment/token workload per service replica set.
73. Target O-29 confidence: medium; estimated because no live load test artifact exists.
74. Target O-30 accepted request throughput OCI Always Free profile: 50 rps mixed workload before admission caps.
75. Target O-30 confidence: medium; derived from 4 OCPU Always Free profile and demo-trial cost cap doctrine.
76. Target O-31 accepted request throughput on-prem: 100 rps minimum certified footprint.
77. Target O-31 confidence: low until hardware certification profile exists.
78. Target O-32 concurrent active editors per artifact: 50 canonical target for paid managed contexts with any contracted revenue_share component.
79. Target O-32 confidence: low because current implementation lacks active-editing model.
80. Target O-33 concurrent reviewers per prototype link: 250 canonical target.
81. Target O-33 confidence: low because prototype-share implementation is absent.
82. Target O-34 webhook delivery p95 after event commit: 2000 ms.
83. Target O-34 deployment overlay: customer-owned endpoints may use delayed retry; accepted internal event must still be durable before response.
84. Target O-35 webhook retry schedule: first retry under 5 minutes, second under 30 minutes, final under 3 hours.
85. Target O-35 rationale: parity with Figma public retry schedule in webhook docs lines 118-121.
86. Target O-36 file webhook equivalent count: minimum 3 hooks per artifact.
87. Target O-36 rationale: parity with Figma file-context count in webhook docs lines 84-88.
88. Target O-37 project-equivalent webhook count: minimum 5 hooks per workspace/project.
89. Target O-37 rationale: parity with Figma project-context count in webhook docs lines 84-88.
90. Target O-38 team-equivalent webhook count: minimum 20 hooks per tenant workspace.
91. Target O-38 rationale: parity with Figma team-context count in webhook docs lines 84-88.
92. Target O-39 design-system export package size synchronous path: up to 25 MiB.
93. Target O-39 overlay: larger exports become async jobs with progress and cancellation.
94. Target O-40 handoff asset export formats: PNG, JPG, SVG, PDF, plus token JSON package.
95. Target O-40 rationale: parity with Figma Dev Mode export formats plus token package need.
96. Target O-41 platform handoff units: Web px, iOS pt at 1x/2x/3x, Android dp.
97. Target O-41 rationale: Adobe XD Share for Development source lines 172-179.
98. Target O-42 prototype link access modes: public, private invited, password, expiring signed link.
99. Target O-42 rationale: Adobe XD public/private/password source lines 159-165 plus enterprise expiry need.
100. Target O-43 comments preserved during link update: 100 percent preservation or explicit migration failure record.
101. Target O-43 rationale: Adobe XD link update and comment preservation source lines 243-250.
102. Target O-44 export audit coverage: 100 percent for migration workflows.
103. Target O-44 rationale: InVision shutdown creates hard migration-loss risk.
104. Target O-45 accepted operation audit emission p95: 1000 ms.
105. Target O-45 source alignment: target SLO file `slos/audit-emission-lag.openslo.yaml:26-28`.
106. Target O-46 service availability: 99.9 percent monthly for paid managed contexts.
107. Target O-46 source alignment: `slos/availability.openslo.yaml:26-28`.
108. Target O-47 demo_trial monthly uptime objective: best-effort but accepted operations still use same latency targets under cap.
109. Target O-47 tenant overlay: demo_trial is constrained by usage and profile, not degraded code path.
110. Target O-48 paid tenant scaling: automatic scale-out by paid usage budget and contractual SLO.
111. Target O-48 tenant overlay: paid permits BYOK and compliance packs when prerequisites pass.
112. Target O-49 paid revenue_share billing-component scaling: at-cost or zero-margin substrate with admission tied to gross-revenue basis and risk cap.
113. Target O-49 tenant overlay: paid tenants with a revenue_share component can use marketplace/deal-set settlement but must not bypass audit or SLO controls.
114. Target O-50 error budget burn alert: page on 2 percent budget burn per hour for latency-critical SLOs.
115. Target O-50 confidence: medium; repo has SLO files, but alert files need canonical label cleanup.

## §4 Comparison Narrative

1. File API: Figma is ahead on documented file/node/image endpoints; Oyatie is catch-up because OpenAPI and Rust routes disagree.
2. File open latency: Oyatie target is parity if implemented at 900 ms p95, because ADR already sets open-file latency intent.
3. Node graph: Figma is ahead; Oyatie has no comparable node graph.
4. API ceilings: Figma publishes request ceilings; Oyatie has no rate-limit contract and is catch-up.
5. Webhooks: Figma is ahead with public context counts and retry schedule; Oyatie is catch-up because AsyncAPI is generic and code-specific events diverge.
6. Dev handoff: Figma and Adobe XD are ahead; Oyatie has handoff intent but lacks route, schema, and export package semantics.
7. Export formats: Figma and Adobe XD define concrete formats/units; Oyatie target includes PNG, JPG, SVG, PDF, token JSON, px, pt, and dp.
8. Prototype sharing: Adobe XD and historical InVision are ahead on share-link workflow; Oyatie is catch-up because prototype-share is only a capability artifact.
9. Link access modes: Adobe XD is ahead with public/private/password; Oyatie target adds expiring signed links.
10. Link update preservation: Adobe XD is ahead; Oyatie target requires 100 percent preservation or explicit failure evidence.
11. Coediting: Figma and Adobe XD are ahead; Oyatie lacks edit-session, cursor, and presence models.
12. Comment resolution: Oyatie is partial; it can resolve comments but lacks full thread workflow.
13. Token promotion: Oyatie has a differentiating governance angle, but Figma variables remain broader.
14. Operation replay: Oyatie target can be ahead if ADR checkpoint cadence and replay p95 are implemented and verified.
15. InVision migration: Oyatie can be ahead only if it adds migration playbooks and export audit coverage.
16. Shutdown resilience: Oyatie target is ahead by requiring export audit coverage and deployable portability, but current artifacts do not yet prove it.
17. Adobe XD maintenance status: Oyatie can exceed future XD feature velocity by continuing development, but must still support migration of XD-era assets.
18. InVision discontinued status: live throughput is not the comparison; migration safety and data-retention assurance are the comparison.
19. Deployment portability: Oyatie target should beat all three counterparts by supporting six deployment contexts, but current IaC fails that gate.
20. OCI Always Free profile: no counterpart directly maps; Oyatie target is a unique demo_trial infrastructure profile and currently missing.
21. Tenant-class scaling: no target counterpart maps cleanly; Oyatie must define this as its own canonical product-control surface.
22. OS support: counterpart products are mostly SaaS/app-scoped; Oyatie requires explicit OS matrix and currently lacks it.
23. Paid tenant performance: Oyatie target is parity if managed contexts hit p95s and scale-out numbers above.
24. Demo-trial performance: Oyatie target is parity for accepted operations but catch-up on caps because cap policy is not modeled.
25. Paid revenue_share billing-component performance: Oyatie target is differentiated but unimplemented.
26. Current implementation readiness: not ready for benchmark claims beyond unit/integration behavior because routes cover only a subset.
27. Current SLO readiness: strong baseline targets exist, but labels and context overlays need cleanup.
28. Current benchmark readiness: blocked until contracts, tenant_class, context modules, and full use cases exist.
29. Final comparison status: Figma ahead, Adobe XD historical parity target, InVision migration target, Oyatie foundational but not yet industry-leader proven.
30. Required next proof: implement contract/source consistency tests, add missing deployment-context modules, then run workload benchmarks W1-W10 across at least local, guest-on-oci, and one managed context.

## §5 Benchmark Acceptance Ledger

1. Acceptance ledger B-001: W1 file-open benchmark is not accepted until OpenAPI and Rust adapter route paths agree.
2. Acceptance ledger B-002: W1 file-open benchmark must include medium and large artifacts because Figma file endpoints expose node subset and depth semantics.
3. Acceptance ledger B-003: W1 file-open benchmark must record p50, p95, p99, payload bytes, permission-decision latency, and audit emission latency.
4. Acceptance ledger B-004: W2 large-file benchmark must record cache hit state and referenced asset count.
5. Acceptance ledger B-005: W2 large-file benchmark must fail closed if artifact payload exceeds configured max payload from `src/config.rs:49-63`.
6. Acceptance ledger B-006: W3 comment-sync benchmark is not accepted until create, reply, resolve, reopen, and audit events are modeled.
7. Acceptance ledger B-007: W3 comment-sync benchmark must compare against the ADR comment p95 target at `decisions/ADR-DC-001-creative-artifact-operation-log-and-token-promotion-gate.md:220-227`.
8. Acceptance ledger B-008: W4 version-save benchmark is not accepted until version-save is implemented beyond SLO documentation.
9. Acceptance ledger B-009: W4 version-save benchmark must include checkpoint cadence validation against 500 operations or 5 minutes.
10. Acceptance ledger B-010: W5 token-promotion benchmark may run against current source because token promotion is implemented.
11. Acceptance ledger B-011: W5 token-promotion benchmark must include policy allow, policy deny, duplicate idempotency, and audit failure cases.
12. Acceptance ledger B-012: W6 prototype-share benchmark is not accepted until prototype-share route and link-access schema exist.
13. Acceptance ledger B-013: W6 prototype-share benchmark must include public, private invited, password, and expiring signed-link modes.
14. Acceptance ledger B-014: W7 handoff-export benchmark is not accepted until Web, iOS, Android, and asset-package outputs are modeled.
15. Acceptance ledger B-015: W7 handoff-export benchmark must record package size, asset count, export format count, and immutable evidence ID.
16. Acceptance ledger B-016: W8 asset-preview benchmark is not accepted until render cache state is modeled.
17. Acceptance ledger B-017: W8 asset-preview benchmark must record cache hit, cache miss, render queue time, and egress bytes.
18. Acceptance ledger B-018: W9 replay benchmark is accepted only when the operation log and checkpoint persistence path are wired.
19. Acceptance ledger B-019: W9 replay benchmark must record checkpoint load time, suffix operation count, replay time, and final artifact hash.
20. Acceptance ledger B-020: W10 policy-decision benchmark is accepted only after old-axis policy context is replaced with tenant_class and deployment_context.
21. Acceptance ledger B-021: every benchmark run must tag deployment_context.
22. Acceptance ledger B-022: every benchmark run must tag tenant_class.
23. Acceptance ledger B-023: every benchmark run must tag data_class.
24. Acceptance ledger B-024: every benchmark run must tag storage profile.
25. Acceptance ledger B-025: every benchmark run must tag cache profile.
26. Acceptance ledger B-026: every benchmark run must tag service version and contract version.
27. Acceptance ledger B-027: demo_trial benchmark runs must enforce usage caps before measuring latency.
28. Acceptance ledger B-028: paid benchmark runs must include scale-out and contractual SLO checks.
29. Acceptance ledger B-029: paid revenue_share billing-component benchmark runs must include cost attribution and admission-control decision.
30. Acceptance ledger B-030: oyatie-public-cloud benchmark runs must prove elasticity through at least one scale-up event.
31. Acceptance ledger B-031: guest-on-aws benchmark runs must record customer-account network and identity dependency timings.
32. Acceptance ledger B-032: guest-on-oci benchmark runs must separately report paid-capacity and OCI Always Free profile results.
33. Acceptance ledger B-033: on-prem benchmark runs must include facility hardware, storage, and network certification metadata.
34. Acceptance ledger B-034: colo benchmark runs must include facility latency and local-cache metadata.
35. Acceptance ledger B-035: oyatie-as-cloud-provider benchmark runs must include managed-cell placement evidence.
36. Acceptance ledger B-036: public-source counterpart numbers must remain labeled as source-backed only when a cited page publishes the number.
37. Acceptance ledger B-037: estimated counterpart numbers must remain labeled estimated and carry confidence.
38. Acceptance ledger B-038: no benchmark conclusion may claim current Oyatie parity until measured runs exist.
39. Acceptance ledger B-039: no benchmark conclusion may hide unsupported use cases behind aggregate averages.
40. Acceptance ledger B-040: headline dashboard must show each workload separately before any combined score is reported.
41. Acceptance ledger B-041: current p95 target acceptance fails if more than 1 percent of accepted operations exceed p95 in a 30-minute run.
42. Acceptance ledger B-042: current p99 target acceptance fails if more than 0.1 percent of accepted operations exceed p99 in a 30-minute run.
43. Acceptance ledger B-043: replay target acceptance fails if artifact hash differs after replay.
44. Acceptance ledger B-044: export target acceptance fails if evidence bundle cannot be verified independently.
45. Acceptance ledger B-045: webhook target acceptance fails if retry schedule and terminal failure state are not visible to operators.
46. Acceptance ledger B-046: link-update target acceptance fails if existing comments are lost without explicit migration failure record.
47. Acceptance ledger B-047: migration target acceptance fails if imported Adobe XD or InVision artifacts cannot be traced to original source IDs.
48. Acceptance ledger B-048: supported-OS acceptance fails until every supported OS row has build/test/package evidence.
49. Acceptance ledger B-049: deployment-context acceptance fails until every canonical context has an OpenTofu path or explicit N/A decision.
50. Acceptance ledger B-050: final benchmark-readiness state for this audit is blocked on implementation, context, and contract prerequisites, not on benchmark methodology.
