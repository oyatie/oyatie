# Real Estate Performance Benchmark Numbers - 2026-05-20

Target microservice: `real-estate`.
Benchmark bar: AppFolio / Yardi Voyager / RealPage.
Local evidence anchor 1: current OpenSLO p99 objective counts requests under `le="0.35"` seconds (`slos/real-estate-latency-p99.openslo.yaml:19-29`).
Local evidence anchor 2: current availability objective targets `0.999` (`slos/real-estate-availability.openslo.yaml:27-29`).
Local evidence anchor 3: current throughput objective targets accepted over received commands at `0.995` (`slos/real-estate-throughput.openslo.yaml:21-29`).
Local evidence anchor 4: current PRD observability target says command p50 120 ms, p95 300 ms, p99 750 ms, and batch lag warning 300 s/page 900 s (`PRD.md:1060-1125`).
Local evidence anchor 5: current capacity model uses Little's Law and old assumptions that must be replaced with single targets plus overlays (`capacity-model.md:13-23`).
Canonical anchor 1: all six deployment contexts are required or need explicit N/A records (`specs/master-plan-sequencing.json:704-746`).
Canonical anchor 2: OpenTofu is the infrastructure substrate (`specs/master-plan-sequencing.json:747-776`).
Canonical anchor 3: OS support must be explicit (`specs/master-plan-sequencing.json:777-815`).
Canonical anchor 4: backend policy is Rust-strict (`specs/master-plan-sequencing.json:817-856`).
Canonical anchor 5: OCI Always Free profile resource envelope is 4 OCPU, 24 GB RAM, 200 GB block, 10 GB object, 2 autonomous DBs, 10 TB egress, and 10 Mbps load balancer (`specs/master-plan-sequencing.json:857-868`).
External source 1: AppFolio product surface includes accounting, marketing/leasing, work orders, inspections, portals, and database API (`https://www.appfolio.com/performance-platform`, opened lines 100-153).
External source 2: AppFolio 2026 benchmark survey reports 1,617 respondents, 77 percent expecting unit-count growth, 31 percent growth expectation for broad AI adopters, 12 percent for non-adopters, and 55 percent vacancy pressure (`https://www.appfolio.com/newsroom/property-manager-benchmark-survey-2026`, opened lines 28-62).
External source 3: AppFolio analysis reports feature scores for AppFolio, Yardi Voyager, and RealPage OneSite across lease templates, tenant portal, amenity management, maintenance requests, and building announcements (`https://www.appfolio.com/blog/top-tools-for-multifamily-property-management-2026`, opened lines 25-51).
External source 4: Yardi Voyager source describes web-based integrated end-to-end platform, mobile access, operations, leasing, analytics, resident/tenant/investor services, accounting, and maintenance (`https://www.yardi.com/suite/voyager-suite/`, opened lines 675-881).
External source 5: RealPage source reports OneSite lifecycle, application screening, portfolio views, AI workflows, vendor management, compliance, and an 83 percent staff-time reduction for selected processes (`https://www.realpage.com/property-management-software/`, opened lines 585-606).
External source 6: RealPage multifamily source reports more than 24 million rental units and integrated leasing, operations, accounting, resident experience, payments, maintenance, portals, and enterprise-scale operations (`https://www.realpage.com/multifamily/`, opened lines 567-641 and 915-947).
Methodology disclosure: none of the three counterparts publishes a complete public p50/p95/p99/RPS benchmark set for the exact property-management workflows in this audit.
Methodology disclosure: public numbers below are marked as `source` when directly public, and `estimated from` when derived from public feature scale plus enterprise SaaS workload assumptions.
Methodology disclosure: estimates are planning targets, not claims about the counterpart's private production systems.
Methodology disclosure: Oyatie targets are normative engineering goals for this microservice, not measured results from this scaffold.
Methodology disclosure: target rows use a single industry-leader target set plus deployment-context and tenant-class overlays.
Methodology disclosure: no retired feature-level headings or rows are used.
Tenant-class disclosure: `demo_trial` has usage caps and OCI Always Free profile infrastructure, but the same product-quality targets as paid tenants until caps reject work.
Tenant-class disclosure: `paid` scales with contractual payment and allowed compliance/SLO packs.
Tenant-class disclosure: `revenue_share` runs at-cost or zero-margin substrate while preserving product-quality targets and settlement evidence.

