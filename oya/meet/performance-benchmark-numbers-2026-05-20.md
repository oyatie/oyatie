# Meet Performance Benchmark Numbers - 2026-05-20

Audit owner: single-agent Wave 3 Batch 3.2 ownership-coherence audit.
Target microservice: `microservices/meet/`.
Counterparts: Zoom, Google Meet, Microsoft Teams Meetings.
Methodology disclosure: this report separates public product-limit numbers from estimated performance targets; public counterpart docs do not publish audited p50/p95/p99 media-path latency distributions.
Retired deliverable note: the former fourth delta report is not authored; targets are single industry-leader targets with deployment-context and tenant_class overlays.

## Five-Citation Anchor Block

1. Local performance commitments: `microservices/meet/PRD.md:80-91` defines room creation, join, intra-region media, inter-region media, caption, summary, recording, broadcast, availability, RPO, and RTO targets.
2. Local capacity commitments: `microservices/meet/capacity-model.md:23-47`, `microservices/meet/capacity-model.md:191-205`, and `microservices/meet/cost-budget.md:1-115` define service components, LiveKit pod capacity, and existing tenant-size language.
3. Local SLO sources: `microservices/meet/slos/availability.openslo.yaml`, `microservices/meet/slos/media-quality.openslo.yaml`, and `microservices/meet/slos/recording-pipeline.openslo.yaml`.
4. Canonical deployment and substrate sources: `specs/master-plan-sequencing.json:704-868` and `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-3828`.
5. Current public counterpart and infrastructure sources: Zoom support participant/license/E2EE docs, Google Workspace Meet feature docs, Microsoft Learn Teams docs, and Oracle OCI Always Free resource docs.

## 1. Methodology

