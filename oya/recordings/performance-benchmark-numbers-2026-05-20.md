# recordings performance benchmark numbers - 2026-05-20

- µservice: `recordings`
- Deliverable status: substantive audit deliverable 3 of 3.
- Counterparts: Zoom Cloud Recording / Gong.io / Otter.ai.
- Benchmark posture: one industry-leader target set with deployment-context and tenant-class overlays.
- Tier-retirement note: this benchmark intentionally avoids retired feature-tier segmentation.
- Methodology disclosure: public vendors rarely publish p50/p95/p99 for recording workflows, so this document separates vendor-published hard limits from estimate-based comparison numbers.
- Public source anchor 1: Zoom cloud recording support, `https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0062627&trk=s-bl`.
- Public source anchor 2: Zoom recording storage capacity, `https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0067670`.
- Public source anchor 3: Zoom AI Companion recording features, `https://library.zoom.com/zoom-workplace/artificial-intelligence/artificial-intelligence-bluepaper/ai-companion/ai-companion-features/zoom-recordings`.
- Public source anchor 4: Gong call recording help, `https://help.gong.io/docs/how-to-record-calls`.
- Public source anchor 5: Otter import and basic-plan limits, `https://help.otter.ai/hc/en-us/articles/360047733574-Import-an-audio-or-video-file` and `https://help.otter.ai/hc/en-us/articles/360047538094-Conversation-import-and-app-limits-on-the-Basic-free-plan`.
- Local benchmark anchor: `benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md`.
- Local capacity anchor: `capacity-model.md`.
- Local SLO anchors: `slos/playback-start-p99.openslo.yaml`, `slos/transcript-search-p99.openslo.yaml`, `slos/transcript-render-p99.openslo.yaml`, `slos/redaction-render-p99.openslo.yaml`, `slos/ediscovery-export-mp4-p99.openslo.yaml`, and `slos/legal-hold-engagement-p99.openslo.yaml`.

## §1 Methodology

1. Benchmark dimension: capture start and recording ingestion.
2. Benchmark dimension: media processing completion.
3. Benchmark dimension: transcript processing completion.
4. Benchmark dimension: transcript render latency.
5. Benchmark dimension: transcript search latency.
6. Benchmark dimension: playback start latency.
7. Benchmark dimension: redaction render latency.
8. Benchmark dimension: legal-hold engagement latency.
9. Benchmark dimension: eDiscovery package generation.
10. Benchmark dimension: storage quota and retained media scale.
11. Benchmark dimension: concurrent playback sessions.
12. Benchmark dimension: concurrent transcription workers.
13. Benchmark dimension: import file size and duration caps.
14. Benchmark dimension: long-call handling.
15. Benchmark dimension: AI-derived artifact generation.
16. Test workload A: 30-minute meeting, 3 speakers, 720p video, shared-screen segment, chat transcript, English transcript.
17. Test workload B: 60-minute meeting, 8 speakers, 1080p video, mixed screen-share and camera layout, chat transcript, transcript search workload.
18. Test workload C: 6-hour long-call workload, 10 speakers, uploaded or captured as continuous recording, chunked for processing where required.
19. Test workload D: 1,000-hour tenant archive search workload aligned to local SLO `transcript-search-p99.openslo.yaml:5-17`.
20. Test workload E: eDiscovery export bundle with MP4, transcript PDF, redaction manifest, legal-hold manifest, and audit-chain hashes.
21. OS disclosure: current service-local artifacts do not include `supported-oses.json`, so Oyatie targets assume canonical OS matrix rather than service-local proof.
22. Architecture disclosure: targets assume Rust backend and Kubernetes/cloud-native runtime per `manifest.json:233-236` and canonical language policy.
23. Deployment context disclosure: targets are stated for six contexts: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.
24. OCI disclosure: demo-trial infrastructure is assumed to fit within an OCI Always Free profile, but current service path lacks `iac/oci-guest/always-free/`.
25. Tenant-class disclosure: targets distinguish `demo_trial`, `paid`, and `revenue_share` only as quota/commercial overlays, not quality tiers.
26. Source classification: "vendor-published" means the number appears in public vendor docs reviewed during this audit.
27. Source classification: "local SLO" means the number appears in current recordings SLO files.
28. Source classification: "local capacity model" means the number appears in `capacity-model.md`.
29. Source classification: "estimate" means a derived planning number based on local target docs, public hard limits, and standard media-processing behavior.
30. Estimate caveat: vendor p99 performance is often not public; an estimate is not a claim about measured vendor production latency.
31. Comparison labels: "ahead" means Oyatie target is materially stronger than available counterpart number or public hard limit.
32. Comparison labels: "parity" means Oyatie target matches the useful public counterpart behavior.
33. Comparison labels: "catch-up" means current Oyatie artifacts need explicit feature/control work to match counterpart.
34. Comparison labels: "needs measurement" means the target is defined but should be validated by load testing before launch.
35. Validation gap: local test code was not found under `microservices/recordings/src/`, so these are benchmark targets, not measured current implementation results.
36. Anti-placeholder note: when a vendor does not publish p99, this document says "not publicly published" and supplies a planning estimate where useful.
37. Retired segmentation note: prior local benchmark `benchmarks/recordings-vs-zoom-vs-stream-vs-otter.md:19-30` used retired segmentation; this document replaces that shape.
38. Required future harness: ingest generator, media fixture store, transcript search load generator, playback CDN warm/cold test, export packager load test, and legal-hold correctness verifier.
39. Required future reporting: publish p50/p95/p99, error rate, saturation point, cost per 1,000 media hours, and per-context bottleneck.
40. Stop condition for this document: define a single numeric target set precise enough for implementation planning and later benchmark harness validation.

