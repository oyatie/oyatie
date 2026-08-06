# cloud-network feature parity matrix — 2026-05-20

## Header anchor block

1. Canonical sequence anchor: `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-4146`.
2. Machine control anchor: `specs/master-plan-sequencing.json:704-868`.
3. µservice PRD anchor: local `PRD.md` absent; purpose inferred from `microservices/cloud-network/retired tenant_class adoption artifact:7-10` and `docs/products/cloud/PRD.md:138-176`.
4. µservice architecture anchor: local `ARCHITECTURE.md` absent; implementation shape inferred from `crates/oya-cloud-network-domain/src/lib.rs:1-7` and the VPC/LB/DNS API crates.
5. Documentation-rigor anchor: `docs/standards/documentation-rigor.md:40-83`, `docs/standards/documentation-rigor.md:175-190`, and `docs/standards/documentation-rigor.md:222-260`.

## Source legend

- AWS VPC docs: https://docs.aws.amazon.com/vpc/.
- AWS VPC quotas: https://docs.aws.amazon.com/vpc/latest/userguide/amazon-vpc-limits.html.
- AWS Direct Connect: https://docs.aws.amazon.com/en_us/directconnect/latest/UserGuide/connection-classic.html.
- AWS Reachability Analyzer: https://docs.aws.amazon.com/vpc/latest/reachability/how-reachability-analyzer-works.html.
- Google Cloud VPC docs: https://docs.cloud.google.com/vpc/docs/vpc.
- Google Cloud VPC quotas: https://docs.cloud.google.com/vpc/docs/quota.
- Google Cloud VPC Flow Logs: https://docs.cloud.google.com/vpc/docs/flow-logs.
- Azure Virtual Network concepts: https://learn.microsoft.com/en-us/azure/virtual-network/concepts-and-best-practices.
- Azure Virtual Network peering: https://learn.microsoft.com/en-us/azure/virtual-network/virtual-network-peering-overview.
- Azure service limits: https://learn.microsoft.com/en-us/azure/azure-resource-manager/management/azure-subscription-service-limits.
- Azure Network Watcher: https://learn.microsoft.com/en-us/azure/network-watcher/network-watcher-overview.
- Oyatie local tenant_class source: `microservices/cloud-network/retired tenant_class adoption artifact:7-95`.
- Oyatie local FAQ source: `microservices/cloud-network/faqs/network-engineer-faq.md:7-175`.
- Oyatie local runbook source: `microservices/cloud-network/runbooks/*.md`.
- Oyatie Rust source: `crates/oya-cloud-network-domain/src/lib.rs:1-240`.

## §1 Counterpart 1 — AWS VPC capability surface

1. Isolated VPC address space and subnet segmentation.
2. Regional VPC resource model.
3. Public subnets and private subnets.
4. Route tables per VPC.
5. Non-propagated and propagated routes.
6. Internet gateway.
7. Egress-only internet gateway for IPv6.
8. NAT gateway.
9. NAT gateway private IP scaling.
10. Security groups.
11. Inbound and outbound security group rules.
12. Network ACLs.
13. VPC sharing across participant accounts.
14. Subnet sharing.
15. VPC peering.
16. Cross-account VPC peering.
17. Cross-region VPC peering.
18. AWS Transit Gateway central hub.
19. Transit gateway routing.
20. AWS Cloud WAN global core network.
21. AWS Network Manager centralized monitoring.
22. AWS PrivateLink.
23. VPC interface endpoints.
24. VPC gateway endpoints.
25. VPC endpoint services.
26. IP Address Manager (IPAM).
27. Bring-your-own IPv4 or IPv6 address planning through IPAM-adjacent workflows.
28. VPC Flow Logs.
29. Traffic Mirroring.
30. Reachability Analyzer.
31. Network Access Analyzer.
32. Route server support.
33. BGP peering through route servers and Direct Connect.
34. AWS Direct private connectivity.
35. Direct 1/10/100/400 Gbps dedicated port options.
36. Site-to-Site VPN.
37. Client VPN.
38. VPN CloudHub patterns.
39. AWS Network Firewall integration.
40. Elastic Load Balancing integration.
41. Gateway Load Balancer integration.
42. Private DNS support for endpoints.
43. VPC DNS hostnames and DNS resolution controls.
44. IPv6 CIDR support.
45. Managed prefix lists.
46. Route table quotas and quota-increase path.
47. Security group quotas and quota-increase path.
48. Network Address Usage quota metric.
49. CloudTrail/API audit integration.
50. Multi-account organization-aware network analysis.
51. VPC Block Public Access and analyzer support.
52. Resource map / inventory in console.
53. Integration with AWS Organizations.
54. Integration with load balancer target groups.
55. Managed regional and cross-region private network services.

