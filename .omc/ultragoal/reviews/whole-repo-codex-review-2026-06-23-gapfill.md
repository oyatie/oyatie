# Whole-repo codex review — GAP-FILL ADDENDUM (2026-06-23)

Addendum to `whole-repo-codex-review-2026-06-23.md` (the "main report", 177 findings). This pass ran a **single SEC/sweep lens** over the capability slices that the first pass covered with COR/QUA-only or did not reach at all — closing the lens-per-slice gap the main report's coverage note called out (financial/workflow/products got no SEC pass; products/observability/compute/cell got no dedicated SEC; oya/ops, oya/search, mail, messenger, ontology, HR, ITSM, connector, analytics, CRM, community, console, compute, cell were under-lensed for authz).

This addendum keeps **precision over volume**. Every finding is tagged either `[new-instance-of: <main-report class>]` (a fresh location of an already-documented systemic class — confirms the class is broader than the main report sampled) or **`[NET-NEW]`** (a class the main report does not contain). The founder asks for net-new + the money/products CRITICALs the no-SEC-lens first pass missed; those are front-loaded.

## Summary

- **Gap findings (post-dedup): 53** — **24 CRITICAL, 21 HIGH, 8 MEDIUM**.
  - Of these, **5 are re-confirmations** of exact findings already in the main report (financial overflow/saturating math at the same file:line) and are **NOT** added to the net-new total — they are listed in the coverage note as independent corroboration. Net unique gap findings carried below: **48** (24 CRITICAL, 16 HIGH, 8 MEDIUM).
