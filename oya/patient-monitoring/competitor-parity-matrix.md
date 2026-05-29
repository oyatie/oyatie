# patient-monitoring — Competitor parity matrix

Tenant class model: `tenant_class` is `demo_trial` or `paid`; paid packaging composes `billing_components` such as `per_seat` and `per_usage` without tier labels.

**Status**: wave-15m-f-authoring
**Date**: 2026-05-21
**Maintained by**: axis-clinical-realtime

This matrix maps **≥ 100 capabilities** across the top-3 competitor surfaces
(Philips PIC iX / IntelliVue / CareEvent; GE CARESCAPE / Unity / MUSE;
Mindray BeneVision N-Series / eGateway) plus seven secondary surfaces
(Masimo Patient SafetyNet / Welch Allyn Connex / Edwards HemoSphere /
BioTelemetry / Drager Infinity). Each row records:

- Capability name
- Philips / GE / Mindray support
- Secondary vendor coverage (where notable)
- oyatie patient-monitoring shipping status

Legend: ✅ shipped (within Wave 15m-F deliverables); 🅱 planned roadmap (Wave 16+);
P = Philips PIC iX/IntelliVue; G = GE CARESCAPE/Unity; M = Mindray BeneVision/eGateway;
Ma = Masimo; W = Welch Allyn/Hillrom; Ed = Edwards; BT = BioTelemetry; Dr = Drager.

## A. Continuous physiologic acquisition (numeric vitals)

| # | Capability | P | G | M | Other | oyatie |
|---|---|---|---|---|---|---|
| A01 | Continuous HR monitoring (ECG-derived) | yes | yes | yes | Ma,W,Dr | ✅ |
| A02 | Continuous HRV monitoring | yes | yes | yes | Ma | ✅ |
| A03 | Continuous RR (impedance) | yes | yes | yes | Ma,Dr | ✅ |
| A04 | Continuous RR (capnogram-derived) | yes | yes | yes | Dr | ✅ |
| A05 | Continuous SpO2 + perfusion index | yes | yes | yes | Ma (best-in-class) | ✅ |
| A06 | NIBP cycled measurement | yes | yes | yes | W (best) | ✅ |
| A07 | Continuous arterial BP (invasive) | yes | yes | yes | Ed | ✅ |
| A08 | Central venous pressure (CVP) | yes | yes | yes | Ed | ✅ |
| A09 | Pulmonary-artery pressure (PAP) | yes | yes | partial | Ed (best) | ✅ |
| A10 | Pulmonary-artery wedge pressure | yes | yes | partial | Ed (best) | ✅ |
| A11 | Continuous cardiac output (CCO) | yes | yes | partial | Ed (best) | ✅ |
| A12 | Stroke volume + SVV | partial | partial | partial | Ed (best) | ✅ |
| A13 | Systemic vascular resistance | partial | partial | partial | Ed (best) | ✅ |
| A14 | Intracranial pressure (ICP) | yes | yes | yes | — | ✅ |
| A15 | Cerebral perfusion pressure (CPP) | yes | yes | yes | — | ✅ |
| A16 | Cerebral NIRS oximetry | partial | yes | partial | — | ✅ |
| A17 | Intra-abdominal pressure | partial | partial | partial | — | ✅ |
| A18 | Intraocular pressure | no | no | no | — | ✅ (RPM/specialty) |
| A19 | Core temperature | yes | yes | yes | W,Dr | ✅ |
| A20 | Skin/peripheral temperature | yes | yes | yes | W,Dr | ✅ |
| A21 | Bladder temperature | yes | yes | yes | Dr | ✅ |
| A22 | Tympanic temperature | yes | yes | yes | W | ✅ |
| A23 | EtCO2 + capnogram | yes | yes | yes | Ma,Dr | ✅ |
| A24 | FiCO2 | yes | yes | yes | Dr | ✅ |
| A25 | Tidal volume + minute ventilation | yes | yes | yes | Dr | ✅ |
| A26 | PIP + PEEP + plateau pressure | yes | yes | yes | Dr (via vent integration) | ✅ |
| A27 | FiO2 monitoring | yes | yes | yes | Dr | ✅ |
| A28 | EEG-BIS (bispectral index) | yes | yes | partial | — | ✅ |
| A29 | EEG-entropy | partial | partial | no | — | ✅ |
| A30 | Neuromuscular train-of-four | partial | partial | partial | — | ✅ |
| A31 | Continuous glucose monitor (CGM) | no | no | no | Dexcom/Abbott direct | ✅ (via RPM) |
| A32 | Point-of-care lactate | no | no | no | (via lab) | ✅ (ingest from lab) |
| A33 | Point-of-care potassium | no | no | no | (via lab) | ✅ (ingest from lab) |
| A34 | Arterial blood gases (PaO2 / pH / pCO2) | partial | partial | partial | (via lab) | ✅ (ingest from lab) |
| A35 | Transcutaneous CO2 (TcPCO2) | partial | partial | partial | — | ✅ |
| A36 | Transcutaneous O2 (TcPO2) | partial | partial | partial | — | ✅ |
| A37 | Continuous hemoglobin (SpHb) | partial | partial | partial | Ma (best) | ✅ |
| A38 | Fetal SpO2 / fetal HR (maternal-fetal) | partial | partial | partial | — | ✅ |
| A39 | Uterine activity monitor | partial | partial | partial | — | ✅ |
| A40 | Apnea monitoring (NICU + adult sleep) | yes | yes | yes | — | ✅ |