## §2 Counterpart 2 — Google Cloud VPC capability surface

1. Global VPC network resource.
2. Regional subnet resource.
3. Auto mode VPC network.
4. Custom mode VPC network.
5. One auto subnet per region in auto mode.
6. Custom CIDR selection in custom mode.
7. IPv4-only, dual-stack, and IPv6 subnet options.
8. Internal IPv6 subnet ranges.
9. External IPv6 subnet ranges.
10. System-generated subnet routes.
11. Static routes.
12. Dynamic routing mode.
13. Cloud Router BGP management.
14. Regional and global dynamic routing behavior.
15. VPC Network Peering.
16. Peering route exchange controls.
17. Shared VPC host project.
18. Shared VPC service projects.
19. Cloud VPN.
20. Cloud Interconnect.
21. VLAN attachments.
22. Router appliance integration.
23. Private Service for Google APIs.
24. Private Service for producer services.
25. Private services access.
26. Cloud NAT.
27. Instance-based proxy route targets.
28. VPC firewall rules.
29. Hierarchical firewall policies.
30. Global network firewall policies.
31. Regional network firewall policies.
32. Firewall Rules Logging.
33. Implied deny ingress rule.
34. Implied allow egress rule.
35. Network tags for routes and firewall targeting.
36. Service account targeted firewall rules.
37. VPC Flow Logs.
38. Organization-level flow log configuration.
39. Project-level flow log configuration.
40. Subnet-level flow log configuration.
41. VLAN attachment and Cloud VPN tunnel flow logs.
42. Packet Mirroring.
43. Connectivity Tests / Network Intelligence Center equivalent diagnostics.
44. Load balancing forwarding rules integration.
45. Internal passthrough Network Load Balancers.
46. Internal Application Load Balancer proxy systems.
47. External load balancer backend integration.
48. Maximum MTU up to 8,896 bytes.
49. Per-network quotas for instances, aliases, subnet ranges, routes, peerings, and PSC.
50. Per-instance throughput guidance, including 7 Gbps sustained all-flow and 3 Gbps sustained single-flow external egress references in quota docs.
51. Flow logs for monitoring, cost optimization, and network forensics.
52. Cloud Monitoring quota metrics.
53. Organization policy controls for default networks.
54. Private Google Access and Google API access patterns.
55. Production guidance favoring custom-mode VPC networks.

## §3 Counterpart 3 — Azure Virtual Network capability surface

1. Virtual network as Azure private network building block.
2. Subscription-scoped VNet resource.
3. Region-scoped VNet resource.
4. Custom private address space.
5. Public address space owned by the organization when appropriate.
6. Subnets for segmentation.
7. Subnet address range controls.
8. Reserved addresses per subnet.
9. Network Security Groups.
10. NSG rules for inbound and outbound traffic.
11. User-defined routes.
12. Route tables.
13. System routes.
14. Virtual network peering.
15. Global virtual network peering.
16. Cross-subscription peering.
17. Cross-tenant peering.
18. Cross-deployment-model peering.
19. Gateway transit.
20. Service chaining through UDRs.
21. Hub-and-spoke topology.
22. Virtual network gateway.
23. VPN Gateway.
24. ExpressRoute Gateway.
25. ExpressRoute private connectivity.
26. ExpressRoute FastPath.
27. ExpressRoute gateway route advertisement controls.
28. Azure NAT Gateway.
29. Azure Private Link.
30. Private Endpoints.
31. Service Endpoints.
32. Azure Firewall integration.
33. Azure DDoS Protection integration.
34. Azure Load Balancer integration.
35. Application Gateway integration.
36. Azure Front Door / WAF adjacency.
37. Azure DNS private resolver adjacency.
38. VNet Flow Logs through Network Watcher.
39. NSG Flow Logs legacy path and migration pressure.
40. Network Watcher monitoring and diagnostics.
41. Connection troubleshoot / connectivity checks.
42. Effective routes inspection.
43. IP flow verify / NSG diagnostics.
44. Virtual Network Manager.
45. Connectivity configurations.
46. Increased peering scale through Virtual Network Manager.
47. 100 virtual networks per subscription default/maximum in listed limits.
48. 20 DNS servers per VNet.
49. 4,096 private IP addresses per VNet.
50. 500,000 concurrent TCP/UDP flows per NIC, up to 1,000,000 for two or more NICs.
51. 200 NSGs default/maximum in listed limits.
52. 1,000 NSG rules per NSG maximum.
53. 200 route tables default/maximum.
54. 600 UDRs per route table.
55. MACsec data-link encryption on Microsoft-controlled datacenter boundary movement described in Azure VNet FAQ.

