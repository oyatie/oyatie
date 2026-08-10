# cloud-network-dns coherence audit — 2026-05-20

AGENT CLASS: microservice-ownership-coherence-audit-agent
AGENT SLUG: codex-cloud-network-dns-audit
MODE: audit-only
BUNDLE: cloud-network-dns-coherence-audit-2026-05-20
SCOPE: `/Users/jasonlee/oyatie/microservices/cloud-network-dns/`
VCS CLAIM: `./bin/oya vcs claim --agent codex-cloud-network-dns-audit --intent "cloud-network-dns ownership coherence audit" microservices/cloud-network-dns::coherence-audit-2026-05-20` accepted.

## Header citation anchors

1. ADR-0328 §D-15..§D-20: deployment contexts, OpenTofu, OS matrix, Rust-strict, OCI Always Free, and audit dimensions are canonical; key line ranges read include `docs/decisions/ADR-0700-ci-admission-live-apex.md:1730-1815`, `:2241-2365`, and `:3140-3235`.
2. Master plan machine contract: `specs/master-plan-sequencing.json:704-867` defines the six deployment contexts, OpenTofu substrate, supported OS matrix, Rust-strict language policy, and OCI Always Free profile.
3. Service PRD anchor: `network/dns/PRD.md` is absent in the service inventory; nearest service-local product-purpose evidence is `microservices/cloud-network-dns/retired tenant_class adoption artifact:7-10`.
4. Service architecture anchor: `microservices/cloud-network-dns/ARCHITECTURE.md` is absent in the service inventory; nearest service-local architecture-equivalent evidence is `network/dns/reference-implementations/provision-zone-dnssec-and-geo-routing-rust-sdk.md:1-5` and `network/dns/faqs/dns-engineer-faq.md:26-30`.
5. Documentation rigor anchor: `docs/standards/documentation-rigor.md:62-80` requires the full per-microservice doc set, and `docs/standards/documentation-rigor.md:133-156` defines intern-buildability plus hyperscaler-grade rigor.

## Investigation basis

- Files inventoried: 7 service-local files.
- Service-local lines audited: 1,219 lines from all seven existing Markdown artifacts.
- Required service-local artifact classes missing: PRD, ARCHITECTURE, README, decisions, implementation plans, contracts, SLOs, IaC, supported OS manifest, source, tests, runbooks, compliance, incident response, capacity model, cost budget, DPIA, cross-microservice handoffs.
- Chat-history matches processed: 7 relevant lines from 34 raw `cloud-network-dns` matches in `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl`.
- Counterpart evidence basis: AWS Route 53, Google Cloud DNS, and Cloudflare DNS official documentation, plus DNSPerf methodology disclosure for public DNS performance comparability.
- Halt-cleanly invoked: no; source evidence is thin but sufficient to produce a substantive gap audit without inventing service ownership.

## §1 Microservice purpose summary

The current service-local docs present `cloud-network-dns` as Oyatie's DNS substrate, not as a thin adapter over AWS Route 53, Google Cloud DNS, Cloudflare DNS, NS1, or Akamai.
That purpose is explicit in `retired tenant_class adoption artifact:7-10`, which says the service owns authoritative and recursive DNS, zone scoping, DNSSEC, routing, encrypted transports, and anycast IP advertising.
The FAQ repeats the displacement claim at `faqs/dns-engineer-faq.md:7-11`, where it states that tenants keep registrars but point NS records at Oyatie name servers.
The onboarding guide turns that purpose into a week-one path: provision a tenant zone, enable DNSSEC, configure geo-routing, attach health checks, and exercise DoH/DoQ transports (`onboarding/dns-engineer-first-week.md:3-5`).
The tutorial provides the most concrete operator path, including `./bin/oya dns zone create`, record creation, DNSSEC enablement, geo records, health checks, transport queries, and query-log inspection (`tutorials/provision-zone-dnssec-geo-routing-and-doq.md:11-222`).
The reference implementation positions the SDK as a Rust client capable of creating a tenant zone, enabling DNSSEC, adding geo-routed record sets, adding health checks, and querying via DoH/3 and DoQ (`reference-implementations/provision-zone-dnssec-and-geo-routing-rust-sdk.md:1-5`).
The current service-local product story is therefore broad: authoritative DNS, recursive resolver behavior, registrar-facing DNSSEC chain-of-trust, traffic steering, health monitoring, telemetry, audit anchoring, Cedar permissions, migration from Route 53/NS1, and private/internal DNS.
The implementation evidence outside the service path is much narrower: the OpenAPI contract is a tenant-scoped DNS zone create contract only (`network/dns/contracts/openapi/cloud/cloud-network-dns-v1.yaml:1-12`).
The Rust API crate similarly describes tenant DNS zone creation, request normalization, idempotency, and authenticated projection before handing typed zone creation to the cloud network kernel (`network/ports/dns/src/lib.rs:1-5`).
The runtime tests prove public-zone creation, idempotent replay, and private-zone binding to a known VPC (`network/ports/dns/tests/cloud_network_dns_api.rs:146-211`).
This creates the core audit result: the service docs describe a full hyperscaler DNS product, but current local contract and runtime evidence only prove a zone-create control-plane slice.
The right product boundary should remain ambitious because ADR-0328 places `cloud-network-dns` in Phase 0 shared infrastructure (`docs/decisions/ADR-0700-ci-admission-live-apex.md:450-465`).
The service belongs in the network seam for public cloud and provider-mode contexts (`docs/decisions/ADR-0700-ci-admission-live-apex.md:1752-1754` and `:1996`).
The docs cannot yet support a GA claim across all six deployment contexts because no service-local manifest, IaC modules, SLOs, policies, or OpenTofu state-backend declarations exist.
The product purpose is coherent as a target: Oyatie needs DNS as its own IaaS/network substrate to avoid a provider-wrapped Route 53 or Cloud DNS dependency.
The artifact set is incoherent as a build package: the most critical documents and machine-readable surfaces are absent from the microservice directory.
The deliverable posture is therefore REVISE-WITH-P1-GAPS, not reject the product idea.
The audit recommends preserving the DNS-substrate ambition while reducing current claims to "design target" until PRD, architecture, contract suite, IaC, OS manifest, source ownership, SLOs, and policies land.

## §2 Inventory snapshot

