# Auth / Onboarding / Session — E2E ground-truth audit (2026-06-19)

Founder spot-check ("is first-time-setup / tenant reg / account reg / passkey CRUD-login / session handling done E2E (UX) + wired?"). Read-only audit (Workflow w88azc5n4, 8 agents). Founder ruling: **NOT a work order — keep reorg/delivery rolling; record to backlog.** This is the actionable backlog for when auth/onboarding delivery (G005 identity + onboarding verticals) is prioritized.

## Headline
**No flow is done end-to-end with UX. A real user cannot onboard, register, enroll a passkey, log in, or hold a session today.** Strong, tested kernel/domain logic exists but dead-ends at three boundaries every flow shares:
1. **Identity service never starts a server** — `oya/identity/.../main.rs:9` is a no-op (`observability::init()` + `config::load()` → `Ok(())`); no axum router/listener.
2. **No REST handler crates** wiring kernels → HTTP for human auth (the WebAuthn relying-party REST crate does not exist).
3. **No login/signup/passkey UI or auth-gate** — the Leptos shell is post-auth-only, renders the dashboard for ANY context (no gate), populated with hardcoded mock data (e.g. tenant_name="Northwind Industrial Group"). Onboarding wizard buttons are `data-*` attrs with zero event handlers + no API to POST to.
Stores are in-memory `BTreeMap`/`MemoryStore` only (no Postgres/Valkey adapters). `finish_authentication` returns a `Credential` with no path to mint a session.

## Per-flow state (all PARTIAL — real kernel, broken delivery chain)
| Flow | state | UX | most damning gap |
|---|---|---|---|
| First-time setup/bootstrap | PARTIAL | mock-only | 9-step onboarding wizard is a static HTML snapshot of mock data; buttons unwired; no provision→invite→setup bridge |
| Tenant registration | PARTIAL | none | adapter-postgres/rest/sdk/app/worker crates MISSING; FSM real but in-memory only; `oya-tenancy-sdk` doesn't exist |
| Account registration | PARTIAL | none | service never starts (main.rs no-op) + in-memory store + no signup form; ~95% of service crate = TODO(ADR-0476) 5-line stubs |
| Passkey CRUD+login (WebAuthn) | PARTIAL | none | `oya-identity-webauthn-relying-party-rest` crate ABSENT; kernel real (10/10 tests) but no HTTP, no store, no session mint |
| Session handling | PARTIAL | none | no `/token` `/refresh` `/logout` + no bearer-auth middleware for humans; RefreshRequest validator exists, nothing calls it |

## WIRED & tested (real, but in-memory + no HTTP/UI trigger)
- `libs/oya-shared-webauthn-server-kernel` (686 ln, 10/10 integ tests: register→auth, AAGUID allowlist, sign-count regression)
- `oya-identity-oidc-issuer-kernel` (2027 ln, JWKS rotation, token claims) — no adapter/HTTP
- `oya-shared-oidc-client-kernel` verifier (sig/exp/aud/iss/kid/ACR)
- shell `token_broker.rs` (deny-by-default, ACR step-up) — server-side only, client can't call it
- `oya-tenancy-tenant-lifecycle-{kernel,domain,usecase}` FSM (14 conformance tests) — in-memory
- `oya-identity-workload-rest` PEP (real axum: /authorize, /tokens/validate, etc.) — **workload/machine auth, NOT human login**
- cloud STS `assume_role` (`oya-cloud-iam-domain`) — no persistence/human trigger
- identity domain/api/usecase contracts (typed, unit-tested)

## DESIGNED-only (specs/ADRs/reference docs, no impl)
tenant provisioning SDK (provision-tenant-rust-sdk.md `.invite_initial_admin()`); WebAuthn REST IP-005 (6 endpoints + Postgres + Valkey schema); tenancy REST/SDK IP-010 + Postgres IP-005; j21/j33 journey docs + ux-flows (75 template rows) + integration-test-plans (60+ named cases, 0 impl); 9-step onboarding wizard.

## STUBBED (TODO(ADR-0476) 5-line placeholders)
`oya-identity/src/`: auth, oidc, oauth2, users, realms, storage, rest, grpc, observability mods; main.rs wiring; webauthn/mod.rs::init().

## MISSING (referenced by phase plans/catalog, no crate — verified)
`oya-identity-webauthn-relying-party-rest` (+ postgres/valkey stores); `oya-tenancy-tenant-lifecycle-{adapter,adapter-postgres,rest,sdk,app,worker}`; `oya-tenancy-sdk`; login/signup/passkey-ceremony UI + auth gate; bearer-token middleware + `/token` `/refresh` `/logout`; any E2E/journey test impl.

## Top gaps to close (priority order) — backlog for auth/onboarding delivery
1. **Start the identity server** — implement `oya/identity/.../main.rs` (axum Router + listener + composition root); nothing downstream is reachable until this exists.
2. **Build `oya-identity-webauthn-relying-party-rest`** (IP-005: register/start+finish, authenticate/start+finish, GET/DELETE credentials) — highest-leverage missing crate for passkey + login.
3. **Wire assertion → session mint** — connect kernel `finish_authentication` → `oya-identity-oidc-issuer-kernel` to issue {id_token, refresh_token}.
4. **Real credential + session stores** — Postgres/Valkey adapters replacing in-memory BTreeMap.
5. **Tenancy delivery crates** — adapter-postgres/rest/sdk/app/worker + oya-tenancy-sdk; bridge provision→invite-initial-admin→first-run setup.
6. **Bearer-auth middleware + `/token` `/refresh` `/logout`** for human sessions.
7. **Frontend** — login/signup/passkey-ceremony UI components + an auth gate on the shell; wire the onboarding-wizard `data-*` buttons to real APIs; replace mock data with live fetches.
8. **E2E/journey tests** — implement the designed integration-test-plans (currently 0 impl).

Full transcript: /private/tmp/claude-501/-Users-jasonlee-Developer-oyatie/a4d4d10d-e9e2-4fc6-b3d5-f3f126df558c/tasks/w88azc5n4.output

NOTE: this is the PRODUCT-delivery state, orthogonal to the G011 reorg (which relocates crates, behavior-preserving). The reorg makes these crates easier to find/own; it does not implement them. When auth/onboarding delivery is prioritized, the catalog-completeness gap (task #70, ~138 uncataloged live crates incl identity/tenancy) + these gaps are the work.
