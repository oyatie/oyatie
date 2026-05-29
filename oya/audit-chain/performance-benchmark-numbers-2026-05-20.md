# audit-chain Performance Benchmark Numbers - 2026-05-20

## Citation Anchors
1. Canonical sequence: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-2125` and `:3900-4235` define deployment-context, OpenTofu, OS, Rust, OCI Always Free, and audit-agent audit requirements.
2. Master plan machine source: `specs/master-plan-sequencing.json:704-868` defines the six deployment contexts, OpenTofu substrate, OS matrix, language policy, and OCI Always Free sub-profile.
3. Product source read: `microservices/audit-chain/PRD.md:18-361` defines audit-chain as the append-only, policy-checked, cryptographically sealed evidence ledger with performance targets.
4. Architecture source read: `microservices/audit-chain/ARCHITECTURE.md:9-754` defines principals, Merkle sealing, Cedar authorization, cell placement, and runtime shape.
5. Rigor source: `docs/standards/documentation-rigor.md:133-156` requires intern-buildability and hyperscaler-grade documentation rather than scaffold summaries.
6. Counterpart source: AWS CloudTrail user guide and quotas: `https://docs.aws.amazon.com/awscloudtrail/latest/userguide/cloudtrail-user-guide.html` and `https://docs.aws.amazon.com/awscloudtrail/latest/userguide/WhatIsCloudTrail-Limits.html`.
7. Counterpart source: Google Cloud Logging and Audit Logs quotas: `https://cloud.google.com/logging/quotas` and `https://docs.cloud.google.com/logging/docs/audit`.
8. Counterpart source: Microsoft Purview Audit and Office 365 Management Activity API: `https://learn.microsoft.com/en-us/purview/audit-solutions-overview`, `https://learn.microsoft.com/en-us/purview/audit-search`, and `https://learn.microsoft.com/en-us/office/office-365-management-api/office-365-management-activity-api-reference`.

## Methodology Disclosure
These are target benchmark numbers and counterpart quota/limit reference numbers, not measured Oyatie benchmark results.
Measured benchmark results must be added during the build phase under the ADR-0212 benchmark discipline referenced by this wave.
The service-local docs already contain target numbers in `PRD.md:55-63`, `PRD.md:266-272`, `capacity-model.md:139-147`, and `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:15-112`.
Where counterpart vendors publish quotas rather than p50/p95/p99 latency benchmarks, this document records those quotas as external reference constraints.
Where counterpart latency is operationally described but not published as a precise benchmark, this document marks the number as an estimated comparison target and does not claim vendor-measured parity.
All Oyatie numbers below are build targets that must become automated benchmarks before release.
All context-specific numbers are capacity-model targets for planning, not evidence that the current audit-chain implementation already achieves the target.
Every OCI demo_trial tenant_class target is reconciled to the OCI Always Free constraint in `specs/master-plan-sequencing.json:857-868`.
Every non-OCI demo_trial tenant_class target is the standard demo_trial tenant_class floor unless explicitly constrained by customer-owned hardware or guest-provider quota.
No target below overrides the P1 finding that service-local six-context OpenTofu IaC is missing.

