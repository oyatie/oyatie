---
id: IP-N-anonymous-fold-extraction
title: "IP-N: Fold microservices/anonymous/ artifacts into microservices/community/"
status: pending
owner: community-team
date: 2026-05-21
acceptance_lane: oya-governance-anonymous-fold-complete
priority: P0
blocking: community corpus completeness; 45-µservice count correctness
references:
  - ADR-0300 (whistleblower + press-freedom doctrine)
  - ADR-0243 (Cedar universal gate)
  - ADR-0244 (tenant scoping primitive)
  - ADR-0245 (substrate vs product layering)
  - community/ARCHITECTURE.md §community-as-4-mixture-product
  - docs/architecture/transition-anonymous-to-community-2026-05-21.json
---

# IP-N: Fold `microservices/anonymous/` into `microservices/community/`

## §A — Purpose and context

### §A.1 — Background

On 2026-05-20, a Wave-3 agent mis-scaffolded `microservices/anonymous/` as a standalone
µservice, misreading §3.2.5 critical-path matrix rows 6, 7, and 27 as requiring an
independent data-plane service. The anonymous µservice accumulated 106 artifacts covering:
affinity attestation, blind-signatures crypto, post-thread storage, vote engine, abuse
classifier, legal-process workflow, retention policy, hard-delete propagation chain, REST
API, and hyperscaler-gate registration.

Per user clarification 2026-05-21, **anonymity is a posting-mode capability tier within
`microservices/community/`**, not a standalone µservice. The four posting modes
(identity-anchored, persona-anchored, pseudonymous, fully-anonymous) are all product
surfaces of the community 4-mixture product (TeamBlind + Reddit + LinkedIn + Handshake).

### §A.2 — What was already extracted (2026-05-21 fold bootstrap)

The following artifacts were created under `community/` during the initial fold on
2026-05-21:

**Cedar policy fragments (4):**
- `community/policy/anonymity-mode-identity-anchored.cedar`
- `community/policy/anonymity-mode-persona-anchored.cedar`
- `community/policy/anonymity-mode-pseudonymous.cedar`
- `community/policy/anonymity-mode-fully-anonymous.cedar`

**Capability records (7):**
- `community/capabilities/teamblind-mode.yaml`
- `community/capabilities/reddit-mode.yaml`
- `community/capabilities/linkedin-mode.yaml`
- `community/capabilities/handshake-mode.yaml`
- `community/capabilities/whistleblower-submission.yaml`
- `community/capabilities/securedrop-press-source.yaml`
- `community/capabilities/bug-bounty-submission.yaml`

**ARCHITECTURE.md §community-as-4-mixture-product** — added to
`community/ARCHITECTURE.md` explaining the 4-mixture product model and anonymity
posting-mode taxonomy.

### §A.3 — What this IP completes

The 2026-05-21 fold bootstrap handled the high-level capability surface. This IP
tracks the full artifact migration: every anonymous/ item has a community/ destination.
The work is organized into §B-§J slices, each a single-PR-sized atomic deliverable.

---

## §B — Artifact inventory and destination mapping

The following table maps every artifact from `microservices/anonymous/` to its
destination in `microservices/community/`. The anonymous/ directory was deleted on
2026-05-21; this table is the canonical extraction record.

### §B.1 — Cedar policy fragments (anonymous/policy/)

| Source artifact | Community destination | Extraction status |
|---|---|---|
| `anonymous/policy/auditor-scope.cedar` | `community/policy/auditor-scope.cedar` | PENDING — merge anonymity-specific auditor permits into existing community auditor-scope.cedar |
| `anonymous/policy/ci-scope.cedar` | `community/policy/ci-scope.cedar` | PENDING — merge CI runner permits (rewrite microservice == "anonymous" → "community") |
| `anonymous/policy/legal-process-disclosure.cedar` | `community/policy/legal-process-disclosure.cedar` | PENDING — port verbatim; update owner comment + microservice reference |
| `anonymous/policy/public-read.cedar` | `community/policy/public-read.cedar` | PENDING — merge; anonymity's "almost nothing public" model is subsumed by community's surface-specific visibility controls |
| `anonymous/policy/tenant-scope.cedar` | Absorbed into `community/policy/anonymity-mode-persona-anchored.cedar` | COMPLETE (2026-05-21 bootstrap) |
| `anonymous/policy/affinity-attestation-verification.md` | `community/policy/affinity-attestation-verification.md` | PENDING — copy + update microservice reference |
| `anonymous/policy/data-residency.md` | Merge into `community/policy/data-residency.md` | PENDING — append anonymity-mode-specific data residency constraints |