| File | Lines | Approx bytes | Role | Coherent with purpose? |
|---|---:|---:|---|---|
| `benchmarks/cloud-network-dns-vs-route53-vs-cloud-dns-vs-cloudflare-vs-ns1.md` | 119 | audited | Vendor benchmark narrative and TCO comparison | partial: useful target numbers, but measured evidence path is missing (`benchmarks/...:3-5`, `:119`) |
| `retired tenant_class adoption artifact` | 96 | audited | demo_trial/paid tenant_class tenant_class definitions | partial: purpose and tenant_class axes are strong, but OCI demo_trial tenant_class Always Free is not stated (`retired tenant_class adoption artifact:12-28`) |
| `faqs/dns-engineer-faq.md` | 167 | audited | DNS engineer FAQ and design assumptions | partial: rich DNS domain coverage, but policy/events/source surfaces are not present (`faqs/dns-engineer-faq.md:82-93`) |
| `migration-playbooks/from-route53-and-ns1.md` | 173 | audited | Route 53 and NS1 migration playbook | partial: concrete migration flow, but it prescribes shell/jq loops and manual registrar work (`migration-playbooks/from-route53-and-ns1.md:16-28`, `:82-87`) |
| `onboarding/dns-engineer-first-week.md` | 201 | audited | Engineer onboarding guide | partial: clear week-one path, but uses `make` and manual DS publication (`onboarding/dns-engineer-first-week.md:22-24`, `:96`) |
| `reference-implementations/provision-zone-dnssec-and-geo-routing-rust-sdk.md` | 241 | audited | Rust SDK example | partial: Rust shape is appropriate, but referenced SDK crate/version is not proven service-local (`reference-implementations/...:14-22`) |
| `tutorials/provision-zone-dnssec-geo-routing-and-doq.md` | 222 | audited | End-to-end tutorial | partial: buildable commands, but uses `make`, a shell loop, and manual DS publication (`tutorials/...:6-8`, `:80`, `:106-120`) |
| `PRD.md` | 0 | missing | Product requirements | no: required by documentation-rigor (`docs/standards/documentation-rigor.md:64-66`) |
| `ARCHITECTURE.md` | 0 | missing | Architecture and system boundary | no: required by documentation-rigor (`docs/standards/documentation-rigor.md:64-66`) |
| `README.md` | 0 | missing | Service entrypoint | no: required by documentation-rigor (`docs/standards/documentation-rigor.md:64-66`) |
| `contracts/` | 0 | missing service-local | API/event/proto contracts | no: documentation-rigor requires OpenAPI, AsyncAPI, proto surfaces (`docs/standards/documentation-rigor.md:70`) |
| `slos/` | 0 | missing | OpenSLO evidence | no: documentation-rigor requires OpenSLO docs (`docs/standards/documentation-rigor.md:73`) |
| `iac/` | 0 | missing | OpenTofu deployment modules | no: ADR-0328 requires per-service and per-context IaC (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2275-2295`) |
| `supported-oses.json` or manifest field | 0 | missing | OS matrix | no: master plan requires per-microservice manifest (`specs/master-plan-sequencing.json:777-815`) |
| `src/` and service-local `Cargo.toml` | 0 | missing | Runtime implementation | no: runtime code is outside service path and proves only zone-create slice (`network/ports/dns/src/lib.rs:1-5`) |
| `tests/` | 0 | missing service-local | Test evidence | no: tests are outside service path and only prove zone-create behavior (`network/ports/dns/tests/cloud_network_dns_api.rs:146-211`) |

Inventory verdict: the service-local directory is a seven-document design pack, not a full microservice ownership bundle.
Inventory risk: every service-local artifact is newly added or untracked in the current worktree, so orchestrator aggregation should review ownership provenance before assuming these are canonical.
Inventory count: 7 files seen, 1,219 lines read, 0 service-local machine-readable contracts, 0 service-local OpenTofu modules, 0 service-local source files, 0 service-local tests.

## §3 9-dimension audit

### §3.1 Dimension 1 — internal coherence within the microservice path

1. Purpose claim resolves internally: `retired tenant_class adoption artifact:7-10` and `faqs/dns-engineer-faq.md:7-11` both present this service as the native DNS substrate replacing vendor DNS.
2. Onboarding path aligns with purpose: week-one target includes zone creation, DNSSEC, geo-routing, health checks, and encrypted transport (`onboarding/dns-engineer-first-week.md:3-5`).
3. Tutorial path aligns with onboarding: it repeats zone creation, DNSSEC, geo-routing, health checks, and transport queries (`tutorials/provision-zone-dnssec-geo-routing-and-doq.md:1-4`).
4. Reference implementation aligns with tutorial: it creates a zone, enables DNSSEC, configures geo routing, attaches health checks, and queries encrypted transports (`reference-implementations/...:1-5`).
5. Capability tenant_class demo_trial says no DNSSEC (`retired tenant_class adoption artifact:18-20`), while onboarding chooses paid for DNSSEC and tutorial chooses paid; this is coherent by tenant_class if demo_trial remains excluded.
6. Capability tenant_class paid says health-check interval is 30 seconds (`retired tenant_class adoption artifact:40`), and FAQ says paid failover is 90 seconds from 30-second interval times three failures (`faqs/dns-engineer-faq.md:57-60`); this resolves.
7. Capability tenant_class paid says 10-second interval and failover within 30 seconds (`retired tenant_class adoption artifact:58`), matching the FAQ (`faqs/dns-engineer-faq.md:57-60`) and tutorial expected failover within 30 seconds (`tutorials/...:123-139`).
8. Capability tenant_class paid says one-second interval and BGP withdraw under eight seconds (`retired tenant_class adoption artifact:75-79`), matching FAQ line 60; this resolves.
9. Capability tenant_class demo_trial says DoH is included (`retired tenant_class adoption artifact:23`), but tutorial's full transport exercise uses paid (`tutorials/...:6-8`); no contradiction because paid includes all transports in the current story.
10. Capability tenant_class paid says ODoH is available (`retired tenant_class adoption artifact:59`), and tutorial exercises ODoH (`tutorials/...:186-192`); this resolves.
11. Capability tenant_class paid says DoQ is included (`retired tenant_class adoption artifact:77`), but tutorial exercises DoQ at paid (`tutorials/...:179-184`); this is a P2 inconsistency unless paid intentionally includes DoQ through the "all supported transport" tutorial path.
12. Benchmark doc says paid supports UDP, TCP, DoH, DoT, DoQ, and ODoH (`benchmarks/...:60-72`), supporting the tutorial's paid DoQ/ODoH exercises; this suggests the tenant_class matrix line 77 is additive language, not exclusive.
13. Capability tenant_class demo_trial claims substrate cost around $5/month (`retired tenant_class adoption artifact:28`), but OCI Always Free doctrine requires OCI demo_trial tenant_class to fit Always Free (`feedback_oci_always_free_maximization_2026_05_20.md:74-82`); this is a P1 gap for OCI context.
14. Capability tenant_class paid pricing is $640/month (`retired tenant_class adoption artifact:64`), and migration playbook explicitly says AWS/GCP/Azure/Cloudflare are much cheaper at mid-market scale (`migration-playbooks/from-route53-and-ns1.md:168-173`); this resolves as an honest tradeoff.
15. Benchmark TCO table also says AWS/GCP/Azure/Cloudflare are cheaper than paid at mid-market scale (`benchmarks/...:74-88`); this supports migration playbook tradeoff lines.
16. FAQ says registrar stays external and NS records point to Oyatie (`faqs/dns-engineer-faq.md:7-11`), while onboarding/tutoring require manual DS publication at registrar (`onboarding/...:96`, `tutorials/...:80`); this is a P2 gap because registrar automation is identified as future.
17. The zero-handroll doctrine forbids manual DNS mutation in onboarding (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1782-1783`); manual DS publication therefore escalates to P1 for production onboarding claims.
18. Onboarding Day 2 uses `make dev-cell.up` and `make dev-tenant.create` (`onboarding/...:22-24`), while Rust-strict canonical build forbids `make` as backend build invocation (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3215-3231`); P2 doc-invocation drift.
19. Tutorial prerequisites repeat `make` commands (`tutorials/...:6-8`); same P2 drift.
20. Benchmark reproducibility also uses `make benchmarks.cloud-network-dns.run` (`benchmarks/...:110-117`); this conflicts with the cargo-only build lane for release evidence.
21. Migration playbook uses `jq | while read` loops for Route 53 and NS1 exports (`migration-playbooks/...:16-28`, `:33-38`); Rust-strict memory forbids shell beyond tiny glue and migration scripts (`feedback_rust_strict_only_no_python_2026_05_20.md:51-64`).
22. Tutorial health-check creation uses a shell `for region in na eu apac` loop (`tutorials/...:106-120`); this should be replaced by Rust CLI or explicit repeated commands for audit-grade docs.
23. Capability tenant_class references Cedar permits (`retired tenant_class adoption artifact:26`, `:44`, `:62`, `:80`) but no `policy/` or `policies/` directory exists; P1 missing policy surface.
24. FAQ references Cedar-gated query stream and emergency failover (`faqs/dns-engineer-faq.md:82-93`) without service-local Cedar fragments; same P1.
25. Capability tenant_class says record CRUD writes `cloud_network_dns.record.*` audit events (`retired tenant_class adoption artifact:90-91`), but no service-local AsyncAPI or event schema exists; P1 missing event contract.
26. FAQ says realtime query telemetry is a Kafka stream (`faqs/...:82-86`), but service-local contracts include no AsyncAPI channel; P1 missing event stream contract.
27. Tutorial query-log output exercises per-query observability (`tutorials/...:194-213`), but no dashboard, OpenSLO, log schema, or observability handoff exists; P2 observability surface gap.
28. FAQ says private zones integrate with `cloud-network` (`faqs/...:126-129`), but no cross-microservice handoff file exists in the service path; P2 missing handoff.
29. FAQ says `cloud-iac` can declaratively create matching DNS records (`faqs/...:141-145`), but no service-local IaC contract declares the API surface; P1 because IaC is a canonical cross-cutting dimension.
30. Migration playbook says KSK import uses HSM and contains a contradictory phrase "paid-only at paid; paid tenant_class allows software-key import" (`migration-playbooks/...:88-98`); P1 hard contradiction in tenant_class semantics.
31. Capability tenant_class says paid tenant_class DNSSEC supports software keys until paid HSM (`retired tenant_class adoption artifact:38`, `:74-80`); this clarifies the migration playbook should say HSM import is paid-only and software-key import is paid/paid.
32. FAQ Q4 says paid HSM uses Thales Luna 7 plus Utimaco Se Gen2 (`faqs/...:33-37`), but no HSM dependency or OpenTofu module exists; P2 because doc is concrete but unsupported by deployable surfaces.
33. FAQ Q6 uses MaxMind GeoIP2 plus RIPE NCC data (`faqs/...:50-53`), but no data-source update plan or licensing note exists; P2 operational gap.
34. Capability tenant_class claims BGP anycast advertising (`retired tenant_class adoption artifact:7-10`, `:94`), but no FRR/BGP/OpenTofu/network adapter docs exist; P1 for a network substrate.
35. Benchmark doc claims measured data across dates (`benchmarks/...:3-5`) but the cited evidence path is absent (`benchmarks/...:119`); P1 evidence gap.
36. Benchmark doc includes Azure DNS, NS1, and Akamai in addition to the requested top-three counterparts (`benchmarks/...:1`); not a contradiction, but audit top-three union coverage should focus on Route 53, Cloud DNS, and Cloudflare DNS.
37. Onboarding references `docs/adr-archive/ADR-0253-network-topology-edge-service-mesh.md` (`onboarding/...:9-11`), but the exact file does not exist; P2 broken internal-reference path.
38. Capability tenant_class cites ADR-0253 more generally (`retired tenant_class adoption artifact:4-5`); this can resolve to existing ADR-0253 amendment or topology files only after path correction.
39. Reference implementation uses `oya-cloud-network-dns-sdk = "0.42.0"` (`reference-implementations/...:14-22`), but the repo evidence read only proves `oya-cloud-network-dns-api`; P2 SDK provenance gap.
40. Reference implementation tests mention an in-process Knot DNS resolver and SoftHSM (`reference-implementations/...:229-236`), but no service-local test fixtures exist; P2 test evidence gap.
41. FAQ says custom RR codecs live under `crates/oya-cloud-network-dns-rrtype-tenant-<tenant>/` (`faqs/...:64-68`), but no such service-local governance doc exists; P2 extensibility governance gap.
42. Capability tenant_class claims HSM attestation receipts at paid (`retired tenant_class adoption artifact:95-96`), but no receipt schema exists; P2 compliance evidence gap.
43. Tutorial expected output includes `audit_chain_event` (`tutorials/...:23-29`), matching capability tenant_class audit-chain invariant (`tenant-class-adoption/...:90-91`); concept resolves but contract absent.
44. Onboarding expected output includes `delegation_status : not-yet-delegated` (`onboarding/...:37-44`), matching the registrar-retained model in FAQ (`faqs/...:7-11`).
45. Migration dual-NS shadow phase (`migration-playbooks/...:100-121`) coheres with zero-downtime migration and vendor coexistence.
46. Migration phase 5 says delete Route 53 zones after 30 days (`migration-playbooks/...:140-145`); rollback lines `:147-156` state re-adding Route 53/NS1 records after cutover, which is coherent only if zones were kept until the clean-run window passes.
47. FAQ says rollback rewrites zone to version N and re-signs within 30 seconds p95 (`faqs/...:156-167`), but no rollback contract exists; P2 missing API/runbook.
48. Capability tenant_class says paid supports private RR types and PQC DNSSEC (`tenant-class-adoption/...:72-80`), but no per-tenant_class compliance caveat says experimental DNSSEC must not be GA-default; P2 launch-risk gap.
49. Internal coherence verdict: the seven docs share a recognizable product thesis but exceed the proven surface by a wide margin.
50. Severity summary for Dimension 1: P1 hard gaps include missing policy/event/IaC surfaces, contradictory HSM import language, and unsupported measured benchmark claims; P2 gaps include broken references, command drift, and missing test/observability surfaces.

### §3.2 Dimension 2 — outbound cross-references and inbound references

1. ADR-0248 reference in onboarding resolves: `onboarding/dns-engineer-first-week.md:9` points to `docs/decisions/ADR-0700-ci-admission-live-apex.md`, and the file exists.
2. ADR-0253 exact path in onboarding does not resolve: `onboarding/...:10` names `docs/adr-archive/ADR-0253-network-topology-edge-service-mesh.md`, but the repo contains differently named ADR-0253 files.
3. Capability tenant_class cites ADR-0244, ADR-0248, ADR-0253, and ADR-0316 without paths (`retired tenant_class adoption artifact:3-5`); these are partially resolvable but should be exact links.
4. Capability tenant_class cites RFC 4033-4035, RFC 8484, RFC 7858, and RFC 9230 (`tenant-class-adoption/...:4-5`); standards are appropriate external references for DNSSEC, DoH, DoT, and ODoH.
5. Onboarding cites the same RFC set plus RFC 9250 for DoQ (`onboarding/...:9-11`); resolves conceptually.
6. Tutorial uses `curl --http3`, `dig`, and `kdig` as tools (`tutorials/...:6-9`); tool dependencies are not tied to OS manifest or package docs.
7. Migration playbook cites Route 53 and NS1 source systems through commands (`migration-playbooks/...:10-44`), but lacks API-version provenance.
8. FAQ names MaxMind GeoIP2 and RIPE NCC routing data (`faqs/...:50-53`); no data-license or refresh-runbook target exists.
9. FAQ names Apple Private Relay and Cloudflare ODoH proxy (`faqs/...:41-46`); external privacy interoperability is a product feature but not in contract.
10. FAQ names Thales Luna 7 and Utimaco Se Gen2 HSM (`faqs/...:33-37`); no service-local hardware support matrix exists.
11. FAQ names `cloud-network`, `cloud-iac`, Foundry, Cedar, `audit-chain`, and `observability` (`faqs/...:82-153`); no service-local cross-microservice handoff file exists.
12. ADR-0273 references this service for DKIM/SPF/DMARC publication and propagation probing (`docs/decisions/ADR-0700-ci-admission-live-apex.md:326-365`, `:443-465`).
13. ADR-0273 expects `microservices/cloud-network-dns/iac/helm/dns-orchestrator/` (`docs/adr-archive/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md:1330-1332`), but no `iac/` directory exists; P1 inbound broken expectation.
14. `docs/products/cloud/PRD.md:138-143` lists `oya-cloud-network-dns-api` as the DNS zone create REST API, narrowing the runtime surface.
15. `docs/products/cloud/PRD.md:161-176` places the DNS contract in the VPC/Network API family with p99 <=500 ms create-boundary SLO.
16. `network/dns/contracts/openapi/cloud/cloud-network-dns-v1.yaml:1-12` confirms the only read contract is zone create.
17. `network/ports/dns/src/lib.rs:1-5` confirms API-boundary ownership is zone creation, request normalization, idempotency, and authenticated projection.
18. `network/ports/dns/tests/cloud_network_dns_api.rs:146-211` confirms tests for zone-create idempotency and private-zone VPC binding.
19. Chat history line 12411 records 0/7 doc-surface coverage for `cloud-network-dns` before the Wave 9 gapfill.
20. Chat history line 12417 dispatched Wave 9 to create seven docs and named the DNS substrate hook: per-tenant zone scoping, DNSSEC, geo-routing, latency routing, and health checks.
21. Chat history line 15231 names current Wave 2 Batch 2.1 audit scope and includes `cloud-network-dns`.
22. Chat history line 15245 confirms the one-per-microservice no-collision dispatch for this audit batch.
23. Inbound references from `microservices/mail/PRD.md` mention `cloud-network-dns` for custom-domain DKIM/SPF/DMARC publication; this creates a cross-service dependency that this service cannot currently fulfill from service-local contracts.
24. Inbound references from docs/architecture Wave 3 materials classify `cloud-network-dns` as DNS control that still needs runtime proof; this aligns with this audit.
25. Inbound references from machine-readable contract registries point to the OpenAPI and runtime crate, not to the broad DNS substrate docs.
26. Orphan risk: service-local docs name events like `cloud_network_dns.record.*` but no central registry entry was proven in this pass; this must be checked in Wave 14 aggregation.
27. Reverse-reference gap: service docs refer to `cloud-iac`, but `microservices/cloud-network-dns/cross-microservice-handoffs.md` is absent, so the reverse handoff cannot be verified.
28. Reverse-reference gap: service docs refer to `audit-chain`, but no service-local event contract is present.
29. Reverse-reference gap: service docs refer to `observability`, but no dashboard, metric schema, or OpenSLO target is present.
30. Reverse-reference gap: service docs refer to `cloud-network`, but no private-zone/VPC handoff doc exists.
31. Reverse-reference gap: ADR-0273 expects a Helm DNS orchestrator under the service; service has no IaC or Helm path.
32. Outbound reference quality is mixed: standards/RFCs are strong; internal file links are often loose or broken.
33. Inbound reference quality is more precise than service-local docs: external contracts state zone create only.
34. Product-surface mismatch: service-local docs imply authoritative plus recursive DNS; OpenAPI contract says only zone create.
35. Product-surface mismatch: service-local docs imply query streams; no AsyncAPI contract in service path.
36. Product-surface mismatch: service-local docs imply health checks; no health-check API contract in service path.
37. Product-surface mismatch: service-local docs imply DNSSEC enablement; no DNSSEC API contract in service path.
38. Product-surface mismatch: service-local docs imply record CRUD; no record CRUD OpenAPI contract in service path.
39. Product-surface mismatch: service-local docs imply routing policy CRUD; no routing policy schema in service path.
40. Product-surface mismatch: service-local docs imply ODoH proxy/target; no resolver transport contract exists.
41. Product-surface mismatch: service-local docs imply BGP anycast; no cloud-network/BGP contract exists.
42. Product-surface mismatch: service-local docs imply HSM-backed KSK/ZSK; no cloud-kms/cloud-secrets handoff exists.
43. Product-surface mismatch: service-local docs imply query log; no logging schema exists.
44. Product-surface mismatch: service-local docs imply versioned rollback; no zone-version contract exists.
45. Product-surface mismatch: service-local docs imply BYO-KSK/PKCS#11; no import contract exists.
46. Product-surface mismatch: service-local docs imply custom RR codec plugin; no registry or sandbox contract exists.
47. Dimension 2 severity: one P1 inbound broken expectation (`iac/helm/dns-orchestrator`) and many P2 missing reverse references.
48. Required remediation: add exact internal links, service-local handoff doc, contract index, and reverse dependency map.
49. Required Wave 14 question: decide whether mail DKIM publication belongs in this service's first implementation slice or only in a later `record_create`/TXT surface.
50. Dimension 2 verdict: cross-reference graph is recoverable but currently not graph-traversable under documentation-rigor `docs/standards/documentation-rigor.md:121-129`.

### §3.3 Dimension 3 — substance bar and intern-buildability

1. A cold intern can understand the intended DNS product from seven docs; the purpose statement is concrete (`retired tenant_class adoption artifact:7-10`).
2. A cold intern cannot build the service from the service directory because `PRD.md` is absent, contrary to the mandatory artifact roster (`docs/standards/documentation-rigor.md:64-66`).
3. A cold intern cannot build the architecture because `ARCHITECTURE.md` is absent, also contrary to the mandatory roster (`docs/standards/documentation-rigor.md:64-66`).
4. A cold intern cannot discover service-local APIs because `contracts/` is absent despite the contract requirement (`docs/standards/documentation-rigor.md:70`).
5. A cold intern cannot discover SLOs because `slos/` is absent despite the OpenSLO requirement (`docs/standards/documentation-rigor.md:73`).
6. A cold intern cannot deploy the service because `iac/` is absent despite ADR-0328's per-service IaC directory contract (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2275-2295`).
7. A cold intern cannot verify OS support because the manifest/supported_oses field is absent despite `specs/master-plan-sequencing.json:777-815`.
8. A cold intern cannot build the runtime from the service path because no `src/` or service-local `Cargo.toml` exists.
9. A cold intern can find an external Rust API crate, but it only proves zone-create behavior (`network/ports/dns/src/lib.rs:1-5`).
10. A cold intern can run conceptual CLI steps, but the commands reference `make` and service-specific `./bin/oya dns` surfaces that are not proven by service-local source (`onboarding/...:22-35`, `tutorials/...:6-21`).
11. A cold intern can copy the reference implementation, but the SDK crate `oya-cloud-network-dns-sdk` version is not proven in the service path (`reference-implementations/...:14-22`).
12. A cold intern cannot implement Cedar policy because no policy directory defines `cloud_network_dns::Action::*` despite tier/FAQ references (`tenant-class-adoption/...:26-80`, `faqs/...:82-93`).
13. A cold intern cannot implement audit events because no event schema defines `cloud_network_dns.record.*` despite record CRUD audit claims (`tenant-class-adoption/...:90-91`).
14. A cold intern cannot implement query streams because no AsyncAPI/Kafka contract exists despite FAQ Q10 (`faqs/...:82-86`).
15. A cold intern cannot implement health checks precisely because docs describe targets, intervals, thresholds, and body checks but not storage model, worker state machine, error taxonomy, or API schema (`tutorials/...:106-153`).
16. A cold intern cannot implement routing policy precedence because docs name geo, latency, weighted, failover, and RUM-correlation but omit conflict rules (`tenant-class-adoption/...:39-59`, `migration-playbooks/...:65-69`).
17. A cold intern cannot implement DNSSEC lifecycle fully because docs include algorithm and cadence values but not key states, rollover timeline tables, DS publication API, or parent-zone failure behavior (`tenant-class-adoption/...:38-39`, `onboarding/...:72-103`).
18. A cold intern cannot implement registrar integration because docs explicitly say it is not automatic (`onboarding/...:96`, `tutorials/...:80`).
19. A cold intern cannot implement private zones because FAQ names VPC integration but omits resolver authorization model and cross-tenant deny behavior (`faqs/...:126-129`).
20. A cold intern cannot implement reverse DNS because FAQ says dedicated egress IPs get PTR zones but no IPAM handoff is specified (`faqs/...:119-123`).
21. A cold intern cannot implement BGP anycast because FAQ gives high-level range advertisement but no AS, route-origin, RPKI, withdraw, or cell-failure policy (`faqs/...:26-30`).
22. A cold intern cannot implement HSM-bound signing because FAQ names HSM products but omits cloud-kms/cloud-secrets interfaces and attestation schema (`faqs/...:33-37`).
23. A cold intern cannot implement ODoH safely because FAQ names proxy and target roles but omits key distribution, proxy trust boundary, and abuse handling (`faqs/...:41-46`).
24. A cold intern cannot implement custom RR codec plugins because FAQ names tenant crate paths but omits sandbox, ABI, versioning, and validation rules (`faqs/...:64-68`).
25. A cold intern cannot implement rollback because FAQ provides a command but no snapshot/version schema, consistency semantics, or DNSSEC re-signing failure branch (`faqs/...:156-167`).
26. A cold intern cannot implement Cloudflare/Route 53/NS1 migration because playbook exports data but no import schema, mapping table, or validation report schema is service-local (`migration-playbooks/...:46-75`).
27. The benchmark artifact is not intern-buildable evidence because it claims measurements but the evidence directory is absent (`benchmarks/...:3-5`, `:119`).
28. Documentation-rigor requires named failure modes, capacity math, observability hooks, rollback paths, multi-region behavior, sovereign-cell behavior, and versioning (`docs/standards/documentation-rigor.md:143-156`); only rollback and partial observability are sketched.
29. Capacity math is missing: tenant_class QPS and latency values are asserted without derivation (`tenant-class-adoption/...:24-25`, `:42-43`, `:60-61`, `:78-79`).
30. Failure-mode tree is missing: docs mention rookie traps but not system behavior under partition, DNSSEC key compromise, anycast hijack, or resolver cache poisoning (`onboarding/...:191-201`).
31. Observability hooks are partial: query-log example exists, but metric names and alert policies are absent (`tutorials/...:194-213`).
32. Rollback is partial: FAQ command exists, but no contract or replay protection is specified (`faqs/...:156-167`).
33. Multi-region behavior is partial: FAQ names 16+ regions and cell replication within 8 seconds, but no region table or RPO/RTO math exists (`faqs/...:26-30`, `:97-101`).
34. Sovereign-cell behavior is partial: FAQ says paid air-gapped DNS avoids public internet, but no context-specific OpenTofu or OS support exists (`faqs/...:15-22`).
35. Versioning/deprecation is absent: no API versioning, RR codec ABI versioning, event versioning, or transport deprecation policy is service-local.
36. The docs are more useful than scaffold: the seven artifacts include real commands, product surfaces, tiers, and named counterpart gaps.
37. The docs are still below substance bar because the mandatory machine-readable and deployable surfaces are missing.
38. The current best intern-buildable slice is "zone create control-plane API", using `network/dns/contracts/openapi/cloud/cloud-network-dns-v1.yaml:1-12` and crate tests `:146-211`.
39. The current service-local docs should label all broad DNS substrate capabilities as target-state until contracts/source/IaC land.
40. Required remediation: add PRD with bounded first deliverable, architecture with DNS authoritative/recursive/control-plane boundaries, contract index, and slice map to existing Rust crate.
41. Required remediation: add OpenAPI for record CRUD, DNSSEC enable/disable, routing policy CRUD, health-check CRUD, query-log read, and zone rollback.
42. Required remediation: add AsyncAPI for query telemetry and audit events.
43. Required remediation: add OpenSLO definitions matching tenant_class matrix numbers.
44. Required remediation: add policy fragments for every Cedar permit named.
45. Required remediation: add failure-modes, incident-response, capacity-model, and cost-budget docs before any GA/hyperscaler claim.
46. Required remediation: add service-local tests or cross-reference external crate tests in a manifest.
47. Required remediation: replace `make` and shell loops with cargo/OpenTofu/Rust CLI flows.
48. Required remediation: convert manual DS publication into a registrar integration boundary or mark it as external prerequisite with zero-handroll exception rationale.
49. Dimension 3 verdict: not intern-buildable as a full microservice; buildable only as a target-state concept plus an external zone-create API slice.
50. Severity summary: P1 for missing mandatory docs/contracts/IaC/policies/events; P2 for incomplete failure/capacity/rollback/observability details.

