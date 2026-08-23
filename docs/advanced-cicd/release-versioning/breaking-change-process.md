---
doc_class: Process
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
adrs_cited: [ADR-0053, ADR-0052, ADR-0054]
purpose: |
  End-to-end process for introducing a breaking change: PR frontmatter, ADR
  template, 180-day sunset entry in SUNSET-LEDGER, dual reviewer-agent gate
  (change-class-reviewer + api-stability-reviewer), and major-version bump on
  next release cut. Calibrated between AWS (12 months) and Stripe (never-break).
planned_enforcement_ref: governance-deprecation-notice, governance-api-version-stability
related_adrs: [ADR-0041, ADR-0050]
doc_status: published
---

# Breaking Change Process — oyatie

> **Status:** Accepted. **Owner:** `axis-foundry`. **Date:** 2026-05-12.

## 1. What counts as breaking

A change is **breaking** if any of the following are true for an external
contract (HTTP API, SDK signature, schema, event envelope):

- A field, method, resource, enum variant, or endpoint is removed.
- A field's type changes (even to a wire-compatible type, per AIP-180).
- A field is renamed.
- A required field is added to a request body.
- A field moves into or out of a `oneOf`.
- A validation rule is tightened (max length shortened, enum closed).
- A previously-optional response field becomes "always present" in a way that
  changes consumer parsing assumptions.

These mirror Google AIP-180 and Cargo SemVer rules. Anything not on this list
is additive and ships freely under the current major.

## 2. The breaking-change funnel

```
Author idea → ADR + PR frontmatter → 180-d sunset entry → dual reviewer
            → land on dev (under flag if needed) → propagate to prod
            → next release-branch cut bumps MAJOR → old major enters
              maintenance / EOL window
```

There is no fast path. The funnel exists to make breakage expensive on the
author and cheap on the consumer.

## 3. PR frontmatter

```yaml
breaking_change: true
adr: 0073-breaking-change-foundry-capability-rename
sunset_date: "2026-11-12"      # 180 days from today
change_class: api | sdk | schema | event | cross-axis
successor_version: "v2"        # or new SDK major / new field name
migration_guide: docs/release/migrate-foundry-v1-to-v2.md
```

Missing or inconsistent frontmatter → CI fails via
`governance-deprecation-notice` (BLOCKER).

## 4. ADR template (`/templates/ADR-BREAKING-CHANGE.md`)

Sections required:

1. **Context** — what data-shape change forces the break (per Linus discipline:
   if there's no data-shape change, there's no breaking change).
2. **Alternatives considered** — explicitly enumerate the additive paths and
   why they were rejected.
3. **Migration path** — concrete code-level migration for downstream
   consumers.
4. **Sunset schedule** — `deprecated_on`, `sunset_on` (180 d later),
   `removed_on` (= sunset on the next major cut).
5. **Successor** — the replacement field / method / endpoint.
6. **Approvals** — names + roles of `change-class-reviewer` and
   `api-stability-reviewer`.
7. **Customer comms plan** — release-notes + EOL-LEDGER updates + RSS feed
   item.

## 5. Sunset ledger (`docs/release/SUNSET-LEDGER.md`)

Append-only, one row per deprecation:

```markdown
| Deprecated on | Sunset on  | Removed on | Surface | Successor | ADR  | Status   |
|---------------|------------|------------|---------|-----------|------|----------|
| 2026-05-12    | 2026-11-12 | 2026-11-12 | /foundry/v1/capability.legacy_field | capability.canonical_field | 0073 | Active   |
| 2025-09-01    | 2026-03-01 | 2026-03-01 | sdk: cloud-sdk::ComputeBuilder::with_legacy_size | ::with_size_v2 | 0061 | Removed  |
```

The ledger is read by the fitness lane and by the API server (to inject
`api-deprecation` response headers).

## 6. Dual-reviewer gate

A breaking-change PR requires **both**:

| Reviewer | Concern | What they check |
|---|---|---|
| `change-class-reviewer` | Is the change-class correctly tagged? | PR frontmatter matches reality; migration path is concrete |
| `api-stability-reviewer` | Is the break necessary, and is the contract upgrade correct? | Alternatives genuinely considered; successor design holds; sunset window adequate |

Neither agent can self-approve their own concern. Both must hit "approve"
before the PR is mergeable. CI checks this via the GH branch-protection
required-reviews matrix.

## 7. Where the break lands

Three landing patterns, in order of preference:

1. **Mint a new path-version** (`v2`) and ship the new shape there; deprecate
   the old in `v1` with `x-sunset = +180 d`. Run both for 12+ months.
2. **Additive replacement** in the same path-version: new field
   `canonical_field` lives alongside `legacy_field`; the old field carries
   `x-deprecated`. After sunset, requests that send only `legacy_field`
   receive a `400 Bad Request`.
3. **Flag-gated cutover**: behind a tenant flag, ship the new contract;
   migrate tenants individually; remove the old shape after 100% adoption.
   Best for high-volume, low-divergence schemas (Ads / Search).

Pattern 1 is the default; the ADR must justify deviation.

## 8. Patient deprecation (180-day window)

For 180 days after `deprecated_on`:

- Old surface continues to work, unchanged.
- Response header carries `api-deprecation: <sunset_date>`.
- SDK emits a runtime warning at first use.
- D14 evidence row counts requests per old surface; if usage doesn't trend
  toward zero, sunset is extended in 90-day increments (with ADR amendment).

This calibrates between AWS (12 months, generous) and Stripe (never break,
infinite). 180 days matches the enterprise-iteration tempo of oyatie.

## 9. Major-version bump on next release cut

The breaking change does NOT take effect until the next `release/X.Y` cut
where `X` increments. On that cut:

- `Cargo.toml` workspace major increments.
- Path-version `v2` becomes default; `v1` enters its 12-month LTS clock.
- EOL-LEDGER and SUNSET-LEDGER reconcile.
- All cherry-picks to the old major are limited per the EOL policy.

## 10. Security exception

A security fix MAY ship as a breaking change WITHOUT the 180-day window IF:

- A vulnerability is in active exploitation (CVSS ≥ 7.0, evidence required).
- An ADR documents the trade-off.
- Customer comms ship simultaneously.
- The next regular cycle adds a non-breaking equivalent for retrospective
  migration.

This mirrors AWS' "security or intellectual property" exception.

## 11. Lift target

`oyatie/docs/release/breaking-change-process.md` on approval.
