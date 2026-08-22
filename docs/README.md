---
purpose: Current docs hub. Index only — not a dump.
doc_status: published
status: current
---

# Docs hub

Current. Short index. Do not restore a line-count dump here.

## Live spine (one source of truth)

1. [`specs/root-hub-pointers.json`](../specs/root-hub-pointers.json) — authority router.
2. [`docs/AGENTS.md`](AGENTS.md) — live operating contract until PHASE-5 promotion evidence.
3. [`specs/masterplan.json#masterplan_v2`](../specs/masterplan.json) — only live plan.
4. [`docs/ADR-INDEX.md`](ADR-INDEX.md) — live ADRs `0700`…`0717`.
5. [`specs/markdown-retirement-policy.json`](../specs/markdown-retirement-policy.json) — Markdown lifecycle. This is the protocol with `docs/AGENTS.md`. There is no second `docs/*.md` protocol twin.

## Bootstrap (how to start)

- Session entry: [`HANDOFF.md`](../HANDOFF.md) (thin redirect, not a plan).
- Agent entry: root [`AGENTS.md`](../AGENTS.md) / [`CLAUDE.md`](../CLAUDE.md) → this spine.
- Ritual: [`templates/checklists/swarm-agent-ritual.md`](../templates/checklists/swarm-agent-ritual.md). `docs/checklists/swarm-agent-ritual.md` does not exist.
- Merge path: `cargo fmt` / `clippy` / `cargo test --workspace` behind `presubmit` (ADR-0716). Cloudflare edge is `tofu -chdir=infra/cloudflare` (see `iac/README.md`).

## Current vs historical

| Status | What |
|---|---|
| **Current** | Spine above. Live ADRs in [`decisions/`](decisions/). Observability at [`observability/`](../observability/) + per-capability `<capability>/observability/slos/` (ADR-0701 / ADR-0706). |
| **Historical** | [`adr-archive/`](adr-archive/). Tombstones: [`DOCUMENTATION.md`](DOCUMENTATION.md), [`DOC-UPDATE-PROTOCOL.md`](DOC-UPDATE-PROTOCOL.md), [`DOC-CATALOG.md`](DOC-CATALOG.md), [`DOC-COVERAGE.md`](DOC-COVERAGE.md). Closed tracker: [`CONTRADICTION-LEDGER.md`](CONTRADICTION-LEDGER.md). May 2026 docs dump: `git show c7724347:docs/README.md`. `cloud/` is gone. |

## Diátaxis (AWS/Google/Azure shape)

Hubs are indexes. Pages stay short. Law lives once.

| Quadrant | Start |
|---|---|
| Tutorials | [`onboarding/`](onboarding/), [`tutorials/`](tutorials/) |
| How-to | [`runbooks/`](runbooks/), [`RUNBOOKS-INDEX.md`](RUNBOOKS-INDEX.md), ritual above |
| Reference | [`ADR-INDEX.md`](ADR-INDEX.md), [`standards/INDEX.md`](standards/INDEX.md), [`SPEC.md`](SPEC.md) |
| Explanation | [`decisions/`](decisions/), [`DESIGN.md`](DESIGN.md) (historical May draft — cite live ADRs first) |

## Do not

- Hand-edit `*.generated.json`.
- Add another `docs/*.md` protocol twin.
- Point at `cloud/cloud-observability/` or `{oya,cloud}/<service>/slos/`.
- Use ADR-0709 as a junk drawer for every citation.
