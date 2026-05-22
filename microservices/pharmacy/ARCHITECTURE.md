# Pharmacy Microservice — Architecture

- **Microservice slug**: `pharmacy`
- **Authority ADR**: ADR-0332
- **Layout authority**: ADR-0131 (per-microservice flat layout)
- **Suite policy**: ADR-0132 (single-concern flat)
- **Inter-microservice substrate**: ADR-0145 direct gRPC + 3 invariants
- **Architecture overlay**: ADR-0328 (v2.4.0 multispectrum + Big 8 + foundry absorption)
- **Cellular topology**: per ADR-0248 hyperscaler-shape cellular architecture (Cloud Hypervisor / Kata pods)
- **Protocol**: per ADR-0253 HTTP/3 + QUIC default, gRPC over HTTP/3
- **Causality**: per ADR-0252 HLC default; TrueTime tier opt-in for fin-grade dispense ledger reconciliation
- **Last reviewed**: 2026-05-21

---

## §1 Layering — 13-layer enum (ADR-0105)

Pharmacy uses every layer of the 13-layer enum where applicable. Per ADR-0056 BNF, every crate name is `oya-pharmacy-<bounded-context>-<layer>`.

| # | Layer | Pharmacy responsibility |
|---|---|---|
| 1 | `kernel` | Pure types, traits, ports; zero IO. RxCUI, NDC, GPI, AllergyMatch, DDIResult, FiveRightsResult value objects. |
| 2 | `domain` | Domain logic with invariants. Allergy-cross-class derivation, DRC computation, BUD calculator, formulary classifier. |
| 3 | `usecase` | Orchestrate ports across domain. ePrescribe transmit orchestration, dispense cycle, BCMA validate sequence. |
| 4 | `adapter` | Outbound IO. Surescripts mTLS, Pyxis HL7, Omnicell vendor API, FDB ingest, NCPDP D.0 PBM. |
| 5 | `api` | Inbound IO transport-agnostic façade. gRPC service trait implementations. |
| 6 | `rest` | REST/HTTP/3 surface. FHIR Medication/Request/Dispense/Administration/Statement. |
| 7 | `sdk` | Outbound consumer SDK. Used by `emr`, `cloud-billing`, `intelligence`. |
| 8 | `worker` | Long-running event processors. Surescripts inbound, audit-sealing, replay. |
| 9 | `app` | Composition root binary; wires kernel + usecase + adapters; reads config. |
| 10 | `check` | Per-microservice check crates (BNF, naming, vocabulary, dependency-seam, schema-cohesion). |
| 11 | `fit` | Foundry fitness verifiers (governance lane, ADR-0132 rename roadmap). |
| 12 | `bench` | Criterion benchmarks for hot path (DDI eval, BCMA scan). |
| 13 | `xtask` | Build-time codegen (proto, OpenAPI clients, Cedar policy compile). |

---

## §2 Cellular topology (per ADR-0248)

### §2.1 Cell tier and shuffle sharding

- Pharmacy runs on cell tier **T0** (patient-safety + DEA-regulated control plane).
- Tenants are shuffle-sharded across cells with shard width 2 and a poison-pill-resistant placement policy.
- Each cell carries:
  - Pharmacy worker pool (Cloud Hypervisor microVMs in Kata Containers).
  - PostgreSQL leader + 2 replicas (16.2 LTS pin).
  - Pulsar cluster for events (3.2 LTS pin).
  - Local Cedar policy cache (5-second freshness budget).
  - OpenBao for secrets (2.0 LTS pin).
- A cell failure SHALL NOT block another cell's dispensing; shuffle sharding guarantees blast radius ≤ 2 cells.

### §2.2 Cell membership

A cell membership manifest is registered with `cell` and carries:
- `compute`: 8 vCPU + 16 GiB per worker by default; scale-out by HPA.
- `storage`: 256 GiB SSD per worker for local Pulsar tier-1 + WAL cache.
- `crypto`: Cloud Hypervisor confidential-VM (SEV-SNP, TDX, or CCA) for EPCS-signing workers in regions where attestation is required.

---

## §3 Bounded contexts and per-context architectures

### §3.1 MedicationCatalog