1. Scope: benchmark dimensions cover meeting control plane, media plane, caption/transcription pipeline, recording pipeline, broadcast fanout, compliance operations, failover, and administrative operations.
2. Scope: this is a target-number audit, not a measured load-test result.
3. Scope: current local artifacts provide intended targets but no executable benchmark harness in `microservices/meet/benchmarks/meetbench/`.
4. Evidence: the benchmark doc references a harness path, but that path was absent from the inventory.
5. Test workload A: one-on-one meeting with audio, 720p video, captions disabled, no recording.
6. Test workload B: 10-person interactive meeting with screen share, captions enabled, recording enabled, one lobby event per participant.
7. Test workload C: 100-person interactive classroom with breakout rooms, captions, transcript generation, and host moderation.
8. Test workload D: 1,000-person all-hands with first-class interactive controls for authorized presenters and constrained attendee controls.
9. Test workload E: 100,000-viewer broadcast/webinar with Q&A, recording, transcript, egress, and compliance audit trail.
10. Test workload F: encrypted meeting mode with recording/transcription disabled or restricted according to the selected security policy.
11. Test workload G: compliance hold and disclosure export during or after a recorded meeting.
12. Test workload H: region failover for room control plane and media relay health.
13. OS disclosure: no service-local `supported-oses.json` was found, so OS coverage cannot be benchmark-certified from this path.
14. OS target set: Linux server runtime; web clients on current Chrome, Edge, Firefox, and Safari; native clients on macOS, Windows, Linux, iOS, iPadOS, and Android once client artifacts exist.
15. Architecture disclosure: target numbers assume Rust backend services, approved frontend stacks, WebRTC media, LiveKit-compatible SFU semantics where cited by local docs, and OpenTofu-managed deployment.
16. Architecture disclosure: no Python, JavaScript app, Ruby, Go, Java, Scala, Groovy, PHP, or F# implementation evidence was found under the microservice path.
17. Deployment contexts: targets are evaluated across `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.
18. Infrastructure disclosure: the inspected path has Helm and Kustomize but lacks canonical per-context OpenTofu modules, so deployment overlay numbers are design targets until IaC lands.
19. OCI Always Free disclosure: the demo_trial tenant_class infrastructure profile must respect the OCI Always Free profile and should reject or cap usage rather than lower quality below the target while serving admitted sessions.
20. Tenant class disclosure: `demo_trial`, `paid`, and `revenue_share` are commercial/usage classes, not feature-quality classes.
21. Tenant class disclosure: the inspected meet path does not yet express `tenant_class` semantics.
22. Measurement disclosure: counterpart p50/p95/p99 latencies are not public audited metrics in the cited docs.
23. Measurement disclosure: counterpart numbers below are public capacity, duration, feature-limit, and event-limit numbers unless explicitly marked as an estimate.
24. Measurement disclosure: Oyatie p50/p95/p99 values are target commitments derived from local PRD intent and industry-leader expectations, not measured results.
25. Methodology rule: do not compare a target latency against a counterpart if the counterpart source does not publish the same metric.
26. Methodology rule: compare public capacity numbers directly only when event forms match.
27. Methodology rule: distinguish interactive capacity from view-only or broadcast capacity.
28. Methodology rule: distinguish meeting duration from recording duration.
29. Methodology rule: distinguish contractual SLO from best-effort demo infrastructure.
30. Methodology rule: all admitted user-facing sessions should target the same quality envelope regardless of tenant_class.
31. Methodology rule: caps, throttles, and quota rejection are acceptable for demo_trial; degrading core media quality after admission is not acceptable.
32. Methodology rule: deployment overlays may reduce admitted concurrency when infrastructure is constrained.
33. Methodology rule: deployment overlays may add region/facility caveats for on-prem and colo.
34. Methodology rule: if local docs lack evidence, this report states the gap rather than inventing a measured result.
35. Methodology result: current Meet artifacts can support target-setting, not completion claims.

## 2. Counterpart Numbers

### 2.1 Zoom Public Numbers

1. Zoom number Z-01: Basic meeting capacity is 100 participants; source: `https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0068002`.
2. Zoom number Z-02: Pro meeting capacity is 100 participants by default; source: same Zoom participant-limit doc.
3. Zoom number Z-03: Business meeting capacity is 300 participants by default; source: same Zoom participant-limit doc.
4. Zoom number Z-04: Enterprise meeting capacity is 500 participants by default; source: same Zoom participant-limit doc.
5. Zoom number Z-05: optional meeting add-ons include 500 and 1,000 participants in the participant-limit doc; source: same Zoom participant-limit doc.
6. Zoom number Z-06: Zoom license comparison lists Large Meeting add-on capacity up to 5,000 participants depending on license; source: `https://support.zoom.com/hc/zt/article?id=zm_kb&sysparm_article=KB0062404`.
7. Zoom number Z-07: Zoom license comparison lists webinar capacity up to 100,000 view-only attendees depending on capacity purchased; source: same Zoom license comparison doc.
8. Zoom number Z-08: Zoom license comparison lists meeting duration of 40 minutes for Basic meetings; source: same Zoom license comparison doc.
9. Zoom number Z-09: Zoom license comparison lists 30-hour duration for Pro, Large Meeting, and Webinar rows; source: same Zoom license comparison doc.
10. Zoom number Z-10: Zoom license comparison states video sharing is unavailable for meetings with 1,000 or more participants in the relevant row; source: same Zoom license comparison doc.
11. Zoom number Z-11: Zoom license comparison states screen sharing is unavailable for meetings with 1,000 or more participants in the relevant row; source: same Zoom license comparison doc.
12. Zoom number Z-12: Zoom license comparison states multi-pinning is host-only in meetings with 500 or more participants; source: same Zoom license comparison doc.
13. Zoom number Z-13: Zoom E2EE meetings are limited to 1,000 meeting participants; source: `https://support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0075502`.
14. Zoom number Z-14: Zoom E2EE disables cloud recording, live streaming, live transcription, polling, and several app/AI features; source: same Zoom E2EE doc.
15. Zoom number Z-15: public p50/p95/p99 join and media latency distributions are not published in the cited Zoom docs; source status: not public in cited docs.

### 2.2 Google Meet Public Numbers

