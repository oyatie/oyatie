# cloud-network-dns performance benchmark numbers — 2026-05-20

AGENT CLASS: microservice-ownership-coherence-audit-agent
AGENT SLUG: codex-cloud-network-dns-audit
MODE: audit-only
BUNDLE: cloud-network-dns-performance-benchmarks-2026-05-20
SCOPE: `/Users/jasonlee/oyatie/network/dns/`

## Header citation anchors

1. ADR-0328 §D-15..§D-20 supplies the required deployment-context, OpenTofu, OS, Rust, OCI, and benchmark disclosure constraints; read anchors include `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-1815`, `:2241-2365`, `:3140-3235`, and `:3756-4151`.
2. `specs/master-plan-sequencing.json:704-867` supplies the six contexts, state backend map, Tier-1 OS list, Rust build invocation, and OCI Always Free resource envelope.
3. Service-local performance claims read: `microservices/cloud-network-dns/retired tenant_class adoption artifact:24-25`, `:42-43`, `:60-61`, `:78-79`, and `network/dns/benchmarks/cloud-network-dns-vs-route53-vs-cloud-dns-vs-cloudflare-vs-ns1.md:3-19`.
4. Service-local architecture-equivalent evidence read: `network/dns/reference-implementations/provision-zone-dnssec-and-geo-routing-rust-sdk.md:148-217` for example query latency logging and failover confirmation.
5. Documentation-rigor performance/capacity bar: `docs/standards/documentation-rigor.md:143-156` requires capacity math, failure modes, observability hooks, rollback, multi-region, sovereign, and versioning rigor.

## Methodology disclosure

These are target numbers plus provenance, not measured Oyatie production benchmarks.
The existing benchmark artifact claims measurements across 2026-04-26 to 2026-05-12, but its cited evidence path is absent (`network/dns/benchmarks/cloud-network-dns-vs-route53-vs-cloud-dns-vs-cloudflare-vs-ns1.md:3-5`, `:119`).
Measured benchmarks must be added in a build phase with signed evidence, raw probe logs, OS/arch/context disclosure, and reproducibility commands that use Rust/Cargo/OpenTofu rather than `make`.
Counterpart figures below are drawn from public documentation when public providers publish limits, intervals, regions, protocols, or topology numbers; latency numbers that are not officially published are labeled as target estimates or service-local unverified claims.
DNSPerf is used only as methodology provenance for external DNS performance comparability because its public page states providers are tested every minute from 200+ global locations with a one-second timeout and hourly public updates (`https://www.dnsperf.com/`).

## §1 Methodology

