---
doc_class: MicroserviceInventoryREADME
microservice: global-trade
status: inventory-provenance-planned-only
current_authority: specs/microservices/global-trade.json
authority_status: metadata-only PRD
inventory_status: inventory/provenance/planned-only
owner_team: council-enterprise
---

# Global Trade

Current authority: specs/microservices/global-trade.json metadata-only PRD; inventory/provenance/planned-only.

This directory is retained as inventory and provenance for the Global Trade vertical. It is not current product-design, implementation, runtime, cloud, on-call, deployment, SLO, dashboard, contract, or live-provider authority.

Current source authority is `specs/microservices/global-trade.json` (`PRD-MICROSERVICE-GLOBAL-TRADE`, `_meta.status=preview`). The only current implementation evidence under this directory is the metadata-only domain crate at `crates/oya-global-trade-compliance-domain`, whose tests keep live-provider, list-download, filing, broker, document-archive, mutation, Workflow, runtime-audit, and cloud flags false.

## Non-claims

- No live denied-party or sanctions-provider network calls.
- No government list downloads, legal-ruling retrieval, customs-authority filing, export declaration transmission, broker workflow, or document archive runtime.
- No durable persistence, business transaction block mutation, product master mutation, shipment mutation, inventory cost update, accounting posting, payment disbursement, Workflow execution, runtime audit-chain emission, or cloud deployment.
- No SAP GTS, Oracle GTM, Microsoft Dynamics 365, customs-agency, export-control, landed-cost exhaustive parity, certification, GA, production-readiness, hyperscaler-readiness, measured SLO, or on-call claim.

## Inventory interpretation

- `contracts/`, `iac/`, `slos/`, `dashboards/`, and `runbooks/` are stale second-pass inventory/provenance only. They do not define active APIs, events, infrastructure, SLOs, dashboards, or operations.
- `catalog/`, `capabilities/`, `decisions/`, `IPs/`, `policy/`, `cedar/`, `dpia/`, `scorecards/`, and `AUDIT-FINDINGS-2026-05-21.json` are planned/provenance artifacts only unless a future authority-gated RED/test-first implementation lane expands the PRD.
- `specs/microservices/manifests-index.json` intentionally has no `global-trade` manifest entry; do not use this directory's `manifest.json` as manifest-index promotion evidence.