1. Google number G-01: Business Starter meeting length is 24 hours; source: `https://support.google.com/a/answer/10037875`.
2. Google number G-02: Business Standard meeting length is 24 hours; source: same Google Business feature doc.
3. Google number G-03: Business Plus meeting length is 24 hours; source: same Google Business feature doc.
4. Google number G-04: Business Starter meeting participant limit is 100; source: same Google Business feature doc.
5. Google number G-05: Business Standard meeting participant limit is 150; source: same Google Business feature doc.
6. Google number G-06: Business Plus meeting participant limit is 500; source: same Google Business feature doc.
7. Google number G-07: Enterprise Standard meeting length is 24 hours; source: `https://support.google.com/a/answer/10037875?co=DASHER._Family%3DEnterprise`.
8. Google number G-08: Enterprise Plus meeting length is 24 hours; source: same Google Enterprise feature doc.
9. Google number G-09: Enterprise Standard meeting participant limit is 500; source: same Google Enterprise feature doc.
10. Google number G-10: Enterprise Plus meeting participant limit is 1,000, with additional viewers after 500 in view-only mode; source: same Google Enterprise feature doc.
11. Google number G-11: Enterprise Standard in-domain and trusted-domain live streaming supports 10,000 viewers; source: same Google Enterprise feature doc.
12. Google number G-12: Enterprise Plus in-domain and trusted-domain live streaming supports 100,000 viewers; source: same Google Enterprise feature doc.
13. Google number G-13: Google Meet live streaming supports up to 50 trusted Workspace sub-domains for cross-domain streaming; source: `https://support.google.com/meet/answer/9308630?hl=En&ref_topic=14074639`.
14. Google number G-14: Google's basic Meet feature page lists 100 participants as the basic participant limit; source: `https://support.google.com/meet/answer/13396001?hl=en`.
15. Google number G-15: public p50/p95/p99 join and media latency distributions are not published in the cited Google docs; source status: not public in cited docs.

### 2.3 Microsoft Teams Meetings Public Numbers

1. Teams number M-01: Teams meetings support audio, video, and screen sharing for up to around 1,000 people; source: `https://learn.microsoft.com/en-us/microsoftteams/plan-meetings`.
2. Teams number M-02: view-only capabilities start when around 900 participants join if view-only is enabled; source: same Teams meetings plan doc.
3. Teams number M-03: up to 10,000 attendees can join a Teams meeting with extra attendees in view-only mode after around 900 users; source: same Teams meetings plan doc.
4. Teams number M-04: if view-only is turned off, meeting attendance is limited to the first 1,000 attendees; source: same Teams meetings plan doc.
5. Teams number M-05: Teams overview states meetings can support 11,000 total participants with 1,000 interactive and 10,000 view-only; source: `https://learn.microsoft.com/en-us/microsoftteams/overview-meetings-webinars-town-halls`.
6. Teams number M-06: Teams webinars support up to 1,000 attendees; source: same Teams overview doc.
7. Teams number M-07: Teams town halls support up to 3,000 attendees before capacity add-on scaling; source: same Teams overview doc.
8. Teams number M-08: Teams town halls can scale up to 100,000 attendees with an attendee capacity add-on; source: same Teams overview doc.
9. Teams number M-09: Teams feature comparison lists view-only attendees of 10,000 for meetings and town halls; source: `https://learn.microsoft.com/nl-nl/Microsoftteams/meeting-webinar-town-hall-feature-comparison`.
10. Teams number M-10: Teams limits doc lists 300-person meeting limits for several Microsoft 365 Business and Essentials plans; source: `https://learn.microsoft.com/en-us/microsoftteams/limits-specifications-teams`.
11. Teams number M-11: Teams limits doc lists 1,000-person meeting limits for specified enterprise/education/government plans; source: same Teams limits doc.
12. Teams number M-12: Teams limits doc lists 20 people in a video or audio call from chat; source: same Teams limits doc.
13. Teams number M-13: Teams limits doc lists meeting recording maximum length as 4 hours or 1.5 GB before restart behavior; source: same Teams limits doc.
14. Teams number M-14: Teams limits doc states breakout rooms can only be created in meetings with fewer than 300 attendees and creating them limits the meeting to 300 attendees; source: same Teams limits doc.
15. Teams number M-15: public p50/p95/p99 join and media latency distributions are not published in the cited Microsoft docs; source status: not public in cited docs.

## 3. Oyatie Target Numbers - Single Industry-Leader Target Set

### 3.1 Canonical Target Metrics