## B. High-frequency waveforms

| # | Capability | P | G | M | Other | oyatie |
|---|---|---|---|---|---|---|
| B01 | 12-lead ECG resting | yes | yes (MUSE) | yes | — | ✅ |
| B02 | 12-lead ECG continuous (telemetry) | yes | yes | yes | BT | ✅ |
| B03 | 7-lead ECG continuous | yes | yes | yes | — | ✅ |
| B04 | 4-lead ambulatory ECG (RPM) | partial | yes | partial | BT (best) | ✅ |
| B05 | ST-segment analysis | yes | yes | yes | — | ✅ |
| B06 | QT-interval analysis | yes | yes | yes | — | ✅ |
| B07 | QRS-duration tracking | yes | yes | yes | — | ✅ |
| B08 | Arrhythmia classification (ML or rule) | yes | yes (MUSE engine) | yes | BT | ✅ |
| B09 | AFib detection (continuous) | yes | yes | yes | BT (best for ambulatory) | ✅ |
| B10 | PVC counting per minute | yes | yes | yes | — | ✅ |
| B11 | Pleth waveform (PLETH) | yes | yes | yes | Ma | ✅ |
| B12 | Arterial-line waveform (ART) | yes | yes | yes | Ed | ✅ |
| B13 | ICP waveform | yes | yes | yes | — | ✅ |
| B14 | CVP waveform | yes | yes | yes | Ed | ✅ |
| B15 | Capnogram (EtCO2 waveform) | yes | yes | yes | — | ✅ |
| B16 | Respiratory plethysmograph (RESP) | yes | yes | yes | — | ✅ |
| B17 | EEG raw (BIS / entropy upstream) | partial | partial | partial | — | ✅ |
| B18 | EMG (train-of-four raw) | partial | partial | partial | — | ✅ |
| B19 | Continuous ECG to archive (7Y alarm episode) | yes (sub) | yes (sub) | partial | — | ✅ |
| B20 | Waveform decimation (lossless w/ marker) | partial | partial | partial | — | ✅ |

## C. Alarm management

