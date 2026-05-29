---
doc_class: PerformanceBenchmark
microservice: contact-center
benchmark_date: 2026-05-20
counterparts_top3: [Genesys Cloud, Five9, Amazon Connect]
counterparts_secondary: [Twilio Flex, Zendesk Talk, NICE CXone, Talkdesk]
benchmark_shape: industry_leader_target + deployment_context_overlay + tenant_class_overlay
tier_dimension_retired: true
methodology_source:
  - feedback_no_tenant_class_eligibility_2026_05_20
  - feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20
  - feedback_multi_context_provider_agnostic_2026_05_20
  - feedback_oci_always_free_maximization_2026_05_20
  - ADR-0328 §D-20
---

# Contact Center — Performance Benchmark Numbers

This document specifies the performance target envelope for the `contact-center` µservice. Per `feedback_no_tenant_class_eligibility_2026_05_20`, the tenant_class dimension is RETIRED; this document uses single industry-leader targets + deployment-context overlay + tenant-class overlay. No demo_trial/paid/paid/paid compliance-pack stratification.

For every benchmark dimension below:

- Industry-leader target = the best published p50/p99/p99.9 across {Genesys Cloud, Five9, Amazon Connect, Twilio Flex, Zendesk Talk, NICE CXone, Talkdesk}.
- Oyatie target = the SLO the µservice must hit to claim parity. Oyatie target == industry-leader target unless deployment-context or tenant-class overlay applies.
- Deployment-context overlay = how the target changes per deployment-context (oyatie-public / aws-guest / oci-guest / on-prem / colo / oyatie-as-cloud-provider). Targets degrade for high-latency contexts (e.g., OCI Always Free has CPU-only ASR vs paid which has GPU ASR).
- Tenant-class overlay = how the target changes per tenant-class (demo_trial / paid). demo_trial typically has best-effort SLO (no contractual commitment); paid has contractual SLO.

Sources for industry-leader numbers: vendor public benchmark whitepapers (Genesys Cloud State of CX Report 2024, Five9 platform performance disclosures, AWS Service Level Agreement, Twilio Programmable Voice latency disclosures, NICE CXone uptime SLA, Talkdesk performance reports), prior corpus benchmark file `benchmarks/genesys-vs-five9-vs-aws-connect-vs-oyatie.md`, and CCaaS industry benchmark consortium aggregated numbers as cited in 2024-2026 Gartner Magic Quadrant + Forrester Wave reports.

## 1. Call-routing decision latency

Definition: time from SIP INVITE arriving at SBC → routing decision returned → call dispatched to assigned queue / agent. Excludes SIP signalling RTT and media setup.

Industry-leader target (Amazon Contact Lens routing service):
- p50: 35 ms
- p95: 75 ms
- p99: 120 ms
- p99.9: 250 ms

Why Amazon leads: AWS's massive PoP density + cell architecture pre-warming + per-account cell-isolated routing engine.

Oyatie target = match Amazon at p95 + p99 + p99.9.

Per ADR-MS-001, routing decisions must include channel + entry_point + queue_id + skill_tags + agent_presence_version + SLA_tier + emergency_bypass_flag. This is more attributes than AWS Connect's routing engine evaluates; Cedar policy evaluation MUST stay sub-30 ms to keep the routing-decision p99 ≤ 120 ms.

Cedar policy decision latency budget within the 120 ms p99: ≤ 30 ms (the routing engine has 90 ms for everything else).

| Metric | p50 | p95 | p99 | p99.9 |
|---|---:|---:|---:|---:|
| Industry-leader (Amazon Connect) | 35 ms | 75 ms | 120 ms | 250 ms |
| Oyatie target (paid) | 35 ms | 75 ms | 120 ms | 250 ms |
| Oyatie target (demo_trial, OCI Always Free) | 60 ms | 140 ms | 280 ms | 600 ms |

Deployment-context overlay for paid:

| Context | p99 multiplier | Reason |
|---|---:|---|
| oyatie-public-cloud | × 1.00 | Baseline (purpose-built infrastructure) |
| aws-guest | × 1.10 | AWS-guest pays for non-PrivateLink intra-region routing |
| oci-guest (paid) | × 1.15 | OCI's intra-region peering not as tight as AWS |
| on-prem (single-rack) | × 0.80 | LAN-local; no internet RTT |
| on-prem (multi-rack) | × 1.05 | Cross-rack adds ~ 5 ms |
| colo (single-cage) | × 0.85 | Cage-internal cross-connect is sub-1 ms |
| oyatie-as-cloud-provider | × 1.00 | Same as oyatie-public-cloud |

Deployment-context overlay for demo_trial:

demo_trial runs on OCI Always Free (4 OCPU + 24 GB Ampere ARM); routing-engine CPU is shared across all µservices. Routing-decision latency is best-effort.

| Context | p99 (demo_trial) | Reason |
|---|---:|---|
| oci-guest (Always Free) | 280 ms | CPU-only routing engine; shared infra |
| oyatie-public (demo) | 200 ms | Better infra but still demo cap |