1. Target O-01: room create API p50 <= 30 ms, p95 <= 80 ms, p99 <= 100 ms; local source target `PRD.md:80-91`.
2. Target O-02: room create accepted error rate <= 0.01% under normal capacity.
3. Target O-03: join path p50 <= 800 ms, p95 <= 1.5 s, p99 <= 2.0 s; local source target `PRD.md:80-91`.
4. Target O-04: lobby admit p50 <= 150 ms, p95 <= 400 ms, p99 <= 750 ms.
5. Target O-05: participant token mint p50 <= 40 ms, p95 <= 100 ms, p99 <= 200 ms.
6. Target O-06: intra-region media one-way latency p50 <= 80 ms, p95 <= 150 ms, p99 <= 200 ms; local source target `PRD.md:80-91`.
7. Target O-07: inter-region media one-way latency p50 <= 130 ms, p95 <= 250 ms, p99 <= 350 ms; local source target `PRD.md:80-91`.
8. Target O-08: audio packet loss recovery should mask loss up to 3% without user-visible disconnect under normal jitter.
9. Target O-09: video adaptive bitrate convergence p95 <= 3 s after bandwidth step-change.
10. Target O-10: screen-share first-frame p50 <= 400 ms, p95 <= 800 ms, p99 <= 1.2 s.
11. Target O-11: captions p50 <= 300 ms, p95 <= 450 ms, p99 <= 500 ms; local source target `PRD.md:80-91`.
12. Target O-12: live translation p50 <= 500 ms, p95 <= 900 ms, p99 <= 1.2 s for supported language pairs.
13. Target O-13: transcript segment persistence p95 <= 5 s after spoken segment finalization.
14. Target O-14: recording ready-to-play p50 <= 30 s, p95 <= 60 s, p99 <= 120 s for meetings under 2 hours.
15. Target O-15: recording pipeline steady-state throughput >= 1.2x real-time per active recorded stream.
16. Target O-16: summary generation for a 60-minute meeting p95 <= 90 s after transcript finalization; local source target is 60 s for AI summary, so implementation should preserve or explicitly re-baseline this target.
17. Target O-17: webinar/broadcast fanout setup p95 <= 3 s for existing session; local source target `PRD.md:80-91`.
18. Target O-18: broadcast viewer join p50 <= 1.0 s, p95 <= 2.0 s, p99 <= 3.5 s once broadcast edge is warm.
19. Target O-19: interactive meeting ceiling >= 1,000 participants per meeting form.
20. Target O-20: broadcast/view-only ceiling >= 100,000 viewers per event form.
21. Target O-21: breakout-room creation p95 <= 2 s for 50 rooms and <= 5 s for 100 rooms.
22. Target O-22: compliance hold creation p95 <= 500 ms after authenticated request.
23. Target O-23: disclosure export manifest generation p95 <= 60 s for a 2-hour meeting with transcript and chat metadata.
24. Target O-24: service availability target >= 99.95% for room create and join control plane; local source target `PRD.md:80-91`.
25. Target O-25: media plane availability target >= 99.9% excluding client network faults.
26. Target O-26: recording pipeline availability target >= 99.9% for accepted recording jobs.
27. Target O-27: RPO <= 5 minutes for meeting metadata, transcripts, and recording manifests; local source target `PRD.md:80-91`.
28. Target O-28: RTO <= 15 minutes for control-plane recovery; local source target `PRD.md:80-91`.
29. Target O-29: per-region active meeting control-plane capacity >= 50,000 concurrent meetings when deployed on elastic public-cloud or Oyatie-provider substrate.
30. Target O-30: per-region participant capacity >= 500,000 concurrent participants when deployed on elastic public-cloud or Oyatie-provider substrate.
31. Target O-31: per-SFU pod capacity baseline remains the local model value of roughly 1,500 interactive participants or 7 Gbps egress until measured benchmarks replace it; local source `capacity-model.md:43-47`.
32. Target O-32: compliance audit event emission p99 <= 1 s after the triggering control-plane action.
33. Target O-33: admin policy propagation p95 <= 30 s across meeting control services.
34. Target O-34: tenant quota decision p95 <= 20 ms in the join path.
35. Target O-35: overload behavior should reject new admission before degrading admitted-session media quality.