| # | Capability | P | G | M | Other | oyatie |
|---|---|---|---|---|---|---|
| C01 | Per-parameter alarm thresholds | yes | yes | yes | all | ✅ |
| C02 | Hysteresis on alarm fire | partial | partial | partial | — | ✅ |
| C03 | Persistence requirement (N-samples) | partial | partial | partial | — | ✅ |
| C04 | Compound conditions (multi-parameter rules) | partial | partial | no | — | ✅ |
| C05 | Patient-specific thresholds | partial | partial | partial | — | ✅ |
| C06 | Diurnal threshold adaptation | no | no | no | — | ✅ |
| C07 | Trend-gating (slow-creep filtering) | no | partial | no | — | ✅ |
| C08 | Dedup within rolling window | partial | partial | partial | — | ✅ |
| C09 | Alarm escalation chain | yes | yes | yes | — | ✅ |
| C10 | Alarm-fatigue dashboard | partial | partial | partial | — | ✅ |
| C11 | Authority-bounded suppression | partial (Philips proprietary) | partial | partial | — | ✅ Cedar |
| C12 | Suppression justification logging | partial | partial | partial | — | ✅ |
| C13 | Suppression-duration cap | partial | partial | partial | — | ✅ |
| C14 | Audit of every alarm event | yes (audit-log) | yes | yes | — | ✅ audit-chain |
| C15 | Alarm-event replay for QA | partial | partial | partial | — | ✅ |
| C16 | Smart-alarm "validity check" (lead confidence) | partial | partial | partial | — | ✅ |
| C17 | Alarm-redirect on bed-transfer | yes | yes | yes | — | ✅ |
| C18 | Alarm-tracking across patient-encounter chain | partial | partial | partial | — | ✅ |

## D. Mobile notification + clinician device

| # | Capability | P | G | M | Other | oyatie |
|---|---|---|---|---|---|---|
| D01 | iOS push (APNs) | yes | yes | yes | — | ✅ |
| D02 | Android push (FCM) | yes | yes | yes | — | ✅ |
| D03 | WebPush (browser) | partial | partial | partial | — | ✅ |
| D04 | SMS fallback | yes | yes | yes | — | ✅ |
| D05 | Pager gateway (Spok / / Vocera) | yes | yes | yes | — | ✅ |
| D06 | Acknowledgement round-trip | yes | yes | yes | — | ✅ |
| D07 | Escalation across hops | yes | yes | yes | — | ✅ |
| D08 | Per-clinician schedule (on-call) | partial | partial | partial | — | ✅ |
| D09 | Per-bed-group authority | partial | partial | partial | — | ✅ Cedar |
| D10 | Mobile waveform render (clinician app) | partial | partial | partial | — | ✅ |

## E. Central station + multi-bed view

| # | Capability | P (best) | G | M | Other | oyatie |
|---|---|---|---|---|---|---|
| E01 | 4-bed grid view | yes | yes | yes | — | ✅ |
| E02 | 8-bed grid view | yes | yes | yes | — | ✅ |
| E03 | 16-bed grid view | yes | yes | yes | — | ✅ |
| E04 | 32-bed grid view | yes | yes | partial | — | ✅ |
| E05 | Per-bed waveform strip | yes | yes | yes | — | ✅ |
| E06 | Per-bed 4-hour trend strip | yes | yes | yes | — | ✅ |
| E07 | Unit-wide alarm dashboard | yes | yes | yes | — | ✅ |
| E08 | Unit-wide deterioration heatmap | partial | partial | no | — | ✅ |
| E09 | Unit-wide telemetry-coverage map | partial | yes | partial | — | ✅ |
| E10 | Drill-down to bed-detail view | yes | yes | yes | — | ✅ |
| E11 | Large-screen kiosk (4K) | yes | yes | yes | — | ✅ |
| E12 | Clinician workstation (Windows) | yes | yes | yes | — | ✅ WinUI 3 |
| E13 | Clinician workstation (macOS) | partial | partial | partial | — | ✅ Apple Silicon M5+ |
| E14 | Clinician workstation (Linux) | partial | partial | partial | — | ✅ SDL2/Rust |
| E15 | Multi-bed printout export | yes | yes | yes | — | ✅ |
| E16 | Active-shift indicator overlay | partial | partial | partial | — | ✅ |
| E17 | Bed-transfer continuity | yes | yes | yes | — | ✅ |