### §3.4 Dimension 4 — canonical-direction alignment

1. Multi-context doctrine applies because this service is a network substrate (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1732-1734`).
2. Six contexts are canonical: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider` (`specs/master-plan-sequencing.json:704-745`).
3. Service-local docs do not enumerate any six-context support matrix; P1 drift from `feedback_multi_context_provider_agnostic_2026_05_20.md:32-38`.
4. Service-local docs imply all contexts through sovereign, on-prem, and provider-displacement claims, but do not bind the claim to context-specific deployment mechanics.
5. `oyatie-public-cloud` requires `iac/oyatie-public-cloud/` (`specs/master-plan-sequencing.json:709-712`); absent.
6. `guest-on-aws` requires `iac/guest-on-aws/` (`specs/master-plan-sequencing.json:715-718`); absent.
7. `guest-on-oci` requires `iac/oci-guest/` (`specs/master-plan-sequencing.json:721-724`); absent.
8. `on-prem` requires `iac/on-prem/` (`specs/master-plan-sequencing.json:727-730`); absent.
9. `colo` requires `iac/colo/` (`specs/master-plan-sequencing.json:733-736`); absent.
10. `oyatie-as-cloud-provider` requires `iac/oyatie-iaas/` (`specs/master-plan-sequencing.json:739-742`); absent.
11. OpenTofu substrate is canonical and Terraform/Pulumi/CloudFormation are forbidden engines (`specs/master-plan-sequencing.json:747-775`).
12. Service-local grep found no Terraform/Pulumi/CloudFormation references; aligned on forbidden engine naming.
13. Service-local path has no OpenTofu modules, state backend, module signing, or `tofu` invocation; P1 missing IaC.
14. ADR-0328 requires module signing through sigstore and cosign (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2357-2365`); no service-local signing plan exists.
15. State backend by context is canonical (`specs/master-plan-sequencing.json:758-765`); no service-local state reference exists.
16. OS support is canonical with 13 Tier-1 OS targets (`specs/master-plan-sequencing.json:777-793`).
17. Service-local path has no supported OS manifest; P1 drift.
18. Service-local docs mention OCI/Oracle indirectly only through counterpart/vendor material, not OS support.
19. Rust-strict backend is canonical (`specs/master-plan-sequencing.json:817-855`).
20. Service-local forbidden source-file grep found no `*.py`, `*.js`, `*.ts`, `*.rb`, `*.go`, `*.java`, `*.scala`, `*.groovy`, `*.php`, or `*.fs` files; aligned on file inventory.
21. Authorized non-Rust service-local files are Markdown only; aligned with `specs/master-plan-sequencing.json:828-839`.
22. Docs contain shell loops and `make` commands; drift from `feedback_rust_strict_only_no_python_2026_05_20.md:51-64`.
23. Canonical backend build invocation is `cargo build --workspace --release --all-features --locked` (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3215-3219`).
24. Service-local docs use `cargo run --release` and `cargo test --features hermetic` in the reference implementation (`reference-implementations/...:201-233`), which is acceptable for example/test but not a full release claim.
25. OCI Always Free is canonical for `guest-on-oci` demo/sandbox/trial/dev tenants (`specs/master-plan-sequencing.json:857-867`).
26. Service tenant_class matrix does not state OCI demo_trial tenant_class = Always Free; P1 drift from `feedback_oci_always_free_maximization_2026_05_20.md:74-82`.
27. demo_trial tenant_class claims `$5/mo` substrate cost (`tenant-class-adoption/...:28`), conflicting with OCI Always Free profile for OCI demo_trial tenant_class.
28. demo_trial record cap of 25 records and 100 QPS could plausibly fit OCI Always Free, but no capacity-check test proves it.
29. paid claims unlimited zones and unbounded QPS (`tenant-class-adoption/...:71-79`), which cannot fit OCI Always Free and must be paid tenant_class or paid context.
30. Documentation-rigor requires full doc set and intern-buildability (`docs/standards/documentation-rigor.md:62-81`, `:133-156`).
31. Service path has 7 artifacts, far below the >=70 artifact floor.
32. The user-required ownership directive says one agent must own every file under the microservice path and verify contradictions (`feedback_microservice_ownership_coherence_2026_05_20.md:18-59`); this audit followed that scope.
33. The verify-deliverables directive rejects line count as a quality proxy (`feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-31`); this report cites actual file evidence.
34. The docs-substance directive rejects thin scaffolds (`feedback_docs_substance_not_scaffold_2026_05_20.md:10-20`); existing docs have substance but incomplete deployability.
35. ADR-0328 D-20 requires dimension-specific audit application for Batch 2.1 (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3756-4151`).
36. Dimension 6/7/8/9 criteria are each evaluated separately below.
37. Alignment classification for multi-context: drifted-fixable.
38. Alignment classification for OpenTofu: drifted-fixable with P1 missing modules.
39. Alignment classification for OS support: drifted-fixable with P1 missing manifest.
40. Alignment classification for Rust-strict files: aligned on file extensions; drifted-fixable on command examples.
41. Alignment classification for OCI Always Free: drifted-fixable; demo_trial tenant_class must be reconciled.
42. Alignment classification for documentation rigor: incoherent with full-service claim; fixable through doc-set and machine-readable surfaces.
43. Alignment classification for current runtime evidence: narrowed to zone create, not full DNS substrate.
44. Alignment classification for service ambition: aligned with Phase 0 network substrate doctrine.
45. Canonical conflict: existing service docs oversell measured performance and full replacement without the required evidence.
46. Canonical risk: if teams build from docs alone, they may implement query streaming, HSM signing, anycast, and ODoH without contracts or source-of-truth ownership.
47. Required remediation: add `manifest.json` with contexts, OS support, owners, dependencies, and supported tenant_class adoption model.
48. Required remediation: add `iac/<context>/` directories or explicit service-local N/A reasons; for this DNS substrate no context is currently justified N/A.
49. Dimension 4 verdict: target direction is correct; execution evidence is incomplete.
50. Severity summary: P1 for each missing canonical cross-cutting artifact; P2 for command and documentation-style drift.

### §3.5 Dimension 5 — industry-counterpart parity

1. Counterpart-1 AWS Route 53 supports hosted-zone records, reusable delegation sets, multiple routing policies, globally distributed data plane, health checks, Resolver, query logging, DNS Firewall, and domain registration (`https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/route-53-concepts.html`).
2. Route 53 routing policies include simple, failover, geolocation, geoproximity, latency, IP-based, multivalue answer, and weighted (`Route 53 concepts`, lines 159-175).
3. Route 53 public/private DNS data plane runs across more than 200 PoP locations (`Route 53 concepts`, lines 186-196).
4. Route 53 health checks can monitor endpoints, notify, and support DNS failover (`Route 53 concepts`, lines 201-231).
5. Route 53 Resolver DNS Firewall supports rule groups, domain lists, allow/block/alert, DGA, DNS tunneling, dictionary DGA, and fail modes (`https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/resolver-dns-firewall-overview.html`).
6. Route 53 quotas include five API requests per second account-level, one `CreateHealthCheck` request every two seconds, traffic policies, profiles, and private hosted-zone limits (`https://docs.aws.amazon.com/Route53/latest/DeveloperGuide/DNSLimitations.html`).
7. Counterpart-2 Google Cloud DNS supports public zones, private managed zones, IAM, inbound/outbound forwarding, DNSSEC, DNS Armor, DNS64, anycast, logging, response policies, server policies, and routing policies (`https://docs.cloud.google.com/dns/docs/overview`).
8. Google routing policies include weighted round robin, geolocation, failover, external endpoint health checks, and Cloud DNS-managed failover (`https://docs.cloud.google.com/dns/docs/routing-policies-overview`).
9. Google external endpoint health checks use three source regions with three probers each and 30-300 second intervals (`Cloud DNS routing policies`, lines 238-259).
10. Google logging tracks private/public zone queries and exposes log fields such as protocol, queryName, queryType, and location (`https://docs.cloud.google.com/dns/docs/monitoring`).
11. Counterpart-3 Cloudflare DNS supports authoritative DNS, DNSSEC, secondary DNS/zone transfers, CNAME flattening, DNS analytics, DNS Firewall, Load Balancing, public resolver, DoH/DoT, ODoH, and global anycast DNS (`https://developers.cloudflare.com/dns/concepts/`, `https://developers.cloudflare.com/dns/zone-setups/zone-transfers/`, `https://developers.cloudflare.com/1.1.1.1/`).
12. Cloudflare Load Balancing includes failover, active monitoring, intelligent routing, custom rules, and analytics (`https://developers.cloudflare.com/load-balancing/`).
13. Cloudflare 1.1.1.1 supports DoH and DoT, runs in hundreds of cities, and integrates encrypted DNS (`https://developers.cloudflare.com/1.1.1.1/`).
14. Cloudflare ODoH acts as a target and separates proxy from target visibility (`https://developers.cloudflare.com/1.1.1.1/encryption/oblivious-dns-over-https/`).
15. Service-local docs cover hosted zones and record types at tenant_class level (`tenant-class-adoption/...:16-24`, `:34-46`, `:52-64`, `:70-82`).
16. Service-local docs cover DNSSEC at paid tenant_class (`tenant-class-adoption/...:38`, `:56`, `:74`).
17. Service-local docs cover geo, latency, weighted, multi-value, and ML-assisted routing across tenant_classes (`tenant-class-adoption/...:39`, `:57`, `:75`).
18. Service-local docs cover health checks at paid tenant_class (`tenant-class-adoption/...:40`, `:58`, `:76`).
19. Service-local docs cover DoH, DoT, DoH/3, ODoH, and DoQ (`tenant-class-adoption/...:23`, `:41`, `:59`, `:77`).
20. Service-local docs cover query telemetry in FAQ (`faqs/...:82-86`) and tutorial (`tutorials/...:194-213`).
21. Service-local docs cover migration from Route 53 and NS1 (`migration-playbooks/...:1-4`), but not Google Cloud DNS or Cloudflare DNS migration despite top-three counterpart requirement.
22. Service-local docs do not cover DNS Firewall/RPZ/threat filtering comparable to Route 53 DNS Firewall or Cloudflare DNS Firewall.
23. Service-local docs do not cover domain registration comparable to Route 53.
24. Service-local docs do not cover secondary DNS/AXFR/IXFR comparable to Cloudflare zone transfers.
25. Service-local docs do not cover Cloud DNS forwarding/peering/server policies with private VPC detail.
26. Service-local docs do not cover DNS64 comparable to Cloud DNS.
27. Service-local docs do not cover apex alias/CNAME flattening with the precision of Cloudflare/Route 53 alias records, though ALIAS/ANAME appear as record types (`tenant-class-adoption/...:36`, `:72`).
28. Service-local docs do not cover query-log retention, field schema, privacy redaction, or export sinks comparable to Google/AWS logs.
29. Service-local docs do not cover Resolver-style recursive forwarding, conditional forwarding, or endpoint management even though the purpose says recursive DNS (`tenant-class-adoption/...:7-8`).
30. Service-local docs do not cover DNS firewall fail-closed/fail-open policy or malicious domain list management.
31. Additive Oyatie surface includes Cedar-gated per-record authority (`tenant-class-adoption/...:26`, `:44`, `:62`, `:80`), stronger than vendor IAM claims if implemented.
32. Additive Oyatie surface includes BLAKE3/audit-chain anchoring (`benchmarks/...:98-104`), stronger than normal vendor change logs if implemented.
33. Additive Oyatie surface includes air-gap paid DNS (`benchmarks/...:105`) and sovereign compliance packs (`tenant-class-adoption/...:81`).
34. Additive Oyatie surface includes experimental PQC DNSSEC (`tenant-class-adoption/...:74`) but this requires careful non-GA caveating.
35. Additive Oyatie surface includes ODoH and DoQ at paid/paid, matching or exceeding most authoritative DNS vendors but overlapping with Cloudflare resolver surface.
36. Additive Oyatie surface includes per-tenant HSM attestation receipts (`tenant-class-adoption/...:95-96`).
37. Additive Oyatie surface includes RR-Type private tenant codecs (`faqs/...:64-68`), a powerful but risky extensibility model.
38. Parity headline: partial.
39. Present by documentation target: zones, record types, DNSSEC, health checks, geo/latency routing, encrypted transports, telemetry, migration, private zones, reverse DNS, rollback.
40. Missing by documentation target: DNS firewall, response policies/RPZ, secondary DNS, domain registration, full registrar automation, DNS64, conditional forwarding, resolver endpoints, query-log schemas, quotas, self-service signup, and migration playbooks for Google/Cloudflare.
41. Missing by implemented evidence: most of the above, because the proven contract is only zone create.
42. Top counterpart parity risk: union coverage requires both authoritative and resolver/DNS-security surfaces; current docs claim both but implement neither fully.
43. Industry benchmark risk: Cloudflare DNS and 1.1.1.1 surfaces blur authoritative versus recursive; Oyatie must separate authoritative DNS from public resolver in architecture.
44. Route 53 route policies are richer than current docs on IP-based routing/geoproximity; Oyatie mentions city-level and ML routing but lacks formal model.
45. Google Cloud DNS private zone/forwarding/peering model is richer than current private-zone FAQ.
46. Cloudflare secondary DNS and zone-transfer model is absent from Oyatie docs.
47. Cloudflare DNS Firewall cache/proxy model is absent from Oyatie docs.
48. Dimension 5 verdict: product target is competitive but incomplete against union coverage; implemented evidence is far below union coverage.
49. Required remediation: add feature parity matrix and map every gap to PRD/IP slices before implementation.
50. Severity summary: P1 for missing implemented contract surfaces behind claimed parity; P2 for missing docs where target-state feature is clear.

### §3.6 Dimension 6 — multi-context deployment support

1. Context `oyatie-public-cloud`: required for managed GA service; no `iac/oyatie-public-cloud/` directory exists.
2. Context `oyatie-public-cloud`: DNS is explicitly part of the context network seam (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1752-1754`).
3. Context `oyatie-public-cloud`: onboarding must run `tofu init`, `tofu plan`, `tofu apply` through cloud-iac (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1778-1783`); service docs use `make` and manual DS steps.
4. Context `guest-on-aws`: required because source migration and backing AWS primitives are in scope, but no `iac/guest-on-aws/` directory exists.
5. Context `guest-on-aws`: AWS Route 53 may be a backing resource but not the product surface (`specs/master-plan-sequencing.json:715-718`); service docs correctly frame Route 53 as displaced source/backing provider, not user-facing surface.
6. Context `guest-on-oci`: required because OCI Always Free is canonical; no `iac/oci-guest/` directory exists.
7. Context `guest-on-oci`: no `iac/oci-guest/always-free/` directory exists, violating `specs/master-plan-sequencing.json:857-867`.
8. Context `guest-on-oci`: demo_trial tenant_class does not state Always Free, violating `feedback_oci_always_free_maximization_2026_05_20.md:74-82`.
9. Context `on-prem`: required for bare-metal/customer data centers; no `iac/on-prem/` exists.
10. Context `on-prem`: FAQ says sovereign and air-gapped DNS are reasons for custom DNS (`faqs/...:15-22`), so on-prem cannot be N/A without contradiction.
11. Context `colo`: required for colocated hardware; no `iac/colo/` exists.
12. Context `colo`: BGP anycast and dedicated ranges imply colo/facility modeling, but no facility/seam docs exist (`tenant-class-adoption/...:71-78`, `faqs/...:26-30`).
13. Context `oyatie-as-cloud-provider`: required because this is a cloud-* IaaS/network service; no `iac/oyatie-iaas/` exists.
14. Context `oyatie-as-cloud-provider`: master plan says cloud-* microservices are the IaaS surface, not wrappers (`specs/master-plan-sequencing.json:739-742`).
15. Correctly N/A contexts: none found; all six are in scope unless Wave 14 narrows first deliverable.
16. Missing manifest means no explicit context support list exists.
17. Missing context matrix means no per-context SLO variation exists.
18. Missing context matrix means no per-context tenant onboarding flow exists.
19. Missing context matrix means no per-context billing/cost event shape exists.
20. Missing context matrix means no per-context observability export exists.
21. Missing context matrix means no per-context IAM/Cedar binding model exists.
22. Missing context matrix means no per-context network/BGP authority model exists.
23. Missing context matrix means no per-context registrar/DS publication boundary exists.
24. Missing context matrix means no per-context HSM/KMS support declaration exists.
25. Missing context matrix means no per-context DNSSEC key storage declaration exists.
26. Missing context matrix means no per-context resolver endpoint/anycast plan exists.
27. Missing context matrix means no per-context state backend exists.
28. Missing context matrix means no per-context supply-chain attestation exists.
29. Missing context matrix means no per-context CI lane exists.
30. Forbidden direct cloud-vendor API in business logic: no service-local source exists to inspect; docs use AWS CLI for migration source export only (`migration-playbooks/...:10-28`).
31. AWS CLI in migration playbook should be scoped to source extraction, not runtime logic.
32. NS1 CLI in migration playbook should be scoped to source extraction, not runtime logic (`migration-playbooks/...:31-44`).
33. Manual registrar edits are currently present (`migration-playbooks/...:82-87`, `onboarding/...:96`, `tutorials/...:80`).
34. Manual registrar edits conflict with zero-handroll onboarding for production.
35. The docs need a registrar provider boundary, not manual production work.
36. The docs need all six context rows in PRD/ARCH and a machine-readable manifest.
37. The docs need context-specific deployment diagrams for public cloud, AWS guest, OCI guest, on-prem, colo, and provider mode.
38. The docs need context-specific failure modes: cloud provider DNS outage, local resolver partition, on-prem isolated mode, colo route leak, OCI Always Free capacity breach, public-cloud cell failover.
39. The docs need context-specific IAM: Oyatie identity+Cedar in all contexts with provider identities hidden behind adapters.
40. The docs need context-specific billing: chargeback in self-hosted contexts and standard meters in managed contexts.
41. The docs need context-specific network seams: VPC/VCN, bare metal VLAN/VRF, colo BGP, and provider-mode tenant VPC.
42. The docs need context-specific OpenTofu state backend matching master plan lines 758-765.
43. The docs need context-specific package/OS support, especially Oracle Linux/Ampere for OCI.
44. Current support classification per context: supported by ambition, not by deployable artifact.
45. P1 finding: claiming universal DNS replacement without context IaC is not coherent under ADR-0328 D-15.
46. P1 finding: OCI Always Free profile absent for demo_trial undermines cost-zero baseline.
47. P2 finding: migration docs include source-provider commands but do not explicitly isolate them from product runtime.
48. Dimension 6 verdict: all six contexts should remain in scope; all six lack service-local deployment evidence.
49. Remediation: add six context subsections to PRD/ARCH, six `iac/<context>/` directories, and OCI Always Free subprofile.
50. Stop condition: no public-cloud or provider-mode readiness claim until all six context rows have IaC, CI lane, and state backend evidence.

### §3.7 Dimension 7 — OpenTofu IaC coverage

1. Service-local `iac/` directory is absent.
2. `iac/oyatie-public-cloud/` is absent.
3. `iac/guest-on-aws/` is absent.
4. `iac/oci-guest/` is absent.
5. `iac/oci-guest/always-free/` is absent.
6. `iac/on-prem/` is absent.
7. `iac/colo/` is absent.
8. `iac/oyatie-iaas/` is absent.
9. ADR-0328 requires a deployable microservice to own `microservices/<name>/iac/` (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2275-2279`).
10. ADR-0328 requires `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and README per context (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2296-2309`).
11. Required variables include tenant_id, deployment_context, cell_id, region/facility_id, tenant_class, billing account, and data residency policy (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2323-2337`).
12. Required outputs include service endpoint, observability export, billing meter IDs, IAM bindings, state backend ref, and module attestation ref (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2339-2355`).
13. Module signing via sigstore/cosign is mandatory (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2357-2365`).
14. Service-local grep found no Terraform engine references.
15. Service-local grep found no Pulumi references.
16. Service-local grep found no CloudFormation references.
17. Service-local grep found no `null_resource` references.
18. Service-local grep found no `local-exec` references.
19. Service-local grep found no `remote-exec` references.
20. Service-local grep found no SSH provisioner references.
21. Service-local grep found no hand-edited `tfstate` references.
22. Service-local grep found no unsigned module references.
23. Absence of forbidden patterns is not enough; absence of OpenTofu modules is the blocking issue.
24. State backend for `guest-on-aws` should be S3+DynamoDB lock (`specs/master-plan-sequencing.json:758-760`); no service-local backend exists.
25. State backend for `guest-on-oci` should be OCI Object Storage + Autonomous DB lock (`specs/master-plan-sequencing.json:759-761`); absent.
26. State backend for `on-prem` should be MinIO+lock-table (`specs/master-plan-sequencing.json:761-762`); absent.
27. State backend for `oyatie-public-cloud` should be internal OCI (`specs/master-plan-sequencing.json:762-763`); absent.
28. State backend for `colo` should be MinIO+lock-table (`specs/master-plan-sequencing.json:763-764`); absent.
29. State backend for provider mode should be internal `cloud-storage` (`specs/master-plan-sequencing.json:764-765`); absent.
30. The service docs do not use `tofu init`, `tofu plan`, or `tofu apply`.
31. The service docs use `make` for dev setup and benchmark reproducibility (`onboarding/...:22-24`, `tutorials/...:6-8`, `benchmarks/...:112-117`).
32. The service docs use CLI migration loops (`migration-playbooks/...:16-28`, `:33-38`).
33. OpenTofu should own DNS cell resources, resolver deployments, anycast/BGP components, HSM/key bindings, health-check worker infrastructure, telemetry sinks, and context-specific network attachments.
34. OpenTofu should not own business logic such as record validation or DNSSEC signing algorithms.
35. `cloud-iac` should orchestrate modules; FAQ mentions `cloud-iac` declarative record creation (`faqs/...:141-145`) but no module contract exists.
36. ADR-0273 expects an iac/helm DNS orchestrator path (`docs/adr-archive/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md:1330-1332`); this conflicts with missing service-local IaC.
37. Helm path expectation also conflicts with OpenTofu-first unless Helm values are generated/managed by OpenTofu through approved modules.
38. Required IaC inventory after remediation should include DNS authoritative pods/daemonsets, CoreDNS/Knot/PowerDNS choice, Cilium policies, anycast advertisement resources, health monitor deployment, HSM/SoftHSM bindings, metrics/logs, and per-context networking.
39. Required OCI Always Free module must fit 4 OCPU/24GB, 200GB block, 10GB object/archive, and 10Mbps LB (`feedback_oci_always_free_maximization_2026_05_20.md:67-84`).
40. Required AWS guest module must use AWS primitives as backing resources only, not expose Route 53 as tenant product (`feedback_multi_context_provider_agnostic_2026_05_20.md:16-23`).
41. Required on-prem module must avoid console/SSH handroll and use declarative providers or custom OpenTofu provider plugins.
42. Required colo module must model facility/BGP/remote-hands assumptions declaratively.
43. Required provider-mode module must consume internal cloud-* resources through Oyatie provider plugins.
44. No IaC tests exist under service path.
45. No provider lock files exist under service path.
46. No module attestation or SBOM ref exists under service path.
47. No cost estimation or cloud-billing meter IDs exist under service path.
48. Dimension 7 verdict: hard P1 miss, not merely a documentation gap.
49. Remediation priority: create six context module skeletons with real variables/outputs and a first Always Free subprofile before claiming any deployable context.
50. Stop condition: no "all six contexts supported" wording until `tofu plan` can run per context through `cloud-iac`.

### §3.8 Dimension 8 — OS support matrix

1. No `supported-oses.json` exists in the service path.
2. No `manifest.json` or `manifest.yaml` with `supported_oses` exists in the service path.
3. Master plan requires per-microservice manifest (`specs/master-plan-sequencing.json:813-815`).
4. Tier-1 OS `talos`: no declaration, no package format, no CI lane.
5. Tier-1 OS `rhel-9.x+`: no declaration, no RPM, no CI lane.
6. Tier-1 OS `oracle-linux-9.x+`: no declaration, no RPM, no CI lane.
7. Tier-1 OS `sles-15-sp6+`: no declaration, no RPM, no CI lane.
8. Tier-1 OS `ubuntu-24.04-lts+`: no declaration, no DEB, no CI lane.
9. Tier-1 OS `debian-13+`: no declaration, no DEB, no CI lane.
10. Tier-1 OS `rocky-9.x+`: no declaration, no RPM, no CI lane.
11. Tier-1 OS `almalinux-9.x+`: no declaration, no RPM, no CI lane.
12. Tier-1 OS `centos-stream-10+`: no declaration, no RPM, no CI lane.
13. Tier-1 OS `amazon-linux-2023+`: no declaration, no RPM, no CI lane.
14. Tier-1 OS `flatcar`: no declaration, no container/system extension, no CI lane.
15. Tier-1 OS `photon-5.x+`: no declaration, no package/container plan, no CI lane.
16. Tier-1 OS `macos-apple-silicon-m5+`: no declaration, no `.pkg`, no Homebrew formula, no CI lane.
17. Tier-2 `linux-ppc64le`: no test-only declaration.
18. Tier-2 `linux-s390x`: no test-only declaration.
19. Out-of-scope `macos-intel`: no explicit unsupported statement.
20. Out-of-scope `macos-apple-silicon-pre-m5`: no explicit unsupported statement.
21. Out-of-scope `freebsd`: no explicit unsupported statement.
22. Out-of-scope `openbsd`: no explicit unsupported statement.
23. Out-of-scope `windows-server`: no explicit unsupported statement.
24. Out-of-scope `solaris`: no explicit unsupported statement.
25. OS memory requires every service manifest enumerate OS support and gotchas (`feedback_os_support_matrix_2026_05_20.md:56-78`).
26. OCI Always Free memory requires Oracle Linux/Ampere support for OCI Always Free (`feedback_oci_always_free_maximization_2026_05_20.md:84`).
27. Service docs do not mention Oracle Linux or Ampere A1.
28. Service docs do not mention SELinux policies for RHEL/Oracle/Rocky/Alma/CentOS/Amazon Linux.
29. Service docs do not mention AppArmor profile for Ubuntu/SUSE.
30. Service docs do not mention Talos no-shell constraints.
31. Service docs do not mention Flatcar/Photon immutable-host deployment.
32. Service docs do not mention macOS M5+ development or Apple Silicon server constraints.
33. Service docs do not mention `linux/amd64` or `linux/arm64`.
34. Service docs do not mention `darwin/arm64-m5+`.
35. Service docs do not mention ppc64le/s390x test-only.
36. Reference implementation uses Rust and should be portable in principle (`reference-implementations/...:24-199`), but portability is not a support claim.
37. `dig`, `curl --http3`, and `kdig` dependency availability varies by OS (`tutorials/...:6-9`), but no OS-specific package instructions are given.
38. HSM dependencies vary by OS and hardware, but no per-OS HSM support matrix exists (`faqs/...:33-37`).
39. BGP/FRR and eBPF dependencies vary by OS, but no per-OS network support matrix exists (`tenant-class-adoption/...:94`).
40. The service likely needs container-image deployment on Talos/Flatcar/Photon, RPM/DEB for enterprise Linux, and `.pkg`/Homebrew for macOS developer tooling.
41. The service likely needs kernel/network capability declarations for raw UDP/TCP 53, QUIC, BGP control, and eBPF/cell routing.
42. The service likely needs SoftHSM or HSM simulator support for non-paid tests.
43. The service likely needs cross-architecture DNS packet parsing tests because endian and alignment bugs are common in network code.
44. The service likely needs integration tests for UDP/TCP fragmentation, DNSSEC packet size, EDNS0, and QUIC on both amd64 and arm64.
45. Current OS support classification: missing.
46. Severity: P1 because canonical OS manifest is required across microservices.
47. Remediation: add manifest with all Tier-1, Tier-2, out-of-scope, package formats, and CI lanes.
48. Remediation: add OS-specific gotchas to onboarding and tutorials after manifest lands.
49. Dimension 8 verdict: no OS support evidence exists in the service path.
50. Stop condition: no cross-OS support claim until manifest and CI lane evidence exist.

### §3.9 Dimension 9 — Rust-strict language coverage

1. Service-local file scan found no `*.py` files.
2. Service-local file scan found no `*.js` files.
3. Service-local file scan found no `*.ts` or `*.tsx` files.
4. Service-local file scan found no `*.rb` files.
5. Service-local file scan found no `*.go` files.
6. Service-local file scan found no `*.java` files.
7. Service-local file scan found no `*.scala` files.
8. Service-local file scan found no `*.groovy` files.
9. Service-local file scan found no `*.php` files.
10. Service-local file scan found no `*.fs` or F# files.
11. Service-local authorized non-Rust files are Markdown docs only; `.md` is whitelisted (`specs/master-plan-sequencing.json:828-839`).
12. No generated SDK outputs exist under service path.
13. No frontend code exists under `frontend/ios`, `frontend/macos`, `frontend/android`, `frontend/windows`, or `frontend/web`.
14. Therefore no Swift/Kotlin/WinUI3 scoping violation exists.
15. No backend C#/.NET exists.
16. No JavaScript application logic exists.
17. No TypeScript application logic exists.
18. No Python validation/migration/deployment helpers exist.
19. No Go helper exists.
20. No Ruby/Perl/PHP helper exists.
21. No Java/Scala/Groovy backend exists.
22. Rust-strict file inventory is aligned.
23. Rust-strict command examples are not fully aligned.
24. Onboarding uses `make dev-cell.up` and `make dev-tenant.create` (`onboarding/...:22-24`).
25. Tutorial uses `make dev-cell.up` and `make dev-tenant.create` (`tutorials/...:6-8`).
26. Benchmark reproducibility uses `make benchmarks.cloud-network-dns.run` (`benchmarks/...:112-117`).
27. ADR-0328 forbids `make` as backend build invocation (`docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3230-3231`).
28. Reference implementation uses Cargo for example execution (`reference-implementations/...:201-205`).
29. Reference implementation uses Cargo for hermetic tests (`reference-implementations/...:229-233`).
30. Canonical release build is absent from service docs: `cargo build --workspace --release --all-features --locked` is not included.
31. Migration playbook uses shell loops (`migration-playbooks/...:16-28`, `:33-38`).
32. Tutorial uses shell loop for three health checks (`tutorials/...:106-120`).
33. Rust-strict memory forbids Bash beyond tiny glue and migration scripts (`feedback_rust_strict_only_no_python_2026_05_20.md:51-64`).
34. Shell examples should be converted to `./bin/oya` single commands or Rust-coded migration subcommands with explicit invocations.
35. If shell remains as illustrative operator commands, docs should classify them as source-system export examples and not durable Oyatie automation.
36. Reference implementation uses Rust 2024 edition and Rust dependencies (`reference-implementations/...:6-22`); this is aligned.
37. Reference implementation imports `oya_cloud_network_dns_sdk` and `oya_trace` (`reference-implementations/...:26-34`); aligned if crates exist and are Rust.
38. Service-local absence of `Cargo.toml` means no direct release build can be run from the service path.
39. Existing external runtime crate is Rust (`network/ports/dns/src/lib.rs:7-15`).
40. Existing external tests are Rust (`network/ports/dns/tests/cloud_network_dns_api.rs:146-211`).
41. No non-Rust generated SDK output exists to classify as authorized.
42. No OpenTofu `.tf` files exist to classify.
43. No Cedar `.cedar` files exist to classify.
44. No YAML/JSON/proto/OpenAPI/OpenSLO service-local files exist to classify.
45. Missing machine-readable files are a substance gap, not a language violation.
46. Build invocation per ADR-0328 should be documented in README/ARCH after those files exist.
47. P1 if shell loops are treated as migration automation; P2 if they are examples only.
48. P2 for `make` in onboarding/tutorial/benchmark docs because current wording presents them as prerequisites/reproducibility commands.
49. Dimension 9 verdict: file-level Rust-strict passes; command-level Rust-strict needs repair.
50. Stop condition: no release-readiness claim until docs use Cargo/Rust CLI/OpenTofu invocations only.

## §4 Findings summary

| Severity | Dimension | Finding | Citation | Remediation hint |
|---|---|---|---|---|
| P1 | D1/D3 | PRD missing despite service claiming broad DNS substrate ownership | `docs/standards/documentation-rigor.md:64-66`; missing `network/dns/PRD.md` | Add PRD with bounded first deliverable and target-state sections |
| P1 | D1/D3 | ARCHITECTURE missing despite anycast/DNSSEC/HSM/ODoH claims | `docs/standards/documentation-rigor.md:64-66`; `retired tenant_class adoption artifact:7-10` | Add architecture with control/data/resolver boundaries |
| P1 | D1/D2 | Service-local docs claim full DNS product, but proven contract is zone create only | `network/dns/contracts/openapi/cloud/cloud-network-dns-v1.yaml:1-12`; `network/ports/dns/src/lib.rs:1-5` | Add contract roadmap or reduce claims to target-state |
| P1 | D1/D3 | Cedar permits referenced but no service-local policy directory exists | `retired tenant_class adoption artifact:26,44,62,80`; `faqs/dns-engineer-faq.md:82-93` | Add Cedar fragments and permit registry |
| P1 | D1/D3 | Audit/event/query stream claims lack AsyncAPI/event schemas | `retired tenant_class adoption artifact:90-91`; `faqs/dns-engineer-faq.md:82-86` | Add AsyncAPI and event schema entries |
| P1 | D1 | HSM/KSK import tenant_class statement is internally contradictory | `migration-playbooks/from-route53-and-ns1.md:88-98`; `retired tenant_class adoption artifact:74-80` | Rewrite import policy by tier: software key vs HSM |
| P1 | D1 | Benchmark claims measured evidence, but cited evidence path is absent | `benchmarks/cloud-network-dns-vs-route53-vs-cloud-dns-vs-cloudflare-vs-ns1.md:3-5,119` | Mark as target numbers or land evidence bundle |
| P1 | D2 | ADR-0273 expects DNS orchestrator IaC path that does not exist | `docs/decisions/ADR-0700-ci-admission-live-apex.md:1330-1332` | Add IaC path or update ADR via owning workflow |
| P1 | D3 | Full doc-set floor is not met: 7 artifacts versus >=70 floor | `docs/standards/documentation-rigor.md:62-81` | Build PR-143-style suite for this service |
| P1 | D4/D6 | No six-context support matrix exists | `specs/master-plan-sequencing.json:704-745`; `feedback_multi_context_provider_agnostic_2026_05_20.md:32-38` | Add manifest/PRD context matrix |
| P1 | D4/D7 | No OpenTofu `iac/` directory exists | `docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2275-2295` | Add per-context OpenTofu modules |
| P1 | D4/D8 | No OS support manifest exists | `specs/master-plan-sequencing.json:777-815`; `feedback_os_support_matrix_2026_05_20.md:56-78` | Add supported OS manifest and CI lanes |
| P1 | D4/D6 | OCI demo_trial tenant_class Always Free not documented; demo_trial says ~$5/month | `retired tenant_class adoption artifact:28`; `feedback_oci_always_free_maximization_2026_05_20.md:74-82` | Add OCI demo_trial tenant_class Always Free reconciliation |
| P1 | D6 | All six contexts lack IaC evidence | `specs/master-plan-sequencing.json:709-742` | Create `iac/oyatie-public-cloud`, `guest-on-aws`, `oci-guest`, `on-prem`, `colo`, `oyatie-iaas` |
| P1 | D7 | No sigstore/cosign module signing wiring | `docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2357-2365` | Add module attestation output and release flow |
| P1 | D7 | No state backend per context | `specs/master-plan-sequencing.json:758-765` | Add backend declaration per module README |
| P1 | D5 | DNS Firewall/RPZ/threat-filtering missing versus AWS/Cloudflare union coverage | AWS DNS Firewall docs; Cloudflare DNS Firewall docs | Add DNS security/filtering product slice |
| P1 | D5 | Secondary DNS/AXFR/IXFR missing versus Cloudflare union coverage | Cloudflare zone transfer docs | Add secondary DNS and TSIG/ACL design |
| P1 | D5 | Resolver forwarding/peering missing versus Google/AWS union coverage | Google Cloud DNS overview and policies docs | Add resolver endpoint and forwarding contracts |
| P2 | D1 | Broken ADR-0253 path in onboarding | `onboarding/dns-engineer-first-week.md:9-11` | Replace with exact existing ADR path |
| P2 | D1/D9 | `make` used in onboarding/tutorial/benchmark docs | `onboarding/...:22-24`; `tutorials/...:6-8`; `benchmarks/...:112-117` | Replace with Cargo/Rust CLI/OpenTofu |
| P2 | D1/D9 | Shell loops used for migration and health-check creation | `migration-playbooks/...:16-28`; `tutorials/...:106-120` | Use Rust migrator subcommands or explicit commands |
| P2 | D1/D6 | Manual DS publication contradicts zero-handroll production onboarding | `onboarding/...:96`; `tutorials/...:80`; `docs/adr-archive/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1782-1783` | Add registrar integration boundary |
| P2 | D1 | SDK crate/version in reference implementation not proven service-local | `reference-implementations/...:14-22` | Add SDK manifest or update crate name/version |
| P2 | D1 | HSM products named without deployment/support matrix | `faqs/dns-engineer-faq.md:33-37` | Add HSM support doc and cloud-kms handoff |
| P2 | D1 | MaxMind/RIPE data source lacks licensing and update runbook | `faqs/dns-engineer-faq.md:50-53` | Add data-source refresh and license controls |
| P2 | D1 | Custom RR codec model lacks sandbox/ABI governance | `faqs/dns-engineer-faq.md:64-68` | Add plugin ABI and tenant isolation design |
| P2 | D3 | Capacity numbers lack derivation | `retired tenant_class adoption artifact:24-25,42-43,60-61,78-79` | Add capacity model math |
| P2 | D3 | Failure-mode tree absent | `docs/standards/documentation-rigor.md:143-156`; service docs lack `failure-modes.md` | Add failure-mode document |
| P2 | D3 | Query log example lacks schema/export/retention | `tutorials/...:194-213` | Add observability contract and retention doc |
| P2 | D5 | Google/Cloudflare migration playbooks absent | Current service has only `migration-playbooks/from-route53-and-ns1.md` | Add top-three counterpart playbooks |
| P2 | D8 | OS package formats absent | `feedback_os_support_matrix_2026_05_20.md:37-44` | Add RPM/DEB/container/.pkg/Homebrew plan |
| P3 | D2 | References to ADRs are mostly numeric, not exact links | `retired tenant_class adoption artifact:3-5` | Convert to exact relative links |
| P3 | D5 | Additive PQC DNSSEC lacks experimental safety caveat | `retired tenant_class adoption artifact:74` | Mark experimental, gated, not GA-default |
| P3 | D7 | Helm expectation in ADR-0273 needs OpenTofu-managed framing | `docs/adr-archive/ADR-0273-per-tenant-dkim-spf-dmarc-email-deliverability.md:1330-1332` | Document Helm as generated deployment artifact if retained |

Severity totals: P0 = 0, P1 = 19, P2 = 12, P3 = 3.

## §5 Open questions for Wave 14 aggregation

1. Should the first implementation deliverable stay bounded to `cloud.network.dns.zone.create`, or must record CRUD and DNSSEC enablement move into the first build slice because mail DKIM publication depends on TXT records?
2. Does `cloud-network-dns` own recursive resolver features, or should recursive/forwarding/DNS Firewall be split into a separate resolver/security bounded context inside the same microservice?
3. Should registrar/DS publication be automated by this service, by `cloud-iac`, or by a dedicated registrar integration surface?
4. Is `microservices/cloud-network-dns/iac/helm/dns-orchestrator/` from ADR-0273 still valid, or should it become OpenTofu-managed Kubernetes resources under the six canonical context modules?
5. Which DNS engine is canonical for authoritative serving: Knot, CoreDNS, PowerDNS, trust-dns/hickory, or an Oyatie Rust-native engine?
6. Does ODoH belong to tenant authoritative DNS, public resolver, or privacy gateway product surface?
7. Are custom RR codecs allowed for tenant-supplied code, and if yes what sandbox/ABI/approval policy governs them?
8. Should PQC DNSSEC remain research/paid experimental until IETF standardization, or is it a regulated-tenant preview feature?
9. Should the benchmark document be retired/reframed as target benchmarks because the cited measured-evidence path is absent?
10. Which cross-microservice handoff owns query telemetry: `observability`, `audit-chain`, or `cloud-network-dns` emitting to both?
11. How should OCI Always Free demo_trial handle anycast and health checks under 10 Mbps LB and 4 OCPU/24GB resource limits?
12. Which Wave 14 owner resolves the discrepancy between service-local broad target docs and existing narrow OpenAPI/runtime surface?

## Verification notes

- Inventory command used read-only file listing under `microservices/cloud-network-dns/`; all seven files were line-counted and sampled/read fully.
- Forbidden source-language scan found no service-local forbidden source files.
- OpenTofu/forbidden-pattern scan found no Terraform, Pulumi, CloudFormation, `null_resource`, `local-exec`, SSH provisioner, or tfstate references in the service path.
- Existing service-local docs are substantive but incomplete; no padding-only scaffold finding was assigned.
- This audit did not modify existing service files; it only adds audit deliverables in the target microservice directory.

<!-- ORCHESTRATOR REPORT
  µservice: cloud-network-dns
  deliverables_landed:
    - /Users/jasonlee/oyatie/network/dns/coherence-audit-2026-05-20.md (635 lines)
    - /Users/jasonlee/oyatie/network/dns/feature-parity-matrix-2026-05-20.md (400 lines)
    - /Users/jasonlee/oyatie/network/dns/performance-benchmark-numbers-2026-05-20.md (326 lines)
    - /Users/jasonlee/oyatie/microservices/cloud-network-dns/capability-tenant_class-deltas-vs-counterparts-2026-05-20.md (484 lines)
  inventory_files_seen: 7
  inventory_lines_read: 1219
  chat_history_matches_processed: 7
  findings_p0: 0
  findings_p1: 19
  findings_p2: 12
  findings_p3: 3
  top_3_counterparts_confirmed: AWS Route 53 / Google Cloud DNS / Cloudflare DNS
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1845
-->
