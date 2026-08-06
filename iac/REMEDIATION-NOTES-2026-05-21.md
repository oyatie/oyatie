<!-- WAVE 15J-BATCH-2 SCRUB REPORT
  µservice: cloud-iac
  capability_tiers_directory_deleted: yes
  manifest_tier_fields_removed: 2
  tier_references_scrubbed: 52
  ADR_0316_citations_replaced: 0
  cellular_criticality_preserved: 1
-->

## Wave 15-IP-substance scrub (2026-05-21)

- Inventoried `IP-*.md` plus `implementation-plans/IP-*.md` under
  `iac` only.
- Rewritten stamped/thin shells in the first pass:
  `IP-GITOPS-001-terraform-to-opentofu-migration.md`,
  `IP-GITOPS-002-argocd-app-of-apps-pattern.md`,
  `IP-GITOPS-003-tier-discipline-rollout.md`,
  `IP-GITOPS-004-opentofu-module-registry-bootstrap.md`,
  `IP-GITOPS-005-drift-detection.md`,
  `IP-GITOPS-006-secret-bootstrap-tier-b.md`,
  `IP-GITOPS-007-namespace-bootstrap-tier-b.md`,
  `IP-GITOPS-008-argocd-project-bootstrap.md`, and
  `implementation-plans/IP-seaweedfs-signed-url-substrate.md`.
- Rewritten after bucket verification caught remaining journey row-floods:
  `IP-journey-j80-cell-infra-declarative.md`,
  `IP-journey-j81-cell-infra-declarative.md`,
  `IP-journey-j83-cell-infra-declarative.md`,
  `IP-journey-j87-cell-infra-declarative.md`,
  `IP-journey-j88-cell-infra-declarative.md`,
  `IP-journey-j91-us-msb-mtl-overlay.md`,
  `IP-journey-j92-br-lgpd-us-parent-dsar.md`,
  `IP-journey-j93-in-dpdpa-rbi-overlay.md`,
  `IP-journey-j94-sox404-public-company-controls.md`,
  `IP-journey-j95-iso27001-soc2-annual-audit.md`,
  `IP-journey-j96-ksa-uae-mena-onboarding.md`,
  `IP-journey-j97-sg-pdpa-mas-tenant.md`,
  `IP-journey-j98-au-privacy-apra-cps234.md`,
  `IP-journey-j99-multi-pack-conflict-resolution.md`, and
  `IP-journey-j100-pack-rollout-first-action.md`.
- Preserved already-substantive plans: `IP-001` through `IP-015`,
  `implementation-plans/IP-seaweedfs-cluster-bootstrap.md`, and
  `implementation-plans/IP-velero-pgbackrest-restic-bootstrap.md`.
- Deleted files: none.
- Scrub rule: rewritten IPs cite only existing cloud-iac service paths and use
  counterpart refs through `cross-microservice-handoffs.md`,
  `contracts/openapi/cloud-iac.yaml`,
  `contracts/asyncapi/cloud-iac-events.yaml`, and
  `contracts/proto/cloud-iac.proto`.

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- `iac/AUDIT-FINDINGS-2026-05-18.json`
- `iac/capacity-model.md`
- `iac/runbooks/gitops-reconciler-restart.md`
- `iac/IP-001-layer-a-argocd-flux-iac.md`
- `iac/iac/helm/argocd/values.yaml`

Counterpart-fact preservations:
- `redis-ha` retained in `IP-001-layer-a-argocd-flux-iac.md` because it is the upstream Argo CD Helm chart key; backing substrate text is Valkey.
- `redis-ha` retained in `iac/helm/argocd/values.yaml` because it is the upstream Argo CD Helm chart key; backing substrate text is Valkey.
- `app=argocd-redis` retained in `runbooks/gitops-reconciler-restart.md` because it is the upstream Argo CD component label.
- `app=argocd-redis` retained in `AUDIT-FINDINGS-2026-05-18.json` excerpts because the audit quotes the same upstream Argo CD label.

Files renamed (git mv):
- None.

