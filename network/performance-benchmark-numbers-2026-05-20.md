# cloud-network performance benchmark numbers — 2026-05-20

## Header anchor block

1. Canonical sequence anchor: `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-4146`.
2. Machine control anchor: `specs/master-plan-sequencing.json:704-868`.
3. µservice PRD anchor: local `PRD.md` absent; purpose inferred from `microservices/cloud-network/retired tenant_class adoption artifact:7-10` and `docs/products/cloud/PRD.md:138-176`.
4. µservice architecture anchor: local `ARCHITECTURE.md` absent; implementation shape inferred from `crates/oya-cloud-network-domain/src/lib.rs:1-7` and the VPC/LB/DNS API crates.
5. Documentation-rigor anchor: `docs/standards/documentation-rigor.md:40-83`, `docs/standards/documentation-rigor.md:175-190`, and `docs/standards/documentation-rigor.md:222-260`.

## Explicit methodology disclosure

- These Oyatie numbers are target numbers and planning budgets.
- They are not measured benchmark results.
- The existing local benchmark doc claims measured results (`microservices/cloud-network/benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md:3-17`), but the named evidence path was not present during this audit (`microservices/cloud-network/benchmarks/cloud-network-vs-aws-vpc-vs-gcp-vpc-vs-azure-vnet-vs-cilium-mesh.md:92-101`).
- Measured benchmarks must be added in a later build phase with signed evidence, OS/arch disclosure, deployment context, tenant class, tool version, raw data, and replay instructions.
- ADR-0328 §D-20 requires benchmark claims to disclose OS, arch, deployment context, and tenant class (`docs/decisions/ADR-0700-ci-admission-live-apex.md:3938-4001`).
- ADR-0328 §D-19 constrains OCI demo_trial tenant_class to OCI Always Free for guest-on-oci (`docs/decisions/ADR-0700-ci-admission-live-apex.md:3418-3438`).
- The target values below are intentionally conservative for demo_trial and aggressive only at paid.
- The purpose is to create an engineering budget that can later be measured, not to claim current runtime performance.

## §1 Methodology

- Benchmark dimension 1: control-plane latency p50, p95, and p99 for create/update/delete operations.
- Benchmark dimension 2: route-convergence p50, p95, and p99 after route table or BGP policy changes.
- Benchmark dimension 3: flow-log delivery p50, p95, and p99 from packet observation to audit projection.
- Benchmark dimension 4: reachability-analysis p50, p95, and p99 for modeled path queries.
- Benchmark dimension 5: API throughput in requests per second for VPC/LB/private-endpoint mutations.
- Benchmark dimension 6: concurrent operations for tenant-scoped network mutations.
- Benchmark dimension 7: scale ceilings for routes, rules, subnets, peerings, flows, and endpoint attachments.
- Benchmark dimension 8: data-plane bandwidth budgets for NAT, ingress load balancing, and interconnect.
- Benchmark dimension 9: failure recovery objectives for route rollback, DDoS mitigation, and mTLS cascade containment.
- Benchmark dimension 10: availability targets by tenant_class and context.
- Workload A: create a VPC-equivalent network with IPv4/IPv6 CIDRs, one route table, and two security groups.
- Workload B: add 100 routes and apply a policy-gated route update.
- Workload C: create an mTLS-enabled load balancer with two listeners and two target groups.
- Workload D: create a private endpoint bound to a DNS zone and VPC.
- Workload E: run reachability analysis between two tenant workloads through routes, security rules, and Cedar decisions.
- Workload F: ingest and project 1 million flow-log records to audit-chain.
- Workload G: simulate cross-cell route stall and measure time to healthy route convergence after mitigation.
- Workload H: simulate DDoS mitigation engagement and measure time from alert to active mitigation.
- Workload I: simulate mTLS tenant CA failure and measure time to contain retry storm.
- OS disclosure for future measured runs: every Tier-1 OS row in ADR-0328 D-17 must be represented or explicitly scoped out.
- Architecture disclosure for future measured runs: x86_64 and aarch64 must be represented where deployable.
- Deployment contexts for future measured runs: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.
- Tenant classes for future measured runs: single-tenant demo_trial, shared-tenant demo_trial, dedicated paid, production paid, and single-tenant paid.
- Isolation disclosure for future measured runs: shared VPC/network namespace, dedicated tenant VPC, per-cell VPC, dedicated VRF, and SR-IOV/DPDK path where applicable.
- Measurement toolchain target: Rust-owned load and validation binaries, not Python/JS/Go scripts.
- IaC setup target: OpenTofu plans through cloud-iac, not manual console or Crossplane.
- Evidence target: signed artifact bundle with raw metrics, replay config, seed, commit SHA, OS, arch, context, tenant tier, and tool version.

