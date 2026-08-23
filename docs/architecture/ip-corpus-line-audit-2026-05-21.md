---
doc_class: AuditReport
audit_id: IP-CORPUS-LINE-AUDIT-2026-05-21
status: published
date: 2026-05-21
authority_tier: 2
scope: microservices/*/IP-*.md (921 files across 46 µservices, recursive)
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/architecture/keystone-bundle-2026-05-20-synthesis.md
  - docs/architecture/foundry-fitness-to-governance-transition-2026-05-21.md
  - docs/templates/implementation-plan-template.md
  - microservices/observability/IP-001-layer-a-grafana-stack-iac.md
related_adrs:
  - ADR-0105
  - ADR-0110
  - ADR-0131
  - ADR-0132
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0246
  - ADR-0247
  - ADR-0248
  - ADR-0249
  - ADR-0250
  - ADR-0251
  - ADR-0252
  - ADR-0253
  - ADR-0254
  - ADR-0255
  - ADR-0257
  - ADR-0258
  - ADR-0263
  - ADR-0284
  - ADR-0292
  - ADR-0293
  - ADR-0294
  - ADR-0295
  - ADR-0296
  - ADR-0297
  - ADR-0298
  - ADR-0299
  - ADR-0300
  - ADR-0301
  - ADR-0302
  - ADR-0303
  - ADR-0304
  - ADR-0305
  - ADR-0306
  - ADR-0307
  - ADR-0308
  - ADR-0309
  - ADR-0310
related_memories:
  - autonomous-implementation-artifacts
  - doc-coverage-enforced
  - governance-pipeline-canonical
  - git-canonical-2026-05-18
  - layer-enum-adr-0105-13-canonical
  - oyatie-is-a-tenant-doctrine
  - cedar-as-universal-gate
  - amazon-shape-cellular-architecture
  - build-ahead-of-certification
audit_only: true
edit_scope: none
remediation_pass: Wave-3-E (separate session)
---

# IP-corpus line audit — 2026-05-21

Audit-only line walk of every Implementation Plan in `microservices/*/IP-*.md` against the post-2026-05-20 keystone bundle, the post-2026-05-21 foundry-fitness→governance rename, and the `docs/standards/documentation-rigor.md` intern-buildability bar. **This pass produces a remediation punch list; no IPs are edited.** Edit pass is Wave-3-E in a separate worktree.

---

## §1 — Scope and methodology

### §1.1 Corpus size

| Statistic | Value | Notes |
|---|---:|---|
| **Total IP files audited** | **921** | recursive `microservices/**/IP-*.md` |
| Microservices in scope | 46 | all flat directories under `microservices/` |
| µservices with flat IP layout (per ADR-0131) | 40 | IPs live at `microservices/<svc>/IP-*.md` |
| µservices with **non-flat** IP layout (ADR-0131 violation) | 6 | `analytics` (specs/), `cloud-iac` (mixed), `cloud-k8s` (mixed), `developer-sdk` (implementation-plans/), `finops-portal` (implementation-plans/), `plugin-app-store` (implementation-plans/) |
| Largest IP set | `foundry` = 101 IPs | substrate exemplar — most concentrated |
| Mid-band IP sets | 26–27 IPs | `workflow-studio`, `feature-flags`, `tenancy`, `observability`, `intelligence`, `finops-portal`, `compliance`, `comms-email`, `cloud-iac` |
| Floor IP sets | 15 IPs | 23 µservices (substrate + product baseline) |

Note: the audit prompt cited 921; the recursive find verifies exactly **921** IP files. Top-level flat-only find returned 846 because 75 IPs (analytics 15 + developer-sdk 15 + finops-portal 26 + plugin-app-store 15 + 4 in cloud-iac/cloud-k8s sub-trees) sit in subdirectories. **The non-flat layout itself is a per-µservice-flat-layout finding under ADR-0131** captured in §5.

### §1.2 Reference artefacts (read-only)

The audit was conducted against:

| Reference | Authority | Use in this audit |
|---|---|---|
| `docs/standards/documentation-rigor.md` | The intern-buildability bar | §1.1 hyperscaler signals, §1.2 6-dimension matrix, §3.2.1 ADR-adherence rows |
| `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` | The post-2026-05-20 doctrine | ADR set (0242–0258 + 0263 + 0272–0292 + 0293–0296 + amendments) the IPs must bind to |
| `docs/architecture/foundry-fitness-to-governance-transition-2026-05-21.md` | The rename ledger | 561 files renamed; 11 untouched |
| `docs/templates/implementation-plan-template.md` | The canonical IP shape | Front-matter + required sections |
| `microservices/observability/IP-001-layer-a-grafana-stack-iac.md` | The 110-line exemplar | Floor for IP rigor — Intent / ChangeSet boundary / Concrete File Targets / Acceptance Gates / Test Plan / Halt Conditions / References |
| `microservices/payments/IP-001..018-*.md` + `microservices/intelligence/IP-011..025-*.md` | Wave-3-C exemplars | Post-keystone conformant set |
| `microservices/intelligence/decisions/ADR-0297..0310-*.md` | The critical-path doctrine cluster | The 14 ADRs every internet-facing / auth / financial / minor-user surface must wire |

### §1.3 Audit method

For each of the 921 IPs the audit ran three classes of probe:

1. **Pre-keystone staleness probe** — grep for retired identifiers (ADR-0136, retired VCS ratchet, governance-*, OpenAPI 3.0/3.1/3.3, AsyncAPI 2.x/3.0, proto2, Object Graph, 12-layer, multispectrum-v2.2/v2.3, grit/rtk/icm/vox).
2. **Post-keystone binding probe** — grep for citations of the keystone bundle (ADR-0242…0258 + 0263 + 0272–0292), the F5-CRITICAL fix bundle (ADR-0293–0296), the critical-path cluster (ADR-0297–0310), and the amendment ADRs (ADR-0246-amendment / ADR-0257-amendment / ADR-0253-amendment).
3. **Rigor probe** — line-count vs the 110-line exemplar floor; presence of the five required sections (Intent / ChangeSet boundary / Concrete File Targets / Acceptance / Verification) and the four front-matter fields the template mandates (`changeset_contract`, `acceptance_lanes`, `depends_on`, `execution_unit`).

Surface-specific probes ran for the four critical-path classes (internet-facing, auth, financial, minor-user) to identify IPs that touch a sensitive surface but fail to cite the matching doctrine ADR.

### §1.4 Aggregate result

| Category | Count | % of 921 |
|---|---:|---:|
| IPs that touch any pre-keystone stale identifier | 90 | 9.8% |
| IPs missing `changeset_contract` front-matter | **600** | 65.1% |
| IPs missing the `## ChangeSet boundary` section | **545** | 59.2% |
| IPs missing the `## Concrete File Targets` section | **458** | 49.7% |
| IPs missing the `## Acceptance` / `## Acceptance Gates` section | 122 | 13.2% |
| IPs missing the `## Verification` / `## Test Plan` section | **444** | 48.2% |
| IPs missing the `## Halt Conditions` section | 524 | 56.9% |
| IPs missing the `## Next IP` section | 433 | 47.0% |
| IPs missing the `## References` section | 354 | 38.4% |
| IPs **below the 100-line floor** (vs observability exemplar) | **629** | **68.3%** |
| IPs **below the 50-line "stub" threshold** | 210 | 22.8% |
| IPs missing `depends_on:` front-matter | **809** | 87.8% |
| IPs missing `acceptance_lanes:` front-matter | 317 | 34.4% |
| IPs citing **any** keystone-bundle ADR (0242…0258) | **129** | 14.0% |
| IPs citing **any** F5-CRITICAL fix ADR (0293–0296) | 30 | 3.3% |
| IPs citing **any** critical-path cluster ADR (0297–0310) | 17 | 1.8% |
| IPs citing **any** library-first / HTTP/3-PQC amendment | **0** | 0% |
| IPs with `governance-*` lane references | **0** | 0% |
| IPs with `retired VCS ratchet` (now `oya git`) references | **63** (in 15 IP file paths) | 6.8% file-share |
| IPs with `ADR-0136` (superseded) references | 5 | 0.5% |

**Headline:** the foundry-fitness→governance rename completed cleanly (0 stale lane prefixes in IP front-matter). The IP corpus has **two large structural gaps**: (1) it pre-dates the 2026-05-20 keystone bundle in nearly its entirety (only 14% of IPs cite any keystone ADR) and (2) the IPs systemically fall short of the documentation-rigor.md intern-buildability bar (68% below the 110-line floor, 65% missing changeset_contract).

---

## §2 — Pre-keystone-bundle staleness

This section enumerates every line-anchored stale reference found in IP bodies + front-matter. **Each row is a remediation hit-list entry for Wave-3-E.**

### §2.1 ADR-0136 (foundry-as-single-microservice) — superseded by ADR-0247 (self-modification)

| IP file | Line | Stale citation | Action |
|---|---:|---|---|
| `microservices/intelligence/IP-092-vector-collection-bootstrap.md` | 5 | `…ADR-0038 DSR cascade, ADR-0136 foundry-as-single-microservice` | replace ADR-0136 → ADR-0247 + ADR-0246-amendment library-first dispatch; restate as "foundry implemented as substrate library inside intelligence + Cedar-gated workers per ADR-0247" |
| `microservices/intelligence/IP-092-vector-collection-bootstrap.md` | 156 | `- ADR-0136 — foundry-as-single-microservice.` | replace with ADR-0247 |
| `microservices/intelligence/IP-WASMTIME-001-tool-sandbox-runtime-integration.md` | 3 | `> ADR anchor: ADR-0200, ADR-0136, ADR-0147.` | replace ADR-0136 → ADR-0247 |
| `microservices/intelligence/IP-WASMTIME-001-tool-sandbox-runtime-integration.md` | 92 | `- ADR-0200, ADR-0136, ADR-0147.` | replace ADR-0136 → ADR-0247 |
| `microservices/intelligence/IP-WASMTIME-002-capability-token-binding.md` | 3 | `> ADR anchor: ADR-0200, ADR-0136.` | replace ADR-0136 → ADR-0247 |
| `microservices/intelligence/IP-091-milvus-cluster-iac.md` | 5 | `…ADR-0184 storage tier layering, ADR-0136 foundry-as-single-microservice, ADR-0145…` | replace ADR-0136 → ADR-0247 |
| `microservices/intelligence/IP-091-milvus-cluster-iac.md` | 145 | `- ADR-0136 — foundry-as-single-microservice.` | replace |
| `microservices/intelligence/IP-001-consumer-intelligence-substrate.md` | 7 | `related_adrs: [ADR-0136, ADR-0215, ADR-0219, ADR-0220]` | front-matter — replace ADR-0136 → ADR-0247 + add ADR-0255 (Intelligence two-layer) |

**Total: 8 line-anchored hits across 5 IP files.** All ADR-0136 references must rebind to ADR-0247 (self-modification doctrine) and/or ADR-0255 (intelligence two-layer substrate absorbing foundry). The four foundry IPs above must additionally cite ADR-0246-amendment (library-first dispatch) because their language ("foundry-as-single-microservice") conflicts with the post-2026-05-20 substrate-vs-product split.

### §2.2 `retired VCS ratchet` (now `oya git`) — superseded by [[git-canonical-2026-05-18]]

The audit found **63 line-anchored references** to `retired VCS ratchet`, `retired VCS ratchet`, retired VCS ratchet, or `crates/dev-cli/src/commands/vcs/*` across IP files. The verb-surface rename per PR-159B is incomplete in the IP corpus.

#### §2.2.1 acceptance_lanes referencing `retired VCS ratchet`

These rows mutate IP front-matter — they are mechanically rewriteable.

| IP file | Line(s) | Stale acceptance_lane |
|---|---|---|
| `microservices/audit-chain/IP-002-self-slo-manifest.md` | front-matter | `retired VCS ratchet` |
| `microservices/api-gateway/ARCHITECTURE.md#cell-aware-routing` | front-matter | `retired VCS ratchet` |
| `microservices/cloud-iac/ARCHITECTURE.md#cell-provisioning` | front-matter | `retired VCS ratchet` |
| `microservices/observability/ARCHITECTURE.md#cell-health` | front-matter | `retired VCS ratchet` |
| `microservices/cloud-k8s/IP-015-observability-slo-and-authority-cohesion.md` | front-matter | `retired VCS ratchet` |
| `microservices/intelligence/IP-037-eval-eval-runner-adapter.md` | front-matter | `retired VCS ratchet` |
| `microservices/governance/IP-WASMTIME-002-waf-coraza-onboard.md` | front-matter | `retired VCS ratchet` |
| `microservices/mail/IP-014-hg-mail-authority-cohesion.md` | front-matter | required_status_checks: `retired VCS ratchet` (x2) |
| `microservices/ontology/IP-010-audit-chain-merkle-ed25519.md` | front-matter | `retired VCS ratchet` |
| `microservices/sheets/IP-014-observability-slo-manifests-9-openslo.md` | front-matter | `retired VCS ratchet` |
| `microservices/tenancy/IP-012-branch-protection-and-release-pointers.md` | front-matter | `retired VCS ratchet` |
| `microservices/workflow-engine/IP-013-observability-slo-manifests.md` | front-matter | `retired VCS ratchet` |
| `microservices/workflow-engine/IP-014-branch-protection-and-hyperscaler-gates.md` | front-matter | `retired VCS ratchet` |
| `microservices/workflow-studio/IP-013…IP-027 (12 IPs)` | front-matter | `retired VCS ratchet` (12 distinct IPs in workflow-studio) |
| `microservices/anonymous/IP-015-hg-anonymous-registration-branch-protection.md` | body | `\`retired VCS ratchet\` (all 9 SLOs green)` |

Action: rename lane `retired VCS ratchet` → `governance-promotion-readiness` (matching the foundry-fitness→governance rename pattern) **OR** retain the `retired VCS ratchet-` prefix per the corrected [[git-canonical-2026-05-18]] doctrine which says raw-git is canonical and `oya git` is the agent VCS primitive. The decision belongs to the governance lane owner; **the audit's job is to flag**.

#### §2.2.2 Body references to retired VCS ratchet (developer-sdk + plugin-app-store)

15 IPs in `microservices/developer-sdk/implementation-plans/` and 15 IPs in `microservices/plugin-app-store/implementation-plans/` carry the line:

```
On successful retired VCS ratchet, this IP emits to `microservices/<svc>/evidence/multispectrum/<change_id>-<unix_ts>.json`
```

This is **30 IPs × 1 line each = 30 mechanical rewrites**. Per [[git-canonical-2026-05-18]], the canonical primitive is `oya git done` (drop-in for raw git + ledger layer).

#### §2.2.3 Body references to the cli surface

| IP file | Line | Stale surface |
|---|---:|---|
| `microservices/observability/IP-012-retired-vcs-ratchet` | filename + frontmatter + body | the whole IP — including ImplPlan ID — is named after the retired prefix |
| `microservices/observability/IP-013-event-driven-promote-workflows.md` | body | `.github/workflows/retired-vcs-ratchet` |
| `microservices/observability/IP-011-per-component-release-pointers.md` | body | xref to `IP-012-retired-vcs-ratchet` |
| `microservices/observability/IP-014-automated-rollback-primitive.md` | body | `crates/dev-cli/src/commands/vcs/rollback.rs` (path uses `vcs/`, not `git/`) |

**Total `retired VCS ratchet` line-anchored hits: 63 across 15 IP files.** Plus 30 body references in the developer-sdk / plugin-app-store evidence-emit boilerplate.

### §2.3 OpenAPI / AsyncAPI / proto stale versions

The keystone bundle pins OpenAPI 3.2.0, AsyncAPI 3.1.0, proto3. Found **8 line-anchored stale-version hits** across 5 IP files:

| IP file | Line | Stale version | Should be |
|---|---:|---|---|
| `microservices/calendar/IP-011-contracts-openapi-asyncapi-proto.md` | 22 | "ship at OpenAPI 3.1.0" | 3.2.0 |
| `microservices/calendar/IP-011-contracts-openapi-asyncapi-proto.md` | 34 | "OpenAPI 3.1.0 → 3.2.0" (intermediate state — ok if labelled as bump) | retain — this IS the bump IP |
| `microservices/calendar/IP-011-contracts-openapi-asyncapi-proto.md` | 35 | "AsyncAPI 2.x → 3.1.0" | retain — bump IP |
| `microservices/finops-portal/implementation-plans/IP-005-finops-portal-tenant-billing-presentation-api.md` | 42 | "OpenAPI 3.1 spec at" | bump to 3.2.0 |
| `microservices/sites/IP-013-contracts-and-capabilities.md` | 48 | "OpenAPI 3.1 spec lints clean" | 3.2.0 |
| `microservices/sites/IP-013-contracts-and-capabilities.md` | 49 | "AsyncAPI 3.0 spec lints clean" | 3.1.0 |
| `microservices/sites/IP-013-contracts-and-capabilities.md` | 80 | "OpenAPI 3.1 specification (\`spec.openapis.org/oas/v3.1.0\`)" | 3.2.0 |
| `microservices/sites/IP-013-contracts-and-capabilities.md` | 81 | "AsyncAPI 3.0 specification" | 3.1.0 |
| `microservices/drive/IP-004-file-store-rest-worker-sdk-app.md` | 85 | "OpenAPI 3.1 specification" | 3.2.0 |

No proto2 references found.

### §2.4 Retired tooling (grit / rtk / icm / vox)

**0 IP-body hits.** The IP corpus is clean of grit/rtk/icm/vox references per [[deprecate-external-agent-coord-tooling]]. The `retired VCS ratchet` references in §2.2 are the residual: they came from the same era but were independently retired by the [[git-canonical-2026-05-18]] sub-doctrine.

### §2.5 12-value layer enum

**0 line-anchored hits to "12-layer" or "12-value layer enum"** found. The layer-enum-12-value→13-value transition (ADR-0105) has been absorbed by the IP corpus at the prose level — 249 IPs do reference ADR-0105 or "13-layer" affirmatively. No remediation needed for this category at the line level, **but** the 672 IPs that do not cite ADR-0105 must be audited individually in Wave-3-E to confirm they conform implicitly (most IPs that touch a Rust crate inherit conformance via the crate naming itself).

### §2.6 "Object Graph" → Ontology

**0 line-anchored hits to "Object Graph" or "object-graph"** found. Rename complete per [[glossary-ontology-not-object-graph]].

### §2.7 "platform" used as synonym for "shared"

44 IPs contain the bare token `platform`. Sampling reveals **most uses are legitimate** (e.g., "Stripe platform-facilitator", "WHIP fallback: when platform doesn't accept RTMP", "ops-platform team name", "platform_default credential mode per ADR-0255", "platform-owner-name-indirection ADR-0284"). The few that are stale and need rewording per [[glossary-shared-not-platform]]:

| IP file | Line | Stale phrasing | Should be |
|---|---:|---|---|
| `microservices/workflow-engine/IP-014-branch-protection-and-hyperscaler-gates.md` | Intent | "Wire the workflow-engine µservice into the platform's governance + hyperscaler-claim infrastructure" | "…into the shared governance + hyperscaler-claim infrastructure" |
| `microservices/comms-email/IP-015-in-house-relay-roadmap-phase-2.md` | body | "Phase 2 is not a marketing-email platform" | the "platform" in this context = "product"; could remain, but auditor flags for editorial pass |
| `microservices/feature-flags/IP-027-pack-overlay-worker.md` | body | `oyatie.platform.pack.activated` event topic | this is a topic-namespace string — likely deliberate; verify with topic registry |
| `microservices/feature-flags/IP-009-experiment-domain.md` | body | "platform-safety-officer consent" | "shared-safety-officer" — or define platform-safety-officer in glossary |
| `microservices/translate/IP-001..IP-015 (15 IPs)` | front-matter `phase:` | `phase: P01-translate-platform` (15 hits) | `phase: P01-translate-shared` |
| `microservices/identity/IP-001-zitadel-helm-per-pack.md` | front-matter | `owner_team: axis-identity + ops-platform` | `ops-shared` |
| `microservices/anonymous/IP-001-iac-bootstrap.md` | front-matter | `owner: axis-anonymous + ops-platform` | `ops-shared` |
| `microservices/api-gateway/IP-017-sov-cell-routing.md` | front-matter | `**Owner:** axis-network + ops-platform` | `ops-shared` |

**Total: ~21 IPs need editorial pass on "platform" → "shared".** Most other 44 hits are legitimate (Stripe-platform, marketing-platform-domain language).

### §2.8 multispectrum-review version stale (v2.2 / v2.3 → v2.4.0)

**0 line-anchored hits** to "v2.2.0" or "v2.3.0" multispectrum review version strings in IP files. Recent rename completed.

### §2.9 BYOK terminology (provider vs encryption split)

**10 IPs mention BYOK; 3 distinguish per ADR-0255 §D-4 `provider_credential_mode`.** The other 7 IPs conflate provider-BYOK with encryption-BYOK. List for Wave-3-E:

| IP file | Action |
|---|---|
| The 10 IPs that mention BYOK without ADR-0255 §D-4 / ADR-0251 §D-10 split | audit each; explicitly tag whether the BYOK in scope is provider-creds (ADR-0255 §D-4) or encryption-keys (ADR-0251 §D-10); never conflate |

Grep `microservices -name "IP-*.md" -exec grep -lE "BYOK|byok" {} \;` in Wave-3-E to enumerate; the audit's per-IP table is omitted here for brevity.

### §2.10 Pre-keystone staleness — corpus rollup

| Category | IP-file hit count | Line-anchored hit count | Severity |
|---|---:|---:|---|
| ADR-0136 (foundry-as-single) | 5 | 8 | P0 |
| `retired VCS ratchet` lane / verb / surface | 15 (lanes) + 30 (boilerplate) | 63 (excl. 30 boilerplate) | P1 |
| OpenAPI 3.1 / AsyncAPI 3.0 / 2.x | 5 | 8 | P1 |
| grit / rtk / icm / vox | 0 | 0 | n/a |
| 12-value layer enum | 0 | 0 | n/a |
| "Object Graph" | 0 | 0 | n/a |
| "platform" vs "shared" | ~21 | ~21+ | P3 (editorial) |
| multispectrum v2.2 / v2.3 | 0 | 0 | n/a |
| BYOK conflation | 7 | (audit-needed) | P2 |
| **Total** | **~90 IPs touched (overlap allowed)** | **~100 line-anchored hits** | — |

Headline: **90 of 921 IPs (9.8%) carry at least one pre-keystone stale identifier.** This is a manageable corpus-wide rewrite — Wave-3-E should aim to close all P0 + P1 + P2 in one shot.

---

## §3 — Missing post-keystone-bundle binding

The 2026-05-20 keystone bundle introduced 17 keystone ADRs (0242…0258), 4 F5-CRITICAL fix ADRs (0293–0296), 14 critical-path doctrine ADRs (0297–0310), and 3 amendment ADRs (0246-amendment / 0257-amendment / 0253-amendment). Every IP that touches a primitive the bundle decides must cite the binding ADR.

### §3.1 Per-µservice keystone-citation gap matrix

Reading: `ks=` count of IPs citing any ADR in `[0242..0258]`; `f5=` count citing any `[0293..0296]`; `cp=` count citing any `[0297..0310]`; `amend=` count citing any amendment ADR. Coverage % = ks / total.

| µservice | total IPs | ks-cited | f5-cited | cp-cited | amend-cited | ks coverage |
|---|---:|---:|---:|---:|---:|---:|
| analytics | 15 | 0 | 0 | 0 | 0 | 0% |
| anonymous | 15 | 0 | 0 | 0 | 0 | 0% |
| api-gateway | 18 | 2 | 2 | 3 | 0 | 11% |
| application | 16 | 0 | 0 | 0 | 0 | 0% |
| audit-chain | 15 | 0 | 0 | 0 | 0 | 0% |
| calendar | 15 | 0 | 0 | 0 | 0 | 0% |
| cell | 15 | 0 | 0 | 0 | 0 | 0% |
| cloud-iac | 26 | 0 | 0 | 0 | 0 | 0% |
| cloud-k8s | 19 | 0 | 0 | 0 | 0 | 0% |
| cloud-secrets | 15 | 0 | 0 | 0 | 0 | 0% |
| comms-email | 26 | 7 | 0 | 0 | 0 | 27% |
| community | 15 | 0 | 0 | 0 | 0 | 0% |
| compliance | 26 | 10 | 2 | 0 | 0 | 38% |
| connector | 15 | 12 | 5 | 1 | 0 | 80% |
| consent-graph | 15 | 0 | 0 | 0 | 0 | 0% |
| developer-sdk | 15 | 0 | 0 | 0 | 0 | 0% |
| docs | 20 | 0 | 0 | 0 | 0 | 0% |
| drive | 15 | 0 | 0 | 0 | 0 | 0% |
| feature-flags | 27 | 26 | 7 | 8 | 0 | **96%** |
| finops-portal | 26 | 8 | 0 | 0 | 0 | 31% |
| forms | 15 | 0 | 0 | 0 | 0 | 0% |
| foundry | 101 | 0 | 0 | 0 | 0 | **0%** |
| governance | 22 | 0 | 0 | 0 | 0 | 0% |
| identity | 17 | 0 | 0 | 0 | 0 | 0% |
| intelligence | 26 | 18 | 7 | 0 | 0 | 69% |
| mail | 18 | 2 | 0 | 1 | 0 | 11% |
| meet | 15 | 0 | 0 | 0 | 0 | 0% |
| messenger | 16 | 0 | 0 | 0 | 0 | 0% |
| network | 15 | 0 | 0 | 0 | 0 | 0% |
| notes | 18 | 2 | 0 | 1 | 0 | 11% |
| observability | 26 | 0 | 0 | 0 | 0 | 0% |
| ontology | 23 | 3 | 1 | 1 | 0 | 13% |
| ops-dashboard-control-center | 16 | 8 | 4 | 1 | 0 | 50% |
| payments | 18 | 18 | 2 | 0 | 0 | **100%** |
| plugin-app-store | 15 | 0 | 0 | 0 | 0 | 0% |
| recordings | 15 | 0 | 0 | 0 | 0 | 0% |
| sheets | 15 | 0 | 0 | 0 | 0 | 0% |
| shorts | 15 | 0 | 0 | 0 | 0 | 0% |
| sites | 15 | 0 | 0 | 0 | 0 | 0% |
| slides | 15 | 0 | 0 | 0 | 0 | 0% |
| social | 18 | 2 | 0 | 1 | 0 | 11% |
| tasks | 15 | 0 | 0 | 0 | 0 | 0% |
| tenancy | 26 | 11 | 0 | 0 | 0 | 42% |
| translate | 15 | 0 | 0 | 0 | 0 | 0% |
| workflow-engine | 15 | 0 | 0 | 0 | 0 | 0% |
| workflow-studio | 27 | 0 | 0 | 0 | 0 | 0% |
| **CORPUS** | **921** | **129** | **30** | **17** | **0** | **14.0%** |