```
┌────────────────────────────────────────────────────────────────┐
│  rest:     /api/v1/medication?rxcui=...                        │
│            /api/v1/medication?ndc=...                          │
│            /api/v1/medication?gpi=...                          │
│  api:      MedicationCatalogService gRPC                       │
│  usecase:  resolve, search, refresh-package                    │
│  domain:   RxCUI linking + NDC normalize + ATC classifier      │
│  adapter:  fdb-ingest, multum-ingest, medi-span-ingest         │
│  worker:   monthly RxNorm release reconciler                   │
└────────────────────────────────────────────────────────────────┘
```

- Read replicas for catalog reads; eventual consistency ≤ 60 s.
- A/B knowledge package switching via `tenant.config.knowledge_package_version`.

### §3.2 Formulary

- Per-tenant formulary base + per-cell overlay.
- P&T committee workflow stored as a state-machine aggregate; effective-date scheduling via worker.
- Therapeutic interchange: source-RxCUI → target-RxCUI mapping with conditions.

### §3.3 ePrescribe

- Surescripts production endpoint over mTLS; certificate material in OpenBao.
- EPCS sign envelope built by `pharmacy-eprescribe-usecase`, signed via `cloud-kms` adapter.
- Inbound NewRx → DDI/DAI/DCI/DPI/DRC pipeline → pharmacist queue.
- NCPDP SCRIPT 2017-071 mapping; future version migrations via adapter version pin.

### §3.4 DrugInteraction

- Eight sub-engines in `pharmacy-drug-interaction-domain`.
- Knowledge data sourced from FDB/Multum/Medi-Span via adapter.
- Severity stratification: contraindicated > severe > major > moderate > minor > informational.
- Suppression rules: per-tenant cap; suppression of severe and contraindicated requires Cedar override.

### §3.5 AllergyCheck

- Patient allergy list mirrored from `emr` via gRPC `emr.Allergy`.
- Allergen normalization to RxNorm ingredient + UNII + SNOMED CT substance.
- Cross-class derivation via knowledge-graph.

### §3.6 DoseCheck

- Computation engines: weight-based, BSA, eGFR (CKD-EPI), CrCl (Cockcroft-Gault), Child-Pugh, age-band.
- Single/daily/lifetime-cumulative caps.

### §3.7 Verification

- Single vs. dual mode driven by DEA schedule.
- Tall-man lettering rendering function in `pharmacy-verification-domain`.

### §3.8 Compounding

- USP 795/797/800 distinct subdomains.
- BUD calculator with USP table fallback explicit.
- Environmental monitoring evidence binding (linked to `compliance` substrate).

### §3.9 Inventory

- Per-location par/min/max state machine.
- Recall sequestration via `RecallNotice` aggregate.
- Expiry stratification: <7d use-first, <14d alert, <30d watch.

### §3.10 AutoDispensing

- Vendor-neutral cabinet adapter contract.
- Offline mode supported; reconciliation queue on reconnect.

### §3.11 BCMA

- Five-rights validator in `pharmacy-bcma-domain`.
- Late-doc flag derived from administered-vs-scheduled time diff.

### §3.12 IVAdmixture

- Smart pump drug library push via vendor adapter (Alaris, Plum 360, Hospira).
- Hard limits non-bypassable in pump-program output.

### §3.13 ControlledSubstance

- Witness signing for every CII transaction.
- DEA Form 222 ordering workflow.
- DEA inspection-ready reporting query.

### §3.14 Reimbursement

- 340B eligibility evaluator with auditable encounter evidence trail.
- NCPDP D.0 PBM claim builder + reject classifier.
- Handoff to `cloud-billing` via gRPC.

### §3.15 Operations

- Workflow queues: order/prep/verify/deliver.
- Pharmacist workload balancer.

### §3.16 Interventions

- Clinical intervention capture with MTM billable mapping.

### §3.17 MedRec

- Admission/transfer/discharge reconciliation sessions.
- Diff signed by clinician.

### §3.18 OutpatientRX

- Retail counter / drive-through / mail-order / specialty.
- Limited-distribution-drug enrollment tracking.

### §3.19 MTM

- CMR / TMR.
- MAP + PML generation via `intelligence` substrate (T3 with redaction contract).

### §3.20 DSCSA

- SGTIN-198 GS1 model.
- EPCIS 2.0-aligned T1/T2/T3.
- Saleable returns verification.
- Suspect-product investigation workflow.