## WAVE-D D-1 manifest-schema consolidation audit (2026-05-21)

Audit scope: verify the 9-ADR doctrine bundle ADR-0337..0345 has landed all five required manifest-schema field blocks (`capacity_model` + `dr` + `pod_runtime_tier` + `tenant_version_pinning` + `oss_stewardship_class`) coherently in `specs/microservices/manifest-schema.json`, and verify the five companion specs exist + validate as JSON. cloud-iac owns this audit because cross-cutting schema concerns and shared OpenTofu IaC module library doctrine (ADR-0339) land in this µservice.

### Why this audit ran

D-0 (naming normalization) just landed; D-2 codex fan-out will populate per-µservice manifests next. The schema MUST be lockable BEFORE D-2 dispatch so 77 µservice authors do not race against schema drift. Each ADR author in waves A/B/C claimed to add their block; D-1 reconciles claims with the on-disk truth.

### Findings before remediation

| Field block | Authority ADR | Present at audit start? | Notes |
|---|---|---|---|
| `capacity_model` | ADR-0340 | YES (lines 588-664) | All required fields present: baseline_cpu_per_tenant + baseline_ram_per_tenant + storage_per_tenant + connections_per_tenant.{valkey,postgres,outbound_http} + scaling_dimension (closed 6-value enum) + cell_placement_class (Tier-0..Tier-4) + tenant_class_deltas + compliance_pack_overrides + notes. Sanity ceilings declared on every numeric. |
| `dr` | ADR-0343 | YES (lines 246-309) | All required fields present: rto_p99_seconds + rpo_p99_seconds + multi_region_active_active + backup_substrate (10-substrate allowlist enum) + failover_runbook (regex-pinned to `^runbooks/[A-Za-z0-9._/-]+\.md$`); optional dr_tier (T1..T4) + last_drill_evidence_id + replication_shape. |
| `pod_runtime_tier` | ADR-0338 | **MISSING** | Only mentioned in capacity_model description as a co-varying axis; the top-level field per ADR-0338 D-1.1/D-1.4 was absent. ADR-0338 author claim was unfulfilled before this audit. |
| `tenant_version_pinning` | ADR-0342 | YES (lines 665-742) | All required fields present: declared_versions + default_version + supported_window_size (≥3) + supported_window_minimum_days (≥180) + supports_per_tenant_pinning + deprecation_calendar + public_surface_files (per-version openapi/asyncapi/proto path map). |
| `oss_stewardship_class` | ADR-0345 | **MISSING (per-µservice override surface)** | The corpus-wide aggregate at /specs/oss-stewardship-registry.json exists and validates; the per-µservice manifest override surface per ADR-0345 B2.003 (and `consumes_upstream_oss` per B2.022) was absent. ADR-0345 explicitly defers per-CRATE manifest authoring to Wave 15X-OSS-stewardship (A.8.1) so the per-µservice override is the correct D-1 deliverable, not crate-level refactoring of `bounded_contexts.crates`. |

Companion specs:

| Spec | Authority ADR | Status |
|---|---|---|
| `/specs/compliance-pack-floors.json` | ADR-0343 | EXISTS + jq empty PASS |
| `/specs/oss-stewardship-registry.json` | ADR-0345 | EXISTS + jq empty PASS |
| `/specs/finops-dimensional-model.json` | ADR-0344 | EXISTS + jq empty PASS |
| `/specs/audit-event-schema.json` | ADR-0344 | EXISTS + jq empty PASS; confirmed `schema_version_log_label = "oyatie/log/v2"` at line 14 |
| `/specs/iac-module-library.json` | ADR-0339 D-8 | **MISSING** — stub created in this audit |

### Remediation actions