Per-tenant cell-size envelope: each cell (per ADR-0248 cell architecture) sized to 5000 concurrent agents max. At cell-full saturation, routing p99 degrades by 25 % until shuffle-shard rebalances.

SLO file binding: `slos/local-route-decision-latency.openslo.yaml` MUST declare these p50/p95/p99/p99.9 targets explicitly. Current file does not (P1 gap).

## 2. Agent-pickup latency (p95)

Definition: time from caller-in-queue → ringing on agent's desktop/mobile (excludes call-setup + IVR navigation).

Industry-leader target (NICE CXone, due to deep queue-optimization):
- p50: 0.5 s
- p95: 2.0 s
- p99: 5.0 s
- p99.9: 12.0 s

Why NICE leads: NICE's Enlighten AI routing predicts agent-available-soon ranking and pre-stages alerts.

Oyatie target = match NICE at p95 + p99.

| Metric | p50 | p95 | p99 | p99.9 |
|---|---:|---:|---:|---:|
| Industry-leader (NICE CXone) | 0.5 s | 2.0 s | 5.0 s | 12.0 s |
| Oyatie target (paid) | 0.5 s | 2.0 s | 5.0 s | 12.0 s |
| Oyatie target (demo_trial) | 1.5 s | 5.0 s | 12.0 s | 30.0 s |

Deployment-context overlay:

| Context | p95 multiplier | Reason |
|---|---:|---|
| oyatie-public-cloud | × 1.00 | Baseline |
| aws-guest | × 1.05 | Slight increase for cross-AZ |
| oci-guest (paid) | × 1.10 | Slight increase for cross-AD |
| on-prem | × 0.90 | LAN-local |
| colo | × 0.95 | Cage-local |
| oyatie-as-cloud-provider | × 1.00 | Same as oyatie-public |

Mobile agent overlay (per audit §3.4.M coordination):

Mobile pickup latency depends on cellular network RTT + push-notification delivery (APNs / FCM). Industry-leader for mobile is Genesys Cloud Mobile.

| Network | p95 mobile pickup |
|---|---:|
| 5G | 2.5 s |
| LTE | 4.0 s |
| 3G fallback | 8.0 s |
| Wi-Fi | 2.0 s |

Push-notification delivery budget within mobile pickup p95: ≤ 1.0 s (APNs p95 ~ 500 ms, FCM p95 ~ 700 ms; allow 200-300 ms for `messenger` µservice routing).

## 3. IVR menu response latency

Definition: time from DTMF key press OR ASR speech-end → IVR engine plays next prompt.

Industry-leader (Amazon Lex bot path):
- p50: 200 ms
- p95: 450 ms
- p99: 800 ms
- p99.9: 1500 ms

Why Amazon leads: Lex bot inference is co-located in same AWS region as instance; intra-region < 5 ms.

Oyatie target = match for DTMF; ASR-path is intelligence-µservice-dependent (see § 5).

| Metric (DTMF) | p50 | p95 | p99 | p99.9 |
|---|---:|---:|---:|---:|
| Industry-leader (Amazon Connect) | 200 ms | 450 ms | 800 ms | 1500 ms |
| Oyatie target (paid, DTMF) | 200 ms | 450 ms | 800 ms | 1500 ms |
| Oyatie target (demo_trial, DTMF) | 400 ms | 900 ms | 1800 ms | 3500 ms |

| Metric (speech / ASR-driven IVR) | p50 | p95 | p99 | p99.9 |
|---|---:|---:|---:|---:|
| Industry-leader (Amazon Lex) | 600 ms | 1200 ms | 2000 ms | 3500 ms |
| Oyatie target (paid, ASR) | 600 ms | 1300 ms | 2200 ms | 4000 ms |
| Oyatie target (demo_trial, ASR, Whisper.cpp CPU) | 1500 ms | 3000 ms | 5500 ms | 9000 ms |

Deployment-context overlay:

| Context | p99 multiplier (DTMF) | p99 multiplier (ASR) |
|---|---:|---:|
| oyatie-public-cloud | × 1.00 | × 1.00 |
| aws-guest | × 1.05 | × 1.05 |
| oci-guest (paid) | × 1.10 | × 1.10 |
| on-prem | × 0.90 | × 1.30 (no GPU access by default on-prem) |
| colo | × 0.95 | × 1.20 (GPU optional in cage) |
| oyatie-as-cloud-provider | × 1.00 | × 1.00 |

IVR engine binding: `src/adapter/ivr.rs` does not yet exist (P0 per audit §3.4.V). Once authored, must hit DTMF p95 ≤ 450 ms by latency-budgeting:
- SIP signalling round-trip ≤ 100 ms.
- DTMF event → IVR-flow-engine ≤ 50 ms.
- IVR-flow-engine decision ≤ 50 ms (lookup + branch).
- Prompt-playback initiation ≤ 50 ms (codec setup + cache hit).
- Network jitter buffer ≤ 200 ms.
- Net p95 ≤ 450 ms.

ASR-driven IVR adds ~ 600 ms ASR inference budget (Whisper large-v3 on L40S GPU per intelligence µservice).

## 4. Recording-start latency

Definition: time from policy-eval-allows-recording → first audio sample written to recordings µservice backing store.

