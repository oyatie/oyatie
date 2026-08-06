# cloud-secrets performance benchmark numbers - 2026-05-20

Audit owner: Wave 2 Batch 2.1 sole-owner audit for `cloud-secrets`.
Methodology disclosure: the Oyatie numbers in this document are target numbers and provenance-aligned planning numbers, not measured benchmark results.
Measurement disclosure: no service-local benchmark evidence bundle was present under `microservices/cloud-secrets/` during this audit.
Build-phase disclosure: measured benchmarks must be added later in the build phase under ADR-0212-style evidence discipline before any production performance claim is made.
Counterpart disclosure: AWS and Google public numbers below are primarily published quota numbers; HashiCorp numbers are official limits/advisory limits plus documented behavior, not a universal SaaS throughput promise.

Citation anchor 1: `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-2495` for multi-context, OpenTofu, and deployment evidence constraints.
Citation anchor 2: `specs/master-plan-sequencing.json:704-866` for six contexts, OpenTofu substrate, supported OSes, Rust policy, and OCI Always Free.
Citation anchor 3: `microservices/cloud-secrets/PRD.md:57-65` for current latency/throughput targets.
Citation anchor 4: `microservices/cloud-secrets/capacity-model.md:60-105` for service capacity envelope and tenant_class drift.
Citation anchor 5: `docs/standards/documentation-rigor.md:133-156` for intern-buildability and hyperscaler-grade proof expectations.
AWS source: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/reference_limits.html`.
AWS source: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/replicate-secrets.html`.
AWS source: `https://docs.aws.amazon.com/secretsmanager/latest/userguide/best-practices.html`.
Google source: `https://docs.cloud.google.com/secret-manager/quotas`.
Google source: `https://docs.cloud.google.com/secret-manager/docs/overview`.
Google source: `https://docs.cloud.google.com/secret-manager/docs/secret-manager-secrets-comparison`.
HashiCorp source: `https://developer.hashicorp.com/vault/docs/internals/limits`.
HashiCorp source: `https://developer.hashicorp.com/vault/docs/concepts/lease`.
HashiCorp source: `https://developer.hashicorp.com/vault/docs/audit`.
HashiCorp source: `https://developer.hashicorp.com/vault/docs/enterprise/replication`.

## §1 Methodology

1. Benchmark dimension: read latency p50, p95, p99.
2. Benchmark dimension: write latency p50, p95, p99.
3. Benchmark dimension: rotation completion latency p95 and p99.
4. Benchmark dimension: HSM envelope latency p95 and p99.
5. Benchmark dimension: audit seal latency p95 and p99.
6. Benchmark dimension: sustained read throughput in requests per second.
7. Benchmark dimension: sustained write throughput in requests per second.
8. Benchmark dimension: sustained rotation throughput in rotations per minute.
9. Benchmark dimension: concurrent tenant namespaces.
10. Benchmark dimension: stored secrets per tenant.
11. Benchmark dimension: stored secret versions per secret.
12. Benchmark dimension: maximum secret payload size.
13. Benchmark dimension: audit backlog drain rate.
14. Benchmark dimension: cross-region replication lag target.
15. Benchmark dimension: outage recovery time objective.
16. Test workload: 70 percent resolve/read operations.
17. Test workload: 10 percent secret metadata reads.
18. Test workload: 8 percent secret writes.
19. Test workload: 5 percent rotations.
20. Test workload: 3 percent namespace operations.
21. Test workload: 2 percent audit queries.
22. Test workload: 2 percent restore/rollback/version operations.
23. Resolve workload: 1 KiB secret payload unless tenant_class-specific payload limits differ.
24. Write workload: 1 KiB secret payload plus metadata labels, tenant ID, context ID, and audit envelope.
25. Rotation workload: database credential rotation with two-phase prepare/promote/seal event.
26. HSM workload: envelope unwrap/wrap path with cached metadata and uncached cryptographic operation.
27. Audit workload: one sealed audit row and one event emission per mutating operation.
28. OS/arch disclosure: Tier-1 Linux amd64 and arm64 targets are required but not yet service-local declared.
29. OS/arch disclosure: macOS Apple Silicon M5+ is a Tier-1 client/developer target, not a server workload target unless separately declared.
30. OS/arch disclosure: Tier-2 ppc64le and s390x are test-only until service-local support manifest says otherwise.
31. Deployment context disclosure: all six canonical contexts are listed because no correct N/A was found.
32. Tenant class disclosure: demo_trial and paid tenant_class are service tenant_class adoption model.
33. OCI disclosure: demo_trial in guest-on-oci must mean OCI Always Free per canonical direction.
34. Evidence disclosure: current benchmark file claims measured data, but this audit did not verify the named evidence path under the service inventory.
35. Safety disclosure: numbers below are deliberately lower than aspirational hyperscaler claims for early tiers.
36. Safety disclosure: paid target numbers are hyperscaler-bar planning targets and require measured validation before publication.
37. Target rule: p99 targets are only valid if audit seal behavior is satisfied.
38. Target rule: any fail-open audit mode must be measured separately and cannot satisfy compliance-critical targets.
39. Target rule: read targets distinguish cache hit and cache miss because PRD currently sets both.
40. Target rule: all contexts must include deployment_context, region, tenant, and tenant_class labels in measured evidence.

