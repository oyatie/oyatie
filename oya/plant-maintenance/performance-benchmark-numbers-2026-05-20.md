# Plant Maintenance Performance Benchmark Numbers - 2026-05-20

Audit owner: sole Codex audit owner for `plant-maintenance`.
Target microservice path: `microservices/plant-maintenance/`.
Counterpart set: SAP Plant Maintenance / IBM Maximo / UpKeep.
Tenant-class constraint: this report uses deployment_context plus tenant_class target rows, not legacy color labels or retired activation rows.
Target model: one industry-leader-grade Oyatie target set with deployment-context and tenant-class overlays.
Tenant classes: `demo_trial` and `paid`; paid billing components: `per_seat` and `per_usage`.
Local citation 1: p99 latency SLO file uses a 0.35-second threshold at `slos/plant-maintenance-latency-p99.openslo.yaml:21`.
Local citation 2: availability SLO target is 0.999 at `slos/plant-maintenance-availability.openslo.yaml:29`.
Local citation 3: throughput SLO target is 0.995 accepted-over-received ratio at `slos/plant-maintenance-throughput.openslo.yaml:29`.
Local citation 4: PRD states 300 ms p95 at 1000 commands per second requires 300 worker slots before headroom at `PRD.md:1127-1130`.
Local citation 5: canonical OCI Always Free profile is 4 OCPU plus 24 GB RAM at `specs/master-plan-sequencing.json:857-868`.
External source note: SAP Plant Maintenance public docs expose functional surfaces, not raw cloud p50/p95/p99/RPS numbers.
External URL: https://help.sap.com/docs/SAP_ERP/11825b10747e4ee4b91ecc1dba612536/d77cb6535fe6b74ce10000000a174cb4.html
External source note: IBM Maximo public docs expose functional and deployment surfaces, not a comparable raw benchmark table.
External URL: https://www.ibm.com/products/maximo/asset-management
External source note: UpKeep public docs expose CMMS feature and workflow surfaces, not a comparable raw benchmark table.
External URL: https://upkeep.com/product/cmms-software/
Methodology disclosure: counterpart numbers below are estimates derived from public product-surface class, normal interactive EAM/CMMS workload shape, and the local Oyatie SLO/capacity model.
Methodology disclosure: any row marked `estimated` is not a vendor-published benchmark and must not be represented as a vendor SLA.
Methodology disclosure: Oyatie targets are proposed engineering targets for this service, not measured production results.

## 1. Methodology

