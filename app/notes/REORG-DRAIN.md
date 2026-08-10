# app/notes reorg drain notes (`integ/notes`)

## Ownership (rule 3e)

- **Forever home:** `app/notes/**` (this rail).
- **Source (read-only):** `oya/notes/**` on `origin/dev` until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/notes/**` on this tip.

## Completed (this rail)

- Slice 1: product metadata absorb — `manifest.json`, `README.md`, `slos/**`.

## Next gaps (ordered)

1. **Contracts + policy** — `contracts/`, `policy/`, `cedar/`, `catalog/` from `oya/notes`.
2. **Capabilities + crates** — bounded-context manifests and `oya-notes-*` crate rehome.
3. **IaC + dashboards** — `iac/`, `dashboards/`, `runbooks/`, `scorecards/`, `decisions/`, `dpia/`, `IPs/`.
4. **Shrink-only burn** — after verify, delete absorbed paths on `integ/oya` (not this rail).

## Out of envelope (do not touch from `integ/notes`)

- `oya/notes/**` deletes — `integ/oya` shrink-only rail only.
- Other products under `oya/*` or `app/*`.
- Hub retargets (`specs/**`) — tip-free `integ/specs` only.
