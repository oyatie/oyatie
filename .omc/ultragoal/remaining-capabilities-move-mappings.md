# Remaining-capabilities move-mapping catalog (front-loaded design sweep `weln02ufy`)

Leader-state design catalog. ONE section per remaining capability, produced by the parallel read-only
design sweep (15 agents, run wf_4d781719-bb4). Every future serial move dispatches by handing the executor
"read the `<capability>` section of this file" — zero design latency. cell + gateway have their own files
(`cell-move-mapping.md`, `gateway-move-mapping.md`); this catalog covers the other 15.

**Each move STILL follows the full protocol** (`strangler-move-playbook.md`): codemod (not hand-move),
commit ONE `specs/reorg/<cap>-move-plan.json`, contract interactions (a)-(e), born-accounting,
MATERIALIZE-LAST, full gate suite vs merge-base, independent review, governed merge, post-merge verify.
These mappings are DESIGN inputs — confirmed at move time by an in-worktree scout against the then-current dev tip
(crate set drifts as siblings move; the codemod targets the live tree).

## Recommended serial order (cleanest-first; violation-sources + huge last, de-risked by rename-aware engine)
flags(1) → marketplace(5) → compliance(7) → console(9) → comms(16) → k8s(17) → tenancy(17) → audit(18) →
data(22) → workflow(47) → iam(62) → **then violation-sources** network(7) → secrets(10) → billing(16) →
**then** intelligence(142, decompose into sub-batches). Rationale: rename-aware engine relabels violation
inversion edges on move, so violation-sources need not go first; size + needs-refinement push workflow/iam/intelligence late.

---

## flags (1 crate, clean, confidence HIGH)
absorbs oya/feature-flags (1 crate) + oya/oya-flags (crate-empty placeholder → drop/phase-2).
| old_crate | new_path | cargo | face |
|---|---|---|---|
| oya/feature-flags/crates/oya-flags | flags/core/server | flags-server | core |
- external_dependents: NONE. violation_edges: NONE.
- Single OFREP/gRPC/REST evaluation server; internal modules stay as modules (the ADR-0481 3-crate split was never built — aspirational).
- Move-time: oya/oya-flags placeholder + oya/feature-flags non-crate artifacts (catalog/contracts/policy/slos/...) → phase-2 #62.

## marketplace (5 crates, clean, confidence HIGH)
absorbs cloud/cloud-marketplace(3) + oya/developer-sdk(1) + oya/marketplace(1) + oya/plugin-app-store(crate-empty).
| old_crate | new_path | cargo | face |
|---|---|---|---|
| cloud/cloud-marketplace/crates/oya-cloud-marketplace-domain | marketplace/core/cloud-domain | marketplace-cloud-domain | core |
| cloud/cloud-marketplace/crates/oya-cloud-marketplace-kernel | marketplace/core/cloud-kernel | marketplace-cloud-kernel | core |
| cloud/cloud-marketplace/crates/oya-saas-plugin-marketplace-kernel | marketplace/core/plugin-kernel | marketplace-plugin-kernel | core |
| oya/developer-sdk/crates/oya-dev-cli | marketplace/facade/dev-cli | marketplace-dev-cli | facade |
| oya/marketplace/crates/oya-marketplace-doc-set-scaffold | marketplace/core/doc-set-scaffold | marketplace-doc-set-scaffold | core |
- external_dependents: cloud-billing/oya-saas-bench-app → plugin-kernel; oya/application/oya-saas-plugin-app → plugin-kernel.
- violation_edges: NONE.
- ⚠ oya-dev-cli = the `oya` gate-runner CLI, retirement-marked (cli_surface_policy), VERY wide leaf consumer (~90 governance/check libs). Reviewers may argue it's a governance/CI tool not a marketplace SDK surface — membership lint adjudicates. Confirm home-then-retire vs skip.

## compliance (7 crates, clean, confidence HIGH)
absorbs oya/compliance(7) + oya/governance(crate-empty — its check-* crates live in governance/ meta-root, NOT here).
| old_crate | new_path | cargo | face |
|---|---|---|---|
| oya/compliance/crates/oya-dlp-domain | compliance/core/dlp | compliance-dlp | core |
| oya/compliance/crates/oya-dsr-domain | compliance/core/dsr | compliance-dsr | core |
| oya/compliance/crates/oya-dsr-usecase | compliance/ports/dsr-usecase | compliance-dsr-usecase | ports |
| oya/compliance/crates/oya-ediscovery-domain | compliance/core/ediscovery | compliance-ediscovery | core |
| oya/compliance/crates/oya-retention-domain | compliance/core/retention | compliance-retention | core |
| oya/compliance/crates/oya-retention-dsr-domain | compliance/core/retention-dsr | compliance-retention-dsr | core |
| oya/compliance/crates/oya-trust-portal-domain | compliance/core/trust-portal | compliance-trust-portal | core |
- external_dependents: NONE. violation_edges: NONE.
- ⚠ core/trust-portal → cloud-network oya-residency-domain (cross-cap; relabels to network-residency when network moves). All cross-cap deps → libs/oya-data-boundary-kernel.
- ⚠ do NOT absorb oya/governance check-* CI-fitness crates here (route to governance/ meta-root).