---

## §4 Inter-microservice contracts

### §4.1 gRPC dependencies (consumer side)

```
pharmacy ──→ emr.Patient            (read patient identity)
pharmacy ──→ emr.Encounter          (read encounter context)
pharmacy ──→ emr.Allergy            (read allergy list)
pharmacy ──→ emr.Problem            (read condition list)
pharmacy ──→ emr.Labs               (read lab values)
pharmacy ──→ emr.MAR                (write MAR records)
pharmacy ──→ identity.Principal     (resolve prescriber + pharmacist + nurse)
pharmacy ──→ cloud-iam.Cedar        (gate every guarded action)
pharmacy ──→ cloud-kms.Sign         (EPCS sign envelope)
pharmacy ──→ cloud-kms.Verify       (audit-chain Merkle verify)
pharmacy ──→ cloud-secrets.Fetch    (Surescripts mTLS, PBM creds)
pharmacy ──→ audit-chain.Seal       (seal every controlled event)
pharmacy ──→ observability.Emit     (SLOs + metrics)
pharmacy ──→ cloud-billing.Charge   (dispense settlement)
pharmacy ──→ compliance.Pack        (HIPAA + DEA pack hooks)
pharmacy ──→ healthcare-integration.HL7  (inbound legacy systems)
pharmacy ──→ tenancy.Resolve        (tenant ctx)
pharmacy ──→ cell.Membership        (cell placement)
pharmacy ──→ comms-email.Send       (refill reminders)
pharmacy ──→ forms.Render           (REMS attestations)
pharmacy ──→ intelligence.Compose   (MTM PML drafting)
pharmacy ──→ consent-graph.Enforce  (hub program data sharing)
```

### §4.2 gRPC services (provider side)

```
pharmacy.MedicationCatalogService
pharmacy.FormularyService
pharmacy.ePrescribeService
pharmacy.DrugInteractionService
pharmacy.AllergyCheckService
pharmacy.DoseCheckService
pharmacy.VerificationService
pharmacy.CompoundingService
pharmacy.InventoryService
pharmacy.AutoDispensingService
pharmacy.BCMAService
pharmacy.IVAdmixtureService
pharmacy.ControlledSubstanceService
pharmacy.ReimbursementService
pharmacy.OperationsService
pharmacy.InterventionsService
pharmacy.MedRecService
pharmacy.OutpatientRXService
pharmacy.MTMService
pharmacy.DSCSAService
```

### §4.3 Event topics (AsyncAPI)

```
oya.pharmacy.rx.prescribed
oya.pharmacy.rx.verified
oya.pharmacy.rx.dispensed
oya.pharmacy.rx.administered
oya.pharmacy.rx.refused
oya.pharmacy.rx.alert.ddi
oya.pharmacy.rx.alert.dai
oya.pharmacy.rx.alert.dci
oya.pharmacy.rx.alert.dpi
oya.pharmacy.rx.alert.drc
oya.pharmacy.compounding.completed
oya.pharmacy.inventory.recall-sequestered
oya.pharmacy.cabinet.discrepancy
oya.pharmacy.controlled.witness-signed
oya.pharmacy.dscsa.suspect-product-opened
oya.pharmacy.dscsa.saleable-return-verified
oya.pharmacy.reimbursement.claim-accepted
oya.pharmacy.reimbursement.claim-rejected
oya.pharmacy.intervention.captured
oya.pharmacy.medrec.session-completed
```

### §4.4 Audit-chain seal events

Every event in §4.3 is sealed to `audit-chain` with bilateral cross-pointer when crossing tenants (e.g., outpatient specialty hub program).

---

## §5 Data stores

| Store | Purpose | Pin |
|---|---|---|
| PostgreSQL (Citus) | Per-cell relational store: catalog, formulary, orders, dispenses, audits. | 16.2 + Citus 12.1 |
| Pulsar | Event bus per cell. | 3.2 |
| OpenBao | Secrets (PBM, Surescripts mTLS). | 2.0 |
| MinIO / S3-compat | Long-term audit + DSCSA T3 history (10y retention). | tenant-local |
| Valkey | Cedar policy decision cache (5s freshness budget). | 8.x |
| Cassandra (optional) | High-throughput BCMA scan log per cell. | 5.0 |

---

## §6 Cedar policy gates (selected)

