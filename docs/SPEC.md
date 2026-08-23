---
purpose: Oyatie — System Specification (SPEC)
doc_status: published
---

# Oyatie — System Specification (SPEC)

## Constitutional authority — [CONSTITUTION.md](CONSTITUTION.md)


> **Status:** Draft v0.1 — 2026-05-09. Per-axis surface enumeration with one-line invariants per surface. The authoritative spec; every public surface has a row.
> **Owner:** `platform-api-sdk` (cross-axis API stability) + per-axis team (per-axis surfaces).
> **Companion:** [PRD.md](PRD.md), [DESIGN.md](DESIGN.md), per-product PRDs in [`products/`](products/), `contracts/openapi/**/*.yaml` + `contracts/**/*.proto` + `contracts/asyncapi/*.yaml`, [`machine-readable/contracts.json`](machine-readable/contracts.json).

---

## 1. Reading guide

Each axis has a §-section enumerating its **surfaces**. A surface is a publicly observable contract with:

- **id** — stable identifier
- **plane** — control / data / analytics
- **kind** — REST / gRPC / event / websocket / file / mcp-tool / etc.
- **invariant** — one-line guarantee (the contract)
- **stability tier** — preview / stable / GA per ADR-0040
- **owning crate** — flat-crates target
- **regulatory pack** — applicable per [COMPLIANCE-MATRIX.md](COMPLIANCE-MATRIX.md)

The full machine-readable form is at [`machine-readable/contracts.json`](machine-readable/contracts.json) and `contracts/`. This doc is the human-readable index.
The first active OpenAPI source contract is [`contracts/openapi/foundry/capability-v1.yaml`](../contracts/openapi/foundry/capability-v1.yaml), covering the `foundry.capability.invoke` REST surface. Its operation-level runtime backing is declared in [`registry/openapi/runtime-bindings.tsv`](../registry/openapi/runtime-bindings.tsv) at the `intelligence-api` inbound boundary, where path/body `capability_id` drift is rejected before Foundation orchestration, reused idempotency keys with different request fingerprints fail as explicit `422` errors, and the typed API status mapping stays constrained to the documented explicit `202`, `400`, `403`, and `422` responses. Runtime-bound operations intentionally reject OpenAPI `default` and `1XX`-through-`5XX` response ranges until a future binding can prove their concrete status vocabulary; typed API status mappings are proven from fieldless enum variants and explicit `Self::Variant => <status>` code arms. Runtime-bound responses require concrete `application/json` schema refs, and the runtime binding declares the exact status-to-schema map so `202` stays bound to the success envelope and `400`/`403`/`422` stay bound to the error envelope. The `202` success envelope is bound to `CapabilityInvokeApiSuccessResponse` and `CapabilityInvokeApiResponseMetadata`, while the `400`/`403`/`422` error envelope is bound to `CapabilityInvokeApiErrorResponse`, `CapabilityInvokeApiErrorBody`, and `CapabilityInvokeApiErrorDetail` in [`registry/openapi/schema-bindings.tsv`](../registry/openapi/schema-bindings.tsv). Its request/receipt schema shape plus scalar type/format parity is declared in the same schema registry.

### 1.1 Glossary cross-doc coverage anchors

This table anchors active glossary terms to product-surface families for the `glossary-cross-doc-coverage` lane. It is a terminology index only; it does not introduce additional product scope beyond the surface rows below.

| Term | Specification anchor |
|---|---|
| MTTD | Incident and observability surfaces report Mean Time To Detect alongside MTTA and MTTR. |
| NACL | Cloud network policy surfaces model Security Group and NACL controls separately. |
| 2FA | Identity surfaces support MFA / 2FA enrollment and recovery flows. |
| DSAR | Privacy surfaces route DSAR exports through the same DSR cascade evidence chain. |
| featurestore | Analytics and ML surfaces consume the featurestore through purpose-bound ports. |
| DSP | Ads serving surfaces expose DSP-facing auction integration boundaries. |
| SSP | Ads serving surfaces expose SSP-facing inventory integration boundaries. |
| VAST | Video ad surfaces support VAST creative validation before campaign publish. |
| VPAID | Video ad surfaces reject unsafe VPAID execution paths unless explicitly sandboxed. |
| OpenRTB | Programmatic ad exchange surfaces use OpenRTB-shaped bid requests and responses. |
| frequency capping | Ads campaign surfaces enforce frequency capping per user, cohort, and tenant. |
| Band (P0-P20) | Planning and issue-routing surfaces preserve Band (P0-P20) as the backlog priority vocabulary. |
| 휴일/야간 근로 | KR corporate payroll surfaces compute holiday / night work premiums. |
| 실명인증 | KR onboarding surfaces bind 실명인증 proof to regulated identity records. |
| 사업자등록 | KR billing and tax surfaces validate 사업자등록 metadata before invoice issuance. |
| SOC2 Type II | Trust and compliance surfaces map SOC2 Type II controls to audit evidence exports. |
| EDI 940 | Logistics vertical surfaces include EDI 940 warehouse shipping order exchange. |
| EDI 944 | Logistics vertical surfaces include EDI 944 warehouse stock transfer receipt exchange. |

