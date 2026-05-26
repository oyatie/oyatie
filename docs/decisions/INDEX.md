---
purpose: Redirect stub — the canonical ADR index is generated at docs/ADR-INDEX.md (SSOT per ADR-0364).
doc_status: published
last_audited: 2026-05-26
---

# Oyatie Decisions Index — moved (single source of truth)

This file is **no longer hand-maintained**. The canonical ADR index is **generated** and lives at:

- **Human index (canonical):** [`docs/ADR-INDEX.md`](../ADR-INDEX.md) — regenerate with `oya doc adr-index --write`
- **Machine-readable mirror:** `docs/machine-readable/decisions.json`
- **Masterplan projection** (generated from ADR `deliverables`, per ADR-0364): `docs/machine-readable/masterplan.generated.json`

Per ADR-0364 (generated masterplan) + the SSOT / flat-no-grouping doctrine, the previous hand-maintained
table here had drifted (it showed a stale ADR span) and was a duplicate of the generated index. This
stub remains only to keep existing inbound links resolving and to point at the single source of truth
above. Do not re-add hand-maintained rows here.
