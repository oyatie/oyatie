# developer-sdk Performance Benchmark Numbers

- Date: 2026-05-20.
- Wave: 3.
- Batch: 3.2.
- Microservice: `developer-sdk`.
- Deliverable: 3 of 3.
- Target model: single industry-leader target set with deployment-context and tenant_class overlays.
- Counterparts: Stainless, Speakeasy, Fern.
- Excluded model: retired capability-ladder segmentation.
- Target tenant classes for this audit: `demo_trial`, `paid`, `revenue_share`.
- Target deployment contexts: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, `oyatie-as-cloud-provider`.

## Citation Anchor Block

1. Developer-sdk benchmark correction: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_developer_sdk_stainless_generator_2026_05_20.md:102-114`.
2. Benchmark structure correction: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16020-16032`.
3. Canonical performance disclosure rule: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4202-4228`.
4. Deployment and IaC constraints: `specs/master-plan-sequencing.json:704-775`.
5. OCI Always Free constraints: `specs/master-plan-sequencing.json:857-867` and `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md:10-80`.
6. Stainless public product/docs source: `https://www.stainless.com/products/sdks/` and `https://app.stainless.com/docs`.
7. Speakeasy public docs source: `https://www.speakeasy.com/docs/sdks/core-concepts` and `https://www.speakeasy.com/docs/sdks/customize/runtime/retries`.
8. Fern public product source: `https://buildwithfern.com/sdks`.
9. Current Oyatie benchmark drift: `performance-bench.md:19-40` and `benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md:1`.
10. Current Oyatie codegen plan evidence: `decisions/ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:76-78`, `:207-235`.

## Explicit Methodology Disclosure

1. Public counterpart docs for Stainless, Speakeasy, and Fern expose capability surfaces, supported languages, configuration concepts, generated docs, and release workflows.
2. The public docs inspected for this audit did not expose official p50, p95, p99, throughput, queue-depth, or package-publication benchmark datasets.
3. Every counterpart number in §2 is therefore marked as an estimate unless it is a direct public capability count.
4. Estimated numbers are planning baselines, not claims about private counterpart infrastructure.
5. Estimates use a fixed reference workload so the comparison is internally consistent.
6. Reference workload A: 100 REST endpoints.
7. Reference workload B: 250 REST endpoints.
8. Reference workload C: 1,000 REST endpoints.
9. Reference workload D: 100 event channels.
10. Reference workload E: 100 proto services/messages.
11. Reference schema density: 2.5 schemas per endpoint for REST workloads.
12. Reference output set for Oyatie targets: ten mandatory SDK families named by the developer-sdk directive.
13. Reference output set for counterpart estimates: each counterpart's public language list, mapped to comparable generator workload.
14. Reference docs workload: endpoint snippets, package README, usage examples, and changelog diff.
15. Reference fixture workload: compile fixture, auth fixture, pagination fixture, retry fixture, idempotency fixture, and serialization fixture per target language.
16. Reference publish workload: dry-run, signing/provenance generation, registry upload, registry visibility check, and rollback metadata.
17. Latency metrics separate synchronous API latency from asynchronous generator completion time.
18. API latency metrics measure control-plane requests such as create run, read status, fetch artifact metadata, and request publish.
19. Generation duration metrics measure queued worker time from admitted run to artifact bundle ready.
20. Publication duration metrics measure artifact ready to registry visibility proof.
21. Deployment-context overlays do not lower the canonical quality target.
22. Deployment-context overlays declare infrastructure ceilings, concurrency caps, or elasticity differences.
23. Tenant_class overlays do not lower generated SDK quality.
24. Tenant_class overlays declare usage caps, billing-scale controls, and SLO posture.
25. The OCI Always Free profile is modeled as a constrained demo-trial infrastructure profile.
26. On-prem and colo overlays are modeled as facility-specific capacity envelopes.
27. Oyatie-public-cloud and Oyatie-as-cloud-provider overlays are modeled as elastic managed contexts.
28. Guest-on-AWS and guest-on-OCI overlays are modeled as customer-account deployment contexts with policy and quota variability.
29. This report uses milliseconds for synchronous API latency.
30. This report uses seconds or minutes for generator and publication workflows.
31. This report uses runs per minute for asynchronous generation throughput.
32. This report uses packages per hour for publication throughput.
33. This report uses percent for success rates and fixture pass rates.
34. This report avoids capability-ladder rows and headings.
35. This report uses single canonical targets plus overlays.