## 1. Methodology

Benchmark dimension 001: interactive command latency p50.
Benchmark dimension 002: interactive command latency p95.
Benchmark dimension 003: interactive command latency p99.
Benchmark dimension 004: interactive read latency p50.
Benchmark dimension 005: interactive read latency p95.
Benchmark dimension 006: interactive read latency p99.
Benchmark dimension 007: mutation accepted/received ratio.
Benchmark dimension 008: availability.
Benchmark dimension 009: sustained command throughput.
Benchmark dimension 010: burst command throughput.
Benchmark dimension 011: event publish lag.
Benchmark dimension 012: worker replay lag.
Benchmark dimension 013: monthly close batch wall clock.
Benchmark dimension 014: rent-roll generation wall clock.
Benchmark dimension 015: lease-accounting recompute throughput.
Benchmark dimension 016: facility service request intake throughput.
Benchmark dimension 017: tenant-scoped report export wall clock.
Benchmark dimension 018: cross-tenant isolation false-positive rate.
Benchmark dimension 019: Cedar authorization decision latency.
Benchmark dimension 020: OpenBao secret lease acquisition latency.
Benchmark dimension 021: source import row throughput.
Benchmark dimension 022: idempotent retry completion.
Benchmark dimension 023: dashboard freshness.
Benchmark dimension 024: SLO burn alerting latency.
Benchmark dimension 025: disaster-recovery replay rate.
Benchmark dimension 026: data-residency decision latency.
Benchmark dimension 027: audit-event signing latency.
Benchmark dimension 028: object storage export bandwidth.
Benchmark dimension 029: tenant-class cap enforcement latency.
Benchmark dimension 030: deployment-context provisioning duration.
Test workload 001: create or amend lease contract.
Test workload 002: create or amend facility master.
Test workload 003: create or amend occupancy allocation.
Test workload 004: create or amend rent schedule.
Test workload 005: create or amend lease-accounting event.
Test workload 006: create or amend facility service request.
Test workload 007: generate rent roll for 10,000 units.
Test workload 008: replay 1,000,000 imported source rows.
Test workload 009: recompute IFRS16 right-of-use balances for 100,000 lease lines.
Test workload 010: reconcile CAM charges for 10,000 tenant-ledger pairs.
Test workload 011: export compliance evidence for one tenant and one fiscal year.
Test workload 012: simulate regional failover with no cross-tenant state leakage.
Test workload 013: enforce `demo_trial` usage cap at ingress.
Test workload 014: enforce paid compliance pack and BYOK context.
Test workload 015: attach revenue-share settlement reference without direct billing ownership.
OS disclosure: final benchmark harness must include the service-supported OS matrix after `supported-oses.json` exists.
Architecture disclosure: baseline target assumes linux amd64/arm64 containers with Rust release build and OpenTofu-managed infrastructure.
Deployment disclosure: all six contexts must run the same target suite; overlays describe capacity constraints, not lower feature quality.
Tenant-class disclosure: all tenant classes use the same correctness, audit, authorization, and data isolation targets.
Stop condition: a performance target is reportable only when benchmark harness, environment, data shape, and result artifact are checked into service-local evidence.

## 2. Counterpart Numbers

### 2.1 AppFolio Numbers