## §2 Counterpart numbers

### AWS VPC public numbers

| Number | Value | Source / provenance |
|---|---:|---|
| Route tables per VPC default | 200 | AWS VPC quotas |
| Routes per route table default | 500 | AWS VPC quotas |
| Routes per route table maximum after increase | 1,000 | AWS VPC quotas |
| Propagated routes per route table | 100 | AWS VPC quotas |
| Route servers per VPC | 5 | AWS VPC quotas |
| Route server endpoints per route server | 10 | AWS VPC quotas |
| Peering sessions per network interface | 20 | AWS VPC quotas |
| Route server endpoints per route server and subnet | 2 | AWS VPC quotas |
| Routes per route server peer | 100 | AWS VPC quotas |
| VPC security groups per Region | 2,500 | AWS VPC quotas |
| Inbound rules per security group default | 60 | AWS VPC quotas |
| Outbound rules per security group default | 60 | AWS VPC quotas |
| Security groups per network interface default | 5 | AWS VPC quotas |
| Security groups per network interface maximum | 16 | AWS VPC quotas |
| Participant accounts per shared VPC | 100 | AWS VPC quotas |
| Subnets shareable with an account | 100 | AWS VPC quotas |
| Network Address Usage default | 64,000 | AWS VPC quotas |
| Network Address Usage maximum after increase | 256,000 | AWS VPC quotas |
| Peered Network Address Usage default | 128,000 | AWS VPC quotas |
| Peered Network Address Usage maximum after increase | 512,000 | AWS VPC quotas |
| Direct dedicated port speeds | 1/10/100/400 Gbps | AWS Direct docs |

### Google Cloud VPC public numbers

| Number | Value | Source / provenance |
|---|---:|---|
| Primary IPv4 range per subnet | 1 | Google VPC quotas |
| Secondary IPv4 ranges per subnet | 170 | Google VPC quotas |
| Network tags per static route | 256 | Google VPC quotas |
| Allocated IP ranges per private connection | 5,000 | Google VPC quotas |
| MTU minimum | 1,300 bytes | Google VPC quotas |
| MTU default common value | 1,460 bytes | Google VPC quotas |
| MTU standard Ethernet common value | 1,500 bytes | Google VPC quotas |
| MTU jumbo maximum | 8,896 bytes | Google VPC quotas |
| Alias IP ranges per network interface | 150 | Google VPC quotas |
| Idle TCP connection duration | 10 minutes | Google VPC quotas |
| External egress all-flow sustained | about 7 Gbps | Google VPC quotas |
| External egress all-flow Tier_1 | 25 Gbps | Google VPC quotas |
| External egress single-flow sustained | 3 Gbps | Google VPC quotas |
| External ingress to external IP destination | no more than 30 Gbps | Google VPC quotas |
| Firewall log entries on f1-micro | 100 connections per 5 seconds | Google VPC quotas |
| Firewall log entries on g1-small | 250 connections per 5 seconds | Google VPC quotas |
| Firewall log entries on 1-8 vCPU instances | 500 connections per vCPU per 5 seconds | Google VPC quotas |
| VPC Flow Logs default Logging retention | 30 days | Google VPC Flow Logs docs |

### Azure Virtual Network public numbers

