# Compliance performance benchmark numbers

Audit date: 2026-05-20.
Target µservice: `microservices/compliance/`.
Methodology disclosure: these are TARGET numbers plus public counterpart provenance, not measured service benchmarks.
Measurement status: no Rust implementation, test harness, or production telemetry exists under `microservices/compliance/`, so no Oyatie number in this report is measured.
Build-phase requirement: measured benchmarks must be added during implementation per ADR-0212 and must disclose OS, architecture, deployment context, tenant class, workload shape, and build invocation.
Safety rule: do not present these targets as current service capability.

## Five-citation anchor block

1. Canonical direction: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` §D-15..§D-20, especially §D-20.152 for benchmark OS/arch/context/tenant disclosure and §D-20.153 for OCI demo_trial.
2. Machine-readable control surface: `specs/master-plan-sequencing.json` lines 704-868, especially deployment contexts, `oci_always_free`, and language build invocation.
3. µservice PRD: `microservices/compliance/PRD.md` lines 48-59 for success metrics and lines 92-98 for SLOs.
4. µservice tier matrix: `microservices/compliance/capability-tiers/tier-matrix.md` lines 15-146 for customer-class ladder capacity and SLO posture.
5. Documentation-rigor anchor: `docs/standards/documentation-rigor.md` lines 133-173 for intern-buildable rigor and operational evidence expectations.

## Public benchmark/provenance anchors

Vanta public claims used as non-latency counterpart numbers: Vanta Trust Center page lines 174-186 reports 93 percent access approval automation and 86 percent NDA collection automation; Vanta TPRM public page reports 62 percent faster vendor evidence collection and 54 percent productivity gains; Vanta risk page reports up to 45 percent faster remediation; Vanta TPRM public page cites 50 percent vendor assessment time reduction.
Drata public claims used as non-latency counterpart numbers: Drata homepage search result reported 7,980 fewer audit-prep hours annually for the average enterprise, 200+ annual hours saved by AI questionnaire automation, $20M annual revenue accelerated with Trust Center, 75 percent SOC 2 audit duration reduction in one customer story, 10x faster trust documentation turnaround, 375+ hours saved per year through questionnaire automation, 30+ frameworks, 8,000+ customers, and 4.8/5 G2 review signal on risk page.
OneTrust public claims used as non-latency counterpart numbers: OneTrust DSR page reports DSR cost-to-fulfill reduction up to 99 percent; OneTrust Privacy Operations page search result reports 227 percent ROI over three years and 7-month payback; OneTrust third-party pages report up to 70 percent faster third-party assessments, 20M+ cyber risk and attack insights, 9.2M critical event workflows per year, more than 50 built-in control frameworks, thousands of vendor Trust Profiles, and 300+ jurisdictions in DataGuidance references.
Counterpart latency disclosure: Vanta, Drata, and OneTrust public product pages reviewed do not publish API p50/p95/p99, RPS, pack-rule throughput, or DSAR pipeline throughput comparable to an implementation benchmark.
Interpretation rule: counterpart numbers are efficiency/productivity/public-scale claims, not apples-to-apples service latency measurements.

## §1 Methodology

M01 Benchmark dimensions: request latency p50, p95, p99.
M02 Benchmark dimensions: sustained throughput in requests per second.
M03 Benchmark dimensions: evidence emission throughput.
M04 Benchmark dimensions: pack-rule evaluation throughput.
M05 Benchmark dimensions: effective-policy compute latency.
M06 Benchmark dimensions: DSAR concurrent active requests per tenant.
M07 Benchmark dimensions: regulator export bundle generation latency.
M08 Benchmark dimensions: audit-chain seal verification latency.
M09 Benchmark dimensions: breach-clock notification enqueue latency.
M10 Benchmark dimensions: third-party/vendor assessment throughput for future parity.
M11 Benchmark dimensions: questionnaire answer retrieval latency for future parity.
M12 Benchmark dimensions: trust-center gated document approval latency for future parity.
M13 Workload A, evidence write: authenticated tenant admin records evidence artifact with tenant, framework, control, artifact kind, and audit-chain seal reference.
M14 Workload B, evidence read: auditor reads artifact metadata and verifies seal.
M15 Workload C, DSAR open: subject or privacy officer opens export/delete/rectify request.
M16 Workload D, DSAR status: request status and SLA counters read.
M17 Workload E, pack publish: compliance admin publishes a versioned pack and triggers soak.
M18 Workload F, effective policy: tenant pack-set evaluated for effective policy projection.
M19 Workload G, conflict report: multi-pack conflict generated after activation.
M20 Workload H, regulator export: signed evidence bundle generated for a pack/time window.
M21 Workload I, trust center: buyer requests gated document access and receives policy decision.
M22 Workload J, questionnaire: buyer questionnaire answer sourced from approved evidence and policy knowledge base.
M23 Workload K, vendor risk: vendor evidence ingested and assessed against rubric.
M24 OS disclosure required for future measured runs: one row per Tier-1 OS from ADR-0328 §D-17.
M25 Architecture disclosure required for future measured runs: x86_64 and aarch64 at minimum.
M26 Deployment disclosure required for future measured runs: one of six contexts.
M27 Tenant class disclosure required for future measured runs: internal, SMB, enterprise, regulated, sovereign, or hyperscaler-scale.
M28 Build disclosure required for future measured runs: `cargo build --workspace --release --all-features --locked`.
M29 Current target numbers assume Rust backend, PostgreSQL/Citus, Kafka, Valkey, ClickHouse, SeaweedFS, OpenBao, and audit-chain client as described in tier docs.
M30 Current target numbers assume HTTP/3 client edge, HTTP/2 service mesh, TLS 1.3, and Cedar pre-storage authorization as described in architecture lines 54-59 and 108-113.
M31 Current target numbers assume no direct AWS/OCI business-logic SDK calls.
M32 Current target numbers assume OpenTofu context modules land before measured deploy benchmarks.
M33 Current target numbers use p95 as primary SLO because PRD/tier matrix names p95/p99 targets.
M34 Current target numbers use p99 for audit/regulator workflows because tail latency controls audit readiness.
M35 Current target numbers separate demo_trial OCI Always Free from general demo_trial because ADR-0328 §D-19 makes that a special subprofile.

## §2 Counterpart numbers

VANTA-01 Public API latency: not published in reviewed public pages; use as comparison gap, not as zero.
VANTA-02 Public throughput: not published in reviewed public pages.
VANTA-03 Trust Center access approval automation: 93 percent public claim.
VANTA-04 Trust Center NDA collection automation: 86 percent public claim.
VANTA-05 Vendor evidence collection acceleration: 62 percent faster public claim.
VANTA-06 TPRM productivity gain: 54 percent public claim.
VANTA-07 Risk remediation acceleration: up to 45 percent faster public claim.
VANTA-08 Vendor security assessment time reduction: up to 50 percent public/customer claim.
VANTA-09 Continuous compliance monitoring cadence: real-time/living posture described publicly.
VANTA-10 Cloud metadata access model: lightweight read-only metadata/configuration checks described publicly.
VANTA-11 Access-review import: screenshots/PDF/CSV import path described publicly.
VANTA-12 Questionnaire automation: AI-powered answering and workflow automation described publicly.
VANTA-13 Benchmark implication: Oyatie needs trust-center approval automation targets, vendor evidence ingestion targets, risk remediation workflow targets, and questionnaire response targets to compare directly.

DRATA-01 Public API latency: not published in reviewed public pages.
DRATA-02 Public throughput: not published in reviewed public pages.
DRATA-03 Audit-prep savings: 7,980 fewer hours annually for average enterprise public claim.
DRATA-04 AI questionnaire savings: 200+ annual hours saved for average enterprise public claim.
DRATA-05 Trust Center revenue acceleration: $20M annual revenue accelerated public claim.
DRATA-06 SOC 2 audit duration reduction: 75 percent customer-story claim.
DRATA-07 Trust documentation turnaround: 10x faster public claim.
DRATA-08 Questionnaire automation savings: 375+ hours/year public claim.
DRATA-09 Framework count: 30+ standard frameworks public claim.
DRATA-10 Customer count: 8,000+ global customers public claim on risk page.
DRATA-11 G2 review signal: 4.8/5 public claim on risk page.
DRATA-12 Third-party risk: AI criteria, vendor sync, summaries, risk register, directory, and executive reports publicly described.
DRATA-13 Benchmark implication: Oyatie needs hours-saved proxy targets and workflow throughput, not just API latency.

ONETRUST-01 Public API latency: not published in reviewed public pages.
ONETRUST-02 Public throughput: not published in reviewed public pages.
ONETRUST-03 DSR cost-to-fulfill reduction: up to 99 percent public claim.
ONETRUST-04 Privacy automation ROI: 227 percent over three years public search-result claim.
ONETRUST-05 Privacy automation payback: 7 months public search-result claim.
ONETRUST-06 Third-party assessment acceleration: up to 70 percent public claim.
ONETRUST-07 Cyber risk and attack insights: 20M+ public claim.
ONETRUST-08 Critical event workflows: 9.2M per year public claim.
ONETRUST-09 Control frameworks: more than 50 built-in public claim.
ONETRUST-10 Vendor Trust Profiles: thousands public claim.
ONETRUST-11 Regulatory jurisdictions: 300+ jurisdictions referenced in DataGuidance material.
ONETRUST-12 AI risk work increase: OneTrust AI page references 37 percent more time managing AI-related risk year over year.
ONETRUST-13 Benchmark implication: Oyatie needs DSR unit-cost target, assessment acceleration target, regulatory update ingestion target, and high-cardinality third-party event targets.

## §3 Oyatie target numbers by tier and deployment context

### demo_trial — oyatie-public-cloud

B-OPC-01 API p50 target: 45 ms for evidence read/status workloads.
B-OPC-02 API p95 target: 150 ms for evidence read/status workloads.
B-OPC-03 API p99 target: 350 ms for evidence read/status workloads.
B-OPC-04 Evidence write throughput target: 250 writes/sec per cell.
B-OPC-05 Effective-policy compute p95 target: 100 ms, matching tier matrix line 50.
B-OPC-06 Pack-projection staleness p95 target: 60 seconds, matching tier matrix line 51.
B-OPC-07 Conflict report generation target: <= 5 minutes, matching tier matrix line 52.
B-OPC-08 DSAR concurrent active requests: 10 per tenant, matching tier matrix line 46.
B-OPC-09 Regulator export typical 90-day bundle p99 target: 4 hours.
B-OPC-10 Active tenant ceiling target: 50 tenants per cell, matching tier matrix line 44.

### demo_trial — guest-on-aws

B-AWS-01 API p50 target: 55 ms.
B-AWS-02 API p95 target: 180 ms.
B-AWS-03 API p99 target: 420 ms.
B-AWS-04 Evidence write throughput target: 220 writes/sec per tenant cell.
B-AWS-05 Effective-policy compute p95 target: 110 ms.
B-AWS-06 Pack-projection staleness p95 target: 75 seconds.
B-AWS-07 Conflict report generation target: <= 6 minutes.
B-AWS-08 DSAR concurrent active requests: 10 per tenant.
B-AWS-09 Regulator export typical 90-day bundle p99 target: 5 hours.
B-AWS-10 Active tenant ceiling target: 50 tenants per cell.

### demo_trial — guest-on-oci Always Free

B-OCI-01 API p50 target: 80 ms under Ampere A1 Always Free sizing.
B-OCI-02 API p95 target: 250 ms under Always Free sizing.
B-OCI-03 API p99 target: 650 ms under Always Free sizing.
B-OCI-04 Evidence write throughput target: 60 writes/sec per tenant cell.
B-OCI-05 Effective-policy compute p95 target: 180 ms.
B-OCI-06 Pack-projection staleness p95 target: 180 seconds.
B-OCI-07 Conflict report generation target: <= 12 minutes.
B-OCI-08 DSAR concurrent active requests: 3 per tenant on Always Free.
B-OCI-09 Regulator export typical 90-day bundle p99 target: 12 hours.
B-OCI-10 Active tenant ceiling target: 5 small tenants or 1 regulated tenant per Always Free cell.

### demo_trial — on-prem

B-ONP-01 API p50 target: 65 ms assuming local enterprise x86_64 hardware.
B-ONP-02 API p95 target: 220 ms.
B-ONP-03 API p99 target: 550 ms.
B-ONP-04 Evidence write throughput target: 150 writes/sec per tenant cell.
B-ONP-05 Effective-policy compute p95 target: 130 ms.
B-ONP-06 Pack-projection staleness p95 target: 120 seconds.
B-ONP-07 Conflict report generation target: <= 8 minutes.
B-ONP-08 DSAR concurrent active requests: 8 per tenant.
B-ONP-09 Regulator export typical 90-day bundle p99 target: 8 hours.
B-ONP-10 Active tenant ceiling target: 25 tenants per on-prem cell unless hardware evidence proves more.

### demo_trial — colo

B-COLO-01 API p50 target: 55 ms.
B-COLO-02 API p95 target: 190 ms.
B-COLO-03 API p99 target: 450 ms.
B-COLO-04 Evidence write throughput target: 200 writes/sec per tenant cell.
B-COLO-05 Effective-policy compute p95 target: 120 ms.
B-COLO-06 Pack-projection staleness p95 target: 90 seconds.
B-COLO-07 Conflict report generation target: <= 7 minutes.
B-COLO-08 DSAR concurrent active requests: 10 per tenant.
B-COLO-09 Regulator export typical 90-day bundle p99 target: 6 hours.
B-COLO-10 Active tenant ceiling target: 40 tenants per cell.

### demo_trial — oyatie-as-cloud-provider

B-IaaS-01 API p50 target: 50 ms.
B-IaaS-02 API p95 target: 170 ms.
B-IaaS-03 API p99 target: 400 ms.
B-IaaS-04 Evidence write throughput target: 240 writes/sec per tenant cell.
B-IaaS-05 Effective-policy compute p95 target: 105 ms.
B-IaaS-06 Pack-projection staleness p95 target: 70 seconds.
B-IaaS-07 Conflict report generation target: <= 6 minutes.
B-IaaS-08 DSAR concurrent active requests: 10 per tenant.
B-IaaS-09 Regulator export typical 90-day bundle p99 target: 5 hours.
B-IaaS-10 Active tenant ceiling target: 50 tenants per cell.

### paid dedicated-cloud — all contexts base targets

S-01 API p50 target: 35 ms for public-cloud/colo/provider, 45 ms guest-on-aws, 50 ms guest-on-oci paid, 55 ms on-prem.
S-02 API p95 target: 120 ms public-cloud/colo/provider, 150 ms guest-on-aws, 170 ms guest-on-oci paid, 190 ms on-prem.
S-03 API p99 target: 300 ms public-cloud/colo/provider, 360 ms guest-on-aws, 420 ms guest-on-oci paid, 480 ms on-prem.
S-04 Evidence write throughput target: 1,000 writes/sec per cell public-cloud/provider, 800 AWS, 700 OCI paid, 500 on-prem, 900 colo.
S-05 Effective-policy compute p95 target: 80 ms, matching tier matrix line 84.
S-06 Pack-projection cross-region convergence p95 target: 5 minutes, matching tier matrix line 85.
S-07 DSAR concurrent active requests: 100 per tenant, matching tier matrix line 80.
S-08 Active tenant ceiling target: 1,000 tenants per cell, matching tier matrix line 78.
S-09 Pack ceiling target: 20 packs per tenant, matching tier matrix line 77.
S-10 Regulator export 1-year typical bundle p99 target: 3 hours.
S-11 Breach notification authority enqueue p99 target: 30 seconds after validated declaration.
S-12 Trust-center gated-access decision p95 target: 150 ms after parity feature lands.

### paid on-prem-connected — all contexts base targets

G-01 API p50 target: 25 ms public-cloud/provider/colo, 35 ms AWS/OCI paid, 45 ms on-prem.
G-02 API p95 target: 90 ms public-cloud/provider/colo, 110 ms AWS/OCI paid, 140 ms on-prem.
G-03 API p99 target: 220 ms public-cloud/provider/colo, 270 ms AWS/OCI paid, 350 ms on-prem.
G-04 Evidence write throughput target: 5,000 writes/sec per cell public-cloud/provider, 4,000 AWS, 3,500 OCI paid, 2,500 on-prem, 4,500 colo.
G-05 Effective-policy compute p95 target: 60 ms, matching tier matrix line 116.
G-06 Multi-pack conflict resolution p99 target: 200 ms, matching tier matrix line 117.
G-07 Regulator export bundle p99 target: 1 hour for typical one-year evidence scope, matching tier matrix line 118.
G-08 DSAR fulfillment p99 target: 14 days, matching tier matrix line 119.
G-09 Availability target: 99.99 percent monthly, matching tier matrix line 120.
G-10 Cross-microservice DSR cascade fanout target: 1,000 service-subject lookups/minute per tenant.
G-11 Trust-center buyer question answer retrieval p95 target: 500 ms after parity feature lands.
G-12 Vendor assessment AI summary p95 target: 2 minutes for 100-page SOC report after parity feature lands.

### paid compliance_pack — all contexts base targets

P-01 API p50 target: 20 ms within certified cell, 35 ms for air-gapped internal edge, 45 ms for on-prem sovereign cell.
P-02 API p95 target: 70 ms within certified cell, 120 ms for air-gapped internal edge, 160 ms for on-prem sovereign cell.
P-03 API p99 target: 180 ms within certified cell, 300 ms for air-gapped internal edge, 420 ms for on-prem sovereign cell.
P-04 Evidence write throughput target: 10,000 writes/sec per pack-bound cell, lower only when air-gap media transfer is active.
P-05 Effective-policy compute p95 target: 50 ms for hot cache, 100 ms for cold pack-set projection.
P-06 Multi-pack conflict resolution p99 target: 150 ms inside one pack-bound cell.
P-07 Regulator export bundle p99 target: 30 minutes for one-year evidence scope inside connected sovereign cell.
P-08 Regulator-attested pack publish ceremony p99 target: <= 30 days, matching tier matrix line 144 because regulator coordination dominates.
P-09 Availability target: 99.99 percent monthly per pack-bound cell, matching tier matrix line 144.
P-10 Cross-pack federation: disabled by construction, so cross-pack throughput target is zero data-plane operations.
P-11 Air-gapped signed-bundle import target: 4 hours from media receipt to validated staging.
P-12 Sovereign audit evidence retention target: 7 years online-searchable for common regulated packs, longer where pack demands.

## §4 Per-context overlay

CTX-01 `oyatie-public-cloud`: use baseline targets from each tier because Oyatie controls cell topology, telemetry, state backend, and deployment cadence.
CTX-02 `guest-on-aws`: add 10-20 percent latency budget versus public-cloud baseline until AWS module and state backend are measured.
CTX-03 `guest-on-oci`: paid OCI can approach AWS targets; Always Free demo_trial uses the dedicated constrained profile.
CTX-04 `on-prem`: add 20-50 percent latency budget until hardware class, storage class, and network are certified.
CTX-05 `colo`: use near-public-cloud targets when hardware and network are Oyatie-certified; otherwise fall back to on-prem targets.
CTX-06 `oyatie-as-cloud-provider`: use public-cloud targets after cloud-* substrate proves compute/storage/network/IAM/KMS performance.
CTX-07 Multi-context evidence required: each context must publish measured p50/p95/p99, RPS, evidence throughput, DSAR throughput, and regulator export latency before launch claim.
CTX-08 Tenant class overlay: SMB tenants use demo_trial/paid dedicated-cloud ceilings; regulated enterprise tenants use paid on-prem-connected; sovereign/public-sector tenants use paid compliance_pack.
CTX-09 Architecture overlay: aarch64 targets must be measured separately from x86_64.
CTX-10 OS overlay: Talos/Flatcar/Photon container-host OSes measure node runtime; RHEL/Oracle/SLES/Ubuntu/Debian/Rocky/Alma/CentOS/Amazon Linux measure package/runtime paths; macOS M5+ measures local dev/admin tooling only unless explicitly declared.
CTX-11 Storage overlay: SeaweedFS evidence durability target remains 99.999 percent from PRD line 57, but measured durability evidence must come from storage service.
CTX-12 Audit-chain overlay: seal verification rate target remains 100 percent from PRD line 56.

## §5 Comparison narrative

CMP-01 API latency: counterparts do not publish comparable API latency, so Oyatie targets set an internal SLO bar rather than claiming advantage.
CMP-02 Evidence automation: Oyatie must match Vanta/Drata continuous evidence posture with measured evidence write and control-monitoring latency.
CMP-03 Audit-prep time: Drata claims large audit-prep savings; Oyatie should measure hours saved through generated evidence packets and auditor self-service events.
CMP-04 Trust center automation: Vanta reports high approval/NDA automation; Oyatie has no product surface today and is behind.
CMP-05 Questionnaire automation: Vanta/Drata publish AI questionnaire productivity claims; Oyatie has no target workload yet and is behind.
CMP-06 Third-party assessment: Vanta and OneTrust publish acceleration percentages; Oyatie has no vendor-risk pipeline and is behind.
CMP-07 DSR automation: OneTrust reports up to 99 percent cost reduction; Oyatie has DSAR endpoints and should set cost-per-DSAR and operator-touch targets.
CMP-08 Pack conflict resolution: Oyatie's p99 200 ms paid on-prem-connected target is additive and likely ahead if implemented and measured.
CMP-09 Regulator export: Oyatie's one-hour paid on-prem-connected target is aggressive and should be measured on one-year evidence bundles.
CMP-10 OCI Always Free demo_trial: this is Oyatie-specific and currently a gap because present demo_trial hardware does not fit Always Free.
CMP-11 OS disclosure: Oyatie has a stricter disclosure bar than counterparts; current service lacks the manifest to satisfy it.
CMP-12 Rust build disclosure: Oyatie has a stricter implementation bar than counterparts; current service lacks a Rust crate.
CMP-13 Hyperscaler-scale claim: only paid on-prem-connected/paid compliance_pack targets approach that bar, and none are measured.
CMP-14 Stop condition: no performance claim should leave target status until benchmark harness, implementation, OpenTofu deploy, OS matrix, and telemetry exist.

## §6 Benchmark readiness and measurement ledger

READY-01 Evidence ingest benchmark must use a fixed event envelope: tenant, pack, control, source, actor, object, seal reference, and retention policy.
READY-02 Evidence ingest benchmark must run separately for manual upload, connector-pulled evidence, audit-chain replay evidence, and regulator-export evidence.
READY-03 Evidence ingest benchmark must measure accepted writes, rejected writes, duplicate writes, and seal-verification failures separately.
READY-04 API latency benchmark must include `/dsar` creation, `/dsar/{id}` read, `/evidence` upload, and auditor/regulator evidence package read once contracts expand.
READY-05 Effective-policy benchmark must include one pack, five packs, twenty packs, and thirty-plus packs because tier limits change conflict complexity.
READY-06 Conflict-resolution benchmark must include clean overlay, direct contradiction, higher-restriction-wins, emergency override, and legal-hold cases.
READY-07 DSAR benchmark must include intake, identity verification, cross-service fanout, export, redaction, subject response, and deletion/rectification evidence.
READY-08 Breach-clock benchmark must include declaration, jurisdiction selection, deadline computation, authority notice queue, subject-notice queue, and proof export.
READY-09 Regulator-export benchmark must include 30-day, 90-day, one-year, and seven-year evidence windows.
READY-10 Trust-center benchmark is blocked until the service owns or delegates trust-center scope; no measured number should be invented before that decision.
READY-11 Questionnaire benchmark is blocked until the service owns or delegates questionnaire automation; current numbers are parity targets only.
READY-12 TPRM benchmark is blocked until vendor-risk ownership is settled; current numbers are derived from counterpart capability gaps.
READY-13 Consent benchmark is blocked until consent/preference ownership is settled; compliance can still benchmark consent-evidence capture.
READY-14 AI governance benchmark is blocked until AI inventory ownership is settled; compliance can still benchmark EU AI Act pack evidence export.
READY-15 demo_trial OCI benchmark must be run on the exact Always Free shape, not on the current demo_trial 3-node EPYC shape in `tier-matrix.md:19`.
READY-16 demo_trial OCI benchmark must fail the release gate if paid shapes, paid state backends, or paid managed services are required for demo_trial claims.
READY-17 paid dedicated-cloud benchmark must include paid guest-on-OCI and guest-on-AWS baselines because demo_trial OCI is intentionally constrained.
READY-18 paid on-prem-connected benchmark must include multi-region pack projection and regulator export under production-size evidence volume.
READY-19 paid compliance_pack benchmark must include air-gap import/export where applicable, and must not report ordinary connected-cloud numbers as air-gap evidence.
READY-20 Public-cloud context benchmark must run in Oyatie-operated infrastructure and use Oyatie-owned telemetry, not guest-provider telemetry only.
READY-21 Guest-on-AWS benchmark must use the AWS OpenTofu module after it exists and must disclose region, instance class, storage class, and KMS path.
READY-22 Guest-on-OCI benchmark must run separate Always Free demo_trial and paid OCI profiles.
READY-23 On-prem benchmark must disclose CPU, memory, disk, NIC, Kubernetes distribution, and OS image because customer hardware variance dominates results.
READY-24 Colo benchmark must disclose facility/network class and whether hardware is Oyatie-certified.
READY-25 Oyatie-as-cloud-provider benchmark must wait for cloud-compute, cloud-storage, cloud-network, IAM, and KMS substrate measurements.
READY-26 OS benchmark must run at least one Tier-1 representative from RPM family, DEB family, immutable/container host family, and macOS M5+ admin tooling.
READY-27 OS benchmark must not include Intel macOS, pre-M5 Apple Silicon, FreeBSD, OpenBSD, Windows Server, or Solaris as supported targets.
READY-28 Architecture benchmark must split x86_64 and aarch64 results because OCI Always Free and Apple M5+ paths are architecture-sensitive.
READY-29 Tenant-class benchmark must publish SMB, regulated enterprise, and sovereign/public-sector profiles separately.
READY-30 Retention benchmark must measure hot searchable evidence, warm archive retrieval, and long-retention proof reconstruction separately.
READY-31 Cost benchmark must report cost per accepted evidence event, cost per DSAR, cost per regulator package, and cost per active tenant per month.
READY-32 Cost benchmark must reconcile PRD line 101 fleet cost with tier-matrix line 122 annual cell cost before a public cost claim.
READY-33 Reliability benchmark must measure seal verification rate, audit-chain lag, pack publish failure rate, and evidence loss/recovery path.
READY-34 Security benchmark must measure rejected unauthorized auditor access, rejected cross-tenant DSAR access, and stale grant revocation latency.
READY-35 Observability benchmark must verify that every performance workload emits metrics, traces, logs, and audit-chain evidence without leaking regulated data.
READY-36 CI benchmark lane must run from `cargo build --workspace --release --all-features --locked` plus service tests after a Rust crate exists.
READY-37 OpenTofu benchmark lane must run `tofu init`, `tofu validate`, `tofu plan`, and signed module verification per context.
READY-38 Benchmark publication must mark every number as `target`, `measured-lab`, `measured-staging`, or `measured-production`.
READY-39 Counterpart comparison must stay conservative because public Vanta/Drata/OneTrust pages publish productivity and product claims more often than raw latency.
READY-40 Final readiness verdict: current document establishes targets and measurement gates; it does not convert any compliance performance target into measured evidence.
