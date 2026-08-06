# cloud-data feature parity matrix - 2026-05-20

## Header citation anchors
1. Canonical sequence and audit dimensions: `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-4148`.
2. Machine-readable deployment/IaC/OS/language/OCI constraints: `specs/master-plan-sequencing.json:704-866`.
3. cloud-data product-purpose anchor: ADR-0329, ADR-0330, and ADR-0331 tenant_class model; `microservices/cloud-data/faqs/data-engineer-faq.md:7-226`.
4. cloud-data architecture substitute anchor: `microservices/cloud-data/reference-implementations/multi-engine-and-pitr-rust-sdk.md:6-243`; `microservices/cloud-data/tutorials/multi-engine-pitr-and-truetime-workflow.md:1-230`.
5. Documentation-rigor anchor: `docs/standards/documentation-rigor.md:137-190`.
6. AWS source anchors: `https://docs.aws.amazon.com/AmazonRDS/latest/AuroraUserGuide/CHAP_AuroraOverview.html`, `https://docs.aws.amazon.com/AmazonRDS/latest/AuroraUserGuide/Aurora.Overview.StorageReliability.html`, `https://docs.aws.amazon.com/AmazonRDS/latest/AuroraUserGuide/Aurora.Managing.Backups.html`, `https://docs.aws.amazon.com/AmazonRDS/latest/AuroraUserGuide/aurora-global-database.html`, `https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/rds-proxy.html`, and `https://docs.aws.amazon.com/AmazonRDS/latest/AuroraUserGuide/UsingWithRDS.IAMDBAuth.html`.
7. Google Spanner source anchors: `https://docs.cloud.google.com/spanner/docs/editions-overview`, `https://docs.cloud.google.com/spanner/docs/performance`, `https://docs.cloud.google.com/spanner/docs/pitr`, `https://docs.cloud.google.com/spanner/docs/change-streams`, and `https://docs.cloud.google.com/spanner/quotas`.
8. Microsoft source anchors: `https://azure.microsoft.com/en-us/products/azure-sql/database/`, `https://learn.microsoft.com/en-us/azure/azure-sql/database/sql-database-paas-overview`, `https://learn.microsoft.com/en-us/azure/azure-sql/database/service-tiers-sql-database-vcore`, `https://learn.microsoft.com/en-my/AZURE/cosmos-db/overview`, `https://learn.microsoft.com/en-us/azure/cosmos-db/distribute-data-globally`, `https://learn.microsoft.com/en-us/azure/cosmos-db/consistency-levels`, `https://learn.microsoft.com/en-us/azure/cosmos-db/change-feed-design-patterns`, and `https://learn.microsoft.com/en-us/sql/relational-databases/security/ledger/ledger-overview`.

