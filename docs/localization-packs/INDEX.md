---
doc_class: LocalizationPackCatalog
shape: anchor
status: Accepted
date: 2026-05-13
owners: ["council-architecture"]
authority_chain: docs/MASTERPLAN.md §2.5, §5.5 → ADR-0064 → this file → docs/localization-packs/<pack>.md
companion_docs:
  - docs/MASTERPLAN.md
  - docs/decisions/ADR-0064-canonical-base-and-localization-packs.md
  - docs/decisions/ADR-0063-documentation-suite-coverage.md
---

# Localization Pack Catalog

This is the canonical catalog of oyatie localization packs. Every active pack has a dedicated overview doc at `docs/localization-packs/<code>.md` and a manifest at `docs/localization-packs/<code>/pack.yaml`.

CI enforcement (`oya-check-doc-coverage-cli` / LEAN-A5) reads this catalog plus each pack's `pack.yaml` to verify required per-µservice overlay artifacts exist for every (pack × µservice in scope) pair.

---

## Active packs

| Pack | Code | Status | Lead milestone | Scope summary | Overview doc |
|---|---|---|---|---|---|
| **Korea** | `kr` | **Pack #1 — foundational** | M01–M07 | 4대보험 EDI, 연말정산, K-GAAP, HIRA/KFDA/NHIS/KHIRA, PIPA, 전자금융업/간편결제, FSS reporting, 산업안전보건법, 화물자동차운수사업법, 의료법, 119 dispatch, 더존/유비케어/비트컴퓨터 cross-walk | [kr.md](kr.md) |

## Planned packs

| Pack | Code | Status | Lead milestone | Scope summary |
|---|---|---|---|---|
| **United States** | `us` | Planned (H3) | M09, M11 | HIPAA-BAA, PCI DSS L1, SOC2 Type II, federal+50-state tax, W-2/W-4/1099/I-9/ACA, 401(k), USCDI v3, Epic/Cerner FHIR R5, ADP/Workday parity |
| **European Union** | `eu` | Planned (H3) | M10, M11 | GDPR (Art 5/6/9/17/28/32/33/35), eIDAS, SEPA DD/CT/Instant, IFRS, NIS2, DORA, MDR, eMedRec/NHS, multi-language |

## Future packs

| Pack | Code | Status | Earliest milestone | Scope sketch |
|---|---|---|---|---|
| **Japan** | `jp` | Future (H4) | M12+ | 国民健康保険, 厚生年金, 源泉徴収, インボイス制度, FSA, 医療法 (JP) |
| **Singapore** | `sea-sg` | Future (H4) | M12+ | CPF, GST, NETS, PDPA |
| **Malaysia** | `sea-my` | Future (H4) | M12+ | EPF/SOCSO, SST, GrabPay, PDPA-MY |
| **Thailand** | `sea-th` | Future (H4) | M12+ | SSO, VAT, PromptPay, PDPA-TH |
| **Vietnam** | `sea-vn` | Future (H4) | M12+ | BHXH, VAT, VNPay, PDPL |
| **Saudi Arabia** | `mena-sa` | Future (H4) | M12+ | GOSI, Zakat/VAT, mada, NCC, MoH |
| **United Arab Emirates** | `mena-ae` | Future (H4) | M12+ | DEWS, VAT, mada equivalent, ADHICS |

## Retired packs

(none)

---

## Pack lifecycle states

- **`planned`** — catalog entry exists; manifest sketched; no crates yet
- **`active`** — at least one µservice has the pack overlay shipped; ≥1 paying tenant uses it
- **`maintained`** — pack remains active; regulatory `corpus.lock` refreshed ≥quarterly with signed bumps
- **`retired`** — pack removed; all crates physically deleted; entry moved to retired-packs

See ADR-0064 §5 for lifecycle definitions and gates.

---

## Per-pack documentation suite (per ADR-0063 §2)

For every (pack × µservice in pack scope) pair, the following artifacts must exist:

- `docs/prds/<microservice>-<pack>.md` — pack overlay PRD (required when pack adds material scope)
- `docs/decisions/ADR-NNNN-<pack>-<microservice>-regulatory.md` — pack regulatory ADR
- `docs/localization-packs/<pack>/evidence/<microservice>.md` — pack acceptance evidence

CI lane `lean-a5-doc-coverage` enforces.

---

## References

- [ADR-0064 Canonical base + localization packs](../decisions/ADR-0064-canonical-base-and-localization-packs.md)
- [ADR-0063 Documentation suite coverage](../decisions/ADR-0063-documentation-suite-coverage.md)
- [MASTERPLAN §2.5, §5.5](../MASTERPLAN.md)
- Bominal ADR-0140 (inherited regional-pack pattern)
- Bominal ADR-0190 (inherited versioned regulatory corpus.lock)