## §2 Counterpart numbers

### §2.1 Zoom Cloud Recording numbers

1. Zoom number Z-01: Pro included cloud recording storage is 10 GB per licensed user, source: Zoom storage capacity `KB0067670:17-20`.
2. Zoom number Z-02: Business included cloud recording storage is 10 GB per licensed user, source: Zoom storage capacity `KB0067670:17-20`.
3. Zoom number Z-03: Business Plus included cloud recording storage is 15 GB per licensed user, source: Zoom storage capacity `KB0067670:21`.
4. Zoom number Z-04: Enterprise included cloud recording storage is unlimited in the public table, source: Zoom storage capacity `KB0067670:22-23`.
5. Zoom number Z-05: Education Core or Legacy Site included meeting recording storage is 0.5 GB per licensed user, source: Zoom storage capacity `KB0067670:24`.
6. Zoom number Z-06: billing admin alert threshold is 80 percent of subscribed storage, source: Zoom storage capacity `KB0067670:2-4`.
7. Zoom number Z-07: live-session cloud recording file generation limit is 150 files, source: Zoom cloud recording `KB0062627:14-18`.
8. Zoom number Z-08: cloud recording can be started by hosts and co-hosts, qualitative source: Zoom cloud recording `KB0062627:27-37`.
9. Zoom number Z-09: processing completion latency is not publicly published in the reviewed support article; planning estimate for normal 30-60 minute meetings is media-duration x 0.3 to x 1.0, source: estimate from common cloud-recording processing and local target comparison.
10. Zoom number Z-10: playback start p99 is not publicly published; planning estimate for warm browser playback is under 1 second, source: estimate from cloud streaming UX expectation.
11. Zoom number Z-11: transcript render p99 is not publicly published; planning estimate for ready transcript view is under 1 second, source: estimate from transcript web UX expectation.
12. Zoom number Z-12: Smart Recording outputs chapters, summaries, highlights, speaker insights, and next steps, source: Zoom AI Companion recording features `zoom-recordings:108-129`.
13. Zoom number Z-13: Zoom Voice Recorder creates an original audio file, searchable transcript with speaker labels, and AI-generated summary, source: Zoom AI Companion recording features `zoom-recordings:122-129`.
14. Zoom number Z-14: cloud recordings cannot be embedded on websites, source: Zoom cloud recording `KB0062627:14-18`.
15. Zoom number Z-15: recording layouts include active speaker, gallery view, and shared screen, source: Zoom cloud recording `KB0062627:2-3`.