## §1 Counterpart 1 - AWS RDS+Aurora capability surface
1. AWS-01: Managed relational database service for MySQL-compatible and PostgreSQL-compatible engines.
2. AWS-02: Aurora storage subsystem customized below the database engine.
3. AWS-03: Aurora overview states up to five times MySQL throughput for some workloads.
4. AWS-04: Aurora overview states up to three times PostgreSQL throughput for some workloads.
5. AWS-05: Cluster volume auto-growth.
6. AWS-06: Aurora overview states cluster volume maximum size of 128 TiB in the opened overview.
7. AWS-07: Aurora storage docs describe cluster volume copies across three Availability Zones.
8. AWS-08: Shared storage independent of number of DB instances.
9. AWS-09: Aurora Standard storage configuration.
10. AWS-10: Aurora I/O-Optimized storage configuration.
11. AWS-11: Storage billing based on used space.
12. AWS-12: Multi-AZ database cluster posture.
13. AWS-13: Reader DB instances for horizontal read scaling.
14. AWS-14: Failover to reader/standby instances.
15. AWS-15: Aurora Serverless v2 compute scaling.
16. AWS-16: Serverless v2 provisioned-cluster mixed mode.
17. AWS-17: Serverless v2 granular per-second resource billing.
18. AWS-18: Serverless v2 reader instances.
19. AWS-19: Serverless v2 global database compatibility.
20. AWS-20: Aurora Global Database spanning one primary and up to ten secondary regions.
21. AWS-21: Global read locality in secondary regions.
22. AWS-22: Dedicated global replication infrastructure with typically sub-second replication.
23. AWS-23: Managed switchover for planned region rotation.
24. AWS-24: Managed failover for region outage recovery.
25. AWS-25: Write forwarding in global database.
26. AWS-26: Continuous and incremental automated backups.
27. AWS-27: Automated backup retention configurable from 1 to 35 days.
28. AWS-28: PITR within backup-retention window.
29. AWS-29: Latest restorable time typically within five minutes for active cluster.
30. AWS-30: Manual snapshots with no expiration.
31. AWS-31: AWS Backup integration for Aurora backups.
32. AWS-32: Database cloning.
33. AWS-33: Aurora MySQL Backtrack.
34. AWS-34: RDS Proxy connection pooling.
35. AWS-35: RDS Proxy standby reconnection preserving application connections.
36. AWS-36: RDS Proxy IAM authentication enforcement.
37. AWS-37: RDS Proxy Secrets Manager credential integration.
38. AWS-38: RDS Proxy throttling/load shedding rather than hard database overwhelm.
39. AWS-39: IAM database authentication with token-based login.
40. AWS-40: IAM authentication tokens generated with Signature Version 4.
41. AWS-41: IAM auth token lifetime of 15 minutes.
42. AWS-42: IAM-based centralized access control.
43. AWS-43: SSL/TLS database connection encryption.
44. AWS-44: Region and engine-version feature availability matrix.
45. AWS-45: CloudWatch metric integration for storage and performance.
46. AWS-46: Performance Insights support for Aurora Serverless v2 and provisioned features.
47. AWS-47: Blue/Green deployment support in Aurora Global Database topics.
48. AWS-48: RDS APIs and console-driven scaling/administration.
49. AWS-49: Parameter groups and engine-version management.
50. AWS-50: VPC-private proxy/network boundary.
51. AWS-51: Read replica scaling limits and proxy endpoint limits.
52. AWS-52: Snapshot restore and retained automated backup restore.
53. AWS-53: Encryption-key failure states in global database limitations.
54. AWS-54: Storage architecture that allows fast DB instance add/remove without copying table data.
55. AWS-55: Cost/performance storage choice between Standard and I/O-Optimized.
56. AWS-56: Application compatibility with existing MySQL/PostgreSQL tools.
57. AWS-57: Aurora cluster endpoints and writer endpoint model.
58. AWS-58: Engine major/minor version constraints for global failover/switchover.
59. AWS-59: DB activity stream mention for monitoring/auditing in limitations.
60. AWS-60: Region-specific backup windows and maintenance windows.

## §2 Counterpart 2 - Google Cloud Spanner capability surface
1. SPANNER-01: Managed mission-critical relational database service.
2. SPANNER-02: Strong ACID transaction model with GoogleSQL.
3. SPANNER-03: PostgreSQL interface support.
4. SPANNER-04: Key-value access model.
5. SPANNER-05: Standard edition.
6. SPANNER-06: Enterprise edition.
7. SPANNER-07: Enterprise Plus edition.
8. SPANNER-08: 99.99% SLA in Standard and Enterprise editions.
9. SPANNER-09: Up to 99.999% SLA in Enterprise Plus with multi-region configuration.
10. SPANNER-10: Regional instance configurations.
11. SPANNER-11: Dual-region instance configurations.
12. SPANNER-12: Multi-region instance configurations.
13. SPANNER-13: Optional custom read-only replicas.
14. SPANNER-14: Geo-partitioning in Enterprise Plus.
15. SPANNER-15: Spanner Graph.
16. SPANNER-16: Full-text search.
17. SPANNER-17: KNN vector search.
18. SPANNER-18: ANN vector search and vector index support.
19. SPANNER-19: Open-source autoscaler.
20. SPANNER-20: Managed autoscaler.
21. SPANNER-21: Asymmetric read-only autoscaling.
22. SPANNER-22: Locality groups.
23. SPANNER-23: Tiered storage.
24. SPANNER-24: BigQuery federation.
25. SPANNER-25: Spanner Data Boost.
26. SPANNER-26: Reverse ETL from BigQuery to Spanner.
27. SPANNER-27: Columnar engine.
28. SPANNER-28: Standard backups.
29. SPANNER-29: Scheduled backups.
30. SPANNER-30: Incremental backups.
31. SPANNER-31: Seven-day PITR.
32. SPANNER-32: Edition upgrade with no data migration.
33. SPANNER-33: Granular instances with 100 processing-unit minimum.
34. SPANNER-34: Feature usage monitoring metric.
35. SPANNER-35: Performance docs with linear scaling as compute capacity increases.
36. SPANNER-36: Regional SSD one-node approximate peak reads of 22,500 QPS.
37. SPANNER-37: Regional SSD one-node approximate peak writes of 3,500 QPS.
38. SPANNER-38: Throughput-optimized writes up to 22,500 QPS per node in regional SSD docs.
39. SPANNER-39: Optional read-only replica additional read throughput.
40. SPANNER-40: Ten TiB storage per node in current production limits.
41. SPANNER-41: Database limits per instance.
42. SPANNER-42: Schema object limits.
43. SPANNER-43: Backup restore operation limits.
44. SPANNER-44: Geo-partitioning limits including max partitions/placements.
45. SPANNER-45: Change streams for insert/update/delete data changes.
46. SPANNER-46: Change streams as DDL schema objects.
47. SPANNER-47: Change streams can watch whole database, table, or selected columns.
48. SPANNER-48: Change-stream retention configurable between one and thirty days.
49. SPANNER-49: Change-stream value capture modes.
50. SPANNER-50: Change-stream filters for TTL deletes and modification types.
51. SPANNER-51: Change-stream transaction-level record exclusion.
52. SPANNER-52: Change stream records include table, primary key, old/new values, commit timestamp, transaction ID, sequence number.
53. SPANNER-53: Dataflow connector integration.
54. SPANNER-54: Kafka connector integration.
55. SPANNER-55: Datastream integration to BigQuery/BigLake/Cloud Storage.
56. SPANNER-56: IAM permissions for quota and change stream operations.
57. SPANNER-57: Free trial limits: 10 GiB, up to five databases, no SLA.
58. SPANNER-58: Backup maximum retention of one year.
59. SPANNER-59: Monitoring metrics for schema object count and limits.
60. SPANNER-60: Official warning that performance numbers are estimates and workload-dependent.