## §1 Methodology
Benchmark dimension: emit latency p50 measures accepted append request duration before durability acknowledgment.
Benchmark dimension: emit latency p95 measures normal-tail producer experience under the stated steady workload.
Benchmark dimension: emit latency p99 measures extreme-tail producer experience under burst plus seal-worker contention.
Benchmark dimension: seal latency p99 measures time from batch close to signed Merkle root persistence.
Benchmark dimension: verification latency p95 measures inclusion-proof verification for a bounded evidence bundle.
Benchmark dimension: query latency p95 measures filtered event lookup through the service query API, not downstream SIEM search.
Benchmark dimension: throughput measures accepted audit events per second after authorization and schema validation.
Benchmark dimension: burst throughput measures accepted audit events per second for a bounded burst window with no data loss.
Benchmark dimension: concurrent writers measures independent producer identities writing through the service API.
Benchmark dimension: scale ceiling measures the target practical maximum before horizontal topology change.
Benchmark workload E1: append-only management events, 1 KiB payload, one tenant, one region, Cedar allow decision already warm.
Benchmark workload E2: mixed management and data-access events, 2 KiB median payload, 20 percent policy-denied events.
Benchmark workload E3: chain-of-custody evidence events, 8 KiB median payload, mandatory integrity metadata, HSM-backed seal.
Benchmark workload E4: regulator export workload, 5 million events selected by tenant, principal, and time range.
Benchmark workload E5: replay verification workload, 100 thousand event proofs plus one signed Merkle checkpoint.
Benchmark workload E6: incident response workload, 25 simultaneous saved searches and 5 export streams per tenant.
Benchmark workload E7: cross-cell replication workload, active cell plus one witness cell and one cold archive target.
Benchmark workload E8: degraded HSM workload, one signer unavailable, quorum falls back to remaining signers.
Benchmark workload E9: OCI Always Free workload, single Ampere instance pair where capacity fits Free Tier envelope.
Benchmark workload E10: on-prem disconnected workload, no public cloud callback, local HSM or software-HSM test profile.
OS disclosure: Tier-1 OS support must cover the 13 OS families in `specs/master-plan-sequencing.json:777-815`.
Architecture disclosure: the current service docs do not include `supported-oses.json`, so per-OS numbers are required targets rather than validated packages.
Architecture disclosure: `ARCHITECTURE.md:445-456` describes Kubernetes pods and Cloud Hypervisor/Kata isolation, but does not provide context OpenTofu modules.
Architecture disclosure: `capacity-model.md:57-69` models Postgres storage, but the audit found no service-local SQL schema.
Architecture disclosure: `capacity-model.md:93-109` models HSM capacity, including an OCI Cloud-HSM baseline.
Architecture disclosure: `capacity-model.md:139-147` names high-level ingest, seal, verify, and query baselines.
Tenant disclosure: demo_trial tenant_class targets assume one tenant cell or constrained shared cell.
Tenant disclosure: paid tenant_class targets assume multi-tenant production cell with paid managed services or customer-owned equivalent.
Tenant disclosure: paid tenant_class targets assume regulated multi-region production with active verification capacity.
Tenant disclosure: paid tenant_class targets assume single-tenant capable, hyperscaler-grade isolation and independent evidence export.
Deployment disclosure: `oyatie-public-cloud` numbers assume Oyatie-managed cloud infrastructure.
Deployment disclosure: `guest-on-aws` numbers assume AWS-hosted guest tenancy using AWS primitives only through provider abstraction.
Deployment disclosure: `guest-on-oci` numbers distinguish OCI Always Free demo_trial from paid OCI paid tenant_class.
Deployment disclosure: `on-prem` numbers assume customer-owned Kubernetes and customer-managed HSM or approved software-HSM test adapter.
Deployment disclosure: `colo` numbers assume dedicated hardware, private networking, and operator-managed HSM.
Deployment disclosure: `oyatie-as-cloud-provider` numbers assume Oyatie-owned provider substrate and provider-grade control-plane observability.
Stop condition: these targets are useful only after implementation reports measured p50/p95/p99 and resource usage per context and per Tier-1 OS.

