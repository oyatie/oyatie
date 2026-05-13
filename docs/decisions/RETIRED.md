# Legacy ADRs — RETIRED 2026-05-09

> Per user directive 2026-05-09: "We are retiring all the old documents. make sure we have everything we need and I want you to write everything. just copy and paste is a no go."

## What this means

The 127 legacy ADRs at `decisions/ADR-NNNN-<slug>.md` are **RETIRED**. They:

- **Remain** in the directory for forensic / git-blame integrity (per Oyatie anti-pattern: "Don't delete legacy ADRs")
- Are **NOT** referenced by active consolidated docs at `docs/`
- Are **NOT** the source of truth for any new decision
- Are **superseded** in aggregate by the new self-contained pack at [`docs/decisions/`]() (50 NEW ADRs, originally authored 2026-05-09)

## Where to find decisions now

| Need | Look here |
|---|---|
| New decisions ledger | [`docs/decisions/`]() (ADR-0001..0051) |
| Index of new pack | [`docs/decisions/README.md`](README.md) + [`docs/ADR-INDEX.md`](../ADR-INDEX.md) (the latter to be regenerated as the new pack's index) |
| Consolidation strategy + supersession map | [`docs/ADR-CONSOLIDATION-PLAN.md`](../ADR-CONSOLIDATION-PLAN.md) |
| Forensic legacy index | (eventually) `docs/ADR-INDEX-LEGACY.md` snapshot of pre-2026-05-09 state |
| Per-axis decisions | per-microservice ADR cluster in `docs/decisions/` (e.g. Cloud = ADR-0028, Foundry = ADR-0020..0027, etc.) |

## Citation rules

**Effective 2026-05-09:**

1. Active consolidated docs (`docs/`) MUST NOT cite legacy `ADR-NNNN` numbers (where NNNN ≤ 0233 and predates 2026-05-09 numbering shift). Citations are restricted to the new pack ` ADR-0001..ADR-0051` (and future additions).
2. Forensic mention is allowed in: ADR-CONSOLIDATION-PLAN.md, CONTRADICTION-LEDGER.md, this RETIRED.md, and any `*-LEGACY.md` snapshot file.
3. CI lane `oya-foundry-fitness-adr-citation` enforces.

## Why retire (not delete)

- Forensic value: every legacy ADR may have shaped current code; commits reference them; git-blame should resolve
- Compliance value: regulators audit decision history; retention is mandatory
- Educational value: future engineers may want to see what was decided + why + what changed
- Cohesion value: the supersession-graph (in [`ADR-CONSOLIDATION-PLAN.md`](../ADR-CONSOLIDATION-PLAN.md) §3) maps every legacy decision to its new-pack successor; deleting breaks the audit chain

## Migration timeline

- 2026-05-09: New pack drafted; 21+ ADRs in `docs/decisions/` (50 target)
- 2026-05-09 (same day): This RETIRED.md note added; consolidated docs sweep started to remove legacy ADR-NNNN refs
- Per [`ADR-CONSOLIDATION-PLAN.md`](../ADR-CONSOLIDATION-PLAN.md): `crew-adr-promotion` produces weekly diff of legacy → new pack supersession; consolidated docs updated per cadence
- Future: optional move of legacy files to `decisions/legacy/` subdirectory (open question per ADR-CONSOLIDATION-PLAN §8)

## Council ratification

The retirement was directed by the user (founder) on 2026-05-09. `council-architecture` + `crew-adr-promotion` ratify the pack governance per [`docs/decisions/README.md`](README.md) §3.

## Sources

- User directive 2026-05-09 (verbatim above)
- [`docs/ADR-CONSOLIDATION-PLAN.md`](../ADR-CONSOLIDATION-PLAN.md)
- [`docs/decisions/README.md`](README.md)
- All 127 legacy ADR files in this directory (forensic; do not edit)