1. Added `pod_runtime_tier` top-level field per ADR-0338 D-1.4: integer enum [0,1,2,3] with full description citing the ADR-0248 cellular numbering co-variance + ADR-0340 capacity_model.cell_placement_class distinction.
2. Added `pod_runtime_tier_justification` (string) per ADR-0338 D-1.5/D-1.6/D-1.7/D-1.8 with note that Tier 0 / 1 / 3 declarations REQUIRE bespoke prose citing A.2 / A.3 / A.5 surfaces.
3. Added `pod_runtime_tier_surface_evidence` (array of repo-relative path strings) per ADR-0338 D-1.5/D-1.6.
4. Added `consumes_upstream_oss` (array of dep_name strings) per ADR-0345 B2.022 as the per-µservice index into the corpus-wide registry.
5. Added `oss_stewardship_class_overrides` (array of objects) per ADR-0345 B2.003 for niche per-µservice upstreams not in the corpus-wide registry. Per-entry shape mirrors `/specs/oss-stewardship-registry.json#entries` (dep_name + stewardship_class enum [maintainer, contributor, consumer] + owner_team + secondary_owner_teams + cve_sla_p0_days + cve_sla_p1_days + cve_sla_pin_update_days + contribution_budget_dev_days_per_quarter + maintainer_engineering_time_percent + audit_subscription_cost_usd + license + source_url + adr_provenance + mitigation_strategies + last_class_change_adr + first_added_adr + notes). The word "tier" is explicitly NOT used per ADR-0345 §A.6 + lane `oya-governance-stewardship-class-vocabulary`.
6. Added `iac_module_invocations` (array of objects with context + primitive + version_pin + cosign_attestation_digest + tenant_class_scope) per ADR-0339 B2.016 as the per-µservice declaration of shared module-library primitive dependencies. Context enum admits the five canonical contexts plus `oci-guest/always-free` sub-context.
7. Created `/specs/iac-module-library.json` stub per ADR-0339 D-8 catalog + discoverability contract. The stub enumerates the ~50 canonical primitives by name + purpose + status, names the five canonical contexts + the always-free sub-context, declares per-primitive required fields for Wave 15Q-IaC-modules to populate, and documents the 7-step primitive-addition protocol.

### Cross-ADR consistency gaps observed

| Gap | Observation | Resolution |
|---|---|---|
| `cell_placement_class` (ADR-0340) vs `pod_runtime_tier` (ADR-0338) | Two distinct tier axes co-vary per ADR-0340 D-6 + ADR-0338 §A.6 / B2.037. ADR-0340 uses string enum (`Tier-0`..`Tier-4`); ADR-0338 uses integer enum (0..3). | Intentional. The two axes are decoupled by design; the integer/string-enum split prevents accidental conflation. Both descriptions cite each other for traceability. |
| `dr` block (ADR-0343) vs legacy `rpo_rto` (ADR-0152) | Both blocks coexist in the schema. | Coexistence is intentional per the existing `rpo_rto` description: "rpo_rto is the legacy five-tier ergonomic; dr is the canonical two-layer matrix per ADR-0343." Migration sequenced under Wave 15W-DR-Matrix-declaration. |
| OSS stewardship vocabulary | ADR-0345 reserves "tier" for ADR-0248 cellular tiers + ADR-0338 pod runtime tiers ONLY; OSS uses "class" (Maintainer / Contributor / Consumer). | The new `oss_stewardship_class_overrides` block uses `stewardship_class` field with 3-value lowercase enum and describes the vocabulary discipline; lane `oya-governance-stewardship-class-vocabulary` is the runtime gate. |
| `iac_module_invocations` (ADR-0339) vs in-flight `iac` (ADR-0202) | The existing top-level `iac` block describes engine choice (OpenTofu vs Terraform-deprecated). The new `iac_module_invocations` block describes per-µservice module-library consumption. | Distinct concerns; both retained. The `iac.engine` field stays at the µservice level (OpenTofu-only per ADR-0218); module invocations layer on top. |

### Verification gates

```
jq empty specs/microservices/manifest-schema.json && echo PASS  -> PASS
jq empty specs/compliance-pack-floors.json && echo PASS         -> PASS
jq empty specs/oss-stewardship-registry.json && echo PASS       -> PASS
jq empty specs/finops-dimensional-model.json && echo PASS       -> PASS
jq empty specs/audit-event-schema.json && echo PASS             -> PASS
jq empty specs/iac-module-library.json && echo PASS             -> PASS
```

