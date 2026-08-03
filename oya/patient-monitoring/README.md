# patient-monitoring µservice

> Continuous physiologic surveillance substrate for ICU/CCU/PACU/ED + Remote Patient Monitoring (RPM)
> with smart alarm management, ML-driven deterioration prediction, and hyperscaler-grade waveform streaming.

**Status**: wave-15m-f-authoring
**Date**: 2026-05-21
**Tier**: product
**Audience**: tenant-b2b-healthcare-provider (acute, post-acute, RPM, home-health, telehealth)
**Owner**: axis-clinical-realtime

---

## 1. Mission

The patient-monitoring µservice owns continuous physiologic-data acquisition, real-time alarm management,
smart-alarm fatigue mitigation, ML-driven deterioration prediction, sepsis early warning, code-blue activation,
remote-patient-monitoring (RPM), and central-station multi-bed view across all six oyatie deployment
contexts (oyatie-public-cloud, guest-on-aws, guest-on-oci, on-prem, colo, oyatie-as-cloud-provider).

It is the canonical clinical-realtime physiologic substrate. Where the EMR µservice owns the
episodic record-of-truth (encounter, problem, medication, allergy, note, order, result) and the
healthcare-integration µservice owns external interoperability (HL7v2, FHIR R5, IHE, DICOM, NCPDP),
this µservice owns **everything that streams from a body at 1-1000 Hz**, the **alarms** triggered by
those streams, and the **derived predictions** computed over them.

Counterparts and parity targets:

| Vendor | Product | Beat-or-match |
|---|---|---|
| Philips | IntelliVue / PIC iX / CareEvent | Multi-bed central station, MEDIBUS, ST-segment monitoring |
| GE Healthcare | CARESCAPE Network / Unity / MUSE | Distributed-patient-database, ECG management, MUSE archive |
| Mindray | BeneVision N-Series / eGateway | Multi-parameter telemetry, mobile alarm forwarding |
| Masimo | Patient SafetyNet / Root | SpO2 + RRa, alarm-fatigue mitigation |
| Welch Allyn (Hillrom / Baxter) | Connex Vitals + Spot Vital Signs | Episodic vitals + bedside-to-EMR plumbing |
| Edwards Lifesciences | HemoSphere | Advanced hemodynamics (cardiac-output continuous, SVV) |
| BioTelemetry (Philips) | Mobile Cardiac Outpatient Telemetry | Outpatient long-term ECG monitoring |
| Drager | Infinity Acute Care | German-engineering ICU monitor |

This µservice **matches every capability** in those products and adds three frontier capabilities
they cannot match: cellular-architecture isolation (per ADR-0248), Cedar-gated alarm-suppression with
audit trail, and federated deterioration-model training across tenants.

---

## 2. Bounded contexts (overview)

The PRD enumerates 18 bounded contexts in full detail. The summary view:

| # | Context | One-liner |
|---|---|---|
| 01 | VitalSignsStream | Continuous 1-Hz numeric vitals (HR, RR, SpO2, BP, Temp, EtCO2, CVP) |
| 02 | Waveform | 50-1000 Hz physiologic waveforms (ECG, PLETH, ART, ICP) |
| 03 | ICUMonitoring | ICU/CCU/PACU/ED-specific bed surveillance + advanced hemodynamics |
| 04 | RPM | Remote patient monitoring (home, post-discharge, chronic-disease) |
| 05 | AlarmManagement | Multi-parameter alarm orchestration + escalation |
| 06 | SmartAlarm | Compound conditions + validity checking + fatigue mitigation |
| 07 | Trending | Numeric vitals trending + retrospective analysis |
| 08 | DeteriorationPrediction | NEWS2 / MEWS / Rothman Index / Epic-Deterioration analog |
| 09 | SepsisEarlyWarning | qSOFA / SOFA / Sepsis-3 + ML augmentation |
| 10 | CodeBlue | Cardiac-arrest / rapid-response activation + post-event capture |
| 11 | TelemetryCoverage | Lead-off / signal-quality / coverage management |
| 12 | CentralStation | Multi-bed unit-wide view (large-screen + clinician workstation) |
| 13 | MobileNotification | Caregiver smartphone / pager alerts (FCM, APNs, paging gateway) |
| 14 | WaveformArchive | Long-term waveform retrieval + study replay |
| 15 | DeviceInterop | HL7 v2 + IEEE 11073 + Continua + vendor connectors |
| 16 | TrendAnalytics | Cohort-level analytics + outcomes (decoupled from per-bed surveillance) |
| 17 | WearableIntegration | Apple HealthKit / Fitbit / Garmin / Withings / Dexcom RPM |
| 18 | ICUBundleCompliance | Head-of-bed + DVT-prophylaxis + SAT/SBT + glucose-control |