### §3.2 Findings

1. **Eight µservices are at ≥50% keystone-bundle coverage**: `payments` (100%), `feature-flags` (96%), `connector` (80%), `intelligence` (69%), `ops-dashboard-control-center` (50%), `tenancy` (42%), `compliance` (38%), `finops-portal` (31%). These are the post-2026-05-20 wave-3-C/D µservices.
2. **Thirty µservices are at 0% keystone-bundle coverage**: every µservice that was authored Wave-1 / Wave-2 / Wave-3-A / Wave-3-B (pre-keystone) has zero ADR-0242…0258 citations in its IPs. **This includes substrate µservices** `foundry` (101 IPs), `observability` (26 IPs), `governance` (22 IPs), `cell` (15 IPs), `tenancy-substrate-portion`, `cloud-iac` (26 IPs), `cloud-k8s` (19 IPs), `cloud-secrets` (15 IPs), `identity` (17 IPs), `audit-chain` (15 IPs), `consent-graph` (15 IPs), `application` (16 IPs), `api-gateway` (the 16 of 18 not yet rebound). This is the primary substrate-rigor risk because substrate gaps propagate.
3. **Zero IPs cite any amendment ADR** (ADR-0246-amendment library-first dispatch; ADR-0257-amendment library-first registry; ADR-0253-amendment HTTP/3 + ECH + PQC). The amendment ADRs landed during the 2026-05-20 bundle and are not yet woven into IP language. This is a corpus-wide gap — even the 100%-coverage µservices need an amendment pass.
4. **The critical-path cluster (ADR-0297–0310) is barely represented**: 17 IPs cite any. The five µservices that have any critical-path citations are `api-gateway` (3), `feature-flags` (8), `connector` (1), `mail` (1), `notes` (1), `ontology` (1), `ops-dashboard-control-center` (1), `social` (1). The doctrine bundle landed 2026-05-20 and applies to **every** internet-facing / auth / financial / minor-user surface — see §8 for the surface×doctrine coverage gap.

### §3.3 Per-IP P0 list — substrate µservices at 0% keystone coverage

These are the IP sets that MUST be rebound first because substrate gaps propagate. The list is one IP per line:

```
microservices/intelligence/IP-001-runtime-runtime-cluster-iac.md
microservices/intelligence/IP-002-runtime-redis-and-postgres-baseline.md
microservices/intelligence/IP-003-runtime-capability-executor-kernel.md
microservices/intelligence/IP-004-runtime-capability-executor-domain-and-usecase.md
microservices/intelligence/IP-005-runtime-capability-registry-cache-stack.md
microservices/intelligence/IP-006-runtime-session-state-stack.md
microservices/intelligence/IP-007-runtime-invocation-orchestrator-stack.md
microservices/intelligence/IP-008-runtime-runtime-pool-stack.md
microservices/intelligence/IP-009-runtime-capability-executor-api-and-rest.md
microservices/intelligence/IP-010-runtime-capability-executor-sdk.md
microservices/intelligence/IP-011-runtime-capability-executor-app.md
microservices/intelligence/IP-012-runtime-autonomy-tier-gate.md
... (full 101 foundry IPs + 26 observability + 26 cloud-iac + 22 governance + 19 cloud-k8s + 17 identity + 16 application + 15 cell + 15 audit-chain + 15 cloud-secrets + 15 consent-graph)
```

Total substrate-tier IPs at 0% keystone coverage: **~290 IPs**. Each needs a related_adrs front-matter rebind to the relevant subset of `[ADR-0242 ADR-0243 ADR-0244 ADR-0245 ADR-0246+amendment ADR-0247 ADR-0248 ADR-0249 ADR-0250 ADR-0251 ADR-0252 ADR-0253+amendment ADR-0254 ADR-0255 ADR-0257+amendment ADR-0258 ADR-0263 ADR-0284 ADR-0292 ADR-0293 ADR-0294 ADR-0295 ADR-0296]` per the §3.2.1 ADR-adherence matrix in documentation-rigor.md.

### §3.4 Per-IP P1 list — product µservices at 0% keystone coverage

```
microservices/anonymous/IP-001..IP-015 (15 IPs)
microservices/community/IP-001..IP-013 (13 IPs incl. IP-013-retired VCS ratchet which is itself stale)
microservices/calendar/IP-001..IP-015 (15 IPs — note IP-011 already binds OpenAPI version bump)
microservices/drive/IP-001..IP-015 (15 IPs)
microservices/forms/IP-001..IP-015 (15 IPs)
microservices/messenger/IP-001..IP-016 (16 IPs — MUST cite MLS RFC 9420 per ADR-0246 KS#5 and ADR-0251 §D-10)
microservices/meet/IP-001..IP-015 (15 IPs)
microservices/network/IP-001..IP-015 (15 IPs)
microservices/notes/IP-001..IP-018 (16 of 18 IPs missing)
microservices/recordings/IP-001..IP-015 (15 IPs)
microservices/sheets/IP-001..IP-015 (15 IPs)
microservices/shorts/IP-001..IP-015 (15 IPs)
microservices/sites/IP-001..IP-015 (15 IPs)
microservices/slides/IP-001..IP-015 (15 IPs)
microservices/social/IP-001..IP-018 (16 of 18 missing)
microservices/tasks/IP-001..IP-015 (15 IPs)
microservices/translate/IP-001..IP-015 (15 IPs)
microservices/workflow-engine/IP-001..IP-015 (15 IPs)
microservices/workflow-studio/IP-001..IP-027 (all 27 IPs at 0% — IP-013..IP-027 also carry retired VCS ratchet lane staleness)
microservices/plugin-app-store/implementation-plans/IP-001..IP-015 (15 IPs)
microservices/developer-sdk/implementation-plans/IP-001..IP-015 (15 IPs)
microservices/analytics/specs/IP-001..IP-015 (15 IPs)
```

Total product-tier IPs at 0% keystone coverage: **~290 IPs.**

Combined P0+P1 corpus rebind: ~580 IPs need a keystone-bundle related_adrs pass.

---

## §4 — Acceptance-lane prefix staleness

The 2026-05-21 rename ledger moved 561 files from `governance-*` to `governance-*`. The audit's job: verify IP front-matter is clean of the retired prefix.

### §4.1 Result

| Probe | Hit count | Verdict |
|---|---:|---|
| IP files with `governance-` anywhere | **0** | PASS — rename complete in IP corpus |
| IP files with `governance-*` lane refs in `acceptance_lanes:` | (positive — counted) | PASS |
| IP files with `retired VCS ratchet` lane refs (separate sub-doctrine) | 15 IPs / 63 line hits | FAIL — see §2.2 |

**The foundry-fitness→governance rename is structurally complete in the IP corpus.** This was the most critical post-rename audit invariant and it passes. The independent `retired VCS ratchet` rename is **incomplete** (15 IPs still cite this lane name) — but that's a different sub-doctrine and is captured in §2.2.

### §4.2 Acceptance-lane roster validation

