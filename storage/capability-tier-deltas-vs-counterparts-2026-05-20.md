# cloud-storage tenant_class deltas vs counterparts - 2026-05-20

Audit owner: single-agent cloud-storage lane.
Scope: demo_trial, paid baseline profile, paid production profile, and paid regulated profile compared against AWS S3, Google Cloud Storage, and Azure Blob Storage.
Verdict style: ahead / parity / catch-up / missing / not-applicable.

## Anchor block

1. Canonical sequence: `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-2214`, `:2241-2494`, `:3441-3754`, and `:3756-4146`.
2. Machine-readable direction: `specs/master-plan-sequencing.json:704-867`.
3. Service PRD citation: service-local `PRD.md` is absent; substitute purpose evidence is `docs/decisions/ADR-0702-identity-authz-live-apex.md:8-11` and root cloud PRD `docs/products/cloud/PRD.md:1587-1665`.
4. Service architecture citation: service-local `ARCHITECTURE.md` is absent; substitute architecture evidence is `storage/faqs/storage-engineer-faq.md:15-22` and root crate map `docs/products/cloud/PRD.md:131-137`.
5. Documentation-rigor citation: `docs/standards/documentation-rigor.md:62-81` and `docs/standards/documentation-rigor.md:133-156`.

## §1 tenant_class Profile Definitions in Oyatie

demo_trial source: `docs/decisions/ADR-0702-identity-authz-live-apex.md:13-32`.
demo_trial current tenant: B2C/community tenant.
demo_trial current backend: shared regional MinIO cluster.
demo_trial current storage classes: Hot only.
demo_trial current buckets: 5 buckets per tenant.
demo_trial current capacity: 50GB per tenant in local doc.
demo_trial current versioning: off.
demo_trial current object lock: off.
demo_trial current replication: none.
demo_trial current encryption: SSE-OYA.
demo_trial current API compatibility: S3 basic GET/PUT/LIST/DELETE.
demo_trial current throughput: 30 MB/s read, 60 MB/s write.
demo_trial current latency: p95 GET 28 ms, p95 PUT 65 ms.
demo_trial current price: about $15/mo substrate cost.
demo_trial canonical correction: OCI guest demo_trial must map to OCI Always Free and cannot assume $15/mo.
demo_trial canonical OCI ceiling: 10GB object storage and 10GB archive storage.
demo_trial canonical block ceiling: 200GB block volume pool.
demo_trial target posture: developer/community class with explicit free-tier guardrails.

paid baseline profile source: `docs/decisions/ADR-0702-identity-authz-live-apex.md:34-53`.
paid baseline profile current tenant: SMB/workgroup.
paid baseline profile current backend: dedicated MinIO pool plus `aws-s3-glacier-deep` adapter.
paid baseline profile current storage classes: Hot, Warm, Cold.
paid baseline profile current buckets: 50 buckets per tenant.
paid baseline profile current capacity: 10TB.
paid baseline profile current versioning: on.
paid baseline profile current object lock: off.
paid baseline profile current replication: async one-region replica.
paid baseline profile current encryption: SSE-KMS with cloud-kms KEK.
paid baseline profile current API compatibility: multipart, copy, presign, lifecycle, policy, CORS, SSE-KMS.
paid baseline profile current throughput: 250 MB/s read, 500 MB/s write.
paid baseline profile current latency: p95 GET 14 ms, p95 PUT 32 ms.
paid baseline profile current price: about $420/mo.
paid baseline profile canonical correction: AWS Glacier adapter must become optional backing adapter, not tier baseline.
paid baseline profile target posture: paid baseline with versioning, lifecycle, one async replica, and daily inventory.

paid production profile source: `docs/decisions/ADR-0702-identity-authz-live-apex.md:55-74`.
paid production profile current tenant: mid-market.
paid production profile current backend: dedicated EC:14+4 cluster plus SSD metadata and edge cache.
paid production profile current storage classes: Hot, Warm, Cold, Archive.
paid production profile current buckets: 5000 buckets per tenant.
paid production profile current capacity: 1PB.
paid production profile current versioning: on plus legal hold.
paid production profile current object lock: WORM governance and compliance.
paid production profile current replication: bidirectional active-active in local tenant_class policy.
paid production profile current encryption: per-bucket DEK, FIPS KMS, optional HSM.
paid production profile current API compatibility: full S3 plus Azure Blob and GCS-like signed URL support.
paid production profile current throughput: 2.5 GB/s read, 5 GB/s write.
paid production profile current latency: p95 GET <= 6 ms hot, <= 2 ms edge; p95 PUT <= 16 ms.
paid production profile current price: about $2200/mo.
paid production profile contradiction: FAQ and benchmark describe async replication, while migration says sync paid production profile.
paid production profile target posture: production parity with S3/GCS/Azure for object storage, plus Cedar/audit additions.

