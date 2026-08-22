# cloud-network capability tenant_class deltas vs counterparts — 2026-05-20

## Header anchor block

1. Canonical sequence anchor: `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-4146`.
2. Machine control anchor: `specs/master-plan-sequencing.json:704-868`.
3. µservice PRD anchor: local `PRD.md` absent; purpose inferred from `microservices/cloud-network/retired tenant_class adoption artifact:7-10` and `docs/products/cloud/PRD.md:138-176`.
4. µservice architecture anchor: local `ARCHITECTURE.md` absent; implementation shape inferred from `crates/cloud-network-domain/src/lib.rs:1-7`, `crates/cloud-network-vpc-api/src/lib.rs:1-17`, `crates/cloud-network-lb-api/src/lib.rs:1-16`, and `crates/cloud-network-dns-api/src/lib.rs:1-16`.
5. Documentation-rigor anchor: `docs/standards/documentation-rigor.md:40-83`, `docs/standards/documentation-rigor.md:175-190`, and `docs/standards/documentation-rigor.md:222-260`.

## §1 Tier definitions in Oyatie

- demo_trial is the minimum viable network substrate tier.
- demo_trial must support tenant-scoped VPC-equivalent isolation.
- demo_trial must support at least one route table.
- demo_trial must support basic security rules.
- demo_trial must support flow-log projection at a low-volume retention floor.
- demo_trial must support mTLS ingress only at the minimum load-balancer envelope.
- demo_trial must support DDoS runbook engagement but not dedicated mitigation capacity.
- demo_trial must support basic Cedar admission for network mutations.
- demo_trial must support context-specific deployment only where OpenTofu modules prove it.
- demo_trial for `guest-on-oci` must equal OCI Always Free, not a generic low-cost plan.
- demo_trial in the current tenant_class matrix incorrectly emphasizes shared AWS/GCP region VPC rather than OCI Always Free (`microservices/cloud-network/retired tenant_class adoption artifact:12-27`).
- paid is the paid baseline tier.
- paid should support dedicated tenant VPC or network namespace per region/cell.
- paid should support private endpoint access.
- paid should support 1 Gbps ingress/egress budgets where the context supports it.
- paid should support flow logs with operational retention and faster audit projection.
- paid should support BGP or provider route exchange as a managed option.
- paid should support per-tenant NAT and load balancer capacity budgets.
- paid should support provider paid features in guest contexts.
- paid should include explicit state backend and signed OpenTofu modules.
- paid should include CI evidence for Tier-1 OS deployment lanes.
- paid is the production-scale tier.
- paid should support per-cell VPC or equivalent network namespace.
- paid should support multi-region route health.
- paid should support advanced DDoS/WAF engagement.
- paid should support private endpoints, interconnect, and BGP route filtering.
- paid should support flow-log search and packet capture under compliance controls.
- paid should support 10 Gbps network budgets where hardware/provider allows.
- paid should support reachability analysis and effective-route inspection.
- paid should support compliance mappings such as FIPS packet path claims.
- paid should support documented rollback and route-convergence SLOs.
- paid is the hyperscaler/single-tenant-capable tier.
- paid should support dedicated VRF or equivalent isolation.
- paid should support large private address spaces and tenant-specific route domains.
- paid should support dedicated ingress, NAT, and load-balancing capacity.
- paid should support SR-IOV/DPDK or equivalent fast path where context permits.
- paid should support RPKI/BGP policy controls and dedicated interconnect.
- paid should support packet capture with explicit DPIA/compliance controls.
- paid should support near-real-time flow-log/audit projection.
- paid should support formal reachability and exposure analysis.
- paid should support single-tenant operational handoff and custom capacity contracts.
- paid should not be sold until measured evidence proves the relevant context can meet the target numbers in `performance-benchmark-numbers-2026-05-20.md`.

## §2 Counterpart tenant_class mapping

### AWS VPC tenant_class mapping

