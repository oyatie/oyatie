# cloud-network-dns feature parity matrix — 2026-05-20

AGENT CLASS: microservice-ownership-coherence-audit-agent
AGENT SLUG: codex-cloud-network-dns-audit
MODE: audit-only
BUNDLE: cloud-network-dns-feature-parity-2026-05-20
SCOPE: `/Users/jasonlee/oyatie/microservices/cloud-network-dns/`

## Header citation anchors

1. ADR-0328 §D-15..§D-20 is the local authority for multi-context deployment, OpenTofu, OS support, Rust-only implementation, OCI Always Free, and audit dimensions; read anchors include `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-1815`, `:2241-2365`, and `:3140-3235`.
2. `specs/master-plan-sequencing.json:704-867` is the machine-readable deployment/IaC/OS/language/OCI profile source.
3. Service-local purpose evidence is `microservices/cloud-network-dns/retired tenant_class adoption artifact:7-10`; service-local PRD is absent.
4. Service-local architecture-equivalent evidence is `microservices/cloud-network-dns/reference-implementations/provision-zone-dnssec-and-geo-routing-rust-sdk.md:1-5`; service-local ARCHITECTURE is absent.
5. Documentation-rigor requires intern-buildability and hyperscaler-grade mechanics at `docs/standards/documentation-rigor.md:133-156`.

## External source set

- AWS Route 53 concepts, routing, control/data plane, health checks: `https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/route-53-concepts.html`.
- AWS Route 53 quotas: `https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/DNSLimitations.html`.
- AWS Route 53 Resolver DNS Firewall: `https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/resolver-dns-firewall-overview.html`.
- Google Cloud DNS overview: `https://docs.cloud.google.com/dns/docs/overview`.
- Google Cloud DNS policies: `https://docs.cloud.google.com/dns/docs/policies-overview`.
- Google Cloud DNS routing policies and health checks: `https://docs.cloud.google.com/dns/docs/routing-policies-overview`.
- Google Cloud DNS logging/monitoring: `https://docs.cloud.google.com/dns/docs/monitoring`.
- Cloudflare DNS concepts: `https://developers.cloudflare.com/dns/concepts/`.
- Cloudflare DNS zone transfers: `https://developers.cloudflare.com/dns/zone-setups/zone-transfers/`.
- Cloudflare DNS Firewall: `https://developers.cloudflare.com/dns/dns-firewall/`.
- Cloudflare Load Balancing: `https://developers.cloudflare.com/load-balancing/`.
- Cloudflare 1.1.1.1 resolver/encrypted DNS: `https://developers.cloudflare.com/1.1.1.1/`, `https://developers.cloudflare.com/1.1.1.1/encryption/dns-over-https/`, `https://developers.cloudflare.com/1.1.1.1/encryption/oblivious-dns-over-https/`.

## §1 Counterpart 1 — AWS Route 53 capability surface

1. Public hosted zones for internet-facing authoritative DNS.
2. Private hosted zones for VPC-scoped DNS.
3. Record-set management for standard DNS record types.
4. Alias records for AWS resources and zone apex targets.
5. Reusable delegation sets for consistent name-server assignments.
6. Domain registration and transfer management through Route 53 Domains.
7. Simple routing policy for single-target answers.
8. Weighted routing policy for proportional traffic distribution.
9. Latency routing policy for lowest-latency region selection.
10. Failover routing policy for active-passive DNS failover.
11. Geolocation routing policy for user-location-based answers.
12. Geoproximity routing policy with optional traffic-bias shifting.
13. IP-based routing policy for source-CIDR-oriented routing.
14. Multivalue answer routing with up to eight healthy records selected randomly.
15. Traffic Flow visual policy modeling and traffic policy records.
16. Health checks against IPv4, IPv6, or domain-name endpoints.
17. Calculated health checks based on child health checks.
18. CloudWatch alarm health checks.
19. DNS failover integration between health checks and records.
20. Route 53 ARC routing-control health-check integration.
21. Globally distributed authoritative data plane across 200+ PoPs.
22. Control plane separated from globally distributed data plane.
23. Route 53 Resolver for VPC recursive resolution.
24. Resolver inbound endpoints for hybrid DNS.
25. Resolver outbound endpoints for forwarding to other resolvers.
26. Resolver rules for conditional forwarding.
27. Resolver query logging for VPC DNS queries.
28. Public hosted-zone query logging through CloudWatch Logs.
29. Resolver DNS Firewall rule groups.
30. DNS Firewall domain lists.
31. DNS Firewall allow, block, alert actions.
32. DNS Firewall custom block responses.
33. DNS Firewall DGA, DNS tunneling, and dictionary-DGA advanced detection.
34. Route 53 Profiles for sharing DNS configuration across VPCs/accounts.
35. Private hosted-zone VPC associations.
36. Cross-account and multi-VPC private hosted-zone patterns.
37. DNSSEC signing for public hosted zones.
38. DNSSEC validation through Route 53 Resolver.
39. AWS KMS integration for DNSSEC key signing keys.
40. API/SDK/CLI coverage for DNS management.
41. Quota model for hosted zones, health checks, traffic policies, profiles, and API request rates.
42. CloudTrail logging for API operations.
43. CloudWatch metrics for health checks and Resolver DNS Firewall.
44. Integration with Elastic Load Balancing and AWS resource alias targets.
45. Integration with S3/CloudFront zone apex targets.
46. Route 53 Resolver regional isolation for VPC resolver control/data planes.
47. Pricing model based on hosted zones, queries, health checks, Resolver endpoints, and DNS Firewall.
48. SLA/availability posture backed by global authoritative infrastructure.
49. Support for private DNS in AWS GovCloud-like controlled VPC patterns through Resolver and private hosted zones.
50. AWS-native ecosystem depth: Organizations, IAM, CloudWatch, CloudTrail, KMS, VPC, ELB, CloudFront, Global Accelerator.

