# Oya Verticals Inventory — Service Dirs `a`–`c`

READ-ONLY architectural inventory. Source-of-truth = the REAL tree under
`/Users/jasonlee/Developer/source/oya/` (crate dirs, `Cargo.toml` package names,
`BUCK` files, `PRD.md` front-matter). NOT derived from ADRs.

**Clean-arch lens applied to crate-name suffixes:**
`-kernel` (pure core / invariants), `-domain` (entities + domain logic),
`-usecase` (application use-cases), `-app` (application/composition surface),
`-runtime` (process host), `-api`/`-rest`/`-grpc` (delivery PORTS),
`-sdk` (client), `-worker` (async consumer), `-*-adapter-*` / `-*-adapter`
(swappable infra IMPLEMENTATIONS = mobility seams).

**Scope:** all dirs in `ls /Users/jasonlee/Developer/source/oya` whose name starts
`a`, `b`, or `c`. (No dir starts with `b`.) 19 dirs in scope:
accounting, analytics, api-gateway, app-shell-frontend, application, audit-chain,
calendar, ci-controller, ci-tide, ci-webhook-gateway, comms-email, community,
compliance, connect, connector, consent-graph, contact-center,
contract-lifecycle-management, crm.

> **NO SILENT CAPS.** Every in-scope dir is treated below. Four dirs ship **zero
> Rust crates today** (spec/doc-only or non-Rust): `api-gateway`,
> `comms-email`, `consent-graph` (doc/spec packs, no `Cargo.toml` anywhere),
> and `app-shell-frontend` (TypeScript/pnpm frontend, not a Rust crate). They are
> listed but have no clean-arch crate layering to report yet.

All Rust crates below carry both `Cargo.toml` **and** `BUCK` (dual Cargo+Buck2
build), verified per-crate.

---

## Summary table

| Service | Type | Rust crates | Layering present | Ports/adapters | Cohesion vs Detachment | Product axis |
|---|---|---|---|---|---|---|
| accounting | oya/ product | 5 | kernel? domain/app/api/runtime + 1 adapter | storage-adapter-inmemory | **Cohesion** (single BC: journal) | vertical-industry (fin/ERP) |
| analytics | oya/ product | 5 | domain/usecase/app/api (+tenant-bootstrap-app) | none yet | **Cohesion** (single BC: analytics) | intelligence / saas-substrate |
| api-gateway | oya/ product (substrate) | 0 (doc-only) | — (PRD says routing/rate-limit/auth/abuse layered crates planned) | — | — | saas-substrate (`shared-substrate`) |
| app-shell-frontend | oya/ product (UI) | 0 (TS/pnpm) | — | — | — | workspace (UI shell) |
| application | oya/ product (suite glue) | 8 | domain/app (+ 4 workspace `-api` surfaces) | none (api = delivery ports) | **Detachment** (multi-BC suite seam: chat/drive/forms/meet) | workspace |
| audit-chain | cloud/-style platform svc (in oya/) | 18 | kernel+domain+api per sub-BC, shared usecase/domain | file-adapter | **Detachment** (5 BCs: emission, query, retention-cascade, sealing, verification) | saas-substrate (`shared-substrate`) |
| calendar | oya/ product | 1 | domain only | none yet | **Cohesion** (single BC) | workspace (`shared-substrate + suite-app`) |
| ci-controller | oya/ product (CI) | 4 | kernel/app + 2 adapters | github-adapter, k8s-adapter | **Cohesion** (single BC: CI control) | ci-cd-tooling |
| ci-tide | oya/ product (CI) | 3 | kernel/app + 1 adapter | github-adapter | **Cohesion** (single BC) | ci-cd-tooling |
| ci-webhook-gateway | oya/ product (CI) | 6 | kernel/app + 4 adapters | github, jenkins, ed25519, authz-cedar | **Cohesion** (single BC: webhook ingress) | ci-cd-tooling |
| comms-email | oya/ product | 0 (doc-only) | — | — | — | workspace (comms substrate) |
| community | oya/ product | 14 | full domain/usecase/app/api/rest/grpc x2 BCs + 2 adapters | post-store-adapter-postgres, social-post-composition-adapter-postgres | **Detachment** (2 BCs: post-store, social) | social / community |
| compliance | cloud/-style platform svc (in oya/) | 7 | domain (+1 usecase) across 5 BCs | none yet | **Detachment** (5 BCs: dlp, dsr, ediscovery, retention, trust-portal) | saas-substrate (governance) |
| connect | oya/ product | 1 | domain only | none yet | **Cohesion** (single BC: address-book) | workspace |
| connector | cloud/-style integration hub | 10 | **all adapters** (no core crates) | adp, epic-fhir, gusto, netsuite, quickbooks, rippling, salesforce, slack, teams, workday | **Detachment** (10 independent vendor adapters) | saas-substrate (integration) |
| consent-graph | oya/ product (privacy) | 0 (doc-only) | — | — | — | saas-substrate (governance/privacy) |
| contact-center | oya/ product | 1 | app only (voice-routing) | none yet | **Cohesion** (single BC) | vertical-industry (CX) |
| contract-lifecycle-management | oya/ product | 1 | app only (contract-obligation) | none yet | **Cohesion** (single BC) | vertical-industry (CLM) |
| crm | oya/ product | 3 | domain/app across 3 BCs | none yet | **Detachment** (3 BCs: crm-customer-engagement, crm-revenue, procurement-source-to-pay) | vertical-industry (CRM/SRM) |