AppFolio number 001: 1,617 U.S. residential property management professionals in the 2026 benchmark survey; source: AppFolio survey methodology.
AppFolio number 002: 77 percent of managers expected to increase unit counts in 2026; source: AppFolio 2026 benchmark survey.
AppFolio number 003: 81 percent of managers reported a positive outlook; source: AppFolio 2026 benchmark survey.
AppFolio number 004: 55 percent cited elevated vacancy rates as the top NOI threat; source: AppFolio 2026 benchmark survey.
AppFolio number 005: broad AI adopters expected 31 percent average portfolio growth; source: AppFolio 2026 benchmark survey.
AppFolio number 006: non-AI adopters expected 12 percent average portfolio growth; source: AppFolio 2026 benchmark survey.
AppFolio number 007: 34 percent of AI adopters planned headcount growth; source: AppFolio 2026 benchmark survey.
AppFolio number 008: 25 percent of non-users planned headcount growth; source: AppFolio 2026 benchmark survey.
AppFolio number 009: 45 percent of operators planned tech-stack consolidation; source: AppFolio 2026 benchmark survey.
AppFolio number 010: 65 percent of residents wanted security-deposit alternatives; source: AppFolio 2026 benchmark survey.
AppFolio number 011: 71 percent of residents valued bundled resident services; source: AppFolio 2026 benchmark survey.
AppFolio number 012: 56 percent of managers faced application fraud in the prior year; source: AppFolio 2026 benchmark survey.
AppFolio number 013: 86 percent lease-template score in AppFolio multifamily analysis; source: AppFolio analysis.
AppFolio number 014: 94 percent tenant-portal score in AppFolio multifamily analysis; source: AppFolio analysis.
AppFolio number 015: 91 percent maintenance-service-request score in AppFolio multifamily analysis; source: AppFolio analysis.
AppFolio number 016: 88 percent building-announcement score in AppFolio multifamily analysis; source: AppFolio analysis.
AppFolio number 017: 84 percent amenity-management score in AppFolio multifamily analysis; source: AppFolio analysis.
AppFolio number 018: 92 percent ease-of-use score in AppFolio 2026 analysis; source: AppFolio ease-of-use analysis.
AppFolio number 019: 89 percent user-adoption rate in AppFolio 2026 analysis; source: AppFolio ease-of-use analysis.
AppFolio number 020: 79 percent autonomous-task-execution score in AppFolio 2026 analysis; source: AppFolio ease-of-use analysis.
AppFolio number 021: 12,000+ units in an AppFolio customer quote on the AppFolio home page; source: AppFolio home page.
AppFolio number 022: p95 interactive command latency is not publicly published; estimated from SaaS user-experience bar at 500-900 ms.
AppFolio number 023: p99 interactive command latency is not publicly published; estimated from enterprise SaaS tolerance at 1.2-2.0 s.
AppFolio number 024: resident portal read latency is not publicly published; estimated from consumer portal expectations at p95 below 800 ms.
AppFolio number 025: batch report export throughput is not publicly published; estimated from property-manager reporting workflows at 10,000-50,000 rows per minute.

### 2.2 Yardi Voyager Numbers

Yardi number 001: Voyager is web-based and integrated end-to-end; source: Yardi Voyager product page.
Yardi number 002: Voyager supports larger portfolios; source: Yardi Voyager product page.
Yardi number 003: Voyager supports mobile access; source: Yardi Voyager product page.
Yardi number 004: Voyager supports operations, leasing, analytics, resident services, tenant services, and investor services; source: Yardi Voyager product page.
Yardi number 005: Yardi property-management software includes accounting, operations, and ancillary services for residential and commercial portfolios; source: Yardi property-management page.
Yardi number 006: Yardi property-management software includes integrated accounting, marketing, lease execution, market intelligence, energy management, procurement, BI, and more; source: Yardi property-management FAQ.
Yardi number 007: Yardi property-management page says tenant self-service includes online payments, leases, requests, and account management; source: Yardi FAQ.
Yardi number 008: 78 percent lease-template score for Yardi Voyager in AppFolio multifamily analysis; source: AppFolio analysis.
Yardi number 009: 82 percent tenant-portal score for Yardi Voyager in AppFolio multifamily analysis; source: AppFolio analysis.
Yardi number 010: 80 percent amenity-management score for Yardi Voyager in AppFolio multifamily analysis; source: AppFolio analysis.
Yardi number 011: 85 percent maintenance-service-request score for Yardi Voyager in AppFolio multifamily analysis; source: AppFolio analysis.
Yardi number 012: 77 percent building-announcement score for Yardi Voyager in AppFolio multifamily analysis; source: AppFolio analysis.
Yardi number 013: p95 interactive command latency is not publicly published; estimated from enterprise web-based platform expectations at 600-1200 ms.
Yardi number 014: p99 interactive command latency is not publicly published; estimated from large-portfolio enterprise SaaS workflows at 1.5-3.0 s.
Yardi number 015: portfolio report export throughput is not publicly published; estimated from BI/reporting workflows at 10,000-100,000 rows per minute depending on deployment.
Yardi number 016: maintenance work-order intake throughput is not publicly published; estimated from large portfolio operations at 50-500 requests per second per tenant group.
Yardi number 017: leasing batch update throughput is not publicly published; estimated from large portfolio update waves at 5,000-25,000 lease records per minute.
Yardi number 018: dashboard freshness is not publicly published; estimated from operational dashboard norms at 30-300 seconds.
Yardi number 019: data replication freshness is not publicly published; estimated from BI-feed workloads at 1-15 minutes.
Yardi number 020: availability SLA is not publicly published in the inspected official product pages; planning comparison uses industry SaaS baseline of 99.9 percent or better.

