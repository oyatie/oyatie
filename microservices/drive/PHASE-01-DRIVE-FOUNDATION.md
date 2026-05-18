---
doc_class: PhasePlan
template_id: TPL-PHASE-PLAN
microservice: drive
phase_id: PHASE-01
phase_title: Drive Foundation — file-store + folder-hierarchy + upload + download + sync + share-link + permissions + search-index + preview + dlp-virus-scan + immutability-tier
status: Accepted
date: 2026-05-17
owner_team: axis-drive
related_adrs: [ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-DRIVE-0001, ADR-DRIVE-0002, ADR-DRIVE-0003, ADR-DRIVE-0004, ADR-DRIVE-0005, ADR-DRIVE-0006]
doc_status: published
---

# PHASE-01 — Drive Foundation

## Intent

Stand up the eleven bounded contexts (file-store, folder-hierarchy, upload, download, sync, share-link, permissions, search-index, preview, dlp-virus-scan, immutability-tier) with full Layer-A + Layer-B substrate, S3-SigV4 + WebDAV (RFC 4918) + tus 1.0 conformance, dual-context isolation, audit-chain emission, WORM immutability, and SLO-gated promotion. Phase exit = AC-01 through AC-17 in `PRD.md` green.

## Phase scope

In-scope:
- 89 crates per the layer mapping table.
- Postgres metadata schema + per-tenant RLS + tenant-DEK envelope encryption (operational metadata only; bytes encrypted upstream of S3).
- Valkey upload-session in-flight + delta-sync cache.
- S3-compatible object store (Garage primary edge-distributed deployment; SeaweedFS secondary single-cluster deployment; SeaweedFS for archive tier) per ADR-DRIVE-0001.
- FastCDC content-defined-chunking (ADR-DRIVE-0002).
- Argon2id + Ed25519 share-link signing (ADR-DRIVE-0003).
- Tenant-DEK envelope via OpenBao Transit (ADR-DRIVE-0004); client-side E2E via libsodium secretstream (opt-in Personal pillar).
- Meilisearch full-text + Apache Tika extract pipeline.
- libvips (image) + qpdf + Mozilla pdf.js (PDF) + LibreOffice in gVisor (Office) + ffmpeg (video) preview pipeline per ADR-DRIVE-0005.
- ClamAV + OPSWAT MetaDefender virus-scan pipeline; in-tree DLP rules + foundry-runtime ML handoff.
- WORM immutability tier with object-lock semantics per ADR-DRIVE-0006.
- Workflow events produced + consumed per `PRD.md`.
- Ontology writes + reads per `PRD.md`.
- HG-DRIVE hyperscaler-maturity claim registered per ADR-0123 + ADR-0133.

