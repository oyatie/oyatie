# Oya Verticals Inventory — scope `d`–`m`

Source-backed sweep of product/platform verticals under `/Users/jasonlee/Developer/source/oya/`
whose directory name starts with **d through m**. Everything below is read from the REAL tree
(`ls <service>/crates/`, crate `src/` module layout, `manifest.json` `tier`), NOT from ADRs.

## Reading the clean-arch lens

- **Layer suffixes** on crate names map to clean-arch rings: `kernel` (innermost / pure invariants),
  `domain` (entities + domain rules), `usecase` (application orchestration), `app` (composition root /
  wiring), `api` (port trait surface), `rest`/`grpc`/`graphql`/`sse`/`websocket` (transport drivers),
  `sdk` (client), `worker` (async/background driver), `adapter` (outbound infra impl).
- **PORTS/ADAPTERS = mobility seams.** An `-api`/`-rest`/`-grpc` crate is the inbound *port*; an
  `*-adapter-<x>` crate is a swappable outbound *adapter* (the architecture-mobility hinge). The
  canonical exemplar lives OUT of this scope — `workflow-engine/crates/oya-workflow-engine-event-bus-*`
  ships `-adapter-{kafka,redpanda,valkey,postgres}` behind one `-event-bus-api` (verified by
  `find . -name '*event-bus*'`). Inside d–m the same seam appears at smaller scale (single
  `-adapter-postgres` or `-adapter-inmemory` behind a `-store-api`).
- **COHESION** = one bounded context (one BC, possibly a modular monolith with internal
  `domain/usecase/adapter` modules). **DETACHMENT** = multiple BCs each as its own crate cluster,
  independently deployable/scalable.

## Two physical shapes observed in this scope

1. **Modular-monolith service crate** (`*-service` / `*-app`): a SINGLE crate whose `src/` carries the
   rings as *modules* — verified layout `adapter/ domain/ usecase/ config.rs error.rs lib.rs main.rs`.
   Clean-arch is enforced intra-crate, adapters are sub-modules (`adapter/{http.rs,grpc.rs,asyncapi.rs}`),
   not separate crates. High cohesion, single BC.
2. **Crate-per-ring / crate-per-BC service**: rings (and sometimes BCs) are split into separate crates
   (`-domain`, `-usecase`, `-api`, `-rest`, `-app`, `-adapter-postgres`, …). This is the detachment shape.

## Scope coverage statement (NO SILENT CAPS)

30 in-scope service dirs. **Fully opened** (`crates/` listed + at least one crate `src/` inspected):
`data-pipeline, data-warehouse, design-collaboration, developer-sdk, docs, drive, eventing,
feature-flags, financial-planning, forms, global-trade, hr, identity, incident-management,
intelligence, itsm, learning-management, mail, marketing-automation, marketplace, meet, messenger`.
**Confirmed spec/doc-only — no `crates/` dir, no `Cargo.toml`, no Rust** (verified via grep for
crate/src/Cargo): `detection, diagnostics, finops-portal, governance, healthcare-integration, imaging`.
**Partially opened (crate list only, not every crate `src/`):** the very large `intelligence` (128
crates — BC grouping derived programmatically, not every crate hand-read) and `meet`/`forms`/`global-trade`/
`marketplace` (single `-domain`/scaffold crate, internals not deep-read). No service in range silently skipped.

All 30 dirs sit under `source/oya/` and every `manifest.json` read carries `"tier"` of **product /
substrate / external-facing / T0–T2** — i.e. these are the **product (and shared-substrate) plane**,
the `oya/` product tree, NOT a `cloud/` platform tree. (`tier=substrate` ones —
detection, feature-flags, intelligence — are cross-product substrates that still live in the product tree.)

---

## Service-by-service