## §2 Counterpart 2 — Google Cloud DNS capability surface

1. Public managed zones for internet-visible authoritative DNS.
2. Private managed zones visible to authorized VPC networks.
3. Forwarding zones for outbound forwarding from VPC networks.
4. Peering zones for name resolution across VPC networks.
5. Cross-project binding and Shared VPC support.
6. IAM at project and managed-zone levels.
7. DNS Administrator and DNS Reader roles.
8. Custom IAM roles for finer Cloud DNS operations.
9. Managed DNSSEC for public zones.
10. DNSSEC key and algorithm configuration.
11. Public-zone authoritative serving through anycast.
12. Inbound server policies for on-premises resolution into Google Cloud.
13. Outbound server policies for alternate resolver targets.
14. Bi-directional DNS forwarding for hybrid environments.
15. Server policies for forwarding and logging.
16. Response policies for private DNS response overrides.
17. Routing policies for traffic steering.
18. Weighted round robin routing policy.
19. Geolocation routing policy.
20. Failover routing policy.
21. Geolocation policy with geofence behavior.
22. Health checks for routing policies.
23. External endpoint health checks.
24. Internal load-balancer health-check integration.
25. Three-region, nine-prober external endpoint health-check model.
26. Health-check interval range of 30 to 300 seconds for external endpoints.
27. TCP, HTTP, and HTTPS health-check protocols for external endpoints.
28. DNS64 for IPv6-only VM access to IPv4 destinations.
29. DNS Armor threat detection for internet-bound DNS queries.
30. Query logging for public zones.
31. Query logging for private/forwarding zones through DNS policies.
32. Logged fields for queryName, queryType, protocol, location, project, and forwarding errors.
33. Monitoring metrics and propagation checks.
34. Private-zone name resolution order within VPC networks.
35. On-premises connectivity through Cloud VPN and Cloud Interconnect.
36. Public NAT/NAT64 integration for DNS64 scenarios.
37. Per-zone labels and managed-zone metadata.
38. API, gcloud, and console management.
39. Quotas and rate limits for zones, changes, policies, records, and APIs.
40. Integration with Google Cloud load balancers for routing-policy targets.
41. Integration with Compute Engine, GKE, Shared VPC, Cloud Logging, Cloud Monitoring, and IAM.
42. Public-zone propagation monitoring guidance.
43. High-performance resilient global DNS service positioning.
44. Private DNS security model scoped to VPC authorization.
45. Cost model for managed zones and query volume.
46. No domain registrar function in Cloud DNS after Google Domains divestiture; registrar is separate.
47. No native DoH/DoT/DoQ authoritative transport product in Cloud DNS docs.
48. No Cloudflare-style proxied DNS/CDN mode in Cloud DNS docs.
49. No built-in secondary DNS/zone-transfer headline comparable to Cloudflare Enterprise zone transfers in the docs read.
50. Strong hybrid/private DNS emphasis relative to general authoritative-only providers.

## §3 Counterpart 3 — Cloudflare DNS capability surface