1. Benchmark dimension: authoritative DNS query latency p50, p95, and p99 for warm-cache A/AAAA/TXT/CAA/MX lookups.
2. Benchmark dimension: DNSSEC-signed authoritative query latency p50, p95, and p99 for ECDSAP256SHA256 and Ed25519 where supported.
3. Benchmark dimension: record mutation control-plane latency p50, p95, and p99 for zone create, record create, record update, and record rollback.
4. Benchmark dimension: DNSSEC key operation latency for enable, ZSK rotation, KSK rotation, DS export, and HSM signing throughput.
5. Benchmark dimension: health-check failover RTO from endpoint failure to DNS answer removal.
6. Benchmark dimension: health-check probe fanout and probe interval by tier.
7. Benchmark dimension: sustained QPS per tenant and burst QPS per tenant.
8. Benchmark dimension: record count per tenant/zone.
9. Benchmark dimension: zone count per tenant.
10. Benchmark dimension: query telemetry export lag p95 and p99.
11. Benchmark dimension: anycast withdraw RTO for paid/provider-mode.
12. Benchmark dimension: resolver encrypted transport overhead for DoH-H2, DoH-H3, DoT, DoQ, and ODoH.
13. Benchmark dimension: OpenTofu deployment time p95 per context.
14. Benchmark dimension: OCI Always Free resource ceiling enforcement for demo_trial on guest-on-oci.
15. Benchmark dimension: cross-OS build/test lane pass time on Tier-1 OSes.
16. Workload A: single unsigned A record authoritative query from six global regions.
17. Workload B: signed-zone SOA/A query with DNSSEC validation and NSEC3 negative proof.
18. Workload C: weighted/geo/latency routing policy query with three regional answer sets.
19. Workload D: health-check endpoint failure with TTL 60 and three consecutive failed probes.
20. Workload E: zone import with 5,000 records and 50 health checks from Route 53/NS1 style export.
21. Workload F: query log stream under sustained tenant load.
22. Workload G: OpenTofu context plan/apply for one tenant zone and health-check fleet.
23. Workload H: demo_trial OCI Always Free tenant with 25 records and 100 QPS sustained.
24. OS disclosure requirement: every measured result must state OS, OS version, arch, kernel, libc, DNS engine, and HSM/SoftHSM mode.
25. Arch disclosure requirement: `linux/amd64`, `linux/arm64`, `darwin/arm64-m5+`, and Tier-2 test-only architectures must be separated.
26. Deployment-context disclosure requirement: all six contexts must be reported separately, per ADR-0328 §D-20.152 style guidance.
27. Tenant-class disclosure requirement: demo/sandbox/trial/dev/paid/self-hosted/sovereign must be recorded.
28. Data-source disclosure: target values from service-local tenant_class matrix are not evidence of measured performance.
29. Counterpart disclosure: public docs often publish limits and intervals but not p50/p95/p99 latency; do not treat absent official latency as a vendor weakness.
30. Stop condition for future benchmark promotion: signed raw data, reproducible Rust benchmark harness, OpenTofu environment manifest, and lineaged evidence path.

## §2 Counterpart numbers from public docs and service-local unverified claims

