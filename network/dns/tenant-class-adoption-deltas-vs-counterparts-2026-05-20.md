# cloud-network-dns capability-tenant_class deltas vs counterparts — 2026-05-20

## Citation anchor block

1. Canonical sequence: `/Users/jasonlee/oyatie/docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-1815`, `:2241-2365`, `:3140-3235`, and `:4081-4151`.
2. Canonical machine plan: `/Users/jasonlee/oyatie/specs/master-plan-sequencing.json:704-867` for deployment contexts, OpenTofu substrate, supported OSes, language policy, and OCI Always Free.
3. Service tenant_class source: `/Users/jasonlee/oyatie/microservices/cloud-network-dns/retired tenant_class adoption artifact:12-82`.
4. Service operating guides: `/Users/jasonlee/oyatie/microservices/cloud-network-dns/faqs/dns-engineer-faq.md:7-93` and `/Users/jasonlee/oyatie/microservices/cloud-network-dns/tutorials/provision-zone-dnssec-geo-routing-and-doq.md:12-87`.
5. External counterpart sources: AWS Route 53 Developer Guide `https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/`, Google Cloud DNS docs `https://docs.cloud.google.com/dns/docs/`, Cloudflare DNS docs `https://developers.cloudflare.com/dns/`.

## Scope and delta method

This report maps the current Oyatie cloud-network-dns tenant_class story against AWS Route 53, Google Cloud DNS, and Cloudflare DNS.
The purpose is not to rank brands.
The purpose is to expose tenant_class-by-tenant_class delivery gaps so Wave 14 aggregation can schedule buildable work.
The Oyatie source of truth is the current service-local tenant_class matrix, FAQ, tutorial, migration guide, benchmark note, onboarding guide, and reference implementation.
The counterpart source of truth is the public feature surface documented by AWS, Google, and Cloudflare as of this audit.
Where a counterpart does not publish a package named exactly demo_trial, paid, paid, or paid, this report maps the nearest commercial or operating posture.
AWS Route 53 is mostly query-priced and feature-priced rather than tenant_class-priced.
Google Cloud DNS is mostly managed-zone and query-priced, with private DNS, forwarding, peering, routing policies, and DNS Armor as feature families.
Cloudflare DNS has clearer account-plan tenant_classes plus add-on products such as Load Balancing, secondary DNS, DNS Firewall, and 1.1.1.1 resolver services.
Oyatie currently declares demo_trial and paid tenant_class in `retired tenant_class adoption artifact:12-82`.
The same file does not reconcile demo_trial to OCI Always Free, which is a canonical requirement from ADR-0328 D-19 and the OCI memory directive.
The current service has no local IaC, no supported OS manifest, no local PRD, and no local architecture document.
That absence affects every tenant_class because the tenant_class promises are not yet tied to deployable context manifests.
The only current implementation proof outside the service-local folder is the cloud DNS API contract and Rust API crate.
The API proof is narrow: tenant DNS zone creation, public/private zone handling, normalized request handling, and idempotency.
The tenant_class delta therefore treats many promised tenant_class features as documented intent rather than implemented proof.
Gap classes used below are:
`parity-documented`: Oyatie documents a capability at the tenant_class and the counterpart has a similar capability.
`parity-implemented-narrow`: Oyatie has a contract or code artifact proving a narrow implementation slice.
`catch-up-build`: counterpart has the capability and Oyatie tenant_class docs do not substantiate it enough to build.
`catch-up-proof`: Oyatie tenant_class docs promise it but local contract, code, IaC, or tests do not prove it.
`ahead-documentation`: Oyatie declares an additive governance or substrate capability not visible as a direct counterpart feature.
`mis-tenant_classed`: Oyatie places a feature in a tenant_class that conflicts with canonical constraints or with its own tenant_class semantics.

## §1 Tier definitions in Oyatie

### §1.1 demo_trial definition from current artifacts

demo_trial is declared as the onboarding and Always-On baseline in `retired tenant_class adoption artifact:14-27`.
demo_trial declares 1 primary authoritative region and no active-active failover in `retired tenant_class adoption artifact:16-17`.
demo_trial declares public authoritative zones in `retired tenant_class adoption artifact:18`.
demo_trial declares no private DNS in `retired tenant_class adoption artifact:19`.
demo_trial declares DNSSEC signing with one KSK/ZSK rotation policy in `retired tenant_class adoption artifact:20`.
demo_trial declares weighted and failover routing only in `retired tenant_class adoption artifact:21`.
demo_trial declares health-check driven failover with 60 second probe cadence in `retired tenant_class adoption artifact:22`.
demo_trial declares tenant-scoped audit trail retained for 30 days in `retired tenant_class adoption artifact:23`.
demo_trial declares 5k sustained authoritative queries per second per tenant in `retired tenant_class adoption artifact:24`.
demo_trial declares 20 ms regional p95 authoritative answer latency in `retired tenant_class adoption artifact:25`.
demo_trial declares Cedar permits for zone and record-set administration in `retired tenant_class adoption artifact:26`.
demo_trial declares target cost below 5 USD per month for small tenants in `retired tenant_class adoption artifact:27`.
demo_trial is not currently mapped to OCI Always Free in the service-local file.
That is a mis-tenant_classing issue because ADR-0328 D-19 requires the OCI Always Free subprofile and the memory directive requires demo_trial to maximize Always Free before paid uplift.
demo_trial has no service-local IaC proving deployment to `guest-on-oci`, `oyatie-public-cloud`, or any other context.
demo_trial has no OS support statement proving package or runtime coverage on the canonical Tier-1 OS matrix.
demo_trial has no SLO file, OpenSLO artifact, or local health-check contract proving the health-check cadence.
demo_trial has no local Cedar policy files even though Cedar permits are promised.
demo_trial has no local DNSSEC key-management ADR even though DNSSEC is included.
demo_trial has no local record API contract even though the FAQ and tutorial describe record administration.
demo_trial has a narrow external API proof for tenant DNS zone creation through `contracts/openapi/cloud/cloud-network-dns-v1.yaml:1-12`.
demo_trial can therefore be considered an intended managed authoritative DNS entry tier, not a build-closed tier.