## console (9 crates, clean, confidence HIGH)
absorbs oya/ops(9 crates) + oya/app-shell-frontend(TS, crate-empty) + oya/ops-dashboard-control-center(crate-empty). Two product cells: docs-portal(4) + workspace-shell(5).
| old_crate | new_path | cargo | face |
|---|---|---|---|
| oya-ops-docs-portal-kernel | console/ports/docs-portal | console-docs-kernel | ports |
| oya-ops-docs-portal-usecase | console/core/docs-portal | console-docs-usecase | core |
| oya-ops-docs-portal-adapter | console/adapters/docs-portal | console-docs-adapter | adapters |
| oya-ops-docs-portal-rest | console/facade/docs-portal-rest | console-docs-rest | facade |
| oya-ops-workspace-shell-kernel | console/ports/workspace-shell | console-workspace-kernel | ports |
| oya-ops-workspace-shell-usecase | console/core/workspace-shell | console-workspace-usecase | core |
| oya-ops-workspace-shell-adapter | console/adapters/workspace-shell | console-workspace-adapter | adapters |
| oya-ops-workspace-shell-rest | console/facade/workspace-shell-rest | console-workspace-rest | facade |
| oya-ops-workspace-shell-app | console/facade/workspace-shell-app | console-workspace-app | facade |
- external_dependents: NONE. violation_edges: NONE. http substrate consumed from libs/ (router/middleware/runtime-hyper). Per-cell namespacing (docs-/workspace-) avoids leaf collisions.

## comms (16 crates, clean, confidence HIGH)
absorbs oya/mail(7) + oya/messenger(7) + oya/meet(1) + oya/contact-center(1) + oya/comms-email(crate-empty) + oya/emergency(crate-empty). All product-side; faces split internally by clean-arch role.
| old_crate | new_path | cargo | face |
|---|---|---|---|
| oya/mail/.../oya-mail-domain | comms/core/mail-domain | comms-mail-domain | core |
| oya/mail/.../oya-mail-mailbox-store-usecase | comms/core/mail-mailbox-usecase | comms-mail-mailbox-usecase | core |
| oya/mail/.../oya-mail-mailbox-store-app | comms/core/mail-mailbox-app | comms-mail-mailbox-app | core |
| oya/mail/.../oya-mail-mailbox-store-api | comms/ports/mail-mailbox-api | comms-mail-mailbox-api | ports |
| oya/mail/.../oya-mail-mailbox-store-adapter-postgres | comms/adapters/mail-mailbox-postgres | comms-mail-mailbox-postgres | adapters |
| oya/mail/.../oya-mail-mailbox-store-grpc | comms/facade/mail-mailbox-grpc | comms-mail-mailbox-grpc | facade |
| oya/mail/.../oya-mail-mailbox-store-rest | comms/facade/mail-mailbox-rest | comms-mail-mailbox-rest | facade |
| oya/messenger/.../oya-messenger-domain | comms/core/messenger-domain | comms-messenger-domain | core |
| oya/messenger/.../oya-messenger-message-stream-usecase | comms/core/messenger-stream-usecase | comms-messenger-stream-usecase | core |
| oya/messenger/.../oya-messenger-app | comms/core/messenger-stream-app | comms-messenger-stream-app | core |
| oya/messenger/.../oya-messenger-message-stream-api | comms/ports/messenger-stream-api | comms-messenger-stream-api | ports |
| oya/messenger/.../oya-messenger-message-stream-adapter-postgres | comms/adapters/messenger-stream-postgres | comms-messenger-stream-postgres | adapters |
| oya/messenger/.../oya-messenger-message-stream-grpc | comms/facade/messenger-stream-grpc | comms-messenger-stream-grpc | facade |
| oya/messenger/.../oya-messenger-message-stream-rest | comms/facade/messenger-stream-rest | comms-messenger-stream-rest | facade |
| oya/meet/.../oya-meet-domain | comms/core/meet-domain | comms-meet-domain | core |
| oya/contact-center/.../oya-contact-center-voice-routing-app | comms/facade/contact-center-voice-routing | comms-contact-center-voice-routing | facade |
- external_dependents: NONE (all external deps → libs/ shared kernels). violation_edges: NONE.
- Move-time: comms-email + emergency crate-empty (pre-reserve leaf prefixes). NOTE emergency = healthcare-ED domain — confirm it belongs in comms vs a healthcare cap before its crates land.

