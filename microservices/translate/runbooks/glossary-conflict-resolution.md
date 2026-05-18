---
doc_class: Runbook
title: Glossary / termbase conflict resolution
microservice: translate
severity: "Sev-3 (single-term conflict) / Sev-2 (tenant-wide termbase load failure)"
status: Accepted
owner_team: axis-translate + tenant-localization-ops
date: 2026-05-18
related_artifacts:
  - microservices/translate/failure-modes.md (FM-60..FM-63)
  - microservices/translate/decisions/ADR-TRANSLATE-0002-translation-memory-and-leverage-model.md
  - microservices/translate/PRD.md (FR-05 + FR-22 + termbase-and-glossary BC)
doc_status: published
---

# Runbook: Glossary / termbase conflict resolution

## Trigger

Any of:

- FM-60: TBX (ISO 30042) import failure — schema-validation error.
- FM-61: Term-concept conflict — same source term has two distinct mandated target translations within same tenant project.
- FM-62: Glossary-override frequency exceeds tenant alert threshold (`oya_translate_glossary_override_total{tenant=<t>}` rate > 10× baseline).
- FM-63: TBX export drops term-attributes (notes, register, partOfSpeech) — round-trip not bit-identical for non-canonical fields.
- Tenant escalation: localization manager reports inconsistent MT output across same project.

## Severity

| Symptom | Severity |
|---|---|
| Single-term conflict | Sev-3 |
| Tenant-wide TBX load failure | Sev-2 |
| Cross-tenant termbase contamination (HARD) | Sev-1 (P0; engage data-residency runbook) |
| TBX round-trip lossy | Sev-3 |

## Symptoms

- `oya_translate_termbase_import_failures_total` non-zero.
- `oya_translate_termbase_concept_conflict_total{tenant=<t>}` non-zero.
- `oya_translate_glossary_override_total` rate spike.
- Tenant Slack / support tickets reporting "MT output ignores our terminology".

## Diagnosis

| Hypothesis | Signal | Investigation |
|---|---|---|
| Schema-level TBX malformed | parser error in import-worker logs | inspect TBX vs ISO 30042; validate with `tbx-validator` |
| Encoding mismatch (UTF-8 BOM / EUC-KR) | invalid character codepoints in import | re-encode to UTF-8 sans BOM; retry import |
| Conflicting term-concepts (two `<termEntry>` for same concept) | concept_conflict metric | manual reconciliation (see Resolution Path A) |
| Termbase not loaded into router | MT output respects no terminology | verify `oya_translate_termbase_load_total{tenant=<t>}` > 0; check termbase-loader worker |
| Pack-specific dialect mismatch | tenant's terms target wrong locale (e.g., zh-CN vs zh-TW) | reload with correct target locale tag |
| Project scoping bug | terms loaded into wrong project bucket | termbase project-isolation review |

## Resolution Path A — Term-concept Conflict

When two `<termEntry>` for the same concept have conflicting target translations:

| Step | Action |
|---|---|
| 1 | Identify conflicting entries: `cargo run -p oya-dev-cli -- translate audit-termbase-conflicts --tenant <t> --project <p>` |
| 2 | Export conflict report (CSV); send to tenant localization manager for adjudication |
| 3 | Tenant chooses canonical translation per concept |
| 4 | Apply chosen translations via TBX re-import: `cargo run -p oya-dev-cli -- translate import-tbx --tenant <t> --project <p> --file <fixed.tbx>` |
| 5 | Verify zero conflicts: `cargo run -p oya-dev-cli -- translate audit-termbase-conflicts` |
| 6 | Audit-chain emit `TermbaseConflictResolved{tenant, project, concept_count, reviewer_ref}` |

## Resolution Path B — TBX Import Failure

| Step | Action |
|---|---|
| 1 | Identify validation error: `oya-translate-cli tbx-validate <file>` |
| 2 | Fix: typical issues — non-UTF-8 encoding, missing required `<langSet>` for source-lang, malformed `<termEntry>` structure |
| 3 | Re-import via API or CLI |
| 4 | Verify count matches expected segments |
| 5 | If recurring: file engineering ticket; suggest tenant use TBX-Basic dialect |

## Resolution Path C — Termbase Not Loaded into Router

| Step | Action |
|---|---|
| 1 | Verify termbase loaded: `cargo run -p oya-dev-cli -- translate verify-termbase-load --tenant <t>` |
| 2 | If not loaded: trigger reload `cargo run -p oya-dev-cli -- translate reload-termbase --tenant <t>` |
| 3 | Verify MT output respects glossary: sample 10 segments containing glossary terms |
| 4 | If still ignored: termbase-router integration bug; engineering escalation |

## Resolution Path D — Cross-tenant Contamination (P0)

If termbase from one tenant leaks into another (`tenant_id` mismatch in router decision):

1. **HALT immediately**: this is a P0 data-isolation breach.
2. Engage `sovereign-tenant-cross-region-leak-incident-p0.md` runbook (the runbook covers all cross-tenant breaches, residency is one class).
3. Council-privacy + legal-counsel + ops-security.

## Glossary Override Frequency Spike (FM-62)

When tenant overrides MT output against glossary > 10× baseline:

1. Inspect override pattern via dashboard.
2. Common causes:
   - Termbase is wrong (tenant disagrees with own previous terms).
   - MT output has improved; old glossary is stale.
   - Per-content-class terminology gap (e.g., legal vs marketing).
3. Tenant comms: offer termbase audit + revision support.

## TBX Round-Trip Verification

Per ADR-TRANSLATE-0002 §"TBX round-trip":

- Required attributes: `concept-id`, `term`, `langSet` per source/target.
- Recommended attributes (preserve where present): `partOfSpeech`, `note`, `definition`, `register`, `usageNote`.
- Round-trip metric: `oya_translate_termbase_roundtrip_fidelity` ≥ 0.99.

Verification:

```bash
cargo run -p oya-dev-cli -- translate verify-tbx-roundtrip \
  --tenant <t> --project <p>
# expects: fidelity ≥ 0.99
```

## Verification Commands

```bash
# Conflict-free termbase
cargo run -p oya-dev-cli -- translate audit-termbase-conflicts \
  --tenant <t> --project <p>
# expects: zero conflicts

# Termbase load status
cargo run -p oya-dev-cli -- translate verify-termbase-load --tenant <t>

# Sample glossary-respecting MT
cargo run -p oya-dev-cli -- translate sample-glossary-output \
  --tenant <t> --project <p> --sample-size 20
```

## Pack-Specific Considerations

| Pack | Note |
|---|---|
| pack-eu | GDPR personal-data-bearing glossaries (e.g., person names) classified PII_IDENTIFYING; access restricted |
| pack-kr | KR PIPA Art. 23 sensitive-data terms (medical, ethnic) require explicit termbase classification |
| pack-us-healthcare | HIPAA — PHI terms in glossary disabled by default; tenant opt-in + BAA |
| pack-jp | APPI sensitive-information categories; termbase audit required |

## Named Industry Sources

- ISO 30042:2019 (TBX).
- LISA / GALA TBX-Basic + TBX-Default dialects — `www.gala-global.org/tbx`.
- TermWiki / IATE (EU terminology) — `iate.europa.eu/`.
- TAUS data clouds for benchmark glossaries.

## References

- ADR-TRANSLATE-0002 (TM + leverage model).
- `microservices/translate/PRD.md` FR-05 + FR-22 + termbase-and-glossary BC.
- `microservices/translate/failure-modes.md`.