## §1 Methodology

1. Benchmark dimension: contract ingestion latency.
2. Benchmark dimension: spec validation latency.
3. Benchmark dimension: generator run admission latency.
4. Benchmark dimension: generation queue wait.
5. Benchmark dimension: per-language generation duration.
6. Benchmark dimension: all-language fanout duration.
7. Benchmark dimension: compile fixture duration.
8. Benchmark dimension: runtime fixture duration.
9. Benchmark dimension: docs/snippet generation duration.
10. Benchmark dimension: changelog and diff duration.
11. Benchmark dimension: package signing duration.
12. Benchmark dimension: package publish duration.
13. Benchmark dimension: package visibility proof duration.
14. Benchmark dimension: rollback metadata duration.
15. Benchmark dimension: synchronous API p50 latency.
16. Benchmark dimension: synchronous API p95 latency.
17. Benchmark dimension: synchronous API p99 latency.
18. Benchmark dimension: generation throughput.
19. Benchmark dimension: concurrent generation runs.
20. Benchmark dimension: maximum spec size.
21. Benchmark dimension: maximum endpoint count.
22. Benchmark dimension: maximum schema count.
23. Benchmark dimension: maximum output-language fanout.
24. Benchmark dimension: generated artifact success rate.
25. Benchmark dimension: fixture pass rate.
26. Benchmark dimension: package publication success rate.
27. Benchmark dimension: reproducibility pass rate.
28. Benchmark dimension: provenance generation success rate.
29. Test workload small: 50 endpoints, 125 schemas, 2 SDK families, no event channels, no proto.
30. Test workload standard: 250 endpoints, 625 schemas, 10 SDK families, 25 event channels, 20 proto messages.
31. Test workload large: 1,000 endpoints, 2,500 schemas, 10 SDK families, 100 event channels, 100 proto messages.
32. Test workload pathological: 1,000 endpoints with deep recursive schemas, auth variants, pagination variants, streaming endpoints, and multipart endpoints.
33. Test workload publication fanout: ten package channels plus GitHub Releases and Homebrew where applicable.
34. Test workload rollback: yank/deprecate/rollback metadata update across all published package channels.
35. OS disclosure: runtime must be verified by `supported-oses.json`, which is currently absent in `microservices/developer-sdk/`.
36. OS target: Linux server rows for generator workers and control plane.
37. OS target: macOS Apple Silicon M5+ row for local generated SDK fixture support where applicable.
38. OS target: explicit out-of-scope rows for unsupported desktop or server OSes required by canonical policy.
39. Architecture disclosure: target architecture is `x86_64` and `aarch64` for server runtime, plus architecture-specific fixture rows for native generated SDKs.
40. Deployment context disclosure: all six contexts require OpenTofu evidence before performance claims are production-admissible.
41. Tenant_class disclosure: `demo_trial` caps usage, `paid` scales with contract and billing, and `revenue_share` runs at at-cost or zero-margin substrate under the prompt-level model.
42. Current Oyatie benchmark file drift: `performance-bench.md:19-40` benchmarks marketplace and payout surfaces, not a generator-first workload.
43. Current Oyatie benchmark file drift: `benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md:1` uses downstream SDKs rather than generator counterparts.
44. Current Oyatie useful seed: `ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:76-78` gives p95 45s and p99 90s per language as a historic codegen target.
45. Current Oyatie useful seed: `ADR-SDK-0002-codegen-pipeline-is-deterministic;-two-runs-on-identical-inp.md:207-235` defines deterministic and fixture-oriented verification.

## §2 Counterpart Numbers

### §2.1 Stainless Numbers