### §B.2 — Capability records (anonymous/capabilities/)

| Source artifact | Community destination | Extraction status |
|---|---|---|
| `anonymous/capabilities/T0-suggest.yaml` | `community/capabilities/T0-suggest-anonymity-mode.yaml` | PENDING — rewrite; scope to persona-anchored + pseudonymous modes only (identity-anchored has its own T0 surface) |
| `anonymous/capabilities/T1-assist.yaml` | `community/capabilities/T1-assist-anonymity-mode.yaml` | PENDING — rewrite; abuse classifier is shared across all modes; scope notes updated |
| `anonymous/capabilities/T2-auto.yaml` | `community/capabilities/T2-auto-anonymity-mode.yaml` | PENDING — rewrite; auto-moderation applies across all community modes |

### §B.3 — Implementation plans (anonymous/IP-*.md)

Each anonymous IP must be rewritten with community ownership. The blinding/attestation
IPs become community sub-IPs under the anonymity-mode slice.

| Source artifact | Community destination | Extraction status |
|---|---|---|
| `anonymous/IP-001-iac-bootstrap.md` | Merge into `community/IP-001-postgres-citus-post-store-iac.md` or new `community/IP-016-anonymity-mode-iac-bootstrap.md` | PENDING |
| `anonymous/IP-002-cargo-workspace-kernels.md` | `community/IP-017-anonymity-mode-cargo-kernels.md` | PENDING |
| `anonymous/IP-003-domain-crates-per-bc.md` | `community/IP-018-anonymity-mode-domain-crates.md` | PENDING |
| `anonymous/IP-004-postgres-adapters-blinding-migration.md` | `community/IP-019-blinding-migration-postgres-adapters.md` | PENDING |
| `anonymous/IP-005-valkey-cache.md` | Merge with `community/IP-006-voting-engine.md` valkey section | PENDING |
| `anonymous/IP-006-affinity-attestation-bc.md` | `community/IP-020-affinity-attestation-bc.md` | PENDING |
| `anonymous/IP-007-blind-signatures-crypto.md` | `community/IP-021-blind-signatures-crypto.md` | PENDING |
| `anonymous/IP-008-post-store-bc.md` | Absorbed into `community/IP-002-post-store-kernel-domain.md` | PENDING — review overlap; append anonymity-mode-specific fields |
| `anonymous/IP-009-vote-engine-bc.md` | Absorbed into `community/IP-006-voting-engine.md` | PENDING — append blinded-vote-token section |
| `anonymous/IP-010-abuse-classifier-wire.md` | `community/IP-022-abuse-classifier-wire.md` | PENDING |
| `anonymous/IP-011-legal-process-workflow.md` | `community/IP-023-legal-process-workflow.md` | PENDING |
| `anonymous/IP-012-retention-policy-worker.md` | `community/IP-024-retention-policy-worker.md` | PENDING |
| `anonymous/IP-013-hard-delete-propagation-chain.md` | Absorbed into community's hard-delete work; new `community/IP-025-hard-delete-propagation-chain.md` | PENDING |
| `anonymous/IP-014-rest-api-openapi-sdk.md` | Merge with `community/IP-003-post-store-usecase-api.md` anonymity-mode endpoints | PENDING |
| `anonymous/IP-015-hg-anonymous-registration-branch-protection.md` | `community/IP-026-hg-community-anonymity-mode-gate.md` (rename HG gate to community scope) | PENDING |

### §B.4 — Catalog records (anonymous/catalog/)

All catalog records require crate-slug rewrite: `oya-anonymous-*` → `oya-community-*-anonymity-mode-*`.