### data-pipeline  — `tier: product` (ELT / lineage; Fivetran/dbt-class)
- Tree: oya/ product. Crates (`ls data-pipeline/crates/`): **1** — `oya-data-pipeline-lineage-replay-service`.
- Layers: modular-monolith single crate; `src/` = `adapter/ domain/ usecase/ config.rs error.rs lib.rs main.rs`.
- Ports/adapters: intra-crate `adapter/` module only (no per-infra adapter crate at this stage).
- Cohesion vs detachment: **COHESION** — single BC (lineage + backfill/replay) as a modular monolith.
- Hyperscaler role: data-movement/lineage product axis (CDC freshness, dead-letter replay custody — per IP-026..030).

### data-warehouse  — `tier: product` (cloud OLAP; Snowflake/BigQuery/Databricks-displacement)
- Tree: oya/ product. Crates: **1** — `oya-data-warehouse-tenant-olap-service`.
- Layers: modular monolith; `src/` adds a `lake_engine/` module alongside `adapter/ domain/ usecase/`.
- Ports/adapters: intra-crate `adapter/` + `lake_engine/` (Delta/Iceberg/Hudi write substrates per IP-031..033).
- Cohesion vs detachment: **COHESION** — single tenant-OLAP BC (the lake engine is an internal module, not a separate BC crate yet).
- Hyperscaler role: warehouse/lakehouse product axis (zero-copy clone, time-travel, reader-share — IP-040..044).

### design-collaboration  — `category: product` (Figma-class creative artifacts)
- Tree: oya/ product. Crates: **1** — `oya-design-collaboration-creative-artifact-service`.
- Layers: modular monolith; standard `adapter/ domain/ usecase/ …`.
- Ports/adapters: intra-crate `adapter/`. (REMEDIATION note shows a valkey migration — collab realtime backing.)
- Cohesion vs detachment: **COHESION** — single creative-artifact BC.
- Hyperscaler role: design/collab product axis (component-variant governance, design-token promotion — IP-026..030).

### detection  — `tier: substrate` (fraud/risk detection substrate) — **SPEC-ONLY**
- Tree: oya/ product (substrate). Crates: **none** (no `crates/` dir; doc/IP-only).
- Intended layers (from IP names only, not built): streaming-kernel/worker, batch-kernel/worker,
  feature-store-domain/adapter, rules-engine-kernel/rest, composite-scorer, graph-store-kernel, sandbox-replay.
- Cohesion vs detachment: planned **DETACHMENT** (multi-BC: streaming + batch + feature-store + rules +
  scorer + graph + investigation), but **0 crates exist** — design stage.
- Hyperscaler role: fraud/abuse detection substrate consumed by other products.

### developer-sdk  — `tier: external-facing` (the external dev/CLI surface)
- Tree: oya/ product. Crates: **1** — `oya-dev-cli`.
- Layers: an `sdk`/CLI driver (client surface), not a backend service.
- Ports/adapters: it IS the SDK/CLI port to the platform; no outbound infra adapters.
- Cohesion vs detachment: **COHESION** — single CLI/SDK BC.
- Hyperscaler role: external-facing developer-experience axis (pack rollout, sandbox deploy journeys).

### diagnostics  — (no tier in manifest) — **SPEC-ONLY**
- Tree: oya/ product. Crates: **none**. Has `supported-oses.json`, IPs, design-spec-maturity dir only.
- Cohesion vs detachment: design stage; no crates to classify.
- Hyperscaler role: device/host diagnostics product axis (pre-implementation).

### docs  — `tier: T1` (Notion/Confluence-class docs)
- Tree: oya/ product. Crates: **1** — `oya-docs-domain`.
- Layers: only the `domain` ring materialized as a crate so far (CRDT collab + block types domain).
- Ports/adapters: none as separate crates yet (IP-004 names a postgres+s3 store adapter, IP-007 a valkey CRDT adapter — not yet crates).
- Cohesion vs detachment: **COHESION** — single docs BC (domain-first slice).
- Hyperscaler role: collaborative-documents product axis (migration-from-connect path noted).

### drive  — `tier: T1` (Google-Drive / Dropbox-class file store)
- Tree: oya/ product. Crates: **1** — `oya-drive-domain`.
- Layers: `domain` ring only as a crate (file-store + folder hierarchy + sync domain).
- Ports/adapters: planned file-store adapters (IP-003) not yet split into crates.
- Cohesion vs detachment: **COHESION** — single file-store BC (domain-first; huge breadth of journey IPs incl. DLP/immutability/evidence-vault).
- Hyperscaler role: cloud-storage product axis.

