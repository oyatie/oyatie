---
id: ADR-MAIL-0002
status: Accepted
date: 2026-05-17
microservice: mail
deciders: axis-mail, ops-deliverability, ops-sre-reliability, council-architecture
owner: axis-mail
supersedes: []
superseded_by: []
related:
  - ADR-0131
  - ADR-0132
  - ADR-0133
  - ADR-MAIL-0001
related_artifacts:
  - microservices/mail/PRD.md (Open Question 4 — gap-fill; mail-server backend selection)
  - microservices/mail/PHASE-01-MAIL-DISSOLUTION-FROM-CONNECT.md (IP-001 IaC, IP-006/IP-007 SMTP frontends)
  - microservices/mail/competitor-parity-matrix.md
  - microservices/mail/capacity-model.md
purpose: Close PRD-mail Open Question 4 (gap-fill) — which SMTP+IMAP/JMAP backend ships per tenant-tier, and how the µservice keeps its BCs backend-neutral.
---

# ADR-MAIL-0002: Mail-server backend per tenant tier — Postfix+Dovecot for enterprise; Stalwart for starter/pro; both behind oya-mail-* port traits

## Status

Accepted — 2026-05-17.

## Context

The `mail` µservice owns SMTP receive (FR-01), SMTP submission (FR-02), IMAP/JMAP/REST (FR-03), and the supporting deliverability, retention, legal-hold, and search surfaces. Per ADR-0131 + ADR-0133 cross-tenant mail-server pattern, every tenant operates a logical mail-server inside a shared software substrate. The substrate's actual SMTP daemon + IMAP server choice is a deliberate trade-off.

Two viable open-source mail-server stacks dominate the field at M03 launch:

- **Postfix + Dovecot** — the canonical large-scale UNIX mail stack. Postfix 3.8.x is the current LTS; Dovecot 2.3.21 LTS. Battle-tested at hyperscale (e.g., FastMail, ProtonMail), broad operator pool, deep tuning surface, integrates with Rspamd / SpamAssassin / ClamAV with established recipes, mature TLS / DKIM / DMARC / MTA-STS / TLS-RPT support.
- **Stalwart Mail Server** — modern Rust-native single-binary SMTP + IMAP + JMAP + Sieve + WebDAV, JMAP-first design (RFC 8620 / 8621 native), single-config-file deployment, integrated full-text search (Tantivy), simpler ops surface. Stalwart 0.10.x LTS; smaller operator pool; less battle-tested at top-tier hyperscale volume.

Naïvely picking one for all tenants forces a trade. Postfix-only over-engineers the starter/pro tenant operator surface (multiple daemons, multiple config languages, hand-tuned LMTP between Postfix and Dovecot). Stalwart-only under-serves enterprise tenants who need (a) reputation-pool sophistication, (b) proven multi-million-mailbox horizontal scale, (c) DKIM/DMARC tooling depth that Postfix has spent twenty years accumulating, (d) operator familiarity from the enterprise mail-admin labor market.

The tenant-tier model already segments oyatie tenants (starter, pro, enterprise, enterprise-plus). Backend selection naturally follows tier because the ops complexity vs. scale-headroom curve is itself tier-shaped.

The `mail` µservice's BCs (`mailbox-store`, `inbound-smtp`, `outbound-smtp`, `imap-frontend`, `search-index`, etc.) already define port traits in their `-kernel` crates (`SmtpInboundReceiver`, `SmtpOutboundSubmitter`, `ImapSessionHandler`, etc., per PRD §"Port traits"). Adapters can implement those ports against either backend without leaking the choice upward.

## Decision

oyatie mail ships **two backend stacks** behind the µservice's port traits, selected per tenant by tenant tier:

1. **Postfix 3.8.x LTS + Dovecot 2.3.21 LTS** for tenant tier ≥ `enterprise`. Adapters: `oya-mail-inbound-smtp-adapter-postfix`, `oya-mail-outbound-smtp-adapter-postfix`, `oya-mail-imap-frontend-adapter-dovecot`. LMTP between Postfix and the `mailbox-store` Postgres+S3 backend; Dovecot reads from the same `mailbox-store` via the `dovecot-rust-lda` LDA adapter. Rspamd for spam classification. Per-tenant SMTP IP pool with warm-up automation (M3AAWG BCP v3).

