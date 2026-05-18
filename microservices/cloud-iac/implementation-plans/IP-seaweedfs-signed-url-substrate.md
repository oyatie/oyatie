---
ip_id: cloud-iac/IP-seaweedfs-signed-url-substrate
authored: 2026-05-18
slice_owner: axis-cloud-iac
related_adrs: [ADR-0083, ADR-0131, ADR-0196]
depends_on: [cloud-iac/IP-seaweedfs-cluster-bootstrap]
ip_status: planned
---

# IP — SeaweedFS signed-URL substrate

## Why this slice

ADR-0196 D-5 makes pre-signed URLs the canonical way blob bytes are
exposed to clients. The kernel trait `ObjectStore::presign` is shipped
in `oya-shared-object-store-kernel` (this batch). This slice wires the
SeaweedFS-specific signing adapter behind the trait so application code
(Workflow Studio file uploads, Drive attachments, audit-chain archive
reads) uses one surface.

## Acceptance criteria

1. New crate `oya-shared-object-store-seaweedfs-adapter` implements
   `oya-shared-object-store-kernel::ObjectStore`.
2. `presign` produces valid SeaweedFS-signed URLs honoring the ADR-
   0196 D-5 TTL caps (15 min GET, 30 min PUT).
3. Integration test with a local SeaweedFS in `docker-compose` verifies
   round-trip via the signed URL.
4. ServiceMonitor metric `oya_object_store_presigned_url_issued_total`
   carries the canonical tenant-cost labels.
5. OpenBao secret projection for signing keys works; key rotation drill
   succeeds without service interruption.

## File-level work plan

1. `crates/oya-shared-object-store-seaweedfs-adapter/Cargo.toml` +
   `src/lib.rs`.
2. Integration test under `crates/oya-shared-object-store-seaweedfs-
   adapter/tests/` with docker-compose harness.
3. Documentation note in `docs/standards/object-store-canonical.md`
   (the standards doc itself is in the follow-up scope).

## Out-of-scope

- The reverse-proxy gateway in front of SeaweedFS S3 (separate IP).
- AWS S3 adapter (separate IP behind the same trait).

## References

- ADR-0196 — object storage canonical (D-5 signed-URL TTL caps).
- `oya-shared-object-store-kernel` (this batch).