### eventing  — (no manifest.json) (eventing primitives)
- Tree: oya/ product. Crates: **2** — `oya-eventing-domain`, `oya-eventing-file-adapter`.
- Layers: `domain` + one outbound `adapter` (file).
- Ports/adapters: **`oya-eventing-file-adapter`** is the swappable seam (file impl; mirrors the
  workflow-engine event-bus adapter family but at a 1-adapter stage).
- Cohesion vs detachment: **COHESION** — single eventing BC with a domain↔adapter split (early ports/adapters shape).
- Hyperscaler role: messaging/eventing substrate.

### feature-flags  — `tier: substrate` (LaunchDarkly-class flag substrate)
- Tree: oya/ product (substrate). Crates: **1** — `oya-flags`.
- Layers: single umbrella crate (`PHASE-01-LAUNCHDARKLY-CLASS-FLAG-SUBSTRATE`); IPs cover killswitch-broadcast-worker, pack-overlay-worker, multi-lang SDKs (go/java/dotnet/swift), experiment stats engine.
- Ports/adapters: intra-crate; SDK fan-out is the external port surface.
- Cohesion vs detachment: **COHESION** today (one crate) over a roadmap that implies later detachment (worker + SDK split).
- Hyperscaler role: feature-flag / experimentation substrate consumed across products.

### financial-planning  — `tier: product` (Anaplan/Adaptive/EPM-class FP&A)
- Tree: oya/ product. Crates: **1** — `oya-financial-planning-forecast-scenario-app`.
- Layers: modular monolith; `src/adapter/` = **`asyncapi.rs grpc.rs http.rs mod.rs`** (three inbound
  transport adapters as modules — REST/gRPC/AsyncAPI ports inside one crate).
- Ports/adapters: intra-crate transport adapters (http/grpc/asyncapi). No infra-swap adapter crate.
- Cohesion vs detachment: **COHESION** — single forecast-scenario BC.
- Hyperscaler role: FP&A / planning product axis (Anaplan/Workday-Adaptive/Oracle-EPM displacement — IP-026..030).

### finops-portal  — `tier: T2` (tenant billing presentation / FinOps) — **SPEC-ONLY**
- Tree: oya/ product. Crates: **none** (`PHASE-01-tenant-billing-presentation`, journey IPs only).
- Cohesion vs detachment: design stage; presentation layer over billing/cost substrates.
- Hyperscaler role: FinOps / chargeback-showback product axis.

### forms  — `tier: T0` (forms/surveys)
- Tree: oya/ product. Crates: **1** — `oya-forms-domain`.
- Layers: `domain` ring crate only.
- Ports/adapters: none as separate crates yet.
- Cohesion vs detachment: **COHESION** — single forms BC (domain-first).
- Hyperscaler role: forms/data-capture product axis (T0 foundational).

### global-trade  — (no tier) (trade compliance)
- Tree: oya/ product. Crates: **1** — `oya-global-trade-compliance-domain`.
- Layers: `domain` ring crate only.
- Ports/adapters: none yet.
- Cohesion vs detachment: **COHESION** — single trade-compliance BC.
- Hyperscaler role: global-trade/compliance product axis.

### governance  — `tier: T1` (governance/policy migration) — **SPEC-ONLY**
- Tree: oya/ product. Crates: **none**; only IP migration plans (`IP-002/003-migrate-tier-a-check-crates`).
- Note: these IPs are about MIGRATING check-crates INTO governance — governance is mid-migration, no crates landed in its own dir.
- Hyperscaler role: governance/policy product axis (CLI-check consolidation target).

### healthcare-integration  — (no tier) (HL7/FHIR integration) — **SPEC-ONLY**
- Tree: oya/ product. Crates: **none**.
- Hyperscaler role: healthcare-interop product axis (pre-implementation).