1. Benchmark dimension: command latency p50 for lightweight validated mutations.
2. Benchmark dimension: command latency p95 for lightweight validated mutations.
3. Benchmark dimension: command latency p99 for lightweight validated mutations.
4. Benchmark dimension: accepted-command throughput in commands per second.
5. Benchmark dimension: concurrent command slots required under Little's Law.
6. Benchmark dimension: batch import rows per minute for migration and replay.
7. Benchmark dimension: event publication lag p95.
8. Benchmark dimension: audit-chain seal latency p95.
9. Benchmark dimension: policy-decision latency p95.
10. Benchmark dimension: mobile or field-sync visible freshness where the counterpart product surface is mobile-heavy.
11. Benchmark dimension: availability target.
12. Benchmark dimension: error-budget or accepted-over-received ratio.
13. Benchmark dimension: tenant-count scale ceiling per service cell.
14. Benchmark dimension: asset-count scale ceiling per service cell.
15. Benchmark dimension: active work-order ceiling per service cell.
16. Test workload: lightweight command for equipment master create/amend.
17. Test workload: lightweight command for maintenance plan schedule/amend.
18. Test workload: lightweight command for work order release/closeout.
19. Test workload: lightweight command for spare-part reservation.
20. Test workload: lightweight command for technician dispatch.
21. Test workload: lightweight command for downtime-window record.
22. Test workload: batch migration import with idempotency and replay.
23. Test workload: policy-deny burst.
24. Test workload: audit-chain seal under regional failover.
25. Test workload: ontology projection lag under normal operation.
26. OS disclosure: current service has no `supported-oses.json`, so OS performance is not verified.
27. Architecture disclosure: current source is a Rust scaffold and not a measured production runtime.
28. Deployment-context disclosure: all six canonical contexts are required, but the service currently has no six-context IaC modules.
29. Tenant-class disclosure: the manifest declares `demo_trial` and `paid`; runtime enforcement evidence still needs implementation.
30. Data disclosure: no vendor provided a directly comparable public raw RPS or latency benchmark in the sources used for this audit.
31. Estimation rule: SAP and Maximo estimates assume heavier enterprise EAM flows than UpKeep because they commonly include complex asset, inventory, workflow, and accounting integrations.
32. Estimation rule: UpKeep estimates assume faster simple-work-order interactions but lower single-tenant enterprise workflow complexity.
33. Estimation rule: Oyatie canonical targets use the stricter local SLO where PRD prose conflicts with OpenSLO.
34. Estimation rule: 350 ms p99 is selected as the canonical lightweight command target because the p99 SLO bucket is 0.35 seconds at `slos/plant-maintenance-latency-p99.openslo.yaml:21`.
35. Estimation rule: 300 ms p95 is retained because PRD capacity math explicitly uses it at `PRD.md:1127-1130`.
36. Estimation rule: demo-trial capacity must be capped by usage policy and the OCI Always Free profile, not by a feature-quality downgrade.
37. Estimation rule: paid capacity scales by contractual resources and usage billing.
38. Estimation rule: revenue-share capacity scales at cost or zero-margin substrate while preserving the same quality target.
39. No row in this report creates a feature ladder.
40. No row in this report says one tenant class receives lower product correctness.

## 2. Counterpart Numbers

### 2.1 SAP Plant Maintenance numbers

1. SAP-01 latency p50: 150 ms to 250 ms estimated for simple PM read or command acknowledgement in a tuned enterprise deployment; source: estimated from SAP PM functional surface and enterprise EAM interaction class.
2. SAP-02 latency p95: 450 ms to 900 ms estimated for order creation or plan scheduling with authorization and accounting hooks; source: estimated from SAP PM functional surface.
3. SAP-03 latency p99: 900 ms to 1500 ms estimated for heavier maintenance-order mutations; source: estimated from integrated ERP workflow class.
4. SAP-04 accepted throughput: 750 to 2000 commands per second per well-sized service landscape estimated; source: estimated from enterprise ERP batch/interactive mixture.
5. SAP-05 concurrent command slots: 225 to 600 slots at 300 ms service time for 750 to 2000 commands per second; source: Little's Law applied to estimate.
6. SAP-06 batch import: 25,000 to 100,000 PM object rows per minute estimated when import validation is parallelized; source: estimated from ERP migration workload class.
7. SAP-07 event publication lag p95: 1 to 5 seconds estimated when downstream accounting, inventory, or workflow hooks are active; source: estimated from integrated ERP event propagation.
8. SAP-08 audit evidence seal p95: 150 ms to 500 ms estimated where evidence is local and not cross-region; source: estimated from enterprise audit persistence.
9. SAP-09 policy decision p95: 30 ms to 120 ms estimated for role and authorization checks; source: estimated from enterprise authorization systems.
10. SAP-10 availability: 99.9 percent or higher is expected for enterprise PM operations; source: estimated enterprise operations expectation, not a SAP public raw figure.
11. SAP-11 asset ceiling: 1 million to 10 million equipment/technical-object records per enterprise landscape estimated; source: estimated from large-enterprise SAP PM usage class.
12. SAP-12 active work-order ceiling: 100,000 to 1 million active or recent work orders per enterprise landscape estimated; source: estimated from large plant operations.
13. SAP-13 mobile sync freshness: 30 seconds to 5 minutes estimated when SAP mobile or offline extensions are used; source: estimated from mobile EAM synchronization class.
14. SAP-14 reporting freshness: 1 to 15 minutes estimated for operational dashboards depending on extraction mode; source: estimated enterprise reporting class.
15. SAP-15 comparison note: SAP's public materials are functional rather than raw benchmark disclosures, so these numbers are estimation anchors only.