| Source artifact | Community destination | Crate rename |
|---|---|---|
| `anonymous/catalog/oya-anonymous-affinity-attestation-adapter-oidc.yaml` | `community/catalog/oya-community-affinity-attestation-adapter-oidc.yaml` | PENDING |
| `anonymous/catalog/oya-anonymous-affinity-attestation-adapter-saml.yaml` | `community/catalog/oya-community-affinity-attestation-adapter-saml.yaml` | PENDING |
| `anonymous/catalog/oya-anonymous-affinity-attestation-kernel.yaml` | `community/catalog/oya-community-affinity-attestation-kernel.yaml` | PENDING |
| `anonymous/catalog/oya-anonymous-blind-signatures-adapter-ring.yaml` | `community/catalog/oya-community-blind-signatures-adapter-ring.yaml` | PENDING |
| `anonymous/catalog/oya-anonymous-blind-signatures-adapter-rust-bls.yaml` | `community/catalog/oya-community-blind-signatures-adapter-rust-bls.yaml` | PENDING |
| `anonymous/catalog/oya-anonymous-blind-signatures-kernel.yaml` | `community/catalog/oya-community-blind-signatures-kernel.yaml` | PENDING |
| `anonymous/catalog/oya-anonymous-content-moderation-adapter-foundry-runtime.yaml` | `community/catalog/oya-community-content-moderation-adapter-foundry-runtime.yaml` | PENDING |
| `anonymous/catalog/oya-anonymous-feed-timeline-adapter-meilisearch.yaml` | Merge with existing community feed catalog | PENDING |
| `anonymous/catalog/oya-anonymous-feed-timeline-adapter-valkey.yaml` | Merge with existing community feed catalog | PENDING |
| `anonymous/catalog/oya-anonymous-legal-process-disclosure-adapter-workflow-engine.yaml` | `community/catalog/oya-community-legal-process-disclosure-adapter-workflow-engine.yaml` | PENDING |
| `anonymous/catalog/oya-anonymous-post-thread-adapter-postgres.yaml` | Merge with existing `community/catalog/` post-thread entries | PENDING |
| `anonymous/catalog/oya-anonymous-post-thread-kernel.yaml` | Merge with existing community post-thread kernel | PENDING |
| `anonymous/catalog/oya-anonymous-pseudonymous-identity-adapter-postgres.yaml` | `community/catalog/oya-community-pseudonymous-identity-adapter-postgres.yaml` | PENDING |
| `anonymous/catalog/oya-anonymous-pseudonymous-identity-domain.yaml` | `community/catalog/oya-community-pseudonymous-identity-domain.yaml` | PENDING |
| `anonymous/catalog/oya-anonymous-pseudonymous-identity-kernel.yaml` | `community/catalog/oya-community-pseudonymous-identity-kernel.yaml` | PENDING |
| `anonymous/catalog/oya-anonymous-pseudonymous-identity-usecase.yaml` | `community/catalog/oya-community-pseudonymous-identity-usecase.yaml` | PENDING |
| `anonymous/catalog/oya-anonymous-retention-policy-worker.yaml` | `community/catalog/oya-community-retention-policy-worker-anonymity-mode.yaml` | PENDING |
| `anonymous/catalog/oya-anonymous-vote-engine-adapter-valkey.yaml` | Merge with existing community vote-engine catalog | PENDING |

### §B.5 — SLO records (anonymous/slos/)

| Source artifact | Community destination | Extraction status |
|---|---|---|
| `anonymous/slos/abuse-classifier-latency.openslo.yaml` | Merge into `community/slos/moderation-action-latency.openslo.yaml` (anonymity-mode label added) | PENDING |
| `anonymous/slos/affinity-attestation-verify-latency.openslo.yaml` | `community/slos/affinity-attestation-verify-latency.openslo.yaml` | PENDING |
| `anonymous/slos/anonymity-correlation-resistance.openslo.yaml` | `community/slos/anonymity-correlation-resistance.openslo.yaml` | PENDING |
| `anonymous/slos/content-policy-enforcement-correctness.openslo.yaml` | Merge with community existing moderation correctness SLO | PENDING |
| `anonymous/slos/feed-render-latency.openslo.yaml` | Merge with `community/slos/feed-render-latency.openslo.yaml` (already exists) | PENDING |
| `anonymous/slos/hard-delete-propagation-correctness.openslo.yaml` | `community/slos/hard-delete-propagation-correctness.openslo.yaml` | PENDING |
| `anonymous/slos/legal-process-disclosure-chain-of-custody-correctness.openslo.yaml` | `community/slos/legal-process-disclosure-chain-of-custody-correctness.openslo.yaml` | PENDING |
| `anonymous/slos/post-create-latency.openslo.yaml` | Merge with `community/slos/post-create-latency.openslo.yaml` (already exists) | PENDING |
| `anonymous/slos/vote-action-latency.openslo.yaml` | Merge with `community/slos/vote-cast-latency.openslo.yaml` (already exists) | PENDING |

