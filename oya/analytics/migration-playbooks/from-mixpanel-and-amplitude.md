---
doc_class: MigrationPlaybook
microservice: analytics
vendor: Mixpanel + Amplitude (parallel migration; both SaaS event-analytics)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Mixpanel / Amplitude → oyatie analytics

Audience: an oyatie tenant or internal product team moving from a managed event-analytics SaaS (Mixpanel or Amplitude) to oyatie's native `analytics` µservice without losing historical funnels, retention cohorts, or in-flight dashboards.

## Why this migration matters

Mixpanel and Amplitude are excellent managed products but they put a third-party processor in the data path. For tenants under GDPR Art. 28, KR-PIPA cross-border-transfer constraints, HIPAA BAA requirements, or SEC 17a-4(f) WORM retention, the sub-processor list extension is a structural blocker. oyatie's `analytics` µservice keeps tenant data in-pack; the funnel / cohort / dashboard queries run on the same data the tenant already owns.

Counterpart vendors use pricing tiers; Oyatie does not — tenant_class is binary.

The migration is non-trivial because:

- Mixpanel and Amplitude have event-shape conventions (distinct_id, super-properties, event-property hierarchy) that don't 1:1 to ClickHouse's flat-row outbox shape.
- Mixpanel's `funnel` and Amplitude's `funnel_v2` are managed primitives — oyatie reproduces these via `windowFunnel()` + a query-template library.
- Retention cohort definitions in Mixpanel/Amplitude UI need to be rewritten as ClickHouse SQL (or as oyatie's cohort DSL once it lands per the M04 cohort-engine ADR).

## Step 1 — Export historical event data (≤ 2-5 days per 1 B events)

Mixpanel:

```sh
# Mixpanel raw-data export API
curl -G "https://data.mixpanel.com/api/2.0/export/" \
    --data-urlencode "from_date=2024-01-01" \
    --data-urlencode "to_date=2026-05-20" \
    -u "$MIXPANEL_API_SECRET:" \
    > mixpanel-export.jsonl
```

The Mixpanel raw export API rate-limits at ~ 60 GB/h. For a 1 B event corpus, plan 24-72 h of export time.

Amplitude:

```sh
# Amplitude bulk export API
curl -G "https://amplitude.com/api/2/export" \
    --data-urlencode "start=20240101T00" \
    --data-urlencode "end=20260520T23" \
    -u "$AMPLITUDE_API_KEY:$AMPLITUDE_SECRET_KEY" \
    > amplitude-export.zip
```

Amplitude exports as JSONL inside a zip; one file per hour. Rate limit ~ 100 GB/h.

## Step 2 — Convert event schema to oyatie outbox shape (≤ 8-24 h depending on event volume)

```sh
oya analytics migrate convert \
    --source-format mixpanel \
    --input mixpanel-export.jsonl \
    --output oyatie-events.parquet \
    --tenant drill-acme \
    --event-mapping mappings/mixpanel-to-oyatie.yaml
```

The converter handles:

- `distinct_id` (Mixpanel) / `user_id` (Amplitude) → `user_id` (oyatie). One-to-one.
- `time` (Mixpanel epoch seconds) / `event_time` (Amplitude ISO 8601) → `event_time DateTime64(3)` (oyatie).
- Super-properties (Mixpanel) / user-properties (Amplitude) → flattened to columns with `user_prop_*` prefix.
- Event-properties (both) → flattened to columns with `event_prop_*` prefix.
- `event_name` (both) → `event_name LowCardinality(String)` (oyatie; LowCardinality saves disk for repeated strings).

Field-level deltas:

| Mixpanel field | Amplitude field | oyatie field | Notes |
|---|---|---|---|
| `distinct_id` | `user_id` | `user_id` | Direct. If both anonymous and identified users are present, use `device_id` for anonymous. |
| `time` (epoch sec) | `event_time` (ISO) | `event_time DateTime64(3)` | Mixpanel emits second precision; Amplitude emits millisecond. ClickHouse stores millisecond. |
| `$insert_id` | `insert_id` | `event_id String` | Use for idempotent ingest dedup. |
| `$ip` | `ip_address` | `ip_address IPv6` | Use IPv6 for both v4 and v6 (`toIPv6()`); deal with GDPR redaction at ingest. |
| `$city`, `$region`, `$country_code` | `city`, `region`, `country` | `geo_city LowCardinality(String)`, `geo_region LowCardinality(String)`, `geo_country FixedString(2)` | Direct. |
| `mp_session_id` | `session_id` | `session_id UInt64` | If absent, derive at conversion via session-window logic. |

The converter flags for human review:

- Custom property names with non-ASCII / special characters — oyatie convention prefers `snake_case`.
- Events with `revenue` super-property — these need to map to a `revenue Decimal128(2)` column for SUMs; the converter draft-emits but flags for tenant sign-off.
- PII-tagged properties (email, phone, IP-address) — these need explicit data-class markers per ADR-0156 PII registry; the converter emits a draft `tags.yaml` with proposed `data_class` levels for tenant approval.

## Step 3 — Stage the converted data into a tenant cell (≤ 6 h per 100 M events)

```sh
oya analytics ingest backfill \
    --tenant drill-acme \
    --table workflow_event \
    --input oyatie-events.parquet \
    --target-cell drill-syd-1 \
    --throttle-rate 50000-rows-per-sec
```

The throttle keeps backfill within the tenant's tenant_class quota (`50 k rows/sec` leaves head-room for live ingest under paid per_usage billing).

Verify after backfill:

```sql
SELECT
    toYearWeek(event_time) AS week,
    count() AS oyatie_count
FROM tenant_drill_acme.workflow_event
GROUP BY week
ORDER BY week;
```

Cross-check the per-week counts against your Mixpanel/Amplitude UI's per-week totals. Acceptable drift: < 0.1 % per week. Larger drift usually means the export missed late-arriving events; re-export the affected weeks.

## Step 4 — Rewrite funnel + cohort queries (≤ 3-5 days per dashboard)

Mixpanel funnel UI definition (example: "Signup → Activation → First-purchase, 30-day window"):

```
Funnel: Signup → Activation → First-purchase
Window: 30 days
Breakdown: country
```

oyatie equivalent SQL:

```sql
SELECT
    geo_country,
    sumIf(1, level >= 1) AS signed_up,
    sumIf(1, level >= 2) AS activated,
    sumIf(1, level >= 3) AS first_purchased,
    round(sumIf(1, level >= 2) * 100.0 / sumIf(1, level >= 1), 2) AS activation_pct,
    round(sumIf(1, level >= 3) * 100.0 / sumIf(1, level >= 1), 2) AS first_purchase_pct
FROM (
    SELECT
        user_id,
        geo_country,
        windowFunnel(2592000)(
            event_time,
            event_name = 'signup',
            event_name = 'activation',
            event_name = 'first_purchase'
        ) AS level
    FROM tenant_drill_acme.workflow_event
    WHERE event_time >= now() - INTERVAL 30 DAY
    GROUP BY user_id, geo_country
)
GROUP BY geo_country
ORDER BY signed_up DESC;
```

The conversion table for common Mixpanel/Amplitude primitives:

| Mixpanel / Amplitude primitive | oyatie ClickHouse equivalent |
|---|---|
| Funnel (ordered steps + window) | `windowFunnel(window_seconds)(event_time, cond1, cond2, …)` |
| Retention cohort (week-N retention) | `retention(cond1, cond2, …)` aggregate + `arrayElement` for week-N |
| Unique users in time window | `uniqExact(user_id)` or `uniqHLL12(user_id)` for approximate |
| Active-user count | `uniqExactIf(user_id, event_time > now() - INTERVAL 1 DAY)` etc. |
| Breakdown by property | `GROUP BY user_prop_*` |
| Path analysis (user journey) | `sequenceMatch` / `sequenceCount` |
| Cohort time-to-event | `min(event_time) - signup_time` per user |

## Step 5 — Shadow dashboards (≤ 14 days)

The target oyatie dashboards shadow the Mixpanel / Amplitude UI:

- Pin the new dashboards in oyatie; keep the old SaaS dashboards live.
- Daily cron job compares each headline metric (active users, funnel conversion, retention) between the two; expects < 1 % drift.
- After 14 d of < 1 % drift on the top-N dashboards, cut over to oyatie as the source of truth and decommission the SaaS subscription (giving the SaaS the required notice per their cancellation terms).

## Step 6 — Sunset evidence

```sh
oya analytics migrate sunset-evidence \
    --source mixpanel \
    --tenant drill-acme \
    --out evidence/migrations/mixpanel-to-oyatie-acme.json
```

The evidence file enumerates: export-size + record-count, conversion log, backfill receipts (per-week count comparison), shadow-dashboard drift over the 14-day window, SaaS cancellation timestamp. Required by the `oya-governance-migration-evidence` lane.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Mixpanel/Amplitude rate-limit slows export | Medium | Schedule export over weekends; budget 3× expected wall-clock. |
| Custom property names break the converter | Low | The converter dry-runs and flags non-ASCII / special-char names; rename at source or override in `event-mapping.yaml`. |
| Funnel definitions in SaaS UI don't translate 1:1 | High | Rewrite each funnel manually with the conversion table above; do not auto-generate. |
| Cohort retention math drifts from SaaS values | Medium | Use `retention()` aggregate strictly; cross-check on a 2-week window before trusting at 12-week. |
| PII fields exported without classification | Critical | Pre-conversion: scan the export for PII patterns; tag in `tags.yaml` before backfill; PII columns are mandatorily encrypted for paid tenant_class compliance-pack workloads. |
| In-flight events dropped during cutover | Medium | Run dual-write (SaaS + oyatie) for ≥ 7 d before declaring oyatie source of truth. |
| SaaS account-team-flagged migration triggers contract review | Low | Schedule the cancellation 30+ d after cutover so the SaaS contract's notice period is satisfied without back-billing. |