## §3 Counterpart 3 - Azure SQL Database+Cosmos DB capability surface
1. AZURE-01: Azure SQL Database fully managed SQL database on SQL Server engine.
2. AZURE-02: General Purpose service tier.
3. AZURE-03: Business Critical service tier.
4. AZURE-04: Hyperscale service tier.
5. AZURE-05: Basic/Standard/Premium DTU-family posture in overview docs.
6. AZURE-06: Provisioned compute.
7. AZURE-07: Serverless compute.
8. AZURE-08: Serverless autoscaling and per-second billing.
9. AZURE-09: Single database deployment model.
10. AZURE-10: Elastic pool deployment model.
11. AZURE-11: Pool resource sharing for many databases.
12. AZURE-12: Manual dynamic scaling without downtime.
13. AZURE-13: Hyperscale up to 128 TB.
14. AZURE-14: vCore tier compute ranges up to 192 vCores in Hyperscale.
15. AZURE-15: Hyperscale local SSD cache.
16. AZURE-16: Business Critical local SSD storage.
17. AZURE-17: General Purpose remote premium storage.
18. AZURE-18: Business Critical maximum 327,680 IOPS in current vCore table.
19. AZURE-19: Hyperscale maximum 544,000 local SSD IOPS in current vCore table.
20. AZURE-20: Backup retention from 1 to 35 days.
21. AZURE-21: Long-term backup retention up to 10 years.
22. AZURE-22: Geo-redundant, zone-redundant, and locally redundant backup choices.
23. AZURE-23: Zone-redundant high availability.
24. AZURE-24: Business Critical replicas and read-scale replica.
25. AZURE-25: Hyperscale read-scale up to 30 named replicas per product page.
26. AZURE-26: Native vector search/AI-ready SQL capabilities.
27. AZURE-27: Mirroring in Microsoft Fabric.
28. AZURE-28: Data API builder REST/GraphQL exposure.
29. AZURE-29: Intelligent query processing.
30. AZURE-30: Azure Copilot/AI assistance.
31. AZURE-31: Ledger tables with SHA-256/Merkle tamper evidence.
32. AZURE-32: Updatable ledger tables.
33. AZURE-33: Append-only ledger tables.
34. AZURE-34: Ledger database and off-database digests.
35. AZURE-35: Cosmos DB fully managed NoSQL and vector database.
36. AZURE-36: Cosmos DB document model.
37. AZURE-37: Cosmos DB vector model.
38. AZURE-38: Cosmos DB key-value model.
39. AZURE-39: Cosmos DB graph model.
40. AZURE-40: Cosmos DB table model.
41. AZURE-41: Cosmos DB multi-region writes.
42. AZURE-42: Cosmos DB global distribution to Azure regions.
43. AZURE-43: Cosmos DB automatic failover.
44. AZURE-44: Cosmos DB 99.999% availability for multi-region databases.
45. AZURE-45: Cosmos DB p99 reads/writes under 10 ms in global distribution docs.
46. AZURE-46: Cosmos DB five consistency levels.
47. AZURE-47: Cosmos DB strong consistency blocked by default beyond 5,000 miles unless support enables it.
48. AZURE-48: Cosmos DB quorum model differences by consistency.
49. AZURE-49: Cosmos DB RPO table by consistency/regions.
50. AZURE-50: Cosmos DB change feed.
51. AZURE-51: Change feed latest version mode.
52. AZURE-52: Change feed all versions and deletes mode.
53. AZURE-53: Change feed for cache/search/warehouse/materialized-view updates.
54. AZURE-54: Change feed event sourcing pattern.
55. AZURE-55: Cosmos DB continuous backup and PITR.
56. AZURE-56: Cosmos DB autoscale throughput.
57. AZURE-57: Cosmos DB serverless throughput.
58. AZURE-58: Cosmos DB RU-based cost governance.
59. AZURE-59: Cosmos DB lifetime free tier: 1000 RU/s and 25 GB.
60. AZURE-60: Cosmos DB emulator for local development.

