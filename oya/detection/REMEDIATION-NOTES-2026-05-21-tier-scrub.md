# Wave 15J-batch-4 tier scrub remediation notes: detection

## Files Modified

- README.md: 92 lines
- benchmarks/detection-vs-stripe-radar-vs-guardduty-vs-chronicle.md: 102 lines
- coherence-audit-2026-05-20.md: 622 lines
- reference-implementations/streaming-score-rust-sdk.md: 212 lines
- tutorials/build-payment-fraud-cedar-rule.md: 288 lines

## Directory Deletion

- capability-tiers/ dir deleted: Y

## Vocabulary Replacement Count

- Rough replacement count: ~200 matches, including deleted capability-tiers/ content.

## Design Decisions

- Replaced model-serving and graph-investigation availability language with `tenant_class`, `cell_topology`, and explicit demo caps.
- Removed old benchmark/customer ladder labels and command flags in favor of paid tenant_class plus deployment topology.
- Reworded audit evidence so it records the retired ladder without preserving the banned vocabulary outside this remediation note.
- Added README tenant-class adoption text linked to ADR-0330.

## Outstanding Follow-ups

- none