- AWS VPC does not expose demo_trial/paid tenant_class product tenant_classes for VPC itself.
- AWS base/free-equivalent tenant_class is default VPC networking, subnets, route tables, security groups, NACLs, internet gateway, and basic quotas.
- AWS paid baseline tenant_class maps to NAT Gateway, interface endpoints, gateway endpoints, load balancers, flow-log storage, and ordinary Site-to-Site VPN.
- AWS production tenant_class maps to Transit Gateway, Network Firewall, PrivateLink producer/consumer, centralized monitoring, and larger quotas.
- AWS dedicated/enterprise tenant_class maps to Direct at 10/100/400 Gbps, Cloud WAN, Network Manager, IPAM, high-scale peering, and dedicated account/network patterns.
- AWS emphasizes quota-managed scale, native provider integration, and broad managed feature coverage.
- AWS does not expose Cedar-native network authorization.
- AWS does not expose Oyatie-style tenant packet tags.
- AWS does not expose a Rust-strict customer-visible control implementation.
- AWS does provide concrete public quotas that Oyatie must match or explicitly reject.

### Google Cloud VPC tenant_class mapping

- Google Cloud VPC does not expose demo_trial/paid tenant_class VPC tiers.
- Google base/free-equivalent tenant_class maps to custom or auto VPC networks, regional subnets, routes, firewall rules, and internal connectivity.
- Google paid baseline tenant_class maps to Cloud NAT, Cloud VPN, flow logs, private services access, and internal/external load balancing.
- Google production tenant_class maps to Shared VPC, VPC Peering, Private Service Connect, Cloud Router, Interconnect, hierarchical firewall policies, and Packet Mirroring.
- Google dedicated/enterprise tenant_class maps to Network Connectivity Center, high-scale Interconnect, organization-level firewall and flow-log configuration, and advanced monitoring.
- Google emphasizes global VPC networks and regional subnet control.
- Google provides strong firewall hierarchy and flow-log governance.
- Google does not expose Oyatie-style Cedar policy gates or cell doctrine.
- Google does not map demo_trial to OCI Always Free.
- Google quotas and MTU/throughput references are useful design constraints for Oyatie paid/paid targets.

### Azure Virtual Network tenant_class mapping

- Azure Virtual Network does not expose demo_trial/paid tenant_class VNet tiers.
- Azure base/free-equivalent tenant_class maps to VNet, subnets, route tables, NSGs, DNS settings, and basic peering.
- Azure paid baseline tenant_class maps to NAT Gateway, VPN Gateway, Private Endpoints, Azure Load Balancer, Application Gateway, and Network Watcher diagnostics.
- Azure production tenant_class maps to ExpressRoute, Azure Firewall, DDoS Protection, Virtual Network Manager, large peering topologies, and flow logs.
- Azure dedicated/enterprise tenant_class maps to ExpressRoute FastPath, high-scale peering through Virtual Network Manager, hub-spoke service chaining, and dedicated network appliances.
- Azure emphasizes VNet peering, private endpoints, route tables, NSGs, and diagnostic tooling.
- Azure published flow and route limits are useful for Oyatie route/flow capacity modeling.
- Azure does not expose Oyatie-style mTLS-as-network-tenant_class semantics.
- Azure does not expose Rust-strict customer-visible control implementation.
- Azure offers a rich diagnostic surface that Oyatie currently lacks locally.

## §3 Per-Oyatie-tenant_class delta tables

### demo_trial tenant_class table

