# app/crm reorg drain notes (`integ/crm`)

## Ownership (rule 3e)

- **Forever home:** `app/crm/**` (this rail).
- **Source (read-only):** `oya/crm/**` on `origin/dev` until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/crm/**` on this tip.

## Completed (this rail)

- Slice 1: product metadata absorb — `manifest.json`, `README.md`, `IPs/**`, `slos/**`.
- Slice 2: top-level IP docs + audit — `IP-024-*.md`, `IP-025-*.md`, `AUDIT-FINDINGS-*.json`.

## Next gaps (ordered)

1. **Contracts + policy** — `contracts/`, `policy/`, `cedar/`, `catalog/` from `oya/crm`.
2. **Capabilities + crates** — bounded-context manifests and `oya-crm-*` crate rehome (exclude procurement eviction judgment).
3. **IaC + dashboards** — `iac/`, `dashboards/`, `runbooks/`, `scorecards/`, `dpia/`, `decisions/`, `evidence/`.
4. **Shrink-only burn** — after verify, delete absorbed paths on `integ/oya` (not this rail).

## Out of envelope (do not touch from `integ/crm`)

- `oya/crm/**` deletes — `integ/oya` shrink-only rail only.
- Other products under `oya/*` or `app/*`.
- Hub retargets (`specs/**`) — tip-free `integ/specs` only.
