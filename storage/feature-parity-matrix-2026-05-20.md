# cloud-storage feature parity matrix - 2026-05-20

Audit owner: single-agent cloud-storage lane.
Counterpart bar: AWS S3 / Google Cloud Storage / Azure Blob Storage.
Purpose: identify union-required object-storage capabilities and classify current Oyatie cloud-storage evidence.

## Anchor block

1. Canonical sequence: `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-2214`, `:2241-2494`, `:3441-3754`, and `:3756-4146`.
2. Machine-readable direction: `specs/master-plan-sequencing.json:704-867`.
3. Service PRD citation: service-local `PRD.md` is absent; substitute purpose evidence is `docs/decisions/ADR-0702-identity-authz-live-apex.md:8-11` and root cloud PRD `docs/products/cloud/PRD.md:1587-1665`.
4. Service architecture citation: service-local `ARCHITECTURE.md` is absent; substitute architecture evidence is `storage/faqs/storage-engineer-faq.md:15-22` and root crate map `docs/products/cloud/PRD.md:131-137`.
5. Documentation-rigor citation: `docs/standards/documentation-rigor.md:62-81` and `docs/standards/documentation-rigor.md:133-156`.

## Source Notes

AWS S3 source used: `https://aws.amazon.com/documentation-overview/s3/` for overview features, request-rate guidance, consistency, access control, and operations families.
AWS S3 source used: `https://docs.aws.amazon.com/AmazonS3/latest/userguide/storage-class-intro.html` for storage-class, durability, and availability claims.
AWS S3 source used: `https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-lock-managing.html` for Object Lock, legal hold, governance bypass, lifecycle, replication, and inventory interactions.
AWS S3 source used: `https://docs.aws.amazon.com/AmazonS3/latest/userguide/configure-inventory.html` for S3 Inventory.
Google Cloud Storage source used: `https://docs.cloud.google.com/storage/docs/introduction` for bucket/object, uploads/downloads, security, protection, notifications, and advanced feature inventory.
Google Cloud Storage source used: `https://docs.cloud.google.com/storage/docs/request-rate` for initial request-rate and ramping guidance.
Google Cloud Storage source used: `https://docs.cloud.google.com/storage/docs/storage-classes` for Standard/Nearline/Coldline/Archive classes.
Google Cloud Storage source used: `https://docs.cloud.google.com/storage/docs/availability-durability` for durability and replication RPO claims.
Azure Blob Storage source used: `https://learn.microsoft.com/en-us/azure/storage/blobs/storage-blobs-introduction` for account/container/blob model, block/append/page blobs, HNS, SFTP, and NFS.
Azure Blob Storage source used: `https://learn.microsoft.com/en-us/azure/storage/blobs/access-tiers-overview` for Hot/Cool/Cold/Archive and online/offline tier behavior.
Azure Blob Storage source used: `https://learn.microsoft.com/en-us/azure/storage/common/scalability-targets-standard-account` for account capacity, requests/sec, ingress, and egress targets.
Azure Blob Storage source used: `https://learn.microsoft.com/en-us/azure/storage/blobs/object-replication-overview` for object replication, change feed, and replication rule constraints.

## §1 Counterpart 1 - AWS S3 Capability Surface