1. Authoritative DNS for customer zones.
2. Cloudflare authoritative nameservers and primary DNS setup.
3. Secondary DNS setup through zone transfers.
4. Incoming AXFR/IXFR when Cloudflare is secondary.
5. Outgoing AXFR/IXFR when Cloudflare is primary.
6. TSIG authentication for zone transfers.
7. Peer DNS server management for zone transfers.
8. Enterprise secondary DNS availability.
9. DNS record management for standard DNS record types.
10. CNAME flattening, including apex-style behavior.
11. Proxied DNS records that connect DNS to Cloudflare reverse proxy/CDN.
12. Automatic HTTPS/SVCB-style protocol hints for proxied records when eligible.
13. DNSSEC for authoritative zones.
14. DNSSEC options for secondary DNS.
15. DNS analytics for evaluating query data.
16. DNS analytics by query type and response detail.
17. Average processing time metrics for DNS analytics.
18. DNS Firewall as an Enterprise add-on protecting authoritative nameservers.
19. DNS Firewall cache in front of upstream nameservers.
20. DNS Firewall DDoS mitigation.
21. DNS Firewall high availability and global distribution.
22. DNS Firewall rate limiting per data center.
23. DNS Firewall min/max cache TTL controls.
24. DNS Firewall DNS ANY query blocking.
25. Cloudflare Load Balancing paid add-on.
26. Load balancing across endpoint pools.
27. Automatic failover when endpoints are unhealthy.
28. Active monitoring across multiple data centers.
29. Monitor status-code and response-text validation.
30. Intelligent routing by endpoint latency.
31. Intelligent routing by visitor geography.
32. Intelligent routing by visitor GPS coordinates where available.
33. Load balancing custom rules.
34. Load balancing analytics for traffic, endpoint health, pools, and pool changes.
35. Public DNS resolver 1.1.1.1.
36. Resolver privacy posture: does not sell user data to advertisers.
37. 1.1.1.1 support for DNS over HTTPS.
38. 1.1.1.1 support for DNS over TLS.
39. DNS over HTTPS supports HTTP, HTTP/2, and HTTP/3.
40. ODoH target support at `odoh.cloudflare-dns.com`.
41. Hundreds-of-cities global network claim for resolver infrastructure.
42. 1.1.1.1 for Families malware/adult-content filtering.
43. DNS over Tor hidden service access.
44. API and dashboard management for DNS.
45. Cloudflare registrar and registrar-adjacent domain operations outside DNS product.
46. Integration with WAF, CDN, DDoS, SSL/TLS, Workers, and Zero Trust through proxied records.
47. Strong edge-network coupling: DNS, proxy, CDN, security, and load balancing share a global network.
48. Plan/tier differentiation across Free/Pro/Business/Enterprise and add-ons.
49. Fast propagation and global authoritative DNS positioning.
50. Formal-verification research pedigree through Cloudflare authoritative DNS publications, outside product docs but relevant to hyperscaler bar.

## §4 UNION-coverage matrix

