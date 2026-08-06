---
id: ADR-0529
title: "The AUTO/ADVISE/GATE safety governor made operational at the gate layer (per-code tier DATA + meta-gate + advisory-until-infra promotion proof); operationalizes ADR-0519 + amends ADR-0515"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-08
door: one-way
owner: founder
supersedes: []
superseded_by: [ADR-700]
depends_on: [ADR-0519, ADR-0515]
amends: [ADR-0515]
related: [ADR-0515, ADR-0516, ADR-0519, ADR-0528, ADR-0530, ADR-0531]
related_specs:
  - /specs/masterplan.json
milestone: W2
---

# ADR-0529: AUTO/ADVISE/GATE governor — operational at the gate layer

## Status

**Accepted — 2026-06-08 (founder-ruled; door: one-way).**

Operationalizes ADR-0519 (the vision-level governor) at the gate layer. **Amends ADR-0515** (binds its
`advisory-until-infra`/`infra_prereq` disposition mechanism). Detail under Component 2 of ADR-0516.
Re-homes the automation-first doctrine of the (now-folded) ADR-0050 into the gate-layer governor.

## Context

ADR-0519 ratified the AUTO/ADVISE/GATE safety governor as a vision principle. The
automation-first default of ADR-0528 (remediation-first) must be BOUNDED at the concrete gate layer:
every finding-code needs a tier as DATA, a meta-gate must reject untagged codes, and a soft gate must
prove zero false-positives before it can flip from advisory to blocking. A write-capable bot fleet
multiplies attack surface, so the safety classification — not convenience — must decide each tier.

## Decision

Ratify the founder safety governor that bounds the automation-first default of ADR-0528, operational at
the gate layer. Every finding-code is classified into exactly one tier, stored as DATA per code (in
`gate-disposition.json`), and a meta-gate REJECTS any registered code with no tier tag:

- **AUTO** iff (deterministic ∧ behavior-preserving ∧ mechanically-falsifiable ∧ reversible+idempotent
  ∧ reviewable) → auto-fix / auto-gen PR.
- **ADVISE** iff (semantics-changing ∨ light-judgment) → propose, human decides, never silent, never a
  sole hard-block.
- **GATE** iff (irreversible/high-blast ∨ security/trust/access ∨ one-way-door ∨ subjective-judgment ∨
  unproven-FP-rate) → block + human, PAUSE-AND-PAIR for prod/access.

Default ORDER is AUTO → ADVISE → GATE, but SAFETY decides the tier, not convenience. Promotion of a
code to AUTO requires PROVING the five safety properties; a soft/structural gate ships
`advisory-until-infra` first and MAY NOT flip `advisory → baseline-block-on-new` until its advisory run
shows zero false-positives on a labeled fixture corpus.

## Drivers

- The antithesis is real: a write-capable bot fleet multiplies attack surface and maintenance for
  marginal toil savings on safe cases and unacceptable risk on dangerous ones.
- The directive is explicit + door:one-way, so the question is HOW TO BOUND it safely — a falsifiable
  per-code safety classification, not a blanket "automate everything."

## Alternatives considered

- **Blanket auto-fix-everything** — rejected (unsafe; semantics-changing edits applied silently).
- **Flag-and-block as the *default*** — rejected (discards the founder automation-first mandate).
- **Synthesis (chosen):** flag-and-block kept as the FALLBACK tier; auto-remediation strictly-additive
  (can only shrink the baseline, can only PROPOSE).

## Consequences

The disposition table becomes the single source of tier truth (reviewed DATA edits, not code); a new
meta-gate (`gate-tier-meta`, extending `automation-ratchet`'s
`enforceable_or_automatable_marked_human_judgment` discipline) is born-blocking on untagged codes;
founder PAUSE-AND-PAIR remains mandatory for branch-protection / mainline-advance / prod-access /
canon-ratification; the un-mechanizable remainder of every soft property is EXPLICITLY not gated.
**Operationalizes ADR-0519; amends ADR-0515.** This ADR (with ADR-0519) resolves the automation-first
doctrine formerly carried by the Proposed ADR-0050 — its BAR-RAISED label maps to GATE, refined for the
auto-remediation era; the "foundry-driven triage" framing is dropped. door:one-way.

---
*Accepted 2026-06-08 (founder-ruled; door:one-way). Source:
AUTOMATED-QUALITY-ENFORCEMENT-AND-AUTOREMEDIATION-ARCHITECTURE.md (RATIFY-TO-ADR). Operationalizes
ADR-0519; amends ADR-0515; re-homes the former ADR-0050 automation labels.*
