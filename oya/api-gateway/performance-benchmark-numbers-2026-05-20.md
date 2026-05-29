# api-gateway performance benchmark numbers - 2026-05-20

Target µservice: `api-gateway`.
Counterpart set: AWS API Gateway, Kong Gateway, Apigee.
Doctrine: single industry-leader target set with deployment-context overlays.
Doctrine: tenant_class overlays use `{demo_trial, paid, revenue_share}`.
Doctrine: no retired commercial-level headings, rows, or segmentation are used.
Method disclosure: this is a target-and-comparison document, not fresh load-test output from a live Oyatie cluster.
Method disclosure: numbers labeled `source` come from local or public source material.
Method disclosure: numbers labeled `estimated from` are derived from public quotas or local capacity models rather than measured live throughput.
Method disclosure: numbers labeled `Oyatie target` are acceptance targets for future benchmark harnesses.
Method disclosure: context overlays are capacity and operating-envelope overlays only; they do not lower the correctness or security quality bar.

## Citation Anchor Block

Anchor 1: local PRD sets availability, p99 added latency, throughput, JWKS refresh, WAF update, and audit success targets at `PRD.md:67-72`.
Anchor 2: local capacity model sets handshake, connection, HTTP RPS, QUIC PPS, Cedar eval, and rate-limit lookup capacity at `capacity-model.md:12-18`.
Anchor 3: local SLO/ADR conflict exists between `slos/edge-latency-p99.openslo.yaml:13-15` and `decisions/ADR-MS-001-api-gateway.md:75-77`.
Anchor 4: AWS API Gateway quota source is `https://docs.aws.amazon.com/apigateway/latest/developerguide/limits.html`.
Anchor 5: AWS REST API execution quota source is `https://docs.aws.amazon.com/apigateway/latest/developerguide/api-gateway-execution-service-limits-table.html`.
Anchor 6: Kong benchmark source is `https://developer.konghq.com/gateway/performance/benchmarks/`.
Anchor 7: Apigee limits source is `https://cloud.google.com/apigee/docs/api-platform/reference/limits`.
Anchor 8: OCI Always Free budget source is memory `feedback_oci_always_free_maximization_2026_05_20.md:14-55`.
Anchor 9: tenant_class replacement source is memory `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:128-143`.

## §1 Methodology

Benchmark dimension 01: edge-added latency p50.
Benchmark dimension 02: edge-added latency p95.
Benchmark dimension 03: edge-added latency p99.
Benchmark dimension 04: full inspected request latency p99.
Benchmark dimension 05: no-policy proxy throughput.
Benchmark dimension 06: route lookup throughput with 100 routes.
Benchmark dimension 07: rate-limit-only throughput.
Benchmark dimension 08: rate-limit plus key/auth throughput.
Benchmark dimension 09: TLS 1.3 handshake capacity.
Benchmark dimension 10: PQC handshake surcharge.
Benchmark dimension 11: sustained concurrent connections.
Benchmark dimension 12: HTTP request throughput per cell.
Benchmark dimension 13: global aggregate request throughput.
Benchmark dimension 14: QUIC packet processing.
Benchmark dimension 15: Cedar evaluation throughput.
Benchmark dimension 16: rate-limit lookup throughput.
Benchmark dimension 17: WAF rule-update propagation.
Benchmark dimension 18: JWKS refresh propagation.
Benchmark dimension 19: audit event delivery success.
Benchmark dimension 20: route/control-plane mutation latency.
Benchmark dimension 21: API import size and count limits.
Benchmark dimension 22: payload size limit.
Benchmark dimension 23: header size limit.
Benchmark dimension 24: cache/key material limits.
Benchmark dimension 25: deployment-context capacity envelope.
Benchmark dimension 26: tenant_class usage envelope.