AWS-001: Object storage resource model: buckets, objects, prefixes, and object tags.
AWS-002: Bucket-level namespace and direct bucket host access.
AWS-003: Object metadata and object tagging.
AWS-004: S3 Inventory daily or weekly reports.
AWS-005: S3 Batch Operations for bulk copy, restore, tag, ACL, and Lambda operations.
AWS-006: Versioning for preserving and restoring object versions.
AWS-007: MFA Delete for delete protection.
AWS-008: Same-Region Replication.
AWS-009: Cross-Region Replication.
AWS-010: Replication Time Control.
AWS-011: Multi-Region Access Points.
AWS-012: Object Lock WORM retention.
AWS-013: Legal hold.
AWS-014: Governance bypass permission.
AWS-015: Object Lock with replication.
AWS-016: Object Lock with lifecycle transition.
AWS-017: Object Lock status in inventory.
AWS-018: S3 Standard storage class.
AWS-019: S3 Express One Zone storage class.
AWS-020: S3 Intelligent-Tiering.
AWS-021: S3 Standard-Infrequent Access.
AWS-022: S3 One Zone-Infrequent Access.
AWS-023: S3 Glacier Instant Retrieval.
AWS-024: S3 Glacier Flexible Retrieval.
AWS-025: S3 Glacier Deep Archive.
AWS-026: S3 Outposts.
AWS-027: Storage Class Analysis.
AWS-028: Lifecycle policy transition and expiration.
AWS-029: Archive restore workflow.
AWS-030: IAM-based access control.
AWS-031: Bucket policies.
AWS-032: Object ACLs.
AWS-033: S3 Access Points.
AWS-034: S3 Access Grants.
AWS-035: Query-string authentication / presigned URLs.
AWS-036: S3 Block Public Access.
AWS-037: S3 Object Ownership.
AWS-038: IAM Access Analyzer for S3.
AWS-039: VPC endpoints.
AWS-040: AWS PrivateLink for S3.
AWS-041: Server-side encryption.
AWS-042: Client-side encryption.
AWS-043: KMS integration.
AWS-044: CloudTrail bucket/object activity logs.
AWS-045: CloudWatch operational metrics and alerts.
AWS-046: S3 Event Notifications.
AWS-047: S3 Storage Lens organization-level analytics.
AWS-048: S3 Object Lambda.
AWS-049: S3 Select query-in-place.
AWS-050: Athena and Redshift Spectrum integration.
AWS-051: DataSync online transfer.
AWS-052: Transfer Family SFTP/FTPS/FTP.
AWS-053: Transfer Acceleration.
AWS-054: Snowball offline transfer.
AWS-055: Snowmobile exabyte transfer.
AWS-056: Per-prefix performance scaling.
AWS-057: At least 3,500 write requests per second per prefix.
AWS-058: At least 5,500 read requests per second per prefix.
AWS-059: Strong read-after-write consistency.
AWS-060: Strong list consistency.
AWS-061: Data residency via Outposts and Local Zones.
AWS-062: Object checksum and integrity controls.
AWS-063: Bucket-level cost allocation tags.
AWS-064: Organization-level security policy hooks.
AWS-065: Public website/content serving.

## §2 Counterpart 2 - Google Cloud Storage Capability Surface

GCS-001: Project, bucket, managed folder, folder, and object resource model.
GCS-002: Global, regional, and locational endpoints.
GCS-003: Regional buckets.
GCS-004: Dual-region buckets.
GCS-005: Multi-region buckets.
GCS-006: Standard storage.
GCS-007: Nearline storage.
GCS-008: Coldline storage.
GCS-009: Archive storage.
GCS-010: Legacy multi-regional/regional/DRA classes.
GCS-011: Autoclass.
GCS-012: Object uploads from files.
GCS-013: Object uploads from memory.
GCS-014: Resumable uploads.
GCS-015: XML API multipart uploads.
GCS-016: Parallel composite uploads.
GCS-017: Streaming uploads.
GCS-018: Copy, rename, and move objects.
GCS-019: Storage batch operations.
GCS-020: Composite objects.
GCS-021: Object transcoding.
GCS-022: Bucket list and metadata APIs.
GCS-023: Bucket labels and tags.
GCS-024: Bucket relocation.
GCS-025: Object list.
GCS-026: Object metadata view and edit.
GCS-027: Object storage class change.
GCS-028: Sliced downloads.
GCS-029: Streaming downloads.
GCS-030: CORS configuration.
GCS-031: Batched requests.
GCS-032: Requester Pays.
GCS-033: Storage Insights datasets.
GCS-034: Inventory reports.
GCS-035: Object contexts.
GCS-036: Cloud Storage Rapid Bucket.
GCS-037: Zonal buckets.
GCS-038: Rapid Cache.
GCS-039: Built-in caching.
GCS-040: Hierarchical namespace.
GCS-041: Cloud Storage FUSE.
GCS-042: HMAC keys.
GCS-043: IAM for Cloud Storage.
GCS-044: Managed folder IAM.
GCS-045: ACLs.
GCS-046: Uniform bucket-level access.
GCS-047: Public access prevention.
GCS-048: Bucket IP filtering.
GCS-049: V4 signed URLs.
GCS-050: CMEK.
GCS-051: CSEK.
GCS-052: Standard encryption.
GCS-053: Client-side keys.
GCS-054: Organization policy constraints.
GCS-055: Soft delete.
GCS-056: Object Versioning.
GCS-057: Object holds.
GCS-058: Bucket Lock.
GCS-059: Object Retention Lock.
GCS-060: Pub/Sub notifications.
GCS-061: Cloud Audit Logs.
GCS-062: Usage logs.
GCS-063: Consistency documentation and request preconditions.
GCS-064: Retry guidance.
GCS-065: 1000 initial write requests per second per bucket.
GCS-066: 5000 initial read requests per second per bucket.
GCS-067: Request-rate ramp guidance.
GCS-068: 11-nines durability.
GCS-069: Default replication RPO.
GCS-070: Turbo replication 15-minute RPO.
GCS-071: Cross-bucket replication.
GCS-072: Monitoring for replication RPO conformance.
GCS-073: Interoperability with S3-style HMAC workflows.
GCS-074: Big-data integration.
GCS-075: Static website hosting.

