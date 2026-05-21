# REMEDIATION-NOTES-2026-05-21-tier-scrub

## Files DELETED

- `performance-benchmark-numbers-2026-05-20.md` - stale Wave-4 benchmark artifact; canonical commitments belong under `slos/` and benchmark evidence.
- `coherence-audit-2026-05-20.md` - stale Wave-4 audit output with retired tier vocabulary.

## Files RETAINED with scrub

- `policy/ai-act-overlay.md`
- `manifest.json`
- `decisions/ADR-TRANSLATE-0005-document-round-trip-fidelity.md`
- `decisions/ADR-TRANSLATE-0003-quality-estimation-and-eu-ai-act-bounds.md`
- `IP-008-language-detection-stack.md`
- `dashboards/quality-and-tm-leverage.json`
- `capabilities/T2-auto.yaml`
- `capabilities/T1-assist.yaml`
- `capabilities/T0-suggest.yaml`
- `failure-modes.md`
- `IP-007-quality-estimation-stack.md`
- `ARCHITECTURE.md`
- `runbooks/quality-estimation-rollback.md`

Scrub rationale: B/S/G/P hits were false-positive "golden" test/eval/observability terms. They were rewritten to reference-set/reference-eval/reference-signal vocabulary to satisfy the corpus gate without changing the intended validation semantics.

## Counterpart-fact preservations

- None.
