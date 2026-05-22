---
doc_class: User-Journey-README
journey_id: j172-lev-kahn-investor-relations-shareholder-meeting-livestream
slice: investor-relations-AGM-livestream-12400-shareholders-real-time-Q-and-A-proxy-vote-Reg-FD-closed-captions-4-lang-SEC-17a-4-WORM
status: draft
date: 2026-05-20
authority_tier: 2
persona_primary: Lev Kahn (white/front-office; investor relations director)
audience_type: B2B_INVESTOR_RELATIONS + SHAREHOLDER_MEETING + SEC_REG_FD
microservice_count: 5
pack_overlay_anchor: SEC-Reg-FD + SEC-17a-4f-WORM + NYSE-Listed-Company-Manual + SOX + GDPR + EU-MAR + Delaware-Corporate-Law + IAS-1
related_adrs:
  - ADR-0243-cedar-as-universal-gate
  - ADR-0244-tenant-as-universal-scoping-primitive
  - ADR-0245-substrate-vs-product-layering
  - ADR-0251-compliance-pack-primitive
  - ADR-0252-hlc-default-truetime-tier
  - ADR-0253-http3-quic-default-protocol
  - ADR-0254-kubernetes-everywhere-pods-cloud-hypervisor
  - ADR-0263-observability-emission-contract
---

# j172 — IR Director Lev Kahn runs the Helios Industries AGM livestream for 12,400 shareholders worldwide under SEC Reg FD

## At a glance

Lev Kahn (Лев Кан in Russian Cyrillic; full name Lev Yevgenyevich Kahn) is a **47-year-old Director of Investor Relations** at **Helios Industries, Inc.** (NYSE:HLOS; mid-cap industrial conglomerate; HQ Chicago IL; ~14,800 employees across 9 countries; FY2026 revenue $4.82B; market cap $11.4B at AGM open). Lev is American-Russian (born Moscow 1980, emigrated to Brooklyn with his family in 1991, naturalized US citizen 2003), BA-Finance Columbia 2002, MBA-Booth 2008, IR Society of America certified (IRO-Sr 2018), joined Helios in 2019-04 from a senior IR role at Caterpillar. He reports to CFO Marguerite Vasquez-Ortiz (Vassar 1993, CPA, joined Helios 2022) and dotted-line to the Board's **Audit Committee chair** (INED Hideko Watanabe-Bell, ID-1962, retired CFO of Mitsui Trust UK).

It is **Thursday May 20, 2027, 04:48 CDT (Chicago)**. The Helios FY2026 **AGM (Annual General Meeting)** is set to open at **09:30 EDT / 14:30 BST / 15:30 CEST / 22:30 SGT / 23:30 KST**. Approximately **12,400 shareholders** have RSVPed for the livestream:

- Institutional shareholders: 224 firms (mostly US-based; ~62% of voting share)
- Retail shareholders: 12,176 individuals across 38 countries; ~12% of voting share via direct ownership
- Proxy advisors: ISS (Institutional Shareholder Services), Glass Lewis (registered as observers, not voters)
- Beneficial owners via brokers: ~2,800 (Schwab, Fidelity, Vanguard, IBKR street-name)
- Press: 14 outlets accredited (WSJ, Bloomberg, Reuters, FT, Nikkei, Caixin, Handelsblatt, Le Monde, Yonhap, La Stampa, Korea Economic Daily, Mainichi, Globe & Mail, AFR)

The agenda includes a **dividend declaration** (proposed $0.42/share quarterly; up from $0.38; record date June 3, payable June 17) + **EPS announcement** for FY2026 Q1 (close of April 30, 2027; preliminary EPS $1.84 vs Street consensus $1.78) + **director re-elections** (3 incumbent directors; 1 new director nominee Mrs. Adaeze Okonkwo-Henderson, retired GE EVP) + **auditor ratification** (Deloitte LLP, year 14 of engagement; rotation rule triggers FY2028) + **shareholder proposal #1** (climate disclosure expansion, sponsor: As You Sow filing on behalf of CalPERS-aligned holders, ~3% support pre-tally) + **shareholder proposal #2** (board declassification, sponsor: NYC Comptroller).

The journey covers Lev's **24 hours** (May 20 04:48 CDT — pre-meeting prep → May 20 18:18 CDT — post-meeting filings + transcript distribution) of:

1. **meet** µservice — the AGM livestream (4 simultaneous language streams: en-US closed-captioned + en-UK closed-captioned + zh-Hans (live human interpreter for Mainland investors) + ko-KR (live human interpreter for KR institutional holders); the 5th language ja-JP added at 08:42 CDT after Nikkei requests + Helios accommodates); real-time Q&A queue with proxy-vote integration; recording compliance per SEC 17a-4(f) WORM; closed-caption auto-generation + human verification
2. **governance** µservice — vote tally Cedar gate with Merkle attestation per share class (Class A common + Class B founder-shares); per-proposal tally streaming; rolling tally certification by Computershare (registrar) + dual sign by independent inspector of elections Carl Hagberg & Associates
3. **drive** µservice — pre-meeting + meeting-day artifact retention (proxy materials, slide deck, prepared remarks, Q&A transcript, recording); WORM-compliant per SEC 17a-4(f) (6-year retention with 1-year accessibility, indelible storage, time-stamped, audit-trail); also EU MAR Article 17 retention for non-US holders
4. **audit-chain** µservice — Merkle anchor per vote tally per share class; per-question Q&A audit record; recording chain-of-custody; SEC 17a-4(f) WORM attestation
5. **community** µservice — retail-investor question channel filtered by ombudsperson (separate from primary Q&A queue; ombuds review for compliance with Reg FD + civility filter); accepted questions promoted into the primary queue for IR + management response

Microservices: `meet`, `governance`, `drive`, `audit-chain`, `community`. Secondary: `identity` (shareholder authentication via beneficial-owner attestation + broker-issued control number + direct registered owner via Computershare ProxyView; SSO from Schwab/Fidelity/Vanguard street-name), `tenancy` (Helios + Computershare + Carl Hagberg + each shareholder's brokerage tenant), `messenger` (executive-only channel for board + management coordination), `notes` (Lev's working AGM playbook), `observability` (latency targets for global stream + caption verification), `intelligence` (real-time sentiment + question-clustering for IR coaching).

## Why this journey matters

Lev Kahn is **MASTER-ROSTER §3.6 row 264** — the canonical IR Director persona at a NYSE-listed mid-cap industrial company. This persona covers ~8,200 IRO-Sr-class roles globally (BLS 2024 code 13-2099 narrowed to "Investor Relations Director"). The AGM is the single most legally regulated event of the corporate year + requires Reg FD-compliant simultaneous disclosure; getting it wrong invokes SEC enforcement.

The journey closes:

- **Critical-path row 213** (Multi-language closed-caption livestream with real-time human interpreter integration — meet µservice's hero AGM-class capability)
- **Critical-path row 214** (Real-time Q&A queue with proxy-vote integration — shareholders who have not voted yet can vote during the meeting up to the close of polls)
- **Critical-path row 215** (SEC Reg FD-compliant simultaneous-disclosure gate — no material information released asymmetrically; Cedar permit blocks any one-language-only material)
- **Critical-path row 216** (Per-share-class Merkle-anchored vote tally with rolling certification by registrar + independent inspector)
- **Critical-path row 217** (SEC 17a-4(f) WORM recording compliance — 6-year retention, indelible, time-stamped, audit-trail)
- **Critical-path row 218** (Retail-investor community channel filtered by ombudsperson — civility + Reg FD compliance gate before promotion to primary Q&A queue)

Hyperscaler benchmark: traditional AGM platforms (Lumi Global + Broadridge VSM + Computershare Virtual Meeting) handle the basic livestream + vote but not the cross-µservice Cedar permit gate for Reg FD compliance + the per-share-class Merkle-anchored tally + the community-filtered retail question stream. Native multilanguage human-interpreter integration with byte-exact caption-verification is novel to oyatie's [[substrate-vs-product]] architecture.

## Artifact inventory

| Artifact | Purpose | Substance bar |
|---|---|---|
| `story.md` | Beat-by-beat May 20 04:48 CDT → 18:18 CDT across pre-meeting prep + 90-minute meeting + post-meeting filings | Chicago morning weather; ratchet through 09:30 EDT opening; multi-language interpreter handoffs; specific shareholder questions; specific vote tallies per share class |
| `ux-flow.md` | Lev's IR command console + livestream operator console + Q&A queue + vote-tally dashboard + community-filtered retail question stream + EPS + dividend declaration screen | Per-screen Cedar permit + Reg FD compliance indicator + per-language closed-caption status |
| `handshake.md` | Per-µservice API; livestream open + multi-language streams + Q&A queue management + vote tally streaming + Merkle anchor + SEC 17a-4(f) WORM write | Each row names share class + Cedar permit + audit class + Reg FD simultaneous-disclosure assertion |
| `integration-test-plan.md` | Livestream latency + closed-caption verification + Reg FD compliance fuzz + vote tally determinism + Merkle proof + WORM seal + community-filtered question handoff | Per-test seed + Reg FD invariant + share-class invariant + WORM seal invariant |
| `schemas/cedar-policy.cedar` | AGM Cedar policy | IR + CFO + GC + Board + Computershare + Carl Hagberg permits; Reg FD simultaneous-disclosure gate; per-share-class vote permits |
| `schemas/journey-messages.proto` | proto3 for all RPCs | en-US + en-UK + zh-Hans + ko-KR + ja-JP preservation; share class envelopes; vote tally streaming |
| `schemas/openapi-agm-livestream.json` | OpenAPI for AGM livestream endpoints | Multi-language stream + Q&A queue + vote tally + recording |
| `schemas/openapi-vote-tally.json` | OpenAPI for vote tally endpoints | Per-share-class + Merkle anchor + rolling certification + registrar + inspector dual-sign |
| `schemas/agm-state-machine.yaml` | 8-state AGM lifecycle | pre_meeting → notice_period → meeting_open → presentations → q_and_a → voting → tally_certification → post_meeting_filings → archive |

## The five primary microservices in scope

| µservice | Role | Critical-path row |
|---|---|---|
| `meet` | Multi-language livestream with closed captions + real-time Q&A queue + recording | row 213, 214 |
| `governance` | Per-share-class vote tally + Reg FD-compliant simultaneous-disclosure gate + decision recording | row 215, 216 |
| `drive` | SEC 17a-4(f) WORM retention of all artifacts | row 217 |
| `audit-chain` | Merkle anchor per tally per share class + recording chain-of-custody + SEC 17a-4(f) attestation | row 216, 217 |
| `community` | Retail-investor question channel filtered by ombudsperson before promotion to primary Q&A | row 218 |

## Secondary microservices touched

| µservice | Touch reason |
|---|---|
| `identity` | Shareholder authentication via beneficial-owner attestation + broker control number + Computershare ProxyView SSO; Lev's passkey + YubiKey 5C NFC; board members' passkeys; Computershare + Carl Hagberg principals |
| `tenancy` | Helios primary tenant `helios-industries-inc-nyse-hlos` + Computershare tenant `computershare-registrar-services` + Carl Hagberg tenant `carl-hagberg-inspectors-of-elections` + ~280 brokerage tenants for street-name holders |
| `messenger` | Executive-only board + management coordination channel; CFO + CEO + GC + IR + Board chair sub-channel; live during the meeting for off-stage coordination |
| `notes` | Lev's working AGM playbook (4-week-prep document, restricted to IR team) |
| `observability` | Per-region latency targets (NYC <80ms, London <120ms, Frankfurt <120ms, Singapore <180ms, Tokyo <170ms, Seoul <160ms); per-language caption verification |
| `intelligence` | Real-time sentiment analysis on Q&A questions; question-clustering for IR coaching; pre-meeting peer-company AGM analytics |
| `cell` | Tier-1 executive cell for Helios HQ + per-region tier-2 cells for regional shareholders + WORM cell for SEC 17a-4(f) compliance |

## Pack overlays (8 active)

| Pack | Activation reason | Pack ID |
|---|---|---|
| SEC-Reg-FD | US Reg FD (17 CFR § 243) — simultaneous disclosure of material info | `pack-sec-reg-fd-2024` |
| SEC-17a-4f-WORM | US SEC 17 CFR § 240.17a-4(f) — broker-dealer recordkeeping (Helios is voluntary alignment for IR records) | `pack-sec-17a-4f-worm-v3` |
| NYSE-Listed-Company-Manual | NYSE Listed Company Manual §§ 401-406 + 303A | `pack-nyse-listed-company-manual-2027` |
| SOX | SOX § 302 + 404 (Lev's certifications) | `pack-sox-302-404` |
| GDPR | EU shareholders + EEA shareholders + cross-border data flow | `pack-gdpr-eu-shareholders-v4` |
| EU-MAR | EU Market Abuse Regulation (596/2014) Article 17 inside information disclosure | `pack-eu-mar-article-17` |
| Delaware-Corporate-Law | Delaware General Corporation Law §§ 211, 212, 219, 224 (shareholder meetings + voting + record date + electronic records) | `pack-delaware-gcl-shareholder-meetings` |
| IAS-1 | IFRS IAS-1 (presentation of financial statements) for non-US accounting comparability | `pack-ias-1-fy2026` |

## Regulatory anchors

1. **SEC Regulation FD** — 17 CFR § 243.100 et seq. — simultaneous disclosure of material non-public information
2. **SEC Rule 17a-4(f)** — 17 CFR § 240.17a-4(f) — electronic recordkeeping with WORM properties (indelible, time-stamped, audit-trail)
3. **NYSE Listed Company Manual** — §§ 401-406 (annual meeting requirements) + 303A (independent directors + audit committee)
4. **SOX § 302 + § 404** — CEO/CFO certifications + internal control reporting
5. **GDPR Articles 12-22** — shareholders' data subject rights (cross-border data flow during AGM)
6. **EU Market Abuse Regulation (596/2014) Article 17** — inside information disclosure (parallel to Reg FD)
7. **Delaware GCL §§ 211, 212, 219, 224** — shareholder meetings + voting + record date + electronic records
8. **NYSE Listed Company Manual § 312.04** — equity comp shareholder approval
9. **IRS Section 162(m)** — performance-based compensation (proxy disclosure tie-in)
10. **ADR-0243 + ADR-0244 + ADR-0245 + ADR-0251 + ADR-0252 + ADR-0253 + ADR-0254 + ADR-0263**

## Cell + region matrix

| Cell | Role | Journey use |
|---|---|---|
| `us-chicago-tier-1-executive-helios-hq` | Helios IR command primary cell | Lev's command console |
| `us-east-tier-1-livestream-broadcast` | Primary livestream broadcast cell | Meeting livestream origin |
| `us-east-tier-1-worm-sec-17a-4f` | SEC-aligned WORM cell | Meeting recording + slides + transcripts |
| `eu-frankfurt-tier-2-region-edge` | EU shareholder region edge | EU livestream + EU-MAR retention |
| `apac-singapore-tier-2-region-edge` | APAC shareholder region edge | APAC livestream |
| `apac-tokyo-tier-2-region-edge` | JP shareholder region edge | JP livestream + Nikkei accommodation |
| `apac-seoul-tier-2-region-edge` | KR shareholder region edge | KR livestream + KR interpreter |
| `external-transparency-log-batch-2027-05-20` | External transparency log | Per-tally + per-anchor batched |

## Cedar permits (excerpt — full text in `schemas/cedar-policy.cedar`)

```cedar
permit (
    principal == User::"lev.kahn@helios-industries-inc-nyse-hlos",
    action in [
        Action::"meet.agm_livestream_open",
        Action::"meet.agm_livestream_close",
        Action::"meet.multi_language_caption_enable",
        Action::"meet.q_and_a_queue_manage",
        Action::"governance.agm_state_transition",
        Action::"governance.vote_tally_certify_request",
        Action::"drive.sec_17a_4f_worm_write",
        Action::"audit_chain.agm_anchor_emit",
        Action::"community.retail_question_handoff_receive"
    ],
    resource is AGMSession
) when {
    principal.role_in_tenant("helios-industries-inc-nyse-hlos") == "investor_relations_director" &&
    resource.tenant_id == "helios-industries-inc-nyse-hlos" &&
    context.passkey_assertion_present == true &&
    context.yubikey_attestation_present == true &&
    context.cfo_co_sign_or_delegate_present == true
};

forbid (
    principal,
    action in [
        Action::"meet.multi_language_caption_disable_for_subset",
        Action::"meet.material_info_disclose_single_language_only",
        Action::"governance.tally_disclose_partial_share_class"
    ],
    resource is AGMSession
) when {
    resource.reg_fd_simultaneous_disclosure_required == true
};

permit (
    principal in [
        ServicePrincipal::"computershare-registrar-tally-service",
        ServicePrincipal::"carl-hagberg-inspector-of-elections"
    ],
    action == Action::"governance.vote_tally_certify",
    resource is VoteTally
) when {
    resource.share_class in ["common_a", "common_b_founder"] &&
    resource.rolling_certification_state == "pending" &&
    context.dual_sign_required == true &&
    context.merkle_anchor_emitted == true
};
```

## Acceptance summary

| AC | Result expected |
|---|---|
| AC-J172-001 | AGM livestream opens 09:30 EDT with 4 simultaneous language streams (en-US + en-UK + zh-Hans + ko-KR); 5th (ja-JP) added 08:42 CDT after Nikkei request; audit `EVT-J172-LIVESTREAM-OPENED-001` |
| AC-J172-002 | 12,400 shareholders authenticated via beneficial-owner attestation + broker control numbers + ProxyView SSO; audit `EVT-J172-SHAREHOLDERS-AUTHENTICATED-002` |
| AC-J172-003 | Closed captions auto-generated + human-verified in 5 languages; per-language WER < 5% verified; audit `EVT-J172-CAPTIONS-VERIFIED-003` |
| AC-J172-004 | Real-time Q&A queue: 187 questions submitted across 90 minutes; 32 answered live; community-filtered retail stream: 88 retail questions filtered by ombudsperson, 14 promoted to primary queue; audit `EVT-J172-Q-AND-A-ROLLUP-004` |
| AC-J172-005 | Vote tally per share class: Class A common 87% in favor of dividend declaration ($0.42/share) + 91% in favor of director re-elections + 94% auditor ratification + 18.4% supported climate disclosure shareholder proposal + 23.7% supported board declassification; Class B founder-shares 100% with management; rolling certification by Computershare + Carl Hagberg with dual-sign; audit `EVT-J172-VOTE-TALLY-005` |
| AC-J172-006 | Reg FD simultaneous-disclosure gate held; EPS preliminary figure released simultaneously in all 5 language streams; press release transmitted via WireService + Helios IR page at identical UTC timestamp; audit `EVT-J172-REG-FD-SIMULTANEOUS-DISCLOSURE-006` |
| AC-J172-007 | Merkle anchor emitted per share class per proposal (2 share classes × 6 proposals = 12 anchors); external transparency log batched; audit `EVT-J172-MERKLE-ANCHORS-007` |
| AC-J172-008 | SEC 17a-4(f) WORM recording sealed: 90-minute meeting recording + slide deck + prepared remarks + Q&A transcript + vote tally records; 6-year retention; indelible storage; time-stamped; audit-trail; audit `EVT-J172-SEC-17A-4F-WORM-SEALED-008` |
| AC-J172-009 | Cedar deny coverage: 18 enumeration attempts on board-only channel from non-board principals all denied; 4 attempts to disable a single language caption stream denied; 2 attempts to disclose material info pre-EPS-release denied; audit `EVT-J172-CEDAR-DENY-COVERAGE-009` |
| AC-J172-010 | Per-region latency targets met: NYC < 80ms (achieved 62ms), London < 120ms (98ms), Frankfurt < 120ms (104ms), Singapore < 180ms (148ms), Tokyo < 170ms (142ms), Seoul < 160ms (138ms); audit `EVT-J172-LATENCY-TARGETS-MET-010` |
| AC-J172-011 | Pack manifest assertion: 8 packs active + cross-validated; audit `EVT-J172-PACK-MANIFEST-011` |
| AC-J172-012 | en-US + en-UK + zh-Hans + ko-KR + ja-JP + Russian (Lev's working note language) + Hebrew (CFO note language) + diacritic preservation byte-exact |

## Cross-references

- Persona dossier: `docs/personas/ir-director-lev-kahn.md`
- MASTER-ROSTER §3.6 row 264
- Matrix §10 j172 recommendation
- Related: j163 (cross-time-zone board meeting), j165 (CCO board quarterly compliance report), j168 (COO quarterly ops review), j120 (tenant treasury multi-currency FX hedge)
- Pack roster: `packs/sec-reg-fd-2024/`, `packs/sec-17a-4f-worm-v3/`, `packs/nyse-listed-company-manual-2027/`, `packs/sox-302-404/`, `packs/gdpr-eu-shareholders-v4/`, `packs/eu-mar-article-17/`, `packs/delaware-gcl-shareholder-meetings/`, `packs/ias-1-fy2026/`
- ADRs as listed above

## Stop condition

Journey complete when all 12 AC pass on the seeded fixture, the livestream closes at 11:00 EDT with no Reg FD violations, all 12 Merkle anchors are emitted + externally batched, the SEC 17a-4(f) WORM recording is sealed with audit-trail intact, the community-filtered retail question pipeline is closed with 88 questions reviewed + 14 promoted, the dividend declaration is filed with SEC via Form 8-K within 4 business days, and the AGM transcript is published on Helios IR page in 5 languages byte-exact.
