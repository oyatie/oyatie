---
doc_class: User-Journey-Integration-Test-Plan
journey_id: j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief
date: 2026-05-20
authority_tier: 2
status: draft
---

# j168 — Integration test plan

## §0 — Fixtures

| Fixture | Description |
|---|---|
| `aurelia-robotics-tenant.json` | Aurelia tenant + 13 named principals (Akira, Hiroshi, Watabe, Tanaka, Ito, Diego, Yamilet, Sofía, Brian, Hugo, Patricia, Patrick, María José) |
| `cell-topology-9-cells.json` | 9 cells × 3 AZs each = 27 AZs; ISO-27001/SOC2/region-pack attestations |
| `japanese-customer-tenants.json` | Komatsu, Sumitomo Heavy Industries, Mitsubishi Logistics tenants with GMO GlobalSign EVCS QES roots |
| `sev2-replay-fixture-2026-04-15.json` | 47-event timeline of the SEV-2 cell-failover cascade |
| `cedar-policy-bundle-j168.cedar` | Per-action Cedar bundle for ops + debrief + capex |
| `okr-q1-2027-capex-cra.pdf` | Pre-signed CRA document by Akira (QES sat-mx-FIEL) |
| `mock-truetime-driver.ts` | TrueTime fence mock (default uncertainty 2.4 ms) |
| `mock-japanese-locale-input.ts` | Japanese ja-JP IME mock for kanji + furigana entry |

## §1 — Q4 metric snapshot tests

### TEST-J168-001 — Snapshot renders 9 cells × 27 metrics = 243 cells

**Setup**: Load tenant + cell-topology + seed metrics.

**Action**: `GET /v1/quarters/Q4-2026/snapshot` as Akira.

**Expected**:
- HTTP 200
- Response includes 9 cell objects, each with 27-field `metrics` block.
- Audit seal `EVT-J168-Q4-METRIC-SNAPSHOT-001` written.
- Merkle root computed over all 243 cells.
- Cedar policy decision latency ≤ 5 ms.

### TEST-J168-002 — Read-only seal — snapshot cannot be modified once sealed

**Setup**: Snapshot sealed.

**Action**: Attempt `PATCH /v1/quarters/Q4-2026/snapshot` with synthetic edit.

**Expected**: HTTP 409 Conflict. Error: `snapshot_sealed_read_only`. No audit modification.

### TEST-J168-003 — Locale toggle preserves audit substance

**Setup**: Snapshot sealed in ja-JP locale.

**Action**: Re-render the snapshot in es-MX locale.

**Expected**:
- UI strings render in Spanish.
- Underlying audit-seal hash + Merkle root are byte-identical to ja-JP rendering.
- Principal names preserve UTF-8 NFC (渡辺 明 stays 渡辺 明).

## §2 — Incident debrief tests

### TEST-J168-010 — Debrief opens with Cedar permit + pre-populated timeline

**Setup**: SEV-2 incident closed 2026-04-15T05:48Z.

**Action**: Akira POSTs debrief-open at 2026-05-14T00:00Z (= 2026-05-14T09:00 JST).

**Expected**:
- HTTP 200
- `debrief_id` issued.
- `incident_timeline_record_count == 47`.
- Audit seal `EVT-J168-DEBRIEF-OPEN-002`.
- Cedar policy `incident.debrief_open` allowed (coo + incident-closed).

### TEST-J168-011 — 5-Whys completion seals root cause

**Setup**: Debrief open.

**Actions**: POST 5 five-whys-step calls, one per Why level.

**Expected**:
- After 5th step: `five_whys_complete: true`.
- Root cause + secondary root cause attached.
- Audit seal `EVT-J168-ROOT-CAUSE-IDENTIFIED-003`.

### TEST-J168-012 — Cedar denies debrief open if incident not closed

**Setup**: SEV-2 incident still in `monitoring` state.

**Action**: Akira attempts debrief open.

**Expected**: HTTP 403 with reason `incident_not_closed`.

## §3 — Corrective-action tests

### TEST-J168-020 — Bulk materialize 87 corrective actions

**Setup**: 5-Whys complete.

**Action**: POST corrective-actions/bulk with 87 items.

**Expected**:
- All 87 items materialized in `tasks` µservice with state `draft_pending_capex_approval`.
- Total engineering-hours = 3,840.
- Total capex estimate MXN 12M.
- Audit seal `EVT-J168-CORRECTIVE-ACTIONS-DRAFTED-003a`.