| Counterpart | Number | Dimension | Source and status |
|---|---:|---|---|
| AWS Route 53 | 200+ | Authoritative DNS PoP locations | Official Route 53 concepts state the data plane runs across over 200 PoPs |
| AWS Route 53 | 5 req/s | Account-level Route 53 API throttle | Official Route 53 quotas |
| AWS Route 53 | 5 req/s/Region | VPC Resolver API throttle | Official Route 53 quotas |
| AWS Route 53 | 1 request / 2 sec | `CreateHealthCheck` account throttle | Official Route 53 quotas |
| AWS Route 53 | 200 | Active health checks default quota | Official Route 53 quota documentation snippet |
| AWS Route 53 | 255 | Child health checks per calculated health check | Official Route 53 quotas |
| AWS Route 53 | 50 | Traffic policies per account | Official Route 53 quotas |
| AWS Route 53 | 1,000 | Traffic policy versions per policy | Official Route 53 quotas |
| AWS Route 53 | 5 | Traffic policy records per account default | Official Route 53 quotas |
| AWS Route 53 | 5 | Route 53 Profiles per account per Region | Official Route 53 quotas |
| AWS Route 53 | 1,000 | VPCs per Route 53 Profile | Official Route 53 quotas |
| AWS Route 53 | 5,000 | Private hosted zones per Profile | Official Route 53 quotas |
| AWS Route 53 | 30 sec | Service-local failover comparison for 10s interval x 3 failures | Unverified local benchmark doc `benchmarks/...:35-46` |
| AWS Route 53 | 18.6 ms p95 | Service-local warm authoritative comparison | Unverified local benchmark doc `benchmarks/...:7-18` |
| AWS Route 53 | 22.6 ms p95 | Service-local signed-zone comparison | Unverified local benchmark doc `benchmarks/...:23-33` |
| Google Cloud DNS | 3 | Source regions selected for external endpoint health checks | Official Cloud DNS routing policies |
| Google Cloud DNS | 3 | Health-check probers per selected region | Official Cloud DNS routing policies |
| Google Cloud DNS | 9 | Total probers for three selected regions | Official Cloud DNS routing policies |
| Google Cloud DNS | 30 sec | Minimum external endpoint health-check interval | Official Cloud DNS routing policies |
| Google Cloud DNS | 300 sec | Maximum external endpoint health-check interval | Official Cloud DNS routing policies |
| Google Cloud DNS | 3 | DNS policy families: server, response, routing | Official Cloud DNS policies overview |
| Google Cloud DNS | 3 | Health-check protocols: TCP, HTTP, HTTPS | Official Cloud DNS routing policies |
| Google Cloud DNS | 1 | Public-zone DNS forwarding support count | Official overview says public zones must be authoritative; forwarding public zones unsupported |
| Google Cloud DNS | 64:ff9b::/96 | DNS64 well-known prefix | Official Cloud DNS overview |
| Google Cloud DNS | 30 sec interval | Service-local comparison health-check minimum | Unverified local benchmark doc `benchmarks/...:35-46` |
| Google Cloud DNS | 24.6 ms p95 | Service-local warm authoritative comparison | Unverified local benchmark doc `benchmarks/...:7-18` |
| Google Cloud DNS | 26.8 ms p95 | Service-local signed-zone comparison | Unverified local benchmark doc `benchmarks/...:23-33` |
| Google Cloud DNS | 500 ms p99 | Oyatie cloud PRD control mutation target for VPC/LB/DNS create boundary | Local cloud PRD `docs/products/cloud/PRD.md:161-176` |
| Cloudflare DNS | hundreds | Cities for 1.1.1.1 resolver network | Official Cloudflare 1.1.1.1 docs |
| Cloudflare DNS | 443 | DoH port | Official Cloudflare DoH docs |
| Cloudflare DNS | 3 | DoH protocol families: HTTP, HTTP/2, HTTP/3 | Official Cloudflare DoH docs |
| Cloudflare DNS | 853 | DoT port | Official Cloudflare DoT docs |
| Cloudflare DNS | 2 | ODoH roles: proxy and target | Official Cloudflare ODoH docs |
| Cloudflare DNS | 30 | Max linked peers per zone for zone transfers | Official Cloudflare zone transfer docs |
| Cloudflare DNS | 13 | Health monitor regions in Cloudflare Load Balancing docs | Official Cloudflare Load Balancing monitor docs |
| Cloudflare DNS | 39 | Probes when all 13 regions send three probes each | Official Cloudflare Load Balancing monitor docs |
| Cloudflare DNS | 3 | Data centers per selected health monitor region | Official Cloudflare Load Balancing monitor docs |
| Cloudflare DNS | 15 sec interval | Service-local comparison for health monitor interval | Unverified local benchmark doc `benchmarks/...:35-46` |
| Cloudflare DNS | 4.2 ms p95 | Service-local warm authoritative/resolver comparison | Unverified local benchmark doc `benchmarks/...:7-18` |
| Cloudflare DNS | 5.2 ms p95 | Service-local signed-zone comparison | Unverified local benchmark doc `benchmarks/...:23-33` |
| DNSPerf methodology | 200+ | Global locations in public test methodology | DNSPerf public methodology |
| DNSPerf methodology | 1 sec | Timeout in public methodology | DNSPerf public methodology |
| DNSPerf methodology | 1 minute | Test cadence in public methodology | DNSPerf public methodology |
| DNSPerf methodology | 1 hour | Public data update cadence | DNSPerf public methodology |

## §3 Oyatie target numbers by tenant_class and deployment context

