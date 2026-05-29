# `cloud-storage` µservice — Storage Engineer FAQ

20 real questions raised against `cloud-storage` (the µservice that owns Oyatie's object-storage substrate).

---

**Q1. Does `cloud-storage` replace AWS S3 / GCS / Azure Blob / Cloudflare R2 / MinIO / Backblaze B2?**

For Oyatie-tenant workloads — yes. `cloud-storage` is API-compatible with all of S3, Azure Blob, GCS, and (in paid tenant_class regulated profiles) B2 and
R2. Tenants point existing applications at the `cloud-storage` endpoints; standard SDKs (boto3, Azure Blob SDK, gcloud-python)
just work. We're not a public S3 alternative; we're the per-tenant storage substrate.

---

**Q2. Why MinIO Enterprise as the backend?**

Three reasons:
1. **S3 API completeness** — MinIO Enterprise has the highest API parity with AWS S3 of any non-AWS implementation.
2. **Erasure coding ergonomics** — built-in EC:14+4 in paid tenant_class production profiles gives us cross-rack durability without RAID-style operational pain.
3. **Open source + paid support** — we control the deployment + get vendor support.

Alternative we considered: SeaweedFS (immature ops), Ceph RGW (operational complexity high), proprietary (lock-in).

---

**Q3. What's the AAD binding?**

Every object's DEK is encrypted under the tenant CMK with AAD = `(tenant_id, bucket, key)`. At decrypt time, the same AAD must
match. A cross-tenant read would have AAD mismatch → `AeadAuthFailure`. Per ADR-0251 §D-10. Cross-bucket-same-tenant reads also
refuse without explicit Cedar permit.

---

**Q4. How are storage classes mapped to physical storage?**

- **Hot**: NVMe SSD; ≤ 6 ms p95 first-byte; full IOPS; intended for active read workloads.
- **Warm**: SATA SSD; ≤ 24 ms p95; lower IOPS; intended for infrequent reads.
- **Cold**: HDD spinning + erasure code; ≤ 8 s p95; very low IOPS; restore in ≤ 5 min for paid tenant_class baseline profiles, ≤ 1 min for paid tenant_class production profiles.
- **Archive**: HDD spinning + deep erasure code (EC:18+6); ≤ 12 h restore time; very cheap.
- **Tape** (paid tenant_class regulated profile): LTO-9/LTO-10 tape library; ≤ 24 h restore time; cheapest.
- **Sovereign-Air-Gapped** (paid tenant_class regulated profile): replicates to an air-gapped tape archive via one-way data diode.

---

**Q5. How does object versioning work?**

Each object has a monotonically-increasing version ID (ULID v0 by default, ULID v1 for paid tenant_class production profiles). PUT creates a new version; older
versions remain. DELETE inserts a "delete marker" which is itself a version; the latest non-marker version becomes the "current"
on delete-marker removal. Non-current versions are subject to lifecycle (typically expiration).

---

**Q6. What's the difference between object lock "compliance" and "governance" mode?**

- **Compliance** (SEC Rule 17a-4 compliant): no one can delete or overwrite during the retention period — including the tenant
  admin, including Oyatie support staff. Period.
- **Governance**: tenant admins with explicit `cloud_storage::Action::BypassObjectLock` Cedar permit can override. Use for
  internal compliance + audit but not for regulatory WORM.

Once compliance mode is applied with a non-zero retention, it cannot be reduced; only extended.

---

**Q7. How does cross-region replication work?**

Asynchronous by default; lag ≤ 5 s p95 between US ↔ EU for paid tenant_class production profiles. Configured per-bucket. Each replication rule specifies:
- Filter prefix (which objects).
- Target bucket (must be in a different region).
- Replicate-versions (`current-only` / `all-versions`).
- Replicate-deletes (`yes` / `no`).
- Storage class override (target can be Cold even if source is Hot).
- KMS key replacement (target can use a different CMK).

Synchronous replication is available in paid tenant_class regulated profiles for regulatory data residency mirroring.

---

**Q8. How is bucket policy + Cedar integrated?**

Cedar is authoritative. Bucket policies (S3-style JSON) translate to Cedar at apply time:
```bash
./bin/oya storage bucket-policy put \
  --tenant ... --bucket ... \
  --policy-file bucket-policy.json
```

The translator emits a Cedar policy under `policies/<tenant>/buckets/<bucket>.cedar` and the JSON is annotated with the Cedar
digest. Direct Cedar authoring is preferred; the JSON path is for migration from AWS.

---

**Q9. What's the multipart upload limit?**

Up to 10,000 parts per object (matches S3). Each part ≥ 5 MB (except the last). Maximum object size in paid tenant_class baseline profile: 100 GB. Paid tenant_class production profile: 5 TB.
Paid tenant_class regulated profile: unbounded (we've tested up to 250 TB single-object).

---

**Q10. How are presigned URLs implemented?**

Presigned URLs are HMAC-signed under a tenant-scoped HMAC key (HMAC-SHA-256 by default). The URL embeds the tenant + bucket +
key + expiration + the HMAC. The object-server validates the HMAC + expiration before serving. Cedar policy can also be applied
to presigned URL issuance (e.g. "only issue presigned URLs that expire within 60 min").

---

**Q11. How is the S3-compatible API authentication done?**

We issue short-lived AWS-Sig-v4-compatible access-key + secret-key pairs via `oya storage s3-credential issue`. The keys are
tenant-scoped + can be further scoped (read-only, write-only, prefix-only). TTL is configurable up to the tenant_class/profile maximum
(1 h demo_trial, 4 h paid baseline, 12 h paid production, 4 h paid regulated — regulated is lower because tighter security posture).

---

**Q12. Can a tenant use IAM roles instead of access keys?**

Yes — `cloud-iam` workload identity federation lets a workload assume an Oyatie role that exposes a presigned-credential-issuance
flow. The workload calls `cloud-storage` directly without needing static access keys.

---

**Q13. What's the encryption rotation story?**

KEK rotation happens via `cloud-kms` (per its tenant_class policy). When the KEK rotates, existing object DEKs encrypted under the old
KEK continue to decrypt during the grace window (90 d paid baseline profile). For new writes, the new KEK is used. To re-key existing objects:
```bash
./bin/oya storage object re-encrypt-batch --tenant <t> --bucket <b> --target-kek-version latest
```

This is an O(N) operation; for large buckets, schedule during a maintenance window.

---

**Q14. How is the lifecycle clock measured?**

Per-object, against `last_modified_at`. Lifecycle daemons scan buckets every 6 h (paid baseline) / 1 h (paid production) / 5 min (paid regulated) and
issue transition / expiration operations. Lifecycle events anchor to `audit-chain`.

---

**Q15. How does this work for petabyte-scale tenants?**

Paid tenant_class production profiles support 1 PB per tenant; paid tenant_class regulated profiles are contract-unbounded. The MinIO Enterprise cluster scales horizontally; we shard tenants across
storage cells. Tenant buckets within a single tenant can span cells (transparent to the API).

---

**Q16. Can I get object inventory reports?**

Yes — `cloud_storage::Action::EnableInventoryReport` (paid production-capable profiles) produces daily/weekly Parquet inventory reports of all objects in
a bucket. Useful for billing reconciliation, lifecycle audit, and FOCUS 1.1 mapping in `cloud-billing`.

---

**Q17. How does object lock interact with versioning?**

Object lock is **per-version**. Locking version V1 doesn't lock V2 (which is a new version). If you DELETE a locked object, a
delete-marker is created but V1 remains (locked) and is restorable until retention expires.

---

**Q18. What's the durability target?**

demo_trial: 99.99 % (4 nines; EC:4+2 single-AZ).
paid baseline profile: 99.9999999 % (9 nines; EC:8+4 multi-AZ + cross-region replica option).
paid production profile: 99.999999999 % (11 nines; EC:14+4 + multi-region active-active).
paid regulated profile: 99.999999999999 % (14 nines; EC:18+6 + tape archive + sovereign air-gap).

Measured per AWS S3 model: probability of object loss per year.

---

**Q19. Where does Foundry hook in?**

Foundry pipelines store build artefacts + test results in `cloud-storage` as `oyatie.foundry.<pipeline-id>` principals. The
artefact bucket has a 90-day lifecycle to Cold + 365-day expiration. Cedar permits for Foundry are narrow: `PutObject`,
`GetObject`, `ListObjects` on the foundry artefact bucket only.

---

**Q20. How do I roll back a bad lifecycle policy?**

Lifecycle policies are versioned. Rollback:
```bash
./bin/oya storage lifecycle policy rollback \
  --tenant <t> --bucket <b> --to-version <n>
```

If the previous policy had already transitioned objects (e.g. to Cold), the rollback doesn't reverse the transition automatically;
you can issue per-object `RestoreFromCold` calls to bring specific objects back to Hot if needed. Object data is not lost
unless an `Expiration` rule fired AND the version was permanently deleted.
