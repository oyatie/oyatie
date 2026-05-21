---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j169-cmo-felix-ng-multi-country-launch-with-locale-pack
date: 2026-05-20
authority_tier: 2
status: draft
---

# j169 — Integration test plan

## §0 — Fixtures

| Fixture | Description |
|---|---|
| `veritem-primary-tenant.json` | Veritem primary + 6 country sub-tenants + 14 named principals |
| `6-cells-asean.json` | 6 cells with per-country residency + ISO/PDPA/local-regulator attestations |
| `12-ambassadors.json` | 12 ambassador identities with per-country credentials, follower counts, NDAB |
| `localization-corpus.json` | 4 surfaces × 7 languages × ~600 strings = ~16,800 strings with NLLB-200 raw + human-edited final + attestation |
| `6-payment-processors-mock.json` | Mock adapters for GrabPay/GoPay/TrueMoney/MoMo/GCash/TouchnGo + Stripe fallback |
| `cedar-policy-bundle-j169.cedar` | Per-action Cedar bundle |
| `mock-truetime-driver.ts` | TrueTime fence mock (default 2.4 ms) |
| `mock-nllb-200-driver.ts` | NLLB-200 translation mock with deterministic outputs |
| `mock-research-ethics-reviewer.ts` | Per-country research-ethics reviewer mock |
| `6-locale-pack-overlays.json` | The 6 ASEAN locale-pack overlays |

## §1 — Readiness tests

### TEST-J169-001 — Readiness dashboard renders 522/522 green

**Action**: Felix GETs readiness as `cmo`.

**Expected**: HTTP 200; `checklist_green==87` × 6 countries; audit seal `EVT-J169-READINESS-COMPLETE-001`.

### TEST-J169-002 — Cedar denies if reader not CMO/CEO/MD/Compliance

**Action**: An unrelated marketing-junior principal attempts GET.

**Expected**: HTTP 403 `principal_role_not_in_allowed_set`.

## §2 — Locale-pack auto-activation tests

### TEST-J169-010 — Locale-pack auto-activates for new subscriber

**Setup**: Subscriber registers with `country_residency_id` attribute set.

**Action**: Watch pack activation.

**Expected**: ID-PP-71/2019 + UU-PDP-27/2022 + ASEAN-Privacy-Framework + RFC-5646(id-ID) packs all activate within 200 ms of subscriber-tenant creation. No manual toggle required.

### TEST-J169-011 — Sample 100 subscribers per country: pack auto-activation 100% success

**Setup**: 600 new subscriber tenants (100 per country).

**Action**: Inspect pack activation chain.

**Expected**: 100% of 600 subscribers have the correct country-specific packs activated. Audit `EVT-J169-LOCALE-PACK-AUTO-ACTIVATION-012`.

## §3 — Content localization + AI transparency tests

### TEST-J169-020 — NLLB-200 batch localizes 7 languages with attestation

**Setup**: 600-string batch in en-SG.

**Action**: POST localizations/batch.

**Expected**:
- Each string returns 6 target-language translations + 1 source.
- Each translation includes `ai_source`, `ai_raw`, `human_editor` (if edited), `transparency_attestation` (one of {ai_translated_then_human_edited, ai_translated_human_reviewed_no_edit, human_authored_only}).
- Cultural-overlay applies where relevant.
- Audit seal per string + aggregate batch seal.

### TEST-J169-021 — Cedar forbids publish of AI-translated string without transparency disclosure

**Setup**: One translated string with `ai_content_transparency_disclosure_present==false`.

**Action**: Attempt to publish.

**Expected**: HTTP 403 from Cedar `forbid` rule. String quarantined.

### TEST-J169-022 — Cultural-adaptation overlay flags amber for sensitive content

**Setup**: Indonesian Lebaran-reset nudge content.

**Action**: Run cultural-adaptation overlay.

**Expected**: Overlay returns `amber` with reason `lebaran-religious-context-requires-local-ambassador-review`. Workflow routes to local-ambassador-panel review.

## §4 — Ambassador onboarding tests

### TEST-J169-030 — All 12 ambassadors credentialed within 5 days

**Setup**: 12 named ambassadors.

**Action**: POST credential-issue × 12.

**Expected**: 12 credentials issued; all 12 passkey-enrolled; attribution-tracking URLs generated; audit seal per ambassador.

### TEST-J169-031 — Ambassador attribution URL captures signups correctly

**Setup**: Ambassador URL `https://veritem.id/signup?ambassador=tania-putri-001`.

**Action**: 100 mock signups via that URL.

**Expected**: All 100 attributed to Tania Putri in analytics; commission accrual correct.

## §5 — Cohort split tests

### TEST-J169-040 — 18 cohort rule-bundles signed with research-ethics approval

**Setup**: 18 rule-bundles drafted.

**Action**: PUT each with MD + CMO + Compliance + research-ethics-reviewer signatures.

**Expected**: All 18 sealed. Audit `EVT-J169-COHORT-SPLITS-SIGNED-004`.

### TEST-J169-041 — Cohort split percentages enforce within ±0.5%

**Setup**: 10,000 mock signups in Indonesia.

**Action**: Inspect cohort distribution.

**Expected**: Each cohort ≈ 33.33% ± 0.5%.

## §6 — Go/no-go vote tests

### TEST-J169-050 — 8-of-8 PERMIT seals go-live

**Setup**: All preconditions green.

**Action**: 8 quorum members vote PERMIT.

**Expected**: After 8th vote: `quorum_decision: PERMIT`. Audit `EVT-J169-GO-LIVE-PERMIT-005`.