### 3.2 Deployment-Context Overlays

1. Overlay D-01 `oyatie-public-cloud`: canonical target set applies with elastic scale and contractual paid/revenue-share SLOs.
2. Overlay D-02 `oyatie-public-cloud`: demo_trial caps may restrict room count, duration, recording minutes, transcript minutes, and broadcast viewers, but admitted sessions still target the same media envelope.
3. Overlay D-03 `guest-on-aws`: canonical target set applies when customer-provided AWS resources meet sizing and network prerequisites.
4. Overlay D-04 `guest-on-aws`: OpenTofu must provision equivalent network, compute, storage, secrets, observability, and media-edge resources without AWS-only code paths becoming product logic.
5. Overlay D-05 `guest-on-oci`: canonical target set applies for paid and revenue_share tenants when customer-provided OCI resources are sized beyond the demo_trial floor.
6. Overlay D-06 `guest-on-oci`: OCI Always Free profile is a demo_trial infrastructure profile, not a feature-quality profile.
7. Overlay D-07 `guest-on-oci`: OCI Always Free public source lists 4 total Ampere A1 OCPUs and 24 GB memory for the profile; source `https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm`.
8. Overlay D-08 `guest-on-oci`: OCI Always Free public source lists 200 GB combined block volume storage, 20 GB object storage in the always-free-only state, 10 Mbps flexible load balancer, and 10 TB monthly outbound data.
9. Overlay D-09 `guest-on-oci`: demo_trial admitted-concurrency target should be constrained to 1-3 active small rooms or an equivalent measured envelope until a meet-specific benchmark proves higher capacity.
10. Overlay D-10 `guest-on-oci`: demo_trial broadcast should default to low viewer caps and short durations because 10 Mbps load-balancer bandwidth is not suitable for high-scale broadcast without alternate edge design.
11. Overlay D-11 `guest-on-oci`: recording and transcription should be quota-limited in demo_trial because 200 GB block and 20 GB object storage can be exhausted by meeting media.
12. Overlay D-12 `on-prem`: canonical latency targets apply only after facility network, TURN/SFU placement, storage, GPU/accelerator availability, and client path are validated.
13. Overlay D-13 `on-prem`: availability and RTO become facility-specific unless Oyatie owns enough redundant substrate.
14. Overlay D-14 `colo`: canonical target set applies when colo network and hardware meet the same media-edge prerequisites.
15. Overlay D-15 `colo`: inter-region media targets may need custom overlays for peering, transit, and customer WAN path.
16. Overlay D-16 `oyatie-as-cloud-provider`: canonical target set applies and should be the reference environment for public benchmark certification.
17. Overlay D-17 `oyatie-as-cloud-provider`: target scale may exceed public-cloud overlays if Oyatie controls media-edge placement, network routing, and cost envelope.
18. Overlay D-18 all contexts: per-context OpenTofu modules are required before these numbers can be claimed as deployment-ready.
19. Overlay D-19 all contexts: no deployment overlay may use Terraform, Pulumi, CloudFormation, ARM, shell provisioners, or manual console steps as the canonical path.
20. Overlay D-20 all contexts: benchmark reports must record OS, architecture, CPU, memory, network, storage, region/facility, and tenant_class.
21. Overlay D-21 all contexts: control-plane targets and media-plane targets must be reported separately.
22. Overlay D-22 all contexts: interactive and broadcast/view-only event forms must be reported separately.
23. Overlay D-23 all contexts: encrypted meeting mode must be reported separately because recording/transcription availability may differ.
24. Overlay D-24 all contexts: failed-admission behavior must be counted as capacity enforcement, not as media-quality degradation.
25. Overlay D-25 all contexts: if a context is judged not applicable, the service must provide an explicit N/A decision with evidence.

### 3.3 Tenant-Class Overlays