Test workload 01: simple HTTPS proxy, no policy plugins, 1 route.
Test workload 02: simple HTTPS proxy, no policy plugins, 100 routes.
Test workload 03: rate-limit check, no authentication, 1 route.
Test workload 04: rate-limit check, no authentication, 100 routes.
Test workload 05: rate-limit plus key/auth check, 1 route and 1 consumer.
Test workload 06: rate-limit plus key/auth check, 100 routes and 100 consumers.
Test workload 07: full Oyatie edge inspection: TLS, route admission, Cedar, WAF, abuse-signal, rate limit, audit emit.
Test workload 08: migration import workload for AWS/Kong/Apigee route and policy bundles.
Test workload 09: control-plane mutation workload for route/canary/domain/certificate changes.
Test workload 10: failure workload for JWKS refresh, WAF update, route deny spike, and regional failover.

Operating system disclosure: current api-gateway artifacts do not include `supported-oses.json`.
Operating system disclosure: benchmark targets assume Linux server runtime unless the missing OS manifest later expands or narrows runtime placement.
Architecture disclosure: target numbers assume x86_64 and aarch64 server-class nodes are both benchmarked before portability claims.
Architecture disclosure: OCI Always Free overlay assumes ARM Ampere A1-style free compute constraints unless the canonical OCI profile changes.
Deployment context disclosure: six contexts are considered because canonical docs require api-gateway-class services to support all six.
Deployment context disclosure: missing context IaC is a P1 coherence gap recorded in `coherence-audit-2026-05-20.md`.
Tenant_class disclosure: `demo_trial` caps usage and may use OCI Always Free profile.
Tenant_class disclosure: `paid` scales by per-seat license plus usage-based billing and can receive contractual SLOs.
Tenant_class disclosure: `revenue_share` runs at-cost or zero-margin substrate sized to revenue-share economics.
Tenant_class disclosure: none of the three classes lowers protocol correctness, security checks, audit obligation, or policy enforcement.

Method limitation 01: AWS API Gateway publishes quotas and limits, not a single comparable public p99 benchmark table.
Method limitation 02: Apigee publishes configuration and platform limits, not a simple per-proxy p99 benchmark table.
Method limitation 03: Kong publishes public gateway benchmark tables, so Kong is the primary latency/throughput numeric comparator.
Method limitation 04: local api-gateway capacity numbers exist but are targets/models, not current measured harness output.
Method limitation 05: existing local benchmark docs use retired commercial terminology, so this report re-expresses targets without that segmentation.
Method limitation 06: live validation must later run on each deployment context with fixed route count, policy set, request body, response body, TLS mode, and observability settings.

## §2 Counterpart Numbers

### §2.1 AWS API Gateway Numbers

AWS number 01: 10,000 requests per second account-level throttle quota per Region across HTTP, REST, WebSocket, and callback APIs; source: AWS API Gateway quotas.
AWS number 02: 5,000 request token bucket maximum burst capacity for the default account-level throttle quota; source: AWS API Gateway quotas.
AWS number 03: 2,500 requests per second default throttle quota in named smaller Regions; source: AWS API Gateway quotas.
AWS number 04: 1,250 request burst quota in the named smaller Regions; source: AWS API Gateway quotas.
AWS number 05: 250,000 requests per second portal throttle quota without access control; source: AWS API Gateway quotas.
AWS number 06: 10,000 requests per second portal throttle quota with access control; source: AWS API Gateway quotas.
AWS number 07: 10 control-plane operations per second with burst 40 for total operations; source: AWS API Gateway quotas.
AWS number 08: 600 Regional REST APIs per Region; source: AWS REST API execution quotas.
AWS number 09: 120 edge-optimized REST APIs per Region; source: AWS REST API execution quotas.
AWS number 10: 600 private REST APIs per account per Region; source: AWS REST API execution quotas.
AWS number 11: 10,000 API keys per account per Region; source: AWS REST API execution quotas.
AWS number 12: 300 usage plans per account per Region; source: AWS REST API execution quotas.
AWS number 13: 10 usage plans per API key; source: AWS REST API execution quotas.
AWS number 14: 10 MB REST API payload size limit; source: AWS REST API execution quotas.
AWS number 15: 10,240 byte total header size for public REST APIs; source: AWS REST API execution quotas.
AWS number 16: 8,000 byte total header size for private REST APIs; source: AWS REST API execution quotas.
AWS number 17: 50 ms to 29 s integration timeout for REST API integrations; source: AWS REST API execution quotas.
AWS number 18: 300 second default API caching TTL, configurable 0 to 3600 seconds; source: AWS REST API execution quotas.
AWS number 19: 1,000 certificates up to 1 MB truststore limit; source: AWS REST API execution quotas.
AWS number 20: 6 MB maximum imported API definition file size; source: AWS REST API execution quotas.
AWS interpretation: AWS is the quota baseline, not the latency leader in this document.
AWS interpretation: Oyatie targets should exceed AWS default per-Region RPS when provisioned in paid elastic contexts.
AWS interpretation: Oyatie demo_trial must instead use explicit hard usage caps rather than pretending to match AWS regional quotas on free compute.

