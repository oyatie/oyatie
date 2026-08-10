# cloud-storage performance benchmark numbers - 2026-05-20

Audit owner: single-agent cloud-storage lane.
Status: target-number audit, not measured benchmark evidence.
Methodology disclosure: every Oyatie number in this document is a target or planning bound derived from local tier claims, counterpart public limits, and canonical context constraints. These are not measured production results. Measured benchmarks must be added during build phase per ADR-0212 and tied to reproducible evidence bundles.

## Anchor block

1. Canonical sequence: `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-2214`, `:2241-2494`, `:3441-3754`, and `:3756-4146`.
2. Machine-readable direction: `specs/master-plan-sequencing.json:704-867`.
3. Service PRD citation: service-local `PRD.md` is absent; substitute purpose evidence is `docs/decisions/ADR-0702-identity-authz-live-apex.md:8-11` and root cloud PRD `docs/products/cloud/PRD.md:1587-1665`.
4. Service architecture citation: service-local `ARCHITECTURE.md` is absent; substitute architecture evidence is `storage/faqs/storage-engineer-faq.md:15-22` and root crate map `docs/products/cloud/PRD.md:131-137`.
5. Documentation-rigor citation: `docs/standards/documentation-rigor.md:62-81` and `docs/standards/documentation-rigor.md:133-156`.

## Source Notes

Local benchmark file claims measurements at `storage/benchmarks/cloud-storage-vs-s3-vs-gcs-vs-azure-blob-vs-r2-vs-minio.md:3-5`.
Local benchmark file cites reproducibility path at `storage/benchmarks/cloud-storage-vs-s3-vs-gcs-vs-azure-blob-vs-r2-vs-minio.md:124-133`.
The referenced `.foundry/evidence/benchmarks/cloud-storage/2026-05-13T22:14:42Z` directory was not present during this audit.
Therefore all reused local values are treated as target or unverified claimed values.
AWS request-rate source: official AWS S3 overview at `https://aws.amazon.com/documentation-overview/s3/`.
AWS storage-class source: official AWS S3 storage class guide at `https://docs.aws.amazon.com/AmazonS3/latest/userguide/storage-class-intro.html`.
GCS request-rate source: official Cloud Storage request-rate guide at `https://docs.cloud.google.com/storage/docs/request-rate`.
GCS durability/RPO source: official Cloud Storage availability and durability guide at `https://docs.cloud.google.com/storage/docs/availability-durability`.
Azure scale source: official Azure Storage scalability targets at `https://learn.microsoft.com/en-us/azure/storage/common/scalability-targets-standard-account`.
Azure tier source: official Azure Blob access tiers overview at `https://learn.microsoft.com/en-us/azure/storage/blobs/access-tiers-overview`.

## §1 Methodology

Benchmark dimension 1: metadata read latency p50/p95/p99.
Benchmark dimension 2: object GET first-byte latency p50/p95/p99.
Benchmark dimension 3: object PUT acknowledgement latency p50/p95/p99.
Benchmark dimension 4: sustained read throughput per tenant.
Benchmark dimension 5: sustained write throughput per tenant.
Benchmark dimension 6: requests per second ceiling.
Benchmark dimension 7: maximum object size.
Benchmark dimension 8: bucket count per tenant.
Benchmark dimension 9: object count per tenant.
Benchmark dimension 10: replication lag p95.
Benchmark dimension 11: lifecycle transition lag p95.
Benchmark dimension 12: archive restore time.
Benchmark dimension 13: inventory generation freshness.
Benchmark dimension 14: durability target.
Benchmark dimension 15: availability target.
Benchmark dimension 16: concurrent multipart upload sessions.
Benchmark dimension 17: presigned URL issue latency.
Benchmark dimension 18: KMS-wrapped object PUT authorization latency.
Benchmark dimension 19: delete/retention denial latency.
Benchmark dimension 20: control-plane bucket create latency.

