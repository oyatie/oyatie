# Standards + Specs Sweep — 2026-06-06

**Lane:** standards-specs (rest-of-docs reviewer)
**Corpus:** `/Users/jasonlee/Developer/source/docs/standards/` (103 files) + `/Users/jasonlee/Developer/source/docs/specs/` (116 files)
**Method:** Read architecturally load-bearing standards in full (data-class, ci-lanes, autonomy-ceiling, cedar-policy-discipline, authz-tier-boundaries, identity-vendor-isolation, gitops-iac-cluster-tier-boundaries, tenant-lifecycle, timescaledb-adoption, dependency-policy, INDEX); corpus-wide grep for every canon needle; sampled the spec set (almost all are `task-*` per-crate implementation slices).

---

## EXECUTIVE DIGEST — genuine canon contradictions, ranked

| # | Canon item | Severity | Scope | Headline |
|---|---|---|---|---|
| C1 | #2 Foundry RETIRED | **CRITICAL** | 50 standards files + 4 specs | "Foundry" still the live brand for the AI/agent substrate — as a 7-axis *pillar*, an `axis-foundry` *team owner* (19 files), a runtime (`oya-foundry-runtime-*`), and a cost center. Should be cloud-intelligence (substrate) / governance (fitness-policy lane). |
| C2 | #1 ADRs=SSOT, masterplan GENERATED | **HIGH** | ~21 standards (frontmatter) + INDEX authority-chain | Standards declare `/specs/decision-principles.json + /specs/forbidden-operations.json` as the *apex authority* and cite hand-authored `MASTERPLAN.md` Directives as authority. ADRs are framed as downstream, not SSOT. Masterplan-is-generated is acknowledged only inside one ci-lane row. |
| C3 | #4 unified oya-ci is endpoint; Argo NOT canonical | **HIGH** | gitops-iac-cluster-tier-boundaries.md; ci-lanes; release-management; regulatory-pack | `gitops-iac-…:87` "oyatie does NOT build replacements for ArgoCD / OpenTofu / Cluster API"; ArgoCD declared **canonical Tier-A owner**. Directly contradicts build-first-cutover-later + oya-ci-as-endpoint. |
| C4 | #3 Forgejo DROPPED (mirror at most) | **HIGH** | slice-ci-webhook-replay-guard{,-research}.md | Both CI slice specs treat **Forgejo as a live first-class webhook source** (`X-Forgejo-Delivery` as primary idempotency key), co-equal with GitHub. Forge canon = GitHub now → bespoke later; Forgejo dropped. |
| C5 | #9 tenant-CLASS not tenant-tier | **MEDIUM** | throttling-tiers; finops-cost-attribution; per-tenant-quotas; 1 spec | `Free / Pro / Enterprise` used as a **tenant-tier** axis; spec filename `task-intel-autonomy-ceiling-tenant-tier-policy.md` + `TenantCeiling` "per-surface tenant ceiling" use tenant-tier framing. Canon: tenant-**class**; reserve "tier" for namespaced axes only. |
| C6 | #4 Jenkins NOT canonical endpoint | **MEDIUM** | agentic-dev-team-optimization; brief-template | Jenkins presented as the **live coordination CI** ("live coordination uses plain git branches, PRs, Jenkins") and a mandatory brief section ("plain-git + PR + Jenkins governance lifecycle") with no operative-until-cutover framing. |