| Context | Tier | Authoritative p50/p95/p99 | Signed p50/p95/p99 | Sustained QPS | Burst QPS | Records/zones | Health interval/RTO | Deployment p95 | Notes |
|---|---|---|---|---:|---:|---|---|---|---|
| oyatie-public-cloud | demo_trial | 8/18/35 ms | no DNSSEC | 100/tenant | 500 | 25 records/shared subdomain | none | 8 min | public shared zone |
| oyatie-public-cloud | paid | 5/12/24 ms | 7/15/32 ms | 5k/tenant | 20k | 5 zones, 5k records/zone | 30s/90s | 12 min | dedicated zone |
| oyatie-public-cloud | paid | 2.4/6/14 ms | 3.4/7.5/16 ms | 50k/tenant | 200k | 50 zones, 100k records/zone | 10s/30s | 18 min | multi-region anycast |
| oyatie-public-cloud | paid | 1.2/3/7 ms | 2.4/5.5/12 ms | 500k/tenant | 2M | unlimited contractually, quota-managed | 1s/8s | 25 min | dedicated anycast |
| guest-on-aws | demo_trial | 10/22/45 ms | no DNSSEC | 100 | 400 | 25/shared | none | 10 min | backing VPC resources |
| guest-on-aws | paid | 6/15/32 ms | 9/20/42 ms | 4k | 15k | 5 zones, 5k/zone | 30s/90s | 16 min | AWS guest adapters |
| guest-on-aws | paid | 3/8/18 ms | 4.5/10/22 ms | 40k | 150k | 50 zones, 100k/zone | 10s/30s | 24 min | AWS network variance |
| guest-on-aws | paid | 1.8/4.5/10 ms | 3/7/15 ms | 300k | 1.2M | dedicated quota | 1s/10s | 35 min | provider-bound ceiling |
| guest-on-oci | demo_trial | 14/28/60 ms | no DNSSEC | 100 | 300 | 25/shared | none | 14 min | must fit Always Free |
| guest-on-oci | paid | 7/16/34 ms | 10/22/45 ms | 3k | 12k | 5 zones, 5k/zone | 30s/90s | 18 min | paid OCI baseline |
| guest-on-oci | paid | 3.5/8.5/20 ms | 5/11/25 ms | 30k | 120k | 50 zones, 100k/zone | 10s/30s | 28 min | paid OCI anycast |
| guest-on-oci | paid | 2/5/12 ms | 3.5/8/18 ms | 250k | 1M | dedicated quota | 1s/12s | 40 min | HSM and paid LB |
| on-prem | demo_trial | 12/30/70 ms | no DNSSEC | 100 | 250 | 25/shared | none | 20 min | small appliance |
| on-prem | paid | 8/20/45 ms | 12/28/60 ms | 2k | 8k | 5 zones, 5k/zone | 30s/120s | 30 min | customer network |
| on-prem | paid | 5/14/32 ms | 8/18/40 ms | 20k | 80k | 50 zones, 100k/zone | 10s/45s | 45 min | depends WAN |
| on-prem | paid | 3/8/20 ms | 5/12/28 ms | 150k | 600k | dedicated quota | 1s/15s | 60 min | air-gap capable |
| colo | demo_trial | 10/24/55 ms | no DNSSEC | 100 | 300 | 25/shared | none | 18 min | small colo footprint |
| colo | paid | 6/16/35 ms | 9/22/48 ms | 3k | 12k | 5 zones, 5k/zone | 30s/100s | 25 min | facility variance |
| colo | paid | 3/9/22 ms | 5/12/28 ms | 35k | 140k | 50 zones, 100k/zone | 10s/35s | 40 min | better BGP control |
| colo | paid | 1.5/4/10 ms | 3/7/15 ms | 300k | 1.5M | dedicated quota | 1s/8s | 55 min | dedicated anycast |
| oyatie-as-cloud-provider | demo_trial | 8/18/35 ms | no DNSSEC | 100 | 500 | 25/shared | none | 8 min | internal cloud substrate |
| oyatie-as-cloud-provider | paid | 5/12/24 ms | 7/15/32 ms | 5k | 20k | 5 zones, 5k/zone | 30s/90s | 12 min | tenant cloud product |
| oyatie-as-cloud-provider | paid | 2.2/5.5/13 ms | 3.2/7/15 ms | 60k | 240k | 50 zones, 100k/zone | 10s/30s | 18 min | internal cell control |
| oyatie-as-cloud-provider | paid | 1/2.8/6 ms | 2.2/5/11 ms | 750k | 3M | dedicated quota | 1s/8s | 25 min | hyperscaler target |

## §3.1 demo_trial target detail