## §4 UNION-coverage matrix

| Capability | AWS VPC | Google Cloud VPC | Azure VNet | UNION required | Oyatie cloud-network has | Gap classification |
|---|---|---|---|---|---|---|
| Tenant VPC/network create API | yes | yes | yes | yes | partial, VPC API crate and OpenAPI exist outside folder | ownership-folder gap |
| Private address-space planning | yes | yes | yes | yes | partial, CIDR fields in VPC API | needs PRD/ARCH |
| Regional subnet segmentation | yes | yes | yes | yes | partial, per-cell subnets in tenant_class matrix | needs contract detail |
| Global VPC resource model | no | yes | no | yes | no | product gap |
| Route tables | yes | yes | yes | yes | yes in Rust domain | needs local docs |
| Static routes | yes | yes | yes | yes | yes in Rust domain | needs local docs |
| Dynamic routing | yes | yes | yes | yes | partial, BGP docs/runbook | needs architecture |
| BGP route filtering | partial | yes | yes | yes | partial, tenant_class matrix paid/paid | needs IaC/SLO |
| Internet gateway equivalent | yes | yes | yes | yes | partial, NAT/proxy topology noted | needs API/IaC |
| Egress-only IPv6 gateway | yes | partial | partial | yes | no | missing |
| NAT gateway | yes | yes | yes | yes | partial, NAT/proxy named | needs capacity model |
| Private NAT / internal egress | partial | yes | partial | yes | partial | missing detail |
| Security groups | yes | no, firewall rules | yes, NSG | yes | yes in Rust domain | needs local docs |
| Network ACLs | yes | no | partial | yes | no | missing |
| Firewall rules | partial | yes | yes | yes | partial, Cedar/Cilium examples | needs policy docs |
| Hierarchical firewall policy | partial | yes | partial | yes | no | missing |
| Org-level firewall governance | partial | yes | yes via manager/policy | yes | no | missing |
| VPC flow logs | yes | yes | yes | yes | partial, FAQ/examples | needs SLO/event schema |
| Packet mirroring | yes | yes | partial | yes | partial, paid packet capture claim | needs contract |
| Network forensics workflow | partial | yes | yes | yes | partial, runbooks | needs evidence model |
| Reachability analysis | yes | yes | yes | yes | no | missing |
| Access exposure analysis | yes | partial | partial | yes | no | missing |
| Effective-route inspection | partial | partial | yes | yes | partial, runbook commands | needs API |
| Central network manager | yes | partial | yes | yes | no | missing |
| Cloud WAN / virtual WAN | yes | partial | yes | yes | no | missing |
| Transit gateway / hub | yes | partial | yes | yes | partial, BGP/hub implied | needs design |
| VPC peering | yes | yes | yes | yes | partial, tenant_class matrix | needs API/IaC |
| Cross-region peering | yes | yes | yes | yes | partial | needs context matrix |
| Cross-account/project/subscription sharing | yes | yes | yes | yes | partial, tenant model implied | needs contract |
| Shared VPC/subnet sharing | yes | yes | partial | yes | no | missing |
| Private service endpoints | yes | yes | yes | yes | partial, private endpoint FAQ | needs API/IaC |
| Endpoint services / producer model | yes | yes | yes | yes | no | missing |
| Private DNS for endpoints | yes | partial | yes | yes | partial, DNS API exists elsewhere | ownership split unresolved |
| DNS server settings | partial | partial | yes | yes | partial | needs DNS handoff |
| IPv6 subnets | yes | yes | yes | yes | yes in FAQ/tier claims | needs contract tests |
| Dual-stack controls | yes | yes | yes | yes | partial | needs API detail |
| BYOIP / public prefix management | yes | partial | yes | yes | no | missing |
| IPAM | yes | partial | partial | yes | no | missing |
| CIDR overlap prevention | yes | yes | yes | yes | partial, Rust validation likely | needs local docs |
| Quota model | yes | yes | yes | yes | no local quotas | missing |
| Route scale limits | yes | yes | yes | yes | no local limits | missing |
| Flow scale limits | partial | yes | yes | yes | no local limits | missing |
| Security rule scale limits | yes | yes | yes | yes | no local limits | missing |
| Direct interconnect | yes | yes | yes | yes | partial, tenant_class matrix | needs SKU/SLA |
| VPN | yes | yes | yes | yes | partial | needs API/IaC |
| Cloud router / route server | yes | yes | partial | yes | partial, BGP docs | needs module |
| Load balancer integration | yes | yes | yes | yes | yes, LB API crate | ownership-folder gap |
| Gateway load balancer / appliance chaining | yes | partial | yes | yes | partial | needs design |
| DDoS protection | partial | partial | yes | yes | partial, DDoS runbook | needs product API |
| WAF integration | yes | yes | yes | yes | partial, WAF fields in LB API | needs docs |
| mTLS ingress | partial | partial | partial | yes for Oyatie | yes | additive strength |
| Cedar policy gating | no | no | no | no | yes | Oyatie additive |
| Audit-chain flow projection | no | no | no | no | partial | Oyatie additive |
| Tenant packet tags | no | no | no | no | yes in tenant_class matrix | Oyatie additive |
| Cell-aware routing | partial | partial | partial | yes for Oyatie | yes | needs architecture |
| On-prem support | yes | yes | yes | yes | partial, FAQ | needs IaC |
| Colo support | yes via Direct | yes via Interconnect | yes via ExpressRoute | yes | partial | needs IaC |
| Public cloud provider mode | no | no | no | yes for Oyatie | partial | needs product API |
| OCI Always Free demo_trial | no | no | no | yes for Oyatie | no | canonical gap |
| OpenTofu context deployment | no | no | no | yes for Oyatie | no | canonical gap |
| Supported OS manifest | no | no | no | yes for Oyatie | no | canonical gap |
| Rust-only backend/control plane | no | no | no | yes for Oyatie | yes in external crates | ownership-folder gap |
| Signed module provenance | partial | partial | partial | yes for Oyatie | no | canonical gap |
| State backend by context | yes | yes | yes | yes for Oyatie | no | canonical gap |
| Route rollback runbook | partial | partial | partial | yes | yes | present |
| mTLS failure runbook | partial | partial | partial | yes | yes | present |
| DDoS runbook | partial | partial | yes | yes | yes | present |
| Migration from AWS VPC/Istio | no | no | no | useful | yes | additive but AWS-only |
| Migration from GCP VPC | no | no | no | useful | no | missing playbook |
| Migration from Azure VNet | no | no | no | useful | no | missing playbook |
| Kubernetes CNI integration | partial | GKE | AKS | yes for Oyatie | partial, Cilium mentioned | needs architecture |
| Service mesh integration | no | partial | partial | yes for Oyatie | partial, Istio migration | needs design |
| Flow-log retention policy | yes | yes | yes | yes | partial | needs compliance |
| Compliance tenant_class mapping | partial | partial | partial | yes | partial, tenant_class matrix | needs compliance.md |
| FIPS packet path | partial | partial | partial | yes for Oyatie | partial | needs proof |
| MACsec / link encryption | Direct options | Interconnect options | Azure backbone MACsec statement | yes | partial | needs context detail |
| SR-IOV / DPDK tenant path | no | no | no | additive | paid claim | needs design/proof |
| eBPF policy enforcement | no | no | no | additive | Cilium/eBPF claim | needs implementation map |
| HTTP/3 ingress default | no | no | no | yes for Oyatie | partial | needs SLO/contracts |
| ECH/PQC transport posture | no | no | no | yes for Oyatie | partial | needs architecture/IaC |
| Tenant CA dependency | no | no | no | yes for Oyatie | partial, runbooks | needs handoff |
| Provider credential isolation | yes | yes | yes | yes for Oyatie | no local doc | missing |
| Cost attribution by tenant | yes | yes | yes | yes | partial, benchmark TCO | needs cost-budget |
| Benchmark evidence | public docs/quotas | public docs/quotas | public docs/quotas | yes | claimed but not evidenced | needs measured evidence |
| Developer onboarding | official docs | official docs | official docs | yes | partial, 181 lines | below rigor floor |
| Incident response consolidation | partial | partial | partial | yes | partial, runbooks only | needs incident-response.md |
| Cross-service handoffs | partial | partial | partial | yes | partial, runbook notes | needs handoff doc |
| API idempotency | yes | yes | yes | yes | yes in Rust API crates | ownership-folder gap |
| Authorization decision binding | IAM | IAM | RBAC | yes | yes in Rust API crates | needs docs |
| Data residency in network create | partial | partial | partial | yes | yes in VPC API | needs docs |
| Data classification tags | no | no | no | yes for Oyatie | yes in API crates | needs docs |
| Load balancer mTLS config | partial | partial | partial | yes | yes in LB API | needs docs |
| DNSSEC reference | Route 53 | Cloud DNS | Azure DNS | yes | partial in DNS API/ref | ownership split |
| WAF policy reference | AWS WAF | Cloud Armor | Azure WAF | yes | partial in LB API | needs design |
| Audit events | CloudTrail | Cloud Audit Logs | Activity Log | yes | partial | needs event schema |

