---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: messenger
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-messenger + council-architecture
deciders: axis-messenger, council-architecture, gtm-customer-success
related_adrs: [ADR-0123, ADR-0135, ADR-0131, ADR-0132, ADR-0133]
related_artifacts:
  - microservices/messenger/PRD.md (§Competitive Benchmark)
  - /specs/hyperscaler-gates.json (HG-MESSENGER gate)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (messenger µservice)

## Purpose

Quantitative + qualitative parity comparison against industry-leading
team-chat + DM products. Drives `oya-governance-hyperscaler-maturity-claims`
gate per HG-MESSENGER (ADR-0123) and constrains what gtm-customer-success
can claim in tenant sales conversations. Re-validated bi-annually because
the chat landscape moves quickly (Slack-Salesforce, Teams release cadence,
Discord enterprise pivot, Matrix v2 federation).

## Competitor Set

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| Slack | Slack Channels + DM + Huddles + Workflow Builder | enterprise-grade SaaS; mature SDK ecosystem; Slack Connect cross-org | `api.slack.com` |
| Microsoft Teams | Teams Channels + Chat + Meetings + Files | M365 integration; eDiscovery + HIPAA + GCC-High | `learn.microsoft.com/microsoftteams` |
| Discord | Voice + text channels + threads + Stage | massive-scale (200M MAU); gaming-oriented; aggressive feature velocity | `discord.com/developers/docs` |
| Matrix / Element | Matrix protocol + Element client + Synapse server | OSS federation; E2E (Megolm + MLS WIP); decentralised | `spec.matrix.org` |
| Mattermost | OSS Slack-alike; self-hosted | data-sovereignty; GitLab integration; gov + finance fit | `docs.mattermost.com` |
| Zulip | OSS topic-threaded chat | topic-first model; async-friendly | `zulip.com/api/` |
| Rocket.Chat | OSS multi-channel chat | self-hosted; E2E DMs; LiveChat | `docs.rocket.chat` |
| Telegram (chats) | Telegram Cloud + Secret Chats | E2E via MTProto (Secret Chats only); 200k-member supergroups | `core.telegram.org` |
| Threema Work | Enterprise Threema | E2E by default; CH-residency | `threema.com/work` |
| Naver Works Chat | KR enterprise messenger | KR-first UX; KakaoTalk integration | `naver.worksmobile.com` |
| Line Works | JP/KR enterprise chat | LINE-style UX | `line.worksmobile.com` |

## Feature Parity Matrix

### Core messaging

| Capability | oyatie | Slack | Teams | Discord | Matrix | Mattermost | Zulip |
|---|---|---|---|---|---|---|---|
| Channels (public + private) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| DM + Group DM | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Threads | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ (topic) |
| Reactions | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| @mentions (Person/Team/Channel) | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Read receipts | ✅ | partial | ✅ | partial | partial | ✅ | partial |
| Search (full-text + faceted) | ✅ Meilisearch | ✅ | ✅ | partial | partial | ✅ | ✅ |
| Edit + delete + tombstone | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| Voice notes | ✅ (M03) | ✅ | ✅ | ✅ | partial | ❌ | ❌ |

### Voice + video

| Capability | oyatie | Slack | Teams | Discord | Matrix | Mattermost |
|---|---|---|---|---|---|---|
| 1:1 voice call | ✅ LiveKit | ✅ | ✅ | ✅ | ✅ (Element-Call) | partial |
| Group voice (huddles) | ✅ | ✅ Huddle | ✅ Meeting | ✅ Voice channel | ✅ | ❌ |
| Group video | ✅ | ✅ | ✅ | ✅ Stage | ✅ | partial |
| Screen-share | ✅ | ✅ | ✅ | ✅ | ✅ | partial |
| Background blur | M03 (LiveKit native) | ✅ | ✅ | ✅ | partial | ❌ |
| Recording | M03 + BAA gating | ✅ | ✅ | ❌ | partial | partial |
| MOS metrics (G.107) | ✅ first-class SLO | hidden | ✅ Call Quality Dashboard | hidden | partial | ❌ |