1. demo_trial authoritative p95 target is 18 ms in managed/public/provider contexts, matching the service tenant_class matrix (`retired tenant_class adoption artifact:24`).
2. demo_trial sustained QPS target is 100/sec/tenant, matching the service tenant_class matrix (`retired tenant_class adoption artifact:25`).
3. demo_trial has no DNSSEC target, matching the tenant_class matrix (`retired tenant_class adoption artifact:18-20`).
4. demo_trial record cap is 25 records per tenant, matching the tenant_class matrix (`retired tenant_class adoption artifact:19`).
5. demo_trial transport target is UDP/TCP plus DoH, matching the tenant_class matrix (`retired tenant_class adoption artifact:23`).
6. demo_trial guest-on-OCI must fit 4 OCPU/24GB total, 200GB block, 10GB object/archive, 10Mbps LB, and 10TB egress from the OCI profile (`specs/master-plan-sequencing.json:857-867`).
7. demo_trial guest-on-OCI target uses lower burst QPS because the Always Free load balancer is capped at 10Mbps.
8. demo_trial on-prem target uses wider p99 because small appliances and local WAN links vary.
9. demo_trial has no health-check failover target because tenant_class matrix says no health checks (`retired tenant_class adoption artifact:22`).
10. demo_trial deployment p95 target must be measured by `tofu apply` after IaC exists; current number is a planning target.

## §3.2 paid target detail

1. paid authoritative p95 target is 12 ms in public/provider contexts, matching the tenant_class matrix (`retired tenant_class adoption artifact:42`).
2. paid sustained QPS target is 5,000/sec/tenant and burst 20,000/sec in managed contexts, matching the tenant_class matrix (`retired tenant_class adoption artifact:43`).
3. paid record target is 5 zones and 5,000 records per zone (`retired tenant_class adoption artifact:35-37`).
4. paid DNSSEC includes NSEC3, RSASHA256, ECDSAP256SHA256, ZSK 30d, and KSK 180d (`retired tenant_class adoption artifact:38`).
5. paid health-check interval target is 30 seconds and 3 failures, yielding 90 seconds (`retired tenant_class adoption artifact:40`; `faqs/dns-engineer-faq.md:57-60`).
6. paid transport adds DoT (`retired tenant_class adoption artifact:41`).
7. paid guest contexts lower QPS targets because backing cloud account/network policies constrain cell density.
8. paid on-prem and colo targets widen RTO because operator-owned networks and firewall rules can delay probes.
9. paid deployment p95 target includes OpenTofu module apply plus health-check worker setup.
10. paid measured benchmark must separate software DNSSEC keys from HSM-bound paid keys.

## §3.3 paid target detail

1. paid authoritative p95 target is 6 ms in public/provider contexts, matching the tenant_class matrix (`retired tenant_class adoption artifact:60`).
2. paid sustained QPS target is 50,000/sec/tenant and burst 200,000/sec in managed contexts, matching the tenant_class matrix (`retired tenant_class adoption artifact:61`).
3. paid record target is up to 50 zones and 100,000 records per zone (`retired tenant_class adoption artifact:52-55`).
4. paid DNSSEC adds ECDSAP384SHA384, Ed25519, and algorithm rollover through dual signing (`retired tenant_class adoption artifact:56`).
5. paid routing includes city-level geo and multivalue answer (`retired tenant_class adoption artifact:57`).
6. paid health-check interval target is 10 seconds and RTO is 30 seconds (`retired tenant_class adoption artifact:58`).
7. paid transport adds DoH/3 and ODoH (`retired tenant_class adoption artifact:59`).
8. paid guest-on-OCI QPS target is lower than public/provider contexts because paid OCI resources still have tenancy-level capacity variance.
9. paid on-prem target prioritizes reliable failover over global p95 parity because WAN/BGP reachability is customer controlled.
10. paid measured benchmark must include RUM-correlated latency steering if the NS1 migration claim remains (`migration-playbooks/from-route53-and-ns1.md:65-69`).

## §3.4 paid target detail