Schema is locked. D-2 codex fan-out for per-µservice manifest population is safe to dispatch.

### Follow-ups (D-2 and beyond)

- Wave 15S-Pod-Runtime-Tier-declaration: 77 µservices declare `pod_runtime_tier` + justification + surface_evidence per ADR-0338 D-1.
- Wave 15U-Capacity-Model-declaration: 77 µservices declare `capacity_model` per ADR-0340 D-1.
- Wave 15V-API-Versioning-declaration: every public-facing µservice declares `tenant_version_pinning` per ADR-0342 D-1.
- Wave 15W-DR-Matrix-declaration: 77 µservices declare `dr` block per ADR-0343 D-1.
- Wave 15X-OSS-stewardship: per-µservice `consumes_upstream_oss` + per-crate Maintainer SLA declaration per ADR-0345 H.3.
- Wave 15Q-IaC-modules: author ~50 module bodies + populate per-primitive contracts in `/specs/iac-module-library.json` per ADR-0339 B2.019.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

D3-BUCKET-1 updated `PRD.md` frontmatter with ADR-0338, ADR-0339, ADR-0340, ADR-0341, ADR-0342, ADR-0343, ADR-0344, and ADR-0345. ADR-0337 was not added because cloud-iac owns IaC/artifact/backup substrate, not the Iceberg OLAP warehouse write path.

### DR posture

Values: RTO 900 seconds, RPO 300 seconds, registry restore + SeaweedFS failover + quarterly restore-drill runbooks; active-active is allowed for deterministic render/validation/provenance reads but not `iac-applier` writes or state-lock ownership. ADR: ADR-0343. Alternatives considered: declare active-active apply or downgrade to pack floors; rejected because apply split-brain is more dangerous than a slower failover, and the existing PRD SLO is stricter than pack floors. Cost: manifest `dr` block still needs D-2 backfill.

### Capacity model

Values: service-unit baseline until D-2 manifest backfill; renderer/validator scale per capability request, applier/rollback are single-writer bounded, and SeaweedFS M-tier shape is 3 masters, 6 volume servers, 3 filers, 4 S3 gateways. ADR: ADR-0340. Alternatives considered: use tenant-count capacity or fabricate CPU/RAM numbers; rejected because cloud-iac load is deployment/change volume and the manifest has no capacity block. Cost: per-tenant CPU/RAM/storage/connection values remain open.

### Sustainability + cost attribution

Values: render/apply/rollback/drift audit rows emit cost, CO2, and watt-hours; carbon routing applies to render, validation, provenance lookup, backup verification, and scheduled drift, but not emergency rollback or DR restore. ADR: ADR-0344. Alternatives considered: charge all IaC cost to platform overhead; rejected because deployment choices are tenant/capability attributable. Cost: apply ledger and finops rollups must preserve capability/provider/cell axes.

### API versioning posture

Values: plan-preview, apply-state, provenance, and chart-signature validation use the YYYY-MM-DD carrier triplet; SDKs use semver; last 3 versions are supported for at least 180 days; paid/regulatory tenants can pin during audit windows; internal GitOps worker mesh is exempt. ADR: ADR-0342. Alternatives considered: keep unversioned internal-only APIs; rejected because tenant operators and automation consume these contracts outside the mesh boundary. Cost: contract registry and generated SDK release process must carry date-version metadata.

## Wave 15-doctrine-propagation-IPs (2026-05-21)

Bucket: D4-BUCKET-3.
Trigger command scope: `microservices/<service>/IP-*.md`.
IPs scanned: 38.
Trigger A matches: 23.
Trigger B matches: 13.
Trigger C matches: 4.
Trigger D matches: 0.

Manifest DR note: when `manifest.json#dr` was absent or unavailable in this checkout, DR posture sections use `specs/compliance-pack-floors.json` floors and mark manifest reconciliation as a follow-up.