---

## Per-service detail (crate lists cited from `ls <svc>/crates/`)

### accounting — `oya/` product (financial/ERP vertical)
Crates (`ls accounting/crates`):
- `oya-accounting-journal-domain` — **domain**
- `oya-accounting-journal-app` — **app** (composition surface)
- `oya-accounting-journal-api` — **api** (delivery PORT)
- `oya-accounting-journal-runtime` — **runtime** (process host)
- `oya-accounting-journal-storage-adapter-inmemory` — **adapter** (swappable persistence; inmemory impl ⇒ postgres/etc. can slot the same port)

Bounded context: **single** (`journal`). **Cohesion.**
Ports/adapters: 1 mobility seam = `storage-adapter-inmemory` (persistence port).
Axis: **vertical-industry** (accounting / ERP general-ledger).

### analytics — `oya/` product
Crates (`ls analytics/crates`):
- `oya-analytics-domain` — **domain**
- `oya-analytics-usecase` — **usecase**
- `oya-analytics-app` — **app**
- `oya-analytics-api` — **api** (PORT)
- `oya-analytics-tenant-bootstrap-app` — **app** (secondary composition: per-tenant OLAP bootstrap)

Bounded context: **single** analytics BC (tenant-bootstrap is an ops surface of the same BC). **Cohesion.** Full domain→usecase→app→api stack — most complete clean-arch ladder in scope after audit-chain/community.
Ports/adapters: none materialized yet (no `-adapter-*`).
Axis: **intelligence / saas-substrate** (OLAP analytics; PHASE-01-ANALYTICS-OLAP-BOOTSTRAP).

### api-gateway — `oya/` product (substrate); **doc/spec-only, 0 crates**
`ls api-gateway/crates` = empty; `find api-gateway -name Cargo.toml` = none.
PRD front-matter: `sales_segment: shared-substrate`, `tier: internal`. IP docs
(IP-002..IP-008) describe a *planned* routing kernel/domain/usecase/adapter/rest/grpc/worker
ladder + rate-limit-adapter-valkey + abuse-defence-adapter-wasm, but **none built yet**.
Axis: **saas-substrate** (edge/ingress gateway).

### app-shell-frontend — `oya/` product (UI shell); **non-Rust**
Contents: `package.json`, `pnpm-workspace.yaml`, `src/`, `app.config.ts`,
`MIGRATION-PLAN.md` — a TypeScript/pnpm frontend, no Rust crates.
Axis: **workspace** (the unified app shell / launcher UI).