1. paid authoritative p95 target is 3 ms global and 1 ms regional in the tenant_class matrix (`retired tenant_class adoption artifact:78`).
2. paid target in provider mode is 2.8 ms p95 global because Oyatie controls both cell and anycast topology.
3. paid target in on-prem is 8 ms p95 because customer WAN and facility constraints dominate.
4. paid QPS is listed as unbounded in the tenant_class matrix, but this audit converts it to quota-managed targets for benchmarkability (`retired tenant_class adoption artifact:79`).
5. paid HSM signing ceiling is 50,000/sec/HSM in the tenant_class matrix (`retired tenant_class adoption artifact:79`).
6. paid health-check target is 1-second interval and <=8-second failover in managed/colo/provider contexts (`retired tenant_class adoption artifact:75-79`).
7. paid transport includes DoQ (`retired tenant_class adoption artifact:77`).
8. paid includes HSM-bound keys and compliance overlays (`retired tenant_class adoption artifact:74-81`).
9. paid benchmark must distinguish software signing, SoftHSM, and hardware HSM results.
10. paid benchmark must report anycast withdraw time separately from DNS answer TTL behavior.

## §4 Per-context overlay

| Context | Performance overlay |
|---|---|
| oyatie-public-cloud | Lowest operational variance among managed contexts; targets assume Oyatie controls cells, BGP policy, DNS engine rollout, observability, and tenant tenant_class enforcement. |
| guest-on-aws | Route 53 must not be the tenant product surface; targets assume AWS VPC/EC2/EBS/ELB are backing infrastructure and DNS serving remains Oyatie-controlled. |
| guest-on-oci | demo_trial is constrained by Always Free resources; paid tenant_class can use paid OCI resources; Oracle Linux/Ampere ARM must be a first-class benchmark lane. |
| on-prem | Highest network variance; targets must include disconnected mode, local resolver behavior, firewall restrictions, and customer-owned HSM/TPM options. |
| colo | Better BGP control than generic on-prem but still facility-dependent; targets must include route leak protection and remote-hands failure modes. |
| oyatie-as-cloud-provider | Highest target ambition; cloud-network-dns is an IaaS product surface and should be benchmarked like a hyperscaler control/data plane. |

## §4.1 Context-specific benchmark dimensions

1. Public cloud requires multi-region anycast p95, tenant onboarding apply time, cost meter emission, and query-log export lag.
2. Public cloud requires p99 control-plane mutation latency at or below the cloud PRD's 500 ms target (`docs/products/cloud/PRD.md:161-176`).
3. AWS guest requires no Route 53 product leakage into tenant contract.
4. AWS guest requires AWS API throttling resilience in backing resources.
5. AWS guest requires resolver behavior under VPC DNS constraints.
6. OCI guest demo_trial requires CPU, RAM, storage, egress, and LB envelope assertions.
7. OCI guest demo_trial requires DNS QPS estimates under 10Mbps LB and Ampere A1 cores.
8. OCI guest paid tenant_class requires paid LB/HSM/network resource separation.
9. On-prem requires offline DNSSEC rollover tests.
10. On-prem requires local-only private-zone resolution tests.
11. On-prem requires no internet dependency in paid air-gap mode.
12. Colo requires BGP withdraw p95 and route-origin validation tests.
13. Colo requires remote-hands and facility outage runbook exercises.
14. Provider mode requires tenant-density stress tests.
15. Provider mode requires noisy-neighbor isolation at DNS query and record-mutation levels.
16. Provider mode requires DDoS/ANY flood simulation once DNS Firewall exists.
17. All contexts require OpenTofu plan/apply timing.
18. All contexts require state-backend lock contention timing.
19. All contexts require Tier-1 OS build/test timing.
20. All contexts require signed evidence path and artifact provenance.

## §5 Comparison narrative