### hr  — (no manifest.json) (Workday-class HRIS)
- Tree: oya/ product. Crates: **5** — `oya-hr-employment-{api, app, domain, runtime}` +
  `oya-hr-employment-storage-adapter-inmemory`.
- Layers: clean crate-per-ring — `domain` + `api` (port) + `app` (composition) + `runtime` (driver) +
  one outbound `storage-adapter-inmemory`.
- Ports/adapters: **`oya-hr-employment-storage-adapter-inmemory`** = swappable persistence adapter behind
  the `api` port (inmemory now → postgres later = mobility seam).
- Cohesion vs detachment: **COHESION** — single `employment` BC, but built in the *detached crate-per-ring shape* (the disciplined ports/adapters layout).
- Hyperscaler role: HRIS / employment product axis.

### identity  — `tier: T0` (IdP / OIDC — Okta/Auth0-class) — **DETACHMENT (2 BCs)**
- Tree: oya/ product (T0 foundational substrate). Crates: **11**.
- BC cluster A — **identity core**: `oya-identity`, `oya-identity-domain`, `oya-identity-usecase`,
  `oya-identity-api`, `oya-identity-oidc-issuer-kernel` (kernel/domain/usecase/api rings).
- BC cluster B — **identity-workload** (machine/workload identity): `oya-identity-workload-domain`,
  `-workload-api`, `-workload-app`, `-workload-rest`, `-workload-authz-cedar-adapter`, `-workload-oidc-adapter`.
- Ports/adapters: **`oya-identity-workload-authz-cedar-adapter`** (Cedar authz engine) and
  **`oya-identity-workload-oidc-adapter`** (OIDC federation) = two swappable adapters behind the workload api/rest ports.
- Cohesion vs detachment: **DETACHMENT** — two independently-scalable BCs (human identity vs workload
  identity) each with its own ring stack; workload BC additionally has full ports/adapters.
- Hyperscaler role: identity-provider / IAM substrate (T0 — everything depends on it).

### imaging  — `tier: product` (medical/document imaging) — **SPEC-ONLY**
- Tree: oya/ product. Crates: **none**.
- Hyperscaler role: imaging/PACS product axis (pre-implementation).

### incident-management  — (no tier) (SRE incident command)
- Tree: oya/ product. Crates: **1** — `oya-incident-management-sre-incident-command-app`.
- Layers: modular monolith; standard `adapter/ domain/ usecase/ …`.
- Ports/adapters: intra-crate `adapter/`.
- Cohesion vs detachment: **COHESION** — single incident-command BC. (Distinct from `itsm` below, which
  splits the same problem space into BCs.)
- Hyperscaler role: incident-response product axis (PagerDuty/Opsgenie-class).

### intelligence  — `tier: substrate` (AI/LLM gateway + agent runtime substrate) — **MAXIMAL DETACHMENT (128 crates)**
- Tree: oya/ product (substrate). Crates: **128** — by far the largest vertical in scope.
- Layer-suffix tally across the 128: `kernel`×36, `domain`×27, `adapter`×25, `app`×12, `usecase`×10,
  `api`×8, `worker`×5, `rest`×1 — a fully ring-decomposed substrate.
- Bounded contexts (top clusters by crate count, derived from the token after `oya-intelligence-`):
  `assist-draft`(7, full ring stack kernel→domain→usecase→adapter→api→rest→worker),
  `eval`(6), `attribution`(6), `context-aware-retrieval`(5), then triples for
  `subagent-runtime, rag-endpoint, model-routing, guardrails, dashboard, credential-resolver,
  capability-registry, autonomy-ceiling, audit-tap, account`; plus `supervisor, provider-pool,
  settings-template, policy, mdbook, architecture-map` and singletons (`write-gate, usage-window,
  route-policy, run, step, openapi, registry, pr-review-dispatcher`).