| Capability | AWS Route 53 | Google Cloud DNS | Cloudflare DNS | UNION required | Oyatie cloud-network-dns current | Gap classification |
|---|---|---|---|---|---|---|
| Public authoritative zones | yes | yes | yes | yes | target docs yes; OpenAPI zone create only | implementation gap |
| Private zones | yes | yes | partial via split/internal patterns | yes | FAQ yes; OpenAPI private kind yes | partial |
| Zone create API | yes | yes | yes | yes | external OpenAPI yes (`contracts/openapi/...:1-12`) | present narrow |
| Record CRUD API | yes | yes | yes | yes | docs yes; no contract | P1 missing |
| Standard record types | yes | yes | yes | yes | tenant_class matrix lists many | target-only |
| Apex alias/CNAME flattening | yes alias | limited | yes CNAME flattening | yes | ALIAS/ANAME named | underspecified |
| DNSSEC signing | yes | yes | yes | yes | docs yes paid tenant_class | target-only |
| DNSSEC validation | Resolver yes | resolver ecosystem | resolver yes | yes for recursive resolver claim | not specified | missing |
| DNSSEC key lifecycle | yes | yes | yes | yes | cadence stated | underspecified |
| DNSSEC DS publication automation | registrar path | registrar path | registrar path | yes as integration boundary | manual step | P1 zero-handroll gap |
| Public hosted-zone query logs | yes | yes | analytics | yes | tutorial query log | no schema |
| Private resolver query logs | yes | yes | partial via gateway/resolver | yes | FAQ query stream | no AsyncAPI |
| Health checks | yes | yes | yes via LB | yes | docs yes | no contract |
| Health-check body matching | yes | yes | yes | yes | tutorial yes | no contract |
| Calculated health checks | yes | no direct equivalent | monitor groups | yes | not specified | missing |
| Failover routing | yes | yes | yes via LB | yes | docs yes | target-only |
| Weighted routing | yes | yes | yes via LB/traffic steering | yes | docs yes | target-only |
| Latency routing | yes | partial/policy | yes via LB | yes | docs yes | target-only |
| Geolocation routing | yes | yes | yes | yes | docs yes | target-only |
| Geoproximity routing | yes | no direct equivalent | GPS/intelligent routing | yes | city/ML routing target | underspecified |
| IP/CIDR-based routing | yes | partial | custom rules | yes | not specified | missing |
| Multivalue answer | yes | partial | load balancing pools | yes | paid multi-value answer | target-only |
| Traffic policy visual model | yes Traffic Flow | no | no | optional | not specified | missing optional |
| Resolver inbound endpoints | yes | yes inbound policies | resolver product | yes if recursive claim retained | not specified | missing |
| Resolver outbound forwarding | yes | yes | gateway/resolver patterns | yes | not specified | missing |
| Conditional forwarding rules | yes | yes forwarding zones | no exact authoritative feature | yes | not specified | missing |
| DNS peering | VPC association/profile | yes peering zones | no direct | yes for hybrid cloud | not specified | missing |
| Response policies/RPZ | firewall/policies | response policies | gateway/firewall patterns | yes | not specified | missing |
| DNS Firewall allow/block/alert | yes | DNS Armor/response policy | DNS Firewall | yes | not specified | missing |
| DGA detection | yes advanced | DNS Armor | security add-ons | yes | not specified | missing |
| DNS tunneling detection | yes advanced | DNS Armor | gateway/firewall | yes | not specified | missing |
| Custom block responses | yes | response policies | firewall/proxy patterns | yes | not specified | missing |
| DNS64 | no primary headline | yes | resolver/network features | yes | not specified | missing |
| Secondary DNS AXFR | no headline | no headline | yes | yes | not specified | missing |
| Secondary DNS IXFR | no headline | no headline | yes | yes | not specified | missing |
| TSIG peer management | partial in migration ecosystems | partial | yes | yes | not specified | missing |
| Reusable delegation sets | yes | no | no | optional | not specified | missing optional |
| Domain registration | yes | no | registrar product | optional but union present | explicit no | accepted gap |
| Registrar transfer | yes | no | registrar product | optional | explicit no | accepted gap |
| Self-service signup | yes | yes | yes | yes for public product | docs say tenant+tier required | product gap |
| API/SDK coverage | yes | yes | yes | yes | target SDK doc only | P2 provenance gap |
| CLI coverage | yes AWS CLI | yes gcloud | yes API/dashboard | yes | `./bin/oya dns` docs | target-only |
| IAM/RBAC | AWS IAM | Google IAM | Cloudflare roles | yes | Cedar target | no policy files |
| Per-record authorization | partial | partial | partial | desirable | Cedar per-record target | additive target |
| Audit event chain | CloudTrail | Cloud Audit Logs | audit/logs | yes | audit-chain target | no schema |
| Compliance pack overlays | AWS compliance | Google compliance | Cloudflare compliance | yes | tenant_class matrix yes | no files |
| DDoS protection for DNS | yes via AWS infra/Shield patterns | Google infra | Cloudflare DNS Firewall | yes | anycast target | underspecified |
| Rate limiting | limited | quota/policies | DNS Firewall per DC | yes | not specified | missing |
| Cache TTL controls | yes TTL | yes TTL | yes DNS Firewall TTL | yes | TTL docs yes | partial |
| DoH authoritative/resolver | resolver only partial | no Cloud DNS headline | yes resolver | yes if resolver included | docs yes | no architecture boundary |
| DoH over HTTP/3 | partial resolver | no | yes resolver | yes for encrypted resolver | docs yes | target-only |
| DoT | no authoritative headline | no Cloud DNS headline | yes resolver | yes if resolver included | docs yes | target-only |
| DoQ | no | no | Cloudflare ecosystem | desirable | docs yes | target-only |
| ODoH | no | no | yes target | desirable | docs yes | target-only |
| Public resolver product | Resolver VPC, not public 1.1.1.1 | Google Public DNS separate | yes 1.1.1.1 | decide | purpose says recursive DNS | boundary unclear |
| Authoritative global anycast | yes | yes | yes | yes | docs yes | no deployable evidence |
| BGP anycast control | provider internal | provider internal | provider internal | yes for Oyatie provider mode | docs yes | no IaC/source |
| Multi-region replication | yes implicit | yes implicit | yes | yes | docs say replicated <=8s | no data model |
| Private/internal DNS | yes | yes | partial | yes | FAQ yes | no contract |
| Reverse DNS/PTR | yes | yes | yes | yes | FAQ yes | no IPAM handoff |
| Zone versioning | API changes | changes API | DNS records history | yes | FAQ rollback version | no contract |
| Rollback API | no simple equivalent | no simple equivalent | limited | desirable | docs yes | target-only |
| BYO-KSK | limited | advanced DNSSEC | secondary DNS options | desirable | docs yes | underspecified |
| HSM-bound signing | KMS KSK | Google-managed keys | managed | desirable | paid target | no KMS handoff |
| PQC DNSSEC | no | no | no | no, additive | target experimental | additive risky |
| Custom RR codecs | no | no | no | no, additive | target paid/paid | additive risky |
| Query telemetry stream | logs | logs | analytics/logs | yes | Kafka target | no AsyncAPI |
| Metrics/dashboard | CloudWatch | Cloud Monitoring | analytics | yes | not service-local | missing |
| SLO/OpenSLO | service docs/SLA | SLA/docs | plan/SLA | yes | none | P1 missing |
| Capacity/quota model | quotas | quotas | plan limits | yes | tenant_class numbers only | no derivation |
| Pricing model | query/hosted-zone | query/zone | plan/add-on | yes | tenant_class cost docs | partial |
| Cost budget | AWS billing | GCP billing | Cloudflare billing | yes | none service-local | missing |
| IaC modules | CloudFormation/Terraform ecosystem | Deployment Manager/Terraform ecosystem | Terraform/API ecosystem | OpenTofu per Oyatie | none | P1 missing |
| OS support | managed service | managed service | managed service | Oyatie must declare | none | P1 missing |
| OCI Always Free demo_trial | no | no | no | Oyatie-specific | absent | P1 missing |
| Multi-context deployment | no same product claim | no same product claim | no same product claim | Oyatie-specific | absent | P1 missing |
| Air-gapped DNS | Gov/Outposts patterns | sovereign patterns | enterprise controls | yes for Oyatie paid | target docs yes | no IaC |
| Sovereign compliance packs | yes ecosystem | yes ecosystem | yes ecosystem | yes | target docs yes | no files |
| Change approval workflow | IAM/CloudTrail | IAM/Audit | roles/logs | yes | Cedar reviewer target | no policy |
| Emergency failover | routing controls | failover policy | LB failover | yes | FAQ command/permit | no contract |
| Zone import from Route 53 | API/export | import tooling | import tooling | yes | playbook yes | shell-loop drift |
| Zone import from Google Cloud DNS | possible API | source itself | import tooling | yes | absent | missing |
| Zone import from Cloudflare | possible API | possible API | source itself | yes | absent | missing |
| Zone import from NS1 | possible | possible | possible | useful | playbook yes | present target |
| Divergence report | possible via logs | possible via logs | analytics | yes for migration | playbook yes | no schema |
| Dual-NS migration | yes DNS standard | yes | yes | yes | playbook yes | target-only |
| Health-check migration | yes | yes | yes | yes | Route53/NS1 playbook | no Google/Cloudflare |
| RUM latency steering | no core | no core | LB analytics | desirable | NS1-equivalent target | no architecture |
| Geo override per prefix | IP-based routing | no direct | custom rules | yes | FAQ yes | no contract |
| EDNS client subnet handling | yes policies vary | yes in geo policy | yes likely | yes | not specified | missing |
| DNS packet-size/EDNS0 policy | yes impl detail | yes impl detail | yes | yes | not specified | missing |
| DNSSEC negative proofs | yes | yes | yes | yes | NSEC3 docs | partial |
| Algorithm rollover | yes | yes | yes | yes | FAQ yes | no runbook |
| Key compromise response | yes docs | yes docs | yes docs | yes | absent | missing |
| Cache poisoning defense | DNSSEC | DNSSEC | DNSSEC | yes | DNSSEC target | no failure mode |
| Route leak/hijack defense | Route 53 resilience | Google anycast | Cloudflare route leak protection | yes | anycast target | missing |
| Abuse handling for public resolver | Resolver policies | Google Public DNS policies | Cloudflare resolver policies | yes if recursive | absent | missing |
| Privacy policy for resolver | AWS logs | Google policies | Cloudflare privacy docs | yes if recursive | ODoH target only | missing |
| Data residency for logs | AWS region/log sink | Google project/log sink | Cloudflare plan/location | yes | compliance packs target | no docs |
| Tenant zone isolation | accounts/VPC | projects/VPC | account/zone | yes | target docs yes | no policy/schema |
| Idempotency | API behavior | API behavior | API behavior | yes | external crate yes | present narrow |
| Request auth boundary | IAM | IAM | API tokens/RBAC | yes | external crate auth context | present narrow |
| OpenAPI 3.2.0 | n/a | n/a | n/a | Oyatie requires | external contract yes | outside service path |
| AsyncAPI 3.1.0 | n/a | n/a | n/a | Oyatie requires for streams | absent | missing |
| proto3 | n/a | n/a | n/a | Oyatie requires for gRPC | absent | missing |
| OpenSLO | n/a | n/a | n/a | Oyatie requires | absent | missing |

