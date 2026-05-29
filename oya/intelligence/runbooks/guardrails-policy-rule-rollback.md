---
doc_class: Runbook
title: Policy / rule rollback — content-safety + Cedar overlay
microservice: foundry-guardrails
severity: "Sev-2 (rule regression / FP surge) / Sev-1 (cluster-wide policy breach)"
status: Accepted
owner_team: axis-foundry-guardrails
date: 2026-05-17
related_artifacts:
  - microservices/intelligence/failure-modes.md (FM-07, FM-05)
  - microservices/intelligence/policy/guardrail-enforcement.md
doc_status: published
---

# Runbook: Policy / rule rollback

## Trigger

ONE of:

1. **FP surge** (FM-07): newly enforced rule produces high false-positive rate; tenants seeing surge.
2. **Cedar bundle default-deny drift** (FM-05): bundle weakened; security risk.
3. **Cedar fragment regression**: new fragment evaluates unexpectedly (deny-overrides logic surprise).
4. **Post-mortem remediation**: prior incident requires rule retraction.

## Severity

- FP surge: Sev-2 (single rule).
- Cedar default-deny drift: Sev-1 (security).
- Cedar fragment broad-effect regression: Sev-2 (multi-tenant).

## Pre-checks

1. Identify offending rule_id (or Cedar fragment id + bundle SHA).
2. Confirm prior-version SHA in git history + Postgres mutation log.
3. Quantify impact: how many tenants affected?

## Steps — rule rollback (FM-07)

| Step | Action | Time |
|---|---|---|
| 1 | Open `#inc-<id>` | ≤ 5 min |
| 2 | Invoke rollback: `oya foundry-guardrails rule-rollback --rule-id <id> --to-version <prior> --reason <rfc>` | ≤ 2 min |
| 3 | CLI: <br> a. fetches prior version from Postgres mutation log; <br> b. sets `status=sunset` on current; <br> c. inserts new row with prior version's threshold + same rule_id; status=enforce; <br> d. audit-chain seal | ≤ 1 min |
| 4 | Pod cache hot-reload via Postgres NOTIFY within 5s | ≤ 10s |
| 5 | Verify FP rate decrease: `foundry_guardrails_fp_rate{rule_id="<id>"} < 0.05` | ≤ 30 min |
| 6 | Tenant comms: rule-author dashboard surfaces; tenant operators auto-notified | ≤ 30 min |
| 7 | Engage rule-author to retune for re-promotion via shadow→enforce | days |

## Steps — Cedar bundle rollback (FM-05)

| Step | Action | Time |
|---|---|---|
| 1 | Declare Sev-1; engage ops-security | ≤ 5 min |
| 2 | ArgoCD auto-rollback to last green Helm state if pre-deploy detection; if live-cluster mutation, manual rollback | ≤ 5 min |
| 3 | Invoke: `oya foundry-guardrails cedar-bundle-rollback --pack <p> --to-sha <prior-sha> --reason <rfc>` | ≤ 2 min |
| 4 | CLI: <br> a. fetches prior bundle SHA from git; <br> b. recompiles bundle (`iac/cedar/build.sh`); <br> c. updates ConfigMap; <br> d. cedar-engine sidecars hot-reload | ≤ 30s |
| 5 | Verify bundle SHA in deployed state matches expected: `oya foundry-guardrails cedar-bundle-show --pack <p>` | ≤ 5 min |
| 6 | Verify default-deny enforced: `oya gate validate cedar-default-deny-enforced` exit 0 | ≤ 5 min |
| 7 | Audit-chain emit `CedarBundleRolledBack` | automatic |
| 8 | If exposure during drift window confirmed (`oya_tenant_unauthorized_read_attempt_total > 0`): begin per-pack breach-notification chain | per regulatory |

## Steps — Cedar fragment rollback

| Step | Action | Time |
|---|---|---|
| 1 | Identify fragment + bundle SHA via Postgres fragment registry | ≤ 5 min |
| 2 | git revert + PR + CODEOWNERS approval | ≤ 30 min (compressed for Sev-2) |
| 3 | Merge → bundle rebuild → ConfigMap update → hot-reload | ≤ 30s post-merge |
| 4 | Verify | ≤ 5 min |

## Rollback (of the rollback)

If the prior version is also bad:
1. Walk back via mutation log to next-prior known-good version.
2. Escalate to ExecSponsor if persistent regression.

## Verification

- Postgres mutation log shows rollback row.
- Audit-chain seal recorded.
- Affected metric SLIs recover.
- LEAN lane confirms default-deny enforced (if Cedar rollback).
- Rule-author dashboard reflects.

## Post-incident updates

- Postmortem.
- If FM-07: rule-author retraining of the shadow→enforce process; threshold defaults reviewed.
- If FM-05: ops-security review of how live-mutation possible despite CI gates; harden control-plane access.

## References

- `microservices/intelligence/failure-modes.md` FM-05 + FM-07.
- `microservices/intelligence/policy/guardrail-enforcement.md`.
- `microservices/intelligence/iac/cedar/build.sh`.
- `microservices/intelligence/iac/postgres/migrations/001-init-schema.sql`.