### §1.2 paid definition from current artifacts

paid is declared as the regional production tenant_class in `retired tenant_class adoption artifact:30-45`.
paid declares 2 active regions with asynchronous zone replication in `retired tenant_class adoption artifact:32`.
paid declares active-passive failover with automatic promotion in `retired tenant_class adoption artifact:33`.
paid declares public and private DNS zones in `retired tenant_class adoption artifact:34`.
paid declares DNSSEC with scheduled automated KSK and ZSK rotation in `retired tenant_class adoption artifact:35`.
paid declares weighted, failover, latency, and geo routing in `retired tenant_class adoption artifact:36`.
paid declares health checks with 30 second cadence and 3-region observers in `retired tenant_class adoption artifact:37`.
paid declares audit trail retained for 180 days in `retired tenant_class adoption artifact:38`.
paid declares delegated sub-zone automation in `retired tenant_class adoption artifact:39`.
paid declares 50k sustained QPS per tenant in `retired tenant_class adoption artifact:40`.
paid declares 12 ms regional p95 authoritative answer latency in `retired tenant_class adoption artifact:41`.
paid declares tenant-scoped DNS change approval workflows in `retired tenant_class adoption artifact:42`.
paid declares a target cost below 50 USD per month for mid-size tenants in `retired tenant_class adoption artifact:43`.
paid is where the FAQ places private zone binding and workload-local resolution expectations in `faqs/dns-engineer-faq.md:25-35`.
paid is where the migration guide expects AWS Route 53 private-hosted-zone migration to become practical in `migration-playbooks/from-route53-and-ns1.md:64-71`.
paid lacks local IaC for every canonical context.
paid lacks local API schemas for record-set CRUD, routing policy CRUD, health-check CRUD, approval workflows, and private-zone bindings.
paid lacks local runbooks for replication lag, promotion, observer failure, and delegated sub-zone rollback.
paid has no local data model for zone replicas, region membership, or change approval state.
paid aligns directionally with the production entry surface of AWS Route 53 and Google Cloud DNS but is not implemented enough to claim parity.

### §1.3 paid definition from current artifacts

paid is declared as the multi-region regulated production tenant_class in `retired tenant_class adoption artifact:48-63`.
paid declares 3 or more active regions with quorum-based zone replication in `retired tenant_class adoption artifact:50`.
paid declares active-active failover with regional evacuation plans in `retired tenant_class adoption artifact:51`.
paid declares public, private, split-horizon, and delegated zones in `retired tenant_class adoption artifact:52`.
paid declares DNSSEC backed by HSM custody policy in `retired tenant_class adoption artifact:53`.
paid declares weighted, failover, latency, geo, and compliance-jurisdiction routing in `retired tenant_class adoption artifact:54`.
paid declares health checks with 15 second cadence and synthetic record probes in `retired tenant_class adoption artifact:55`.
paid declares audit retention for 1 year with export hooks in `retired tenant_class adoption artifact:56`.
paid declares 250k sustained QPS per tenant in `retired tenant_class adoption artifact:57`.
paid declares 8 ms regional p95 authoritative answer latency in `retired tenant_class adoption artifact:58`.
paid declares policy-bound change windows and emergency break-glass in `retired tenant_class adoption artifact:59`.
paid declares delegated vanity domain onboarding in `retired tenant_class adoption artifact:60`.
paid declares target cost below 500 USD per month for regulated tenants in `retired tenant_class adoption artifact:61`.
paid is the first tenant_class where Oyatie claims a governance surface stronger than the baseline counterpart DNS product.
paid is also where the absence of local policy, compliance, incident-response, and OpenSLO documents becomes severe.
paid has no local HSM design or key custody model.
paid has no local emergency change flow contract.
paid has no local audit export contract.
paid has no local compliance-jurisdiction routing schema.
paid has no local synthetic probe schema.
paid has no local capacity model proving 250k sustained QPS.
paid therefore reads as a target tenant_class that combines counterpart parity goals with Oyatie governance aspirations.
paid should not be treated as buildable until the missing contracts and deployment manifests land.

### §1.4 paid definition from current artifacts

paid is declared as the hyperscaler and single-tenant capable tenant_class in `retired tenant_class adoption artifact:66-81`.
paid declares dedicated authoritative clusters per tenant or regulated tenant group in `retired tenant_class adoption artifact:68`.
paid declares 5 or more regions with tenant-selected topology in `retired tenant_class adoption artifact:69`.
paid declares active-active failover with custom RTO and RPO contracts in `retired tenant_class adoption artifact:70`.
paid declares public, private, split-horizon, delegated, and bring-your-own-authority zones in `retired tenant_class adoption artifact:71`.
paid declares DNSSEC and offline-root ceremonies in `retired tenant_class adoption artifact:72`.
paid declares programmable routing policy under policy review in `retired tenant_class adoption artifact:73`.
paid declares health checks with 5 second cadence and custom probes in `retired tenant_class adoption artifact:74`.
paid declares audit retention for 7 years with signed export bundles in `retired tenant_class adoption artifact:75`.
paid declares 1M sustained QPS per tenant in `retired tenant_class adoption artifact:76`.
paid declares 5 ms regional p95 authoritative answer latency in `retired tenant_class adoption artifact:77`.
paid declares dedicated support runbooks and tenant-owned disaster recovery exercises in `retired tenant_class adoption artifact:78`.
paid declares target cost custom priced but capacity-isolated in `retired tenant_class adoption artifact:79`.
paid is where Oyatie’s stated tenant_class ambition most resembles enterprise DNS contracts, dedicated Cloudflare enterprise plans, and bespoke Route 53 enterprise support patterns.
paid has the largest proof gap.
paid lacks dedicated-cluster IaC.
paid lacks single-tenant admission criteria.
paid lacks a bring-your-own-authority protocol.
paid lacks programmable routing review contracts.
paid lacks signed export bundle format.
paid lacks DR exercise runbooks.
paid lacks dedicated support runbooks.
paid lacks benchmark evidence for 1M QPS and 5 ms p95.
paid should be treated as an architectural intent tenant_class rather than an implementation-ready tier.

## §2 Counterpart tenant_class mapping

