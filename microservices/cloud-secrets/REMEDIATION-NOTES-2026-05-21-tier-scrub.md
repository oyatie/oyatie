# cloud-secrets tier scrub remediation notes

Wave: 15J-batch-4 BUCKET-01.

Files modified, with current line counts:

- README.md: 12
- ARCHITECTURE.md: 754
- PRD.md: 363
- PHASE-01-OPENBAO-SECRETREFERENCE-SUBSTRATE.md: 133
- benchmarks/cloud-secrets-vs-vault-vs-aws-sm-vs-azure-kv-vs-gcp-sm-vs-akeyless.md: 105
- capabilities/audit-query.yaml: 106
- capabilities/secret-reference-resolve.yaml: 104
- capabilities/secret-rotate.yaml: 100
- capacity-model.md: 125
- coherence-audit-2026-05-20.md: 731
- competitor-parity-matrix.md: 127
- compliance.md: 1162
- contracts/openapi/cloud-secrets.yaml: 447
- cost-budget.md: 111
- dpia.md: 230
- failure-modes.md: 268
- faqs/security-engineer-faq.md: 174
- feature-parity-matrix-2026-05-20.md: 409
- manifest.json: 410
- migration-playbooks/from-hashicorp-vault.md: 189
- multi-region.md: 212
- onboarding/security-engineer-first-week.md: 120
- performance-benchmark-numbers-2026-05-20.md: 437
- policy/data-residency.md: 233
- policy/secret-isolation.md: 173
- policy/tenant-scope.cedar: 189
- reference-implementations/static-and-dynamic-secret-flow-rust-sdk.md: 153
- sdk-plan.md: 219
- slos/hsm-availability.openslo.yaml: 48
- tenant-class-adoption-deltas-vs-counterparts-2026-05-20.md: 353
- threat-model.md: 506
- tutorials/issue-rotate-and-revoke-dynamic-postgres-credential.md: 152

capability-tiers/ dir deleted: Y.

Vocabulary replacement count: roughly 476 source lines matched before scrub.

Design decisions:

- Replaced Bronze/Silver/Gold/Platinum language with `demo_trial` and `paid tenant_class`.
- Converted structured capability entries in `manifest.json` from customer ladder fields to `availability` plus ADR-0330 `tenant_class_model`.
- Preserved HSM, encryption-key BYOK, and compliance distinctions as paid availability, compliance-pack activation, and HSM/cell topology requirements.
- Renamed the previous capability-tier delta artifact to a tenant-class adoption delta artifact.

Outstanding follow-ups: none for the vocabulary scrub. Separate implementation work remains for the actual tenant_class adoption IP and cap-breach behavior.

## Wave 15-IP-substance scrub (2026-05-21)

- Rewritten thin foundation IPs: 12
  - `IP-002-secretreference-uri-spec.md`
  - `IP-004-resolver-domain.md`
  - `IP-005-resolver-usecase.md`
  - `IP-006-resolver-adapter-openbao.md`
  - `IP-007-resolver-rest-and-sdk-rust.md`
  - `IP-008-sdk-ts-python-bindings.md`
  - `IP-009-openbao-operator.md`
  - `IP-010-key-rotation-scheduler-worker.md`
  - `IP-011-hsm-integration-adapter-hsm.md`
  - `IP-012-per-tenant-namespace-controller.md`
  - `IP-013-audit-emitter-bridge-to-audit-chain.md`
  - `IP-014-observability-slo-branch-protection-hg-cloud-secrets.md`
- Preserved as already substantive with counterpart anchors added: 3
  - `IP-001-layer-a-openbao-postgres-hsm-iac.md`
  - `IP-003-resolver-kernel.md`
  - `IP-015-lean-a11-raw-secret-emission-lane-wiring.md`
- Deleted IP files: 0
- Bounded verification grep-recognized counterpart anchors added: 32 IP files. `GitHub Actions Secrets` appears only as a CI secret-distribution anchor; primary comparator truth remains OpenBao/Vault, managed secret stores, KMS/HSM, and audit-chain evidence.
- Evidence basis: `PRD.md`, `ARCHITECTURE.md`, `manifest.json`, `catalog/`, `contracts/`, `policy/`, `slos/`, `competitor-parity-matrix.md`, `feature-parity-matrix-2026-05-20.md`, and benchmark artifacts.