## k8s (17 crates, clean, confidence HIGH)
absorbs 4 managed-k8s-* dirs (cluster-lifecycle/control-plane-host/sla-observability/tenant-quota) + cloud/cloud-k8s(crate-empty, docs/IaC only). SOLD managed-k8s product; faces by clean-arch ring.
| old_crate | new_path | cargo | face |
|---|---|---|---|
| managed-k8s-cluster-lifecycle/...-kernel | k8s/core/cluster-lifecycle | k8s-cluster-lifecycle-kernel | core |
| managed-k8s-cluster-lifecycle/...-api | k8s/ports/cluster-lifecycle | k8s-cluster-lifecycle-api | ports |
| managed-k8s-cluster-lifecycle/...-app | k8s/facade/cluster-lifecycle | k8s-cluster-lifecycle-app | facade |
| managed-k8s-control-plane-host/...-kernel | k8s/core/control-plane-host | k8s-control-plane-host-kernel | core |
| managed-k8s-control-plane-host/...-api | k8s/ports/control-plane-host | k8s-control-plane-host-api | ports |
| managed-k8s-control-plane-host/...-adapter-inmemory | k8s/adapters/control-plane-host-inmemory | k8s-control-plane-host-adapter-inmemory | adapters |
| managed-k8s-control-plane-host/...-adapter-capi | k8s/adapters/control-plane-host-capi | k8s-control-plane-host-adapter-capi | adapters |
| managed-k8s-control-plane-host/...-app | k8s/facade/control-plane-host | k8s-control-plane-host-app | facade |
| managed-k8s-sla-observability/...-kernel | k8s/core/sla-observability | k8s-sla-observability-kernel | core |
| managed-k8s-sla-observability/...-api | k8s/ports/sla-observability | k8s-sla-observability-api | ports |
| managed-k8s-sla-observability/...-adapter-inmemory | k8s/adapters/sla-observability-inmemory | k8s-sla-observability-adapter-inmemory | adapters |
| managed-k8s-sla-observability/...-app | k8s/facade/sla-observability | k8s-sla-observability-app | facade |
| managed-k8s-tenant-quota/...-kernel | k8s/core/tenant-quota | k8s-tenant-quota-kernel | core |
| managed-k8s-tenant-quota/...-api | k8s/ports/tenant-quota | k8s-tenant-quota-api | ports |
| managed-k8s-tenant-quota/...-adapter-inmemory | k8s/adapters/tenant-quota-inmemory | k8s-tenant-quota-adapter-inmemory | adapters |
| managed-k8s-tenant-quota/...-adapter-cedar | k8s/adapters/tenant-quota-cedar | k8s-tenant-quota-adapter-cedar | adapters |
| managed-k8s-tenant-quota/...-app | k8s/facade/tenant-quota | k8s-tenant-quota-app | facade |
- external_dependents: NONE. violation_edges: NONE.
- ⚠ tenant-quota-adapter-cedar has OUTBOUND dep on oya/identity (→ iam/adapters/workload-authz-cedar + iam/core/workload-domain) — rename-aware engine rewrites once iam moves; if iam not yet moved, points at live oya/identity path.