1. Overlay C-01 `demo_trial`: uses OCI Always Free profile or another explicitly capped low-cost profile.
2. Overlay C-02 `demo_trial`: has hard usage caps for room count, duration, storage, transcript minutes, recording minutes, broadcast viewers, and monthly egress.
3. Overlay C-03 `demo_trial`: has best-effort SLO, no compliance packs, and no BYOK.
4. Overlay C-04 `demo_trial`: admitted sessions still target the same join, media, caption, recording, and transcript quality numbers until caps reject new work.
5. Overlay C-05 `demo_trial`: benchmarks must report cap-hit behavior and user-facing rejection latency.
6. Overlay C-06 `paid`: per-seat license plus usage-based billing.
7. Overlay C-07 `paid`: any deployment context is allowed if the substrate passes admission and benchmark gates.
8. Overlay C-08 `paid`: contractual SLO applies.
9. Overlay C-09 `paid`: compliance packs and BYOK are allowed subject to policy and implementation support.
10. Overlay C-10 `paid`: scaling follows payment, quota, and deployed capacity.
11. Overlay C-11 `revenue_share`: Oyatie takes a percentage of customer gross revenue for suitable marketplace, B2C, embedded SaaS reseller, or affiliate-partner use.
12. Overlay C-12 `revenue_share`: substrate runs at-cost or zero-margin unless the contract says otherwise.
13. Overlay C-13 `revenue_share`: quality target remains identical to paid for admitted workloads.
14. Overlay C-14 `revenue_share`: benchmark reports must expose unit economics, egress, recording storage, transcription cost, and support cost.
15. Overlay C-15 all tenant classes: no feature-quality stratification is introduced by this benchmark model.
16. Overlay C-16 all tenant classes: usage caps and contractual rights are explicit commercial controls.
17. Overlay C-17 all tenant classes: legal-hold, disclosure, privacy, and audit behavior must be explicit, not inferred.
18. Overlay C-18 all tenant classes: the current meet path lacks tenant_class semantics, so these overlays are target requirements.
19. Overlay C-19 all tenant classes: benchmark data must identify tenant_class to avoid mixing demo caps with paid/revenue-share scale.
20. Overlay C-20 all tenant classes: any rejected workload must emit auditable quota and reason metadata.

## 4. Comparison Narrative