### §2.1 AWS Route 53 tenant_class mapping

AWS Route 53 does not present a simple demo_trial/paid tenant_class ladder in its public DNS product documentation.
The closest demo_trial equivalent is a standard public hosted zone with authoritative DNS, basic record management, DNSSEC signing, and pay-per-zone plus query pricing.
The closest demo_trial equivalent can include simple and weighted routing where configured.
The closest demo_trial equivalent can include failover routing only when health checks are purchased and configured.
The closest paid equivalent adds private hosted zones, Resolver endpoints, inbound and outbound forwarding, latency or geolocation routing, traffic flow, and operational health checks.
The closest paid equivalent adds multi-account governance, Resolver DNS Firewall, query logging, deeper health-check integration, traffic policies, and regulated operation through IAM and organization-level controls.
The closest paid equivalent is a bespoke enterprise posture using dedicated account structures, support escalation, Route 53 ARC for recovery readiness, DNS Firewall policy estates, and custom operational contracts.
AWS has strong public-zone maturity.
AWS has strong private DNS integration with VPCs and Resolver.
AWS has strong traffic-policy and routing-policy depth.
AWS has strong DNS firewalling through Route 53 Resolver DNS Firewall.
AWS has mature IAM, CloudTrail, CloudWatch, and organization integration.
AWS does not expose an Oyatie-style Cedar-native tenant change approval system as a Route 53 feature.
AWS does not expose an Oyatie-style OpenTofu-only substrate rule because AWS supports many IaC tools and native CloudFormation.
AWS does not expose an OCI Always Free demo_trial reconciliation because that is an Oyatie deployment doctrine.
AWS paid-equivalent contracts are assembled from enterprise support, account design, and adjacent Route 53 products rather than one named DNS tier.
AWS is the strongest counterpart for public and private DNS feature depth.
AWS is also the strongest pressure source for Route 53 migration parity because the Oyatie migration playbook names Route 53 directly.

### §2.2 Google Cloud DNS tenant_class mapping

Google Cloud DNS also does not present a simple demo_trial/paid tenant_class DNS ladder.
The closest demo_trial equivalent is a standard public managed zone with authoritative DNS and DNSSEC.
The closest paid equivalent adds private zones, forwarding zones, peering zones, response policies, routing policies, and VPC integration.
The closest paid equivalent adds DNS Armor, deeper monitoring, hybrid forwarding estates, policy controls, and regulated GCP organization operation.
The closest paid equivalent is a bespoke enterprise posture using organization policy, private service access, Cloud Logging, Cloud Monitoring, DNS Armor, and custom support contracts.
Google has strong managed-zone primitives.
Google has strong private DNS and forwarding primitives.
Google has strong VPC-native resolution integration.
Google has routing policies including weighted round robin and geolocation.
Google has DNS peering and forwarding patterns that matter for guest and on-prem contexts.
Google has DNSSEC support for public managed zones.
Google has monitoring and logging integrations.
Google DNS Armor adds protective filtering capabilities.
Google’s model is less plan-tenant_classed and more feature-attached.
Google does not expose Oyatie’s tenant-owned disaster-recovery exercise pattern as a Cloud DNS tenant_class feature.
Google does not expose OpenTofu-only deployment doctrine.
Google does not expose OCI demo_trial tenant_class economics.
Google provides strong pressure for paid private-zone and hybrid DNS parity.
Google provides strong pressure for paid policy, monitoring, forwarding, and response-policy parity.
Google provides a weaker direct comparator for paid dedicated authoritative clusters than Cloudflare Enterprise or bespoke AWS architectures.

### §2.3 Cloudflare DNS tenant_class mapping

Cloudflare has more visible commercial plan names than AWS or Google.
The closest demo_trial equivalent is the Free or low-plan authoritative DNS posture for a public zone with standard record management and anycast authoritative serving.
The closest paid equivalent is Business-style managed authoritative DNS with stronger operational controls, secondary DNS, load balancing add-ons, and broader support expectations.
The closest paid equivalent is Enterprise authoritative DNS with advanced support, DNS Firewall, secondary DNS, load balancing, analytics, and stronger security posture.
The closest paid equivalent is Enterprise plus dedicated custom contracts, DNS Firewall scale, load-balancing steering, secondary DNS, custom nameservers, and account-level support.
Cloudflare has very strong public authoritative DNS.
Cloudflare has strong DNSSEC, analytics, secondary DNS, zone transfer, and DNS Firewall product surfaces.
Cloudflare has strong load-balancing and traffic steering as adjacent DNS capability.
Cloudflare has a public recursive resolver product, including encrypted DNS variants.
Cloudflare’s private DNS and cloud-VPC native resolver surface is different from AWS and Google because Cloudflare is not a hyperscaler VPC provider in the same sense.
Cloudflare does not map cleanly to Oyatie private hosted zone bindings without adjacent Cloudflare One or Magic WAN product context.
Cloudflare does not expose an Oyatie-style OpenTofu-only doctrine.
Cloudflare does not expose OCI Always Free reconciliation.
Cloudflare provides the strongest counterpart pressure for analytics, DNS Firewall, secondary DNS, custom nameservers, and global anycast performance.
Cloudflare provides less direct pressure for cloud-provider-internal VPC private DNS than AWS or Google.
Cloudflare is the strongest named-tenant_class comparator for paid and paid customer-facing expectations.

## §3 Per-Oyatie-tenant_class delta tables

### §3.1 demo_trial tenant_class delta table