| Feature | Oyatie demo_trial | AWS base-equivalent | Google base-equivalent | Azure base-equivalent | Gap classification |
|---|---|---|---|---|---|
| Tenant VPC-equivalent network | shared or minimal dedicated; OCI must fit Always Free | VPC | VPC network | VNet | partial |
| CIDR assignment | basic CIDR fields | VPC CIDR | subnet ranges | address space | partial, no IPAM |
| IPv6 | claimed in FAQ | supported | supported | supported | needs contract tests |
| Subnets | per-cell subnet concept | subnets | regional subnets | subnets | partial |
| Route table | one basic table target | route tables | routes | route tables | partial |
| Static routes | limited | yes | yes | UDRs | partial |
| Dynamic routing | not demo_trial | no base | Cloud Router paid/advanced | gateway path | acceptable defer |
| Internet egress | NAT/proxy shared | IGW/NAT optional | Cloud NAT optional | NAT Gateway optional | partial |
| OCI Always Free ingress | must be 10 Mbps budget | not applicable | not applicable | not applicable | missing local docs |
| Security rules | basic SG model | security groups/NACLs | firewall rules | NSGs | partial |
| Firewall hierarchy | none | Network Firewall extra | hierarchy advanced | Azure Firewall extra | acceptable defer |
| Flow logs | low-volume target | VPC Flow Logs | VPC Flow Logs | VNet Flow Logs | partial, no schema |
| Audit-chain projection | target | no | no | no | additive |
| mTLS ingress | minimal | not VPC-native | not VPC-native | not VNet-native | additive but needs LB docs |
| Private endpoints | not demo_trial except minimal | endpoints optional | PSC optional | Private Link optional | acceptable defer |
| VPC peering | not demo_trial | peering | peering | peering | defer |
| Transit hub | not demo_trial | Transit Gateway extra | NCC/Cloud Router advanced | Virtual WAN/Manager extra | defer |
| DDoS protection | runbook only | Shield Standard baseline | baseline platform controls | Basic platform controls | partial |
| DDoS paid capacity | no | Shield Advanced | Cloud Armor extra | DDoS Protection extra | acceptable defer |
| WAF | no | AWS WAF extra | Cloud Armor extra | Azure WAF extra | defer |
| Reachability analysis | no | Reachability Analyzer | Connectivity Tests | Network Watcher | missing |
| Traffic mirroring | no | yes | yes | partial | defer |
| IPAM | no | AWS IPAM | partial | partial | missing |
| Quotas | no local | public quotas | public quotas | public limits | missing |
| Cost model | no local | pay-as-used | pay-as-used | pay-as-used | missing |
| OpenTofu module | absent | not applicable | not applicable | not applicable | canonical gap |
| OS manifest | absent | not applicable | not applicable | not applicable | canonical gap |
| Rust implementation | external crates | provider internal | provider internal | provider internal | Oyatie additive but unlinked |
| Supported contexts | undocumented | AWS only | GCP only | Azure only | canonical gap |
| Tenant packet tags | yes in tenant_class matrix | tags not packet semantics | tags not packet semantics | tags not packet semantics | additive |
| BGP | no | Direct Connect/VPN extras | Cloud Router extra | Gateway/ExpressRoute extra | acceptable defer |
| Interconnect | no | Direct paid | Interconnect paid | ExpressRoute paid | acceptable defer |
| Private DNS | partial via DNS API | endpoint private DNS | private zones | private DNS zone | ownership split |
| Onboarding | 181-line doc | official docs | official docs | official docs | below rigor floor |
| Incident runbooks | strong | managed docs | managed docs | managed docs | Oyatie strength |
| Compliance docs | absent | shared responsibility docs | compliance docs | compliance docs | missing |
| FIPS path | no | service-specific | service-specific | service-specific | defer |
| Packet capture | no | traffic mirroring | packet mirroring | watcher/capture | defer |
| Route rollback | runbook | provider operations | provider operations | provider operations | partial |
| demo_trial verdict | low-cost target with OCI constraint | mature base | mature base | mature base | catch-up |

### paid tenant_class table