| Policy file | Purpose |
|---|---|
| `pharmacist-can-verify.cedar` | Only licensed pharmacist of correct state can verify; DEA registration required for CII. |
| `prescriber-can-eprescribe.cedar` | Only DEA-bound prescriber can EPCS; KMS key binding required for CII–CV. |
| `nurse-can-administer.cedar` | Only RN/LPN at active shift can record administration; license-residency enforced. |
| `dea-controlled-2x-verify.cedar` | Schedule II prescriptions require dual pharmacist verification. |
| `allergy-override-requires-justification.cedar` | Allergy alert override at severity ≥ severe requires reason code + two-step. |
| `formulary-non-formulary-dispense.cedar` | Non-formulary dispense requires medical-director approval case. |
| `compounding-usp800-cell-capability.cedar` | USP 800 compounding only on cells tagged `iso-7-negative-pressure`. |
| `cabinet-override-witness.cedar` | Cabinet override > Schedule III requires witness signature. |
| `b340-mixed-use-eligibility.cedar` | 340B determination requires eligible encounter + eligible provider evidence. |
| `mtm-pml-redaction.cedar` | MTM PML draft via intelligence must apply redaction profile. |

---

## §7 Observability

### §7.1 SLOs (≥ 10; registered with `observability`)

1. `oya-pharmacy-eprescribe-roundtrip-latency` — p95 ≤ 5 s.
2. `oya-pharmacy-ddi-check-latency` — p99 ≤ 200 ms.
3. `oya-pharmacy-dispense-cycle-latency` — p99 ≤ 2 s.
4. `oya-pharmacy-bcma-scan-latency` — p99 ≤ 100 ms.
5. `oya-pharmacy-dispense-availability` — ≥ 99.99%.
6. `oya-pharmacy-bcma-availability` — ≥ 99.99%.
7. `oya-pharmacy-eprescribe-availability` — ≥ 99.9%.
8. `oya-pharmacy-catalog-read-availability` — ≥ 99.95%.
9. `oya-pharmacy-audit-chain-coverage-completeness` — 1.0.
10. `oya-pharmacy-controlled-substance-witness-integrity` — 1.0.
11. `oya-pharmacy-340b-classification-accuracy` — ≥ 99.99% (audit-sample-driven).
12. `oya-pharmacy-dscsa-saleable-return-verification-rate` — ≥ 99.9%.

### §7.2 Metrics

- `oya_pharmacy_orders_total{state}` — per-state counter.
- `oya_pharmacy_alerts_total{kind, severity}` — DDI/DAI/DCI/DPI/DRC alerts.
- `oya_pharmacy_overrides_total{kind, reason}` — overrides with justification reason.
- `oya_pharmacy_dispense_duration_seconds{cell}` — histogram.
- `oya_pharmacy_bcma_duration_seconds{cell}` — histogram.
- `oya_pharmacy_eprescribe_duration_seconds{outbound|inbound}` — histogram.
- `oya_pharmacy_controlled_witnesses_total{schedule}` — counter.
- `oya_pharmacy_dscsa_serials_received_total` — counter.
- `oya_pharmacy_cabinet_discrepancies_total{vendor, location}` — counter.
- `oya_pharmacy_b340_determinations_total{outcome}` — counter.

### §7.3 Tracing

- W3C Trace Context + B3 multi-header.
- Every event in §4.3 carries `trace_id` propagated from inbound HTTP/3 or NCPDP envelope.

---

## §8 Security model

### §8.1 Identity

- Prescribers, pharmacists, nurses authenticated via `identity` OIDC.
- DEA registration verified against DEA Diversion Control number registry (out-of-band cache, monthly refresh).
- License residency verified against `cloud-iam` license-overlay.

### §8.2 Cryptography

- All at-rest data encrypted with tenant-CMK via `cloud-kms` (BYOK supported per ADR-0255 §D-4).
- EPCS signing keys: HSM-backed, never present in worker memory longer than the sign operation.
- Audit-chain Merkle keys: HSM-backed.

### §8.3 Network

- HTTP/3 + QUIC by default (per ADR-0253).
- mTLS for Surescripts, PBM connections, cabinet vendor adapters.
- Cilium L4 + Istio Ambient (ztunnel + waypoint) for service mesh.
- Egress allowlist for: Surescripts, PBM endpoints, FDB/Multum, GS1 EPCIS partners, cabinet vendor cloud (where applicable).

