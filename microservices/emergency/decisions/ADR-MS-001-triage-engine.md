# ADR-MS-001 — Triage Engine: ESI 5-Level Core + Reassessment + Pediatric Overlay

Status: Proposed
Microservice: `emergency`
Owner: emergency-medicine-platform-engineer
Date: 2026-05-21
Supersedes: none
Authority: ADR-0332 (in flight) | ADR-0131 | ADR-0251 | ADR-0064

---

## Context

ED triage is the highest-criticality gate in the µservice. A miscalibrated triage misroutes patients into the wrong bed, the wrong protocol, the wrong wait queue, and may produce harm or regulatory exposure. The triage engine must:

- Implement the canonical 5-level Emergency Severity Index (ESI v4) algorithm.
- Support re-triage as a first-class event (not an overwrite).
- Apply pediatric overlay (PEWS) when the patient is under 18.
- Honor pack-driven extensions without weakening canonical-base rules.
- Project a FHIR R4B Observation per triage close with LOINC-coded acuity.
- Surface a room-recommendation hint at triage close.
- Stay under 600 ms p95 wall-clock for save.

Counterpart products (T-System, Wellsoft, Cerner FirstNet, Epic ASAP) each take a different stance — T-System leans heavily on complaint-driven structured templates, Wellsoft uses a vital-driven decision tree, Cerner FirstNet integrates with Millennium's order catalog from the triage screen. ED-IS deliberately decouples triage from catalog choice so that the triage engine remains portable across packs.

## Decision

Adopt ESI v4 as the canonical-base triage algorithm. Implement re-triage as an append-only sequence of `TriageEncounter` records keyed by `(encounter_id, sequence)`. Implement PEWS as a parallel pediatric overlay that is auto-selected when the patient's age band is ≤ 17, with attending override permitted (e.g., pregnancy in a 16-year-old). Pack extensions may add complaint-driven shortcuts but cannot remove the canonical ESI decision tree.

Triage save executes in a single transaction:

1. Validate vitals, acuity, chief complaint, and optional pediatric overlay.
2. Persist the `TriageEncounter` row with the next sequence.
3. Project FHIR Observation (acuity LOINC 75636-1, vitals codes) async post-commit.
4. Publish `ed.triage.completed` (or `ed.triage.reassessed`) event.
5. Apply room-recommendation hint based on acuity and current bed grid state.

The acuity-driven room recommendation is a rule-based default; the AI-assisted variant is opt-in per tenant pack and lives in `roomassignment` context, not in `triage`.

## Consequences

Positive:

- Single canonical algorithm; no fork between products.
- Re-triage history is forensically intact; quality teams can replay.
- Pediatric overlay does not impose a separate code path on the adult flow.
- Pack extensions are additive only, preserving canonical-base neutrality per ADR-0064.

Negative / cost:

- ESI v4 has known calibration drift between raters; we mitigate with a calibration dashboard but cannot solve the underlying social problem.
- LOINC mapping must be maintained as terminology updates land; we'll subscribe to `healthcare-integration` terminology updates.

## Alternatives Considered

- **CTAS** (Canadian Triage and Acuity Scale) as canonical — rejected: ESI v4 has the broader install base and is what US verifiers expect.
- **MTS** (Manchester Triage System) as canonical — rejected: MTS will be available as a pack overlay for UK / EU tenants but not the canonical base.
- **AI-driven triage** — deferred: an AI acuity recommender will be opt-in under `intelligence` µservice and EU-AI-Act Annex III gating, but the canonical base remains a human-judgement ESI v4 entry.

## Open Items

- Configuration of complaint-driven shortcut catalog per tenant pack (US, EU, KR) — handled in IP-001.
- Migration story from a T-System-anchored ED migrating into ED-IS — outlined in `IP-001` migration appendix.

## Authority Trail

- ADR-0332 (in flight) — the parent µservice ADR.
- ADR-0131 — flat layout.
- ADR-0251 — compliance pack primitive (TJC + EMTALA).
- ADR-0064 — canonical-base neutrality.
