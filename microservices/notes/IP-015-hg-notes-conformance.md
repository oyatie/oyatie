---
doc_class: ImplementationPlan
impl_plan_id: IP-015-hg-notes-conformance
milestone: M02-foundation
phase: P01-notes-foundation
status: pending
owner: axis-notes + council-architecture
acceptance_lanes: [hg-notes, oya-governance-per-microservice-layout, oya-governance-authority-cohesion]
---

# IP-015: HG-NOTES hyperscaler-grade conformance gate (per ADR-0133)

## Intent

Register the `HG-NOTES` hyperscaler-grade conformance gate per ADR-0133. The gate asserts the notes µservice meets the conformance bar across all relevant facets:

- per-microservice flat layout (ADR-0131)
- dual-context isolation (parallel ADR-0135)
- e2e-ai-refusal invariant (ADR-NOTES-0005)
- pack residency (ADR-0117)
- documentation suite coverage (ADR-0063)
- canonical base + localization packs (ADR-0064)
- SLO-gated promotion (ADR-0130)
- authority cohesion (ADR-0133)
- statelessness + shardability (per ADR-0131)
- version pinning conformance (LTS pins)

## Registration

`specs/hyperscaler-gates.json` adds:

```json
{
  "id": "HG-NOTES",
  "microservice": "notes",
  "lanes": [
    "per-microservice-layout",
    "dual-context-isolation",
    "e2e-ai-refusal",
    "notes-pack-residency",
    "doc-coverage",
    "version-pinning-conformance",
    "authority-cohesion",
    "statelessness",
    "shardability"
  ],
  "owner_team": "axis-notes",
  "first_pass_target": "M02-P01-exit-gate"
}
```

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate hg-notes
# expected: all lanes green
```

## Halt Conditions

- Any sub-lane returns a finding — fix forward; HG-NOTES gates the PHASE exit.

## References

- ADR-0133 industry best-practice conformance.
- ADR-0131 per-microservice flat layout.
- `microservices/notes/PHASE-01-NOTES-FOUNDATION.md` exit_gate.
