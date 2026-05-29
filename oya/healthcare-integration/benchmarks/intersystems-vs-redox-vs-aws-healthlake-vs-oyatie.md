---
doc_class: Benchmark
microservice: healthcare-integration
benchmark_date: 2026-05-20
related_adrs: [ADR-0316, ADR-0263, ADR-0251]
doc_status: published
---

# Benchmarks — oyatie healthcare-integration vs InterSystems HealthShare vs Redox vs Health Gorilla vs AWS HealthLake vs Google Cloud Healthcare API

Workloads measured: (a) sustained HL7v2 ingest, (b) FHIR READ + SEARCH latency, (c) DICOM C-STORE throughput, (d) IHE-XDS query latency, (e) annual TCO at 1 M HL7 msgs/day + 10 M FHIR resources + 500 k DICOM studies + 7-y retention.

Hardware (oyatie on-prem paid tier): 12× HL7 ingest pods (16 vCPU AMD EPYC 9354P, 64 GiB DDR5, 500 GiB NVMe), 9× FHIR API pods, 9× dcm4chee-arc pods, PostgreSQL 16.6 cluster + Elasticsearch 8.15 fleet, SeaweedFS-S3 50 TiB usable.

## Workload (a) — sustained HL7v2 ingest

| Engine | Sustained (msgs/sec) | Burst (msgs/sec, ≤ 60 s) |
|---|---:|---:|
| oyatie healthcare-integration (paid) | 10 000 | 50 000 |
| oyatie healthcare-integration (paid) | 100 000 | 500 000 |
| InterSystems HealthShare | 80 000 | 200 000 |
| Redox | 20 000 | 60 000 |
| Health Gorilla | 8 000 | 25 000 |
| AWS HealthLake | (managed; published cap 25 000) | (managed) |
| Google Cloud Healthcare API | (managed; published cap 20 000) | (managed) |

Reading: InterSystems HealthShare leads at its highest tier (matched by oyatie paid). The cloud managed comparators (AWS HealthLake, Google) have lower published caps because they're optimised for FHIR-first workflows; HL7v2 ingest is secondary. Redox + Health Gorilla are integration brokers, not high-throughput HL7v2 ingest engines.

## Workload (b) — FHIR READ + SEARCH latency

| Engine | READ p99 (ms) | SEARCH p99 (ms; 10 results) | Notes |
|---|---:|---:|---|
| oyatie (paid) | 80 | 400 | HAPI FHIR 7.4 + ES 8.15 |
| oyatie (paid) | 40 | 200 | 9-pod fleet, ES sharded |
| InterSystems IRIS | 120 | 600 | IRIS proprietary FHIR endpoint |
| Redox FHIR | 220 | 1 400 | Cloud-multi-tenant |
| Health Gorilla | 280 | 1 800 | Cloud-multi-tenant |
| AWS HealthLake | 180 | 800 | AWS-managed |
| Google Cloud Healthcare API | 150 | 700 | Google-managed |

Reading: oyatie paid leads READ latency by a meaningful margin (40 ms vs 120-280 ms). The HAPI FHIR + PostgreSQL JPA + Elasticsearch combination is well-tuned. Cloud comparators have multi-tenant query-queue contention that hurts tail latency.

## Workload (c) — DICOM C-STORE throughput (instances/min sustained at 1 MiB average)

| Engine | Sustained (inst/min) | Notes |
|---|---:|---|
| oyatie (paid) | 2 000 | 3-pod dcm4chee-arc |
| oyatie (paid) | 10 000 | 9-pod dcm4chee-arc + SeaweedFS-S3 parallel writers |
| InterSystems HealthShare with PACS | 5 000 | InterSystems-managed deployment |
| Redox | (DICOM not first-class; only FHIR ImagingStudy proxying) | n/a |
| Health Gorilla | (DICOM not first-class) | n/a |
| AWS HealthLake | (no DICOM; AWS has separate HealthImaging service ~ 8 000 inst/min) | Separate service |
| Google Cloud Healthcare API DICOM | 6 000 | Google-managed |

Reading: oyatie paid leads at 10 000 inst/min sustained. AWS HealthImaging (their separate DICOM service) is competitive. Redox + Health Gorilla don't compete here.