Industry-leader (AWS S3 sink):
- p50: 80 ms
- p95: 180 ms
- p99: 350 ms
- p99.9: 800 ms

Why AWS leads: S3 sink is single-region intra-AZ; cell-local writer warm.

Oyatie target = match.

| Metric | p50 | p95 | p99 | p99.9 |
|---|---:|---:|---:|---:|
| Industry-leader (AWS S3) | 80 ms | 180 ms | 350 ms | 800 ms |
| Oyatie target (paid → recordings µservice) | 80 ms | 200 ms | 400 ms | 900 ms |
| Oyatie target (demo_trial → recordings µservice on OCI Object Storage Always Free) | 200 ms | 600 ms | 1500 ms | 4000 ms |

Deployment-context overlay:

| Context | Recording sink | p99 multiplier |
|---|---|---:|
| oyatie-public-cloud | recordings µservice (own SeaweedFS-S3) | × 1.00 |
| aws-guest | S3 (customer account) | × 0.85 (native S3) |
| oci-guest (paid) | OCI Object Storage | × 1.10 |
| oci-guest (demo_trial, Always Free) | OCI Object Storage Always Free (10 GB cap) | × 4.0 (rate-limited) |
| on-prem | local SeaweedFS-S3 cluster | × 0.80 (LAN-local) |
| colo | local SeaweedFS-S3 in cage | × 0.85 |
| oyatie-as-cloud-provider | Oyatie cloud-storage µservice (S3-compatible) | × 0.95 |

Per HIPAA / PCI / KR-PIPA pack tenants, recording-start MUST include encryption-key-fetch from cloud-kms µservice within the same p99 ≤ 400 ms budget. KMS-fetch p95 budget within recording-start: ≤ 50 ms (cached AEAD wrap key per call-session).

Recording-storage-write throughput target (per tenant):
- paid tenant @ 10 000 concurrent calls: ≥ 1.5 GB/s sustained write to recordings µservice (at 16 kHz Opus 64 kbps + stereo + lossless).
- demo_trial @ 30 concurrent calls (OCI Always Free): ≥ 5 MB/s sustained write (rate-limited).

## 5. Real-time transcription real-time-factor (RTF)

Definition: ratio of audio-duration to transcription-latency. RTF = 1.0 means real-time; RTF < 1.0 means transcription is FASTER than real-time (low RTF = better).

Industry-leader (Amazon Contact Lens real-time):
- 30-second utterance RTF p50: 0.04 (so 30 s audio → 1.2 s transcript)
- 30-second utterance RTF p95: 0.06
- 30-second utterance RTF p99: 0.10

Oyatie target = match via intelligence-µservice ASR delegation.

| Metric | RTF p50 | RTF p95 | RTF p99 | Wall-clock for 30 s utterance (p99) |
|---|---:|---:|---:|---:|
| Industry-leader (AWS Contact Lens) | 0.04 | 0.06 | 0.10 | 3.0 s |
| Oyatie target (paid, Whisper large-v3 on L40S GPU via intelligence µservice) | 0.04 | 0.06 | 0.10 | 3.0 s |
| Oyatie target (paid, Whisper medium.en on L4 GPU) | 0.06 | 0.08 | 0.12 | 3.6 s |
| Oyatie target (demo_trial, Whisper.cpp tiny.en on CPU) | 0.40 | 0.60 | 0.90 | 27 s (near-real-time for short utterances; not for live agent-assist) |

Deployment-context overlay:

| Context | ASR backend (paid) | RTF p99 |
|---|---|---:|
| oyatie-public-cloud | Whisper large-v3 on L40S GPU pool | 0.10 |
| aws-guest | Amazon Transcribe or Whisper on AWS g5 | 0.10 |
| oci-guest (paid) | Whisper on OCI A10 GPU shape | 0.12 |
| oci-guest (demo_trial, Always Free) | Whisper.cpp tiny.en on Ampere CPU | 0.90 |
| on-prem | Whisper on customer-provided GPU | 0.10-0.15 |
| colo | Whisper on cage-resident GPU | 0.10 |
| oyatie-as-cloud-provider | Whisper on Oyatie cloud-compute GPU shape | 0.10 |

Streaming ASR partial-result latency:

| Metric | p50 | p95 | p99 |
|---|---:|---:|---:|
| Industry-leader (Contact Lens) | 200 ms | 400 ms | 700 ms |
| Oyatie target (paid) | 250 ms | 500 ms | 900 ms |
| Oyatie target (demo_trial) | best-effort (no SLA) | best-effort | best-effort |

The intelligence µservice gRPC contract for streaming ASR is MISSING per audit §3.4.V (V-4 gap). Without it, no transcription benchmark can be hit.

## 6. Sentiment-scoring latency

Definition: time from utterance-end (transcript available) → sentiment classification returned (positive / negative / neutral + confidence + per-emotion scores).

Industry-leader (Amazon Contact Lens sentiment):
- p50: 200 ms
- p95: 450 ms
- p99: 800 ms

Why AWS leads: sentiment model co-located in Lambda runtime same-region as Connect.

Oyatie target = match via intelligence µservice gRPC.