### §2.2 Kong Gateway Numbers

Kong number 01: 140,382 RPS with p99 5.24 ms and p95 3.55 ms for proxy with no plugins, 1 route, 0 consumers; source: Kong Gateway 3.14 benchmark table.
Kong number 02: 137,545.8 RPS with p99 5.48 ms and p95 3.06 ms for proxy with no plugins, 100 routes; source: Kong Gateway 3.14 benchmark table.
Kong number 03: 116,084.4 RPS with p99 7.54 ms and p95 3.62 ms for rate limit and no auth, 1 route; source: Kong Gateway 3.14 benchmark table.
Kong number 04: 113,706.4 RPS with p99 7.84 ms and p95 3.64 ms for rate limit and no auth, 100 routes; source: Kong Gateway 3.14 benchmark table.
Kong number 05: 99,512.6 RPS with p99 8.73 ms and p95 4.40 ms for rate limit and key auth, 1 route and 1 consumer; source: Kong Gateway 3.14 benchmark table.
Kong number 06: 95,660.8 RPS with p99 9.05 ms and p95 4.59 ms for rate limit and key auth, 100 routes and 100 consumers; source: Kong Gateway 3.14 benchmark table.
Kong number 07: 95,605.5 RPS with p99 9.30 ms and p95 4.53 ms for rate limit and basic auth, 1 route and 1 consumer; source: Kong Gateway 3.14 benchmark table.
Kong number 08: 91,423.9 RPS with p99 9.32 ms and p95 4.77 ms for rate limit and basic auth, 100 routes and 100 consumers; source: Kong Gateway 3.14 benchmark table.
Kong number 09: benchmark method uses Kubernetes on AWS infrastructure; source: Kong benchmark method.
Kong number 10: benchmark method uses HTTPS only; source: Kong benchmark method.
Kong number 11: each test case runs five times for 15 minutes; source: Kong benchmark method.
Kong number 12: benchmark node is a dedicated c5.4xlarge with 16 vCPU and 16 worker processes; source: Kong test environment and configuration.
Kong number 13: 100-route scenarios reduce no-plugin throughput by roughly 2.0 percent versus 1-route in the 3.14 table; estimated from Kong numbers 01-02.
Kong number 14: rate-limit-only 100-route throughput is roughly 19 percent lower than no-plugin 100-route throughput; estimated from Kong numbers 02 and 04.
Kong number 15: key-auth plus rate-limit 100-route throughput is roughly 31 percent lower than no-plugin 100-route throughput; estimated from Kong numbers 02 and 06.
Kong interpretation: Kong is the primary public p99/RPS comparator.
Kong interpretation: Oyatie must beat or match Kong on simple proxy path while accepting explicit overhead for stronger WAF/Cedar/abuse/PQC checks.
Kong interpretation: full Oyatie edge inspection needs its own benchmark category rather than pretending to be a no-plugin proxy.

