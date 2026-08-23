---
doc_class: Playbook
shape: anchor
length_cap: 120
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Vertical-pack updates (healthcare/fintech/etc.) with DPIA refresh per pack.
planned_enforcement_ref:
  - governance-data-class
  - governance-cohort-honor
related_adrs: [ADR-0033, ADR-0034, ADR-0038, ADR-0053, ADR-0055]
adr_citations: [ADR-0053, ADR-0055]
doc_status: published
---

# Playbook: Vertical-Pack Update


## 1. Surface

Vertical regulatory packs ([ADR-0033](../../decisions/ADR-0033-vertical-industry-cloud-pack-architecture.md), [ADR-0034](../../decisions/ADR-0034-per-vertical-data-class-overrides.md)): healthcare (HIPAA / KR-HIPAA-equivalent), fintech (KR-FSC / PCI-DSS), legal (privilege / hold), public-sector (KISA / FedRAMP-equivalent), education (FERPA-equivalent).

## 2. Default rail

**Blue/green** (mandatory) for any pack change that affects:

- Data classification overrides.
- Retention rules.
- Cross-region residency rules ([ADR-0049](../../decisions/ADR-0049-cross-region-replication-and-residency.md)).
- DSR / proof-of-erasure cascade ([ADR-0038](../../decisions/ADR-0038-trust-framework-and-dsr-cascade-and-proof-of-erasure.md)).
- Cedar policy overlays ([ADR-0007](../../decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md)).

**Canary** acceptable for pack additions that are purely *additive* (new audit fields, new DPA wording with no data-flow change).

Per [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md), regulated-vertical packs: bi-weekly staging → prod; M=7d canary; `privacy-reviewer` + `database-reviewer` re-affirm mandatory at gate 5; `requires_human_signoff: true` for regulated packs.

## 3. Mandatory DPIA refresh

Every regulated-pack update triggers a DPIA refresh:

1. Identify the changed data flows.
2. Re-evaluate purpose / proportionality / minimisation.
3. Per-vertical regulator alignment check (KR-PIPC for healthcare; KR-FSC for fintech; etc.).
4. Tenant DPA amendment (or notice-only update for non-material changes).
5. DPIA artefact stored as D14 evidence; trust portal updated ([ADR-0038](../../decisions/ADR-0038-trust-framework-and-dsr-cascade-and-proof-of-erasure.md)).

`governance-data-class` (existing) verifies DPIA presence.

## 4. Cohort gating (regulated-only)

Regulated packs roll out to `stable-regulated` cohort **only** — they never roll out to general tenants. The pack is the cohort's release-train.

Sequence:

| Phase | Action | Duration |
|---|---|---|
| 1 | Pack drafted; per-vertical regulator consulted | weeks |
| 2 | Pack staged on internal regulated-pilot cell | 14 d |
| 3 | Dark-launch on opted-in `stable-regulated` pilot tenants (3-5 tenants) | 14 d |
| 4 | Canary 25% → 50% → 100% of pilot cohort | 14 d |
| 5 | All `stable-regulated` tenants in vertical | 28 d soak |

## 5. Per-vertical pack approval

Each pack has a named approver (per-vertical compliance officer). Approval is gated by D14 emit; no auto-approval. Planned advisory lane `governance-data-class` records approver-field gaps until the PR-blocking workflow exists.

## 6. Rollback

Per-tenant rollback (default for regulated; per-tenant in [`blue-green-spec.md`](blue-green-spec.md) §5). A regulated tenant rolling back stays on the prior pack indefinitely until a new pack passes the same gates.

## 7. DSR cascade verification

Per [ADR-0038](../../decisions/ADR-0038-trust-framework-and-dsr-cascade-and-proof-of-erasure.md), every pack update must re-verify the DSR cascade still completes within the per-vertical SLA (e.g. 30 d for healthcare). Re-verification = run a synthetic DSR end-to-end and capture proof-of-erasure.

## 8. Audit-chain artefacts

Per pack update, emit:
- Pack version + content-hash.
- DPIA artefact hash.
- Per-vertical regulator alignment statement.
- DSR cascade re-verification result.
- Per-tenant DPA amendment ID (if applicable).
- Cohort-roll-out timeline.


## 9. Hyperscaler equivalent

Microsoft Government Community Cloud (GCC / GCC-High) release pattern; AWS GovCloud release lag; Oracle Government Cloud release pattern. We adopt the per-vertical-pack-as-release-train pattern.

## 10. ADR citations

- [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md) — regulated vertical: bi-weekly staging → prod; M=7d canary; `privacy-reviewer` + `database-reviewer` re-affirm at gate 5; human-signoff required.