### 2.3 RealPage Numbers

RealPage number 001: RealPage reports more than 24 million rental units in its ecosystem; source: RealPage multifamily page.
RealPage number 002: RealPage reports more than 8,000 employees; source: RealPage multifamily page.
RealPage number 003: RealPage says OneSite supports conventional, student, affordable, tax-credit, military, and senior-living properties; source: RealPage OneSite page.
RealPage number 004: RealPage says selected OneSite processes show 83 percent reduction in staff time; source: RealPage OneSite page.
RealPage number 005: RealPage positions Essentials for operators managing less than 2,000 units; source: RealPage OneSite page.
RealPage number 006: 85 percent lease-template score for RealPage OneSite in AppFolio multifamily analysis; source: AppFolio analysis.
RealPage number 007: 71 percent tenant-portal score for RealPage OneSite in AppFolio multifamily analysis; source: AppFolio analysis.
RealPage number 008: 83 percent amenity-management score for RealPage OneSite in AppFolio multifamily analysis; source: AppFolio analysis.
RealPage number 009: 83 percent maintenance-service-request score for RealPage OneSite in AppFolio multifamily analysis; source: AppFolio analysis.
RealPage number 010: 70 percent building-announcement score for RealPage OneSite in AppFolio multifamily analysis; source: AppFolio analysis.
RealPage number 011: RealPage says its platform is enterprise-scale; source: RealPage multifamily page.
RealPage number 012: RealPage says its platform includes online payments and financial reports; source: RealPage multifamily page.
RealPage number 013: RealPage says its platform includes maintenance requests and renter communication tools; source: RealPage multifamily page.
RealPage number 014: RealPage says its platform integrates marketing, payment processors, utility billing, smart building, screening, CRM, and BI systems; source: RealPage multifamily page.
RealPage number 015: p95 interactive command latency is not publicly published; estimated from enterprise property-management platform expectations at 600-1000 ms.
RealPage number 016: p99 interactive command latency is not publicly published; estimated from enterprise SaaS workflows at 1.5-2.5 s.
RealPage number 017: centralized portfolio dashboard freshness is not publicly published; estimated from operational analytics expectation at 30-180 seconds.
RealPage number 018: AI workflow completion time is not publicly published; estimated from the public 83 percent staff-time reduction statement as a process-level benchmark.
RealPage number 019: application screening throughput is not publicly published; estimated from large multifamily applicant workloads at 100-1,000 applications per minute across a portfolio.
RealPage number 020: maintenance request throughput is not publicly published; estimated from large multifamily operations at 100-1,000 requests per second across a portfolio.

## 3. Oyatie Target Numbers - Single Industry-Leader Target Set