| Number | Value | Source / provenance |
|---|---:|---|
| Virtual networks per subscription | 100 | Azure service limits |
| Local network sites default | 20 | Azure service limits |
| Local network sites maximum | 50 | Azure service limits |
| DNS servers per virtual network | 20 | Azure service limits |
| Private IP addresses per virtual network | 4,096 | Azure service limits |
| Concurrent TCP/UDP flows per NIC | 500,000 | Azure service limits |
| Concurrent TCP/UDP flows with two or more NICs | up to 1,000,000 | Azure service limits |
| Network Security Groups | 200 | Azure service limits |
| NSG rules per NSG default | 200 | Azure service limits |
| NSG rules per NSG maximum | 1,000 | Azure service limits |
| User-defined route tables | 200 | Azure service limits |
| User-defined routes per route table | 600 | Azure service limits |
| VNet peerings default | up to 500 | Azure VNet peering docs |
| VNet peerings with Virtual Network Manager | up to 1,000 | Azure VNet peering docs |
| Network Watcher capability classes | monitoring, diagnostics, traffic | Azure Network Watcher docs |

## §3 Oyatie target numbers by tenant_class and context

### demo_trial target envelope

| Context | Metric | demo_trial target | Notes |
|---|---|---:|---|
| oyatie-public-cloud | VPC create API p99 | 750 ms | control-plane receipt, not data-plane ready |
| oyatie-public-cloud | Route update p99 | 900 ms | single tenant route-table mutation |
| oyatie-public-cloud | LB create p99 | 900 ms | API accepted state |
| oyatie-public-cloud | Reachability query p95 | 2.5 s | modeled path under 500 nodes |
| oyatie-public-cloud | Flow-log delivery p95 | 90 s | audit projection target |
| oyatie-public-cloud | Route convergence p95 | 30 s | intra-cell only |
| oyatie-public-cloud | Per-tenant routes | 100 | demo_trial guardrail |
| oyatie-public-cloud | Per-tenant security rules | 200 | demo_trial guardrail |
| oyatie-public-cloud | NAT/ingress throughput | 100 Mbps | aligns current tenant_class prose |
| oyatie-public-cloud | Availability | 99.5% | pre-production/basic |
| guest-on-aws | VPC create API p99 | 800 ms | OpenTofu module receipt target |
| guest-on-aws | Route update p99 | 1.0 s | includes provider adapter queue |
| guest-on-aws | LB create p99 | 1.0 s | API accepted state |
| guest-on-aws | Reachability query p95 | 3.0 s | modeled path under 500 nodes |
| guest-on-aws | Flow-log delivery p95 | 120 s | cloud log export path |
| guest-on-aws | Route convergence p95 | 45 s | provider route propagation budget |
| guest-on-aws | Per-tenant routes | 100 | default demo_trial |
| guest-on-aws | Per-tenant security rules | 200 | default demo_trial |
| guest-on-aws | NAT/ingress throughput | 100 Mbps | shared budget |
| guest-on-aws | Availability | 99.5% | basic |
| guest-on-oci | VPC create API p99 | 900 ms | Always Free constrained |
| guest-on-oci | Route update p99 | 1.2 s | Always Free constrained |
| guest-on-oci | LB create p99 | 1.2 s | only if Always Free LB envelope fits |
| guest-on-oci | Reachability query p95 | 3.5 s | small graph budget |
| guest-on-oci | Flow-log delivery p95 | 180 s | keep object/log use low |
| guest-on-oci | Route convergence p95 | 60 s | Always Free constrained |
| guest-on-oci | Per-tenant routes | 50 | demo_trial OCI guardrail |
| guest-on-oci | Per-tenant security rules | 100 | demo_trial OCI guardrail |
| guest-on-oci | NAT/ingress throughput | 10 Mbps | OCI Always Free load balancer budget |
| guest-on-oci | Availability | 99.0% | no paid redundancy assumption |
| on-prem | VPC create API p99 | 1.0 s | local control receipt |
| on-prem | Route update p99 | 1.5 s | depends on local route server |
| on-prem | LB create p99 | 1.2 s | local ingress |
| on-prem | Reachability query p95 | 3.0 s | local graph |
| on-prem | Flow-log delivery p95 | 120 s | local audit sink |
| on-prem | Route convergence p95 | 60 s | local BGP/FRR budget |
| on-prem | Per-tenant routes | 75 | conservative |
| on-prem | Per-tenant security rules | 150 | conservative |
| on-prem | NAT/ingress throughput | 100 Mbps | hardware-dependent floor |
| on-prem | Availability | 99.0% | depends on operator infra |
| colo | VPC create API p99 | 1.0 s | local control receipt |
| colo | Route update p99 | 1.5 s | BGP/route-server budget |
| colo | LB create p99 | 1.2 s | local ingress |
| colo | Reachability query p95 | 3.0 s | local graph |
| colo | Flow-log delivery p95 | 120 s | local audit sink |
| colo | Route convergence p95 | 45 s | cross-connect route budget |
| colo | Per-tenant routes | 100 | demo_trial colo |
| colo | Per-tenant security rules | 200 | demo_trial colo |
| colo | NAT/ingress throughput | 250 Mbps | colo baseline |
| colo | Availability | 99.5% | if redundant cross-connect exists |
| oyatie-as-cloud-provider | VPC create API p99 | 600 ms | native control plane |
| oyatie-as-cloud-provider | Route update p99 | 750 ms | native path |
| oyatie-as-cloud-provider | LB create p99 | 750 ms | native path |
| oyatie-as-cloud-provider | Reachability query p95 | 2.0 s | native graph |
| oyatie-as-cloud-provider | Flow-log delivery p95 | 60 s | native audit |
| oyatie-as-cloud-provider | Route convergence p95 | 20 s | native cell |
| oyatie-as-cloud-provider | Per-tenant routes | 150 | native demo_trial |
| oyatie-as-cloud-provider | Per-tenant security rules | 300 | native demo_trial |
| oyatie-as-cloud-provider | NAT/ingress throughput | 250 Mbps | native demo_trial |
| oyatie-as-cloud-provider | Availability | 99.7% | native platform floor |