## §5 Capability families summary table

| Family | UNION required count | Oyatie present count | Present quality | Dominant gap |
|---|---:|---:|---|---|
| Isolation and addressing | 13 | 7 | partial | IPAM, BYOIP, CIDR governance, global VPC model |
| Subnets and routes | 12 | 8 | partial | quota model, egress-only IPv6, dynamic route scale |
| Security controls | 14 | 8 | partial | hierarchy, org policy, ACL parity, firewall contracts |
| Private service access | 8 | 3 | weak | PrivateLink/PSC/Private Endpoint producer-consumer model |
| Hybrid and interconnect | 10 | 6 | partial | SKU/SLA, VPN details, per-context modules |
| Observability and diagnostics | 12 | 6 | partial | reachability analyzer, traffic mirroring, flow evidence |
| Load balancing and ingress | 9 | 6 | partial | gateway appliance chaining, WAF docs, context IaC |
| DNS adjacency | 6 | 3 | partial | ownership split with cloud-network-dns |
| DDoS and edge security | 5 | 3 | partial | product API and tiered capacity proof |
| Quotas and governance | 12 | 2 | weak | tenant/tier/context quotas absent |
| Deployment substrate | 12 | 2 | weak | OpenTofu, state backend, signing, OS manifest absent |
| Oyatie additive policy | 9 | 7 | strong conceptually | architecture proof and local docs absent |