## §5 Capability families summary

| Family | UNION required count | Oyatie present as implemented evidence | Oyatie present as target docs | Gap count | Notes |
|---|---:|---:|---:|---:|---|
| Authoritative DNS core | 12 | 1 | 10 | 11 | Zone create exists externally; record CRUD/contracts missing |
| Routing and traffic steering | 13 | 0 | 9 | 13 | Rich target story; no routing policy API |
| Health checks and failover | 8 | 0 | 6 | 8 | No health-check contract/source |
| DNSSEC and key lifecycle | 12 | 0 | 8 | 12 | Manual DS and missing KMS/HSM schemas |
| Resolver/private DNS/hybrid | 13 | 0 | 3 | 13 | Recursive claim underdefined |
| DNS security/firewall | 10 | 0 | 0 | 10 | Major missing union family |
| Observability/logging/analytics | 10 | 0 | 4 | 10 | Query-log examples without schemas |
| Migration/import/export | 8 | 0 | 4 | 8 | Route53/NS1 target only; Google/Cloudflare absent |
| IAM/policy/audit | 8 | 1 | 6 | 7 | Auth context exists in external API; Cedar absent |
| Deployment/IaC/OS | 10 | 0 | 0 | 10 | Complete canonical gap |
| Cost/tier/compliance | 8 | 0 | 5 | 8 | Tier doc exists; no cost budget/compliance files |
| Encrypted DNS/privacy transports | 8 | 0 | 6 | 8 | Needs architecture boundary for recursive vs authoritative |