## tenancy (17 crates, cloud SUBSTRATE → all core/ports/adapters, confidence HIGH)
absorbs cloud/tenancy. tier=substrate; NO facade (tenant mgmt IS the substrate).
| old_crate | new_path | cargo | face |
|---|---|---|---|
| oya-tenancy-kernel | tenancy/core/kernel | tenancy-kernel | core |
| oya-tenancy-domain | tenancy/core/domain | tenancy-domain | core |
| oya-tenancy-cell-assignment-kernel | tenancy/core/cell-assignment | tenancy-cell-assignment | core |
| oya-tenancy-dsr-cascade-kernel | tenancy/core/dsr-cascade | tenancy-dsr-cascade | core |
| oya-tenancy-isolation-policy-kernel | tenancy/core/isolation-policy | tenancy-isolation-policy | core |
| oya-tenancy-lifecycle-locks-kernel | tenancy/core/lifecycle-locks | tenancy-lifecycle-locks | core |
| oya-tenancy-sub-scope-registry-kernel | tenancy/core/sub-scope-registry | tenancy-sub-scope-registry | core |
| oya-tenancy-kyb-kyc-verifier-domain | tenancy/core/kyb-kyc-verifier | tenancy-kyb-kyc-verifier | core |
| oya-tenancy-tenant-lifecycle-domain | tenancy/core/tenant-lifecycle-domain | tenancy-tenant-lifecycle-domain | core |
| oya-tenancy-tenant-lifecycle-kernel | tenancy/core/tenant-lifecycle-kernel | tenancy-tenant-lifecycle-kernel | core |
| oya-tenancy-tenant-lifecycle-usecase | tenancy/core/tenant-lifecycle-usecase | tenancy-tenant-lifecycle-usecase | core |
| oya-tenancy-dr-pairing-usecase | tenancy/core/dr-pairing | tenancy-dr-pairing | core |
| oya-tenancy-per-tenant-quota-usecase | tenancy/core/per-tenant-quota | tenancy-per-tenant-quota | core |
| oya-tenancy-reserved-namespace-usecase | tenancy/core/reserved-namespace | tenancy-reserved-namespace | core |
| oya-tenancy-api | tenancy/ports/api | tenancy-api | ports |
| oya-tenant-cli | tenancy/ports/cli | tenancy-cli | ports (bin name `oya-tenant` preserved) |
| oya-tenancy-data-residency-enforcer-adapter | tenancy/adapters/data-residency-enforcer | tenancy-data-residency-enforcer | adapters |
- external_dependents: oya/application/oya-application-app → tenancy-domain; libs/oya-http-tenant-middleware-infrastructure → tenancy-kernel.
- ⚠ tenancy-api + tenancy-domain depend on oya-residency-domain (→ network-residency). tenant-cli retirement-marked.

## audit (18 crates, FOUNDATIONAL SUBSTRATE → core/ports/adapters, confidence HIGH)
absorbs oya/audit-chain. 6 external capabilities consume it. NO facade. De-dup: `chain` doubling dropped (cargo prefix `audit` not `audit-chain`).
| old_crate | new_path | cargo | face |
|---|---|---|---|
| oya-audit-chain-domain | audit/core/chain | audit-chain | core |
| oya-audit-chain-usecase | audit/core/usecase | audit-usecase | core |
| oya-audit-chain-emission-kernel | audit/ports/emission | audit-emission-kernel | ports |
| oya-audit-chain-emission-api | audit/ports/emission-api | audit-emission-api | ports |
| oya-audit-chain-emission-domain | audit/core/emission | audit-emission | core |
| oya-audit-chain-query-kernel | audit/ports/query | audit-query-kernel | ports |
| oya-audit-chain-query-api | audit/ports/query-api | audit-query-api | ports |
| oya-audit-chain-query-domain | audit/core/query | audit-query | core |
| oya-audit-chain-retention-cascade-kernel | audit/ports/retention | audit-retention-kernel | ports |
| oya-audit-chain-retention-cascade-api | audit/ports/retention-api | audit-retention-api | ports |
| oya-audit-chain-retention-cascade-domain | audit/core/retention | audit-retention | core |
| oya-audit-chain-sealing-kernel | audit/ports/sealing | audit-sealing-kernel | ports |
| oya-audit-chain-sealing-api | audit/ports/sealing-api | audit-sealing-api | ports |
| oya-audit-chain-sealing-domain | audit/core/sealing | audit-sealing | core |
| oya-audit-chain-verification-kernel | audit/ports/verification | audit-verification-kernel | ports |
| oya-audit-chain-verification-api | audit/ports/verification-api | audit-verification-api | ports |
| oya-audit-chain-verification-domain | audit/core/verification | audit-verification | core |
| oya-audit-chain-file-adapter | audit/adapters/file | audit-file-adapter | adapters |
- external_dependents: observability/core/aggregate + /api → audit-chain; oya/intelligence cloud-mutation-domain → audit-chain; oya-dev-cli → audit-chain + audit-file-adapter; oya/application app → audit-chain; oya/tenant-rbac audit-chain-emission → audit-emission-api + audit-emission-kernel.
- ⚠ audit/core/usecase → messaging/core/domain (verify messaging stays lower DAG tier, no inversion). 15 non-crate dirs → phase-2.
- ⚠ registry-granularity Q: the 5 *-api DTO crates placed in ports/ — if registry treats request/response DTOs as core, they move to core/.