### §B.6 — Runbooks (anonymous/runbooks/)

| Source artifact | Community destination | Extraction status |
|---|---|---|
| `anonymous/runbooks/abuse-classifier-rollback.md` | `community/runbooks/abuse-classifier-rollback.md` | PENDING |
| `anonymous/runbooks/affinity-attestation-key-rotation.md` | `community/runbooks/affinity-attestation-key-rotation.md` | PENDING |
| `anonymous/runbooks/anonymity-leak-incident-response.md` | `community/runbooks/anonymity-mode-leak-incident-response.md` | PENDING — rename to reflect mode scope |
| `anonymous/runbooks/blind-signature-key-ceremony.md` | `community/runbooks/blind-signature-key-ceremony.md` | PENDING |
| `anonymous/runbooks/employer-affinity-employer-domain-takeover.md` | `community/runbooks/employer-affinity-domain-takeover.md` | PENDING |
| `anonymous/runbooks/geo-affinity-cluster-rebalance.md` | `community/runbooks/geo-affinity-cluster-rebalance.md` | PENDING |
| `anonymous/runbooks/hard-delete-tombstone-corruption.md` | `community/runbooks/hard-delete-tombstone-corruption.md` | PENDING |
| `anonymous/runbooks/legal-process-court-order-receipt.md` | `community/runbooks/legal-process-court-order-receipt.md` | PENDING |

### §B.7 — Dashboards (anonymous/dashboards/)

| Source artifact | Community destination | Extraction status |
|---|---|---|
| `anonymous/dashboards/anonymity-health.json` | `community/dashboards/anonymity-mode-health.json` | PENDING |
| `anonymous/dashboards/moderation-and-safety.json` | Merge with `community/dashboards/moderation-queue-depth.json` | PENDING |
| `anonymous/dashboards/retention-and-deletion-correctness.json` | `community/dashboards/retention-and-deletion-correctness.json` | PENDING |

### §B.8 — Decisions (anonymous/decisions/)

The anonymous-scoped ADRs (ADR-ANON-0001..0007) are now cross-referenced under
community. They are not moved verbatim because they carry `microservice: anonymous`
scope — instead, they are superseded-and-referenced via community's decisions registry.

| Source ADR | Community action |
|---|---|
| `ADR-ANON-0001-cryptographic-blinding-protocol.md` | Reference from community decisions README; superseded by community's affinity-attestation IP |
| `ADR-ANON-0002-affinity-attestation-verification.md` | Reference from community IP-020-affinity-attestation-bc.md |
| `ADR-ANON-0003-legal-process-disclosure-workflow.md` | Reference from community IP-023-legal-process-workflow.md |
| `ADR-ANON-0004-retention-and-deletion-policy.md` | Reference from community IP-024-retention-policy-worker.md |
| `ADR-ANON-0005-abuse-classifier-bounds.md` | Reference from community IP-022-abuse-classifier-wire.md |
| `ADR-ANON-0006-federation-refusal-and-anti-pattern-anchoring.md` | Reference from community ARCHITECTURE.md §federation |
| `ADR-ANON-0007-affinity-cluster-design.md` | Reference from community IP-020-affinity-attestation-bc.md |

### §B.9 — IaC (anonymous/iac/)