### §8.4 Break-glass

- Pharmacist-in-charge can elevate to "all-policies-permissive-with-audit" via `cloud-iam` break-glass; every action sealed and reviewed within 24 h.

---

## §9 Tenancy + scoping

- Every row in every store carries `tenant_id` (per ADR-0244 tenant scoping primitive).
- Per-tenant formulary and per-tenant suppression rules.
- Per-tenant knowledge package selection.
- Per-tenant DEA registration (multi-facility tenants enumerate facilities under one tenant).

---

## §10 Compliance pack overlays (per ADR-0251)

| Pack | Effect |
|---|---|
| `hipaa` | PHI classification + minimum-necessary + access review + 6y audit retention. |
| `dea-controlled-substance` | EPCS, dual-verify CII, witnessed waste, perpetual inventory cadence, Form 222 ordering, DEA inspection-ready report. |
| `gdpr` | EU-residency + DSAR cascade + data-subject revocation + Article 30 RoPA. |
| `pci-dss` | Cardholder-data on outpatient retail capture isolated to pci-zone cells; tokenization via `cloud-billing`. |
| `eu-ai-act` | Intelligence-substrate calls (MTM PML drafting) classified Annex III; risk-mitigation evidence linked. |
| `lgpd` | BR data-subject overlay. |
| `cn-pipl-2021` | CN residency overlay (used only in CN-hosted deployments). |
| `kr-pipa` | KR overlay. |
| `state-board-of-pharmacy` | Per-state pharmacy registration overlay (CA, TX, NY, FL, etc.). |
| `dscsa` | Title II FDASIA serialization + T3 retention 6y. |
| `usp-797` | Sterile compounding cell-capability gate. |
| `usp-800` | Hazardous-drug ISO-7 negative-pressure cell-capability gate. |
| `340b` | OPAIS reporting + replenishment lot tagging. |
| `ncpdp-script` | ePrescribing message conformance. |
| `surescripts` | Surescripts EHR-vendor accreditation evidence. |

---

## §11 Deployment contexts (6) and IaC layout

Per `feedback_zero_handroll_opentofu_only_2026_05_20`, every context lands via OpenTofu modules.

```
iac/
├── aws-guest/          OpenTofu modules for AWS-hosted oyatie ⇒ pharmacy on EKS + RDS + MSK.
├── oci-guest/          OpenTofu modules for OCI-hosted oyatie ⇒ pharmacy on OKE + Autonomous DB + Streaming.
│   └── always-free/    OCI Always Free for sandbox/demo/trial/dev tenants ⇒ Ampere A1 ARM, Autonomous DB Always Free, Object Storage.
├── on-prem/            OpenTofu modules for customer-controlled on-prem ⇒ K8s (Talos / OKD / vanilla) + PostgreSQL Citus + Pulsar.
├── colo/               OpenTofu modules for colo-controlled deployment ⇒ Talos K8s + storage abstractions.
├── oyatie-cloud/       OpenTofu modules for oyatie-as-cloud-provider ⇒ tenant-isolated cells with Cloud Hypervisor + Kata pods.
└── sovereign/          OpenTofu modules for sovereign clouds (CSAP/IL5/C5/CCCS) ⇒ air-gapped substrate + offline knowledge package distribution.
```

Each context has:
- `main.tf` — module entrypoint.
- `versions.tf` — provider pins.
- `variables.tf` — context-specific knobs.
- `outputs.tf` — endpoint exports.
- `secrets.tf` — secret bindings.

---

## §12 Failure modes + degraded operation

| Failure | Behavior |
|---|---|
| Surescripts outbound endpoint down | Queue messages; alert at 5 min queue depth; replay on recovery. |
| FDB knowledge package corrupt | Roll back to previous version (A/B); alert P&T committee. |
| Cabinet vendor cloud unreachable | Cabinets operate in offline mode (last-known state); reconcile on reconnect. |
| EMR gRPC unavailable | Pharmacy continues to dispense already-active orders; reject new order ingestion; alert. |
| Cedar policy decision endpoint down | Fail closed for elevated capabilities; fail open for read-only capabilities. |
| Audit-chain emission backlog | Pharmacy continues to operate; backlog queued; alert at 1 min. |
| 340B determination data stale | Mark dispenses as "pending-340b-classification"; reconcile on freshness. |
| PBM connection down | Queue claims; alert at 10 min; reconcile on reconnect. |
| BCMA endpoint down (rare) | Scanner falls back to local cache; reconcile on reconnect; pharmacist callback within 5 min. |

