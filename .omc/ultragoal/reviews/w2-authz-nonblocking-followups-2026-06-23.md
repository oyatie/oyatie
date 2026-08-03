# Wave-2 authz review — nonblocking follow-ups (2026-06-23)

Source: adversarial review (codex 5.5-xhigh + in-house) of the AUTH-005 Wave-2 authz PRs (#817 KMS, #819 audit, #820 network, #822 observability, #823 finops). All five APPROVED with ZERO blocking findings. The items below are real but nonblocking — every one of these five PRs is a **dead-until-edge boundary library** (the authz seam exists, removes the old caller-trusted bypass, and is unforgeable at the type level, but has NO live HTTP/gRPC composition root yet). These obligations come due **when the edge adapters that wire these seams are built**.

## A. Per-seam edge-adapter obligations (when each seam goes live)
Every seam's edge adapter MUST, before it serves:
- **authn before body**: verify bearer/SVID over request `Parts` via `route_layer`/`FromRequestParts`/middleware, short-circuit 401/403 BEFORE body deserialization, + `DefaultBodyLimit`. (The library signatures guarantee a verifier+PDP are *required*, but cannot enforce ordering — that's the edge's job. This is the exact class that shipped as the #821 CRITICAL on the one seam that DID have a live binary.)
- **boot-refuse without authz**: the binary must refuse to start if verifier/PDP provider is absent.
- **production verifier, not the reference**: swap `ConfiguredBearer*PrincipalVerifier` (single static secret → one identity, break-glass only) for the cloud-iam mTLS/SPIFFE peer-SVID verifier (ADR-0561).
- **add the serving crate to `cloud-ci-authz-coverage` gate `scan_roots`**: today `secrets`, `audit`, `observability`, `billing/finops` serving crates aren't gate-scanned (route-discovery based; these have no route yet). When the route lands, add the root so route-level authz is mechanically backstopped (enforcement-layering doctrine).

## B. #819 audit — decorative blast-radius (fix when the edge sources tenant)
`audit_emit_resource` sources PDP `resource.tenant_id` from `payload.tenant_id`, but `validate_envelope_payload_binding` forces envelope.tenant==payload.tenant AND the verified cross-check forces envelope.tenant==verified.tenant — so the PDP tenant axis can NEVER differ from the verified principal. Cross-tenant is denied by the cross-check, NOT the PDP; the `AuditEmitScope::Platform` branch is unreachable. NOT an escalation (cross-tenant denies fail-closed; platform-emit is inexpressible), but the ADR-0588 claim "a tenant producer cannot forge a platform-level audit record" is not load-bearing. Fix at edge: source the target tenant/scope from a TRUSTED edge context (path/datastore) independent of the caller-bound verified tenant, OR drop the platform-scope claim + dead branch. Also: dead error variant `AuditEventEmitAppError::PrincipalUnverified` (declared+mapped+doc'd, never constructed).

## C. #820 network — pre-existing reorg drift
`registry/openapi/runtime-bindings.tsv` (lines ~15-17) still points `createCloudNetwork{Vpc,DnsZone,LoadBalancer}` at stale `crates/oya-cloud-network-*-api/src/lib.rs` paths that don't exist; canonical impl is `network/ports/{vpc,dns,lb}/src/lib.rs`. Pre-existing capability-move drift (not introduced by #820). Fold into the reorg drift backlog.

## D. PRODUCTIZE — gate: forbid `derive(Debug)` on secret-bearing structs (the spreading class)
#817's `CallerCredential` implements a CUSTOM `Debug` that REDACTS the bearer; #819's `CallerCredential` DERIVES `Debug` while holding `authorization: Option<String> // data_class: SECRET` → any `{:?}`/panic/tracing leaks the bearer (sufficient to mint the bound principal). The review noted #819's is "IDENTICAL to" other instances — i.e. the anti-pattern is spreading. This is exactly a "make it impossible" candidate: a hermetic Rust gate that fails when a struct deriving `Debug` (or `Display`) contains a field annotated `data_class: SECRET` or named with a secret token-list (authorization/token/secret/password/key-material), unless it has a manual redacting impl. Pairs with the existing kernel-purity / canonical-json gate fleet. Until the gate lands, hand-fix #819's `CallerCredential` to a redacting `Debug` matching #817 when the audit edge is wired.

## E. Cosmetic / accepted
- #822: scope label `all_tenant_audit` is misleadingly named (it means all record-CLASSES within one tenant; `matches()` hard-filters `record.tenant_id==self.tenant_id`); `request_hash` is FNV-1a (honest non-MAC binding token, PDP is the authority).
- #823: ADR-0591 + authz.rs describe `PLATFORM_AGGREGATE_TENANT_ID='ten_platform'` as a cross-tenant aggregate, but the kernel filters strictly by allocation tenant — doc overclaim, not a vuln.
- All seams: `constant_time_eq` leaks input length via the `a.len()^b.len()` seed (matches the iam reference; accepted for fixed-length bearers; real prod credential is SPIFFE SVID).
- ADRs 0573/0586/0587/0588/0590/0591 are status=Proposed; landing under founder standing merge authority — confirm Accepted via governance later. No firewall door was self-signed.