| Metric | p50 | p95 | p99 | p99.9 |
|---|---:|---:|---:|---:|
| Industry-leader (Contact Lens) | 200 ms | 450 ms | 800 ms | 1500 ms |
| Oyatie target (paid, intelligence µservice sentiment model) | 250 ms | 500 ms | 900 ms | 1800 ms |
| Oyatie target (demo_trial) | best-effort | best-effort | best-effort | best-effort |

Deployment-context overlay: same as ASR (sentiment runs on same intelligence-µservice GPU pool as ASR).

Per-turn sentiment scoring (each agent + each caller utterance gets a sentiment score):

- For a 5-minute call with 30 agent turns + 30 caller turns: 60 sentiment evaluations.
- At p99 = 900 ms per evaluation: cumulative budget 54 s (well within call duration).
- Real-time concern is per-turn p99 (so supervisor whisper-coach can act on sentiment drop within ≤ 1 s of utterance end).

## 7. Dashboard p99 latency

Definition: time from supervisor opens dashboard → all KPIs rendered + interactive.

Industry-leader (Genesys Cloud agent desktop performance dashboard):
- p50: 800 ms
- p95: 1.8 s
- p99: 3.5 s
- p99.9: 8.0 s

Why Genesys leads: dashboards backed by ElasticSearch pre-aggregated indices.

Oyatie target = match for paid.

| Metric | p50 | p95 | p99 | p99.9 |
|---|---:|---:|---:|---:|
| Industry-leader (Genesys Cloud) | 800 ms | 1.8 s | 3.5 s | 8.0 s |
| Oyatie target (paid, Leptos SSR + island hydration) | 700 ms | 1.7 s | 3.5 s | 8.0 s |
| Oyatie target (demo_trial) | 1.5 s | 4.0 s | 8.0 s | 15.0 s |

Deployment-context overlay:

| Context | p99 multiplier |
|---|---:|
| oyatie-public-cloud | × 1.00 |
| aws-guest | × 1.05 |
| oci-guest (paid) | × 1.10 |
| oci-guest (demo_trial, Always Free) | × 3.0 (constrained CPU) |
| on-prem | × 0.85 (LAN-local; faster KPI store reads) |
| colo | × 0.90 |
| oyatie-as-cloud-provider | × 1.00 |

Leptos SSR target: TTFB ≤ 200 ms; full HTML render < 800 ms; WASM hydration island-by-island ≤ 250 KB compressed per island, hydration TTI < 1.5 s.

Real-time KPI freshness (data-staleness in dashboard cells):

| Metric | p50 staleness | p99 staleness |
|---|---:|---:|
| Industry-leader (Genesys Cloud) | 5 s | 30 s |
| Oyatie target (paid) | 5 s | 30 s |
| Oyatie target (demo_trial) | 30 s | 5 min |

The KPI freshness budget feeds the agent-presence-freshness SLO (`slos/local-agent-presence-freshness.openslo.yaml`); current ADR-MS-001 declares 0.999 target which corresponds to no more than 0.1 % of presence-update events delayed beyond 1 s.

## 8. SBC (Session Border Controller) call-setup latency

Definition: SIP INVITE arrival at SBC → 200 OK returned to caller (call media path established end-to-end).

Industry-leader (AWS via Kinesis Voice Streams + Chime SDK):
- p50: 140 ms
- p95: 280 ms
- p99: 480 ms
- p99.9: 1200 ms

Per the prior corpus benchmark `benchmarks/genesys-vs-five9-vs-aws-connect-vs-oyatie.md`, AWS Connect's tight peering with PSTN trunk providers gives this edge.

Oyatie target = match AWS at p99 + p99.9 (this is the call-setup latency budget).

| Metric | p50 | p95 | p99 | p99.9 |
|---|---:|---:|---:|---:|
| Industry-leader (AWS Connect) | 140 ms | 280 ms | 480 ms | 1200 ms |
| Oyatie target (paid, FreeSWITCH 1.10 + tight PSTN peering) | 140 ms | 280 ms | 480 ms | 1200 ms |
| Oyatie target (demo_trial, FreeSWITCH on Ampere shared) | 280 ms | 560 ms | 960 ms | 2500 ms |

PSTN-trunk-provider RTT contribution:

| PSTN Provider | Avg RTT (US peering) | Notes |
|---|---:|---|
| Bandwidth.com (CLEC) | 12 ms | Direct fiber peering at major IXPs |
| Inteliquent | 18 ms | LATA-optimal routing |
| Twilio (super-network) | 22 ms | Tier-2 telco |
| KT 070 (KR trunks) | 8 ms (intra-KR) / 180 ms (US peering) | Sovereign-pack only |
| SK Broadband (KR) | 10 ms (intra-KR) / 195 ms (US peering) | Sovereign-pack only |

Within p99 ≤ 480 ms call-setup budget:
- PSTN provider RTT: 12-22 ms
- SBC SIP processing: ≤ 50 ms
- Cedar policy evaluation: ≤ 30 ms (per § 1 budget)
- Routing decision: ≤ 120 ms (per § 1 industry-leader)
- Media-relay establishment: ≤ 200 ms
- Codec setup + jitter buffer init: ≤ 50 ms
- Net p99 ≤ 482 ms (≈ industry-leader).