paid regulated profile source: `docs/decisions/ADR-0702-identity-authz-live-apex.md:76-95`.
paid regulated profile current tenant: regulated / large enterprise.
paid regulated profile current backend: dedicated per-tenant per-region clusters plus HSM and optional tape escrow.
paid regulated profile current storage classes: Hot, Warm, Cold, Archive, LegalHold, DeepTape.
paid regulated profile current buckets: unlimited with quota policy.
paid regulated profile current capacity: unlimited by contract.
paid regulated profile current versioning: mandatory.
paid regulated profile current object lock: legal hold plus retention lock.
paid regulated profile current replication: sync multi-region Raft group.
paid regulated profile current encryption: FIPS 140-3 Level 3 HSM plus PQC-wrapped DEK escrow.
paid regulated profile current API compatibility: full S3, Azure Blob, GCS JSON/XML, B2, R2.
paid regulated profile current throughput: 100 GB/s aggregate.
paid regulated profile current latency: p95 GET <= 2 ms hot, <= 0.6 ms edge; p95 PUT <= 8 ms local quorum.
paid regulated profile current price: custom committed hardware.
paid regulated profile target posture: hyperscaler bar and single-tenant-capable.

## §2 Counterpart Storage-Class Mapping

AWS free-equivalent tier: AWS Free Tier S3 trial/allocation, not a storage-class product tier.
AWS baseline tier: S3 Standard.
AWS performance tier: S3 Express One Zone.
AWS infrequent-access tier: S3 Standard-IA and One Zone-IA.
AWS archive tier: Glacier Instant Retrieval, Glacier Flexible Retrieval, and Glacier Deep Archive.
AWS enterprise axes: replication, object lock, Storage Lens, access points, KMS, private networking, organization controls, and Outposts.
AWS primary tier axis: access frequency, latency, replication topology, and compliance controls.

GCS free-equivalent tier: Google Cloud free program/trial, not a storage-class product tier.
GCS baseline tier: Standard storage.
GCS nearline tier: Nearline.
GCS coldline tier: Coldline.
GCS archive tier: Archive.
GCS performance tier: Rapid Bucket and zonal buckets.
GCS enterprise axes: turbo replication, hierarchical namespace, Storage Insights, IAM, retention locks, soft delete, bucket IP filtering, and FUSE.
GCS primary tier axis: access frequency, location scope, replication RPO, and namespace model.

Azure free-equivalent tier: Azure free account allocation, not a storage-class product tier.
Azure baseline tier: Hot.
Azure cool tier: Cool.
Azure cold tier: Cold.
Azure archive tier: Archive.
Azure enterprise axes: Data Lake Gen2 hierarchical namespace, immutable storage, object replication, private endpoints, SFTP, NFS 3.0, change feed, and inventory.
Azure primary tier axis: access frequency, online/offline access, network/privacy integration, and data-lake namespace.

Oyatie demo_trial equivalent: free/community/dev baseline, closest to cloud free-tier plus hot-only minimal object store.
Oyatie paid baseline profile equivalent: paid standard production baseline, closest to S3 Standard plus Standard-IA/Glacier entry, GCS Standard/Nearline, Azure Hot/Cool.
Oyatie paid production profile equivalent: business production object service, closest to S3 Standard + RTC/Object Lock/Storage Lens, GCS dual/multi-region/turbo, Azure Hot/Cool/Cold + object replication.
Oyatie paid regulated profile equivalent: dedicated regulated storage plane, closest to hyperscaler enterprise options plus custom single-tenant/dedicated deployments.

## §3 Per-Oyatie tenant_class Delta Tables

### demo_trial tenant_class Table