### application — `oya/` product (suite glue / workspace seam)
Crates (`ls application/crates`):
- `oya-application-app` — **app**
- `oya-cloud-surface-domain` — **domain** (cloud surface model)
- `oya-saas-plugin-app` — **app** (plugin/extensibility surface)
- `oya-application-shell-frontend-prototype` — **app** (shell prototype, Rust side)
- `oya-workspace-chat-api` — **api** (PORT, chat BC)
- `oya-workspace-drive-api` — **api** (PORT, drive BC)
- `oya-workspace-forms-api` — **api** (PORT, forms BC)
- `oya-workspace-meet-api` — **api** (PORT, meet BC)

Bounded contexts: **multiple** — this is the aggregation seam exposing the chat /
drive / forms / meet workspace BCs behind one application surface. **Detachment**
(each `-api` is an independently routable workspace product surface).
Ports/adapters: the four `*-api` crates are delivery ports (no infra `-adapter-*`).
PRD `sales_segment: Enterprise`. Axis: **workspace**.

### audit-chain — platform service (`cloud/`-style, residing in `oya/`)
Crates (`ls audit-chain/crates`, 18) — clean-arch ladder **per bounded context**:
- shared: `oya-audit-chain-domain`, `oya-audit-chain-usecase`, `oya-audit-chain-file-adapter` (**adapter** — file sink, swappable)
- **emission** BC: `-emission-kernel`, `-emission-domain`, `-emission-api`
- **query** BC: `-query-kernel`, `-query-domain`, `-query-api`
- **retention-cascade** BC: `-retention-cascade-kernel`, `-retention-cascade-domain`, `-retention-cascade-api`
- **sealing** BC: `-sealing-kernel`, `-sealing-domain`, `-sealing-api`
- **verification** BC: `-verification-kernel`, `-verification-domain`, `-verification-api`

Bounded contexts: **5** (emission, query, retention-cascade, sealing, verification),
each with its own kernel+domain+api triad ⇒ classic **Detachment** (independently
scalable sub-services sharing one audit-chain domain + usecase). Best example in
scope of crate-per-BC inside a multi-BC service.
Ports/adapters: `file-adapter` (the audit sink port; pluggable storage).
PRD `sales_segment: shared-substrate`. Axis: **saas-substrate** (tamper-evident
audit log — governance plane).

### calendar — `oya/` product
Crates (`ls calendar/crates`): `oya-calendar-domain` — **domain** only.
Bounded context: **single**. **Cohesion** (earliest-stage; domain skeleton only).
Ports/adapters: none yet. PRD `sales_segment: shared-substrate + suite-app`.
Axis: **workspace**.

### ci-controller — `oya/` product (CI/CD tooling)
Crates (`ls ci-controller/crates`):
- `oya-ci-controller-kernel` — **kernel**
- `oya-ci-controller-app` — **app**
- `oya-ci-controller-github-adapter` — **adapter** (SCM port → GitHub)
- `oya-ci-controller-k8s-adapter` — **adapter** (executor port → Kubernetes)

Bounded context: **single** (CI control loop). **Cohesion.**
Ports/adapters: 2 mobility seams — `github-adapter` (SCM), `k8s-adapter` (runner backend).
Axis: **ci-cd-tooling** (dogfooded oya-ci control plane).

### ci-tide — `oya/` product (CI/CD tooling)
Crates (`ls ci-tide/crates`):
- `oya-ci-tide-kernel` — **kernel**
- `oya-ci-tide-app` — **app**
- `oya-ci-tide-github-adapter` — **adapter** (SCM port)

Bounded context: **single**. **Cohesion.** Ports/adapters: `github-adapter`.
Axis: **ci-cd-tooling**.

### ci-webhook-gateway — `oya/` product (CI/CD ingress)
Crates (`ls ci-webhook-gateway/crates`):
- `oya-ci-webhook-gateway-kernel` — **kernel**
- `oya-ci-webhook-gateway-app` — **app**
- `oya-ci-webhook-gateway-github-adapter` — **adapter** (provider: GitHub)
- `oya-ci-webhook-gateway-jenkins-adapter` — **adapter** (provider: Jenkins)
- `oya-ci-webhook-gateway-ed25519-adapter` — **adapter** (signature-verify crypto port)
- `oya-ci-webhook-gateway-authz-cedar-adapter` — **adapter** (authz policy port → Cedar)