| Feature | Oyatie paid | AWS paid-baseline | Google paid-baseline | Azure paid-baseline | Gap classification |
|---|---|---|---|---|---|
| Dedicated VPC/network namespace | intended | VPC/account patterns | custom VPC | VNet | parity target |
| CIDR governance | not local | IPAM optional | address management | address management | missing IPAM |
| Route table scale | 500 target | 500 default routes | route quotas | 600 UDRs | parity target |
| Security rule scale | 1,000 target | SG/NACL quotas | firewall quotas | NSG 1,000 max | parity target |
| NAT throughput | 1 Gbps target | NAT Gateway | Cloud NAT | NAT Gateway | parity target |
| Private endpoint consumer | intended | PrivateLink | PSC | Private Endpoint | missing API |
| Private endpoint producer | intended | Endpoint service | PSC producer | Private Link Service | missing API |
| Flow logs | operational | VPC Flow Logs | VPC Flow Logs | VNet Flow Logs | missing schema |
| Audit retention | target | CloudWatch/S3 configured | Cloud Logging configured | Log Analytics configured | needs compliance |
| Load balancer | LB API exists externally | ELB | Cloud Load Balancing | Azure Load Balancer/App Gateway | ownership-folder gap |
| mTLS config | LB API supports | service-specific | service-specific | service-specific | Oyatie additive |
| WAF reference | LB API WAF field | AWS WAF | Cloud Armor | Azure WAF | needs docs |
| VPN | intended | Site-to-Site/Client VPN | Cloud VPN | VPN Gateway | missing API/IaC |
| BGP | intended | VPN/DX/TGW | Cloud Router | VPN/ExpressRoute | needs design |
| VPC peering | intended | peering | peering | peering | needs API |
| Shared network | intended | VPC sharing | Shared VPC | cross-subscription patterns | missing model |
| DDoS | runbook | Shield Standard/Advanced | Armor/DDoS posture | DDoS Protection | product gap |
| Reachability | not present | Reachability Analyzer | Connectivity Tests | Network Watcher | missing |
| Effective routes | runbook command only | route table view | routes view | effective routes | needs API |
| Quotas | not local | public quotas | public quotas | public limits | missing |
| Cost budgets | not local | billing | billing | billing | missing |
| OpenTofu | absent | provider native | provider native | provider native | canonical gap |
| State backend | absent | provider state not customer-visible | provider state | provider state | canonical gap |
| Sigstore modules | absent | not customer-visible | not customer-visible | not customer-visible | canonical gap |
| OS manifest | absent | not customer-visible | not customer-visible | not customer-visible | canonical gap |
| OCI paid | paid escape from Always Free | not applicable | not applicable | not applicable | needs docs |
| On-prem paid | hardware dependent | hybrid through DX/VPN | hybrid through VPN/Interconnect | hybrid through VPN/ExpressRoute | needs IaC |
| Colo paid | cross-connect aware | DX | Interconnect | ExpressRoute | needs SKU |
| Tenant packet tags | present concept | no | no | no | additive |
| Cedar network policy | present concept | IAM/SG/NACL | IAM/firewall | RBAC/NSG | additive |
| Data classes | external API comments | no customer field | no customer field | no customer field | additive |
| Residency field | VPC API field | region/account | region/project | region/subscription | additive |
| Idempotency ledger | API crates | API idempotency varies | API idempotency varies | API idempotency varies | positive |
| CI lanes | absent | provider internal | provider internal | provider internal | canonical gap |
| Tutorial | present | official tutorials | official tutorials | official tutorials | useful but Make-drift |
| Migration | AWS/Istio only | native | native | native | missing breadth |
| paid verdict | achievable paid baseline | mature | mature | mature | catch-up to parity |

### paid tenant_class table

| Feature | Oyatie paid | AWS production-equivalent | Google production-equivalent | Azure production-equivalent | Gap classification |
|---|---|---|---|---|---|
| Per-cell VPC/network namespace | intended | multi-account/VPC patterns | Shared VPC/regional subnets | hub-spoke VNets | parity target |
| Multi-region routing | intended | Transit Gateway/Cloud WAN | dynamic routing/NCC | VNet peering/Virtual WAN | missing design |
| 10 Gbps budget | tenant_class matrix claim | DX 10 Gbps | Interconnect 10/100 Gbps classes | ExpressRoute classes | needs evidence |
| BGP route filtering | tenant_class claim | TGW/DX route policy | Cloud Router policy | ExpressRoute/gateway routes | needs API |
| RPKI | tenant_class claim | not base VPC | not base VPC | not base VNet | additive but unproven |
| DDoS mitigation | runbook | Shield Advanced option | Armor/partner options | DDoS Protection | product gap |
| WAF | implied | AWS WAF | Cloud Armor | Azure WAF | needs docs |
| Private service access | intended | PrivateLink | PSC | Private Link | missing API |
| Packet capture | tenant_class claim | Traffic Mirroring | Packet Mirroring | Network Watcher capture | needs compliance |
| Flow analytics | target | CloudWatch/Athena | Logging/BigQuery | Log Analytics | needs architecture |
| Reachability analysis | missing | Reachability Analyzer | Connectivity Tests | Network Watcher | missing |
| Network manager | missing | Network Manager | NCC | Virtual Network Manager | missing |
| IPAM | missing | AWS IPAM | partial | partial | missing |
| Route quota | not local | public quotas | public quotas | public limits | missing capacity model |
| Flow quota | not local | NAU/ENI limits | throughput notes | NIC flow limits | missing capacity model |
| Route convergence SLO | target | provider internal | provider internal | provider internal | needs measurement |
| Availability | 99.99 target | mature | mature | mature | needs evidence |
| Compliance mapping | tenant_class claim | compliance docs | compliance docs | compliance docs | missing local docs |
| FIPS path | tenant_class claim | service-specific | service-specific | service-specific | needs proof |
| Multi-context OpenTofu | absent | not applicable | not applicable | not applicable | canonical gap |
| State backend | absent | not applicable | not applicable | not applicable | canonical gap |
| Signed modules | absent | not applicable | not applicable | not applicable | canonical gap |
| Tier-1 OS support | absent | not applicable | not applicable | not applicable | canonical gap |
| On-prem paid | intended | hybrid | hybrid | hybrid | needs hardware profile |
| Colo paid | intended | DX | Interconnect | ExpressRoute | needs SKU/SLA |
| Native provider paid | intended | AWS native | GCP native | Azure native | needs Oyatie fabric |
| mTLS cascade runbook | present | not VPC-native | not VPC-native | not VNet-native | Oyatie strength |
| Cross-cell stall runbook | present | provider docs | provider docs | provider docs | Oyatie strength |
| DDoS runbook | present | provider docs | provider docs | provider docs | Oyatie strength |
| Cedar gates | concept | IAM/policies | IAM/policies | RBAC/policies | additive |
| Tenant audit | concept | logs | logs | logs | additive but needs schema |
| Migration breadth | AWS only | not needed | not needed | not needed | missing |
| Benchmark evidence | claimed but absent | public docs/benchmarks | public docs/benchmarks | public docs/benchmarks | needs evidence |
| paid verdict | strong concept, weak ownership artifacts | mature | mature | mature | parity only after buildout |

