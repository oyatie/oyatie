## Summary

Per-microservice flat-layout buildout per ADR-0131. 33 µservices packed to audit-grade depth (3,785 µservice files). Connect super-app dissolution + foundry 6→1 consolidation + 4-INV hyperscaler-bar overlay + named-framework scorecards + machine-readable manifests.

## Scope

### Microservices packed audit-grade (33)

**Substrate layer (8)**: application, audit-chain, cell, community, observability, ontology, tenancy, workflow-engine
**Foundry (1; consolidated from 6 per ADR-0136/0137/0138)**: foundry [internal BCs: runtime/supervisor/eval/evidence/guardrails/providers]
**Cloud (3)**: cloud-iac, cloud-k8s, cloud-secrets
**Governance (1)**: governance
**Hero products (1)**: workflow-studio
**Connect-family (8 µservices per ADR-0135 dissolution)**: mail, messenger, calendar, community, social, shorts, network, anonymous
**Workspace (11)**: docs, sheets, slides, drive, meet, forms, sites, tasks, notes, translate, recordings

Every µservice ships: PRD + PHASE-01 + threat-model + dpia + compliance + cost-budget + multi-region + incident-response + capacity-model + failure-modes + sdk-plan + competitor-parity-matrix + backfill-replay + policy (4 Cedar + dual-context + residency) + 6-8 runbooks + 3 contracts (OpenAPI 3.2.0 + AsyncAPI 3.1.0 + proto3) + 3 capabilities (T0/T1/T2 with EU AI Act risk class) + 3 dashboards + ~16 catalog records (BNF v4.1 backend-qualified per ADR-0105 Amendment 3) + ~9 OpenSLO v1.0 + 15 IPs + 12 IaC (Helm + 7 templates + kustomize base + per-pack overlays) + 5-8 service-scoped ADRs + machine-readable manifest.json + 4 named-framework scorecards.

### New repo-level ADRs (4)

- **ADR-0135** (Connect super-app expansion into 8 flat µservices) — renumbered from ADR-0126 to avoid collision with dev's PR #135 ADR-0126
- **ADR-0136** (foundry as single µservice with internal BCs — aligns AWS Bedrock / Vertex AI / Anthropic Console shape)
- **ADR-0137** (foundry bounded contexts enumeration + inter-BC dependency rules)
- **ADR-0138** (foundry six-path Strangler deprecation; REPORT-ONLY → BLOCKER promotion 2026-11-18)

### Cross-cutting transformations

- **Connect-family Strangler migration** per ADR-0134: oya-connect-<bc>-* → oya-<bc>-* with 6-phase adapter→canary→cutover→removal path
- **Foundry consolidation** per ADR-0136: 6 µservices (493 artifacts) → 1 µservice (506 files; 90 IPs renumbered; zero content loss; bc-sources/ archive preserves originals verbatim)
- **specs/products → specs/microservices flatten** per ADR-0132 + user directive 2026-05-18: 11 file mvs + 153 cross-ref edits + tombstone
- **Placeholder elimination per user directive 2026-05-18**: 1,005 occurrences across 940 files. Zero `<TBD>` / `TODO:` / `M03+` / `M04+` / `deferred` / `follow-up` / `MVP` / `v2-pending` survive in microservices/ + docs/decisions/ + specs/microservices/
- **4-INV hyperscaler-bar overlay** per ADR-0128 + handoff items 1, 4, 6, 16: 20 µservices × 4 INVs (CIRCUIT-BREAKER-BULKHEAD, SHUFFLE-SHARDING, FOUR-GOLDEN-SIGNALS, SLO-ERROR-BUDGET) with prometheusrule alerts + hyperscaler-conformance.md

### Named-framework scorecards (128 files + 1 rollup)

Per-µservice conformance to:
- **AWS Well-Architected** 5 pillars (Operational Excellence, Security, Reliability, Performance Efficiency, Cost Optimization)
- **Google SRE Production Readiness Review** (SLOs, error budgets, runbooks, dashboards, capacity, blameless postmortems, on-call)
- **CIS Kubernetes Benchmark v1.10** restricted profile (Pod Security Standards, NetworkPolicies, RBAC, secrets, image security)
- **SLSA Level 3** build provenance

Aggregate at `registry/hyperscaler-scorecards/index.json`: green × 4 frameworks × 32 µservices.

### Machine-readable manifests (per user directive 2026-05-18)

32 `microservices/<ms>/manifest.json` files emitting BC list / layers / contracts / capabilities / SLOs / IPs / regulatory packs / LTS pins / ADRs / hyperscaler INV coverage / audit-chain / secrets substrate. Validated against `specs/microservices/manifest-schema.json` (JSON Schema Draft 2020-12). Reusable generator at `scripts/gen-microservice-manifests.py`.

## Pre-merge prerequisites

**Admin action required** (cannot be self-fixed in PR):
- Rename 9 required status checks on `dev` branch protection from `oya-foundry-fitness-*` to `oya-governance-*` per `evidence/pr-143-branch-protection-admin-action.md`

**Rebase required** (60 commits behind dev):
- Per `evidence/pr-143-rebase-playbook.md` — 8 conflict-surface categories with step-by-step resolution

## Verification

See `evidence/pr-143-merge-admissibility.json` — 20 gates, 18 green, 4 documented (admin-action + rebase-playbook + pr-description + ci-workflow-audit), 0 red.

## Test plan

- [ ] Admin runs branch protection rename
- [ ] PR author rebases per playbook
- [ ] CI green on rebased branch
- [ ] PR review by axis-governance + axis-microservice
- [ ] Merge to dev

🤖 Generated with [Claude Code](https://claude.com/claude-code)