Bounded context: **single** (webhook ingress/verify/route). **Cohesion** with a
**rich adapter ring** (4 swappable ports: 2 SCM providers, 1 crypto, 1 authz).
Strongest hexagonal-ports example in the CI family.
Axis: **ci-cd-tooling**.

### comms-email — `oya/` product; **doc/spec-only, 0 crates**
`ls comms-email/crates` = empty; no `Cargo.toml`. PRD status `Draft`, milestone
`PHASE-01-COMMS-EMAIL-SUBSTRATE`, many related ADRs. Axis: **workspace** (email/comms substrate).

### community — `oya/` product (social/community)
Crates (`ls community/crates`, 14) — **two bounded contexts**, each with a full ladder:
- **post-store** BC: `-post-store-domain`, `-post-store-usecase`, `-post-store-app`,
  `-post-store-api`, `-post-store-rest`, `-post-store-grpc`,
  `-post-store-adapter-postgres` (**adapter** — persistence port)
- **social** BC: `-social-domain`, `-social-app`, plus the
  `-social-post-composition-*` sub-stack: `-usecase`, `-api`, `-rest`, `-grpc`,
  `-social-post-composition-adapter-postgres` (**adapter** — persistence port)

Bounded contexts: **2** (post-store, social) — **Detachment**. Notably exposes
**both REST and gRPC** delivery ports per BC (dual-protocol), and a postgres
persistence adapter per BC.
Ports/adapters: `post-store-adapter-postgres`, `social-post-composition-adapter-postgres`
(2 persistence seams); rest+grpc = delivery ports.
Axis: **social / community**.

### compliance — platform service (`cloud/`-style governance, in `oya/`)
Crates (`ls compliance/crates`, 7) — domain crates across **5 bounded contexts**:
- `oya-dlp-domain` — **DLP** BC (domain)
- `oya-dsr-domain` + `oya-dsr-usecase` — **DSR** BC (domain + usecase)
- `oya-ediscovery-domain` — **eDiscovery** BC (domain)
- `oya-retention-domain` + `oya-retention-dsr-domain` — **Retention** BC (domain; retention-dsr bridge)
- `oya-trust-portal-domain` — **Trust Portal** BC (domain)

Bounded contexts: **5** (dlp, dsr, ediscovery, retention, trust-portal) ⇒
**Detachment** (independent compliance capabilities under one service). Early stage
(mostly domain crates; only DSR has a usecase; no app/api/adapters yet).
Axis: **saas-substrate (governance / compliance plane)**.

### connect — `oya/` product
Crates (`ls connect/crates`): `oya-address-book-domain` — **domain** only.
Bounded context: **single** (address-book). **Cohesion** (skeleton stage).
Axis: **workspace** (contacts/people directory).

### connector — integration hub (`cloud/`-style, in `oya/`)
Crates (`ls connector/crates`, 10) — **all adapters, no core crates**:
- `oya-connector-adp-adapter`, `-gusto-adapter`, `-rippling-adapter`, `-workday-adapter` (HR/payroll vendors)
- `oya-connector-netsuite-adapter`, `-quickbooks-adapter` (finance/ERP vendors)
- `oya-connector-salesforce-adapter` (CRM vendor)
- `oya-connector-slack-adapter`, `-teams-adapter` (chat vendors)
- `oya-connector-epic-fhir-adapter` (healthcare FHIR vendor)

Bounded contexts: this is the **canonical adapter-ring service** — 10 independent
swappable third-party connectors, each a self-contained mobility seam. **Detachment**
(each vendor adapter ships/scales independently against a shared connector port).
The single richest ports/adapters surface in the entire `a`–`c` scope.
Axis: **saas-substrate (integration / connectors)**.

### consent-graph — `oya/` product (privacy); **doc/spec-only, 0 crates**
`ls consent-graph/crates` = empty; no `Cargo.toml`. PRD `authority_tier: 2`,
`owner_team: axis-consent-graph`. Spec/cedar/arch docs only.
Axis: **saas-substrate (governance / consent & privacy plane)**.

