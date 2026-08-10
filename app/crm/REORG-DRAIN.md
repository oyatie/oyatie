# app/crm reorg drain notes (`integ/crm`)

## Ownership (rule 3e)

- **Forever home:** `app/crm/**` (this rail).
- **Source (read-only):** `oya/crm/**` on `origin/dev` / this tip until shrink-only delete lands on `integ/oya`.
- **Writes:** only under `app/crm/**` on this tip.

## Completed (this rail)

- Slice 1: product metadata absorb — `manifest.json`, `README.md`, `IPs/**`, `slos/**`.
- Slice 2: top-level IP docs + audit — `IP-024-*.md`, `IP-025-*.md`, `AUDIT-FINDINGS-*.json`.
- **Wave-1 crates absorb:** 2 CRM crates (14 `.rs`) restored from `PRE=2a3dc1ebb^` / tip `oya/crm/crates/` → `app/crm/crates/`. BUCK cites retargeted `//oya/crm/` → `//app/crm/`.
- **Procurement eviction:** `oya-procurement-source-to-pay-domain` **not** absorbed into `app/crm/**` (see Elevate).

## Inventory (absorbed crates)

| Dir under `app/crm/crates/` | Face | `.rs` |
|-----------------------------|------|------:|
| `oya-crm-customer-engagement-domain` | domain | 2 |
| `oya-crm-revenue-app` | app | 12 |

Source dual-home remains under `oya/crm/crates/` until `integ/oya` shrink-only delete.

## Elevate (out of envelope)

1. **procurement rail** — rehome `oya/crm/crates/oya-procurement-source-to-pay-domain` (2 `.rs`) out of CRM forever home. Candidate path: `app/procurement/**` or capability registry destination once an `integ/procurement` envelope exists. Do **not** keep under `app/crm/**`.
2. **integ/oya** — shrink-only delete drained `oya/crm/**` after verify (including leftover procurement until procurement rail absorbs it).
3. **integ/specs** — hub / `specs/microservices/{crm,procurement}.json` path retargets; stale `microservices/crm` refs.

## Next gaps (ordered)

1. **Contracts + policy** — `contracts/`, `policy/`, `cedar/`, `catalog/` from `oya/crm`.
2. **Capabilities** — bounded-context manifests under `capabilities/`.
3. **IaC + dashboards** — `iac/`, `dashboards/`, `runbooks/`, `scorecards/`, `dpia/`, `decisions/`, `evidence/`.
4. **Shrink-only burn** — after verify, delete absorbed paths on `integ/oya` (not this rail).

## Out of envelope (do not touch from `integ/crm`)

- `oya/crm/**` deletes — `integ/oya` shrink-only rail only.
- Procurement forever-home writes — procurement rail only.
- Other products under `oya/*` or `app/*`.
- Hub retargets (`specs/**`) — tip-free `integ/specs` only.

## Reclaim (scaffold-vs-dump audit)
- Completed remaining oya/crm → app/crm absorb after premature integ/oya shrink (`2a3dc1ebb`): catalog/contracts/policy/cedar/iac/runbooks/dashboards/capabilities (~105 files).
- **Procurement remains evicted** from `app/crm/**` (elevate to procurement rail) — do not re-absorb `oya-procurement-source-to-pay-domain`.
- Source still on origin/dev for procurement until that rail exists.
