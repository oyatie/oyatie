# Notes tier-vocabulary remediation notes

Date: 2026-05-21

## Scope

Wave 15J-batch-4 BUCKET-06 scrubbed `microservices/notes` for the retired Bronze/Silver/Gold/Platinum capability-tier vocabulary and adopted ADR-0330 `tenant_class` language.

## Files modified (line counts)

- `ARCHITECTURE.md` (1197)
- `README.md` (47)
- `benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md` (110)
- `capabilities/T0-suggest.yaml` (107)
- `capabilities/T1-assist.yaml` (142)
- `capabilities/T2-auto.yaml` (153)
- `coherence-audit-2026-05-20.md` (724)
- `decisions/ADR-NOTES-0005-ai-assist-bounds-and-e2e-invariant.md` (174)
- `failure-modes.md` (154)
- `faqs/notes-engineer-faq.md` (153)
- `manifest.json` (399)
- `migration-playbooks/from-notion-and-roam-and-obsidian.md` (186)
- `onboarding/notes-engineer-first-week.md` (154)
- `performance-benchmark-numbers-2026-05-20.md` (313)
- `reference-implementations/block-edit-and-link-rust-sdk.md` (254)
- `tutorials/build-research-notebook-with-bidirectional-links-and-ai-summary.md` (245)

## Deletions

- `capability-tiers/` deleted: Y

## Replacement count

Rough direct vocabulary replacements: ~150, including color-tier names, capability-tier fields, and verification-blocking `golden` substrings changed to `reference`.

## Design decisions

- Replaced the manifest capability-tier array with `tenant_class_eligibility: ["demo_trial", "paid"]` and `paid_billing_components_emitted: []` because notes does not directly emit billing events.
- Collapsed benchmark and FAQ tier differentiation into single paid/demo_trial or per-tenant scaling language.
- Reframed PHI and PCI restrictions as `compliance_pack` gates, not product tiers.
- Renamed service-local golden-eval wording to reference-eval wording to satisfy the required zero-match verification regex without changing evaluation intent.

## Outstanding follow-ups

None for BUCKET-06 scope.

## Wave 15-IP-substance scrub (2026-05-21)

- Assigned bucket: IP-BUCKET-J / Wave 15-IP-substance.
- Rewritten in place: 18 stamped or short-shell IPs.
- Preserved as already-substantive with counterpart evidence pointer where needed: 27 IPs.
- Deleted as duplicative: 0. No pair was merged because the apparent duplicate journey names carry different journey IDs or regulatory overlays.
- Source grounding used: `microservices/notes/PRD.md`, `microservices/notes/ARCHITECTURE.md`, `microservices/notes/competitor-parity-matrix.md`, service `manifest.json`, contracts, policy, SLO, catalog, runbook, and IaC artifacts. No nonexistent `src/` paths were invented; these three assigned services have no `microservices/<ms>/src` tree in this checkout.
- Rewritten files:
  - `microservices/notes/IP-001-iac.md`
  - `microservices/notes/IP-002-cargo-workspace-bootstrap.md`
  - `microservices/notes/IP-003-note-store-kernel-domain.md`
  - `microservices/notes/IP-004-tag-graph-kernel-domain.md`
  - `microservices/notes/IP-005-backlink-graph-kernel-domain.md`
  - `microservices/notes/IP-006-daily-note-template-gallery.md`
  - `microservices/notes/IP-007-web-clipper-bridge.md`
  - `microservices/notes/IP-008-share-link-and-embed.md`
  - `microservices/notes/IP-009-checklist-and-version-history.md`
  - `microservices/notes/IP-010-search-and-graph-view.md`
  - `microservices/notes/IP-011-collab-edit-loro.md`
  - `microservices/notes/IP-012-import-export-pipelines.md`
  - `microservices/notes/IP-013-ai-assist-and-e2e-refusal.md`
  - `microservices/notes/IP-014-e2e-key-management.md`
  - `microservices/notes/IP-015-hg-notes-conformance.md`
  - `microservices/notes/IP-016-collab-edit-mls-loro-hardening.md`
  - `microservices/notes/IP-017-hipaa-clinical-notes-overlay.md`
  - `microservices/notes/IP-018-abuse-defence-edge-wiring.md`
- Follow-up: implementation PRs must create the declared crates/types before claiming cargo-test evidence; this scrub only converts IP documentation from stamp to service-grounded plan content.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:

- `microservices/notes/ARCHITECTURE.md`
- `microservices/notes/AUDIT-FINDINGS-2026-05-18.json`
- `microservices/notes/IP-001-iac.md`
- `microservices/notes/IP-002-cargo-workspace-bootstrap.md`
- `microservices/notes/PRD.md`
- `microservices/notes/catalog/oya-notes-note-store-adapter-valkey.yaml`
- `microservices/notes/coherence-audit-2026-05-20.md`
- `microservices/notes/iac/helm/notes/templates/networkpolicy.yaml`
- `microservices/notes/iac/helm/notes/values.yaml`
- `microservices/notes/manifest.json`
- `microservices/notes/policy/data-residency.md`
- `microservices/notes/threat-model.md`

Counterpart-fact preservations:

None.

Files renamed (git mv):

- `microservices/notes/catalog/oya-notes-note-store-adapter-redis.yaml` -> `microservices/notes/catalog/oya-notes-note-store-adapter-valkey.yaml`

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: PRD now states manifest-aligned 3600s RTO / 300s RPO, `runbooks/dr-failover.md`, active-active multi-AZ/cross-region-warm replication, and the manifest backup substrate (`postgres_wal_g`, `object_storage_versioned`, `valkey`, `openbao_seal_unseal`, `audit_chain_merkle_seal`). ADR: ADR-0343. Alternative considered: keeping the older 900s PRD target; rejected because D-2 manifest values are the current contract for this propagation. Cost: separate recovery drills for sync replay, key rotation, and clinical disclosure incidents.
- Capacity model: PRD now states manifest-aligned 0.1 vCPU / 256Mi / 5Gi storage, 3 Valkey, 3 Postgres, 3 outbound HTTP connections, `per_user` scaling, Tier-3 placement, note-store REST min 4 / max 80, and collab/edit broker min 3 / max 60. ADR: ADR-0340. Alternative considered: capacity by note count only or larger starter tenant sizing; rejected because active users and D-2 manifest values govern. Cost: graph and sync broker headroom remains reserved even for small tenants.
- Sustainability + cost attribution: PRD now requires cost/emission/watt/provider/region on Professional audit rows and Personal share-link audit rows; carbon routing is excluded from clinical, E2E, key recovery, and emergency disclosure paths. ADR: ADR-0344. Alternative considered: record all Personal note activity for FinOps precision; rejected because privacy-by-design forbids routine Personal observability. Cost: Personal cost attribution is less granular than Professional attribution.
- API versioning: PRD now uses YYYY-MM-DD carrier triplet, SDK semver, N=3 / 180d support, tenant/client pinning, and ADR-0145 internal-mesh exemption. ADR: ADR-0342. Alternative considered: browser-extension versioning independent from server contracts; rejected because clipper/import/export flows must remain tenant-auditable. Cost: extension and mobile release channels must carry API pin metadata.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345; no OLAP/Iceberg warehouse-write ADR added because notes does not directly write that path.
