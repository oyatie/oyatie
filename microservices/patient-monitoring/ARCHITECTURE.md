# patient-monitoring — Architecture

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

**Status**: wave-15m-f-authoring
**Date**: 2026-05-21
**Owner**: axis-clinical-realtime
**Pair-read with**: PRD.md, manifest.json, ADR-MS-001..003

---

## 1. North-star architectural picture

The patient-monitoring µservice is shaped like the AWS clinical-grade real-time substrate
would be if AWS had built it for Philips/GE/Mindray-class on-prem ICU plus public-cloud RPM
from day one. Three orthogonal axes determine every component:

1. **Time-criticality axis** — sub-second alarm + waveform vs. minutes-class trending vs.
   hours-class cohort analytics.
2. **Deployment-context axis** — on-prem (sovereign + air-gapped) vs. public-cloud (RPM) vs.
   guest-on-AWS/OCI (hybrid).
3. **Compliance-pack axis** — HIPAA + FDA 21 CFR + IEC 62304 SaMD Class C + EU AI Act
   high-risk + KR PIPA + EU MDR + GxP CSV.

Every component lives at a specific (time, context, pack) intersection and is sized
accordingly. The streaming substrate is hot-path tuned (gRPC + FlatBuffers + zero-copy); the
analytics tier is cost-tuned (ClickHouse columnar with cohort-aggregate views); the audit
tier is durability-tuned (append-only chain in audit-chain µservice).

---

## 2. Component map