## §3 Counterpart 3 - Azure Blob Storage Capability Surface

AZ-001: Storage account resource model.
AZ-002: Container resource model.
AZ-003: Blob resource model.
AZ-004: Block blobs.
AZ-005: Append blobs.
AZ-006: Page blobs.
AZ-007: Azure Data Lake Storage Gen2 hierarchical namespace.
AZ-008: HTTP/HTTPS object access.
AZ-009: Azure Storage REST API.
AZ-010: Azure CLI access.
AZ-011: Azure PowerShell access.
AZ-012: Client libraries across major languages.
AZ-013: SFTP support.
AZ-014: NFS 3.0 mount support.
AZ-015: Hot tier.
AZ-016: Cool tier.
AZ-017: Cold tier.
AZ-018: Archive tier.
AZ-019: Smart tier.
AZ-020: Online access tiers.
AZ-021: Offline archive rehydration.
AZ-022: Early deletion penalties.
AZ-023: Redundancy options for tiers.
AZ-024: Lifecycle management.
AZ-025: Last-access-time lifecycle.
AZ-026: Blob versioning.
AZ-027: Blob snapshots.
AZ-028: Soft delete.
AZ-029: Immutable storage.
AZ-030: Legal hold.
AZ-031: Time-based retention.
AZ-032: Version-level WORM.
AZ-033: Blob inventory.
AZ-034: Object replication.
AZ-035: Change feed requirement for object replication.
AZ-036: Cross-tenant replication controls.
AZ-037: Replication rules with up to 1000 rules.
AZ-038: Prefix filters for replication.
AZ-039: Customer-managed keys.
AZ-040: Microsoft-managed keys.
AZ-041: Customer-provided-key caveat for replication.
AZ-042: Private endpoints.
AZ-043: IP address rules.
AZ-044: Virtual network rules.
AZ-045: Resource instance rules.
AZ-046: Storage account capacity up to 5 PiB default.
AZ-047: No limit on blobs/containers under account-level target.
AZ-048: 40,000 request/sec default in listed regions.
AZ-049: 20,000 request/sec default in other regions.
AZ-050: 60 Gbps ingress default in listed regions.
AZ-051: 25 Gbps ingress default in other regions.
AZ-052: 200 Gbps egress default in listed regions.
AZ-053: 50 Gbps egress default in other regions.
AZ-054: Error behavior 503/500 when partition limit reached.
AZ-055: Exponential backoff guidance.
AZ-056: Azure Monitor integration.
AZ-057: Storage account DNS endpoint.
AZ-058: GPv2 account recommendation.
AZ-059: Data analysis and backup/restore use cases.
AZ-060: Streaming media use cases.
AZ-061: Log-file storage use case.
AZ-062: Disaster recovery use case.
AZ-063: Archive compliance use case.
AZ-064: Minimum billable size for cooler tiers.
AZ-065: Access-tier transaction charge differentiation.

## §4 UNION-Coverage Matrix