## §2 Counterpart Numbers
| Counterpart | Number | Source/provenance | Audit-chain interpretation |
|---|---:|---|---|
| AWS CloudTrail | 90 days event history | AWS user guide describes Event history as the past 90 days of management events | demo_trial tenant_class must provide at least 90-day management-event visibility unless OCI Always Free forces a smaller local hot storage layer plus archive |
| AWS CloudTrail | 3,653 days CloudTrail Lake retention | AWS user guide states One-year extendable retention can keep event data for up to 3,653 days | paid tenant_class cold retention should reach 10-year-equivalent policy where law requires it |
| AWS CloudTrail | 2,557 days seven-year retention option | AWS user guide states seven-year retention pricing keeps data up to 2,557 days | paid tenant_class retention should expose a seven-year control |
| AWS CloudTrail | 5 trails per Region | AWS quotas table | Oyatie should not artificially cap tenants below equivalent trail partitioning without documented reason |
| AWS CloudTrail | 10 event data stores per Region | AWS quotas table | Audit-chain should declare equivalent ledger partitions per tenant/cell |
| AWS CloudTrail | 5 event selectors per trail | AWS quotas table | Audit-chain event-class filtering should document selector limits |
| AWS CloudTrail | 500 advanced selector conditions | AWS quotas table | Policy/event filters should support comparable complexity at paid tenant_class |
| AWS CloudTrail | 250 data resources across event selectors | AWS quotas table | Resource-scoped audit selection should have a documented scale limit |
| AWS CloudTrail | 256 KiB event size for CloudWatch/EventBridge delivery | AWS quotas table | Audit-chain contracts should cap or split oversize events explicitly |
| AWS CloudTrail | 50 MB S3 log file before compression | AWS quotas table | Sealed archive segment size should be explicit and less than operationally unsafe object size |
| AWS CloudTrail | 2 TPS for LookupEvents API example | AWS quotas page | Oyatie query targets should exceed this for tenant-owned evidence queries |
| AWS CloudTrail | Near-real-time within minutes | AWS decision guide and CloudTrail operational docs | Audit-chain emit acknowledgement should target sub-second local durability and minutes-or-less downstream export |
| AWS CloudTrail | Multi-account organization event data stores | AWS user guide | paid tenant_class targets require organization/tenant aggregation capacity |
| AWS CloudTrail | External event-source channel quota 25 | AWS quotas table | Audit-chain should model producer integration limits per tenant |
| AWS CloudTrail | Saved Lake query results available up to 7 days | AWS concepts docs | Audit-chain exports should state retention for query artifacts |
| Google Cloud Audit Logs | 512 KiB max audit log entry | Google Logging quotas | Audit-chain event contract should define a comparable max and oversize envelope |
| Google Cloud Logging | 256 KB generic LogEntry size | Google Logging quotas | Audit-chain generic event path should not rely on unbounded payloads |
| Google Cloud Logging | 10 MB `entries.write` request size | Google Logging API quotas | Audit-chain batch append API should publish request size limits |
| Google Cloud Logging | 4.8 GB/min ingestion in major regions | Google quotas | paid tenant_class public-cloud target should size toward multi-GB/min ingest, even if not immediate GA |
| Google Cloud Logging | 300 MB/min ingestion in other regions | Google quotas | Guest or constrained regions need explicit lower ceilings |
| Google Cloud Logging | 10 live-tail sessions per project | Google quotas | Audit-chain live/near-live observer limits should be declared if implemented |
| Google Cloud Logging | 60,000 live-tail entries/minute | Google quotas | Stream export capacity should define per-session event ceiling |
| Google Cloud Logging | 200 sinks per project, increasable to 4,000 | Google quotas | Audit-chain export sinks should define demo_trial tenant_class/paid tenant_class limits |
| Google Cloud Logging | 30 log views per bucket | Google quotas | Audit-chain saved views and evidence views should have tenant_class limits |
| Google Cloud Logging | 200 bucket query fanout | Google quotas | Audit-chain query fanout across cells/buckets should have a published ceiling |
| Google Cloud Logging | 20 restricted fields per bucket | Google quotas | Audit-chain restricted-field controls need an explicit limit if privacy filtering is provided |
| Google Cloud Logging | `_Required` retention 400 days | Google quotas | demo_trial tenant_class/paid tenant_class immutable compliance event retention should document a floor |
| Google Cloud Logging | `_Default` retention 30 days | Google quotas | Short hot retention is acceptable only with archive policy |
| Google Cloud Logging | user bucket retention 1 to 3,650 days | Google quotas | paid tenant_class should expose 10-year-equivalent retention policy |
| Google Cloud Logging | 20,000 character query length | Google quotas | Audit-chain query language needs a bounded max length |
| Microsoft Purview Audit | 180 days Audit Standard default retention | Microsoft Purview audit overview | demo_trial tenant_class retention below 180 days is a known catch-up gap unless OCI Always Free constrained |
| Microsoft Purview Audit | 1 year default for Entra, Exchange, OneDrive, SharePoint in Audit Premium | Microsoft Purview audit overview | paid tenant_class should match one-year high-value workload retention |
| Microsoft Purview Audit | 10 years with add-on license | Microsoft Purview audit overview | paid tenant_class must map long legal hold to paid tenant_class |
| Microsoft Purview Audit | 10 concurrent audit search jobs per admin user | Microsoft audit search docs | Audit-chain search concurrency should declare per-principal limits |
| Microsoft Purview Audit | one unfiltered search job per admin user | Microsoft audit search docs | Audit-chain should prevent unbounded scans from starving scoped searches |
| Microsoft Purview Audit | completed search jobs kept 30 days | Microsoft audit search docs | Audit-chain export/search-artifact retention should be explicit |
| Office 365 Management Activity API | 2,000 requests/minute initial tenant allocation | Microsoft API reference and troubleshooting docs | paid tenant_class API egress should size above basic tenant polling needs |
| Office 365 Management Activity API | E5 organizations approximately twice non-E5 bandwidth | Microsoft API reference | Tier differentiation may include higher polling/export capacity |
| Office 365 Management Activity API | content blobs by workload type | Microsoft API reference | Audit-chain exports should preserve workload/event-class channels |
| Office 365 Management Activity API | webhook endpoint validation on subscription | Microsoft API reference | Audit-chain export webhooks need validation semantics |
| Office 365 Management Activity API | API does not deduplicate duplicates automatically | Microsoft troubleshooting docs | Audit-chain export clients need idempotency tokens and duplicate handling |
| Microsoft Purview Audit | unified auditing must be enabled | Microsoft API reference | Audit-chain should document service enablement/precondition checks |
| Microsoft Purview Audit | audit log search on by default for most enterprise organizations | Microsoft audit enablement docs | Audit-chain managed tenants should enable baseline auditing by default |
| Microsoft schema | common schema plus service-specific schema | Microsoft schema docs | Audit-chain event schema should support common fields and service-specific extensions |
| Microsoft Purview Audit | 30-day completed-search history | Microsoft audit search docs | Audit-chain saved evidence-search retention should not be implicit |

## §3 Oyatie Target Numbers
### oyatie-public-cloud demo_trial tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 20,000 events/sec | `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:15-42` demo_trial tenant_class baseline |
| burst ingest | 80,000 events/sec for 5 minutes | `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:15-42` burst baseline |
| emit API p50 | 20 ms | target derived from `PRD.md:55-63` low-latency emit requirement |
| emit API p95 | 60 ms | target stays inside `PRD.md:55-63` p95 class |
| emit API p99 | 120 ms | planning target for burst tail |
| seal batch p99 | 1.5 sec | `PRD.md:55-63` and tenant_class model seal target |
| verification p95 | 500 ms for 100k-proof bundle | `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:15-42` verification class |
| query p95 | 2.0 sec for scoped 7-day query | derived from `capacity-model.md:139-147` query baseline |
| concurrent writers | 1,000 producer identities | demo_trial tenant_class managed-cell floor |
| hot/cold retention | 90 days hot, 1 year cold | `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:15-42` |
| availability | 99.9 percent monthly | `PRD.md:88-91` and demo_trial tenant_class |