1. Stainless number 1: public supported language count observed in product/docs navigation is at least 10 including SDK, provider, and CLI-adjacent outputs; source: `https://www.stainless.com/products/sdks/`, public capability count.
2. Stainless number 2: comparable SDK-output language count for Oyatie target comparison is estimated at 8 mainstream SDK languages after excluding provider and CLI outputs; source: Stainless public language list, estimate.
3. Stainless number 3: estimated p50 control-plane API latency for generation run creation is 120 ms for a managed SaaS generator; source: planning estimate from public managed-SaaS posture.
4. Stainless number 4: estimated p95 control-plane API latency for generation run creation is 450 ms; source: planning estimate.
5. Stainless number 5: estimated p99 control-plane API latency for generation run creation is 900 ms; source: planning estimate.
6. Stainless number 6: estimated standard workload all-language generation duration is 8 minutes; source: estimate from OpenAPI-driven multi-language generation capability.
7. Stainless number 7: estimated large workload all-language generation duration is 22 minutes; source: estimate from 1,000-endpoint reference workload.
8. Stainless number 8: estimated compile fixture pass-rate target for generated SDK releases is 99.5%; source: estimate from compile-quality public positioning.
9. Stainless number 9: estimated package publication visibility duration is 8 minutes for registry fanout; source: estimate from publication workflow.
10. Stainless number 10: estimated generated docs/snippet refresh duration is 4 minutes for standard workload; source: estimate from generated docs capability.
11. Stainless number 11: estimated supported endpoint ceiling for a single spec is 1,000 endpoints; source: planning estimate, not public official ceiling.
12. Stainless number 12: estimated concurrent generation runs per organization is 10; source: planning estimate, not public official quota.

### §2.2 Speakeasy Numbers

1. Speakeasy number 1: public supported output list observed includes TypeScript, Python, Go, Java, C#, PHP, Ruby, Swift, and Terraform; source: `https://www.speakeasy.com/docs/sdks/core-concepts`, public capability count.
2. Speakeasy number 2: comparable SDK-output language count for Oyatie target comparison is estimated at 7 after excluding provider-style output and Oyatie-excluded language families; source: Speakeasy public language list, estimate.
3. Speakeasy number 3: estimated p50 generation-run admission latency is 150 ms; source: planning estimate from workflow-driven generator posture.
4. Speakeasy number 4: estimated p95 generation-run admission latency is 500 ms; source: planning estimate.
5. Speakeasy number 5: estimated p99 generation-run admission latency is 1,000 ms; source: planning estimate.
6. Speakeasy number 6: estimated standard workload all-language generation duration is 10 minutes; source: estimate from workflow-driven multi-language generation.
7. Speakeasy number 7: estimated large workload all-language generation duration is 25 minutes; source: estimate from 1,000-endpoint reference workload.
8. Speakeasy number 8: retry customization supports strategy, status codes, connection-error policy, and backoff configuration; source: `https://www.speakeasy.com/docs/sdks/customize/runtime/retries`, public capability count of 4 retry knobs.
9. Speakeasy number 9: estimated compile fixture pass-rate target for generated releases is 99.0%; source: planning estimate.
10. Speakeasy number 10: estimated package publication visibility duration is 10 minutes; source: planning estimate.
11. Speakeasy number 11: estimated generated docs/snippet refresh duration is 5 minutes for standard workload; source: planning estimate.
12. Speakeasy number 12: estimated concurrent generation runs per organization is 8; source: planning estimate, not public official quota.

### §2.3 Fern Numbers