## §4 UNION-coverage matrix
| capability | AWS has | Spanner has | Azure has | UNION required | Oyatie cloud-data has | gap classification |
| --- | --- | --- | --- | --- | --- | --- |
| Managed relational SQL | yes | yes | yes | yes | concept yes via CRDB/Postgres | present-concept |
| MySQL compatibility | yes | no | no | yes | no local claim | missing |
| PostgreSQL compatibility | yes | yes | partial via SQL Server not PG | yes | concept yes | present-concept |
| SQL Server compatibility | no | no | yes | yes | no | missing |
| Distributed SQL | partial | yes | partial | yes | concept yes | present-concept |
| Key-value model | no | yes | yes | yes | partial via tenant KV/doc examples | partial |
| Document model | no | no | yes | yes | partial via Mongo-compatible/FerretDB | partial |
| Graph model | no | yes | yes | yes | concept yes via Neo4j | partial |
| Vector search | no | yes | yes | yes | no local contract | missing |
| Full-text search | no | yes | yes via Azure SQL/Search ecosystem | yes | external PRD assigns search | missing-local |
| Managed cache | no | no | partial external | yes due product union | concept Valkey deployment component | partial |
| Managed Kafka/topic | no | no | no | additive because PRD assigns | external PRD only | missing-local |
| Queue service | no | no | no | additive because PRD assigns | external PRD only | missing-local |
| Topic/pubsub service | no | no | no | additive because PRD assigns | external PRD only | missing-local |
| Analytics columnar path | no | yes | yes | yes | concept ClickHouse | partial |
| HTAP/no-ETL analytics | no | yes | yes | yes | concept CDC to ClickHouse | partial |
| Change streams/feed | no | yes | yes | yes | concept CDC | partial |
| Event sourcing | no | no | yes Cosmos change feed | yes | concept TigerBeetle/audit-chain | partial |
| Ledger/tamper evidence | no | no | yes Azure SQL ledger | yes | concept TigerBeetle/audit-chain | partial |
| PITR | yes | yes | yes | yes | concept yes | present-concept |
| Backup retention policy | yes | yes | yes | yes | concept tenant_class retention | partial |
| Long-term retention | snapshots | one-year backup max | ten-year LTR | yes | paid tenant_class 7y archive claim | partial |
| Database cloning | yes | no | no | yes | no | missing |
| Backtrack/rewind | yes Aurora MySQL | PITR restore | PITR restore | yes | PITR restore only | partial |
| Multi-AZ/zone HA | yes | yes | yes | yes | concept multi-AZ/multi-cell | partial |
| Multi-region reads | yes | yes | yes | yes | concept cross-region replicas | partial |
| Multi-region writes | primary+forwarding | yes via Spanner configs | yes Cosmos | yes | no clear write topology | partial |
| Region switchover | yes | yes operationally | yes | yes | no runbook | missing-local |
| Region failover | yes | yes | yes | yes | external PRD assigns failover | missing-local |
| RTO/RPO contract | partial | yes-ish via SLA/features | yes consistency/RPO docs | yes | not local | missing |
| Read replicas | yes | yes | yes | yes | concept read replicas | partial |
| Connection pooling/proxy | yes RDS Proxy | no direct counterpart | no direct counterpart | yes | no | missing |
| IAM database auth | yes | IAM | Azure AD/Entra integration | yes | Cedar concept | partial |
| Secret-manager credential integration | yes | GCP IAM/secrets adjacent | Azure Key Vault adjacent | yes | no local handoff | missing-local |
| TLS enforcement | yes | yes | yes | yes | concept HSM TLS | partial |
| Key encryption | yes KMS | yes CMEK | yes | yes | cloud-kms concept | partial |
| Customer-managed keys | yes | yes | yes | yes | concept CMK envelope | partial |
| Private networking | yes VPC | yes VPC SC/private service | yes VNet/private endpoints | yes | no local IaC | missing-local |
| Autoscale compute | yes Serverless v2 | yes managed autoscaler | yes serverless/autoscale | yes | no local control plane | missing |
| Autoscale storage | yes | yes by nodes/storage limits | yes Cosmos storage | yes | concept unbounded paid tenant_class | partial |
| Serverless tier | yes | trial/min nodes not full serverless | yes SQL/Cosmos | yes | no | missing |
| Free tier | AWS free tier not Aurora always | Spanner free trial | SQL/Cosmos free tier | yes | OCI Always Free missing | missing |
| Cost governance | yes storage choice | CUD/editions | RU/vCore/serverless | yes | benchmark TCO only | partial |
| Billing tags | AWS tags | labels | tags | yes | absent | missing |
| Edition/tenant_class feature gates | yes | yes | yes | yes | concept demo_trial/paid tenant_class | partial |
| API contract | yes SDK/API | yes API | yes SDK/API | yes | no local contract | missing |
| CLI/admin API | yes | yes | yes | yes | examples only | partial |
| Observability metrics | CloudWatch/PI | Cloud Monitoring | Azure Monitor | yes | one ADR metric external | partial |
| Performance Insights/advisor | yes | Query plans/monitoring | Intelligent query processing | yes | no local equivalent | missing |
| Slow query/performance diagnostics | yes | yes | yes | yes | no | missing |
| Backup scheduling | yes | yes | yes | yes | concept only | partial |
| Backup copy/export | yes | export/import | Azure backup storage | yes | cloud-storage concept | partial |
| Data export | yes | yes | yes | yes | product PRD assignment | missing-local |
| Data deletion lifecycle | yes | yes | yes | yes | product PRD assignment | missing-local |
| Legal hold | no direct DB core | no direct DB core | storage/compliance ecosystem | yes for Oyatie | product PRD assignment | missing-local |
| DSR cascade | no direct DB core | no direct DB core | privacy ecosystem | yes for Oyatie | product PRD assignment | missing-local |
| Geo-partitioning | no | yes | partitioning/global dist | yes | concept yes | partial |
| Tenant isolation | AWS account/DB isolation | IAM/database isolation | account/container/db isolation | yes | concept tenant_id prefix | partial |
| Physical tenancy modes | instance classes | instance configs | single/elastic/pool/account | yes | tenant_class context mentions shared/dedicated | partial |
| Dedicated single tenant | yes | yes | yes | yes | paid tenant_class concept | partial |
| Maintenance windows | yes | yes | yes | yes | product PRD assignment only | missing-local |
| Upgrade dry run | partial blue/green | edition migration no data migration | service upgrade docs | yes | product PRD assignment only | missing-local |
| Engine version policy | yes | yes | yes | yes | named versions only | partial |
| Parameter/config groups | yes | yes configs | yes DB configs | yes | no local config model | missing |
| Schema migration | yes tools | DDL operations | SQL migrations | yes | examples only | partial |
| Schema object limits | yes quotas | yes quotas | yes quotas | yes | no local quotas | missing |
| Storage limits | yes | yes | yes | yes | tenant_class caps only | partial |
| Throughput limits | yes | yes | yes | yes | tenant_class targets only | partial |
| p95/p99 latency targets | yes via docs/SLA | performance docs | Cosmos p99 SLA | yes | tenant_class/prose targets | partial |
| SLA document | yes | yes | yes | yes | no OpenSLO | missing |
| Incident failover drill | yes docs | yes operations | yes failover APIs | yes | no runbook | missing |
| Data residency | regions/global | geo-partitioning | regions/sovereign clouds | yes | concept YAML/Cedar | partial |
| Sovereign cloud support | Gov/China regions | Google regions/sovereign offerings | Azure clouds | yes | no context modules | missing-local |
| Compliance packs | yes AWS compliance | yes Google compliance | yes Microsoft compliance | yes | tenant_class prose only | partial |
| Audit logging | CloudTrail/activity streams | audit/monitoring | Azure Monitor | yes | audit-chain concept | partial |
| Tamper-evident audit | partial QLDB adjacent | no core | Azure SQL ledger | yes | audit-chain/TigerBeetle concept | partial |
| Local emulator | no Aurora local | emulator not core | Cosmos emulator | yes | no | missing |
| Migration playbooks | DMS/SCT | Dataflow/Datastream | Database Migration Service | yes | Aurora/Dynamo only | partial |
| Vendor-to-Oyatie migration | no | no | no | additive | one playbook | partial |
| OpenTofu deployability | external IaC | external IaC | external IaC | Oyatie required | absent | missing |
| Six-context deployment | no | no | no | Oyatie required | absent | missing |
| OS support manifest | no | no | no | Oyatie required | absent | missing |
| Rust backend implementation | no | no | no | Oyatie required | absent | missing |
| OCI Always Free demo_trial tenant_class | no | no | no | Oyatie required | absent | missing |
| Cedar per-tenant policy | no | no | no | Oyatie additive | concept yes | partial |
| Cloud-kms handoff | yes KMS native | yes KMS native | yes Key Vault native | yes | prose only | missing-local |
| Cloud-storage handoff | yes S3 native | GCS export | Blob/Synapse | yes | prose only | missing-local |
| Audit-chain handoff | no | no | partial ledger | additive | prose only | missing-local |