---

## 3. Quick-start (developer)

This µservice is **Rust-strict-only** per the global `feedback_rust_strict_only_no_python_2026_05_20`
constraint. The crates live under `microservices/patient-monitoring/src/` per ADR-0131.

```bash
# Build
cargo build -p patient-monitoring-app

# Test (unit + integration)
cargo test -p patient-monitoring-app

# Lint (mandatory pre-commit)
cargo clippy -p patient-monitoring-app -- -D warnings
cargo fmt --check

# Lane: vitals streaming verification
cargo test -p patient-monitoring-vital-signs-stream-kernel --features=streaming-integration

# Lane: smart-alarm rules verification
cargo test -p patient-monitoring-smart-alarm-domain --features=rules-replay
```

Local dev environment requires:

- Rust 1.85.0-stable (MSRV 1.83.0)
- gRPC tooling: `protoc 25.x` + `prost 0.12` + `tonic 0.11`
- FlatBuffers tooling: `flatc 24.x`
- A Kafka-compatible stream broker (Redpanda dev container is acceptable)
- A Postgres 16 instance for the trending store
- A TimescaleDB extension for vital-signs continuous-aggregate views
- A ClickHouse 24.x instance for waveform-archive cold-tier
- An OPA / Cedar engine binding (per ADR-0243)

---

## 4. API surfaces

| Surface | Path | Description |
|---|---|---|
| FHIR Observation (vital-signs profile) | `/fhir/r5/Observation?category=vital-signs` | Read/write 1-Hz numeric vitals |
| FHIR Device + DeviceMetric | `/fhir/r5/Device` and `/fhir/r5/DeviceMetric` | Device + measurement-channel registry |
| FHIR Encounter | `/fhir/r5/Encounter` (cross-ref to EMR) | Encounter scoping |
| gRPC streaming (waveforms) | `oya.patient_monitoring.v1.WaveformService/StreamWaveform` | Bidi 50-1000 Hz |
| gRPC streaming (vitals) | `oya.patient_monitoring.v1.VitalSignsService/StreamVitalSigns` | Bidi 1-Hz numeric |
| gRPC alarms | `oya.patient_monitoring.v1.AlarmService/AcknowledgeAlarm` | Alarm ack + suppression |
| AsyncAPI / events | (CloudEvents-shaped) `vital.streamed`, `waveform.streamed`, `alarm.fired`, `deterioration.predicted`, `sepsis.warning`, `code.blue.activated`, `telemetry.coverage.lost`, `wearable.session.started` |
| REST (admin / central-station) | `/api/v1/units/{unit}/beds`, `/api/v1/central-station/sessions` | Operator surfaces |

Full contracts under `contracts/`.

---

## 5. Compliance posture

- **Mandatory packs**: HIPAA-2024, FDA 21 CFR Part 11 (electronic signatures), IEC 62304 SaMD Class C
  (life-supporting/sustaining software lifecycle), ISO 14971 (risk management).
- **Recommended packs**: SOC2-T2, ISO 27001, ISO 13485 (medical-device QMS), EU MDR 2017/745,
  GDPR baseline, KR PIPA + KR Medical Law + KR MFDS Medical Device, EU AI Act high-risk,
  FERPA, GxP CSV.
- **Data classification**: phi-restricted (default); pii_emitted; biometric_data_emitted;
  minor_data_emitted (pediatric ICU patients).
- **AI inference class**: high-risk-clinical (per EU AI Act Annex III — biometric + healthcare;
  US FDA SaMD Class C).
- **RPO / RTO**: 1 second / 30 seconds (life-critical; per ADR-0332).
- **Retention**: 7Y for vitals + alarms + deterioration; 10Y audit; 21Y consent; 90D continuous
  waveform; 7Y alarm-episode waveform.

---

## 6. Documentation inventory

| File | Purpose | Floor |
|---|---|---|
| `PRD.md` | Product requirements + bounded contexts + user flows + KPI | ≥800 lines |
| `ARCHITECTURE.md` | Component diagrams + streaming substrate + cell topology | ≥600 lines |
| `README.md` | This file | ≥300 lines |
| `manifest.json` | Machine-readable µservice descriptor | — |
| `competitor-parity-matrix.md` | 100+ capabilities mapped vs Philips/GE/Mindray | — |
| `supported-oses.json` | Per-OS matrix + tiers + CI lanes | — |
| `contracts/openapi.yaml` | FHIR Observation + Device + DeviceMetric REST | — |
| `contracts/asyncapi.yaml` | Event surface | — |
| `contracts/proto/patient-monitoring.proto` | gRPC streaming | — |
| `decisions/ADR-MS-001..003.md` | µservice-scoped ADRs | — |
| `implementation-plans/IP-001..010.md` | Sliced implementation plan | — |
| `policies/*.cedar` | Cedar policy bundles | — |
| `slos/*.openslo.yaml` | OpenSLO files (≥10) | — |
| `iac/<context>/` | Per-context OpenTofu module references | — |

