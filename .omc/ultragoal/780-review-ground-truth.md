# #780 authz-coverage gate — orchestrator's INDEPENDENT ground truth (adversarial-review criteria) — 2026-06-22

Built by reading PR head `bea3aa523d0eb49131f1e7923356bbbcd0034194` directly (NOT trusting the executor's verdict). Use this to reconcile the round-2 executor's report — do NOT rubber-stamp.

## Established facts (file:line evidence)
1. **`publish_handler` is genuinely unauthenticated — self-attested header trust.**
   - `iam/ports/policy-cedar-api/src/rest/mod.rs:223` `publish_handler` reads `x-principal-id`, `x-principal-tenant-id`, `x-authorization-{decision-id,tenant-id,principal-id,surfaces}` straight from `HeaderMap`, builds `CedarPolicyApiAuthorization`, calls `publish_cedar_policy_from_api(...)`. NO bearer/mTLS/PDP-decide.
   - `iam/ports/policy-cedar-api/src/lib.rs:665` `validate_authorization` only checks the caller-supplied headers are internally consistent (`authorization.tenant_id == principal.tenant_id`, `principal_id` match, requested surface ∈ caller's own `allowed_surfaces`). An attacker reaching the socket sets ALL those headers → `Ok(())`. Textbook AUTH-005 self-attestation bypass. (#124 remediates the surface.)
2. **`build_router` has NO router-level auth layer** — `rest/mod.rs:206-210` is `Router::new().route(PUBLISH_ROUTE, post(publish_handler)).with_state(state)`. So no `auth_layer_idents` match.
3. **`publish_handler` body contains NO recognized `authz_guard_idents`** (policy list: `authorize`, `.decide(`, `authenticate_caller`, `require_bearer`, `verify_principal`, `ensure_authorized`, `check_authz`, `constant_time_eq`, `verify_*`, …). The real check is in the `lib.rs` delegate, which isn't a recognized ident.
4. **The gate's guard recursion is fail-CLOSED but single-file.** `has_guard_rec` (gate `src/lib.rs:1615`) scans only the *current file's* `text` for `fn <delegate>`. `publish_cedar_policy_from_api`/`validate_authorization` live in `lib.rs`, not the scanned `rest/mod.rs`, so the recursion cannot reach them → finds no guard → returns `false` (line 1661 default). Good: no fail-open in the recursion itself.
5. **The 16-entry `frozen_unauthenticated_surfaces` baseline does NOT contain `iam/ports/policy-cedar-api`.** (Verified all 16: billing accounting-http, console workspace-shell, iac infra, iam identity-service SCIM, iam identity-workload-rest /authorize*, iam tenant-rbac, k8s ×3, oya ci-controller, oya hr, oya intelligence ×4, oya payroll. None is policy-cedar.)

## The inescapable conclusion
Given 1-5, a CORRECT gate run on the integrated tip MUST emit a NEW finding for the policy-cedar publish surface (control-plane: POST + path params, no guard, not baselined). But oya-ci-required is currently **GREEN**. So one of these is true:
- **(BLOCKER-real)** The gate's route DISCOVERY does not see `.route(PUBLISH_ROUTE, post(publish_handler))` (const-path resolution miss, or `iam/ports/**` not actually scanned) → a discovery FAIL-OPEN. Must be fixed so the surface becomes visible, THEN baselined.
- **(false-probe)** The executor's "handler IS found to have a guard" was a preliminary misread; the gate actually DOES flag it and the PR is only green because the surface was added to the baseline in an unpushed commit.

## ACCEPTANCE BAR for the executor's round-2 (hold firm — security gate)
- [ ] **The gate, on the integrated tip, DEMONSTRABLY SEES the policy-cedar publish surface** (proven by a run that lists it — either as a NEW finding before baselining, or as a baselined entry after). A verdict of "the gate classifies it as guarded, nothing to do" is REJECTED: that would mean header-trust counts as a guard (defeats the gate's purpose) OR a discovery hole hides it.
- [ ] **It is captured in `frozen_unauthenticated_surfaces` as category-A pre-existing debt** (so the gate lands green AND the surface is tracked), with a note pointing at #124 for remediation. NOT silently guarded-away.
- [ ] **MAJOR fixed**: an unclassifiable owned-kernel dynamic `.route(method, route.path, handler)` → `AC-UNCLASSIFIED-SURFACE` (fail-closed), NOT dropped to None. RED fixture that fails when the hole is reintroduced.
- [ ] **MINOR fixed**: `tests/*.rs` excluded from the prod scan (verify no real surface is thereby hidden).
- [ ] Gate GREEN on integrated tip; born-accounting/gate-registration intact; ADR-0566 honest about the header-trust limitation.

## Heuristic-limitation finding (file as follow-up regardless)
Even after baselining, this gate counts ANY recognized guard-ident call as "guarded" — it CANNOT distinguish a real principal-verifying guard from a header-trust validator. So a future surface that calls e.g. `validate_authorization`-style header trust but happens to also name a guard ident would pass. The gate as scoped (#770) blocks routes with ZERO guard-shaped code; a v2 "guard must verify a principal / call a PDP, not trust headers" is a separate, stronger gate. Document the boundary; do not overclaim the gate prevents AUTH-005 self-attestation.