2. **Stalwart 0.10.x LTS** for tenant tiers `starter` and `pro`. Adapter: `oya-mail-{inbound-smtp,outbound-smtp,imap-frontend}-adapter-stalwart`. Stalwart's internal SMTP queue + JMAP server + IMAP server unify under one process per cell; full-text search routed through the µservice's own `oya-mail-search-index-adapter-search-index` (Tantivy) rather than Stalwart's internal index to keep the search posture uniform across tiers.

3. **Both stacks share the same `oya-mail-*` BCs.** No business logic lives in adapters; ports in `-kernel`, math in `-domain`, orchestration in `-usecase`. Tier-specific behaviour (reputation pool size, warmup cadence, queue depth limits) lives in tenant configuration consumed by the usecase layer, not in adapter code. Cross-product rule LEAN-A2 unaffected.

4. **Tier promotion path.** A tenant moving starter→enterprise migrates from Stalwart to Postfix+Dovecot with the `mail-backend-migration` IP (zero-downtime: dual-publish DKIM keys + parallel-deliver during cutover, then drain Stalwart). The migration is operator-driven, not automatic.

5. **Both backends LTS-pinned** with quarterly upgrade IPs; security-patches consumed within 7d (Sev-1) or per next quarterly cycle.

## Alternatives Considered

### A. Postfix + Dovecot only (all tiers)
- Pros: one ops surface, one operator skill set, deepest battle-testing, broadest deliverability tooling.
- Cons: ops complexity (multiple daemons, multiple config languages, separate auth/quota/sieve paths) is excessive for starter/pro tenants whose volume rarely justifies it; operator cost for small tenants effectively makes mail unprofitable at low tiers.
- Rejected: violates the autonomous-decision principle "long-term right > short-term cost" *in the inverse direction* — the long-term right answer is to match tool weight to tenant scale.

### B. Stalwart only (all tiers)
- Pros: single binary, modern Rust, JMAP-first, simpler ops, integrated stack.
- Cons: not yet proven at the high-volume hyperscale enterprise tenants will throw at it (≥10k msg/s sustained per cell; ≥1M mailboxes/cell per PRD capacity envelope); thinner DKIM/DMARC/MTA-STS tooling ecosystem; smaller operator labor pool to hire from.
- Rejected: under-served enterprise tenant tier kills the hyperscaler-grade mandate at the tier that pays for hyperscaler grade.

### C. Build a custom SMTP + IMAP daemon (NIH)
- Pros: full control; perfect alignment with `oya-mail-*` types.
- Cons: violates NIH refusal principle; 5+ engineer-years to reach Postfix-class deliverability + RFC 5321/5322/8551/9051/8620 conformance; massive opportunity cost; perpetual maintenance burden for protocol edge cases (greylisting heuristics, sender reputation, BATV, SRS, ARC chains).
- Rejected: NIH violation; opportunity cost dwarfs differentiation value.

### D. Stalwart for inbound + Postfix for outbound (split by direction)
- Pros: leverage Stalwart's modern JMAP-side strength + Postfix's deliverability tooling.
- Cons: doubles operational surface per tenant (two daemons per tenant regardless of tier); breaks the per-tenant logical-mail-server boundary; doesn't actually solve the tier-fit problem.
- Rejected: worst of both worlds; ops surface increase without commensurate benefit.

### E. Microsoft Exchange / commercial mail-server licensed in
- Pros: known enterprise-grade.
- Cons: vendor coupling (the exact problem PRD Outcome 1 is designed to solve); licensing costs that perversely scale with success; per-tenant residency violations; not self-hostable under tenant pack-pinning.
- Rejected: contradicts the entire µservice's vendor-coupling-refusal posture.

## Consequences

### Positive

- Tenant-tier-shaped operator surface: starter/pro tenants get a single-binary stack their volume justifies; enterprise tenants get the proven hyperscale stack their volume demands.
- Backend-neutral BCs preserved: `oya-mail-*-kernel` knows nothing about Postfix or Stalwart; replacement of either backend is an adapter change, not a kernel change. Future addition of a third backend (e.g., for a regulator-mandated stack) is a new adapter, no kernel diff.
- Stalwart's JMAP-native posture aligns with ADR-MAIL-0003 (JMAP-first SDK launch) for starter/pro tenants — JMAP is already the native protocol, not a bolt-on.
- Enterprise tenants inherit Postfix's twenty-year deliverability tooling (sender-rep heuristics, DKIM rotation tooling, DMARC report aggregation, ARC chain support, M3AAWG BCP-conformant warmup automation).
- Operator labor market: Postfix admins are plentiful and cheap to hire; Stalwart's smaller pool is offset by Stalwart's much smaller ops surface.

