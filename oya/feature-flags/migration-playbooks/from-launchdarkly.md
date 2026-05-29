---
doc_class: MigrationPlaybook
microservice: feature-flags
vendor: LaunchDarkly
date: 2026-05-20
doc_status: published
---

# Migration playbook — LaunchDarkly → oyatie feature-flags

Audience: a tenant or internal owner consolidating their LaunchDarkly footprint onto the oyatie substrate. Assumes the source LaunchDarkly project is "server-side-and-relay" mode (the most common Enterprise pattern); pure client-side projects are noted at the bottom.

## Prerequisites

- LaunchDarkly API token with `Reader` on the source project + `Writer` on archive.
- oyatie tenant in paid or paid capability tier (per `tenant-class/tier-matrix.md`).
- `oya-dev-cli` ≥ 1.42.0 (the LaunchDarkly importer landed in this version).

## Step 1 — Inventory (LaunchDarkly side)

Run `oya ff import inventory --vendor launchdarkly --project <ld-project-key>`. The command pages the LaunchDarkly REST API and emits `inventory.yaml` listing every flag, its variants, its targeting rules, its segments, and its rollout %. Expected runtime: ≤ 30 s per 1 000 flags.

Manually classify each flag in `inventory.yaml`:

- `release_toggle`: maps to oyatie `release_toggle` lifecycle.
- `experiment`: maps to oyatie `experiment` lifecycle.
- `permission`: maps to oyatie `permission_toggle` lifecycle.
- `kill-switch` / `ops`: maps to oyatie `kill_switch` lifecycle.
- `unknown`: human review required — open a per-flag ticket.

Anything flagged `unknown` BLOCKERs the migration; LaunchDarkly does not enforce lifecycle classification, so this gap is normal — budget 30 minutes per 100 unknown flags for human review.

## Step 2 — Segment-to-cohort translation

LaunchDarkly Segments map to oyatie cohorts via the `analytics` materialised-audience surface. The importer auto-generates a draft `audience.yaml` per Segment. Verify the predicate translation — LaunchDarkly's `email ends with @acme.com` becomes the oyatie SQL `WHERE tenant_id IN (SELECT tenant_id FROM tenant.metadata WHERE primary_email_domain = 'acme.com')`. The semantics differ when LaunchDarkly's Segment is per-user but oyatie's cohort is per-tenant; if a Segment is per-user, you need a `cohort_member_user.yaml` (per-user cohort form) — the importer adds these but flags them as `human-review-required`.

Pay attention to Big Segments (LD's bulk-membership feature) — they map to oyatie's `analytics.bulk_audience` which has different refresh semantics (bulk audiences refresh hourly, not 5-minutely). For any flag targeting a Big Segment, expect a 1 h freshness delta vs LaunchDarkly's 30 s.

## Step 3 — Cedar fragment authoring

The importer cannot auto-translate LaunchDarkly's JSON rule expressions to Cedar — they are semantically different (LaunchDarkly uses an OR-of-AND tree; Cedar uses Datalog-style permits). The importer emits Cedar skeletons; you edit them.

Sample translation:

LaunchDarkly rule:

```json
{
  "clauses": [
    {"attribute": "country", "op": "in", "values": ["US", "CA"]},
    {"attribute": "betaOptIn", "op": "is", "values": [true]}
  ],
  "variation": 1
}
```

oyatie Cedar fragment:

```cedar
permit(principal, action == Action::"evaluate-flag-yourµservice.beta.dashboard", resource)
when {
  ["US", "CA"].contains(principal.country) &&
  principal.beta_opt_in == true
};
```

Note the attribute-name convention shift (LaunchDarkly camelCase → oyatie snake_case per the per-microservice naming policy).

## Step 4 — Shadow-evaluate

Stand the oyatie evaluator beside LaunchDarkly for ≥ 7 days with shadow-mode traffic. The oyatie SDK in shadow mode emits both the LaunchDarkly variant and the oyatie variant to the audit-chain; the lane `oya-governance-flag-shadow-delta` flags any per-flag delta > 0.5 %.

Pay close attention to:

- Percentage-rollout determinism. LaunchDarkly's bucket-by-key uses MD5; oyatie uses xxHash3. A tenant that was at 50 % in LaunchDarkly will likely land in a different bucket in oyatie even at the same percentage. Plan for 100 % rollout *first*, observe steady-state, then dial down — do not dial up from 0 % in oyatie if LaunchDarkly was already at 50 %, because a non-trivial cohort will see the feature flip variant during cutover.
- Default-when-missing-attribute. LaunchDarkly treats missing attributes as "rule does not match" → default. oyatie's Cedar treats missing attributes as deny-on-evaluation → default. Same surface outcome but different internal flow.

## Step 5 — Cutover

Per flag, in a sustained 30 % → 70 % → 100 % traffic cutover over ≤ 7 days:

1. Tag the LaunchDarkly flag `archived-pending-oyatie-migration`.
2. Switch the SDK call site to oyatie's `FFClient.evalBool(...)`.
3. Watch the `feature-flags-cutover` Grafana dashboard for the `evaluations_total` line moving from LaunchDarkly to oyatie.
4. After 24 h of pure-oyatie traffic with zero delta in the downstream product metric, archive the LaunchDarkly flag (`Maintainer` role required in LaunchDarkly).

## Step 6 — LaunchDarkly project sunset

Once every flag is archived, run:

```sh
oya ff import sunset-evidence \
    --vendor launchdarkly --project <ld-project-key> \
    --out evidence/migrations/launchdarkly-<ld-project-key>-sunset.json
```

The evidence file is committed and forms the auditor-visible "migration completed" record per the `oya-governance-migration-evidence` lane.

## Edge cases

- **LaunchDarkly client-side flags (mobile / browser).** Not in this playbook — oyatie's `feature-flags` is server-side-only per PRD §Scope. Migrate to oyatie's `client-feature-flags` µservice (separate; see its migration playbook).
- **LaunchDarkly Experimentation product.** Not migrated by this playbook — that's the future `experiments` µservice scope. Re-instantiate experiments using `analytics` outcome tables and the cohort-rollout tutorial.
- **LaunchDarkly Big Segments > 10M entries.** Currently rate-limited at 100K entries / 5 min into oyatie's bulk-audience materialiser. Plan multi-day backfill.
- **LaunchDarkly Code References scanner.** No direct oyatie equivalent; use `oya code-search ff` for similar capability.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Bucket-determinism delta during cutover | High | 100 % rollout first; then dial down. Never dial up across the cutover. |
| Cedar fragment mistranslation | High | Pair-review every fragment; lane `feature-flag-cedar-fragment-shape` enforces shape. |
| Segment refresh-cadence delta (5 min oyatie vs 30 s LD) | Medium | Document the freshness in the SLO; update the `flag-eligibility-freshness` lane if needed. |
| LaunchDarkly Experiments untranslatable | Medium | Defer to `experiments` µservice; do not block migration on this. |
| BIG Segments backfill capacity | Low-Medium | Schedule overnight; coordinate with the analytics on-call. |