### §2.2 Gong.io numbers

1. Gong number G-01: Gong can process calls up to 6 hours long, source: Gong help `how-to-record-calls:130-141`.
2. Gong number G-02: ad hoc Gong homepage recording may take up to 10 minutes to start, source: Gong help `how-to-record-calls:70-88`.
3. Gong number G-03: valid web conferencing URL is required for scheduled recording, source: Gong help `how-to-record-calls:53`.
4. Gong number G-04: consent settings can block recording when consent is enabled or enforced, source: Gong help `how-to-record-calls:55` and `understanding-call-recording:67-71`.
5. Gong number G-05: native recording is available with Zoom when Native Zoom recording is enabled, source: Gong help `understanding-call-recording:73-85`.
6. Gong number G-06: assistant recording adds a virtual participant when native recording is not available, source: Gong help `understanding-call-recording:86-90`.
7. Gong number G-07: Gong captures automatic call recording and transcription, source: Gong conversation intelligence `conversation-intelligence:32-35`.
8. Gong number G-08: Gong includes keyword/topic detection, sentiment, talk-ratio, deal/pipeline tracking, and CRM integrations, source: Gong conversation intelligence `conversation-intelligence:34-38`.
9. Gong number G-09: Gong indexes recordings/transcripts by rep, account, deal stage, and keyword, source: Gong call recording page `call-recording-software:99-106`.
10. Gong number G-10: numeric redaction and PHI redaction are available for call recordings and transcripts, source: Gong redaction help `redact-sensitive-information:21-27`.
11. Gong number G-11: redaction removes data permanently and cannot be recovered by Gong, source: Gong redaction help `redact-sensitive-information:21-27`.
12. Gong number G-12: recording start p99 for scheduled calls is not publicly published; planning estimate is under 60 seconds for scheduled/native integrations, source: estimate based on automated calendar recording behavior.
13. Gong number G-13: transcript availability p99 is not publicly published; planning estimate is media-duration x 0.5 to x 1.5 for long revenue calls, source: estimate.
14. Gong number G-14: search p99 is not publicly published; planning estimate for indexed calls is under 1 second for common filters, source: estimate from SaaS indexed-search UX.
15. Gong number G-15: long-call handling requires splitting calls over 6 hours into shorter recordings, source: Gong help `how-to-record-calls:130-141`.

### §2.3 Otter.ai numbers

1. Otter number O-01: imported audio/video files must be less than 5 GB, source: Otter import help `Import an audio or video file:33-34`.
2. Otter number O-02: Otter supports AAC, MP3, M4A, WAV, WMA, and OGG audio imports, source: Otter import help `Import an audio or video file:36-44`.
3. Otter number O-03: Otter supports AVI, MOV, MPEG, MP4, WMV, MPG, MKV, M4P, and 3GP video imports, source: Otter import help `Import an audio or video file:46-56`.
4. Otter number O-04: Basic free plan transcription limit is 300 minutes per month, source: Otter basic limits `Conversation, import, and app limits:34-42`.
5. Otter number O-05: Basic free plan transcription duration limit is 30 minutes per conversation/import, source: Otter basic limits `Conversation, import, and app limits:45-47`.
6. Otter number O-06: Basic free plan import limit is three audio/video files per account, source: Otter basic limits `Conversation, import, and app limits:50-52`.
7. Otter number O-07: Basic free plan visible conversation history is 25 most recent conversations, source: Otter basic limits `Conversation, import, and app limits:53-55`.
8. Otter number O-08: Business plan public help text mentions up to 6,000 import minutes per month, source: Otter basic limits `Conversation, import, and app limits:50-52`.
9. Otter number O-09: OtterPilot auto-joins meetings through calendar connection and creates live notes, source: OtterPilot blog `OtterPilot:87-96`.
10. Otter number O-10: OtterPilot captures slide images during meetings, source: OtterPilot blog `OtterPilot:98-100`.
11. Otter number O-11: OtterPilot automated summaries include links to key moments and slide captures, source: OtterPilot blog `OtterPilot:101-105`.
12. Otter number O-12: Otter AI Chat can be used during or after a meeting, source: Otter features `Otter.ai features:45-52`.
13. Otter number O-13: upload processing time is not publicly published for all plans; planning estimate is media-duration x 0.3 to x 1.0 for common files, source: estimate based on imported-file transcription behavior.
14. Otter number O-14: live transcript latency is not publicly published; planning estimate is 1-5 seconds for live captions, source: estimate.
15. Otter number O-15: search/chat response latency is not publicly published; planning estimate is under 2 seconds for indexed meeting history interactions, source: estimate from AI meeting assistant UX.