### TEST-J168-021 — Action items transition to `funded` after capex Cedar approval

**Setup**: Bulk materialized; capex approval pending.

**Action**: Capex Cedar permit completes.

**Expected**: All 87 items auto-transition to `funded` state. Audit seal `EVT-J168-CORRECTIVE-ACTIONS-FUNDED-007b`.

## §4 — Customer-relationship-repair tests

### TEST-J168-030 — 3 customer attestations dual-seal in source + target tenants

**Setup**: Debrief complete. Customer meetings scheduled.

**Actions**: POST cross-tenant-attestations to Komatsu, Sumitomo Heavy Industries, Mitsubishi Logistics.

**Expected**:
- Each attestation dual-seals in source + target tenant.
- Audit seals: `EVT-J168-CUSTOMER-REPAIR-004a / 004b / 004c`.
- Customer-side signer signs with GMO GlobalSign EVCS QES.
- TrueTime uncertainty ≤ 10 ms for each.

### TEST-J168-031 — Cedar denies attestation send if MLS not active

**Setup**: Mock MLS-encryption disabled.

**Action**: POST attestation.

**Expected**: HTTP 403 `cedar_policy_denied: mls_encryption_active_false`.

### TEST-J168-032 — Locale + diacritic preservation in customer attestations

**Setup**: Attestation drafted in ja-JP.

**Action**: Render at customer-side.

**Expected**: All Japanese kanji preserve UTF-8 NFC; honorifics (-san) render correctly; service-credit amounts in MXN with comma-separator per ja-JP locale (MXN 84,000).

## §5 — Merkle attestation tests

### TEST-J168-040 — Joint QES Merkle attestation by Akira + Hiroshi

**Setup**: Debrief + corrective-actions + customer-repair all sealed.

**Action**: POST merkle-attestation with two-signer body.

**Expected**:
- Merkle root computed over scope events.
- Audit seal `EVT-J168-MERKLE-ATTESTED-005`.
- Both signers' QES providers (sat-mx-FIEL + gmo-globalsign-evcs) attest.
- Dual-seal in Aurelia tenant + governance substrate.

## §6 — Capex Cedar quorum tests

### TEST-J168-050 — 5-of-5 PERMIT seals line item > MXN 5M

**Setup**: CRA signed. Capex Cedar window opens at 2026-05-18T09:00 CDT.

**Actions**: 5 quorum members vote PERMIT sequentially.

**Expected**:
- Each vote increments quorum count 1 → 5.
- After 5th: `quorum_decision: PERMIT`. Audit seal `EVT-J168-CAPEX-LINE-1-PERMIT-007a`.
- TrueTime uncertainty ≤ 10 ms.

### TEST-J168-051 — Single DENY blocks capex line

**Setup**: Same. Patricia (CFO) votes DENY with rationale `"Loaded rate calculation overstates by 8%, want re-estimate"`.

**Expected**: `quorum_decision: DENY`. Capex line moves to `revised_required` state. Notification fires.

### TEST-J168-052 — Cedar denies vote outside business-hours-CDT

**Setup**: Mock clock to 22:42 CDT.

**Action**: Diego attempts vote.

**Expected**: HTTP 403 `business_hours_cdt_false`. Vote deferred.

### TEST-J168-053 — Bulk vote allowed for line items ≤ MXN 5M

**Setup**: 8 line items totaling MXN 42M, each individually ≤ MXN 5M.

**Action**: Bulk vote payload from 5 quorum members.

**Expected**: Single 5-of-5 PERMIT seals all 8 items. Audit seal `EVT-J168-CAPEX-BULK-PERMIT-007z`.

### TEST-J168-054 — Capex line links to source incident

**Setup**: Capex line 1 approved.

**Action**: Auto-trigger `link_capex_to_incident` job.

**Expected**: Capex line 1's `linked_incident` field references the SEV-2 incident. Audit seal `EVT-J168-CAPEX-LINKED-008`.

## §7 — Quarterly report submission tests

### TEST-J168-060 — Q4 report submitted to 3 auditors with correct evidence packs

**Setup**: All capex approvals sealed.

**Action**: POST quarterly-reports/Q4-2026/submit.

**Expected**:
- 3 submissions written:
  - PwC México with SOC2-CC7.3 + ISO-22301 + ITIL-v4-IM + ISO-27035 packs
  - KPMG México with IFRS-15-service-credit-deduction pack
  - DEKRA with EU-AI-Act-Art-19-post-market-monitoring pack