### §2.3 Apigee Numbers

Apigee number 01: 6,000 user/service-account/UI initiated Apigee API calls per minute; source: Apigee limits.
Apigee number 02: 1,000 synchronizer-initiated calls per minute for hybrid; source: Apigee limits.
Apigee number 03: 15 MB API proxy or shared flow bundle size; source: Apigee limits.
Apigee number 04: 250 API proxy revisions retained in history; source: Apigee limits.
Apigee number 05: 6,000 deployed API proxies and shared flows per organization for Apigee; source: Apigee limits.
Apigee number 06: 6,000 deployed proxies per environment for Apigee; source: Apigee limits.
Apigee number 07: 75 deployed shared flows per environment for Apigee; source: Apigee limits.
Apigee number 08: 6,000 proxy deployment units per instance; source: Apigee limits.
Apigee number 09: 200 maximum flows per API proxy; source: Apigee limits.
Apigee number 10: SpikeArrest per-second maximum rate 4,000 and per-minute maximum 240,000; source: Apigee limits.
Apigee number 11: 1 million developers per organization; source: Apigee limits.
Apigee number 12: 10 API keys per app; source: Apigee limits.
Apigee number 13: 100 apps per developer; source: Apigee limits.
Apigee number 14: 1 million apps per organization; source: Apigee limits.
Apigee number 15: 5,000 API products per organization; source: Apigee limits.
Apigee number 16: 30 MB buffered request/response size; source: Apigee limits.
Apigee number 17: 10 KB API proxy request URL size; source: Apigee limits.
Apigee number 18: 60 KB request header size and 60 KB response header size; source: Apigee limits.
Apigee number 19: 300 second target connection timeout; source: Apigee limits.
Apigee number 20: 100 calls per minute to Analytics Metrics API; source: Apigee limits.
Apigee number 21: 300 asynchronous query API calls per day; source: Apigee limits.
Apigee number 22: 6 weeks API monitoring retention; source: Apigee limits.
Apigee interpretation: Apigee is the API-management scale and product-governance comparator.
Apigee interpretation: Oyatie must decide whether api-gateway owns API products, apps, developer portal, and analytics or only enforces artifacts from another service.
Apigee interpretation: performance targets alone are not sufficient for Apigee parity.

## §3 Oyatie Target Numbers

