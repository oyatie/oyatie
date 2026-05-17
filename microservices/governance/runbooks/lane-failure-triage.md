---
doc_class: Runbook
title: Lane Failure Triage
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-foundry + ops-sre-reliability
severity_default: Sev-2
related_failure_modes: [F-01, F-02, F-09, F-13]
related_artifacts:
  - microservices/governance/failure-modes.md
  - microservices/governance/incident-response.md
review_cadence: quarterly + post-incident
doc_status: published
---

# Runbook: Lane Failure Triage

## When to invoke

A fitness lane is producing false-positive BLOCKERs, lane registry is corrupt, or governance has self-locked. Default Sev-2; escalates to Sev-1 if multi-µservice impact or if every PR is blocked.

## Pre-flight

- You are: ops-sre-reliability on-call OR axis-foundry on-call.
- You have: `gh` CLI authenticated; `cargo run -p oya-dev-cli` workspace ready; Grafana access.
- You verified the failure mode: F-01 (single-PR OOM) vs F-02 (false-positive) vs F-09 (registry) vs F-13 (self-lock).

## Decision tree

```text
                Is the failure on a single PR?
                       ├─ Yes → goto §A
                       └─ No  → goto §B

§A (single-PR; F-01-class)
   Does the PR have > 10k files in diff?
       ├─ Yes → §A1 (oversized PR)
       └─ No  → §A2 (genuine OOM; rare)

§B (multi-PR or systemic)
   Does the failure repeat across PRs to multiple µservices?
       ├─ Yes → §C (systemic; F-02 or F-09 or F-13)
       └─ No  → §D (single-µservice; F-02 false-positive scoped)

§C (systemic; promote Sev-1 if all PRs blocked)
   Is the lane registry corrupted?
       ├─ Yes → §C1 (F-09)
       └─ No  → §C2 (F-02 mass false-positive OR F-13 self-lock)
```

## §A1 — Oversized PR

1. **Confirm**: `gh pr view <N> --json files | jq '.files | length'` → expect > 10k.
2. **Comment** on PR with the standard `pr-too-large` template (linked from `policy/lane-execution.md` Invariant 3).
3. **Refuse**: lane already emitted BLOCKER; no further action.
4. **Outcome**: PR author splits diff per `docs/standards/git-workflow.md` §"branch splitting".

## §A2 — Genuine OOM (rare)

1. **Identify** the failing lane: `cargo run -p oya-dev-cli -- governance lane-status --pr <N>`.
2. **Inspect** evidence blob for the failing lane: `cargo run -p oya-dev-cli -- governance evidence get --pr <N> --lane <lane>` → look for `cgroup-oom` marker.
3. **Re-queue** once: `cargo run -p oya-dev-cli -- governance lane-rerun --pr <N> --lane <lane>` (1 retry per `failure-modes.md` F-01).
4. **If re-run fails**: file `oya-check-<lane>` issue on `microservices/governance/` repo with PR diff + OOM marker; engage axis-foundry SME.
5. **Outcome**: PR may proceed if lane returns PASS on retry; otherwise file follow-up IP.

## §B — Single-µservice scoped false-positive

(Sev-2; F-02)

1. **Confirm scope**: query lane fail rate for the µservice: `cargo run -p oya-dev-cli -- governance finding-rate --microservice <ms> --lane <lane> --window 1h`.
2. **Reproduce** locally: `cargo run -p oya-dev-cli -- gate validate <lane> --microservice <ms>` against the failing SHA.
3. **Diagnose**: check rule-pack at `microservices/governance/src/crates/oya-check-<topic>/rules/`.
4. **Mitigate (hot-patch)**:
   - If rule-pack has a clear bug: open PR to downgrade BLOCKER → WARN with ADR-NNNN rationale per `failure-modes.md` F-02.
   - Two CODEOWNERS (axis-foundry + ops-security) must approve.
   - Tag PR with `incident-hot-patch` label.
5. **Notify** µservice owner team via `#incident-<id>` channel.
6. **Follow-up**: open RCA postmortem; structural fix in subsequent IP.

## §C1 — Lane registry corruption (F-09)

(Sev-2; can escalate to Sev-1 if no lanes dispatch)

1. **Confirm**: `cargo run -p oya-dev-cli -- governance lane-registry status` → expect inconsistency report.
2. **Capture** state: dump `lanes` table for forensics: `kubectl exec -n governance postgres-primary -- pg_dump -t lanes > /tmp/lanes-incident-<id>.sql`.
3. **Truncate** + re-register:
   ```bash
   kubectl exec -n governance postgres-primary -- psql -c "TRUNCATE lanes CASCADE;"
   cargo run -p oya-dev-cli -- governance lane-registry rebuild
   ```
4. **Verify**: `cargo run -p oya-dev-cli -- governance lane-registry status` → expect green.
5. **Smoke test**: open a no-op PR; full ~50-lane suite runs successfully.
6. **Post-incident**: RCA on what caused corruption; consider PITR if integrity uncertain.

## §C2 — Mass false-positive OR self-lock (F-02 or F-13)

(Sev-1 if all merges blocked; Sev-2 otherwise)

### F-02 mass false-positive

1. **Identify** the failing rule via Grafana `top finding rules by rate` panel.
2. **Confirm** false-positive: reproduce against multiple known-good SHAs.
3. **Hot-patch** rule-pack to downgrade severity (BLOCKER → WARN) via emergency PR.
4. **Apply break-glass override** if mass-merge needed:
   - Two ops-security signatures.
   - Documented in `runbooks/lane-bypass-emergency.md`.
   - Per-bypass record in `evidence/audits/break-glass/<incident-id>.md`.
5. **Permanent fix**: follow-up IP with rule-pack correction + retroactive scan + restoration of BLOCKER severity.

### F-13 self-lock (governance fails its own conformance lane)

1. **Confirm**: every PR to `microservices/governance/` shows the same lane fail.
2. **Use synthetic-probe fallback** per PRD Open Q3:
   - `cargo run -p oya-dev-cli -- governance self-application-mode --enable-amnesty --duration 24h`
   - This grants 24h amnesty to governance's own PRs while structural fix lands.
3. **Land structural fix** in same 24h window (priority IP).
4. **Verify** self-application passes on next PR.
5. **Disable amnesty**: `cargo run -p oya-dev-cli -- governance self-application-mode --disable-amnesty`.
6. **Post-incident**: RCA + ADR follow-up if amnesty was needed.

## §D — Other scoped failure

1. **Open** GitHub Issue on `microservices/governance/` repo.
2. **Triage** with axis-foundry team within 1 business day.
3. **Standard PR + review** cycle.

## Stand-down criteria

- Lane fail rate returns to baseline (< 1% of PRs).
- Smoke-test PR passes full suite.
- No further alerts in 1h.
- Postmortem assigned per `incident-response.md` lifecycle step 5.

## Post-incident actions

- File postmortem at `evidence/audits/postmortems/<incident-id>.md` within 1 week.
- Update this runbook if new failure pattern observed.
- File follow-up IP for any structural change.

## References

- `microservices/governance/failure-modes.md` (F-01, F-02, F-09, F-13).
- `microservices/governance/incident-response.md` (lifecycle).
- `microservices/governance/policy/lane-execution.md` (invariants).
- `microservices/governance/runbooks/lane-bypass-emergency.md` (break-glass).
- Google SRE Workbook ch. 7.