### paid tenant_class table

| Feature | Oyatie paid | AWS dedicated-equivalent | Google dedicated-equivalent | Azure dedicated-equivalent | Gap classification |
|---|---|---|---|---|---|
| Dedicated VRF/network namespace | intended | dedicated account/VPC/DX | dedicated projects/VPC/Interconnect | dedicated subscription/VNet/ExpressRoute | needs architecture |
| /16 or large tenant CIDR | tenant_class claim | VPC CIDR/IPAM | subnet/VPC ranges | address space | needs IPAM |
| 100-400 Gbps path | target | 100/400 Gbps DX | high-capacity Interconnect | high ExpressRoute capacity | needs evidence |
| SR-IOV/DPDK | tenant_class claim | EC2 ENA/EFA adjacent | high-perf NIC adjacent | accelerated networking adjacent | needs design |
| Dedicated BGP sessions | intended | DX/TGW | Cloud Router/Interconnect | ExpressRoute | needs API |
| RPKI enforcement | intended | advanced network policy | advanced network policy | advanced network policy | additive but unproven |
| MACsec | tenant_class claim | DX / link options | Interconnect options | Azure backbone MACsec statement | needs context docs |
| Packet capture retention | tenant_class claim | Traffic Mirroring | Packet Mirroring | Network Watcher | needs DPIA |
| Formal exposure analysis | missing | Network Access Analyzer | Connectivity Tests | Network Watcher | missing |
| Global topology manager | missing | Network Manager/Cloud WAN | NCC | Virtual Network Manager | missing |
| Private endpoint at scale | intended | PrivateLink | PSC | Private Link | missing API |
| Tenant-specific service producer | intended | endpoint service | PSC producer | Private Link Service | missing API |
| Ultra-low flow logs | target | CloudWatch/S3 tuned | Logging tuned | Log Analytics tuned | needs evidence |
| Dedicated DDoS | intended | Shield Advanced | Armor/partner | DDoS Protection | product gap |
| Dedicated WAF | intended | WAF | Cloud Armor | Azure WAF | needs docs |
| Single-tenant availability | target | enterprise design | enterprise design | enterprise design | needs evidence |
| Data residency | API field | region/account | region/project | region/subscription | needs compliance docs |
| Data classification | API comments | labels/tags | labels/tags | tags | additive |
| Cedar authorization | concept | IAM/policies | IAM/policies | RBAC/policies | additive |
| Tenant packet tags | concept | no | no | no | additive |
| Native provider mode | intended | AWS itself | GCP itself | Azure itself | requires Oyatie fabric |
| OpenTofu modules | absent | not comparable | not comparable | not comparable | canonical gap |
| OS matrix | absent | not comparable | not comparable | not comparable | canonical gap |
| Signed modules | absent | not comparable | not comparable | not comparable | canonical gap |
| Benchmark evidence | absent | public service experience | public service experience | public service experience | major gap |
| paid cost model | absent | enterprise cost | enterprise cost | enterprise cost | missing |
| paid compliance | absent | enterprise docs | enterprise docs | enterprise docs | missing |
| paid runbooks | partial | provider docs | provider docs | provider docs | partial |
| paid migration | absent | partner/pro services | partner/pro services | partner/pro services | missing |
| paid verdict | aspirational | mature | mature | mature | catch-up until measured |

