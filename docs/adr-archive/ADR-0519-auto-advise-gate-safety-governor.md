---
id: ADR-0519
title: "AUTO/ADVISE/GATE safety governor: safety (not convenience) decides the automation tier; per-finding-code tier as DATA, meta-gate rejects untagged codes"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-700]
depends_on: [ADR-0516, ADR-0515]
amends: [ADR-0515]
related: [ADR-0515, ADR-0516, ADR-0529, ADR-0530, ADR-0531]
related_specs:
  - /specs/masterplan.json
  - /.omc/specs/deep-interview-agentic-delivery-fabric.md
milestone: W1
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0519: AUTO/ADVISE/GATE safety governor

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Decomposes ADR-0516 Component 2. **Amends ADR-0515** by extending its born-blocking/advisory model
with the per-finding-code tier as schema-enforced DATA. Operationalized at the gate layer by ADR-0529.
This ADR re-homes the automation-first doctrine formerly recorded as the (now-folded) ADR-0050.

## Context

The Agentic Delivery Fabric's automation-first default (auto-fix and auto-generate come before manual
enforcement) must be safety-bounded: a write-capable bot fleet multiplies attack surface, and a
blanket "automate everything" policy applies unsafe edits silently. The directive is explicit and
door:one-way, so the question is HOW TO BOUND automation safely — not whether to automate. The answer
is a falsifiable per-code safety classification.

## Decision

Every quality finding-code carries a mandatory automation tier, decided by **SAFETY, not convenience**,
stored as DATA per code (in `gate-disposition.json`), with a meta-gate that REJECTS any registered code
with no tier tag:

- **AUTO** iff deterministic ∧ behavior-preserving ∧ mechanically-falsifiable ∧ reversible+idempotent
  ∧ reviewable → auto-fix / auto-gen PR.
- **ADVISE** iff semantics-changing ∨ light-judgment → propose, human decides; never silent, never a
  sole hard-block.
- **GATE** iff irreversible / high-blast ∨ security / trust / access ∨ one-way-door ∨
  subjective-judgment ∨ unproven-FP-rate → block + human (PAUSE-AND-PAIR for prod / access).

Default ordering preference is AUTO → ADVISE → GATE, but the safety predicate is authoritative.
Promotion of a code to AUTO requires PROVING the five safety properties.

## Drivers

- Automate-everything pressure vs. the founder safety principle: no semantics-changing auto-fix, no
  subjective gates, advisory-until-infra before a soft gate blocks.
- The clean re-homing of the automation-first doctrine (formerly ADR-0050's
  AUTOMATED/SCHEDULED/BAR-RAISED labels) into a safety-bounded, auto-remediation-era governor:
  BAR-RAISED corresponds to GATE; the addition is the per-finding-code safety classification.

## Alternatives considered

- **Auto-fix everything** — rejected (unsafe; semantics-changing edits applied silently).
- **Convenience-ordered tiers** — rejected (lets unsafe fixes land).
- **Hand-classified tiers with no meta-gate** — rejected (lets untagged codes slip through).

## Consequences

**Amends ADR-0515** — extends its firewall / born-blocking-vs-advisory model (the W0 floor) with the
governor's per-finding-code tier as schema-enforced DATA in `gate-disposition.json`, the meta-gate over
untagged codes, and the five-property AUTO-promotion proof. PAUSE-AND-PAIR binds
branch-protection / mainline-advance / prod-access / canon-ratification. ADR-0529 makes this governor
operational at the gate layer. This ADR resolves the formerly-Proposed ADR-0050 automation-first
doctrine by re-authoring its automation labels into this safety-bounded governor (the
"foundry-driven triage" framing of the old doc is dropped, not carried). door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source: settled-vision spec (PASSED). Decomposes
ADR-0516; amends ADR-0515; re-homes the former ADR-0050 automation-first doctrine. Operationalized by
ADR-0529.*