### contact-center — `oya/` product (CX vertical)
Crates (`ls contact-center/crates`): `oya-contact-center-voice-routing-app` — **app** only
(Cargo desc: "Rust src scaffold for the Contact Center voice routing application surface").
Bounded context: **single** (voice-routing). **Cohesion** (app scaffold stage).
Axis: **vertical-industry (CX / contact-center)**.

### contract-lifecycle-management — `oya/` product (CLM vertical)
Crates (`ls contract-lifecycle-management/crates`):
`oya-contract-lifecycle-management-contract-obligation-app` — **app** only.
Bounded context: **single** (contract-obligation). **Cohesion** (app scaffold stage).
Axis: **vertical-industry (CLM)**.

### crm — `oya/` product (CRM/SRM vertical)
Crates (`ls crm/crates`):
- `oya-crm-customer-engagement-domain` — **domain** (CRM customer-engagement BC)
- `oya-crm-revenue-app` — **app** (CRM revenue BC; desc: "CRM revenue application surface")
- `oya-procurement-source-to-pay-domain` — **domain** (procurement source-to-pay BC — note: a *procurement/SRM* BC living inside the crm service dir)

Bounded contexts: **3** (crm-customer-engagement, crm-revenue, procurement-source-to-pay)
⇒ **Detachment**. The `oya-procurement-*` crate is a distinct SRM bounded context
co-located under `crm/` — a detachment/seam worth flagging (procurement may merit
its own service later).
Ports/adapters: none yet.
Axis: **vertical-industry (CRM + SRM/procurement)**.

---

## Cross-cut findings

**Ports/adapters (mobility seams) present in scope:**
- accounting: `storage-adapter-inmemory` (persistence)
- audit-chain: `file-adapter` (audit sink)
- ci-controller: `github-adapter`, `k8s-adapter` (SCM, runner)
- ci-tide: `github-adapter` (SCM)
- ci-webhook-gateway: `github-adapter`, `jenkins-adapter`, `ed25519-adapter`, `authz-cedar-adapter` (2 SCM, crypto, authz)
- community: `post-store-adapter-postgres`, `social-post-composition-adapter-postgres` (persistence x2)
- connector: **10 vendor adapters** (adp, epic-fhir, gusto, netsuite, quickbooks, rippling, salesforce, slack, teams, workday) — the richest seam set
- (analytics, calendar, compliance, connect, contact-center, contract-lifecycle-management, crm have no `-adapter-*` yet)

**Detachment cases (multi-BC, crate-per-BC, independently scalable):**
- **audit-chain** — 5 BCs (emission/query/retention-cascade/sealing/verification), each kernel+domain+api
- **compliance** — 5 BCs (dlp/dsr/ediscovery/retention/trust-portal)
- **community** — 2 BCs (post-store/social), dual REST+gRPC per BC
- **application** — 4 workspace surfaces (chat/drive/forms/meet) behind one app
- **crm** — 3 BCs (customer-engagement/revenue/procurement-source-to-pay)
- **connector** — 10 detached vendor adapters

**Cohesion cases (single-BC):** accounting, analytics, calendar, ci-controller,
ci-tide, ci-webhook-gateway, connect, contact-center, contract-lifecycle-management.

**Maturity ladder (deepest clean-arch stacks):** audit-chain (kernel→api x5) and
community (domain→usecase→app→api→rest→grpc x2) are the most complete; many
verticals (calendar, connect, contact-center, CLM, crm-revenue) are domain- or
app-skeleton scaffolds only.

**Product axes represented in `a`–`c`:**
- **saas-substrate:** api-gateway, audit-chain, compliance, connector, consent-graph
- **workspace:** app-shell-frontend, application, calendar, comms-email, connect
- **vertical-industry:** accounting, contact-center, contract-lifecycle-management, crm
- **intelligence:** analytics
- **ci-cd-tooling:** ci-controller, ci-tide, ci-webhook-gateway
- **social/community:** community
- (no **search** or **ads** vertical falls in the `a`–`c` range)