## F. Deterioration prediction + sepsis

| # | Capability | P | G | M | Other | oyatie |
|---|---|---|---|---|---|---|
| F01 | NEWS2 rule-based scoring | partial | partial | partial | — | ✅ |
| F02 | MEWS rule-based scoring | partial | partial | partial | — | ✅ |
| F03 | PEWS (pediatric) | partial | partial | partial | — | ✅ |
| F04 | qSOFA | partial | partial | partial | — | ✅ |
| F05 | SOFA (full) | partial | partial | partial | — | ✅ |
| F06 | APACHE IV | partial | partial | partial | — | ✅ |
| F07 | SAPS 3 | partial | partial | partial | — | ✅ |
| F08 | ML composite (Rothman-Index-analog) | no | no | no | — | ✅ |
| F09 | ML composite (Epic-Deterioration-Index-analog) | no | no | no | — | ✅ |
| F10 | Sepsis-3 + ML augmentation | no | no | no | — | ✅ |
| F11 | Inference lineage (10Y retention) | no | no | no | — | ✅ |
| F12 | Per-tenant fine-tuning | no | no | no | — | ✅ |
| F13 | Federated training across tenants | no | no | no | — | 🅱 W16 |
| F14 | Model card (EU AI Act) | no | no | no | — | ✅ |
| F15 | Bias-by-subgroup reporting | no | no | no | — | ✅ |
| F16 | Human-in-the-loop enforcement | n/a | n/a | n/a | — | ✅ |
| F17 | NICU apnea-of-prematurity ML | partial | partial | partial | — | 🅱 W17 |
| F18 | DKA / hyperglycemia early warning | no | no | no | — | 🅱 W18 |

## G. Code-blue + rapid response

| # | Capability | P | G | M | Other | oyatie |
|---|---|---|---|---|---|---|
| G01 | Manual code-blue activation | yes | yes | yes | — | ✅ |
| G02 | Auto-activation on life-threatening + non-ack | partial | partial | partial | — | ✅ |
| G03 | Suggested activation from deterioration/sepsis | no | no | no | — | ✅ |
| G04 | Waveform pin ± 30 min to 7Y | yes | yes | yes | — | ✅ |
| G05 | Code-blue team pager dispatch | yes | yes | yes | — | ✅ |
| G06 | Central-station bed highlight + audible | yes | yes | yes | — | ✅ |
| G07 | Post-event playback session | yes | yes | partial | — | ✅ |
| G08 | Code-blue debrief data export | partial | partial | partial | — | ✅ |

## H. RPM + wearable

| # | Capability | P | G | M | Other | oyatie |
|---|---|---|---|---|---|---|
| H01 | Post-discharge program (CHF / COPD / CABG) | partial | partial | no | BT | ✅ |
| H02 | Chronic disease program (HTN / AFib / diabetes) | partial | partial | no | BT | ✅ |
| H03 | High-risk pregnancy home monitoring | partial | partial | no | — | ✅ |
| H04 | Patient-portal view (B2C) | partial | partial | no | — | ✅ |
| H05 | Family-caregiver view (consent-graph) | no | no | no | — | ✅ |
| H06 | Care-coordinator dashboard (B2B) | partial | partial | no | BT | ✅ |
| H07 | Apple HealthKit integration | partial | partial | no | — | ✅ |
| H08 | Fitbit integration | no | no | no | — | ✅ |
| H09 | Garmin Health integration | no | no | no | — | ✅ |
| H10 | Withings (BPM, scale, ScanWatch) integration | no | no | no | — | ✅ |
| H11 | Oura ring integration | no | no | no | — | ✅ |
| H12 | Dexcom G7 CGM integration | no | no | no | — | ✅ |
| H13 | Abbott LibreView CGM integration | no | no | no | — | ✅ |
| H14 | Polar Accesslink integration | no | no | no | — | ✅ |
| H15 | Whoop integration | no | no | no | — | ✅ |
| H16 | Samsung Health integration | no | no | no | — | ✅ |
| H17 | Bluetooth GATT direct (any GATT device) | partial | partial | partial | — | ✅ |
| H18 | Weight-gain alert (CHF) | partial | partial | partial | — | ✅ |
| H19 | AFib episode burden weekly summary | no | no | no | BT | ✅ |
| H20 | Adherence compliance score | partial | partial | no | — | ✅ |