**Counts:** 50/103 standards carry Foundry residue; 19 name `axis-foundry` as a team owner; 21 assert the decision-principles/forbidden-ops apex. Data-tier canon (#5) is the **bright spot** — Redis→Valkey and the own-the-tier endpoint framing are handled correctly (see "Clean" section).

---

## TIER-1 CONTRADICTIONS (full detail)

### C1 — "Foundry" brand is still live everywhere (canon #2)

This is the single largest stale-framing problem in the corpus. Foundry appears not as incidental prose but as **live load-bearing architecture**:

- **As a pillar / axis.** `standards/data-class.md:98` — "The seven axes (SaaS, Workspace, Vertical, **Foundry**, Cloud, Search, Ads + Analytics) are pillars." `:104` — an entire cross-pillar transition-matrix row "SaaS → Foundry (Foundry agents act ON tenant data)". The 7-axis model itself names Foundry as Axis 4.
- **As a team owner (`axis-foundry`), 19 files.** Frontmatter `owner_team` / `deciders` / `owner` across: `code-style.md:8`, `ci-lanes.md:8`, `autonomy-ceiling.md:73`, `layer-enum-adr-0105.md:6`, `naming-convention-bnf-v4.md:6`, `git-workflow.md:122` (default issue owner), `plugin-authoring.md:8`, `release.md:8`, `event-schema-versioning-canonical.md:8`, `request-id-canonical.md:8`, `cursor-pagination-canonical.md:8`, `outbox-pattern-canonical.md:8`, `idempotency-keys-canonical.md:8`, `agentic-dev-team-optimization.md:7-8`, `code-review.md:24`, `observability.md:138`, `finops-cost-attribution.md:109`, `finops-cost-attribution-canonical.md:100`, `voice-video-call-architecture.md:1991` (`axis-foundry-runtime`).
- **As a runtime + capability substrate.** `autonomy-ceiling.md` is built around `oya-foundry-runtime-*::invoke` (`:81,84,89,132`), capability IDs `foundry.rag.semantic-search`, the "Foundry approval inbox" (`:181`), and cites **"ADR-0025 (Foundry consolidation)"** as authority (`:62,249`). `brownout-degradation-signal.md:113` — "Foundry runtime µservice in cell c-9876". `finops-cost-attribution-canonical.md:172` / `finops-cost-attribution.md:138` — "Foundry capability invocations" as a cost line.
- **In specs.** `specs/deep-dive-oyatie-sst-consolidation.md` is the smoking gun: `:12` "Builder-OS → **Foundry**", `:26` "Reversing ADR-0025 (Builder-OS → Foundry consolidation)", `:90-91` "seven axes (…**Foundry**…)" + "**Foundry** | Axis 4: AI agent runtime + engineering platform + control plane. Unified per ADR-0025", `:110` "`oya-foundry-*-kernel` crates … survive". `deep-dive-trace-…` mirrors it. `task-comms-email-bounce-dsn-classification.md:36` and `task-gate-run-all-affected-scope.md:92` reference a live "Foundry" pipeline/layer.

**Disposition:** systemic rename. Foundry → **cloud-intelligence** where it denotes the AI/agent runtime + capability substrate (data-class pillar row, autonomy-ceiling runtime, finops cost line, `axis-foundry-runtime`); Foundry → **governance** where it denotes the fitness/policy lane ownership (the `oya-foundry-*-kernel` fitness/policy crates the deep-dive flags as "survive", `axis-foundry` as the CI-lanes/quality owner). Note `ci-lanes.md:47` already has an `oya-governance-brand-residue` lane "tautological brand transition check (ADR-0017)" — it is **not catching** axis-foundry/Foundry, which means either the lane is unwired or its needle list is stale. Flag the lane itself.

### C2 — Authority inversion: decision-principles/forbidden-ops as apex, masterplan cited as hand-authored authority (canon #1)

Canon: ADRs are SSOT; the masterplan is a **generated projection** of the planning-impact ADR log.

The standards corpus asserts a *different* authority chain:
- `INDEX.md:24-25` `authority_chain_declaration`: "`/specs/decision-principles.json + /specs/forbidden-operations.json` > rest of docs/ > catalog records > Redirect-class files > working drafts" — **ADRs do not appear in the chain at all**, and the two JSON spec files sit at the apex.
- 21 standards carry `canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json` in frontmatter (data-class, autonomy-ceiling, dependency-policy, release-management, INDEX, etc.). Newer standards (i18n, a11y, otel-tail-sampling, realtime-transport, tenant-lifecycle) correctly point `canonical_authority` at a specific ADR — so the corpus is **internally inconsistent** about whether the apex is an ADR or the two JSON files.
- **MASTERPLAN.md cited as authority**, not as a generated artifact: `image-discipline.md:17,39` "Implements MASTERPLAN Directive 5"; `dependency-policy.md:19,45,153` "MASTERPLAN Directive 4/8"; `agent-instructions-discipline.md:19,43,107,112,188` "MASTERPLAN §7"; `claude-code-harness.md:46,99,113` "per MASTERPLAN §6/§12"; `git-workflow.md:36` "codifies MASTERPLAN.md"; `INDEX.md:107`. These treat MASTERPLAN.md as a **hand-authored directive source**.
- The *only* place masterplan-is-generated is acknowledged: `ci-lanes.md:97` `oya-governance-masterplan-drift` "the committed masterplan.generated.json equals the projection regenerated from the planning_impact ADR log (wraps `gen masterplan --check`)" and `:96` `oya-governance-adr-planning-completeness`. So the generation machinery exists in the gate catalog but the prose standards still cite the *old hand-authored* MASTERPLAN.md as authority.

**Disposition:** re-frame `canonical_authority`/`authority_chain_declaration` to place ADRs as SSOT and the masterplan (+ decision-principles/forbidden-ops, if those are themselves ADR-generated) as **generated** downstream. Reconcile the two competing apex conventions. Every "Implements MASTERPLAN Directive N" cite should resolve to the originating ADR.

### C3 — ArgoCD/OpenTofu/ClusterAPI declared canonical + explicit "we do NOT build replacements" (canon #4)

`standards/gitops-iac-cluster-tier-boundaries.md`:
- `:11` "**Tier A — ArgoCD** owns app deploy" — ArgoCD is the canonical owner of *all* app-deploy resource kinds (`:22-26` boundary table).
- `:87-89` **"oyatie does NOT build replacements for ArgoCD / OpenTofu / Cluster API. The in-house contribution is the boundary table itself + the discipline gate."**

This is the cleanest single contradiction with canon #4/#11: the canon is **build-first-cutover-later / own-endpoint-vendor-bridge-ratchet**, with Argo *operative-until-cutover then retired* and a unified **oya-ci (Run + graph; Prow+Tekton+Argo absorbed)** as the canonical endpoint. A standard that hard-codes "we will never replace ArgoCD" is the opposite posture. Supporting hits: `regulatory-pack-authzpolicy-overlays.md:119` "multi-cluster federation via ArgoCD ApplicationSets"; `release-management.md:103,230` Argo Rollouts/Flagger as the progressive-delivery rail (this one is more defensible as a transitional bridge, but carries no operative-until-cutover framing).

**Disposition:** re-frame ArgoCD/OpenTofu/ClusterAPI as **transitional vendor bridges** under the oya-ci endpoint, with explicit cutover/ratchet language; delete the "does NOT build replacements" sentence or invert it to "bridges until oya-ci absorbs the tier."

### C4 — Forgejo treated as a live webhook source (canon #3)

`specs/slice-ci-webhook-replay-guard-research.md` + `slice-ci-webhook-replay-guard.md`:
- `research:3` scope "Standards governing webhook idempotency … **Forgejo**"; `:14` "Every mainstream webhook sender (GitHub, **Forgejo**, …)"; `:28-53` an entire section "Forgejo delivery-header semantics"; `:79,100` **Rule MUST-1 / MUST-3 make `X-Forgejo-Delivery` the primary idempotency key**; `:168-170` "Forgejo automatic retry behaviour."

Canon #3: Forge = GitHub NOW → bespoke VCS later; **Forgejo DROPPED (mirror at most)**. Building the webhook-replay guard's *primary key* around Forgejo delivery headers bakes a dropped vendor into a contract.

**Disposition:** re-scope the replay-guard to GitHub delivery semantics (`X-GitHub-Delivery`) as primary; demote Forgejo headers to an optional mirror-compat alias or drop. The implemented crate `oya-ci-webhook-gateway-app` naming is fine (canon-aligned with oya-ci).

---

## TIER-2 CONTRADICTIONS / STALE FRAMING

### C5 — tenant-tier vocabulary (canon #9)
- `throttling-tiers.md:41,89` `Free=1k, Pro=10k, Enterprise=negotiated` — a tenant-**tier** plan axis. `finops-cost-attribution.md:135` "Tenant t-1234, **Pro tier**". `per-tenant-resource-quotas-canonical.md:80` "Sovereign-tenant **tier**". `tenant-lifecycle.md:122` "Pro → Enterprise **tier** upgrade".
- Spec: `task-intel-autonomy-ceiling-tenant-tier-policy.md` (filename + body `TenantCeiling` "per-surface tenant ceiling policy"). Canon-OK underlying mechanic, but the **tenant-tier** label is the retired vocab.

**Disposition:** rename to tenant-**class** (Free/Pro/Enterprise are classes). Reserve "tier" for namespaced axes (`autonomy_tier`, `eu_ai_act_risk_tier`, `dr_tier`, `storage_tier`). Note `authz-tier-boundaries.md` ("two tiers": edge/origin) and `throttling-tiers.md`'s `oya-throttle-class` labels are **fine** — those are not tenant tiers.

### C6 — Jenkins as live canonical CI (canon #4)
- `agentic-dev-team-optimization.md:23` "live coordination uses plain git branches, PRs, **Jenkins**, and `oya gate` / `oya verify`." `brief-template.md:53,314` make "**Jenkins** contexts" / "plain-git + PR + **Jenkins** governance lifecycle" a mandatory brief section. No operative-until-cutover framing; Jenkins reads as the endpoint.

**Disposition:** re-frame Jenkins as operative-until-cutover; the canonical endpoint is **oya-ci (Run + graph)**. (Caveat: confirm against ADRs whether Jenkins is even the current operative bridge — canon #4 names Jenkins/Argo as the operative-until-cutover pair, so a *bridge* mention is allowed; it just needs the cutover framing.)

### Authority-tier nuance (not a contradiction, but flag)
`per-tenant-resource-quotas-canonical.md:5` `classification: INTERNAL_ONLY` and the `authority_tier: 2` frontmatter convention across standards are fine; but the "Tier 2" authority-depth column in `INDEX.md` plus tenant "tier" plus authz "tier" plus dr/storage "tier" creates **vocabulary collision** around the word "tier." Recommend the glossary explicitly partition tenant-class vs namespaced-tier vs authority-tier vs edge/origin-tier vs IaC-tier (gitops-iac uses Tier A/B/C for *tooling layers*). This is exactly the kind of overload canon #9 is trying to kill.

---

## CLEAN / CANON-ALIGNED (positive findings — do NOT churn these)

- **Data-tier vendor-bridge framing (canon #5) — correct.** `dependency-policy.md:93,219` and `lts-versions-verified.md:35,112,152` handle **Redis → Valkey** exactly right: Valkey BSD-3 canonical per ADR-0336, Redis≥7.4 tri-license forbidden, DragonflyDB BSL-1.1 forbidden. `realtime-transport-tier.md:55`, `messenger-e2e-encryption-mls.md:2627`, `voice-video-call-architecture.md:277` already use Valkey. **Kafka → Pulsar** partially present (`stream-processing-rubric.md:77` references Pulsar consumer offsets) — but note Kafka still appears as a live broker in `event-schema-versioning-canonical.md:30`, `asyncapi-3-1-authoring.md:49`, `layer-enum-adr-0105.md:751`, `emoji-sticker-reaction-system.md` without Pulsar-bridge framing. **Sub-flag:** Kafka→Pulsar transition is *less complete* than Redis→Valkey; recommend a sweep to add Pulsar-canonical framing wherever raw "Kafka" appears as the endpoint.
- **Citus/Milvus/ClickHouse as transitional (canon #5) — mostly OK.** `ci-lanes.md:88-89` frames Milvus-canonical / ClickHouse-canonical as *advisory tier-discipline gates* per ADR-0192/0193, and `dependency-policy.md:216-217` layers ClickHouse on Apache Iceberg (own-the-format posture). These read as bridges, not permanent endpoints. `schema-migration.md:24` + `emoji-sticker-reaction-system.md` use "Citus" as the shard runtime — acceptable as a transitional bridge; no "permanent" claim found.
- **Identity vendor isolation (canon #6) — correct.** `identity-vendor-isolation.md` treats **Zitadel as a bridge**: confined to an explicit adapter set, vendor-neutral kernel traits everywhere else, explicit Phase-2 swap to owned `oya-identity-server`. This is the model the data/CI standards should imitate.
- **Cedar as contract / owned engine (canon #6) — correct.** `cedar-policy-discipline.md` + `cedar-policy-authoring.md` + `authz-tier-boundaries.md` treat Cedar as the policy **contract** with an owned PDP; no "Amazon Verified Permissions as the engine" lock-in (AVP only cited as a doc reference). Aligned.
- **Isolation (canon #7) — broadly OK.** `regulatory-pack-authzpolicy-overlays.md:73` uses `runc` by default with `kata-clh-sev-snp` upgrade for sovereign packs per the ADR-0147 ladder; `wasm-runtime-canonical.md:35` routes full-POSIX-untrusted to gVisor/Firecracker. No "secure-by-default-native" / "native-default" framing found that contradicts assume-breach-microVM-default. **Caveat:** I did not find an explicit statement of *framekernel-host as committed endpoint* or *assume-breach microVM as the DEFAULT* in these standards — the default here is `runc` with microVM as an *upgrade*, which is arguably weaker than "assume-breach microVM DEFAULT" (canon #7). Worth a closer ADR cross-check; flagging as a possible soft contradiction, not asserting it.

---

## AI-SLOP / QUALITY NOTES

- **`hyperscaler-best-practices.md` (333 lines)** — heavy duplication: the TL;DR (`:25-31`) is restated almost verbatim in the conclusion (`:329-333`). Reads as research-dump with fabricated-precision citations density; defensible as a one-time research artifact but should be marked GENERATED-REFERENCE / archived, not maintained as a living standard. It also references M01-P15 wave vocab (`:314`) — borderline retired-wave-vocab (canon #9), though "M01-P15" is a gate-phase label, not MVP-wave, so likely OK.
- **`anti-patterns.md` (2914 lines)** — enormous; self-aware about it (`:1595` "this catalogue itself is hand-authored because the user explicitly…"). Not slop per se, but a buildability/length-cap outlier vs the 250-line standard cap declared in `INDEX.md:92` and most frontmatter `length_cap: 250`. Flag as length-cap violation. `cedar-policy-authoring.md` (806), `layer-enum-adr-0105.md` (834), `naming-convention-bnf-v4.md` (825) similarly blow the 250 cap.
- No fabricated-stability/internal-contradiction slop detected in the sampled `task-*` specs — they are tight, single-crate, pure-function implementation slices with explicit ADR authority cites (ADR-0509 single-crate-per-service, ADR-0131 flat layout). These are the healthiest part of the corpus.

## REACHABILITY CLASSES

- **DECISION→ADR:** the canon-contradiction fixes (C1–C6) are all DECISIONs requiring ADR amendment (rename ADR-0025 Foundry-consolidation framing; re-author the authority-chain ADR; re-scope the ArgoCD/Forgejo/Jenkins endpoint ADRs; tenant-class glossary ADR).
- **GENERATED-REFERENCE:** `INDEX.md`, `lts-versions-verified.md`, `hyperscaler-best-practices.md`, `ci-lanes.md` (explicitly "human-readable mirror" of `registry/quality/lanes.yaml`) — these are projections; should be regenerated post-amendment, not hand-edited.
- **INSTRUCTION→session-context-bundle:** `brief-template.md`, `agent-instructions-discipline.md`, `claude-code-harness.md`, `multi-agent-tool-map.md` are agent-operating instructions.
- **ORPHAN candidates:** `deep-dive-oyatie-sst-consolidation.md` + `deep-dive-trace-…` (specs/) are bominal→oyatie consolidation deep-dives pinned to the retired 2026-05-09 reframing (Builder-OS→Foundry); once C1/C2 land these are historical — archive, do not maintain.

---

## SAMPLED-BUT-CLEAN (scanned, no canon issue)
`a11y-canonical`, `i18n-canonical`, `wcag-2-2-aa-checklist`, `rtl-rendering`, `locale-routing`, `openapi-3-2-authoring`, `asyncapi-3-1-authoring`, `proto3-authoring`, `openslo-authoring`, `incident-severity`, `postmortem-template`, `on-call`, `observability-slo`, `step-up-auth-classes`, `mls-rfc-9420-conformance`, `messenger-e2e-encryption-mls`, `wasm-runtime-canonical`, `saga-compensation-policy`, `outbox-pattern-canonical`, `idempotency-keys-canonical`, `request-id-canonical`, `cursor-pagination-canonical`, and the bulk of the `task-*` spec slices. The `task-*` specs (≈100 of 116) are implementation slices; spot-checks (cedar-policy-authoring-lint, cloud-iac-gitops-drift, webauthn-packtier, intel-autonomy-ceiling) showed no canon contradictions beyond the tenant-tier filename (C5) and the shared Foundry/Forgejo residue already captured.