## §6 Headline gap analysis — top 15 missing capabilities

1. IPAM: AWS IPAM is a first-class VPC family member; Oyatie has CIDR fields but no IPAM product model, pool hierarchy, allocation workflow, audit path, or conflict-resolution policy.
2. Private endpoint producer-consumer model: AWS PrivateLink, Google Private Service Connect, and Azure Private Link all support private service access; Oyatie has only private endpoint references without a full producer/consumer contract.
3. Transit hub / global network manager: AWS Transit Gateway/Cloud WAN, Google NCC-adjacent patterns, and Azure Virtual Network Manager/Virtual WAN create a global control surface; Oyatie has BGP/interconnect prose but no control API.
4. Reachability analyzer: all three clouds have diagnostic or route-inspection surfaces; Oyatie runbooks inspect incidents but do not expose a modeled reachability analysis product.
5. Traffic mirroring / packet capture: AWS and Google expose mirroring; Oyatie claims packet capture retention at high tenant_classes but lacks contract, consent, retention, and compliance docs.
6. Firewall hierarchy: Google has hierarchical/global/regional firewall policies, Azure has NSG and manager patterns, AWS has Network Firewall; Oyatie has Cedar and Cilium prose but no network firewall hierarchy artifact.
7. Route and flow quotas: AWS/GCP/Azure publish quotas; Oyatie has throughput claims but no per-tenant route, rule, flow, NAT, or BGP scale limits.
8. OpenTofu context modules: canonical Oyatie deployment requires per-context OpenTofu; the folder has no IaC directories.
9. OCI Always Free demo_trial: canonical demo_trial for guest-on-OCI must map to Always Free; the tenant_class matrix does not say this and no always-free module exists.
10. Supported OS manifest: canonical OS support is missing entirely.
11. Signed module provenance: D-16 expects signing and provenance; local docs have no sigstore/cosign plan.
12. State backend per context: D-16 expects state backend per context; local docs have no state backend.
13. Compliance/DPIA: tenant_class matrix mentions FIPS and packet captures; no compliance or DPIA file governs that surface.
14. Cost budget: benchmark doc claims TCO; no cost-budget file models per-tenant_class/per-context economics.
15. Migration breadth: AWS/Istio migration exists; GCP, Azure, on-prem, colo, and Oyatie-provider migration playbooks do not.