| Feature | Oyatie demo_trial | AWS Route 53 equivalent | Google Cloud DNS equivalent | Cloudflare DNS equivalent | Gap classification |
| --- | --- | --- | --- | --- | --- |
| Public authoritative zone | Declared in tenant_class matrix, narrow zone-create API proof exists | Public hosted zone | Public managed zone | Public authoritative zone | parity-implemented-narrow |
| Record-set CRUD | Tutorial implies record administration, no local contract | Record sets supported | Resource record sets supported | DNS records supported | catch-up-build |
| DNSSEC signing | Declared one KSK/ZSK rotation policy | DNSSEC supported for hosted zones | DNSSEC supported for public managed zones | DNSSEC supported | catch-up-proof |
| DNSSEC rotation automation | Declared minimal policy, no local runbook | Supported operationally through AWS controls | Supported through managed-zone DNSSEC controls | Supported by Cloudflare DNSSEC controls | catch-up-proof |
| Weighted routing | Declared | Weighted routing policy | Weighted round robin routing policy | Load balancing steering add-on | parity-documented |
| Failover routing | Declared | Failover routing with health checks | Health-checked routing via adjacent patterns is less direct | Load Balancing health monitors | parity-documented |
| Health check cadence | 60 second cadence declared, no contract | Route 53 health checks configurable | Cloud Monitoring plus routing patterns | Cloudflare Load Balancing monitors | catch-up-proof |
| Public zone import | Migration playbook covers import workflow | Hosted-zone record import possible through APIs | Managed-zone record import possible through APIs | Zone import and scan workflows | parity-documented |
| Tenant audit trail | 30 day retention declared | CloudTrail and Route 53 logs | Cloud Audit Logs and DNS logs | Audit logs by plan | catch-up-proof |
| Tenant-scoped IAM | Cedar permits declared, no policy files | IAM policies | IAM roles and permissions | Account roles and API tokens | catch-up-proof |
| Zone create idempotency | Rust API tests prove a narrow path | API idempotency via change batches and caller controls | API operations with managed changes | API operation idempotency patterns | parity-implemented-narrow |
| Private DNS | Explicitly absent in demo_trial | Available in Route 53 but higher posture here | Available in Cloud DNS but higher posture here | Less direct; not public authoritative baseline | intentionally-lower |
| Geo routing | Not declared in demo_trial | Geolocation routing available | Geolocation routing policies available | Load Balancing steering available | catch-up-tenant_class-choice |
| Latency routing | Not declared in demo_trial | Latency routing available | Regional routing policies available | Load Balancing steering available | catch-up-tenant_class-choice |
| Secondary DNS | Not declared | Reusable delegation and external secondary possible through architecture, not core simple baseline | Not a simple baseline feature | Cloudflare secondary DNS product | catch-up-build |
| Zone transfer | Not declared | Not baseline Route 53 authoritative outbound transfer posture | Limited by product model | Cloudflare zone transfers supported | catch-up-build |
| Query logging | Not declared in demo_trial | Route 53 query logging | Cloud DNS logging and monitoring | DNS analytics/logging by plan | catch-up-build |
| DNS Firewall | Not declared | Resolver DNS Firewall | DNS Armor / response policy families | Cloudflare DNS Firewall | catch-up-build |
| Analytics dashboard | Not declared | CloudWatch metrics and logs | Cloud Monitoring dashboards | Cloudflare Analytics | catch-up-build |
| Anycast authoritative network | Not explicitly stated | AWS global authoritative service | Google global authoritative service | Cloudflare anycast authoritative network | catch-up-proof |
| Per-tenant QPS target | 5k sustained QPS declared | AWS quotas and request-rate practices are product-specific | Google quotas are product-specific | Cloudflare plan limits vary | parity-documented |
| Regional p95 target | 20 ms regional p95 declared | Counterpart public latency claims are not tenant_class-contract direct | Counterpart public latency claims are not tenant_class-contract direct | DNSPerf often shows low global latency | catch-up-proof |
| Cost ceiling | Less than 5 USD/month declared | Possible for low query volume | Possible for low query volume | Free/low plan possible | parity-documented |
| OCI Always Free fit | Not documented, conflicts with canonical profile | Not applicable | Not applicable | Not applicable | mis-tenant_classed |
| OpenTofu deployment | No local IaC | AWS supports many IaC tools | Google supports many IaC tools | Cloudflare supports APIs/providers | catch-up-build |
| OS matrix | No supported OS manifest | Managed service hides customer OS | Managed service hides customer OS | Managed service hides customer OS | catch-up-build |
| Public API breadth | Zone create only is evidenced | Broad Route 53 API | Broad Cloud DNS API | Broad Cloudflare DNS API | catch-up-build |
| SLA/SLO document | No local OpenSLO file | AWS service SLAs and internal metrics | Google service SLAs and monitoring | Cloudflare plan SLAs by contract | catch-up-build |
| Incident runbook | Missing | AWS operational docs plus support | Google operational docs plus support | Cloudflare support/runbooks by plan | catch-up-build |
| demo_trial verdict | Entry intent is coherent but under-proved | Strong mature baseline | Strong mature baseline | Strong mature baseline | catch-up-build |

### §3.2 paid tenant_class delta table