## §6 Headline gap analysis — top 15 missing capabilities

1. **Record CRUD contract is missing.** Route 53, Cloud DNS, and Cloudflare all expose record management; Oyatie docs use `record create`, but service-local contracts do not define create/update/delete/list records. Hook: add `contracts/openapi-v1.yaml` record endpoints and bind to `cloud_network_dns.record.*` audit events.
2. **DNSSEC lifecycle contract is missing.** Docs say enable DNSSEC, rotate KSK/ZSK, import KSK, and publish DS. Hook: add DNSSEC state machine with key states, DS publication boundary, KMS/HSM key refs, and failure modes.
3. **Health-check API is missing.** Route 53, Cloud DNS, and Cloudflare expose health-check/monitoring surfaces. Hook: add `health_check_create`, `health_check_attach`, status, simulated failure only for dev, and failover policy contracts.
4. **Routing-policy model is missing.** Union coverage includes weighted, latency, geolocation, geoproximity, IP/CIDR, multivalue, failover. Hook: define a routing-policy algebra with conflict rules and tenant_class gates.
5. **Resolver and forwarding boundary is missing.** Route 53 Resolver and Cloud DNS forwarding/peering are large counterpart surfaces; Oyatie purpose says recursive DNS but no resolver contract exists. Hook: split authoritative and recursive resolver sub-bounded-contexts in ARCHITECTURE.
6. **DNS Firewall/threat filtering is missing.** AWS and Cloudflare both have DNS firewall products; Google has DNS Armor. Hook: add domain-list, response-policy, DGA/tunnel detection, allow/block/alert, and fail-open/fail-closed policy design.
7. **Secondary DNS and zone transfers are missing.** Cloudflare supports AXFR/IXFR with TSIG and peers. Hook: add primary/secondary mode, TSIG secret refs, ACLs, transfer audit events, and import/export tests.
8. **Query-log schemas are missing.** Tutorial and FAQ show telemetry, but there is no AsyncAPI/event schema. Hook: add `cloud_network_dns.query.observed.v1` and retention/redaction rules.
9. **Cedar policy files are missing.** Docs name concrete actions, but no policy fragments exist. Hook: add default deny, zone/record/health/DNSSEC/query-stream/emergency-failover actions and tests.
10. **OpenTofu modules are missing.** Canonical direction requires per-context modules. Hook: add six context directories, always-free OCI subprofile, state backend refs, module attestations, and `tofu plan` CI.
11. **OS manifest is missing.** DNS infrastructure touches raw sockets, BGP, QUIC, HSMs, and packet parsing. Hook: add Tier-1/Tier-2/out-of-scope manifest plus package formats and CI lanes.
12. **OCI demo_trial tenant_class Always Free reconciliation is missing.** demo_trial says about $5/month while canonical OCI demo_trial tenant_class must be Always Free. Hook: add a demo_trial OCI subprofile capacity table and paid-upgrade gates.
13. **Cloudflare and Google migration playbooks are missing.** Route 53/NS1 migration exists; top-three union requires Cloud DNS and Cloudflare. Hook: add source-specific export/import schemas and divergence reports.
14. **Capacity math is missing.** Tier numbers are useful but not derived. Hook: add capacity-model with Little's Law, packet-size assumptions, signing throughput, health-check probe fanout, and anycast PoP math.
15. **Anycast/BGP architecture is missing.** Docs mention BGP anycast and withdraw timing but no deployable or operational detail. Hook: add BGP route-origin, RPKI, FRR/OVN/Cilium integration, route-leak response, and per-context differences.