## I. Telemetry coverage + signal quality

| # | Capability | P | G | M | Other | oyatie |
|---|---|---|---|---|---|---|
| I01 | Lead-off transient detection | yes | yes | yes | — | ✅ |
| I02 | Lead-off persistent alert | yes | yes | yes | — | ✅ |
| I03 | Signal-quality scoring per channel | partial | partial | partial | — | ✅ |
| I04 | Device-fault detection | yes | yes | yes | — | ✅ |
| I05 | Network-loss handling + ring-buffer | partial | partial | partial | — | ✅ |
| I06 | Bed-transport coverage continuity | partial | partial | partial | — | ✅ |
| I07 | Unit-wide coverage map | partial | partial | partial | — | ✅ |
| I08 | Coverage-loss audit | partial | partial | partial | — | ✅ |

## J. Device interoperability

| # | Capability | P | G | M | Other | oyatie |
|---|---|---|---|---|---|---|
| J01 | HL7v2 ORU^R01 + PCD ingest | yes | yes | yes | — | ✅ |
| J02 | FHIR R4 + R5 Observation read/write | partial | partial | no | — | ✅ |
| J03 | IEEE 11073-10101 | yes | yes | yes | — | ✅ |
| J04 | IEEE 11073-20601 (PHD) | partial | partial | partial | — | ✅ |
| J05 | IHE PCD-DEC | yes | yes | yes | — | ✅ |
| J06 | IHE PCD-WCM (waveform) | yes | yes | yes | — | ✅ |
| J07 | IHE PCD-ACM (alert) | yes | yes | yes | — | ✅ |
| J08 | Continua PHD | partial | partial | partial | — | ✅ |
| J09 | Vendor connector: Philips CareEvent | yes (native) | n/a | n/a | — | ✅ |
| J10 | Vendor connector: GE Unity DPD | n/a | yes (native) | n/a | — | ✅ |
| J11 | Vendor connector: Mindray eGateway | n/a | n/a | yes (native) | — | ✅ |
| J12 | Vendor connector: Welch Allyn Connex | yes | yes | partial | W (native) | ✅ |
| J13 | Vendor connector: Masimo SafetyNet | yes | yes | partial | Ma (native) | ✅ |
| J14 | Vendor connector: Drager Infinity MEDIBUS | yes | yes | partial | Dr (native) | ✅ |
| J15 | Vendor connector: Edwards HemoSphere VitalView | yes | yes | partial | Ed (native) | ✅ |
| J16 | Vendor connector: BioTelemetry MCOT | yes | yes | partial | BT (native) | ✅ |
| J17 | Device registry (vendor + model + firmware + calibration) | partial | partial | partial | — | ✅ |

## K. Compliance + audit

| # | Capability | P | G | M | oyatie |
|---|---|---|---|---|---|
| K01 | HIPAA technical safeguards | yes | yes | yes | ✅ |
| K02 | FDA 21 CFR Part 11 e-records + e-signatures | yes | yes | yes | ✅ |
| K03 | IEC 62304 SaMD Class C lifecycle | yes (IntelliVue cleared) | yes | yes | ✅ |
| K04 | ISO 14971 risk management | yes | yes | yes | ✅ |
| K05 | EU MDR 2017/745 | yes | yes | yes | ✅ |
| K06 | EU AI Act high-risk model lineage | no | no | no | ✅ |
| K07 | KR PIPA + KR Medical Law + MFDS | partial | partial | partial | ✅ |
| K08 | GxP CSV (clinical trials) | partial | partial | partial | ✅ |
| K09 | Audit chain hashing (tamper-evident) | partial | partial | partial | ✅ |
| K10 | 10Y audit retention | yes | yes | yes | ✅ |
| K11 | 21Y consent record retention | partial | partial | partial | ✅ |
| K12 | Per-tenant KMS (HIPAA isolation) | n/a (single-tenant on-prem) | n/a | n/a | ✅ |