---

## §13 Capacity model (high level)

- 1 cell sized for 250 hospital beds equivalent ≈ 1 M dispenses/year ≈ 50 M BCMA scans/year.
- Outpatient retail cell sized for 10 K prescriptions/day ≈ 3 M Rx/year.
- Specialty pharmacy cell sized for 500 patients on limited-distribution programs.

---

## §14 Roadmap

- Wave 15M-E (this wave): scaffold, manifest, contracts, 10 SLOs, 5 Cedar policies, 10 IPs, 2 service ADRs.
- Wave 16: IP-001..IP-010 execution; first end-to-end inpatient flow on dev.
- Wave 17: outpatient retail + specialty pharmacy paths.
- Wave 18: USP 797/800 cell-capability gating fully wired.
- Wave 19: DSCSA T3 with two named upstream wholesalers.
- Wave 20: 340B OPAIS reporting MVP.

---

## §15 References

- ADR-0056 Rust Clean Architecture BNF.
- ADR-0105 13-layer enum.
- ADR-0131 per-microservice flat layout.
- ADR-0132 no-suite microservices.
- ADR-0145 inter-microservice direct gRPC.
- ADR-0214 cross-tenant real-time visibility.
- ADR-0248 hyperscaler-shape cellular architecture.
- ADR-0251 compliance-pack primitive.
- ADR-0252 HLC default + TrueTime tier.
- ADR-0253 HTTP/3 + QUIC default.
- ADR-0254 K8s + Cloud Hypervisor.
- ADR-0255 intelligence two-layer substrate.
- ADR-0328 multispectrum review v2.4.0.
- ADR-0332 pharmacy substrate authorization.

---

## §16 Sequence: inpatient order-to-administration (full path)

```
PRESCRIBER (emr UI)
  ▼ POST /MedicationRequest (emr)
emr ──gRPC─▶ pharmacy.ePrescribe.CreatePrescription
                       ▼
              pharmacy.MedicationCatalog.Resolve(rxcui)         (cache 60s)
                       ▼
              pharmacy.Formulary.Lookup(tenant, cell, rxcui)    (cache 5s)
                       ▼  parallel fan-out
              pharmacy.DrugInteraction.Evaluate(8 engines)      (p99 ≤ 200ms)
              pharmacy.AllergyCheck.Check
              pharmacy.DoseCheck.Check
              pharmacy.MedicationCatalog.duplicate-therapy
                       ▼  (Cedar gate: prescriber-can-eprescribe)
              audit-chain.Seal('rx.prescribed')                 (sub-tick seal)
                       ▼
              pulsar emit oya.pharmacy.rx.prescribed
                       ▼  Verification worker subscribes
              pharmacy.Verification.Verify
                  if schedule == CII: dual-verify path (Cedar gate)
                       ▼
              audit-chain.Seal('rx.verified')
                       ▼
              pulsar emit oya.pharmacy.rx.verified
                       ▼  AutoDispensing worker subscribes
              pharmacy.Inventory.Decrement(lot, qty)
                  if recall-sequestered → HB-5 abort
                       ▼
              pharmacy.AutoDispensing.IngestTransaction
                       ▼
              audit-chain.Seal('rx.dispensed')
                       ▼
              pulsar emit oya.pharmacy.rx.dispensed
                       ▼
NURSE (BCMA handheld) ──HTTP/3──▶ pharmacy.BCMA.Scan
              five-rights validator
                  if any failed → HB-7 abort (or Cedar override)
                       ▼
              pharmacy.BCMA → emr.MAR.write
                       ▼
              audit-chain.Seal('rx.administered')
                       ▼
              pulsar emit oya.pharmacy.rx.administered
                       ▼
              pharmacy.Reimbursement.Charge (handoff)
              cloud-billing.Charge.Post
                       ▼
              audit-chain.Seal('reimbursement.claim-accepted' | 'claim-rejected')
```

