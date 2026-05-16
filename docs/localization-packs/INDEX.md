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
doc_status: published
---

# Localization Pack Catalog

This is the canonical catalog of oyatie localization packs. Every active pack has a dedicated overview doc at `docs/localization-packs/<code>.md` and a manifest at `docs/localization-packs/<code>/pack.yaml`.

CI enforcement (`oya-check-doc-coverage-cli` / LEAN-A5) reads this catalog plus each pack's `pack.yaml` to verify required per-µservice overlay artifacts exist for every (pack × µservice in scope) pair.

---

## Active packs

(none yet — KR pack will flip from `planned/foundational` to `active` when promotion blockers in `kr/pack.yaml` are met; see kr.md frontmatter)

## Planned packs

| Pack | Code | Status | Lead milestone | Manifest | Overview |
|---|---|---|---|---|---|
| **Korea** | `kr` | **planned/foundational (pack #1)** | M01–M07 | [kr/pack.yaml](kr/pack.yaml) | [kr.md](kr.md) |
| **United States** | `us` | Planned (H3) | M09, M11 | TBD | TBD |
| **European Union** | `eu` | Planned (H3) | M10, M11 | TBD | TBD |

Scope summary — see each pack's `pack.yaml` manifest for authoritative `microservices_in_scope` and `regulatory_bindings` lists. KR pack covers ~28 µservices across Workforce / Healthcare / FinTech / Industrial / Connect / Hospitality clusters; full list and `material_scope` per µservice in `kr/pack.yaml`.

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