### oyatie-public-cloud paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 100,000 events/sec | `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:44-66` |
| burst ingest | 500,000 events/sec for 10 minutes | paid tenant_class burst target |
| emit API p50 | 15 ms | paid baseline target |
| emit API p95 | 45 ms | inside `PRD.md:55-63` |
| emit API p99 | 90 ms | paid baseline burst target |
| seal batch p99 | 1.0 sec | paid tenant_class and PRD seal target |
| verification p95 | 250 ms for 1M-proof bundle | paid tenant_class verification target |
| query p95 | 1.5 sec for 30-day scoped query | paid query target |
| concurrent writers | 5,000 producer identities | multi-tenant production floor |
| hot/cold retention | 365 days hot, 7 years cold | paid tenant_class retention target |
| availability | 99.95 percent monthly | paid tenant_class service target |

### oyatie-public-cloud paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 500,000 events/sec | `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:68-92` |
| burst ingest | 2,000,000 events/sec for 15 minutes | paid tenant_class burst target |
| emit API p50 | 12 ms | regulated public-cloud target |
| emit API p95 | 35 ms | regulated public-cloud target |
| emit API p99 | 70 ms | regulated public-cloud target |
| seal batch p99 | 500 ms | paid tenant_class verification/sealing class |
| verification p95 | 150 ms for 10M-proof bundle | paid tenant_class verification target |
| query p95 | 1.0 sec for 90-day scoped query | paid tenant_class search target |
| concurrent writers | 25,000 producer identities | organization-scale floor |
| hot/cold retention | 7 years hot-indexed, 25 years archive | paid tenant_class retention target |
| availability | 99.99 percent monthly | `PRD.md:88-91` upper availability class |

### oyatie-public-cloud paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 250,000 events/sec per single-tenant cell | `ADR-0329 + ADR-0330 + ADR-0331 tenant_class model:94-112` |
| burst ingest | 1,000,000 events/sec per isolated cell | paid tenant_class single-tenant cell target |
| emit API p50 | 10 ms | hyperscaler public-cloud target |
| emit API p95 | 30 ms | hyperscaler public-cloud target |
| emit API p99 | 60 ms | hyperscaler public-cloud target |
| seal batch p99 | 400 ms | dedicated HSM and witness target |
| verification p95 | 100 ms for 10M-proof bundle | paid tenant_class proof target |
| query p95 | 800 ms for 90-day scoped query | dedicated query plane target |
| concurrent writers | 50,000 producer identities | dedicated tenant cell target |
| hot/cold retention | 10 years hot-policy capable, 25 years archive | Purview/CloudTrail parity target |
| availability | 99.99 percent plus single-tenant isolation SLO | paid tenant_class cell target |

### guest-on-aws demo_trial tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 10,000 events/sec | guest-provider demo_trial tenant_class planning target |
| burst ingest | 40,000 events/sec for 5 minutes | constrained guest baseline |
| emit API p50 | 25 ms | AWS guest latency target |
| emit API p95 | 80 ms | AWS guest latency target |
| emit API p99 | 160 ms | AWS guest burst-tail target |
| seal batch p99 | 2.0 sec | external-provider HSM/storage overhead target |
| verification p95 | 750 ms for 100k-proof bundle | guest demo_trial tenant_class proof target |
| query p95 | 2.5 sec for scoped 7-day query | guest demo_trial tenant_class query target |
| concurrent writers | 500 producer identities | guest demo_trial tenant_class floor |
| hot/cold retention | 90 days hot, 1 year cold | counterpart and demo_trial tenant_class floor |
| availability | 99.9 percent if customer AWS account meets prerequisites | deployment-contract target |

### guest-on-aws paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 80,000 events/sec | paid guest baseline |
| burst ingest | 320,000 events/sec for 10 minutes | paid guest burst target |
| emit API p50 | 18 ms | AWS guest paid target |
| emit API p95 | 55 ms | AWS guest paid target |
| emit API p99 | 110 ms | AWS guest paid target |
| seal batch p99 | 1.2 sec | managed HSM/storage target |
| verification p95 | 300 ms for 1M-proof bundle | paid proof target |
| query p95 | 1.8 sec for 30-day scoped query | paid query target |
| concurrent writers | 4,000 producer identities | paid guest floor |
| hot/cold retention | 365 days hot, 7 years cold | CloudTrail/Purview parity target |
| availability | 99.95 percent with validated AWS primitives | paid guest SLO |

### guest-on-aws paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 300,000 events/sec | guest-account regulated target |
| burst ingest | 1,200,000 events/sec for 15 minutes | regulated guest burst target |
| emit API p50 | 15 ms | AWS guest regulated target |
| emit API p95 | 45 ms | AWS guest regulated target |
| emit API p99 | 90 ms | AWS guest regulated target |
| seal batch p99 | 800 ms | multi-AZ HSM target |
| verification p95 | 200 ms for 10M-proof bundle | regulated proof target |
| query p95 | 1.2 sec for 90-day scoped query | regulated search target |
| concurrent writers | 15,000 producer identities | large tenant floor |
| hot/cold retention | 7 years hot-indexed, 25 years archive | paid tenant_class retention target |
| availability | 99.99 percent if customer enables required regional services | regulated SLO |