Workload 1: small object read, 64 KiB object, warmed metadata path.
Workload 2: medium object read, 16 MiB object, sequential client.
Workload 3: large object multipart upload, 5 GiB object, 64 MiB parts.
Workload 4: metadata-heavy listing, 1 million keys under 100 prefixes.
Workload 5: object-lock denial, delete request against governance/compliance object.
Workload 6: lifecycle transition, 10 million eligible objects.
Workload 7: cross-region replication, 10 MiB objects at sustained write rate.
Workload 8: inventory generation, 1 billion object metadata rows.
Workload 9: archive restore, 10 GiB object from archive class.
Workload 10: presigned URL issue and validation under tenant policy.

Disclosure axis 1: OS.
Tier-1 OS matrix is absent, so targets are platform-neutral until `supported-oses.json` lands.
Disclosure axis 2: architecture.
Targets assume `x86_64` and `aarch64` parity unless service-local OS manifest says otherwise.
Disclosure axis 3: deployment context.
Targets are broken down by six contexts in §4.
Disclosure axis 4: tenant class.
demo_trial is community/small tenant, paid baseline profile is paid baseline, paid production profile is production mid-market, paid regulated profile is hyperscaler/single-tenant-capable.
Disclosure axis 5: storage backend.
Current local docs imply MinIO Enterprise for object storage; this target doc treats backend as an adapter until architecture lands.
Disclosure axis 6: measurement status.
None of the target numbers below should be used as release claims without ADR-0212 measured evidence.

## §2 Counterpart Numbers

### AWS S3 Public Numbers and Public Claims

AWS-1: Request write ceiling baseline: at least 3,500 PUT/COPY/POST/DELETE requests per second per prefix.
AWS-2: Request read ceiling baseline: at least 5,500 GET/HEAD requests per second per prefix.
AWS-3: Prefix count scaling: no limit on number of prefixes, enabling horizontal request-rate scale.
AWS-4: Durability: S3 Standard is designed for 99.999999999% durability.
AWS-5: Availability: S3 Standard is designed for 99.99% availability.
AWS-6: Availability: S3 Standard-IA is designed for 99.9% availability.
AWS-7: Availability: S3 One Zone-IA is designed for 99.5% availability.
AWS-8: Availability: S3 Glacier Instant Retrieval is designed for 99.9% availability.
AWS-9: Availability: S3 Express One Zone is designed for 99.95% availability.
AWS-10: Durability scope: One Zone classes are designed for 11 nines within one availability zone.
AWS-11: Object size: standard S3 object maximum is 5 TiB.
AWS-12: Multipart upload: standard S3 multipart upload supports up to 10,000 parts.
AWS-13: Storage Lens: provides organization/bucket/account-level metrics, not a latency number.
AWS-14: Object Lock: WORM retention is a correctness target, not a throughput target.
AWS-15: Replication RTC: public product target is 15-minute replication completion for most objects when configured.

### Google Cloud Storage Public Numbers and Public Claims

GCS-1: Initial write request capacity: 1000 object write requests per second per bucket.
GCS-2: Initial read request capacity: 5000 object read requests per second per bucket.
GCS-3: Request-rate ramp: bucket request rates can scale beyond initial rates with ramp guidance.
GCS-4: Durability: Cloud Storage is designed for 99.999999999% annual durability.
GCS-5: Turbo replication: RPO target is 15 minutes for dual-region turbo replication.
GCS-6: Default dual/multi-region replication: RPO target is 1 hour for most objects.
GCS-7: Public docs state most objects replicate within minutes under default replication.
GCS-8: Regional storage stores data in one region; dual-region stores data in two regions.
GCS-9: Storage classes: Standard, Nearline, Coldline, Archive.
GCS-10: Soft delete is available as data-protection feature.
GCS-11: Object versioning is available.
GCS-12: Bucket Lock and Object Retention Lock are available.
GCS-13: Parallel composite uploads are documented for upload throughput.
GCS-14: Storage batch operations support bulk work.
GCS-15: Inventory reports and Storage Insights support object metadata analytics.

### Azure Blob Storage Public Numbers and Public Claims

