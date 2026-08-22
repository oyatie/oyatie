---
doc_status: archived
doc_class: Standard
shape: tombstone
length_cap: 120
authority_tier: 4
status: Retired
date: 2026-05-12
retired_at: 2026-08-05
live_authority: false
purpose: |
  Retirement tombstone for the former Claude Code / external-harness brand
  standard. This path is retained only so historical links resolve. It is not
  live operating authority for hooks, skills, magic-keyword routing, or
  agent coordination.
canonical_authority: docs/AGENTS.md + ADR-0515 + ADR-0619
companion_docs:
  - docs/AGENTS.md
  - docs/MASTERPLAN.md
  - docs/decisions/ADR-0709-general-live-apex.md
  - docs/decisions/ADR-0700-ci-admission-live-apex.md
  - docs/decisions/ADR-0709-general-live-apex.md
related_adrs:
  - ADR-0116
  - ADR-0363
  - ADR-0515
  - ADR-0619
---

# Claude Code Harness — RETIRED (no live authority)

> **Status:** Retired — 2026-08-05 (RR-HARNESS-0619 / ADR-0619).
> **Live authority:** [`docs/AGENTS.md`](../AGENTS.md) operating contract;
> merge admission per [ADR-0515](../decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md)
> (`presubmit`); harness-brand retirement per
> [ADR-0619](../decisions/ADR-0619-zero-live-context-retirement-of-external-agent-harness-brand.md)
> and historical external-tool retirement per
> [ADR-0116](../decisions/ADR-0116-retire-external-agent-coordination-tooling.md).
> **Do not** treat OMC / OMX / GJC / Hermes / oh-my-claudecode magic-keyword
> routing, `.omc/` / `.omx/` / `.gjc/` trees, or this file's former body as
> current coordination authority.

## Where to go instead

| Concern | Live surface |
|---|---|
| Agent operating contract | [`docs/AGENTS.md`](../AGENTS.md) |
| Machine-readable entry points | [`/specs/root-hub-pointers.json`](../../specs/root-hub-pointers.json) |
| Plan / sequencing authority | [`/specs/masterplan.json#masterplan_v2`](../../specs/masterplan.json) (human projection: [`docs/MASTERPLAN.md`](../MASTERPLAN.md)) |
| Merge / CI admission | [ADR-0515](../decisions/ADR-0515-phase0-firewall-one-canonical-ci-cloud-native-posture.md) — single protected context `presubmit` |
| Contribution path | Isolated worktree → SSH-signed commit → PR against `dev` → reviewer APPROVE + `presubmit` green (plain `git` / `gh`; no external harness lock) |
| Local multi-model delivery kit (optional, **not** merge authority) | `.grok/` (mm-delivery: `mm-drive`, stage packs, dual-critic) when present on the operator machine |
| Installed agent runtime skills / roles | Runtime catalogs (Codex: `~/.codex/skills` + `~/.codex/agents`; project `.codex/` / `.claude/` overlays only when intentionally checked in). Do not re-vendor external harness skill trees. |
| Cross-agent tool-name mapping (reference) | [`multi-agent-tool-map.md`](multi-agent-tool-map.md) — OMC columns are historical / compatibility-only |
| Git rationale discipline | [`git-workflow.md`](git-workflow.md) + fence discipline in [`agent-instructions-discipline.md`](agent-instructions-discipline.md) |

## Retirement rationale (summary)

1. **ADR-0116** retired out-of-repo coordination tooling (grit/rtk/icm/vox) in favour of the in-repo governance pipeline.
2. **ADR-0363** retired the bespoke Oya VCS claim/verify/done/promote ratchet; plain git + protected PR is the substrate.
3. **ADR-0515** is the single canonical cloud-ci admission context (`presubmit`).
4. **ADR-0619** forbids re-entry of a retired external agent-harness brand as live authority or source-specific plan ingest.
5. Local session stores under `.omc/` / `.omx/` / `.gjc/` are gitignored provenance at best; live machine-readable authority is under `/specs`, `/registry`, `/evidence`, and `/templates`.

## What this path is not

- Not a hook SSOT (see [`.claude/settings.json`](../../.claude/settings.json) and `docs/AGENTS.md` §Per-agent appendices when a Claude Code session is in use).
- Not a skill catalog (installed runtime + optional `.grok/` kit only).
- Not a cancellation protocol authority (`/oh-my-claudecode:*` keywords are compatibility residue only).
- Not plan authority (never was after masterplan v2; do not follow `.omc/plans/**` as executable authority).

## Historical body

The pre-retirement normative body (sanctioned-primitive triad, OMC magic-keyword
routing, SessionStart OMC skill loaders, Directive-12 logging to external
topics) is **deleted from this path** so agents cannot re-ingest it as live
procedure. Git history retains the prior blob for authorized forensic recovery
only (ADR-0619 §4 provenance model).

If a linked procedure is still required, re-author it under the live surfaces
above with provider-neutral language and ADR-0515 admission semantics — do not
restore external harness brand names as current authority.