### paid target envelope

| Context | Metric | paid target | Notes |
|---|---|---:|---|
| oyatie-public-cloud | VPC create API p99 | 500 ms | aligns cloud PRD control target |
| oyatie-public-cloud | Route update p99 | 600 ms | paid baseline |
| oyatie-public-cloud | LB create p99 | 600 ms | paid baseline |
| oyatie-public-cloud | Reachability query p95 | 1.5 s | graph under 2,000 nodes |
| oyatie-public-cloud | Flow-log delivery p95 | 60 s | near-real-time |
| oyatie-public-cloud | Route convergence p95 | 15 s | intra-region |
| oyatie-public-cloud | Per-tenant routes | 500 | paid guardrail |
| oyatie-public-cloud | Per-tenant security rules | 1,000 | paid guardrail |
| oyatie-public-cloud | NAT/ingress throughput | 1 Gbps | tenant_class matrix aligned |
| oyatie-public-cloud | Availability | 99.9% | paid baseline |
| guest-on-aws | VPC create API p99 | 550 ms | provider adapter |
| guest-on-aws | Route update p99 | 800 ms | provider adapter |
| guest-on-aws | LB create p99 | 800 ms | provider adapter |
| guest-on-aws | Reachability query p95 | 2.0 s | graph under 2,000 nodes |
| guest-on-aws | Flow-log delivery p95 | 90 s | provider log export |
| guest-on-aws | Route convergence p95 | 20 s | provider route propagation |
| guest-on-aws | Per-tenant routes | 500 | paid |
| guest-on-aws | Per-tenant security rules | 1,000 | paid |
| guest-on-aws | NAT/ingress throughput | 1 Gbps | paid baseline |
| guest-on-aws | Availability | 99.9% | paid baseline |
| guest-on-oci | VPC create API p99 | 650 ms | paid OCI baseline |
| guest-on-oci | Route update p99 | 900 ms | paid OCI baseline |
| guest-on-oci | LB create p99 | 900 ms | paid OCI baseline |
| guest-on-oci | Reachability query p95 | 2.5 s | graph under 2,000 nodes |
| guest-on-oci | Flow-log delivery p95 | 90 s | paid log path |
| guest-on-oci | Route convergence p95 | 30 s | paid baseline |
| guest-on-oci | Per-tenant routes | 300 | paid OCI |
| guest-on-oci | Per-tenant security rules | 600 | paid OCI |
| guest-on-oci | NAT/ingress throughput | 500 Mbps | paid OCI |
| guest-on-oci | Availability | 99.8% | paid baseline |
| on-prem | VPC create API p99 | 700 ms | local HA controller |
| on-prem | Route update p99 | 1.0 s | local route server |
| on-prem | LB create p99 | 900 ms | local ingress |
| on-prem | Reachability query p95 | 2.0 s | graph under 2,000 nodes |
| on-prem | Flow-log delivery p95 | 90 s | local audit path |
| on-prem | Route convergence p95 | 30 s | local BGP |
| on-prem | Per-tenant routes | 300 | paid |
| on-prem | Per-tenant security rules | 600 | paid |
| on-prem | NAT/ingress throughput | 1 Gbps | hardware baseline |
| on-prem | Availability | 99.8% | operator-dependent |
| colo | VPC create API p99 | 650 ms | colo HA controller |
| colo | Route update p99 | 900 ms | route-server path |
| colo | LB create p99 | 850 ms | colo ingress |
| colo | Reachability query p95 | 2.0 s | graph under 2,000 nodes |
| colo | Flow-log delivery p95 | 75 s | local audit path |
| colo | Route convergence p95 | 20 s | redundant BGP |
| colo | Per-tenant routes | 500 | paid colo |
| colo | Per-tenant security rules | 1,000 | paid colo |
| colo | NAT/ingress throughput | 1 Gbps | cross-connect aware |
| colo | Availability | 99.9% | redundant colo |
| oyatie-as-cloud-provider | VPC create API p99 | 400 ms | native |
| oyatie-as-cloud-provider | Route update p99 | 500 ms | native |
| oyatie-as-cloud-provider | LB create p99 | 500 ms | native |
| oyatie-as-cloud-provider | Reachability query p95 | 1.0 s | native graph |
| oyatie-as-cloud-provider | Flow-log delivery p95 | 30 s | native audit |
| oyatie-as-cloud-provider | Route convergence p95 | 10 s | native route fabric |
| oyatie-as-cloud-provider | Per-tenant routes | 1,000 | native paid |
| oyatie-as-cloud-provider | Per-tenant security rules | 2,000 | native paid |
| oyatie-as-cloud-provider | NAT/ingress throughput | 2 Gbps | native paid |
| oyatie-as-cloud-provider | Availability | 99.95% | native paid |