## §4 OCI demo_trial tenant_class = Always Free reconciliation

- ADR-0328 §D-19 makes OCI Always Free a guest-on-oci sub-profile, not an optional marketing tier.
- ADR-0328 §D-19 says demo_trial for OCI means Always Free.
- The local tenant_class matrix demo_trial row does not mention OCI, Always Free, 4 OCPU, 24 GB RAM, 200 GB block, 10 GB object/archive, 2 autonomous databases, one VCN, one 10 Mbps load balancer, or 10 TB egress.
- The local tenant_class matrix demo_trial row mentions shared AWS/GCP region VPC and 100 Mbps / 5 GB per day, which is incompatible with the OCI demo_trial tenant_class definition.
- OCI demo_trial tenant_class network create must fit inside one VCN unless a documented Always Free-compatible split exists.
- OCI demo_trial tenant_class ingress/load-balancing must stay within the Always Free 10 Mbps load balancer envelope.
- OCI demo_trial tenant_class flow logs must avoid storage and logging volume that exceeds the Always Free object/archive budget.
- OCI demo_trial tenant_class route count should be smaller than generic demo_trial until measured.
- OCI demo_trial tenant_class security rule count should be smaller than generic demo_trial until measured.
- OCI demo_trial tenant_class private endpoints should be minimal or paid tenant_class unless Always Free compatibility is proven.
- OCI demo_trial tenant_class interconnect is out of scope because Always Free does not provide paid FastConnect-style capacity.
- OCI demo_trial tenant_class DDoS mitigation should be detection plus runbook engagement, not paid scrubbing capacity.
- OCI demo_trial tenant_class WAF should be paid tenant_class unless free-compatible capacity exists.
- OCI demo_trial tenant_class packet capture should be disabled by default because storage and compliance costs are not free.
- OCI demo_trial tenant_class high-retention flow logs should be paid tenant_class because retention consumes budget.
- OCI demo_trial tenant_class reachability analysis can remain enabled because it is control-plane compute if Rust-local and low-cost.
- OCI demo_trial tenant_class OpenTofu state backend must use OCI Object Storage plus lock per ADR-0328, not AWS S3, Terraform Cloud, GitHub artifacts, or local state.
- OCI demo_trial tenant_class module path must be `microservices/cloud-network/iac/guest-on-oci/always-free/`.
- OCI demo_trial tenant_class must emit budget telemetry before and after plan/apply.
- OCI demo_trial tenant_class must fail closed when a plan exceeds Always Free envelope.
- OCI demo_trial tenant_class should expose a clear upgrade path to paid when tenants require more than 10 Mbps ingress.
- OCI demo_trial tenant_class should expose a clear upgrade path to paid for dedicated private endpoints.
- OCI demo_trial tenant_class should expose a clear upgrade path to paid for high-retention flow logs.
- OCI demo_trial tenant_class should expose a clear upgrade path to paid for WAF/DDoS paid controls.
- OCI demo_trial tenant_class should expose a clear upgrade path to paid for larger route/security-rule quotas.
- OCI demo_trial tenant_class should not claim parity with AWS/GCP/Azure paid VPC products.
- OCI demo_trial tenant_class should claim cost-disciplined viability only.
- Current status: non-compliant until tenant_class matrix and IaC modules are updated.

## §5 Findings: per-tenant_class ahead/parity/catch-up classifications