### Negative

- Two backends to maintain, monitor, security-patch, and capacity-plan. Mitigated by LTS-pinning both with quarterly upgrade IPs and a shared dashboards/SLO surface so the µservice operator team sees a unified picture.
- Tier migration (Stalwart → Postfix+Dovecot on tier promotion) is a non-trivial operator-driven IP; can't be automated end-to-end because DKIM key rotation + reputation pool warmup + DNS propagation are inherently slow.
- Tier downgrade (Postfix+Dovecot → Stalwart) is not supported because Stalwart cannot match Postfix's reputation-pool sophistication. Documented as a "no path back" in the tenant SLA so enterprise tenants can plan accordingly.
- CI must run integration tests against BOTH backends; doubles the `oya-mail-inbound-smtp-adapter-*` + `oya-mail-outbound-smtp-adapter-*` + `oya-mail-imap-frontend-adapter-*` test matrix. Mitigated by sharing fixture data via the contract tests in `microservices/mail/tests/contract/`.

### Operational

- IaC charts split: `microservices/mail/iac/helm/postfix/`, `microservices/mail/iac/helm/dovecot/`, `microservices/mail/iac/helm/stalwart/`. Each chart references the shared `oya-mail-*-app` images; backend choice is a Helm value (`backend: postfix+dovecot | stalwart`).
- Per-tenant config records which backend the tenant runs; tenant-provisioning workflow consumes tier + region to pick the backend; migration record retained in `microservices/mail/evidence/backend-migration/<tenant>-<ts>.json`.
- Dashboards: per-backend SLO panels + cross-backend deliverability comparison panel so the µservice operator can detect "starter-tier Stalwart deliverability score lagging enterprise-tier Postfix" and triage at the backend level.
- Cargo workspace adds `oya-mail-*-adapter-postfix`, `oya-mail-*-adapter-dovecot`, `oya-mail-*-adapter-stalwart` per BC; total adapter crate count rises by ~12 (3 backends × 4 BCs that need adapters).

### Regulatory

- **RFC 5321** (SMTP) + **RFC 5322** (Internet Message Format) + **RFC 6376/8463** (DKIM) + **RFC 7208** (SPF) + **RFC 7489** (DMARC) + **RFC 8617** (ARC) + **RFC 8460** (TLS-RPT) + **RFC 8461** (MTA-STS): both stacks conform; the µservice's CI lane `dkim-key-rotation-conformance` exercises both adapters.
- **RFC 9051** (IMAP4rev2): both Dovecot and Stalwart support it; the µservice's port trait `ImapSessionHandler` is conformant via either adapter.
- **RFC 8620 / 8621** (JMAP Core / Mail): Stalwart native; Postfix+Dovecot via the `httpie-jmap-gateway` shim adapter `oya-mail-imap-frontend-adapter-jmap-bridge` (enterprise tier).
- KR-pack: both backends supportable under KR-resident KMS + 5y retention floor; pack onboarding chooses based on tier as above.
- HIPAA-pack: enterprise-only by definition (BAA required); Postfix+Dovecot stack inherits.

## References

- RFC 5321 — Simple Mail Transfer Protocol
- RFC 5322 — Internet Message Format
- RFC 9051 — IMAP4rev2
- RFC 8620 — JMAP Core; RFC 8621 — JMAP Mail
- RFC 6376 — DKIM; RFC 8463 — Ed25519 for DKIM
- RFC 7208 — SPF; RFC 7489 — DMARC; RFC 8617 — ARC
- RFC 8460 — TLS-RPT; RFC 8461 — MTA-STS
- Postfix Project (postfix.org) — 3.8.x LTS docs
- Dovecot Project (dovecot.org) — 2.3.21 LTS docs
- Stalwart Mail Server (stalw.art) — 0.10.x LTS docs
- Rspamd Project (rspamd.com) — spam classification
- M3AAWG Sender Best Common Practices v3
- ADR-0131 — Per-microservice flat layout
- ADR-0132 — Product-suite-and-bundle dissolution
- ADR-0133 — Industry best-practice conformance program
- `microservices/mail/PRD.md` §"Bounded Contexts" + §"Horizontal Scalability"
- `microservices/mail/competitor-parity-matrix.md`
- `microservices/mail/capacity-model.md`
- `microservices/mail/runbooks/dkim-key-rotation.md` (operational ceremony shared across both backends)