Total expected hops sealed in audit-chain per inpatient dispense: 9 (prescribe / verify / dispense / administer / claim / DSCSA-serial-attach / cabinet-decrement / EPCS-signed-if-CII / break-glass-if-any).

---

## §17 Sequence: outpatient retail with PBM

```
PATIENT/PRESCRIBER → Surescripts NewRx
              ▼
pharmacy.ePrescribe.ReceiveSurescriptsInbound
              ▼
pharmacy.OutpatientRX.EnrollPatient (if new)
              ▼
pharmacy.Reimbursement.SubmitPBMClaim (NCPDP D.0)
              if reject: re-route to pharmacist queue
              if accept: continue
              ▼
pharmacy.Verification.Verify
              ▼
pharmacy.OutpatientRX.PrintLabel + counsel-required gating
              ▼
PATIENT pickup → barcode scan → audit-chain seal
              ▼
pharmacy.Reimbursement.SettleToCloudBilling
              ▼
pulsar emit oya.pharmacy.rx.dispensed (channel=retail)
```

---

## §18 Sequence: EPCS Schedule II controlled-substance

```
PRESCRIBER → emr UI initiates CII Rx
              ▼
pharmacy.ePrescribe.EPCSSignEnvelopeRequest
              ▼  identity step-up (two-factor evidence captured)
              ▼  cloud-kms.Sign(DEA-bound key)
              ▼
audit-chain.Seal('eprescribe.epcs-signed')
              ▼
Surescripts EPCS transmission
              ▼
Receiving pharmacy (oyatie or peer) intake
              ▼
pharmacy.Verification.Verify (dual-verify required)
              ▼
pharmacy.ControlledSubstance.RecordTransaction (witness-signed)
              ▼
audit-chain.Seal('controlled.witness-signed')
              ▼
Dispense and administration follow §16
```

---

## §19 Sequence: DSCSA T3 receiving + perpetual inventory entry

```
WHOLESALER delivers carton with case-level SGTIN-198 hierarchy
              ▼
RECEIVING TECH scans each SGTIN-198 at dock
              ▼
pharmacy.DSCSA.IngestT3 (TI + TH + TS bundle)
              for each SGTIN:
                  pharmacy.DSCSA.VerifyAgainstT3
                  if mismatch → SuspectProductCase
                  if verified → pharmacy.Inventory.IngestSerial
              ▼
audit-chain.Seal('dscsa.transaction-verified')
              ▼
SGTIN serials enter perpetual inventory; available for dispense
```

---

## §20 Capacity model (worked example)

A 250-bed acute-care hospital ≈

- 1,000 dispenses/day (inpatient + outpatient retail)
- 50,000 BCMA scans/year
- 4,000 EPCS controlled-substance Rx/year
- 200 IV admixture prep/day
- 50 USP 797 sterile compounds/day
- 5 USP 800 hazardous compounds/day

Cell sizing:
- 8 worker pods (8 vCPU + 16 GiB each).
- PostgreSQL leader + 2 replicas (db.r6g.xlarge equivalent).
- Pulsar 3-broker cluster.
- Valkey 3-node Cedar decision cache.

Auto-scale triggers:
- HPA on `oya_pharmacy_dispense_queue_depth > 100`.
- HPA on `oya_pharmacy_eprescribe_queue_depth > 50`.
- HPA on CPU > 70% sustained 5 min.

Scaleout pattern: shuffle-sharded second cell carries another 250-bed unit before the first cell saturates.

---

## §21 Frontend touchpoints

Pharmacy contributes the following frontend surfaces (per `feedback_rust_strict_only_no_python_2026_05_20` frontend-language allowance):

| Surface | Stack | Audience |
|---|---|---|
| Pharmacist workstation | WinUI 3 (Windows 11) — preferred | Inpatient pharmacist (verification, queue) |
| Pharmacist workstation | macOS Apple Silicon native (SwiftUI) | Alternative for Apple-stack pharmacy |
| BCMA handheld | iOS native (Swift) on iOS 18+ iPad/iPhone | Nurse |
| BCMA handheld | Android (Kotlin) on Zebra TC52/TC57 enterprise devices | Nurse alternative |
| Retail counter | WinUI 3 / Web (HTTP/3) | Retail technician + pharmacist |
| Patient B2C app | iOS (Swift) + Android (Kotlin) | Outpatient patient (refills, will-call) |