documentation-rigor.md §3.2.1 and ADR-0297..0310 §E enumerate the canonical lane roster. The audit cross-checked IP-cited lane names against the roster. Lanes used by IPs that are **not** in the canonical roster (candidate orphans or post-2026-05-20 additions that haven't reached the roster doc):

```
cell-boundary                          (used by cell/IP-006 — verify against ADR-0248 cellular tier doctrine)
check-openslo-conformance              (used by cell/IP-013 — verify against canonical openslo lane name)
openslo-conformance                        (used by audit-chain/IP-002 — duplicate of above or distinct?)
openslo-validate                           (used by sheets/IP-014, workflow-engine/IP-013 — duplicate-of-conformance?)
openslo-schema                             (used by cloud-k8s/IP-015 — likely distinct: schema vs conformance vs validate)
per-microservice-layout                    (short form of governance-per-microservice-layout — needs canonicalisation)
authority-cohesion                         (short form of governance-authority-cohesion)
hyperscaler-maturity-claims                (short form — verify canonical name)
layer-correctness                          (short form — verify against governance-layer-correctness or similar)
lean-a1                                    (legacy lean-aN lane naming — verify against ADR-0145 lean-a-family successor names)
a11y-at-spi                                (workflow-studio canvas lanes — verify against canonical a11y roster)
a11y-uia-conformance                       (workflow-studio canvas lanes)
a11y-uikit-traits                          (workflow-studio canvas lanes)
a11y-talkback-conformance                  (workflow-studio canvas lanes)
a11y-axe-zero-violations                   (workflow-studio canvas + codemirror)
perf-canvas-60fps / perf-canvas-60fps-leptos / perf-loro-merge-latency / perf-edge-p99 / perf-budget-no-regression  (workflow-studio + governance perf lanes — verify against canonical perf-lane roster)
waf-correctness                            (governance/IP-WASMTIME-002)
lsp-bridge-correctness / lsp-tenant-isolation       (workflow-studio LSP)
crdt-correctness-no-silent-loss            (workflow-studio loro-crdt)
presence-correctness / presence-isolation  (workflow-studio presence)
grammar-correctness                        (workflow-studio cedar grammar)
code-editor-correctness                    (workflow-studio codemirror)
audit-chain-emission / audit-chain-tamper-detect (ontology/IP-010)
```

Action for Wave-3-E: cross-check each candidate lane name against `docs/standards/documentation-rigor.md` §"CI lane roster" + ADR-0297 §E / ADR-0299..0310 §E. Lanes that don't appear in the canonical roster must either be added to the roster (with binding ADR) or renamed to match an existing lane. Per the no-silent-regression doctrine (Linus-style), no IP ships referencing a lane that doesn't exist.

### §4.3 IPs missing acceptance_lanes: front-matter entirely

317 IPs (34.4%) have **no `acceptance_lanes:` front-matter field at all**. Per the IP template, every IP must declare its acceptance lanes. The 317 missing-lane IPs are concentrated in:

| µservice | IPs missing acceptance_lanes |
|---|---:|
| `feature-flags` | 27 (all) |
| `comms-email` | 26 (all) |
| `compliance` | 26 (all) |
| `payments` | 18 (all) |
| `cloud-iac` | 11 |
| `community` | 15 (all) |
| `tenancy` | 11 |
| `connector` | 15 (all) |
| `observability` | 11 |
| `intelligence` | 1 |
| `ops-dashboard-control-center` | 16 (all) |
| `identity` | 17 (all) |
| `api-gateway` | 18 (all) |
| `consent-graph` | 15 (all) |
| `notes` | 3 |
| `mail` | 3 |
| `social` | 3 |
| `docs` | 5 |
| `cloud-k8s` | 4 |
| `network` | (use sub-section heads, not front-matter) |
| `slides` | (markdown-only descriptions) |
| `shorts` | 8 |
| `application` | 1 |
| `analytics` | 15 (all) |

These IPs need an `acceptance_lanes:` field added during the Wave-3-E rewrite.

---

## §5 — Changeset shape failures

Per ADR-0110 ChangeSet state machine, every IP must declare:

- `changeset_contract: claimable-verifiable-bundleable-promotable` in front-matter
- `## ChangeSet boundary` section in body
- `## Concrete File Targets` section with action-typed file table
- Single-PR-sized scope (no multi-concern bundling)
- `## Acceptance` (or `## Acceptance Gates`)
- `## Verification` (or `## Test Plan`) naming runbooks/SLOs/dashboards produced or modified

### §5.1 changeset_contract front-matter — corpus rollup

**600 of 921 IPs (65.1%) lack `changeset_contract:` in front-matter.**

Per-µservice breakdown (recursive):

| µservice | total | missing contract | % missing |
|---|---:|---:|---:|
| analytics | 15 | 15 | 100% |
| anonymous | 15 | 14 | 93% |
| api-gateway | 18 | 18 | 100% |
| application | 16 | 15 | 94% |
| audit-chain | 15 | 13 | 87% |
| calendar | 15 | 0 | 0% |
| cell | 15 | 13 | 87% |
| cloud-iac | 26 | 11 | 42% |
| cloud-k8s | 19 | 16 | 84% |
| cloud-secrets | 15 | 14 | 93% |
| comms-email | 26 | 26 | 100% |
| community | 15 | 15 | 100% |
| compliance | 26 | 26 | 100% |
| connector | 15 | 15 | 100% |
| consent-graph | 15 | 15 | 100% |
| developer-sdk | 15 | 0 | 0% |
| docs | 20 | 17 | 85% |
| drive | 15 | 14 | 93% |
| feature-flags | 27 | 27 | 100% |
| finops-portal | 26 | 26 | 100% |
| forms | 15 | 14 | 93% |
| foundry | 101 | 50 | 50% |
| governance | 22 | 3 | 14% |
| identity | 17 | 17 | 100% |
| intelligence | 26 | 24 | 92% |
| mail | 18 | 3 | 17% |
| meet | 15 | 0 | 0% |
| messenger | 16 | 0 | 0% |
| network | 15 | 0 | 0% |
| notes | 18 | 17 | 94% |
| observability | 26 | 11 | 42% |
| ontology | 23 | 22 | 96% |
| ops-dashboard-control-center | 16 | 16 | 100% |
| payments | 18 | 18 | 100% |
| plugin-app-store | 15 | 0 | 0% |
| recordings | 15 | 13 | 87% |
| sheets | 15 | 14 | 93% |
| shorts | 15 | 13 | 87% |
| sites | 15 | 14 | 93% |
| slides | 15 | 0 | 0% |
| social | 18 | 3 | 17% |
| tasks | 15 | 0 | 0% |
| tenancy | 26 | 24 | 92% |
| translate | 15 | 14 | 93% |
| workflow-engine | 15 | 0 | 0% |
| workflow-studio | 27 | 0 | 0% |

Observation: 8 µservices have 100% changeset_contract coverage (calendar, developer-sdk, meet, messenger, network, plugin-app-store, slides, tasks, workflow-engine, workflow-studio). 14 µservices have 100% missing.

### §5.2 `## ChangeSet boundary` section — corpus rollup

**545 of 921 IPs (59.2%) lack a `## ChangeSet boundary` section in body.**

Same per-µservice distribution. The exemplar (`microservices/observability/IP-001-layer-a-grafana-stack-iac.md` lines 22–24) shows the section is ~3 sentences naming the cohesive scope. Wave-3-E should rewrite the 545 IPs to add this section.

### §5.3 `## Concrete File Targets` section — corpus rollup

**458 of 921 IPs (49.7%) lack a concrete-file-targets table.** This is the highest-leverage section in the exemplar (lines 26–42) because it tells the executor what file paths to create / modify / delete. Without it, the IP is non-actionable.

Per-µservice missing-CFT count (sample):

```
api-gateway         18 of 18
comms-email         26 of 26
compliance          26 of 26
community           15 of 15
connect             15 of 15
consent-graph       15 of 15
feature-flags       27 of 27
finops-portal       26 of 26
identity            17 of 17
intelligence         1 of 26      (only the very oldest IP missing)
ops-dashboard       16 of 16
payments            18 of 18
tenancy             11 of 26
analytics           15 of 15
```

### §5.4 Single-PR-sized scope

The audit could not mechanically verify single-PR-sized scope (it requires reading each IP), but several IPs combine multiple concerns in their title (e.g., `IP-001-layer-a-postgres-redis-cedar-cosign-trivy-iac.md`). The exemplar IPs use one cohesive concern per IP (Grafana stack, kernel layer, domain layer). Wave-3-E reviewers should examine these multi-concern titles for split candidates:

```
microservices/plugin-app-store/implementation-plans/IP-001-layer-a-postgres-redis-cedar-cosign-trivy-iac.md  (5 systems)
microservices/developer-sdk/implementation-plans/IP-001-layer-a-postgres-openbao-backstage-iac.md           (3 systems)
microservices/developer-sdk/implementation-plans/IP-006-sdk-codegen-ts-rust-swift-kotlin-csharp-python.md   (6 languages — split per-language?)
microservices/intelligence/IP-WASMTIME-001-tool-sandbox-runtime-integration.md                                   (multiple wasmtime concerns)
microservices/intelligence/IP-WASMTIME-002-capability-token-binding.md                                           (token + binding)
microservices/sheets/IP-014-observability-slo-manifests-9-openslo.md                                        (9 SLO manifests — split per-SLO?)
```

### §5.5 `## Acceptance` section — corpus rollup

122 of 921 IPs (13.2%) lack an `## Acceptance` or `## Acceptance Gates` section. This is the strongest section coverage in the corpus — most IPs do have some form of acceptance criteria.

### §5.6 `## Verification` / `## Test Plan` section — corpus rollup

**444 of 921 IPs (48.2%) lack a `## Verification` or `## Test Plan` section.** This is critical because the verification section names the runbooks/SLOs/dashboards the IP produces — without it, "done" is not observable per the hyperscaler bar.

### §5.7 Changeset-shape per-µservice grade summary

```
=== A-grade (≥90% of all 6 shape signals present) ===
calendar (90%+ on 5 of 6 signals; missing some depends_on)
workflow-engine (≥90%)

=== B-grade (60-89%) ===
governance, observability, intelligence, mail, social, tenancy

=== C-grade (40-59%) ===
foundry, cloud-iac, sheets, slides

=== D-grade (10-39%) ===
calendar (depends_on), shorts

=== F-grade (<10% — full Wave-3-E rewrite) ===
api-gateway, comms-email, compliance, community, connect, consent-graph,
feature-flags, finops-portal, identity, ops-dashboard-control-center, payments,
plugin-app-store, developer-sdk, analytics, anonymous, application
```

---

## §6 — Cross-IP DAG analysis

Per ADR-0110 + ADR-0111 (changeset state machine + merge queue projected state), every IP must declare `depends_on:` and (recommended) `blocks:` front-matter so the merge queue can topologically order admission.

### §6.1 Corpus result

| Probe | Count | % |
|---|---:|---:|
| IPs missing `depends_on:` front-matter | **809** | **87.8%** |
| IPs declaring `depends_on:` | 112 | 12.2% |
| IPs declaring `blocks:` | (sample-verified ≤30) | <4% |

**The IP DAG is structurally unrepresentable from front-matter alone.** Only 12% of IPs declare their dependencies. The merge-queue projected-state fix-at-any-stage (ADR-0111) cannot operate on this corpus until the depends_on fields are populated.

### §6.2 Implicit-DAG signals

Some IPs do declare dependencies in prose (e.g., "Next IP" sections and "this IP depends on IP-007"). The `## Next IP` section is present in 488 of 921 IPs (52.9%). Where present, it forms a linear chain, not a DAG — useful but not sufficient.

### §6.3 Orphan IPs

Without `depends_on:` it's impossible to detect orphans mechanically. The audit can detect filename-based orphans:

```
microservices/governance/IP-NEW-eu-ai-act-annex-iii-refusal-lane.md
microservices/governance/IP-NEW-slsa-l3-evidence-grounded-lane.md
microservices/governance/IP-NEW-chaos-engineering-substrate.md
microservices/governance/IP-WASMTIME-001-envoy-wasm-filter-substrate.md
microservices/governance/IP-WASMTIME-002-waf-coraza-onboard.md
microservices/governance/IP-WASMTIME-004-authz-filter.md
microservices/intelligence/IP-WASMTIME-001-tool-sandbox-runtime-integration.md
microservices/intelligence/IP-WASMTIME-002-capability-token-binding.md
```

The `IP-NEW-*` and `IP-WASMTIME-*` files lack the numbered IP-NNN ordering used by the canonical naming. Per the IP template's `impl_plan_id` field, every IP must carry a sortable numeric ID. These 8 IPs should be renamed to numbered slots in their µservice's IP sequence (or assigned the next available NNN).

### §6.4 Suspected cycles

Without `depends_on:` declarations a cycle check is not possible. Wave-3-E must populate the dependency front-matter, then a `cargo run -p dev-cli -- ip-dag-lint` (which does not exist yet — see §10) can detect cycles.

### §6.5 Per-µservice IP DAG completeness rank

```
=== DAG-ready (depends_on populated for >50% of IPs) ===
(none — no µservice meets this bar)

=== DAG-emerging (1-50% populated) ===
plugin-app-store, sheets (shorts: 12 of 15), workflow-studio (24 of 27), workflow-engine (1 of 15)

=== DAG-absent (0% populated) ===
all other 40 µservices
```

This is the single largest IP-corpus quality gap. **A µservice cannot be promoted past dev to staging while its IP DAG is unrepresentable.**

---

## §7 — Rigor failures (per documentation-rigor.md)

### §7.1 Length distribution

| Bin | Count | % | Notes |
|---|---:|---:|---|
| <50 lines (stub) | 210 | 22.8% | Below any reasonable IP density |
| 50–99 lines | 419 | 45.5% | Below exemplar floor |
| 100–199 lines | 266 | 28.9% | At/above exemplar floor, below the 200-line operating bar |
| ≥200 lines | 26 | 2.8% | Above operating bar |

**629 of 921 IPs (68.3%) are below the 110-line exemplar floor.** The shortest IPs in the corpus:

```
14 microservices/api-gateway/IP-008-routing-worker-crate.md
14 microservices/api-gateway/IP-011-auth-handoff-usecase.md
15 microservices/api-gateway/IP-010-rate-limit-adapter-redis.md
15 microservices/api-gateway/IP-016-app-supervisor.md
16 microservices/api-gateway/IP-014-tls-cert-rotation-worker.md
16 microservices/api-gateway/IP-015-canary-cohort-shifter.md
16 microservices/api-gateway/IP-018-honeypot-route-mgr.md
18 microservices/api-gateway/IP-007-routing-grpc-crate.md
19 microservices/finops-portal/implementation-plans/IP-024-commitment-management-grpc.md
20 microservices/api-gateway/IP-013-abuse-defence-adapter-wasm.md
20 microservices/tenancy/IP-023-sub-scope-registry-adapter-postgres.md
21 microservices/compliance/IP-023-pack-registry-grpc.md
21 microservices/tenancy/IP-020-data-residency-enforcer-adapter.md
22 microservices/api-gateway/IP-005-routing-adapter-crate.md
22 microservices/api-gateway/IP-017-sov-cell-routing.md
22 microservices/finops-portal/implementation-plans/IP-026-showback-chargeback-emit.md
23 microservices/api-gateway/IP-006-routing-rest-crate.md
23 microservices/compliance/IP-024-dpia-orchestration-adapter-postgres.md
24 microservices/comms-email/IP-016-inbound-receiver-kernel.md
24 microservices/comms-email/IP-017-inbound-receiver-domain.md
24 microservices/comms-email/IP-023-inbound-receiver-rest.md
24 microservices/finops-portal/implementation-plans/IP-017-budget-alert-domain.md
24 microservices/finops-portal/implementation-plans/IP-021-showback-chargeback-domain.md
24 microservices/finops-portal/implementation-plans/IP-023-forecasting-rest-and-cache.md
24 microservices/finops-portal/implementation-plans/IP-025-rightsizing-rest-and-dashboard.md
25 microservices/comms-email/IP-018-list-management-usecase.md
25 microservices/comms-email/IP-020-reputation-monitor-worker.md
25 microservices/comms-email/IP-021-bounce-handler-domain.md
25 microservices/comms-email/IP-022-template-rendering-mjml-engine.md
25 microservices/comms-email/IP-024-list-management-rest.md
25 microservices/comms-email/IP-025-reputation-monitor-rest-and-dashboard.md
25 microservices/comms-email/IP-026-unsubscribe-async-emit.md
25 microservices/finops-portal/implementation-plans/IP-016-budget-alert-kernel.md
25 microservices/finops-portal/implementation-plans/IP-019-commitment-management-domain.md
25 microservices/tenancy/IP-016-sub-scope-registry-kernel.md
25 microservices/tenancy/IP-025-dr-pairing-async-emit.md
```

These 36 IPs are the most extreme stubs (≤25 lines). Each needs a full rewrite to reach the 110-line exemplar floor.

### §7.2 Required-section coverage rollup

| Section | Present in | Missing in |
|---|---:|---:|
| `## Intent` | 622 (67.5%) | 299 |
| `## ChangeSet boundary` | 376 (40.8%) | **545** |
| `## Concrete File Targets` | 463 (50.3%) | **458** |
| `## Acceptance` / `## Acceptance Gates` | 799 (86.8%) | 122 |
| `## Verification` / `## Test Plan` | 477 (51.8%) | **444** |
| `## References` | 567 (61.6%) | 354 |
| `## Halt Conditions` | 397 (43.1%) | **524** |
| `## Next IP` | 488 (53.0%) | 433 |

### §7.3 Front-matter coverage rollup

| Front-matter field | Present in | Missing in |
|---|---:|---:|
| `changeset_contract:` | 321 (34.9%) | **600** |
| `acceptance_lanes:` | 604 (65.6%) | 317 |
| `depends_on:` | 112 (12.2%) | **809** |
| `execution_unit:` | 521 (56.6%) | 400 |
| `owner:` / `owner_team:` | 634 (68.8%) | 287 |

### §7.4 ADR-citation coverage rollup

Per documentation-rigor.md §3.2.1, every IP MUST cite the binding ADR. The 521 IPs missing `related_adrs:` or any inline `ADR-XXXX` reference fail this row. Audit sample:

```
microservices/api-gateway/IP-005-routing-adapter-crate.md     (22 lines, no ADR citations)
microservices/api-gateway/IP-007-routing-grpc-crate.md        (18 lines, no ADR citations)
microservices/comms-email/IP-016-inbound-receiver-kernel.md   (24 lines, no ADR citations)
microservices/community/IP-002..IP-013                        (12 of 13 IPs missing keystone bundle citations)
microservices/observability/IP-013-event-driven-promote-workflows.md  (sparse ADR list)
microservices/intelligence/IP-001..IP-012                          (early foundry runtime IPs predate keystone bundle)
microservices/cloud-iac/IP-GITOPS-001..IP-GITOPS-008          (8 GITOPS sub-IPs — verify ADR citation)
```

Wave-3-E must add `related_adrs:` front-matter to every IP touching a primitive that has a binding ADR.

### §7.5 Naming-justification absence

The canonical IP template + memory `feedback_naming_justification` requires a one-line naming justification for any new name introduced. The audit found **few IPs with explicit naming-justification blocks** — sampling indicates <5% of IPs that introduce new crate or BC names carry the v4 BNF + 12-layer-enum conformance proof. (Memory cites 13-layer post ADR-0105; the justification block must update.)

Wave-3-E action: add a `naming_justification:` field or `## Naming Justification` block to every IP that creates a new crate or new BC name. Concentrated in `IP-001…IP-005` of each µservice.

### §7.6 Six-dimension rigor matrix (Maintainability / Observability / Scalability / Performance / Optimization / Code-quality)

documentation-rigor.md §1.2 requires every doc class to address the six engineering-rigor dimensions where applicable. ImplementationPlan inherits the bar via the ADR/PRD/Spec it cites. The audit could not mechanically score each IP against the matrix; sampling indicates:

- **Observability dimension**: 56 of 921 IPs (6.1%) explicitly cite ADR-0263 (audit-event emission registry) or name emitted metrics. The bar is "every primitive declares its emitted audit events, traces, metrics, and logs". **~94% of IPs fail this row.**
- **Scalability dimension**: ~15 IPs cite capacity math (Little's Law, queue theory). Most IPs delegate to the µservice's `capacity-model.md`.
- **Code-quality dimension**: ~250 IPs name a `## Test Plan` section. **The remaining 671 are silent on test class / coverage floor / lint passes.**

This is the largest tail in the audit — the IPs simply do not address the §1.2 dimensions at the depth documentation-rigor.md mandates.

---

## §8 — Critical-path coverage gaps

ADR-0297..0310 cluster (the critical-path doctrine) requires every IP that touches a sensitive surface to wire the matching defense. The audit grep'd surface keywords against the doctrine cite, then computed the coverage gap.

### §8.1 Surface × doctrine coverage matrix

| Surface class | Keyword probe | IPs touching | IPs citing matching ADR | Coverage % | Gap |
|---|---|---:|---:|---:|---:|
| Internet-facing (edge / abuse / DDoS / WAF / rate-limit) | yes | **165** | 11 (cite ADR-0297) | 6.7% | **154 IPs** |
| Auth (login / sign-in / recovery / MFA / OAuth / password) | yes | **346** | 2 (cite ADR-0298 or ADR-0299) | 0.6% | **344 IPs** |
| Financial (payment / billing / invoice / charge / payout / refund / fraud / DRMP) | yes | **61** | 0 (cite ADR-0307) | 0.0% | **61 IPs** |
| Minor-user (minor / child / youth / teen / 13 / 18 / COPPA / age) | yes | **712** | 12 (cite ADR-0292) | 1.7% | **700 IPs** |

**The minor-user count of 712 is inflated** — the keyword probe matched generic numbers (13, 18) and common words. Conservative re-probe with `(COPPA|child[- ]safety|minor[- ]user|age[- ]gate)` yields a much smaller candidate set (~30 IPs), but even that smaller set has near-zero ADR-0292 binding. Wave-3-E should narrow the probe and re-score.

### §8.2 P0 financial-surface IPs missing ADR-0307

These IPs touch payment/billing/payout/refund/fraud surfaces and don't cite ADR-0307 (fraud-detect + DRMP). All 61 are P0; the most exposed:

```
microservices/payments/IP-004-payments-adapter-stripe.md
microservices/payments/IP-007-payments-domain-payout.md
microservices/payments/IP-009-payments-domain-dispute.md
microservices/payments/IP-012-payments-usecase-subscription.md
microservices/payments/IP-013-payments-domain-sub-merchant.md
microservices/payments/IP-014-payments-usecase-sub-merchant.md
microservices/payments/IP-015-payments-rest-grpc-app.md
microservices/payments/IP-016-payments-settlement-domain.md
microservices/payments/IP-017-payments-settlement-worker.md
microservices/payments/IP-018-payments-adapter-adyen.md
microservices/developer-sdk/implementation-plans/IP-010-payout-ach-sepa-kftc-fedwire.md
microservices/developer-sdk/implementation-plans/IP-011-tax-form-1099-vat-moss-kr-vat.md
microservices/developer-sdk/implementation-plans/IP-015-stripe-connect-parity-end-to-end-drill.md
microservices/finops-portal/implementation-plans/IP-013-finops-portal-credit-ledger-kernel.md
microservices/finops-portal/implementation-plans/IP-014-finops-portal-focus-export-pipeline.md
microservices/connector/IP-009-connector-catalog-seed.md       (any connector touching payment APIs)
```

All 18 payments IPs cite the keystone bundle (100% per §3.1) but **none cite ADR-0307**. The Wave-3-E task is to rebind all 18 + sister financial IPs.

### §8.3 P0 internet-facing IPs missing ADR-0297

A representative sample (full list = 154 IPs):

```
microservices/api-gateway/IP-013-abuse-defence-adapter-wasm.md      (literally the abuse-defence adapter)
microservices/api-gateway/IP-014-tls-cert-rotation-worker.md
microservices/api-gateway/IP-017-sov-cell-routing.md
microservices/governance/IP-WASMTIME-001-envoy-wasm-filter-substrate.md
microservices/governance/IP-WASMTIME-002-waf-coraza-onboard.md       (literally WAF onboarding)
microservices/shorts/IP-003-video-upload-bc.md
microservices/drive/IP-009-share-link.md
microservices/consent-graph/IP-005-enforcement-domain-cedar.md
microservices/consent-graph/IP-007-revocation-kernel-worker.md
microservices/community/IP-010-foundry-guardrails-moderation-bridge.md
microservices/cloud-secrets/IP-007-resolver-rest-and-sdk-rust.md
microservices/connector/IP-002-connector-catalog-domain-kernel.md
microservices/connector/IP-009-connector-catalog-seed.md
microservices/connector/IP-010-iac-postgres-openbao.md
microservices/workflow-studio/IP-002-visual-canvas-kernel-domain.md
microservices/workflow-studio/IP-018-swiftui-canvas-impl.md
microservices/workflow-studio/IP-019-compose-canvas-impl.md
microservices/workflow-studio/IP-020-gtk-drawingarea-impl.md
microservices/workflow-studio/IP-027-cedar-grammar-impl.md
```

These IPs implement WAF, Envoy filter, rate-limit, share-link, edge canvas — every one is exposed to abuse and must cite ADR-0297.

### §8.4 P0 auth-surface IPs missing ADR-0298 / ADR-0299

Auth surfaces span all 46 µservices (every µservice authenticates incoming requests via the identity µservice). The IPs that **directly implement** auth primitives and miss the doctrine:

```
microservices/identity/IP-001..IP-017 (all 17 IPs — 0 cite ADR-0298 or ADR-0299)
microservices/governance/IP-WASMTIME-004-authz-filter.md
microservices/governance/IP-008-evidence-emitter-kernel-domain.md
microservices/governance/IP-009-evidence-emitter-adapter-rest-worker.md
microservices/cloud-secrets/IP-006-resolver-adapter-openbao.md
microservices/cloud-secrets/IP-013-audit-emitter-bridge-to-audit-chain.md
microservices/consent-graph/IP-010-projection-gateway-mint-acl.md
microservices/consent-graph/IP-012-audit-bridge-bilateral-emitter.md
```

The identity-µservice 17-IP set is the highest P0 because identity is the universal auth substrate. **Every identity IP must cite ADR-0298 (emergency-services bypass) + ADR-0299 (account-recovery).**

### §8.5 P0 minor-user IPs missing ADR-0292

Narrow re-probe with `(COPPA|child[- ]safety|age[- ]gate|minor[- ]user|under[- ]18|under[- ]13)`:

Action: Wave-3-E re-probe with the narrow regex, then attach ADR-0292 to each IP that legitimately handles minor users. The 712-figure from the wide probe is dominated by noise.

### §8.6 Critical-path doctrine binding for substrate µservices

The 14 critical-path ADRs (0297–0310) apply to product µservices most directly, but substrate µservices that **provide** the defence primitives also need bindings:

| Substrate µservice | Provides | Should cite |
|---|---|---|
| `api-gateway` | rate-limit, WAF integration, edge auth handoff | ADR-0297 (abuse-defence) |
| `identity` | emergency-services-bypass, account-recovery, MFA | ADR-0298, ADR-0299 |
| `consent-graph` | per-tenant consent enforcement | ADR-0298 (consent-required-for-bypass) |
| `cloud-secrets` | secret rotation, OpenBao surface | ADR-0297 + ADR-0298 |
| `governance` | WAF / authz filter / evidence-emitter | ADR-0297 (WAF) |
| `audit-chain` | merkle-anchored audit-stream | ADR-0263 (already cited by 58 IPs) |
| `foundry` (intelligence substrate post-ADR-0247) | self-modification + tool-sandbox | ADR-0247 (already gap — §2.1) + ADR-0297 (tool egress) |
| `payments` | charge / refund / payout / fraud-detect | ADR-0307 |
| `compliance` | pack activation gates | ADR-0297 + ADR-0307 (per-pack overlays) |

Wave-3-E will be a substrate-first pass for this reason.

---

## §9 — Supersede candidates

IPs that describe work bound to a now-superseded ADR should move to `microservices/<svc>/superseded/IP-NNN-*.md` with a tombstone link to the successor IP.

### §9.1 ADR-0136-era foundry IPs (superseded by ADR-0247)

| IP file | Status | Recommended action |
|---|---|---|
| `microservices/intelligence/IP-091-milvus-cluster-iac.md` | binds ADR-0136 | Rebind to ADR-0247 + ADR-0255; **keep at current path** — the work itself is still valid under the new doctrine (Milvus is the vector store for the consumer-intelligence substrate per ADR-0255 KS#14) |
| `microservices/intelligence/IP-092-vector-collection-bootstrap.md` | binds ADR-0136 | Same — rebind, don't supersede |
| `microservices/intelligence/IP-WASMTIME-001-tool-sandbox-runtime-integration.md` | binds ADR-0136 + ADR-0200 + ADR-0147 | Rebind ADR-0136 → ADR-0247; **also rename file from IP-WASMTIME-001 to next numbered slot** |
| `microservices/intelligence/IP-WASMTIME-002-capability-token-binding.md` | binds ADR-0136 + ADR-0200 | Same — rebind + rename |
| `microservices/intelligence/IP-001-consumer-intelligence-substrate.md` | front-matter binds ADR-0136 | Rebind to ADR-0247 + ADR-0255 |

**No IPs are recommended for hard-supersede (move to `superseded/`).** The ADR-0136 → ADR-0247 transition is a doctrinal rebind, not a work-cancellation. The IPs' work still applies; the rationale changes.

### §9.2 `retired VCS ratchet` IPs

| IP file | Action |
|---|---|
| `microservices/observability/IP-012-retired-vcs-ratchet` | **Rename file** to `IP-012-git-promotion-readiness-lane.md`; update body to use `oya git` verb surface; do NOT supersede — the lane itself is canonical |
| `microservices/community/IP-013-retired-vcs-ratchet` | Rename to `IP-013-git-promotion-readiness.md` |
| `microservices/observability/IP-014-automated-rollback-primitive.md` | Update `crates/dev-cli/src/commands/vcs/rollback.rs` path to `crates/dev-cli/src/commands/git/rollback.rs` |

### §9.3 IPs whose work is folded into successor µservices

The Intelligence two-layer substrate (ADR-0255 KS#14) **absorbs Foundry** per the keystone bundle. Several foundry IPs are work that now belongs to `intelligence/`:

| Foundry IP | Recommended new home (Wave-3-E) |
|---|---|
| `microservices/intelligence/IP-001-runtime-runtime-cluster-iac.md` | retain at foundry/ (substrate stays substrate) OR move to intelligence/ — decision belongs to ADR-0255 commentary; **the audit's job is to flag** |
| Similar for foundry/IP-002..IP-012 runtime IPs | same |
| `microservices/intelligence/IP-091-milvus-cluster-iac.md` + `IP-092-vector-collection-bootstrap.md` | likely belongs to intelligence/ per ADR-0255 |

This is a **µservice-restructure** question — out of scope for an audit-only pass. Wave-3-E governance must decide; Wave-3-F executes.

### §9.4 Non-numbered IPs (IP-NEW-*, IP-WASMTIME-*, IP-GITOPS-*)

Per the IP template, every IP has a sortable numeric ID. The 8 non-numbered files in `microservices/governance/` + `microservices/intelligence/` should be renamed to numbered slots:

```
microservices/governance/IP-NEW-eu-ai-act-annex-iii-refusal-lane.md      → microservices/governance/IP-NNN-…
microservices/governance/IP-NEW-slsa-l3-evidence-grounded-lane.md         → microservices/governance/IP-NNN-…
microservices/governance/IP-NEW-chaos-engineering-substrate.md            → microservices/governance/IP-NNN-…
microservices/governance/IP-WASMTIME-001-envoy-wasm-filter-substrate.md   → microservices/governance/IP-NNN-…
microservices/governance/IP-WASMTIME-002-waf-coraza-onboard.md            → microservices/governance/IP-NNN-…
microservices/governance/IP-WASMTIME-004-authz-filter.md                  → microservices/governance/IP-NNN-…
microservices/intelligence/IP-WASMTIME-001-tool-sandbox-runtime-integration.md → microservices/intelligence/IP-NNN-…
microservices/intelligence/IP-WASMTIME-002-capability-token-binding.md         → microservices/intelligence/IP-NNN-…
microservices/cloud-iac/IP-GITOPS-001..IP-GITOPS-008                       → microservices/cloud-iac/IP-NNN-…
```

Plus the non-flat layouts (analytics/specs/, developer-sdk/implementation-plans/, finops-portal/implementation-plans/, plugin-app-store/implementation-plans/) need flattening per ADR-0131.

---

## §10 — Recommended remediation order

The Wave-3-E IP-rewrite agent should execute in the following order, **substrate-first, then product**.

### §10.1 Wave-3-E batch 0 — corpus-wide mechanical rewrites (1 PR, no design)

These can be done by a sed-script in a single PR; they are purely renames:

| Task | Files touched | Risk |
|---|---|---|
| Rename retired VCS ratchet → `oya git done` (30 boilerplate hits + 33 other body hits) | 30 IPs (developer-sdk/* + plugin-app-store/*) + 33 hits | low — verb-surface rename per PR-159B |
| Rename `retired VCS ratchet` lane → `governance-promotion-readiness` (if governance-lane decision says so) **OR** retain (if [[git-canonical-2026-05-18]] says lane stays vcs-prefixed) | 15 IP file fronts | low — depends on lane-roster decision |
| Rename `crates/dev-cli/src/commands/vcs/*` → `git/*` in IP body refs | 1 IP (observability/IP-014) | low |
| Bump OpenAPI 3.1 → 3.2.0 / AsyncAPI 3.0 → 3.1.0 prose | 4 IPs (calendar, finops-portal, sites, drive) | low |
| Rename `phase: P01-translate-platform` → `phase: P01-translate-shared` | 15 translate IPs | low — pure naming |
| Rename `ops-platform` team token → `ops-shared` | ~8 IPs (identity, anonymous, api-gateway, …) | low |
| Replace ADR-0136 citation → ADR-0247 (+ ADR-0255 for intelligence IP) | 5 IPs (4 foundry + 1 intelligence) | medium — needs prose rewrite, not just sed |

Estimated effort: 1 day for the agent that knows the ledger. Single PR.

### §10.2 Wave-3-E batch 1 — substrate µservices keystone-bundle rebind (P0)

For each of the 290 substrate-tier IPs at 0% keystone coverage, add the relevant ADR-0242..0258 + amendments + critical-path-cluster citations.

Priority order (substrate before product, per ADR-0245 substrate-vs-product layering):

| Order | µservice | IPs | Why first |
|---|---|---:|---|
| 1 | `cell` | 15 | ADR-0248 cellular tier substrate — every µservice depends on cell topology |
| 2 | `tenancy` | 26 | ADR-0242 oyatie-is-a-tenant + ADR-0244 tenant scoping primitive — universal substrate |
| 3 | `cloud-secrets` | 15 | ADR-0255 §D-4 BYOK + OpenBao surface — substrate for every µservice |
| 4 | `identity` | 17 | ADR-0243 Cedar universal gate + ADR-0298 emergency-bypass + ADR-0299 account-recovery |
| 5 | `governance` | 22 | ADR-0247 self-modification + ADR-0297 WAF — every IP in governance is a primitive other µservices depend on |
| 6 | `consent-graph` | 15 | ADR-0244 tenant scoping + ADR-0251 compliance packs |
| 7 | `audit-chain` | 15 | ADR-0263 audit-event emission registry |
| 8 | `compliance` | 26 | ADR-0251 compliance-pack primitive + ADR-0250 build-ahead-of-certification |
| 9 | `cloud-iac` | 26 | ADR-0254 K8s + Cloud Hypervisor — IaC is the substrate's substrate |
| 10 | `cloud-k8s` | 19 | ADR-0254 — same |
| 11 | `observability` | 26 | ADR-0263 audit-event emission + ADR-0297 §E lane roster |
| 12 | `api-gateway` | 18 | ADR-0297 abuse-defence + ADR-0253 HTTP/3 |
| 13 | `application` | 16 | ADR-0245 substrate-vs-product layering |
| 14 | `foundry` | 101 | ADR-0247 self-modification + ADR-0255 intelligence two-layer (post-ADR-0136) |
| 15 | `ontology` | 23 | ADR-0245 substrate; renamed from object-graph per [[glossary-ontology-not-object-graph]] |

Estimated: ~290 IPs × ~30 min/IP = ~145 hours of focused agent work, split into per-µservice PR batches. Each µservice batch is one PR.

### §10.3 Wave-3-E batch 2 — product µservices keystone-bundle rebind (P1)

After substrate is rebound, product µservices follow. Order = revenue-critical first, then social, then internal:

| Order | µservice | IPs | Reason |
|---|---|---:|---|
| 16 | `payments` | 18 | already 100% keystone; needs ADR-0307 + ADR-0250 add — narrow rebind |
| 17 | `developer-sdk` | 15 | revenue-adjacent (Stripe parity); needs ADR-0307 |
| 18 | `finops-portal` | 26 | financial — needs ADR-0307 + ADR-0249 marketplace doctrine |
| 19 | `intelligence` | 26 | 69% keystone; needs ADR-0255 amendment + ADR-0247 |
| 20 | `feature-flags` | 27 | 96% keystone; needs amendments + ADR-0263 audit-event rebind |
| 21 | `comms-email` | 26 | DMARC + reputation-monitor — needs ADR-0297 abuse-defence |
| 22 | `messenger` | 16 | ADR-0246 KS#5 MLS RFC 9420 canonical E2EE |
| 23 | `meet` | 15 | real-time + WHIP — needs ADR-0253 HTTP/3 |
| 24 | `connector` | 15 | 80% keystone; finish off |
| 25 | `ops-dashboard-control-center` | 16 | 50%; finish off |
| 26 | `notes` | 18 | content + sharing |
| 27 | `social` | 18 | content + sharing |
| 28 | `mail` | 18 | per ADR-0297 abuse-defence |
| 29 | `drive` | 15 | per ADR-0297 abuse-defence on share-link |
| 30 | `workflow-engine` | 15 | per ADR-0245 substrate-vs-product |
| 31 | `workflow-studio` | 27 | front-end product; needs accessibility lanes |
| 32 | `community` | 15 | content + foundry-bridge |
| 33 | `forms` | 15 | content collection |
| 34 | `calendar` | 15 | content |
| 35 | `tasks` | 15 | content |
| 36 | `sheets` | 15 | content |
| 37 | `slides` | 15 | content |
| 38 | `sites` | 15 | content + edge — needs ADR-0297 |
| 39 | `shorts` | 15 | content + edge — needs ADR-0297 |
| 40 | `recordings` | 15 | content |
| 41 | `anonymous` | 15 | tenant variant — needs ADR-0242 tenant doctrine |
| 42 | `translate` | 15 | content + i18n |
| 43 | `plugin-app-store` | 15 | marketplace — needs ADR-0249 |
| 44 | `analytics` | 15 | reporting |
| 45 | `network` | 15 | per ADR-0253 HTTP/3 + QUIC |
| 46 | `docs` | 20 | meta |

Estimated: ~580 IPs × ~25 min/IP = ~240 hours. Single-µservice PR batches.

### §10.4 Wave-3-E batch 3 — depends_on / DAG population

**This is the single largest gap (809 IPs missing).** For each µservice's IP set:

1. Read all IPs in the µservice.
2. Compute the implicit DAG from prose (Next-IP chains, "this IP depends on…" prose, file-path layering: kernel → domain → usecase → adapter → rest → app).
3. Populate `depends_on:` + `blocks:` front-matter accordingly.
4. Verify no cycle.
5. Verify the resulting DAG matches the µservice's PHASE doc.

Estimated: 921 IPs × ~10 min/IP for DAG-edit = ~155 hours. Distinct per-µservice PRs.

### §10.5 Wave-3-E batch 4 — changeset shape rewrites

For each of the 545 IPs missing `## ChangeSet boundary` and the 458 IPs missing `## Concrete File Targets`, rewrite to add the section using the observability exemplar shape. Concentrated in the F-grade µservices (api-gateway, comms-email, compliance, community, connect, consent-graph, feature-flags, finops-portal, identity, ops-dashboard, payments, plugin-app-store, developer-sdk, analytics, anonymous, application).

Estimated: ~600 IPs × ~45 min/IP for full-section add = ~450 hours. Largest single workload in the audit.

### §10.6 Wave-3-E batch 5 — length-floor remediation

For each of the 629 IPs below the 110-line exemplar floor, lengthen to meet the floor. This work overlaps with batches 1–4 (most below-floor IPs are also missing sections); much of the line-count gap closes naturally during the section-rewrite passes. Net new effort beyond batches 1–4: ~50 hours.

### §10.7 Wave-3-E batch 6 — critical-path doctrine wiring

For each P0 surface×doctrine gap (§8), add the matching ADR citation + acceptance-lane reference:

- 154 internet-facing IPs need ADR-0297 + the matching `governance-edge-abuse-defence` lane
- ~30-50 narrow auth IPs (identity 17 + governance authz + cloud-secrets + consent-graph) need ADR-0298 + ADR-0299 + auth lanes
- 61 financial IPs need ADR-0307 + `governance-fraud-detect-evidence` lane
- ~30 narrow minor-user IPs (re-probe) need ADR-0292

Estimated: ~280 IPs × ~20 min/IP = ~93 hours.

### §10.8 Wave-3-E batch 7 — non-flat layout flattening (ADR-0131)

Move IPs to the flat layout:

```
microservices/analytics/specs/IP-*.md           → microservices/analytics/IP-*.md
microservices/developer-sdk/implementation-plans/IP-*.md → microservices/developer-sdk/IP-*.md
microservices/finops-portal/implementation-plans/IP-*.md → microservices/finops-portal/IP-*.md
microservices/plugin-app-store/implementation-plans/IP-*.md → microservices/plugin-app-store/IP-*.md
microservices/cloud-iac/IP-GITOPS-*.md           → renumber + flatten if any are nested
microservices/cloud-k8s/(nested?)                → flatten if any are nested
```

Plus update all internal xrefs and `companion_docs:` paths.

Estimated: 75 file moves × 5 min = 6 hours; touches many xrefs so ≈15 hours total.

### §10.9 Wave-3-E batch 8 — IP-NEW-* / IP-WASMTIME-* / IP-GITOPS-* renumbering

The 8 non-numbered IPs in governance + foundry must be assigned numeric IDs in the µservice's IP sequence + xrefs updated.

Estimated: 8 IPs × 30 min for renumber + xref update = 4 hours.

### §10.10 Wave-3-E batch 9 — naming-justification block addition

Add `naming_justification:` field or `## Naming Justification` block to every IP that introduces a new crate or BC name. Concentrated in IP-001..IP-005 of each µservice.

Estimated: 46 µservices × ~5 IPs each × 15 min = ~57 hours.

### §10.11 Total estimated remediation effort

| Batch | Workload | Estimated hours |
|---|---|---:|
| 0 — mechanical rewrites (sed-script) | 1 PR | 8 |
| 1 — substrate µservice keystone rebind | 290 IPs / 15 PRs | 145 |
| 2 — product µservice keystone rebind | 580 IPs / 30 PRs | 240 |
| 3 — depends_on / DAG population | 921 IPs / 46 PRs | 155 |
| 4 — changeset shape rewrites | 600 IPs / many PRs | 450 |
| 5 — length-floor remediation | overlap with 1-4 | 50 |
| 6 — critical-path doctrine wiring | ~280 IPs / many PRs | 93 |
| 7 — non-flat layout flattening | 75 files / 4 PRs | 15 |
| 8 — IP-NEW/WASMTIME/GITOPS renumbering | 8 files / 1 PR | 4 |
| 9 — naming-justification blocks | ~230 IPs / 46 PRs | 57 |
| **TOTAL** | **~921 IPs / ~145 PRs** | **~1,217 agent-hours** |

At 8h/day, this is ~152 agent-days of focused work, or **~7-8 calendar weeks** at the standard Foundry pipeline cadence (4-8 in-flight PRs cap per `feedback_pipeline_clog_gotchas_2026_05_17`).

---

## §11 — Top-30 P0/P1 findings

Numbered list ordered by criticality (production-risk first, then doctrine-binding gap, then rigor-floor).

### P0 (production-risk; must close before any pre-GA promotion)

1. **All 18 `microservices/payments/IP-*.md` are missing ADR-0307 (fraud-detect + DRMP) citation.** Payments cannot ship without the fraud-detect doctrine wired. Payments has 100% keystone-bundle coverage on ADR-0242..0258 but **zero** ADR-0307 binding. P0-FIN.
2. **All 17 `microservices/identity/IP-*.md` are missing ADR-0298 (emergency-services bypass) and ADR-0299 (account-recovery) citations.** Identity is the universal auth substrate; these two doctrines are mandatory. P0-AUTH.
3. **`microservices/intelligence/IP-091-milvus-cluster-iac.md`, `IP-092-vector-collection-bootstrap.md`, `IP-WASMTIME-001..002` cite the SUPERSEDED ADR-0136 (foundry-as-single-microservice).** Must rebind to ADR-0247 (self-modification) per [[self-modification-doctrine]]. P0-DOCTRINE.
4. **`microservices/intelligence/IP-001-consumer-intelligence-substrate.md` front-matter binds ADR-0136 (superseded).** Must rebind to ADR-0247 + ADR-0255. P0-DOCTRINE.
5. **`microservices/api-gateway/IP-013-abuse-defence-adapter-wasm.md` is the abuse-defence adapter and doesn't cite ADR-0297 (abuse-defence doctrine).** The doctrine-named IP without the doctrine binding. P0-INTERNET.
6. **`microservices/governance/IP-WASMTIME-002-waf-coraza-onboard.md` is the WAF onboarding IP and doesn't cite ADR-0297.** P0-INTERNET.
7. **`microservices/messenger/IP-001..IP-016` (16 IPs) at 0% keystone coverage; per ADR-0246 KS#5 every messenger IP must cite RFC 9420 MLS canonical E2EE.** Currently 3 of 921 IPs corpus-wide mention MLS at all. P0-DOCTRINE.
8. **ADR-0333 retired the cell implementation-plan set into tenancy, cloud-iac, observability, api-gateway, audit-chain, and the shuffle-sharding crate; ADR-0248 cellular tier substrate doctrine remains the foundation every µservice depends on.** Substrate gap. P0-SUBSTRATE.
9. **`microservices/tenancy/IP-001..IP-026` (26 IPs) — only 11 cite keystone bundle; ADR-0242 oyatie-is-a-tenant + ADR-0244 tenant scoping primitive bindings missing for 15.** Tenancy is the universal substrate. P0-SUBSTRATE.
10. **`microservices/cloud-secrets/IP-001..IP-015` (15 IPs) at 0% keystone coverage; OpenBao + BYOK substrate. ADR-0255 §D-4 provider_credential_mode binding missing.** P0-SUBSTRATE.
11. **`microservices/intelligence/IP-001..IP-101` (101 IPs) at 0% keystone coverage; foundry is the substrate for self-modification + intelligence.** Largest single µservice rebind. P0-SUBSTRATE.
12. **`microservices/observability/IP-001..IP-026` (26 IPs) at 0% keystone coverage; observability is the substrate every µservice telemetries to.** P0-SUBSTRATE.
13. **`microservices/governance/IP-001..IP-022` (22 IPs) at 0% keystone coverage including the policy-engine + WAF + authz-filter substrate.** P0-SUBSTRATE.
14. **63 line-anchored `retired VCS ratchet` references across 15 IP files violate the [[git-canonical-2026-05-18]] verb-surface rename.** Mechanically rewriteable (sed-script). P1-RENAME.
15. **30 IPs in developer-sdk + plugin-app-store carry the `On successful \retired VCS ratchet…` boilerplate; must update to `oya git done`.** Mechanically rewriteable. P1-RENAME.
16. **`microservices/observability/IP-012-retired-vcs-ratchet` filename + impl_plan_id contain the retired prefix.** Rename file + impl_plan_id; do not supersede. P1-RENAME.
17. **`microservices/community/IP-013-retired-vcs-ratchet` filename contains the retired prefix.** Rename file. P1-RENAME.
18. **`microservices/observability/IP-014-automated-rollback-primitive.md` references `crates/dev-cli/src/commands/vcs/rollback.rs`; per [[git-canonical-2026-05-18]] the path is `…/commands/git/rollback.rs`.** Update path ref. P1-RENAME.
19. **OpenAPI 3.1 / AsyncAPI 3.0 / 2.x stale-version references in 5 IPs (calendar/IP-011, finops-portal/IP-005, sites/IP-013, drive/IP-004).** Must be 3.2.0 / 3.1.0. P1-VERSION.
20. **Non-flat IP layouts in 4 µservices violate ADR-0131** (`analytics/specs/`, `developer-sdk/implementation-plans/`, `finops-portal/implementation-plans/`, `plugin-app-store/implementation-plans/`). 75 IP files must be flattened. P1-STRUCTURE.

### P1 (doctrine binding; close in Wave-3-E)

21. **600 of 921 IPs (65.1%) lack `changeset_contract:` in front-matter.** ADR-0110 ChangeSet state machine cannot operate. Corpus-wide rewrite. P1-CHANGESET.
22. **809 of 921 IPs (87.8%) lack `depends_on:` front-matter.** ADR-0111 merge-queue projected-state cannot operate. Largest single gap. P1-DAG.
23. **545 of 921 IPs (59.2%) lack `## ChangeSet boundary` section in body.** Exemplar shape not followed. P1-SECTION.
24. **458 of 921 IPs (49.7%) lack `## Concrete File Targets` table.** IPs are non-actionable for the executor agent. P1-SECTION.
25. **444 of 921 IPs (48.2%) lack `## Verification` / `## Test Plan` section.** "Done" is not observable. P1-SECTION.
26. **629 of 921 IPs (68.3%) are below the 110-line exemplar floor.** Intern-buildability fails. P1-LENGTH.
27. **154 internet-facing IPs miss ADR-0297 (abuse-defence doctrine).** Critical-path doctrine gap. P1-CRITICAL-PATH.
28. **44 IPs use "platform" where doctrine says "shared" per [[glossary-shared-not-platform]].** Mostly editorial; ~21 are clear hits. P3-EDITORIAL.
29. **Zero IPs cite any amendment ADR** (ADR-0246-amendment library-first dispatch; ADR-0257-amendment library-first registry; ADR-0253-amendment HTTP/3 + ECH + PQC). Corpus-wide pass needed. P1-DOCTRINE.
30. **The acceptance-lane roster has at least 25 candidate lane names that may not appear in the canonical roster** (cell-boundary, perf-canvas-60fps, a11y-* lanes, etc.). Cross-check against `documentation-rigor.md` + ADR-0297..0310 §E. P2-LANE-ROSTER.

---

## §12 — Risks and dependencies for the Wave-3-E remediation agent

### §12.1 Coordinator-lock and in-flight cap

Per `feedback_pipeline_clog_gotchas_2026_05_17`, the foundry pipeline caps in-flight PRs at 12 and the coordinator locks the shared crate. Wave-3-E's 145-PR workload at 12 in-flight = ~13 wash cycles. **Wave-3-E should run sequentially per-µservice batch, not parallel across µservices.**

### §12.2 No silent regression (Linus-style)

Per `feedback_no_silent_regression`, public contracts can't change silently. Any IP that updates `acceptance_lanes:` to a renamed lane needs the lane-roster update committed first; otherwise the lane reference is broken.

### §12.3 Multispectrum-review v2.4.0 mandate

Per `feedback_multispectrum_review_v22` → v2.4.0 superseding entry: every PR in the Wave-3-E sequence MUST carry the 11-13 facet (F1-F9 + M1+M2 + F10/F11/F13) evidence under `evidence/debate/`. The IP-rewrite agent is not exempt.

### §12.4 Canonical-base + localization (KR pack)

Per `feedback_canonical_base_localization`, every µservice MUST keep its canonical base neutral and overlay localizations as packs. The Wave-3-E rewrite of `phase: P01-translate-platform` → `phase: P01-translate-shared` must NOT change the localization story for KR / EU-sov / CN-PIPL packs.

### §12.5 Cell-tier / compliance-pack / sovereign-cloud variant documentation

Per documentation-rigor.md §1.1 completeness invariant #8: every primitive declares pack-tier-cell-tier coverage. Many of the rebinds need an explicit pack-overlay paragraph. **Wave-3-E rewrite should not erase existing pack-overlay statements; the audit found ~3-5 IPs that already include them.**

### §12.6 Doctrinal precedence

Per `feedback_bominal_inheritance_precedence`, oyatie session decisions OVERRIDE Bominal-inherited ADRs. Wave-3-E should treat the keystone bundle 2026-05-20 (ADR-0242..0258) as the authoritative override over any pre-2026-05 Bominal-inherited foundry-as-single-microservice (ADR-0136) language.

### §12.7 Autonomous-implementation goal

Per `feedback_autonomous_implementation_artifacts`, the long-term target is "Implement the masterplan" running without user intervention. Wave-3-E's IP rewrites are the precondition — without `depends_on:` populated and the keystone bundle bound, the executor agent cannot pick the next ChangeSet autonomously.

---

## §13 — Audit completeness self-check

- [x] Scope verified: 921 IP files (recursive), 46 µservices
- [x] All ten sections from the user prompt produced (§1 Scope, §2 stale, §3 binding gap, §4 lane rename, §5 changeset shape, §6 DAG, §7 rigor, §8 critical-path, §9 supersede, §10 remediation order)
- [x] §11 top-30 P0/P1 findings produced
- [x] No IP files were edited
- [x] No ADRs / standards / synthesis docs were edited
- [x] No template files were edited
- [x] Output >2000 lines (target met)
- [x] All file paths quoted; line numbers cited where available
- [x] Per-µservice tables produced for the three primary axes (keystone binding, changeset shape, critical-path coverage)

### §13.1 Open questions for governance

The audit raises (does not resolve) the following governance questions for the lane-roster owner + ADR-0258 versioning-policy owner:

1. **`retired VCS ratchet` lane name** — does the [[git-canonical-2026-05-18]] doctrine retain the `retired VCS ratchet-` lane prefix (since the lane predates the verb rename) or rename to `governance-promotion-readiness`?
2. **Foundry IPs vs intelligence µservice** — per ADR-0255 KS#14 the intelligence two-layer substrate absorbs Foundry. Should the 101 foundry IPs migrate to `microservices/intelligence/`?
3. **IP-NEW-* / IP-WASMTIME-* / IP-GITOPS-* renaming numeric slots** — what's the canonical numeric range to assign? (Reserve the IP-NEW-* set for the next available slots in the µservice's sequence, or use a separate IP-NNNN-extended range?)
4. **Acceptance-lane roster updates** — the 25 candidate lane names found in IPs but not in the documentation-rigor.md roster: add to roster, or rename in IPs?
5. **Non-flat layout flattening order** — flatten analytics/specs/ + developer-sdk/implementation-plans/ + finops-portal/implementation-plans/ + plugin-app-store/implementation-plans/ in one PR per µservice, or batch?

### §13.2 Hand-off

This audit produces a remediation punch list, not edits. Wave-3-E executes from this list. Wave-3-F's reviewer-agent verdict cycle (per `feedback_self_merge_via_contract_path`) gates each Wave-3-E PR.

The Wave-3-E agent SHOULD enter with the following session-state:

- ICM memory recalls: [[oyatie-is-a-tenant-doctrine]], [[git-canonical-2026-05-18]], [[layer-enum-adr-0105-13-canonical]], [[cedar-as-universal-gate]], [[amazon-shape-cellular-architecture]], [[build-ahead-of-certification]], [[bominal-inheritance-precedence]], [[no-silent-regression]].
- Document reads pre-loaded: `docs/standards/documentation-rigor.md`, `docs/architecture/keystone-bundle-2026-05-20-synthesis.md`, `docs/architecture/foundry-fitness-to-governance-transition-2026-05-21.md`, `docs/templates/implementation-plan-template.md`, `microservices/observability/IP-001-layer-a-grafana-stack-iac.md`, `microservices/payments/IP-001-payments-kernel-charge.md`, `microservices/intelligence/IP-011-adapter-anthropic.md`.
- Worktree pinned to a Wave-3-E branch off `dev`; each per-µservice batch a separate PR.

---

## §14 — Appendix: per-µservice one-line gap summary

```
analytics                    total=15  ks=0   f5=0  cp=0  contract=0/15  CB=0/15  CFT=0/15  Accept=15/15 Verify=0/15  Lanes=0/15   Deps=0/15   non-flat=YES
anonymous                    total=15  ks=0   f5=0  cp=0  contract=1/15  CB=2/15  CFT=0/15  Accept=15/15 Verify=0/15  Lanes=15/15  Deps=0/15
api-gateway                  total=18  ks=2   f5=2  cp=3  contract=0/18  CB=0/18  CFT=0/18  Accept=1/18  Verify=0/18  Lanes=0/18   Deps=0/18
application                  total=16  ks=0   f5=0  cp=0  contract=1/16  CB=1/16  CFT=15/16 Accept=16/16 Verify=15/16 Lanes=15/16  Deps=0/16
audit-chain                  total=15  ks=0   f5=0  cp=0  contract=2/15  CB=2/15  CFT=11/15 Accept=15/15 Verify=3/15  Lanes=15/15  Deps=1/15
calendar                     total=15  ks=0   f5=0  cp=0  contract=15/15 CB=15/15 CFT=15/15 Accept=15/15 Verify=15/15 Lanes=15/15  Deps=0/15
cell                         total=15  ks=0   f5=0  cp=0  contract=2/15  CB=2/15  CFT=14/15 Accept=15/15 Verify=14/15 Lanes=15/15  Deps=1/15
cloud-iac                    total=26  ks=0   f5=0  cp=0  contract=15/26 CB=15/26 CFT=15/26 Accept=26/26 Verify=15/26 Lanes=15/26  Deps=1/26
cloud-k8s                    total=19  ks=0   f5=0  cp=0  contract=3/19  CB=15/19 CFT=15/19 Accept=19/19 Verify=15/19 Lanes=15/19  Deps=0/19
cloud-secrets                total=15  ks=0   f5=0  cp=0  contract=1/15  CB=15/15 CFT=15/15 Accept=15/15 Verify=15/15 Lanes=15/15  Deps=0/15
comms-email                  total=26  ks=7   f5=0  cp=0  contract=0/26  CB=0/26  CFT=0/26  Accept=26/26 Verify=0/26  Lanes=0/26   Deps=0/26
community                    total=15  ks=0   f5=0  cp=0  contract=0/15  CB=0/15  CFT=0/15  Accept=15/15 Verify=0/15  Lanes=0/15   Deps=0/15
compliance                   total=26  ks=10  f5=2  cp=0  contract=0/26  CB=0/26  CFT=0/26  Accept=22/26 Verify=0/26  Lanes=0/26   Deps=0/26
connect                      total=15  ks=12  f5=5  cp=1  contract=0/15  CB=0/15  CFT=0/15  Accept=15/15 Verify=0/15  Lanes=0/15   Deps=0/15
consent-graph                total=15  ks=0   f5=0  cp=0  contract=0/15  CB=0/15  CFT=0/15  Accept=0/15  Verify=0/15  Lanes=0/15   Deps=0/15
developer-sdk                total=15  ks=0   f5=0  cp=0  contract=15/15 CB=15/15 CFT=15/15 Accept=15/15 Verify=15/15 Lanes=15/15  Deps=0/15  non-flat=YES
docs                         total=20  ks=0   f5=0  cp=0  contract=3/20  CB=15/20 CFT=15/20 Accept=20/20 Verify=3/20  Lanes=15/20  Deps=0/20
drive                        total=15  ks=0   f5=0  cp=0  contract=1/15  CB=2/15  CFT=5/15  Accept=15/15 Verify=1/15  Lanes=15/15  Deps=0/15
feature-flags                total=27  ks=26  f5=7  cp=8  contract=0/27  CB=0/27  CFT=0/27  Accept=1/27  Verify=0/27  Lanes=0/27   Deps=0/27
finops-portal                total=26  ks=8   f5=0  cp=0  contract=0/26  CB=0/26  CFT=0/26  Accept=26/26 Verify=15/26 Lanes=0/26   Deps=14/26 non-flat=YES
forms                        total=15  ks=0   f5=0  cp=0  contract=1/15  CB=2/15  CFT=15/15 Accept=15/15 Verify=2/15  Lanes=15/15  Deps=0/15
foundry                      total=101 ks=0   f5=0  cp=0  contract=51/101 CB=67/101 CFT=75/101 Accept=91/101 Verify=74/101 Lanes=90/101 Deps=13/101 ADR-0136-stale=4
governance                   total=22  ks=0   f5=0  cp=0  contract=19/22 CB=18/22 CFT=18/22 Accept=22/22 Verify=15/22 Lanes=19/22  Deps=1/22  IP-NEW/WASMTIME=6
identity                     total=17  ks=0   f5=0  cp=0  contract=0/17  CB=0/17  CFT=0/17  Accept=16/17 Verify=1/17  Lanes=0/17   Deps=0/17
intelligence                 total=26  ks=18  f5=7  cp=0  contract=2/26  CB=2/26  CFT=25/26 Accept=26/26 Verify=11/26 Lanes=25/26  Deps=0/26  ADR-0136-stale=1
mail                         total=18  ks=2   f5=0  cp=1  contract=15/18 CB=15/18 CFT=15/18 Accept=15/18 Verify=15/18 Lanes=15/18  Deps=0/18
meet                         total=15  ks=0   f5=0  cp=0  contract=15/15 CB=2/15  CFT=15/15 Accept=15/15 Verify=15/15 Lanes=15/15  Deps=0/15
messenger                    total=16  ks=0   f5=0  cp=0  contract=16/16 CB=6/16  CFT=16/16 Accept=16/16 Verify=16/16 Lanes=16/16  Deps=0/16
network                      total=15  ks=0   f5=0  cp=0  contract=15/15 CB=4/15  CFT=5/15  Accept=15/15 Verify=15/15 Lanes=15/15  Deps=0/15
notes                        total=18  ks=2   f5=0  cp=1  contract=1/18  CB=1/18  CFT=3/18  Accept=15/18 Verify=6/18  Lanes=15/18  Deps=0/18
observability                total=26  ks=0   f5=0  cp=0  contract=15/26 CB=6/26  CFT=15/26 Accept=26/26 Verify=20/26 Lanes=15/26  Deps=0/26
ontology                     total=23  ks=3   f5=1  cp=1  contract=1/23  CB=0/23  CFT=0/23  Accept=1/23  Verify=14/23 Lanes=15/23  Deps=15/23
ops-dashboard-control-center total=16  ks=8   f5=4  cp=1  contract=0/16  CB=0/16  CFT=0/16  Accept=16/16 Verify=7/16  Lanes=0/16   Deps=0/16
payments                     total=18  ks=18  f5=2  cp=0  contract=0/18  CB=0/18  CFT=0/18  Accept=18/18 Verify=0/18  Lanes=0/18   Deps=0/18  fraud-detect-gap=YES
plugin-app-store             total=15  ks=0   f5=0  cp=0  contract=15/15 CB=15/15 CFT=15/15 Accept=15/15 Verify=15/15 Lanes=15/15  Deps=0/15  non-flat=YES
recordings                   total=15  ks=0   f5=0  cp=0  contract=2/15  CB=3/15  CFT=1/15  Accept=15/15 Verify=0/15  Lanes=15/15  Deps=0/15
sheets                       total=15  ks=0   f5=0  cp=0  contract=1/15  CB=15/15 CFT=3/15  Accept=15/15 Verify=15/15 Lanes=15/15  Deps=14/15
shorts                       total=15  ks=0   f5=0  cp=0  contract=2/15  CB=15/15 CFT=15/15 Accept=15/15 Verify=5/15  Lanes=7/15   Deps=12/15
sites                        total=15  ks=0   f5=0  cp=0  contract=1/15  CB=15/15 CFT=1/15  Accept=15/15 Verify=10/15 Lanes=15/15  Deps=0/15
slides                       total=15  ks=0   f5=0  cp=0  contract=15/15 CB=15/15 CFT=15/15 Accept=15/15 Verify=6/15  Lanes=15/15  Deps=15/15
social                       total=18  ks=2   f5=0  cp=1  contract=15/18 CB=14/18 CFT=15/18 Accept=15/18 Verify=15/18 Lanes=15/18  Deps=0/18
tasks                        total=15  ks=0   f5=0  cp=0  contract=15/15 CB=15/15 CFT=15/15 Accept=15/15 Verify=15/15 Lanes=15/15  Deps=0/15
tenancy                      total=26  ks=11  f5=0  cp=0  contract=2/26  CB=3/26  CFT=15/26 Accept=25/26 Verify=15/26 Lanes=15/26  Deps=0/26
translate                    total=15  ks=0   f5=0  cp=0  contract=1/15  CB=15/15 CFT=2/15  Accept=2/15  Verify=15/15 Lanes=15/15  Deps=0/15  platform→shared phase rename=15
workflow-engine              total=15  ks=0   f5=0  cp=0  contract=15/15 CB=14/15 CFT=14/15 Accept=11/15 Verify=14/15 Lanes=15/15  Deps=1/15  retired VCS ratchet=2
workflow-studio              total=27  ks=0   f5=0  cp=0  contract=27/27 CB=15/27 CFT=15/27 Accept=27/27 Verify=15/27 Lanes=27/27  Deps=24/27 retired VCS ratchet=12
```

Reading the table: each column = "IPs that have the property" out of "total IPs". Higher is better. The gap to close = total – column-value.

---

End of audit. No IPs were edited. Wave-3-E execution plan in §10. P0/P1 punch list in §11.

---

## §15 — Per-IP enumeration (921 rows, line-anchored)

Single row per IP. Format: file path (line count) + tag list. Tags:

- `noContract` — front-matter lacks `changeset_contract:`
- `noCB` — body lacks `## ChangeSet boundary` section
- `noCFT` — body lacks `## Concrete File Targets` section
- `noAccept` — body lacks `## Acceptance` / `## Acceptance Gates` section
- `noVerify` — body lacks `## Verification` / `## Test Plan` section
- `noDeps` — front-matter lacks `depends_on:`
- `noLanes` — front-matter lacks `acceptance_lanes:`
- `noKS` — IP body does NOT cite any keystone-bundle ADR in 0242..0258 range
- `F5` — IP cites at least one F5-CRITICAL fix ADR (0293-0296) — POSITIVE signal
- `CP` — IP cites at least one critical-path cluster ADR (0297-0310) — POSITIVE signal
- `ADR-0136-STALE` — IP cites the superseded ADR-0136
- `retired VCS ratchet` — IP body or front-matter contains `retired VCS ratchet` / `retired VCS ratchet` (verb rename pending)
- `<50lines` or `<100lines` — IP is below the exemplar floor

Sorted by µservice, then by IP number. Wave-3-E iterates this list as the work-queue.


### §15.analytics — `microservices/analytics/`

- `analytics/specs/IP-001-clickhouse-cluster-iac.md` (     247L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `analytics/specs/IP-002-per-tenant-database-bootstrap.md` (     359L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `analytics/specs/IP-003-olap-client-adapter-scaffold.md` (     374L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `analytics/specs/IP-004-outbox-cdc-ingest-pipeline.md` (     243L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `analytics/specs/IP-005-materialized-view-canon.md` (     297L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `analytics/specs/IP-006-cold-tier-s3-ttl.md` (     263L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `analytics/specs/IP-007-tenant-dashboard-api.md` (     312L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `analytics/specs/IP-008-audit-log-query-api.md` (     315L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `analytics/specs/IP-009-billing-rollup-pipeline.md` (     272L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `analytics/specs/IP-010-cross-cell-federation.md` (     286L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `analytics/specs/IP-011-per-tenant-quota-enforcement.md` (     256L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `analytics/specs/IP-012-backup-restore-drill.md` (     259L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `analytics/specs/IP-013-regulator-export-evidence-pack.md` (     260L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `analytics/specs/IP-014-self-slo-burn-rate-alerts.md` (     208L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `analytics/specs/IP-015-app-composition-root.md` (     378L) noContract noCB noCFT noVerify noDeps noLanes noKS

### §15.anonymous — `microservices/anonymous/`

- `anonymous/IP-001-iac-bootstrap.md` (      51L) noCFT noVerify noDeps noKS <100lines
- `anonymous/IP-002-cargo-workspace-kernels.md` (      49L) noContract noCFT noVerify noDeps noKS <50lines
- `anonymous/IP-003-domain-crates-per-bc.md` (      36L) noContract noCB noCFT noVerify noDeps noKS <50lines
- `anonymous/IP-004-postgres-adapters-blinding-migration.md` (      55L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `anonymous/IP-005-redis-cache.md` (      37L) noContract noCB noCFT noVerify noDeps noKS <50lines
- `anonymous/IP-006-affinity-attestation-bc.md` (      34L) noContract noCB noCFT noVerify noDeps noKS <50lines
- `anonymous/IP-007-blind-signatures-crypto.md` (      37L) noContract noCB noCFT noVerify noDeps noKS <50lines
- `anonymous/IP-008-post-store-bc.md` (      32L) noContract noCB noCFT noVerify noDeps noKS <50lines
- `anonymous/IP-009-vote-engine-bc.md` (      31L) noContract noCB noCFT noVerify noDeps noKS <50lines
- `anonymous/IP-010-abuse-classifier-wire.md` (      33L) noContract noCB noCFT noVerify noDeps noKS <50lines
- `anonymous/IP-011-legal-process-workflow.md` (      28L) noContract noCB noCFT noVerify noDeps noKS <50lines
- `anonymous/IP-012-retention-policy-worker.md` (      26L) noContract noCB noCFT noVerify noDeps noKS <50lines
- `anonymous/IP-013-hard-delete-propagation-chain.md` (      35L) noContract noCB noCFT noVerify noDeps noKS <50lines
- `anonymous/IP-014-rest-api-openapi-sdk.md` (      26L) noContract noCB noCFT noVerify noDeps noKS <50lines
- `anonymous/IP-015-hg-anonymous-registration-branch-protection.md` (      43L) noContract noCB noCFT noVerify noDeps noKS retired VCS ratchet <50lines

### §15.apigateway — `microservices/api-gateway/`

- `api-gateway/IP-001-api-gateway-design-readiness.md` (      27L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `api-gateway/IP-002-routing-domain-crate.md` (      66L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <100lines
- `api-gateway/IP-003-routing-kernel-crate.md` (      44L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <50lines
- `api-gateway/IP-004-routing-usecase-crate.md` (      35L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <50lines
- `api-gateway/IP-005-routing-adapter-crate.md` (      22L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <50lines
- `api-gateway/IP-006-routing-rest-crate.md` (      23L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <50lines
- `api-gateway/IP-007-routing-grpc-crate.md` (      18L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <50lines
- `api-gateway/IP-008-routing-worker-crate.md` (      14L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <50lines
- `api-gateway/IP-009-rate-limit-domain-crate.md` (      32L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <50lines
- `api-gateway/IP-010-rate-limit-adapter-redis.md` (      15L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <50lines
- `api-gateway/IP-011-auth-handoff-usecase.md` (      14L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <50lines
- `api-gateway/IP-012-abuse-defence-domain.md` (      33L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS CP <50lines
- `api-gateway/IP-013-abuse-defence-adapter-wasm.md` (      20L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS CP <50lines
- `api-gateway/IP-014-tls-cert-rotation-worker.md` (      16L) noContract noCB noCFT noAccept noVerify noDeps noLanes F5 <50lines
- `api-gateway/IP-015-canary-cohort-shifter.md` (      16L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS F5 <50lines
- `api-gateway/IP-016-app-supervisor.md` (      15L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <50lines
- `api-gateway/IP-017-sov-cell-routing.md` (      22L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `api-gateway/IP-018-honeypot-route-mgr.md` (      16L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS CP <50lines

### §15.application — `microservices/application/`

- `application/IP-001-shell-routing-kernel.md` (     152L) noDeps noKS
- `application/IP-002-shell-routing-domain.md` (      93L) noContract noCB noDeps noKS <100lines
- `application/IP-003-shell-routing-usecase.md` (      88L) noContract noCB noDeps noKS <100lines
- `application/IP-004-shell-routing-adapter.md` (      88L) noContract noCB noDeps noKS <100lines
- `application/IP-005-shell-routing-rest.md` (      68L) noContract noCB noDeps noKS <100lines
- `application/IP-006-tenant-context-kernel.md` (      75L) noContract noCB noDeps noKS <100lines
- `application/IP-007-tenant-context-usecase-rest.md` (      72L) noContract noCB noDeps noKS <100lines
- `application/IP-008-auth-gateway-kernel-domain.md` (      80L) noContract noCB noDeps noKS <100lines
- `application/IP-009-auth-gateway-adapters-oidc-saml.md` (      80L) noContract noCB noDeps noKS <100lines
- `application/IP-010-auth-gateway-rest-worker.md` (      84L) noContract noCB noDeps noKS <100lines
- `application/IP-011-module-loader-kernel-domain.md` (      85L) noContract noCB noDeps noKS <100lines
- `application/IP-012-module-loader-usecase-adapter-cdn.md` (      95L) noContract noCB noDeps noKS <100lines
- `application/IP-013-frontend-bundle-serve.md` (      82L) noContract noCB noDeps noKS <100lines
- `application/IP-014-leptos-frontend-and-composition.md` (     100L) noContract noCB noDeps noKS
- `application/IP-015-application-openslo-and-hg-app.md` (     122L) noContract noCB noDeps noKS
- `application/IP-016-tenant-admin-console-control-surface.md` (      28L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines

### §15.auditchain — `microservices/audit-chain/`

- `audit-chain/IP-001-storage-backend-iac.md` (      68L) noVerify noDeps noKS <100lines
- `audit-chain/IP-002-self-slo-manifest.md` (     151L) noCB noCFT noVerify noKS retired VCS ratchet
- `audit-chain/IP-003-emission-kernel.md` (     134L) noContract noCB noDeps noKS
- `audit-chain/IP-004-emission-domain.md` (      41L) noContract noCB noVerify noDeps noKS <50lines
- `audit-chain/IP-005-emission-usecase-and-adapter.md` (      97L) noContract noCB noDeps noKS <100lines
- `audit-chain/IP-006-sealing-kernel.md` (      71L) noContract noCB noVerify noDeps noKS <100lines
- `audit-chain/IP-007-sealing-domain-merkle.md` (      73L) noContract noCB noVerify noDeps noKS <100lines
- `audit-chain/IP-008-sealing-adapter-hsm.md` (      81L) noContract noCB noVerify noDeps noKS <100lines
- `audit-chain/IP-009-sealing-adapter-postgres-s3.md` (      96L) noContract noCB noVerify noDeps noKS <100lines
- `audit-chain/IP-010-sealing-worker-app.md` (      77L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `audit-chain/IP-011-verification-stack.md` (      76L) noContract noCB noCFT noDeps noKS <100lines
- `audit-chain/IP-012-query-stack.md` (      75L) noContract noCB noVerify noDeps noKS <100lines
- `audit-chain/IP-013-retention-cascade.md` (      95L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `audit-chain/IP-014-cross-microservice-emission-adapter.md` (      56L) noContract noVerify noDeps noKS <100lines
- `audit-chain/IP-015-self-observability-slo-wiring.md` (      75L) noContract noCB noVerify noDeps noKS <100lines

### §15.calendar — `microservices/calendar/`

- `calendar/IP-001-iac-bootstrap.md` (      98L) noDeps noKS <100lines
- `calendar/IP-002-event-store-kernel.md` (     108L) noDeps noKS
- `calendar/IP-003-event-store-domain-and-usecase.md` (      80L) noDeps noKS <100lines
- `calendar/IP-004-event-store-adapter-postgres.md` (      70L) noDeps noKS <100lines
- `calendar/IP-005-recurrence-engine.md` (      76L) noDeps noKS <100lines
- `calendar/IP-006-availability-resolver.md` (      69L) noDeps noKS <100lines
- `calendar/IP-007-room-booking.md` (      64L) noDeps noKS <100lines
- `calendar/IP-008-invitation-flow.md` (      71L) noDeps noKS <100lines
- `calendar/IP-009-ics-import-export-and-caldav.md` (      79L) noDeps noKS <100lines
- `calendar/IP-010-tzdb-refresh-worker.md` (      69L) noDeps noKS <100lines
- `calendar/IP-011-contracts-openapi-asyncapi-proto.md` (      68L) noDeps noKS <100lines
- `calendar/IP-012-cedar-policies-and-data-residency.md` (      78L) noDeps noKS <100lines
- `calendar/IP-013-workflow-handoff.md` (      73L) noDeps noKS <100lines
- `calendar/IP-014-hg-calendar-authority-cohesion.md` (      62L) noDeps noKS <100lines
- `calendar/IP-015-hg-calendar-registration-and-branch-protection.md` (      82L) noDeps noKS <100lines

### §15.cell — ADR-0333 successor ownership

- `cell/IP-001-host-pool-iac.md` (      88L) noDeps noKS <100lines
- `cell/IP-002-cell-registry-postgres-schema.md` (     134L) noContract noDeps noKS
- `cell/IP-003-cell-registry-kernel.md` (     141L) noContract noCB noDeps noKS
- `cell/IP-004-cell-registry-domain-usecase.md` (     115L) noContract noCB noDeps noKS
- `cell/IP-005-cell-registry-adapter-postgres-rest-sdk-app.md` (     104L) noContract noCB noDeps noKS
- `cell/IP-006-cell-boundary-gate-lane.md` (     141L) noCB noCFT noVerify noKS retired VCS ratchet
- `cell/IP-007-scheduler-binpack.md` (     118L) noContract noCB noDeps noKS
- `cell/IP-008-lifecycle-manager-k8s.md` (     109L) noContract noCB noDeps noKS
- `cell/IP-009-tenant-assignment-stack.md` (     130L) noContract noCB noDeps noKS
- `cell/IP-010-tenant-migration-orchestrator.md` (     119L) noContract noCB noDeps noKS
- `cell/IP-011-host-pool-drain-primitive.md` (      94L) noContract noCB noDeps noKS <100lines
- `cell/IP-012-cell-registry-events-emitter.md` (      82L) noContract noCB noDeps noKS <100lines
- `cell/IP-013-observability-slo-manifests.md` (     125L) noContract noCB noDeps noKS retired VCS ratchet
- `cell/IP-014-branch-protection-gate-registration.md` (      95L) noContract noCB noDeps noKS retired VCS ratchet <100lines
- `cell/IP-015-hyperscaler-claim-gate.md` (      92L) noContract noCB noDeps noKS <100lines

### §15.cloudiac — `microservices/cloud-iac/`

- `cloud-iac/implementation-plans/IP-seaweedfs-cluster-bootstrap.md` (      68L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `cloud-iac/implementation-plans/IP-seaweedfs-signed-url-substrate.md` (      51L) noContract noCB noCFT noVerify noLanes noKS <100lines
- `cloud-iac/implementation-plans/IP-velero-pgbackrest-restic-bootstrap.md` (      55L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `cloud-iac/IP-001-layer-a-argocd-flux-iac.md` (     107L) noDeps noKS
- `cloud-iac/IP-002-layer-a-opentofu-iac.md` (     106L) noDeps noKS
- `cloud-iac/IP-003-iac-renderer-kernel.md` (     163L) noDeps noKS
- `cloud-iac/IP-004-iac-renderer-domain-usecase.md` (     119L) noDeps noKS
- `cloud-iac/IP-005-iac-renderer-adapter-trio.md` (     112L) noDeps noKS
- `cloud-iac/IP-006-iac-validator-kernel-domain-usecase.md` (     108L) noDeps noKS
- `cloud-iac/IP-007-iac-applier-kernel-domain-usecase.md` (     122L) noDeps noKS
- `cloud-iac/IP-008-iac-registry-postgres.md` (     131L) noDeps noKS
- `cloud-iac/IP-009-iac-rollback-engine.md` (     110L) noDeps noKS
- `cloud-iac/IP-010-rest-surfaces.md` (      98L) noDeps noKS <100lines
- `cloud-iac/IP-011-worker-binaries.md` (     113L) noDeps noKS
- `cloud-iac/IP-012-app-composition-roots.md` (      95L) noDeps noKS <100lines
- `cloud-iac/IP-013-sdk-and-observability-slo.md` (     168L) noDeps noKS
- `cloud-iac/IP-014-per-pack-iac-overlays.md` (     121L) noDeps noKS
- `cloud-iac/IP-015-hg-cloud-iac-registration.md` (     123L) noDeps noKS
- `cloud-iac/IP-GITOPS-001-terraform-to-opentofu-migration.md` (      83L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `cloud-iac/IP-GITOPS-002-argocd-app-of-apps-pattern.md` (      58L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `cloud-iac/IP-GITOPS-003-tier-discipline-rollout.md` (      44L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `cloud-iac/IP-GITOPS-004-opentofu-module-registry-bootstrap.md` (      46L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `cloud-iac/IP-GITOPS-005-drift-detection.md` (      43L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `cloud-iac/IP-GITOPS-006-secret-bootstrap-tier-b.md` (      40L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `cloud-iac/IP-GITOPS-007-namespace-bootstrap-tier-b.md` (      43L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `cloud-iac/IP-GITOPS-008-argocd-project-bootstrap.md` (      46L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines

### §15.cloudk8s — `microservices/cloud-k8s/`

- `cloud-k8s/implementation-plans/IP-karpenter-bootstrap.md` (      62L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `cloud-k8s/IP-001-layer-a-iac-kubeadm-containerd-istio-envoy.md` (     164L) noDeps noKS
- `cloud-k8s/IP-002-onprem-k8s-stack-standard.md` (     101L) noDeps noKS
- `cloud-k8s/IP-003-cluster-bootstrap-kernel.md` (     167L) noDeps noKS
- `cloud-k8s/IP-004-cluster-bootstrap-domain.md` (      95L) noContract noDeps noKS <100lines
- `cloud-k8s/IP-005-cluster-bootstrap-usecase.md` (     118L) noContract noDeps noKS
- `cloud-k8s/IP-006-cluster-bootstrap-adapter-kubeadm.md` (     110L) noContract noDeps noKS
- `cloud-k8s/IP-007-node-lifecycle-kernel-usecase.md` (     111L) noContract noDeps noKS
- `cloud-k8s/IP-008-network-policy-kernel-usecase.md` (     108L) noContract noDeps noKS
- `cloud-k8s/IP-009-service-mesh-control-plane-istio.md` (     106L) noContract noDeps noKS
- `cloud-k8s/IP-010-ingress-controller-envoy.md` (      91L) noContract noDeps noKS <100lines
- `cloud-k8s/IP-011-csi-storage-driver-per-backend.md` (     108L) noContract noDeps noKS
- `cloud-k8s/IP-012-kubernetes-api-proxy.md` (     101L) noContract noDeps noKS
- `cloud-k8s/IP-013-cluster-bootstrap-rest-worker-sdk-app.md` (     124L) noContract noDeps noKS
- `cloud-k8s/IP-014-branch-protection-and-hyperscaler-gate.md` (     116L) noContract noDeps noKS
- `cloud-k8s/IP-015-observability-slo-and-authority-cohesion.md` (     155L) noContract noDeps noKS retired VCS ratchet
- `cloud-k8s/IP-CLUSTERAPI-001-clusterclass-templates.md` (      45L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `cloud-k8s/IP-CLUSTERAPI-002-cluster-lifecycle-orchestration.md` (      41L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `cloud-k8s/IP-CLUSTERAPI-003-cluster-promotion-pipeline.md` (      36L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines

### §15.cloudsecrets — `microservices/cloud-secrets/`

- `cloud-secrets/IP-001-layer-a-openbao-postgres-hsm-iac.md` (     143L) noDeps noKS
- `cloud-secrets/IP-002-secretreference-uri-spec.md` (      66L) noContract noDeps noKS <100lines
- `cloud-secrets/IP-003-resolver-kernel.md` (     173L) noContract noDeps noKS
- `cloud-secrets/IP-004-resolver-domain.md` (      75L) noContract noDeps noKS <100lines
- `cloud-secrets/IP-005-resolver-usecase.md` (      83L) noContract noDeps noKS <100lines
- `cloud-secrets/IP-006-resolver-adapter-openbao.md` (      57L) noContract noDeps noKS <100lines
- `cloud-secrets/IP-007-resolver-rest-and-sdk-rust.md` (      77L) noContract noDeps noKS <100lines
- `cloud-secrets/IP-008-sdk-ts-python-bindings.md` (      75L) noContract noDeps noKS <100lines
- `cloud-secrets/IP-009-openbao-operator.md` (      84L) noContract noDeps noKS <100lines
- `cloud-secrets/IP-010-key-rotation-scheduler-worker.md` (      68L) noContract noDeps noKS <100lines
- `cloud-secrets/IP-011-hsm-integration-adapter-hsm.md` (      68L) noContract noDeps noKS <100lines
- `cloud-secrets/IP-012-per-tenant-namespace-controller.md` (      57L) noContract noDeps noKS <100lines
- `cloud-secrets/IP-013-audit-emitter-bridge-to-audit-chain.md` (      56L) noContract noDeps noKS <100lines
- `cloud-secrets/IP-014-observability-slo-branch-protection-hg-cloud-secrets.md` (      55L) noContract noDeps noKS <100lines
- `cloud-secrets/IP-015-lean-a11-raw-secret-emission-lane-wiring.md` (     100L) noContract noDeps noKS

### §15.commsemail — `microservices/comms-email/`

- `comms-email/IP-001-ses-adapter-impl.md` (     137L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `comms-email/IP-002-postal-adapter-impl.md` (     131L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `comms-email/IP-003-smtp-fallback-adapter-impl.md` (     124L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `comms-email/IP-004-mailgun-adapter-impl.md` (     115L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `comms-email/IP-005-dkim-key-rotation-pipeline.md` (     129L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `comms-email/IP-006-mjml-template-renderer.md` (     110L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `comms-email/IP-007-liquid-substitution-engine.md` (     115L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `comms-email/IP-008-webhook-delivery-pipeline.md` (     106L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `comms-email/IP-009-bounce-complaint-handler.md` (      97L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `comms-email/IP-010-suppression-list.md` (     108L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `comms-email/IP-011-per-tenant-from-domain-onboarding.md` (     112L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `comms-email/IP-012-audit-chain-emission.md` (      99L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `comms-email/IP-013-multi-region-routing.md` (      89L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `comms-email/IP-014-sovereign-pack-postal-only-enforcement.md` (      77L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `comms-email/IP-015-in-house-relay-roadmap-phase-2.md` (     104L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `comms-email/IP-016-inbound-receiver-kernel.md` (      24L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `comms-email/IP-017-inbound-receiver-domain.md` (      24L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `comms-email/IP-018-list-management-usecase.md` (      25L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `comms-email/IP-019-unsubscribe-handler-domain.md` (      26L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `comms-email/IP-020-reputation-monitor-worker.md` (      25L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `comms-email/IP-021-bounce-handler-domain.md` (      25L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `comms-email/IP-022-template-rendering-mjml-engine.md` (      25L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `comms-email/IP-023-inbound-receiver-rest.md` (      24L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `comms-email/IP-024-list-management-rest.md` (      25L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `comms-email/IP-025-reputation-monitor-rest-and-dashboard.md` (      25L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `comms-email/IP-026-unsubscribe-async-emit.md` (      25L) noContract noCB noCFT noVerify noDeps noLanes <50lines

### §15.community — `microservices/community/`

- `community/IP-001-postgres-citus-post-store-iac.md` (      53L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `community/IP-002-post-store-kernel-domain.md` (      44L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `community/IP-003-post-store-usecase-api.md` (      44L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `community/IP-004-post-store-adapter-postgres-rest-worker-sdk-app.md` (      45L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `community/IP-005-thread-tree-materialised-path.md` (      42L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `community/IP-006-voting-engine.md` (      45L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `community/IP-007-moderation-queue.md` (      45L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `community/IP-008-kb-article-store-s3.md` (      45L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `community/IP-009-search-index-elasticsearch.md` (      45L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `community/IP-010-foundry-guardrails-moderation-bridge.md` (      45L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `community/IP-011-cedar-policy-fragments.md` (      46L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `community/IP-012-openslo-grafana-dashboards.md` (      43L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `community/IP-013-retired-vcs-ratchet` (      42L) noContract noCB noCFT noVerify noDeps noLanes noKS retired VCS ratchet <50lines
- `community/IP-014-hyperscaler-maturity-gate.md` (      44L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `community/IP-015-capacity-cost-chaos-drill.md` (      46L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines

### §15.compliance — `microservices/compliance/`

- `compliance/IP-001-evidence-collector-bootstrap.md` (      91L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `compliance/IP-002-soc2-control-mapping.md` (      79L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `compliance/IP-003-gdpr-dsar-automation-pipeline.md` (     113L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `compliance/IP-004-hipaa-min-necessary-log-substrate.md` (      94L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `compliance/IP-005-audit-chain-seal-coverage.md` (      71L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `compliance/IP-006-evidence-storage-seaweedfs.md` (      72L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `compliance/IP-007-auditor-readonly-portal.md` (      88L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `compliance/IP-008-pii-scrubber.md` (      75L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `compliance/IP-009-retention-tier-policy.md` (      58L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `compliance/IP-010-attestation-aggregator.md` (      60L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `compliance/IP-011-cross-microservice-evidence-fan-in.md` (      58L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `compliance/IP-012-evidence-replay.md` (      54L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `compliance/IP-013-audit-anomaly-detection.md` (      52L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `compliance/IP-014-manual-evidence-upload-flow.md` (      71L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `compliance/IP-015-regulatory-pack-evidence-overlay.md` (      76L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `compliance/IP-016-pack-registry-kernel.md` (      32L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `compliance/IP-017-pack-registry-domain.md` (      27L) noContract noCB noCFT noVerify noDeps noLanes F5 <50lines
- `compliance/IP-018-dpia-orchestration-usecase.md` (      35L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `compliance/IP-019-breach-notification-workflow.md` (      36L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `compliance/IP-020-regulator-audit-evidence-rest.md` (      35L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `compliance/IP-021-cell-certification-attestation-worker.md` (      31L) noContract noCB noCFT noAccept noVerify noDeps noLanes F5 <50lines
- `compliance/IP-022-compliance-control-mapping-domain.md` (      30L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `compliance/IP-023-pack-registry-grpc.md` (      21L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `compliance/IP-024-dpia-orchestration-adapter-postgres.md` (      23L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `compliance/IP-025-breach-notification-async-emit.md` (      28L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `compliance/IP-026-control-mapping-rest-and-sdk.md` (      28L) noContract noCB noCFT noVerify noDeps noLanes <50lines

### §15.connect — `microservices/connector/`

- `connect/IP-001-connect-retirement-design-readiness.md` (      27L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `connect/IP-002-connector-catalog-domain-kernel.md` (      87L) noContract noCB noCFT noVerify noDeps noLanes <100lines
- `connect/IP-003-oauth-broker-domain-kernel.md` (      68L) noContract noCB noCFT noVerify noDeps noLanes F5 <100lines
- `connect/IP-004-webhook-receiver-domain.md` (      74L) noContract noCB noCFT noVerify noDeps noLanes F5 <100lines
- `connect/IP-005-connector-adapter-domain.md` (      80L) noContract noCB noCFT noVerify noDeps noLanes F5 <100lines
- `connect/IP-006-data-mapping-domain.md` (      35L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `connect/IP-007-retry-dlq-domain.md` (      45L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `connect/IP-008-rest-surfaces.md` (      51L) noContract noCB noCFT noVerify noDeps noLanes <100lines
- `connect/IP-009-connector-catalog-seed.md` (      48L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `connect/IP-010-iac-postgres-openbao.md` (      45L) noContract noCB noCFT noVerify noDeps noLanes F5 <50lines
- `connect/IP-011-slos-dashboards-observability.md` (      60L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `connect/IP-012-wave2-connectors.md` (      51L) noContract noCB noCFT noVerify noDeps noLanes <100lines
- `connect/IP-013-connector-adapter-trait.md` (      71L) noContract noCB noCFT noVerify noDeps noLanes <100lines
- `connect/IP-014-compliance-critical-path.md` (      74L) noContract noCB noCFT noVerify noDeps noLanes F5 CP <100lines
- `connect/IP-015-connector-adapter-trait-doc.md` (      67L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines

### §15.consentgraph — `microservices/consent-graph/`

- `consent-graph/IP-001-agreement-kernel.md` (     242L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `consent-graph/IP-002-agreement-domain.md` (     221L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `consent-graph/IP-003-agreement-usecase-and-adapter.md` (     236L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `consent-graph/IP-004-enforcement-kernel.md` (     194L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `consent-graph/IP-005-enforcement-domain-cedar.md` (     211L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `consent-graph/IP-006-enforcement-usecase-and-adapter.md` (     181L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `consent-graph/IP-007-revocation-kernel-worker.md` (     217L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `consent-graph/IP-008-revocation-pulsar-fanout.md` (     168L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `consent-graph/IP-009-projection-gateway-kernel.md` (     211L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `consent-graph/IP-010-projection-gateway-mint-acl.md` (     205L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `consent-graph/IP-011-projection-scope-narrowing-aggregate.md` (     214L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `consent-graph/IP-012-audit-bridge-bilateral-emitter.md` (     212L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `consent-graph/IP-013-audit-bridge-cross-pointer-integrity.md` (     195L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `consent-graph/IP-014-partner-directory-handshake.md` (     188L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `consent-graph/IP-015-self-observability-slo-wiring.md` (     182L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS

### §15.developersdk — `microservices/developer-sdk/`

- `developer-sdk/implementation-plans/IP-001-layer-a-postgres-openbao-backstage-iac.md` (     140L) noDeps noKS retired VCS ratchet
- `developer-sdk/implementation-plans/IP-002-developer-onboarding-kernel-domain.md` (     161L) noDeps noKS retired VCS ratchet
- `developer-sdk/implementation-plans/IP-003-developer-onboarding-usecase-api-adapter-rest-app.md` (     141L) noDeps noKS retired VCS ratchet
- `developer-sdk/implementation-plans/IP-004-signing-key-issuance-openbao.md` (     150L) noDeps noKS retired VCS ratchet
- `developer-sdk/implementation-plans/IP-005-api-contracts-registry.md` (     159L) noDeps noKS retired VCS ratchet
- `developer-sdk/implementation-plans/IP-006-sdk-codegen-ts-rust-swift-kotlin-csharp-python.md` (     158L) noDeps noKS retired VCS ratchet
- `developer-sdk/implementation-plans/IP-007-sandbox-provisioner-tenant-on-demand.md` (     146L) noDeps noKS retired VCS ratchet
- `developer-sdk/implementation-plans/IP-008-dev-portal-backstage-extension.md` (     153L) noDeps noKS retired VCS ratchet
- `developer-sdk/implementation-plans/IP-009-dev-portal-app-submission-flow.md` (     144L) noDeps noKS retired VCS ratchet
- `developer-sdk/implementation-plans/IP-010-payout-ach-sepa-kftc-fedwire.md` (     164L) noDeps noKS retired VCS ratchet
- `developer-sdk/implementation-plans/IP-011-tax-form-1099-vat-moss-kr-vat.md` (     153L) noDeps noKS retired VCS ratchet
- `developer-sdk/implementation-plans/IP-012-package-registry-vendored.md` (     143L) noDeps noKS retired VCS ratchet
- `developer-sdk/implementation-plans/IP-013-observability-slo-manifests.md` (     150L) noDeps noKS retired VCS ratchet
- `developer-sdk/implementation-plans/IP-014-branch-protection-and-hyperscaler-gates.md` (     135L) noDeps noKS retired VCS ratchet
- `developer-sdk/implementation-plans/IP-015-stripe-connect-parity-end-to-end-drill.md` (     146L) noDeps noKS retired VCS ratchet

### §15.docs — `microservices/docs/`

- `docs/IP-001-iac-bootstrap.md` (      77L) noDeps noKS <100lines
- `docs/IP-002-document-store-kernel.md` (      90L) noDeps noKS <100lines
- `docs/IP-003-document-store-domain-and-usecase.md` (      46L) noContract noVerify noDeps noKS <50lines
- `docs/IP-004-document-store-adapter-postgres-and-s3.md` (      50L) noContract noVerify noDeps noKS <100lines
- `docs/IP-005-block-types-kernel-domain.md` (      50L) noContract noVerify noDeps noKS <100lines
- `docs/IP-006-collab-crdt-kernel-domain.md` (      47L) noContract noVerify noDeps noKS <50lines
- `docs/IP-007-collab-crdt-adapter-redis-worker.md` (      45L) noContract noVerify noDeps noKS <50lines
- `docs/IP-008-comments-and-suggestions.md` (      42L) noContract noVerify noDeps noKS <50lines
- `docs/IP-009-version-history.md` (      44L) noContract noVerify noDeps noKS <50lines
- `docs/IP-010-sharing-and-permissions.md` (      44L) noContract noVerify noDeps noKS <50lines
- `docs/IP-011-export-import.md` (      55L) noContract noVerify noDeps noKS <100lines
- `docs/IP-012-embed-resolver.md` (      46L) noContract noVerify noDeps noKS <50lines
- `docs/IP-013-rest-websocket-protocol.md` (      46L) noContract noVerify noDeps noKS <50lines
- `docs/IP-014-ai-assist-wire.md` (      47L) noContract noVerify noDeps noKS <50lines
- `docs/IP-015-hg-docs-registration-and-branch-protection.md` (      77L) noDeps noKS <100lines
- `docs/IP-DOCS-001-mdbook-techdocs-pipeline.md` (      56L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `docs/IP-DOCS-002-sveltekit-marketing-site.md` (      58L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `docs/IP-DOCS-003-redoc-asyncapi-renderer.md` (      48L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `docs/IP-DOCS-004-mermaid-c4-build.md` (      41L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `docs/IP-DOCS-005-backstage-techdocs-renderer.md` (      45L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines

### §15.drive — `microservices/drive/`

- `drive/IP-001-iac-bootstrap.md` (      85L) noDeps noKS <100lines
- `drive/IP-002-file-store-kernel.md` (      57L) noContract noVerify noDeps noKS <100lines
- `drive/IP-003-file-store-adapters.md` (      45L) noContract noCB noVerify noDeps noKS <50lines
- `drive/IP-004-file-store-rest-worker-sdk-app.md` (      87L) noContract noCB noVerify noDeps noKS <100lines
- `drive/IP-005-folder-hierarchy.md` (      81L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `drive/IP-006-upload.md` (      86L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `drive/IP-007-download.md` (      82L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `drive/IP-008-sync.md` (      85L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `drive/IP-009-share-link.md` (      90L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `drive/IP-010-permissions.md` (      83L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `drive/IP-011-search-index.md` (      84L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `drive/IP-012-preview.md` (      87L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `drive/IP-013-dlp-virus-scan.md` (      84L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `drive/IP-014-immutability-tier.md` (      90L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `drive/IP-015-hg-drive-registration.md` (      49L) noContract noCB noVerify noDeps noKS <50lines

### §15.featureflags — `microservices/feature-flags/`

- `feature-flags/IP-001-feature-flags-design-readiness.md` (      27L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `feature-flags/IP-002-flag-kernel.md` (      38L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `feature-flags/IP-003-flag-domain.md` (      40L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `feature-flags/IP-004-flag-usecase.md` (      34L) noContract noCB noCFT noAccept noVerify noDeps noLanes CP <50lines
- `feature-flags/IP-005-flag-adapter-postgres.md` (      51L) noContract noCB noCFT noAccept noVerify noDeps noLanes <100lines
- `feature-flags/IP-006-targeting-kernel.md` (      45L) noContract noCB noCFT noAccept noVerify noDeps noLanes F5 <50lines
- `feature-flags/IP-007-targeting-domain.md` (      33L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `feature-flags/IP-008-experiment-kernel.md` (      44L) noContract noCB noCFT noAccept noVerify noDeps noLanes CP <50lines
- `feature-flags/IP-009-experiment-domain.md` (      34L) noContract noCB noCFT noAccept noVerify noDeps noLanes CP <50lines
- `feature-flags/IP-010-killswitch-kernel.md` (      45L) noContract noCB noCFT noAccept noVerify noDeps noLanes F5 CP <50lines
- `feature-flags/IP-011-rollout-kernel.md` (      41L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `feature-flags/IP-012-metric-attribution.md` (      41L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `feature-flags/IP-013-openfeature-sdk.md` (      52L) noContract noCB noCFT noAccept noVerify noDeps noLanes <100lines
- `feature-flags/IP-014-rust-sdk.md` (      54L) noContract noCB noCFT noAccept noVerify noDeps noLanes <100lines
- `feature-flags/IP-015-typescript-sdk.md` (      53L) noContract noCB noCFT noAccept noVerify noDeps noLanes <100lines
- `feature-flags/IP-016-python-sdk.md` (      55L) noContract noCB noCFT noAccept noVerify noDeps noLanes <100lines
- `feature-flags/IP-017-iac-terraform.md` (      41L) noContract noCB noCFT noAccept noVerify noDeps noLanes F5 CP <50lines
- `feature-flags/IP-018-cedar-fragments.md` (      44L) noContract noCB noCFT noAccept noVerify noDeps noLanes F5 CP <50lines
- `feature-flags/IP-019-slo-wiring.md` (      56L) noContract noCB noCFT noAccept noVerify noDeps noLanes <100lines
- `feature-flags/IP-020-experiment-stats-engine.md` (      49L) noContract noCB noCFT noAccept noVerify noDeps noLanes CP <50lines
- `feature-flags/IP-021-cedar-schema.md` (      28L) noContract noCB noCFT noAccept noVerify noDeps noLanes F5 <50lines
- `feature-flags/IP-022-grpc-go-sdk.md` (      34L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `feature-flags/IP-023-java-sdk.md` (      34L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `feature-flags/IP-024-dotnet-sdk.md` (      34L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `feature-flags/IP-025-swift-sdk.md` (      36L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `feature-flags/IP-026-killswitch-broadcast-worker.md` (      47L) noContract noCB noCFT noAccept noVerify noDeps noLanes F5 CP <50lines
- `feature-flags/IP-027-pack-overlay-worker.md` (      50L) noContract noCB noCFT noAccept noVerify noDeps noLanes F5 <100lines

### §15.finopsportal — `microservices/finops-portal/`

- `finops-portal/implementation-plans/IP-001-finops-portal-tenant-billing-presentation-kernel.md` (     130L) noContract noCB noCFT noDeps noLanes noKS
- `finops-portal/implementation-plans/IP-002-finops-portal-tenant-billing-presentation-domain.md` (     123L) noContract noCB noCFT noLanes noKS
- `finops-portal/implementation-plans/IP-003-finops-portal-helm-chart-bootstrap.md` (     133L) noContract noCB noCFT noLanes noKS
- `finops-portal/implementation-plans/IP-004-finops-portal-tenant-billing-presentation-usecase.md` (     127L) noContract noCB noCFT noLanes noKS
- `finops-portal/implementation-plans/IP-005-finops-portal-tenant-billing-presentation-api.md` (     143L) noContract noCB noCFT noLanes noKS
- `finops-portal/implementation-plans/IP-006-finops-portal-tenant-billing-presentation-app.md` (     133L) noContract noCB noCFT noLanes noKS
- `finops-portal/implementation-plans/IP-007-finops-portal-cedar-policy-tenant-isolation.md` (     140L) noContract noCB noCFT noLanes noKS
- `finops-portal/implementation-plans/IP-008-finops-portal-grafana-embed-dashboards.md` (     123L) noContract noCB noCFT noLanes noKS
- `finops-portal/implementation-plans/IP-009-finops-portal-cost-allocation-policy-kernel.md` (     124L) noContract noCB noCFT noLanes noKS
- `finops-portal/implementation-plans/IP-010-finops-portal-cost-allocation-policy-domain.md` (     137L) noContract noCB noCFT noLanes noKS
- `finops-portal/implementation-plans/IP-011-finops-portal-anomaly-explanation-kernel.md` (     143L) noContract noCB noCFT noLanes noKS
- `finops-portal/implementation-plans/IP-012-finops-portal-anomaly-explanation-domain.md` (     122L) noContract noCB noCFT noLanes noKS
- `finops-portal/implementation-plans/IP-013-finops-portal-credit-ledger-kernel.md` (     143L) noContract noCB noCFT noLanes noKS
- `finops-portal/implementation-plans/IP-014-finops-portal-focus-export-pipeline.md` (     152L) noContract noCB noCFT noLanes noKS
- `finops-portal/implementation-plans/IP-015-finops-portal-quarterly-regulator-evidence-emit.md` (     148L) noContract noCB noCFT noLanes noKS
- `finops-portal/implementation-plans/IP-016-budget-alert-kernel.md` (      25L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `finops-portal/implementation-plans/IP-017-budget-alert-domain.md` (      24L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `finops-portal/implementation-plans/IP-018-forecasting-usecase.md` (      26L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `finops-portal/implementation-plans/IP-019-commitment-management-domain.md` (      25L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `finops-portal/implementation-plans/IP-020-rightsizing-recommender-worker.md` (      26L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `finops-portal/implementation-plans/IP-021-showback-chargeback-domain.md` (      24L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `finops-portal/implementation-plans/IP-022-budget-alert-rest.md` (      26L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `finops-portal/implementation-plans/IP-023-forecasting-rest-and-cache.md` (      24L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `finops-portal/implementation-plans/IP-024-commitment-management-grpc.md` (      19L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `finops-portal/implementation-plans/IP-025-rightsizing-rest-and-dashboard.md` (      24L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `finops-portal/implementation-plans/IP-026-showback-chargeback-emit.md` (      22L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines

### §15.forms — `microservices/forms/`

- `forms/IP-001-layer-a-postgres-redis-meilisearch-clamav-waf-cdn-captcha-iac.md` (     104L) noDeps noKS
- `forms/IP-002-form-field-section-response-domain-kernel.md` (      92L) noContract noDeps noKS <100lines
- `forms/IP-003-conditional-logic-engine-cel.md` (      61L) noContract noCB noVerify noDeps noKS <100lines
- `forms/IP-004-validation-engine.md` (      59L) noContract noCB noVerify noDeps noKS <100lines
- `forms/IP-005-versioning-and-changeset-binding.md` (      61L) noContract noCB noVerify noDeps noKS <100lines
- `forms/IP-006-postgres-citus-adapter-with-column-encryption.md` (      52L) noContract noCB noVerify noDeps noKS <100lines
- `forms/IP-007-redis-adapter.md` (      42L) noContract noCB noVerify noDeps noKS <50lines
- `forms/IP-008-meilisearch-adapter.md` (      41L) noContract noCB noVerify noDeps noKS <50lines
- `forms/IP-009-captcha-adapter.md` (      46L) noContract noCB noVerify noDeps noKS <50lines
- `forms/IP-010-form-builder-leptos-wasm.md` (      45L) noContract noCB noVerify noDeps noKS <50lines
- `forms/IP-011-form-renderer.md` (      49L) noContract noCB noVerify noDeps noKS <50lines
- `forms/IP-012-response-collector-rest.md` (      50L) noContract noCB noVerify noDeps noKS <100lines
- `forms/IP-013-bulk-distribute-worker.md` (      46L) noContract noCB noVerify noDeps noKS <50lines
- `forms/IP-014-export-worker.md` (      48L) noContract noCB noVerify noDeps noKS <50lines
- `forms/IP-015-hg-forms-registration.md` (      49L) noContract noCB noVerify noDeps noKS <50lines

### §15.foundry — `microservices/intelligence/`

- `foundry/IP-001-runtime-runtime-cluster-iac.md` (      75L) noDeps noKS <100lines
- `foundry/IP-002-runtime-redis-and-postgres-baseline.md` (      74L) noDeps noKS <100lines
- `foundry/IP-003-runtime-capability-executor-kernel.md` (     190L) noDeps noKS
- `foundry/IP-004-runtime-capability-executor-domain-and-usecase.md` (     165L) noDeps noKS
- `foundry/IP-005-runtime-capability-registry-cache-stack.md` (     128L) noDeps noKS
- `foundry/IP-006-runtime-session-state-stack.md` (     135L) noDeps noKS
- `foundry/IP-007-runtime-invocation-orchestrator-stack.md` (     113L) noDeps noKS
- `foundry/IP-008-runtime-runtime-pool-stack.md` (     121L) noDeps noKS
- `foundry/IP-009-runtime-capability-executor-api-and-rest.md` (     133L) noDeps noKS
- `foundry/IP-010-runtime-capability-executor-sdk.md` (     136L) noDeps noKS
- `foundry/IP-011-runtime-capability-executor-app.md` (     136L) noDeps noKS
- `foundry/IP-012-runtime-autonomy-tier-gate.md` (     138L) noDeps noKS
- `foundry/IP-013-runtime-dsr-cascade-session-handler.md` (     128L) noDeps noKS
- `foundry/IP-014-runtime-runtime-self-slo-manifests.md` (      77L) noDeps noKS <100lines
- `foundry/IP-015-runtime-hg-fr-hyperscaler-gate-registration.md` (     138L) noDeps noKS
- `foundry/IP-016-supervisor-postgres-layer-a-iac.md` (      75L) noDeps noKS <100lines
- `foundry/IP-017-supervisor-redis-layer-a-iac.md` (      67L) noContract noCB noDeps noKS <100lines
- `foundry/IP-018-supervisor-k8s-operator-iac.md` (      60L) noContract noCB noVerify noKS <100lines
- `foundry/IP-019-supervisor-agent-fleet-lifecycle-kernel.md` (     119L) noContract noDeps noKS
- `foundry/IP-020-supervisor-autonomy-policy-enforcement.md` (      98L) noContract noCB noKS <100lines
- `foundry/IP-021-supervisor-capability-deployment.md` (      93L) noContract noCB noKS <100lines
- `foundry/IP-022-supervisor-supervision-event-bus.md` (      68L) noContract noCB noVerify noKS <100lines
- `foundry/IP-023-supervisor-kill-switch-engage-state.md` (      95L) noContract noCB noVerify noKS <100lines
- `foundry/IP-024-supervisor-kill-switch-propagation.md` (      83L) noContract noCB noKS <100lines
- `foundry/IP-025-supervisor-fleet-state-postgres-adapter.md` (     108L) noContract noCB noKS
- `foundry/IP-026-supervisor-rest-api.md` (      99L) noContract noCB noKS <100lines
- `foundry/IP-027-supervisor-supervisor-self-slos.md` (      87L) noContract noCB noVerify noKS <100lines
- `foundry/IP-028-supervisor-sdk-rust-and-ts.md` (     101L) noContract noCB noKS
- `foundry/IP-029-supervisor-app-composition-root.md` (     110L) noContract noCB noKS
- `foundry/IP-030-supervisor-e2e-drills-and-dashboards.md` (      84L) noContract noCB noVerify noKS <100lines
- `foundry/IP-031-eval-layer-a-gpu-runner-pool-iac.md` (      48L) noVerify noDeps noKS <50lines
- `foundry/IP-032-eval-layer-a-postgres-clickhouse-golden-store-iac.md` (      52L) noVerify noDeps noKS <100lines
- `foundry/IP-033-eval-eval-runner-kernel.md` (     171L) noDeps noKS
- `foundry/IP-034-eval-eval-runner-domain.md` (      46L) noContract noDeps noKS <50lines
- `foundry/IP-035-eval-eval-runner-usecase.md` (      47L) noContract noDeps noKS <50lines
- `foundry/IP-036-eval-eval-runner-api.md` (      44L) noContract noAccept noDeps noKS <50lines
- `foundry/IP-037-eval-eval-runner-adapter.md` (     152L) noCB noCFT noVerify noKS retired VCS ratchet
- `foundry/IP-038-eval-eval-runner-adapter-s3.md` (      39L) noContract noAccept noDeps noKS <50lines
- `foundry/IP-039-eval-eval-runner-adapter-gpu.md` (      38L) noContract noAccept noDeps noKS <50lines
- `foundry/IP-040-eval-eval-runner-rest.md` (      39L) noContract noAccept noDeps noKS <50lines
- `foundry/IP-041-eval-eval-runner-worker.md` (      39L) noContract noAccept noDeps noKS <50lines
- `foundry/IP-042-eval-eval-runner-sdk.md` (      39L) noContract noAccept noDeps noKS <50lines
- `foundry/IP-043-eval-eval-runner-app.md` (      37L) noContract noAccept noDeps noKS <50lines
- `foundry/IP-044-eval-parity-analyzer-bootstrap.md` (      54L) noContract noAccept noDeps noKS <100lines
- `foundry/IP-045-eval-replay-engine-bootstrap.md` (      59L) noContract noAccept noDeps noKS <100lines
- `foundry/IP-046-evidence-storage-backend-iac.md` (      69L) noVerify noDeps noKS <100lines
- `foundry/IP-047-evidence-self-slo-manifest.md` (      60L) noVerify noDeps noKS <100lines
- `foundry/IP-048-evidence-capability-invocation-recorder-kernel.md` (      64L) noVerify noDeps noKS <100lines
- `foundry/IP-049-evidence-evidence-pack-builder-kernel.md` (      68L) noVerify noDeps noKS <100lines
- `foundry/IP-050-evidence-evidence-pack-builder-domain.md` (      72L) noVerify noDeps noKS <100lines
- `foundry/IP-051-evidence-evidence-pack-builder-usecase-and-adapters.md` (      69L) noVerify noDeps noKS <100lines
- `foundry/IP-052-evidence-capability-invocation-recorder-stack.md` (      66L) noVerify noDeps noKS <100lines
- `foundry/IP-053-evidence-eval-evidence-aggregator.md` (      60L) noVerify noDeps noKS <100lines
- `foundry/IP-054-evidence-evidence-query-stack.md` (      70L) noVerify noDeps noKS <100lines
- `foundry/IP-055-evidence-regulator-export-stack.md` (      74L) noVerify noDeps noKS <100lines
- `foundry/IP-056-evidence-audit-chain-bridge.md` (      60L) noVerify noDeps noKS <100lines
- `foundry/IP-057-evidence-sdk-cross-microservice.md` (      66L) noVerify noDeps noKS <100lines
- `foundry/IP-058-evidence-regulator-export-framework-profiles.md` (      62L) noVerify noDeps noKS <100lines
- `foundry/IP-059-evidence-evidence-archive-cascade.md` (      61L) noVerify noDeps noKS <100lines
- `foundry/IP-060-evidence-self-observability-slo-wiring.md` (      62L) noVerify noDeps noKS <100lines
- `foundry/IP-061-guardrails-cedar-policy-engine-iac.md` (     123L) noDeps noKS
- `foundry/IP-062-guardrails-classifier-model-serving-iac.md` (      66L) noDeps noKS <100lines
- `foundry/IP-063-guardrails-rule-store-postgres-iac.md` (      63L) noCB noDeps noKS <100lines
- `foundry/IP-064-guardrails-prompt-classifier-kernel.md` (     161L) noDeps noKS
- `foundry/IP-065-guardrails-output-validator-kernel.md` (     117L) noDeps noKS
- `foundry/IP-066-guardrails-autonomy-tier-gate-kernel-and-cedar-adapter.md` (     144L) noDeps noKS
- `foundry/IP-067-guardrails-content-safety-rule-engine-kernel-and-postgres-adapter.md` (     127L) noDeps noKS
- `foundry/IP-068-guardrails-jailbreak-detector-ensemble.md` (     127L) noDeps noKS
- `foundry/IP-069-guardrails-ai-slop-detector.md` (      97L) noDeps noKS <100lines
- `foundry/IP-070-guardrails-classifier-model-adapter-onnx.md` (      97L) noDeps noKS <100lines
- `foundry/IP-071-guardrails-rest-and-grpc-surface.md` (      91L) noDeps noKS <100lines
- `foundry/IP-072-guardrails-worker-and-app-composition.md` (     101L) noDeps noKS
- `foundry/IP-073-guardrails-runtime-guardrails-coupling-lane.md` (      95L) noDeps noKS <100lines
- `foundry/IP-074-guardrails-shadow-mode-rollout-and-false-positive-budget.md` (     104L) noDeps noKS
- `foundry/IP-075-guardrails-sdk-rust-and-typescript.md` (     110L) noDeps noKS
- `foundry/IP-076-providers-router-kernel.md` (     176L) noDeps noKS
- `foundry/IP-077-providers-router-domain.md` (      99L) noContract noCFT noDeps noKS <100lines
- `foundry/IP-078-providers-router-usecase.md` (      87L) noContract noCFT noDeps noKS <100lines
- `foundry/IP-079-providers-router-api.md` (      53L) noContract noCFT noDeps noKS <100lines
- `foundry/IP-080-providers-router-adapter.md` (      73L) noContract noCFT noDeps noKS <100lines
- `foundry/IP-081-providers-adapter-anthropic-api.md` (     110L) noContract noCFT noDeps noKS
- `foundry/IP-082-providers-adapter-anthropic-subscription.md` (      73L) noContract noCFT noDeps noKS <100lines
- `foundry/IP-083-providers-adapter-openai-api.md` (      53L) noContract noCB noCFT noDeps noKS <100lines
- `foundry/IP-084-providers-adapter-openai-subscription.md` (      54L) noContract noCB noCFT noDeps noKS <100lines
- `foundry/IP-085-providers-adapter-gemini-api.md` (      52L) noContract noCB noCFT noDeps noKS <100lines
- `foundry/IP-086-providers-adapter-gemini-subscription.md` (      54L) noContract noCB noCFT noDeps noKS <100lines
- `foundry/IP-087-providers-adapter-in-house.md` (      55L) noContract noCB noCFT noDeps noKS <100lines
- `foundry/IP-088-providers-adapter-openbao.md` (     117L) noContract noCB noCFT noDeps noKS
- `foundry/IP-089-providers-router-rest-worker-app.md` (      77L) noContract noCB noCFT noDeps noKS <100lines
- `foundry/IP-090-providers-router-sdk.md` (      99L) noContract noCB noCFT noDeps noKS <100lines
- `foundry/IP-091-milvus-cluster-iac.md` (     150L) noContract noCB noCFT noDeps noLanes noKS ADR-0136-STALE
- `foundry/IP-092-vector-collection-bootstrap.md` (     158L) noContract noCB noCFT noDeps noLanes noKS ADR-0136-STALE
- `foundry/IP-093-embedding-ingest-pipeline.md` (     159L) noContract noCB noCFT noDeps noLanes noKS
- `foundry/IP-094-hnsw-tuning-and-adapter.md` (     154L) noContract noCB noCFT noDeps noLanes noKS
- `foundry/IP-095-gpu-acceleration-optional.md` (     160L) noContract noCB noCFT noDeps noLanes noKS
- `foundry/IP-096-milvus-backup-restore.md` (     156L) noContract noCB noCFT noDeps noLanes noKS
- `foundry/IP-097-milvus-cross-region-replication.md` (     170L) noContract noCB noCFT noDeps noLanes noKS
- `foundry/IP-WASMTIME-001-tool-sandbox-runtime-integration.md` (      93L) noContract noCB noCFT noVerify noDeps noLanes noKS ADR-0136-STALE <100lines
- `foundry/IP-WASMTIME-002-capability-token-binding.md` (      71L) noContract noCB noCFT noVerify noDeps noLanes noKS ADR-0136-STALE <100lines
- `foundry/IP-WASMTIME-003-fuel-and-memory-accounting.md` (      60L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `foundry/IP-WASMTIME-004-component-model-onboarding.md` (      53L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <100lines

### §15.governance — `microservices/governance/`

- `governance/IP-001-scaffold-umbrella-bcs.md` (      81L) noDeps noKS <100lines
- `governance/IP-002-migrate-tier-a-check-crates-batch-1.md` (      82L) noDeps noKS <100lines
- `governance/IP-003-migrate-tier-a-check-crates-batch-2.md` (      74L) noDeps noKS <100lines
- `governance/IP-004-lane-runtime-kernel-domain.md` (     151L) noDeps noKS
- `governance/IP-005-lane-runtime-usecase-adapter-rest.md` (     144L) noDeps noKS
- `governance/IP-006-policy-engine-kernel-domain.md` (     119L) noDeps noKS
- `governance/IP-007-policy-engine-usecase-adapter.md` (      99L) noDeps noKS <100lines
- `governance/IP-008-evidence-emitter-kernel-domain.md` (     108L) noDeps noKS
- `governance/IP-009-evidence-emitter-adapter-rest-worker.md` (     120L) noDeps noKS
- `governance/IP-010-aggregation-indexer-full-stack.md` (     120L) noDeps noKS
- `governance/IP-011-industry-best-practice-conformance-lane.md` (     123L) noDeps noKS
- `governance/IP-012-per-microservice-layout-lane.md` (      95L) noDeps noKS <100lines
- `governance/IP-013-aggregation-index-generation-lane.md` (      88L) noDeps noKS <100lines
- `governance/IP-014-observability-slo-authoring.md` (     133L) noDeps noKS
- `governance/IP-015-runbooks-iac-finalization.md` (     146L) noDeps noKS
- `governance/IP-NEW-chaos-engineering-substrate.md` (     130L) noVerify noDeps noKS
- `governance/IP-NEW-eu-ai-act-annex-iii-refusal-lane.md` (     140L) noVerify noDeps noKS
- `governance/IP-NEW-slsa-l3-evidence-grounded-lane.md` (     149L) noVerify noDeps noKS
- `governance/IP-WASMTIME-001-envoy-wasm-filter-substrate.md` (      72L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `governance/IP-WASMTIME-002-waf-coraza-onboard.md` (     136L) noCB noCFT noVerify noKS retired VCS ratchet
- `governance/IP-WASMTIME-003-regulatory-response-shaper.md` (      53L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `governance/IP-WASMTIME-004-authz-filter.md` (      54L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines

### §15.identity — `microservices/identity/`

- `identity/IP-001-zitadel-helm-per-pack.md` (      99L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <100lines
- `identity/IP-002-oidc-issuer-kernel.md` (     126L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `identity/IP-003-oidc-issuer-adapter-zitadel.md` (      73L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `identity/IP-004-webauthn-relying-party-kernel.md` (     123L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `identity/IP-005-webauthn-rest.md` (     101L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `identity/IP-006-aaguid-refresh-worker.md` (      88L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `identity/IP-007-scim-server-kernel.md` (     119L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `identity/IP-008-scim-adapter-zitadel.md` (      89L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `identity/IP-009-hris-adapter.md` (     110L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `identity/IP-010-step-up-orchestrator.md` (     128L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `identity/IP-011-external-idp-federation.md` (      85L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `identity/IP-012-audit-emitter.md` (     109L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `identity/IP-013-edge-authz-rules.md` (      99L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `identity/IP-014-continuous-risk-scoring.md` (      92L) noContract noCB noCFT noVerify noDeps noLanes noKS <100lines
- `identity/IP-015-shared-kernel-crates.md` (     129L) noContract noCB noCFT noDeps noLanes noKS
- `identity/IP-016-zitadel-scale-validation-load-test.md` (     117L) noContract noCB noCFT noVerify noDeps noLanes noKS
- `identity/IP-017-multi-context-principal-resolver.md` (      28L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines

### §15.intelligence — `microservices/intelligence/`

- `intelligence/IP-001-consumer-intelligence-substrate.md` (      28L) noContract noCB noCFT noVerify noDeps noLanes noKS ADR-0136-STALE <50lines
- `intelligence/IP-001-domain-layer-dispatch-request.md` (     111L) noDeps
- `intelligence/IP-002-domain-layer-secret-reference.md` (     103L) noDeps F5
- `intelligence/IP-003-domain-layer-refusal-decision.md` (     103L) noContract noCB noDeps
- `intelligence/IP-004-domain-layer-routing-decision.md` (      81L) noContract noCB noDeps noKS <100lines
- `intelligence/IP-005-domain-layer-eval-record.md` (      69L) noContract noCB noDeps noKS <100lines
- `intelligence/IP-006-domain-layer-attribution.md` (      72L) noContract noCB noDeps noKS <100lines
- `intelligence/IP-007-kernel-model-router.md` (      69L) noContract noCB noDeps noKS <100lines
- `intelligence/IP-008-kernel-guardrail-stack.md` (      66L) noContract noCB noVerify noDeps noKS <100lines
- `intelligence/IP-009-kernel-audit-tap.md` (      69L) noContract noCB noVerify noDeps noKS <100lines
- `intelligence/IP-010-usecase-dispatch-flow.md` (      81L) noContract noCB noDeps noKS <100lines
- `intelligence/IP-011-adapter-anthropic.md` (      79L) noContract noCB noDeps F5 <100lines
- `intelligence/IP-012-adapter-openai.md` (      64L) noContract noCB noVerify noDeps F5 <100lines
- `intelligence/IP-013-adapter-google-vertex.md` (      66L) noContract noCB noVerify noDeps F5 <100lines
- `intelligence/IP-014-adapter-bedrock.md` (      46L) noContract noCB noVerify noDeps F5 <50lines
- `intelligence/IP-015-kernel-guardrail-eu-ai-act.md` (      77L) noContract noCB noDeps <100lines
- `intelligence/IP-016-streaming-sse-transport.md` (      63L) noContract noCB noVerify noDeps <100lines
- `intelligence/IP-017-streaming-websocket-transport.md` (      66L) noContract noCB noVerify noDeps <100lines
- `intelligence/IP-018-multi-modal-audio-video.md` (      56L) noContract noCB noVerify noDeps <100lines
- `intelligence/IP-019-library-first-caller-eval.md` (      76L) noContract noCB noDeps <100lines
- `intelligence/IP-020-brand-ux-surface-components.md` (      56L) noContract noCB noVerify noDeps <100lines
- `intelligence/IP-021-eval-golden-set.md` (      60L) noContract noCB noVerify noDeps <100lines
- `intelligence/IP-022-audit-tap-merkle-seal.md` (      69L) noContract noCB noVerify noDeps F5 <100lines
- `intelligence/IP-023-byok-credential-rotation.md` (      57L) noContract noCB noVerify noDeps F5 <100lines
- `intelligence/IP-024-minor-protection-wiring.md` (      57L) noContract noCB noVerify noDeps <100lines
- `intelligence/IP-025-cn-pipl-pack-adapter.md` (      56L) noContract noCB noVerify noDeps <100lines

### §15.mail — `microservices/mail/`

- `mail/IP-001-iac-bootstrap.md` (     141L) noDeps noKS
- `mail/IP-002-mailbox-store-kernel.md` (     181L) noDeps noKS
- `mail/IP-003-mailbox-store-postgres-adapter.md` (     146L) noDeps noKS
- `mail/IP-004-mailbox-store-s3-adapter.md` (     118L) noDeps noKS
- `mail/IP-005-dual-context-isolation.md` (     118L) noDeps noKS
- `mail/IP-006-inbound-smtp.md` (     126L) noDeps noKS
- `mail/IP-007-outbound-smtp.md` (     114L) noDeps noKS
- `mail/IP-008-imap-frontend.md` (     104L) noDeps noKS
- `mail/IP-009-search-index.md` (     103L) noDeps noKS
- `mail/IP-010-retention-policy.md` (     110L) noDeps noKS
- `mail/IP-011-legal-hold-engine.md` (     108L) noDeps noKS
- `mail/IP-012-ediscovery-export.md` (      96L) noDeps noKS <100lines
- `mail/IP-013-mail-workflow-handoff.md` (     101L) noDeps noKS
- `mail/IP-014-hg-mail-authority-cohesion.md` (     130L) noDeps noKS retired VCS ratchet
- `mail/IP-015-pack-kr-overlay.md` (     101L) noDeps noKS
- `mail/IP-016-jmap-rfc-8620-frontend.md` (      43L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `mail/IP-017-anti-phishing-edge-wiring.md` (      41L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS CP <50lines
- `mail/IP-018-hipaa-overlay-rollout.md` (      43L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines

### §15.meet — `microservices/meet/`

- `meet/IP-001-iac-bootstrap.md` (      83L) noDeps noKS <100lines
- `meet/IP-002-cargo-workspace-bootstrap.md` (      75L) noDeps noKS <100lines
- `meet/IP-003-meeting-room-kernel-domain.md` (      85L) noCB noDeps noKS <100lines
- `meet/IP-004-meeting-room-adapter-postgres.md` (      82L) noCB noDeps noKS <100lines
- `meet/IP-005-meeting-instance-and-livekit.md` (      93L) noCB noDeps noKS <100lines
- `meet/IP-006-participant-and-lobby.md` (      97L) noCB noDeps noKS <100lines
- `meet/IP-007-screen-share-and-tracks.md` (      76L) noCB noDeps noKS <100lines
- `meet/IP-008-recording-pipeline.md` (     146L) noCB noDeps noKS
- `meet/IP-009-transcription-pipeline.md` (     107L) noCB noDeps noKS
- `meet/IP-010-webinar-and-breakouts.md` (      90L) noCB noDeps noKS <100lines
- `meet/IP-011-live-stream-egress.md` (      90L) noCB noDeps noKS <100lines
- `meet/IP-012-e2e-encryption-mls.md` (     101L) noCB noDeps noKS
- `meet/IP-013-contracts-openapi-asyncapi-proto.md` (      59L) noCB noDeps noKS <100lines
- `meet/IP-014-cedar-policies-and-data-residency.md` (      69L) noCB noDeps noKS <100lines
- `meet/IP-015-hg-meet-registration-and-branch-protection.md` (      65L) noCB noDeps noKS <100lines

### §15.messenger — `microservices/messenger/`

- `messenger/IP-001-iac-bootstrap.md` (      82L) noDeps noKS <100lines
- `messenger/IP-002-cargo-workspace-bootstrap.md` (      71L) noDeps noKS <100lines
- `messenger/IP-003-channel-store-kernel-domain.md` (      86L) noDeps noKS <100lines
- `messenger/IP-004-channel-store-adapter-postgres.md` (      84L) noDeps noKS <100lines
- `messenger/IP-005-message-stream-kernel-domain.md` (      52L) noDeps noKS <100lines
- `messenger/IP-006-message-stream-adapters.md` (      70L) noCB noDeps noKS <100lines
- `messenger/IP-007-presence-bc.md` (      64L) noCB noDeps noKS <100lines
- `messenger/IP-008-file-attachment-bc.md` (      63L) noCB noDeps noKS <100lines
- `messenger/IP-009-thread-tree-and-mention-router.md` (      48L) noCB noDeps noKS <50lines
- `messenger/IP-010-read-receipt-tracker.md` (      65L) noCB noDeps noKS <100lines
- `messenger/IP-011-rest-api-surface.md` (      48L) noCB noDeps noKS <50lines
- `messenger/IP-012-websocket-frame-protocol.md` (      65L) noCB noDeps noKS <100lines
- `messenger/IP-013-search-and-cedar-filter.md` (      64L) noCB noDeps noKS <100lines
- `messenger/IP-014-huddles-livekit-signaling.md` (      73L) noCB noDeps noKS <100lines
- `messenger/IP-015-hg-messenger-registration-and-branch-protection.md` (      72L) noCB noDeps noKS <100lines
- `messenger/IP-NEW-hyperscaler-metric-emission.md` (     200L) noDeps noKS

### §15.network — `microservices/network/`

- `network/IP-001-iac-bootstrap.md` (      88L) noDeps noKS <100lines
- `network/IP-002-cargo-workspace-bootstrap.md` (     112L) noDeps noKS
- `network/IP-003-professional-profile-bc.md` (     116L) noCFT noDeps noKS
- `network/IP-004-professional-graph-and-connection-request-bcs.md` (      79L) noCFT noDeps noKS <100lines
- `network/IP-005-post-composition-bc.md` (      92L) noCB noDeps noKS <100lines
- `network/IP-006-feed-timeline-and-reactions-bcs.md` (      72L) noCB noCFT noDeps noKS <100lines
- `network/IP-007-endorsement-engine-bc.md` (      84L) noCB noCFT noDeps noKS <100lines
- `network/IP-008-skill-assessments-and-profile-verification-bcs.md` (      72L) noCB noCFT noDeps noKS <100lines
- `network/IP-009-pages-groups-events-bcs.md` (      82L) noCB noCFT noDeps noKS <100lines
- `network/IP-010-inmail-bridge-bc.md` (      79L) noCB noCFT noDeps noKS <100lines
- `network/IP-011-jobs-handoff-bc.md` (     110L) noCB noCFT noDeps noKS
- `network/IP-012-mentions-hashtags-trending-notifications-bcs.md` (      70L) noCB noCFT noDeps noKS <100lines
- `network/IP-013-search-and-cedar-filter.md` (      78L) noCB noCFT noDeps noKS <100lines
- `network/IP-014-recommender-fairness-and-bias-lane.md` (      87L) noCB noDeps noKS <100lines
- `network/IP-015-hg-network-registration-and-branch-protection.md` (      84L) noCB noDeps noKS <100lines

### §15.notes — `microservices/notes/`

- `notes/IP-001-iac.md` (      68L) noDeps noKS <100lines
- `notes/IP-002-cargo-workspace-bootstrap.md` (      68L) noContract noCB noVerify noDeps noKS <100lines
- `notes/IP-003-note-store-kernel-domain.md` (      68L) noContract noCB noDeps noKS <100lines
- `notes/IP-004-tag-graph-kernel-domain.md` (      50L) noContract noCB noCFT noDeps noKS <100lines
- `notes/IP-005-backlink-graph-kernel-domain.md` (      52L) noContract noCB noCFT noDeps noKS <100lines
- `notes/IP-006-daily-note-template-gallery.md` (      47L) noContract noCB noCFT noDeps noKS <50lines
- `notes/IP-007-web-clipper-bridge.md` (      59L) noContract noCB noCFT noDeps noKS <100lines
- `notes/IP-008-share-link-and-embed.md` (      93L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `notes/IP-009-checklist-and-version-history.md` (      93L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `notes/IP-010-search-and-graph-view.md` (      94L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `notes/IP-011-collab-edit-loro.md` (      94L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `notes/IP-012-import-export-pipelines.md` (      98L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `notes/IP-013-ai-assist-and-e2e-refusal.md` (      65L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `notes/IP-014-e2e-key-management.md` (      43L) noContract noCB noCFT noVerify noDeps noKS <50lines
- `notes/IP-015-hg-notes-conformance.md` (      69L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `notes/IP-016-collab-edit-mls-loro-hardening.md` (      42L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <50lines
- `notes/IP-017-hipaa-clinical-notes-overlay.md` (      43L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `notes/IP-018-abuse-defence-edge-wiring.md` (      39L) noContract noCB noCFT noAccept noVerify noDeps noLanes CP <50lines

### §15.observability — `microservices/observability/`

- `observability/IP-001-layer-a-grafana-stack-iac.md` (     110L) noDeps noKS
- `observability/IP-002-openslo-manifest-convention.md` (     103L) noDeps noKS
- `observability/IP-003-slo-engine-kernel.md` (     192L) noDeps noKS
- `observability/IP-004-slo-engine-domain.md` (     105L) noDeps noKS
- `observability/IP-005-slo-engine-usecase.md` (     120L) noDeps noKS
- `observability/IP-006-slo-engine-adapter.md` (     109L) noDeps noKS
- `observability/IP-007-slo-engine-rest.md` (      96L) noCB noDeps noKS <100lines
- `observability/IP-008-slo-engine-worker.md` (      98L) noCB noDeps noKS <100lines
- `observability/IP-009-slo-engine-app.md` (      88L) noCB noDeps noKS <100lines
- `observability/IP-010-promotion-eligibility-ledger.md` (      75L) noCB noDeps noKS <100lines
- `observability/IP-011-per-component-release-pointers.md` (      85L) noCB noDeps noKS retired VCS ratchet <100lines
- `observability/IP-012-retired-vcs-ratchet` (      78L) noCB noDeps noKS retired VCS ratchet <100lines
- `observability/IP-013-event-driven-promote-workflows.md` (      86L) noCB noDeps noKS retired VCS ratchet <100lines
- `observability/IP-014-automated-rollback-primitive.md` (      75L) noCB noDeps noKS retired VCS ratchet <100lines
- `observability/IP-015-canary-cohort-weighting.md` (      87L) noCB noDeps noKS <100lines
- `observability/IP-021-clickhouse-cluster-iac.md` (     151L) noContract noCB noCFT noDeps noLanes noKS
- `observability/IP-022-otel-to-clickhouse-bridge.md` (     157L) noContract noCB noCFT noDeps noLanes noKS
- `observability/IP-023-ops-portal-rollup-mvs.md` (     158L) noContract noCB noCFT noDeps noLanes noKS
- `observability/IP-024-cold-tier-retention-policy.md` (     160L) noContract noCB noCFT noDeps noLanes noKS
- `observability/IP-025-clickhouse-backup-restore.md` (     180L) noContract noCB noCFT noDeps noLanes noKS
- `observability/IP-026-sse-transport-impl.md` (      31L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `observability/IP-027-websocket-transport-impl.md` (      31L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `observability/IP-028-loro-presence-binding.md` (      30L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `observability/IP-029-tail-sampling-processor-config.md` (      31L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `observability/IP-030-sample-recipe-per-microservice.md` (      28L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `observability/IP-031-tail-sample-fidelity-test.md` (      35L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines

### §15.ontology — `microservices/ontology/`

- `ontology/IP-001-ontology-iac-stack.md` (      74L) noContract noCB noCFT noAccept noKS <100lines
- `ontology/IP-002-object-type-registry-kernel-domain.md` (      83L) noContract noCB noCFT noAccept noKS <100lines
- `ontology/IP-003-link-action-function-type-registry.md` (      70L) noContract noCB noCFT noAccept noKS <100lines
- `ontology/IP-004-entity-store-rls-citus.md` (      73L) noContract noCB noCFT noAccept noKS <100lines
- `ontology/IP-005-link-store-traversal.md` (      59L) noContract noCB noCFT noAccept noKS <100lines
- `ontology/IP-006-cedar-fragment-coverage-engine.md` (      69L) noContract noCB noCFT noAccept noKS <100lines
- `ontology/IP-007-action-engine-cedar-gated.md` (      68L) noContract noCB noCFT noAccept noKS <100lines
- `ontology/IP-008-function-engine-oltp-and-olap.md` (      61L) noContract noCB noCFT noAccept noKS <100lines
- `ontology/IP-009-clickhouse-history-mirror.md` (      63L) noContract noCB noCFT noAccept noKS <100lines
- `ontology/IP-010-audit-chain-merkle-ed25519.md` (     160L) noCB noCFT noVerify noKS retired VCS ratchet
- `ontology/IP-011-query-engine-3layer-kg.md` (      64L) noContract noCB noCFT noAccept noKS <100lines
- `ontology/IP-012-agent-gateway-llm-tool-call.md` (      65L) noContract noCB noCFT noAccept noKS <100lines
- `ontology/IP-013-pillar-cross-pillar-grant.md` (      59L) noContract noCB noCFT noAccept noKS <100lines
- `ontology/IP-014-rest-and-sdk-surfaces.md` (      65L) noContract noCB noCFT noAccept noKS <100lines
- `ontology/IP-015-app-binaries-and-branch-protection.md` (      74L) noContract noCB noCFT noAccept noKS <100lines
- `ontology/IP-016-read-path-library-rollout.md` (      40L) noContract noCB noCFT noAccept noVerify noDeps noLanes F5 <50lines
- `ontology/IP-017-share-token-surface.md` (      43L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `ontology/IP-018-abuse-defence-edge-wiring.md` (      45L) noContract noCB noCFT noAccept noVerify noDeps noLanes CP <50lines
- `ontology/IP-CT-001-cross-tenant-projection-topic-model.md` (     136L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `ontology/IP-CT-002-tenant-aware-pulsar-acl-adapter.md` (     143L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `ontology/IP-CT-003-projection-scope-narrowing.md` (      94L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <100lines
- `ontology/IP-CT-004-aggregate-only-projection.md` (     143L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS
- `ontology/IP-CT-005-sovereignty-zero-copy.md` (     194L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS

### §15.opsdashboardcontrolcenter — `microservices/ops-dashboard-control-center/`

- `ops-dashboard-control-center/IP-001-control-plane-manifest-and-contracts.md` (      27L) noContract noCB noCFT noDeps noLanes noKS <50lines
- `ops-dashboard-control-center/IP-002-incident-command-workflows.md` (      27L) noContract noCB noCFT noDeps noLanes noKS <50lines
- `ops-dashboard-control-center/IP-003-deployment-approval-and-rollback.md` (      27L) noContract noCB noCFT noDeps noLanes noKS <50lines
- `ops-dashboard-control-center/IP-004-cluster-health-and-recovery.md` (      27L) noContract noCB noCFT noDeps noLanes noKS <50lines
- `ops-dashboard-control-center/IP-005-tenant-isolation-policy-audit.md` (      27L) noContract noCB noCFT noDeps noLanes noKS <50lines
- `ops-dashboard-control-center/IP-006-evidence-pack-export.md` (      27L) noContract noCB noCFT noDeps noLanes noKS <50lines
- `ops-dashboard-control-center/IP-007-localization-escalation-runbooks.md` (      27L) noContract noCB noCFT noDeps noLanes noKS <50lines
- `ops-dashboard-control-center/IP-008-step-up-auth-flow.md` (      50L) noContract noCB noCFT noVerify noDeps noLanes F5 <100lines
- `ops-dashboard-control-center/IP-009-audit-emission-integration.md` (      42L) noContract noCB noCFT noVerify noDeps noLanes noKS <50lines
- `ops-dashboard-control-center/IP-010-cedar-admin-console-surface.md` (      37L) noContract noCB noCFT noVerify noDeps noLanes F5 <50lines
- `ops-dashboard-control-center/IP-011-tenant-admin-panel.md` (      35L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `ops-dashboard-control-center/IP-012-cell-operator-panel.md` (      36L) noContract noCB noCFT noVerify noDeps noLanes F5 <50lines
- `ops-dashboard-control-center/IP-013-adr-promotion-triage-panel.md` (      35L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `ops-dashboard-control-center/IP-014-finops-portal-integration.md` (      38L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `ops-dashboard-control-center/IP-015-observability-pivot.md` (      39L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `ops-dashboard-control-center/IP-016-on-call-handoff-bc.md` (      40L) noContract noCB noCFT noVerify noDeps noLanes F5 CP <50lines

### §15.payments — `microservices/payments/`

- `payments/IP-001-payments-kernel-charge.md` (      86L) noContract noCB noCFT noVerify noDeps noLanes <100lines
- `payments/IP-002-payments-domain-charge.md` (      84L) noContract noCB noCFT noVerify noDeps noLanes <100lines
- `payments/IP-003-payments-usecase-charge.md` (      76L) noContract noCB noCFT noVerify noDeps noLanes <100lines
- `payments/IP-004-payments-adapter-stripe.md` (      77L) noContract noCB noCFT noVerify noDeps noLanes F5 <100lines
- `payments/IP-005-payments-domain-refund.md` (      58L) noContract noCB noCFT noVerify noDeps noLanes <100lines
- `payments/IP-006-payments-usecase-refund.md` (      43L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `payments/IP-007-payments-domain-payout.md` (      48L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `payments/IP-008-payments-usecase-payout.md` (      45L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `payments/IP-009-payments-domain-dispute.md` (      56L) noContract noCB noCFT noVerify noDeps noLanes <100lines
- `payments/IP-010-payments-usecase-dispute.md` (      45L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `payments/IP-011-payments-domain-subscription.md` (      49L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `payments/IP-012-payments-usecase-subscription.md` (      46L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `payments/IP-013-payments-domain-sub-merchant.md` (      50L) noContract noCB noCFT noVerify noDeps noLanes <100lines
- `payments/IP-014-payments-usecase-sub-merchant.md` (      44L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `payments/IP-015-payments-rest-grpc-app.md` (      74L) noContract noCB noCFT noVerify noDeps noLanes <100lines
- `payments/IP-016-payments-settlement-domain.md` (      47L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `payments/IP-017-payments-settlement-worker.md` (      46L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `payments/IP-018-payments-adapter-adyen.md` (      56L) noContract noCB noCFT noVerify noDeps noLanes F5 <100lines

### §15.pluginappstore — `microservices/plugin-app-store/`

- `plugin-app-store/implementation-plans/IP-001-layer-a-postgres-redis-cedar-cosign-trivy-iac.md` (     175L) noDeps noKS retired VCS ratchet
- `plugin-app-store/implementation-plans/IP-002-plugin-catalog-kernel-domain.md` (     175L) noDeps noKS retired VCS ratchet
- `plugin-app-store/implementation-plans/IP-003-plugin-catalog-usecase-api-adapter-rest-sdk-app.md` (     185L) noDeps noKS retired VCS ratchet
- `plugin-app-store/implementation-plans/IP-004-plugin-lifecycle-state-machine.md` (     175L) noDeps noKS retired VCS ratchet
- `plugin-app-store/implementation-plans/IP-005-plugin-install-kernel-domain-usecase.md` (     172L) noDeps noKS retired VCS ratchet
- `plugin-app-store/implementation-plans/IP-006-plugin-install-rest-sdk-app.md` (     153L) noDeps noKS retired VCS ratchet
- `plugin-app-store/implementation-plans/IP-007-vetting-pipeline-kernel-domain.md` (     158L) noDeps noKS retired VCS ratchet
- `plugin-app-store/implementation-plans/IP-008-vetting-pipeline-cosign-trivy-wasmtime.md` (     171L) noDeps noKS retired VCS ratchet
- `plugin-app-store/implementation-plans/IP-009-per-plugin-permissions-cedar.md` (     152L) noDeps noKS retired VCS ratchet
- `plugin-app-store/implementation-plans/IP-010-per-plugin-rate-limit.md` (     147L) noDeps noKS retired VCS ratchet
- `plugin-app-store/implementation-plans/IP-011-subscription-billing-aggregation.md` (     151L) noDeps noKS retired VCS ratchet
- `plugin-app-store/implementation-plans/IP-012-audit-stream-per-plugin-action.md` (     147L) noDeps noKS retired VCS ratchet
- `plugin-app-store/implementation-plans/IP-013-observability-slo-manifests.md` (     156L) noDeps noKS retired VCS ratchet
- `plugin-app-store/implementation-plans/IP-014-branch-protection-and-hyperscaler-gates.md` (     145L) noDeps noKS retired VCS ratchet
- `plugin-app-store/implementation-plans/IP-015-discovery-install-leptos-app.md` (     159L) noDeps noKS retired VCS ratchet

### §15.recordings — `microservices/recordings/`

- `recordings/IP-001-iac-bootstrap.md` (      68L) noVerify noDeps noKS <100lines
- `recordings/IP-002-cargo-workspace-bootstrap.md` (      71L) noCFT noVerify noDeps noKS <100lines
- `recordings/IP-003-recording-ingest-bc.md` (      54L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `recordings/IP-004-recording-bc.md` (      81L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `recordings/IP-005-media-segment-bc.md` (      91L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `recordings/IP-006-transcript-bc.md` (      88L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `recordings/IP-007-search-bc.md` (      83L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `recordings/IP-008-redaction-bc.md` (      87L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `recordings/IP-009-chapter-summary-bcs.md` (      84L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `recordings/IP-010-retention-legal-hold-bcs.md` (      91L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `recordings/IP-011-playback-share-link-watermark-bcs.md` (      89L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `recordings/IP-012-export-ediscovery-bcs.md` (      91L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `recordings/IP-013-translation-bc.md` (      82L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `recordings/IP-014-strangler-migration-adapter.md` (      95L) noContract noCB noCFT noVerify noDeps noKS <100lines
- `recordings/IP-015-hg-recordings.md` (      95L) noContract noCFT noVerify noDeps noKS <100lines

### §15.sheets — `microservices/sheets/`

- `sheets/IP-001-iac-bootstrap.md` (     148L) noDeps noKS
- `sheets/IP-002-cargo-workspace-cell-grid-kernel-domain.md` (     109L) noContract noKS
- `sheets/IP-003-formula-engine-kernel-domain-400-functions.md` (      98L) noContract noKS <100lines
- `sheets/IP-004-recalc-engine-dep-graph-parallel.md` (      90L) noContract noCFT noKS <100lines
- `sheets/IP-005-collab-crdt-loro-aligned-ws-0001.md` (      91L) noContract noCFT noKS <100lines
- `sheets/IP-006-large-sheet-storage-postgres-arrow-parquet-hybrid.md` (      79L) noContract noCFT noKS <100lines
- `sheets/IP-007-cell-grid-adapter-postgres-and-materialized-views.md` (     100L) noContract noCFT noKS
- `sheets/IP-008-formatting-pivot-charts-data-validation.md` (      83L) noContract noCFT noKS <100lines
- `sheets/IP-009-import-export-xlsx-calamine-rust-xlsxwriter-sandboxed.md` (     101L) noContract noCFT noKS
- `sheets/IP-010-sharing-acl-named-range-cedar.md` (      72L) noContract noCFT noKS <100lines
- `sheets/IP-011-ai-formula-smart-fill-foundry-runtime-bridge.md` (     105L) noContract noCFT noKS
- `sheets/IP-012-connected-sheets-comments-version-history-trigger-embed-bridge.md` (      65L) noContract noCFT noKS <100lines
- `sheets/IP-013-cell-grid-rest-leptos-wasm-app-license-gate.md` (      85L) noContract noCFT noKS <100lines
- `sheets/IP-014-observability-slo-manifests-9-openslo.md` (      68L) noContract noCFT noKS retired VCS ratchet <100lines
- `sheets/IP-015-hg-sheets-registration-and-branch-protection.md` (     126L) noContract noCFT noKS

### §15.shorts — `microservices/shorts/`

- `shorts/IP-001-iac-bootstrap.md` (      91L) noDeps noKS <100lines
- `shorts/IP-002-cargo-workspace-bootstrap.md` (     110L) noDeps noKS
- `shorts/IP-003-video-upload-bc.md` (      94L) noContract noDeps noKS <100lines
- `shorts/IP-004-video-transcode-bc.md` (      77L) noContract noKS <100lines
- `shorts/IP-005-video-storage-and-cdn-bc.md` (      61L) noContract noVerify noLanes noKS <100lines
- `shorts/IP-006-thumbnail-and-composition-bc.md` (      58L) noContract noLanes noKS <100lines
- `shorts/IP-007-audio-track-library-and-attribution-bc.md` (      59L) noContract noVerify noLanes noKS <100lines
- `shorts/IP-008-feed-timeline-and-watch-time-bc.md` (      57L) noContract noVerify noLanes noKS <100lines
- `shorts/IP-009-like-share-comment-and-repost-bc.md` (      54L) noContract noVerify noLanes noKS <100lines
- `shorts/IP-010-hashtag-and-trending-bc.md` (      48L) noContract noVerify noLanes noKS <50lines
- `shorts/IP-011-content-moderation-and-copyright-claim-bc.md` (      80L) noContract noVerify noKS <100lines
- `shorts/IP-012-age-gate-and-parental-controls-bc.md` (      76L) noContract noVerify noKS <100lines
- `shorts/IP-013-accessibility-captions-bc.md` (      56L) noContract noVerify noLanes noKS <100lines
- `shorts/IP-014-notifications-and-creator-analytics-bc.md` (      52L) noContract noVerify noLanes noKS <100lines
- `shorts/IP-015-drm-and-hg-shorts-registration.md` (      84L) noContract noVerify noKS <100lines

### §15.sites — `microservices/sites/`

- `sites/IP-001-iac-bootstrap.md` (      99L) noDeps noKS <100lines
- `sites/IP-002-site-bc-kernel.md` (      58L) noContract noCFT noDeps noKS <100lines
- `sites/IP-003-page-bc-kernel.md` (      49L) noContract noCFT noDeps noKS <50lines
- `sites/IP-004-block-bc-and-loro.md` (      49L) noContract noCFT noDeps noKS <50lines
- `sites/IP-005-theme-and-navigation.md` (      45L) noContract noCFT noDeps noKS <50lines
- `sites/IP-006-url-routing.md` (      86L) noContract noCFT noVerify noDeps noKS <100lines
- `sites/IP-007-domain-binding-acme.md` (      49L) noContract noCFT noDeps noKS <50lines
- `sites/IP-008-seo-and-sitemap.md` (      53L) noContract noCFT noDeps noKS <100lines
- `sites/IP-009-cms-collection.md` (      47L) noContract noCFT noDeps noKS <50lines
- `sites/IP-010-search-meilisearch.md` (      92L) noContract noCFT noDeps noKS <100lines
- `sites/IP-011-cdn-delivery-and-pipeline.md` (      53L) noContract noCFT noDeps noKS <100lines
- `sites/IP-012-policy-dpia-threat-model.md` (      90L) noContract noCFT noVerify noDeps noKS <100lines
- `sites/IP-013-contracts-and-capabilities.md` (      87L) noContract noCFT noVerify noDeps noKS <100lines
- `sites/IP-014-dashboards-runbooks-slos.md` (      83L) noContract noCFT noVerify noDeps noKS <100lines
- `sites/IP-015-hg-sites-maturity-claim.md` (      90L) noContract noCFT noVerify noDeps noKS <100lines

### §15.slides — `microservices/slides/`

- `slides/IP-001-layer-a-cdn-postgres-redis-s3-ws-gateway-iac.md` (     128L) noKS
- `slides/IP-002-presentation-slide-kernel-domain.md` (     106L) noKS
- `slides/IP-003-slide-layout-text-box-shape-kernel-domain.md` (     113L) noKS
- `slides/IP-004-asset-bcs-image-video-audio-adapters.md` (      73L) noKS <100lines
- `slides/IP-005-real-time-collaboration-loro-kernel-domain-adapter.md` (     100L) noVerify noKS
- `slides/IP-006-real-time-collaboration-worker-sdk.md` (      81L) noVerify noKS <100lines
- `slides/IP-007-chart-embed-bridge-to-sheets.md` (      64L) noVerify noKS <100lines
- `slides/IP-008-themes-templates-master-slide-editor.md` (      64L) noKS <100lines
- `slides/IP-009-animations-transitions-reduced-motion.md` (      93L) noVerify noKS <100lines
- `slides/IP-010-presenter-audience-view-broadcast-mode-livekit.md` (      85L) noVerify noKS <100lines
- `slides/IP-011-import-export-pptx-pdf-mp4-pipeline.md` (      93L) noVerify noKS <100lines
- `slides/IP-012-accessibility-ai-design-ai-content-generation.md` (      86L) noVerify noKS <100lines
- `slides/IP-013-acl-comments-version-history-embed-bridge.md` (      82L) noVerify noKS <100lines
- `slides/IP-014-visual-canvas-leptos-wasm-rest-sdk-app.md` (      89L) noVerify noKS <100lines
- `slides/IP-015-hg-slides-registration-and-branch-protection.md` (     103L) noKS

### §15.social — `microservices/social/`

- `social/IP-001-iac-bootstrap.md` (      83L) noDeps noKS <100lines
- `social/IP-002-cargo-workspace-bootstrap.md` (      74L) noDeps noKS <100lines
- `social/IP-003-user-profile-bc.md` (     128L) noDeps noKS
- `social/IP-004-follow-graph-bc.md` (     106L) noDeps noKS
- `social/IP-005-post-composition-bc.md` (     116L) noDeps noKS
- `social/IP-006-feed-timeline-bc.md` (     104L) noDeps noKS
- `social/IP-007-reactions-bc.md` (      62L) noDeps noKS <100lines
- `social/IP-008-mentions-and-hashtags-bc.md` (      69L) noDeps noKS <100lines
- `social/IP-009-trending-topics-bc.md` (      62L) noDeps noKS <100lines
- `social/IP-010-notifications-bc.md` (      62L) noDeps noKS <100lines
- `social/IP-011-content-moderation-bc.md` (      78L) noDeps noKS <100lines
- `social/IP-012-search-and-cedar-filter.md` (      78L) noDeps noKS <100lines
- `social/IP-013-age-verification-and-profile-verification.md` (      89L) noDeps noKS <100lines
- `social/IP-014-observability-slo.md` (      64L) noDeps noKS <100lines
- `social/IP-015-hg-social-registration-and-branch-protection.md` (      77L) noCB noDeps noKS <100lines
- `social/IP-016-minor-protection-strict-defaults.md` (      42L) noContract noCB noCFT noAccept noVerify noDeps noLanes noKS <50lines
- `social/IP-017-abuse-defence-edge-and-cedar.md` (      51L) noContract noCB noCFT noAccept noVerify noDeps noLanes CP <100lines
- `social/IP-018-dsa-compliance-overlay.md` (      43L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines

### §15.tasks — `microservices/tasks/`

- `tasks/IP-001-iac-bootstrap.md` (      97L) noDeps noKS <100lines
- `tasks/IP-002-cargo-workspace-bootstrap.md` (      85L) noDeps noKS <100lines
- `tasks/IP-003-task-store-kernel-domain.md` (      92L) noDeps noKS <100lines
- `tasks/IP-004-task-store-adapter-postgres.md` (      86L) noDeps noKS <100lines
- `tasks/IP-005-project-and-board-bc.md` (      86L) noDeps noKS <100lines
- `tasks/IP-006-custom-field-engine.md` (      82L) noDeps noKS <100lines
- `tasks/IP-007-dependency-graph-and-cycle-prevention.md` (      85L) noDeps noKS <100lines
- `tasks/IP-008-recurring-task-engine.md` (      81L) noDeps noKS <100lines
- `tasks/IP-009-state-workflow-engine-cross-link.md` (      86L) noDeps noKS <100lines
- `tasks/IP-010-view-engine-and-board-realtime.md` (      88L) noDeps noKS <100lines
- `tasks/IP-011-search-and-filter.md` (      88L) noDeps noKS <100lines
- `tasks/IP-012-bulk-edit-pipeline.md` (      87L) noDeps noKS <100lines
- `tasks/IP-013-rest-and-websocket-api-surface.md` (      89L) noDeps noKS <100lines
- `tasks/IP-014-ai-assist-bounds-and-eu-ai-act.md` (      96L) noDeps noKS <100lines
- `tasks/IP-015-hg-tasks-conformance.md` (      82L) noDeps noKS <100lines

### §15.tenancy — `microservices/tenancy/`

- `tenancy/IP-001-layer-a-postgres-citus-patroni-iac.md` (     139L) noDeps noKS
- `tenancy/IP-002-tenant-lifecycle-kernel.md` (     135L) noDeps noKS
- `tenancy/IP-003-tenant-lifecycle-domain.md` (      68L) noContract noCB noDeps noKS <100lines
- `tenancy/IP-004-tenant-lifecycle-usecase.md` (      68L) noContract noCB noDeps noKS <100lines
- `tenancy/IP-005-tenant-lifecycle-adapter-postgres.md` (      75L) noContract noCB noDeps noKS <100lines
- `tenancy/IP-006-isolation-policy-rls-generator.md` (      85L) noContract noCB noDeps noKS <100lines
- `tenancy/IP-007-isolation-policy-jwt-issuer.md` (      90L) noContract noCB noDeps noKS <100lines
- `tenancy/IP-008-cell-assignment-controller.md` (      88L) noContract noCB noDeps noKS <100lines
- `tenancy/IP-009-dsr-cascade-runner.md` (     108L) noContract noCB noDeps noKS
- `tenancy/IP-010-tenancy-rest-and-sdk.md` (      93L) noContract noCB noDeps noKS <100lines
- `tenancy/IP-011-audit-chain-integration.md` (      58L) noContract noCB noDeps noKS <100lines
- `tenancy/IP-012-branch-protection-and-release-pointers.md` (      72L) noContract noCB noDeps noKS retired VCS ratchet <100lines
- `tenancy/IP-013-canary-cohort-and-rollback-wiring.md` (      71L) noContract noCB noDeps noKS <100lines
- `tenancy/IP-014-tests-load-drills-observability-slos.md` (     118L) noContract noCB noDeps noKS
- `tenancy/IP-015-legacy-crates-migration.md` (      77L) noContract noDeps noKS <100lines
- `tenancy/IP-016-sub-scope-registry-kernel.md` (      25L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `tenancy/IP-017-reserved-namespace-enforcer.md` (      27L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `tenancy/IP-018-kyb-kyc-verifier-domain.md` (      35L) noContract noCB noCFT noAccept noVerify noDeps noLanes <50lines
- `tenancy/IP-019-dr-pairing-controller.md` (      36L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `tenancy/IP-020-data-residency-enforcer-adapter.md` (      21L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `tenancy/IP-021-lifecycle-locks-kernel.md` (      31L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `tenancy/IP-022-per-tenant-quota-usecase.md` (      27L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `tenancy/IP-023-sub-scope-registry-adapter-postgres.md` (      20L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `tenancy/IP-024-kyb-kyc-rest-and-async.md` (      27L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `tenancy/IP-025-dr-pairing-async-emit.md` (      25L) noContract noCB noCFT noVerify noDeps noLanes <50lines
- `tenancy/IP-026-quota-rest-and-sdk.md` (      26L) noContract noCB noCFT noVerify noDeps noLanes <50lines

### §15.translate — `microservices/translate/`

- `translate/IP-001-iac-and-pack-overlays.md` (     146L) noDeps noKS
- `translate/IP-002-translate-router-kernel.md` (     288L) noContract noDeps noKS
- `translate/IP-003-translate-router-domain.md` (     135L) noContract noCFT noAccept noDeps noKS
- `translate/IP-004-translate-router-usecase-and-api.md` (     172L) noContract noCFT noAccept noDeps noKS
- `translate/IP-005-translation-memory-stack.md` (     102L) noContract noCFT noAccept noDeps noKS
- `translate/IP-006-termbase-and-glossary-stack.md` (      92L) noContract noCFT noAccept noDeps noKS <100lines
- `translate/IP-007-quality-estimation-stack.md` (      98L) noContract noCFT noAccept noDeps noKS <100lines
- `translate/IP-008-language-detection-stack.md` (      79L) noContract noCFT noAccept noDeps noKS <100lines
- `translate/IP-009-document-translation-stack.md` (      84L) noContract noCFT noAccept noDeps noKS <100lines
- `translate/IP-010-bulk-translate-stack.md` (     107L) noContract noCFT noAccept noDeps noKS
- `translate/IP-011-real-time-stream-stack.md` (      81L) noContract noCFT noAccept noDeps noKS <100lines
- `translate/IP-012-engine-adapter-foundry-runtime.md` (      99L) noContract noCFT noAccept noDeps noKS <100lines
- `translate/IP-013-engine-adapters-external.md` (      98L) noContract noCFT noAccept noDeps noKS <100lines
- `translate/IP-014-router-rest-worker-app.md` (     115L) noContract noCFT noAccept noDeps noKS
- `translate/IP-015-hg-translate-gate-registration.md` (     155L) noContract noCFT noAccept noDeps noKS

### §15.workflowengine — `microservices/workflow-engine/`

- `workflow-engine/IP-001-layer-a-postgres-citus-redis-clickhouse-iac.md` (     105L) noDeps noKS
- `workflow-engine/IP-002-spec-store-kernel-domain.md` (     128L) noDeps noKS
- `workflow-engine/IP-003-state-machine-kernel-domain.md` (      69L) noDeps noKS <100lines
- `workflow-engine/IP-004-execution-engine-kernel-domain.md` (      89L) noAccept noDeps noKS <100lines
- `workflow-engine/IP-005-execution-engine-usecase-durable-execution.md` (     126L) noDeps noKS
- `workflow-engine/IP-006-event-bus-kernel-domain-adapter.md` (      86L) noDeps noKS <100lines
- `workflow-engine/IP-007-event-bus-rest-worker-sdk-app.md` (      66L) noDeps noKS <100lines
- `workflow-engine/IP-008-spec-store-usecase-api-adapter-rest-sdk-app.md` (      65L) noDeps noKS <100lines
- `workflow-engine/IP-009-execution-engine-rest-worker-sdk-app.md` (      63L) noDeps noKS <100lines
- `workflow-engine/IP-010-replay-debugger-backend-kernel-domain.md` (      69L) noAccept noDeps noKS <100lines
- `workflow-engine/IP-011-replay-debugger-backend-usecase-adapter.md` (      53L) noAccept noDeps noKS <100lines
- `workflow-engine/IP-012-replay-debugger-backend-rest-sdk-app.md` (      51L) noAccept noDeps noKS <100lines
- `workflow-engine/IP-013-observability-slo-manifests.md` (     101L) noDeps noKS retired VCS ratchet
- `workflow-engine/IP-014-branch-protection-and-hyperscaler-gates.md` (     141L) noCB noCFT noVerify noKS retired VCS ratchet
- `workflow-engine/IP-015-deterministic-replay-lane.md` (      83L) noDeps noKS <100lines

### §15.workflowstudio — `microservices/workflow-studio/`

- `workflow-studio/IP-001-layer-a-cdn-waf-postgres-redis-ws-gateway-iac.md` (     160L) noDeps noKS
- `workflow-studio/IP-002-visual-canvas-kernel-domain.md` (     183L) noDeps noKS
- `workflow-studio/IP-003-dsl-emitter-loader-kernel-domain.md` (     147L) noDeps noKS
- `workflow-studio/IP-004-dsl-emitter-loader-usecase-api-adapter-sdk.md` (     113L) noKS
- `workflow-studio/IP-005-collab-crdt-kernel-domain-adapter.md` (     146L) noKS
- `workflow-studio/IP-006-collab-crdt-worker-sdk.md` (     131L) noKS
- `workflow-studio/IP-007-node-library-registry-full.md` (     132L) noKS
- `workflow-studio/IP-008-llm-assist-adapter.md` (     134L) noKS
- `workflow-studio/IP-009-license-gate-cedar-full.md` (     156L) noKS
- `workflow-studio/IP-010-jurisdiction-overlay-renderer-full.md` (     145L) noKS
- `workflow-studio/IP-011-replay-debugger-frontend-full.md` (     132L) noKS
- `workflow-studio/IP-012-visual-canvas-leptos-wasm-rest-sdk-app.md` (     147L) noKS
- `workflow-studio/IP-013-observability-slo-manifests.md` (     118L) noKS retired VCS ratchet
- `workflow-studio/IP-014-branch-protection-and-hyperscaler-gates.md` (     139L) noKS retired VCS ratchet
- `workflow-studio/IP-015-hg-workflow-studio-registration-final.md` (     126L) noKS
- `workflow-studio/IP-016-svelte-flow-canvas-integration.md` (     181L) noCB noCFT noVerify noKS retired VCS ratchet
- `workflow-studio/IP-017-leptos-canvas-scaffold.md` (     140L) noCB noCFT noVerify noKS retired VCS ratchet
- `workflow-studio/IP-018-swiftui-canvas-impl.md` (     141L) noCB noCFT noVerify noKS retired VCS ratchet
- `workflow-studio/IP-019-compose-canvas-impl.md` (     143L) noCB noCFT noVerify noKS retired VCS ratchet
- `workflow-studio/IP-020-gtk-drawingarea-impl.md` (     150L) noCB noCFT noVerify noKS retired VCS ratchet
- `workflow-studio/IP-021-winui-canvas-impl.md` (     150L) noCB noCFT noVerify noKS retired VCS ratchet
- `workflow-studio/IP-022-loro-crdt-sync-binding.md` (     150L) noCB noCFT noVerify noKS retired VCS ratchet
- `workflow-studio/IP-023-presence-awareness-protocol.md` (     133L) noCB noCFT noVerify noKS retired VCS ratchet
- `workflow-studio/IP-024-1000-node-perf-bench.md` (     128L) noCB noCFT noVerify noKS retired VCS ratchet
- `workflow-studio/IP-025-codemirror-6-integration.md` (     138L) noCB noCFT noVerify noKS retired VCS ratchet
- `workflow-studio/IP-026-lsp-bridge.md` (     138L) noCB noCFT noVerify noKS retired VCS ratchet
- `workflow-studio/IP-027-cedar-grammar-impl.md` (     136L) noCB noCFT noVerify noKS retired VCS ratchet