### Compliance + enterprise

| Capability | oyatie | Slack | Teams | Discord | Matrix | Mattermost |
|---|---|---|---|---|---|---|
| eDiscovery hold | ✅ | ✅ Enterprise Grid | ✅ | ❌ | partial | ✅ |
| Retention per regulatory pack | ✅ (11 packs) | tenant-level only | tenant-level only | ❌ | self-host responsibility | ✅ |
| HIPAA BAA | conditional (pack-us-hc) | Enterprise Grid only | GCC-High + BAA | ❌ | self-host | partial |
| KR PIPA + KISA | ✅ pack-kr | ❌ (region only) | ❌ | ❌ | self-host | ❌ |
| SEC 17a-4 + FINRA 4511 retention | ✅ pack-us-financial overlay | ✅ Enterprise Grid + Smarsh/Globanet | ✅ Purview | ❌ | self-host | partial |
| Dual-context (personal/professional) | ✅ data-model invariant | personal-account ≠ work-account (account-level) | personal-MS-Teams ≠ work (account-level) | ❌ | identity-level | ❌ |
| E2E DM | ✅ MLS (RFC 9420) M03 | ❌ | ❌ | ✅ Megolm; MLS WIP | ❌ | ❌ |
| Federation | optional Matrix-bridge (M04) | Slack Connect (cross-org) | Teams federation | ❌ | ✅ native | partial |
| Four-eyes admin disclosure | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Cedar / Rego / OPA policy | ✅ Cedar v4 | partial (admin-only) | partial | ❌ | partial | partial |

### Substrate

| Capability | oyatie | Slack | Teams | Discord | Matrix | Mattermost |
|---|---|---|---|---|---|---|
| Self-hosted (no vendor lock) | ✅ (Helm + Kustomize) | ❌ | ❌ | ❌ | ✅ Synapse | ✅ |
| Multi-region data-residency | ✅ 11 packs | partial (Enterprise Grid) | partial (Sovereign Cloud) | partial (DC regions only) | self-host responsibility | self-host |
| OpenSLO + agentic gate | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Ed25519 audit-chain | ✅ | ❌ (vendor logs) | ❌ (vendor logs) | ❌ | partial | partial |

## Quantitative Performance Parity

| Metric | oyatie target | Slack ref | Teams ref | Discord ref | Notes |
|---|---|---|---|---|---|
| Message-send p99 | ≤ 100ms | ~120ms (P99 published) | ~150ms | ~80ms | parity within reasonable range |
| WS fanout p99 | ≤ 100ms | ~150ms | ~200ms | ~50ms | Discord leads; oyatie targets parity |
| Presence propagation p99 | ≤ 200ms | ~500ms | ~1s | ~200ms | parity with Discord |
| Search p95 | ≤ 400ms (10M corpus) | ~500ms | ~600ms | n/a (limited search) | parity |
| Voice/video setup p95 | ≤ 1.5s | ~2s (Huddle) | ~2.5s (Meeting) | ~1s (channel) | within range |
| MOS (in-call) | ≥ 4.0 mean | ~4.1 | ~4.2 | ~4.3 | parity expected |
| Channel join 1k-member p95 | ≤ 300ms | n/a published | n/a published | ~200ms | parity with Discord |

## Key Parity Gaps to Close (oyatie → industry leader)

