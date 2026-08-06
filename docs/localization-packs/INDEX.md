---
doc_class: LocalizationPackCatalog
shape: anchor
status: Accepted
date: 2026-05-13
owners: ["council-architecture"]
authority_chain: docs/MASTERPLAN.md §2.5, §5.5 → ADR-0064 → this file → docs/localization-packs/<pack>.md
companion_docs:
  - docs/MASTERPLAN.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
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

Scope summary — see each pack's `pack.yaml` manifest for authoritative `microservices_in_scope` and `regulatory_bindings` lists. KR pack covers ~28 µservices across Workforce / Healthcare / FinTech / Industrial / / Hospitality clusters; full list and `material_scope` per µservice in `kr/pack.yaml`.

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


## Repository-local pack directories

This table is a traceability inventory only. A listed directory does not imply tenant adoption, executable runtime evidence, or a shipped localization pack; lifecycle status remains governed by the pack docs/manifests and promotion gates.

| Pack directory | Repository path | Documentation status |
|---|---|---|
| `au` | [`../../packs/au/`](../../packs/au/) | Repository-local localization docs present; no `docs/localization-packs/au.md` overview yet. |
| `br` | [`../../packs/br/`](../../packs/br/) | Repository-local localization docs present; no `docs/localization-packs/br.md` overview yet. |
| `cn` | [`../../packs/cn/`](../../packs/cn/) | Repository-local compliance pack docs present; no `docs/localization-packs/cn.md` overview yet. |
| `eu` | [`../../packs/eu/`](../../packs/eu/) | Repository-local localization/sovereignty docs present; `docs/localization-packs/INDEX.md` tracks EU as planned. |
| `in` | [`../../packs/in/`](../../packs/in/) | Repository-local localization docs present; no `docs/localization-packs/in.md` overview yet. |
| `jp` | [`../../packs/jp/`](../../packs/jp/) | Repository-local localization/sovereignty docs present; index tracks JP as future. |
| `kr` | [`../../packs/kr/`](../../packs/kr/) | Repository-local localization/sovereignty docs plus [`kr.md`](kr.md) and [`kr/pack.yaml`](kr/pack.yaml). |
| `ksa` | [`../../packs/ksa/`](../../packs/ksa/) | Repository-local sovereignty overlay present; no `docs/localization-packs/ksa.md` overview yet. |
| `mx` | [`../../packs/mx/`](../../packs/mx/) | Repository-local localization docs present; no `docs/localization-packs/mx.md` overview yet. |
| `us` | [`../../packs/us/`](../../packs/us/) | Repository-local localization/sovereignty docs present; index tracks US as planned. |

## Pack lifecycle states

- **`planned`** — catalog entry exists; manifest sketched; no crates yet
- **`active`** — at least one µservice has the pack overlay shipped; ≥1 paying tenant uses it
- **`maintained`** — pack remains active; regulatory `corpus.lock` refreshed ≥quarterly with signed bumps
- **`retired`** — pack removed; all crates physically deleted; entry moved to retired-packs

See ADR-0064 §5 for lifecycle definitions and gates.

---

## Per-pack documentation set (per ADR-0063 §2)

For every (pack × µservice in pack scope) pair, the following artifacts must exist:

- `docs/prds/<microservice>-<pack>.md` — pack overlay PRD (required when pack adds material scope)
- `docs/decisions/ADR-####-<pack>-<microservice>-regulatory.md` — pack regulatory ADR
- `docs/localization-packs/<pack>/evidence/<microservice>.md` — pack acceptance evidence

CI lane `lean-a5-doc-coverage` enforces.

---

## References

- [ADR-0064 Canonical base + localization packs](../decisions/ADR-0064-canonical-base-and-localization-packs.md)
- [ADR-0063 Documentation set coverage](../decisions/ADR-0063-documentation-set-coverage.md)
- [MASTERPLAN §2.5, §5.5](../MASTERPLAN.md)
- Bominal ADR-0140 (retired per ADR-0145) (inherited regional-pack pattern)
- Bominal ADR-0190 (inherited versioned regulatory corpus.lock)