| Feature | Oyatie paid | AWS Route 53 equivalent | Google Cloud DNS equivalent | Cloudflare DNS equivalent | Gap classification |
| --- | --- | --- | --- | --- | --- |
| Public authoritative DNS | Declared | Public hosted zones | Public managed zones | Authoritative DNS | parity-documented |
| Private zones | Declared | Private hosted zones | Private managed zones | Not direct public DNS product equivalent | parity-documented-with-AWS-Google |
| VPC or workload binding | FAQ describes private binding expectation | VPC association | VPC association | Requires adjacent network products | catch-up-proof |
| Forwarding zones | Not declared | Resolver outbound forwarding | Forwarding zones | DNS Firewall/forwarding patterns differ | catch-up-build |
| Peering zones | Not declared | Resolver and VPC sharing patterns | DNS peering zones | Different network product model | catch-up-build |
| Split-horizon DNS | Not declared until paid | Private/public zone combination possible | Private/public split possible | Enterprise/custom patterns possible | catch-up-tenant_class-choice |
| Regional replication | 2 active regions async declared | AWS global managed service abstracts this | Google global managed service abstracts this | Cloudflare anycast abstracts this | catch-up-proof |
| Active-passive failover | Declared automatic promotion | Failover routing | Routing policies plus monitoring | Load Balancing failover | parity-documented |
| Latency routing | Declared | Latency routing | Routing policies | Load Balancing steering | parity-documented |
| Geo routing | Declared | Geolocation routing | Geolocation routing policies | Load Balancing steering | parity-documented |
| Delegated sub-zone automation | Declared | Delegation sets and NS records | Managed delegation through records | Delegation and custom nameserver patterns | catch-up-proof |
| Health checks | 30 second cadence, 3 observers declared | Route 53 health checks | Monitoring plus routing | Cloudflare monitors | catch-up-proof |
| Change approvals | Declared | IAM/Change Manager can be composed | IAM/approval tooling composed | Account roles and workflows composed | ahead-documentation-but-unproven |
| Audit retention | 180 days declared | CloudTrail/log retention configurable | Cloud Logging retention configurable | Audit logs by plan | catch-up-proof |
| Query logging | Not explicitly declared | Query logging | Cloud DNS logging | Cloudflare DNS analytics/logs | catch-up-build |
| Analytics | Not explicitly declared | CloudWatch/Route 53 metrics | Cloud Monitoring | Cloudflare Analytics | catch-up-build |
| DNS Firewall | Not declared | Resolver DNS Firewall | DNS Armor/response policy | Cloudflare DNS Firewall | catch-up-build |
| Response policies | Not declared | DNS Firewall rule groups | Response policies | DNS Firewall policies | catch-up-build |
| Secondary DNS | Not declared | Not a core Route 53 standard surface | Not a core Cloud DNS standard surface | Cloudflare secondary DNS | catch-up-build |
| Zone transfer | Not declared | Limited/architectural | Limited/product-dependent | Zone transfers supported | catch-up-build |
| API contracts | Missing record, policy, health, private-zone contracts | Broad AWS APIs | Broad Google APIs | Broad Cloudflare APIs | catch-up-build |
| Data model | Missing | AWS internal managed model | Google internal managed model | Cloudflare internal managed model | catch-up-build |
| IaC coverage | Missing every context | Provider support exists | Provider support exists | Provider support exists | catch-up-build |
| OpenTofu rule | Required by Oyatie, not evidenced | AWS also supports CloudFormation, so not comparable | Google not OpenTofu-only | Cloudflare not OpenTofu-only | catch-up-build |
| CI lanes | Missing | Managed service not customer OS tested | Managed service not customer OS tested | Managed service not customer OS tested | catch-up-build |
| QPS target | 50k sustained per tenant declared | Counterpart capacity mature | Counterpart capacity mature | Counterpart capacity mature | catch-up-proof |
| Latency target | 12 ms regional p95 declared | Mature global authoritative service | Mature global authoritative service | Mature anycast authoritative service | catch-up-proof |
| Cost target | Less than 50 USD/month declared | Plausible at moderate load | Plausible at moderate load | Plan-dependent | parity-documented |
| Migration from Route 53 | Playbook exists | Source product | Migration target if reverse | Cloudflare import patterns | parity-documented |
| paid verdict | Feature intent matches AWS/Google private DNS baseline but lacks build proofs | Mature private DNS | Mature private DNS | Strong public DNS, less private-VPC direct | catch-up-proof |

### §3.3 paid tenant_class delta table

| Feature | Oyatie paid | AWS Route 53 equivalent | Google Cloud DNS equivalent | Cloudflare DNS equivalent | Gap classification |
| --- | --- | --- | --- | --- | --- |
| Multi-region active operation | 3+ active regions with quorum declared | Managed global service | Managed global service | Managed anycast service | catch-up-proof |
| Active-active failover | Declared with evacuation plans | Route 53 policies plus ARC patterns | Routing policies plus cloud operations | Load Balancing and tenant RBAC ops | catch-up-proof |
| Split-horizon DNS | Declared | Public/private hosted zones | Public/private managed zones | Enterprise/custom network products | parity-documented |
| Delegated zones | Declared | Delegation sets and NS delegation | Delegation via records and zones | Delegation/custom nameservers | parity-documented |
| HSM-backed DNSSEC | Declared | AWS KMS/CloudHSM-adjacent patterns, Route 53 managed DNSSEC | Cloud HSM-adjacent governance, Cloud DNS DNSSEC | Enterprise key governance patterns | catch-up-build |
| Compliance jurisdiction routing | Declared | Geoproximity/geolocation plus policy architecture | Geolocation routing policies plus org policy | Load Balancing steering and rules | ahead-documentation-but-unproven |
| Synthetic record probes | Declared | Route 53 health checks and CloudWatch Synthetics | Cloud Monitoring synthetic checks | Cloudflare monitors | catch-up-proof |
| 1-year audit retention | Declared | Configurable log retention | Configurable log retention | Enterprise audit/log retention | parity-documented |
| Audit export hooks | Declared | CloudTrail exports | Cloud Logging sinks | Cloudflare logpush/audit logs | catch-up-proof |
| Emergency break-glass | Declared | IAM break-glass can be composed | IAM break-glass can be composed | Account break-glass can be composed | ahead-documentation-but-unproven |
| Policy-bound change windows | Declared | Change Manager/IAM composition | IAM/approval composition | Workflow composition | ahead-documentation-but-unproven |
| Vanity domain onboarding | Declared | Delegation and hosted zone operations | Delegation and managed zone operations | Custom nameservers and zone onboarding | parity-documented |
| DNS Firewall | Not declared explicitly | Resolver DNS Firewall | DNS Armor/response policies | Cloudflare DNS Firewall | catch-up-build |
| Abuse detection | Not declared | CloudWatch/Security services composition | Security Command Center/log analysis | Cloudflare security analytics | catch-up-build |
| Query analytics | Not declared | Query logging plus CloudWatch | Monitoring and logging | Cloudflare Analytics | catch-up-build |
| Secondary DNS | Not declared | Limited standard public product mapping | Limited standard product mapping | Secondary DNS product | catch-up-build |
| Zone transfers | Not declared | Not direct standard baseline | Not direct standard baseline | Zone transfers supported | catch-up-build |
| Resolver firewall | Not declared | Resolver DNS Firewall | DNS Armor/response policies | DNS Firewall | catch-up-build |
| Hybrid DNS | Not declared directly | Resolver endpoints | Forwarding and peering zones | Adjacent network products | catch-up-build |
| Policy language | Cedar implied by lower tenant_classes | IAM policies | IAM policies | Roles/API tokens/rules | ahead-documentation-but-unproven |
| Compliance docs | No local compliance.md | AWS compliance programs | Google compliance programs | Cloudflare compliance programs | catch-up-build |
| DPIA | No local dpia.md | Customer responsibility/compliance support | Customer responsibility/compliance support | Customer responsibility/compliance support | catch-up-build |
| Incident response | No local incident-response.md | AWS support and operational docs | Google support and operational docs | Cloudflare enterprise support | catch-up-build |
| Capacity model | No local capacity-model.md | Mature service capacity | Mature service capacity | Mature service capacity | catch-up-build |
| Benchmark evidence | Existing benchmark file has target-style claims without raw evidence | Public measurements vary | Public measurements vary | DNSPerf frequently compares providers | catch-up-proof |
| IaC per context | Missing | Providers and modules exist | Providers and modules exist | Providers and modules exist | catch-up-build |
| Sigstore module signing | Required by ADR-0328, missing | Not AWS product-tenant_class feature | Not Google product-tenant_class feature | Not Cloudflare product-tenant_class feature | catch-up-build |
| OS package support | Missing | Managed service abstracts OS | Managed service abstracts OS | Managed service abstracts OS | catch-up-build |
| QPS target | 250k sustained per tenant declared | Mature global scale | Mature global scale | Mature global scale | catch-up-proof |
| Latency target | 8 ms regional p95 declared | Mature global latency | Mature global latency | Mature anycast latency | catch-up-proof |
| paid verdict | Governance ambition is strong, but counterpart operational features and local proofs are missing | Mature plus adjacent controls | Mature plus adjacent controls | Mature enterprise controls | catch-up-build |

