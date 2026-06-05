---
doc_class: PhaseSpec
milestone: M03-sheets-preview
phase: P01-sheets-foundation
status: Active
entry_gate: |
  PRD-sheets accepted; ADR-0135 net-new microservice scope accepted; Buck2 workspace and Cargo
  metadata are present for Rust ecosystem compatibility; cloud cell substrates expose Postgres,
  Valkey, object storage, Arrow/Parquet, audit-chain, tenancy, ontology, and workflow integration
  seams.
exit_gate: |
  Sheets product slice has Buck2-owned build/test/check/coverage evidence, Prow/Kubernetes-native
  oya-ci-required status, CUE/KRM desired-state validation, multispectrum evidence, and product
  operation-ledger entries for any tenant/workbook/cell/shard migration drill. GitHub checks may
  mirror this during the temporary lane-unlocker period but do not replace Buck2/Prow authority.
depends_on:
  - milestone: M01-foundation
    phase: P01-agentic-slo-gated-promotion
    reason: sheets SLO promotion gate must exist before sheets itself can advance past dev
  - milestone: M02b-substrate-ready
    phase: P01-cell-substrate
    reason: cell substrate must be live before Sheets persists cell rows to it
owner_team: axis-sheets + council-design-system
related_adrs: [ADR-0065, ADR-0135, ADR-0139, ADR-0131, ADR-0140, ADR-0513]
related_specs: [/specs/microservices/sheets.json, /specs/per-microservice-flat-layout.json]
date: 2026-05-17
doc_status: published
---

# P01-sheets-foundation: Land the Sheets microservice end-to-end

## Purpose

This plan keeps the Sheets product scope while removing obsolete local CLI and bridge-authority procedures. Sheets remains a hero product surface: collaborative grid editing, formula/recalc engine, CRDT collaboration, import/export, named ranges, charts, conditional formatting, pivot tables, comments, version history, AI formula assistance, connected-sheets queries, embed integrations, workflow triggers, and Cedar/PBAC/ABAC authorization.

## Current authority

- Buck2 is the build, test, check, benchmark, and LLVM source-based coverage authority.
- Cargo files are maintained for Rust ecosystem metadata, editor support, reindeer/vendor flow, and dual-build compatibility; they are not the merge gate authority.
- Prow plus Kubernetes-native oya-ci-required jobs own CI evidence.
- GitHub pull requests and GitHub Actions are temporary lane-unlocker publication/shadow surfaces.
- CUE/KRM desired-state reconciliation owns Kubernetes intent; compatibility packaging is generated only where an adapter requires it.
- Product control-plane operations and signed operation-ledger events own replay, backfill, sharding, rollback, and incident drills.

## Lane boundaries

| Lane | Owns | Shared-surface rule |
|---|---|---|
| Product core | grid model, formula/recalc, CRDT, import/export, charting, named ranges | Keep product docs inside `oya/sheets`; update root pointers only when a new canonical entrypoint is needed. |
| Policy/security | Cedar/PBAC/ABAC, per-range ACL, sandboxed import, AI prompt boundaries | Keep policy packs in service-owned paths and register them through machine-readable indexes. |
| Cloud/runtime | cell placement, network policy, workload identity, runtime isolation, service mesh | Keep CUE/KRM desired state in cell/runtime substrates; Sheets references the registry rather than copying manifests. |
| CI/evidence | Buck2 targets, Prow jobs, multispectrum evidence | Add target-specific evidence files; avoid editing global CI docs for service-only changes. |

## Verification checklist

- Buck2 targets cover changed Rust crates, policy fixtures, CUE/KRM fixtures, docs hygiene, benchmark harnesses, and coverage reports.
- Prow oya-ci-required consumes Buck2 evidence.
- Runtime plans enforce workload identity, default-deny network policy, restricted container privileges, immutable file systems, dropped Linux capabilities, disabled default service-account token automount, and service-mesh mTLS where applicable.
- Replay/backfill drills require two-person approval, signed operation-ledger evidence, audit-chain seals, and tenant-visible diff summaries where state may change.
- Benchmarks are reproducible only after a Buck2-owned harness target exists; historical benchmark tables stay marked as model inputs until then.

## Product backlog slices

1. Formula/recalc conformance with Excel/LibreOffice reference corpus.
2. CRDT collaboration no-silent-loss drill with concurrent workbook edits.
3. XLSX/ODS/CSV/TSV/JSON-Sheet import/export best-effort fidelity lane.
4. Per-range ACL and Cedar/PBAC/ABAC preview lane.
5. AI formula and smart-fill prompt-boundary lane.
6. Replay/backfill control-plane operation lane.
7. Sharding automation lane: autosharding, auto-rebalance, hot-split, cold-merge.
8. Runtime hardening lane: workload identity, network segmentation, sandboxed import, and pod runtime isolation.

## Evidence shape

Each lane writes a multispectrum evidence file containing:

- branch, base SHA, changed surfaces, and affected Buck2 targets;
- Prow status or local Buck2 output when the lane has not yet entered CI;
- operation-ledger event IDs for any replay, backfill, sharding, rollback, or incident drill;
- non-claims that explicitly exclude live mutation when the evidence is static-only.

## Cutover note

Native SCM/CI/CD remains the destination. GitHub is kept only to unlock concurrent lanes until the native substrate is ready. Future edits should add service-local evidence and registry pointers rather than broad shared-doc rewrites.