All frontends are thin clients over the pharmacy REST + AsyncAPI surfaces — no business logic on the device.

---

## §22 Knowledge-package distribution

Knowledge packages (FDB, Multum, Medi-Span, RxNorm) are non-trivial to distribute, especially to air-gapped sovereign deployments.

- **Online deployments** (aws-guest / oci-guest / oyatie-cloud) — monthly fetch from vendor endpoint via signed package; tenant `knowledge_package_version` selects A or B.
- **Sovereign (air-gapped)** — package is delivered via signed offline media; OpenTofu `sovereign` module accepts a path and verifies signature + checksum at apply time.
- **A/B switching** — every tenant carries two pinned versions; switching is an atomic database update; rollback < 5 min.
- **Audit** — every knowledge-package activation is sealed under `oya.pharmacy.catalog.knowledge-package-activated`.

---

## §23 Reference architectures (counterparts)

The architecture borrows shape from:

- **Stripe** — idempotency keys on every dispense + every claim write; ETag on every read; versioned API.
- **Palantir Ontology** — bounded context aggregates project into the ontology substrate; no direct ontology mutation from pharmacy.
- **Linear** — sub-second event-driven update stream backing the operations queue UI.
- **AWS IAM service-linked roles** — pharmacy capability principals are Cedar-modeled like IAM SLRs.
- **Google Cloud service agents** — pharmacy workers run under `oyatie.pharmacy.*` service agent identities with least-privilege.
- **Cerner Millennium PowerChart Pharmacy** — verification workflow shape borrowed (tall-man + alert dismissal + dual-verify CII), without the monolith.
- **Epic Willow** — Bedside Verification flow (BCMA five-rights) shape borrowed.
- **BD Pyxis MedStation** — cabinet adapter contract shape borrowed for vendor-neutral abstraction.

---

## §24 Performance benchmarks (target on Tier-1 OS amd64)

Performance numbers below are TARGETS for IP-001..IP-010 completion; measured benchmarks land in `benchmarks/` under each bounded context.

| Operation | Target p50 | Target p99 | Notes |
|---|---|---|---|
| MedicationCatalog.Resolve (cache hit) | 0.5 ms | 5 ms | Valkey cache layer. |
| MedicationCatalog.Resolve (cache miss) | 15 ms | 50 ms | PostgreSQL fall-through. |
| Formulary.Lookup | 1 ms | 10 ms | Cached 5 s. |
| DrugInteraction.Evaluate (8 engines parallel) | 50 ms | 200 ms | Parallel fan-out. |
| AllergyCheck.Check | 5 ms | 30 ms | RxNorm + UNII match. |
| DoseCheck.Check | 5 ms | 30 ms | Pure compute. |
| Verification.Verify | 50 ms | 200 ms | Audit + Cedar gate. |
| BCMA.Scan | 20 ms | 100 ms | Hot path. |
| Inventory.Decrement | 10 ms | 50 ms | Transactional. |
| AutoDispensing.IngestTransaction | 5 ms | 30 ms | Idempotent. |
| ePrescribe.Transmit (Surescripts) | 1 s | 5 s | Network-dominated. |
| DSCSA.VerifySaleableReturn | 30 ms | 150 ms | Lookup. |

All benchmarks run under `cargo bench` via `criterion`; numbers persisted to `benchmarks/<bounded-context>.md` per IP completion.

---

## §25 Open-source dependencies (LTS-pinned)

| Crate / package | Version pin | Reason |
|---|---|---|
| `tokio` | 1.43+ | Async runtime. |
| `tonic` | 0.13+ | gRPC over HTTP/3. |
| `axum` | 0.8+ | REST over HTTP/3. |
| `quinn` | 0.11+ | QUIC. |
| `sqlx` | 0.8+ | PostgreSQL async client. |
| `cedar-policy` | 3.2 | Policy compile + eval. |
| `pulsar-rs` | 6.x | Pulsar client. |
| `prost` | 0.13+ | Protobuf. |
| `serde` | 1.x | Serialization. |
| `opentelemetry` | 0.27+ | Tracing + metrics. |
| `criterion` | 0.5+ | Benchmarks. |
| `proptest` | 1.x | Property tests. |

LTS pins are tracked in the workspace `Cargo.toml`; tenant deployments cannot diverge.

---