| Feature | Oyatie demo_trial | AWS equivalent | GCS equivalent | Azure equivalent | Gap classification |
|---|---|---|---|---|---|
| Free-tier fit | missing for OCI; $15/mo local | AWS Free Tier limited | Google free program limited | Azure free account limited | missing canonical OCI free |
| Object storage capacity | 50GB local, 10GB OCI target | free allocation lower, paid scalable | free allocation lower, paid scalable | free allocation lower, paid scalable | partial |
| Archive capacity | absent local, 10GB OCI target | Glacier paid | Archive paid | Archive paid | missing local |
| Block storage capacity | absent local, 200GB OCI target | EBS separate | Persistent Disk separate | Managed Disks separate | missing service split |
| Bucket count | 5 | scalable | scalable | containers scalable | catch-up |
| Object put/get | yes basic | yes | yes | yes | parity basic |
| List/delete | yes basic | yes | yes | yes | parity basic |
| Versioning | off | available | available | available | catch-up |
| Object lock | off | available | retention available | immutable available | catch-up |
| Soft delete | absent | partial via versioning | yes | yes | missing |
| Lifecycle | absent or minimal | yes | yes | yes | catch-up |
| Replication | none | yes | yes | yes | catch-up |
| Storage classes | Hot only | many | many | many | catch-up |
| Presigned URLs | not listed demo_trial | yes | signed URLs | SAS | partial |
| Multipart upload | not listed demo_trial | yes | resumable/multipart | block list | partial |
| KMS | SSE-OYA | SSE-S3/SSE-KMS | Google-managed/CMEK | Microsoft-managed/CMK | partial |
| Customer keys | absent | SSE-C/client | CSEK | CPK | missing |
| Public access block | absent | yes | yes prevention | network/public controls | missing |
| Private endpoint | absent | VPC endpoint | no direct equivalent | private endpoint | missing |
| Events | absent | EventBridge/SNS/SQS/Lambda | Pub/Sub | Event Grid | missing |
| Inventory | absent | yes | yes | yes | missing |
| Batch operations | absent | yes | yes | partial | missing |
| Analytics | absent | Storage Lens | Storage Insights | Azure Monitor | missing |
| Static website | absent | yes | yes | yes | missing |
| SFTP/NFS/FUSE | absent | Transfer Family/adjacent | FUSE | SFTP/NFS | missing |
| Durability | 99.99 local FAQ | 11 nines common | 11 nines | 11 nines via redundancy | catch-up |
| Availability | unstated file | 99.99 Standard | class dependent | account/SLA dependent | missing SLO |
| Request rate | 500/250 target | 5500/3500 per prefix | 5000/1000 initial | 20k/40k account | catch-up |
| OpenTofu module | absent | not applicable | not applicable | not applicable | P1 Oyatie gap |
| OS matrix | absent | not applicable | not applicable | not applicable | P2 Oyatie gap |
| Cedar policy | concept only | IAM | IAM | RBAC/ACL/SAS | additive partial |
| Audit-chain | absent schema | CloudTrail | Audit Logs | Azure Monitor/diagnostics | additive missing |
| OCI Always Free | required but absent | no | no | no | additive missing |

demo_trial verdict: catch-up.
demo_trial main blocker: OCI Always Free reconciliation is absent.
demo_trial second blocker: demo_trial lacks counterpart baseline protection controls such as versioning, soft delete, lifecycle, events, and inventory.
demo_trial ahead axis: none currently evidenced.
demo_trial safe claim: basic hot object store only.

### paid baseline profile Table