---

## 7. Cross-µservice handoffs

| Direction | Counterpart µservice | Purpose |
|---|---|---|
| ingests from | `healthcare-integration` | Inbound HL7v2 / IEEE 11073 / FHIR Observation feed |
| emits to | `emr` | Vitals charting (signed FHIR Observation refs) |
| emits to | `audit-chain` | Per-event WORM audit (HIPAA + 21 CFR Part 11) |
| emits to | `compliance` | Pack evidence (alarm-suppression justification ledger) |
| emits to | `notification` | Mobile + pager dispatch |
| emits to | `data-warehouse` | Trend + cohort analytics offload |
| emits to | `stream-platform` | All AsyncAPI events |
| consumes from | `consent-graph` | Patient consent for RPM + telemetry-sharing |
| consumes from | `cloud-iam` + `policy-engine` | Cedar evals for view/ack/suppress |
| consumes from | `cloud-kms` | PHI envelope keys + waveform encryption keys |
| consumes from | `ml-platform` | Deterioration + sepsis model serving |
| consumes from | `tenant` + `cell` | Tenant scoping + cell placement |
| consumes from | `cloud-data` | Postgres + ClickHouse + TimescaleDB substrate |
| publishes alarm-fired (downstream) | `code-blue-coordinator`, `sepsis-watch`, `clinical-decision-support`, `icu-bundle-compliance`, `ventilator-management`, `medication-administration`, `population-health`, `quality-measures-reporting` |

---

## 8. Performance posture

| Surface | Target | Worst-case budget |
|---|---|---|
| Vital-signs streaming end-to-end (device → central station) | p99 ≤ 250 ms | 500 ms |
| Waveform streaming jitter (50-1000 Hz) | p99 jitter ≤ 8 ms | 16 ms |
| Alarm fire → mobile-notification delivered | p99 ≤ 3 s | 6 s |
| Central station 8-bed render | p99 ≤ 400 ms | 800 ms |
| Deterioration model inference (per bed, every 5 min) | p99 ≤ 200 ms | 500 ms |
| FHIR Observation write (single resource) | p99 ≤ 150 ms | 300 ms |
| FHIR Observation batched (100 resources) | p99 ≤ 750 ms | 1500 ms |
| gRPC stream establishment (handshake + first byte) | p99 ≤ 120 ms | 250 ms |
| Smart-alarm rule evaluation per parameter sample | p99 ≤ 5 ms | 10 ms |
| Cedar policy eval for view/ack/suppress | p99 ≤ 5 ms | 10 ms |

Per the multi-region cell layout (ADR-0248), each ICU unit binds to a single cell; cross-cell
fan-out is prohibited at runtime (only as offline outcomes analytics in TrendAnalytics).

---

## 9. Authority chain (Cedar)

Three principal classes have authority over patient-monitoring resources:

- **nurse_assigned_to_bed_group**: view + acknowledge alarms on the assigned bed group
- **physician_attending**: view + acknowledge + suppress alarms; mark code-blue
- **clinical_engineer**: device-registry administration; calibration write
- **rpm_patient**: view-own only (B2C RPM tenants)
- **rpm_caregiver_designated**: view per consent-graph designation
- **central_station_operator**: multi-bed view + escalation; cannot suppress
- **icu_bundle_steward**: compliance-rollup read; cannot write to bedside data

Cedar bundles live in `policies/`. Authority chain is enforced at every API boundary
(REST, gRPC, AsyncAPI subscribe) and at the per-bed bed-group resolution.

---

## 10. Failure-mode posture

The full failure-mode catalogue lives in `failure-modes.md` (authored in Wave-Z extension);
this README documents the three highest-impact classes:

1. **Stream broker failure → loss of waveform fidelity**: dual-path fallback to local-cell ring
   buffer (≥ 4 hours per bed); reconciliation on broker recovery.
2. **Cedar policy engine timeout → alarm-fire muted**: fail-safe **OPEN** for alarm-fire
   (deliver to default unit pager) per ADR-0332 §B-7; suppression operations fail **CLOSED**.
3. **Deterioration model unavailable → fall back to deterministic NEWS2/MEWS scoring**: model
   inference is auxiliary; rule-based scoring continues uninterrupted.

---

## 11. Operating bar

This µservice ships the standard 100-bar (per `feedback_go_with_original_ambition_2026_05_20`):