## data (22 crates, mixed substrate/product by dep direction, confidence HIGH, 1 violation edge)
absorbs cloud/cloud-data(2) + oya/ontology(6) + oya/search(8) + oya/analytics(5) + oya/data-pipeline(1) + oya/data-warehouse(1). Plus phase-2 libs/oya-data-* + OLAP libs (see open Qs).
| old_crate | new_path | cargo | face |
|---|---|---|---|
| cloud/cloud-data/.../oya-cloud-data-kernel | data/core/cloud-kernel | data-cloud-kernel | core |
| cloud/cloud-data/.../oya-cloud-data-domain | data/core/cloud-domain | data-cloud-domain | core |
| oya/ontology/.../oya-ontology-kernel | data/core/ontology-kernel | data-ontology-kernel | core |
| oya/ontology/.../oya-ontology-domain | data/core/ontology-domain | data-ontology-domain | core |
| oya/ontology/.../oya-ontology-query-engine-domain | data/core/ontology-query-engine-domain | data-ontology-query-engine-domain | core |
| oya/ontology/.../oya-ontology-query-engine-usecase | data/core/ontology-query-engine-usecase | data-ontology-query-engine-usecase | core |
| oya/ontology/.../oya-ontology-api | data/ports/ontology-api | data-ontology-api | ports |
| oya/ontology/.../oya-resolve-scorecards-app | data/facade/ontology-scorecards-resolver | data-ontology-scorecards-resolver | facade |
| oya/search/.../oya-search-crawler-domain | data/core/search-crawler | data-search-crawler | core |
| oya/search/.../oya-search-parser-domain | data/core/search-parser | data-search-parser | core |
| oya/search/.../oya-search-index-inverted-domain | data/core/search-index-inverted | data-search-index-inverted | core |
| oya/search/.../oya-search-index-vector-domain | data/core/search-index-vector | data-search-index-vector | core |
| oya/search/.../oya-search-query-domain | data/core/search-query | data-search-query | core |
| oya/search/.../oya-search-rank-domain | data/core/search-rank | data-search-rank | core |
| oya/search/.../oya-search-serp-domain | data/core/search-serp | data-search-serp | core |
| oya/search/.../oya-search-rag-domain | data/core/search-rag | data-search-rag | core |
| oya/analytics/.../oya-analytics-domain | data/core/analytics-domain | data-analytics-domain | core |
| oya/analytics/.../oya-analytics-usecase | data/core/analytics-usecase | data-analytics-usecase | core |
| oya/analytics/.../oya-analytics-api | data/ports/analytics-api | data-analytics-api | ports |
| oya/analytics/.../oya-analytics-app | data/facade/analytics-app | data-analytics-app | facade |
| oya/analytics/.../oya-analytics-tenant-bootstrap-app | data/facade/analytics-tenant-bootstrap | data-analytics-tenant-bootstrap | facade |
| oya/data-pipeline/.../oya-data-pipeline-lineage-replay-service | data/facade/pipeline-lineage-replay-service | data-pipeline-lineage-replay-service | facade |
| oya/data-warehouse/.../oya-data-warehouse-tenant-olap-service | data/facade/warehouse-tenant-olap-service | data-warehouse-tenant-olap-service | facade |
- external_dependents: oya/application/oya-application-app → data-ontology-domain.
- violation_edges: policy-engine→ontology historical inversion (DAG L548); inverted runtime edge relabels on move.
- ⚠ open: OLAP libs (libs/oya-shared-olap-client-kernel + clickhouse-adapter) + libs/oya-data-* (4) are phase-2 strangler candidates into data/{ports,adapters} — confirm at move time (currently in libs/ frozen baseline).

## workflow (47 crates, engine=substrate / products=facade, confidence NEEDS-MOVE-TIME-REFINEMENT)
absorbs oya/{workflow-engine,workflow-studio,tasks,forms}. Engine = 4 hexagonal sub-domains (event-bus/execution/state-machine/trigger), each kernel<-domain<-usecase=core, api/rest/sdk/worker/app=ports, adapter+broker-adapters=adapters. Products (saas-workflow/studio/tasks/forms)=facade.
ENGINE (event-bus): -kernel/-domain/-usecase → workflow/core/event-bus-{kernel,domain,usecase} (core); -api/-rest/-sdk/-worker/-app → workflow/ports/event-bus-* (ports); -adapter → workflow/adapters/event-bus-generic, -adapter-{kafka,nats,postgres,pulsar,redpanda,valkey} → workflow/adapters/event-bus-{broker} (adapters).
ENGINE (execution-engine→"execution"): kernel/domain/usecase→core/execution-*; api/rest/sdk/worker/app→ports/execution-*; adapter+adapter-postgres→adapters/execution-{generic,postgres}.
ENGINE (state-machine, partial stack): kernel/domain/usecase→core; api→ports; adapter+adapter-postgres→adapters. (no rest/sdk/worker/app)
ENGINE (trigger-orchestrator→"trigger"): kernel/domain/usecase→core; api/rest/sdk/worker/app→ports.
PRODUCTS (facade): oya-saas-workflow-{kernel,domain,app}→workflow/facade/saas-*; oya-workflow-studio-{dsl-emitter,dsl-loader,policy-preview,visual-canvas}→workflow/facade/studio-*; oya-tasks-domain→workflow/facade/tasks-domain; oya-forms-domain→workflow/facade/forms-domain.
- cargo grammar: workflow-<leaf> (e.g. workflow-execution-domain, workflow-event-bus-adapter-kafka, workflow-saas-kernel, workflow-studio-dsl-emitter). All 47 distinct.
- external_dependents: cloud-billing/oya-saas-bench-app → workflow-saas-{kernel,domain,app}; oya/application/oya-workspace-forms-api → workflow-forms-domain.
- ⚠ open: engine *-app crates (event-bus/execution/trigger) placed in ports/ as composition-wiring — confirm ports vs a wiring sub-fold. event-bus broker substrate may belong under messaging cap (cross-check). tasks/forms cross-cap deps (intelligence/data) — confirm home. RE-VERIFY dep direction at move time.