### 2.2 IBM Maximo numbers

1. MAX-01 latency p50: 120 ms to 220 ms estimated for tuned asset or work-order interactions; source: estimated from Maximo Manage enterprise EAM surface.
2. MAX-02 latency p95: 400 ms to 800 ms estimated for work order update, PM generation, or inventory reservation; source: estimated from enterprise EAM workflow class.
3. MAX-03 latency p99: 800 ms to 1400 ms estimated for integrated work-management flows; source: estimated from Maximo-style asset, inventory, and workflow integration.
4. MAX-04 accepted throughput: 1000 to 2500 commands per second per service cell or cluster estimated; source: estimated from large Maximo deployment class.
5. MAX-05 concurrent command slots: 300 to 750 slots at 300 ms service time for 1000 to 2500 commands per second; source: Little's Law applied to estimate.
6. MAX-06 batch import: 30,000 to 120,000 rows per minute estimated with staged validation; source: estimated from asset/work-order migration workloads.
7. MAX-07 event publication lag p95: 1 to 4 seconds estimated for integrated work, inventory, and reporting hooks; source: estimated enterprise EAM event class.
8. MAX-08 audit evidence seal p95: 120 ms to 450 ms estimated for local evidence persistence; source: estimated enterprise audit workload.
9. MAX-09 policy decision p95: 25 ms to 100 ms estimated; source: estimated enterprise access-control workload.
10. MAX-10 availability: 99.9 percent or higher expected for enterprise asset-management operations; source: estimated operational expectation.
11. MAX-11 asset ceiling: 1 million to 20 million assets/locations/meters estimated for a large estate; source: estimated from Maximo enterprise asset-management class.
12. MAX-12 active work-order ceiling: 250,000 to 2 million active or recent work-management records estimated; source: estimated from heavy EAM operations.
13. MAX-13 mobile sync freshness: 15 seconds to 2 minutes estimated for connected mobile flows, longer for offline replay; source: estimated from Maximo mobile field-service class.
14. MAX-14 scheduling freshness: 5 seconds to 60 seconds estimated for dispatcher-visible queue changes under normal load; source: estimated scheduling workflow class.
15. MAX-15 comparison note: Maximo public docs emphasize capability and deployment behavior; raw latency/RPS tables were not available in public docs used here.

### 2.3 UpKeep numbers

1. UPK-01 latency p50: 80 ms to 180 ms estimated for simple CMMS work-order and asset interactions; source: estimated from mobile-first SaaS CMMS workflow class.
2. UPK-02 latency p95: 250 ms to 600 ms estimated for work-order create/update and PM schedule operations; source: estimated from UpKeep CMMS workflow class.
3. UPK-03 latency p99: 600 ms to 1200 ms estimated for mobile work-order flows under integration load; source: estimated from mobile CMMS class.
4. UPK-04 accepted throughput: 500 to 1500 commands per second per tenant fleet or service partition estimated; source: estimated from SaaS CMMS interaction class.
5. UPK-05 concurrent command slots: 150 to 450 slots at 300 ms service time for 500 to 1500 commands per second; source: Little's Law applied to estimate.
6. UPK-06 batch import: 10,000 to 60,000 asset, part, or PM rows per minute estimated; source: estimated from SaaS CMMS migration workload.
7. UPK-07 event publication lag p95: 0.5 to 3 seconds estimated for connected mobile updates and notifications; source: estimated mobile SaaS event class.
8. UPK-08 audit evidence seal p95: 100 ms to 350 ms estimated for simple activity logs and attachments; source: estimated CMMS audit workload.
9. UPK-09 policy decision p95: 15 ms to 80 ms estimated for simple requester/technician/admin role checks; source: estimated SaaS RBAC class.
10. UPK-10 availability: 99.9 percent or higher expected for SaaS CMMS operations; source: estimated SaaS product expectation.
11. UPK-11 asset ceiling: 100,000 to 5 million assets per large multi-site tenant estimated; source: estimated CMMS SaaS scale class.
12. UPK-12 active work-order ceiling: 50,000 to 500,000 active or recent work orders per large tenant estimated; source: estimated CMMS workload class.
13. UPK-13 mobile sync freshness: 5 seconds to 60 seconds for connected devices estimated; source: estimated mobile-first CMMS behavior.
14. UPK-14 offline replay window: 15 minutes to 24 hours of local collection before replay estimated; source: estimated offline mobile workflow class.
15. UPK-15 comparison note: UpKeep public docs expose feature surfaces; raw p50/p95/p99/RPS benchmark tables were not available in public docs used here.

