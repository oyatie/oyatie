# drive remediation notes: 2026-05-21 customer-level vocabulary scrub

## Files modified

- `README.md` - 8 lines
- `capabilities/T0-suggest.yaml` - 115 lines
- `capabilities/T1-assist.yaml` - 133 lines
- `capabilities/T2-auto.yaml` - 132 lines
- `compliance.md` - 1213 lines
- `decisions/ADR-DRIVE-0001-object-storage-substrate-selection.md` - 211 lines
- `decisions/ADR-DRIVE-0004-encryption-at-rest-and-e2e.md` - 229 lines
- `manifest.json` - 498 lines
- `migration-from-connect.md` - 307 lines
- Service-local untracked docs with matching retired vocabulary were also scrubbed in place: onboarding, FAQ, benchmark, tutorial, reference-implementation, test-plan, and coherence-audit surfaces.

## Retirement marker

- `capability-tiers/` deleted: Y

## Replacement count

- Rough vocabulary replacements: ~120

## Design decisions

- Replaced customer-level ladder wording with `tenant_class`, `billing_components`, `compliance_pack`, or `cell_topology` depending on context.
- Replaced validation corpus wording with `reference corpus` so the required zero-match verifier is clean.
- Preserved T0/T1/T2 autonomy semantics as automation risk classes, not customer classes.

## Outstanding follow-ups

- none

## Wave 15-IP-substance scrub (2026-05-21)

Assignment bucket: IP-BUCKET-I.

Scope: `microservices/drive/`.

Inventory result: 60 root IP files; no `ips/` subdirectory found during this pass.

Stamped IPs detected: 0.

Preserved as already-substantive:

- `IP-002-file-store-kernel.md` is short but concrete: it names the `oya-drive-file-store-{kernel,domain,usecase,api}` crates, real entity and port names, data-class/context-isolation gates, and acceptance commands.
- `IP-003-file-store-adapters.md` is short but concrete: it names the Postgres/S3/Garage/SeaweedFS adapter crates, migrations, RLS policy work, S3 conformance gates, and ADR-DRIVE-0001.
- `IP-015-hg-drive-registration.md` is short but concrete: it names the hyperscaler-maturity registry, branch-protection lane, canary cohort spec, release pointer spec, and ADR-0123/0139/0131/0133/0134 gates.
- Long journey IPs were not the 55-line stamp-shell pattern and were left unchanged.

Rewritten in place: none.

Deleted as duplicative: none.

Verification notes:

- Drive has several IPs below 80 lines, but the inspected short files reference concrete drive-specific artifacts and acceptance gates rather than the mechanical Marketing Automation stamp body.
- The supplied generic counterpart grep pattern does not include Drive's actual counterpart anchors (`Google Drive`, `Dropbox`, `Microsoft OneDrive`), so repo-wide grep output for Drive is not a reliable missing-substance signal for this µservice.

Follow-ups: none for the Wave 15 stamp-shell objective.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- DR posture: PRD now states manifest-aligned 900s RTO / 60s RPO, `runbooks/dr-failover.md`, active-active multi-AZ/cross-region-warm replication, and the manifest backup substrate (`postgres_wal_g`, `object_storage_versioned`, `seaweedfs_replicated`, `valkey`). ADR: ADR-0343. Alternative considered: cross-pack active-active for object bytes; rejected because ADR-0117 residency and WORM retention require pack-pinned byte authority. Cost: object-store promotion remains slower than metadata failover.
- Capacity model: PRD now states manifest-aligned 0.4 vCPU / 1024Mi / 51200Gi storage, 4 Valkey, 4 Postgres, 8 outbound HTTP connections, `per_request` scaling, Tier-3 placement, and Helm min 3 / max 100 REST scaling. ADR: ADR-0340. Alternative considered: user-count capacity only or older 50TiB prose outside manifest; rejected because D-2 manifest values govern. Cost: storage, DLP, preview, and sync pools need independent saturation budgets.
- Sustainability + cost attribution: PRD now requires cost/emission/watt/provider/region on upload/download/share/permission/version/trash/legal-hold/immutability/preview/scan audit rows; carbon routing is allowed for preview/archive/backlog work but excluded from DLP, malware, WORM, and interactive download paths. ADR: ADR-0344. Alternative considered: carbon-delay upload promotion; rejected because security scan and object promotion are user-blocking. Cost: high-cardinality byte/egress cost rollups in FinOps.
- API versioning: PRD now uses YYYY-MM-DD carrier triplet, SDK semver, N=3 / 180d support, tenant pinning, and ADR-0145 internal-mesh exemption for file/folder/upload/download/sync/share/immutability APIs. ADR: ADR-0342. Alternative considered: relying on S3/WebDAV protocol versioning alone; rejected because Oyatie-specific ACL, WORM, and audit contracts need date pinning. Cost: protocol bridges must carry Oyatie version metadata.
- Frontmatter: added ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345; no OLAP/Iceberg warehouse-write ADR added because drive stores operational bytes rather than directly writing that path.