### §3.4 paid tenant_class delta table

| Feature | Oyatie paid | AWS Route 53 equivalent | Google Cloud DNS equivalent | Cloudflare DNS equivalent | Gap classification |
| --- | --- | --- | --- | --- | --- |
| Dedicated authoritative clusters | Declared per tenant or regulated group | Enterprise account/isolation architecture, not standard named tenant_class | Enterprise org/isolation architecture, not standard named tenant_class | Enterprise custom contracts and dedicated arrangements | catch-up-build |
| Tenant-selected topology | Declared | Custom architecture with Route 53 and ARC | Custom architecture with Cloud DNS and org policy | Enterprise architecture and custom network routing | catch-up-build |
| 5+ regions | Declared | AWS global authoritative infrastructure abstracts region count | Google global authoritative infrastructure abstracts region count | Cloudflare anycast abstracts region count | catch-up-proof |
| Custom RTO/RPO contracts | Declared | Enterprise support and Route 53 ARC patterns | Enterprise support and DR architecture | Enterprise SLA/support contracts | catch-up-build |
| Bring-your-own-authority zones | Declared | Delegation and external authority integration possible | Delegation and external authority integration possible | Secondary DNS and zone-transfer patterns | catch-up-build |
| Offline-root ceremonies | Declared | Custom governance outside core Route 53 | Custom governance outside core Cloud DNS | Enterprise DNSSEC governance patterns | ahead-documentation-but-unproven |
| Programmable routing under policy review | Declared | Traffic Flow and API-driven policies | Routing policies and APIs | Rules/load-balancing steering | catch-up-build |
| 5 second health-check cadence | Declared | Route 53 health-check cadence options and adjacent probes | Monitoring probes | Cloudflare monitors | catch-up-proof |
| Custom health probes | Declared | Health checks and custom monitoring composition | Cloud Monitoring custom checks | Cloudflare monitor customization | catch-up-proof |
| 7-year audit retention | Declared | Configurable log archival | Configurable log archival | Enterprise log retention/export | catch-up-proof |
| Signed export bundles | Declared | Can be composed with signing systems | Can be composed with signing systems | Can be composed with export/signing systems | ahead-documentation-but-unproven |
| Tenant-owned DR exercises | Declared | Enterprise operational practice | Enterprise operational practice | Enterprise operational practice | ahead-documentation-but-unproven |
| Dedicated support runbooks | Declared | Enterprise support | Enterprise support | Enterprise support | catch-up-build |
| Single-tenant isolation | Declared by dedicated clusters | Account/zone isolation possible | Project/org isolation possible | Enterprise account isolation possible | catch-up-build |
| Custom nameservers | Implied by dedicated authority | Reusable delegation sets/custom NS patterns | Managed zone NS delegation | Custom nameservers by plan | catch-up-build |
| DNS Firewall at scale | Not declared explicitly | Resolver DNS Firewall | DNS Armor/response policy | Cloudflare DNS Firewall | catch-up-build |
| Secondary DNS | Not declared explicitly | Limited standard Route 53 mapping | Limited standard Cloud DNS mapping | Secondary DNS product | catch-up-build |
| Zone transfer | Not declared explicitly | External architecture needed | External architecture needed | Zone transfers supported | catch-up-build |
| Resolver service | Tutorial includes DoQ resolver answer path | Route 53 Resolver | Cloud DNS forwarding/private resolution | 1.1.1.1 recursive resolver and Gateway products | catch-up-proof |
| DoH/DoT/DoQ | Tutorial demonstrates DoQ output | Resolver endpoint encrypted protocols are not a simple public feature | Cloud DNS does not map directly | Cloudflare 1.1.1.1 supports encrypted DNS variants | additive-documented-but-unproven |
| Global analytics | Not declared explicitly | CloudWatch/log analysis | Cloud Monitoring/logging | Cloudflare Analytics | catch-up-build |
| Abuse controls | Not declared explicitly | DNS Firewall/security integrations | DNS Armor/security integrations | DNS Firewall/security analytics | catch-up-build |
| Regulatory residency | Implied through topology and jurisdiction routing | Region/account policy composition | Organization policy and region controls | Enterprise data localization products | catch-up-build |
| Benchmark proof | No raw evidence | Mature global scale | Mature global scale | Mature global scale | catch-up-proof |
| 1M sustained QPS target | Declared | Enterprise-grade scale expected | Enterprise-grade scale expected | Enterprise DNS scale expected | catch-up-proof |
| 5 ms p95 target | Declared | Aggressive relative to global DNS variability | Aggressive relative to global DNS variability | Aggressive but plausible with anycast locality | catch-up-proof |
| paid IaC | Missing | Enterprise IaC possible | Enterprise IaC possible | Enterprise IaC possible | catch-up-build |
| paid tests | Missing | Provider validation via mature service | Provider validation via mature service | Provider validation via mature service | catch-up-build |
| paid runbooks | Missing despite promise | Enterprise runbooks exist privately | Enterprise runbooks exist privately | Enterprise runbooks exist privately | catch-up-build |
| paid verdict | Aspirational and potentially differentiated, but least buildable today | Bespoke enterprise parity target | Bespoke enterprise parity target | Closest named enterprise comparator | catch-up-build |