### paid target envelope

| Context | Metric | paid target | Notes |
|---|---|---:|---|
| oyatie-public-cloud | VPC create API p99 | 350 ms | production |
| oyatie-public-cloud | Route update p99 | 450 ms | production |
| oyatie-public-cloud | LB create p99 | 450 ms | production |
| oyatie-public-cloud | Reachability query p95 | 750 ms | graph under 10,000 nodes |
| oyatie-public-cloud | Flow-log delivery p95 | 15 s | operational |
| oyatie-public-cloud | Route convergence p95 | 5 s | fast path |
| oyatie-public-cloud | Per-tenant routes | 5,000 | production |
| oyatie-public-cloud | Per-tenant security rules | 10,000 | production |
| oyatie-public-cloud | NAT/ingress throughput | 10 Gbps | tenant_class matrix aligned |
| oyatie-public-cloud | Availability | 99.99% | production |
| guest-on-aws | VPC create API p99 | 400 ms | production |
| guest-on-aws | Route update p99 | 600 ms | adapter |
| guest-on-aws | LB create p99 | 600 ms | adapter |
| guest-on-aws | Reachability query p95 | 900 ms | provider graph |
| guest-on-aws | Flow-log delivery p95 | 30 s | export path |
| guest-on-aws | Route convergence p95 | 10 s | provider route |
| guest-on-aws | Per-tenant routes | 2,500 | provider-aware |
| guest-on-aws | Per-tenant security rules | 5,000 | provider-aware |
| guest-on-aws | NAT/ingress throughput | 10 Gbps | paid |
| guest-on-aws | Availability | 99.99% | production |
| guest-on-oci | VPC create API p99 | 500 ms | paid production |
| guest-on-oci | Route update p99 | 700 ms | paid production |
| guest-on-oci | LB create p99 | 700 ms | paid production |
| guest-on-oci | Reachability query p95 | 1.2 s | provider graph |
| guest-on-oci | Flow-log delivery p95 | 45 s | paid log path |
| guest-on-oci | Route convergence p95 | 15 s | provider route |
| guest-on-oci | Per-tenant routes | 1,500 | paid OCI |
| guest-on-oci | Per-tenant security rules | 3,000 | paid OCI |
| guest-on-oci | NAT/ingress throughput | 5 Gbps | paid OCI |
| guest-on-oci | Availability | 99.95% | paid production |
| on-prem | VPC create API p99 | 500 ms | production local |
| on-prem | Route update p99 | 750 ms | local route fabric |
| on-prem | LB create p99 | 650 ms | local ingress |
| on-prem | Reachability query p95 | 1.2 s | graph under 10,000 nodes |
| on-prem | Flow-log delivery p95 | 30 s | local audit |
| on-prem | Route convergence p95 | 10 s | local BGP |
| on-prem | Per-tenant routes | 1,500 | operator floor |
| on-prem | Per-tenant security rules | 3,000 | operator floor |
| on-prem | NAT/ingress throughput | 10 Gbps | hardware required |
| on-prem | Availability | 99.9% | operator-dependent |
| colo | VPC create API p99 | 450 ms | production colo |
| colo | Route update p99 | 600 ms | redundant route servers |
| colo | LB create p99 | 600 ms | colo ingress |
| colo | Reachability query p95 | 900 ms | graph under 10,000 nodes |
| colo | Flow-log delivery p95 | 20 s | local audit |
| colo | Route convergence p95 | 5 s | BGP tuned |
| colo | Per-tenant routes | 3,000 | colo paid |
| colo | Per-tenant security rules | 6,000 | colo paid |
| colo | NAT/ingress throughput | 10 Gbps | cross-connect aware |
| colo | Availability | 99.99% | redundant colo |
| oyatie-as-cloud-provider | VPC create API p99 | 250 ms | native production |
| oyatie-as-cloud-provider | Route update p99 | 300 ms | native production |
| oyatie-as-cloud-provider | LB create p99 | 300 ms | native production |
| oyatie-as-cloud-provider | Reachability query p95 | 500 ms | native graph |
| oyatie-as-cloud-provider | Flow-log delivery p95 | 5 s | native audit |
| oyatie-as-cloud-provider | Route convergence p95 | 2 s | native route fabric |
| oyatie-as-cloud-provider | Per-tenant routes | 10,000 | native paid |
| oyatie-as-cloud-provider | Per-tenant security rules | 25,000 | native paid |
| oyatie-as-cloud-provider | NAT/ingress throughput | 25 Gbps | native paid |
| oyatie-as-cloud-provider | Availability | 99.99% | native production |