Target 001: command write latency p50 <= 75 ms.
Target 002: command write latency p95 <= 200 ms.
Target 003: command write latency p99 <= 350 ms.
Target 004: command read latency p50 <= 50 ms.
Target 005: command read latency p95 <= 150 ms.
Target 006: command read latency p99 <= 250 ms.
Target 007: Cedar authorization decision p50 <= 5 ms.
Target 008: Cedar authorization decision p95 <= 15 ms.
Target 009: Cedar authorization decision p99 <= 30 ms.
Target 010: OpenBao secret acquisition p95 <= 100 ms.
Target 011: idempotency check p99 <= 25 ms.
Target 012: audit-event signing p99 <= 20 ms.
Target 013: event publish lag p95 <= 500 ms.
Target 014: event publish lag p99 <= 2 s.
Target 015: dashboard freshness p95 <= 30 s.
Target 016: SLO burn alert delivery <= 60 s.
Target 017: sustained command throughput per standard cell >= 5,000 rps.
Target 018: burst command throughput per standard cell >= 15,000 rps for 5 minutes.
Target 019: tenant-scoped report export >= 100,000 rows/minute.
Target 020: source import replay >= 250,000 rows/minute per worker pool.
Target 021: lease-accounting recompute >= 50,000 lease lines/minute.
Target 022: CAM reconciliation >= 100,000 ledger pairs/hour.
Target 023: rent-roll generation for 10,000 units <= 60 s.
Target 024: compliance evidence export for one tenant-year <= 120 s.
Target 025: facility service request intake p99 <= 350 ms.
Target 026: facility service request event fanout p99 <= 2 s.
Target 027: cross-tenant isolation false positives <= 1 per 10 million requests.
Target 028: cross-tenant isolation false negatives = 0 accepted.
Target 029: availability monthly target >= 99.95 percent for paid contractual contexts.
Target 030: availability monthly target >= 99.9 percent for demo trial infrastructure profile.
Target 031: mutation accepted/received ratio >= 99.95 percent excluding valid rejects.
Target 032: data-residency decision latency p99 <= 50 ms.
Target 033: tenant-class cap rejection latency p99 <= 50 ms.
Target 034: disaster-recovery replay >= 1,000,000 events/hour per recovery cell.
Target 035: regional failover control-plane decision <= 60 s after confirmed incident.
Target 036: p99 API latency stays <= 350 ms during policy bundle refresh.
Target 037: p99 API latency stays <= 500 ms during background import.
Target 038: p99 API latency stays <= 700 ms during regional failover.
Target 039: memory footprint idle service <= 256 MiB.
Target 040: memory footprint under 1,000 rps <= 1.5 GiB per replica.
Target 041: CPU under 1,000 rps <= 1 vCPU average per replica.
Target 042: startup readiness <= 5 s after container start.
Target 043: graceful shutdown drain <= 20 s.
Target 044: OpenTofu context plan duration <= 5 minutes for standard module.
Target 045: OpenTofu context apply duration <= 20 minutes for standard module.

### 3.1 Deployment-Context Overlays

Context overlay 001: `oyatie-public-cloud` can horizontally scale to the full target set.
Context overlay 002: `oyatie-public-cloud` target sustained throughput starts at 5,000 rps per cell and scales by adding cells.
Context overlay 003: `oyatie-public-cloud` target availability is 99.95 percent or higher for paid contractual contexts.
Context overlay 004: `oyatie-public-cloud` target p99 command latency remains <= 350 ms inside one region.
Context overlay 005: `guest-on-aws` inherits customer account quota constraints.
Context overlay 006: `guest-on-aws` target sustained throughput is 2,500 rps per provisioned cell unless account quotas allow more.
Context overlay 007: `guest-on-aws` target p99 command latency remains <= 450 ms after customer-network overhead.
Context overlay 008: `guest-on-aws` target report export remains >= 50,000 rows/minute if storage and network quotas permit.
Context overlay 009: `guest-on-oci` full paid context can meet standard targets when provisioned above Always Free limits.
Context overlay 010: `guest-on-oci` OCI Always Free profile caps sustained command throughput at 250 rps planning target.
Context overlay 011: `guest-on-oci` OCI Always Free profile caps burst command throughput at 500 rps for 60 seconds.
Context overlay 012: `guest-on-oci` OCI Always Free profile caps tenant-scoped report export at 10,000 rows/minute.
Context overlay 013: `guest-on-oci` OCI Always Free profile targets p99 command latency <= 700 ms under cap.
Context overlay 014: `guest-on-oci` OCI Always Free profile must reject excess work before queue saturation.
Context overlay 015: `on-prem` target depends on customer hardware and storage latency.
Context overlay 016: `on-prem` minimum certified target is 1,000 rps sustained per certified appliance profile.
Context overlay 017: `on-prem` target p99 command latency is <= 500 ms on certified hardware.
Context overlay 018: `on-prem` report export target is >= 25,000 rows/minute on certified storage.
Context overlay 019: `colo` target assumes customer-provided network and facility redundancy.
Context overlay 020: `colo` minimum certified target is 2,000 rps sustained per rack-cell.
Context overlay 021: `colo` target p99 command latency is <= 450 ms inside the metro.
Context overlay 022: `colo` disaster-recovery replay target depends on cross-facility link capacity.
Context overlay 023: `oyatie-as-cloud-provider` target equals or exceeds `oyatie-public-cloud` for core command paths.
Context overlay 024: `oyatie-as-cloud-provider` target sustained throughput starts at 10,000 rps per provider cell.
Context overlay 025: `oyatie-as-cloud-provider` target availability is 99.99 percent when multi-cell routing is enabled.
Context overlay 026: `oyatie-as-cloud-provider` target p99 command latency remains <= 300 ms inside one provider region.