Oyatie target 01: simple HTTPS proxy p50 added latency <= 1.5 ms.
Oyatie target 02: simple HTTPS proxy p95 added latency <= 4 ms.
Oyatie target 03: simple HTTPS proxy p99 added latency <= 5 ms.
Oyatie target 04: 100-route HTTPS proxy p99 added latency <= 6 ms.
Oyatie target 05: rate-limit-only p99 added latency <= 8 ms.
Oyatie target 06: rate-limit plus key/auth p99 added latency <= 9 ms.
Oyatie target 07: full edge inspection p50 added latency <= 4 ms.
Oyatie target 08: full edge inspection p95 added latency <= 12 ms.
Oyatie target 09: full edge inspection p99 added latency <= 22 ms.
Oyatie target 10: no-policy proxy throughput >= 150,000 RPS per tuned 16-vCPU gateway node.
Oyatie target 11: 100-route no-policy throughput >= 140,000 RPS per tuned 16-vCPU gateway node.
Oyatie target 12: rate-limit-only throughput >= 120,000 RPS per tuned 16-vCPU gateway node.
Oyatie target 13: rate-limit plus key/auth throughput >= 105,000 RPS per tuned 16-vCPU gateway node.
Oyatie target 14: full edge inspection throughput >= 50,000 RPS per replica or equivalent node, matching `PRD.md:69`.
Oyatie target 15: sustained HTTP throughput >= 250,000 RPS per cell, matching `capacity-model.md:14`.
Oyatie target 16: global aggregate throughput target >= 6,000,000 RPS, matching `capacity-model.md:46`.
Oyatie target 17: TLS handshakes >= 50,000 per second per cell, matching `capacity-model.md:12`.
Oyatie target 18: sustained concurrent connections >= 5,000,000 per cell, matching `capacity-model.md:13`.
Oyatie target 19: QUIC packet processing >= 4,000,000 packets per second per cell, matching `capacity-model.md:15`.
Oyatie target 20: Cedar evaluations >= 500,000 per second per cell, matching `capacity-model.md:16`.
Oyatie target 21: rate-limit lookups >= 1,000,000 per second per cell, matching `capacity-model.md:18`.
Oyatie target 22: JWKS propagation <= 60 seconds, matching `PRD.md:70`.
Oyatie target 23: WAF rule propagation <= 5 minutes, matching `PRD.md:71`.
Oyatie target 24: audit delivery success >= 99.9 percent, matching `PRD.md:72`.
Oyatie target 25: public ingress availability >= 99.99 percent, matching `PRD.md:67`.
Oyatie target 26: route/canary/domain control-plane mutation p95 <= 2 seconds inside one region.
Oyatie target 27: route/canary/domain control-plane mutation p99 <= 10 seconds across three regions.
Oyatie target 28: migration import validation for 10,000 routes completes <= 10 minutes on paid elastic substrate.
Oyatie target 29: policy simulation for 10,000 route-policy pairs completes <= 5 minutes on paid elastic substrate.
Oyatie target 30: synthetic probe detection-to-alert p95 <= 60 seconds.
Oyatie target 31: edge shed decision propagation p99 <= 30 seconds inside one cell.
Oyatie target 32: regional failover route convergence p99 <= 120 seconds for public-cloud contexts.

### §3.1 Deployment-Context Overlays

Context overlay 01: `oyatie-public-cloud` should meet all canonical targets through elastic cell scaling.
Context overlay 02: `oyatie-public-cloud` should enforce p99 full edge inspection <= 22 ms under normal load.
Context overlay 03: `oyatie-public-cloud` should support >= 250,000 RPS per cell and >= 6,000,000 RPS global aggregate when provisioned.
Context overlay 04: `guest-on-aws` should meet canonical targets when tenant-provisioned AWS capacity matches the reference cell shape.
Context overlay 05: `guest-on-aws` should report an AWS-specific quota failure when account-level API or load-balancer quotas constrain traffic.
Context overlay 06: `guest-on-oci` paid capacity should meet canonical targets when provisioned beyond free-profile constraints.
Context overlay 07: `guest-on-oci` OCI Always Free profile should cap sustained throughput to a conservative demo envelope rather than lowering correctness.
Context overlay 08: OCI Always Free profile target is 300 sustained RPS and 1,000 RPS short burst per demo_trial tenant until live tests prove a higher envelope.
Context overlay 09: OCI Always Free profile full edge inspection p99 target is <= 75 ms at the capped load.
Context overlay 10: OCI Always Free profile should cap retained routes to 25 active routes per tenant until capacity tests prove more.
Context overlay 11: OCI Always Free profile should cap concurrent connections to 2,000 per tenant until capacity tests prove more.
Context overlay 12: `on-prem` should meet canonical targets only when facility hardware, network, HSM, and observability nodes are sized to the reference cell.
Context overlay 13: `on-prem` must publish facility-specific exception records instead of silently relaxing targets.
Context overlay 14: `colo` should meet canonical targets when cross-connect, load-balancer, and hardware shape match the reference cell.
Context overlay 15: `colo` should measure carrier routing and TLS handshake variance separately from gateway-added latency.
Context overlay 16: `oyatie-as-cloud-provider` should meet or exceed `oyatie-public-cloud` targets because Oyatie controls substrate placement.
Context overlay 17: all six contexts must run the same correctness/security path for route admission, Cedar, WAF, rate limits, audit, and auth handoff.
Context overlay 18: all six contexts need OpenTofu modules before these targets can be claimed as deployable.