## §7 Additive surface — Oyatie capabilities not in any counterpart

1. Cedar-native network authorization: Oyatie examples integrate Cedar policy into network provisioning and ingress policy, which is not a native VPC product feature in AWS/GCP/Azure.
2. Tenant packet tags: tenant_class matrix describes persistent tenant/workload/pack classification on packets; this is more domain-specific than ordinary VPC tags.
3. Audit-chain flow projection: FAQ describes flow logs projected into audit-chain, giving stronger tenant audit semantics than default flow logging if implemented.
4. mTLS as network-tenant_class primitive: local docs treat mTLS as a first-class ingress and failure domain, not just load-balancer TLS configuration.
5. Cell-aware routing doctrine: runbooks and tenant_class matrix speak in cells and cross-cell route health, which is broader than a single-cloud VPC model.
6. Illegitimate-flow doctrine: tenant_class matrix says every legitimate flow must pass through cloud-network, a platform-wide invariant rather than a cloud resource feature.
7. ECH/PQC posture: FAQ and ADR-linked transport doctrine imply network surfaces must track ECH/PQC adoption where supported.
8. Rust-strict control implementation: external crates show Rust-owned VPC/LB/DNS API boundaries, unlike cloud-provider black-box control planes.
9. OCI Always Free as demo_trial doctrine: not present yet in the folder, but canonical direction makes no-cost OCI viability a product tenant_class constraint, not a marketing free-trial option.
10. Oya VCS/governed delivery integration: ownership docs should tie network changes to claim/verify/done/promote gates, which public clouds do not expose as customer-facing product semantics.

## Matrix verdict

- AWS VPC parity: partial.
- Google Cloud VPC parity: partial.
- Azure Virtual Network parity: partial.
- Top-3 union coverage: no.
- Oyatie cloud-network has a strong conceptual substrate and Rust/API implementation evidence.
- Oyatie cloud-network lacks ownership-folder parity docs for IPAM, private endpoints, transit/global networking, diagnostics, quotas, OpenTofu modules, OS support, and OCI Always Free.
- The parity target should be "portable VPC-equivalent network substrate plus Oyatie additive policy/audit/mTLS/cell semantics", not a provider vocabulary clone.

## §8 Implementation-hook backlog by capability family