| Capability | AWS S3 | GCS | Azure Blob | UNION required | Oyatie has | Gap classification |
|---|---|---|---|---|---|---|
| Bucket/container namespace | yes | yes | yes | yes | present in docs | present |
| Object put/get/delete/list | yes | yes | yes | yes | present in onboarding/tutorial | present |
| Object metadata | yes | yes | yes | yes | partial | gap: schema absent |
| Object tags/labels/context | yes | yes | partial | yes | missing | gap: model absent |
| Prefix/folder abstraction | yes | yes | yes via HNS | yes | partial | gap: HNS/folder semantics absent |
| Managed folders | no | yes | yes via HNS | yes | missing | gap |
| Versioning | yes | yes | yes | yes | present | present |
| Soft delete | partial via version/delete marker | yes | yes | yes | missing | top gap |
| Legal hold | yes | yes | yes | yes | present concept | gap: contract absent |
| WORM/object lock | yes | yes | yes | yes | present concept | gap: SLO/schema absent |
| Governance bypass | yes | no | no | yes | present concept | gap: policy not formalized |
| Retention mode | yes | yes | yes | yes | present concept | gap: schema absent |
| Retention duration limits | yes | yes | yes | yes | partial | gap: tier bounds absent |
| Lifecycle transitions | yes | yes | yes | yes | present | present but contradictory |
| Lifecycle expiration | yes | yes | yes | yes | present | present |
| Archive restore | yes | yes | yes | yes | partial | gap: restore SLA absent |
| Hot/standard tier | yes | yes | yes | yes | present | present |
| Warm/cool tier | yes | yes Nearline | yes Cool | yes | present | present |
| Cold tier | yes Glacier Instant/Flexible | yes Coldline | yes Cold | yes | present | present |
| Deep archive/offline archive | yes | yes Archive | yes Archive | yes | present | gap: restore evidence absent |
| Intelligent/autoclass/smart tiering | yes | yes | yes | yes | missing | top gap |
| Express/zonal high-performance class | yes | yes Rapid/zonal | no direct | yes | missing | top gap |
| Outposts/on-prem class | yes | no | no | yes | partial | gap: on-prem IaC absent |
| Multi-region active endpoint | yes | yes dual/multi | partial | yes | partial | gap: access-point design absent |
| Cross-region replication | yes | yes | yes | yes | present | contradictory sync/async |
| Same-region replication | yes | yes | yes | yes | partial | gap: rule model absent |
| Replication RPO/RTC | yes | yes turbo | partial SLA | yes | partial | gap: formal SLO absent |
| Cross-bucket replication | yes | yes | yes | yes | present concept | gap: filters/rules absent |
| Replication filters | yes | partial | yes | yes | missing | gap |
| Replication status monitoring | yes | yes | partial | yes | partial | gap |
| Inventory reports | yes | yes | yes | yes | present concept | gap: schema absent |
| Batch operations | yes | yes | partial tasks | yes | missing | top gap |
| Storage analytics/recommendations | yes | yes | partial | yes | missing | top gap |
| Organization-wide lens | yes | yes | Azure Monitor | yes | missing | gap |
| Event notifications | yes | yes Pub/Sub | yes Event Grid | yes | missing | top gap |
| Audit logs | yes | yes | yes | yes | partial | gap: event schema absent |
| Access logs | yes | yes | yes | yes | missing | gap |
| IAM integration | yes | yes | yes Entra/RBAC | yes | partial via Cedar | additive but incomplete |
| Resource policies | yes | yes | yes | yes | present concept | gap: Cedar files absent |
| ACLs | yes | yes | no recommended | yes | missing | gap or deliberate |
| Public access prevention | yes | yes | yes | yes | missing | top gap |
| Access points | yes | no | private endpoint | yes | missing | top gap |
| Private endpoint/VPC endpoint | yes | no direct | yes | yes | missing | top gap |
| Bucket IP filtering | no | yes | IP rules | yes | missing | gap |
| CORS | yes | yes | yes | yes | present in tier | gap: contract absent |
| Presigned/signed URLs | yes | yes | yes SAS | yes | present | present but schema absent |
| Requester pays | yes | yes | no direct | yes | missing | gap |
| Cost allocation tags | yes | labels | tags | yes | partial via billing | gap: storage meters absent |
| KMS/CMEK | yes | yes | yes | yes | present | present concept |
| Customer-supplied keys | partial SSE-C | yes CSEK | yes CPK | yes | missing | gap |
| Client-side encryption | yes | yes | yes | yes | missing | gap |
| Default encryption | yes | yes | yes | yes | partial | gap: default behavior absent |
| Key deletion behavior | yes caveat | yes | yes | yes | missing | top gap |
| Multipart/resumable upload | yes | yes | yes block list | yes | present | present |
| Parallel composite upload | no | yes | partial blocks | yes | missing | gap |
| Append blobs / append semantics | no direct | append in Rapid | yes | yes | missing | gap |
| Page blobs / block volume bridge | no | no | yes | yes | root only | gap: service-local absent |
| SFTP | AWS Transfer Family | no direct | yes | yes | missing | gap |
| NFS mount | Storage Gateway/EFS adjacent | FUSE | yes | yes | root file API only | top gap |
| FUSE | no direct | yes | no direct | yes | missing | gap |
| Static website hosting | yes | yes | yes | yes | missing | gap |
| Query-in-place | yes S3 Select | no direct | no direct | yes | missing | gap |
| Object transform | yes Object Lambda | object transcoding | no direct | yes | missing | gap |
| Data lake integration | yes Athena/Redshift | yes BigQuery | yes ADLS Gen2 | yes | missing | top gap |
| Offline transfer appliance | yes Snowball/Snowmobile | Transfer Appliance | Data Box | yes | missing | gap |
| Online transfer service | yes DataSync | Storage Transfer Service | AzCopy/Data Box Gateway | yes | partial migration | gap |
| Transfer acceleration | yes | global edge behavior | CDN adjacent | yes | missing | gap |
| Strong read-after-write | yes | yes | yes | yes | implied | gap: explicit contract absent |
| Strong list consistency | yes | yes | yes | yes | implied | gap: explicit contract absent |
| Request-rate guidance | yes per prefix | yes per bucket | yes per account | yes | benchmark only | gap: capacity model absent |
| 11-nines durability | yes many classes | yes | yes via redundancy docs | yes | present concept | gap: proof absent |
| Availability SLA tiers | yes | yes | yes | yes | partial | gap: SLO files absent |
| Data residency | yes Outposts/regions | yes locations | yes regions | yes | partial | gap: context manifest absent |
| Sovereign/air-gapped operation | yes Outposts/Snow | partial | Azure Stack adjacent | yes | missing | top gap |
| OpenTofu/IaC deployment | not native | Terraform docs | ARM/Bicep native | yes Oyatie canonical | missing | P1 canonical gap |
| OS support matrix | no service equivalent | no service equivalent | no service equivalent | yes Oyatie canonical | missing | P2 canonical gap |
| OCI Always Free demo_trial | no | no | no | yes Oyatie additive | missing | P1 canonical gap |
| Cedar policy | no | no | no | yes Oyatie additive | concept only | additive gap |
| Per-object KMS shred | no explicit | no explicit | no explicit | yes Oyatie additive | root only | additive gap |
| Audit-chain evidence | no | no | no | yes Oyatie additive | concept only | additive gap |