1. Headline H-01 interactive capacity: Oyatie target of 1,000 interactive participants reaches Google Enterprise Plus and Teams interactive range, and is below Zoom large-meeting 5,000 add-on capacity.
2. Headline H-02 interactive capacity status: catch-up versus Zoom high-capacity add-on; parity versus Google Enterprise Plus and Teams interactive meetings.
3. Headline H-03 broadcast capacity: Oyatie target of 100,000 broadcast/view-only viewers matches public Zoom webinar and Google Enterprise Plus live-stream ceilings and matches Microsoft high-capacity town hall add-on claims.
4. Headline H-04 broadcast capacity status: parity target, but local evidence lacks eCDN and benchmark harness proof.
5. Headline H-05 meeting duration: Oyatie target should support at least 24-hour scheduled meetings to match Google and 30-hour meeting/event duration to match Zoom and Teams where event forms require it.
6. Headline H-06 meeting duration status: catch-up because the PRD does not pin duration targets as clearly as the counterparts.
7. Headline H-07 join latency: Oyatie target p95 <= 1.5 s is aggressive and useful, but counterparts do not publish equivalent p95 numbers in cited docs.
8. Headline H-08 join latency status: evidence-gap comparison; target is industry-leader-shaped but unmeasured.
9. Headline H-09 media latency: Oyatie intra-region p95 <= 150 ms and inter-region p95 <= 250 ms are appropriate for real-time collaboration.
10. Headline H-10 media latency status: evidence-gap comparison; counterpart public docs do not publish comparable p95 media-latency distributions.
11. Headline H-11 caption latency: Oyatie p99 <= 500 ms target is stronger than most public product docs disclose.
12. Headline H-12 caption latency status: ahead target if measured, currently unproven.
13. Headline H-13 recording readiness: Oyatie target p95 <= 60 s after meeting completion is practical for enterprise workflows.
14. Headline H-14 recording readiness status: evidence-gap comparison because counterpart public docs focus on feature availability and storage behavior, not processing p95.
15. Headline H-15 encrypted meetings: Zoom and Teams document restrictions; Oyatie must explicitly model recording/transcription/AI restrictions in encrypted mode.
16. Headline H-16 encrypted meeting status: partial because local ADR coverage exists but user-facing and benchmark matrices are incomplete.
17. Headline H-17 breakout rooms: Teams has a public 300-attendee restriction for breakout creation; Zoom and Google support breakout rooms with licensing/context limits.
18. Headline H-18 breakout status: Oyatie has API coverage but needs measured room creation and participant reassignment numbers.
19. Headline H-19 room hardware: competitors have mature room systems; Oyatie has no room hardware benchmark or compatibility artifact.
20. Headline H-20 room hardware status: gap.
21. Headline H-21 client OS coverage: competitors cover mainstream web/mobile/desktop surfaces; Oyatie lacks service-local supported OS evidence.
22. Headline H-22 client status: gap until client artifacts and `supported-oses.json` exist.
23. Headline H-23 public-cloud elasticity: Oyatie can target high scale if OpenTofu modules and benchmarks prove it.
24. Headline H-24 public-cloud status: target only because meet path has no canonical OpenTofu modules.
25. Headline H-25 OCI Always Free profile: demo_trial should cap concurrency and duration because 4 OCPU, 24 GB memory, 200 GB block, 20 GB object, and 10 Mbps load balancer are materially constrained.
26. Headline H-26 OCI Always Free status: not a parity environment; it is a low-cost admission profile for demo_trial.
27. Headline H-27 paid tenant_class: should meet full canonical target if substrate is sized.
28. Headline H-28 paid status: target only because tenant_class contracts are absent.
29. Headline H-29 revenue_share tenant_class: should meet full canonical target while exposing unit economics.
30. Headline H-30 revenue_share status: target only because revenue_share semantics are absent.
31. Headline H-31 compliance operations: Oyatie can exceed generic meeting tools through legal hold and disclosure endpoints if implemented.
32. Headline H-32 compliance status: ahead target, partial proof.
33. Headline H-33 deployment context breadth: Oyatie ambition exceeds standard SaaS competitors by including customer-owned and provider contexts.
34. Headline H-34 deployment breadth status: gap because the modules are missing.
35. Headline H-35 benchmark confidence: current confidence is medium for product-target clarity and low for measured completion.
36. Headline H-36 top remediation: build the benchmark harness, then report measured results by context, OS, architecture, and tenant_class.
37. Headline H-37 top evidence rule: no future report should claim measured p50/p95/p99 values without command output and run metadata.
38. Headline H-38 top product rule: do not convert infrastructure caps into feature-quality stratification.
39. Headline H-39 top architecture rule: reject new sessions before admitted users experience below-target media quality.
40. Headline H-40 final performance verdict: Meet has plausible industry-leader targets, but current artifacts do not provide measured benchmark evidence.

## 5. Required Benchmark Evidence Before Claiming Completion

1. Evidence E-01: load-test harness path exists and is runnable from the repository.
2. Evidence E-02: harness records git SHA, service version, OpenTofu plan ID, deployment context, tenant_class, OS, architecture, CPU, memory, storage, region/facility, and network path.
3. Evidence E-03: room-create p50/p95/p99 from at least 10,000 operations.
4. Evidence E-04: join p50/p95/p99 from at least 10,000 joins across warm and cold sessions.
5. Evidence E-05: media one-way p50/p95/p99 from synthetic clients across intra-region and inter-region paths.
6. Evidence E-06: packet loss, jitter, and bitrate adaptation results.
7. Evidence E-07: caption p50/p95/p99 measured against final audio segment time.
8. Evidence E-08: transcript segment persistence p50/p95/p99.
9. Evidence E-09: recording readiness p50/p95/p99.
10. Evidence E-10: summary generation p50/p95/p99 for 15-minute, 60-minute, and 120-minute meetings.
11. Evidence E-11: broadcast fanout setup p50/p95/p99.
12. Evidence E-12: broadcast viewer join p50/p95/p99.
13. Evidence E-13: compliance hold p50/p95/p99.
14. Evidence E-14: disclosure export p50/p95/p99.
15. Evidence E-15: failover RTO proof.
16. Evidence E-16: RPO proof for meeting metadata and transcript records.
17. Evidence E-17: overload rejection test proving no admitted-session quality downgrade.
18. Evidence E-18: OCI Always Free profile cap test.
19. Evidence E-19: paid tenant_class scale test.
20. Evidence E-20: revenue_share unit-economics telemetry test.
21. Evidence E-21: encrypted-mode limitations test.
22. Evidence E-22: legal hold and recording/transcript retention test.
23. Evidence E-23: per-context OpenTofu plan/apply validation.
24. Evidence E-24: supported OS matrix smoke test.
25. Evidence E-25: final report with raw outputs committed or attached as evidence artifacts.