### paid target envelope

| Context | Metric | paid target | Notes |
|---|---|---:|---|
| oyatie-public-cloud | VPC create API p99 | 250 ms | hyperscaler bar |
| oyatie-public-cloud | Route update p99 | 300 ms | hyperscaler bar |
| oyatie-public-cloud | LB create p99 | 300 ms | hyperscaler bar |
| oyatie-public-cloud | Reachability query p95 | 400 ms | graph under 50,000 nodes |
| oyatie-public-cloud | Flow-log delivery p95 | 5 s | near-real-time |
| oyatie-public-cloud | Route convergence p95 | 1 s | fabric optimized |
| oyatie-public-cloud | Per-tenant routes | 25,000 | dedicated tenant |
| oyatie-public-cloud | Per-tenant security rules | 100,000 | dedicated tenant |
| oyatie-public-cloud | NAT/ingress throughput | 100 Gbps | dedicated path |
| oyatie-public-cloud | Availability | 99.995% | single-tenant capable |
| guest-on-aws | VPC create API p99 | 300 ms | provider ceiling aware |
| guest-on-aws | Route update p99 | 450 ms | provider ceiling aware |
| guest-on-aws | LB create p99 | 450 ms | provider ceiling aware |
| guest-on-aws | Reachability query p95 | 600 ms | local modeled graph |
| guest-on-aws | Flow-log delivery p95 | 10 s | export optimized |
| guest-on-aws | Route convergence p95 | 3 s | provider dependent |
| guest-on-aws | Per-tenant routes | 10,000 | may require provider quotas |
| guest-on-aws | Per-tenant security rules | 50,000 | may require provider quotas |
| guest-on-aws | NAT/ingress throughput | 100 Gbps | requires paid provider design |
| guest-on-aws | Availability | 99.99% | provider dependent |
| guest-on-oci | VPC create API p99 | 400 ms | paid high-end OCI |
| guest-on-oci | Route update p99 | 550 ms | paid high-end OCI |
| guest-on-oci | LB create p99 | 550 ms | paid high-end OCI |
| guest-on-oci | Reachability query p95 | 800 ms | local modeled graph |
| guest-on-oci | Flow-log delivery p95 | 20 s | paid log path |
| guest-on-oci | Route convergence p95 | 5 s | provider dependent |
| guest-on-oci | Per-tenant routes | 5,000 | may require provider quotas |
| guest-on-oci | Per-tenant security rules | 20,000 | may require provider quotas |
| guest-on-oci | NAT/ingress throughput | 25 Gbps | paid provider design |
| guest-on-oci | Availability | 99.99% | provider dependent |
| on-prem | VPC create API p99 | 350 ms | dedicated controllers |
| on-prem | Route update p99 | 500 ms | dedicated fabric |
| on-prem | LB create p99 | 500 ms | dedicated ingress |
| on-prem | Reachability query p95 | 700 ms | local graph |
| on-prem | Flow-log delivery p95 | 10 s | local audit |
| on-prem | Route convergence p95 | 3 s | tuned BGP/EVPN |
| on-prem | Per-tenant routes | 10,000 | hardware validated |
| on-prem | Per-tenant security rules | 50,000 | hardware validated |
| on-prem | NAT/ingress throughput | 100 Gbps | dedicated appliance/fabric |
| on-prem | Availability | 99.99% | customer infra dependent |
| colo | VPC create API p99 | 300 ms | dedicated controllers |
| colo | Route update p99 | 400 ms | dedicated route servers |
| colo | LB create p99 | 450 ms | dedicated ingress |
| colo | Reachability query p95 | 600 ms | local graph |
| colo | Flow-log delivery p95 | 8 s | local audit |
| colo | Route convergence p95 | 2 s | tuned BGP |
| colo | Per-tenant routes | 25,000 | dedicated tenant |
| colo | Per-tenant security rules | 100,000 | dedicated tenant |
| colo | NAT/ingress throughput | 100 Gbps | dedicated ports |
| colo | Availability | 99.995% | redundant colo |
| oyatie-as-cloud-provider | VPC create API p99 | 150 ms | native hyperscaler |
| oyatie-as-cloud-provider | Route update p99 | 200 ms | native hyperscaler |
| oyatie-as-cloud-provider | LB create p99 | 200 ms | native hyperscaler |
| oyatie-as-cloud-provider | Reachability query p95 | 250 ms | native graph |
| oyatie-as-cloud-provider | Flow-log delivery p95 | 1 s | native audit |
| oyatie-as-cloud-provider | Route convergence p95 | 500 ms | native fabric |
| oyatie-as-cloud-provider | Per-tenant routes | 100,000 | native paid |
| oyatie-as-cloud-provider | Per-tenant security rules | 250,000 | native paid |
| oyatie-as-cloud-provider | NAT/ingress throughput | 400 Gbps | native dedicated path |
| oyatie-as-cloud-provider | Availability | 99.999% | native hyperscaler target |