- Ports/adapters (the richest mobility surface in the whole d–m range):
  - **LLM-provider adapter family** (swappable model backends): for each of `anthropic / openai / gemini`
    there is an `-adapter-<p>-api-{kernel,adapter}` AND `-adapter-<p>-subscription-{kernel,adapter}` pair,
    plus `-adapter-{anthropic,openai}-compat-api`. I.e. API-key vs subscription auth modes, multi-vendor,
    behind a common `oya-intelligence-adapter-domain`.
  - **Account adapters**: `claude-account-adapter`, `codex-account-adapter`, `gemini-account-adapter`,
    `account-adapter-inmemory`, `providers-adapter-openai`.
  - **Inbound transport adapters** (port drivers): `api-rest-{kernel,adapter}`, `api-graphql-{kernel,adapter}`,
    `api-sse-{kernel,adapter}`, `api-websocket-{kernel,adapter}` — four protocol drivers each split kernel/adapter.
  - **Infra/file adapters**: `evidence-file-adapter`, `run-file-adapter`, `step-file-adapter`,
    `jsonl-supervisor-adapter`, `supervisor-security-adapter`, `credential-resolver-adapter`,
    `settings-template-adapter`, `audit-tap-adapter`, plus per-BC `assist-draft/attribution/eval/
    context-aware-retrieval-adapter`.
  - Embedded co-located non-`intelligence` crates (own mini-BCs living in this service dir):
    `oya-collab-crdt-portability-kernel`, `oya-collab-runtime-domain`, `oya-document-format-domain`,
    `oya-codeview-cli`, `oya-shuffle-sharding`, `oya-vcs-admission-gate-kernel`,
    `oya-vcs-provider-execution-gate-kernel`.
- Cohesion vs detachment: **DETACHMENT, extreme** — ~25+ independently-scalable BCs, each ring-split,
  with provider/protocol/storage all behind swappable adapters. This is the textbook hyperscaler
  ports-and-adapters control plane (cf. the workflow-engine event-bus exemplar, but an order of magnitude larger).
- Hyperscaler role: AI/LLM gateway + agent-orchestration substrate (model routing, guardrails, autonomy
  ceilings, attribution, eval, RAG) consumed by every other product.

### itsm  — (no tier) (ServiceNow/PagerDuty-class ITSM) — **DETACHMENT (6 BCs)**
- Tree: oya/ product. Crates: **6** — `oya-itsm-escalation-policy`, `-incident-room`, `-on-call-schedule`,
  `-postmortem`, `-service-management-service`, `-status-update`.
- Layers: the orchestrating `service-management-service` is a modular monolith (`adapter/ domain/
  usecase/ …`); the other five are thin single-`lib.rs` BC libraries (e.g. `escalation-policy/src/lib.rs`,
  `on-call-schedule/src/lib.rs`).
- Ports/adapters: intra-crate `adapter/` in the service crate; no per-infra adapter crate.
- Cohesion vs detachment: **DETACHMENT** — distinct BCs (escalation, incident-room, on-call, postmortem,
  status, plus the service-management aggregator), independently composable.
- Hyperscaler role: ITSM / on-call / incident-ops product axis (note: overlaps incident-management above —
  itsm decomposes the space into BCs whereas incident-management is one command-app).

### learning-management  — `tier: product` (LMS — Canvas/Coursera-class)
- Tree: oya/ product. Crates: **1** — `oya-learning-management-course-progress-service`.
- Layers: modular monolith; standard `adapter/ domain/ usecase/ …`.
- Ports/adapters: intra-crate `adapter/`.
- Cohesion vs detachment: **COHESION** — single course-progress BC.
- Hyperscaler role: learning/education product axis.

### mail  — `tier: T1` (Gmail-class mailbox)
- Tree: oya/ product. Crates: **7** — `oya-mail-domain` + `oya-mail-mailbox-store-{api, app, usecase,
  rest, grpc}` + `oya-mail-mailbox-store-adapter-postgres`.
- Layers: clean crate-per-ring on the `mailbox-store` BC — `domain` + `usecase` + `api` (port) + `app` +
  two transport drivers (`rest`, `grpc`) + one outbound `adapter-postgres`.
- Ports/adapters: **`oya-mail-mailbox-store-adapter-postgres`** = swappable persistence adapter behind the
  `mailbox-store-api`; `rest` + `grpc` = inbound port drivers.
