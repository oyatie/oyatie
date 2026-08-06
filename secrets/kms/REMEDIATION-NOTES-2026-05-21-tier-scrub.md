# cloud-kms tier scrub remediation notes

Wave: 15J-batch-4 BUCKET-01.

Files modified, with current line counts:

- README.md: 12
- benchmarks/cloud-kms-vs-aws-kms-vs-azure-key-vault-vs-vault-enterprise.md: 100
- coherence-audit-2026-05-20.md: 609
- faqs/kms-engineer-faq.md: 193
- feature-parity-matrix-2026-05-20.md: 411
- migration-playbooks/from-aws-kms-and-vault-enterprise.md: 166
- onboarding/kms-engineer-first-week.md: 161
- performance-benchmark-numbers-2026-05-20.md: 314
- reference-implementations/envelope-encrypt-rust-sdk.md: 208
- runbooks/hsm-cluster-failover.md: 269
- runbooks/key-material-quorum-loss.md: 269
- runbooks/rotation-cadence-drift-detection.md: 267
- tenant-class-adoption-deltas-vs-counterparts-2026-05-20.md: 370
- tutorials/envelope-encrypt-rotate-and-cryptoshred.md: 210

capability-tiers/ dir deleted: Y.

Vocabulary replacement count: roughly 417 source lines matched before scrub.

Design decisions:

- Replaced Bronze/Silver/Gold/Platinum KMS ladder language with `demo_trial` and `paid tenant_class`.
- Preserved HSM assurance, PQC, quorum approval, replication, and cryptoshred differences as paid availability, compliance-pack, or cell-topology requirements.
- Renamed the previous capability-tier delta artifact to a tenant-class adoption delta artifact.

Outstanding follow-ups: none for the vocabulary scrub. Separate implementation work remains for OpenSLO files and HSM boundary evidence.