AZ-1: Standard storage account default maximum capacity: 5 PiB.
AZ-2: Request-rate target in listed regions: 40,000 requests per second per storage account.
AZ-3: Request-rate target in other regions: 20,000 requests per second per storage account.
AZ-4: Ingress target in listed regions: 60 Gbps per general-purpose v2 account.
AZ-5: Ingress target in other regions: 25 Gbps per general-purpose v2 account.
AZ-6: Egress target in listed regions: 200 Gbps per general-purpose v2 account.
AZ-7: Egress target in other regions: 50 Gbps per general-purpose v2 account.
AZ-8: Capacity has no limit on number of blob containers, blobs, file shares, tables, queues, entities, or messages under the account target.
AZ-9: Hot/Cool/Cold are online tiers.
AZ-10: Archive is offline tier with rehydration requirement.
AZ-11: Object replication supports block blobs.
AZ-12: Object replication supports up to 1000 replication rules per policy.
AZ-13: Object replication requires change feed.
AZ-14: Immutable storage supports time-based retention and legal hold.
AZ-15: Hierarchical namespace supports Data Lake Storage Gen2 workloads.

## §3 Oyatie Target Numbers by tenant_class Profile

### demo_trial Targets - Baseline and OCI Always Free Ceiling

demo_trial-1: Target tenant class: small community or developer tenant.
demo_trial-2: OCI guest demo_trial storage ceiling: 10GB object data when using OCI Always Free object storage.
demo_trial-3: OCI guest demo_trial archive ceiling: 10GB archive storage when using OCI Always Free archive storage.
demo_trial-4: OCI guest demo_trial block ceiling: 200GB block volume pool.
demo_trial-5: Non-OCI demo_trial target capacity: 50GB per tenant only when paid substrate is available.
demo_trial-6: GET first-byte p50 target: <= 12 ms intra-region.
demo_trial-7: GET first-byte p95 target: <= 35 ms intra-region.
demo_trial-8: PUT ack p50 target: <= 45 ms.
demo_trial-9: PUT ack p95 target: <= 100 ms.
demo_trial-10: Sustained read target: 30 MB/s per tenant, matching local tenant_class policy.
demo_trial-11: Sustained write target: 60 MB/s per tenant, matching local tenant_class policy.
demo_trial-12: Request ceiling target: 500 read requests/sec and 250 write requests/sec per tenant.
demo_trial-13: Bucket count target: 5 buckets per tenant.
demo_trial-14: Object count target: 1 million objects per tenant.
demo_trial-15: Maximum object size target: 10GB for OCI Always Free, 25GB for paid baseline profile.
demo_trial-16: Replication target: none by default.
demo_trial-17: Lifecycle transition p95 target: <= 24 h.
demo_trial-18: Archive restore target: not included in OCI demo_trial; paid baseline profile restore <= 24 h if archive enabled.
demo_trial-19: Inventory freshness target: weekly if enabled.
demo_trial-20: Durability target: 99.99% in local FAQ; must be recomputed per context.
demo_trial-21: Availability target: 99.5% for single-context dev/community posture.
demo_trial-22: Concurrent multipart upload sessions: 10 per tenant.
demo_trial-23: Presigned URL issue latency p95: <= 50 ms.
demo_trial-24: KMS-wrapped PUT authorization p95: <= 120 ms.
demo_trial-25: Retention-denial p95: <= 80 ms.

### paid baseline profile Targets - Paid Baseline

paid baseline profile-1: Target tenant class: serious SMB or departmental workload.
paid baseline profile-2: Capacity target: 10TB per tenant.
paid baseline profile-3: Bucket count target: 50 buckets per tenant.
paid baseline profile-4: Maximum object size target: 100GB unless architecture raises it.
paid baseline profile-5: GET first-byte p50 target: <= 6 ms intra-region.
paid baseline profile-6: GET first-byte p95 target: <= 18 ms intra-region.
paid baseline profile-7: GET first-byte p99 target: <= 45 ms intra-region.
paid baseline profile-8: PUT ack p50 target: <= 14 ms.
paid baseline profile-9: PUT ack p95 target: <= 40 ms.
paid baseline profile-10: PUT ack p99 target: <= 95 ms.
paid baseline profile-11: Sustained read target: 250 MB/s per tenant.
paid baseline profile-12: Sustained write target: 500 MB/s per tenant.
paid baseline profile-13: Request ceiling target: 2500 read requests/sec and 1200 write requests/sec per tenant.
paid baseline profile-14: Replication target: async single target.
paid baseline profile-15: Replication lag p95 target: <= 60 s.
paid baseline profile-16: Lifecycle transition p95 target: <= 6 h until contradiction is resolved; target should become <= 1 h if tenant_class invariant wins.
paid baseline profile-17: Archive restore target: <= 5 min for Cold, <= 12 h for Archive.
paid baseline profile-18: Inventory freshness target: daily.
paid baseline profile-19: Durability target: 99.9999999%.
paid baseline profile-20: Availability target: 99.9%.
paid baseline profile-21: Concurrent multipart upload sessions: 100 per tenant.
paid baseline profile-22: Presigned URL issue latency p95: <= 35 ms.
paid baseline profile-23: KMS-wrapped PUT authorization p95: <= 75 ms.
paid baseline profile-24: Retention-denial p95: <= 45 ms.
paid baseline profile-25: Bucket-create control-plane p99: <= 500 ms.