- PRD ≥ 800 lines, ARCHITECTURE ≥ 600 lines, README ≥ 300 lines
- 18 bounded contexts fully scoped
- ≥ 100 capabilities mapped against Philips + GE + Mindray
- ≥ 10 OpenSLO files (latency + correctness + jitter + inference latency + alarm delivery)
- ≥ 10 implementation plans (IP-001..IP-010)
- ≥ 4 Cedar policy bundles
- 3 µservice-scoped ADRs (substrate, smart-alarm engine, ML stack)
- 6 deployment-context iac/ stubs
- supported-oses.json with 13 OSes + 5 arches
- competitor-parity-matrix.md with 100+ capabilities

---

## 12. Read order

If you are joining this µservice cold, read in this order:

1. `manifest.json` (5 minutes)
2. `README.md` (this file, 20 minutes)
3. `PRD.md` (90 minutes)
4. `ARCHITECTURE.md` (60 minutes)
5. `competitor-parity-matrix.md` (30 minutes)
6. `decisions/ADR-MS-001-streaming-substrate-grpc-flatbuffers.md` (15 minutes)
7. `decisions/ADR-MS-002-smart-alarm-engine.md` (15 minutes)
8. `decisions/ADR-MS-003-deterioration-ML-stack.md` (20 minutes)
9. `implementation-plans/IP-001..IP-010` (browse — read the ones you'll work on first)
10. `contracts/` (skim openapi + asyncapi + proto)
11. `policies/` (read every Cedar file — they encode authority chain)
12. `slos/` (browse — they encode SRE posture)

---

## 13. Open questions / known gaps (as of 2026-05-21)

| # | Topic | Status |
|---|---|---|
| 1 | Federated learning for deterioration models across tenants | Open — ADR draft TBD |
| 2 | Integration with eICU tele-ICU services (Philips eICU, Mercy Virtual) | Spec'd in PRD §11; impl deferred to IP-011 (post-wave) |
| 3 | Wearable-derived AFib detection delivery into alarm stream | Spec'd in PRD §17.6; impl deferred to IP-012 |
| 4 | NICU-specific apnea-of-prematurity prediction model | Roadmap entry; off-critical-path |
| 5 | Continuous-glucose-monitor (CGM) waveform-class archival | Spec'd in PRD §17.8 |

All other capabilities are in-scope and addressed by the wave-15m-f deliverables.

---

## 14. License + provenance

This µservice is part of the oyatie monorepo. License headers per the repo root.
Authored by axis-clinical-realtime under the Foundry pipeline.
ADR-0332 binds the bounded contexts; ADR-0131 binds the flat layout; ADR-0332's
sibling clinical-realtime ADRs bind the streaming substrate + smart-alarm engine.

---

## 15. Contact + escalation

- L1 on-call: `patient-monitoring-l1-oncall@oyatie.com`
- L3 SaMD-engineering oncall: `patient-monitoring-l3-oncall@oyatie.com`
- Clinical safety officer: `clinical-safety-officer@oyatie.com`
- Regulatory officer: `regulatory-officer@oyatie.com`
- Security officer: `security-officer@oyatie.com`

## 16. Change log

| Date | Change | Owner |
|---|---|---|
| 2026-05-21 | Initial Wave 15m-F authoring | axis-clinical-realtime |

## 17. Related µservices (read alongside)

- `emr` — episodic record-of-truth
- `healthcare-integration` — HL7v2 + FHIR + IHE inbound
- `clinical-decision-support` — downstream advisor surface
- `code-blue-coordinator` — code-blue dispatch + team-side
- `sepsis-watch` — downstream sepsis-program µservice
- `icu-bundle-compliance` — downstream bundle-program µservice
- `ventilator-management` — paired closed-loop weaning + vent-parameter consumer
- `medication-administration` — alarm-context consumer (titration response)
- `telehealth` + `rpm-portal` — paired RPM patient-facing surfaces

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0346-oya-verify-must-run-full-ci-mirror.md) (amended by ADR-0515): legacy `oya verify` / `./bin/oya verify --ci-required` output is optional local-feedback/provenance only; protected-branch merge authority is the GitHub Actions + branch-protection `oya-ci-required` context produced by cloud-ci Rust gate packets. Historical `oya-governance-oya-verify-*` lane references are retained only as provenance unless reintroduced by current cloud-ci gates.
- [ADR-0347](../../docs/decisions/ADR-0347-governance-fitness-bulk-rename.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0348-autosharding-auto-rebalance-dynamic-sharding.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0349-jenkins-argocd-self-hostable-ci-cd-substrate.md): ADR-0349 Jenkins CI wording is historical/provenance after ADR-0515; GitHub Actions produces `oya-ci-required` until explicit owned-runner cutover, and ArgoCD remains the separately authorized GitOps CD evidence surface where applicable. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
