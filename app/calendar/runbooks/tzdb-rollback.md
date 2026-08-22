---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: calendar
runbook_id: RB-CAL-TZDB-ROLLBACK
severity_class: sev-1
related_adrs: [ADR-CAL-0004]
related_slos: [tzdb-staleness-bound]
owner_team: axis-calendar + ops-sre-reliability
date: 2026-05-17
doc_status: published
---

# Runbook: IANA tzdb release rollback

## Symptom

A newly-promoted IANA tzdb release (typically auto-promoted by the
`calendar-tzdb-refresh-worker` per ADR-CAL-0004) introduces a
regression. Visible as one or more of:

- RFC 5545 RRULE corpus regression in CI post-promotion (named
  edge-case test from ADR-CAL-0002 fails).
- Tenant reports incorrect DST renderings on appointments (e.g.,
  Lebanon 2023 tzdb 2023a/2023b precedent).
- Cross-tenant divergence rate
  `calendar_tzdb_cross_tenant_divergence_total` spikes >10× the
  baseline.
- Agenda-render p95 regresses (rare, but a tzdb bug can break
  in-flight calculations).

## Severity

**Sev-1** by default — tzdb errors affect appointment correctness
across the entire µservice.

## First responder

axis-calendar on-call + ops-sre-reliability on-call.

## Diagnosis

### Step 1 — Confirm the regression is tzdb-related

```bash
# Which tzdb release is currently in cluster-default?
kubectl -n calendar exec deploy/calendar-event-store-app -- \
  dev-cli calendar tzdb show-current

# Which release was previously in cluster-default?
git log --oneline --grep='chrono-tz' -- microservices/calendar/src/crates/calendar-event-store-adapter/Cargo.toml | head -5
```

### Step 2 — Quick reproducer

```bash
# Run the RFC 5545 corpus + DST edge-case matrix against the
# current tzdb pin
cargo nextest run -p calendar-recurrence-engine-domain \
  -- rfc_5545_libical_corpus rrule_edge_cases 2>&1 | tail -30
```

Identify which case fails. The case description gives the affected
tz + rule change (e.g., "Asia/Beirut: spring-forward 2023-03-26 03:00
LST").

### Step 3 — Confirm the prior tzdb release is clean

```bash
# Pull the prior chrono-tz release from the bumping ChangeSet's parent commit
git stash
git checkout <prior-LTS-commit>
cargo nextest run -p calendar-recurrence-engine-domain \
  -- rfc_5545_libical_corpus rrule_edge_cases 2>&1 | tail -10
git checkout -
git stash pop
```

If the prior release is clean, proceed to rollback. If both are
broken, the regression is NOT tzdb — escalate to council-architecture.

## Mitigation

### Step 1 — Open the rollback ChangeSet

```bash
# Create a rollback branch + ChangeSet
git checkout -b cal/tzdb-rollback-$(date +%Y%m%d)

# Pin chrono-tz back to the prior LTS version
# (replace 0.10.X with the actual prior pin)
sed -i.bak 's|chrono-tz = "0\.10\..*"|chrono-tz = "0.10.X"|' \
  microservices/calendar/src/crates/calendar-event-store-adapter/Cargo.toml

cargo build --workspace
cargo nextest run -p calendar-recurrence-engine-domain
```

### Step 2 — Validate corpus + edge-case matrix passes

```bash
# Full corpus + DST matrix MUST be green before rollback ships
cargo nextest run -p calendar-recurrence-engine-domain \
  -- rfc_5545_libical_corpus rrule_edge_cases rfc_5545_python_dateutil_corpus 2>&1 | tail -5
```

### Step 3 — Open the rollback PR

```bash
gh pr create \
  --base dev \
  --title "ops: rollback chrono-tz to 0.10.X (tzdb regression: <release name>)" \
  --body "$(cat <<'EOF'
## Summary
- IANA tzdb release <release name> introduced a regression in RFC 5545 corpus / DST edge-case matrix.
- Per ADR-CAL-0004, rolling back chrono-tz to prior LTS.

## Failing case
<case description from Step 2>

## Verification
- cargo nextest run -p calendar-recurrence-engine-domain — green
- `oya gate validate tzdb-staleness-bound --microservice calendar` — REPORT-ONLY (rollback exceeds 30d if upstream lingers; track via fix-up)

🤖 Generated with [Claude Code](https://claude.com/claude-code)
EOF
)"
```

Mark the PR with the `ops-rollback` label for expedited admission.

### Step 4 — Suppress the auto-promoter for the bad release

```bash
# Pin the refresh worker to skip the offending release
kubectl -n calendar patch configmap calendar-tzdb-refresh-config --type merge -p \
  '{"data":{"blocklist":"<release name>"}}'
```

### Step 5 — Tenant comms

Per-pack tenant notification via the standard ops-comms channel:
- pack-kr: no impact if the rolled-back release was DST-affecting; KR
  has no DST.
- pack-eu / pack-us: DST-relevant; notify tenant operators that
  appointments in <affected tz> may have rendered with incorrect
  local times during the offending window; offer to re-render.

## Verification

```bash
# After PR merge to dev and admission to staging:
# 1. Cluster-default tzdb release rolled back
kubectl -n calendar exec deploy/calendar-event-store-app -- \
  dev-cli calendar tzdb show-current
# expect: prior LTS release

# 2. Corpus + edge-case matrix green
cargo run -p dev-cli -- gate validate rfc-5545-conformance --microservice calendar

# 3. Cross-tenant divergence rate returning to baseline
kubectl -n calendar exec deploy/calendar-event-store-app -- \
  curl -s localhost:9090/metrics |
  grep 'calendar_tzdb_cross_tenant_divergence_total'

# 4. Staleness SLO (REPORT-ONLY tracking the rollback)
cargo run -p dev-cli -- gate validate slo --microservice calendar --slo tzdb-staleness-bound
```

## Post-incident

- File the regression upstream (IANA tz mailing list +
  `chrono-tz` GitHub issue).
- Update the `rrule_edge_cases` test set with the offending case
  if it was not already covered.
- Review the auto-promoter's gating logic — did the bumping
  ChangeSet run the full corpus before opening for review? If not,
  tighten the worker.
- If staleness SLO breaches 30d while waiting for upstream fix,
  consider vendoring the prior tzdb directly (one-time exception
  to ADR-CAL-0004 §"buy not build").

## References

- ADR-CAL-0004 — IANA tzdb refresh + per-tenant pinning policy.
- ADR-CAL-0002 — RRULE engine RFC 5545 conformance + named DST edge cases.
- IANA tz mailing list — `mm.icann.org/pipermail/tz/`.
- `chrono-tz` — `crates.io/crates/chrono-tz`.
- Lebanon 2023 DST precedent — tzdb 2023a/2023b release notes.
- `microservices/calendar/runbooks/timezone-db-refresh.md` — refresh path (this is the rollback path).
- `microservices/calendar/slos/tzdb-staleness-bound.openslo.yaml`.