IP changes:
- `iac/IP-001-layer-a-argocd-flux-iac.md`: Trigger B -> DR posture.
- `iac/IP-009-iac-rollback-engine.md`: Trigger B -> DR posture.
- `iac/IP-010-rest-surfaces.md`: Trigger A -> API Versioning.
- `iac/IP-013-sdk-and-observability-slo.md`: Trigger B -> DR posture.
- `iac/IP-014-per-pack-iac-overlays.md`: Trigger B -> DR posture.
- `iac/IP-GITOPS-001-terraform-to-opentofu-migration.md`: Trigger A -> API Versioning.
- `iac/IP-GITOPS-002-argocd-app-of-apps-pattern.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `iac/IP-GITOPS-003-tier-discipline-rollout.md`: Trigger A -> API Versioning; Trigger C -> Sustainability emission.
- `iac/IP-GITOPS-005-drift-detection.md`: Trigger A -> API Versioning; Trigger C -> Sustainability emission.
- `iac/IP-GITOPS-006-secret-bootstrap-tier-b.md`: Trigger A -> API Versioning.
- `iac/IP-GITOPS-007-namespace-bootstrap-tier-b.md`: Trigger A -> API Versioning.
- `iac/IP-GITOPS-008-argocd-project-bootstrap.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `iac/IP-journey-j100-pack-rollout-first-action.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `iac/IP-journey-j80-cell-infra-declarative.md`: Trigger A -> API Versioning.
- `iac/IP-journey-j81-cell-infra-declarative.md`: Trigger A -> API Versioning.
- `iac/IP-journey-j83-cell-infra-declarative.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `iac/IP-journey-j87-cell-infra-declarative.md`: Trigger A -> API Versioning.
- `iac/IP-journey-j88-cell-infra-declarative.md`: Trigger A -> API Versioning; Trigger C -> Sustainability emission.
- `iac/IP-journey-j91-us-msb-mtl-overlay.md`: Trigger A -> API Versioning; Trigger B -> DR posture; Trigger C -> Sustainability emission.
- `iac/IP-journey-j92-br-lgpd-us-parent-dsar.md`: Trigger A -> API Versioning.
- `iac/IP-journey-j93-in-dpdpa-rbi-overlay.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `iac/IP-journey-j94-sox404-public-company-controls.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `iac/IP-journey-j95-iso27001-soc2-annual-audit.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `iac/IP-journey-j96-ksa-uae-mena-onboarding.md`: Trigger A -> API Versioning.
- `iac/IP-journey-j97-sg-pdpa-mas-tenant.md`: Trigger A -> API Versioning; Trigger B -> DR posture.
- `iac/IP-journey-j98-au-privacy-apra-cps234.md`: Trigger A -> API Versioning.
- `iac/IP-journey-j99-multi-pack-conflict-resolution.md`: Trigger A -> API Versioning.

Unmatched IPs:
- `iac/IP-002-layer-a-opentofu-iac.md`.
- `iac/IP-003-iac-renderer-kernel.md`.
- `iac/IP-004-iac-renderer-domain-usecase.md`.
- `iac/IP-005-iac-renderer-adapter-trio.md`.
- `iac/IP-006-iac-validator-kernel-domain-usecase.md`.
- `iac/IP-007-iac-applier-kernel-domain-usecase.md`.
- `iac/IP-008-iac-registry-postgres.md`.
- `iac/IP-011-worker-binaries.md`.
- `iac/IP-012-app-composition-roots.md`.
- `iac/IP-015-hg-cloud-iac-registration.md`.
- `iac/IP-GITOPS-004-opentofu-module-registry-bootstrap.md`.

Follow-ups:
- Reconcile `manifest.json#dr` numeric service targets when the D-2 manifest DR fields land for this service.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.18 vCPU, 384 MiB RAM, 3 GB storage per active tenant; Valkey/Postgres/outbound connections 2/3/10; scaling_dimension=per_workflow_run; cell_placement_class=Tier-1.
- ADR: ADR-0340 plus ADR-0248/ADR-0340 D-6 co-variance with pod_runtime_tier=1.
- Why: Renderer/applier/validator load scales by render/apply/drift workflow runs and iac-state-index catalog growth; capacity-model.md lists p99 render/apply and drift ceilings.
- Rejected: Tier-0 was rejected because identity/key roots belong there; cloud-iac is critical substrate but not the foundation authority of record.
- Cost: Commits IaC apply-state indexes and rendered artifacts to multi-region restore drills and signed module-pin hygiene.