## 3. Oyatie Target Numbers

### 3.1 Single industry-leader target set

1. OYA-01 canonical command latency p50: less than or equal to 90 ms for lightweight accepted command acknowledgement.
2. OYA-02 canonical command latency p95: less than or equal to 300 ms.
3. OYA-03 canonical command latency p99: less than or equal to 350 ms.
4. OYA-04 canonical command throughput: 2500 accepted commands per second per service cell before horizontal scale-out.
5. OYA-05 canonical concurrent command slots: 750 slots per service cell at 300 ms p95 and 2500 commands per second.
6. OYA-06 canonical batch import: 120,000 validated rows per minute per service cell.
7. OYA-07 canonical event publication lag p95: less than or equal to 1 second.
8. OYA-08 canonical audit-chain seal latency p95: less than or equal to 120 ms.
9. OYA-09 canonical Cedar policy decision p95: less than or equal to 25 ms.
10. OYA-10 canonical ontology projection lag p95: less than or equal to 2 seconds.
11. OYA-11 canonical availability: greater than or equal to 99.9 percent, matching the OpenSLO target at `slos/plant-maintenance-availability.openslo.yaml:29`.
12. OYA-12 canonical accepted-over-received ratio: greater than or equal to 99.5 percent, matching throughput SLO target at `slos/plant-maintenance-throughput.openslo.yaml:29`.
13. OYA-13 canonical equipment-master success ratio: greater than or equal to 99.9 percent, matching `slos/equipment-master-success-rate.openslo.yaml:29`.
14. OYA-14 canonical asset ceiling: 10 million equipment or asset records per service cell.
15. OYA-15 canonical active work-order ceiling: 1 million active or recent work-order records per service cell.
16. OYA-16 canonical maintenance-plan ceiling: 5 million active schedule records per service cell.
17. OYA-17 canonical spare-reservation ceiling: 2 million active reservation records per service cell.
18. OYA-18 canonical technician-dispatch ceiling: 500,000 active dispatch records per service cell.
19. OYA-19 canonical downtime-window ceiling: 1 million recent downtime-window records per service cell.
20. OYA-20 canonical mobile connected sync freshness: less than or equal to 10 seconds once mobile contracts exist.
21. OYA-21 canonical offline replay acceptance: 24 hours of signed local collection once mobile offline contracts exist.
22. OYA-22 canonical import idempotency collision error rate: less than or equal to 0.01 percent.
23. OYA-23 canonical duplicate-command acceptance rate: 0 accepted duplicates for identical tenant and idempotency key.
24. OYA-24 canonical policy-deny visibility: dashboard update within 10 seconds of deny spike.
25. OYA-25 canonical regional failover RTO: less than or equal to 5 minutes for command acceptance.
26. OYA-26 canonical regional failover RPO: less than or equal to 30 seconds for accepted command evidence.
27. OYA-27 canonical batch replay lag warning: 300 seconds, matching PRD telemetry at `PRD.md:1090` and related sections.
28. OYA-28 canonical batch replay lag page: 900 seconds, matching PRD telemetry at `PRD.md:1090` and related sections.
29. OYA-29 canonical cost attribution completeness: 100 percent of accepted events carry tenant, deployment context, tenant class, bounded context, workflow run ref, and audit chain ref.
30. OYA-30 canonical unsupported-context rejection: 100 percent of commands naming unknown deployment context or tenant class are rejected before mutation.

### 3.2 Deployment-context overlays

