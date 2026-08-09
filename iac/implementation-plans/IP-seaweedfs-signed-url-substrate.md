---
ip_id: cloud-iac/IP-seaweedfs-signed-url-substrate
authored: 2026-05-18
scrubbed: 2026-05-21
slice_owner: axis-cloud-iac
related_adrs: [ADR-0083, ADR-0131, ADR-0196]
depends_on: [cloud-iac/IP-seaweedfs-cluster-bootstrap]
ip_status: planned
---

# IP — SeaweedFS signed-URL substrate

## Why this slice

Cloud-iac owns the SeaweedFS substrate chart and the S3 gateway values that make
signed object access possible. This IP records the cloud-iac side of the
contract: gateway topology, secret-reference inputs, provenance reads, and audit
handoffs. It does not claim ownership of an adapter crate path that is not
present in this service.

## Real service paths

| Path | Contract |
|---|---|
| `microservices/cloud-iac/iac/helm/seaweedfs/Chart.yaml` | SeaweedFS 4.22 chart wrapper and dependency |
| `microservices/cloud-iac/iac/helm/seaweedfs/values.yaml` | S3 gateway replicas, bucket list, OpenBao references, metrics |
| `microservices/cloud-iac/contracts/openapi/cloud-iac.yaml` | `getProvenance` and `validateChartSignature` REST surfaces |
| `microservices/cloud-iac/contracts/asyncapi/cloud-iac-events.yaml` | apply/render events that carry artifact lifecycle facts |
| `microservices/cloud-iac/policy/public-read.cedar` | public SLSA attestation and chart-signature validation boundaries |
| `microservices/cloud-iac/policy/ci-scope.cedar` | worker permissions for render/apply/provenance writes |
| `microservices/cloud-iac/cross-microservice-handoffs.md` | counterpart handoffs for artifact provenance, audit, and secret reads |

## Acceptance criteria

1. `values.yaml` keeps the S3 gateway enabled with four replicas and
   OpenBao-backed `accessKeySecretRef` / `secretKeySecretRef` values.
2. Bucket declarations remain explicit in
   `seaweedfs.s3.defaultBuckets`, including workflow artifacts, evidence,
   Velero backup, and audit-chain archive buckets.
3. Public reads remain limited to SLSA attestations and chart-signature
   validation by `policy/public-read.cedar`; signed object URLs are not exposed
   as anonymous apply-state or drift reads.
4. Secret rotation is modeled through the `cloud-secrets.secret.rotated` and
   `cloud-secrets.secret.revoked` subscriptions in the service handoff matrix.
5. Audit/provenance consumers use `getProvenance` and the audit-chain handoff;
   this IP does not introduce a parallel signed-URL API.

## Counterpart refs

- `microservices/cloud-iac/cross-microservice-handoffs.md` inbound
  `application` row uses `GET /charts/{digest}/provenance` for public
  attestation reads.
- `microservices/cloud-iac/cross-microservice-handoffs.md` outbound
  `cloud-secrets` rows provide signer and kubeconfig secret references.
- `microservices/cloud-iac/cross-microservice-handoffs.md` outbound
  `audit-chain` row records apply provenance before infrastructure mutation is
  considered durable.

## Validation commands

```bash
rg "s3:|replicas: 4|accessKeySecretRef|secretKeySecretRef|defaultBuckets" microservices/cloud-iac/iac/helm/seaweedfs/values.yaml
rg "getProvenance|validateChartSignature" microservices/cloud-iac/contracts/openapi/cloud-iac.yaml
rg "read_slsa_attestation_public|validate_chart_signature" microservices/cloud-iac/policy/public-read.cedar
rg "cloud-secrets.secret.rotated|cloud-secrets.secret.revoked" microservices/cloud-iac/cross-microservice-handoffs.md
```