1. Route 53 is ahead on mature routing policies, Resolver, DNS Firewall, Profiles, domain registration, and documented quotas.
2. Oyatie target is ahead only if Cedar per-record authorization, audit-chain anchoring, HSM attestation, and six-context portability are implemented.
3. Route 53's 200+ PoP data plane is a high bar for Oyatie paid and paid anycast.
4. Oyatie's current docs claim paid p95 6 ms and paid p95 3 ms, but these are not measured in the available evidence.
5. Google Cloud DNS is ahead on private DNS forwarding, peering zones, server policies, response policies, DNS64, and logging field documentation.
6. Oyatie target is ahead on tenant-level Cedar policy and sovereign/air-gap DNS if implemented.
7. Google external health-check interval minimum of 30 seconds creates an opportunity for Oyatie paid/paid 10s/1s targets, but only after health-check contracts and probes exist.
8. Cloudflare is ahead on global edge footprint, DNS Firewall, secondary DNS, load-balancing analytics, CNAME flattening, and resolver privacy docs.
9. Oyatie target can match Cloudflare on DoH/3, DoQ, and ODoH only after resolver/authoritative architecture is clarified.
10. Cloudflare ODoH target support is officially documented; Oyatie ODoH is currently service-local target prose.
11. OCI demo_trial tenant_class Always Free is not counterpart parity; it is an Oyatie deployment promise and must be benchmarked separately.
12. Current Oyatie implementation evidence is catch-up for almost every counterpart headline surface except zone-create idempotency.
13. Current Oyatie documentation target is parity or ahead on DNSSEC, routing, health checks, and privacy transports, but target-only.
14. Current Oyatie deployment evidence is behind all counterparts because no service-local OpenTofu modules exist.
15. Current Oyatie OS portability evidence is behind the canonical Oyatie bar, not directly comparable to managed DNS counterparts.
16. Future measured benchmark should publish raw data rather than prose numbers.
17. Future benchmark should include authoritative DNS and recursive resolver surfaces as separate graphs.
18. Future benchmark should include Cloudflare and Google migration workloads, not only Route 53 and NS1.
19. Future benchmark should include public, private, and air-gapped deployment modes separately.
20. Future benchmark should compare cost per million queries and cost per health check for paid tiers, while treating OCI demo_trial tenant_class as capacity-limited free baseline.

## Benchmark acceptance checklist for build phase

1. Raw probe logs landed under a signed evidence directory.
2. Evidence includes OS, arch, context, tier, tenant class, DNS engine, commit SHA, OpenTofu module version, and HSM mode.
3. Evidence includes at least six global regions for public/provider mode.
4. Evidence includes at least one AWS guest tenancy.
5. Evidence includes at least one OCI Always Free demo_trial tenancy.
6. Evidence includes at least one on-prem or disconnected testbed.
7. Evidence includes at least one colo/provider-mode anycast test.
8. Evidence includes authoritative unsigned, authoritative signed, and negative DNSSEC proofs.
9. Evidence includes record CRUD, zone rollback, and idempotency replay.
10. Evidence includes health-check failover with packet capture or event timeline.
11. Evidence includes query telemetry lag and export success.
12. Evidence includes `cargo build --workspace --release --all-features --locked`.
13. Evidence includes `tofu plan` and `tofu apply` per context once IaC exists.
14. Evidence includes no `make`-based benchmark entrypoint.
15. Evidence includes no Python/JavaScript/Ruby/Go helper scripts.
16. Evidence includes counterpart run configuration and source citations.
17. Evidence labels public-doc limits separately from measured probe latency.
18. Evidence labels estimates separately from measured values.
19. Evidence stores failures and outliers, not only successful runs.
20. Evidence can be rerun by a cold engineer from the benchmark README.

## §6 Required benchmark harness slices