## 6. Non-Claims And Stop Conditions

1. Non-claim N-01: this report does not claim that Meet currently meets the p50, p95, or p99 targets.
2. Non-claim N-02: this report does not claim that the existing benchmark markdown is measured evidence.
3. Non-claim N-03: this report does not claim six deployment contexts are deployable from the current meet path.
4. Non-claim N-04: this report does not claim OCI Always Free profile viability for large meetings.
5. Non-claim N-05: this report does not claim room hardware, PSTN, CVI, NDI, or eCDN support exists in current artifacts.
6. Non-claim N-06: this report does not claim tenant_class semantics exist in current contracts.
7. Non-claim N-07: this report does not claim that counterpart vendors publish latency distributions comparable to Oyatie target metrics.
8. Non-claim N-08: this report does not claim that all counterpart product numbers are permanent; vendor limits must be refreshed before launch-gate comparison.
9. Stop S-01: stop any completion claim if `microservices/meet/benchmarks/meetbench/` or an equivalent harness remains absent.
10. Stop S-02: stop any completion claim if `supported-oses.json` remains absent.
11. Stop S-03: stop any completion claim if per-context OpenTofu modules remain absent.
12. Stop S-04: stop any completion claim if demo_trial, paid, and revenue_share overlays remain absent.
13. Stop S-05: stop any completion claim if recording/transcript storage ownership remains unresolved.
14. Stop S-06: stop any completion claim if encrypted-mode recording/transcription/AI limitations remain unspecified.
15. Stop S-07: stop any completion claim if benchmark output lacks deployment context.
16. Stop S-08: stop any completion claim if benchmark output lacks tenant_class.
17. Stop S-09: stop any completion claim if benchmark output lacks OS and architecture.
18. Stop S-10: stop any completion claim if admitted-session media quality is allowed to degrade instead of rejecting new over-capacity admissions.
19. Refresh R-01: refresh Zoom public numbers from official support docs before any external-facing claim.
20. Refresh R-02: refresh Google Meet public numbers from Workspace Admin Help before any external-facing claim.
21. Refresh R-03: refresh Microsoft Teams public numbers from Microsoft Learn before any external-facing claim.
22. Refresh R-04: refresh OCI Always Free profile numbers from Oracle documentation before any external-facing demo_trial sizing claim.
23. Refresh R-05: refresh internal capacity targets after the first measured SFU, TURN, recording, transcript, and broadcast load tests.
24. Evidence E-26: store raw benchmark outputs as immutable artifacts tied to git SHA and OpenTofu plan ID.
25. Evidence E-27: include client logs and server logs for failed joins, delayed captions, recording delays, and broadcast fanout failures.
26. Evidence E-28: include network impairment scenarios with packet loss, jitter, bandwidth collapse, and TURN relay fallback.
27. Evidence E-29: include cost telemetry for egress, recording storage, transcript processing, summary generation, and support load.
28. Evidence E-30: include compliance-mode tests for legal hold, deletion hold, export, audit event latency, and disclosure access.
29. Evidence E-31: include demo_trial cap-hit tests that prove user-facing rejection is clean and auditable.
30. Evidence E-32: include paid and revenue_share tests that prove scale follows deployed capacity and contract terms.
31. Evidence E-33: include on-prem and colo facility prerequisites before applying standard latency and availability targets.
32. Evidence E-34: include a view-only and interactive split in every large-event benchmark.
33. Evidence E-35: include separate encrypted-mode benchmark rows because some recording, transcript, and AI features may be unavailable by policy.
34. Final stop condition: until this evidence exists, the correct status is target-defined and audit-ready, not benchmark-proven.
35. Final report posture: this document gives the numbers future implementation must prove; it is not itself proof of runtime performance.