### Block 2: dr
- Values: RTO=900s, RPO=300s, multi_region_active_active=true, backup_substrate=postgres_wal_g+object_storage_versioned+seaweedfs_replicated+audit_chain_merkle_seal, failover_runbook=runbooks/restore-drill-quarterly.md.
- ADR: ADR-0343 and compliance-pack floors; tighter service-specific values are used where service collateral names lower targets or foundation criticality demands it.
- Why: The service owns IaC render, validation, apply, rollback, registry, and shared module-library control plane; downtime or data loss would corrupt tenant/auditor-facing state rather than only delay a background task.
- Rejected: backup-restore-cold was rejected because it cannot honor the declared p99 RTO/RPO for this service class.
- Cost: Warm regional capacity, backup-drill evidence, and audit-chain continuity are mandatory operating expenses.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=1; evidence=iac/PRD.md, iac/ARCHITECTURE.md, iac/capacity-model.md.
- ADR: ADR-0338, cross-checked against ADR-0340 cell placement Tier-1.
- Why: Shared substrate control plane: cloud-iac applies and validates cluster/IaC state for other services and can expose tenant cluster state in plan previews, so it belongs on the tenant-data-touching substrate runtime rather than default runc.
- Rejected: defaulting blindly to Tier 2 was rejected because runtime isolation must follow tenant-code, substrate, app, or edge semantics rather than service-name convention.
- Cost: RuntimeClass/nodepool placement now becomes an admission-gated contract for this service.

### Block 4: tenant_version_pinning
- Values: declared_versions=2026-05-21,2026-02-21,2025-11-21; default_version=2026-05-21; supported_window_size=3; supported_window_minimum_days=180; surfaces=openapi.
- ADR: ADR-0342.
- Why: operator APIs and apply/preview contracts need stable date versions for tenant/auditor evidence even when surfaces are internal-first.
- Rejected: unversioned v1-only behavior was rejected because tenant automation and audit replay need stable behavior across upgrades.
- Cost: Every breaking change now needs a migration document, sunset ADR, and 180-day support window.

### Block 5: consumes_upstream_oss + oss_stewardship_class_overrides
- Values: consumes_upstream_oss=opentofu,postgresql,valkey,kafka,openbao,cilium,istio,kyverno,cosign; oss_stewardship_class_overrides=[] because registry defaults are accepted for these upstreams.
- ADR: ADR-0345; classes, owners, and CVE SLAs remain centralized in specs/oss-stewardship-registry.json.
- Why: The manifest now indexes the service to the registry so SBOM, SOC2, ISO 27001, and CVE-response evidence can be generated without free-text dependency inference.
- Rejected: embedding per-dependency owner/class objects in this manifest was rejected because manifest-schema.json defines this field as dep_name strings, not local copies of registry rows.
- Cost: Any new direct upstream now needs a registry entry or an explicit local override before the service can pass the governance lane.

### Block 6: iac_module_invocations
- Values: Declared 6 shared module primitive invocations from the service's IaC context evidence; inline OpenTofu resource bodies remain a migration risk until Wave 15Q lands module bodies.
- ADR: ADR-0339.
- Why: IaC dependency on shared primitives must be machine-readable so module pins, signatures, and wrapper-thinness can be checked at admission.
- Rejected: hand-authored, per-service OpenTofu resources were rejected as the long-term target because they preserve the duplication ADR-0339 was created to remove.
- Cost: Future IaC edits must use shared module pins and keep service wrappers thin.