1. Authoritative answer latency harness: emits A, AAAA, TXT, CAA, SRV, HTTPS, SVCB, DNSKEY, DS, NSEC, and NSEC3 queries against each tier.
2. Authoritative throughput harness: drives sustained QPS at demo_trial and paid tenant_class target ceilings with signed and unsigned zone mixes.
3. Negative-cache harness: probes NXDOMAIN, NODATA, wildcard, and DNSSEC negative proof paths.
4. Record mutation harness: measures create, update, delete, bulk import, rollback, and idempotency replay.
5. Propagation harness: measures commit-to-authoritative-visibility across all configured authoritative nodes.
6. DNSSEC rollover harness: measures KSK/ZSK scheduling, prepublish, activate, retire, DS publication boundary, and emergency key compromise flow.
7. Health-check harness: measures probe latency, observer disagreement, failover trigger time, route withdrawal time, and recovery time.
8. Routing-policy harness: measures weighted distribution error, failover correctness, geo match correctness, latency-policy selection, and jurisdiction-policy enforcement.
9. Private-zone harness: measures VPC/workload binding latency, resolver visibility, split-horizon correctness, and private/public conflict rejection.
10. Query-log harness: measures query event emission lag, redaction correctness, export lag, and retention boundary enforcement.
11. Audit-chain harness: measures audit-event completeness for zone, record, DNSSEC, route, health, and emergency operations.
12. DNS Firewall harness: once implemented, measures policy evaluation latency, block/allow correctness, threat-list refresh, and fail-open/fail-closed behavior.
13. Secondary DNS harness: once implemented, measures AXFR/IXFR duration, TSIG rejection, serial skew, and peer recovery.
14. Resolver transport harness: if recursive scope remains in this µservice, measures UDP, TCP, DoT, DoH, DoH/3, DoQ, and ODoH separately.
15. OpenTofu harness: measures plan, apply, drift detection, state-lock contention, module attestation, and rollback timing per context.
16. OS harness: measures build and service startup on every Tier-1 OS declared by the canonical matrix.
17. OCI demo_trial tenant_class harness: measures Always Free CPU, memory, disk, network, and cost ceiling under realistic query and mutation workloads.
18. Noisy-neighbor harness: measures per-tenant latency and QPS isolation while another tenant drives record mutation and query load.
19. Failure harness: measures node crash, region evacuation, authority withdrawal, HSM unavailability, key compromise, log sink outage, and control-plane replay.
20. Cold-intern reproducibility harness: requires a fresh engineer to run the benchmark from docs and produce comparable evidence without private tribal knowledge.

## §7 Benchmark data fields required in every result row

1. `service_id` must equal `cloud-network-dns`.
2. `tier` must be one of demo_trial, paid, paid, or paid.
3. `deployment_context` must be one of the six canonical context IDs from the master plan.
4. `os_id` must name the tested operating system or mark the managed-control-plane part as not OS-bound.
5. `arch` must identify arm64, amd64, ppc64le test-only, or s390x test-only.
6. `tenant_class` must identify single tenant, shared tenant, regulated tenant, or air-gapped tenant.
7. `zone_count` must record the number of active zones in the test.
8. `record_count` must record the number of records per zone and total records.
9. `signed_zone_ratio` must record the percentage of DNSSEC-signed zones.
10. `negative_query_ratio` must record the percentage of NXDOMAIN or NODATA traffic.
11. `transport` must distinguish UDP, TCP, DoT, DoH, DoH/3, DoQ, and ODoH when applicable.
12. `routing_policy` must identify simple, weighted, failover, latency, geo, jurisdiction, or programmable policy.
13. `health_check_cadence_ms` must record configured probe cadence.
14. `observer_count` must record health-check observer count and regions.
15. `hsm_mode` must distinguish none, software-key, tenant key, HSM, and offline-root ceremony.
16. `iac_module_ref` must name the OpenTofu module version or state that IaC was not used for a target-only estimate.
17. `commit_sha` must name the source revision used for measured data.
18. `raw_evidence_uri` must point to immutable logs for measured data.
19. `source_kind` must be `measured`, `public-doc-limit`, `estimated-from-public-doc`, or `target`.
20. `confidence` must be low, medium, or high, with target-only rows capped at medium until measured.

## §8 Target-to-evidence promotion rule

Target numbers in this document are not production claims.
Target numbers become candidate claims only when a benchmark row contains raw evidence, source revision, context, tier, OS, arch, and tenant class.
Candidate claims become release claims only when the benchmark is repeated on at least two runs without unexplained variance above the accepted error budget.
demo_trial OCI claims require explicit proof that the run remained inside the Always Free envelope.
paid claims require proof that paid resource use is declared and isolated from demo_trial.
paid claims require compliance and audit-export benchmarks in addition to latency and QPS.
paid claims require dedicated-cluster evidence and tenant-owned DR exercise timing.
Any number copied from a counterpart public document remains counterpart provenance, not Oyatie measured evidence.
Any number estimated from DNSPerf or public docs must stay labeled as an estimate.
Any number from the existing service benchmark document must be treated as target evidence until the missing raw evidence path is supplied.
The build phase should reject benchmark tables that omit failures, outliers, or environment details.