### TEST-J169-051 — Single DENY blocks launch

**Setup**: One MD votes DENY with rationale (e.g., "Last-minute local-regulator audit issue").

**Expected**: Launch blocked. Audit `EVT-J169-GO-LIVE-DENIED-005b`. Notification to all members.

## §7 — Per-country launch flip tests

### TEST-J169-060 — SG launch flip at exactly 00:00 UTC

**Setup**: SG country flag scheduled at `2026-06-15T00:00:00Z`.

**Action**: Watch clock.

**Expected**: Flag flips within 50 ms of scheduled UTC time. First signup arrives at 00:00:18 UTC. Audit `EVT-J169-LAUNCH-LIVE-SG-006a`.

### TEST-J169-061 — Indonesia launch flip 1 hour later at 01:00 UTC

**Setup**: ID country flag scheduled at `2026-06-15T01:00:00Z` (= 08:00 WIB).

**Expected**: Flag flips at 01:00:00 ± 50 ms UTC. Audit `EVT-J169-LAUNCH-LIVE-ID-006d`.

### TEST-J169-062 — Country boundary isolation — SG signups don't leak to ID cohort

**Setup**: SG subscriber registers at 00:30 UTC.

**Action**: Confirm cohort assignment.

**Expected**: SG cohort split rules apply (not ID). Audit chain seals in SG sub-tenant only.

## §8 — Payment processor tests

### TEST-J169-070 — All 6 processors + Stripe fallback initialize

**Action**: Initialize each.

**Expected**: All 7 return `active`. Audit per processor.

### TEST-J169-071 — Per-country currency precision

**Setup**: IDR uses 0 decimals; SGD uses 2.

**Action**: Issue invoice in each currency.

**Expected**: IDR 49000 (no decimals); SGD 24.99 (2 decimals); rounding rules per ISO-4217.

## §9 — Day-7 analytics tests

### TEST-J169-080 — Day-7 signup tally accurate

**Setup**: Mock 71,400 signups across 7 days.

**Action**: GET day-7-report.

**Expected**: Per-country counts match seed; total = 71,400; beat-pct = 11.6%.

### TEST-J169-081 — Ambassador attribution percentage correct

**Expected**: 38.4% of signups attributed to ambassadors per the seed data.

### TEST-J169-082 — Cohort winner identification

**Expected**: For each country, the cohort with the highest activation-conversion-rate is identified. Significance test (p ≤ 0.05) included.

## §10 — Compliance attestation tests

### TEST-J169-090 — 6 country PDPA attestations + ASEAN + EU-AI-Act-Art-50 sealed

**Setup**: Day-7 complete.

**Action**: POST cross-border-transfer attestation.

**Expected**: All 8 attestation packs (6 country + ASEAN + EU-AI-Act-Art-50) sealed. Audit `EVT-J169-COMPLIANCE-ATTESTATIONS-011`.

### TEST-J169-091 — DEKRA acknowledgement workflow

**Expected**: DEKRA Singapore confirms receipt within 14 business days (mocked).

## §11 — Diacritic + script fidelity tests

### TEST-J169-100 — Thai script preserves UTF-8 NFC

**Setup**: 1,000 Thai strings.

**Action**: Round-trip through audit-chain + render.

**Expected**: Byte-identical UTF-8 NFC; tone marks + vowels + diacritics preserved.

### TEST-J169-101 — Vietnamese tone marks preserved

**Setup**: 1,000 Vietnamese strings including names (Trần Thị Mỹ Linh, Châu Mai Thi, Nguyễn Văn Quang).

**Expected**: All tone marks preserved.

### TEST-J169-102 — Traditional Chinese (zh-Hant-SG) rendering

**Setup**: Subset for Singapore Chinese-speaking-elderly cohort.

**Expected**: Traditional characters (餐單 not 餐单); Singapore-Mandarin lexicon preferences (e.g., 烤面包 → 烤土司 acceptable per local norms).

## §12 — Acceptance criteria coverage

| AC | Tests |
|---|---|
| AC-J169-001 | TEST-J169-001 |
| AC-J169-002 | TEST-J169-020 + TEST-J169-021 + TEST-J169-022 |
| AC-J169-003 | TEST-J169-030 + TEST-J169-031 |
| AC-J169-004 | TEST-J169-040 + TEST-J169-041 |
| AC-J169-005 | TEST-J169-050 + TEST-J169-051 |
| AC-J169-006 | TEST-J169-060 + TEST-J169-061 + TEST-J169-062 |
| AC-J169-007 | TEST-J169-070 + TEST-J169-071 |
| AC-J169-008 | TEST-J169-080 |
| AC-J169-009 | TEST-J169-081 |
| AC-J169-010 | TEST-J169-082 |
| AC-J169-011 | TEST-J169-090 + TEST-J169-091 |
| AC-J169-012 | TEST-J169-010 + TEST-J169-011 |
| AC-J169-013 | TEST-J169-100 + TEST-J169-101 + TEST-J169-102 |
| AC-J169-014 | TEST-J169-021 |
| AC-J169-015 | TEST-J169-090 |

## §13 — Pass/fail thresholds

- All TEST-J169-* pass.
- Cedar p99 ≤ 5 ms.
- Audit-chain dual-seal p99 ≤ 10 ms.
- TrueTime uncertainty ≤ 10 ms.
- Locale-pack auto-activation 100% across 600 sampled subscribers.
- Day-7 signups ≥ 64,000.
- Ambassador attribution ≥ 30%.
- 0 launch-blocking SEV-1/SEV-2.
- All 8 compliance attestation packs sealed.