## §2 Counterpart numbers

### AWS Secrets Manager published quota and behavior numbers

1. AWS-01 source: quota table, `GetSecretValue` default quota is 10,000 requests/second per supported Region.
2. AWS-02 source: quota table, `DescribeSecret` default quota is 40,000 requests/second per supported Region.
3. AWS-03 source: quota table, `BatchGetSecretValue` default quota is 100 requests/second per supported Region.
4. AWS-04 source: quota table, `ListSecrets` default quota is 100 requests/second per supported Region.
5. AWS-05 source: quota table, `CreateSecret` default quota is 50 requests/second per supported Region.
6. AWS-06 source: quota table, combined mutating/update/replication API default quota is 50 requests/second per supported Region.
7. AWS-07 source: quota table, `RotateSecret` plus `CancelRotateSecret` default quota is 50 requests/second per supported Region.
8. AWS-08 source: quota table, secret value maximum is 65,536 bytes.
9. AWS-09 source: quota table, maximum secrets per Region/account is 500,000.
10. AWS-10 source: quota table, versions per secret maximum is 100.
11. AWS-11 source: quota table, staging labels across all versions maximum is 20.
12. AWS-12 source: quota guidance, avoid updating a secret value more than once every 10 minutes sustained.
13. AWS-13 source: replication doc, rotation of primary propagates new value to replica secrets.
14. AWS-14 source: best-practices doc, automatic rotation can be as often as every four hours.
15. AWS-15 source: best-practices doc, client-side caching is recommended to reduce retrieval latency and improve availability.

### Google Secret Manager published quota and behavior numbers

1. GCP-01 source: quota table, access request quota is 90,000 per minute per project.
2. GCP-02 source: quota table, access request quota converts to 1,500 requests/second per project before increases.
3. GCP-03 source: quota table, read request quota is 600 per minute per project.
4. GCP-04 source: quota table, read request quota converts to 10 requests/second per project before increases.
5. GCP-05 source: quota table, write request quota is 600 per minute per project.
6. GCP-06 source: quota table, write request quota converts to 10 requests/second per project before increases.
7. GCP-07 source: Parameter Manager quota table, render request quota is 1,800 per minute per project.
8. GCP-08 source: Parameter Manager quota table, render request quota converts to 30 requests/second per project.
9. GCP-09 source: Parameter Manager content limit, payload size is 1 MiB for a parameter version.
10. GCP-10 source: overview, automatic replication is charged as one location.
11. GCP-11 source: overview, user-managed replication can select custom regions.
12. GCP-12 source: regional comparison, regional service stores data in a single location.
13. GCP-13 source: regional comparison, regional service prevents cross-region access.
14. GCP-14 source: locations doc, Secret Manager does not support zones, dual-regional locations, or multi-regional locations for Secret Manager resources.
15. GCP-15 source: overview, secrets are encrypted at rest using AES-256 and in transit by TLS.

### HashiCorp Vault and HCP Vault Secrets published limits and behavior numbers

