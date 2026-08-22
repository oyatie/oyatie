---
purpose: Oyatie — ADR Consolidation Plan
doc_status: published
---

# Oyatie — ADR Consolidation Plan

> **Status:** Draft v0.1 — 2026-05-09. Authored per user directive: existing ADRs may be removed; the consolidated docs should point to NEW ADRs and consolidate old ones. This doc is the cleanup plan + the policy for how existing ADR-#### refs in the consolidated docs should be treated.
> **Owner:** `crew-adr-promotion` + `council-architecture`.
> **Companion:** [ADR-INDEX.md](ADR-INDEX.md), [DOC-CATALOG.md](DOC-CATALOG.md).

---

## 1. The premise

The 127-ADR corpus accumulated organically from 2024-2026. With the 2026-05-09 reframing (7 axes / Foundry consolidation / Workspace addition / global-canonical + regional-pack / in-house preference + license-conscious / multi-year cost-of-deferral horizon / wave naming / anti-scope clarifications), many existing ADRs are:

- **Stale** — describing now-superseded posture (e.g. ADR-0013 license language)
- **Drifted** — labeled Proposed but the content shipped (multiple)
- **Numbered-but-empty** — referenced in plans but no file exists (ADR-0028, ADR-0014, ADR-0021, ADR-0039 per audit; partly fixed)
- **Ratified-but-fragmented** — the same decision spans 3-5 separate ADRs that should be one
- **Vocabulary-drifted** — using retired M0/M1/M2/M3/MVP terms or the legacy "Foundry engineering platform" axis name

The cleanup converts the 127-ADR corpus to a smaller, cleaner New ADR Set that reflects the current consolidation, with explicit supersession from each old ADR to its new consolidated successor.

## 2. Policy for ADR references in consolidated docs

**Effective immediately:**

1. **New consolidated docs SHOULD prefer "ADR cluster" or "new ADR (TBD)" references** over specific legacy ADR-#### numbers, until the cleanup completes.
2. **Where a specific legacy ADR is cited and remains authoritative**, append `(legacy; subject to consolidation)`.
3. **Where a specific legacy ADR is cited and is superseded by the 2026-05-09 reframing**, append `(legacy; SUPERSEDED by drafted ADR-0050-<topic>)`.
4. **Drafted new ADRs** use the placeholder number `ADR-0050` until `crew-adr-promotion` assigns a final number; the slug names them.
5. **Doc-catalog validator** `governance-adr-citation` warns on bare ADR-#### refs without a status annotation.

## 3. The new ADR set (planned consolidation)

The ~127 ADRs are consolidated into ~30-40 new ADRs grouped by axis + cross-cutting concern. The new set is designed to be:

- **Self-consistent** — no two new ADRs disagree
- **Cohesion-enforcing** — every cross-axis contract has exactly one ADR
- **Wave-aligned** — every new ADR cites which wave it gates
- **Pack-aware** — per-region overlays declared explicitly when applicable

### 3.1 Foundation ADRs (cross-cutting)

