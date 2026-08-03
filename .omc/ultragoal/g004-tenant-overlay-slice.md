# G004 slice — per-tenant Cedar policy overlays (architect-scoped 2026-06-21, dev 8558cfd72)

Closes the "per-tenant policy" gap: PolicyBundle has ONE global policies_src (kernel lib.rs:124-136), no per-tenant scoping. Tenant isolation today = structural forbid + SVID binding + per-tenant template_links only. The G002 SVID work (#793) makes per-tenant policy CONTENT meaningful (caller tenant now cryptographically trusted). One reviewable PR; single decision algorithm preserved (ADR-0243 — do NOT touch the legacy PolicySet::authorize() retirement, that's a separate deferred multi-crate slice in oya/application/).

## What EXISTS (consume, don't rebuild)
- libs/oya-shared-pdp-kernel/src/lib.rs — PolicyDecisionPoint trait, PolicyBundle, DecisionCache, DecisionAuditRecord, PdpError (fail-closed).
- libs/oya-shared-pdp-adapter-cedar/src/lib.rs:60-65,111-135,265-350 — CedarPdp: default-deny, forbid-overrides-permit, strict-validate, @id-rekey, swap_bundle, zookie staleness, cache, one-audit-per-decision. LoadedBundle, compile(), authorize().
- libs/oya-shared-pdp-adapter-cedar/cedar/platform-policies.cedar — @id("structural-tenant-isolation") forbid backing all overlays.
- iam/facade/cloud-pdp-app/src/grpc.rs:230-253 + mtls.rs:184-235 — SVID-derived tenant SUPERSEDES body tenant_id (the trust anchor overlay selection keys off).
- iam/adapters/cloud-pdp-bundle-file/src/lib.rs:57-72 — closed-schema parse_bundle + version-token re-validation (new field must round-trip + not bypass it).
- tests: cedar_pdp_conformance.rs:75-156(two-tenant acme/globex fixture),186/229/268/300/335 (RBAC/ABAC/PBAC/forbid); cloud-pdp-app tests/mtls_live_socket.rs:288 (cross-tenant 403 E2E); tests/seed_parity.rs:32.

## The change
PolicyBundle += `pub tenant_policies: BTreeMap<String,String>` (tenant_id -> tenant-scoped Cedar src; deterministic; deny_unknown_fields-compatible; defaults empty = backward compatible so flat bundles still parse).
- compile() (adapter lib.rs:111+): parse each tenant overlay into a Cedar PolicySet, @id-rekey exactly as the global set (lib.rs:120-135), store BTreeMap<String,PolicySet> on LoadedBundle. FAIL-CLOSED BundleRejected on: an overlay that isn't strict-valid; AND (LOAD-BEARING INVARIANT) any overlay whose policies could grant ACROSS tenants — reject at load so the forbid stays structural, not emergent.
- authorize() (adapter lib.rs:265+): after the SVID-bound request.tenant_id is known, evaluate against the UNION of {global policy set} ∪ {that tenant's overlay ONLY} — never another tenant's overlay. Structural forbid + forbid-overrides-permit still guarantee no escape; union scoping makes it structural at selection too. Confirm request_fingerprint includes tenant_id (cache correctness).
- cloud-pdp-bundle-file: no code change (closed schema picks up the field); extend its seed_bundle() fixture. Seed .cedar/JSON fixtures under libs/oya-shared-pdp-adapter-cedar/cedar/ AND iam/facade/cloud-pdp-app/cedar/ (keep seed_parity green).

## Clean-arch
Port (kernel PolicyBundle) gains the overlay field — the W5 CRD carries the SAME per-tenant compiled overlays (cutover litmus holds: store always compiles+pushes per-tenant content, only transport changes). Adapter (cedar) does overlay compile + tenant-scoped eval — ONE decision algorithm. File store adapter interface unchanged (durable per-tenant store swaps in behind PolicyBundleStore later, zero port change). Tenant selection keys off the SVID-bound tenant (grpc.rs:252-253), never the body.

## Tests (RED→GREEN, reuse acme/globex fixture)
1. tenant_overlay_permit_applies_only_within_owning_tenant — acme overlay grants bob ReadResource on acme-doc-1 → ALLOW for tenant acme, attributed to overlay id.
2. tenant_overlay_does_not_leak_to_other_tenant (SVID-bound isolation) — same overlay, request tenant_id=globex → DENY.
3. fail_closed_default_deny_with_empty_overlays — empty tenant_policies, no global permit → DENY.
4. malformed_tenant_overlay_rejects_whole_bundle_fail_closed — invalid Cedar overlay → BundleRejected, nothing loads.
5. tenant_overlay_authoring_cross_tenant_permit_is_rejected_at_load — overlay permitting across tenants → BundleRejected (structural, not emergent).
6. facade: extend seed_parity.rs for the new field; optionally one mtls_live_socket.rs fixture proving the SVID-bound tenant selects the correct overlay E2E.

## Deps / done-bar
- PolicyVersion opaque-token re-validation still runs on the extended bundle (must not bypass).
- seed_parity.rs:32 canonical-vs-crate-local seed equality stays green across all 3 cedar/ seed locations.
- request_fingerprint includes tenant_id (cache; confirm).
- buck2+cargo TARGET PARITY: any new tests/ integration target mirrored as a rust_test BUCK target using option_env!("CARGO_MANIFEST_DIR") + skip-with-stderr (NOT env!), per FRIC-020/story-G004 + buck2-build-green≠CI-green. Regen lock+faces, run freshness/affected-set gates before done.
- born-account any NEW tracked file (new cedar fixtures) — ADR justification + reachability; re-settle faces; firewall GO-LIVE green (the #793 lesson: non-crate files like fixtures/yaml need explicit reachability).

## OUT of scope (do NOT pull in)
- Retiring the legacy PolicySet::authorize() (policy-cedar-domain/src/lib.rs:194-251, live in oya/application/oya-application-app/src/lib.rs:782,1326) — separate multi-crate vertical (ADR-0243), deferred.
- Bundle signature verification — gated on the policy-bundle CRD/operator delivery fabric.