## §5 Capability family summary table
| family | UNION required count | Oyatie present concept count | Oyatie executable/local count | headline status |
| --- | ---: | ---: | ---: | --- |
| Relational SQL | 6 | 4 | 0 | concept only |
| NoSQL/document/key-value | 6 | 3 | 0 | partial concept |
| Graph/search/vector | 6 | 2 | 0 | mostly missing locally |
| Analytics/HTAP | 5 | 3 | 0 | concept only |
| Change streams/eventing | 5 | 2 | 0 | partial concept |
| Ledger/audit | 5 | 3 | 0 | partial concept |
| Backup/PITR/restore | 8 | 5 | 0 | concept only |
| HA/failover/replication | 10 | 5 | 0 | no runbook/IaC |
| Tenant/IAM/security | 9 | 5 | 0 | policy missing |
| Networking/private access | 5 | 1 | 0 | IaC missing |
| Autoscale/serverless/cost | 8 | 2 | 0 | mostly missing |
| Deployment contexts/IaC | 7 | 0 | 0 | missing |
| OS/runtime support | 4 | 0 | 0 | missing |
| API/SDK/contracts | 7 | 2 | 0 | examples only |
| Observability/SLOs | 6 | 2 | 0 | no OpenSLO |
| Compliance/residency/legal | 8 | 4 | 0 | prose only |
| Migration | 6 | 2 | 0 | Aurora/Dynamo only |