1. HCV-01 source: limits doc, integrated storage default entry size limit is 1 MiB.
2. HCV-02 source: limits doc, Vault automatically chunks entries larger than 512 KiB and smaller than configured maximum.
3. HCV-03 source: limits doc, HTTP request maximum defaults to 32 MiB.
4. HCV-04 source: limits doc, maximum request duration defaults to 90 seconds.
5. HCV-05 source: limits doc, client-side `VAULT_CLIENT_TIMEOUT` default is 60 seconds.
6. HCV-06 source: limits doc, maximum cluster size has no fixed implementation limit and is bounded by active node capability.
7. HCV-07 source: limits doc, maximum DR replicas have no fixed implementation limit but depend on active node capability.
8. HCV-08 source: limits doc, maximum performance replicas have no fixed implementation limit but depend on active node capability.
9. HCV-09 source: limits doc, lease count advisory limit is 256,000.
10. HCV-10 source: limits doc, maximum lease/token duration defaults to 768 hours.
11. HCV-11 source: lease doc, dynamic secrets always have leases.
12. HCV-12 source: lease doc, leases can be revoked manually or automatically at expiry.
13. HCV-13 source: audit doc, Vault recommends at least two audit devices.
14. HCV-14 source: audit doc, if all audit devices are unavailable, Vault effectively becomes unavailable.
15. HCV-15 source: replication doc, local/shared/ignored replication categories shape cross-cluster behavior.

## §3 Oyatie target numbers by tenant_class and deployment context

### demo_trial targets

1. demo_trial / oyatie-public-cloud target p50 cache-hit resolve: 4 ms.
2. demo_trial / oyatie-public-cloud target p95 cache-hit resolve: 18 ms.
3. demo_trial / oyatie-public-cloud target p99 cache-hit resolve: 35 ms.
4. demo_trial / oyatie-public-cloud target p99 cache-miss resolve: 90 ms.
5. demo_trial / oyatie-public-cloud target sustained read throughput: 150 rps.
6. demo_trial / oyatie-public-cloud target sustained write throughput: 20 rps.
7. demo_trial / oyatie-public-cloud target rotation throughput: 20 rotations/minute.
8. demo_trial / oyatie-public-cloud target concurrent namespaces: 100.
9. demo_trial / oyatie-public-cloud target secrets per tenant: 250.
10. demo_trial / oyatie-public-cloud target RTO: 60 minutes.
11. demo_trial / guest-on-aws target p50 cache-hit resolve: 5 ms.
12. demo_trial / guest-on-aws target p95 cache-hit resolve: 22 ms.
13. demo_trial / guest-on-aws target p99 cache-hit resolve: 45 ms.
14. demo_trial / guest-on-aws target p99 cache-miss resolve: 110 ms.
15. demo_trial / guest-on-aws target sustained read throughput: 125 rps.
16. demo_trial / guest-on-aws target sustained write throughput: 15 rps.
17. demo_trial / guest-on-aws target rotation throughput: 15 rotations/minute.
18. demo_trial / guest-on-aws target concurrent namespaces: 75.
19. demo_trial / guest-on-aws target secrets per tenant: 200.
20. demo_trial / guest-on-aws target RTO: 75 minutes.
21. demo_trial / guest-on-oci target p50 cache-hit resolve: 6 ms.
22. demo_trial / guest-on-oci target p95 cache-hit resolve: 25 ms.
23. demo_trial / guest-on-oci target p99 cache-hit resolve: 55 ms.
24. demo_trial / guest-on-oci target p99 cache-miss resolve: 130 ms.
25. demo_trial / guest-on-oci target sustained read throughput: 90 rps.
26. demo_trial / guest-on-oci target sustained write throughput: 10 rps.
27. demo_trial / guest-on-oci target rotation throughput: 8 rotations/minute.
28. demo_trial / guest-on-oci target concurrent namespaces: 40.
29. demo_trial / guest-on-oci target secrets per tenant: 150.
30. demo_trial / guest-on-oci target RTO: 120 minutes.
31. demo_trial / on-prem target p50 cache-hit resolve: 6 ms.
32. demo_trial / on-prem target p95 cache-hit resolve: 28 ms.
33. demo_trial / on-prem target p99 cache-hit resolve: 60 ms.
34. demo_trial / on-prem target p99 cache-miss resolve: 150 ms.
35. demo_trial / on-prem target sustained read throughput: 80 rps.
36. demo_trial / on-prem target sustained write throughput: 10 rps.
37. demo_trial / on-prem target rotation throughput: 8 rotations/minute.
38. demo_trial / on-prem target concurrent namespaces: 40.
39. demo_trial / on-prem target secrets per tenant: 150.
40. demo_trial / on-prem target RTO: 120 minutes.
41. demo_trial / colo target p50 cache-hit resolve: 6 ms.
42. demo_trial / colo target p95 cache-hit resolve: 30 ms.
43. demo_trial / colo target p99 cache-hit resolve: 65 ms.
44. demo_trial / colo target p99 cache-miss resolve: 160 ms.
45. demo_trial / colo target sustained read throughput: 75 rps.
46. demo_trial / colo target sustained write throughput: 10 rps.
47. demo_trial / colo target rotation throughput: 8 rotations/minute.
48. demo_trial / colo target concurrent namespaces: 40.
49. demo_trial / colo target secrets per tenant: 150.
50. demo_trial / colo target RTO: 120 minutes.
51. demo_trial / oyatie-as-cloud-provider target p50 cache-hit resolve: 4 ms.
52. demo_trial / oyatie-as-cloud-provider target p95 cache-hit resolve: 16 ms.
53. demo_trial / oyatie-as-cloud-provider target p99 cache-hit resolve: 32 ms.
54. demo_trial / oyatie-as-cloud-provider target p99 cache-miss resolve: 80 ms.
55. demo_trial / oyatie-as-cloud-provider target sustained read throughput: 200 rps.
56. demo_trial / oyatie-as-cloud-provider target sustained write throughput: 25 rps.
57. demo_trial / oyatie-as-cloud-provider target rotation throughput: 25 rotations/minute.
58. demo_trial / oyatie-as-cloud-provider target concurrent namespaces: 150.
59. demo_trial / oyatie-as-cloud-provider target secrets per tenant: 300.
60. demo_trial / oyatie-as-cloud-provider target RTO: 45 minutes.