## §5 Capability Families Summary Table

| Family | UNION required count | Oyatie present count | Oyatie partial count | Major missing items |
|---|---:|---:|---:|---|
| Resource model | 8 | 3 | 3 | managed folders, object contexts, HNS |
| Object operations | 12 | 5 | 4 | batch ops, append/page semantics, transform |
| Data protection | 15 | 5 | 6 | soft delete, snapshots, key-loss behavior |
| Lifecycle/storage classes | 15 | 8 | 4 | smart tiering, high-performance zonal tier |
| Replication/resiliency | 12 | 3 | 6 | RTC/turbo equivalence, filters, monitoring |
| IAM/access control | 15 | 3 | 5 | access points, public access prevention, private endpoints |
| Encryption/KMS | 8 | 2 | 3 | CSEK/CPK, client-side encryption, defaults |
| Observability/audit | 9 | 1 | 3 | event notifications, logs, analytics |
| Migration/transfer | 8 | 2 | 2 | acceleration, offline appliance, transfer service |
| Performance/scale | 8 | 1 | 3 | request model, capacity model, prefix/bucket/account targets |
| Multi-context/IaC | 9 | 0 | 1 | all OpenTofu modules, state backends, signing |
| OS/build/test | 7 | 0 | 1 | supported OS manifest, CI, package formats |
| Additive Oyatie doctrine | 5 | 0 | 3 | OCI Always Free, Cedar files, audit-chain schemas |

## §6 Headline Gap Analysis - Top 15 Missing Capabilities