| Source artifact | Community destination | Extraction status |
|---|---|---|
| `anonymous/iac/helm/Chart.yaml` | Merge anonymity-mode Helm component into community Helm chart | PENDING |
| `anonymous/iac/helm/values.yaml` | Merge anonymity-mode values into community Helm values | PENDING |
| `anonymous/iac/helm/templates/deployment.yaml` | Merge as separate deployment for affinity-attestation + blind-sig components | PENDING |
| `anonymous/iac/helm/templates/hpa.yaml` | Merge into community HPA config | PENDING |
| `anonymous/iac/helm/templates/networkpolicy.yaml` | Merge; anonymity-mode network policy scoped to affinity-attestation BC | PENDING |
| `anonymous/iac/helm/templates/pdb.yaml` | Merge into community PDB | PENDING |
| `anonymous/iac/helm/templates/prometheusrule.yaml` | Merge anonymity-mode PrometheusRules into community rules | PENDING |
| `anonymous/iac/helm/templates/service.yaml` | Merge affinity-attestation service into community Helm | PENDING |
| `anonymous/iac/helm/templates/servicemonitor.yaml` | Merge into community ServiceMonitor | PENDING |
| `anonymous/iac/kustomize/base/kustomization.yaml` | Merge into community kustomize base | PENDING |
| `anonymous/iac/kustomize/base/namespace.yaml` | No migration needed — community namespace already exists | SKIP |
| `anonymous/iac/kustomize/overlays/pack-eu/kustomization.yaml` | Merge EU pack overlay into community pack-eu overlay | PENDING |
| `anonymous/iac/kustomize/overlays/pack-eu/patch-pack-eu-region.yaml` | Merge | PENDING |
| `anonymous/iac/kustomize/overlays/pack-kr/kustomization.yaml` | Merge KR pack overlay into community pack-kr overlay | PENDING |
| `anonymous/iac/kustomize/overlays/pack-kr/patch-pack-kr-region.yaml` | Merge | PENDING |

### §B.10 — Strategic and operational docs (anonymous/ root-level)

| Source artifact | Community action | Extraction status |
|---|---|---|
| `anonymous/PRD.md` | Extract anonymity-mode user stories into community PRD §anonymity-mode appendix | PENDING |
| `anonymous/ARCHITECTURE.md` | Content absorbed into community ARCHITECTURE.md §community-as-4-mixture-product | COMPLETE (2026-05-21) |
| `anonymous/PHASE-01-ANONYMOUS-FOUNDATION.md` | Reference from community PHASE docs as historical note | PENDING |
| `anonymous/threat-model.md` | Merge anonymity-specific threat actors into community threat-model.md | PENDING |
| `anonymous/dpia.md` | Merge anonymity-mode DPIA into community dpia.md §anonymity-mode | PENDING |
| `anonymous/compliance.md` | Merge anonymity-mode compliance overlays into community compliance.md | PENDING |
| `anonymous/capacity-model.md` | Merge affinity-attestation + blind-sig throughput math into community capacity-model.md | PENDING |
| `anonymous/cost-budget.md` | Merge anonymity-mode cost line items into community cost-budget.md | PENDING |
| `anonymous/competitor-parity-matrix.md` | Merge TeamBlind parity rows into community competitor-parity-matrix.md | PENDING |
| `anonymous/sdk-plan.md` | Merge anonymity-mode SDK surface into community sdk-plan.md | PENDING |
| `anonymous/failure-modes.md` | Merge blinding-key and affinity-attestation failure modes into community failure-modes.md | PENDING |
| `anonymous/incident-response.md` | Merge anonymity-leak IR playbook into community incident-response.md | PENDING |
| `anonymous/multi-region.md` | Merge affinity-attestation multi-region config into community multi-region.md | PENDING |
| `anonymous/backfill-replay.md` | Merge blinded-credential backfill notes into community backfill-replay.md | PENDING |
| `anonymous/manifest.json` | Crates from anonymous/manifest.json added to community/manifest.json (crate rename required) | PENDING |
| `anonymous/AUDIT-FINDINGS-2026-05-18.json` | Move to `evidence/anonymous-fold/AUDIT-FINDINGS-2026-05-18.json` for historical reference | PENDING |
| `anonymous/scorecards/overrides.json` | Merge overrides into community scorecards/overrides.json | PENDING |
| `anonymous/contracts/openapi/anonymous.yaml` | Merge anonymity-mode endpoints into community OpenAPI contract | PENDING |
| `anonymous/contracts/asyncapi/anonymous-events.yaml` | Merge anonymity-mode events into community AsyncAPI contract | PENDING |
| `anonymous/contracts/proto/anonymous.proto` | Merge anonymity-mode RPCs into community proto (or standalone anonymity_mode.proto imported by community) | PENDING |