| Feature | Oyatie paid baseline profile | AWS equivalent | GCS equivalent | Azure equivalent | Gap classification |
|---|---|---|---|---|---|
| Paid standard baseline | yes | S3 Standard | GCS Standard | Azure Hot | parity concept |
| Capacity | 10TB | scalable | scalable | scalable | catch-up |
| Bucket count | 50 | scalable | scalable | scalable containers | catch-up |
| Versioning | on | yes | yes | yes | parity |
| Object lock | off | yes | retention lock | immutable | catch-up |
| Soft delete | absent | partial | yes | yes | missing |
| Lifecycle | yes | yes | yes | yes | parity concept |
| Lifecycle cadence | 6h FAQ | configurable | configurable | configurable | partial |
| Storage classes | Hot/Warm/Cold | Standard/IA/Glacier | Standard/Nearline/Coldline | Hot/Cool/Cold | parity concept |
| Archive/deep archive | via AWS adapter | Glacier Deep Archive | Archive | Archive | partial, adapter drift |
| Replication | one async replica | SRR/CRR | cross-bucket/dual | object replication | partial |
| Replication RPO | not formal | RTC available | turbo/default RPO | async rules | missing SLO |
| KMS | cloud-kms KEK | KMS | CMEK | CMK | parity concept |
| Customer supplied keys | absent | SSE-C | CSEK | CPK | missing |
| Resource policy | Cedar + S3 policy translation | bucket policy/IAM | IAM/ACL | RBAC/SAS | additive partial |
| Public access controls | absent | block public access | prevention | network rules | missing |
| Private endpoint | absent | VPC endpoint | IP filtering | private endpoint | missing |
| Presigned URL | yes | yes | signed URLs | SAS | parity concept |
| CORS | yes | yes | yes | yes | parity concept |
| Multipart/copy | yes | yes | yes | yes | parity concept |
| Inventory | absent in tier | yes | yes | yes | missing |
| Batch operations | absent | yes | yes | partial | missing |
| Analytics | absent | Storage Lens | Insights | Monitor | missing |
| Events | absent | yes | Pub/Sub | Event Grid | missing |
| Static website | absent | yes | yes | yes | missing |
| FUSE/NFS/SFTP | absent | adjacent | FUSE | SFTP/NFS | missing |
| Data lake namespace | absent | no direct | HNS folders | ADLS Gen2 | missing |
| Read throughput | 250 MB/s | scalable | scalable | 50-200 Gbps account egress | catch-up |
| Write throughput | 500 MB/s | scalable | scalable | 25-60 Gbps ingress | catch-up |
| Latency | p95 GET 14ms, PUT 32ms | product dependent | product dependent | product dependent | plausible target |
| Durability | 99.9999999 | 11 nines common | 11 nines | redundancy dependent | catch-up |
| Availability | not formal | 99.99 Standard | class/SLA dependent | SLA dependent | missing SLO |
| OpenTofu | absent | not applicable | not applicable | not applicable | P1 Oyatie gap |
| OS matrix | absent | not applicable | not applicable | not applicable | P2 Oyatie gap |

paid baseline profile verdict: partial catch-up.
paid baseline profile main blocker: AWS Glacier adapter in baseline violates provider-neutral tiering unless documented as one adapter choice.
paid baseline profile second blocker: object-lock, soft-delete, events, inventory, private networking, and analytics are absent.
paid baseline profile ahead axis: Cedar resource policy could become ahead if formalized.
paid baseline profile safe claim: paid object store with versioning, lifecycle, KMS, and one async replica.

### paid production profile Table

| Feature | Oyatie paid production profile | AWS equivalent | GCS equivalent | Azure equivalent | Gap classification |
|---|---|---|---|---|---|
| Enterprise production tier | yes | S3 Standard + enterprise features | dual/multi-region + turbo | Hot/Cool/Cold + enterprise controls | partial |
| Capacity | 1PB | scalable | scalable | 5PiB account default | parity planning |
| Bucket count | 5000 | scalable | scalable | containers scalable | catch-up |
| Object size | 5TB | 5TB | large objects | large blobs | parity |
| Versioning | yes + legal hold | yes | yes | yes | parity |
| Object lock | governance/compliance | yes | retention lock | immutable | parity concept |
| Soft delete | absent | partial | yes | yes | missing |
| Lifecycle | yes | yes | yes | yes | parity concept |
| Lifecycle p95 | <=1h target | configurable | configurable | configurable | plausible |
| Storage classes | Hot/Warm/Cold/Archive | Standard/IA/Glacier | Standard/Nearline/Coldline/Archive | Hot/Cool/Cold/Archive | parity concept |
| Smart/autoclass | absent | Intelligent-Tiering | Autoclass | Smart tier | missing |
| Express/zonal tier | edge cache only | S3 Express One Zone | Rapid/zonal bucket | no direct | partial |
| Replication | active-active matrix; async FAQ | CRR/RTC/MRAP | dual/multi/turbo | object replication | contradictory |
| Replication filters | absent | yes | yes | yes prefix filters | missing |
| Replication RPO | 5s p95 target, not measured | 15min RTC | 15min turbo | async | ahead target, unproven |
| KMS/HSM | FIPS KMS, optional HSM | KMS/CloudHSM adjacent | Cloud KMS/HSM | Key Vault/HSM | parity concept |
| Customer supplied keys | absent | SSE-C | CSEK | CPK | missing |
| Resource policy | Cedar + S3 translation | IAM/bucket policy | IAM | RBAC/SAS | additive partial |
| Public access controls | absent | block public access | public access prevention | public/private network | missing |
| Private endpoint | absent | VPC endpoint | IP filtering | private endpoint | missing |
| Access points | absent | S3 Access Points/MRAP | managed folders IAM | private endpoints | missing |
| Events | absent | yes | Pub/Sub | Event Grid | missing |
| Inventory | present concept | yes | yes | yes | partial |
| Batch ops | absent | yes | yes | partial | missing |
| Analytics | absent | Storage Lens | Storage Insights | Monitor | missing |
| Query-in-place | absent | S3 Select | BigQuery integration | Synapse/Data Lake | missing |
| Data lake namespace | absent | no direct | HNS | ADLS Gen2 HNS | missing |
| SFTP/NFS/FUSE | absent | Transfer Family/adjacent | FUSE | SFTP/NFS | missing |
| Static website | absent | yes | yes | yes | missing |
| Transfer acceleration | absent | yes | global infra | CDN/transfer adjacent | missing |
| Offline import | absent | Snowball | Transfer Appliance | Data Box | missing |
| Read throughput | 2.5 GB/s | scalable | scalable | up to 50/200 Gbps egress account | catch-up |
| Write throughput | 5 GB/s | scalable | scalable | 25/60 Gbps ingress account | catch-up |
| Request rate | 20k/10k target | prefix scalable | bucket scalable | 20k/40k account | parity lower-bound |
| Durability | 11 nines | 11 nines | 11 nines | redundancy dependent | parity target |
| Availability | 99.99 target | 99.99 Standard | SLA dependent | SLA dependent | parity target |
| OpenTofu | absent | not applicable | not applicable | not applicable | P1 Oyatie gap |
| OS matrix | absent | not applicable | not applicable | not applicable | P2 Oyatie gap |

