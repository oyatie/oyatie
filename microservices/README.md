---
doc_status: published
last_audited: 2026-05-20
---

# Microservices Roster

This directory contains the live flat µservice catalog. The table is intentionally boring: one row per `microservices/*` directory, a one-line responsibility, the capability tier, and the service-local ADR-MS pointer when one exists.

## Roster

| µservice | Capability tier | One-line responsibility | ADR-MS |
|---|---|---|---|
| `analytics` | product | Tenant analytics, reporting, attribution, and decision-support surfaces over governed product data. | Not authored yet |
| `api-gateway` | substrate | Dedicated Tier-0 north-south edge for tenant, partner, mobile, browser traffic. Owns TLS 1.3 + ECH + PQC termination, HTTP/3 fallback chain, anti-bot/anti-spoof/anti-scrape defence-in-depth, per-tenant... | [`ADR-MS-001`](api-gateway/decisions/ADR-MS-001-edge-admission-policy-and-pqc-contract.md) |
| `application` | product | Unified B2B shell where tenants enable products, switch contexts, and reach cross-service workflows. | Not authored yet |
| `audit-chain` | substrate | Tamper-evident event, evidence, retention, and replay substrate for every regulated action. | Not authored yet |
| `calendar` | product | Workspace calendar scheduling, availability, resource booking, and cross-tenant meeting coordination. | Not authored yet |
| `cell` | substrate | Per-tenant/per-region blast-radius cell routing, residency, and failure-isolation substrate. | Not authored yet |
| `cloud-iac` | substrate | Infrastructure-as-code substrate for clusters, cells, secrets, networking, and deployment environments. | Not authored yet |
| `cloud-k8s` | substrate | Kubernetes platform substrate for workload scheduling, mesh integration, policy, and cluster lifecycle. | Not authored yet |
| `cloud-secrets` | substrate | Secret reference, rotation, envelope encryption, and OpenBao/HSM integration substrate. | [`ADR-MS-001`](cloud-secrets/decisions/ADR-MS-001-secret-reference-namespace-and-rotation-contract.md) |
| `comms-email` | substrate | Transactional and tenant-facing email communications substrate with deliverability and audit hooks. | Not authored yet |
| `community` | product | Tenant/community social spaces, channels, forums, moderation, and mutual-aid interaction surfaces. | Not authored yet |
| `compliance` | product | Compliance evidence automation per ADR-0209. In-house pipeline covering SOC 2 Type II, GDPR (incl. DSAR automation), HIPAA, PCI-DSS — replacing Drata / Vanta / Tugboat Logic / AuditBoard / ServiceNow GRC... | Not authored yet |
| `connector` | retired umbrella | Retiring umbrella coordination surface for the dissolved platform. | [`ADR-MS-001`](connect/decisions/ADR-MS-001-connector-broker-webhook-and-dlq-contract.md) |
| `consent-graph` | substrate | Consent, purpose, delegation, and cross-tenant visibility graph for privacy-bound data sharing. | Not authored yet |
| `contact-center` | product | Omnichannel support routing, queues, consent-aware transcripts, and service-level contact workflows. | [`ADR-MS-001`](contact-center/decisions/ADR-MS-001-omnichannel-routing-queue-and-consent-contract.md) |
| `contract-lifecycle-management` | product | Contract authoring, negotiation, obligation tracking, renewal, and legal approval workflows. | Not authored yet |
| `crm` | product | Customer record, pipeline, account, opportunity, and revenue-lineage product surface. | [`ADR-MS-001`](crm/decisions/ADR-MS-001-customer-record-mutation-and-revenue-lineage-contract.md) |
| `data-pipeline` | product | Ingest, transform, replay, lineage, and tenant-governed data movement product surface. | [`ADR-MS-001`](data-pipeline/decisions/ADR-MS-001-lineage-first-ingest-transform-and-replay-contract.md) |
| `data-warehouse` | product | Tenant OLAP warehouse, freshness windows, governed marts, and analytical lineage product surface. | [`ADR-MS-001`](data-warehouse/decisions/ADR-MS-001-tenant-olap-freshness-and-lineage-contract.md) |
| `design-collaboration` | product | Collaborative design files, review flows, comments, handoff, and visual asset workspace. | Not authored yet |
| `detection` | substrate | Streaming and batch detection substrate for anomaly, abuse, fraud, policy, and risk signals. | Not authored yet |
| `developer-sdk` | external-facing | External developer SDKs, API ergonomics, examples, and integration support surface. | Not authored yet |
| `docs` | substrate | Documentation product surface and docs-engine runtime for generated, searchable, governed docs. | Not authored yet |
| `drive` | product | Workspace file storage, sharing, permissions, retention, and cross-device document access. | Not authored yet |
| `feature-flags` | substrate | Canonical OpenFeature-compatible runtime flag substrate with tenant, persona, cohort, and emergency kill-switch targeting. Consumed by all 46+ µservices as shared substrate per ADR-0245. | [`ADR-MS-001`](feature-flags/decisions/ADR-MS-001-flag-evaluation-killswitch-and-experiment-contract.md) |
| `financial-planning` | product | Budgeting, forecasting, scenario planning, variance analysis, and finance planning workflows. | Not authored yet |
| `finops-portal` | product | Cost attribution, showback/chargeback, budgets, optimization, and tenant FinOps reporting. | Not authored yet |
| `forms` | product | Form builder, structured submissions, approval intake, validation, and routed evidence capture. | Not authored yet |
| `global-trade` | product | Cross-border trade, customs, restricted-party, tariff, and logistics compliance product surface. | Not authored yet |
| `governance` | substrate | Policy, standards, controls, evidence, quality gates, and corpus governance substrate. | Not authored yet |
| `healthcare-integration` | product | Healthcare interoperability, FHIR/EHR handoff, clinical workflow, and regulated integration surface. | Not authored yet |
| `identity` | substrate | Identity, passkeys, OIDC/SAML/SCIM, step-up auth, memberships, and account recovery substrate. | Not authored yet |
| `incident-management` | product | Incident intake, response coordination, escalation, postmortem, and reliability workflow product. | Not authored yet |
| `intelligence` | substrate | AI-agent platform + two-layer AI substrate: model routing, policy-bound inference, agent runtime/supervisor, adapters, evals, capability routing, and RAG (absorbed the former `foundry` platform per ADR-0363). | Not authored yet |
| `itsm` | product | IT service management: service catalog, tickets, approvals, asset linkage, and change workflows. | Not authored yet |
| `learning-management` | product | Training catalog, assignments, completion evidence, certification, and learning workflows. | Not authored yet |
| `mail` | product | Workspace mail, threading, DKIM/SPF/DMARC, retention, delegation, and secure messaging workflows. | Not authored yet |
| `marketing-automation` | product | Campaigns, segments, consent-aware outreach, attribution, and marketing workflow automation. | Not authored yet |
| `marketplace` | product | Universal deal, app, service, supplier, and monetization marketplace settlement product. | Not authored yet |
| `meet` | product | Meetings, conferencing, recordings handoff, reactions, scheduling joins, and collaboration sessions. | Not authored yet |
| `messenger` | product | Secure messaging, channels, presence, moderation, retention, and cross-context communication. | Not authored yet |
| `network` | product | Professional and personal network graph, follows, referrals, discovery, and relationship surfaces. | Not authored yet |
| `notes` | product | Workspace notes, knowledge capture, sharing, retention, and lightweight collaboration. | Not authored yet |
| `observability` | substrate | Metrics, logs, traces, SLOs, alerting, tail sampling, and operational telemetry substrate. | Not authored yet |
| `ontology` | substrate | Typed entity, relationship, action, projection, and policy-aware knowledge graph substrate. | Not authored yet |
| `ops-dashboard-control-center` | substrate | Operations control center for deployment status, incidents, cells, gates, and platform health. | Not authored yet |
| `payments` | substrate | Payment, payout, ledger, settlement, tax, refund, and regulated money-movement substrate. | Not authored yet |
| `performance-management` | product | Goals, reviews, calibration, feedback, promotion, and HR performance workflows. | Not authored yet |
| `plant-maintenance` | product | Asset maintenance, work orders, inspections, spares, reliability, and plant operations workflows. | Not authored yet |
| `plugin-app-store` | external-facing | Plugin publishing, trust tiers, monetization, installation, and ecosystem app distribution. | Not authored yet |
| `production-planning` | product | Manufacturing planning, capacity, work orders, MRP, scheduling, and production control. | Not authored yet |
| `quality-management` | product | Quality inspections, nonconformance, CAPA, recalls, supplier quality, and audit workflows. | Not authored yet |
| `real-estate` | product | Property, lease, facilities, occupancy, maintenance, and real-estate portfolio workflows. | Not authored yet |
| `recordings` | product | Meeting/media recording capture, transcript, retention, access, and evidence handoff service. | Not authored yet |
| `sheets` | product | Spreadsheet workspace, formulas, collaboration, governed import/export, and analytical grids. | Not authored yet |
| `shorts` | product | Short-form creator/media surface with moderation, monetization, and tenant/community distribution. | Not authored yet |
| `sites` | product | Internal/external sites, pages, publishing, permissioned content, and tenant web surfaces. | Not authored yet |
| `slides` | product | Presentation authoring, collaboration, templates, export, and controlled sharing. | Not authored yet |
| `social` | product | Social feed, profiles, groups, moderation, creator/community experiences, and relationship surfaces. | Not authored yet |
| `supply-chain-planning` | product | Demand, supply, procurement, vendor, inventory, disruption, and planning workflows. | Not authored yet |
| `tasks` | product | Task lists, assignments, reminders, lightweight workflow, and personal/team execution tracking. | Not authored yet |
| `tenancy` | substrate | Tenant, membership, environment, quota, residency, and lifecycle substrate. | Not authored yet |
| `translate` | product | Translation, localization, terminology memory, pack overlays, and multilingual content workflows. | Not authored yet |
| `treasury` | product | Cash, liquidity, bank connectivity, FX, hedging, and treasury operations product surface. | Not authored yet |
| `warehouse` | product | Warehouse inventory, receiving, picking, packing, shipping, and fulfillment operations. | Not authored yet |
| `whiteboard` | product | Collaborative canvas, diagrams, ideation, review, and visual planning workspace. | Not authored yet |
| `workflow-engine` | substrate | Durable state-machine/DAG orchestration, approvals, sagas, retries, and cross-service execution substrate. | Not authored yet |
| `workflow-studio` | product | No-code workflow builder, canvas, templates, versioning, and simulation product surface. | Not authored yet |
| `workplace-integration` | product | Workplace app/integration layer for employee tools, connectors, migrations, and tenant adoption flows. | Not authored yet |

## Reading Rules
- Portfolio-wide architecture decisions live in `docs/ADR-INDEX.md`; service-local decisions live under `microservices/<service>/decisions/ADR-MS-*.md`.
- `Not authored yet` in the ADR-MS column is a real gap marker, not permission to proceed without a decision. Create service-local ADRs in a dedicated service-doc slice, not during roster gardening.
- Capability tiers are `substrate`, `product`, `external-facing`, or `retired umbrella`; when a manifest declares a tier, the manifest value wins.
- Before implementing a service change, read its PRD/manifest, its ADR-MS row if present, and the relevant keystone ADR cluster in `docs/ADR-INDEX.md`.
