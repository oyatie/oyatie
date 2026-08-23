---
doc_class: Policy
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  Define LTS support window, EOL signaling, and per-major-version end-of-life
  process for oyatie. 12 months from major release; 90-day pre-EOL warning;
  EOL-LEDGER as the single source of truth for support status.
planned_enforcement_ref: governance-version-eol-warning
related_adrs: [ADR-0041, ADR-0050]
doc_status: published
---

# Version EOL Policy — oyatie

> **Status:** Accepted. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. Support window

**Every major version is LTS for 12 months from its `vX.Y.0` tag date.**

| Phase | Duration from tag | What ships |
|---|---|---|
| Active | 0 → 6 months | features (minor bumps) + patches |
| Maintenance | 6 → 9 months | security + critical-bug patches only |
| Pre-EOL warning | 9 → 12 months | security patches only; EVT-EOL-APPROACHING emitted |
| EOL | 12 months | branch archived; no further patches |
| Archive | 12+ months | read-only; `EOL` flag visible on tag |

Calibration: AWS gives 12 months (customer-agreement), Kubernetes ~14 months
(3 minor releases), .NET LTS = 3 years, Java LTS = 5+ years. Oyatie's
12 months is the median commitment that matches enterprise expectation
without freezing forward motion.

## 2. EOL ledger (`docs/release/EOL-LEDGER.md`)

One row per major version. The file is append-only; rows are never deleted.

```markdown
| Major | Tag | Released   | Active until | Maintenance until | Pre-EOL warning | EOL date    | Status        |
|-------|-----|------------|--------------|-------------------|-----------------|-------------|---------------|
| 3.0   | v3.0.0 | 2026-01-15 | 2026-07-15   | 2026-10-15        | 2026-10-15      | 2027-01-15  | Active        |
| 2.0   | v2.0.0 | 2025-05-01 | 2025-11-01   | 2026-02-01        | 2026-02-01      | 2026-05-01  | EOL'd 2026-05-01 |
```

The ledger is the source of truth read by the fitness lane and by the
`api-deprecation` response header on running services.

## 3. EOL signaling (90-day notice)

`governance-version-eol-warning` (HIGH severity) emits
`EVT-VERSION-EOL-APPROACHING` exactly 90 days before EOL. The event triggers:

1. PR comment on every active PR targeting the EOL'ing release branch.
2. Customer-comms artifact dropped under `docs/release/notices/EOL-<major>.md`.
3. Response header on the running API: `api-deprecation: <eol-date>`.
4. Per-axis playbook owner is paged via the configured notification channel.
5. Migration guide auto-stub: `docs/release/migrate-vX-to-vX+1.md`.

## 4. EOL day (D-0)

On the EOL date:

1. `release/X.Y` branch protection switches to read-only.
2. Final patch tag is minted with the marker: `vX.Y.<final>-eol`.
3. `EOL-LEDGER.md` row status flips to `EOL'd YYYY-MM-DD`.
4. `release-cherry-pick` agent refuses all further cherry-picks targeting `X.Y`.
5. Service operators see `api-deprecation: <today>` and a 410 Gone trap
   for any service path with `x-sunset <= today`.
6. `EVT-VERSION-EOL` emitted to D14.

## 5. Security-fix exception

Within the 12-month window, security patches MAY ship to a release branch via
the standard cherry-pick path. Outside the window:

- Extended-support contract holders MAY request a backport via the
  break-glass ADR.
- The agent will mint `vX.Y.<final+1>-security` ONLY with an approved
  ADR + signed-off operator.
- All extended-support patches are logged in `docs/release/EXTENDED-SUPPORT.md`.

This mirrors AWS' "economic / technical burden" exception in its
12-month policy.

## 6. Per-axis EOL nuance

| Axis | LTS source of truth | Notes |
|---|---|---|
| Foundry | the platform major | Foundry-axis EOL = product EOL |
| Cloud | the platform major | Same |
| SaaS | the platform major | Hosted; auto-migrated by operator |
| Workspace | the platform major | Same |
| Vertical-pack | per-pack major | Some packs may EOL early per partner contract |
| Search | the platform major | Index hot-swap |
| Ads | the platform major | Auction model versioned with platform |

## 7. EOL of API path-versions

Independent from product EOL but on the same 12-month rhythm:

- A path-version (`v1`, `v1beta1`, `v2`) carries `x-sunset` 12 months after
  `x-deprecated`.
- At sunset: requests return `410 Gone` with a `Link: <migration-guide>;
  rel="successor-version"` header.
- Beta path-versions may sunset earlier (90 days minimum notice; matches Azure
  preview).

## 8. Communication contract

Three customer-facing channels signal EOL:

1. `api-deprecation` response header (per Kubernetes pattern).
2. `docs/release/notices/EOL-<major>.md` (markdown release notes).
3. `governance-eol-feed` (RSS/Atom feed at `/foundry/v1/eol-feed`).

Customers and downstream operators subscribe to the feed for advance notice.

## 9. Enforcement (`governance-version-eol-warning`)

Severity: HIGH (escalates to BLOCKER on EOL day for PRs targeting an EOL'd
branch).

Checks (run nightly):
1. For each row in `EOL-LEDGER.md`, compute days-to-EOL.
2. If `≤ 90`, emit `EVT-VERSION-EOL-APPROACHING`.
3. If `≤ 0`, switch branch protection + refuse cherry-picks.
4. For each `x-deprecated` path-version, compute days-to-sunset; same
   thresholds.

## 10. Lift target

`oyatie/docs/release/version-eol-policy.md` on approval.