## §6 Headline gap analysis - top 15 missing capabilities
1. Gap 01 - Executable API contracts: FAQ promises SDK plus REST/gRPC, but no contracts exist. Hook: add OpenAPI 3.2, AsyncAPI 3.1, and protobuf contracts under `contracts/`.
2. Gap 02 - OpenTofu deployability: no `iac/` directory for any context. Hook: add six context modules with shared cloud-data variables and outputs.
3. Gap 03 - OCI Always Free demo_trial tenant_class: demo_trial tenant_class is priced at about $25/month. Hook: add `iac/oci-guest/always-free/` and a demo_trial tenant_class-on-OCI resource envelope.
4. Gap 04 - OS support matrix: no Tier-1/Tier-2/exclusion/package manifest. Hook: add `supported-oses.json`.
5. Gap 05 - Rust implementation: only Markdown Rust sample exists. Hook: add a real Rust crate and hermetic simulator tests.
6. Gap 06 - Vector/search parity: Azure Cosmos and Spanner include vector/search capabilities; local cloud-data has only external PRD assignments. Hook: add vector/search APIs and tenant_class/deployment placement.
7. Gap 07 - Change-feed/change-stream contract: Spanner/Cosmos define records, retention, consumers, and limitations. Hook: add CDC schema, retention, replay, and audit semantics.
8. Gap 08 - Multi-region failover runbook: Aurora/Spanner/Cosmos/Azure SQL publish failover and availability models. Hook: add RTO/RPO, switchover, and failover tests.
9. Gap 09 - Connection proxy/pooling: RDS Proxy is a major union capability. Hook: add cloud-data data-plane proxy, connection caps, load shedding, and Cedar-aware session policy.
10. Gap 10 - Cost governance: counterparts expose vCore/RU/serverless/storage choices. Hook: add cost-budget and billing-tag outputs per context/tenant_class.
11. Gap 11 - Benchmark provenance: benchmark doc claims measured numbers without evidence. Hook: add raw result manifest, workload definition, OS/arch/context/tenant class, and reproduction command.
12. Gap 12 - Compliance/DPIA: tenant_class model claims regulated packs without service evidence. Hook: add compliance.md and dpia.md with data classes and residency flows.
13. Gap 13 - Legal hold/DSR: cloud product PRD assigns these to cloud-data but local docs omit model. Hook: add retention lock, legal hold, DSR cascade schema and tests.
14. Gap 14 - Maintenance/upgrade dry-run: product PRD assigns these to cloud-data but local docs omit plan. Hook: add upgrade state machine, backup checkpoint, and rollback plan.
15. Gap 15 - Counterpart migration coverage: only Aurora/DynamoDB migration exists. Hook: add Spanner, Azure SQL, Cosmos DB, Firestore, and Cloud SQL playbooks.