### guest-on-aws paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 200,000 events/sec per dedicated tenant cell | single-tenant AWS target |
| burst ingest | 800,000 events/sec per isolated cell | single-tenant burst target |
| emit API p50 | 12 ms | dedicated AWS target |
| emit API p95 | 35 ms | dedicated AWS target |
| emit API p99 | 75 ms | dedicated AWS target |
| seal batch p99 | 600 ms | dedicated signer target |
| verification p95 | 125 ms for 10M-proof bundle | dedicated proof target |
| query p95 | 900 ms for 90-day scoped query | dedicated query target |
| concurrent writers | 40,000 producer identities | single-tenant floor |
| hot/cold retention | 10 years policy-capable, 25 years archive | long-retention parity target |
| availability | 99.99 percent plus dedicated-tenant isolation | paid tenant_class AWS target |

### guest-on-oci demo_trial tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 300 events/sec | OCI Always Free envelope from `specs/master-plan-sequencing.json:857-868` |
| burst ingest | 1,000 events/sec for 60 seconds | Always Free constrained burst |
| emit API p50 | 35 ms | single-Ampere-cell target |
| emit API p95 | 120 ms | constrained demo_trial tenant_class target |
| emit API p99 | 300 ms | constrained demo_trial tenant_class target |
| seal batch p99 | 5.0 sec | no paid Cloud-HSM assumption in Always Free |
| verification p95 | 500 ms for 10k-proof bundle | reduced Always Free proof workload |
| query p95 | 3.0 sec for 7-day scoped query | constrained storage target |
| concurrent writers | 25 producer identities | Always Free tenant floor |
| hot/cold retention | 7 days hot, 30 days cold unless external customer storage is attached | Always Free reconciliation |
| availability | 99.0 percent planning floor, not the standard demo_trial tenant_class SLO | Always Free reconciliation |

### guest-on-oci paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 80,000 events/sec | paid OCI baseline target |
| burst ingest | 320,000 events/sec for 10 minutes | paid OCI burst target |
| emit API p50 | 18 ms | paid OCI target |
| emit API p95 | 55 ms | paid OCI target |
| emit API p99 | 110 ms | paid OCI target |
| seal batch p99 | 1.2 sec | OCI Cloud-HSM paid target from service capacity model |
| verification p95 | 300 ms for 1M-proof bundle | paid proof target |
| query p95 | 1.8 sec for 30-day scoped query | paid query target |
| concurrent writers | 4,000 producer identities | paid OCI floor |
| hot/cold retention | 365 days hot, 7 years cold | paid tenant_class retention target |
| availability | 99.95 percent with paid OCI primitives | paid OCI SLO |

### guest-on-oci paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 300,000 events/sec | regulated OCI target |
| burst ingest | 1,200,000 events/sec for 15 minutes | regulated burst target |
| emit API p50 | 15 ms | regulated OCI target |
| emit API p95 | 45 ms | regulated OCI target |
| emit API p99 | 90 ms | regulated OCI target |
| seal batch p99 | 800 ms | multi-signer OCI target |
| verification p95 | 200 ms for 10M-proof bundle | regulated proof target |
| query p95 | 1.2 sec for 90-day scoped query | regulated query target |
| concurrent writers | 15,000 producer identities | large tenant floor |
| hot/cold retention | 7 years hot-indexed, 25 years archive | paid tenant_class retention target |
| availability | 99.99 percent with validated OCI HA topology | regulated SLO |

### guest-on-oci paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 200,000 events/sec per dedicated tenant cell | dedicated OCI target |
| burst ingest | 800,000 events/sec per isolated cell | dedicated OCI burst target |
| emit API p50 | 12 ms | dedicated OCI target |
| emit API p95 | 35 ms | dedicated OCI target |
| emit API p99 | 75 ms | dedicated OCI target |
| seal batch p99 | 600 ms | dedicated signer target |
| verification p95 | 125 ms for 10M-proof bundle | dedicated proof target |
| query p95 | 900 ms for 90-day scoped query | dedicated query target |
| concurrent writers | 40,000 producer identities | dedicated tenant floor |
| hot/cold retention | 10 years policy-capable, 25 years archive | long-retention parity target |
| availability | 99.99 percent plus dedicated-tenant isolation | paid tenant_class OCI target |

### on-prem demo_trial tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 5,000 events/sec | customer-owned minimum hardware target |
| burst ingest | 20,000 events/sec for 5 minutes | on-prem demo_trial tenant_class burst target |
| emit API p50 | 30 ms | local cluster target |
| emit API p95 | 100 ms | local cluster target |
| emit API p99 | 220 ms | constrained local storage target |
| seal batch p99 | 3.0 sec | customer HSM/software-HSM variance target |
| verification p95 | 900 ms for 100k-proof bundle | local proof target |
| query p95 | 3.0 sec for 7-day scoped query | local query target |
| concurrent writers | 250 producer identities | on-prem floor |
| hot/cold retention | 90 days hot, customer archive policy for cold | on-prem retention target |
| availability | 99.5 percent unless customer cluster declares higher | on-prem demo_trial tenant_class SLO |