## §3 Oyatie target numbers - single industry-leader target set

### §3.1 Canonical target set

1. Target T-01 capture-start accepted p50: 150 ms from source event receipt to ingest acknowledgment.
2. Target T-02 capture-start accepted p95: 500 ms.
3. Target T-03 capture-start accepted p99: 1,000 ms.
4. Target T-04 upload ingest throughput: 2,000 concurrent uploads per region in elastic public-cloud contexts.
5. Target T-05 source-event ingest throughput: 10,000 recording lifecycle events per second per region.
6. Target T-06 normal media processing completion: media-duration x 0.25 p50 for 30-60 minute recordings.
7. Target T-07 normal media processing completion: media-duration x 0.50 p95.
8. Target T-08 normal media processing completion: media-duration x 0.75 p99.
9. Target T-09 long-recording chunk threshold: chunk any continuous recording over 6 hours into policy-controlled segments before downstream export.
10. Target T-10 maximum single import file size canonical target: 10 GB for paid and revenue-share tenants when storage policy permits.
11. Target T-11 demo-trial single import file size cap: 2 GB under OCI Always Free profile.
12. Target T-12 transcript availability p50: media-duration x 0.20 for 30-60 minute English recordings.
13. Target T-13 transcript availability p95: media-duration x 0.35.
14. Target T-14 transcript availability p99: media-duration x 0.50.
15. Target T-15 live transcript interim latency p50: 1.0 second.
16. Target T-16 live transcript interim latency p95: 2.0 seconds.
17. Target T-17 live transcript interim latency p99: 4.0 seconds.
18. Target T-18 transcript render p99: 500 ms, source: current local SLO `slos/transcript-render-p99.openslo.yaml:5-15`.
19. Target T-19 transcript search p99: 300 ms across a 1,000-hour archive, source: current local SLO `slos/transcript-search-p99.openslo.yaml:5-17`.
20. Target T-20 transcript search p95: 150 ms across a 1,000-hour archive.
21. Target T-21 recording list p99: 200 ms, source: current local SLO `slos/recording-list-p99.openslo.yaml:5-16`.
22. Target T-22 playback warm start p50: 120 ms.
23. Target T-23 playback warm start p95: 250 ms.
24. Target T-24 playback warm start p99: 400 ms, source: current local SLO `slos/playback-start-p99.openslo.yaml:5-16`.
25. Target T-25 playback cold start p99: 1,000 ms, source: current local SLO `slos/playback-start-p99.openslo.yaml:5-16`.
26. Target T-26 concurrent playback canonical target: 200,000 sessions per large public-cloud region, source baseline: `capacity-model.md:17-28`.
27. Target T-27 baseline concurrent playback: 5,000 sessions, source: `capacity-model.md:17-28`.
28. Target T-28 daily recording baseline: 50,000 recordings per day, source: `capacity-model.md:17-28`.
29. Target T-29 daily recording maximum planning ceiling: 1,000,000 recordings per day, source: `capacity-model.md:17-28`.
30. Target T-30 media-hour baseline: 10,000 media hours per day, source: `capacity-model.md:17-28`.
31. Target T-31 media-hour maximum planning ceiling: 200,000 media hours per day, source: `capacity-model.md:17-28`.
32. Target T-32 search QPS baseline: 100 QPS, source: `capacity-model.md:17-28`.
33. Target T-33 search QPS maximum planning ceiling: 5,000 QPS, source: `capacity-model.md:17-28`.
34. Target T-34 active legal holds baseline: 100, source: `capacity-model.md:17-28`.
35. Target T-35 active legal holds maximum planning ceiling: 50,000, source: `capacity-model.md:17-28`.
36. Target T-36 legal-hold engagement p99: 1,000 ms, source: current local SLO `slos/legal-hold-engagement-p99.openslo.yaml:5-24`.
37. Target T-37 legal-hold chain correctness: 100 percent, source: current local SLO `slos/legal-hold-chain-correctness.openslo.yaml:5-31`.
38. Target T-38 retention-policy correctness: 100 percent, source: current local SLO `slos/retention-policy-correctness.openslo.yaml:5-30`.
39. Target T-39 redaction render p99: 1,000 ms for preview clips, source: current local SLO `slos/redaction-render-p99.openslo.yaml:5-16`.
40. Target T-40 eDiscovery transcript PDF export p99: 3 seconds, source: current local SLO `slos/ediscovery-export-transcript-pdf-p99.openslo.yaml:5-15`.
41. Target T-41 eDiscovery MP4 export p99: media-duration x 0.30, source: current local SLO `slos/ediscovery-export-mp4-p99.openslo.yaml:5-16`.
42. Target T-42 AI summary availability p50: transcript-ready plus 5 seconds for 60-minute recordings.
43. Target T-43 AI summary availability p95: transcript-ready plus 15 seconds.
44. Target T-44 AI summary availability p99: transcript-ready plus 30 seconds.
45. Target T-45 chapter/highlight generation p99: transcript-ready plus 45 seconds for 60-minute recordings.
46. Target T-46 action-item extraction p99: transcript-ready plus 30 seconds.
47. Target T-47 conversation-fact event emission p99: transcript-ready plus 60 seconds if Gong-style facts are emitted.
48. Target T-48 quota warning threshold: emit warning at 80 percent of tenant storage/monthly transcript budget, matching Zoom's public alert threshold behavior.
49. Target T-49 over-budget in-progress recording behavior: allow current recording to complete, then block new captures or route to paid/revenue-share policy.
50. Target T-50 export bundle manifest correctness: 100 percent hash-chain and source-artifact completeness for legal/export workflows.