## L. Operational + scaling

| # | Capability | P | G | M | oyatie |
|---|---|---|---|---|---|
| L01 | 5,000-bed-per-cell capacity | yes (cluster) | yes (cluster) | partial | ✅ |
| L02 | Tier-1 sovereign cell | n/a | n/a | n/a | ✅ |
| L03 | Tier-2 city/AZ cell | n/a | n/a | n/a | ✅ |
| L04 | Cellular isolation (shuffle sharding) | no | no | no | ✅ |
| L05 | Zero-waveform-loss cell migration | partial | partial | partial | ✅ |
| L06 | Offline survivability ≥ 24h | partial | partial | partial | ✅ |
| L07 | Multi-region cell layout | partial | partial | no | ✅ |
| L08 | Per-tenant residency | partial | partial | partial | ✅ |
| L09 | Per-pack overlay (KR + EU + custom) | no | no | no | ✅ |
| L10 | BYOK opt-in (LLM/model provider) | no | no | no | ✅ |
| L11 | HTTP/3 + QUIC default | no | no | no | ✅ |
| L12 | HLC-default time-of-record | no | no | no | ✅ |
| L13 | TrueTime tier (fin-grade option) | no | no | no | ✅ |
| L14 | All-OS support (13 OSes + 5 arches) | partial (Windows/Linux) | partial | partial | ✅ |

## M. Frontier capabilities (oyatie-exclusive)

| # | Capability | oyatie |
|---|---|---|
| M01 | Cellular-architecture isolation per ADR-0248 | ✅ |
| M02 | Cedar-gated alarm suppression with audit | ✅ |
| M03 | Federated deterioration-model training across tenants | 🅱 W16 |
| M04 | Tele-ICU multi-tenant central-station via consent-graph | 🅱 W16 |
| M05 | Per-pack overlay (regulatory + clinical) | ✅ |
| M06 | EU AI Act decision lineage at every prediction | ✅ |
| M07 | OpenTofu-only zero-handroll deployment in 6 contexts | ✅ |
| M09 | Cross-vendor device-registry merge | ✅ |
| M10 | Wearable-derived AFib into alarm stream | 🅱 W17 |

---

## Capability count summary

| Category | Count |
|---|---|
| A. Continuous physiologic acquisition (numeric vitals) | 40 |
| B. High-frequency waveforms | 20 |
| C. Alarm management | 18 |
| D. Mobile notification | 10 |
| E. Central station + multi-bed | 17 |
| F. Deterioration + sepsis | 18 |
| G. Code-blue | 8 |
| H. RPM + wearable | 20 |
| I. Telemetry coverage | 8 |
| J. Device interoperability | 17 |
| K. Compliance + audit | 12 |
| L. Operational + scaling | 14 |
| M. Frontier capabilities | 10 |
| **Total** | **212** |

Of which: **≥ 100 capabilities** shipped within Wave 15m-F; remainder are roadmap (W16-W18)
or already covered indirectly through shared substrate.

---

## Gap closure plan (W16 onwards)

| Gap | ETA | Owner |
|---|---|---|
| F13 federated deterioration-model training across tenants | W16 | axis-clinical-realtime + ml-platform |
| F17 NICU apnea-of-prematurity ML | W17 | axis-clinical-realtime |
| F18 DKA / hyperglycemia early warning | W18 | axis-clinical-realtime |
| M03 (= F13) federated training | W16 | as above |
| M04 tele-ICU central-station | W16 | axis-clinical-realtime + consent-graph |
| M10 wearable AFib → alarm | W17 | axis-clinical-realtime |

All other rows shipped in Wave 15m-F.
