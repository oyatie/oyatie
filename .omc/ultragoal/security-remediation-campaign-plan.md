# Security Remediation Campaign — Prioritized Plan (2026-06-23)

Source: whole-repo codex review (177 findings) + gap-fill addendum (53 findings) = **~230 findings, 54 CRITICAL, 119 HIGH**. Reports in `.omc/ultragoal/reviews/whole-repo-codex-review-2026-06-23{,-gapfill}.md`. Founder authorized (2026-06-23): run the campaign + **standing merge authority for reviewed PRs** (no per-PR approval).

## Operating model (the merge train)
Per PR: build in an **isolated worktree** (off latest dev) → **adversarial review** (me + codex 5.5-xhigh cross-model for every security surface; multi-round until severity converges to zero — the #815/#99 norm was 2-3 rounds) → merge when reviewed + CI-green (freshness/firewall/faces verified, not just buck2). Merges are **serial** (whole-tree faces conflict) → rebase+regen-faces between merges. **Fix the CLASS in a gate first**, then the instances (founder doctrine: impossible-by-design > one-off fix).

## Wave 0 — CLASS-FIX GATES (highest leverage; lead with these)
- **DTO-authz-detection gate** [IN FLIGHT, lane gate-dto-authz] — flags any surface trusting a caller-supplied `{decision_id,tenant_id,principal_id,allowed_surfaces}` blob without a server-side PDP call; born-blocking + frozen baseline of the ~36+ existing instances (shrink-only) → no NEW instance can ship + existing tracked. THE structural fix for systemic class #1.
- (later) idempotency-scope gate (keys must be tenant-scoped + body-fingerprinted); checked-money-arithmetic gate; webhook-must-HMAC gate; no-OR-auth-in-OpenAPI gate; Cedar-policy-sanity gate (no wildcard-permit, no deny-all-kills-permit, no caller-side-eval). One gate per mechanizable class.

## Wave 1 — KEYSTONE (IAM = fleet root of trust)
- C1 cedar policy-publish — **#815 ✅ MERGED**.
- C3 workload principal suspend/retire — **#816 (rebasing → merge next)**.
- C5 KMS encrypt/decrypt caller-authz — **IN FLIGHT, lane authz-c5-kms** (crown-jewel, parallel-safe).
- C2 workload `/authorize` forged-principal + C4 token keyed by WorkloadId only (cross-tenant) — **next, after #816 merges** (same crate; one PR; verified PEP identity + (TenantId,WorkloadId) keying).

## Wave 2 — caller-authz INSTANCES (uniform proven pattern; fan-out build → review → serial merge, after the gate lands so each is gate-validated)
tenancy C7 (operator scope self-attested via header) + C8 (tenant-create caller-authz); network C9/C10/C11 (LB/VPC/DNS create caller-authz); audit C15 (event auth self-attested); compliance C16 (DSR/GDPR-erasure caller-authz); observability C18 (audit-read caller-authz) + coarse-scope (audit-read ignores requested topic/tenant); **money (gap-fill):** FinOps report API cross-tenant spend exfil, billing tenantless idempotency (suppress/cross-bill), accounting+payroll empty `MiddlewareChain::new()`, CRM default-open mutation.

## Wave 3 — other CRITICAL classes
- Cedar wildcard template C12-C14 (one template → cloud-network/dns/kms/secrets/iac: add `tenant_id==resource.tenant_id` + action allowlist).
- Cedar **self-defeating** policies (net-new): ITSM caller-side-eval-mandated; ops-dashboard deny-all-kills-permits.
- C19 kernel `sys_wait4` cross-page `write_u32` memory-corruption (x86-64 + aarch64): checked multi-page user-copy → -EFAULT.
- C6 secrets sealing-root `exportable=true` single-custodian → **DESIGN pass needed** (HSM/PKCS#11 non-exportable vs Shamir M-of-N; founder architecture call) + rotate/shred existing exportable roots. NOT a blind fix.
- ClickHouse SQL/identifier injection via tenant-id interpolation (analytics mv-templates) — parameterize/validate identifiers.
- Webhook ingress HMAC+nonce+timestamp (workflow trigger-orchestrator).
- Mail SPF/DKIM/DMARC fail-open accept (mail-mailbox-rest forces dmarc_check=None→Accept) → fail-closed.
- Caller-supplied quota counters trusted for admission (compute vm / k8s ComputeQuotaEnvelope) → server-side usage lookup.
- OpenAPI OR-of-requirements making X-Scope-OrgID an auth alternative to bearer (connector) → AND, bearer-required.
- Global audit-completeness flag → per-tenant (observability aggregate).

## Wave 4 — systemic non-authz classes (productize gate + remediate)
- Idempotency fingerprinting + tenant-scoping across all money/event paths (billing/credit-notes/accounting/payroll/workflow/outbox/moderation) — shared idempotency kernel + gate.
- Checked money/integer arithmetic (invoice/journal/payroll/tax/token-expiry) — shared checked-money type + gate.
- Fail-open boundaries → fail-closed (missing durable state must refuse, not boot empty/in-memory).
- CI green-when-should-be-red holes (rename/copy laundering, FULL-tier ratchet skips tests, exempt-prefix authz bypass) + **delete `scripts/ci/oya-ci-post.sh`** context-forge (folds into #130; confirm branch-protection required-contexts = only `oya-ci-required`).
- Placeholder/no-op `run()` binaries shipping while SLOs claim availability (office/oya-flags/payments-charge/CAPI) — implement or mark non-live + fix SLO claims.
- Hot-path full-store scans / unbounded clones (kernel/COSI/cell/dcops/object/analytics/audit) — indexes/affected-set.
- SSRF prefix-only URL validation (network OCI/selfhosted + intelligence base-URL) → allowlist + block link-local/loopback/cluster-svc; intelligence CLI-subprocess→in-process HTTPS (ties to #90).

## Coverage note
Both review passes are single-lens-per-slice codex (triage/verify each finding before fixing — codex over-rated #815's in-process-forgeability). A few infra roots may still warrant a fresh-worktree re-run if findings there look thin.