### 3.2 Tenant-Class Overlays

Tenant overlay 001: `demo_trial` quality target is identical to paid quality until cap rejection.
Tenant overlay 002: `demo_trial` command write p99 target is <= 700 ms on OCI Always Free profile.
Tenant overlay 003: `demo_trial` sustained throughput cap is 250 rps per tenant on OCI Always Free profile.
Tenant overlay 004: `demo_trial` burst throughput cap is 500 rps for 60 seconds.
Tenant overlay 005: `demo_trial` monthly command cap must be enforced at ingress with p99 rejection <= 50 ms.
Tenant overlay 006: `demo_trial` compliance pack activation is disabled by commercial policy, not by lower service correctness.
Tenant overlay 007: `demo_trial` BYOK is disabled by commercial policy, not by weaker cryptography.
Tenant overlay 008: `paid` command write p99 target is <= 350 ms in elastic contexts.
Tenant overlay 009: `paid` sustained throughput scales with contract and provisioned cells.
Tenant overlay 010: `paid` compliance packs are allowed where policy and deployment context support them.
Tenant overlay 011: `paid` BYOK is allowed where OpenBao and customer key workflow are provisioned.
Tenant overlay 012: `paid` availability target is contractual and starts at 99.95 percent for production cells.
Tenant overlay 013: `revenue_share` command write p99 target is <= 350 ms on provisioned at-cost substrate.
Tenant overlay 014: `revenue_share` settlement reference attach p99 target is <= 50 ms.
Tenant overlay 015: `revenue_share` gross-revenue event evidence must publish within 2 s p99.
Tenant overlay 016: `revenue_share` sustained throughput scales with marketplace volume and at-cost capacity.
Tenant overlay 017: `revenue_share` zero-margin substrate must not weaken audit, isolation, or data-residency targets.
Tenant overlay 018: all tenant classes require zero accepted cross-tenant isolation false negatives.
Tenant overlay 019: all tenant classes require audit-event signing p99 <= 20 ms.
Tenant overlay 020: all tenant classes require idempotency p99 <= 25 ms.

## 4. Comparison Narrative

