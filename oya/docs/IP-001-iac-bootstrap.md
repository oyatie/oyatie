---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-docs-foundation
impl_plan_id: IP-001-iac-bootstrap
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-docs + ops-sre-reliability
acceptance_lanes: [helm-lint, kubectl-apply-dry-run, oya-governance-per-microservice-layout, oya-governance-version-pinning-conformance]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-001: IaC bootstrap — Helm + Kustomize for Postgres + S3 + Valkey + ClamAV + OPSWAT + gVisor

## Intent

Author Helm + Kustomize manifests for the docs µservice substrate. Postgres 16 LTS for document metadata (per-tenant + per-block RLS per ADR-DOCS-0004); S3-compatible object storage for content blobs + attachments (per-tenant prefix; Object Lock for legal-hold); Valkey 8.1 (RESP3 wire-compatible) cluster mode for collab presence + CRDT op spool + cache; ClamAV scanner (default); OPSWAT MetaDefender (pack-us-healthcare overlay); gVisor pool for export workers per ADR-DOCS-0003. Pack-aware overlays for 11 packs.

## ChangeSet boundary

10 Helm template files + Kustomize base + per-pack overlay (pack-kr + pack-eu first; us/jp/sg/au/in/br/ae/ksa/us-healthcare follow). No Rust code; pure IaC + values. All secrets via `${openbao:secret/docs/...}` SecretReferences.

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/docs/iac/helm/Chart.yaml` | created in this ChangeSet | dependencies: postgres 16.4, valkey 8.1 (RESP3 wire-compatible), ClamAV 1.3, OPSWAT MetaDefender container 5.x, gVisor 2024-Q4 LTS |
| `microservices/docs/iac/helm/values.yaml` | created | per-BC replica sizing; gVisor pool size; OpenBao SecretReferences |
| `microservices/docs/iac/helm/templates/deployment.yaml` | created | per-BC Deployment (8 BCs) |
| `microservices/docs/iac/helm/templates/service.yaml` | created | per-BC Service |
| `microservices/docs/iac/helm/templates/hpa.yaml` | created | per-BC HPA (CPU 70%; min 5 max 100) |
| `microservices/docs/iac/helm/templates/pdb.yaml` | created | PodDisruptionBudget min-available 50% |
| `microservices/docs/iac/helm/templates/networkpolicy.yaml` | created | mesh-only ingress; egress to OpenBao + Postgres + Valkey + S3 + ClamAV + cross-µservice mTLS for embed-resolver |
| `microservices/docs/iac/helm/templates/servicemonitor.yaml` | created | Prometheus scrape config |
| `microservices/docs/iac/helm/templates/prometheusrule.yaml` | created | per-BC fast-burn + slow-burn alert rules |
| `microservices/docs/iac/kustomize/base/kustomization.yaml` | created | shared base |
| `microservices/docs/iac/kustomize/overlays/pack-kr/kustomization.yaml` | created | initial active pack |
| `microservices/docs/iac/kustomize/overlays/pack-eu/kustomization.yaml` | created | EU pack (eIDAS + EU AI Act overlays) |
| (additional packs: us, us-healthcare, jp, sg, au, in, br, ae, ksa) | successor-IP | per-pack overlays |

## Acceptance Gates

```bash
helm lint microservices/docs/iac/helm
kubectl --dry-run=client apply -k microservices/docs/iac/kustomize/overlays/pack-kr
buck2 build //:quality-lane-registry-authority-check # lane=per-microservice-layout --microservice docs
buck2 build //:quality-lane-registry-authority-check # lane=version-pinning-conformance
```

## Test Plan

- helm lint + helm-test per chart against kind/k3d cluster.
- E2E smoke: spin kind cluster; apply pack-kr overlay; verify all 8 BC deployments + Postgres + S3 emulator + Valkey + ClamAV reach Ready within 10 min.
- gVisor sandbox smoke: spawn an export job; verify tmpfs-only + no network egress.

## Halt Conditions

- Upstream chart version drifts past LTS pin — escalate per `docs/standards/observability-slo.md`.
- OpenBao secret-reference resolution fails — block.
- Helm chart fails kubectl-dry-run — root-cause; do not mask.

## Next IP

[`IP-002-document-store-kernel.md`](IP-002-document-store-kernel.md)

## References

- ADR-0117 (data residency); ADR-0131 (per-µservice flat layout); ADR-0133.
- ADR-DOCS-0001 (Loro CRDT); ADR-DOCS-0003 (export pipeline backends).
- Postgres CloudNativePG operator — `cloudnative-pg.io`.
- Valkey cluster mode — `valkey.io/topics/cluster-tutorial/`.
- ClamAV — `clamav.net`.
- OPSWAT MetaDefender — `opswat.com/products/metadefender`.
- gVisor — `gvisor.dev`.

## Wave 15-IP-substance conversion (2026-05-21)
This addendum converts the short implementation note into a buildable docs-service slice for IaC bootstrap; it does not rely on line count as proof.
Counterpart anchors: Google Docs, Microsoft Word Online, Notion, Coda, Quip, GitHub.

### Problem closed
The gap is not generic documentation infrastructure. `docs` owns collaborative document authoring where rich blocks, CRDT edits, comments, suggestions, version history, sharing, export/import, embeds, and AI assist must work under one tenant and policy model.
For IaC bootstrap, the implementation must preserve dual-context separation, per-block ACL, legal hold, audit-chain records, and export/import safety described in `microservices/docs/PRD.md` and `microservices/docs/ARCHITECTURE.md`.

### Concrete mechanism
Primary artifact: `microservices/docs/iac/`.
Domain entities or operational surfaces: Postgres, S3-compatible blobs, Valkey, ClamAV/OPSWAT, gVisor export workers.
Contracts and gates: `microservices/docs/contracts/openapi/docs.yaml`, `microservices/docs/contracts/asyncapi/docs-events.yaml`, `microservices/docs/contracts/proto/docs.proto`, and Cedar files under `microservices/docs/policy/`.
SLO/runbook evidence: `microservices/docs/slos/*.openslo.yaml`, `microservices/docs/dashboards/*.json`, and `microservices/docs/runbooks/*.md`.

### Implementation steps
1. Bind IaC bootstrap to the matching bounded context in `microservices/docs/manifest.json` and the catalog row named in this addendum.
2. Confirm every command/event carries tenant_id, principal_id, document context, data_class, traceparent, idempotency key for mutations, and audit_event_class.
3. Extend the contract or catalog artifact only where the cited file already owns that surface; do not invent a sibling µservice or fake Terraform module.
4. Add or update policy checks in `tenant-scope.cedar`, `editor-isolation.md`, `public-read.cedar`, or `auditor-scope.cedar` according to the feature's access path.
5. Add tests around accepted path, cross-tenant denial, stale version/anchor handling, legal-hold or retention interaction, and export/import rollback where applicable.
6. Attach SLO, dashboard, and runbook evidence before promotion so a reviewer can verify more than the presence of this Markdown file.

### Acceptance and counterpart comparison
| Counterpart | Expected behavior | Oyatie closure |
|---|---|---|
| Google Docs / Microsoft Word Online | Native collaborative authoring with comments, sharing, history, and export. | `IaC bootstrap` must be first-party, tenant-scoped, auditable, and legal-hold aware. |
| Notion / Coda | Block-centric documents with embeds and structured workflows. | `IaC bootstrap` must preserve block ACL, embed policy checks, and cross-service refresh evidence instead of hidden workspace coupling. |
| Quip / GitHub | Team review and change history expectations. | `IaC bootstrap` must expose signed audit events, version/revert evidence, and contract-rendered developer docs. |

### Evidence
- `microservices/docs/PRD.md`
- `microservices/docs/ARCHITECTURE.md`
- `microservices/docs/manifest.json`
- `microservices/docs/competitor-parity-matrix.md`
- `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md`
- `docs/decisions/ADR-0324-anti-script-anti-template-doctrine.md`
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`