## §4 Per-context overlay

- `oyatie-public-cloud` should be the reference paid public deployment and should target parity with hyperscaler control-plane p99 budgets by paid.
- `guest-on-aws` should respect AWS route, security group, and NAU quotas; paid may require AWS quota increases and paid Direct Connect/Transit Gateway design.
- `guest-on-oci` demo_trial is the only context with a canonical zero-cost requirement; its 10 Mbps ingress/LB budget is intentionally much lower than other demo_trial contexts.
- `guest-on-oci` paid and above should exit Always Free constraints explicitly and document paid OCI capacity.
- `on-prem` numbers are only valid when local hardware, route servers, storage, and observability meet the declared floor.
- `colo` numbers assume redundant cross-connect, BGP route servers, local flow-log collector, and dedicated ingress path at paid tenant_class.
- `oyatie-as-cloud-provider` has the most aggressive targets because it can avoid provider API propagation and own native fabric internals.
- Control-plane p99 targets measure acceptance and validation, not physical network convergence.
- Route-convergence targets measure the interval from accepted route update to healthy forwarding state in the measured context.
- Flow-log delivery targets measure first durable audit projection, not long-term analytics availability.
- Reachability targets measure modeled path analysis, not active packet probing.
- Throughput targets are minimum tenant_class budgets; tenant contracts can cap lower.
- Availability targets assume the tier's documented redundancy pattern exists.
- paid should not be offered in a context until its provider/hardware quota evidence proves the listed ceilings.
- demo_trial should not advertise hyperscaler parity; it is a low-cost viability tier.