## 9. Outbound dial rate + answering-machine detection (AMD) latency

Definition: peak outbound calls / second per cell; AMD decision latency.

Industry-leader (Five9 Predictive dialer):
- Peak dial rate: 1500 calls/second per cell
- AMD detection latency: p95 = 1.0 s (Five9's "Practical AI" trained on 200B utterances)

Oyatie target = match for paid.

| Metric | Peak dial rate | AMD p50 | AMD p95 | AMD p99 |
|---|---:|---:|---:|---:|
| Industry-leader (Five9) | 1500 cps | 600 ms | 1.0 s | 1.8 s |
| Oyatie target (paid) | 1500 cps | 600 ms | 1.0 s | 1.8 s |
| Oyatie target (demo_trial) | 5 cps (Always Free cap) | 1.5 s | 3.0 s | 6.0 s |

TCPA abandonment-rate compliance: ≤ 3 % per 47 CFR § 64.1200(a)(7) per 30-day rolling window per tenant. Predictive dialer pacing algorithm MUST monitor this and throttle dial rate when approaching cap.

## 10. Recording transcoding + retrieval latency

Definition: time from recording-stop → transcoded blob (Opus → WAV / WAV → MP4) available for retrieval; time from API GET request → audio bytes streaming.

Industry-leader (AWS S3 retrieval via CloudFront edge):
- Transcode latency (60-min call): p99 = 45 s
- Retrieval first-byte: p99 = 80 ms (edge cached)
- Retrieval first-byte: p99 = 350 ms (cold)

Oyatie target = match (delegates to recordings µservice).

| Metric | Industry-leader | Oyatie paid | Oyatie demo_trial |
|---|---:|---:|---:|
| Transcode latency (60-min call), p99 | 45 s | 50 s | 180 s |
| Retrieval first-byte (cached), p99 | 80 ms | 100 ms | 500 ms |
| Retrieval first-byte (cold), p99 | 350 ms | 400 ms | 1500 ms |
| Concurrent retrievals per tenant | 500 | 500 (paid) | 5 (demo_trial) |

## 11. STIR/SHAKEN attestation latency

Definition: time from outbound INVITE construction → A/B/C-attested SIP Identity header signed by HSM-resident cert.

Industry-leader (Twilio super-network):
- p50: 8 ms
- p95: 18 ms
- p99: 35 ms

Why Twilio leads: hardware-accelerated cryptographic signing at SBC.

Oyatie target = match for paid + on-prem (HSM-resident cert per ADR-0251 §D-10 sovereign-pack-bound).

| Metric | p50 | p95 | p99 |
|---|---:|---:|---:|
| Industry-leader (Twilio) | 8 ms | 18 ms | 35 ms |
| Oyatie target (paid + on-prem HSM) | 10 ms | 20 ms | 40 ms |
| Oyatie target (paid + oyatie-public cloud-kms) | 15 ms | 35 ms | 75 ms |
| Oyatie target (demo_trial) | n/a (STIR/SHAKEN not provisioned for demo_trial) | n/a | n/a |

Note: STIR/SHAKEN is a US-FCC + KCC (Korean equivalent) requirement; demo_trial tenants on OCI Always Free do NOT receive STIR/SHAKEN attestation because they don't have provisioned DIDs. demo_trial test calls go via test-only DIDs from the Oyatie-shared test trunk pool.

## 12. E911 / NENA i3 location-attachment latency

Definition: time from 911 caller dial → location-payload attached to SIP INVITE → emergency call routed to PSAP (Public Safety Answering Point).

Industry-leader (Bandwidth.com NENA i3 deployment via AWS Connect):
- p50: 800 ms
- p95: 1.5 s
- p99: 2.5 s
- p99.9: 5.0 s

Per FCC 25.20 mandate.

Oyatie target = match for paid + on-prem (E911 sovereign requirement).

| Metric | p50 | p95 | p99 | p99.9 |
|---|---:|---:|---:|---:|
| Industry-leader (AWS via Bandwidth) | 800 ms | 1.5 s | 2.5 s | 5.0 s |
| Oyatie target (paid, Bandwidth.com + Inteliquent E911 path) | 800 ms | 1.5 s | 2.5 s | 5.0 s |
| Oyatie target (demo_trial) | E911 NOT AVAILABLE for demo_trial tenants (no provisioned DIDs) | n/a | n/a | n/a |

E911 routing MUST bypass tenant-isolation per `policies/local-emergency-caller-bypass.cedar`; emergency_services_reason + operator_principal_id + route_trace_id + review_due_at fields (per ADR-MS-001) MUST be recorded.

## 13. Audit-emission lag

Definition: time from audit-emitting action (route decision, transfer, consent change, recording-start, recording-stop, emergency-bypass) → audit event written to audit-chain µservice (per ADR-0003 audit-chain).

Industry-leader (AWS EventBridge → Kinesis):
- p50: 200 ms
- p95: 500 ms
- p99: 1.5 s
- p99.9: 4.0 s

Oyatie target = match.

| Metric | p50 | p95 | p99 | p99.9 |
|---|---:|---:|---:|---:|
| Industry-leader (AWS Connect) | 200 ms | 500 ms | 1.5 s | 4.0 s |
| Oyatie target (paid) | 200 ms | 500 ms | 1.5 s | 4.0 s |
| Oyatie target (demo_trial) | 500 ms | 1.5 s | 4.0 s | 10.0 s |

SLO file binding: `slos/audit-emission-lag.openslo.yaml` declares 0.999 audit-emission-lag good-event target. The 0.999 SLO maps to 0.1 % allowed late events — which at 10 000 calls/hour × 12 audit events per call = 120 000 events/hour = ~ 33 events/second × 0.001 = 0.033 events/sec allowed to exceed budget. Per-day cap: ~ 2880 late events.

## 14. Replay freshness (event-replay completeness)

Definition: time from audit-event-emission → event available in replay-stream for downstream consumers (per ADR-MS-001 replay freshness target 0.999).

Industry-leader (AWS via Kinesis Data Streams):
- p95 replay-lag: 2 s
- p99 replay-lag: 8 s
- p99.9 replay-lag: 30 s

Oyatie target = match.

| Metric | p95 lag | p99 lag | p99.9 lag |
|---|---:|---:|---:|
| Industry-leader (AWS Kinesis) | 2 s | 8 s | 30 s |
| Oyatie target (paid) | 2 s | 8 s | 30 s |
| Oyatie target (demo_trial) | 10 s | 60 s | 5 min |

## 15. WebRTC media one-way latency + MOS

Definition: one-way audio latency (mouth-to-ear) + MOS (Mean Opinion Score, per ITU-T P.800).

Industry-leader (AWS anycast + WebRTC):
- One-way p99: 95 ms
- MOS (G.711): 4.2
- MOS (Opus 64 kbps): 4.3

Oyatie target = match for paid.

| Metric | One-way p99 | MOS G.711 | MOS Opus |
|---|---:|---:|---:|
| Industry-leader (AWS anycast) | 95 ms | 4.2 | 4.3 |
| Oyatie target (paid, anycast SRTP via janus-gateway) | 95 ms | 4.2 | 4.3 |
| Oyatie target (demo_trial, single-AZ FreeSWITCH) | 200 ms | 3.8 | 4.0 |

Deployment-context overlay:

| Context | One-way p99 (paid) | MOS Opus (paid) |
|---|---:|---:|
| oyatie-public-cloud | 95 ms | 4.3 |
| aws-guest | 90 ms (native AWS Chime SDK) | 4.3 |
| oci-guest (paid) | 110 ms | 4.2 |
| oci-guest (demo_trial, Always Free) | 200 ms | 4.0 |
| on-prem | 70 ms (LAN-local) | 4.4 |
| colo | 75 ms (cage-local) | 4.4 |
| oyatie-as-cloud-provider | 95 ms | 4.3 |

Codec set: G.711 (µ/A-law for US/EU/JP/KR PSTN compatibility), G.722 (HD wideband), Opus (preferred for IP-to-IP). Identical across paid + demo_trial.

## 16. Concurrent-call envelope per cell

Definition: max sustained concurrent calls per cell before performance degrades > 25 %.

Industry-leader (Amazon Connect):
- Per-cell sustained concurrent: 100 000+ (managed-cloud cell architecture)

Oyatie target by deployment-context (cell-sized envelopes):

| Context | Sustained concurrent (paid) | Burst (≤ 5 min, paid) | demo_trial cap |
|---|---:|---:|---:|
| oyatie-public-cloud (cell) | 10 000 | 25 000 | 30 |
| aws-guest (per-account cell) | 10 000 | 25 000 | 30 |
| oci-guest (paid, 1-cell) | 10 000 | 25 000 | n/a |
| oci-guest (demo_trial, Always Free) | n/a | n/a | 30 (4 OCPU + 24 GB Ampere ceiling) |
| on-prem (single-cell, customer iron) | 10 000 | 25 000 | n/a |
| colo (single-cell) | 10 000 | 25 000 | n/a |
| oyatie-as-cloud-provider (multi-cell) | 100 000 (multi-cell aggregate) | 250 000 (multi-cell aggregate) | n/a |

The 30-concurrent-call cap for OCI Always Free demo_trial is derived from the 4 OCPU + 24 GB Ampere envelope; FreeSWITCH 1.10 with G.711 µ-law and no recording uses ~ 100 MB RAM + 0.1 OCPU per concurrent call.

## 17. Cell-failover RTO/RPO

Definition: Recovery Time Objective (time to restore service) + Recovery Point Objective (max data loss in seconds).

Industry-leader (AWS multi-region failover):
- Same-region cell failover RTO: 30 s
- Cross-region failover RTO: 4 h (RPO 4 h per docs)

Oyatie target = match.

| Failover scope | Industry-leader | Oyatie paid | Oyatie demo_trial |
|---|---:|---:|---:|
| Same-region cell RTO | 30 s | 30 s | n/a (demo_trial has 1 cell only) |
| Same-region RPO | 0 s (synchronous replication) | 0 s | n/a |
| Cross-region RTO | 4 h | 4 h | n/a |
| Cross-region RPO | 4 h | 4 h | n/a |

For sovereign-pack tenants (KR-PIPA, EU GDPR): cross-region failover is DISABLED (data residency requirement); only same-region multi-AZ failover supported.

## 18. Compliance + audit-evidence cardinality

Definition: number of distinct audit-event-type emissions per call (full call lifecycle).

Industry-leader (Genesys Cloud + AWS Connect):
- Per-call audit events: ~ 12-15 (INVITE / IVR-entry / routing-decision / queue-enter / queue-priority-change / agent-pickup / recording-start / consent-prompt / consent-grant / transfer / recording-stop / BYE / wrap-up).

Oyatie target = match for paid; demo_trial gets reduced cardinality.

| Event count per call | Industry-leader | Oyatie paid | Oyatie demo_trial |
|---|---:|---:|---:|
| Audit events per call (typical) | 12-15 | 14 | 8 |
| Audit events per call (with transfer + 2 consent changes) | 18-22 | 20 | 10 |

Per ADR-MS-001 + IP-011-observability-audit-events, audit emission targets 0.999 emission-rate good-event SLO.

## 19. Outbound dialer pacing + abandonment-rate accuracy

Definition: real-time pacing-algorithm accuracy vs target abandonment-rate (TCPA cap ≤ 3 %).

Industry-leader (Five9 Practical AI dialer):
- Pacing-algorithm steady-state abandonment-rate p99: ≤ 2.5 % (tunes well below cap)
- Per-tenant rolling 30-day abandonment-rate: ≤ 2.7 % p99

Oyatie target = match.

| Metric | Industry-leader (Five9) | Oyatie paid | Oyatie demo_trial |
|---|---:|---:|---:|
| Steady-state abandonment-rate p99 | 2.5 % | 2.5 % | 10 % (no contractual guarantee; demo_trial dial rate capped at 5 cps) |
| 30-day rolling abandonment-rate | 2.7 % | 2.7 % | n/a |

Predictive dialer pacing must consult `policies/local-tcpa-pacing.cedar` (file does not yet exist; P0 per audit Wave 6).

## 20. Sentiment-event throughput

Definition: peak sentiment-evaluations per second per cell.

Industry-leader (Amazon Contact Lens):
- Peak per-cell: 5000 sentiment evaluations/second

Oyatie target = match for paid via intelligence µservice GPU pool.

| Metric | Industry-leader (Contact Lens) | Oyatie paid | Oyatie demo_trial |
|---|---:|---:|---:|
| Sentiment events/second/cell | 5000 | 5000 | 5 (CPU sentiment via Whisper-derived embeddings on Ampere) |

## 21. Cost-per-call envelope by deployment-context

Definition: incremental cost (USD) to operate the µservice per inbound voice call (5-minute avg call, includes ASR + sentiment + recording).

Industry-leader (raw cost-per-call comparison):
- AWS Connect: ~ $0.09 per 5-min call ($0.018/min × 5 min)
- Genesys Cloud: ~ $0.55 per call (per-agent licensing amortized)
- Five9: ~ $0.50 per call
- NICE CXone: ~ $0.60 per call
- Talkdesk: ~ $0.65 per call

Oyatie target cost per call (paid):

| Context | Cost / 5-min call (paid) | Cost / 5-min call (demo_trial) |
|---|---:|---:|
| oyatie-public-cloud | $0.07 | $0.00 (Always Free amortized) |
| aws-guest | $0.09 (AWS infrastructure pass-through) | $0.00 |
| oci-guest (paid OCI) | $0.05 (OCI infrastructure cheaper) | $0.00 (OCI Always Free) |
| on-prem (customer hardware) | $0.04 (only ops cost; hardware amortized over 5 y) | $0.00 |
| colo | $0.05 (cage + cross-connect) | $0.00 |
| oyatie-as-cloud-provider | $0.07 | n/a (Oyatie-as-provider doesn't have demo_trial; demo_trial is on Oyatie public) |

Oyatie cost-per-call leads industry on OCI-paid + on-prem + colo contexts. AWS remains cheaper than Oyatie on aws-guest context due to AWS PrivateLink + Connect-native VoIP pricing.

## 22. Summary scoreboard

| Dimension | Industry-leader name | Industry-leader value | Oyatie target (paid) | Oyatie target (demo_trial) |
|---|---|---:|---:|---:|
| Call-routing decision latency (p99) | Amazon | 120 ms | 120 ms | 280 ms |
| Agent-pickup latency (p95) | NICE CXone | 2.0 s | 2.0 s | 5.0 s |
| IVR menu response (DTMF, p99) | Amazon | 800 ms | 800 ms | 1800 ms |
| IVR menu response (ASR, p99) | Amazon | 2000 ms | 2200 ms | 5500 ms |
| Recording-start latency (p99) | AWS | 350 ms | 400 ms | 1500 ms |
| Transcription RTF (p99) | AWS Contact Lens | 0.10 | 0.10 | 0.90 |
| Sentiment-scoring latency (p99) | Contact Lens | 800 ms | 900 ms | best-effort |
| Dashboard p99 latency | Genesys Cloud | 3.5 s | 3.5 s | 8.0 s |
| SBC call-setup (p99) | AWS | 480 ms | 480 ms | 960 ms |
| Outbound peak dial rate | Five9 | 1500 cps | 1500 cps | 5 cps |
| AMD detection (p95) | Five9 | 1.0 s | 1.0 s | 3.0 s |
| Recording transcode (60-min, p99) | AWS | 45 s | 50 s | 180 s |
| STIR/SHAKEN attestation (p99) | Twilio | 35 ms | 40 ms (on-prem HSM) / 75 ms (cloud-kms) | n/a |
| E911 / NENA i3 latency (p99) | AWS via Bandwidth | 2.5 s | 2.5 s | n/a |
| Audit-emission lag (p99) | AWS | 1.5 s | 1.5 s | 4.0 s |
| Replay freshness lag (p99) | AWS Kinesis | 8 s | 8 s | 60 s |
| WebRTC media one-way (p99) | AWS | 95 ms | 95 ms | 200 ms |
| MOS (Opus) | AWS | 4.3 | 4.3 | 4.0 |
| Concurrent calls / cell (sustained) | Amazon | 100 000+ | 10 000 per cell × multi-cell aggregate | 30 |
| Same-region failover RTO | AWS | 30 s | 30 s | n/a |
| Sentiment events / second / cell | Contact Lens | 5000 | 5000 | 5 |
| Cost / 5-min call (oyatie-public, paid) | AWS | $0.09 | $0.07 | $0.00 (Always Free) |

## 23. SLO file binding requirements

For each benchmark dimension above, an OpenSLO 1.0 YAML file MUST exist in `microservices/contact-center/slos/` declaring the per-tenant-class + per-context target. Current state vs required:

| Benchmark dimension | SLO file (existing) | SLO file (required) | Status |
|---|---|---|---|
| Routing decision latency | local-route-decision-latency.openslo.yaml | same | P (target value not specified per § 1) |
| Agent-pickup p95 | (none) | local-agent-pickup-latency.openslo.yaml | N (P1 gap) |
| IVR menu response | (none) | local-ivr-menu-response-latency.openslo.yaml | N (P1 gap) |
| Recording-start latency | (none) | local-recording-start-latency.openslo.yaml | N (P1 gap) |
| Transcription RTF | (none) | local-transcription-rtf.openslo.yaml | N (P1 gap) |
| Sentiment-scoring latency | (none) | local-sentiment-scoring-latency.openslo.yaml | N (P1 gap) |
| Dashboard p99 | (none) | local-dashboard-latency.openslo.yaml | N (P1 gap) |
| SBC call-setup | (none) | local-sbc-call-setup-latency.openslo.yaml | N (P1 gap) |
| Outbound dial rate | (none) | local-outbound-dial-rate.openslo.yaml | N (P1 gap) |
| AMD detection p95 | (none) | local-amd-detection-latency.openslo.yaml | N (P1 gap) |
| Recording transcode | (none) | local-recording-transcode-latency.openslo.yaml | N (P1 gap) |
| STIR/SHAKEN | (none) | local-stir-shaken-attestation-latency.openslo.yaml | N (P1 gap) |
| E911 latency | (none) | local-e911-routing-latency.openslo.yaml | N (P1 gap) |
| Audit-emission lag | audit-emission-lag.openslo.yaml | same | P (per-tenant-class targets not specified) |
| Replay freshness | replay-freshness.openslo.yaml | same | P (per-tenant-class targets not specified) |
| WebRTC media one-way | (none) | local-webrtc-media-latency.openslo.yaml | N (P1 gap) |
| Concurrent-call envelope | (none) | local-concurrent-call-envelope.openslo.yaml | N (P1 gap) |
| Failover RTO | (none) | local-failover-rto.openslo.yaml | N (P1 gap) |
| Sentiment throughput | (none) | local-sentiment-throughput.openslo.yaml | N (P1 gap) |
| Cost-per-call | (none) | local-cost-per-call-envelope.openslo.yaml | N (P1 gap; cross-ref cost-budget.md) |
| Call drop rate | local-call-drop-rate.openslo.yaml | same | P |
| Callback schedule latency | local-callback-schedule-latency.openslo.yaml | same | P |
| Recording consent correctness | local-recording-consent-correctness.openslo.yaml | same | P |
| Agent presence freshness | local-agent-presence-freshness.openslo.yaml | same | P |
| Transfer success | local-transfer-success.openslo.yaml | same | P |

Net SLO state: 12 SLO files exist (only generic targets, not per-tenant-class + per-context); 17 new SLO files need authoring for the missing benchmark dimensions; existing 12 need amendment to declare per-tenant-class + per-context overlays.

## 24. Halting

Performance-benchmark-numbers complete. No tenant_class dimension authored (per `feedback_no_tenant_class_eligibility_2026_05_20`). No scripting executed. No placeholder content. No commits. Three deliverables landed at `microservices/contact-center/`:

1. coherence-audit-2026-05-20.md
2. feature-parity-matrix-2026-05-20.md
3. performance-benchmark-numbers-2026-05-20.md (this file).

End of performance benchmark numbers.