## iam (62 crates, iam-pattern substrate/product, confidence MEDIUM — bulk = 38 tenant-rbac crates)
absorbs cloud/cloud-iam(8) + oya/policy(2) + oya/identity(13) + oya/oya-authn-device-firmware(1) + oya/tenant-rbac(38) + crate-empty oya/oya-identity, oya/consent-graph. cloud-iam-domain DEPENDS ON identity-domain (identity kernels are core).
KEY crates: cloud-iam-domain→iam/core/cloud-domain; cloud-iam-api→iam/ports/cloud-api; cloud-iam-app→iam/core/cloud-app; cloud-iam-adapter-{oci,selfhosted}→iam/adapters/cloud-{oci,selfhosted}; cloud-iam-pdp-kernel→iam/core/pdp-kernel; cloud-iam-pdp-app→iam/core/pdp-app; cloud-iam-pdp-bundle-file-adapter→iam/adapters/pdp-bundle-file.
policy: oya-policy-cedar-domain→iam/core/policy-cedar-domain; oya-policy-cedar-api→iam/ports/policy-cedar-api.
identity: identity-domain→iam/core/identity-domain; identity-usecase→iam/core/identity-usecase; identity-api→iam/ports/identity-api; identity-oidc-issuer-kernel→iam/core/identity-oidc-issuer-kernel; workload-domain→iam/core/workload-domain; workload-svid-kernel→iam/core/workload-svid-kernel; workload-api→iam/ports/workload-api; workload-app→iam/core/workload-app; workload-{oidc,authz-cedar,svid-trustd}-adapter→iam/adapters/workload-{oidc,authz-cedar,svid-trustd}; workload-rest→iam/facade/workload-rest; oya-identity(bin)→iam/facade/identity-service (cargo iam-identity-service).
authn: oya-authn-device-firmware→iam/core/authn-device-firmware.
tenant-rbac (38): domain/usecase/auth-app/idp-verification(identity-provider-verification)/admission-policy(tenant-admission-policy)/workload-manifest(tenant-workload-manifest)→core; api + all 12 *-contract crates (autoscaling/availability/cost-allocation/egress/image-provenance/residency/resource-quota/secret-boundary/workload-identity + postgres-rls-{write,transaction}-contract)→ports; storage-adapter-inmemory/workflow-adapter-inmemory/postgres-rls-storage/audit-chain-emission→adapters; app + ALL *-runtime-evidence + listener-gateway + local-{inmemory-harness,runtime-composition} + cloud-deployment-{manifest,evidence} + cloud-readiness-gate + disbursement/statutory-filing/slo/erp-parity-map evidence→facade. De-dup: drop `tenant-rbac-tenant-` doubling (cargo iam-tenant-rbac-residency-contract etc).
- external_dependents: k8s tenant-quota-adapter-cedar → workload-authz-cedar + workload-domain; compute/core/domain → cloud-domain; observability/core/aggregate → cloud-domain; oya/application/oya-application-app → identity-domain + policy-cedar-domain.
- ⚠ open: ports→core edges from *-contract crates (verify layering DAG allows, else reclassify core). authz-cedar-adapter reused cross-cap by k8s (promote to libs/?). local-*-harness pull accounting/hr/payroll (test harnesses — confirm iam/facade vs integration cap). consent-graph cedar/SLOs → phase-2 home under iam/.

---
# VIOLATION-SOURCE capabilities (move LATER; inversion edges relabel on move via rename-aware engine — commit move-plan)