### paid production profile Targets - Production Scale

paid production profile-1: Target tenant class: mid-market production tenant.
paid production profile-2: Capacity target: 1PB per tenant.
paid production profile-3: Bucket count target: 5000 buckets per tenant.
paid production profile-4: Maximum object size target: 5TB.
paid production profile-5: GET first-byte p50 target: <= 2 ms with edge cache.
paid production profile-6: GET first-byte p95 target: <= 8 ms intra-region.
paid production profile-7: GET first-byte p99 target: <= 20 ms intra-region.
paid production profile-8: PUT ack p50 target: <= 6 ms.
paid production profile-9: PUT ack p95 target: <= 18 ms.
paid production profile-10: PUT ack p99 target: <= 50 ms.
paid production profile-11: Sustained read target: 2.5 GB/s per tenant.
paid production profile-12: Sustained write target: 5 GB/s per tenant.
paid production profile-13: Request ceiling target: 20,000 read requests/sec and 10,000 write requests/sec per tenant.
paid production profile-14: Replication target: explicit decision needed; default audit target is async p95 <= 5 s.
paid production profile-15: Alternative sync target if Wave 14 chooses sync paid production profile: p95 PUT ack must be recalculated above 18 ms.
paid production profile-16: Lifecycle transition p95 target: <= 1 h.
paid production profile-17: Archive restore target: <= 1 min for Cold, <= 4 h for Archive.
paid production profile-18: Inventory freshness target: daily with 4 h completion for billion-object tenant.
paid production profile-19: Durability target: 99.999999999%.
paid production profile-20: Availability target: 99.99%.
paid production profile-21: Concurrent multipart upload sessions: 5000 per tenant.
paid production profile-22: Presigned URL issue latency p95: <= 20 ms.
paid production profile-23: KMS-wrapped PUT authorization p95: <= 40 ms.
paid production profile-24: Retention-denial p95: <= 25 ms.
paid production profile-25: Bucket-create control-plane p99: <= 300 ms.

### paid regulated profile Targets - Hyperscaler Bar