```
                                    ┌──────────────────────────────────────────┐
                                    │       External Devices + Wearables        │
                                    │  Philips IntelliVue / GE CARESCAPE /      │
                                    │  Mindray BeneVision / Masimo /            │
                                    │  Welch Allyn / Drager / Edwards /         │
                                    │  Apple HealthKit / Fitbit / Garmin /      │
                                    │  Withings / Oura / Dexcom / Abbott /      │
                                    │  Bluetooth GATT direct                    │
                                    └────────────────────┬─────────────────────┘
                                                         │
                            HL7v2 / FHIR / IEEE 11073 / Continua / Vendor APIs / GATT
                                                         │
                                                         ▼
   ┌─────────────────────────────────────────────────────────────────────────────────┐
   │                            DeviceInterop bounded context                          │
   │  ┌───────────────────┐  ┌───────────────────┐  ┌───────────────────────────────┐ │
   │  │  HL7v2 Listener   │  │ IEEE 11073 Driver │  │ Wearable Cloud-API Connectors │ │
   │  │  (ORU^R01 PCD)    │  │ (10101 + 20601)   │  │ (Apple/Fitbit/Garmin/Withings/│ │
   │  │                   │  │                   │  │  Oura/Dexcom/Abbott/Polar/    │ │
   │  │                   │  │                   │  │  Whoop/Samsung)               │ │
   │  └────────┬──────────┘  └─────────┬─────────┘  └────────────┬──────────────────┘ │
   │           │                       │                          │                    │
   │           ▼                       ▼                          ▼                    │
   │  ┌────────────────────────────────────────────────────────────────────────────┐  │
   │  │                      Device Registry (Postgres-16)                          │  │
   │  │  - vendor, model, serial, firmware, calibration, bound-bed                  │  │
   │  └────────────────────────────────────────────────────────────────────────────┘  │
   └────────────────────────────────────────────┬───────────────────────────────────┘
                                                │
                                                ▼ canonicalized samples + frames
   ┌─────────────────────────────────────────────────────────────────────────────────┐
   │            VitalSignsStream + Waveform bounded contexts (hot path)                │
   │  ┌──────────────────────────┐         ┌────────────────────────────────────────┐│
   │  │  VitalSignsService gRPC  │         │       WaveformService gRPC bidi         ││
   │  │  bidi (1-Hz numeric)     │         │       (50-1000 Hz; FlatBuffers payload) ││
   │  └─────────┬────────────────┘         └────────────────────┬───────────────────┘│
   │            │                                                │                    │
   │            ▼                                                ▼                    │
   │  ┌──────────────────────────┐         ┌────────────────────────────────────────┐│
   │  │ TimescaleDB hyper-table  │         │     Local-cell ring buffer (4h/bed)     ││
   │  │ continuous-aggregate     │         │     + ZSTD-FlatBuffers object archive   ││
   │  │ views (1m/5m/1h)         │         │     hot tier ⟶ warm tier ⟶ cold tier   ││
   │  └──────────────────────────┘         └────────────────────┬───────────────────┘│
   │                                                            │                    │
   │                                                            ▼                    │
   │                                          ┌─────────────────────────────────────┐│
   │                                          │  WaveformArchive bounded context     ││
   │                                          │  ClickHouse cold tier (cohort)       ││
   │                                          │  alarm-episode pin → 7Y              ││
   │                                          └─────────────────────────────────────┘│
   └─────────────────────────────────────────────────────────────────────────────────┘
                                                │
                                                ▼ events (AsyncAPI fan-out via stream-platform)
   ┌─────────────────────────────────────────────────────────────────────────────────┐
   │                  AlarmManagement + SmartAlarm bounded contexts                    │
   │  ┌──────────────────────────────────────────┐  ┌──────────────────────────────┐ │
   │  │ Smart-alarm rule engine                  │  │ Alarm routing + escalation    │ │
   │  │  - validity (lead_confidence ≥ 0.5)      │  │  - bedside → charge → on-call │ │
   │  │  - persistence (N samples)               │  │  - per-severity ladder        │ │
   │  │  - compound (HR ∧ SpO2 ∧ RR)             │  │  - Cedar-gated suppression    │ │
   │  │  - patient-specific thresholds           │  └──────────────────────────────┘ │
   │  │  - diurnal adaptation                    │                                    │
   │  │  - trend gating                          │                                    │
   │  │  - dedup (5-min rolling)                 │                                    │
   │  └──────────────────────────────────────────┘                                    │
   └─────────────────────────────────────────────────────────────────────────────────┘
                  │                                                       │
                  │ alarm-fired event                                      │ ack/suppress event
                  ▼                                                       ▼
   ┌──────────────────────────────────────────┐  ┌──────────────────────────────────┐
   │ MobileNotification bounded context        │  │ AuditChain µservice (append-only)│
   │  APNs / FCM / WebPush / SMS / Pager       │  │  HIPAA + 21 CFR + IEC 62304      │
   └──────────────────────────────────────────┘  └──────────────────────────────────┘
                  │
                  ▼
   ┌──────────────────────────────────────────┐
   │ Clinician devices (iOS / Android / WinUI3 │
   │  workstation / Linux SDL2 kiosk / macOS   │
   │  M5+ Apple Silicon kiosk)                  │
   └──────────────────────────────────────────┘

   ┌─────────────────────────────────────────────────────────────────────────────────┐
   │       DeteriorationPrediction + SepsisEarlyWarning bounded contexts (ML)         │
   │  ┌──────────────────────────────────┐   ┌────────────────────────────────────┐  │
   │  │  Rule-based: NEWS2/MEWS/PEWS/    │   │  ML-based: LightGBM-rs              │  │
   │  │  qSOFA/SOFA/APACHE-IV/SAPS-3     │   │  (Rothman/Epic-DI analog)            │  │
   │  └─────────────┬────────────────────┘   └───────────────┬────────────────────┘  │
   │                │                                         │                       │
   │                └───────────────────┬─────────────────────┘                       │
   │                                    ▼                                              │
   │              ┌─────────────────────────────────────────┐                          │
   │              │   ml-platform µservice (model serving + │                          │
   │              │   inference + federated training        │                          │
   │              │   substrate via secure aggregation)      │                          │
   │              └─────────────────────────────────────────┘                          │
   └─────────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Streaming substrate (the hot path)

### 3.1 Why gRPC + FlatBuffers (not Kafka-only)

A live ICU patient's electrocardiogram is sampled at 500 Hz (commonly) and must reach a
central station with jitter ≤ 8 ms. The naive choice — Kafka — adds 20-200 ms of broker
queueing variance at production-grade durability settings, which is unacceptable for the
patient-safety contract.

gRPC bidi streaming with FlatBuffers payload achieves:

- **Zero-copy deserialization** on both server and client (waveform sample arrays are
  contiguous in memory, no per-sample object allocation).
- **HTTP/3 + QUIC** (per ADR-0253) gives < 1 ms head-of-line blocking recovery on
  hospital-Wi-Fi mobile-device deliveries.
- **Server-driven push** without poll overhead.
- **Single hop** from acquisition node to consumer, no broker round-trip.

Kafka remains the AsyncAPI substrate for:

- **Cross-µservice fan-out**: vital.streamed event consumed by data-warehouse, audit-chain,
  analytics.
- **At-least-once delivery to ml-platform** for deterioration + sepsis inference.
- **Hard durability** for events that must survive any consumer crash.

The hot-path waveform stream is **best-effort low-latency** with the local-cell ring buffer
providing the durability backstop; the AsyncAPI fan-out is **at-least-once durable**.

### 3.2 Per-bed session lifecycle

```
Device attach → device-interop authenticate → vital-signs-session + waveform-session created
              ↓
       canonicalize → enrich (HLC + tenant + bed + channel meta)
              ↓
       fan-out: (a) gRPC bidi to subscribed clients
                (b) ring buffer (4h hot retention per cell)
                (c) AsyncAPI emit to stream-platform
                (d) smart-alarm engine evaluation
                (e) deterioration-prediction sampler (5-min cadence)
                (f) TimescaleDB write (1-min aggregate for vitals)
                (g) object-storage write (waveform batch every 1 s)
                ↓
       on alarm-fire: pin ± 30 s of waveform to 7Y archive
                ↓
       on session end: finalize, archive, emit session-ended event