---

## 2. Cross-cutting platform surfaces

Owned by M01 foundation teams. Consumed by every axis; runtime crate names follow BNF v4.1 live workspace names, while external contract paths keep their stability-preserving platform URLs.

| id | plane | kind | invariant | tier | crate |
|---|---|---|---|---|---|
| `tenant.create` | control | REST | tenant_id is globally unique; region binding immutable post-create; OpenAPI source `contracts/openapi/platform/platform-tenant-v1.yaml` | stable | `tenancy-kernel` |
| `tenant.dsr.cascade` | control | REST + event | DSR cascade completes ≤ 30d (preview) / 14d (stable) / 7d (GA); proof-of-erasure per affected store | stable | `dsr-application` |
| `identity.user.upsert` | control | REST | per-tenant unique by primary identifier; per-region IdP binding; OpenAPI source `contracts/openapi/platform/platform-identity-user-v1.yaml` | stable | `platform-identity-api` |
| `identity.token.issue` (STS) | control | REST | short-lived (≤ 1h) per-purpose-bound credentials; never long-lived API key; OpenAPI source `contracts/openapi/platform/platform-identity-token-v1.yaml` | stable | `identity-application` |
| `cedar.policy.publish` | control | REST | versioned; semver; per-tenant-or-global scope; superseded-by chain; OpenAPI source `contracts/openapi/platform/platform-policy-cedar-v1.yaml` | stable | `platform-policy-cedar-api` |
| `audit.event.emit` (per ADR-0003) | data | event | append-only; SHA-256 hash-chained; Merkle-rooted; Ed25519-signable; per-tenant-shard; AsyncAPI source `contracts/asyncapi/platform/audit-events-v1.yaml`; Protobuf source `contracts/proto/platform/audit/v1/audit-event-v1.proto` | stable | `audit-chain-application` |
| `eventing.outbox.publish` | data | event | exactly-once via outbox + Kafka per ADR-0046; AsyncAPI source `contracts/asyncapi/platform/eventing-outbox-v1.yaml`; Protobuf source `contracts/proto/platform/eventing/v1/eventing-outbox-v1.proto` | stable | `eventing-domain` (superseded: `eventing-application` stub orphan deleted per ADR-0106 §Consequences + audit #6; canonical `-app` scaffold pending M01-P04) |
| `object-graph.entity.upsert` | data | REST + event | engine-enforced row-level isolation per ADR-0006; data_class annotated; OpenAPI source `contracts/openapi/platform/platform-object-graph-v1.yaml`; implemented by the Ontology domain per ADR-0055 | stable | `ontology-domain` |
| `object-graph.property.{vector,timeseries,geo,ciphertext,struct}` | data | REST | per-property-tier semantics per ADR-0006..0112; implemented by the Ontology domain per ADR-0055 | preview | `ontology-domain` |
| `dsr.cascade.execute` | data | REST | ≤ 30d (preview); proof-of-erasure across all data-class-touching stores; OpenAPI source `contracts/openapi/platform/platform-dsr-v1.yaml` | stable | `dsr-application` |
| `consent.receipt.emit` | control | event | per-purpose × per-data-class × per-tenant × per-subject grant; revocable; per [PRIVACY-PROGRAM §2.2.2](PRIVACY-PROGRAM.md) | stable | `platform-consent-app` |
| `webhook.delivery.signed` | data | webhook | rotating-key signed; retry-with-backoff; replay endpoint | stable | `platform-webhook-app` |
| `metering.event.ingest` | data | event | per-resource per-tenant; idempotency key; AsyncAPI source `contracts/asyncapi/platform/metering-events-v1.yaml`; Protobuf source `contracts/proto/platform/metering/v1/metering-event-v1.proto` | stable | `platform-metering-app` |
| `regulatory-pack.bind` | control | REST | per-tenant pack binding; multi-pack supported (rare); residency immutable post-bind; OpenAPI source `contracts/openapi/platform/platform-regulatory-pack-v1.yaml` | stable | `platform-regulatory-pack-api` |

## 3. Axis 1 — SaaS surfaces

| id | plane | kind | invariant | tier | crate |
|---|---|---|---|---|---|
| `workflow.definition.publish` | control | REST | versioned; semver per ADR-0035; jurisdiction overlay per regional pack | stable | `saas-workflow-api` |
| `workflow.run.start` | data | REST | per-tenant cell-routed; idempotent | stable | `saas-workflow-app` |
| `workflow.run.event` | data | event | per-step audit emission per ADR-0003 | stable | (per workflow runtime) |
| `plugin.manifest.register` | control | REST | per ADR-0036 schema; Cosign-signed per ADR-0039 | stable | `saas-plugin-marketplace-api` |
| `plugin.invocation` | data | REST + WASM | Wasmtime-sandboxed per ADR-0023; capability-gated PluginContext | stable | `saas-plugin-runtime-app` |
| `marketplace.listing.publish` | control | REST | per-vertical / per-region filterable; trust-tier per ADR-0036 | stable | `saas-marketplace-api` |
| `bench.surface.render` | data | REST + websocket | per-tenant shell renderer per ADR-0017 | stable | `saas-bench-app` |

## 4. Axis 2 — Workspace surfaces

| id | plane | kind | invariant | tier | crate |
|---|---|---|---|---|---|
| `workspace.mail.smtp.receive` | data | SMTP | RFC 5321; per-tenant routing; phishing+DLP+classify before store | stable | `workspace-mail-api` |
| `workspace.mail.imap` | data | IMAP | RFC 3501; per-tenant; per-folder access | stable | `workspace-mail-api` |
| `workspace.mail.jmap` | data | JMAP | RFC 8620; per-tenant | stable | `workspace-mail-api` |
| `workspace.calendar.caldav` | data | CalDAV | RFC 4791; per-tenant | stable | `workspace-calendar-api` |
| `workspace.docs.crdt.connect` | data | websocket | Yrs CRDT; per-doc state-vector compatibility ≥ 2 versions | stable | `workspace-docs-api` |
| `workspace.drive.put` | data | REST | per-object KMS-shred (per record DEK); per-permission ACL; OpenAPI source `contracts/openapi/workspace/workspace-drive-v1.yaml` | stable | `workspace-drive-api` |
| `workspace.drive.get` | data | REST | per-tenant cell-routed; signed-URL OK; audit-emit per access; OpenAPI source `contracts/openapi/workspace/workspace-drive-v1.yaml` | stable | `workspace-drive-api` |
| `workspace.meet.session.start` | control | REST | per-tenant SFU placement; per-region; OpenAPI source `contracts/openapi/workspace/workspace-meet-v1.yaml` | stable | `workspace-meet-api` |
| `workspace.meet.recording.archive` | data | event | KMS-shred per recording; trust-portal-only access | stable | `workspace-meet-recording-app` |
| `workspace.chat.message.send` | data | websocket + REST | per-tenant per-channel; threading per RFC; OpenAPI source `contracts/openapi/workspace/workspace-chat-v1.yaml` | stable | `workspace-chat-api` |
| `workspace.forms.submission.ingest` | data | REST + event | routed into Ontology (legacy: Object Graph — renamed per MASTERPLAN.md §2.4); per-form schema; OpenAPI source `contracts/openapi/workspace/workspace-forms-v1.yaml` | stable | `workspace-forms-api` |
| `workspace.address-book.carddav` | data | CardDAV | RFC 6352; per-tenant | stable | `workspace-address-book-api` |
| `workspace.translate.invoke` | data | REST | Foundry-routed per provider; per-pack language coverage | preview | `workspace-translate-api` |

## 5. Axis 3 — Vertical surfaces

Per-vertical surface set; each vertical's PRD §4.3 enumerates. Examples:

- `vertical-healthcare.fhir.{read,write}` — FHIR R4 server (read first, write at stable)
- `vertical-healthcare.hl7.v2.ingest` — HL7 v2 messaging
- `vertical-healthcare.dicom.exchange` — DICOM imaging exchange
- `vertical-healthcare.x12.{270,271,278,837,835}` — eligibility/auth/claim/remittance
- `vertical-fintech.payment.charge` — per [`standards/fintech-compliance.md`](standards/fintech-compliance.md); CDE in scope
- `vertical-fintech.kyc.onboard` — per regional pack identity
- `vertical-industrial.opcua.subscribe` — OPC UA telemetry
- `vertical-industrial.mes.workorder` — MES work order
- `vertical-logistics.edi.{214,990,997}` — logistics EDI signals
- `vertical-corporate.payroll.close` — KR statutory + per-region
- `vertical-legal.corpus.search` — regulated corpus per ADR-0033

(Per-vertical full enumeration in each `products/vertical-<id>/PRD.md` §4.3.)

## 6. Axis 4 — Foundry surfaces

| id | plane | kind | invariant | tier | crate |
|---|---|---|---|---|---|
| `foundry.capability.invoke` | data | REST + MCP | autonomy-tier-gated; data-class-class-allowlisted; evidence-emitted | preview | `intelligence-api` |
| `foundry.capability.publish` | control | REST | per [`templates/capability-record-template.yaml`](templates/capability-record-template.yaml); eval-set pass required; OpenAPI source `contracts/openapi/foundry/registry-v1.yaml` | stable | `intelligence-registry-api` |
| `foundry.adapter.{anthropic,openai,gemini}.{api,subscription}.invoke` | data | provider-bound | provider-failover-supported; cost-ceiling-enforced | stable | `intelligence-adapter-{anthropic,openai,gemini}-{api,subscription}-*` |
| `foundry.policy.autonomy-ceiling.publish` | control | REST | Cedar-backed; per-tenant per-capability scope; OpenAPI source `contracts/openapi/foundry/policy-v1.yaml` | stable | `intelligence-policy-api` |
| `foundry.evidence.emit` | data | event | every capability invocation emits to `oyatie.foundry.capability.invoked`; audit-chain anchored | stable | `intelligence-evidence-app` |
| `foundry.rag.retrieve` | data | REST | per-tenant boundary; per-class allowlist; consent-receipt cited; OpenAPI source `contracts/openapi/foundry/rag-v1.yaml` | stable | `intelligence-rag-api` |
| `foundry.eval.run` | analytics | REST | per-capability golden-set evaluation; pass-threshold per capability; OpenAPI source `contracts/openapi/foundry/eval-v1.yaml` | stable | `intelligence-eval-application` |
| `foundry.sandbox.spawn` | data | REST | Wasmtime / Firecracker; per-tool resource caps; per-agent worktree | stable | `intelligence-sandbox-app` |
| `foundry.cli` (`oya dev/admin/build/agent/ops/pack/catalog/gate`) | control | CLI + MCP | persona-split per [DESIGN §13.4.1](DESIGN.md) | stable | `intelligence-cli-{persona}-*` |
| `foundry.mcp-server` | control | MCP | exposes every CLI subcommand as MCP tool per [TOOLCHAIN §4.A](TOOLCHAIN.md) | stable | `intelligence-mcp-server-app` |
| `foundry.catalog.{record,validate,promote,supersede}` | control | REST + CLI | flat-crates target catalog per ADR-0015/0222 | stable | `intelligence-catalog-app` |
| `foundry.gate.{claim-ceiling,foundation-bypass,plane-class}` | control | CI gate | mechanical pre-merge enforcement | stable | `governance-gate-{claim,bypass,plane}-*` |
| `foundry.scorecard.publish` | analytics | REST | per Maturity Move #7 | stable | `governance-scorecard-app` |
| `foundry.fitness.run` | control | CI lane | per Maturity Move #8 + cross-axis cohesion check per Foundry-improvements top-20 #13 | stable | `governance-*` |
| `foundry.model.train` (long-horizon W-AI-Model-Substrate) | data | distributed | per-capability training; per-purpose data binding per Data Use Boundary | preview | `intelligence-model-train-app` |
| `foundry.model.serve` | data | gRPC | in-house model inference per [DESIGN §3.0.1](DESIGN.md) | preview | `intelligence-model-serve-app` |

## 7. Axis 5 — Cloud surfaces

| id | plane | kind | invariant | tier | crate |
|---|---|---|---|---|---|
| `cloud.iam.role.create` | control | REST | per-tenant per-region; Cedar policy attached; OpenAPI source `contracts/openapi/cloud/cloud-iam-v1.yaml` | stable | `cloud-iam-api` |
| `cloud.iam.sts.token` | control | REST | short-lived (≤ 1h); per-purpose bound; OpenAPI source `contracts/openapi/cloud/cloud-iam-v1.yaml` | stable | `cloud-iam-api` |
| `cloud.kms.encrypt` / `cloud.kms.decrypt` | data | REST | KCMVP HSM (KR) / FIPS 140-3 (global); per-tenant key; OpenAPI source `contracts/openapi/cloud/cloud-kms-v1.yaml` | stable | `cloud-kms-api` |
| `cloud.region.list` | control | REST | provider-facing static regional pack list; immutable; OpenAPI source `contracts/openapi/cloud/cloud-region-v1.yaml` | stable | `cloud-region-api` |
| `cloud.az.list` | control | REST | per-region AZ enumeration; OpenAPI source `contracts/openapi/cloud/cloud-region-v1.yaml` | stable | `cloud-region-api` |
| `cloud.cell.bind` | control | REST | per-tenant cell-routing assignment; OpenAPI source `contracts/openapi/cloud/cloud-cell-bind-v1.yaml` | preview | `cloud-region-api` (superseded: `cloud-cell-application` stub orphan deleted per ADR-0106 §Consequences + audit #6; canonical `-app` scaffold pending M02-P18) |
| `cloud.compute.vm.create` | control | REST | per-region per-cell; per-tenant quota; OpenAPI source `contracts/openapi/cloud/cloud-compute-vm-v1.yaml` | stable | `cloud-compute-vm-api` |
| `cloud.compute.k8s.cluster.create` | control | REST | managed control plane; per-tenant; per-region; OpenAPI source `contracts/openapi/cloud/cloud-compute-k8s-v1.yaml` | stable | `cloud-compute-k8s-api` |
| `cloud.compute.functions.invoke` | data | REST | cold-start budget; per-tenant; per-region; OpenAPI source `contracts/openapi/cloud/cloud-compute-functions-v1.yaml` | stable | `cloud-compute-functions-api` |
| `cloud.storage.object.put` / `.get` | data | REST | per-object KMS-shred; per-bucket lifecycle; OpenAPI source `contracts/openapi/cloud/cloud-storage-object-v1.yaml` | stable | `cloud-storage-object-api` |
| `cloud.storage.block.create` | control | REST | per-region per-AZ; per-tenant quota; OpenAPI source `contracts/openapi/cloud/cloud-storage-block-v1.yaml` | stable | `cloud-storage-block-api` |
| `cloud.network.vpc.create` | control | REST | per-tenant; per-region; OpenAPI source `contracts/openapi/cloud/cloud-network-vpc-v1.yaml` | stable | `cloud-network-vpc-api` |
| `cloud.network.lb.create` | control | REST | L4 + L7; per-region; mTLS-supported; OpenAPI source `contracts/openapi/cloud/cloud-network-lb-v1.yaml` | stable | `cloud-network-lb-api` |
| `cloud.network.dns.zone.create` | control | REST | per-tenant; per-region; OpenAPI source `contracts/openapi/cloud/cloud-network-dns-v1.yaml` | stable | `cloud-network-dns-api` |
| `cloud.billing.event.ingest` | data | event | per-resource per-tenant; idempotent; AsyncAPI source `contracts/asyncapi/cloud/cloud-billing-events-v1.yaml`; Protobuf source `contracts/proto/cloud/billing/v1/cloud-billing-event-v1.proto` | stable | `cloud-billing-app` |
| `cloud.billing.invoice.generate` | analytics | REST | per-region tax-invoice format (KR 전자세금계산서, JP 適格請求書, EU per-country, IN GST, BR NF-e, KSA FATOORA, UAE); OpenAPI source `contracts/openapi/cloud/cloud-billing-invoice-v1.yaml` | preview | `cloud-billing-kernel` (superseded: `cloud-billing-tax-application` stub orphan deleted per ADR-0106 §Consequences + audit #6; canonical `-app` scaffold pending M03 cloud-billing) |
| `cloud.observability.audit.read` | analytics | REST | per-tenant per-control-plane mutation log; CloudTrail-class; OpenAPI source `contracts/openapi/cloud/cloud-observability-audit-v1.yaml` | stable | `cloud-observability-api` |
| `cloud.finops.report` | analytics | REST | per-tenant per-axis cost; anomaly detection; OpenAPI source `contracts/openapi/cloud/cloud-finops-report-v1.yaml` | stable | `cloud-finops-api` |
| `cloud.dcops.{dcim,bms,power,cooling,network,security,asset,capacity,workorder,sustainability}` (long-horizon W-DataCenter-Operations) | control | REST + adapter | per-DC hierarchy; per-equipment lifecycle | preview | `cloud-dcops-*` |

## 8. Axis 6 — Search surfaces

| id | plane | kind | invariant | tier | crate |
|---|---|---|---|---|---|
| `search.crawler.schedule` | control | REST | per-host politeness; per-corpus rights ledger | preview | `search-crawler-api` |
| `search.crawler.frontier` | data | event | per-URL deduplication; per-host budget | preview | `search-crawler-frontier-app` |
| `search.parser.process` | data | REST + event | HTML / PDF / DOCX / OCR / Korean morphology | preview | `search-parser-app` |
| `search.index.inverted.upsert` | data | REST + event | engine-enforced isolation per ADR-0006; per-tenant + cross-tenant per consent | preview | `search-index-inverted-app` |
| `search.index.vector.upsert` | data | REST | pgvector day-1 per ADR-0050/0177; per-tenant | preview | `search-index-vector-app` |
| `search.query.execute` | data | REST | per-tenant + per-pack consent-tier filter | preview | `search-query-api` |
| `search.serp.render` | data | REST | per-locale + per-region | preview | `search-serp-app` |
| `search.rag.retrieve` (Foundry-internal) | data | REST | per-tenant boundary; per-class allowlist | stable | `search-rag-api` |
| `search.dsr.cascade.execute` | data | REST | per-tenant + per-record cascade-purge with proof-of-erasure | stable | `search-dsr-app` |

## 9. Axis 7 — Ads + Analytics surfaces

| id | plane | kind | invariant | tier | crate |
|---|---|---|---|---|---|
| `ads.campaign.publish` | control | REST | per-advertiser; per-tenant inventory | preview | `ads-campaign-api` |
| `ads.audience.publish` | control | REST | DECLARED_PREFERENCE only; per-class allowlist | preview | `ads-audience-api` |
| `ads.auction.bid` | data | REST | sub-100ms; per-cell-routed; per-policy gate | preview | `ads-auction-api` |
| `ads.serve` | data | REST | per-policy gate; brand-safety + IVT filter | preview | `ads-serve-app` |
| `ads.impression.record` | data | event | per-impression audit-emit | stable | `ads-impression-app` |
| `ads.click.record` | data | event | per-click audit-emit + DSR cascade | stable | `ads-click-app` |
| `ads.conversion.record` | data | event | server-API + privacy-preserving aggregation | stable | `ads-conversion-app` |
| `ads.attribution.batch` | analytics | batch | per-conversion cross-touch attribution; per-DP-budget | preview | `ads-attribution-app` |
| `ads.advertiser-console.report` | analytics | REST | per-advertiser; per-cohort | preview | `ads-console-api` |
| `analytics.event.ingest` | data | event | consent-tier-bound; data-class annotated | preview | `analytics-event-app` |
| `analytics.dashboard.query` | analytics | REST | per-tenant + per-purpose; DP-aggregated | preview | `analytics-warehouse-api` |
| `analytics.dp-budget.consume` | analytics | internal | per-tenant per-class ε-budget enforcement | stable | `analytics-dp-app` |
| `analytics.dsr.cascade.execute` | data | REST | per-tenant + per-record purge | stable | `analytics-dsr-app` |

---

## 10. API stability tier policy (per ADR-0040)

| Tier | Guarantee | Deprecation horizon |
|---|---|---|
| `preview` | breaking changes possible | none |
| `stable` | semver per ADR-0040; major bumps with 6-month deprecation | 6 months |
| `GA` | semver; major bumps with 12-month deprecation; per-endpoint deprecation telemetry | 12 months |

## 11. Sources

- All consolidated docs at `docs/`
- ADRs 0028, 0106, 0108-0112, 0123, 0125, 0130, 0131, 0132, 0148, 0149, 0156, 0157, 0161, 0162, 0167, 0168, 0170, 0171, 0173, 0174, 0184, 0186, 0188, 0189, 0190, 0204, 0207, 0210, 0211, 0222, 0225, 0228, 0229, 0230, 0231, 0232, 0233
- Per-product PRDs at `products/`
- Foundry-improvements research at `docs/raw/foundry-improvements.md` (top-20 + 10 PRD-shaping)
- Codex critic verdict at `docs/raw/codex-verdict.md`
- v2 backlog at `docs/raw/plan-v2-draft.md`

*Footer regenerated whenever this doc is edited.*