## §5 Comparison narrative

- AWS route-table quota of 500 default routes and 1,000 maximum after increase is above Oyatie demo_trial/paid route targets but below Oyatie paid native targets.
- AWS security group scale is strong at 2,500 groups per region, but Oyatie paid targets a tenant-level policy model that must avoid mapping one-to-one onto provider security groups in guest contexts.
- AWS Direct 400 Gbps provides a public precedent for Oyatie paid 400 Gbps native provider target.
- Google MTU up to 8,896 bytes sets a jumbo-frame precedent that Oyatie should explicitly model for paid/paid colocated and native contexts.
- Google 7 Gbps ordinary external egress and 25 Gbps Tier_1 egress provide a useful catch-up bar for Oyatie paid guest-on-GCP-like expectations, but this audit target is for `cloud-network` contexts rather than a GCP guest context.
- Google flow-log retention default of 30 days is a useful minimum; Oyatie must define its own retention by compliance pack because packet tags and audit-chain projection are stronger claims.
- Azure 500,000 concurrent flows per NIC is a concrete flow-scale precedent; Oyatie should not claim paid flow scale until a comparable per-NIC or per-endpoint flow model exists.
- Azure 600 UDRs per route table provides a route-table scale reference; Oyatie demo_trial/paid route targets should remain at or below this in Azure guest-like contexts.
- Azure VNet peering up to 500 by default and 1,000 through Virtual Network Manager is a precedent for Oyatie transit/global network manager work.
- Oyatie demo_trial guest-on-OCI is intentionally behind AWS/GCP/Azure paid capabilities because Always Free is a hard cost constraint.
- Oyatie paid targets parity for basic paid network operations, not all advanced network-manager features.
- Oyatie paid targets production parity for common VPC, routing, ingress, logging, and incident-response operations.
- Oyatie paid targets a native-provider hyperscaler bar that is ahead only if native fabric, SR-IOV/DPDK, flow-log, and route-analysis evidence are built.
- Current target-path docs do not contain measured data, so every comparison is a planning classification.
- Current classification for demo_trial: catch-up except OCI cost discipline.
- Current classification for paid: catch-up with achievable parity.
- Current classification for paid: parity target for core network operations, catch-up for diagnostics/global manager/IPAM.
- Current classification for paid: aspirational until native fabric and measured evidence exist.
