---
doc_class: Runbook
title: Industry-Baseline Refresh (Quarterly)
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: council-architecture + axis-foundry
severity_default: Sev-4 (informational; Sev-3 if soft baseline auto-promoted)
related_failure_modes: [F-08, F-12]
related_artifacts:
  - microservices/governance/failure-modes.md
  - microservices/governance/compliance.md
review_cadence: quarterly
doc_status: published
---

# Runbook: Industry-Baseline Refresh (Quarterly)

## When to invoke

- Quarterly cron fires (`axis-foundry-bot` per `oya-governance-baseline-refresh` workflow).
- Manual review needed (e.g., new ADR signals baseline drift).
- F-08 (refresh fetch fails repeatedly) or F-12 (refresh proposes softer baseline).

## Pre-flight

- You are: council-architecture on-call (primary) OR axis-foundry on-call (executor).
- You have: workspace access; `gh` authenticated; OpenBao token for baseline-diff client.

## Industry baselines tracked

Per ADR-0133 §"6 Axes" + `/specs/industry-best-practice-conformance.json`:

| Axis | Source | URL | Pin frequency |
|---|---|---|---|
| Pipeline | SLSA v1.0 | `slsa.dev/spec/v1.0/levels` | quarterly |
| Pipeline | NIST SSDF SP 800-218 | `csrc.nist.gov` | annual (NIST cadence) |
| Pipeline | OpenSSF Best Practices | `openssf.org/badges/` | quarterly |
| Pipeline | GitHub Actions Hardening | `docs.github.com/en/actions/security-guides/` | quarterly |
| Directory | AWS service-team template | (internal AWS reference; oyatie tracks public Smithy `smithy-lang/smithy`) | quarterly |
| Directory | Google `google3` patterns | (proxy: published "Software Engineering at Google" 2020; oyatie tracks delta to spec) | quarterly |
| Directory | Microsoft Eng. Playbook | `microsoft/code-with-engineering-playbook` | quarterly |
| Naming | BNF v4.1 | ADR-0056 (internal) | n/a |
| Naming | Rust API guidelines | `rust-lang.github.io/api-guidelines/` | annual |
| Naming | conventional-commits | `conventionalcommits.org` | annual |
| Standards | Diátaxis | `diataxis.fr` | annual |
| Standards | OpenSLO | `openslo.com` | quarterly |
| Standards | OpenTelemetry semconv LTS | `opentelemetry.io/docs/specs/semconv/` | quarterly |
| Standards | Google AIP | `google.aip.dev` | quarterly |
| Standards | Stripe API design | `stripe.com/docs/api` (continuous) | annual sample |
| Practices | agentic-dev-team-optimization | `docs/standards/agentic-dev-team-optimization.md` (internal) | per major drift |
| Policies | OWASP ASVS | `owasp.org/www-project-application-security-verification-standard/` | quarterly |
| Policies | CIS Benchmarks (Kubernetes) | `cisecurity.org` | quarterly |
| Policies | AWS Well-Architected | `aws.amazon.com/architecture/well-architected/` | quarterly |
| Policies | Azure Well-Architected | `learn.microsoft.com/azure/well-architected/` | quarterly |
| Policies | Google SRE Workbook | `sre.google/workbook/` | annual |

## Procedure (quarterly automatic flow)

### Step 1 — Cron trigger

`axis-foundry-bot` runs `.github/workflows/governance-baseline-refresh.yml` on the first Monday of each quarter (2026-04-06, 2026-07-06, 2026-10-05, 2027-01-05, ...).

### Step 2 — Fetch + diff

Workflow invokes:

```bash
cargo run -p oya-dev-cli -- governance baseline-refresh --quarter 2026-Q2
```

Behaviour:
- For each baseline in `/specs/industry-best-practice-conformance.json`:
  - HTTPS fetch from `source_url` (with retry per F-08 mitigation).
  - Compute canonical-form hash of fetched content.
  - Diff against `pinned_sha`.
  - If diff: stage update in PR branch.
- If any fetch fails: emit `baseline-refresh-fetch-failure` Finding (severity = OPERATIONAL); retry with backoff (1h, 6h, 24h); after 72h, escalate to council-architecture review.

### Step 3 — Auto-PR

Workflow opens PR with title `Quarterly industry-baseline refresh — 2026-Q2`:

- Updates `/specs/industry-best-practice-conformance.json` with new pins.
- Includes per-axis diff summary.
- Labels: `quarterly-refresh`, `axis-foundry-bot`, plus `softer-baseline-proposed` if any pin is moving in the relaxation direction.
- Body includes:
  - Per-axis pin delta.
  - Source-URL changelog excerpts.
  - Recommended response per axis.

### Step 4 — Council-architecture review (REQUIRED — cannot self-merge)

PR is opened by `axis-foundry-bot` but **cannot be merged by the bot**. Council-architecture reviewer must:

1. **Validate** each pin delta against the source-URL changelog.
2. **Apply** ADR-0133 §"Operational" baseline-softening guard:
   - If any pin SOFTER than previous → require explicit ADR follow-up rationale before merge.
   - If pin STRICTER → standard review.
3. **Identify** affected µservices: a stricter pin may surface new findings on existing µservices.
4. **File** per-axis remediation IPs at `microservices/governance/IP-M01-AUDIT-<axis>-<NNN>.md` for each new finding the stricter pin would emit.
5. **Approve + merge** when:
   - Pin deltas validated.
   - ADR rationale filed (if softer).
   - Remediation IPs filed (if stricter).
6. **Merge** triggers `oya-check-industry-best-practice-conformance` lane re-run on every PR going forward against the new pins.

### Step 5 — Per-axis remediation IP execution

Each remediation IP follows the standard IP lifecycle per `microservices/governance/PHASE-01-CI-FITNESS-CONSOLIDATION.md`:

1. **Author**: axis-foundry SME or affected µservice owner.
2. **Implement**: code/doc change that closes the new finding the stricter pin emits.
3. **PR**: full ~50-lane suite + the new tighter pin must pass.
4. **Merge**: standard process.

### Step 6 — Quarterly refresh report

Workflow writes `evidence/audits/industry-best-practice-conformance/<quarter>.json` at the merge of the refresh PR. Contents:

```json
{
  "quarter": "2026-Q2",
  "refreshed_at": "2026-04-06T00:00:00Z",
  "axes_refreshed": [...],
  "softer_baselines": [...],
  "stricter_baselines": [...],
  "remediation_ips_filed": [...],
  "next_quarter_due": "2026-07-06"
}
```

## §F-08 — Refresh fetch fails (operational)

(Sev-4)

1. **Confirm**: `cargo run -p oya-dev-cli -- governance baseline-refresh status` → expect `fetch-failure` markers.
2. **Identify** which source(s) failed.
3. **Retry** manually:
   ```bash
   cargo run -p oya-dev-cli -- governance baseline-refresh --axis <axis> --retry
   ```
4. **If repeated failures > 72h**:
   - Manual fetch with browser; commit + sign as `axis-foundry-bot` proxy.
   - Open ADR-NNNN if vendor source moved permanently (e.g., URL change).
5. **Update** runbook with new URL if applicable.

## §F-12 — Softer baseline proposed (review)

(Sev-3)

1. **Read** the PR's `softer-baseline-proposed` annotation + per-axis diff.
2. **Validate** the softening was actually published (not a hash mistake).
3. **Decide**:
   - Accept: rare; requires ADR rationale ("the industry standard relaxed; we follow"); council-architecture explicit approval.
   - Reject: more common; close PR with rationale; restore previous pin; track upstream changelog for justification.
4. **If accepted**: open ADR-NNNN documenting the softening + the rationale.
5. **Run** `oya-check-claim-ceiling` against the new posture: ensure no marketing claim (per ADR-0123) implicitly relied on the softened pin.

## Stand-down criteria

- Refresh PR merged or rejected with rationale.
- Per-axis remediation IPs filed (if stricter).
- `evidence/audits/industry-best-practice-conformance/<quarter>.json` written.
- Next quarter's cron fires correctly.

## Post-action

- Update this runbook annually with any new baseline sources.
- Quarterly review meeting: council-architecture + ops-security review the refresh + remediation IP progress.

## References

- ADR-0133 §"Operational" baseline-softening guard.
- ADR-0123 (claim-ceiling).
- `/specs/industry-best-practice-conformance.json`.
- `microservices/governance/failure-modes.md` F-08, F-12.
- `microservices/governance/compliance.md` (frameworks tracked).