### §3.2 Deployment-context overlays

1. `oyatie-public-cloud` overlay: targets T-01 through T-50 apply with horizontal elasticity and regional failover.
2. `oyatie-public-cloud` overlay: throughput ceilings T-04, T-05, T-26, T-29, T-31, and T-33 scale by region count and paid capacity.
3. `guest-on-aws` overlay: same target latencies apply when provisioned with equivalent CPU/GPU/object-storage/search resources.
4. `guest-on-aws` overlay: AWS account quotas may cap maximum media-hour and playback ceilings until account limit increases are in place.
5. `guest-on-oci` overlay: same target latencies apply for paid/revenue-share deployments with equivalent resources.
6. `guest-on-oci` overlay: OCI Always Free profile constrains demo-trial throughput and storage; current service path lacks the required module to prove exact caps.
7. `on-prem` overlay: target quality remains the same, but maximum throughput is facility-specific and must be declared in customer capacity worksheets.
8. `on-prem` overlay: transcription acceleration must be local GPU, CPU fallback, or approved policy-based offload.
9. `colo` overlay: target quality remains the same, but WAN egress, CDN proximity, and storage replication govern playback and export ceilings.
10. `colo` overlay: legal-hold and retention correctness targets remain non-negotiable at 100 percent.
11. `oyatie-as-cloud-provider` overlay: targets apply with Oyatie-owned cell capacity and substrate quotas.
12. `oyatie-as-cloud-provider` overlay: cell-level isolation must preserve recording legal holds and retained media under tenant move/rebalance events.
13. Common overlay: legal-hold engagement, retention correctness, DSR correctness, and export manifest correctness do not degrade by context.
14. Common overlay: playback and search p99 may require smaller advertised concurrency ceilings in on-prem/colo/Always Free contexts.
15. Common overlay: if a context lacks local GPU capacity, transcript and summary completion targets must be recalculated and disclosed before sale or deployment.