paid regulated profile-1: Target tenant class: single-tenant-capable regulated hyperscale workload.
paid regulated profile-2: Capacity target: practically unbounded by tenant contract, with explicit committed media envelope.
paid regulated profile-3: Bucket count target: unbounded by default, quota-governed.
paid regulated profile-4: Maximum object size target: 5TB for S3 compatibility; larger native composite object target requires separate contract.
paid regulated profile-5: GET first-byte p50 target: <= 0.8 ms with local/edge path.
paid regulated profile-6: GET first-byte p95 target: <= 2 ms intra-cell.
paid regulated profile-7: GET first-byte p99 target: <= 8 ms intra-cell.
paid regulated profile-8: PUT ack p50 target: <= 3 ms local quorum.
paid regulated profile-9: PUT ack p95 target: <= 10 ms local quorum.
paid regulated profile-10: PUT ack p99 target: <= 30 ms local quorum.
paid regulated profile-11: Sustained read target: 100 GB/s aggregate per tenant.
paid regulated profile-12: Sustained write target: 100 GB/s aggregate per tenant.
paid regulated profile-13: Request ceiling target: 100,000 read requests/sec and 50,000 write requests/sec per tenant before custom sharding.
paid regulated profile-14: Replication target: sync local quorum plus async remote, p95 remote visibility <= 250 ms for selected datasets.
paid regulated profile-15: Lifecycle transition p95 target: <= 5 min for policy engine visibility.
paid regulated profile-16: Archive restore target: <= 1 min from warm/cold, <= 1 h from deep archive when media permits.
paid regulated profile-17: Inventory freshness target: hourly delta plus daily full snapshot.
paid regulated profile-18: Durability target: 99.9999999999% planning target, requiring proof before claim.
paid regulated profile-19: Availability target: 99.995% minimum, 99.999% option for dedicated regions.
paid regulated profile-20: Concurrent multipart upload sessions: 50,000 per tenant.
paid regulated profile-21: Presigned URL issue latency p95: <= 10 ms.
paid regulated profile-22: KMS-wrapped PUT authorization p95: <= 20 ms.
paid regulated profile-23: Retention-denial p95: <= 15 ms.
paid regulated profile-24: Bucket-create control-plane p99: <= 150 ms.
paid regulated profile-25: Cross-region failover RTO target: <= 60 s for active-active configured buckets.

## §4 Per-Context Overlay

Context `oyatie-public-cloud` overlay: demo_trial uses shared cells, paid baseline profile uses paid shared clusters, paid production profile uses multi-region dedicated pools, paid regulated profile uses dedicated tenant cells.
Context `oyatie-public-cloud` read target adjustment: no penalty for public regions with local edge cache.
Context `oyatie-public-cloud` write target adjustment: no penalty for local region; replication adds tier-specific latency.
Context `oyatie-public-cloud` capacity adjustment: full paid baseline profile/paid production profile/paid regulated profile targets allowed.
Context `oyatie-public-cloud` availability adjustment: platform SLO can be claimed only after SLO files exist.

Context `guest-on-aws` overlay: storage may use Oyatie data plane backed by AWS infrastructure only through adapters.
Context `guest-on-aws` read target adjustment: add 2 ms p95 when routing through tenant VPC endpoint without local edge cache.
Context `guest-on-aws` write target adjustment: add 5 ms p95 if KMS and storage adapters cross AZ.
Context `guest-on-aws` capacity adjustment: AWS account quota and tenant budget cap maximum throughput.
Context `guest-on-aws` forbidden adjustment: no AWS SDK calls in business logic.

Context `guest-on-oci` overlay: demo_trial must fit OCI Always Free.
Context `guest-on-oci` read target adjustment: demo_trial p95 <= 50 ms because Always Free object budget and network may dominate.
Context `guest-on-oci` write target adjustment: demo_trial p95 <= 150 ms planning bound.
Context `guest-on-oci` capacity adjustment: demo_trial object 10GB, archive 10GB, block 200GB; paid baseline-or-higher profile paid OCI resources required for higher numbers.
Context `guest-on-oci` spillover adjustment: cross-cloud spillover is forbidden.

Context `on-prem` overlay: customer hardware controls disk, network, and failure domains.
Context `on-prem` read target adjustment: demo_trial/paid baseline profile targets only apply after hardware profile is certified.
Context `on-prem` write target adjustment: sync replication targets need rack/AZ mapping.
Context `on-prem` capacity adjustment: capacity ceiling equals purchased media minus erasure and replica overhead.
Context `on-prem` evidence adjustment: restore tests and hardware bill of materials must be included.

Context `colo` overlay: Oyatie or customer-owned colo racks can provide stronger network locality than guest cloud.
Context `colo` read target adjustment: paid production profile/paid regulated profile p50 can improve if compute is in same rack fabric.
Context `colo` write target adjustment: cross-colo replication adds WAN latency.
Context `colo` capacity adjustment: expansion requires rack, power, cooling, and media procurement gates.
Context `colo` availability adjustment: single-colo demo_trial cannot claim multi-region availability.