## §7 Additive surface — Oyatie capabilities not clearly present in any counterpart

1. Cedar-gated per-record authorization, if implemented, is finer-grained than typical account/project/zone IAM; cited target: `retired tenant_class adoption artifact:26,44,62,80`.
2. Audit-chain anchoring for every record diff is stronger than standard vendor logs; cited target: `retired tenant_class adoption artifact:90-91`.
3. Sovereign/air-gapped paid DNS is a native service-tenant_class promise; cited target: `faqs/dns-engineer-faq.md:15-22` and `benchmarks/...:105`.
4. HSM attestation receipts attached to DNSSEC signing operations are a high-assurance addition; cited target: `retired tenant_class adoption artifact:95-96`.
5. Experimental PQC DNSSEC is not union-required and should remain gated; cited target: `retired tenant_class adoption artifact:74`.
6. Tenant-private RR-Type codec plugins are unusual and powerful; cited target: `faqs/dns-engineer-faq.md:64-68`.
7. RUM-correlated latency steering as a built-in DNS tenant_class feature is an additive NS1-like direction; cited target: `migration-playbooks/from-route53-and-ns1.md:65-69`.
8. Integrated cell-affinity between DNS, cell routing, and Oyatie platform tenancy is additive; cited target: `retired tenant_class adoption artifact:94` and `faqs/dns-engineer-faq.md:97-101`.
9. Per-tenant compliance pack overlays at DNS-record publication time are additive; cited target: `faqs/dns-engineer-faq.md:15-22`.
10. Foundry principals limited to dev/staging DNS records are additive governance; cited target: `faqs/dns-engineer-faq.md:149-153`.
11. Emergency failover governed by Cedar and audit-chain severity is additive; cited target: `faqs/dns-engineer-faq.md:90-93`.
12. Zone rollback with re-signing and SOA bump is a useful productized recovery feature; cited target: `faqs/dns-engineer-faq.md:156-167`.
13. OCI Always Free demo_trial as a deployment-tenant_class contract is Oyatie-specific and not part of counterpart union; required by `specs/master-plan-sequencing.json:857-867`.
14. Six-context deployment across Oyatie public cloud, AWS guest, OCI guest, on-prem, colo, and Oyatie-as-provider is Oyatie-specific; required by `specs/master-plan-sequencing.json:704-745`.
15. Additive features are acceptable only after core parity surfaces are contracted; otherwise they raise implementation risk by expanding scope beyond proven DNS baseline.

## Parity verdict

Headline verdict: partial.
The service-local docs know the right DNS domain and include several ambitious differentiators, but the currently evidenced implementation surface is only tenant-scoped zone creation.
Against AWS Route 53, the largest gaps are resolver/firewall/profiles/API quota maturity, domain registration, and full routing-policy contracts.
Against Google Cloud DNS, the largest gaps are private DNS forwarding/peering/server policies, response policies, DNS64, logging schemas, and health-check constraints.
Against Cloudflare DNS, the largest gaps are secondary DNS/AXFR/IXFR, DNS Firewall, CNAME flattening/proxied-DNS semantics, load-balancing analytics, and resolver privacy architecture.
The next buildable path is to define authoritative DNS core first: zone, record, DNSSEC, routing policy, health check, query log, audit event, and OpenTofu deployment surfaces.

## §8 Implementation hook index for top union gaps