paid production profile verdict: feature-rich but not parity.
paid production profile main blocker: replication semantics conflict.
paid production profile second blocker: counterpart enterprise controls are missing.
paid production profile ahead axis: Cedar plus audit-chain could exceed provider logs after contracts land.
paid production profile safe claim: production object tier with strong target ambitions, not measured hyperscaler parity.

### paid regulated profile Table

| Feature | Oyatie paid regulated profile | AWS equivalent | GCS equivalent | Azure equivalent | Gap classification |
|---|---|---|---|---|---|
| Dedicated tenant storage | yes planned | Outposts/custom/private contracts | dedicated/sovereign options | dedicated/sovereign options | partial |
| Unlimited capacity | by contract | scalable | scalable | scalable/account quota | parity planning |
| Six storage classes | yes | many classes | many classes | many tiers | parity concept |
| Deep tape | yes planned | Glacier Deep Archive/Snow | Archive/Transfer | Archive/Data Box | partial |
| Mandatory versioning | yes | optional | optional | optional | ahead policy |
| Legal hold | yes | yes | yes | yes | parity |
| Retention lock | yes | yes | yes | yes | parity |
| Sync multi-region | yes planned | MRAP + replication not same | turbo RPO not sync writes | async object replication | ahead target, unproven |
| PQC-wrapped DEK escrow | yes planned | not standard | not standard | not standard | ahead target |
| FIPS 140-3 L3 HSM | yes planned | CloudHSM/KMS options | Cloud HSM/KMS options | Managed HSM | parity concept |
| Customer supplied keys | absent | yes | yes | yes | missing |
| Audit-chain proof | planned | CloudTrail | Audit Logs | Monitor | ahead target |
| Cedar policy | planned | IAM | IAM | RBAC/SAS | additive |
| Public access prevention | absent | yes | yes | yes | missing |
| Private endpoint | absent | yes | partial/IP filtering | yes | missing |
| Access points | absent | yes | partial | partial | missing |
| Batch operations | absent | yes | yes | partial | missing |
| Analytics | absent | Storage Lens | Insights | Monitor | missing |
| Inventory | yes concept | yes | yes | yes | partial |
| Change feed | absent | event/logs | Pub/Sub/logs | change feed | missing |
| Events | absent | event notifications | Pub/Sub | Event Grid | missing |
| HNS/data lake | absent | no direct | yes | yes | missing |
| SFTP | absent | Transfer Family | no direct | yes | missing |
| NFS/FUSE | absent | adjacent | FUSE | NFS 3.0 | missing |
| Static website | absent | yes | yes | yes | missing |
| Query-in-place | absent | S3 Select | BigQuery | Synapse/Data Lake | missing |
| Transfer acceleration | absent | yes | global | CDN/transfer | missing |
| Offline migration | absent | Snowball/Snowmobile | Transfer Appliance | Data Box | missing |
| Request target | 100k/50k | scalable prefixes | scalable buckets | 20k/40k account default | ahead target, unproven |
| Throughput target | 100 GB/s | scalable | scalable | 50/200 Gbps account | ahead target, unproven |
| Latency target | sub-2ms p95 GET | S3 Express low-latency | Rapid/zonal bucket | premium/local options | ahead target, unproven |
| Durability target | >11 nines planning | 11 nines | 11 nines | redundancy dependent | ahead target, unproven |
| Availability target | 99.995-99.999 | class/SLA dependent | class/SLA dependent | SLA dependent | ahead target, unproven |
| OpenTofu | absent | not applicable | not applicable | not applicable | P1 Oyatie gap |
| OS matrix | absent | not applicable | not applicable | not applicable | P2 Oyatie gap |