## network (7 crates, cloud SUBSTRATE → core/ports/adapters, confidence HIGH; DOMINANT INVERSION HUB)
absorbs cloud/cloud-network(6) + cloud/cloud-network-dns(1). residency-domain = inversion TARGET (25 inbound dependents).
| old_crate | new_path | cargo | face |
|---|---|---|---|
| cloud/cloud-network/.../oya-cloud-network-domain | network/core/domain | network-domain | core |
| cloud/cloud-network/.../oya-residency-domain | network/core/residency | network-residency | core |
| cloud/cloud-network/.../oya-cloud-network-vpc-api | network/ports/vpc | network-vpc | ports |
| cloud/cloud-network/.../oya-cloud-network-lb-api | network/ports/lb | network-lb | ports |
| cloud/cloud-network-dns/.../oya-cloud-network-dns-api | network/ports/dns | network-dns | ports |
| cloud/cloud-network/.../oya-cloud-network-adapter-oci | network/adapters/oci | network-oci | adapters |
| cloud/cloud-network/.../oya-cloud-network-adapter-selfhosted | network/adapters/selfhosted | network-selfhosted | adapters |
- ⚠ 25 external_dependents of oya-residency-domain → network-residency (relabel oya_residency_domain → network_residency): cloud-cell(cell-domain, region-api, region-domain, regional-pack-api, regional-pack-domain), cloud-data-domain, cloud-kms(api, domain, operator-k8s-adapter), tenancy(api, domain), compute(adapters/aws, adapters/oci, core/domain, core/resource, facade/functions, facade/k8s, facade/vm), observability(core/aggregate, core/api), oya/application/oya-application-app, oya/compliance/oya-trust-portal-domain, storage(core/domain, ports/block-api, ports/object-api). Plus compute/core/domain → network-domain.
- ⚠ residency placement Q: it's a platform-wide ADR-0049 kernel (zero network-specific deps) — kept at network/core/residency per registry; flag for founder if it should be its own cross-cutting cap. THIS IS THE BIGGEST BLAST RADIUS — review every one of the 25 rewrites.

## secrets (10 crates, two cloud substrates → core/ports/adapters+1 facade, confidence HIGH; violation-source)
absorbs cloud/cloud-kms(8) + cloud/cloud-secrets(2). KMS=crypto root, secrets=SecretProvider.
| old_crate | new_path | cargo | face |
|---|---|---|---|
| cloud/cloud-kms/.../oya-cloud-kms-domain | secrets/core/kms-domain | secrets-kms-domain | core |
| cloud/cloud-kms/.../oya-cloud-kms-enclave-kernel | secrets/core/kms-enclave | secrets-kms-enclave | core |
| cloud/cloud-kms/.../oya-cloud-kms-operator-kernel | secrets/core/kms-operator-kernel | secrets-kms-operator-kernel | core |
| cloud/cloud-kms/.../oya-cloud-kms-api | secrets/ports/kms-api | secrets-kms-api | ports |
| cloud/cloud-kms/.../oya-cloud-kms-adapter-oci | secrets/adapters/kms-oci | secrets-kms-oci | adapters |
| cloud/cloud-kms/.../oya-cloud-kms-adapter-openbao | secrets/adapters/kms-openbao | secrets-kms-openbao | adapters |
| cloud/cloud-kms/.../oya-cloud-kms-operator-k8s-adapter | secrets/adapters/kms-operator-k8s | secrets-kms-operator-k8s | adapters |
| cloud/cloud-kms/.../oya-cloud-kms-operator-app | secrets/facade/kms-operator-app | secrets-kms-operator-app | facade |
| cloud/cloud-secrets/.../oya-secrets-domain | secrets/core/domain | secrets-domain | core |
| cloud/cloud-secrets/.../oya-secrets-file-adapter | secrets/adapters/file | secrets-file | adapters |
- external_dependents: cloud-data-domain → kms-domain; storage(core/domain, ports/object-api) → kms-domain; oya/application app → secrets-domain; oya-dev-cli → secrets-domain + secrets-file; oya/intelligence adapter-domain → secrets-domain.
- violation_edges (relabel on move): kms-domain → residency(network), region(cell), compute-resource; kms-api → residency(network-dev-dep), region(cell); kms-operator-k8s → residency(network).
- ⚠ open: secrets→compute core edge — verify acyclicity-clean or new inversion. operator-app facade-vs-core run-face Q.