| Gap | First implementation hook | Required local artifact | Provenance anchor |
|---|---|---|---|
| Record-set CRUD | Add create, update, delete, list, import, diff, and rollback endpoints | Service-local OpenAPI contract | Current external contract only proves zone create at `contracts/openapi/cloud/cloud-network-dns-v1.yaml:1-12` |
| DNSSEC lifecycle | Add key state machine, DS boundary, algorithm policy, and rollover scheduler | `ARCHITECTURE.md` plus DNSSEC runbook | demo_trial and paid tenant_class promise DNSSEC in `retired tenant_class adoption artifact:20-72` |
| Private zones | Add private-zone binding and resolver visibility model | Private-zone contract and context IaC | paid declares private DNS in `retired tenant_class adoption artifact:34` |
| Routing policies | Add policy algebra for weighted, failover, latency, geo, and jurisdiction routing | Routing-policy schema | Routing tenant_classes appear in `retired tenant_class adoption artifact:21-73` |
| Health checks | Add probe resource, observer topology, and failure semantics | Health-check OpenAPI and OpenSLO | Cadences appear in `retired tenant_class adoption artifact:22-74` |
| Audit events | Add event schema for zone, record, DNSSEC, policy, and failover changes | AsyncAPI audit-event contract | Audit retention appears in `retired tenant_class adoption artifact:23-75` |
| Query logs | Add tenant query-log stream and redaction contract | Query-log AsyncAPI contract | FAQ promises audit and policy visibility in `faqs/dns-engineer-faq.md:79-93` |
| DNS Firewall | Decide resolver-security ownership and add policy resources | PRD scope decision plus firewall contract | Counterpart union includes AWS Resolver DNS Firewall, Google DNS Armor, and Cloudflare DNS Firewall |
| Secondary DNS | Add primary/secondary zone-transfer role model | AXFR/IXFR and TSIG contract | Cloudflare counterpart exposes zone transfers and secondary DNS docs |
| Anycast posture | Define serving topology per context and tenant_class | Architecture plus capacity model | Current docs declare latency targets but no topology proof |
| OCI demo_trial tenant_class | Add Always Free admission profile | `iac/guest-on-oci/always-free/` plus tenant_class matrix row | Master plan `oci_always_free` is at `specs/master-plan-sequencing.json:857-867` |
| OS matrix | Add supported OS manifest and package map | `supported-oses.json` or service manifest field | OS directive requires Tier-1 coverage in `feedback_os_support_matrix_2026_05_20.md:37-44` |
| IaC substrate | Add OpenTofu modules per context | `iac/<context>/main.tf`, lockfile, and signing evidence | ADR-0328 D-16 at `docs/decisions/ADR-0328-...md:2241-2365` |
| SLO proof | Add authoritative-answer, propagation, and failover SLOs | OpenSLO file | Current service has benchmark text but no OpenSLO artifact |
| Benchmarks | Add measured harness and raw-result retention | Benchmark result directory and methodology note | Existing benchmark doc cites missing evidence at `benchmarks/cloud-network-dns-vs-route53-vs-cloud-dns-vs-cloudflare-vs-ns1.md:119` |

## §9 Wave 14 scheduling notes

The first build slice should close authoritative DNS core rather than the whole union matrix.
That slice should include zone create, record CRUD, DNSSEC enablement, audit event emission, and demo_trial OpenTofu deployment.
Private-zone support should be the second slice because it carries context-specific network bindings and likely needs cloud-networking handoffs.
Routing policies should be the third slice because weighted and failover policy tests can land before latency, geo, and jurisdiction routing.
Health checks should land with routing policies because failover routing cannot be verified without probe semantics.
Query logging and analytics should land before DNS Firewall because firewall tuning needs query evidence.
Secondary DNS and zone transfer should follow the core public-authoritative path because they introduce external authority relationships.
OCI Always Free should be implemented as a demo_trial admission envelope before any paid tenant_class OCI uplift work.
The OS matrix should be added before self-hosted or installable contexts because OS support is canonical µservice ownership.
paid governance features should not block demo_trial and paid, but their schemas need early shape so IAM and audit-chain can align.
paid dedicated-cluster work should wait for topology architecture, capacity evidence, and runbook proof.
Cloudflare parity gaps are disproportionately public-authoritative, analytics, and enterprise-operational.
AWS and Google parity gaps are disproportionately private-zone, resolver, forwarding, and cloud-network integration.
Oyatie additive governance is valuable only after it becomes contracts, policies, tests, and deployable modules.
The current local docs are enough to guide planning, but not enough for a cold intern to build the service end to end.
