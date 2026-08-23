---
doc_status: archived
---

# ADR full disposition audit
Tip: `c7f60a9dbfe28f5d2f17b094ba25ae8c8602e6cb`  
Queried: 2026-08-06T07:31:06Z
Total ADRs: **448**
## Policy
- **Do not mass-Accept** Proposed ADRs. Proposed is not implement authority.
- Mechanical fixes OK: status case, missing successor pointers when successor is known.
- Accept/Supersede/Amend substance requires dual-critic + presubmit PR (and founder when planning_impact).

## Disposition histogram

| Disposition | Count |
|-------------|------:|
| KEEP_ACCEPTED | 114 |
| PROPOSED_REVIEW_QUEUE | 81 |
| PROPOSED_ADMISSION_QUEUE | 69 |
| MECHANICAL_STATUS_CASE | 66 |
| KEEP_ACCEPTED_READ_WITH_AMENDS | 38 |
| KEEP_SUPERSEDED | 30 |
| NEEDS_STATUS_TRIAGE | 26 |
| PLAN_LAG_EDGE | 10 |
| NEEDS_SUPERSEDED_BY | 5 |
| KEEP_PROPOSED_BLOCKING_ACTIVATION | 5 |
| MANUAL_TRIAGE | 4 |

## Priority work queues

### 1. Plan-lag: Accepted depends_on/amends Proposed
- **ADR-0565**: ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0358, ACCEPTED_AMENDS_PROPOSED:ADR-0253
- **ADR-0614**: ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0595, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0597, ACCEPTED_AMENDS_PROPOSED:ADR-0563
- **ADR-0616**: ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0539, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0551, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0552, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0595, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0604, ACCEPTED_AMENDS_PROPOSED:ADR-0604
- **ADR-0619**: ACCEPTED_AMENDS_PROPOSED:ADR-0328, ACCEPTED_AMENDS_PROPOSED:ADR-0609
- **ADR-0630**: ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0560, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0556
- **ADR-0635**: ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0245, ACCEPTED_AMENDS_PROPOSED:ADR-0245
- **ADR-0636**: ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0554, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0560, ACCEPTED_AMENDS_PROPOSED:ADR-0554
- **ADR-0637**: ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0013, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0538, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0597, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0633, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0634, ACCEPTED_AMENDS_PROPOSED:ADR-0538
- **ADR-0638**: ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0013, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0597, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0605, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0606, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0608, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0627, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0633, ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0634
- **ADR-0639**: ACCEPTED_DEPENDS_ON_PROPOSED:ADR-0554

### 2. Superseded without successor
- **ADR-0057**: ADR-0057: Cutover Mechanics — Rename Plan v4 (Hybrid C)
- **ADR-0097**: Rename intelligence-account-adapter-{claude-code,codex-cli,gemini-cli} → foundry-{claude,codex,gemini}-account-adapter
- **ADR-0101**: Foundry Supervisor Mountpoint (Direct Hyper)
- **ADR-0102**: Foundry Settings Template Canonical Rendering
- **ADR-0138**: ADR-0138: Foundry six-path deprecation — Strangler migration

### 3. Missing status frontmatter
- **ADR-0130**: ADR-0130: Deprecate `registry/knowledge-graph-semantic.json` and Migrate to Ontology Type System
- **ADR-0146**: ADR-0146 — Container base image: distroless `static-debian12:nonroot`
- **ADR-0149**: ADR-0149: Idempotency Keys Canonical
- **ADR-0150**: ADR-0150: Cursor Pagination Canonical
- **ADR-0151**: ADR-0151: X-Request-Id Propagation
- **ADR-0152**: ADR-0152: RPO/RTO Canonical (Five-Tier Recovery Model)
- **ADR-0153**: ADR-0153: Outbox Pattern
- **ADR-0154**: ADR-0154: Event Schema Versioning
- **ADR-0155**: ADR-0155: Per-Tenant Resource Quotas
- **ADR-0156**: ADR-0156: PII Registry Canonical (Cross-Cutting Data Classification)
- **ADR-0173**: ADR-0173 — Vendor lock-in avoidance and stack ownership
- **ADR-0200**: ADR-0200 — WASM runtime canonical: Wasmtime
- **ADR-0201**: ADR-0201 — Email + transactional comms adapter substrate
- **ADR-0202**: ADR-0202 — GitOps + IaC + Cluster lifecycle: three-tier separation
- **ADR-0203**: ADR-0203 — Documentation engine: three-tier separation
- **ADR-0211**: ADR-0211 — In-House Tech Stack Policy
- **ADR-0212**: ADR-0212 — Buildability Doctrine
- **ADR-0214**: ADR-0214: Cross-Tenant Real-Time Visibility (Consent-Graph + Ontology Projection Extension)
- **ADR-0215**: ADR-0215: Multi-Context Platform Architecture
- **ADR-0216**: ADR-0216: Open Integration and Migration-Out Policy
- **ADR-0217**: ADR-0217: Service Packaging Rollout Order
- **ADR-0218**: ADR-0218: Tenant Granular Control Surface
- **ADR-0219**: ADR-0219: No-Code-First UX with Optional AI-Assist
- **ADR-0220**: ADR-0220: Consumer Intelligence Substrate
- **ADR-0221**: ADR-0221 — Agentic Development Pipeline Hardening
- **ADR-0239**: ADR-0239: Foundry Scope Clarification (Internal-Only Amendment)