paid regulated profile verdict: aspirational ahead targets, current implementation evidence missing.
paid regulated profile main blocker: no architecture, SLO, IaC, benchmark evidence, or threat model supports the claims.
paid regulated profile second blocker: counterpart enterprise management controls are absent.
paid regulated profile ahead axis: mandatory versioning, sync Raft, PQC escrow, audit-chain, and Cedar are promising but unproved.
paid regulated profile safe claim: target profile only.

## §4 OCI demo_trial = Always Free Reconciliation

Canonical OCI source: `docs/decisions/ADR-0700-ci-admission-live-apex.md:3441-3466`.
Canonical OCI module source: `docs/decisions/ADR-0700-ci-admission-live-apex.md:3593-3655`.
Canonical OCI stop condition: `docs/decisions/ADR-0700-ci-admission-live-apex.md:3748-3754`.
OCI Always Free compute budget: up to 4 Arm OCPU and 24GB memory.
OCI Always Free block budget: 200GB total block volume.
OCI Always Free object budget: 10GB object storage.
OCI Always Free archive budget: 10GB archive storage.
OCI Always Free database budget: two Autonomous Databases of 20GB each, relevant only if metadata store chooses ADB.
OCI Always Free load balancer budget: 10Mbps load balancer.
OCI Always Free egress budget: 10TB/month outbound transfer under the canonical memory file.
demo_trial local mismatch: local tier says 50GB per tenant and $15/mo.
demo_trial reconciliation rule 1: guest-on-oci demo_trial must cap object data at 10GB.
demo_trial reconciliation rule 2: guest-on-oci demo_trial must cap archive data at 10GB.
demo_trial reconciliation rule 3: guest-on-oci demo_trial must cap block-storage dependency at 200GB aggregate.
demo_trial reconciliation rule 4: guest-on-oci demo_trial must not spill to AWS S3, GCS, Azure Blob, or paid OCI resources.
demo_trial reconciliation rule 5: guest-on-oci demo_trial must emit zero-cost billing events.
demo_trial reconciliation rule 6: guest-on-oci demo_trial must expose capacity-exhausted errors before paid resource creation.
demo_trial reconciliation rule 7: guest-on-oci demo_trial must use OCI Object Storage state backend if OpenTofu state is needed.
demo_trial reconciliation rule 8: demo_trial docs must distinguish free OCI from paid baseline profile in other contexts.
Feature requiring paid baseline-or-higher profile on OCI: capacity above 10GB object data.
Feature requiring paid baseline-or-higher profile on OCI: archive above 10GB.
Feature requiring paid baseline-or-higher profile on OCI: cross-region replication with paid egress/storage.
Feature requiring paid baseline-or-higher profile on OCI: high request-rate dedicated pools.
Feature requiring paid baseline-or-higher profile on OCI: object lock with large compliance archives.
Feature requiring paid baseline-or-higher profile on OCI: daily inventory over large object sets.
Feature requiring paid baseline-or-higher profile on OCI: SFTP/NFS gateway nodes.
Feature requiring paid baseline-or-higher profile on OCI: private load balancer beyond free envelope.
Feature requiring paid baseline-or-higher profile on OCI: dedicated KMS/HSM topology.
Feature requiring paid baseline-or-higher profile on OCI: 50GB paid baseline profile from current local tier.
OCI demo_trial verdict: currently incoherent with local demo_trial definition.
OCI demo_trial remediation: add `iac/oci-guest/always-free/`, tier text, quota tests, zero-cost meters, and hard fail-closed capacity enforcement.