### §3.2 Tenant_Class Overlays

Tenant overlay 01: `demo_trial` uses hard usage caps and best-effort SLO.
Tenant overlay 02: `demo_trial` may run on OCI Always Free profile where context permits.
Tenant overlay 03: `demo_trial` target cap is 300 sustained RPS and 1,000 RPS short burst on OCI Always Free profile.
Tenant overlay 04: `demo_trial` may have shorter log retention and fewer active routes, but not weaker security checks.
Tenant overlay 05: `demo_trial` does not receive compliance packs or BYOK unless upgraded by governance exception.
Tenant overlay 06: `paid` scales with per-seat license plus usage-based billing.
Tenant overlay 07: `paid` receives contractual SLO when provisioned context and plan meet the target shape.
Tenant overlay 08: `paid` may use BYOK and compliance packs when policy prerequisites are met.
Tenant overlay 09: `paid` can scale from single-cell to multi-cell by payment and capacity plan.
Tenant overlay 10: `revenue_share` runs at-cost or zero-margin substrate sized to expected gross revenue.
Tenant overlay 11: `revenue_share` should use the same latency and security target set as paid when provisioned.
Tenant overlay 12: `revenue_share` may enforce commercial safety caps when revenue volume does not justify elastic expansion.
Tenant overlay 13: all tenant classes emit the same required audit events.
Tenant overlay 14: all tenant classes enforce route authorization and abuse controls.
Tenant overlay 15: all tenant classes use tenant_class from trusted authority-chain context, not from client-supplied request body.
Tenant overlay 16: tenant_class labels must be low-cardinality and privacy-reviewed before appearing in metrics.

### §3.3 Target Consistency Repairs

Repair 01: treat `PRD.md:67-69` as the stronger current latency anchor for edge-added latency.
Repair 02: treat `slos/edge-latency-p99.openslo.yaml:13-15` 500 ms wording as stale or end-to-end-only until clarified.
Repair 03: treat `decisions/ADR-MS-001-api-gateway.md:75-77` 50/200/500 ms as stale for edge-added latency.
Repair 04: keep full edge inspection p99 at 22 ms because the existing local benchmark narrative uses that as full inspected target.
Repair 05: avoid mixing edge-added latency with upstream/application latency.
Repair 06: benchmark no-policy proxy, route+rate, route+rate+auth, and full inspection as separate workloads.
Repair 07: benchmark per-node, per-replica, per-cell, and global aggregate separately.
Repair 08: benchmark OCI Always Free profile separately and never present it as equivalent to paid elastic contexts.
Repair 09: publish the test harness route count, request size, response size, TLS mode, policy set, and observability settings with every result.
Repair 10: record whether the benchmark used Rust implementation, Envoy config, Wasm filters, Cedar engine, and Valkey rate-limit backend.

## §4 Comparison Narrative

