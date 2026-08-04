# G030-S earlier-retained specs JSON canonical-gate correction — 2026-08-02

State: **PLANNING_ONLY — ELEVEN EARLIER-RETAINED SPECS JSON ROWS GRAPH-WIRED; DOMAIN GAPS UNCHANGED; NO DELETION/ACTIVATION**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements corrected G030-R. No repository path or policy was changed.

## Result

Corrected G030-R proved that the Buck2 Rust canonical-JSON gate recursively selects, byte-reads, canonicalizes, and evaluates every `specs/**/*.json` row except `specs/fixtures/**` and `*.generated.json`.

Reconstructing the remaining protected queue found **11 earlier-retained JSON rows** from G030-H and G030-K that satisfy that same complete-corpus contract:

```text
specs/design-system/job-board-search.json
specs/design-system/network-feed.json
specs/design-system/professional-profile-card.json
specs/design-system/recruiter-pipeline.json
specs/design-system/sales-copilot-panel.json
specs/design-system/shorts-creator-analytics-dashboard.json
specs/design-system/shorts-for-you-feed.json
specs/design-system/shorts-live-viewer.json
specs/design-system/shorts-video-editor.json
specs/reorg/ci-graph-additions.json
specs/reorg/kernel-move-plan.BLOCKED.json
```

All 11 promote from `POLICY_PROTECTED_MACHINE_ARTIFACT` to `GRAPH_WIRED_INPUT` under the census precedence. G030-T subsequently found that G030-N had counted the out-of-universe `registry/release/evidence-packs.tsv` as one of the 1,176 focus rows. With that one-row arithmetic correction, reconciled totals after S are **152 `MACHINE_SSOT` + 992 `GRAPH_WIRED_INPUT` + 32 `POLICY_PROTECTED_MACHINE_ARTIFACT` = 1,176**. Remaining protected queue: 19 fixture + 13 non-fixture. Delete candidates remain 0.

## Executable edge

At immutable tip:

- `ci/facade/canonical-json/canonical-json-policy.json` governs root `specs`;
- exclusions are only `.generated.json` and prefix `specs/fixtures/`;
- `ci/facade/canonical-json/src/lib.rs::collect_observed` calls recursive `walk_json` and `fs::read`s every selected JSON path;
- `ci/facade/canonical-json/tests/canonical_json.rs::live_governed_corpus_is_canonical_at_zero_baseline` evaluates the live corpus at zero baseline;
- `ci/facade/canonical-json/BUCK` declares `ci-canonical-json-gate` as a Buck2 Rust test.

Every path above is under `specs`, ends `.json`, is outside `specs/fixtures/`, and does not end `.generated.json`.

## Domain-boundary preservation

Canonical byte validation is a graph edge, not proof of domain completeness:

- The nine design-system rows remain catalog-only for design semantics: no executable `component_refs` resolver or implementation-conformance validator was proven. G030-K's catalog-gap finding stands.
- `ci-graph-additions.json` remains outside the generic `*-move-plan.json` codemod grammar and remains an ADR-0563 compatibility companion.
- `kernel-move-plan.BLOCKED.json` remains outside active move-plan discovery; its old paths remain present and destinations absent. It is not approved or executable as a move.

Their census class changes because the canonical gate reads and evaluates their bytes; their domain status does not.

## Anti-double-count and arithmetic

These 11 were explicitly retained by prior proofs:

- G030-K: nine design-system catalog-only JSON rows;
- G030-H: two reorg JSON companions.

They are not among corrected G030-R's exact residual 28. No row is counted twice.

```text
after corrected R (pre-T): 152 + 982 + 42 = 1176
promote k=11:              152 + 993 + 31 = 1176
G030-T TSV correction:     152 + 992 + 32 = 1176
remaining after T:           19 fixture + 13 non-fixture = 32
delete candidates: 0
```

## Review boundary

A final-queue independent audit was dispatched but failed with encrypted-content transport error. It remains **FAILED_TRANSPORT_NOT_APPROVE**. The 11-row correction is coordinator mechanical evidence from the immutable gate policy/source/test/Buck graph; it is not independent approval for mutation.

## Non-actions

- No design-system contract, reorg plan, gate policy, or baseline edited.
- No move-plan activated; blocked kernel plan remains blocked.
- No G028 push/apply; no G023 deletion; no #1523 restack push.
- No cluster or canonical dirty checkout mutation.