### §3.3 Tenant-class overlays

1. `demo_trial` overlay: quality target remains industry-leader grade, but usage is capped.
2. `demo_trial` overlay: assumes OCI Always Free profile where applicable.
3. `demo_trial` overlay: suggested monthly transcript budget is 300 minutes to match the public Otter Basic reference point unless product policy sets a lower cap.
4. `demo_trial` overlay: suggested single-recording duration cap is 30 minutes to match the public Otter Basic reference point unless product policy sets a lower cap.
5. `demo_trial` overlay: suggested file import cap is 2 GB under OCI Always Free profile, below the 5 GB public Otter import ceiling because substrate budget is tighter.
6. `demo_trial` overlay: compliance packs, BYOK, legal-hold creation, and eDiscovery export should be disabled or simulated with explicit non-production labels.
7. `demo_trial` overlay: playback concurrency should be capped per tenant; recommended initial cap is 10 concurrent playback sessions.
8. `demo_trial` overlay: share links should expire quickly; recommended cap is 24 hours or less.
9. `demo_trial` overlay: usage exhaustion should return a policy-coded response, not a lower-quality feature.
10. `paid` overlay: quality target remains industry-leader grade and contractual SLOs apply.
11. `paid` overlay: storage, transcription, playback, export, and AI-derived artifact generation scale with paid entitlements and usage billing.
12. `paid` overlay: compliance packs, BYOK, legal hold, DSR, and eDiscovery workflows may be enabled by contract and policy.
13. `paid` overlay: file import target is 10 GB by default with larger limits available after capacity review.
14. `paid` overlay: concurrent playback target scales from 5,000 baseline to 200,000 regional maximum when capacity is provisioned.
15. `paid` overlay: p99 targets remain T-18 through T-41 unless customer-specific deployment capacity is documented.
16. `revenue_share` overlay: quality target remains industry-leader grade and does not become a cheaper feature set.
17. `revenue_share` overlay: substrate should run at cost or zero-margin with usage transparent enough to calculate gross-revenue share economics.
18. `revenue_share` overlay: storage and bandwidth caps should be tied to commercial risk controls rather than feature downgrades.
19. `revenue_share` overlay: compliance workflows require explicit contract terms because legal evidence-chain support creates nonzero operational burden.
20. `revenue_share` overlay: high-volume creator, marketplace, B2C, embedded SaaS, and affiliate workloads should prefer cell-level isolation and quota dashboards.

## §4 Comparison narrative