## §5 Findings: Per-Tier Ahead/Parity/Catch-Up Classifications

demo_trial classification 1: basic object put/get/list/delete is parity at minimal API level.
demo_trial classification 2: storage class model is catch-up because demo_trial only has Hot.
demo_trial classification 3: durability is catch-up because local target is below 11-nines counterpart posture.
demo_trial classification 4: data protection is missing because versioning, object lock, soft delete, and replication are off or absent.
demo_trial classification 5: OCI Always Free is additive but missing.
demo_trial classification 6: OpenTofu and OS support are canonical gaps.
demo_trial overall: catch-up with one missing additive requirement.

paid baseline profile classification 1: versioning and lifecycle are parity conceptually.
paid baseline profile classification 2: object lock is catch-up because off.
paid baseline profile classification 3: replication is catch-up because single async replica lacks formal RPO.
paid baseline profile classification 4: provider neutrality is drifted because AWS Glacier is baseline.
paid baseline profile classification 5: events, analytics, batch, private networking, and soft delete are missing.
paid baseline profile classification 6: KMS and Cedar are additive partial.
paid baseline profile overall: partial catch-up.

paid production profile classification 1: capacity and object size target approach counterpart parity.
paid production profile classification 2: object lock and legal hold approach parity.
paid production profile classification 3: replication target could be ahead if 5s p95 is measured, but current docs contradict sync/async semantics.
paid production profile classification 4: enterprise operations are catch-up because access points, events, analytics, batch, private networking, data lake namespace, transfer acceleration, and offline transfer are absent.
paid production profile classification 5: Cedar/audit-chain could be ahead after contracts and evidence.
paid production profile classification 6: benchmark claims cannot be used until evidence lands.
paid production profile overall: partial; not union-coverage.

paid regulated profile classification 1: mandatory versioning and sync multi-region are ahead targets.
paid regulated profile classification 2: PQC escrow and audit-chain are ahead targets.
paid regulated profile classification 3: dedicated per-tenant substrate is ahead target for regulated customers.
paid regulated profile classification 4: management features are catch-up because many counterpart controls are absent.
paid regulated profile classification 5: performance targets are ahead but unmeasured.
paid regulated profile classification 6: missing PRD/architecture/IaC/SLO/source/test makes the tier aspirational.
paid regulated profile overall: target-only ahead claims, current evidence catch-up.

Cross-tier finding 1: all tiers need explicit object-size ceilings.
Cross-tier finding 2: all tiers need retention duration bounds.
Cross-tier finding 3: all tiers need lifecycle cadence and transition SLO alignment.
Cross-tier finding 4: all tiers need replication semantics resolved.
Cross-tier finding 5: all tiers need storage-class mapping to counterpart names and Oyatie native classes.
Cross-tier finding 6: all tiers need per-context overlays.
Cross-tier finding 7: all tiers need OpenTofu module coverage.
Cross-tier finding 8: all tiers need supported OS and CI coverage.
Cross-tier finding 9: all tiers need customer-SDK compatibility separated from implementation language policy.
Cross-tier finding 10: all tiers need measured benchmark evidence before performance claims.
Cross-tier finding 11: all tiers need cross-service handoffs with cloud-billing, cloud-data, cloud-kms, cloud-iam, cloud-network, cloud-region, and cloud-observability.
Cross-tier finding 12: all tiers need an explicit threat model.
Cross-tier finding 13: all tiers need incident runbooks.
Cross-tier finding 14: all tiers need a capacity model.
Cross-tier finding 15: all tiers need billing meter definitions.

## Tier Delta Verdict

demo_trial is not coherent until OCI Always Free is separated from paid baseline profile.
paid baseline profile is useful but provider-drifted because of the AWS Glacier baseline.
paid production profile is the strongest local object-storage tier but has a replication contradiction.
paid regulated profile is a clear ambition but lacks buildable evidence.
AWS S3, Google Cloud Storage, and Azure Blob Storage each provide mature management, protection, analytics, networking, transfer, and scale surfaces that are missing or partial in Oyatie cloud-storage.
Oyatie's differentiators are Cedar, KMS shred evidence, audit-chain, policy-visible lifecycle, and OCI Always Free accessibility.
Those differentiators are not yet sufficiently codified in service-local artifacts.
The next tier repair should start with demo_trial/OCI, then replication semantics, then management/analytics gaps.