Context `oyatie-as-cloud-provider` overlay: public provider posture requires stronger endpoint, namespace, DNS, billing, and abuse controls.
Context `oyatie-as-cloud-provider` read target adjustment: targets must include public internet and private endpoint variants.
Context `oyatie-as-cloud-provider` write target adjustment: p95 targets must include KMS, IAM, quota, and audit-chain receipts.
Context `oyatie-as-cloud-provider` capacity adjustment: paid regulated profile must support committed single-tenant contracts.
Context `oyatie-as-cloud-provider` evidence adjustment: external SLA claims require measured benchmark bundles and incident history.

## §5 Comparison Narrative

Headline 1: request-rate ceiling.
AWS has public per-prefix request baselines of 3,500 writes and 5,500 reads per second.
GCS has public initial per-bucket baselines of 1000 writes and 5000 reads per second.
Azure has public account targets of 20,000 or 40,000 requests/sec depending on region/account type.
Oyatie demo_trial target is intentionally below all three.
Oyatie paid baseline profile target approaches one or a few AWS/GCS prefixes but lacks proven scaling.
Oyatie paid production profile target reaches Azure lower account target and multiple AWS/GCS prefix-equivalent scale.
Oyatie paid regulated profile target aims above default account/prefix baselines but requires sharding proof.

Headline 2: durability.
AWS, GCS, and Azure all advertise 11-nines-class durability for common classes or redundancy configurations.
Oyatie demo_trial local FAQ says 99.99%, which is below counterpart object durability.
Oyatie paid baseline profile target is below 11 nines and should not claim hyperscaler parity.
Oyatie paid production profile target reaches 11 nines planning target.
Oyatie paid regulated profile target exceeds 11 nines as a planning target only, requiring proof before claim.

Headline 3: availability.
AWS S3 Standard public availability design target is 99.99%.
GCS and Azure availability depends on location class and redundancy/SLA.
Oyatie demo_trial and paid baseline profile targets are catch-up.
Oyatie paid production profile targets parity for public cloud object workloads.
Oyatie paid regulated profile targets ahead-of-default only for dedicated topology with measured proof.

Headline 4: lifecycle and archive.
AWS, GCS, and Azure all support multiple online/offline storage tiers.
Oyatie has Hot/Warm/Cold/Archive in docs.
Oyatie lacks measured lifecycle scale and has a paid baseline profile 6h versus invariant 1h contradiction.
Oyatie status is catch-up until tier semantics are corrected.

Headline 5: replication.
AWS RTC and GCS turbo replication both provide 15-minute-class RPO targets.
Azure object replication supports rules but is asynchronous.
Oyatie paid production profile docs conflict between async and sync.
Oyatie cannot claim parity until replication mode and RPO evidence are formalized.

Headline 6: object management analytics.
AWS Storage Lens, GCS Storage Insights, and Azure inventory/monitoring are mature.
Oyatie only has inventory report concepts.
Oyatie status is catch-up.

Headline 7: access/private networking.
AWS and Azure expose endpoint/private-network controls; GCS exposes IAM/public access/IP filtering controls.
Oyatie service-local docs lack endpoint/network controls.
Oyatie status is catch-up and requires cloud-network handoff.

Headline 8: compliance retention.
All three counterparts support retention/immutability patterns.
Oyatie has strong conceptual WORM and Cedar hooks.
Oyatie could be ahead if audit-chain and Cedar proofs are formalized.
Current status is partial.

Headline 9: OCI Always Free.
AWS/GCS/Azure counterparts do not define an OCI Always Free demo_trial.
This is Oyatie-additive.
Current status is missing locally.

Headline 10: measured evidence.
AWS/GCS/Azure have public docs and production history.
Oyatie local benchmark evidence is absent.
Current status is planning-only.

## Benchmark Remediation Queue

Remediation 1: create a service-local benchmark harness with workload definitions from §1.
Remediation 2: store raw evidence under a real evidence path.
Remediation 3: publish per-context results, not one global score.
Remediation 4: include OS/arch/tenant class in every benchmark row.
Remediation 5: separate target, lab measurement, and production SLO.
Remediation 6: add object-lock, lifecycle, and replication correctness tests beside latency tests.
Remediation 7: add cost and quota ceilings for OCI Always Free demo_trial.
Remediation 8: rerun competitor comparison only after Oyatie measurements exist.
