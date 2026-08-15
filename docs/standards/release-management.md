---
purpose: Cross-cutting release-management standard. Codifies trunk-based development with short-lived branches, feature-flag + canary progressive delivery, the SLO-burn-rate auto-rollback rail, and the Sigstore-signed-release pipeline.
doc_status: published
---

---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Cross-cutting release-management standard. Codifies trunk-based development
  with short-lived branches, feature-flag + canary progressive delivery, the
  SLO-burn-rate auto-rollback rail, and the Sigstore-signed-release pipeline.
  Operates downstream of `docs/RELEASE-MANAGEMENT.md` (program-level mechanics)
  and supplies the per-PR / per-release authoring rules.
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
planned_enforcement_ref: oya-governance-flag-debt
companion_docs:
  - docs/RELEASE-MANAGEMENT.md
  - docs/SLO-CATALOG.md
  - docs/standards/on-call.md
  - docs/standards/observability.md
  - docs/standards/image-discipline.md
  - docs/standards/security-review.md
related_adrs:
  - ADR-0053
  - ADR-0052
  - ADR-0054
---

# Release Management

## Doctrinal authority — [decision-principles.json](../../specs/decision-principles.json) + [forbidden-operations.json](../../specs/forbidden-operations.json)

The program-level lifecycle (CI gates, lanes, rollout strategies) lives in
[`docs/RELEASE-MANAGEMENT.md`](../RELEASE-MANAGEMENT.md). This standard
adds the per-PR / per-release authoring discipline: how branches are
shaped, how flags retire, how canaries roll forward, how burn-rate
breaches roll back, and how artifacts are signed.

## 1. Trunk-based development