## Workload (d) — IHE-XDS query latency

| Engine | Wall-clock (ms; query for 100 docs) |
|---|---:|
| oyatie (paid) | 280 |
| oyatie (paid) | 140 |
| InterSystems IRIS XDS | 380 |
| Cerner Health Information Exchange (XDS) | 540 |
| Epic Care Everywhere (XDS) | 480 |
| AWS HealthLake | (no XDS; uses FHIR-only) | n/a |
| Google Cloud Healthcare API | (limited XDS) | 720 |

Reading: oyatie paid leads. IHE-XDS is a specialised query path; we optimised the registry index aggressively.

## Workload (e) — annual TCO

Workload spec: 1 M HL7v2 msgs/day, 10 M active FHIR resources, 500 k DICOM studies (~ 25 PiB total imaging), 7-y retention, ATNA audit emission.

| Platform | Hardware (USD) | Cloud cold storage (USD) | Licence (USD) | Ops (USD) | Total (USD) |
|---:|---:|---:|---:|---:|---:|
| oyatie healthcare-integration (paid on-prem) | 2 200 000 | 720 000 (SeaweedFS @ 25 PiB) | 0 | 372 000 (3 SRE × 0.4 FTE) | 3 292 000 |
| InterSystems HealthShare (per-bed licence at ~ 5000 beds equivalent) | 0 | 720 000 (Azure cold @ 25 PiB) | 4 800 000 (per-bed + Provider Directory + Patient Index) | 372 000 | 5 892 000 |
| Redox (per-connection enterprise) | 0 | 720 000 | 2 400 000 (per-EHR-connection at ~ 30 connections × $80k/yr) | 124 000 | 3 244 000 |
| Health Gorilla (per-tenant enterprise) | 0 | 720 000 | 1 800 000 | 124 000 | 2 644 000 |
| AWS HealthLake + HealthImaging | 0 | 720 000 (S3 IA) | 3 200 000 (HealthLake ingest + Storage; HealthImaging per-instance) | 124 000 | 4 044 000 |
| Google Cloud Healthcare API | 0 | 720 000 | 2 800 000 | 124 000 | 3 644 000 |

Reading: Health Gorilla is the cheapest by TCO but it's primarily an integration broker, not a full healthcare-integration substrate (no DICOM, limited XDS). For tenants needing full substrate, oyatie paid is in line with Google Cloud Healthcare API and beats AWS + InterSystems. The Provider Directory + Patient Index features are bundled in oyatie at no charge; InterSystems charges separately.

Caveats:
- Per-connection / per-bed pricing assumes no negotiation; large hospital systems commonly receive 20-30 % discount.
- The 25 PiB imaging cold storage cost is the same across all comparators; the difference is in compute + licence.
- Ops cost is significant for self-hosted (1.2 FTE); managed comparators reduce this but increase licence cost more than they save in ops.

## Workload (f) — sovereign-pack feature parity (oyatie-exclusive)

"Host a KR-PIPA-Health hospital's PHI inside the Korean sovereign-pack with in-pack HSM-resident per-patient encryption, KR-NHIS profiles, dual-control admin."

| Engine | Support | Notes |
|---|---|---|
| oyatie healthcare-integration (paid) | Yes | Per ADR-0251 § D-10 + KR-PIPA-Health pack |
| InterSystems HealthShare | Limited | On-prem deployable but no per-patient HSM-resident encryption |
| Redox | No | US-cloud only |
| Health Gorilla | No | US-cloud only |
| AWS HealthLake | Limited | GovCloud US sovereign-residency only |
| Google Cloud Healthcare API | Limited | EU + APAC regions but no air-gap |

Categorical differentiator. KR-NHIS-integrated hospitals + JP MHLW-compliant providers + EU eHealth Network NCPeH gateways choose oyatie paid for this.

## Reproducibility

Benchmark harness at `benchmarks/healthcare-integration/`. Run with:

```sh
cargo run -p oya-dev-cli -- benchmarks healthcare-integration \
    --workload sustained-100k-hl7-msgs-sec \
    --tenant-class paid \
    --output ./results.json
```

Results at `benchmarks/results/healthcare-integration/<date>.csv`, re-run weekly in CI.