```

### 3.3 Backpressure model

Per-channel watermark; if the downstream gRPC client falls > 2 s behind, the per-client
buffer triggers a lossy-policy decision:

- **Waveform**: drop oldest batch (alert is emitted, but a back-buffer pull is offered to
  the client).
- **Numeric vitals**: never drop — they're 1 Hz, easy to keep up.
- **Alarms**: never drop — fail-fast disconnect; client must reconnect.

The local-cell ring buffer is the source of truth for waveform replay; gRPC clients can
request a "from-replay" window to recover.

### 3.4 Cellular isolation (per ADR-0248)

Each ICU unit binds to a single cell (Tier-1 sovereign for federal/academic; Tier-2
city/AZ for general hospitals). Cross-cell fan-out is forbidden at runtime; cohort analytics
(TrendAnalytics) operates on cold-tier cross-cell aggregates only.

Cell migration is supported for capacity rebalancing. Migration protocol:

1. Source-cell continues serving for migration window (T_m, typically 30 minutes).
2. Destination-cell pre-warmed: device-registry replication, Cedar bundle activation,
   alarm-definition migration.
3. Dual-emit for `[T_m]`: source emits to both source-and-destination-cell consumers.
4. Cutover: device sessions reconnect to destination via the device-interop redirector.
5. Source-cell session terminates after destination confirms first-byte-delivered.

Migration guarantees **zero-waveform-loss** (alarm-episode pinning preserves all event
windows; the ring buffer in both cells absorbs any 5-second cutover variance).

---

## 4. Component decomposition

### 4.1 DeviceInterop

| Component | Role | Tech |
|---|---|---|
| HL7v2 Listener | Receive ORU^R01 messages, parse PCD segments | Rust + hl7-rs (or HAPI-equivalent in Rust) |
| IEEE 11073 Driver | Receive 10101 (point-of-care) + 20601 (PHD) | Rust + custom IEEE 11073 decoder |
| Continua PHD Bridge | Bluetooth GATT → Continua → canonical | Rust + BlueR (Bluetooth-LE in Rust) |
| Vendor connectors | Philips CareEvent, GE Unity, Mindray eGateway, Welch Allyn Connex, Masimo SafetyNet, Drager MEDIBUS, Edwards HemoSphere | Rust per-vendor SDK shim |
| Wearable connectors | Apple HealthKit, Fitbit, Garmin, Withings, Oura, Dexcom, Abbott LibreView, Polar, Whoop, Samsung Health | Rust HTTP + per-vendor SDK |
| Device Registry | Postgres-16 + Rust kernel | postgres + sqlx |

### 4.2 VitalSignsStream + Waveform

| Component | Role | Tech |
|---|---|---|
| VitalSignsService gRPC server | Bidi numeric streaming | Rust + tonic |
| WaveformService gRPC server | Bidi waveform streaming | Rust + tonic + flatbuffers |
| Local-cell ring buffer | 4h-per-bed durability backstop | Rust + mmap files on local SSD |
| Hot tier object writer | ZSTD-compressed batches to S3-compatible store | Rust + object_store + zstd-rs |
| Cold tier ETL | Periodic move to ClickHouse | Rust + clickhouse-rs |
| TimescaleDB writer | 1-min aggregates | Rust + sqlx + Timescale extension |
| FHIR Observation emitter | Per-minute aggregate FHIR resource write | Rust + fhir-rs internal lib |
| AsyncAPI emitter | vital.streamed + waveform.streamed | Rust + stream-platform binding |

### 4.3 AlarmManagement + SmartAlarm

| Component | Role | Tech |
|---|---|---|
| Rule engine | Validity + persistence + compound + diurnal + dedup | Rust + custom DSL evaluator |
| Cedar policy guard | Suppression authorization | Rust + cedar-policy-rs |
| Routing + escalation | Hop chain per severity | Rust + state-machine |
| Suppression ledger | Append-only justification log | Postgres-16 + audit-chain emit |
| Alarm definition store | Per-tenant alarm-definition catalog | Postgres-16 |

### 4.4 MobileNotification

| Component | Role | Tech |
|---|---|---|
| Channel registry | Per-clinician device-token registry | Postgres-16 |
| APNs dispatcher | iOS push | Rust + a2 (APNs lib) |
| FCM dispatcher | Android push | Rust + fcm-rust |
| WebPush dispatcher | Browser push | Rust + web-push |
| SMS dispatcher | Twilio + AWS SNS fallback | Rust HTTP |
| Pager gateway | Spok / Connect / Vocera | Rust HTTP / SOAP |
| Dispatch log | Per-event delivery tracking | Postgres-16 + audit-chain |

### 4.5 DeteriorationPrediction + SepsisEarlyWarning

| Component | Role | Tech |
|---|---|---|
| Rule scorer | NEWS2/MEWS/PEWS/qSOFA/SOFA/APACHE-IV/SAPS-3 | Rust |
| Feature builder | Roll-up of vital + lab + waveform features | Rust + ndarray |
| ML inference | LightGBM-rs forward pass | Rust |
| Calibration | Platt scaling | Rust |
| Lineage logger | Inputs + features + score per inference | Postgres-16 + audit-chain |
| Model registry binding | Pull model + card from ml-platform | Rust + grpc to ml-platform |
| Federated training client | Secure-aggregation client per tenant | Rust + ml-platform binding |

### 4.6 CodeBlue

| Component | Role | Tech |
|---|---|---|
| Activation FSM | manual / auto / suggested | Rust state-machine |
| Waveform pinner | ± 30 min snapshot to 7Y archive | Rust + object_store |
| Pager dispatcher | Highest-priority channel | Rust |
| Playback service | Post-event review | Rust + gRPC + flatbuffers |

### 4.7 TelemetryCoverage

| Component | Role | Tech |
|---|---|---|
| Lead-off detector | Per-channel state machine | Rust |
| Signal-quality scorer | Rolling-window quality | Rust + ndarray |
| Coverage emitter | telemetry.coverage.lost event | Rust + stream-platform |

### 4.8 CentralStation

| Component | Role | Tech |
|---|---|---|
| Unit-wide WebSocket / gRPC stream | Per-unit subscription | Rust + tonic |
| Render kiosk (Linux 4K) | SDL2 + GPU shader | Rust + SDL2 + wgpu |
| Render workstation (Windows) | WinUI 3 C#/.NET front-end + Rust gRPC client | C# + Rust |
| Render kiosk (macOS Apple Silicon) | Metal + SDL2 | Rust + SDL2 + Metal |
| Mobile clinician app | SwiftUI + Rust gRPC client via FFI | Swift + Rust |
| Android clinician app | Jetpack Compose + Rust gRPC client | Kotlin + Rust (frontend ok per global memory) |

### 4.9 RPM + WearableIntegration

| Component | Role | Tech |
|---|---|---|
| RPM enrollment service | Patient onboarding, consent capture | Rust |
| Wearable poller | Periodic poll of wearable cloud APIs | Rust + tokio + scheduler |
| Wearable webhook receiver | Push-mode wearable updates | Rust + axum/tonic |
| RPM portal (patient self-service) | B2C surface | Rust backend + frontend pack |
| RPM care coordinator dashboard | B2B surface | Rust backend + frontend pack |

### 4.10 ICUBundleCompliance

| Component | Role | Tech |
|---|---|---|
| Bundle observation writer | Per-bed bundle-element observations | Rust + Postgres-16 |
| Compliance scorer | Rolling per-shift score | Rust |
| Bundle-alert emitter | Overdue-element alerts | Rust + stream-platform |
| Quality measure export | CMS-format export | Rust + csv-rs |

---

## 5. Data architecture

### 5.1 Hot path

- **TimescaleDB hyper-table**: `vital_signs_samples` partitioned by tenant + bed + day;
  continuous-aggregate views at 1-min / 5-min / 1-hour / 1-day rollup.
- **Object storage hot tier**: per-tenant prefix; ZSTD-compressed FlatBuffers waveform
  batches; 7-day local retention.
- **Postgres-16**: device-registry, alarm-definitions, alarm-instances, alarm-suppressions,
  Cedar bundles cache, clinician-device-registrations, bundle-observations, RPM-enrollments.
- **Local-cell ring buffer**: mmap'd files on NVMe SSD; 4h-per-bed; per-bed slot allocated
  at session-start.

### 5.2 Warm path (7-30 days)

- Object storage warm tier (same store, lifecycle-policy-bound).
- TimescaleDB continuous aggregates retained at full resolution.

### 5.3 Cold path (30 days → 7Y)

- **ClickHouse**: compressed columnar; per-tenant database; tenant-scoped query enforcement
  per ADR-0244.
- Alarm-episode waveform: lossless reconstruction via FlatBuffers decimation marker; 7Y
  retention.
- Continuous waveform: 90-day retention then aged out (regulatory floor, not patient-safety
  driven).

### 5.4 Audit path

- **audit-chain µservice**: every alarm-fire, alarm-ack, alarm-suppress, code-blue,
  deterioration-score, sepsis-warning, RPM-consent, federated-training-opt-in, model-card-update,
  break-glass-emergency-view → hashed and chained.
- 10Y retention per HIPAA + EU AI Act baseline; 21Y for consent records.

### 5.5 Tenant scoping (ADR-0244)

Every row in every table carries `tenant_id`. Postgres RLS enforces this at the DB layer.
ClickHouse uses per-tenant databases. Object storage uses per-tenant prefixes. KMS keys are
per-tenant. Cedar policies cannot escape `tenant_id` scope.

---

## 6. Cellular topology (ADR-0248)

### 6.1 Tier mapping

| Tier | Hosts | Use |
|---|---|---|
| Tier-0 (oyatie-global) | small set of regional KMS + tenant-registry + identity hubs | identity + KMS escrow |
| Tier-1 (national/regional sovereign) | major sovereign customers (US-DoD, EU member-state sovereign, KR-MFDS-certified) | sovereign healthcare |
| Tier-2 (city/AZ) | regional hospital clusters | most acute hospital tenants |
| Tier-3 (edge / on-site) | per-hospital edge sidecars | device-acquisition pre-flight |
| Tier-4 (in-room) | bedside acquisition node sidecar | last-mile capture |

This µservice's primary deployment is **Tier-1** (sovereign) and **Tier-2** (city/AZ). The
Tier-3 edge handles device-acquisition pre-flight (the local-cell ring buffer lives here for
on-prem deployments). Tier-4 is reserved for in-room device sidecars (vendor-specific HL7v2
gateways co-located with the bedside).

### 6.2 Shuffle sharding

Per ADR-0248: each tenant is shuffle-sharded across a subset of cells in their Tier. For a
50,000-bed tenant, shuffle sharding spreads beds across N=4 cells in the tier; a single cell
failure impacts at most ~12,500 beds. Cross-cell promotion logic is in the cell µservice;
this µservice obeys cell assignments.

### 6.3 Cell migration

See §3.4. Zero-waveform-loss guarantee.

---

## 7. Cedar authority chain integration

### 7.1 Policy ingestion

Each tenant's Cedar bundle for patient-monitoring lives in the policy-engine µservice. This
µservice pulls the bundle at startup and on bundle-update events. Policies cover:

- nurse-can-view-bed-group: nurse principals scoped to assigned bed groups (active shift)
- physician-can-acknowledge-alarm: physician principals with bed-group authority
- alarm-suppression-requires-justification: any physician suppressing must include
  justification text + duration ≤ 4h
- rpm-patient-can-view-own: patient principals scoped to self
- rpm-caregiver-designated: caregiver scoped per consent-graph designation
- central-station-operator: multi-bed view + escalation; cannot suppress
- device-registry-clinical-engineer: clinical engineer scoped to tenant
- code-blue-activation-any-clinician: every authorized clinician may activate; no per-bed
  scoping (life-critical)

### 7.2 Eval performance

Cedar eval p99 ≤ 5 ms per check. The 5,000-bed-per-cell upper bound implies ~25,000 evals
per second per cell at peak (alarm-fire bursts), well within the policy-engine µservice
capacity.

### 7.3 Fail-safe behavior

- `alarm-fire delivery`: fail OPEN (deliver) per ADR-0332 §B-7. Rationale: a muted alarm
  is more dangerous than a permissive eval.
- `alarm-suppress`: fail CLOSED. Rationale: a suppression is more dangerous than a missed
  attempt; the clinician retries.
- `code-blue-activation`: fail OPEN. Rationale: never block a code-blue.
- `break-glass-view`: fail CLOSED. Rationale: never grant emergency-view on a failed eval.
- `device-registry-write`: fail CLOSED. Rationale: never let a faulty principal mutate
  device records.
- `rpm-consent-revocation`: fail CLOSED. Rationale: never let consent-revocation be missed.
- `federated-training-opt-in change`: fail CLOSED.

---

## 8. ML stack (ADR-MS-003 summary)

### 8.1 Model classes

- **Rothman-Index-analog**: composite deterioration score. Gradient-boosted ensemble.
- **Epic-DI-analog**: alternative deterioration score; ensemble.
- **Sepsis-watch**: sepsis prediction. Ensemble + clinical-rule guard.
- **NICU-apnea (roadmap)**: pediatric apnea-of-prematurity prediction.

### 8.2 Training

- Public + de-identified-tenant data: MIMIC-IV, eICU-CRD, plus consenting-tenant cohort.
- Federated training across consenting tenants via secure aggregation (no raw data leaves
  the tenant cell).

### 8.3 Inference

- Per-bed every 5 min; immediate recomputation on input change.
- LightGBM-rs forward pass: p99 ≤ 200 ms.
- Calibration via Platt scaling.

### 8.4 Lineage

Per-inference: input snapshot ID + feature vector + model version + score + calibration +
clinical-rule-augmentation outcome → audit-chain. 10Y retention per EU AI Act.

### 8.5 Model card

Maintained in `models/deterioration/MODEL-CARD.md` (added in IP-006). Covers training data,
performance per subgroup (age, sex, race, ethnicity, geography), known failure modes,
intended use, contraindicated use, monitoring metrics.

### 8.6 Human-in-the-loop

Every prediction is **advisory**. No automatic actuation. The system can suggest a
code-sepsis but the clinician must confirm. This satisfies EU AI Act human-oversight
requirement.

---

## 9. Storage budget per bed per day

| Class | Volume |
|---|---|
| Numeric vitals raw (1 Hz, ~30 channels) | ~30 MiB/day uncompressed; ~6 MiB compressed |
| Numeric vitals aggregates (1m/5m/1h) | ~2 MiB/day |
| Waveforms continuous (5 channels @ 500 Hz, 16-bit, ZSTD-3) | ~1.8 GiB/day |
| Alarm-episode waveforms (typical 5 episodes/day × 60 s × 5 channels) | ~30 MiB/day |
| Alarm event log | ~50 KiB/day |
| Deterioration score log | ~10 KiB/day (5-min cadence) |
| Bundle compliance observations | ~5 KiB/day |
| **Total hot** | ~1.9 GiB/day/bed |
| **Total cold (post-90-day rollup; alarm-episodes only)** | ~30 MiB/day/bed retained |

For a 5,000-bed-per-cell sizing: ~9.5 TiB/day hot ingest. Hot tier 7-day retention =
~66 TiB. Cold tier 7-year retention = ~75 TiB per cell.

---

## 10. Performance walkthrough (waveform end-to-end)

### 10.1 Path

```
Bedside device → IEEE 11073 frame → DeviceInterop driver → canonicalize → enrich
  → WaveformService gRPC bidi server → fan-out → mobile app + central station