| # | Gap | Owner | Target close |
|---|---|---|---|
| 1 | E2E DM with MLS (matrix/element only competitor) | axis-messenger + council-privacy | M03 |
| 2 | Federation (Matrix-bridge or native ActivityPub) | axis-messenger | M04 |
| 3 | Mobile SDK polish (iOS/Android parity with Slack native) | axis-messenger + gtm | M02-onward1 |
| 4 | Background blur / face-AR (LiveKit add-on) | axis-messenger | M03 |
| 5 | Mature bot / app marketplace (Slack/Discord lead by years) | axis-messenger + gtm | M05-onward |
| 6 | Enterprise SSO depth (SCIM provisioning all 11 packs) | ops-security + gtm | M03 |
| 7 | Recording + transcription (call-recording compliance overlay) | axis-messenger | M03 |

## Key oyatie Differentiators (NOT in any competitor)

1. **Dual-context isolation by data-model invariant** — Personal ≠ Professional
   enforced at compile-time + LEAN-lane (per parallel ADR-0135); no competitor
   does this at data-model level.
2. **Multi-pack residency by design** — 11 region-pinned packs; no SaaS
   competitor matches the breadth (Slack/Teams region-coarse).
3. **OpenSLO-gated promotion** — feature rollouts gated by burn-rate (ADR-0130);
   no competitor enforces SLO-based rollout halting.
4. **Cedar v4 policy substrate** — fine-grained per-channel + per-message
   policy; competitors expose only admin-level RBAC.
5. **Cryptographic audit-chain** — Ed25519 + Merkle over every state transition;
   competitors deliver opaque vendor logs.
6. **Four-eyes admin disclosure** — two-principal approval for PII reads;
   no competitor enforces.
7. **Workflow + Ontology native integration** — first-class events typed
   into Workflow Studio; competitors expose webhooks only.

## Claim-Boundary Rules

Sales claims permitted (citation-bounded):
- ✅ "Dual-context personal/professional enforced as a data-model invariant
  is unique to oyatie" (true as of 2026-05-17; review bi-annually).
- ✅ "11-pack residency exceeds Slack Enterprise Grid + Teams Sovereign Cloud
  combined" (true; Slack EG has ~7 regions, Teams ~4 sovereign).
- ✅ "OpenSLO-gated feature rollout is unique to oyatie among production
  team-chat solutions" (review bi-annually).
- ✅ "Cedar v4 fine-grained policy substrate exceeds Slack admin RBAC depth"
  (true; Slack admin RBAC is coarse-grained).

Sales claims FORBIDDEN (per ADR-0123 hyperscaler-maturity-claim-gate):
- ❌ "oyatie messenger is faster than Discord" (no published benchmark;
  would be unsourced superiority).
- ❌ "oyatie has more features than Slack" (feature-count is unmeasurable +
  Slack has 10+ years head start on marketplace).
- ❌ "HIPAA-compliant out of the box" (conditional on BAA + pack-us-healthcare
  activation; do not claim universal).
- ❌ "Slack-compatible" (we accept Slack incoming-webhook URL shape only;
  full Slack-API parity not claimed; do not market as drop-in replacement).
- ❌ "More secure than Telegram Secret Chats" (Telegram MTProto + ours MLS
  are different threat-models; no published cryptanalysis comparison).

## Bi-Annual Refresh Process

| Step | Owner |
|---|---|
| 1. Survey competitor docs for changes (new features / pricing / claims) | gtm-customer-success |
| 2. Update this matrix; cite sources | axis-messenger |
| 3. Re-run quantitative benchmarks (load tests in staging cluster) | ops-sre-reliability |
| 4. Council-architecture review for claim-boundary rule updates | council-architecture |
| 5. Publish + notify sales/gtm | gtm-customer-success |

## References

- `microservices/messenger/PRD.md` §Competitive Benchmark.
- `/specs/hyperscaler-gates.json` HG-MESSENGER gate.
- ADR-0123 (hyperscaler-maturity-claim-gate).
- ADR-0135 (Connect dual-context, parallel).
- ADR-0130 (agentic SLO-gated promotion).
- ADR-0132 (suite-and-bundle dissolution).
- ADR-0133 (industry best-practice conformance).
- Competitor docs as cited inline above.