### paid targets

1. paid / oyatie-public-cloud target p50 cache-hit resolve: 3 ms.
2. paid / oyatie-public-cloud target p95 cache-hit resolve: 12 ms.
3. paid / oyatie-public-cloud target p99 cache-hit resolve: 24 ms.
4. paid / oyatie-public-cloud target p99 cache-miss resolve: 60 ms.
5. paid / oyatie-public-cloud target sustained read throughput: 1,000 rps.
6. paid / oyatie-public-cloud target sustained write throughput: 100 rps.
7. paid / oyatie-public-cloud target rotation throughput: 100 rotations/minute.
8. paid / oyatie-public-cloud target concurrent namespaces: 1,000.
9. paid / oyatie-public-cloud target secrets per tenant: 1,000.
10. paid / oyatie-public-cloud target RTO: 30 minutes.
11. paid / guest-on-aws target p50 cache-hit resolve: 4 ms.
12. paid / guest-on-aws target p95 cache-hit resolve: 14 ms.
13. paid / guest-on-aws target p99 cache-hit resolve: 28 ms.
14. paid / guest-on-aws target p99 cache-miss resolve: 70 ms.
15. paid / guest-on-aws target sustained read throughput: 800 rps.
16. paid / guest-on-aws target sustained write throughput: 80 rps.
17. paid / guest-on-aws target rotation throughput: 80 rotations/minute.
18. paid / guest-on-aws target concurrent namespaces: 800.
19. paid / guest-on-aws target secrets per tenant: 1,000.
20. paid / guest-on-aws target RTO: 30 minutes.
21. paid / guest-on-oci target p50 cache-hit resolve: 4 ms.
22. paid / guest-on-oci target p95 cache-hit resolve: 16 ms.
23. paid / guest-on-oci target p99 cache-hit resolve: 35 ms.
24. paid / guest-on-oci target p99 cache-miss resolve: 85 ms.
25. paid / guest-on-oci target sustained read throughput: 600 rps.
26. paid / guest-on-oci target sustained write throughput: 60 rps.
27. paid / guest-on-oci target rotation throughput: 60 rotations/minute.
28. paid / guest-on-oci target concurrent namespaces: 600.
29. paid / guest-on-oci target secrets per tenant: 800.
30. paid / guest-on-oci target RTO: 45 minutes.
31. paid / on-prem target p50 cache-hit resolve: 5 ms.
32. paid / on-prem target p95 cache-hit resolve: 18 ms.
33. paid / on-prem target p99 cache-hit resolve: 40 ms.
34. paid / on-prem target p99 cache-miss resolve: 100 ms.
35. paid / on-prem target sustained read throughput: 500 rps.
36. paid / on-prem target sustained write throughput: 50 rps.
37. paid / on-prem target rotation throughput: 50 rotations/minute.
38. paid / on-prem target concurrent namespaces: 500.
39. paid / on-prem target secrets per tenant: 800.
40. paid / on-prem target RTO: 45 minutes.
41. paid / colo target p50 cache-hit resolve: 5 ms.
42. paid / colo target p95 cache-hit resolve: 20 ms.
43. paid / colo target p99 cache-hit resolve: 45 ms.
44. paid / colo target p99 cache-miss resolve: 110 ms.
45. paid / colo target sustained read throughput: 450 rps.
46. paid / colo target sustained write throughput: 45 rps.
47. paid / colo target rotation throughput: 45 rotations/minute.
48. paid / colo target concurrent namespaces: 450.
49. paid / colo target secrets per tenant: 750.
50. paid / colo target RTO: 45 minutes.
51. paid / oyatie-as-cloud-provider target p50 cache-hit resolve: 3 ms.
52. paid / oyatie-as-cloud-provider target p95 cache-hit resolve: 10 ms.
53. paid / oyatie-as-cloud-provider target p99 cache-hit resolve: 20 ms.
54. paid / oyatie-as-cloud-provider target p99 cache-miss resolve: 50 ms.
55. paid / oyatie-as-cloud-provider target sustained read throughput: 1,500 rps.
56. paid / oyatie-as-cloud-provider target sustained write throughput: 150 rps.
57. paid / oyatie-as-cloud-provider target rotation throughput: 150 rotations/minute.
58. paid / oyatie-as-cloud-provider target concurrent namespaces: 1,500.
59. paid / oyatie-as-cloud-provider target secrets per tenant: 1,500.
60. paid / oyatie-as-cloud-provider target RTO: 20 minutes.