## billing (16 crates, substrate=core / products=facade, confidence HIGH; violation-source via saas-bench)
absorbs cloud/cloud-billing + cloud/cloud-finops + accounting-journal + metering + service crates + 4 crate-empty (oya/oya-billing,oya-meter,oya-cost,finops-portal).
| old_crate | new_path | cargo | face |
|---|---|---|---|
| oya-cloud-billing-kernel | billing/core/billing-kernel | billing-kernel | core |
| oya-cloud-billing-domain | billing/core/billing | billing-domain | core |
| oya-metering-domain | billing/core/metering | billing-metering | core |
| oya-cloud-finops-kernel | billing/core/finops-kernel | billing-finops-kernel | core |
| oya-cloud-finops-domain | billing/core/finops | billing-finops | core |
| oya-accounting-journal-domain | billing/core/accounting-journal | billing-accounting-journal | core |
| oya-accounting-journal-app | billing/core/accounting-app | billing-accounting-app | core |
| oya-cloud-finops-api | billing/ports/finops-api | billing-finops-api | ports |
| oya-accounting-journal-api | billing/ports/accounting-api | billing-accounting-api | ports |
| oya-cloud-billing-tax-app | billing/ports/tax-api | billing-tax-api | ports |
| oya-accounting-journal-infrastructure | billing/adapters/accounting-http | billing-accounting-http-adapter | adapters |
| oya-accounting-journal-storage-adapter-inmemory | billing/adapters/accounting-storage-inmemory | billing-accounting-storage-inmemory-adapter | adapters |
| oya-billing | billing/facade/billing-service | billing-service | facade |
| oya-meter | billing/facade/meter-service | billing-meter-service | facade |
| oya-cost | billing/facade/cost-service | billing-cost-service | facade |
| oya-saas-bench-app | billing/facade/saas-bench | billing-saas-bench | facade |
- external_dependents: cloud-marketplace-domain → billing-domain + metering; cloud-capacity-domain → billing-domain + metering; tenant-rbac-local-inmemory-harness → accounting-journal-{app,domain,storage-adapter-inmemory}; tenant-rbac-local-runtime-composition → accounting-journal-infrastructure.
- violation_edges (relabel on move): saas-bench → saas-plugin-app(oya/application), saas-workflow-{kernel,domain,app}(workflow), saas-plugin-marketplace-kernel(marketplace). Final relabel depends on those caps' destinations.

## intelligence (142 crates, DECOMPOSE INTO SUB-BATCHES, confidence NEEDS-MOVE-TIME-REFINEMENT; violation-source + brand-debt)
absorbs cloud/cloud-intelligence(16) + oya/intelligence(126) + oya/detection(crate-empty) + oya/intelligence/_legacy-foundry(brand-debt #55, de-brand "Foundry"→"intelligence" in lockstep).
**DO NOT move as one PR.** Suggested sub-batches: (a) cloud-intelligence oauth-pool service[16]; (b) provider-adapters family anthropic/openai/gemini api+subscription kernels+adapters[~14]; (c) account/supervisor/runtime cluster[~18]; (d) hexagonal feature stacks assist-draft/attribution/audit-tap/context-aware-retrieval/eval/credential-resolver[~36]; (e) dashboard/api-transport/capability-registry/autonomy-ceiling/rag/dispatch[~30]; (f) fitness/dev-cli-consumed domains cargo-prefix/openapi/mdbook/api-semver/catalog/architecture-map[~8]; (g) collab/document-format trio + shuffle-sharding + codeview-cli[~6].
Grammar: intelligence-<leaf>. cloud-intelligence → intelligence/{core,ports,adapters,facade}/oauth-pool-*. oya substrate kernels/domains→core; api/rest/sse/ws/graphql transport + *-api→ports; *-adapter/account-adapters→adapters; *-app/*-worker/*-usecase/*-cli→facade. FULL 142-row table in design sweep output (`weln02ufy.output` lines 1946-2799) — read it at move time per sub-batch.
- external_dependents (core kernels widely consumed): oya-application-app → 8 domains (adapter/bypass/capability/evidence/mcp-gateway/policy/run/step); oya-dev-cli → 12 (api-semver/architecture-map-app/bypass/cargo-prefix/catalog/evidence-file/mdbook/openapi/run+run-file/step+step-file); oya-tasks-domain → capability-domain; oya-{slides,sheets,sites,notes}-domain → collab-runtime-domain; oya-slides also document-format-domain; oya-governance-tos-policy-kernel + upstream-api-drift-kernel → account-kernel(+provider-pool-kernel); oya-check-claim-ceiling → catalog-domain; oya-cloud-ci-cargo-prefix-app → cargo-prefix-domain.
- violation_edges: intelligence-api → oya-application-app (UPWARD re-export); intelligence-backbone-workload-live-app → oya-community-* (UPWARD harness).
- ⚠ open: many *-app crates are composition LIBS not bins (real bins: cloud-intelligence-app, codeview-cli, pr-review-dispatcher-app, provider-pool-app, subagent-runtime-app, supervisor-app, backbone-workload-live-app) — confirm app-without-bin → ports vs facade. COLLAB CREEP: collab-crdt-portability/collab-runtime/document-format consumed by slides/sheets/sites/notes — may belong to a collab/docs cap, NOT intelligence (flag registry granularity). shuffle-sharding may belong to cell/. _legacy-foundry de-brand mandatory.