Gap 1: service-local PRD and architecture are missing.
Implementation hook: create PRD/ARCH that reconcile root object/block/file scope from `docs/products/cloud/PRD.md:131-137`.
Gap 2: six-context deployment manifest is missing.
Implementation hook: add `manifest.json` with all six context IDs from `specs/master-plan-sequencing.json:704-745`.
Gap 3: OpenTofu modules are missing.
Implementation hook: add `iac/oyatie-public-cloud`, `iac/guest-on-aws`, `iac/oci-guest`, `iac/on-prem`, `iac/colo`, `iac/oyatie-iaas`, and `iac/oci-guest/always-free`.
Gap 4: OCI demo_trial Always Free reconciliation is missing.
Implementation hook: cap OCI demo_trial around 10GB object + 10GB archive + 200GB block limits and zero paid spillover.
Gap 5: supported OS matrix is missing.
Implementation hook: add `supported-oses.json` with Tier-1, Tier-2, out-of-scope, package, and CI lane fields.
Gap 6: soft delete is missing.
Implementation hook: add bucket-level soft-delete retention state and restore API, inspired by GCS/Azure.
Gap 7: event notification system is missing.
Implementation hook: emit object-created, object-deleted, lifecycle-transitioned, lock-denied, replication-lagged, and inventory-written events.
Gap 8: batch operations are missing.
Implementation hook: add bulk copy/tag/retention/restore jobs with policy-gated execution.
Gap 9: storage analytics are missing.
Implementation hook: define Storage Lens/Storage Insights equivalent with per-tenant metrics and cost recommendations.
Gap 10: private endpoint and network policy model is missing.
Implementation hook: hand off to cloud-network for VPC endpoint, private link, IP rules, and colo/on-prem access.
Gap 11: hierarchical namespace and file API parity are missing.
Implementation hook: model managed folders, ADLS-style namespace, NFS/SMB/FUSE bridges, or explicitly split file storage into a subservice.
Gap 12: block/page volume parity is missing.
Implementation hook: document block volume API, snapshots, IOPS classes, restore tests, and relationship to Azure page blobs.
Gap 13: smart/autoclass tiering is missing.
Implementation hook: add access-pattern classifier and lifecycle recommender to move objects across cost classes.
Gap 14: replication rule semantics are under-specified.
Implementation hook: define sync/async tier semantics, filters, rule count, RPO, RTO, and residency compatibility.
Gap 15: language-boundary hygiene is missing.
Implementation hook: rewrite examples so operator tooling is Rust/OpenTofu and non-Rust SDKs are customer-client compatibility only.

## §7 Additive Surface

Additive 1: Cedar-first resource policy can exceed provider IAM readability if formalized.
Rationale: local docs already name Cedar actions and translated S3 JSON policy; service lacks policy files.
Additive 2: Per-object KMS shred binding can exceed counterpart encryption evidence if tied to cloud-kms receipts.
Rationale: root cloud PRD says object metadata PUT/GET has per-object KMS shred binding.
Additive 3: Audit-chain anchored object events can exceed ordinary provider logs if events are immutable and tenant-visible.
Rationale: root Oyatie architecture emphasizes audit-chain, but service-local schemas are absent.
Additive 4: OCI Always Free demo_trial is an Oyatie-specific accessibility target.
Rationale: no counterpart has this exact service tier; canonical profile requires it.
Additive 5: Cross-service billing export to cloud-storage creates native FinOps workflows.
Rationale: cloud-billing writes FOCUS exports to tenant-controlled cloud-storage buckets.
Additive 6: Cross-service PITR from cloud-data creates platform-wide recovery semantics.
Rationale: cloud-data streams WAL records into cloud-storage.
Additive 7: Policy-visible lifecycle and retention can become stronger than provider-specific scattered controls.
Rationale: current docs mention lifecycle, retention, Cedar, and audit-chain; formal contracts are the missing step.
Additive 8: Provider-neutral backing adapters can be stronger than AWS/GCS/Azure single-provider coupling.
Rationale: ADR-0328 says S3/OCI Object Storage are adapters, not the product.
Additive 9: Compliance-pack-aware retention can combine legal hold with regional pack obligations.
Rationale: current docs mention SEC 17a-4(f), FINRA 4511(c), GDPR, HIPAA, and ISO packs.
Additive 10: Tenant-class-conditioned object storage can expose explicit demo_trial/paid baseline profile/paid production profile/paid regulated profile obligations.
Rationale: local tenant_class policy exists but needs constraints and context overlays.

## Matrix Verdict

AWS S3, Google Cloud Storage, and Azure Blob Storage all exceed the current service-local cloud-storage surface.
Oyatie cloud-storage has credible object-storage intent and several strong domain concepts.
Oyatie cloud-storage does not yet have union coverage.
The current parity state is partial.
The largest product gap is missing service-local ownership artifacts.
The largest feature gap is provider-grade object management: soft delete, events, batch jobs, analytics, private networking, access-point semantics, and explicit consistency/scale contracts.
The largest canonical gap is missing six-context OpenTofu and OCI Always Free demo_trial.
The next remediation should author PRD and ARCHITECTURE before expanding benchmark numbers or tutorial examples.