### paid targets

1. paid / oyatie-public-cloud target p50 cache-hit resolve: 2 ms.
2. paid / oyatie-public-cloud target p95 cache-hit resolve: 8 ms.
3. paid / oyatie-public-cloud target p99 cache-hit resolve: 18 ms.
4. paid / oyatie-public-cloud target p99 cache-miss resolve: 45 ms.
5. paid / oyatie-public-cloud target sustained read throughput: 8,000 rps.
6. paid / oyatie-public-cloud target sustained write throughput: 500 rps.
7. paid / oyatie-public-cloud target rotation throughput: 500 rotations/minute.
8. paid / oyatie-public-cloud target concurrent namespaces: 10,000.
9. paid / oyatie-public-cloud target secrets per tenant: 10,000.
10. paid / oyatie-public-cloud target RTO: 10 minutes.
11. paid / guest-on-aws target p50 cache-hit resolve: 3 ms.
12. paid / guest-on-aws target p95 cache-hit resolve: 10 ms.
13. paid / guest-on-aws target p99 cache-hit resolve: 22 ms.
14. paid / guest-on-aws target p99 cache-miss resolve: 55 ms.
15. paid / guest-on-aws target sustained read throughput: 5,000 rps.
16. paid / guest-on-aws target sustained write throughput: 350 rps.
17. paid / guest-on-aws target rotation throughput: 350 rotations/minute.
18. paid / guest-on-aws target concurrent namespaces: 7,500.
19. paid / guest-on-aws target secrets per tenant: 7,500.
20. paid / guest-on-aws target RTO: 15 minutes.
21. paid / guest-on-oci target p50 cache-hit resolve: 3 ms.
22. paid / guest-on-oci target p95 cache-hit resolve: 12 ms.
23. paid / guest-on-oci target p99 cache-hit resolve: 28 ms.
24. paid / guest-on-oci target p99 cache-miss resolve: 70 ms.
25. paid / guest-on-oci target sustained read throughput: 3,500 rps.
26. paid / guest-on-oci target sustained write throughput: 250 rps.
27. paid / guest-on-oci target rotation throughput: 250 rotations/minute.
28. paid / guest-on-oci target concurrent namespaces: 5,000.
29. paid / guest-on-oci target secrets per tenant: 5,000.
30. paid / guest-on-oci target RTO: 20 minutes.
31. paid / on-prem target p50 cache-hit resolve: 4 ms.
32. paid / on-prem target p95 cache-hit resolve: 15 ms.
33. paid / on-prem target p99 cache-hit resolve: 35 ms.
34. paid / on-prem target p99 cache-miss resolve: 90 ms.
35. paid / on-prem target sustained read throughput: 2,500 rps.
36. paid / on-prem target sustained write throughput: 200 rps.
37. paid / on-prem target rotation throughput: 200 rotations/minute.
38. paid / on-prem target concurrent namespaces: 3,500.
39. paid / on-prem target secrets per tenant: 5,000.
40. paid / on-prem target RTO: 25 minutes.
41. paid / colo target p50 cache-hit resolve: 4 ms.
42. paid / colo target p95 cache-hit resolve: 16 ms.
43. paid / colo target p99 cache-hit resolve: 38 ms.
44. paid / colo target p99 cache-miss resolve: 95 ms.
45. paid / colo target sustained read throughput: 2,000 rps.
46. paid / colo target sustained write throughput: 180 rps.
47. paid / colo target rotation throughput: 180 rotations/minute.
48. paid / colo target concurrent namespaces: 3,000.
49. paid / colo target secrets per tenant: 4,000.
50. paid / colo target RTO: 25 minutes.
51. paid / oyatie-as-cloud-provider target p50 cache-hit resolve: 2 ms.
52. paid / oyatie-as-cloud-provider target p95 cache-hit resolve: 7 ms.
53. paid / oyatie-as-cloud-provider target p99 cache-hit resolve: 15 ms.
54. paid / oyatie-as-cloud-provider target p99 cache-miss resolve: 35 ms.
55. paid / oyatie-as-cloud-provider target sustained read throughput: 12,000 rps.
56. paid / oyatie-as-cloud-provider target sustained write throughput: 750 rps.
57. paid / oyatie-as-cloud-provider target rotation throughput: 750 rotations/minute.
58. paid / oyatie-as-cloud-provider target concurrent namespaces: 15,000.
59. paid / oyatie-as-cloud-provider target secrets per tenant: 15,000.
60. paid / oyatie-as-cloud-provider target RTO: 8 minutes.