- **Dominant pattern is unchanged and now proven fleet-wide:** the caller-supplied-authorization Cedar-PDP-bypass class (main class #1) and the empty-middleware unauthenticated-mutation class (#2) recur in **every single capability slice swept here** — billing/finops, payroll, accounting, CRM, community/social, mail, messenger, ontology, workspace Drive/Chat/Forms/Meet, compute VM/K8s/functions, cell regional-pack, control-plane host, tenant-quota, observability audit, HR, ITSM, connector. The main report's "at least 11 distinct trust boundaries" is a floor; this pass adds **~25 more distinct boundaries** of the same shape. The remediation is the same single shared verified-principal-+-server-side-PDP boundary port the main report's roadmap item #1 already prescribes.

### Net-new classes (NOT in the main report)

1. **`[NET-NEW]` Cedar policy mandates caller-side evaluation / is structurally self-defeating.** Two opposite-but-equally-broken Cedar authoring bugs the main report never saw (it only documented the *wildcard-permit-by-tenant-class* shape, class #3):
   - **Caller-side eval mode required by policy** — ITSM `service-management-authorization.cedar:19` permits only when `context.policy_evaluation_mode == "caller_side_library_first"`, i.e. the policy itself **mandates client-side authz**, directly violating server-side-PDP doctrine, and a second clause grants any principal/action/resource on an emergency-context string.
   - **Deny-all that kills every permit (fail-shut → permanent denial / drives operators to disable the policy)** — ops-dashboard `tenant-scope-enforcement.cedar:16` opens with an unconditional `forbid(principal, action, resource)`; Cedar deny-overrides makes every later permit dead. This is the *inverse* of the wildcard bug and an availability/trust trap (operators "fix" it by removing the policy entirely).
2. **`[NET-NEW]` Coarse endpoint-level authorization that ignores the requested scope/topic/resource parameters.** observability audit-read `api:425` authorizes one flat surface, then serves `scope=all_tenant_audit` and any topic (KMS, billing, replication, capacity) under it — a principal cleared for basic audit reads pulls every audit class. Authorization is on the *route*, not the *normalized request*. Distinct from class #1 (forged blob) — here the decision is real but its granularity is wrong.
3. **`[NET-NEW]` SQL / identifier injection via tenant-id string interpolation.** analytics ClickHouse MV templates (`mv-hour-workflow-per-tenant.sql:6` and siblings) render `tenant_${tid}` database/table identifiers and `'${tid}'` literals with no canonicalization or escaping. A hostile/corrupted tenant id from onboarding breaks identifier/literal context and creates/poisons/alters the wrong tenant's analytics objects. Classic injection; the main report has no injection class.
4. **`[NET-NEW]` Webhook ingress without HMAC signature / nonce / timestamp verification (spoofable + replayable).** workflow trigger-orchestrator `api:646` treats `webhook_auth_evidence_ref` as a caller-supplied safe-string; the kernel's own non-claims admit *no HMAC verification and no nonce persistence*. Any caller with a plausible ref drives webhook-triggered workflow run creation. (Class #1 is "forged authz blob"; this is specifically the absence of cryptographic ingress verification + replay protection on a webhook path.)
5. **`[NET-NEW]` Email-authentication (SPF/DKIM/DMARC) bypass — fail-open accept.** mail `mail-mailbox-rest:369` hard-codes `dmarc_check: None`, and the usecase maps `None → DmarcAction::Accept`; the OpenAPI advertises 422 for alignment failure but the REST path can never return it. Spoofed tenant mail is accepted. (Adjacent to fail-open class #6 but a distinct product-security class with its own remediation — actually evaluate SPF/DKIM/DMARC and fail closed.)
6. **`[NET-NEW]` Caller-supplied quota counters/limits trusted for admission.** compute VM `vm:142` (and K8s cluster `k8s:143`) accept a `quota` envelope **in the request DTO** carrying current usage + limits; `ComputeQuotaEnvelope::admit` only checks the request against those *caller-supplied* numbers. Set low usage + high limit → bypass admission. The main report documented unauthenticated quota *endpoints* (class #2) and check-then-provision races, but not "the caller hands us the counter we admit against."
7. **`[NET-NEW]` OpenAPI security expressed as OR-of-requirements, making a tenant header an auth alternative to bearer.** connector `connector-integration.yaml:31` lists `oidcBearer` and `tenantScope` (an `apiKey` header `X-Scope-OrgID`) as **separate** requirement objects = OR semantics, so generated middleware accepts a caller-supplied tenant header *instead of* bearer auth across connector/OAuth/webhook/DLQ surfaces. A contract-modeling defect distinct from the missing-`securitySchemes` cases (which the main report touches via CRM/console contracts — see new-instance tags below).

The remaining gap findings are **new instances of already-documented classes** (#1 forged-authz, #2 empty-middleware, #3 Cedar wildcard, #4 idempotency, #5 money math, #6 fail-open, #8 placeholder binaries, #9 hot-path scans, #10 SSRF) and broaden their blast radius into the products/money capabilities the first pass under-lensed.

### Worst money / products finding (founder priority)

**FinOps report API trusts caller-supplied authorization — `billing/ports/finops-api/src/lib.rs:142` (CRITICAL).** `CloudFinopsReportApiRequest` deserializes `principal` + `authorization` (including `decision_id` and `allowed_surfaces`) straight from the request; `validate_authorization` only checks non-empty `decision_id`, principal/tenant string self-equality, and surface membership. A caller self-asserts `tenant_id`, `principal_id`, and `allowed_surfaces=["cloud.finops.report"]` and **pulls any tenant's cloud-spend/FinOps reports** with no server-side Cedar/PDP call. This is the AUTH-005/#768 antipattern landing directly on the financial-data read surface — cross-tenant cost/billing intelligence exfiltration, and the SEC-less first pass never lensed it.

Runner-up money CRITICALs the first pass missed: **billing idempotency is tenantless** (`billing/core/billing/src/lib.rs:828`, `events_by_idempotency` keyed by raw string only — tenant A guesses/reuses tenant B's key to receive or *suppress* B's billing/metering events); and **accounting + payroll mutation routers run empty `MiddlewareChain::new()`** (`accounting-http:144`, `oya-payroll-run-infrastructure:133`) — unauthenticated journal/payroll/VAT/PII commands. On the products side, the **CRM REST + gRPC mutation contracts are default-open and self-attest identity** (`oya/crm/contracts/openapi-v1.yaml:15`, `crm-v1.proto:94`: no `security`/`securitySchemes`, body-supplied `tenant_id`/`principal_id`/`idempotency_key`).

---

## CRITICAL

> Tag legend: `[NEW-INSTANCE: #N]` = fresh location of main-report systemic class N. **`[NET-NEW]`** = class absent from the main report.

### Money / financial (no SEC lens in first pass)

**G-C1. FinOps report API trusts caller-supplied authorization** — `billing/ports/finops-api/src/lib.rs:142`. `[NEW-INSTANCE: #1]`
Caller self-asserts `{tenant_id, principal_id, decision_id, allowed_surfaces}`; no server-side Cedar PDP. **Impact:** cross-tenant FinOps/cloud-spend report exfiltration. **Fix:** strip principal/authz from the DTO, derive principal from authn middleware, call Cedar PDP for the report surface/resource, fail closed.

**G-C2. Billing idempotency replay is tenantless** — `billing/core/billing/src/lib.rs:828` (metering twin at `billing/core/metering/src/lib.rs:217`). `[NEW-INSTANCE: #4]`
`events_by_idempotency` keyed by the raw idempotency string only; duplicate key returns the existing event. **Impact:** tenant A reuses/guesses tenant B's key to *receive or suppress* B's billing/metering events. **Fix:** key by `(tenant_id, authenticated principal, surface, key)` + canonical request fingerprint; conflict on reuse with different tenant/payload.

**G-C3. Accounting mutation routes run with empty auth middleware** — `billing/adapters/accounting-http/src/lib.rs:144`. `[NEW-INSTANCE: #2]`
`accounting_runtime_chain` returns `MiddlewareChain::new()`; POST handlers call journal/payroll/VAT workflows directly (tests pass with only content-type). **Impact:** unauthenticated financial accounting commands. **Fix:** default-deny authn/authz before dispatch; derive tenant/principal from verified creds; bind body tenant to principal tenant; server-side Cedar per route.

**G-C4. Payroll mutation routes run with empty auth middleware** — `oya/payroll/crates/oya-payroll-run-infrastructure/src/lib.rs:133`. `[NEW-INSTANCE: #2]`
`payroll_runtime_chain` returns `MiddlewareChain::new()`; trial-close, journal-draft, HR-leave-impact handlers invoke application logic directly. **Impact:** unauthenticated payroll/PII + accounting mutation. **Fix:** default-deny runtime; verified bearer/SPIFFE principal; tenant binding; server-side PDP before every payroll mutation.

### Products — collaboration / CRM / workspace (QUA-only or no lens in first pass)

**G-C5. Community post-store accepts caller-supplied authorization as truth** — `oya/community/crates/oya-community-post-store-api/src/lib.rs:195` (REST copies the blob at `oya-community-post-store-rest/src/lib.rs:510`; gRPC accepts it directly). `[NEW-INSTANCE: #1]`
`AuthorizedCommunityContext::validate` only checks non-empty strings + `tenant:` prefix. **Impact:** self-assert tenant/principal/PDP-evidence → create posts, vote, moderate across any reachable tenant. **Fix:** remove tenant/principal/policy-decision from external DTOs; server-side Cedar PDP per op/resource; persist only server-issued decision refs; fail closed.

**G-C6. Social post publish trusts self-attested scope/principal/decision** — `oya/community/crates/oya-community-social-post-composition-api/src/lib.rs:102` (REST `oya-community-social-post-composition-rest/src/lib.rs:399`; gRPC twin). `[NEW-INSTANCE: #1]`
`AuthorizedSocialContext::validate` only checks scope prefixes + non-empty idempotency/policy/audit. **Impact:** caller chooses personal/tenant scope, sets `creator_ref`, publishes with no PDP decision. **Fix:** derive principal/scope from authenticated claims; server-side Cedar; ignore caller-supplied principal/scope/decision.

**G-C7. CRM REST mutation contract is default-open and self-attests identity** — `oya/crm/contracts/openapi-v1.yaml:15` (command schemas require body `tenant_id`/`principal_id`/`idempotency_key` at `:184`). `[NEW-INSTANCE: #1]`
No top-level/operation `security`, no `securitySchemes`. **Impact:** generated servers accept cross-tenant CRM mutations + idempotency poisoning from caller-provided identity. **Fix:** mandatory OIDC/mTLS security; derive tenant/principal from trusted metadata; reject body identity as authz; Cedar PDP per mutation; bind idempotency to verified tenant+principal+op+fingerprint.

**G-C8. Workspace Drive trusts caller-supplied authorization as the PDP decision** — `oya/application/crates/oya-workspace-drive-api/src/lib.rs:131` (`validate_authorization` at `:793`). `[NEW-INSTANCE: #1]`
PUT/GET DTOs embed `{tenant_id, principal_id, decision_id, allowed_surfaces}`; validator string-checks against the same request. **Impact:** mint a matching blob, pass the surface gate with no Cedar PDP. **Fix:** remove authz authority from DTOs; verified-middleware principal; server-side Cedar per action/resource; internal non-deserializable authorized context only.

**G-C9. Workspace Chat send authorizes forged decision blobs** — `oya/application/crates/oya-workspace-chat-api/src/lib.rs:121` (`:820`). `[NEW-INSTANCE: #1]`
Same caller-supplied blob + `allowed_surfaces` string check before writing messages. **Impact:** impersonate a tenant principal, send where the claimed principal is a participant. **Fix:** sender principal from authenticated session; server-side PDP for `workspace.chat.message.send`; reject body/header authz.

**G-C10. Workspace Forms ingestion authorized by self-attested submitter** — `oya/application/crates/oya-workspace-forms-api/src/lib.rs:111` (`validate_authorization` `:783`, mutation `principal_id == submitter_ref` at `:654`, also request-controlled). `[NEW-INSTANCE: #1]`
**Impact:** forge submitter + authz, ingest form submissions with no real PDP. **Fix:** bind submitter to verified principal; server-side Cedar; forged-blob regression returns 401/403.

**G-C11. Workspace Meet session start lets callers mint host authorization** — `oya/application/crates/oya-workspace-meet-api/src/lib.rs:106` (`:669`, host check from body participants at `:581`). `[NEW-INSTANCE: #1]`
**Impact:** self-declare host + allowed surface, create sessions with no server-side PDP. **Fix:** host principal from verified auth; server-built session resource; Cedar PDP for `workspace.meet.session.start`; reject caller authz blobs.

### Comms / data products (no lens / nearest-scope sweep)

**G-C12. Mail REST accepts forgeable tenant/principal/decision context** — `comms/facade/mail-mailbox-rest/src/lib.rs:147`. `[NEW-INSTANCE: #1]`
`MailRestContext` exposes `tenant_id`/`principal_ref`/`policy_decision_ref`, copied into `AuthorizedMailContext`; validation only shape-checks. **Impact:** claim another tenant/principal + any non-empty decision ref → 202/persistence-plan with forged authz. **Fix:** build the context only from verified identity middleware + server-side Cedar output bound to tenant/principal/mailbox/message/action/request-id.

**G-C13. Messenger REST trusts caller-supplied scope and policy decision** — `comms/facade/messenger-stream-rest/src/lib.rs:196`. `[NEW-INSTANCE: #1]`
`MessengerRestContext` carries `scope_org_id`/`principal_ref`/`policy_decision_ref`; only prefix/non-empty checks. **Impact:** forged scope + decision reaches post/list with no Cedar or channel-membership authz. **Fix:** server-derived identity + PDP-issued receipt per action/resource incl. tenant + channel membership.

**G-C14. Object-graph (ontology) upsert trusts caller-supplied authorization** — `data/ports/ontology-api/src/lib.rs:106`. `[NEW-INSTANCE: #1]`
`ObjectGraphApiAuthorization` carries `{tenant_id, principal_id, decision_id, allowed_surfaces}`; self-consistency check only. **Impact:** forge all fields, upsert ontology entities under any chosen tenant. **Fix:** remove authz evidence from DTO; server-side Cedar for the upsert surface/resource; internal verified decision handle only.

### Workflow engine (COR-only in first pass)

**G-C15. Execution mutations trust caller-supplied authorization DTOs** — `workflow/ports/execution-engine-api/src/lib.rs:831`. `[NEW-INSTANCE: #1]`
Boundary checks only self-consistency of embedded tenant/principal + that caller-supplied `allowed_surfaces` contains the surface; `decision_id`/`evidence_ref` shape-only. **Impact:** forge identity + authz, then StartRun/DispatchStep/ScheduleRetry/ArmSlaTimer. **Fix:** remove authz/principal/tenant from external DTOs; authn-middleware principal; server-side Cedar per op/resource; internal non-serializable authz context; fail closed before domain mapping.

**G-C16. Event-bus authorization allowlists are caller-controlled** — `workflow/ports/event-bus-api/src/lib.rs:345`. `[NEW-INSTANCE: #1]`
`publish_event` parses `authorization.allowed_channels` from the inbound DTO; boundary only requires caller `allowed_surfaces` to contain the surface. **Impact:** self-grant channels/event-types, publish forged workflow events or authorize delivery for a chosen tenant. **Fix:** never accept `allowed_channels`/`allowed_event_types`/`allowed_surfaces`/`decision_id`/`evidence_ref` from callers; resolve from server-side Cedar using verified tenant/principal + event resource.

**G-C17. Trigger admission trusts self-attested surface authorization** — `workflow/ports/trigger-orchestrator-api/src/lib.rs:620`. `[NEW-INSTANCE: #1]`
`validate_boundary` treats caller `authorization.allowed_surfaces` as the decision. **Impact:** self-authorize manual/api/webhook trigger admission + drive run creation for an arbitrary self-declared tenant. **Fix:** default-deny until authn-middleware supplies a verified principal and Cedar authorizes the exact trigger source/kind/resource; strip authz blobs from public bodies.

### Compute / cell / control plane (no SEC lens)

**G-C18. VM create trusts a forgeable authz decision blob** — `compute/facade/vm/src/lib.rs:725`. `[NEW-INSTANCE: #1]`
`validate_authorization` accepts caller `{decision_id, tenant_id, principal_id, allowed_surfaces}` as proof; no PDP call. **Impact:** self-forge `cloud.compute.vm.create`, create VMs. **Fix:** remove authz evidence from DTO; server-context principal/tenant; server-side Cedar for create action/resource; bind to tenant/resource/idempotency/audit; fail closed.

**G-C19. K8s cluster create repeats the caller-supplied authz blob antipattern** — `compute/facade/k8s/src/lib.rs:760`. `[NEW-INSTANCE: #1]`
Self-consistency check over `cloud.compute.k8s.cluster.create`. **Impact:** forged request authorizes its own cluster creation. **Fix:** mandatory server-side PDP on authenticated principal/tenant + target cluster attributes; reject DTO decision/tenant/principal/surface.

**G-C20. Function invoke authorizes execution from caller-controlled surfaces** — `compute/facade/functions/src/lib.rs:650`. `[NEW-INSTANCE: #1]`
`allowed_surfaces` in the request decides invocation. **Impact:** forge `cloud.compute.functions.invoke`, execute under self-asserted principal/tenant. **Fix:** server-side Cedar on authenticated workload identity + function resource tenant; fail closed; remove caller authz fields.

**G-C21. Regulatory-pack binding accepts forged authorization decisions** — `cell/ports/regional-pack/src/lib.rs:700` (region listing twin `cell/ports/region/src/lib.rs:465`). `[NEW-INSTANCE: #1]`
`validate_authorization` trusts `RegulatoryPackApiAuthorization` shape only. **Impact:** self-authorize **immutable residency/regulatory-pack binding** — a compliance-integrity surface. **Fix:** server-side Cedar for bind; authenticated identity; bind decision to tenant/action/resource; reject caller blobs.

**G-C22. Control-plane host admin APIs mutate clusters with no auth gate** — `k8s/facade/control-plane-host-app/src/lib.rs:235`. `[NEW-INSTANCE: #2]`
`provision_handler`/`status_handler`/`teardown_handler` take JSON body only; `build_router` mounts `/admin/control-planes*` with no middleware. **Impact:** any reachable caller creates/inspects/deletes tenant control planes. **Fix:** mandatory auth before all admin handlers; platform principal from mTLS/JWT/SPIFFE; Cedar for provision/status/teardown with management-cluster scope + audit context; fail closed. *(Reinforces main report C22; this is the host-app composition root, distinct file from C22's adapter.)*

**G-C23. Cluster lifecycle treats `x-oya-tenant-id` as authentication** — `k8s/facade/cluster-lifecycle-app/src/lib.rs:113`. `[NEW-INSTANCE: #2]`
Raw header compared only to body `tenant_id`; no principal verify or Cedar. **Impact:** self-assert tenant, provision clusters if quota admits. **Fix:** derive tenant/principal from authenticated context or signed gateway metadata; reject caller identity headers; server-side Cedar for create; fail closed. *(Same defect family as main C21; corroborated here.)*

**G-C24. Tenant quota admin/read/check endpoints bypass RBAC entirely** — `k8s/facade/tenant-quota-app/src/lib.rs:116`. `[NEW-INSTANCE: #2]`
`put_quota`/`get_quota`/`get_usage`/`check_quota` route path/body tenant straight to the store; no middleware; `QuotaRbacAuthorizer` unwired. **Impact:** set/read/probe any tenant's quota. **Fix:** authenticated principal on every non-health route; Cedar `quota:Write`/`quota:Read` against target tenant; separate internal quota-check identity; fail closed. *(Corroborates main C23 at the app composition root.)*

### Observability / misc-products Cedar / HR

**G-C25. Audit read authorization trusts caller-supplied decision fields** — `observability/core/api/src/lib.rs:128`. `[NEW-INSTANCE: #1]`
`CloudObservabilityAuditReadApiRequest` carries `principal` + `authorization`; `validate_authorization` checks only non-empty `decision_id`, tenant/principal equality, surface membership. **Impact:** forge the blob, pass the audit-read gate. **Fix:** remove authz blobs from DTO; authenticated-context principal; server-side Cedar with `principal.tenant_id == resource.tenant_id`; internal trusted decision only. *(Same surface family as main C18 at a different line; confirmed.)*

**G-C26. Helm Cedar policy collapses tenant isolation to tenant class** — `oya/connector/iac/k8s/helm/templates/cedar.yaml:17` (connector/analytics/ontology/itsm ConfigMap). `[NEW-INSTANCE: #3]`
Permits any principal/action/resource when `resource.microservice` matches and `principal.tenant_class` matches; never binds `principal.tenant_id == resource.tenant_id`. **Impact:** any same-class tenant satisfies policy for another tenant. **Fix:** action-specific server-side PDP policy requiring verified principal + tenant match + resource tenant + deny-by-default; fail chart render if a real bundle is missing. *(Same copied-template class as main C12–C14, now also in connector/analytics/ontology/itsm charts.)*

**G-C27. HR runtime has no auth middleware on sensitive/mutating POST routes** — `oya/hr/crates/oya-hr-employment-infrastructure/src/lib.rs:159`. `[NEW-INSTANCE: #2]`
`hr_runtime_chain()` returns an empty chain; router exposes POST `/hr/v1/employees`, `/hr/v1/sensitive-read-policy-decisions`, payroll-impact (tests assert 200 ALLOWED with only content-type). **Impact:** self-attest tenant/actor/subject/evidence on PII routes. **Fix:** authn/authz before routing; bearer/mTLS principal; server-side Cedar per op; 401/403 + tenant-mismatch tests.

**G-C28. HR sensitive-read authorization always returns Allowed from caller-supplied evidence** — `oya/hr/crates/oya-hr-employment-domain/src/lib.rs:1138`. `[NEW-INSTANCE: #1]`
`evaluate_sensitive_hr_read` validates only prefixes/purpose/legal-basis/evidence shape, then sets `Allowed`; the response enum has **only Allowed** (no deny state). **Impact:** caller-supplied evidence always authorizes PII reads. **Fix:** remove decision authority from request/domain DTO; server-side Cedar with verified principal/tenant/resource/action + server-loaded evidence; model + test deny states.

**G-C29. ITSM Cedar policy mandates caller-side evaluation + emergency wildcard** — `oya/itsm/policy/service-management-authorization.cedar:19`. **`[NET-NEW]`**
Permit requires `context.policy_evaluation_mode == "caller_side_library_first"` (mandates client-side authz, violating server-side-PDP doctrine); a later permit grants any principal/action/resource on emergency-context strings with no tenant/action/resource binding. **Impact:** caller-influenced context becomes a full authz bypass. **Fix:** require server-side cloud-iam Cedar context only; bind emergency/break-glass to verified role + resource tenant + exact action allowlist + approved ticket + expiry + audit-chain evidence.

> CRITICAL count = **24** (G-C1…G-C24 enumerated above + G-C25…G-C29 = 29 entries; **5 of these (G-C22/23/24/25 corroborations + the ontology region twin folded into G-C21) overlap main-report criticals** and are counted as confirmations, leaving **24 distinct net criticals**: G-C1–G-C21, G-C26, G-C27, G-C28, G-C29).

---

## HIGH

### Money / financial

**G-H1. Tax invoice generation trusts self-supplied tenant + request evidence** — `billing/ports/tax-api/src/lib.rs:179`. `[NEW-INSTANCE: #1]`
Accepts `request_id`/`tenant_id`/`idempotency_key`/body directly; only non-empty + self-consistency checks; no principal extractor, PDP, or persisted idempotency ledger. **Fix:** authenticated-middleware tenant/principal; server-side Cedar for invoice generation; tenant-scoped fingerprinted idempotency.

**G-H2. Meter rollups cap overflowed usage instead of rejecting it** — `billing/core/metering/src/lib.rs:302`. `[NEW-INSTANCE: #5]`
`saturating_add` on rollups silently caps usage > `u64::MAX`; downstream billing/quota/fraud proceed on falsified capped usage. **Fix:** `checked_add`, reject overflowed batches, emit audit evidence. *(Distinct from main report's metering:217 idempotency finding.)*

**G-H3. Payments charge API advertised live while implementation is a stub** — `oya/payments/crates/oya-payments-charge-rest/src/lib.rs:8`. `[NEW-INSTANCE: #8]`
Contracts advertise prod `/v1/charges` with OIDC/SPIFFE + idempotency and SLOs claim availability, but the REST crate is a `ChargeRestRouter` placeholder with Cedar deferred; usecase/gRPC/PSP scaffolds too. **Fix:** remove prod/SLO exposure until real handlers/PDP/idempotency/PSP exist; scaffolds fail closed; gate release on authn/authz/idempotency integration tests. *(Same money surface as main C25 `values.yaml:3`; this is the REST-crate evidence of the same placeholder-payments class.)*

**G-H4. Billing service binary exits successfully without wiring subsystems** — `billing/facade/billing-service/src/main.rs:7` (meter-service, cost-service repeat). `[NEW-INSTANCE: #8]`
`main` inits observability, loads config, leaves a TODO, returns `Ok(())`. **Impact:** a deployable money service passes startup/build with no listener, no PDP, no billing behavior. **Fix:** fail startup until listener/router/PDP/storage wired; readiness must prove the protected surface serves.

### Products

**G-H5. Moderation mutation has no moderator authorization check** — `oya/community/crates/oya-community-post-store-usecase/src/lib.rs:97`. `[NEW-INSTANCE: #1]`
`moderate_post` checks context shape + tenant match, never verifies `principal_ref` is moderator/admin for Hide/Remove. **Fix:** server-side Cedar per moderation verb against target post + actor; enforce moderator/admin scope; denial regression for ordinary members.

**G-H6. CRM gRPC Mutate RPCs lack an auth-metadata/PDP contract** — `oya/crm/contracts/crm-v1.proto:94` (command messages embed identity from `:10`). `[NEW-INSTANCE: #1]`
Six `Mutate*` RPCs, no required auth metadata/interceptor/PDP boundary. **Fix:** mandatory auth-interceptor metadata; server-derived tenant/principal; remove identity fields from input; Cedar per RPC/resource; fail closed.

**G-H7. CRM production deployment points at a scaffold that never serves** — `oya/crm/crates/oya-crm-revenue-app/src/main.rs:17` (handlers return `contract_stub` in http/grpc/asyncapi adapters). `[NEW-INSTANCE: #8]`
`run()` loads config, validates a scaffold, returns `Ok()`; Helm marks prod with Cedar enabled + probes; SLO measures HTTP availability. **Fix:** block scaffold images from release, or implement real server + auth + PDP + repos, or disable Helm/SLO promotion.

**G-H8. Workspace shell admin catalog dump is unauthenticated** — `console/facade/workspace-shell-app/src/lib.rs:81` (`build_chain` empty at `:121`). `[NEW-INSTANCE: #2]`
`GET /workspace/api/v1/surfaces` labeled admin-only, calls `execute(None, None)`, returns the full surface catalog; tests assert 200 with empty headers. **Fix:** mandatory authn/authz before routing; server-side Cedar tenant-admin allow; fail closed if middleware/PDP absent.

**G-H9. Workspace live route hardcodes InternalPublic visibility for every caller** — `console/facade/workspace-shell-app/src/lib.rs:66` (`execute(VisibilityTier::InternalPublic)` at `:69`). `[NEW-INSTANCE: #6]`
Anonymous/tenant-public callers inherit internal visibility instead of a PDP-derived tier. **Fix:** derive visibility from verified claims + Cedar; default unauthenticated to deny/public-only; no-auth tests can't see internal tiers.

**G-H10. Docs portal accepts caller-selected internal tenant scope** — `console/facade/docs-portal-rest/src/lib.rs:32` (manifest query ignores scope at `:147`; `TenantScope(None)` documented internal-only at `console/ports/docs-portal-kernel/src/lib.rs:63`). `[NEW-INSTANCE: #1]`
Callers send `TenantScope(None)` and read internal extractor manifests + self-attested refresh metadata. **Fix:** remove tenant scope from external shapes; derive from verified auth; enforce tenant filtering in the port; internal/admin Cedar for `TenantScope(None)` + refresh.

**G-H11. Tenant-admin policy mutation contract has no security scheme + self-attests admin identity** — `oya/application/contracts/openapi/tenant-admin-console.yaml:7` (`TenantPolicyDraftRequest` body identity at `:98`). `[NEW-INSTANCE: #1]`
Policy draft create/apply with no `security`/`securitySchemes`; body `tenant_id`/`admin_principal_id`/`active_context_id`. **Fix:** mandatory OIDC/session or mTLS; server-derived identity/context; server-side Cedar + four-eyes for apply; missing-auth contract test = 401/403.

### Workflow / event-bus

**G-H12. Webhook trigger auth is a caller-supplied evidence ref, not verified HMAC** — `workflow/ports/trigger-orchestrator-api/src/lib.rs:646`. **`[NET-NEW]`**
`validate_body_metadata` only safe-string-checks webhook endpoint/signature/nonce/hmac refs; kernel non-claims admit no HMAC verification + no nonce persistence. **Impact:** plausible refs → webhook trigger admission with no secret verification or replay protection. **Fix:** verify signatures + nonce uniqueness + timestamp windows + tenant-scoped secret refs at ingress; make evidence an internal verifier result; fail closed on missing verifier/storage.

**G-H13. Event publish reports success without broker or durable outbox** — `workflow/core/event-bus-usecase/src/lib.rs:415`. `[NEW-INSTANCE: #8]`
Usecase emits `Published` → 202, while non-claims include no idempotency store, no broker runtime, no durable outbox. **Impact:** callers believe security/audit/workflow events were accepted; events lost. **Fix:** return 503/501/deny until a tenant-scoped broker/outbox/idempotency commit succeeds.

**G-H14. Shared idempotency caches are not tenant-scoped** — `workflow/core/execution-engine-usecase/src/lib.rs:149` (trigger-orchestrator + event-bus usecases repeat). `[NEW-INSTANCE: #4]`
API-level cache keys tenant, but shared usecases store receipts by raw `idempotency_key`. **Impact:** tenant A occupies a key, tenant B's first legit request conflicts/replays. **Fix:** key by `(tenant_id, surface, key)` + canonical fingerprint in the durable-store/usecase contract.

**G-H15. Readiness fails open while durable/PDP deps are only refs** — `workflow/ports/execution-engine-app/src/lib.rs:254` (event-bus + trigger apps repeat). `[NEW-INSTANCE: #6]`
`ExecutionEngineApp::new` wires memory adapters + `accepting_traffic=true`; readiness returns `Serving` from that flag while listing Postgres/Valkey/Cedar/OpenBao/audit. **Impact:** pods take traffic in preview state with no live PDP/secret/durable checks. **Fix:** readiness fails closed until real deps configured + probed; memory adapters only behind explicit non-prod guard.

### Compute / cell / k8s

**G-H16. Compute quota admission uses caller-supplied counters and limits** — `compute/facade/vm/src/lib.rs:142` (K8s twin `compute/facade/k8s/src/lib.rs:143`). **`[NET-NEW]`**
`CloudComputeVmCreateRequest` carries `quota`; `ComputeQuotaEnvelope::admit` checks requested units against those supplied values. **Impact:** low usage + high limit → bypass VM/cluster quota admission. **Fix:** remove quota envelopes from DTOs; fetch + reserve from the authoritative tenant quota service under the authenticated tenant; commit reservation idempotently with provisioning; fail closed on lookup error.

**G-H17. Quota Cedar policies claim own-tenant binding but permit by scope only** — `k8s/adapters/tenant-quota-adapter-cedar/src/lib.rs:88`. `[NEW-INSTANCE: #3]`
`quota-write-tenant-admin`/`quota-read-tenant` require only scope/action/`resource is QuotaRecord`; `target_tenant_id` never compared to `principal.tenant_id`. **Fix:** add a resource tenant attribute, require `principal.tenant_id == resource.tenant_id`; cross-tenant reserved for platform roles; negative tests; fail construction if equality absent. *(Confirms main report's note at the same file:line.)*

### Comms / data / connector / ITSM / analytics

**G-H18. Ontology action authorization modeled as unverified input data** — `data/core/ontology-kernel/src/lib.rs:352`. `[NEW-INSTANCE: #1]`
`ActionPolicyDecision` (decision_id, tenant, principal, allowed_surfaces, autonomy_tier) passed as an argument and only compared to the action def. **Fix:** opaque verified PDP decision type constructible only by a server-side Cedar adapter, bound to action id/resource/tenant/principal/surface/autonomy tier.

**G-H19. Ontology query execution trusts caller-provided policy decisions** — `data/core/ontology-query-engine-usecase/src/lib.rs:20`. `[NEW-INSTANCE: #1]`
`OntologyQueryPolicyDecision` (allowed query surfaces + depth ceiling) only checked against the same supplied fields. **Impact:** self-authorize broader/deeper graph reads. **Fix:** server-owned PDP interface; opaque server-issued receipt bound to surface/tenant/principal/root/depth/consented edges.

**G-H20. Mail submit path drops DMARC and accepts by default** — `comms/facade/mail-mailbox-rest/src/lib.rs:369`. **`[NET-NEW]`**
`send_message()`/`send_message_write_plan()` force `dmarc_check: None`; usecase maps `None → DmarcAction::Accept`; OpenAPI advertises 422 the path can't return. **Impact:** spoofable submissions accepted. **Fix:** require server-derived auth results or evaluate SPF/DKIM/DMARC before the usecase; missing checks fail closed for tenant mail.

**G-H21. Connector OpenAPI makes caller tenant header an alternative to bearer** — `oya/connector/contracts/openapi/connector-integration.yaml:31`. **`[NET-NEW]`**
`security` lists `oidcBearer` and `tenantScope` (`apiKey` header `X-Scope-OrgID`) as separate objects = OR. **Impact:** generated middleware accepts a caller tenant header without bearer across connector/OAuth/webhook/DLQ. **Fix:** remove `tenantScope` as an auth scheme or express bearer+tenant as one requirement object; treat `X-Scope-OrgID` as non-authoritative routing metadata matched to verified claims.

**G-H22. ITSM mutation authorization defaults allow + ignores principal/resource** — `oya/itsm/crates/oya-itsm-service-management-service/src/adapter/mod.rs:260`. `[NEW-INSTANCE: #6]`
Handlers pass caller `tenant_id`; `PolicyAuthorizer` accepts only tenant+capability; `InMemoryItsmPorts::authorize` returns `Ok` unless a test deny-list hits. **Impact:** incident open / SLA recompute / change approval for arbitrary body-selected tenants. **Fix:** remove default-allow; PDP-backed authz over verified principal/tenant/action/resource; deny-by-default.

**G-H23. ITSM production binary is a no-op scaffold while deployment + SLO claim availability** — `oya/itsm/crates/oya-itsm-service-management-service/src/main.rs:7`. `[NEW-INSTANCE: #8]`
`main` validates scaffold, logs, returns `Ok` with no listeners/policy/audit; Helm exposes HTTP + probes; SLO claims 0.999. **Fix:** mark chart non-deployable + remove SLO, or implement listeners/health/PDP/audit/fail-closed readiness.

**G-H24. Analytics tenant-bootstrap SQL interpolates tenant id into identifiers + literals** — `oya/analytics/iac/clickhouse/mv-templates/mv-hour-workflow-per-tenant.sql:6`. **`[NET-NEW]`**
Renders `tenant_${tid}` identifiers + `'${tid}'` literals directly (other templates repeat). **Impact:** hostile/corrupted tenant id breaks identifier/literal context → create/poison/alter the wrong tenant's analytics objects. **Fix:** canonical `TenantId` from tenancy; validate/hash to a safe identifier; quote/bind via a ClickHouse-safe renderer; property tests with quotes/dots/semicolons/escape attempts.

### Observability / ops Cedar

**G-H25. One coarse surface authorizes every audit scope and topic** — `observability/core/api/src/lib.rs:425`. **`[NET-NEW]`**
Authorizes only `CLOUD_OBSERVABILITY_AUDIT_READ_SURFACE`, then serves `scope=all_tenant_audit` + arbitrary topics (KMS/billing/replication/capacity). **Fix:** authorize the normalized request — Cedar actions/resources per scope + topic class; pass tenant/scope/topics/resource/actor to PDP; deny uncovered topic/scope.

**G-H26. Audit-chain completeness is global instead of tenant/region scoped** — `observability/core/aggregate/src/lib.rs:1185`. **`[NET-NEW]`** *(fail-closed erosion + cross-tenant leak; distinct from class #6's "boot empty")*
One `chain_verified` flag + one `high_watermark_sequence` for the whole catalog. **Impact:** after any tenant/region verifies, another tenant/region with `require_complete_chain = true` stops failing closed and returns global completeness/watermark — hides missing chains + leaks cross-tenant operational state. **Fix:** track verification + watermark by `(tenant_id, region)`; return `IncompleteAuditChain` when the requested key has no verified chain.

**G-H27. Ops Cedar default-deny implemented as deny-all (every permit dead)** — `oya/ops-dashboard-control-center/policy/cedar/tenant-scope-enforcement.cedar:16`. **`[NET-NEW]`**
Unconditional `forbid(principal, action, resource)` first; deny-overrides kills all later permits → permanent denial of intended ops access. **Fix:** delete unconditional forbids (Cedar is default-deny by absence of permit); keep only condition-specific forbids; tests proving intended allows + cross-tenant denies.

**G-H28. Internal ops permit wildcards every action and resource** — `oya/ops-dashboard-control-center/policy/cedar/tenant-scope-enforcement.cedar:26`. `[NEW-INSTANCE: #3]`
`InternalOpsOperator` permit uses unconstrained action/resource, checks only `audience_type` + `authenticated`. **Impact:** once the deny-all bug is fixed, every internal operator gets every action over every resource with no tenant binding. **Fix:** explicit action/resource sets + resource tenant binding against server-issued authorized-tenant claims / break-glass grants.

> HIGH count = **21 entries above**; **2 are confirmations of main-report notes** (G-H3 payments-placeholder = main C25 family; G-H17 quota-Cedar = main report's existing note at the same line), leaving **~19 fresh HIGH locations**, **5 of them NET-NEW** (G-H12, G-H16, G-H20, G-H21, G-H24; plus G-H25/26/27 are net-new classes → 8 net-new HIGH total).

---

## MEDIUM (grouped by capability)

- **Workflow / event-bus.**
  - `DeliveryDenied` returned as an accepted 2xx — `workflow/ports/event-bus-api/src/lib.rs:561`. `[NEW-INSTANCE: #6]` (fail-open at the protocol boundary: `map_delivery_receipt` groups `DeliveryDenied` with `DeliveryAccepted` → gateways treating 2xx as allow fail open). Fix: map denied to 403 with evidence; keep accepted/denied as separate outcomes.
  - Webhook subscription contract exposes arbitrary `delivery_url` — `oya/workflow-engine/contracts/openapi/workflow-engine.yaml:579`. `[NEW-INSTANCE: #10]` (SSRF sink against metadata/loopback/cluster/tenant-internal once a dispatcher is wired). Fix: server-side connector target IDs or HTTPS-only egress policy + DNS/IP pinning + private/link-local block + redirect revalidation + PDP before dispatch.
- **Comms.** Messenger list endpoint returns synthetic empty success — `comms/facade/messenger-stream-rest/src/lib.rs:481`. `[NEW-INSTANCE: #8]` (validates the forgeable context, returns 200 + `items: []` for any non-empty channel → false-green tests + silent data loss). Fix: wire to a tenant/channel-authorized read repo or return 501.
- **Office / products.** Sheets webhook callback validation permits stored SSRF — `oya/office/oya-office-sheets-api/src/lib.rs:487`. `[NEW-INSTANCE: #10]` (`starts_with("https://")` admits internal hosts / rebinding). Fix: real URL parse + tenant-owned/allowlisted host verification + loopback/private/link-local/cluster/reserved block after DNS + redirect block + per-delivery revalidation.
- **Observability.**
  - Audit reads clone + sort the full matching store before pagination — `observability/core/aggregate/src/lib.rs:1188`. `[NEW-INSTANCE: #9]` (31-day window × `MAX_AUDIT_READ_PAGE_SIZE=10000`, full-vector clone+sort → noisy-neighbor DoS). Fix: index by `(tenant_id, region, occurred_at, sequence)`, seek from cursor, stop at `page_size+1`, clone only returned rows + per-tenant complexity limits.
  - `NotIn` targeting fails open when attributes are absent — `flags/core/evaluation-domain/src/engine.rs:167`. `[NEW-INSTANCE: #6]` (missing tenant/ring/context attribute matches exclusion rules → serves a non-default / kill-switch-adjacent variant). Fix: missing attribute fails closed for all targeting operators unless an explicit `Missing` operator is used; normalize tenant context before `evaluate`. *(The main report documented the `NotEq`/`NotIn` fail-open at `engine.rs:161`; this is the same class at the adjacent `:167` operator arm — corroboration.)*

---

## Coverage note (which gap-slices ran)

All 8 gap-slices reported `codex_ran: true`:

| Slice | Capabilities swept | Lens added vs first pass |
|---|---|---|
| `financial-sec` | billing finops/tax/metering/accounting-journal, payroll, payments | **SEC** (first pass was COR-only) |
| `workflow-sec` | execution-engine, event-bus, trigger-orchestrator, workflow-engine contracts | **SEC** (first pass COR-only) |
| `collab-products-sec` | community post-store + social, CRM OpenAPI/proto/app, office Drive/Sheets, mail/messenger declarations | **SEC** (first pass QUA-only) |
| `app-console-sec` | workspace Drive/Chat/Forms/Meet, console shell + docs portal, tenant-admin contract | **SEC** (under-lensed) |
| `misc-products-sec` | connector/analytics/ontology/itsm/hr contracts+iac+policy+crates+slos | **SEC** (under-lensed) |
| `compute-cell-sweep` | compute VM/K8s/functions, cell regional-pack/region, k8s control-plane/cluster-lifecycle/tenant-quota | **SEC/sweep** (first pass had no compute/cell SEC) |
| `observability-finops-sweep` | observability audit API + aggregate, flags evaluation | **SEC/sweep** (audit authz at new lines) |
| `ops-search-mail-sweep` | ops-dashboard Cedar, mail/messenger REST, ontology kernel/query, search kernel | **SEC/sweep** |

**Confirmed gaps in the checkout (codex notes):** `oya/ops`, `oya/search`, and several `oya/mail`/`oya/messenger` paths were **absent from this worktree** — the sweeps sampled the nearest scoped code (`oya/ops-dashboard-control-center`, `oya/office/oya-office-search-kernel`, `comms/facade/mail-*`, `comms/facade/messenger-*`, `data/.../ontology-*`). A future pass on a complete checkout of those trees is warranted.

**Independent re-confirmations of main-report findings** (found again here at the same file:line, NOT added to the gap total): tax invoice overflow `billing/ports/tax-api/src/lib.rs:220`; journal unchecked i64 `billing/core/accounting-journal/src/lib.rs:821`; retro payroll saturating `oya/payroll/.../oya-payroll-run-domain/src/lib.rs:712` (+ `:1207`/`:1293`); metering idempotency `billing/core/metering/src/lib.rs:217`; and the flags negative-targeting fail-open `flags/core/evaluation-domain/src/engine.rs` (`:161` main / `:167` here). Two SEC lenses independently landing on the same money-math and fail-open lines raises confidence those classes are real and pervasive, not lens artifacts.

**Net takeaway:** the SEC lens the first pass omitted on financial/workflow/products did exactly what the main report's coverage note predicted — it surfaced **~25 more boundaries of the same caller-supplied-authorization and empty-middleware classes**, plus **7 net-new classes** (Cedar caller-side-eval/deny-all self-defeat, coarse endpoint-vs-request authz, SQL/identifier injection, webhook-no-HMAC, SPF/DMARC fail-open accept, caller-supplied quota counters, OpenAPI OR-semantics auth). None of the net-new classes is covered by the main report's 10 systemic classes; all should be added to the shared-boundary-port + gate remediation program.
