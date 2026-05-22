# Treasury Remediation Notes - 2026-05-21 Tier Scrub

## Wave 15J-final-cleanup

- Rewrote the stamped PRD activation and policy blocks from retired tenant-class fields to `tenant_class` plus `billing_components`.
- Replaced IP frontmatter tier metadata with tenant-class and paid billing-component metadata.
- Replaced manifest activation metadata with `tenant_class_support` and `billing_components_emitted`.
- Scrubbed performance benchmark residue that still described Bronze/Silver/Gold/Platinum-style commercial grouping.
- Removed stale 2026-05-20 coherence audit and feature parity artifacts after this sweep superseded their tier-retirement residue.
- Verification target: zero retired B/S/G/P and `capability_tier` residue outside remediation notes.