1. `oyatie-public-cloud` overlay: keep canonical p50, p95, p99, and throughput targets through elastic service-cell scale-out.
2. `oyatie-public-cloud` overlay: target 2500 commands per second per service cell and horizontal sharding above that.
3. `oyatie-public-cloud` overlay: availability target remains at least 99.9 percent and can be contractually raised per paid tenant.
4. `guest-on-aws` overlay: keep canonical latency targets when tenant supplies required compute and managed-network prerequisites.
5. `guest-on-aws` overlay: throughput target is 2500 commands per second per service cell when provisioned above 8 vCPU equivalent.
6. `guest-on-aws` overlay: when constrained below that, throughput is capacity-planned by measured OCPU/vCPU and memory envelope.
7. `guest-on-oci` overlay: full paid or revenue-share deployments target canonical numbers when resources exceed the demo-trial profile.
8. `guest-on-oci` overlay: OCI Always Free profile caps demo-trial throughput to 300 accepted commands per second per service cell.
9. `guest-on-oci` overlay: OCI Always Free profile caps batch import to 15,000 rows per minute.
10. `guest-on-oci` overlay: OCI Always Free profile caps asset records to 250,000 per tenant unless paid or revenue-share scale-out is activated.
11. `guest-on-oci` overlay: OCI Always Free profile still targets p95 300 ms and p99 350 ms while under caps.
12. `on-prem` overlay: canonical targets apply only after certified OS, storage, network, and HLC evidence are verified.
13. `on-prem` overlay: throughput is capped by facility hardware, but product correctness and authorization semantics remain unchanged.
14. `on-prem` overlay: failover RTO/RPO can be facility-specific if the customer accepts documented constraints.
15. `colo` overlay: canonical latency applies inside the colo region when network peering and storage SLOs meet Oyatie prerequisites.
16. `colo` overlay: throughput target is 2500 commands per second per service cell with scale-out by additional cells.
17. `colo` overlay: batch import may be scheduled to protect production lines from bandwidth contention.
18. `oyatie-as-cloud-provider` overlay: canonical targets apply, with strongest control over placement, HLC, audit, and policy substrate.
19. `oyatie-as-cloud-provider` overlay: failover RTO target can be less than or equal to 2 minutes when owned substrate supports warm standbys.
20. `oyatie-as-cloud-provider` overlay: batch import target can exceed 120,000 rows per minute when dedicated cells are allocated.

### 3.3 Tenant-class overlays

1. `demo_trial` overlay: quality target remains industry-leader grade.
2. `demo_trial` overlay: usage caps are hard caps, not degraded correctness.
3. `demo_trial` overlay: recommended command cap is 300 accepted commands per second per tenant on OCI Always Free profile.
4. `demo_trial` overlay: recommended asset cap is 250,000 records per tenant on OCI Always Free profile.
5. `demo_trial` overlay: recommended active work-order cap is 25,000 records per tenant.
6. `demo_trial` overlay: recommended batch import cap is 15,000 rows per minute.
7. `demo_trial` overlay: no compliance packs by default.
8. `demo_trial` overlay: no BYOK by default.
9. `demo_trial` overlay: best-effort commercial SLO, while technical implementation targets still use canonical latency inside usage caps.
10. `paid` overlay: quality target remains industry-leader grade.
11. `paid` overlay: throughput scales with contracted resources and usage-based billing.
12. `paid` overlay: p95 300 ms and p99 350 ms remain canonical for lightweight commands.
13. `paid` overlay: compliance packs are allowed.
14. `paid` overlay: BYOK is allowed.
15. `paid` overlay: contractual SLO can bind availability and failover terms.
16. `paid` overlay: asset and work-order ceilings scale by additional service cells.
17. `per_seat` component: seat-counted users retain the same functional surface.
18. `per_seat` component: active-user counts feed cloud-billing.
19. `per_usage` component: usage scales with paid tenant consumption.
20. `per_usage` component: compliance packs are allowed when contractually included.
21. `per_usage` component: BYOK is allowed when contractually included.
22. `per_usage` component: finops events must carry usage meter identifiers.
23. All tenant classes: same data isolation requirement.
24. All tenant classes: same Cedar default-deny requirement.
25. All tenant classes: same audit-chain evidence integrity.
26. All tenant classes: same source-system provenance and idempotency guarantees.
27. All tenant classes: same typed command contract once implemented.
28. All tenant classes: same six-context product surface where usage caps permit.
29. All tenant classes: same incident response taxonomy.
30. All tenant classes: no retired activation fallback.