## §7 Additive surface - Oyatie capabilities not directly in any one counterpart
1. Additive 01: Cedar-mediated tenant policy boundary for every database action. Rationale: if implemented, this unifies IAM and data-plane authorization.
2. Additive 02: `tenant_id` primary-key prefix as platform invariant. Rationale: explicit tenant data layout is stronger than generic account separation.
3. Additive 03: audit-chain-backed PITR replay events. Rationale: restores become attestable platform events.
4. Additive 04: TigerBeetle ledger engine in the same managed data substrate. Rationale: ledger-specific double-entry safety is not part of Aurora/Spanner/Cosmos union core.
5. Additive 05: cloud product legal hold ownership. Rationale: legal hold is broader than database backup retention.
6. Additive 06: DSR cascade ownership. Rationale: privacy workflow orchestration exceeds database service alone.
7. Additive 07: OCI Always Free demo_trial tenant_class doctrine. Rationale: counterpart free tiers differ, but Oyatie requires OCI-specific free deployment maximization.
8. Additive 08: six-context deployability. Rationale: public cloud, guest clouds, on-prem, colo, and Oyatie-as-provider are broader than any single vendor.
9. Additive 09: Rust-strict backend doctrine. Rationale: this is a platform implementation constraint, not counterpart feature parity.
10. Additive 10: OpenTofu-only IaC doctrine. Rationale: provider-neutral deployment is canonical Oyatie behavior.
11. Additive 11: multi-engine deployment ladder from CRDB to Yugabyte/ClickHouse/TigerBeetle/Neo4j. Rationale: no one counterpart packages the same exact engine union.
12. Additive 12: service-owned cost and evidence wiring to cloud-billing. Rationale: counterpart billing exists, but Oyatie requires service-local outputs and cost events.
13. Additive 13: sovereign-cell policy through Cedar and data residency declarations. Rationale: cloud-data can become policy-native rather than region-selection-only.
14. Additive 14: cross-service handoff with cloud-kms/cloud-storage/cloud-iam/audit-chain. Rationale: Oyatie can expose internal trust boundaries explicitly.
15. Additive 15: Foundry-as-tenant testing. Rationale: the platform can dogfood cloud-data before customer workloads.
16. Additive 16: data substrate as a replacement for teams creating their own RDS/Spanner/Cosmos/Dynamo instances. Rationale: this reduces product-team data sprawl.
17. Additive 17: HLC default with TrueTime permit gating. Rationale: explicit time-source cost/control can be a platform differentiator.
18. Additive 18: service-local OS matrix and package contracts. Rationale: vendor PaaS hides OS, while Oyatie must support hybrid and self-hosted contexts.
19. Additive 19: OpenSLO-based data service contracts. Rationale: if added, service-level SLOs become machine-auditable.
20. Additive 20: signed OpenTofu modules. Rationale: IaC supply-chain trust is part of the service readiness bar.