Comparison 01: simple proxy p99 target <= 5 ms is ahead/parity versus Kong 3.14 no-plugin p99 5.24 ms.
Comparison 02: 100-route proxy p99 target <= 6 ms is near parity versus Kong 3.14 100-route no-plugin p99 5.48 ms.
Comparison 03: rate-limit-only p99 target <= 8 ms is near parity versus Kong 3.14 100-route rate-limit p99 7.84 ms.
Comparison 04: rate-limit plus key/auth p99 target <= 9 ms is near parity versus Kong 3.14 100-route key-auth p99 9.05 ms.
Comparison 05: full inspection p99 target <= 22 ms is a catch-up-plus-security target because it includes WAF/Cedar/abuse/audit work not present in Kong no-plugin rows.
Comparison 06: no-policy throughput target >= 150,000 RPS is ahead of Kong 3.14 no-plugin 1-route 140,382 RPS.
Comparison 07: 100-route no-policy throughput target >= 140,000 RPS is ahead/parity versus Kong 3.14 137,545.8 RPS.
Comparison 08: rate-limit-only throughput target >= 120,000 RPS is ahead of Kong 3.14 116,084.4 RPS for 1 route and ahead of 113,706.4 RPS for 100 routes.
Comparison 09: rate-limit plus key/auth target >= 105,000 RPS is ahead of Kong 3.14 99,512.6 RPS for 1 route and 95,660.8 RPS for 100 routes.
Comparison 10: full inspection per-replica target >= 50,000 RPS is behind simple Kong proxy numbers but reflects heavier policy/security work.
Comparison 11: per-cell target >= 250,000 RPS exceeds AWS default account-level regional quota of 10,000 RPS and Apigee SpikeArrest 4,000 per-second policy limit.
Comparison 12: global aggregate target >= 6,000,000 RPS exceeds public default quotas from AWS and Apigee, but requires Oyatie-controlled horizontal cell scale.
Comparison 13: AWS custom-domain/API-key/usage-plan numbers expose management-scale requirements that api-gateway contracts do not yet meet.
Comparison 14: Apigee API product/app/developer numbers expose governance-scale requirements that api-gateway contracts do not yet meet.
Comparison 15: OCI Always Free profile cap is intentionally below AWS/Kong/Apigee production numbers because it is a demo_trial infrastructure envelope.
Comparison 16: paid tenant_class should meet canonical targets whenever substrate is paid and provisioned accordingly.
Comparison 17: revenue_share tenant_class should meet canonical targets when expected revenue supports the at-cost substrate envelope.
Comparison 18: current service artifacts are strongest on target ambition and weakest on live benchmark proof.
Comparison 19: current service artifacts must not reuse the old benchmark document's retired commercial segmentation.
Comparison 20: current service artifacts need one fresh benchmark harness before any completion claim.

## Required Live Harness Evidence

Harness evidence 01: fixed version of Rust gateway crate.
Harness evidence 02: fixed Envoy build and config digest.
Harness evidence 03: fixed Wasm filter digest.
Harness evidence 04: fixed Cedar policy digest.
Harness evidence 05: fixed Valkey backend topology.
Harness evidence 06: fixed route count and route distribution.
Harness evidence 07: fixed consumer count.
Harness evidence 08: fixed request body size.
Harness evidence 09: fixed response body size.
Harness evidence 10: fixed TLS mode.
Harness evidence 11: fixed HTTP/2 versus HTTP/3 mode.
Harness evidence 12: fixed observability/exporter configuration.
Harness evidence 13: fixed audit sink configuration.
Harness evidence 14: fixed deployment context.
Harness evidence 15: fixed tenant_class overlay.
Harness evidence 16: fixed hardware shape.
Harness evidence 17: warm-up window.
Harness evidence 18: run duration.
Harness evidence 19: sample count.
Harness evidence 20: error budget accounting.
Harness evidence 21: p50/p95/p99 latency output.
Harness evidence 22: RPS output.
Harness evidence 23: CPU and memory utilization.
Harness evidence 24: packet loss and retransmit counters.
Harness evidence 25: tail-latency regression threshold.
Harness evidence 26: comparison to previous benchmark artifact.
Harness evidence 27: signed output artifact.
Harness evidence 28: OpenTofu context path used for deployment.
Harness evidence 29: supported OS/runtime manifest version.
Harness evidence 30: confirmation that no retired commercial-level dimension is used.

## Stop Decision

Stop decision 01: this document defines benchmark targets and counterpart comparisons.
Stop decision 02: it does not claim the current api-gateway implementation has achieved the targets.
Stop decision 03: live achievement requires Rust implementation, OpenTofu context modules, supported OS manifest, and benchmark harness evidence.
Stop decision 04: the current service can claim target intent but not measured performance completion.
Stop decision 05: the next benchmark artifact must report measured numbers in the same workload taxonomy used here.
