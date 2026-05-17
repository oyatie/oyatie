---
doc_class: Runbook
title: Lane Bypass Emergency (Break-Glass)
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-security + council-architecture
severity_default: Sev-1 CRITICAL (intentional gate bypass; high audit exposure)
related_failure_modes: [F-11]
related_artifacts:
  - microservices/governance/threat-model.md
  - microservices/governance/incident-response.md
review_cadence: per-invocation + annually
doc_status: published
---

# Runbook: Lane Bypass Emergency (Break-Glass)

## When to invoke

You are about to admin-merge a PR that lacks the green `required_status_checks` from one or more BLOCKER lanes.

**This procedure is for emergencies only.** Examples of valid invocation:

- Production-tier rollback PR where the failing lane is itself the production-tier rollback's responsibility, and rollback delay would worsen tenant impact.
- Mass false-positive incident (F-02) requiring emergency hot-patch merge while the rule-pack fix is in flight.
- Self-application bootstrap paradox (F-13) where the only path forward requires merging a structural fix that retroactively passes lanes.

**Invalid invocations** (will be refused at post-incident review):

- Convenience bypass (i.e., "the lane is being annoying").
- PR with known security-relevant findings where you believe the findings are acceptable but lack the ADR rationale.
- Bypass of a lane unrelated to the emergency.

## Pre-flight + authorization

### Required signatures (two ops-security + one council-architecture)

| Signature | Role | Required for |
|---|---|---|
| Signature 1 | ops-security on-call | every break-glass |
| Signature 2 | ops-security secondary (independent of primary) | every break-glass |
| Signature 3 | council-architecture on-call | every Sev-1 break-glass |

### Required pre-conditions

- Active incident in `#incident-<id>` Slack channel.
- Severity is Sev-1 OR escalated Sev-2.
- IC has declared break-glass procedure necessary; ETA for non-bypass path would exceed RTO.
- Justification documented in the PR description + the `#incident-<id>` channel.

## Procedure

### Step 1 — Pre-bypass record

Create the break-glass record **before** the merge:

```bash
cat > evidence/audits/break-glass/<incident-id>.md <<'EOF'
---
doc_class: BreakGlassRecord
incident_id: <incident-id>
pr_number: <N>
sev: Sev-1
ic: <ic-handle>
sig1_ops_security_primary: <handle>
sig2_ops_security_secondary: <handle>
sig3_council_architecture: <handle>
timestamp: 2026-MM-DDTHH:MMZ
date: 2026-05-17
related_failure_mode: F-NN
classification: AUDIT
---

# Break-glass invocation — incident <incident-id>

## Justification (why this PR must merge without lane green)

<one paragraph; reference the specific lane(s) being bypassed + the emergency
context + why the non-bypass path's ETA is unacceptable>

## Bypassed lanes

| Lane | Severity | Why bypassing is acceptable |
|---|---|---|
| <lane-1> | BLOCKER | <reason> |
| ... | | |

## Retroactive remediation plan

- IP-INCIDENT-<incident-id>-<slug>.md: lane-fix PR follow-up
- Target close-date: <YYYY-MM-DD>
- Owner: <handle>

## Post-incident review committed

YES — scheduled for <YYYY-MM-DD> with ops-security + council-architecture.

EOF
git add evidence/audits/break-glass/<incident-id>.md
git commit -m "audit(break-glass): record incident <incident-id> PR #<N>"
git push
```

### Step 2 — Three-signature verification

```bash
# Each signer signs the break-glass record commit
git fetch origin
git verify-commit HEAD     # asserts the record commit is signed
```

Both ops-security signers + council-architecture review the justification in the PR description **before** Step 3.

### Step 3 — Admin-merge

Only after all signatures confirmed in `#incident-<id>`:

```bash
gh pr merge <N> --admin --merge
```

The merge will fire a GitHub audit-log event; `oya-check-protection-context-match` lane will emit an AUDIT-CRITICAL Finding on the next PR.

### Step 4 — Immediate post-merge

1. **Comment** on the merged PR with a link to `evidence/audits/break-glass/<incident-id>.md`.
2. **Tag** the merge commit with `break-glass-<incident-id>` for traceability.
3. **Notify** the µservice owner team via `#incident-<id>` + per-µservice owner channel.
4. **Open** the retroactive remediation IP: `microservices/governance/IP-INCIDENT-<incident-id>-<slug>.md` per `incident-response.md` lifecycle step 7.

### Step 5 — Post-incident review

| Step | Owner | Cadence |
|---|---|---|
| Postmortem at `evidence/audits/postmortems/<incident-id>.md` | IC + SME | within 1 week |
| Post-incident review meeting (ops-security + council-architecture; blameless) | ops-security | within 2 weeks |
| Retroactive remediation IP merged | original PR author + SME | per-IP close-date |
| Quarterly review of all break-glass records | ops-security | quarterly |
| Annual review of break-glass cadence + frequency | council-architecture | annual |

## What CANNOT be bypassed

Some lanes cannot be bypassed even by break-glass:

| Lane | Why no-bypass |
|---|---|
| `oya-check-data-class` | Cross-pack data leakage risk is irreversible |
| `oya-check-license-policy` | License-incompatible code introduces legal liability |
| `oya-check-cross-ref-validity` | Broken cross-refs cascade to every downstream consumer |
| `oya-check-naming-bnf-v41` | BNF-violating crate name is structural |
| `oya-check-protection-context-match` | This is the lane that detects break-glass itself |

If one of these lanes fails, the underlying issue MUST be fixed in the PR; admin-merge of these is refused at the `enforce_admins = true` level + at the `oya-check-protection-context-match` re-detection level + at the post-incident review level (the review will request revert).

## Penalty for unauthorized invocation

Per `compliance.md` SOC 2 CC8.1 + ISO 27001 A.5.34 + organizational policy:

- Unauthorized break-glass without two ops-security + one council-architecture signatures → revert + disciplinary review.
- Break-glass without recorded justification → revert + retroactive justification required + disciplinary review.
- Repeated break-glass for same underlying issue without remediation → escalation to council-architecture for structural fix mandate.

## Quarterly review cadence

Per `compliance.md` ISO 27001 A.5.35 independent review:

| Quarter | Reviewer | Output |
|---|---|---|
| Each quarter | ops-security secondary + council-architecture | `evidence/audits/break-glass-quarterly-review/<quarter>.md` |

Review covers:
- Frequency of invocations (target ≤2/quarter).
- Recurring underlying causes (signal for structural fix).
- Quality of justifications (every record + signature complete).
- Remediation IP close rate (target 100% within the IP close-date).

## References

- `microservices/governance/failure-modes.md` F-11.
- `microservices/governance/threat-model.md` T-E-01.
- `microservices/governance/incident-response.md` lifecycle.
- ADR-0131 §"branch-protection".
- ADR-0133 §"agentic-dev-team optimisation #5 fail-closed on every gate".
- SOC 2 CC6.3 + ISO 27001 A.5.34 separation-of-duties.