## §8 Implementation hook backlog for parity closure
1. Hook 01: create `PRD.md` that restates cloud-data purpose, counterpart scope, tenant classes, and explicit non-goals.
2. Hook 02: create `ARCHITECTURE.md` with control plane, data plane, engine adapters, consistency layer, and cross-service dependencies.
3. Hook 03: create `contracts/cloud-data-admin.openapi.yaml` for provisioning, maintenance, upgrade, restore, export, delete, legal hold, and DSR operations.
4. Hook 04: create `contracts/cloud-data-tenant.proto` for tenant data-plane calls if gRPC remains in scope.
5. Hook 05: create `contracts/cloud-data-events.asyncapi.yaml` for CDC, backup, restore, failover, audit-chain, and billing events.
6. Hook 06: create `contracts/cloud-data-vector.openapi.yaml` if search/vector remains assigned to cloud-data.
7. Hook 07: create `contracts/cloud-data-queue-topic.asyncapi.yaml` if queue/topic remains assigned to cloud-data.
8. Hook 08: create `policies/cloud-data.cedar` for tenant, operator, break-glass, restore, replica promotion, and export decisions.
9. Hook 09: create `slos/cloud-data.openslo.yaml` with tenant_class-scoped latency, durability, availability, CDC lag, and failover objectives.
10. Hook 10: create `capacity-model.md` that maps demo_trial/paid tenant_class to CPU, memory, disk, IOPS, network, and storage growth.
11. Hook 11: create `failure-modes.md` for engine crash, quorum loss, stale replicas, backup corruption, key denial, policy denial, and region outage.
12. Hook 12: create `incident-response.md` for restore, failover, tenant isolation, data corruption, legal hold conflict, and CDC lag incidents.
13. Hook 13: create `cost-budget.md` with tenant_class budgets, context overlays, cloud-billing events, and OCI Always Free zero-paid-spend proof.
14. Hook 14: create `dpia.md` because cloud-data processes tenant operational data, DSR cascades, and legal holds.
15. Hook 15: create `compliance.md` with HIPAA, PCI, CJIS, ITAR, KR residency, FINMA, data class, and evidence owner mapping.
16. Hook 16: create `cross-microservice-handoffs.md` for cloud-kms, cloud-storage, cloud-iam, audit-chain, cloud-billing, cloud-iac, observability, foundry, and compliance.
17. Hook 17: create `supported-oses.json` with Tier-1, Tier-2, exclusions, architectures, package formats, and CI lane policy.
18. Hook 18: create `iac/oyatie-public-cloud/` OpenTofu module with service endpoint, resource IDs, KMS refs, and observability bindings.
19. Hook 19: create `iac/guest-on-aws/` OpenTofu module without direct AWS business-logic coupling.
20. Hook 20: create `iac/oci-guest/` OpenTofu module for paid OCI paid tenant_class posture.
21. Hook 21: create `iac/oci-guest/always-free/` OpenTofu module for demo_trial tenant_class zero-paid-spend posture.
22. Hook 22: create `iac/on-prem/` OpenTofu module for operator-owned hardware and local key custody.
23. Hook 23: create `iac/colo/` OpenTofu module for rack-local storage, WAN, and offsite backup.
24. Hook 24: create `iac/oyatie-iaas/` OpenTofu module for Oyatie-as-cloud-provider external customers.
25. Hook 25: add sigstore/cosign signature artifacts under each module's `module-signatures/`.
26. Hook 26: add remote state backend definitions per context; do not use local-disk state.
27. Hook 27: add `src/` Rust crate with admin service, data-plane proxy, engine adapters, and policy evaluation.
28. Hook 28: add `tests/` for direct database access denial, tenant isolation, PITR restore, CDC, failover, and tenant_class throttling.
29. Hook 29: add a hermetic simulator matching the reference implementation's `cargo test --features hermetic` claim.
30. Hook 30: replace `make dev-cell.up` examples with Oya/Cargo/OpenTofu commands that state underlying canonical invocation.
31. Hook 31: add benchmark workloads for point read, single write, transaction, CDC, PITR, failover, vector search, ledger post, and OLAP freshness.
32. Hook 32: add benchmark metadata fields for OS, arch, context, tenant class, engine, hardware, and dataset.
33. Hook 33: add migration playbook from Google Spanner, including schema, interleaved tables, change streams, and cutover.
34. Hook 34: add migration playbook from Azure SQL Database, including T-SQL, elastic pool, Hyperscale, and ledger cases.
35. Hook 35: add migration playbook from Azure Cosmos DB, including partition keys, consistency, RU budget, change feed, and TTL.
36. Hook 36: add migration playbook from Cloud SQL and Firestore if FAQ displacement remains broad.
37. Hook 37: add runbook for Aurora Global Database-like region switchover.
38. Hook 38: add runbook for Spanner-like geo-partition relocation.
39. Hook 39: add runbook for Cosmos-like multi-region write conflict handling.
40. Hook 40: add vector/search schema before claiming parity with Spanner Enterprise or Cosmos vector.
41. Hook 41: add ledger digest/attestation model before claiming parity with Azure SQL ledger.
42. Hook 42: add connection proxy/pooler model before claiming RDS Proxy parity.
43. Hook 43: add IAM/Cedar/database principal mapping before claiming IAM auth parity.
44. Hook 44: add change-feed value-capture modes and delete semantics before claiming Cosmos/Spanner CDC parity.
45. Hook 45: add storage quota and schema-object quota model before claiming Spanner/Azure/Aurora scale parity.
46. Hook 46: add edition/tenant_class feature gate manifest so demo_trial/paid tenant_class can be machine-checked.
47. Hook 47: add tenant lifecycle states for create, active, suspended, restore, export, legal hold, delete pending, deleted, and archived.
48. Hook 48: add data residency policy schema with home cell, allowed replicas, denied jurisdictions, and audit evidence.
49. Hook 49: add cloud-billing cost event schema for provision, resize, backup growth, restore, failover, and archive.
50. Hook 50: add observability dashboards or metric catalogs for replication lag, CDC lag, restore time, lock waits, p95/p99 latency, and cost burn.
51. Hook 51: add failure injection tests for writer crash, replica stale read, KMS denial, policy denial, and backup checksum failure.
52. Hook 52: add docs tying legal hold and DSR cascade to retention lock and backup redaction behavior.
53. Hook 53: add service-level ADR for whether Kafka/queue/topic belong in cloud-data or separate messaging service.
54. Hook 54: add service-level ADR for whether vector/search belong in cloud-data or separate search/vector service.
55. Hook 55: add service-level ADR for TrueTime/time-master owner and interface.
56. Hook 56: add service-level ADR for TigerBeetle ledger ownership and data recovery model.
57. Hook 57: add service-level ADR for graph/Cedar entity-store ownership with cloud-iam.
58. Hook 58: add product claim guardrail that target numbers cannot be used as measured results.
59. Hook 59: add Wave 14 aggregation checklist mapping every counterpart capability row to present/partial/missing evidence.
60. Hook 60: add reviewer checklist requiring file:line citations for every parity claim.