1. Fern number 1: public SDK page says SDKs are generated in more than ten languages; source: `https://buildwithfern.com/sdks`, public capability count.
2. Fern number 2: public SDK matrix includes TypeScript, Python, Java, Go, C#, PHP, Ruby, Kotlin, Swift, and Rust; source: `https://buildwithfern.com/sdks`, public capability count of 10 listed SDK languages.
3. Fern number 3: comparable SDK-output language count for Oyatie target comparison is 8 after excluding Oyatie-excluded PHP and Ruby, before adding Oyatie-required C and C++; source: Fern public matrix plus Oyatie directive.
4. Fern number 4: estimated p50 generation-run admission latency is 130 ms; source: planning estimate from managed generator posture.
5. Fern number 5: estimated p95 generation-run admission latency is 475 ms; source: planning estimate.
6. Fern number 6: estimated p99 generation-run admission latency is 950 ms; source: planning estimate.
7. Fern number 7: estimated standard workload all-language generation duration is 9 minutes; source: estimate from multi-language generation and docs integration.
8. Fern number 8: estimated large workload all-language generation duration is 23 minutes; source: estimate from 1,000-endpoint reference workload.
9. Fern number 9: public feature matrix includes in-memory mock server support for several languages; source: `https://buildwithfern.com/sdks`, public capability.
10. Fern number 10: public feature matrix includes auto-pagination, OAuth2, websocket, error discrimination, webhook verification, and custom code features across language rows; source: `https://buildwithfern.com/sdks`, public capability cluster.
11. Fern number 11: estimated compile fixture pass-rate target for generated releases is 99.5%; source: planning estimate.
12. Fern number 12: estimated package publication visibility duration is 8 minutes; source: planning estimate.
13. Fern number 13: estimated generated docs/snippet refresh duration is 4 minutes for standard workload; source: planning estimate from docs-plus-SDK positioning.
14. Fern number 14: estimated concurrent generation runs per organization is 10; source: planning estimate, not public official quota.

## §3 Oyatie Target Numbers — Single Industry-Leader Target Set