| Gap | Proposed Oyatie hook | First artifact to create | Evidence source |
|---|---|---|---|
| IPAM | Add `cloud.network.ipam.pool.create` and CIDR allocation structs beside VPC domain types. | `ARCHITECTURE.md §ipam` plus OpenAPI contract. | AWS IPAM docs; `crates/oya-cloud-network-domain/src/lib.rs:40-240` |
| BYOIP | Add public-prefix admission with ownership proof and route-advertisement limits. | `contracts/openapi/cloud/cloud-network-prefix-v1.yaml`. | AWS/GCP/Azure public prefix surfaces |
| CIDR overlap prevention | Promote CIDR validation from API implementation to documented invariant. | `ARCHITECTURE.md §address-planning`. | Google custom-mode VPC production guidance |
| Private endpoint consumer | Add endpoint create API with tenant, service, VPC, DNS zone, and policy decision fields. | `contracts/openapi/cloud/cloud-network-private-endpoint-v1.yaml`. | AWS PrivateLink / Google PSC / Azure Private Link |
| Private endpoint producer | Add service attachment / producer registration model. | `PRD.md §private-service-access`. | AWS endpoint services / Google PSC producer / Azure Private Link Service |
| Transit hub | Add tenant transit graph with explicit spokes, advertised prefixes, and policy. | `ARCHITECTURE.md §transit-graph`. | AWS Transit Gateway / Azure hub-spoke peering |
| Global network manager | Add read model for cross-cell network topology and intended reachability. | `contracts/openapi/cloud/cloud-network-topology-v1.yaml`. | AWS Network Manager / Azure Virtual Network Manager |
| Reachability analyzer | Add formal path query that models routes, rules, policies, and endpoints without sending packets. | `contracts/openapi/cloud/cloud-network-reachability-v1.yaml`. | AWS Reachability Analyzer / Azure Network Watcher |
| Exposure analyzer | Add denied/allowed path linter for public ingress and private endpoint exposure. | `tests/reachability/` plus Rust checker crate binding. | AWS Network Access Analyzer |
| Flow-log schema | Define event fields, sampling, retention, tenant tags, and audit-chain projection. | `contracts/events/cloud.network_flow.v1.avsc` or local equivalent pointer. | Google VPC Flow Logs / Azure VNet Flow Logs |
| Traffic mirroring | Define capture session lifecycle, consent, retention, and packet redaction. | `compliance.md §packet-capture`. | AWS Traffic Mirroring / Google Packet Mirroring |
| Firewall hierarchy | Layer Cedar admission, network firewall policy, security group, and CNI enforcement order. | `ARCHITECTURE.md §network-policy-order`. | Google hierarchical firewall policies |
| Network ACLs | Decide whether ACLs are a separate layer or compiled into security groups/firewall policies. | `PRD.md §anti-stories`. | AWS NACLs |
| Route quotas | Set tier/context route table, route, dynamic-prefix, and peering limits. | `capacity-model.md §routes`. | AWS/GCP/Azure quota docs |
| Security rule quotas | Set rules-per-group, groups-per-resource, and hierarchy limits. | `capacity-model.md §policy-scale`. | AWS SG quotas / Azure NSG limits |
| Flow quotas | Set concurrent flow, connection-rate, NAT port, and idle-time limits. | `capacity-model.md §flows`. | Azure flow limits / Google flow notes |
| NAT capacity | Define NAT throughput, port exhaustion handling, failover, and per-context primitives. | `failure-modes.md §nat-egress`. | AWS NAT / Google Cloud NAT / Azure NAT Gateway |
| IPv6 egress | Define egress-only IPv6 or policy-equivalent egress construct. | `ARCHITECTURE.md §ipv6-egress`. | AWS egress-only IGW / GCP/Azure IPv6 guidance |
| DNS handoff | Split private DNS inside cloud-network from authoritative DNS in cloud-network-dns. | `cross-microservice-handoffs.md §dns`. | `cloud.network.dns.zone.create` registry bindings |
| Load balancer ownership | Keep LB API in cloud-network or split to dedicated LB sub-surface with clear owner. | `ARCHITECTURE.md §lb-boundary`. | `crates/oya-cloud-network-lb-api/src/lib.rs:1-16` |
| WAF policy | Bind WAF policy references to Cedar, edge WAF, and context IaC. | `policy/network-waf.cedar` plus IaC module outputs. | LB API WAF field |
| DDoS tenant_classes | Convert DDoS runbook into product capability by tenant_class and context. | `PRD.md §ddos-protection`. | `runbooks/ddos-mitigation-engagement.md:15-186` |
| Direct interconnect | Define port speeds, BGP sessions, RPKI, MACsec, SLAs, and billing. | `capacity-model.md §interconnect`. | AWS Direct / Google Interconnect / Azure ExpressRoute |
| VPN | Define site-to-site and client VPN equivalent, tenant policy, and routing. | `contracts/openapi/cloud/cloud-network-vpn-v1.yaml`. | AWS Site-to-Site VPN / Google Cloud VPN / Azure VPN Gateway |
| Cloud router | Define portable route-server API with context adapters. | `ARCHITECTURE.md §route-server`. | AWS route server / Google Cloud Router |
| Shared network | Define tenant/org sharing model without provider account vocabulary leakage. | `PRD.md §shared-network`. | AWS VPC sharing / Google Shared VPC |
| Observability dashboards | Name dashboards, metrics, traces, logs, and cardinality budgets. | `ARCHITECTURE.md §observability`. | Documentation-rigor §1.2 |
| Incident handoff | Convert runbook coordination notes into governed owner contracts. | `cross-microservice-handoffs.md`. | Existing runbook coordination sections |
| OCI demo_trial tenant_class | Define exact Always Free network envelope and paid escape hatches. | `retired tenant_class adoption artifact` revision plus `iac/guest-on-oci/always-free/`. | ADR-0328 §D-19 |
| OpenTofu context modules | Create per-context module set with signed provenance and state backend. | `iac/<context>/{main,variables,outputs,versions}.tf`. | ADR-0328 §D-16 |
| OS support | Declare packages/test lanes for every Tier-1 OS and Tier-2 arch. | `supported-oses.json`. | ADR-0328 §D-17 |
| Benchmark evidence | Replace unverified measured prose with signed target/measured split. | `benchmarks/evidence-index.md`. | Existing benchmark doc lines 3-101 |
| Cost model | Convert TCO claims into per-tenant_class/per-context budget math. | `cost-budget.md`. | Existing benchmark TCO table |
| Compliance | Bind FIPS, packet capture, tenant logs, DPIA, and residency. | `compliance.md` and `dpia.md`. | tenant_class adoption matrix paid/paid claims |
| Rust build lane | Publish canonical Cargo and Oya gate commands. | `README.md §build-and-test`. | ADR-0328 §D-18 |
| API idempotency | Document idempotency ledger behavior and error states. | `ARCHITECTURE.md §idempotency`. | VPC/LB/DNS API crates |
| Authorization binding | Document authorization decision ID, allowed surfaces, tenant/principal checks. | `ARCHITECTURE.md §authorization`. | VPC/LB/DNS API structs |
| Data classification | Document public/internal-only field boundaries. | `compliance.md §data-classes`. | API crate data_class comments |
| Residency | Document residency controls for VPC create and private endpoint placement. | `ARCHITECTURE.md §residency`. | VPC OpenAPI residency field |
| Cell routing | Define cell graph, route health, failover, and cross-cell stall semantics. | `failure-modes.md §cross-cell-routing`. | Cross-cell runbook |
| mTLS ingress | Promote tutorial/runbook behavior to formal LB mTLS capability. | `contracts/openapi/cloud/cloud-network-lb-v1.yaml` extension or local pointer. | LB API mTLS fields |
| CNI integration | Decide Cilium implementation boundary and abstraction contract. | `ARCHITECTURE.md §cni-enforcement`. | FAQ Cilium answer |
| Service mesh migration | Keep Istio migration as one playbook and add general service-mesh abstraction. | `migration-playbooks/from-service-mesh-generic.md`. | AWS/Istio migration playbook |
| GCP migration | Add import and dual-run plan for Google VPC, firewall rules, Cloud NAT, PSC, and Cloud Router. | `migration-playbooks/from-google-vpc.md`. | Google VPC docs |
| Azure migration | Add import and dual-run plan for VNet, NSG, UDR, Private Link, ExpressRoute, and Network Watcher. | `migration-playbooks/from-azure-vnet.md`. | Azure VNet docs |
| On-prem migration | Add physical-network intake for CIDR, BGP, DNS, firewall, and appliance inventory. | `migration-playbooks/from-on-prem-network.md`. | ADR-0328 context matrix |
| Colo migration | Add colo intake for cross-connect, BGP, MACsec, route filtering, and remote hands. | `migration-playbooks/from-colo-network.md`. | ADR-0328 context matrix |
