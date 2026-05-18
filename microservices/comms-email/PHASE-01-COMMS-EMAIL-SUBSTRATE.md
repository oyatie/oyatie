# PHASE-01 — Comms-Email Substrate

> Status: Draft v0.1.0
> Authored: 2026-05-18
> ADR anchors: ADR-0201, ADR-0145, ADR-0166, ADR-0173, ADR-0064.
> IPs covered: IP-001 .. IP-015.

## 1. Phase goal

Stand up the canonical transactional-email substrate so every
oyatie µservice that needs to email a human routes through the
`oya-shared-email-comms-kernel` trait. By end of phase:

- Four real adapters exist (SES, Postal, Mailgun, SMTP) with
  no Noop fallback.
- DKIM + SPF + DMARC are enforced at preflight.
- Per-tenant from-domains + locale overlays work end-to-end.
- Webhook delivery events flow into the ADR-0145 audit chain.
- Postal Helm chart deploys cleanly for the sovereign tier.
- SES configuration knobs are surfaced through the manifest
  delta for the cloud-hosted tier.
- All six existing senders are migrated.

## 2. Phase exit criteria

1. `crates/oya-shared-email-comms-kernel` ships with the trait
   surface + four real adapter shells (delivered in this batch).
2. `cargo test -p oya-shared-email-comms-kernel` is green
   (delivered in this batch).
3. IP-001 .. IP-015 are delivered and accepted in code review.
4. `microservices/comms-email/iac/helm/postal/` chart installs
   on a kind cluster and accepts a test send with DKIM signing.
5. SES adapter is wired behind `SesEmailComms` feature flag and
   sends via SESv2 with the per-tenant DKIM identity.
6. Webhook delivery events normalize into the audit chain on a
   ADR-0166 schema-versioned shape.
7. `oya-check-iac-tier-discipline` + the layered-architecture
   lane both pass for the new µservice.
8. The `developer-portal` (Backstage TechDocs) picks up this
   µservice's docs surface without manual intervention.

## 3. Phase IP map

| IP    | Title                                       | Scope      |
| ----- | ------------------------------------------- | ---------- |
| 001   | SES adapter implementation                  | adapter    |
| 002   | Postal adapter implementation               | adapter    |
| 003   | SMTP fallback adapter implementation        | adapter    |
| 004   | Mailgun adapter implementation              | adapter    |
| 005   | DKIM key rotation pipeline                  | crypto     |
| 006   | MJML template renderer                      | rendering  |
| 007   | Liquid substitution engine                  | rendering  |
| 008   | Webhook delivery pipeline                   | events     |
| 009   | Bounce / complaint handler                  | events     |
| 010   | Suppression list                            | events     |
| 011   | Per-tenant from-domain onboarding           | tenancy    |
| 012   | Audit-chain emission                        | audit      |
| 013   | Multi-region routing                        | platform   |
| 014   | Sovereign pack — Postal-only enforcement    | residency  |
| 015   | In-house relay roadmap (Phase 2 marker)     | roadmap    |

## 4. Dependencies

- ADR-0145 audit-chain (must exist; already does).
- ADR-0166 schema registry (must exist; already does).
- ADR-0173 vendor lock-in policy (must exist; already does).
- ADR-0064 localization packs (must exist; already does).
- ADR-0149 idempotency-key kernel (must exist; already does).
- `crates/oya-shared-email-comms-kernel` (delivered this batch).

## 5. Out of phase

- Inbound email ingestion.
- BIMI logos.
- Phase-2 in-house Rust-native MTA (`oya-comms-email-server`).

## 6. Risks

- DKIM key leak — mitigated by OpenBao storage + rotation IP.
- SES quota exhaustion — mitigated by Mailgun second-source +
  Postal fallback (sovereign pack).
- Bounce-storm cascade — mitigated by suppression list +
  rate-ceiling IP.
- DMARC misalignment — mitigated by enforcement at preflight.

See `failure-modes.md` for the exhaustive enumeration.

## 7. Sign-off

Phase exits on the substrate-authority reviewer-agent APPROVE
plus admission-gate green per ADR-0110 / ADR-0111.