1. Metric: create generator run API latency p50.
2. Canonical target: 75 ms.
3. Deployment overlay: `oyatie-public-cloud` target 75 ms with elastic control plane.
4. Deployment overlay: `guest-on-aws` target 100 ms subject to customer-region quota and network policy.
5. Deployment overlay: `guest-on-oci` target 100 ms, with OCI Always Free profile capped separately.
6. Deployment overlay: `on-prem` target 150 ms when customer facility latency is within declared envelope.
7. Deployment overlay: `colo` target 125 ms when colocated registry egress is provisioned.
8. Deployment overlay: `oyatie-as-cloud-provider` target 75 ms with provider-native routing.
9. Tenant_class overlay: `demo_trial` uses same latency target until usage caps reject work.
10. Tenant_class overlay: `paid` uses target under contractual SLO.
11. Tenant_class overlay: `revenue_share` uses target when at-cost substrate has provisioned capacity.
12. Metric: create generator run API latency p95.
13. Canonical target: 250 ms.
14. Deployment overlay: OCI Always Free profile p95 cap is 500 ms under demo-trial concurrency.
15. Tenant_class overlay: `demo_trial` rejects above cap rather than degrading generated SDK quality.
16. Metric: create generator run API latency p99.
17. Canonical target: 500 ms.
18. Deployment overlay: on-prem and colo may declare site-specific p99 up to 900 ms with facility evidence.
19. Tenant_class overlay: `paid` and `revenue_share` require contractual evidence for any higher p99.
20. Metric: spec validation p50 for standard workload.
21. Canonical target: 500 ms.
22. Deployment overlay: all elastic contexts target 500 ms.
23. Deployment overlay: OCI Always Free profile target 1,500 ms for standard workload.
24. Tenant_class overlay: `demo_trial` may cap maximum spec size before validation starts.
25. Metric: spec validation p95 for standard workload.
26. Canonical target: 2 seconds.
27. Deployment overlay: on-prem and colo target 3 seconds if local storage latency is declared.
28. Tenant_class overlay: all classes receive same validation rules.
29. Metric: spec validation p99 for large workload.
30. Canonical target: 10 seconds.
31. Deployment overlay: OCI Always Free profile does not accept large workload by default.
32. Tenant_class overlay: `paid` and `revenue_share` can buy or allocate larger validation envelopes.
33. Metric: standard workload all-language generation duration p50.
34. Canonical target: 5 minutes.
35. Deployment overlay: `oyatie-public-cloud` target 5 minutes.
36. Deployment overlay: `guest-on-aws` target 7 minutes if customer account permits worker scale.
37. Deployment overlay: `guest-on-oci` target 7 minutes outside Always Free.
38. Deployment overlay: `on-prem` target 10 minutes unless facility has declared worker pool.
39. Deployment overlay: `colo` target 8 minutes with local build-cache.
40. Deployment overlay: `oyatie-as-cloud-provider` target 5 minutes.
41. Tenant_class overlay: `demo_trial` standard workload is capped to 2 SDK families unless the profile is explicitly enlarged.
42. Tenant_class overlay: `paid` can run all ten families.
43. Tenant_class overlay: `revenue_share` can run all ten families when usage economics are approved.
44. Metric: standard workload all-language generation duration p95.
45. Canonical target: 12 minutes.
46. Deployment overlay: OCI Always Free profile cap is 20 minutes for reduced fanout.
47. Tenant_class overlay: `demo_trial` receives reduced fanout, not lower quality outputs.
48. Metric: large workload all-language generation duration p50.
49. Canonical target: 18 minutes.
50. Deployment overlay: elastic contexts target 18 minutes.
51. Deployment overlay: on-prem target 30 minutes unless facility worker count is certified.
52. Tenant_class overlay: large workload is unavailable to default `demo_trial`.
53. Metric: large workload all-language generation duration p95.
54. Canonical target: 35 minutes.
55. Deployment overlay: guest contexts require quota evidence before accepting this SLO.
56. Tenant_class overlay: `paid` and `revenue_share` use provisioned capacity gates.
57. Metric: pathological workload admission.
58. Canonical target: accepted only with preflight warnings and expected-cost estimate within 30 seconds.
59. Deployment overlay: OCI Always Free profile rejects pathological workload with a typed limit response.
60. Tenant_class overlay: `demo_trial` receives deterministic refusal with upgrade or usage-path explanation.
61. Metric: per-language compile fixture p50.
62. Canonical target: 90 seconds per language.
63. Deployment overlay: native-heavy languages may run on dedicated fixture workers in elastic contexts.
64. Tenant_class overlay: same compile gate across all tenant classes.
65. Metric: per-language compile fixture p95.
66. Canonical target: 4 minutes per language.
67. Deployment overlay: on-prem and colo can publish local fixture-worker class in `supported-oses.json`.
68. Tenant_class overlay: no tenant class can bypass compile fixture gate for published output.
69. Metric: all-language fixture suite p50.
70. Canonical target: 8 minutes.
71. Deployment overlay: OCI Always Free profile default target is 15 minutes for two-language fanout.
72. Tenant_class overlay: `demo_trial` may receive delayed queue, not reduced validation.
73. Metric: all-language fixture suite p95.
74. Canonical target: 18 minutes.
75. Deployment overlay: elastic contexts target 18 minutes with warm caches.
76. Tenant_class overlay: paid and revenue-share workloads scale by purchased or allocated worker pool.
77. Metric: generated docs/snippet generation p50.
78. Canonical target: 60 seconds for standard workload.
79. Deployment overlay: all contexts target 60 seconds when object storage and cache are healthy.
80. Tenant_class overlay: same docs correctness across all classes.
81. Metric: generated docs/snippet generation p95.
82. Canonical target: 3 minutes.
83. Deployment overlay: OCI Always Free profile target 6 minutes.
84. Tenant_class overlay: demo-trial caps publish frequency rather than doc quality.
85. Metric: changelog/diff generation p50.
86. Canonical target: 30 seconds.
87. Deployment overlay: all contexts target 30 seconds for standard workload.
88. Tenant_class overlay: all classes receive same breaking-change classification.
89. Metric: package signing and provenance p50.
90. Canonical target: 45 seconds per language package.
91. Deployment overlay: offline on-prem signing target requires local key ceremony evidence.
92. Tenant_class overlay: no class can publish unsigned artifacts.
93. Metric: package publication p50 for ten-family release.
94. Canonical target: 10 minutes to submitted registry uploads.
95. Deployment overlay: external registry latency can move visibility proof, but upload submission target remains 10 minutes.
96. Tenant_class overlay: demo-trial publication to public registries is disabled unless explicitly allowed.
97. Metric: package visibility proof p95.
98. Canonical target: 30 minutes across public registries.
99. Deployment overlay: on-prem private registries target 10 minutes if registry is local.
100. Tenant_class overlay: paid and revenue-share public registry publication require configured credentials and billing authority.
101. Metric: rollback metadata p50.
102. Canonical target: 2 minutes after operator approval.
103. Deployment overlay: public registry rollback depends on registry semantics but local metadata target remains 2 minutes.
104. Tenant_class overlay: all classes use same safety path for already-published artifacts.
105. Metric: reproducibility pass rate.
106. Canonical target: 99.99% for two-run deterministic generation on unchanged inputs.
107. Deployment overlay: all contexts must meet the same reproducibility target.
108. Tenant_class overlay: no tenant class receives non-reproducible generator output.
109. Metric: compile fixture pass rate.
110. Canonical target: 99.9% for publishable SDK artifacts.
111. Deployment overlay: context-specific worker failures count against context reliability.
112. Tenant_class overlay: same quality across all classes.
113. Metric: package publication success rate.
114. Canonical target: 99.5% excluding external registry outage windows.
115. Deployment overlay: private registry contexts must disclose local registry SLO.
116. Tenant_class overlay: demo-trial can be disabled for public publication without lowering generation quality.
117. Metric: concurrent generation runs.
118. Canonical target: 1,000 active queued or running jobs per managed region.
119. Deployment overlay: OCI Always Free profile target is 2 active jobs.
120. Deployment overlay: on-prem target is facility-declared, with minimum 10 active jobs for production support.
121. Tenant_class overlay: demo-trial default is 1 active job, paid scales by entitlement, revenue-share scales by approved cost envelope.
122. Metric: generation throughput.
123. Canonical target: 60 standard all-language runs per minute per managed region after warm-up.
124. Deployment overlay: guest contexts target 10 standard runs per minute unless customer account quotas allow more.
125. Tenant_class overlay: demo-trial default target is 6 reduced-fanout runs per hour.
126. Metric: maximum spec size.
127. Canonical target: 50 MB contract bundle after compression.
128. Deployment overlay: OCI Always Free profile default is 5 MB.
129. Tenant_class overlay: paid and revenue-share can request larger bundles through cost control.
130. Metric: maximum endpoint count.
131. Canonical target: 2,000 endpoints.
132. Deployment overlay: OCI Always Free profile default is 100 endpoints.
133. Tenant_class overlay: demo-trial caps endpoint count while preserving generated output correctness.
134. Metric: maximum schema count.
135. Canonical target: 5,000 schemas.
136. Deployment overlay: on-prem and colo require memory evidence for this ceiling.
137. Tenant_class overlay: paid and revenue-share can raise limits through contract.
138. Metric: maximum output-language fanout.
139. Canonical target: ten required SDK families in one run.
140. Deployment overlay: OCI Always Free profile default is two SDK families in one run.
141. Tenant_class overlay: demo-trial caps fanout, paid and revenue-share allow full fanout.
142. Metric: control-plane read throughput.
143. Canonical target: 20,000 status/artifact metadata reads per second per managed region.
144. Deployment overlay: guest contexts target 2,000 reads per second unless provisioned higher.
145. Tenant_class overlay: demo-trial reads are rate-limited after caps.
146. Metric: artifact download throughput.
147. Canonical target: 10 Gbps aggregate per managed region for generated artifacts.
148. Deployment overlay: on-prem and colo target equals facility egress declaration.
149. Tenant_class overlay: paid and revenue-share can buy higher egress; demo-trial has hard caps.
150. Metric: cache hit rate.
151. Canonical target: 85% for repeated generation of unchanged contract/configuration pairs.
152. Deployment overlay: OCI Always Free profile target is 60% because cache size is constrained.
153. Tenant_class overlay: cache correctness is uniform across classes.
154. Metric: queue fairness.
155. Canonical target: no tenant may consume more than declared worker-share for more than 60 seconds.
156. Deployment overlay: all contexts require fairness enforcement.
157. Tenant_class overlay: demo-trial share is capped, paid share follows entitlement, revenue-share share follows cost envelope.