| New ADR (drafted slug) | Consolidates from legacy | Status |
|---|---|---|
| `ADR-0006-tenant-identity-model` | ADR-0018 (tenancy + RLS) + ADR-0006 (cross-product cookie + redirect) + ADR-0017 (domain naming canon) + ADR-0022 (persona tier) + ADR-0008 (data ownership pillars) | Drafted; aggregates |
| `ADR-0008-data-use-boundary` | ADR-0008 + ADR-0133 (tier-classified OG props) + ADR-0134 (DP gateway) + ADR-0008 (mining exclusion) + ADR-0008 (multi-jurisdiction policy) — incl. 12-class taxonomy + orthogonal subject_class + purpose-permission matrix + four-pillar matrix per [PRIVACY-PROGRAM §2](PRIVACY-PROGRAM.md) | Drafted (in PRIVACY-PROGRAM) |
| `ADR-0013-product-license-policy` | (NEW; supersedes ADR-0013 license language) | [Drafted](decisions/ADR-0013-product-license-policy.md) |
| `ADR-0014-build-vs-buy-policy` | (NEW per Codex review §4) | Pending |
| `ADR-0009-cell-architecture` | (NEW; cross-cuts Cloud + Search + Ads) | Pending |
| `ADR-0010-regional-pack-architecture` | (NEW per [DESIGN §12](DESIGN.md)) | Pending |
| `ADR-0011-cross-axis-contract-registry` | (NEW; source of truth for DESIGN §10 table) | Pending |
| `ADR-0012-axis-admission-protocol` | (NEW per `LEDG-008` contradiction; supersedes ADR-0013 axis horizon) | Pending |
| `ADR-0050-audit-chain-immutability` | ADR-0028 (audit-chain Merkle-Ed25519) + ADR-0003 (audit chain) | Consolidate |
| `ADR-0050-eventing-backbone` | ADR-0116 (Redpanda; Superseded) + ADR-0046 (Kafka gated) — pick one as the new authoritative | Consolidate |
| `ADR-0050-object-graph-model` | ADR-0006 (engine-enforced typed-entity) + ADR-0108 (vector) + ADR-0109 (geo) + ADR-0110 (timeseries) + ADR-0043 (ciphertext) + ADR-0112 (struct) + ADR-0113 (schema-evolution) + ADR-0034 (form schema) + ADR-0030 (data model) | Consolidate (one umbrella + per-property addenda) |
| `ADR-0050-clean-architecture-boundaries` | ADR-0100 (corp attendance) + ADR-0101 (microservice standard) + ADR-0102 (migration plan) + ADR-0103 (workflow) + ADR-0033 (HR/payroll) + ADR-0105 (clean-arch layering) | Consolidate |
| `ADR-0050-flat-crates-target` | ADR-0129 (modules/services/platform — LEGACY, supersede) + ADR-0015 (repo structure) + ADR-0015 (flat-crates target) | Consolidate; ADR-0129 marked Superseded |

### 3.2 Per-axis ADR clusters

| Axis | New ADR cluster |
|---|---|
| SaaS multi-tenant | `ADR-0050-saas-platform-{workflow,plugin-substrate,marketplace,bench-naming,public-api-stability}` (consolidate from ADR-0035, 0149, 0156, 0157, 0161, 0162, 0164, 0130, 0191, 0230) |
| Workspace | `ADR-0050-workspace-{mail-server,calendar,doc-crdt,drive-storage,meet-sfu,chat,plugin}` (NEW; greenfield) |
| Vertical industry cloud | `ADR-0050-vertical-{name}-domain-model` per vertical + `ADR-0050-vertical-industry-cloud-umbrella` (consolidate from ADR-0016, 0104, 0114, 0115, 0120, 0126, 0127, 0137, 0141, 0142, 0143, 0145, 0190, 0216-0220) |
| Foundry | `ADR-0050-foundry-{capability-registry,autonomy-ceiling,evidence-chain,multi-provider-adapter,model-substrate,robotics-control,vision-substrate,speech-substrate,sandbox,eval-harness}` (consolidate from ADR-0019, 0020, 0103, 0107, 0144, 0145, 0148, 0149, 0164, 0167, 0172, 0173, 0180, 0186, 0187) |
| Cloud | `ADR-0050-cloud-{iam,kms,region-az-cell,compute-vm,compute-k8s,compute-functions,storage-object,storage-block,network-vpc,network-lb,network-dns,billing-tax,observability,finops,dcops}` (consolidate from ADR-0021, 0022, 0117, 0119, 0147, 0167, 0168, 0169, 0170, 0173, 0174, 0175, 0176, 0177, 0178, 0179, 0182, 0183, 0184, 0186, 0233 + dcops greenfield) |
| Search | `ADR-0050-search-{crawler,parser,index-inverted,index-vector,ranker,query-understanding,serp,safety-rights,backend-port}` (consolidate from ADR-0047 + greenfield) |
| Ads + analytics | `ADR-0050-ads-{auction,targeting-classes,attribution-firewall,advertiser-console,publisher-network,analytics-warehouse,dp-budget,clean-room,ad-policy-gate}` (NEW; greenfield) |

### 3.3 Per-region ADRs