### paid targets

1. paid / oyatie-public-cloud target p50 cache-hit resolve: 1.5 ms.
2. paid / oyatie-public-cloud target p95 cache-hit resolve: 5 ms.
3. paid / oyatie-public-cloud target p99 cache-hit resolve: 10 ms.
4. paid / oyatie-public-cloud target p99 cache-miss resolve: 25 ms.
5. paid / oyatie-public-cloud target sustained read throughput: 50,000 rps.
6. paid / oyatie-public-cloud target sustained write throughput: 2,000 rps.
7. paid / oyatie-public-cloud target rotation throughput: 2,000 rotations/minute.
8. paid / oyatie-public-cloud target concurrent namespaces: 100,000.
9. paid / oyatie-public-cloud target secrets per tenant: 100,000.
10. paid / oyatie-public-cloud target RTO: 5 minutes.
11. paid / guest-on-aws target p50 cache-hit resolve: 2 ms.
12. paid / guest-on-aws target p95 cache-hit resolve: 6 ms.
13. paid / guest-on-aws target p99 cache-hit resolve: 15 ms.
14. paid / guest-on-aws target p99 cache-miss resolve: 35 ms.
15. paid / guest-on-aws target sustained read throughput: 20,000 rps.
16. paid / guest-on-aws target sustained write throughput: 1,000 rps.
17. paid / guest-on-aws target rotation throughput: 1,000 rotations/minute.
18. paid / guest-on-aws target concurrent namespaces: 50,000.
19. paid / guest-on-aws target secrets per tenant: 50,000.
20. paid / guest-on-aws target RTO: 8 minutes.
21. paid / guest-on-oci target p50 cache-hit resolve: 2 ms.
22. paid / guest-on-oci target p95 cache-hit resolve: 8 ms.
23. paid / guest-on-oci target p99 cache-hit resolve: 18 ms.
24. paid / guest-on-oci target p99 cache-miss resolve: 45 ms.
25. paid / guest-on-oci target sustained read throughput: 15,000 rps.
26. paid / guest-on-oci target sustained write throughput: 800 rps.
27. paid / guest-on-oci target rotation throughput: 800 rotations/minute.
28. paid / guest-on-oci target concurrent namespaces: 40,000.
29. paid / guest-on-oci target secrets per tenant: 40,000.
30. paid / guest-on-oci target RTO: 10 minutes.
31. paid / on-prem target p50 cache-hit resolve: 3 ms.
32. paid / on-prem target p95 cache-hit resolve: 10 ms.
33. paid / on-prem target p99 cache-hit resolve: 25 ms.
34. paid / on-prem target p99 cache-miss resolve: 65 ms.
35. paid / on-prem target sustained read throughput: 10,000 rps.
36. paid / on-prem target sustained write throughput: 600 rps.
37. paid / on-prem target rotation throughput: 600 rotations/minute.
38. paid / on-prem target concurrent namespaces: 25,000.
39. paid / on-prem target secrets per tenant: 25,000.
40. paid / on-prem target RTO: 15 minutes.
41. paid / colo target p50 cache-hit resolve: 3 ms.
42. paid / colo target p95 cache-hit resolve: 12 ms.
43. paid / colo target p99 cache-hit resolve: 28 ms.
44. paid / colo target p99 cache-miss resolve: 70 ms.
45. paid / colo target sustained read throughput: 8,000 rps.
46. paid / colo target sustained write throughput: 500 rps.
47. paid / colo target rotation throughput: 500 rotations/minute.
48. paid / colo target concurrent namespaces: 20,000.
49. paid / colo target secrets per tenant: 20,000.
50. paid / colo target RTO: 15 minutes.
51. paid / oyatie-as-cloud-provider target p50 cache-hit resolve: 1 ms.
52. paid / oyatie-as-cloud-provider target p95 cache-hit resolve: 4 ms.
53. paid / oyatie-as-cloud-provider target p99 cache-hit resolve: 8 ms.
54. paid / oyatie-as-cloud-provider target p99 cache-miss resolve: 20 ms.
55. paid / oyatie-as-cloud-provider target sustained read throughput: 75,000 rps.
56. paid / oyatie-as-cloud-provider target sustained write throughput: 3,000 rps.
57. paid / oyatie-as-cloud-provider target rotation throughput: 3,000 rotations/minute.
58. paid / oyatie-as-cloud-provider target concurrent namespaces: 150,000.
59. paid / oyatie-as-cloud-provider target secrets per tenant: 150,000.
60. paid / oyatie-as-cloud-provider target RTO: 3 minutes.