## §4 Comparison Narrative

1. Headline: Oyatie's p50 generator-run API target of 75 ms is ahead of the estimated counterpart planning band of 120-150 ms.
2. Evidence: counterpart API numbers are estimates because no public official benchmark dataset was found in the inspected docs.
3. Risk: this target is only credible after the service has a Rust control-plane implementation and real context tests.
4. Headline: Oyatie's p95 API target of 250 ms is ahead of the estimated 450-500 ms counterpart band.
5. Risk: guest and facility contexts need overlay disclosures before this can be claimed outside managed contexts.
6. Headline: Oyatie's standard all-language generation p50 target of 5 minutes is ahead of the estimated 8-10 minute counterpart band.
7. Risk: this is aggressive because Oyatie's target fanout includes ten mandatory SDK families and two additional native-family outputs not visible in all counterpart public matrices.
8. Headline: Oyatie's standard all-language generation p95 target of 12 minutes is ahead of the estimated counterpart planning band.
9. Risk: this requires warm build caches, deterministic templates, parallel workers, and per-language fixture partitioning.
10. Headline: Oyatie's large workload p50 target of 18 minutes is ahead of the estimated 22-25 minute counterpart band.
11. Risk: the large workload target should not be published until 1,000-endpoint fixture data exists.
12. Headline: Oyatie's large workload p95 target of 35 minutes is parity-to-ahead against the planning estimates.
13. Risk: customer-owned AWS or OCI quotas can make guest context overlays slower.
14. Headline: Oyatie's compile fixture p95 target of 4 minutes per language is a quality-preserving gate.
15. Risk: native families and JVM/.NET ecosystems require OS and cache disclosure before measurement.
16. Headline: Oyatie's reproducibility target of 99.99% is ahead of generic counterpart public positioning because it turns deterministic regeneration into a numeric SLO.
17. Risk: the current repo has ADR intent but no implementation evidence for this rate.
18. Headline: Oyatie's package publication p50 target of 10 minutes is parity with the estimated counterpart publication band.
19. Risk: public registry visibility can exceed internal submission targets, so visibility proof is separately targeted at p95 30 minutes.
20. Headline: Oyatie's maximum endpoint target of 2,000 endpoints is above the reference large workload and should be treated as a scale ceiling.
21. Risk: recursive schemas and multipart/streaming endpoints may be harder than raw endpoint count suggests.
22. Headline: Oyatie's maximum language fanout target of ten families meets the developer-sdk directive.
23. Risk: current contracts and plans show only six families, so this is a catch-up target.
24. Headline: OCI Always Free profile caps do not reduce quality; they reduce accepted workload size, fanout, concurrency, and publication privileges.
25. Risk: no `iac/oci-guest/always-free/` directory currently exists, so the profile remains a documentation target.
26. Headline: `demo_trial` caps usage and public publication but must not receive lower-quality generated SDKs.
27. Risk: current docs have no `demo_trial` contract.
28. Headline: `paid` uses full target set with contractual SLO and scalable capacity.
29. Risk: current docs mention paid billing components rather than a `paid` tenant_class.
30. Headline: `revenue_share` uses the same targets when at-cost or zero-margin substrate is approved.
31. Risk: current docs expose revenue share as payout ledger kind, not tenant class.
32. Headline: Oyatie-public-cloud and Oyatie-as-cloud-provider are the natural contexts for the strongest latency and throughput targets.
33. Risk: they still require OpenTofu modules and live measurements.
34. Headline: guest-on-AWS and guest-on-OCI can meet the same product quality but need quota-aware overlays.
35. Risk: customer account limits and private networking can dominate runtime.
36. Headline: on-prem and colo can meet generator correctness targets but need facility-specific performance disclosures.
37. Risk: local registry, storage, and worker sizing vary.
38. Headline: current Oyatie benchmark docs are not sufficient evidence for any of these targets.
39. Evidence: `performance-bench.md:19-40` benchmarks catalog, vetting, sandbox, payout, and older marketplace baselines.
40. Evidence: `benchmarks/developer-sdk-vs-stripe-twilio-auth0-aws-sdk-v3.md:1` compares downstream SDKs rather than generator platforms.
41. Required next measurement: implement a Rust benchmark harness for contract ingestion, validation, generation, fixtures, docs, package publication, and rollback metadata.
42. Required next measurement: collect measurements separately for small, standard, large, and pathological workloads.
43. Required next measurement: collect per-context overlays only after OpenTofu context modules exist.
44. Required next measurement: collect per-tenant_class quota behavior after tenant-class contract exists.
45. Required next measurement: publish measurement evidence as generator-run artifacts, not as line-count documentation.
46. Final benchmark posture: ahead on target ambition, catch-up on current artifact evidence, blocked on implementation measurement and canonical IaC/tenant/OS manifests.