| Region | ADR cluster |
|---|---|
| KR (pack-kr) | `ADR-0050-kr-pack-{regulatory-binding,identity-verification,mydata-open-banking,csap-isms-kcmvp,망분리,전자세금계산서,청소년-의료-금융-광고-policy}` |
| JP / US / EU / IN / BR / KSA / UAE / AU / SG | per-pack ADR clusters |

### 3.4 Cross-cutting governance ADRs

| Topic | New ADR |
|---|---|
| ADR governance + promotion process | `ADR-0050-adr-promotion-process` (consolidates ADR-0122 + this consolidation plan) |
| Deprecation governance | `ADR-0050-deprecation-governance` (ADR-0001 + ADR-0040 — keep largely as-is) |
| Wave + Plane integration framework | `ADR-0050-wave-plane-integration` (ADR-0040 launch readiness + ADR-0017 wave framework + new wave-name vocabulary) |
| Trust framework + autonomy ceiling | `ADR-0050-trust-framework` (ADR-0003 + per-vertical overrides; consolidates ADR-0022, 0132) |

## 4. Migration mechanics

For each consolidation:

1. New ADR drafted at `decisions/ADR-0050-<slug>.md` with `Supersedes:` listing every legacy ADR it replaces.
2. Each legacy ADR gets `Superseded-by: ADR-0050-<slug>` added in-place.
3. Legacy ADR Status moves to `Superseded`.
4. Legacy ADR file is **NOT deleted** (forensic value); it stays but with the supersession header.
5. Consolidated docs update their citations to point to the new ADR.
6. ADR-INDEX.md regenerates from the directory state (some rows now show Superseded).
7. machine-readable/decisions.json regenerates.

## 5. Sequencing

- **Wave W-Foundation** (current): consolidate the Foundation ADR cluster (§3.1) — these ADRs gate every other axis.
- **Wave W-Foundry-Preview**: consolidate the Foundry ADR cluster (§3.2 row 4).
- **Wave W-Cloud-Preview / W-Search-Preview / W-SaaS-Preview / W-Workspace-Preview**: consolidate per-axis clusters.
- **Per Vertical-Preview**: consolidate per-vertical clusters.
- **W-Region-Fan-Out**: consolidate per-region clusters as packs onboard.

## 6. Authority during transition

- This `ADR-CONSOLIDATION-PLAN.md` is the authoritative source on **which ADR is current** for any given decision.
- Where a legacy ADR-#### is cited in the consolidated docs, the citation is *current as of the doc's last edit* but may be eclipsed by a newer ADR drafted in this plan.
- The `crew-adr-promotion` team produces a weekly diff: which legacy ADRs were superseded this week + what the consolidated docs need to update.

## 7. Anti-pattern

- **Don't delete legacy ADRs.** Forensic value + git-blame integrity matter.
- **Don't re-number.** ADR numbers are stable identifiers; new ADRs get the next available; legacy stays at its number with Status = Superseded.
- **Don't treat this plan as the new ADR.** This plan is a *meta-doc*. Each row in §3 produces a real ADR file when drafted.
- **Don't cite a Proposed-but-not-yet-drafted ADR-0050** in production code; cite "(planned ADR; per ADR-CONSOLIDATION-PLAN §3.x)" until drafted.

## 8. Open questions

1. Final number assignment policy — strict sequential or grouped?
2. Should we emit a one-time `ADR-INDEX-LEGACY.md` snapshot before the consolidation pass for forensics?
3. Should superseded ADRs move to `decisions/superseded/` subdirectory?
4. How to surface the supersession graph visually — Mermaid diagram or interactive page on `dev.oyatie.com`?

## 9. Sources scanned

- All 127 legacy ADRs at `decisions/`
- [ADR-INDEX.md](ADR-INDEX.md) (current)
- [docs/raw/codex-verdict.md](raw/codex-verdict.md) §6 + §11 + §17 (re-numbering / consolidation feedback)
- ADR-0122 (ADR due-diligence + polish roadmap) — partly subsumed by this plan


---

> **§Note (2026-05-21 transition):** References to `governance-*` in this historical document are intentional — they describe past state. New work uses `governance-*` per the 2026-05-21 transition directive.