### 3.4 Target-versus-current evidence

1. Current source cannot yet measure OYA-01 through OYA-30 because HTTP handling is stubbed at `src/adapter/http.rs:65-67`.
2. Current tests cannot yet enforce OYA-01 through OYA-30 because key integration tests are ignored at `tests/integration.rs:59-77`.
3. Current contracts cannot yet enforce typed workload payloads because OpenAPI uses generic payload objects at `contracts/openapi-v1.yaml:132-176`.
4. Current deployment cannot yet enforce context overlays because the service has no six-context IaC directories.
5. Current OS posture cannot yet enforce OS overlays because `supported-oses.json` is absent.
6. Current tenant-class posture cannot yet enforce commercial overlays because no tenant-class fields exist.
7. Current capacity model provides useful math and now uses deployment_context plus tenant_class assumptions at `capacity-model.md:16-23`.
8. Current PRD provides useful worker-slot math at `PRD.md:1127-1130`.
9. Current SLO files provide useful target anchors but need deployment-context and tenant-class overlays.
10. Current audit status must remain "targets defined, runtime measurement unavailable" until implementation exists.

## 4. Comparison Narrative

1. Latency p50: Oyatie target 90 ms is ahead of estimated SAP, Maximo, and UpKeep simple-command p50 bands.
2. Latency p95: Oyatie target 300 ms is ahead of estimated SAP and Maximo bands and at the strong edge of UpKeep's estimated range.
3. Latency p99: Oyatie target 350 ms is ahead of estimated counterpart p99 bands, but not yet measured.
4. Throughput: Oyatie target 2500 commands per second per service cell is parity or ahead of estimated Maximo upper band and ahead of estimated UpKeep and SAP mid bands.
5. Concurrent slots: Oyatie target 750 slots is consistent with 2500 commands per second at 300 ms p95.
6. Batch import: Oyatie target 120,000 rows per minute is parity with Maximo upper estimate and ahead of most UpKeep/SAP estimated bands.
7. Event lag: Oyatie target 1 second is ahead of SAP and Maximo estimates and parity-to-ahead of UpKeep.
8. Audit seal latency: Oyatie target 120 ms is ahead of all estimated counterpart bands.
9. Policy-decision latency: Oyatie target 25 ms is ahead of most estimated counterpart authorization paths.
10. Availability: Oyatie 99.9 percent target is parity with baseline enterprise and SaaS expectations.
11. Asset ceiling: Oyatie 10 million records per service cell is parity with SAP/Maximo enterprise estimates and ahead of most UpKeep tenant estimates.
12. Work-order ceiling: Oyatie 1 million active or recent records per service cell is parity with SAP/Maximo high estimates and ahead of UpKeep estimated mid range.
13. Mobile connected sync: Oyatie 10 seconds target is ahead of estimated SAP and Maximo mobile sync bands and ahead of many UpKeep connected estimates.
14. Offline replay: Oyatie 24-hour target is parity with common mobile offline collection expectations, but the mobile contract is absent.
15. Deployment elasticity: Oyatie can be ahead only after OpenTofu modules and six-context overlays land.
16. Demo trial: Oyatie can be ahead by making OCI Always Free a real profile, but the path is absent.
17. Paid class: Oyatie can reach parity or ahead when contractual resources are provisioned.
18. Revenue-share class: Oyatie can be distinctive, but the service currently has no contract or event semantics for it.
19. Current implementation status: catch-up because the source is scaffolded.
20. Current documentation status: partial because SLO and capacity anchors exist but counterpart language still needs review.
21. Current contract status: catch-up because payloads are generic and tenant class is absent.
22. Current deployment status: catch-up because six contexts and OS matrix are missing.
23. Current observability status: parity-potential because metrics and dashboards are planned but not measured.
24. Current policy status: parity-potential because Cedar posture exists and must stay aligned to tenant_class prose.
25. Current plant-operator UX status: catch-up because UpKeep mobile and request workflows are absent.
26. Current enterprise-EAM status: catch-up because SAP/Maximo inventory, scheduling, and asset lifecycle depth are partial.
27. Headline number risk: all Oyatie targets are unmeasured until handlers, storage, policy, audit, and deployment substrate exist.
28. Headline number requirement: every future benchmark run must report deployment context, tenant class, OS, arch, database, storage class, cell count, and workload mix.
29. Headline number requirement: demo-trial caps must be tested separately from paid and revenue-share scale paths.
30. Headline number requirement: benchmark reports must not create or imply feature ladders.