### on-prem paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 50,000 events/sec | paid on-prem baseline |
| burst ingest | 200,000 events/sec for 10 minutes | paid on-prem burst target |
| emit API p50 | 22 ms | paid on-prem target |
| emit API p95 | 70 ms | paid on-prem target |
| emit API p99 | 140 ms | paid on-prem target |
| seal batch p99 | 1.5 sec | customer HSM target |
| verification p95 | 350 ms for 1M-proof bundle | paid proof target |
| query p95 | 2.0 sec for 30-day scoped query | paid query target |
| concurrent writers | 2,500 producer identities | paid on-prem floor |
| hot/cold retention | 365 days hot, 7 years customer archive | paid tenant_class retention target |
| availability | 99.9 percent with validated customer cluster | paid on-prem SLO |

### on-prem paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 200,000 events/sec | regulated on-prem target |
| burst ingest | 800,000 events/sec for 15 minutes | regulated on-prem burst target |
| emit API p50 | 18 ms | regulated on-prem target |
| emit API p95 | 55 ms | regulated on-prem target |
| emit API p99 | 110 ms | regulated on-prem target |
| seal batch p99 | 900 ms | clustered HSM target |
| verification p95 | 250 ms for 10M-proof bundle | regulated proof target |
| query p95 | 1.5 sec for 90-day scoped query | regulated query target |
| concurrent writers | 10,000 producer identities | regulated floor |
| hot/cold retention | 7 years hot-indexed, 25 years archive | paid tenant_class retention target |
| availability | 99.95 percent with audited customer HA | regulated on-prem SLO |

### on-prem paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 150,000 events/sec per dedicated tenant cell | dedicated on-prem target |
| burst ingest | 600,000 events/sec per isolated cell | dedicated on-prem burst target |
| emit API p50 | 15 ms | dedicated on-prem target |
| emit API p95 | 45 ms | dedicated on-prem target |
| emit API p99 | 90 ms | dedicated on-prem target |
| seal batch p99 | 700 ms | dedicated signer target |
| verification p95 | 150 ms for 10M-proof bundle | dedicated proof target |
| query p95 | 1.0 sec for 90-day scoped query | dedicated query target |
| concurrent writers | 30,000 producer identities | dedicated on-prem floor |
| hot/cold retention | 10 years policy-capable, 25 years archive | long-retention parity target |
| availability | 99.99 percent if customer provides audited redundant facilities | paid tenant_class on-prem target |

### colo demo_trial tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 15,000 events/sec | dedicated hardware demo_trial tenant_class target |
| burst ingest | 60,000 events/sec for 5 minutes | colo demo_trial tenant_class burst target |
| emit API p50 | 22 ms | low-latency private network target |
| emit API p95 | 70 ms | colo demo_trial tenant_class target |
| emit API p99 | 150 ms | colo demo_trial tenant_class burst-tail target |
| seal batch p99 | 1.8 sec | local HSM target |
| verification p95 | 650 ms for 100k-proof bundle | colo proof target |
| query p95 | 2.2 sec for 7-day scoped query | colo query target |
| concurrent writers | 750 producer identities | colo demo_trial tenant_class floor |
| hot/cold retention | 90 days hot, 1 year cold | demo_trial tenant_class retention target |
| availability | 99.9 percent with redundant power/network | colo demo_trial tenant_class SLO |

### colo paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 120,000 events/sec | paid colo target |
| burst ingest | 480,000 events/sec for 10 minutes | paid colo burst target |
| emit API p50 | 16 ms | paid colo target |
| emit API p95 | 50 ms | paid colo target |
| emit API p99 | 100 ms | paid colo target |
| seal batch p99 | 1.0 sec | local HSM paid target |
| verification p95 | 250 ms for 1M-proof bundle | paid proof target |
| query p95 | 1.5 sec for 30-day scoped query | paid query target |
| concurrent writers | 6,000 producer identities | paid colo floor |
| hot/cold retention | 365 days hot, 7 years cold | paid tenant_class retention target |
| availability | 99.95 percent with redundant colo design | paid tenant_class colo SLO |

### colo paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 600,000 events/sec | high-throughput colo target |
| burst ingest | 2,400,000 events/sec for 15 minutes | high-throughput burst target |
| emit API p50 | 12 ms | high-throughput colo target |
| emit API p95 | 35 ms | high-throughput colo target |
| emit API p99 | 70 ms | high-throughput colo target |
| seal batch p99 | 500 ms | local clustered HSM target |
| verification p95 | 150 ms for 10M-proof bundle | paid tenant_class proof target |
| query p95 | 1.0 sec for 90-day scoped query | paid tenant_class query target |
| concurrent writers | 30,000 producer identities | high-throughput floor |
| hot/cold retention | 7 years hot-indexed, 25 years archive | paid tenant_class retention target |
| availability | 99.99 percent with multi-site colo witness | paid tenant_class colo SLO |