```

### 10.2 Budget per hop

| Hop | Budget p99 |
|---|---|
| IEEE 11073 frame receive + parse | 10 ms |
| Canonicalize + enrich (HLC stamp, tenant scope, channel meta) | 5 ms |
| gRPC bidi enqueue | 2 ms |
| Network (QUIC, hospital LAN) | 3 ms |
| Client receive + render (clinician kiosk SDL2) | 30 ms (visual 25-Hz frame) |
| **Total to glass** | ~50 ms p99 typical; budget ceiling 250 ms |

Waveform jitter (variance between consecutive batches): ≤ 8 ms p99.

### 10.3 Failure / mitigation

- Network hiccup > 2 s: ring buffer absorbs; client gets replay on reconnect.
- Cell failure: shuffle-shard isolation limits blast radius; tenant rebind to a healthy
  cell via cell µservice.
- gRPC server pod restart: in-flight streams reconnect within 1 s via QUIC connection
  resumption.

---

## 11. Observability

### 11.1 Metrics

Per ADR-0130 SLO-gated promotion:

- Vital-signs streaming latency histogram (per channel class)
- Waveform jitter histogram
- Alarm-fire counter (per priority, per unit, per tenant)
- Alarm-fatigue rolling 24h count
- Deterioration-score distribution
- Sepsis-watch sensitivity / specificity (computed from labeled outcomes)
- Coverage-loss frequency
- Cedar-eval latency
- Federated-training round-trip
- RPM ingest latency per wearable class

### 11.2 Traces

- OpenTelemetry traces span: device-receipt → canonicalize → gRPC fan-out → client
  receive (where possible across HTTP/3).
- Alarm-fire trace: rule-eval → routing → notification dispatch → client ack.

### 11.3 Logs

- Structured JSON; clinician-identity always redacted at log layer (PHI handling).
- 10Y retention for clinical actions; 90D for routine debug logs.

### 11.4 Dashboards

`dashboards/` (Wave-Z extension): unit-wide alarm-fatigue, deterioration-prediction
top-quartile beds, RPM-adherence cohort, telemetry-coverage uptime, code-blue activation
trend.

---

## 12. Failure modes (top 10)

| # | Mode | Mitigation |
|---|---|---|
| 1 | Stream broker (Kafka) failure | Local-cell ring buffer 4h fallback |
| 2 | Cedar policy engine timeout | Fail-OPEN for alarm-fire delivery |
| 3 | ML inference unavailable | Fall back to rule-based scoring (NEWS2/MEWS/qSOFA) |
| 4 | Device gateway disconnect | Lead-off detector fires; ring buffer accumulates pending |
| 5 | Bedside device clock drift | HLC stamp at ingest + NTP reconciliation alarm |
| 6 | gRPC server pod restart | QUIC connection resumption < 1 s |
| 7 | TimescaleDB write back-pressure | Buffer in ring; drop 1-min aggregates last; raw retained |
| 8 | Object storage write failure | Retry with exponential backoff; ring buffer holds |
| 9 | Cell-migration cutover | Dual-emit window; ring buffer absorbs cutover variance |
| 10 | Wearable cloud-API outage | Mark RPM session paused; resume on reconnect |

---

## 13. Security architecture

### 13.1 Transport

- HTTP/3 + QUIC for all client-server (per ADR-0253).
- mTLS between µservice instances (per ADR-0254 deployment-model spectrum); cert rotation
  via cloud-kms.
- gRPC waveform streams: end-to-end encrypted with tenant-scoped session keys.

### 13.2 At-rest

- TimescaleDB + Postgres: TDE with tenant-scoped key (per ADR-0244 + ADR-0251).
- Object storage: per-tenant SSE-KMS with tenant-scoped CMK.
- ClickHouse: TDE.
- Audit chain: hash-chained, per-tenant root.

### 13.3 PHI handling

- Per HIPAA: minimum necessary; per-clinician access via Cedar bedside-group scoping.
- Per 21 CFR Part 11: every alarm-ack carries clinician identity + HLC timestamp.
- Per EU AI Act: every deterioration score has inference lineage.
- BYOK supported (per ADR-0255 §D-4): tenant may opt-in to bring their own LLM/model
  provider for non-mandatory inference (clinical-decision-support text generation).

### 13.4 Network segmentation

- Tier-3 (edge) communicates with Tier-2 only via per-tenant VPN or per-tenant direct connect.
- Tier-4 (in-room) communicates with Tier-3 only on local hospital network.
- RPM (Tier-0/Tier-2 mix): wearable cloud APIs reached via per-tenant egress proxy with
  cipher-suite enforcement.

---

## 14. Operational runbook (top 5 scenarios)

(Full runbooks in `runbooks/` — Wave-Z extension; this section lists the top scenarios.)

1. **Cell evacuation**: migration protocol per §3.4. Zero-waveform-loss.
2. **ML model rollback**: model registry version pin; rollback completes within 5 min.
3. **Alarm-fatigue investigation**: query alarm-fatigue rolling-7-day; identify top-noisy
   rule; charge nurse + clinical engineer review.
4. **Vendor connector outage** (e.g., Philips CareEvent CMS down): switch to direct HL7v2
   listener; alert biomed.
5. **Wearable cloud-API rate-limit**: backoff + circuit-breaker; RPM session marked paused
   until rate-limit clears.

---

## 15. Bill-of-materials (per cell at 5,000-bed capacity)

| Component | Count | Sizing |
|---|---|---|
| Rust app pods (waveform-service) | 32 | 4 vCPU / 8 GiB each |
| Rust app pods (vital-signs-service) | 16 | 2 vCPU / 4 GiB each |
| Rust app pods (alarm-service) | 8 | 2 vCPU / 4 GiB each |
| Rust app pods (smart-alarm-engine) | 8 | 2 vCPU / 4 GiB each |
| Rust app pods (ml-inference) | 16 | 4 vCPU / 8 GiB each (LightGBM forward) |
| Rust app pods (central-station-fanout) | 8 | 2 vCPU / 4 GiB each |
| Rust app pods (rpm-ingest) | 8 | 2 vCPU / 4 GiB each |
| Rust app pods (mobile-notification) | 4 | 2 vCPU / 4 GiB each |
| TimescaleDB cluster | 1 primary + 2 replicas | 16 vCPU / 64 GiB / 4 TiB NVMe |
| Postgres-16 (registry + meta) | 1 primary + 2 replicas | 8 vCPU / 32 GiB / 500 GiB |
| ClickHouse | 6-node | 8 vCPU / 32 GiB / 10 TiB |
| Object storage | n/a | S3-compatible regional |
| Stream platform (Kafka/Redpanda) | 6-node | 8 vCPU / 32 GiB |
| Local-cell ring buffer | per-pod NVMe | 50 GiB per bed-session aggregate |
| Cedar policy-engine | (shared substrate) | per cell µservice spec |

Estimated cell cost (US-East-1 + on-prem equivalent): ~$50K/month at 5,000-bed steady-state.

---

## 16. Deployment-context posture

### 16.1 oyatie-public-cloud

- Default for RPM, oyatie-as-cloud-provider tenants, small healthcare-network tenants.
- Tier-2 cells, regional spread.
- Public-cloud waveform archive (per-tenant prefix in regional object store).

### 16.2 guest-on-aws

- Common for US hospital tenants with existing AWS contracts.
- Tier-1 or Tier-2; per-tenant VPC.

### 16.3 guest-on-oci

- iac/guest-on-oci/always-free/ exploits Always Free for sandbox.

### 16.4 on-prem

- Standard for acute-care ICU/CCU/PACU/ED.
- Tier-2 cell + Tier-3 edge + Tier-4 in-room.
- Offline-survivable ≥ 24h.

### 16.5 colo

- Standard for EU hospitals + regulated KR hospitals.
- Tier-1 or Tier-2 in customer colo facility.

### 16.6 oyatie-as-cloud-provider

- The fully oyatie-operated cellular substrate.
- Tier-1 sovereign hosts for EU + KR + DoD; Tier-2 for general.

---

## 17. Sequence diagrams (top 5)

### 17.1 Alarm-fire end-to-end

```
Device → DeviceInterop : ORU^R01 (HR=145)
DeviceInterop → VitalSignsStream : canonicalize + emit
VitalSignsStream → SmartAlarm : rule-eval
SmartAlarm → AlarmManagement : compound met (HR>130 ∧ SpO2<92 ∧ RR>28) ⇒ fire 'critical'
AlarmManagement → MobileNotification : fanout dispatch
MobileNotification → APNs : push notification
APNs → ClinicianDevice : delivered
ClinicianDevice → MobileNotification : ack (clinician_id, HLC)
MobileNotification → AlarmManagement : ack received
AlarmManagement → AuditChain : alarm-acked event hashed + chained
```

### 17.2 Code-blue auto-activation

```
SmartAlarm → AlarmManagement : life-threatening fire (cardiac asystole)
AlarmManagement → MobileNotification : fanout dispatch (hop 1)
[10s] no ack → AlarmManagement : escalate hop 2 (charge nurse)
[30s] no ack → AlarmManagement : escalate hop 3 (on-call physician)
[90s] no ack → CodeBlue : auto-activate
CodeBlue → WaveformArchive : pin ± 30 min for all channels (bed-X)
CodeBlue → MobileNotification : code-blue team pager dispatch (highest priority)
CodeBlue → CentralStation : highlight bed-X in red + audible alarm
CodeBlue → AuditChain : code-blue activation hashed + chained
```

### 17.3 Deterioration prediction lineage

```
Scheduler [5-min cadence] → DeteriorationPrediction : trigger bed-X
DeteriorationPrediction → VitalSignsStream : pull last 4h trend (10 channels)
DeteriorationPrediction → emr : pull last 24h labs (via healthcare-integration)
DeteriorationPrediction → feature-builder : compute features
DeteriorationPrediction → ml-platform : LightGBM forward + Platt-calibrate
ml-platform → DeteriorationPrediction : score = 0.78
DeteriorationPrediction → rule-augment : NEWS2 = 7 (high)
DeteriorationPrediction → emit DeteriorationScore { score, components, lineage_id }
DeteriorationPrediction → AuditChain : lineage hashed + chained
DeteriorationPrediction → AlarmManagement : threshold check; suggest moderate alert if > 0.75
```

### 17.4 RPM wearable ingest

```
WearableCloud(Withings) → WearableConnector(Webhook) : POST /webhook { patient_id, weight=78kg }
WearableConnector → consent-graph : verify consent active
WearableConnector → VitalSignsStream : canonicalize + emit
VitalSignsStream → RPM bounded context : update RPMReadingSession
RPM → smart-alarm : weight-gain rule check (delta > 2kg / 3d)
smart-alarm → AlarmManagement : fire 'medium' (CHF weight-gain)
AlarmManagement → MobileNotification : alert care coordinator
MobileNotification → coordinator : FCM push
```

### 17.5 Cell migration cutover

```
ops → cell µservice : initiate migration cell-A → cell-B for tenant-T
cell µservice → patient-monitoring (cell-A) : begin dual-emit
patient-monitoring (cell-A) : continue serving + replicate to cell-B
cell µservice → patient-monitoring (cell-B) : pre-warm registry + Cedar bundles
cell µservice → device-interop redirector : update routing T → cell-B
device-interop redirector → bedside devices : connection-refresh hint
bedside devices → patient-monitoring (cell-B) : reconnect (QUIC resume)
patient-monitoring (cell-B) : first-byte-delivered → confirm to cell µservice
cell µservice → patient-monitoring (cell-A) : finalize + tear down
```

---

## 18. Open architectural questions

1. **Federated learning protocol**: secure-aggregation vs. differential-privacy gradient
   noise — see ml-platform µservice IP-014.
2. **Tele-ICU multi-tenant central-station**: requires Cedar cross-tenant view via
   consent-graph; out of Wave-15m-F scope (W16).
3. **Edge-of-edge (Tier-4) in-room device sidecar packaging**: per-vendor or generic? Likely
   generic Rust sidecar with vendor-plugin shim.
4. **Continuous-glucose-monitor waveform-class archival**: do we treat CGM as 5-min-cadence
   numeric vital or as a 0.1-Hz "slow waveform"? Roadmap W18.

---

## 19. References

- ADR-0131 (per-microservice flat layout)
- ADR-0132 (suite/bundle dissolution)
- ADR-0243 (Cedar universal gate)
- ADR-0244 (tenant scoping primitive)
- ADR-0245 (substrate-vs-product layering)
- ADR-0248 (Amazon-shape cellular)
- ADR-0251 (compliance pack + cell certification)
- ADR-0252 (HLC default; TrueTime tier opt-in)
- ADR-0253 (HTTP/3 + QUIC default)
- ADR-0254 (deployment-model spectrum)
- ADR-0255 (BYOK + intelligence two-layer)
- ADR-0328 (substance bar + batch discipline)
- ADR-0332 (clinical-realtime substrate — this µservice's binding ADR)
- ADR-MS-001 (this µservice — streaming substrate gRPC + FlatBuffers)
- ADR-MS-002 (this µservice — smart-alarm engine)
- ADR-MS-003 (this µservice — deterioration ML stack)
- IEEE 11073-10101 (Point-of-Care Medical Device Communication)
- IEEE 11073-20601 (Personal Health Device exchange)
- IHE PCD profiles (DEC, WCM, ACM)
- HL7 v2.5.1 + v2.8 + FHIR R5
- FDA 21 CFR Part 11 (electronic records + electronic signatures)
- IEC 62304:2015 (Software lifecycle for medical devices)
- ISO 14971:2019 (Risk management for medical devices)
- EU MDR 2017/745
- EU AI Act 2024 (high-risk classification)
- KR PIPA 2023 amendment
- KR Medical Law 2024
- KR MFDS Medical Device 2024
- Surviving Sepsis Campaign 1-hour bundle (clinical reference)
- MIMIC-IV + eICU-CRD (training data references)

---

End of ARCHITECTURE.md