## §4 OCI demo_trial tenant_class = Always Free reconciliation

ADR-0328 and the master plan require an OCI Always Free subprofile for relevant services.
The master plan lists `oci_always_free` as a named profile with `required_for_context` set to `guest-on-oci` in `specs/master-plan-sequencing.json:857-864`.
The same plan lists `avoid_paid_managed_services_when_always_free_substitute_exists` as a principle in `specs/master-plan-sequencing.json:865-867`.
The OCI memory directive requires Always Free constraints to be explicit, measurable, and upgradeable rather than hidden in prose.
The current cloud-network-dns demo_trial tenant_class says only that cost should stay below 5 USD per month.
That is not sufficient for the canonical OCI demo_trial tenant_class rule.
OCI demo_trial tenant_class must be a first-class operating profile, not a rough monthly target.
For `guest-on-oci`, demo_trial should map to an Always Free compute shape or equivalent Always Free capacity envelope.
For `guest-on-oci`, demo_trial should avoid paid managed DNS dependencies when an in-service authoritative DNS deployment can run inside the Always Free budget.
For `guest-on-oci`, demo_trial should declare resource ceilings for CPU, memory, disk, egress, authoritative QPS, zone count, record count, and health-check cadence.
For `guest-on-oci`, demo_trial should declare which optional features are disabled to protect the Always Free envelope.
For `guest-on-oci`, demo_trial should document whether DNSSEC keys are software-held, tenant-supplied, or HSM-backed.
For `guest-on-oci`, demo_trial should not claim HSM-backed DNSSEC because HSM custody is a paid-tenant_class promise and likely paid.
For `guest-on-oci`, demo_trial should not claim dedicated authoritative clusters because that is paid and paid.
For `guest-on-oci`, demo_trial should cap health-check probes to avoid paid observer infrastructure.
For `guest-on-oci`, demo_trial should cap retention to a local or tenant-owned low-cost store that fits the Always Free envelope.
For `guest-on-oci`, demo_trial should make secondary DNS optional and probably paid tenant_class because secondary providers or transfer partners can introduce cost.
For `guest-on-oci`, demo_trial should make DNS Firewall optional and paid tenant_class because firewall rule engines, threat feeds, and resolver-policy estates exceed the minimal baseline.
For `guest-on-oci`, demo_trial should make global anycast optional and paid tenant_class because Always Free cannot honestly promise global authoritative anycast infrastructure.
For `guest-on-oci`, demo_trial should make multi-region replication paid tenant_class because Always Free capacity does not provide a general multi-region substrate.
For `guest-on-oci`, demo_trial should make programmable routing policy paid tenant_class or paid tenant_class because it requires policy review, test lanes, and possibly additional compute.
For `guest-on-oci`, demo_trial should expose a migration path to paid when tenant demand exceeds QPS, zone count, record count, or retention limits.
For `guest-on-oci`, demo_trial should define an admission gate that refuses tenants whose expected DNS traffic would exceed the Always Free envelope.
For `guest-on-oci`, demo_trial should include an OpenTofu directory at `iac/guest-on-oci/always-free/` or the canonical context spelling selected by Wave 14.
The user prompt named `iac/oci-guest/always-free/`, while canonical context IDs use `guest-on-oci`.
This naming mismatch should be resolved centrally before implementation to avoid dual directory conventions.
The audit recommendation is to keep the master plan context ID `guest-on-oci` and add an alias only if the orchestrator requires compatibility.
demo_trial on `oyatie-public-cloud` can be small paid shared infrastructure rather than Always Free, but it should not inherit the OCI-specific cost claim.
demo_trial on `guest-on-aws` can map to low-cost EC2 or containerized authoritative DNS infrastructure, but AWS free-tenant_class semantics differ from OCI Always Free.
demo_trial on `on-prem` can map to a single-node or small HA authoritative deployment, with customer hardware cost excluded from Oyatie cloud spend.
demo_trial on `colo` can map to a small colocated authoritative pair, with rack and transit economics disclosed separately.
demo_trial on `oyatie-as-cloud-provider` can map to shared Oyatie authoritative clusters rather than tenant-dedicated clusters.
paid should be required on OCI when private DNS bindings are needed across multiple networks.
paid should be required on OCI when asynchronous replication across regions is required.
paid should be required on OCI when health checks require multiple observer locations.
paid should be required on OCI when tenant audit retention exceeds the demo_trial local storage envelope.
paid should be required on OCI when HSM custody, compliance-jurisdiction routing, synthetic probes, or break-glass governance are required.
paid should be required on OCI when regulated retention and export hooks require durable object storage or managed key systems beyond Always Free.
paid should be required on OCI when tenant-dedicated authoritative clusters, offline-root ceremonies, 7-year signed export bundles, or custom DR exercises are required.
The current demo_trial tenant_class cannot honestly claim OCI Always Free compliance until the profile is represented in docs and IaC.
The current tenant_class file should change the demo_trial cost line from a generic less-than-5-USD claim to an explicit per-context economics row.
The current tenant_class file should add a row named `OCI guest profile`.
The current tenant_class file should add a row named `Always Free admission ceiling`.
The current tenant_class file should add a row named `paid uplift triggers`.
The current tenant_class file should add a row named `disabled-on-demo_trial features`.
The current tenant_class file should cite the master plan `oci_always_free` object directly after the service-local profile lands.
The final reconciliation classification is `mis-tenant_classed` for demo_trial today and `fixable` once the profile and IaC directory land.

## §5 Findings by tier

### §5.1 demo_trial findings