Per [Trunk Based Development](https://trunkbaseddevelopment.com/continuous-review/)
and Google / Microsoft consensus:

- Protected integration branch is `dev` per the current operating contract. All work happens on **short-lived branches**
  (target ≤ 24 h, MUST NOT exceed 7 days without a re-base or an ADR
  exemption).
- Lane `oya-governance-branch-age` warns ≥ 5 d, blocks ≥ 7 d.
- Feature flags hide incomplete work behind a runtime gate (§3) so
  partial merges to `dev` do not ship to users.
- Branch protection: required reviews per `RACI-OWNERSHIP.md` and green CI. Reviewer evidence and
  the required CI context remain distinct; `F-PR5-06` tracks the automated review-admission gap.
- Force-push to protected integration/release branches is forbidden per
  [`forbidden-operations.json`](../../specs/forbidden-operations.json) FO-03.

Sources: [DORA — Trunk-Based Development](https://dora.dev/capabilities/trunk-based-development/),
[Aviator — What is Trunk-Based Development](https://www.aviator.co/blog/trunk-based-development/).

## 2. Small CLs / small PRs

Per Google's [Standard of Code Review](https://google.github.io/eng-practices/review/reviewer/standard.html):

- One logical change per PR; related tests included; reviewer can hold it
  in their head.
- Target median review latency: **< 24 hours**. Surfaced via
  `oya-governance-review-latency` (advisory).
- Refactors and renames are separate PRs from behavior changes.
- "Code health > correctness alone" — a PR that lands correct behavior but
  worsens the codebase health is REQUEST CHANGES.

## 3. Feature flags + canary

### 3.1 Feature-flag substrate

- Per ADR-REL-001 (pending), the workspace adopts a feature-flag library
  (default: in-tree `oya-platform-flags-kernel`; fallback: Unleash OSS).
- Every behavior-changing PR ships **behind a flag** with:
  - **Owner**: a team-charter ID.
  - **Created**: ISO date.
  - **Retire-by**: ISO date ≤ 30 days from creation (the **flag-debt SLO**).
  - **Rollout strategy**: percentage / tenant cohort / capability binding.
- Once implemented, the advisory lane `oya-governance-flag-debt` opens a blocking PR check at
  retire-by + 1d.
- Stale flags are an anti-pattern: lifetime > 30 d without renewal
  triggers `EVT-FLAG-OVERDUE` and a team-lead escalation.

### 3.2 Canary rollout

Default rollout shape for an `oya-*-runtime-*` deploy:

1. **Stage 0** (1% traffic, stable cohort): hold ≥ 30 min; require
   burn rate ≤ 1× across all golden signals.
2. **Stage 1** (10% traffic): hold ≥ 2 h.
3. **Stage 2** (50% traffic): hold ≥ 6 h.
4. **Stage 3** (100% traffic).

Rails:

- **Argo Rollouts** or **Flagger** for k8s-native progressive delivery
  (per `.omc/scratch/hyperscaler-best-practices-2026-05-12.md` Domain 2).
- The canary controller subscribes to the metric backend (per
  [`observability.md`](observability.md)) and computes burn rate per
  stage.

### 3.3 Burn-rate-driven auto-rollback

Per [`on-call.md`](on-call.md) §2:

- Burn rate ≥ 14.4× over 1h → **immediate rollback** (canary controller
  reverts to N-1).
- Burn rate ≥ 6× over 6h → **pause + page** (controller halts promotion,
  pages on-call).
- Per ADR-EE-002, the error budget defines the release gate: budget
  exhausted ⇒ feature work freezes; only fixes that reduce error rate
  ship until the budget recovers.

### 3.4 Blue-green vs canary

- **Canary + flags** is the default for stateless services.
- **Blue-green** is reserved for stateful cutovers (schema migrations,
  data backfills, region failover); requires a runbook entry per
  [`on-call.md`](on-call.md) §3 and a dry-run on staging.

### 3.5 Post-merge product-completion gate

A squash merge proves merge admission, not product completion. Product-complete
requires a post-merge packet with promoted SHA + `oya-ci-required` status URL,
rollout verification, rollback note, observability/golden-signal check,
browser UX/user-story evidence, and release-governance/release-note impact
(Release Please only when repo config proves it).
Docs-only or no-deploy changes record explicit `no deployable artifact` / `not
user-visible` rationales; blank evidence means incomplete.

Sources: [Flagsmith — Progressive Delivery](https://www.flagsmith.com/blog/progressive-delivery),
[Unleash — Canary vs Progressive Delivery](https://www.getunleash.io/blog/canary-release-vs-progressive-delivery),
[Visualpath — Progressive Delivery SRE 2025](https://visualpathblogs.com/site-reliability-engineering/what-is-the-best-way-to-implement-progressive-delivery-sre-in-2025/).

## 4. Schema migrations

Every schema migration ships:

1. **Up**: forward DDL.
2. **Down**: revert DDL.
3. **Dry-run**: against a staging clone.
4. **Per-tenant**: tenant-scoped batching.
5. **Per-cell rollback**: cell-by-cell revert procedure.

Lane: `oya-governance-schema-migration` (per AGENTS.md D14).

## 5. Sigstore-signed releases

Per [`security-review.md`](security-review.md) §4 and
[`image-discipline.md`](image-discipline.md):

| Step | Tool | Output |
|---|---|---|
| 1. SBOM | Syft (CycloneDX) | `sbom.cdx.json` |
| 2. Sign | Cosign keyless OIDC + Fulcio | `cosign.sig` |
| 3. Provenance | SLSA L2 in-toto attestation | `provenance.json` |
| 4. Log | Rekor transparency log | Rekor entry UUID |
| 5. Pin | Image digest in deploy manifest | `sha256:...` |
| 6. Admit | policy-controller / Kyverno | cluster admission |

- Cosign pin: ≥ **v3.0.6** (per LTS spec).
- Image pin: **never `latest`**; always `sha256:` digest.
- Trivy scan: ≥ **v0.70.0**; HIGH/CRITICAL CVEs block release.

Lane: `oya-governance-release-supply-chain` per DOC-CATALOG.md §4.

## 6. Error-budget release gate

| Burn rate | Window | Release-gate action |
|---|---|---|
| ≥ 14.4× | 1h | **block all new releases**; rollback in flight |
| ≥ 6× | 6h | **block feature releases**; permit fix-only |
| 1× | 3d | normal release cadence |
| ≤ 0.1× | 30d | release backlog accelerated; canary stages shortened by 25% |

The gate is computed at release-pipeline start; the CI lane refuses
promotion to higher canary stages if burn-rate state forbids.

Source: [Google SRE Workbook — Error Budget Policy](https://sre.google/workbook/error-budget-policy/).

## 7. Deploy event audit-chain

Every deploy emits per [`observability.md`](observability.md) §4:

- `EVT-RELEASE-DEPLOYED` with `service`, `version`, `image_digest`,
  `cosign_signature`, `rekor_uuid`, `canary_stage`.
- `EVT-RELEASE-ROLLED-BACK` with `service`, `from_version`, `to_version`,
  `reason`, `burn_rate_metric`.

## 8. Rollback procedure

1. Canary controller flips the stable selector to N-1 (image digest).
2. On-call validates SLO recovery within 15 min.
3. `EVT-RELEASE-ROLLED-BACK` emitted.
4. Incident commander declares Sev-2 (if customer-impacting) → triggers
   blameless postmortem per [`on-call.md`](on-call.md) §5.
5. The mechanical prevention shipped from the postmortem closes the loop.

## 9. CI lane summary

Per the hyperscaler-quality CI gate set (per
`.omc/scratch/hyperscaler-best-practices-2026-05-12.md` Domain 4):

1. `cargo fmt --check`.
2. `cargo clippy --workspace --all-targets -- -D warnings`.
3. `cargo test --workspace`.
4. `cargo deny check` + `cargo audit` + `cargo vet`.
5. `cargo llvm-cov` (delta-coverage report).
6. Syft SBOM generation.
7. Cosign keyless signing + Rekor log entry.
8. SLSA L2 provenance attestation.
9. `gitleaks` + `trufflehog` secret scan.
10. License-policy gate (`cargo-deny` + `cargo-vet`).
11. Doc / ADR / runbook lanes (per `DOC-CATALOG.md` §4).
12. Reviewer-agent verdict captured in PR body.

## 10. Anti-patterns

1. **Atomic deploy without canary.** Forbidden for any service serving
   tenant traffic.
2. **Flag without retire-by.** Refused by `flag-debt` lane.
3. **`latest` tag in a manifest.** Refused by `image-discipline` lane.
4. **Skipping SLO burn-rate check.** Canary controller refuses to
   promote.
5. **Force-push to a protected integration/release branch.** Forbidden.

## 11. Sources scanned

- [`docs/RELEASE-MANAGEMENT.md`](../RELEASE-MANAGEMENT.md).
- [Trunk Based Development](https://trunkbaseddevelopment.com/continuous-review/).
- [DORA — Trunk-Based Development](https://dora.dev/capabilities/trunk-based-development/).
- [Google SRE Workbook — Error Budget Policy](https://sre.google/workbook/error-budget-policy/).
- [Argo Rollouts](https://argoproj.github.io/argo-rollouts/), [Flagger](https://flagger.app/).
- [SLSA Provenance v0.1](https://slsa.dev/spec/v0.1/provenance).
- [Chainguard — Sign SBOM with Cosign](https://edu.chainguard.dev/open-source/sigstore/cosign/how-to-sign-an-sbom-with-cosign/).
- [`.omc/scratch/hyperscaler-best-practices-2026-05-12.md`](../../.omc/scratch/hyperscaler-best-practices-2026-05-12.md)
  Domain 2 "Feature flags + progressive delivery" + Domain 4 CI gates.