1. Capture start: Oyatie target T-01/T-03 is ahead of Gong ad hoc start behavior because Gong publicly states ad hoc homepage recording can take up to 10 minutes to start.
2. Capture start: Oyatie target is parity with native meeting capture expectations for Zoom when host/co-host starts recording.
3. Capture start: Oyatie needs calendar/assistant semantics to reach Otter and Gong auto-join parity.
4. Storage quota: Oyatie target T-48 matches Zoom's public 80 percent alert threshold.
5. Storage quota: Oyatie catch-up item is to add quota events and tenant-class cap enforcement to contracts.
6. Cloud storage scale: Zoom publishes 10 GB, 15 GB, and unlimited storage classes; Oyatie should not copy plan segmentation, but paid/revenue-share storage must be capacity-priced and explicit.
7. File count and layout: Zoom's 150-file live-session limit is a clear counterpart number; Oyatie needs artifact-variant and file-fragment models before it can claim parity.
8. Import file size: Otter's 5 GB public import cap is the comparison number; Oyatie paid target of 10 GB is ahead, while demo-trial 2 GB is intentionally constrained by infrastructure budget.
9. Basic usage caps: Otter's 300 minutes/month and 30 minutes/conversation numbers are useful demo-trial reference points; Oyatie should codify or consciously reject them.
10. Long-call processing: Gong's public 6-hour processing limit is the comparison number; Oyatie target T-09 matches the operational reality by chunking long recordings rather than pretending unlimited single-unit processing.
11. Transcript availability: vendor p99 numbers are not publicly published; Oyatie target media-duration x 0.50 p99 is aggressive and needs measurement.
12. Transcript render: local SLO 500 ms p99 is industry-leader grade for ready transcript viewing.
13. Transcript search: local SLO 300 ms p99 across 1,000 hours is a strong searchable-archive target and should beat common SaaS user expectations.
14. Playback start: local SLO 400 ms warm p99 and 1,000 ms cold p99 is strong compared with public cloud-streaming expectations.
15. AI summaries: Zoom and Otter both expose summaries and key moments; Oyatie target transcript-ready plus 30 seconds p99 is competitive if measured.
16. Chapters/highlights: Zoom exposes chapters/highlights; Oyatie needs contract-level artifacts before numeric targets matter.
17. AI chat: Otter exposes AI Chat during or after meetings; Oyatie has no explicit API and is in catch-up until product ownership is decided.
18. Conversation intelligence: Gong exposes revenue analytics, risk, coaching, CRM integration, and talk metrics; Oyatie recordings is in catch-up unless it emits or owns those facts.
19. Redaction: Gong redacts numeric/PHI data; Oyatie's redaction render p99 plus evidence-chain target is strong, but derived AI artifacts must honor redaction versions.
20. Legal hold: Oyatie target of 1,000 ms p99 engagement and 100 percent chain correctness is ahead of meeting-assistant baseline and appropriate for compliance workloads.
21. eDiscovery export: Oyatie transcript PDF p99 of 3 seconds and MP4 duration x 0.30 p99 are strong local targets.
22. Retention correctness: Oyatie's 100 percent retention correctness target is mandatory and cannot vary by deployment context.
23. DSR/delete cascade: Oyatie workflows exist, but benchmark harness must verify retained, held, redacted, and exported artifacts across state transitions.
24. Concurrent playback: Oyatie 200,000 regional ceiling is ahead as a target, but current service-local IaC cannot prove it.
25. Search QPS: Oyatie 5,000 QPS maximum planning ceiling is credible as a target, but needs Meilisearch/index capacity validation from `capacity-model.md:72-80`.
26. Transcription throughput: `capacity-model.md:83-90` states GPU-driven throughput assumptions; production benchmark must separate Whisper, diarization, and queueing time.
27. Cost efficiency: `capacity-model.md:112-120` estimates transcription, storage, search, CDN, and export costs per 1,000 media hours; benchmark reporting should include cost alongside latency.
28. OCI Always Free: Oyatie cannot claim demo-trial infrastructure readiness until `iac/oci-guest/always-free/` exists and enforces caps.
29. OS matrix: Oyatie cannot claim full canonical OS readiness until `supported-oses.json` exists and build/runtime checks are attached.
30. Deployment contexts: Oyatie cannot claim all six deployable contexts until OpenTofu modules or N/A manifests exist.
31. Public-cloud elasticity: Oyatie can set aggressive targets for public cloud, but current IaC gap makes the targets unvalidated.
32. On-prem and colo: Oyatie should preserve the same latency/quality bar but publish smaller maximum ceilings when local facility resources are smaller.
33. Revenue-share tenants: target quality stays constant, but quotas must keep at-cost substrate from becoming unbounded loss.
34. Paid tenants: paid capacity should scale with payment and contract, not unlock a better feature set.
35. Demo-trial tenants: demo-trial caps should restrict usage, not legal correctness of any allowed workflow.
36. Benchmark conclusion: Oyatie's stated SLOs are stronger than many public SaaS limits, but current deployment-control gaps mean the service is target-rich rather than production-proven.
37. Immediate benchmark follow-up: author a load-test plan that validates T-01 through T-50 against a Rust implementation and OpenTofu-provisioned environments.
38. Immediate documentation follow-up: retire old benchmark segmentation and replace it with this target-set shape.
39. Immediate product follow-up: decide Gong ownership before implementing conversation-intelligence metrics in recordings.
40. Immediate policy follow-up: encode tenant-class quota behavior in machine-readable service policy.

## §5 Benchmark harness requirements

