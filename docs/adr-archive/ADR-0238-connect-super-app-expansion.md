---
id: ADR-0238
status: Superseded
deciders: council-architecture, council-product, council-privacy, axis-mail, axis-messenger, axis-calendar, axis-community, axis-social, axis-shorts, axis-network, axis-anonymous, ops-sre-reliability
date: 2026-05-17
owner: council-architecture
supersedes: []
superseded_by: [ADR-709]
amended_by: [ADR-0334]
related: [ADR-0056, ADR-0060, ADR-0105, ADR-0110, ADR-0123, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0237]
related_specs:
  - /specs/per-microservice-flat-layout.json
  - /specs/microservices/mail.json
  - /specs/microservices/messenger.json
  - /specs/microservices/calendar.json
session_context:
  authored: 2026-05-17
  renumbered_2026_05_18: |
    Originally drafted as ADR-0126 in the oyatie 2026-05-17 session.
    Renumbered to ADR-0238 on 2026-05-18 to avoid collision with dev's
    ADR-0126 (Employment classification, PR #135). See "Numbering note"
    section at end of file for the full rebase note.
  parallel_session_caveat: |
    Authored in oyatie 2026-05-17 as the Connect-specific dissolution
    decision body. The oyatie side decision (this ADR) takes precedence
    per feedback_bominal_inheritance_precedence ("oyatie session decisions
    override Bominal").
bominal_source: |
  Authored in oyatie 2026-05-17 (originally ADR-0126); renumbered to
  ADR-0238 on 2026-05-18. Bominal-side ADR-0126 is a different decision
  (Employment classification — 8-class enum) per docs/decisions/ADR-0060
  §inheritance table; the Bominal number is inherited only into the HR/
  payroll product line. The renumber eliminates any cross-axis collision
  (HR/payroll vs. Connect-dissolution) so the two IDs never coexist in a
  single artifact.
purpose: |
  Decompose the legacy super-app into 8 first-class, flat, single-concern
  µservices per ADR-0131 + ADR-0132. Establish the new µservice topology,
  per-µservice SLO authority, per-µservice ChangeSet lane, and the umbrella
  retirement trigger that ADR-0237 then operationalises as a Strangler-pattern
  migration.
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0238: super-app expansion into 8 flat µservices

## Status

Accepted — 2026-05-17.

## Date

2026-05-17.

## Context

The legacy `Connect` super-app — inherited as a Bominal-shaped grouping wrapper —
bundled mail, messenger, calendar, community, social, shorts (short-form video),
network (professional graph) and anonymous (zero-knowledge messaging) into a
single product line with shared release cadence, shared SLOs, shared deployment
topology, and shared identity surface. The wrapper materialised in oyatie as:

- `crates/oya-mail-domain` (extant) and the planned `oya-mail-*`
  layer fan-out (kernel, usecase, api, adapter, rest, worker, sdk, app).
- `crates/oya-messenger-domain` (extant) and its planned layer fan-out.
- `crates/oya-calendar-domain` (extant) and its planned layer fan-out.
- Speculative crates for community / social / shorts / network / anonymous that
  were *scoped but never landed* (zero crates of those names exist in the
  current workspace — verified 2026-05-17 via `find crates -maxdepth 1 -type d
  -name 'oya-connect-{community,social,shorts,network,anonymous}-*'`).

Three structural pressures forced the unbundling:

1. **Independent scaling dimensions.** Mail scales on mailbox-count + inbound
   message-rate; messenger scales on persistent-connection-count + message-rate;
   calendar scales on event-write-rate + recurring-expansion-rate; community
   scales on post-write-rate + thread-tree fan-out; social scales on feed-read
   QPS + interaction-write-rate; shorts scales on video-ingest GB/s +
   transcode-CPU + CDN egress; network scales on edge-write-rate + graph
   traversal QPS; anonymous scales on E2E-key-rotation rate. A single
   Connect-pod HPA cannot satisfy all eight simultaneously.
2. **Per-concern SLO targets.** ADR-0139 (agentic SLO-gated promotion) requires
   each µservice to own an OpenSLO file; an umbrella SLO that aggregates eight
   user-visible surfaces hides regressions and violates
   `feedback_no_silent_regression`. The Hyrum's-Law surface of a bundled SLO
   becomes any latency quirk observable across any of the eight, with no clean
   migration path when one surface needs a tighter target.
3. **Per-concern compliance overlays.** Mail must comply with KR-FSS 5-year
   retention + 전자문서법 audit-chain seal; calendar must support CalDAV (RFC
   4791); messenger must ship MLS RFC 9420 (E2E group messaging); anonymous
   must satisfy zero-knowledge / unlinkability invariants. These overlays do
   not share data classifications or retention floors; co-locating them inside
   a single `Connect` µservice forces the strictest overlay onto all eight,
   pricing every surface at the cost of the most-regulated one.

ADR-0132 establishes the universal no-grouping forward-policy and explicitly
delegates Connect-specific topology to this ADR. ADR-0131 establishes the
per-µservice flat layout each new µservice must adopt. This ADR is the
Connect-specific decomposition decision body that completes the triple.

## Decision

The legacy super-app is **dissolved** into first-class flat
µservices and community-hosted posting modes, each owning one user-facing
concern per ADR-0131:

| µservice | Concern | Folder | Crate prefix (BNF v4.1) |
|---|---|---|---|
| `mail` | Email (SMTP/IMAP/JMAP, mailbox, search, retention, legal-hold, eDiscovery) | `microservices/mail/` | `oya-mail-*` |
| `messenger` | Real-time messaging (channels, DMs, threads, MLS E2E, huddles, presence) | `microservices/messenger/` | `oya-messenger-*` |
| `calendar` | Scheduling (events, invitations, recurring, CalDAV, time-zones, rooms) | `microservices/calendar/` | `oya-calendar-*` |
| `community` | Forum-class post store, thread trees, voting, moderation, KB articles, Teamblind-style anonymous workplace discussion, Handshake-style jobs/recruitment, and LinkedIn jobs/profile/recruiter subset absorbed from retired network | `microservices/community/` | `oya-community-*` |
| `social` | Personal-context feed, interactions, follow graph | `microservices/social/` | `oya-community-social-*` |
| `shorts` | Short-form video (ingest, transcode, feed, CDN) | `microservices/shorts/` | `oya-shorts-*` |
| `community` (anonymity-mode) | Anonymity posting-mode capability tier within community: persona-anchored (TeamBlind-class), pseudonymous (Reddit-class), fully-anonymous (whistleblower/press-source/bug-bounty per ADR-0300) | `microservices/community/` (see `community/policy/anonymity-mode-*.cedar`) | `oya-community-*` |

### Structural commitments per µservice

1. **One ChangeSet lane per µservice.** Each ships, claims, verifies, and
   promotes independently via `oya vcs claim/verify/done/promote` per ADR-0110.
   No bundled ChangeSet may straddle two of the eight.
2. **One OpenSLO file per µservice.** Authored at
   `microservices/<ms>/slos/*.openslo.yaml` per ADR-0139 before any µservice
   can promote past dev. No umbrella SLO file may aggregate across two of the
   eight.
3. **One IaC slice per µservice.** Per-µservice Helm/Kustomize charts under
   `microservices/<ms>/iac/`; per-µservice K8s namespace; per-µservice
   PodDisruptionBudget; per-µservice HPA on the metric matching its actual
   scaling dimension.
4. **One HG-<MS> hyperscaler-maturity gate per µservice** per ADR-0123 + ADR-0133:
   HG-MAIL, HG-MESSENGER, HG-CALENDAR, HG-COMMUNITY, HG-SOCIAL, HG-SHORTS,
   HG-NETWORK, HG-ANONYMOUS — each registered in
   `/specs/hyperscaler-gates.json`.
5. **No direct cross-µservice imports.** Cross-µservice data flow uses Workflow
   events + Ontology reads/writes only, per
   `feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145).md`. Example: mail emits
   `MailReceived`; calendar may consume it via Workflow handoff, never via
   `use oya_mail_*` in a calendar crate.
6. **Dual-context isolation invariant** (Personal vs. Professional) is preserved
   at the kernel layer of each µservice that owns user-context-bound data
   (mail, messenger, calendar, social, anonymous). Details never cross context
   boundaries except via explicit invitation or policy-bound projection.

### umbrella µservice retirement trigger

The umbrella µservice (currently materialised only as the
`crates/oya-connect-{mail,messenger,calendar}-domain` legacy stubs plus three
`/specs/microservices/*.json` reference pointers) **retires** when **all 8**
of the following conditions hold simultaneously:

1. HG-MAIL accepts at p99 SLOs sustained 30d.
2. HG-MESSENGER accepts at p99 SLOs sustained 30d.
3. HG-CALENDAR accepts at p99 SLOs sustained 30d.
4. HG-COMMUNITY accepts at p99 SLOs sustained 30d.
5. HG-SOCIAL accepts at p99 SLOs sustained 30d.
6. HG-SHORTS accepts at p99 SLOs sustained 30d.
7. HG-NETWORK accepts at p99 SLOs sustained 30d.
8. HG-ANONYMOUS accepts at p99 SLOs sustained 30d.

Until all 8 trigger, the umbrella stubs remain in place and the
Strangler-pattern migration owned by ADR-0237 governs traffic-shifting,
deprecation notices, and the eventual code-removal sweep.

## Alternatives Considered

### (a) Keep the umbrella µservice

- **Pros**:
  - Zero migration cost — current state preserved.
  - Single product brand surface (matches "Google Workspace" / "Microsoft 365"
    marketing model).
  - One identity-and-billing surface for end users.
- **Cons**:
  - Forces independent scaling dimensions into a single HPA → wastes capacity
    on under-utilised surfaces and under-provisions the bottleneck.
  - Bundled SLO hides regressions per `feedback_no_silent_regression`.
  - Strictest compliance overlay (KR-FSS retention floor, MLS E2E, zero-
    knowledge) prices all eight at the cost of the most-regulated.
  - Violates ADR-0132 forward-policy outright.
- **Rejected** because the scaling-dimension and SLO-hiding harms are
  structural, not addressable by tooling. Marketing-brand unification is a
  GTM-layer concern resolvable without an architecture-layer bundle (per
  ADR-0132 §"brand-layer vs architecture-layer").

### (b) Split only mail + messenger

- **Pros**:
  - Lower migration cost — only the two extant legacy crate families need
    dissolution (community / social / shorts / network / anonymous never
    materialised as crates).
  - Mail + messenger have the most distinct scaling dimensions (mailbox-count
    vs. persistent-connection-count); biggest win for least disruption.
  - Calendar can stay bundled with because CalDAV scaling is closer to
    mail's pattern.
- **Cons**:
  - Leaves calendar / community / social / shorts / network / anonymous in a
    bundle that still violates ADR-0132 forward-policy.
  - Half-measure forces a second dissolution ADR within 12 months; doubles ADR
    churn and CI-lane authoring cost.
  - Compliance-overlay mismatches still apply: anonymous's zero-knowledge
    invariant cannot coexist in the same µservice as calendar's CalDAV plaintext
    invitations.
- **Rejected** because half-dissolution is not a stable terminal state; the
  remaining six concerns each violate ADR-0132 individually, so the cost of a
  second ADR + second migration is strictly higher than doing the full split now.

### (c) Split all 8 into first-class flat µservices  ← **CHOSEN**

- **Pros**:
  - One terminal-state ADR; no successor-IP dissolution churn.
  - Per-µservice SLO, IaC, HPA, HG gate, ChangeSet lane — uniform shape
    matching all other oyatie µservices (workflow-studio, ontology, tenancy,
    audit-chain, governance, observability, etc.).
  - Independent scaling dimensions honored per-concern.
  - Per-concern compliance overlay isolated; KR pack, EU pack, US pack land
    once per µservice instead of once across an unbundled umbrella.
  - Aligns with industry precedent: AWS / Google / Microsoft / Stripe each
    ship per-surface microservices, never per-suite bundles, at architecture
    layer.
- **Cons**:
  - Highest one-time migration cost: 3 extant legacy crate families
    (`oya-connect-{mail,messenger,calendar}-*`) must be deprecated and
    eventually removed per ADR-0237's Strangler migration; 5 net-new
    µservices (`community`, `social`, `shorts`, `network`, `anonymous`) must
    be stood up from scratch.
  - 8 HG gates to author, 8 OpenSLO files to author, 8 PRDs to author, 8 IaC
    slices to author.
  - Brand-layer concept of "Connect" persists only in marketing copy; some
    end-user-facing surfaces must rename (e.g., "Mail" → "Mail").
- **Accepted** despite higher one-time cost because (i) the terminal-state
  shape matches the rest of oyatie's flat catalog, (ii) the Strangler
  migration in ADR-0237 spreads the cost over 6–12 months, and (iii) ADR-0132
  forward-policy already constrains every new µservice to this shape — making
  the only legacy exception would be a unique-snowflake violation.

## Consequences

### Positive

- **8 independent ChangeSet lanes.** Each µservice claims, verifies, ships
  separately; no cross-µservice serialisation of the agentic merge queue.
- **Per-µservice SLO authority.** Each `microservices/<ms>/slos/*.openslo.yaml`
  governs only that µservice's release-pointer; ADR-0139's gated promotion
  applies per-µservice.
- **Per-µservice IaC + HPA + cost-budget.** Mimir cardinality budgets, K8s
  PodDisruptionBudgets, per-µservice cost budgets all flow from the flat
  µservice topology.
- **Per-µservice compliance overlay.** pack-kr lands as
  `microservices/<ms>/policy/pack-kr/` per µservice; no umbrella overlay file
  exists.
- **Brand-layer "Connect" survives.** Marketing / GTM may still call the
  end-user bundle "Connect" or rebrand to "Oyatie Personal" / "Oyatie Work";
  brand-layer concept persists, architecture-layer bundle does not.

### Negative

- **3 legacy crate families need explicit deprecation.** `oya-mail-*`,
  `oya-messenger-*`, `oya-calendar-*` exist today; ADR-0237
  owns their Strangler migration to `oya-{mail,messenger,calendar}-*`.
- **5 net-new µservices to stand up.** `community` (PRD authored, 126 files populated as of 2026-05-17 scaffold), `social` (96 files populated), `shorts` (97 files populated), `network` (100 files populated), `anonymous`
  (102 files populated). Each needs PRD + threat-model + dpia + compliance + capacity-model
  + cost-budget + IaC + SLO + 15-IP phase plan + HG gate. File counts verified via `find microservices/<ms>/ -type f | wc -l` on 2026-05-17.
- **Hyrum's-Law exposure.** Any external consumer who depended on
  `oya-connect-*` symbol paths, error variant ordering, timing characteristics
  of bundled handoffs, or umbrella SLO targets must migrate per ADR-0237's
  feature-flagged canary. Per `feedback_no_silent_regression`, every removed
  symbol or changed behaviour requires deprecation notice + ADR + sunset
  schedule.

### Migration cost

| Cost class | Quantity | Owner | Carrier ADR |
|---|---|---|---|
| Extant legacy crate families to deprecate | 3 (mail, messenger, calendar) | axis-mail + axis-messenger + axis-calendar | ADR-0237 |
| Net-new µservices to stand up | 5 (community, social, shorts, network, anonymous) | new axis-community + axis-social + axis-shorts + axis-network + axis-anonymous (axes to be commissioned in M03 phase plan) | per-µservice phase-01 |
| HG gates to register in `/specs/hyperscaler-gates.json` | 8 | per-µservice owners | per-µservice IP-NNN-hg-*-authority-cohesion |
| OpenSLO files to author | 8 | per-µservice owners | per-µservice IP-NNN-iac-bootstrap |
| PRDs to author | 5 net-new (mail/messenger/calendar PRDs already shipped) | per-µservice owners | per-µservice phase-00 |
| External consumers of `oya-connect-*` to migrate | unknown (verified by dependency-graph CI lane per ADR-0237 §Step 4) | each consumer's owning axis | ADR-0237 |

### Operational

- **New CI lane: `oya-governance-connect-umbrella-retirement-readiness`**
  (REPORT-ONLY on dev until HG-MAIL, HG-MESSENGER, HG-CALENDAR all green;
  then BLOCKER for the umbrella-retirement ChangeSet).
- **Per-µservice CI lanes** already covered by `per-microservice-layout`
  (ADR-0131) and `no-grouping` (ADR-0132). No additional lane needed.
- **Deprecation surface:** `microservices/{mail,messenger,calendar}/deprecation-notice.md`
  (authored per ADR-0237); `microservices/connector/RETIREMENT-PLAN.md` (this
  ADR cycle).

## Clean Architecture Impact

| Lane | Impact | Action |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Reinforced — no cross-µservice direct imports | per-µservice owners ensure no `use oya_<other-ms>_*` in their crates |
| `per-microservice-layout` (ADR-0131) | Reinforced — 8 new conforming layouts | each new µservice ships flat per ADR-0131 |
| `no-grouping` (ADR-0132) | Reinforced — dissolution is the terminal-state proof | this ADR proves the policy at concrete scale |
| `connect-umbrella-retirement-readiness` (NEW, lane authored under ADR-0237) | New REPORT-ONLY→BLOCKER | gates the eventual `microservices/connector/` folder removal |

## Verification

```bash
# All 8 µservice folders exist (5 net-new + 3 already present)
test -d microservices/mail && test -d microservices/messenger && test -d microservices/calendar && \
test -d microservices/community && test -d microservices/social && \
test -d microservices/shorts && test -f microservices/network/RETIRED.md && test -d microservices/community
# Note: microservices/anonymous/ was deleted 2026-05-21; anonymity is a
# posting-mode capability tier within microservices/community/.

# Each µservice has its OpenSLO file (ADR-0139 invariant)
for ms in mail messenger calendar community social shorts network anonymous; do
  ls microservices/$ms/slos/*.openslo.yaml >/dev/null 2>&1 || echo "MISSING SLO: $ms"
done

# Each µservice has its HG-<MS> gate registered
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims

# Per-µservice flat layout
cargo run -p oya-dev-cli -- gate validate per-microservice-layout

# No-grouping forward-policy holds
cargo run -p oya-dev-cli -- gate validate no-grouping

# Cross-µservice import refusal
cargo run -p oya-dev-cli -- gate validate authority-cohesion
```

## References

- ADR-0056: BNF v4.1 naming.
- ADR-0060: Bominal inheritance precedence.
- ADR-0105: 13-layer enum.
- ADR-0110: ChangeSet state machine.
- ADR-0123: Hyperscaler maturity claim gate.
- ADR-0139: Agentic SLO-gated promotion.
- ADR-0131: Per-microservice flat layout.
- ADR-0132: No-grouping forward-policy.
- ADR-0133: Industry best-practice conformance program.
- ADR-0237: dissolution Strangler migration (operational companion).
- `/specs/per-microservice-flat-layout.json`.
- `/specs/microservices/{mail,messenger,calendar}.json` (flat per-µservice specs; legacy `/specs/microservices/*.json` retired via the specs/products → specs/microservices flatten — see `specs/microservices/RETIREMENT.md`).
- `feedback_workflow_objectgraph_adapter_layer.md`.
- `feedback_no_silent_regression.md`.
- `feedback_flat_product_catalog.md`.
- `feedback_bominal_inheritance_precedence.md`.
- Industry: AWS / Google / Microsoft / Stripe per-surface microservice precedent.

## Numbering note

Originally drafted as **ADR-0126** in the oyatie session 2026-05-17. Renumbered
to **ADR-0238** on 2026-05-18 to avoid collision with dev's ADR-0126
(Employment classification, PR #135), which is the Bominal-inherited HR/payroll
ADR-0126.

- **Source slot**: ADR-0126 (oyatie 2026-05-17 session draft).
- **Target slot**: ADR-0238 (next free slot after ADR-0237 Connect-dissolution
  Strangler migration).
- **Reason**: Cross-axis collision avoidance. Per
  `feedback_bominal_inheritance_precedence`, oyatie session decisions override
  Bominal; renumbering rather than overriding leaves the HR/payroll lineage
  intact and preserves trace-back for the Connect-dissolution lineage too.
- **Cross-reference sweep**: All in-repo cross-references to ADR-0126 in the
  Connect-dissolution lineage (ADR-0132 §related, ADR-0132 §body, ADR-0133
  §related, ADR-0237 §related + §body) were rewritten to ADR-0238 in the same
  ChangeSet that performed this rename. The dev-side ADR-0126
  (Employment classification) is untouched.
- **Spec-path sweep**: At the same time, `/specs/microservices/{connect,enterprise}/*.json`
  flattened into `/specs/microservices/*.json` per ADR-0132 + the 2026-05-18
  user directive "retire products terminology"; references within this ADR
  were updated accordingly.