- demo_trial ahead: Cedar-gated network policy concept is ahead of default VPC products.
- demo_trial ahead: tenant packet tags are ahead if implemented.
- demo_trial ahead: audit-chain flow projection is ahead if implemented.
- demo_trial parity: basic VPC/network, subnet, route, and security-rule concepts are parity targets.
- demo_trial catch-up: IPAM, private endpoints, reachability, quotas, OpenTofu, OS manifest, and OCI reconciliation.
- demo_trial catch-up: measured evidence does not exist.
- demo_trial offerability: guest-on-oci demo_trial should not be offered until Always Free module exists.
- paid ahead: mTLS-as-network-tenant_class semantics can be ahead of base VPC products.
- paid ahead: data-class and residency fields in API crates are stronger than typical VPC create payloads.
- paid parity: paid NAT, load balancer, VPN, private endpoint, and flow log goals can match public cloud baselines.
- paid catch-up: private endpoint producer model, route/flow quotas, state backend, signed OpenTofu modules, and OS matrix.
- paid offerability: paid can become credible once OpenTofu and ownership docs land.
- paid ahead: cell-aware routing and Cedar/audit integration can be differentiators.
- paid parity: 10 Gbps, BGP filtering, DDoS/WAF, flow analytics, and private service access are public-cloud production expectations.
- paid catch-up: topology manager, reachability analyzer, IPAM, packet capture compliance, and measured route-convergence evidence.
- paid offerability: do not claim production scale until capacity model and SLOs are in place.
- paid ahead: dedicated tenant policy/audit/mTLS/cell semantics could be stronger than generic VPC products.
- paid parity: dedicated VRF, 100+ Gbps path, interconnect, DDoS, WAF, and single-tenant operational handling match enterprise cloud expectations.
- paid catch-up: native fabric, SR-IOV/DPDK proof, 400 Gbps evidence, exposure analyzer, enterprise compliance, and cost model.
- paid offerability: aspirational only until `oyatie-as-cloud-provider` native network fabric is measured.
- Cross-tenant_class P1: missing PRD/ARCH makes every tenant_class less governable.
- Cross-tenant_class P1: missing OpenTofu modules makes every deployment context unproven.
- Cross-tenant_class P1: missing OS manifest makes runtime support unproven.
- Cross-tenant_class P1: missing OCI Always Free reconciliation makes demo_trial wrong for guest-on-oci.
- Cross-tenant_class P2: Make-first examples should be corrected before they become canonical delivery paths.
- Cross-tenant_class P2: benchmark claims must be downgraded or evidenced.
- Cross-tenant_class P2: FAQ provider-wrapper language should be rewritten to portable semantics.
- Cross-tenant_class positive: Rust API crates provide a credible implementation foundation.
- Cross-tenant_class positive: runbooks provide real operational depth.
- Cross-tenant_class positive: the tenant_class vocabulary is a useful scaffold once canonical gaps are repaired.
- Final tenant_class verdict: demo_trial catch-up, paid catch-up-to-parity, paid parity-target-with-gaps, paid aspirational.

## §6 Required tenant_class-document changes

- Add `OCI demo_trial tenant_class = Always Free` row to tenant_class matrix.
- Replace demo_trial shared AWS/GCP wording with context-specific demo_trial envelopes.
- Add private endpoint axis across paid, paid, and paid.
- Add IPAM axis across all tiers.
- Add reachability-analysis axis across all tiers.
- Add traffic-mirroring / packet-capture axis only for paid/paid with compliance controls.
- Add route quota axis.
- Add security rule quota axis.
- Add flow quota axis.
- Add NAT throughput axis.
- Add flow-log retention axis.
- Add route-convergence SLO axis.
- Add OpenTofu module readiness axis.
- Add OS support readiness axis.
- Add state backend readiness axis.
- Add signed-module readiness axis.
- Add measured benchmark evidence axis.
- Add per-context deployment support row for all six contexts.
- Add on-prem and colo hardware assumptions.
- Add `oyatie-as-cloud-provider` native fabric assumptions.
- Add explicit paid escape hatches for every OCI demo_trial tenant_class limit.

## §7 Non-offerability gates

- demo_trial guest-on-oci is not offerable until the Always Free OpenTofu module exists and fails closed on paid-resource drift.
- paid is not offerable until private endpoint, NAT, flow-log, and state-backend behavior are documented per context.
- paid is not offerable until route convergence, DDoS/WAF, packet-capture, and reachability evidence are measured against signed runs.
- paid is not offerable until native fabric, dedicated VRF, 100+ Gbps path, exposure analysis, and compliance artifacts are proven.
- All tenant_classes are not offerable as canonical products until local `PRD.md`, `ARCHITECTURE.md`, `supported-oses.json`, `slos/`, `iac/`, `capacity-model.md`, `cost-budget.md`, and `cross-microservice-handoffs.md` exist or are explicitly replaced by machine-readable equivalents.