## 5. Benchmark Acceptance Gate

1. Gate 1: source implements all six bounded contexts before measurement.
2. Gate 2: HTTP, event, and proto contracts use typed PM schemas.
3. Gate 3: Cedar fixture tests are unignored and passing.
4. Gate 4: idempotency and duplicate-command tests are passing.
5. Gate 5: audit-chain seal and ontology projection test doubles exist.
6. Gate 6: OpenTofu modules exist for all six deployment contexts.
7. Gate 7: OCI Always Free profile exists for demo-trial infrastructure.
8. Gate 8: `supported-oses.json` exists and lists required OS/arch support.
9. Gate 9: benchmark harness reports OS and arch.
10. Gate 10: benchmark harness reports deployment context.
11. Gate 11: benchmark harness reports tenant class.
12. Gate 12: benchmark harness reports warmup duration and sample count.
13. Gate 13: benchmark harness reports p50, p95, p99, max, and error rate.
14. Gate 14: benchmark harness reports throughput, queue depth, and worker slots.
15. Gate 15: benchmark harness reports batch import rows per minute.
16. Gate 16: benchmark harness reports policy-decision latency.
17. Gate 17: benchmark harness reports audit seal latency.
18. Gate 18: benchmark harness reports event publication lag.
19. Gate 19: benchmark harness reports ontology projection lag.
20. Gate 20: benchmark harness reports storage and network class.
21. Gate 21: benchmark harness reports exact commit, config, and fixture set.
22. Gate 22: benchmark harness reports whether test ran under demo-trial cap, paid scale, or revenue-share scale.
23. Gate 23: benchmark harness rejects old retired activation labels.
24. Gate 24: benchmark harness writes machine-readable output and human summary.
25. Gate 25: published claims distinguish measured Oyatie results from estimated counterpart anchors.

## 6. Immediate Target Summary

1. Immediate target p50: 90 ms.
2. Immediate target p95: 300 ms.
3. Immediate target p99: 350 ms.
4. Immediate target throughput: 2500 commands per second per service cell.
5. Immediate target availability: 99.9 percent or better.
6. Immediate target accepted-over-received ratio: 99.5 percent or better.
7. Immediate target batch import: 120,000 rows per minute per service cell.
8. Immediate target audit seal p95: 120 ms.
9. Immediate target policy decision p95: 25 ms.
10. Immediate target event lag p95: 1 second.
11. Immediate target ontology projection lag p95: 2 seconds.
12. Immediate target regional failover RTO: 5 minutes.
13. Immediate target regional failover RPO: 30 seconds.
14. Immediate target demo-trial cap on OCI Always Free profile: 300 commands per second.
15. Immediate target paid cap: contract-driven scale by service cell.
16. Immediate target revenue-share cap: contract-driven scale at cost or zero-margin substrate.
17. Immediate target retired activation usage: zero new rows, zero new labels, zero new authorization gates.
18. Immediate target measurement caveat: no target is claimable as measured until runtime and benchmark harness exist.
19. Immediate target counterpart caveat: SAP, IBM Maximo, and UpKeep rows are estimates, not vendor-published benchmark claims.
20. Immediate target next action: implement typed runtime and benchmark harness before any public performance claim.