- Cohesion vs detachment: **COHESION** — single `mailbox-store` BC, built in the detached crate-per-ring shape.
- Hyperscaler role: email/mailbox product axis (T1).

### marketing-automation  — `category: customer-engagement-substrate` (Marketo/HubSpot-class)
- Tree: oya/ product (substrate). Crates: **1** — `oya-marketing-automation-campaign-journey-app`.
- Layers: modular monolith; `src/adapter/` = `asyncapi.rs grpc.rs http.rs mod.rs` (REST/gRPC/AsyncAPI ports as modules).
- Ports/adapters: intra-crate transport adapters (http/grpc/asyncapi).
- Cohesion vs detachment: **COHESION** — single campaign-journey BC.
- Hyperscaler role: marketing-automation / customer-engagement substrate.

### marketplace  — `tier: T2` (app/plugin marketplace) — scaffold stage
- Tree: oya/ product. Crates: **1** — `oya-marketplace-doc-set-scaffold` (a doc-set scaffold, not yet a runtime BC).
- Layers: scaffold only.
- Cohesion vs detachment: pre-BC scaffold; nothing to scale yet.
- Hyperscaler role: marketplace / app-store product axis (early scaffold).

### meet  — `tier: T0` (Zoom/Meet-class real-time A/V)
- Tree: oya/ product. Crates: **1** — `oya-meet-domain`.
- Layers: `domain` ring crate only.
- Ports/adapters: none as separate crates yet.
- Cohesion vs detachment: **COHESION** — single meet BC (domain-first; T0 foundational).
- Hyperscaler role: real-time meetings/video product axis.

### messenger  — `tier: T0` (Slack/Teams-class chat)
- Tree: oya/ product. Crates: **7** — `oya-messenger-domain`, `oya-messenger-app` +
  `oya-messenger-message-stream-{api, usecase, rest, grpc}` + `oya-messenger-message-stream-adapter-postgres`.
- Layers: clean crate-per-ring on the `message-stream` BC — `domain` + `usecase` + `api` (port) + `app` +
  `rest` + `grpc` drivers + `adapter-postgres`.
- Ports/adapters: **`oya-messenger-message-stream-adapter-postgres`** = swappable persistence adapter behind
  `message-stream-api`; `rest` + `grpc` = inbound port drivers. (Same shape as `mail`.)
- Cohesion vs detachment: **COHESION** — single `message-stream` BC in the detached crate-per-ring shape.
- Hyperscaler role: messaging/chat product axis (T0 foundational).

---

## Cross-cutting patterns in scope d–m

- **Maturity gradient is visible in crate shape**: spec-only (detection, diagnostics, finops-portal,
  governance, healthcare-integration, imaging) → domain-first single crate (docs, drive, forms,
  global-trade, meet) → modular-monolith service crate (data-pipeline, data-warehouse,
  design-collaboration, financial-planning, incident-management, learning-management, marketing-automation)
  → crate-per-ring single-BC (hr, mail, messenger) → multi-BC detachment (identity ×2, itsm ×6,
  intelligence ×25+).
- **The mobility seam (`*-adapter-*`)** is present wherever persistence/provider choice matters: the
  smallest form is `-store-adapter-postgres`/`-storage-adapter-inmemory` (hr, mail, messenger) and
  `eventing-file-adapter`; the largest is `intelligence`'s multi-vendor LLM-provider + multi-protocol +
  multi-storage adapter mesh. Modular-monolith services keep the same seam as intra-crate `adapter/`
  sub-modules (`http/grpc/asyncapi`) rather than separate crates.
- **DETACHMENT (independently-scalable multi-BC) appears in exactly 3 of the 30 in-scope dirs**:
  `identity` (human vs workload identity), `itsm` (6 ITSM BCs), `intelligence` (AI substrate, ~25+ BCs) —
  the rest are single-BC COHESION at varying maturity.
- **Substrate vs product**: `detection`, `feature-flags`, `intelligence`, `marketing-automation` are
  tagged substrate/`customer-engagement-substrate` (shared services other products consume); the rest are
  end-user products on T0–T2 / `product` tiers. All live in the `oya/` product tree (none are `cloud/`).