1. Harness H-01 must generate synthetic 30-minute, 60-minute, and 6-hour recordings with audio, video, chat, and slide markers.
2. Harness H-02 must submit source events through the canonical ingest contract.
3. Harness H-03 must measure ingest acknowledgment p50/p95/p99.
4. Harness H-04 must measure media normalization duration as a multiple of media duration.
5. Harness H-05 must measure transcript generation duration as a multiple of media duration.
6. Harness H-06 must measure diarization accuracy and latency separately from speech-to-text.
7. Harness H-07 must measure transcript render p99.
8. Harness H-08 must measure transcript search p99 across 1,000-hour, 10,000-hour, and 100,000-hour archives.
9. Harness H-09 must measure playback warm and cold start p50/p95/p99.
10. Harness H-10 must measure eDiscovery transcript PDF export p99.
11. Harness H-11 must measure eDiscovery MP4 export duration multiplier.
12. Harness H-12 must measure redaction preview p99 and final render duration.
13. Harness H-13 must measure legal-hold engagement p99 and chain-correctness invariants.
14. Harness H-14 must verify retention delete never removes held artifacts.
15. Harness H-15 must verify DSR deletion cascades across media, transcripts, summaries, shares, search indexes, and exports.
16. Harness H-16 must verify quota-warning threshold at 80 percent.
17. Harness H-17 must verify in-progress recording completion after quota exhaustion.
18. Harness H-18 must verify new recording denial or upgrade routing after quota exhaustion.
19. Harness H-19 must verify demo-trial caps under OCI Always Free profile.
20. Harness H-20 must verify paid tenant scaling under provisioned public-cloud capacity.
21. Harness H-21 must verify revenue-share usage accounting and cost attribution.
22. Harness H-22 must verify on-prem/colo advertised capacity does not exceed declared facility resources.
23. Harness H-23 must verify every AI-derived artifact carries source transcript, redaction version, and legal-hold state.
24. Harness H-24 must verify conversation-fact events if Gong-style analytics are emitted.
25. Harness H-25 must publish benchmark evidence as JSON alongside human-readable summaries.

## §6 Numeric target ledger for follow-up validation

1. Ledger item L-01: p99 ingest acknowledgment target is 1,000 ms.
2. Ledger item L-02: p99 normal media processing target is media-duration x 0.75.
3. Ledger item L-03: p99 transcript availability target is media-duration x 0.50.
4. Ledger item L-04: p99 live interim transcript latency target is 4.0 seconds.
5. Ledger item L-05: p99 transcript render target is 500 ms.
6. Ledger item L-06: p99 transcript search target is 300 ms across a 1,000-hour archive.
7. Ledger item L-07: p99 recording list target is 200 ms.
8. Ledger item L-08: p99 warm playback target is 400 ms.
9. Ledger item L-09: p99 cold playback target is 1,000 ms.
10. Ledger item L-10: public-cloud regional playback ceiling target is 200,000 concurrent sessions.
11. Ledger item L-11: daily recordings maximum planning ceiling is 1,000,000.
12. Ledger item L-12: daily media-hour maximum planning ceiling is 200,000.
13. Ledger item L-13: transcript search maximum planning ceiling is 5,000 QPS.
14. Ledger item L-14: active legal-hold maximum planning ceiling is 50,000.
15. Ledger item L-15: p99 legal-hold engagement target is 1,000 ms.
16. Ledger item L-16: legal-hold chain correctness target is 100 percent.
17. Ledger item L-17: retention-policy correctness target is 100 percent.
18. Ledger item L-18: p99 redaction preview target is 1,000 ms.
19. Ledger item L-19: p99 eDiscovery transcript PDF export target is 3 seconds.
20. Ledger item L-20: p99 eDiscovery MP4 export target is media-duration x 0.30.
21. Ledger item L-21: p99 AI summary target is transcript-ready plus 30 seconds.
22. Ledger item L-22: p99 chapter/highlight target is transcript-ready plus 45 seconds.
23. Ledger item L-23: quota warning target is 80 percent budget consumption.
24. Ledger item L-24: paid/revenue-share single import target is 10 GB where capacity permits.
25. Ledger item L-25: demo-trial single import cap target is 2 GB under OCI Always Free profile.