Out-of-scope (scheduled-for-distinct-tracked-work):
- Cross-pack file replication (M04-onward).
- Post-quantum crypto roadmap (M05-onward; reflect MEGA's roadmap).
- Native co-authoring real-time collaboration (handled by docs/sheets/slides; drive only owns bytes).
- ML-based smart organisation auto-folder (T2 capability scheduled-for-distinct-tracked-work to subsequent-to-GA-tier-promotion).

## Phase outputs

| Output | Path | Owner |
|---|---|---|
| 89 crates | `microservices/drive/src/crates/oya-drive-*` | axis-drive |
| Postgres schema migrations | `microservices/drive/iac/helm/postgres/migrations/` | axis-drive |
| Helm charts | `microservices/drive/iac/helm/{garage,minio,seaweedfs,postgres,redis,meilisearch,tika,clamav,opswat,libreoffice}` | ops-sre-reliability |
| Kustomize overlays | `microservices/drive/iac/kustomize/{base,overlays/pack-kr,overlays/pack-eu,...}` | ops-sre-reliability |
| OpenAPI / AsyncAPI / Proto contracts | `microservices/drive/contracts/` | axis-drive |
| Cedar policies | `microservices/drive/policy/*.cedar` | ops-security |
| Runbooks | `microservices/drive/runbooks/*.md` | ops-sre-reliability |
| Dashboards | `microservices/drive/dashboards/*.json` | axis-observability |
| HG-DRIVE claim entry | `registry/hyperscaler-maturity-claims.json` | axis-drive |

## Phase milestones (ChangeSets, per ADR-0110)

| CS | Title | DAG-position | Slice |
|---|---|---|---|
| CS-01 | file-store kernel + domain + usecase + api | Layer-B base | A |
| CS-02 | file-store -adapter-postgres + -adapter-s3 + -adapter-garage + -adapter-seaweedfs (object backends) | depends CS-01 | A |
| CS-03 | file-store rest + worker + sdk + app | depends CS-02 | A |
| CS-04 | folder-hierarchy kernel..app (8 crates) | depends CS-01 | B |
| CS-05 | upload kernel..adapter-redis + adapter-s3 + rest + worker + app (10 crates) | depends CS-01 + CS-02 | B |
| CS-06 | download kernel..adapter-s3 + rest + app (8 crates) | depends CS-01 + CS-02 | B |
| CS-07 | sync kernel..worker + sdk + app (10 crates) | depends CS-01 | C |
| CS-08 | share-link kernel..worker + app (10 crates) | depends CS-01 | C |
| CS-09 | permissions kernel..rest + app (8 crates) | depends CS-01 + CS-04 | C |
| CS-10 | search-index kernel..adapter-meilisearch + adapter-tika + rest + worker + app (10 crates) | depends CS-01 + foundry-runtime | D |
| CS-11 | preview kernel..adapter-libvips + adapter-qpdf + adapter-libreoffice + adapter-ffmpeg + rest + worker + app (12 crates) | depends CS-01 | D |
| CS-12 | dlp-virus-scan kernel..adapter-clamav + adapter-opswat + worker + app (9 crates) | depends CS-01 | D |
| CS-13 | immutability-tier kernel..worker + app (8 crates) | depends CS-01 | E |
| CS-14 | Cedar policy + DPIA + threat-model sign-off | depends CS-01..CS-13 | F |
| CS-15 | OpenAPI + AsyncAPI + Proto contracts + capabilities | depends CS-01..CS-13 | F |
| CS-16 | Helm + Kustomize + dashboards + runbooks | depends CS-01..CS-13 | F |
| CS-17 | HG-DRIVE maturity-claim entry + SLO manifests + canary cohort weighting | depends all | F |

## Phase gate

Phase-exit gate (per ADR-0139): all 17 AC-IDs green; SLO eligibility verdict `eligible` for `drive` µservice over `dev → staging` window; reviewer-agent APPROVE on each ChangeSet; per-changeset evidence committed at `microservices/drive/evidence/multispectrum/*.json`.

## Risks + mitigations

| Risk | Mitigation |
|---|---|
| S3 SigV4 conformance edge cases (presigned URLs; multipart copy edge cases) | Adopt `s3-tests` (Ceph public suite) as conformance corpus; 100% pass before GA |
| WebDAV client diversity (macOS Finder, Windows Explorer, davfs2, Cyberduck, Nextcloud sync client) | E2E test against five real clients in staging |
| Office-preview sandbox escape | gVisor isolation per ADR-DRIVE-0005; no network + no host filesystem; rasterise to PNG; macro execution refused |
| WORM violation by tenant-root | object-lock implemented in domain layer; AC-09 ensures even tenant-root is refused; periodic integrity scan compares hold-set vs storage layer; mismatch alerts |
| Delta-sync chunk-boundary drift | FastCDC normalised-bounds enforced per ADR-DRIVE-0002; corpus tests on rolling-hash boundaries |
| ClamAV false-positive on tenant business-critical file | quarantine + tenant-policy review path (per `runbooks/dlp-quarantine-release.md`); OPSWAT multi-engine verdict for high-stakes packs |
| Object-store backend failure (Garage cell loss) | replication-factor 3; rebuild from neighbour cells; runbook `object-storage-degraded.md` |
| Share-link takeover (signing-key compromise) | Ed25519 + HKDF; per-link key rotation; revocation cascade per `runbooks/share-link-takeover-incident.md` |