### colo paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 300,000 events/sec per dedicated tenant cell | isolated colo target |
| burst ingest | 1,200,000 events/sec per isolated cell | isolated burst target |
| emit API p50 | 10 ms | private colo target |
| emit API p95 | 30 ms | private colo target |
| emit API p99 | 60 ms | private colo target |
| seal batch p99 | 400 ms | dedicated HSM target |
| verification p95 | 100 ms for 10M-proof bundle | paid tenant_class proof target |
| query p95 | 800 ms for 90-day scoped query | dedicated query target |
| concurrent writers | 60,000 producer identities | dedicated colo floor |
| hot/cold retention | 10 years policy-capable, 25 years archive | long-retention parity target |
| availability | 99.99 percent plus isolated dedicated facilities | paid tenant_class colo target |

### oyatie-as-cloud-provider demo_trial tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 20,000 events/sec | provider-substrate demo_trial tenant_class target |
| burst ingest | 80,000 events/sec for 5 minutes | provider-substrate burst target |
| emit API p50 | 20 ms | provider control-plane target |
| emit API p95 | 60 ms | provider control-plane target |
| emit API p99 | 120 ms | provider control-plane target |
| seal batch p99 | 1.5 sec | demo_trial tenant_class seal target |
| verification p95 | 500 ms for 100k-proof bundle | demo_trial tenant_class verification target |
| query p95 | 2.0 sec for scoped 7-day query | demo_trial tenant_class query target |
| concurrent writers | 1,000 producer identities | provider demo_trial tenant_class floor |
| hot/cold retention | 90 days hot, 1 year cold | demo_trial tenant_class retention target |
| availability | 99.9 percent with provider control-plane redundancy | provider demo_trial tenant_class SLO |

### oyatie-as-cloud-provider paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 150,000 events/sec | provider paid baseline |
| burst ingest | 600,000 events/sec for 10 minutes | provider paid burst target |
| emit API p50 | 14 ms | provider paid target |
| emit API p95 | 42 ms | provider paid target |
| emit API p99 | 85 ms | provider paid target |
| seal batch p99 | 900 ms | provider signer target |
| verification p95 | 225 ms for 1M-proof bundle | provider proof target |
| query p95 | 1.3 sec for 30-day scoped query | provider query target |
| concurrent writers | 7,500 producer identities | provider paid floor |
| hot/cold retention | 365 days hot, 7 years cold | paid tenant_class retention target |
| availability | 99.95 percent with provider-managed substrate | provider paid tenant_class SLO |

### oyatie-as-cloud-provider paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 750,000 events/sec | provider-grade paid tenant_class target |
| burst ingest | 3,000,000 events/sec for 15 minutes | provider-grade burst target |
| emit API p50 | 10 ms | provider-grade target |
| emit API p95 | 30 ms | provider-grade target |
| emit API p99 | 60 ms | provider-grade target |
| seal batch p99 | 450 ms | provider-grade signer target |
| verification p95 | 125 ms for 10M-proof bundle | provider-grade proof target |
| query p95 | 800 ms for 90-day scoped query | provider-grade query target |
| concurrent writers | 40,000 producer identities | provider-grade floor |
| hot/cold retention | 7 years hot-indexed, 25 years archive | paid tenant_class retention target |
| availability | 99.99 percent with multi-cell provider architecture | provider paid tenant_class SLO |

### oyatie-as-cloud-provider paid tenant_class
| Metric | Target | Provenance |
|---|---:|---|
| sustained ingest | 500,000 events/sec per dedicated tenant cell | provider hyperscaler target |
| burst ingest | 2,000,000 events/sec per isolated cell | provider hyperscaler burst target |
| emit API p50 | 8 ms | hyperscaler target |
| emit API p95 | 25 ms | hyperscaler target |
| emit API p99 | 50 ms | hyperscaler target |
| seal batch p99 | 350 ms | dedicated provider signer target |
| verification p95 | 80 ms for 10M-proof bundle | hyperscaler proof target |
| query p95 | 600 ms for 90-day scoped query | hyperscaler query target |
| concurrent writers | 100,000 producer identities | hyperscaler producer target |
| hot/cold retention | 10 years hot-policy capable, 25 years archive | long-retention parity target |
| availability | 99.99 percent plus dedicated provider-cell isolation | provider paid tenant_class SLO |