### 4. Status case normalize
- **ADR-0001**: `accepted` → normalize
- **ADR-0002**: `proposed` → normalize
- **ADR-0003**: `proposed` → normalize
- **ADR-0004**: `proposed` → normalize
- **ADR-0005**: `proposed` → normalize
- **ADR-0006**: `accepted` → normalize
- **ADR-0007**: `proposed` → normalize
- **ADR-0008**: `accepted` → normalize
- **ADR-0009**: `proposed` → normalize
- **ADR-0010**: `proposed` → normalize
- **ADR-0011**: `accepted` → normalize
- **ADR-0013**: `proposed` → normalize
- **ADR-0014**: `proposed` → normalize
- **ADR-0016**: `proposed` → normalize
- **ADR-0017**: `accepted` → normalize
- **ADR-0018**: `accepted` → normalize
- **ADR-0019**: `proposed` → normalize
- **ADR-0020**: `proposed` → normalize
- **ADR-0021**: `proposed` → normalize
- **ADR-0022**: `proposed` → normalize
- **ADR-0023**: `proposed` → normalize
- **ADR-0024**: `proposed` → normalize
- **ADR-0025**: `proposed` → normalize
- **ADR-0026**: `proposed` → normalize
- **ADR-0027**: `proposed` → normalize
- **ADR-0028**: `accepted` → normalize
- **ADR-0029**: `accepted` → normalize
- **ADR-0030**: `accepted` → normalize
- **ADR-0031**: `accepted` → normalize
- **ADR-0032**: `proposed` → normalize
- **ADR-0034**: `accepted` → normalize
- **ADR-0035**: `proposed` → normalize
- **ADR-0036**: `proposed` → normalize
- **ADR-0038**: `proposed` → normalize
- **ADR-0039**: `proposed` → normalize
- **ADR-0040**: `proposed` → normalize
- **ADR-0042**: `superseded` → normalize
- **ADR-0043**: `proposed` → normalize
- **ADR-0044**: `proposed` → normalize
- **ADR-0045**: `proposed` → normalize
- **ADR-0047**: `proposed` → normalize
- **ADR-0048**: `proposed` → normalize
- **ADR-0049**: `proposed` → normalize
- **ADR-0051**: `accepted` → normalize
- **ADR-0055**: `accepted` → normalize
- **ADR-0058**: `accepted` → normalize
- **ADR-0059**: `accepted` → normalize
- **ADR-0060**: `accepted` → normalize
- **ADR-0061**: `accepted` → normalize
- **ADR-0062**: `accepted` → normalize
- **ADR-0063**: `accepted` → normalize
- **ADR-0064**: `accepted` → normalize
- **ADR-0065**: `accepted` → normalize
- **ADR-0066**: `accepted` → normalize
- **ADR-0067**: `accepted` → normalize
- **ADR-0069**: `accepted` → normalize
- **ADR-0090**: `accepted` → normalize
- **ADR-0091**: `accepted` → normalize
- **ADR-0092**: `accepted` → normalize
- **ADR-0093**: `accepted` → normalize
- **ADR-0094**: `accepted` → normalize
- **ADR-0095**: `accepted` → normalize
- **ADR-0096**: `accepted` → normalize
- **ADR-0098**: `accepted` → normalize
- **ADR-0099**: `accepted` → normalize
- **ADR-0116**: `accepted` → normalize

### 5. Accepted cites Superseded

## Stale Accepted rule (binding)

**Must not treat `status: Accepted` as current whole law.**

1. Follow `superseded_by` when status is Superseded.
2. Reverse-index: if a later Accepted lists this id under `supersedes:`, treat as superseded even if status field lags.
3. Always load live `amended_by` peers (e.g. 0515→0624/0639; 0562→0615/0635).
4. Record resolution path in authority receipts.

See `2026-08-06-live-resolution-rule.json`.

Examples of resolve (tip at audit):

| Cited | Live resolution |
|-------|-----------------|
| 0513 | → 0515 + amended_by fabric/0624/0639 |
| 0110 | → 0363 + amended_by 0510/0515 |
| 0596 | → 0616 |
| 0562 | 0562 **with** 0615+0635 (not bare 0562) |
| 0550 | → 0562 + 0615+0635 |