Comparison 001: Oyatie command p99 target of 350 ms is ahead of the estimated counterpart interactive p99 range because public competitor p99 values are not disclosed and enterprise SaaS estimates are higher.
Comparison 002: Oyatie command p95 target of 200 ms is ahead of the estimated counterpart p95 range of 500-1200 ms.
Comparison 003: Oyatie dashboard freshness target of 30 s is ahead of the estimated Yardi/RealPage operational dashboard range of 30-300 s.
Comparison 004: Oyatie report export target of 100,000 rows/minute is ahead of the estimated AppFolio/Yardi export planning range for normal tenant reports.
Comparison 005: Oyatie source replay target of 250,000 rows/minute is a catch-up target until an implementation harness proves it.
Comparison 006: Oyatie rent-roll generation target of 60 s for 10,000 units is ahead if achieved; current service only has an IP, not an implementation.
Comparison 007: Oyatie CAM reconciliation target is a catch-up target because current evidence is an IP and not a benchmarked path.
Comparison 008: Oyatie availability target of 99.95 percent for paid contexts is ahead of a 99.9 planning baseline, but current local SLO is only `0.999`.
Comparison 009: Oyatie demo-trial availability target of 99.9 percent matches the current local availability objective and the expected best-effort cap posture.
Comparison 010: Oyatie OCI Always Free throughput cap of 250 rps is below paid context throughput by design, but quality targets remain uniform within cap.
Comparison 011: Oyatie paid context throughput target of 5,000 rps per standard cell is ahead of small/mid-market property-management expectations.
Comparison 012: Oyatie `oyatie-as-cloud-provider` target of 10,000 rps per provider cell is ahead if the service gains real OpenTofu and autoscaling evidence.
Comparison 013: Oyatie Cedar decision targets have no direct counterpart public benchmark; they are additive governance targets.
Comparison 014: Oyatie OpenBao secret acquisition target has no direct counterpart public benchmark; it is additive governance overhead.
Comparison 015: Oyatie event publish target of 500 ms p95 is ahead of normal back-office property-management expectations if achieved.
Comparison 016: Oyatie AI workflow parity is behind AppFolio and RealPage because no AI workflow artifact exists in this service.
Comparison 017: Oyatie tenant portal performance is not comparable because no tenant portal exists in this service.
Comparison 018: Oyatie owner portal performance is not comparable because no owner portal exists in this service.
Comparison 019: Oyatie payment workflow performance is not comparable because payment is only an integration/handoff reference here.
Comparison 020: Oyatie screening workflow performance is not comparable because applicant screening is absent.
Comparison 021: Oyatie mobile workflow performance is not comparable because no mobile surface exists.
Comparison 022: Oyatie maintenance intake target is catch-up because current service has facility-service-request mutation but no resident-facing intake.
Comparison 023: Oyatie document-management performance is catch-up because current service references documents but does not own document workflows.
Comparison 024: Oyatie API p99 target is stronger than current PRD p99 of 750 ms and should replace the old PRD number after implementation planning.
Comparison 025: Oyatie SLO p99 bucket at 350 ms aligns with this target set and should become the primary latency anchor.
Comparison 026: Oyatie current capacity model's old assumptions should be retired and replaced with this single target plus overlays.
Comparison 027: Oyatie target set is not proven by code today because integration tests include ignored contract and policy fixtures.
Comparison 028: Oyatie benchmark maturity is blocked until source, policy, repository, OpenAPI, gRPC, AsyncAPI, and deployment harness tests are active.
Comparison 029: Oyatie can claim architectural ambition but not measured parity until benchmark evidence lands.
Comparison 030: The immediate stop condition is target definition, not performance certification.

## 5. Benchmark Evidence Required Before Certification

Evidence 001: checked-in Rust benchmark harness for six command paths.
Evidence 002: checked-in Rust benchmark harness for report export and rent-roll generation.
Evidence 003: checked-in replay benchmark for source import rows.
Evidence 004: checked-in Cedar decision benchmark.
Evidence 005: checked-in OpenBao integration benchmark.
Evidence 006: checked-in audit-event signing benchmark.
Evidence 007: checked-in event publish lag benchmark.
Evidence 008: checked-in dashboard freshness benchmark.
Evidence 009: checked-in regional failover replay benchmark.
Evidence 010: checked-in tenant-class cap enforcement benchmark.
Evidence 011: checked-in `demo_trial` OCI Always Free profile benchmark.
Evidence 012: checked-in paid elastic cell benchmark.
Evidence 013: checked-in revenue-share settlement-reference benchmark.
Evidence 014: checked-in `oyatie-public-cloud` OpenTofu deployment benchmark.
Evidence 015: checked-in `guest-on-aws` OpenTofu deployment benchmark.
Evidence 016: checked-in `guest-on-oci` OpenTofu deployment benchmark.
Evidence 017: checked-in `on-prem` certified hardware benchmark.
Evidence 018: checked-in `colo` rack-cell benchmark.
Evidence 019: checked-in `oyatie-as-cloud-provider` provider-cell benchmark.
Evidence 020: checked-in supported OS benchmark matrix after `supported-oses.json` exists.
Evidence 021: checked-in CPU and memory profile under 1,000 rps.
Evidence 022: checked-in startup and shutdown timings.
Evidence 023: checked-in failure-injection benchmark for policy bundle refresh.
Evidence 024: checked-in failure-injection benchmark for background import.
Evidence 025: checked-in failure-injection benchmark for regional failover.
Evidence 026: checked-in burn-rate alert latency proof.
Evidence 027: checked-in availability simulation or historical service data after live operation.
Evidence 028: checked-in p50/p95/p99 histogram artifacts.
Evidence 029: checked-in per-context capacity envelope.
Evidence 030: checked-in benchmark report replacing estimates with measured values.
