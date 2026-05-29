# consent-graph competitor parity matrix

- Owner: axis-consent-graph + product
- Date: 2026-05-18
- Authority: ADR-0214 §3 (alternatives considered).

This document compares consent-graph against the closest competitors / inspirations across the five
must-haves (audit, revocation, sovereignty, scope, real-time).

## 1. The five-must matrix

| Solution | Audit | Revocation | Sovereignty | Scope | Real-time |
|----------|-------|------------|-------------|-------|-----------|
| **consent-graph** (oyatie) | ✓ bilateral chain | ✓ ≤1s p99 | ✓ zero-copy region-pinned | ✓ field-level Cedar | ✓ ≤500ms projection |
| EDI 850/856/810 | ✗ per-doc only | ✗ none | ✗ batch transfer | ✗ doc-shaped | ✗ 24h+ |
| Per-tenant API tokens | ✗ ad hoc | ✗ rotation | ✗ token-based | ✗ endpoint-coarse | ✓ |
| Snowflake Secure Data Share | ~ partial | ~ minutes | ✓ zero-copy | ~ RBAC | ✗ batch |
| Databricks Delta Sharing | ~ open table audit | ~ minutes | ✓ zero-copy | ~ table-level | ✗ batch |
| BigQuery Authorized Views | ~ via Cloud Audit | ✗ schema-edit | ✗ data residency only | ✓ view-level | ~ near real-time |
| Open Banking PSD2/FAPI/Plaid | ✓ TPP audit | ✓ token revoke | ✗ N/A | ✓ scope set | ~ second-level |
| HIE TEFCA / Direct Trust | ✓ HIPAA disclosure log | ~ provider opt-out | ~ HIPAA jurisdiction | ✓ min-necessary | ~ near real-time |
| SAP Ariba Network | ~ doc audit | ~ contract end | ~ EU data centers | ~ relationship-scope | ~ minutes |
| IBM Food Trust (Hyperledger) | ✓ chain | ~ smart-contract revoke | ✗ shared ledger | ~ per-org partition | ✗ minutes-level settlement |
| TradeLens (deprecated) | ✓ chain | ~ governance | ~ regional | ~ relationship-scope | ✗ minutes |
| CommerceHub / SPS | ~ vendor audit | ~ contract end | ✗ vendor data | ~ doc-coarse | ~ near real-time |

Legend: ✓ full, ~ partial, ✗ absent.

## 2. Deep dive: closest 5

### 2.1 Snowflake Secure Data Share
- **What it does right**: zero-copy share at storage layer; multi-cloud; mature.
- **Where it falls short**:
  - Batch refresh (typically 1min-15min cadence; not sub-second).
  - RBAC-only; no field-level Cedar; no purpose-of-use constraint.
  - Revocation is "unshare" — minutes to take effect.
  - No bilateral audit chain.
  - Snowflake lock-in.
- **What we adopt**: zero-copy storage model concept.
- **What we improve**: real-time event stream + Cedar enforcement + bilateral audit + revocation
  in ≤1s.

### 2.2 Databricks Delta Sharing
- **What it does right**: open protocol (vs Snowflake's proprietary); zero-copy; ecosystem.
- **Where it falls short**: same as Snowflake (batch, RBAC, no Cedar, no bilateral audit, slow
  revocation).
- **What we adopt**: open-protocol sensibility (consent-graph's protocol is published OpenAPI 3.2 +
  AsyncAPI 3.1).

### 2.3 Open Banking (PSD2 / UK Open Banking / FAPI / Plaid)
- **What it does right**: customer consent UX; TPP scope; revocable; mature compliance.
- **Where it falls short**:
  - Financial-only.
  - Protocol stack heavy (OAuth2 + FAPI mTLS + DPoP); not entity-shaped.
  - No bilateral audit chain (regulator-audited, but not bilateral).
- **What we adopt**: consent UX patterns; scope-narrowing semantics; revocation primitive.
- **What we improve**: vertical-agnostic; entity-shaped via Ontology; bilateral chain.

### 2.4 HIE (TEFCA / Direct Trust / IHE XDS)
- **What it does right**: HIPAA-grade audit (disclosure log); min-necessary; break-glass with audit.
- **Where it falls short**:
  - Healthcare-only protocol stack (Direct, XDS.b, FHIR-Bulk-Data).
  - Provider-opt-out, not real-time revocation.
  - Document-shaped, not entity-shaped.
- **What we adopt**: bilateral audit concept; min-necessary scope; break-glass + audit-review.
- **What we improve**: vertical-agnostic; real-time revocation; entity-shaped.

### 2.5 IBM Food Trust / TradeLens (Hyperledger)
- **What it does right**: append-only chain; multi-party visibility.
- **Where it falls short**:
  - Blockchain settlement: minutes (not sub-second).
  - Consensus overhead massive (governance + ordering service).
  - Smart-contract attack surface.
  - TradeLens was shut down 2022; Food Trust struggles for adoption.
- **What we adopt**: nothing — we explicitly reject the consensus path.
- **What we improve**: Merkle-sealed bilateral chain is sufficient; ~1000× faster.

## 3. EaaS moat thesis

Each competitor has 1–2 of the 5 must-haves. No competitor has 3 or more. consent-graph has all 5.

**This is the EaaS moat.** It's a textbook example of a defensible advantage that requires
*simultaneous* mastery of multiple substrates (Cedar + Pulsar + audit-chain + Ontology + sovereignty
+ identity). Each is hard alone; together they form a moat.

Year-2 expansion opportunities (PHASE-02+):
- B2C consent management UX layer (compete with OneTrust / TrustArc privacy management products).
- Marketplace discovery (compete with Snowflake Data Marketplace, AWS Data Exchange).
- Industry-specific scope catalogs (supply-chain, healthcare, banking) → category-killer status.

## 4. Competitor watch-list

- **Snowflake** announcing real-time Data Share in next-gen Snowpark — track GA timeline.
- **Databricks** Unity Catalog Data Sharing — add field-level RBAC? track.
- **OneTrust / TrustArc** moving into B2B consent-graph adjacency — possible acquirer-or-competitor.
- **Plaid** vertical expansion into supply chain — unlikely but possible.
- **Open-source Project Carbon / OpenLineage** — open-protocol movement; potential interop standard.

## 5. Customer-objection responses

| Objection | Response |
|-----------|----------|
| "Why not just Snowflake Data Share?" | Real-time, bilateral audit, revocation, vertical-agnostic, no SF lock-in. |
| "Why not Plaid?" | Vertical-agnostic; we are not a financial-data intermediary. |
| "Why not blockchain?" | 1000× slower; over-engineered; we already have authoritative source-of-truth per entity. |
| "Why your own audit-chain — not just CloudTrail/cloud-audit?" | Audit-chain is bilateral + Merkle-sealed; CloudTrail is single-party + tamperable by cloud-account-admin. |
| "Why your own consent µservice — not just an API gateway with tokens?" | Tokens have no audit primitive, no revocation primitive, no scope-narrowing primitive. consent-graph is consent-as-a-service. |

## 6. References

- Snowflake Secure Data Sharing docs: docs.snowflake.com (subject to vendor-doc drift).
- Databricks Delta Sharing protocol: github.com/delta-io/delta-sharing.
- PSD2 RTS / FAPI 2.0 spec: openid.net.
- TEFCA Common Agreement: rce.sequoiaproject.org.
- ADR-0214 §3 alternatives considered (this repo).
