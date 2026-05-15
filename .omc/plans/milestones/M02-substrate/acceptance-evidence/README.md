---
purpose: Per ADR-0063 §3 every milestone has an acceptance-evidence directory.
---

# M02-substrate acceptance evidence

Per ADR-0063 §3 every milestone has an acceptance-evidence directory.

Contents (signed evidence artifacts):
- Cargo gate transcripts (cargo check/build/clippy/nextest/deny exit 0)
- Fitness lane reports (lean-a1..a5 green per the milestone's exit gate)
- Ed25519-signed audit chain segment per (tenant, period) per Bominal ADR-0028
- Load test results meeting Performance Targets per ADR-0062 / per impl-plan
- Per-phase ICM phase-complete row excerpts

This README serves as the manifest; evidence files land here as the milestone progresses.