## §4 Per-context Overlay
| Context | Performance overlay |
|---|---|
| oyatie-public-cloud | Uses the standard tenant_class model because Oyatie controls cloud placement, storage layout, signer placement, and query-worker density. |
| oyatie-public-cloud | Must publish benchmark runs per Tier-1 OS package even if Kubernetes hides host variance, because ADR-0328 requires OS evidence. |
| oyatie-public-cloud | Primary risk is proving provider-neutral controls while still using managed cloud primitives below the port boundary. |
| guest-on-aws | Throughput should be lower than Oyatie public cloud at demo_trial tenant_class because customer accounts and regional quotas vary. |
| guest-on-aws | paid tenant_class can recover parity if OpenTofu modules provision validated HSM, storage, and queue capacity. |
| guest-on-aws | CloudTrail parity is strong for AWS-native events, but audit-chain must also seal Oyatie domain events that AWS CloudTrail does not know about. |
| guest-on-oci | demo_trial tenant_class is explicitly constrained by OCI Always Free and cannot claim the standard demo_trial tenant_class 20k events/sec target. |
| guest-on-oci | paid tenant_class can use paid OCI primitives including Cloud-HSM assumptions already referenced by `capacity-model.md:99`. |
| guest-on-oci | The missing `iac/oci-guest/always-free/` module means every OCI demo_trial tenant_class number is a target needing implementation proof. |
| on-prem | Numbers depend on customer hardware, HSM, storage, and network; the docs must publish minimum profiles instead of single global guarantees. |
| on-prem | The current docs do not publish OS package or CI lane evidence, so no on-prem number is release-proven today. |
| on-prem | Verification workloads are often more important than ingest because disconnected investigations need local proof generation. |
| colo | Colo can exceed cloud demo_trial tenant_class/paid tenant_class due to dedicated hardware and private networking, but only if the module defines storage and signer layouts. |
| colo | Multi-site witness topology is required before paid tenant_class availability claims become credible. |
| colo | Dedicated archive paths can support long retention but must be modeled in OpenTofu, not Helm-only values. |
| oyatie-as-cloud-provider | Provider-grade numbers are the highest targets because Oyatie controls substrate, tenant cells, and evidence export surfaces. |
| oyatie-as-cloud-provider | ADR-0328 currently names `iac/oyatie-iaas/` while this wave prompt names `oyatie-as-cloud-provider`; benchmark artifacts must normalize that naming. |
| oyatie-as-cloud-provider | This context should become the proving ground for paid tenant_class single-tenant evidence-chain isolation. |

## §5 Comparison Narrative
AWS publishes strong quota surfaces for event history, event data stores, selector counts, event size, and retention, but not complete p50/p95/p99 API latency for every audit operation.
Oyatie targets are ahead of AWS CloudTrail LookupEvents throughput because even demo_trial tenant_class query targets are higher than the AWS example quota of 2 TPS.
Oyatie is currently catch-up on documented selector and channel limits because audit-chain contracts do not publish equivalent hard ceilings.
Oyatie is partial parity on retention because the tenant_class model reaches one year, seven years, ten years, and twenty-five years in places, but OCI demo_trial tenant_class conflicts with that floor.
Oyatie is ahead on cryptographic seal semantics when implemented because Merkle/HSM sealing is a first-class product purpose in `PRD.md:18-26`.
Oyatie is behind AWS on proven service maturity because the audit-chain docs still contain missing OpenTofu context modules and no measured benchmark runs.
Google publishes ingestion limits, audit-entry limits, sink counts, live-tail limits, view counts, query fanout, and retention windows.
Oyatie paid tenant_class ingest targets are lower than Google major-region 4.8 GB/minute unless average event size is small; this is a catch-up area for provider-scale audit ingest.
Oyatie is ahead of Google on evidence-chain verification as a first-class user-facing proof surface, based on PRD and architecture intent.
Oyatie is behind Google on documented sink/export quotas because no service-local export sink limit matrix exists.
Oyatie should treat Google `_Required` 400-day retention as a useful compliance floor for immutable baseline audit classes.
Microsoft publishes retention tiers, search concurrency, API request baseline, content blob model, and audit schema surfaces.
Oyatie demo_trial tenant_class is behind Microsoft Audit Standard if it retains only 90 days hot and one year cold but does not expose 180 days searchable in all contexts.
Oyatie paid tenant_class/paid tenant_class can reach Microsoft retention parity if seven-year and ten-year policy controls are implemented and measured.
Oyatie is behind Microsoft on API export mechanics until webhook validation, de-duplication, and blob/content-channel semantics are documented.
Oyatie is ahead of Microsoft on tenant-owned cryptographic proof objectives if Merkle inclusion and HSM signatures are available to customers.
The headline p95 emit target of 50 ms from `PRD.md:55-63` is stronger than the counterpart quota-style public data, but it is only an assertion until benchmark harnesses land.
The headline p99 seal target near one second is defensible for demo_trial tenant_class/paid tenant_class public cloud but not OCI Always Free demo_trial without a paid signer.
The headline verification target is strong when proof bundles stay bounded, but the docs need exact proof-size language in contracts.
The headline query target is weaker than cloud-native managed log analytics unless the query plane adds indexes, saved views, and export partitioning comparable to CloudTrail Lake and Cloud Logging.
The top remediation is to add measured benchmark harnesses keyed by context, tenant_class, OS, arch, tenant class, and workload ID.
The second remediation is to split `demo_trial-OCI-AF` from standard demo_trial tenant_class or explicitly state that OCI Always Free demo_trial is a constrained evaluation tenant_class.
The third remediation is to publish hard maximums for event size, batch request size, selectors, sinks, views, search concurrency, export streams, and saved-query retention.
The fourth remediation is to wire benchmark evidence to CI lanes once `supported-oses.json` exists.
The fifth remediation is to make every benchmark run record OpenTofu module version, provider version, HSM profile, storage backend, CPU arch, and OS package provenance.