## §4 Per-context overlay

1. `oyatie-public-cloud` overlay: internal platform control should allow tighter p99 targets than guest contexts.
2. `oyatie-public-cloud` overlay: state backend should be Oyatie-managed and signed OpenTofu-backed once implemented.
3. `oyatie-public-cloud` overlay: audit sink should be local to Oyatie evidence infrastructure.
4. `oyatie-public-cloud` overlay: paid and paid targets may approach AWS read quota class only after measured evidence.
5. `guest-on-aws` overlay: read throughput target should stay below AWS Secrets Manager direct quota unless Oyatie proves local cache/offload path.
6. `guest-on-aws` overlay: KMS/HSM latency may depend on AWS KMS, CloudHSM, or local HSM adapter choice.
7. `guest-on-aws` overlay: private endpoint target must map to AWS VPC endpoint or Oyatie mesh endpoint.
8. `guest-on-aws` overlay: cross-region replication must not call AWS Secrets Manager business APIs directly from service logic.
9. `guest-on-oci` overlay: demo_trial must fit Always Free constraints.
10. `guest-on-oci` overlay: demo_trial read target is lower because Always Free resources are bounded.
11. `guest-on-oci` overlay: paid tenant_class can use paid OCI primitives only when tenant_class explicitly allows.
12. `guest-on-oci` overlay: audit retention and object storage must respect Always Free budget in demo_trial.
13. `on-prem` overlay: targets depend on customer HSM, storage, and network latency.
14. `on-prem` overlay: fail-closed audit mode may dominate p99 in poorly provisioned customer environments.
15. `on-prem` overlay: RTO assumes customer-provided backup and restore prerequisites.
16. `on-prem` overlay: package support must follow the OS support manifest once written.
17. `colo` overlay: targets are close to on-prem but should assume weaker elasticity and slower spare-capacity recovery.
18. `colo` overlay: HSM custody and unseal workflow need explicit operator constraints.
19. `colo` overlay: replication lag must account for leased or private circuits.
20. `colo` overlay: paid requires dedicated hardware and cannot be claimed on shared low-end nodes.
21. `oyatie-as-cloud-provider` overlay: this context should have the tightest control-plane and quota targets.
22. `oyatie-as-cloud-provider` overlay: KMS/secrets is mandatory for provider maturity.
23. `oyatie-as-cloud-provider` overlay: paid targets require single-tenant cells and isolated audit devices.
24. `oyatie-as-cloud-provider` overlay: target read throughput is intentionally above AWS default single-Region read quota, but only as a measured future claim.
25. Cross-context rule: no context can claim a tenant_class until its OpenTofu module, OS manifest, and SLO evidence exist.
26. Cross-context rule: no target number can be marketed as measured until benchmark evidence includes context/tier/OS/arch labels.
27. Cross-context rule: audit seal p99 must be reported alongside read p99.
28. Cross-context rule: HSM p99 must be separated from secret resolve p99.
29. Cross-context rule: cache hit ratio must be reported and cannot be assumed.
30. Cross-context rule: capacity ceilings must include namespace count, secret count, version count, and audit backlog.