---

## §C — Acceptance criteria

This IP is complete when ALL of the following hold:

1. `! [ -d microservices/anonymous ]` — directory deleted (COMPLETE 2026-05-21)
2. `ls community/policy/anonymity-mode-*.cedar` returns 4 files (COMPLETE 2026-05-21)
3. `ls community/capabilities/{teamblind,reddit,linkedin,handshake,whistleblower,securedrop,bug-bounty}*.yaml` returns 7 files (COMPLETE 2026-05-21)
4. `community/IP-N-anonymous-fold-extraction.md` exists (this file — COMPLETE 2026-05-21)
5. All §B.1–§B.10 rows marked COMPLETE (PENDING — follow-on PRs)
6. `grep -rl "microservices/anonymous/" --include="*.md"` returns NO results outside of
   historical/transition documents
7. `community/manifest.json` updated with oya-community-*-anonymity-mode-* crate roster
8. `docs/architecture/transition-anonymous-to-community-2026-05-21.json` present and valid JSON
9. `docs/standards/documentation-rigor.md` §1 corpus snapshot updated: 46 → 45
10. `oya-governance-anonymous-fold-complete` lane green

---

## §D — Sliced delivery plan

### Slice 1 (bootstrapped 2026-05-21 — this session)
- Cedar policy fragments × 4
- Capability records × 7
- ARCHITECTURE.md §community-as-4-mixture-product
- This IP file
- Corpus-wide mention sweep (category B + C)
- Keystone-bundle doc updates (46 → 45)
- Directory deletion

### Slice 2 — Cedar policy merges
- Merge `auditor-scope.cedar`, `ci-scope.cedar`, `legal-process-disclosure.cedar`, `public-read.cedar`
- Acceptance: `community/policy/legal-process-disclosure.cedar` present and linted

### Slice 3 — Capability record completions
- `T0-suggest-anonymity-mode.yaml`, `T1-assist-anonymity-mode.yaml`, `T2-auto-anonymity-mode.yaml`
- Acceptance: 10 capability records total under community/capabilities/

### Slice 4 — IP migrations (affinity-attestation + blind-sig)
- IP-016 through IP-021 (IaC bootstrap, cargo kernels, domain crates, blinding migration, affinity attestation BC, blind-signatures crypto)
- Acceptance: each IP has ≥150 lines + §A–§J sections

### Slice 5 — IP migrations (moderation + legal-process + retention + hard-delete)
- IP-022 through IP-025
- Acceptance: each IP has ≥150 lines + §A–§J sections; cross-references to ADR-ANON-0003/0004/0005

### Slice 6 — Catalog record rewrites (18 records → oya-community-*-anonymity-mode-*)
- Acceptance: no `oya-anonymous-*` slug in community/catalog/

### Slice 7 — SLO merges (9 records → community/slos/)
- Acceptance: `community/slos/anonymity-correlation-resistance.openslo.yaml` present;
  `community/slos/legal-process-disclosure-chain-of-custody-correctness.openslo.yaml` present

### Slice 8 — Runbook migrations (8 runbooks → community/runbooks/)
- Acceptance: all 8 runbooks present under community/runbooks/ with community ownership header

### Slice 9 — Dashboard migrations (3 dashboards → community/dashboards/)
- Acceptance: all 3 dashboards present under community/dashboards/

### Slice 10 — Operational doc merges (PRD, threat-model, DPIA, compliance, capacity, cost, contracts)
- Acceptance: `community/threat-model.md` contains §anonymity-mode section;
  `community/dpia.md` contains §anonymity-mode section;
  `community/manifest.json` updated with oya-community-*-anonymity-mode-* crates

---