- Audit seal `EVT-J168-REPORT-SUBMITTED-009`.

### TEST-J168-061 — IFRS-15 service-credit deduction journal entry

**Setup**: Service credits MXN 312k issued during SEV-2 quarter (Q1-2026).

**Action**: Auto-generate IFRS-15 journal entry.

**Expected**: Journal entry deducts MXN 312k from Q1-2026 revenue. KPMG audit-trace clean. Audit seal `EVT-J168-IFRS-15-CREDIT-DEDUCTION-010`.

## §8 — NPS + on-call burnout tests

### TEST-J168-070 — APAC-Tokyo NPS recovery curve

**Setup**: Baseline pre-SEV-2 NPS = 71. Post-SEV-2 NPS = 41.

**Action**: Re-measure post-debrief at 2026-05-15.

**Expected**: NPS recovers to 62 (above the 60-target floor). Audit seal `EVT-J168-NPS-RECOVERY-011a`.

### TEST-J168-071 — On-call burnout drops with workload-redistribution

**Setup**: APAC-Tokyo on-call burnout during incident quarter = 6.8/10.

**Action**: Q1-2027 OKR includes redistributing on-call rotation (more engineers, fewer pages per person).

**Expected**: Projected Q1-2027 burnout ≤ 4.0/10 (target). Audit seal `EVT-J168-ON-CALL-BURNOUT-PROJECTION-011b`.

## §9 — Time-zone correctness tests

### TEST-J168-080 — Dual UTC + IANA-zoned timestamp on every audit seal

**Setup**: Any audit event.

**Action**: Inspect audit record.

**Expected**:
- `seal_ts_utc` field present.
- `seal_ts_local` field with explicit IANA zone (e.g., `Asia/Tokyo`).
- Spread between zones matches expected offset.
- DST transitions handled correctly (e.g., America/Mexico_City has no DST, but Asia/Tokyo never has DST, so no edge cases here; America/Chicago for Austin does have DST).

### TEST-J168-081 — Cross-time-zone vote ordering preserved

**Setup**: Akira votes 09:18:42 CDT; Patrick Reilly votes 08:18:42 PDT (= 11:18:42 EDT = 16:18:42 UTC).

**Action**: Audit chain ordering.

**Expected**: Votes ordered by UTC monotonic; HLC tracks happens-before; no out-of-order seals.

## §10 — Cross-tenant audit-dual-seal invariant tests

### TEST-J168-090 — Every event dual-seals correctly

**Setup**: Complete journey.

**Action**: Query both Aurelia tenant + governance substrate tenant for the journey's audit seals.

**Expected**: Byte-identical seals + Merkle roots in both tenants for all key events.

## §11 — Acceptance criteria coverage

| AC | Tests |
|---|---|
| AC-J168-001 | TEST-J168-001 + TEST-J168-002 + TEST-J168-003 |
| AC-J168-002 | TEST-J168-010 + TEST-J168-012 |
| AC-J168-003 | TEST-J168-011 + TEST-J168-020 |
| AC-J168-004 | TEST-J168-030 + TEST-J168-031 + TEST-J168-032 |
| AC-J168-005 | TEST-J168-040 |
| AC-J168-006 | (CRA signing — captured in fixture validation) |
| AC-J168-007 | TEST-J168-050 + TEST-J168-051 + TEST-J168-052 + TEST-J168-053 |
| AC-J168-008 | TEST-J168-054 |
| AC-J168-009 | TEST-J168-060 |
| AC-J168-010 | TEST-J168-061 |
| AC-J168-011 | TEST-J168-070 + TEST-J168-071 |
| AC-J168-012 | TEST-J168-090 |
| AC-J168-013 | TEST-J168-003 + TEST-J168-032 |
| AC-J168-014 | TEST-J168-080 + TEST-J168-081 |

## §12 — Pass/fail thresholds

- All TEST-J168-* pass.
- Cedar policy decision p99 ≤ 5 ms.
- Audit-chain dual-seal p99 ≤ 10 ms.
- TrueTime uncertainty ≤ 10 ms at every gate.
- IFRS-15 service-credit deduction reconciles to KPMG with no variance.
- EU-AI-Act-Art-19 post-market monitoring report acknowledged by DEKRA within 14 business days.
- NPS post-debrief ≥ 60.
- On-call burnout projection ≤ 4.0/10 for Q1-2027.
- 0 SEV-1 incidents during the journey.
- Customer-relationship-repair attestations: 3-of-3 customers sign.