demo_trial is conceptually correct as an entry tier.
demo_trial has a narrow implementation proof for tenant-scoped zone creation through the external OpenAPI contract and Rust tests.
demo_trial does not yet have enough local API surface to match AWS, Google, or Cloudflare for record management.
demo_trial does not yet have local evidence for DNSSEC automation.
demo_trial does not yet have local evidence for health checks.
demo_trial does not yet have local evidence for audit retention.
demo_trial does not yet have local Cedar policies.
demo_trial does not yet have local IaC.
demo_trial does not yet have a supported OS manifest.
demo_trial conflicts with the OCI Always Free doctrine because it uses a generic small-dollar cost target.
demo_trial is ahead only in the intent to make tenant-scoped governance part of the entry DNS tier.
demo_trial is at parity with counterparts for the idea of public authoritative zones.
demo_trial is catch-up for record APIs, DNSSEC operational proof, logging, analytics, DNS firewall, and deployment evidence.
demo_trial should be remediated first because every higher tenant_class inherits its missing primitives.

### §5.2 paid findings

paid has the right feature direction for AWS Route 53 and Google Cloud DNS private-zone parity.
paid declares private DNS, latency routing, geo routing, active-passive failover, delegated sub-zone automation, and change approvals.
paid does not prove those capabilities with contracts, schemas, IaC, tests, or runbooks.
paid trails AWS and Google most clearly on private DNS binding, forwarding, peering, resolver policies, and query logging.
paid trails Cloudflare most clearly on analytics, secondary DNS, zone transfer, and DNS Firewall features.
paid is ahead in the declared tenant-scoped DNS change approval workflow, but that advantage is documentation-only.
paid needs the first serious record-set, routing-policy, private-zone, health-check, and audit contracts.
paid also needs a context-by-context OpenTofu layout before it can be considered production buildable.
paid should be the minimum tenant_class for multi-region guest workloads that need private DNS.
paid should remain the uplift boundary for any OCI tenant that exceeds Always Free demo_trial ceilings.

### §5.3 paid findings

paid is the first tenant_class with regulated-production aspirations.
paid’s strongest additive idea is compliance-jurisdiction routing under policy.
paid’s strongest governance idea is policy-bound change windows plus break-glass controls.
paid’s strongest infrastructure claim is quorum-based replication across three or more active regions.
paid does not have the compliance, incident, capacity, SLO, or runbook documents needed to support that claim.
paid trails AWS on Resolver DNS Firewall, mature query logging, and account-level operational integrations.
paid trails Google on DNS Armor, response policies, forwarding and peering policy depth, and Cloud Monitoring integration.
paid trails Cloudflare on DNS Firewall, analytics, secondary DNS, and enterprise authoritative DNS operations.
paid is ahead only as a declared governance model, not as an implemented product.
paid needs policy schemas, audit export schemas, HSM custody design, and synthetic probe contracts before implementation can be sequenced.
paid should not inherit demo_trial’s cost model.
paid should be priced and deployed as regulated production infrastructure with explicit context overlays.

### §5.4 paid findings

paid is a credible ambition tenant_class but has the weakest local proof.
paid’s dedicated authoritative cluster promise is not backed by any local architecture document.
paid’s bring-your-own-authority promise is not backed by any zone-transfer, delegation, or authority-handoff protocol.
paid’s offline-root ceremony promise is not backed by a key ceremony runbook.
paid’s programmable routing promise is not backed by a policy-review API.
paid’s signed export bundle promise is not backed by a bundle format.
paid’s tenant-owned disaster recovery exercise promise is not backed by an exercise template.
paid trails AWS, Google, and Cloudflare on mature enterprise operating evidence even where the conceptual feature is comparable.
paid may be ahead conceptually on tenant-owned DR exercises and signed export bundles, but only after those are turned into contracts and runbooks.
paid must be isolated from OCI Always Free expectations.
paid must require paid capacity, dedicated topology, and explicit admission review.
paid should not be marketed as hyperscaler-equivalent until benchmark evidence, deployment topology, SLOs, and runbooks exist.

### §5.5 Cross-tenant_class remediation order

First, land a local PRD and local architecture document for cloud-network-dns.
Second, land record-set, routing-policy, health-check, private-zone, audit-event, and DNSSEC key-management contracts.
Third, land the demo_trial OCI Always Free profile and context-specific OpenTofu directories.
Fourth, land the supported OS manifest and CI lane plan for the canonical Tier-1 OS matrix.
Fifth, land OpenSLO files for latency, availability, propagation, replication lag, and change durability.
Sixth, land incident response, capacity model, cost budget, compliance, and DPIA documents.
Seventh, reconcile the current tenant_class matrix with the counterpart parity matrix so every tenant_class row has implementation ownership.
Eighth, split additive Oyatie governance features from counterpart parity features so Wave 14 can schedule them separately.
Ninth, replace tutorial-only proof with contracts and tests.
Tenth, add measured benchmarks in the build phase and mark current target numbers as targets only.

### §5.6 Final classification

demo_trial classification: `catch-up-build` with one `mis-tenant_classed` OCI economics issue.
paid classification: `catch-up-proof` for promised private DNS and routing features.
paid classification: `catch-up-build` with additive governance intent.
paid classification: `catch-up-build` and aspirational until dedicated-cluster architecture lands.
AWS Route 53 comparison: strongest pressure on public DNS, private DNS, routing policies, resolver firewall, health checks, and migration parity.
Google Cloud DNS comparison: strongest pressure on private zones, forwarding, peering, response policies, DNS Armor, and monitoring.
Cloudflare DNS comparison: strongest pressure on anycast public DNS, analytics, DNS Firewall, secondary DNS, zone transfers, load balancing, and enterprise plan expectations.
Oyatie additive surface: tenant-scoped Cedar-style governance, OpenTofu-only substrate, multi-context deployment doctrine, OCI Always Free demo_trial economics, signed export bundles, and tenant-owned DR exercises.
Current state: tenant_class ambition is coherent as a product ladder but incoherent as a build-ready µservice because local proof artifacts are missing.
Required Wave 14 aggregation action: preserve the four-tenant_class ladder but re-open every tenant_class row as implementation work until contracts, IaC, SLOs, OS manifests, and tests exist.