## §E — Risk register

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Duplicate Cedar policy fragments (anonymous + community both define same action) | Medium | High | §F policy-dedup validation: run Cedar policy linter across all community/policy/*.cedar |
| Blinded-credential crate name collision (oya-anonymous-* vs oya-community-*) | Low | Medium | Cargo workspace rename in Slice 6; no runtime impact until crates are built |
| SLO target drift (anonymous SLO targets ≠ community SLO targets for same action) | Medium | Medium | Slice 7 uses higher of the two targets; conflicts escalated to ops-security |
| Legal-process workflow ADR-ANON-0003 lost from cross-reference | Low | High | ADR-ANON-0003 referenced from community/IP-023 and community/policy/legal-process-disclosure.cedar |
| Anonymous AUDIT-FINDINGS-2026-05-18.json lost entirely | Low | High | Moved to evidence/anonymous-fold/ in Slice 10 |

---

## §F — Validation gates per slice

Each slice PR must pass:

1. `cargo clippy --workspace -- -D warnings` (no new Rust warnings)
2. `grep -l "microservices/anonymous" community/**/*.{md,yaml,cedar,json}` returns empty (for slices that touch community/ docs)
3. Cedar policy lint on `community/policy/` (no duplicate entity-type definitions)
4. `python3 -c "import json; json.load(open('docs/architecture/transition-anonymous-to-community-2026-05-21.json'))"` (JSON validity)
5. `oya-governance-anonymous-fold-complete` lane advisory (BLOCKER once acceptance criteria §C items 5–10 complete)

---

## §G — Dependency ordering

```
Slice 1 (done) → Slice 2 (Cedar merges) → Slice 3 (capability completions)
                ↓
              Slice 4 (affinity/blind-sig IPs) → Slice 6 (catalog rewrites)
                ↓                                        ↓
              Slice 5 (moderation IPs)          Slice 7 (SLO merges)
                ↓
              Slice 8 (runbooks) → Slice 9 (dashboards) → Slice 10 (ops docs)
```

Slices 2, 4, and 8 can parallelize with each other. Slice 6 depends on Slice 4 crate
names being finalized. Slice 10 must be last (absorbs manifest.json).

---

## §H — Owner and team

- **IP owner:** community-team (axis-community)
- **Security review:** ops-security (Cedar policy correctness; legal-process-disclosure)
- **Privacy review:** council-privacy (affinity attestation; DPIA merge)
- **Legal review:** general-counsel (legal-process workflow; whistleblower dual-control)
- **Architecture review:** council-architecture (substrate deduplication; ADR-0245 compliance)

---

## §I — Intern-buildability checklist

An intern with Rust + YAML + Cedar experience and zero prior architecture knowledge
MUST be able to execute each slice from this IP alone. Per §D, each slice has:

- **What:** explicit artifact list
- **Where:** destination path for every artifact
- **Acceptance gate:** concrete `ls` or `grep` command that verifies completion
- **Validation:** §F gates applicable to that slice

If any acceptance gate is ambiguous, file a comment on this IP before starting the slice.

---

## §J — Cross-references

- `community/ARCHITECTURE.md §community-as-4-mixture-product` — product surface model
- `community/policy/anonymity-mode-*.cedar` × 4 — Cedar gates per posting mode
- `community/capabilities/*.yaml` × 7 — capability records
- `docs/architecture/transition-anonymous-to-community-2026-05-21.json` — classification artifact
- `docs/standards/documentation-rigor.md` — §1 corpus snapshot (45 µservices)
- `docs/architecture/keystone-bundle-2026-05-20-synthesis.md §11` — fold noted
- `docs/architecture/corpus-rigor-audit-2026-05-20.md §-Update-2026-05-21` — fold noted
- `ADR-0300` — whistleblower + press-freedom doctrine (binding for fully-anonymous mode)
- `ADR-0243` — Cedar universal gate
- `ADR-0244` — tenant scoping primitive
- `ADR-0245` — substrate vs product layering (rationale for fold)
- `ADR-ANON-0001..0007` — anonymous-scoped ADRs (retained as historical reference in git history)

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-N-anonymous-fold-extraction.md` matched `openapi, asyncapi, .proto`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-N-anonymous-fold-extraction.md` matched `SLO, multi-region`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/community/IP-N-anonymous-fold-extraction.md` matched `cost`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
