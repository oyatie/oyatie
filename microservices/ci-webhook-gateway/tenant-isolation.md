# Tenant Isolation — ci-webhook-gateway

> Design surface for ADR-0374. How the webhook receiver keeps one tenant's
> source-forge events from triggering another tenant's pipeline. The gateway is
> a **stateless event router** (Forgejo PR event → admission → `oya gate run-all`
> → reviewer → merge dispatch); it holds no tenant data at rest, so isolation is
> enforced at the trust boundary and on the dispatch target, not via stored state.

## Tenant identity

A tenant is a distinct source-forge trust domain — a Forgejo org/instance whose
repositories enter the governance pipeline. Per `oyatie-dogfood-tenancy`, Oyatie's
own repository is the **first tenant** of its own gateway; it receives no internal
bypass and is isolated by the same mechanism as any external tenant.

## Trust boundary: per-tenant HMAC

Each tenant registers its Forgejo webhook with a **per-tenant HMAC secret**, stored
in OpenBao (`sref://openbao/oya/ci/<tenant>/forgejo-webhook-secret`, never in the
process image — see `SETUP-RUNBOOK.md`). On every request `signature.rs` performs a
**constant-time HMAC-SHA256 verification of the raw body against that tenant's
secret, before any parse or routing**. Consequences:

- A request that does not authenticate against a registered tenant secret is
  rejected `401` and never reaches dispatch.
- An event forged or replayed toward another tenant fails that tenant's HMAC and
  is rejected — cross-tenant event injection is not possible at the edge.
- Secret rotation is per-tenant; rotating or revoking one tenant's secret cannot
  affect another tenant's admission.

## Dispatch isolation

The dispatch target (which pipeline / `oya gate run-all` scope is kicked) is derived
from the **HMAC-validated tenant binding**, not from untrusted payload fields alone.
An event authenticated for tenant A can only dispatch tenant A's pipeline. The
`PipelineDispatcher` port (`dispatch.rs`) receives the resolved tenant context; the
`JenkinsDispatcher` targets only that tenant's lane. A misconfigured or compromised
single-tenant webhook therefore cannot trigger another tenant's build, merge, or
gate run — the blast radius is contained to the originating tenant.

## No shared mutable state

The receiver is stateless between requests: no tenant secrets, prompts, or event
bodies persist beyond the request lifetime; secrets are resolved per-request from
OpenBao and dropped. There is no shared in-memory cache keyed across tenants that
could leak one tenant's event metadata into another's handling.

## Boundaries / not-yet-enforced (honest)

- **Per-tenant rate limiting / quota** is not yet enforced at the gateway; a single
  tenant cannot reach another tenant's pipeline, but global gateway capacity is
  currently shared. Tracked for the production-hardening pass.
- The **reviewer-gate (ADR-0367) and merge-queue (ADR-0111)** dispatch legs return
  a typed `Unimplemented` (`501`) today (recorded in `registry/placeholder-debt`);
  their per-tenant ordering/isolation guarantees land when those downstreams are
  wired. The admission → `oya gate run-all` leg is the live, tenant-scoped path.

## Verification

- `signature.rs` RFC-4231 known-answer test + fail-closed bad-signature test
  (`401`) prove the HMAC trust boundary.
- `event.rs` closed-router test proves unroutable events do not reach dispatch.
- Cutover (`SETUP-RUNBOOK.md`) registers one secret per tenant; the dogfood tenant
  (oyatie) is provisioned identically to any external tenant.
