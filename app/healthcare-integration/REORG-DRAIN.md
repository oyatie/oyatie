# app/healthcare-integration reorg drain notes (`integ/healthcare-integration`)

## Ownership (rule 3d / 3e)

- **Forever home:** `app/healthcare-integration/**` (this rail).
- **Source:** deleted on trunk by MERGED #1611; reclaim from pre-delete SHA `4aa692919b12`.
- **Writes:** only under `app/healthcare-integration/**` on this tip.
- **OVERRULE 3d / DECIDED FLAT (discovery 90/91):** product rail owns `app/healthcare-integration/**` — never dump onto `integ/app` or nest under `integ/healthcare` / `app/healthcare/<ctx>`.
- **BAN:** `integ/healthcare` mega-rail and `app/healthcare/**` as product forever homes (`#1678` SUPERSEDED).
- **Soft suite only:** portfolio/Kanban label `healthcare` is non-authoritative SKU cluster (siblings: `emr` · `pharmacy` · `patient-monitoring` · `healthcare-integration`) — never `destination_integ` / envelope root.
- **Shared HIPAA/PHI/Cedar:** packs/caps (`compliance`/`iam`/`audit`/`data`), not a parent product directory.

## Completed

- Wave-1 absorb: product scaffold reclaim `oya/healthcare-integration/**` → `app/healthcare-integration/**` from `4aa692919b12` (AUDIT-FINDINGS excluded as dump-class).
- Judgment: envelopes **1.16.11+** per-product rail (OVERRULE 1.16.9 nest + #1611 delete).
- Wave-2 deepen (FLAT): in-tree durable cites `microservices/healthcare-integration/` → `app/healthcare-integration/`; REORG-DRAIN encodes multi-product grain.

## Remaining

1. Verify destination tip contains forever bytes (this tip) — Wave-1+2 complete for forever faces.
2. Shrink N/A on trunk (already deleted). Do not resurrect under `oya/`.
3. Hub retargets / `app_products_note` soft-suite amend — tip-free `integ/specs` only (**no specs push** NOW; PARKED after/separate from NO_RAIL → 1.16.14).
4. PARKED — no merge to `dev` until wave review. Observation≠APPROVE. **STOP #1661.**

## Out of envelope

- `oya/healthcare-integration/**` resurrect or deletes — N/A / not this rail.
- `Cargo.lock` / root workspace membership — lock tip only.
- `specs/**` hub edits — `integ/specs` only (**no specs push** from this rail).
- Sibling products under `app/*` other than `healthcare-integration`.
- Nest under `app/healthcare/healthcare-integration/**` or birth `integ/healthcare` — **BAN**.
- `#1661` product shrink — STOP (do not touch).
