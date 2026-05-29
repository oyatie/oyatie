# `comms-email` source root

The Rust crates that compose this µservice live in
`crates/oya-comms-email-*` per ADR-0131 flat-layout discipline +
ADR-0136 bounded-context layout.

This directory is the µservice's documentation root, not a code
root — per ADR-0131 ("src/ is the canonical code root [for new
µservices]"), the live code lands at the workspace crate level
because comms-email is a multi-bounded-context µservice (see
`catalog/bounded-contexts.json` for the 16 BCs).

Phase-1 crate map (parent-wired at registration):

- `crates/oya-comms-email-transactional-send-kernel` — trait
  surface (uses `oya-shared-email-comms-kernel`).
- `crates/oya-comms-email-transactional-send-domain` —
  preflight + send orchestration.
- `crates/oya-comms-email-transactional-send-api` — REST + gRPC
  surface.
- `crates/oya-comms-email-deliverability-kernel` — DKIM / SPF /
  DMARC kernel.
- `crates/oya-comms-email-webhook-ingest-api` — webhook
  endpoint.
- `crates/oya-comms-email-webhook-ingest-domain` — normalization
  + audit-chain emission.
- `crates/oya-comms-email-suppression-list-domain` — suppression
  business logic.
- `crates/oya-comms-email-suppression-list-adapter-postgres` —
  Postgres backing.
- `crates/oya-comms-email-dkim-rotation-domain` — rotation logic.
- `crates/oya-comms-email-dkim-rotation-adapter-bao` — OpenBao
  backing.
- `crates/oya-comms-email-from-domain-onboarding-domain` — state
  machine.
- `crates/oya-comms-email-audit-emission-domain` — schema-versioned
  emission.
- `crates/oya-comms-email-template-mjml-domain` — mrml compile.
- `crates/oya-comms-email-template-liquid-domain` — Liquid
  substitution.
- `crates/oya-comms-email-multi-region-routing-domain` — routing
  decisions.
- `crates/oya-comms-email-app` — composition root (binary).

This batch ships the kernel + the µservice's audit-grade
documentation pack. The 16 BC crates land in follow-up
implementation PRs scoped to each IP.

See also:
- `PRD.md`
- `PHASE-01-COMMS-EMAIL-SUBSTRATE.md`
- `catalog/bounded-contexts.json`
- `manifest.json`