## §5 Comparison narrative

1. Read throughput: demo_trial targets are below AWS and Google public access quota classes and are realistic planning values for early cells.
2. Read throughput: paid targets are below AWS `GetSecretValue` default quota and below Google's 1,500 access rps project quota except Oyatie provider context.
3. Read throughput: paid targets approach AWS quota class in controlled contexts and exceed Google default project access quota in Oyatie-managed contexts.
4. Read throughput: paid provider-context target exceeds AWS default per-Region `GetSecretValue` quota and must be treated as a future measured hyperscaler-bar claim.
5. Write throughput: demo_trial/paid targets are above Google default write rps only in some rows; those rows require local OpenBao architecture rather than direct Google Secret Manager parity.
6. Write throughput: AWS control-plane mutation quota is 50 rps for many combined mutating APIs, so Oyatie paid/paid write targets require local infrastructure and measured proof.
7. Rotation throughput: AWS quota allows 50 `RotateSecret`/`CancelRotateSecret` rps, but practical rotation cadence guidance warns against frequent value updates.
8. Rotation throughput: Oyatie targets should be interpreted as scheduler capacity across tenants, not repeated rotation of one secret.
9. Secret size: AWS 65,536 bytes and Vault 1 MiB storage defaults set comparison bounds; Oyatie must publish a payload size limit.
10. Version count: AWS 100 versions per secret is a clear public number; Oyatie must publish version retention per tier.
11. Lease count: Vault advisory lease count 256,000 is relevant only if Oyatie implements dynamic leases.
12. Audit behavior: Vault's audit-device fail-closed behavior is a stronger operational comparator than AWS/GCP management-event logging.
13. Audit behavior: Oyatie's audit-chain target could be ahead if it proves sealed audit rows with Merkle/Ed25519 evidence.
14. Regional behavior: Google regional endpoint semantics are a strong comparator for Oyatie data-residency endpoints.
15. Replication behavior: AWS replica promotion and Vault DR promotion are both missing from Oyatie docs.
16. OCI demo_trial tenant_class: no direct AWS/GCP/HashiCorp counterpart exists; it is an Oyatie-specific cost target.
17. OCI demo_trial tenant_class: the current service docs do not meet the target because no Always Free module or tenant_class note exists.
18. Latency: the PRD target of cache hit p99 <=10ms is compatible with paid but too strict for all demo_trial contexts.
19. Latency: the OpenSLO target p99 <=100ms is compatible with demo_trial/paid cache miss, not paid cache-hit goals.
20. Conclusion: current numbers should become tiered SLOs, not one global SLO.
21. Classification for demo_trial: catch-up against AWS/GCP public quota surface, ahead only on zero-raw-secret doctrine.
22. Classification for paid: catch-up on quotas and deployment evidence, partial parity on read scale.
23. Classification for paid: target parity in Oyatie-managed contexts, catch-up in customer/guest contexts.
24. Classification for paid: hyperscaler-bar target, not a current claim.
25. Stop condition: measured evidence must replace this target document before any launch or counterpart-performance claim.
