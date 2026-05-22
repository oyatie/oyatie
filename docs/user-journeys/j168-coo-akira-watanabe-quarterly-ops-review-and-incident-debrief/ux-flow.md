---
doc_class: User-Journey-UX-Flow
journey_id: j168-coo-akira-watanabe-quarterly-ops-review-and-incident-debrief
date: 2026-05-20
authority_tier: 2
status: draft
---

# j168 — UX flow: ops dashboard + 5-Whys form + capex Cedar modal

## §0 — Devices in scope

| Person | Primary device | Locale | OS |
|---|---|---|---|
| Akira Watanabe (COO) | MacBook Pro M4 16" (Space Black, 64 GB RAM) | ja-JP primary, es-MX + en-US secondary | macOS 15.4 Sequoia; Safari 18.2 |
| Akira mobile | iPhone 15 Pro Max | ja-JP / es-MX dual | iOS 18.4 |
| Hiroshi Takei | ThinkPad X1 Carbon Gen 13 | ja-JP | Ubuntu 24.04 + GNOME |
| Watabe Toshio | MacBook Air M3 13" | ja-JP | macOS 15.4 |
| Kazumi Tanaka | ThinkPad T14s | ja-JP / en-US dual | Ubuntu 24.04 |
| Diego Vargas (dial-in) | MacBook Pro M4 14" | es-MX | macOS 15.4 |
| Hugo Ávila (CEO) | iMac M4 24" + iPad Pro M4 13" | es-MX | macOS 15.4 |
| Patricia Carrillo (CFO) | Surface Laptop 7 | es-MX / en-US | Windows 11 23H2 |
| Patrick Reilly (Board ops chair) | iPad Pro M4 11" (Webex from Tahoe) | en-US | iPadOS 18.4 |

## §1 — Q4-2026 quarterly ops dashboard (Akira's MacBook, Wed May 13, 09:00 JST)

### Screen: `https://ops.aurelia-robotics-internacional-sa-de-cv-mx.oyatie.cloud/quarters/Q4-2026/snapshot`

**Layout** (1728 × 1117 px in macOS Safari, Japanese ja-JP locale, dark mode):

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│ 🏢 aurelia-robotics-internacional-sa-de-cv-mx · ja-JP · 渡辺 明 (Akira Watanabe)     │
│ ops > quarters > Q4-2026 > snapshot                                                   │
├───────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                        │
│  Q4-2026 四半期運用レビュー (Q4-2026 Quarterly Operations Review)                     │
│  期間: 2026年4月1日 - 2026年6月30日                                                   │
│  Generated: 2026-05-12T09:00 +09:00 (Asia/Tokyo) / 2026-05-12T00:00 UTC                │
│  Sealed: EVT-J168-Q4-METRIC-SNAPSHOT-001                                              │
│                                                                                        │
│  ┌─ 全体メトリクス (Headline) ─────────────────────────────────────────────────────┐│
│  │ p99 レイテンシ:    94ms  (目標≤100ms)   ✓ GREEN                                  ││
│  │ スループット:      418K req/sec sustained  ✓ GREEN                               ││
│  │ Error budget burn: 0.74×                 ✓ GREEN                                 ││
│  │ 容量使用率:        62%                   ✓ GREEN                                 ││
│  │ AZあたり要員数:    14.2                  ✓ GREEN                                 ││
│  │ NPS:               68                    ✓ GREEN                                 ││
│  │ オンコール疲弊度:  4.8/10  (target ≤5.0) ✓ GREEN (但 APAC-Tokyo 6.8 RED)        ││
│  └────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                        │
│  ┌─ アクティブインシデント ────────────────────────────────────────────────────────┐│
│  │ SEV-1: 0                                                                          ││
│  │ SEV-2: 1 → incident-j168-sev2-apac-tokyo-2026-04-15-cell-failover-cascade-001     ││
│  │ SEV-3: 4                                                                          ││
│  └────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                        │
│  ┌─ セル別サマリ (Per-cell) ────────────────────────────────────────────────────────┐│
│  │ apac-tokyo-cell-tier-1-primary       p99 94ms · burn 0.94× · NPS 41 · burnout 6.8 ││
│  │ apac-sydney-cell-tier-1-secondary    p99 88ms · burn 0.62× · NPS 72 · burnout 4.1 ││
│  │ apac-singapore-cell-tier-1-tertiary  p99 92ms · burn 0.71× · NPS 69 · burnout 4.4 ││
│  │ eu-frankfurt-cell-tier-1-primary     p99 86ms · burn 0.58× · NPS 74 · burnout 4.0 ││
│  │ eu-dublin-cell-tier-1-secondary      p99 89ms · burn 0.64× · NPS 71 · burnout 4.3 ││
│  │ amer-cdmx-cell-tier-1-primary        p99 92ms · burn 0.78× · NPS 68 · burnout 5.0 ││
│  │ amer-aus-tx-cell-tier-1-secondary    p99 84ms · burn 0.54× · NPS 70 · burnout 4.2 ││
│  │ amer-qro-cell-tier-1-tertiary        p99 90ms · burn 0.66× · NPS 73 · burnout 4.5 ││
│  │ amer-sao-cell-tier-1-tertiary        p99 96ms · burn 0.82× · NPS 65 · burnout 4.9 ││
│  └────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                        │
│  Click any cell to drill into AZ-level metrics (27 metrics per AZ).                   │
│  Sealed snapshot — read-only.                                                          │
└───────────────────────────────────────────────────────────────────────────────────────┘
```

Akira clicks on `apac-tokyo-cell-tier-1-primary`. The cell-detail screen slides in showing the 3 AZ pivots side-by-side, each with the 27 metrics.

## §2 — SEV-2 incident debrief 5-Whys form (Thu May 14, 09:00 JST)

### Screen: `https://incident.aurelia-robotics-internacional-sa-de-cv-mx.oyatie.cloud/incidents/incident-j168-sev2-apac-tokyo-2026-04-15-cell-failover-cascade-001/debrief`

```
┌───────────────────────────────────────────────────────────────────────────────────────┐
│ SEV-2 デブリーフ · APAC-Tokyo セルフェイルオーバーカスケード                          │
│ Incident ID: incident-j168-sev2-apac-tokyo-2026-04-15-cell-failover-cascade-001       │
│ Trigger: 2026-04-15T03:42:18 +09:00 · Duration: 47 min · Affected: 12% tenants        │
│ Facilitator: 渡辺 明 (Akira Watanabe, COO)                                            │
├───────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                        │
│  Framework: 5-Whys (NIST-800-61-rev3 §3.5 + ISO-27035-1:2023 §6.5)                    │
│                                                                                        │
│  ┌─ Why 1: なぜSEV-2が発生したか? ────────────────────────────────────────────────┐│
│  │ Answer (ja-JP):                                                                  ││
│  │ セルフェイルオーバー制御が、失敗したプライマリと同じAZにフェイルオーバー先の    ││
│  │ podをスケジュールし、カスケード障害を引き起こした。                             ││
│  │ Answer (en-US):                                                                  ││
│  │ The cell-failover controller scheduled the failover-target pods on the SAME AZ  ││
│  │ as the failed primary, causing cascade failure.                                  ││
│  │ Evidence: EVT-INCIDENT-2026-04-15-CASCADE-DETECTED-04a                          ││
│  └──────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                        │
│  ┌─ Why 2: なぜpodが同じAZにスケジュールされたか? ────────────────────────────────┐│
│  │ Answer: Kubernetes anti-affinity rule was set to topologyKey:                  ││
│  │ kubernetes.io/hostname (node-level) instead of                                   ││
│  │ topology.kubernetes.io/zone (AZ-level).                                          ││
│  │ Evidence: EVT-INCIDENT-2026-04-15-ANTI-AFFINITY-MISCONFIG-06b                    ││
│  └──────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                        │
│  ┌─ Why 3: なぜanti-affinityがnode-levelに設定されていたか? ─────────────────────────┐│
│  │ Answer: Rule originally written in 2023 when apac-tokyo had only 1 AZ.          ││
│  │ When AZs b + c were added in Q1-2026, the rule was not updated.                  ││
│  │ Evidence: git-blame on k8s manifest manifest-2023-08-12-v0.4.2.yaml               ││
│  └──────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                        │
│  ┌─ Why 4: なぜAZ追加時にruleが更新されなかったか? ──────────────────────────────────┐│
│  │ Answer: Cell-topology-expansion runbook in 2024-Q1 did NOT include a step       ││
│  │ to audit anti-affinity rules against the new AZ topology.                       ││
│  │ Evidence: runbook git-log + the 2024-Q1 cell-expansion change-record             ││
│  └──────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                        │
│  ┌─ Why 5: なぜrunbookにこの監査ステップがなかったか? ──────────────────────────────┐│
│  │ Answer: Runbook written before ADR-0248 (cellular architecture) was fully       ││
│  │ formalized. Author tested locally without topology-aware scheduler installed,   ││
│  │ assumed anti-affinity would "just work" with new topology.                       ││
│  │ Evidence: ADR-0248 effective-date 2024-Q3; runbook date 2024-Q1                  ││
│  └──────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                        │
│  Root cause: kubernetes_anti_affinity_topology_key_misconfigured_to_node_instead_of_zone│
│  Secondary root cause: observability_failover_readiness_check_missing_az_boundary_audit│
│                                                                                        │
│  Next step: 是正措置(corrective actions)を定義する  →  [ Define 87 corrective actions ]│
└───────────────────────────────────────────────────────────────────────────────────────┘
```

## §3 — Capex Cedar quorum modal (Mon May 18, 09:18 CDT)

### Screen: per-line-item modal in the `governance` µservice's capex-approval flow

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│                                                                                       │
│           CAPEX LINE ITEM · Cedar quorum vote                                         │
│                                                                                       │
│  Change record:  CHG-OKR-Q1-2027-CAPEX-2026-05-18                                     │
│  Line item:      capex-line-1-sev2-corrective-action                                  │
│  Amount:         MXN 12,000,000                                                       │
│  OKR cycle:      Q1-2027                                                              │
│  Linked incident: incident-j168-sev2-apac-tokyo-2026-04-15-cell-failover-cascade-001  │
│                                                                                       │
│  Description:                                                                         │
│  Funding for 87 corrective-action items totaling 3,840 engineering-hours              │
│  across 4 teams (cell-topology + observability + platform + incident-management),    │
│  blended rate MXN 3,124/hr loaded. Linked to SEV-2 debrief.                          │
│                                                                                       │
│  Preconditions:                                                                       │
│  ✓ CRA signed by COO (Akira Watanabe, 2026-05-17T18:42 CDT, QES sat-mx-FIEL)         │
│  ✓ Linked incident debrief sealed (EVT-J168-DEBRIEF-COMPLETE-003)                    │
│  ✓ Business-hours-CDT (09:18 CDT)                                                    │
│  ✓ TrueTime uncertainty: 1.8 ms (≤ 10 ms target)                                     │
│                                                                                       │
│  Quórum requerido: 5 de 5 PERMIT (amount > MXN 5M)                                   │
│                                                                                       │
│  ┌─ Voters ─────────────────────────────────────────────────────────────────────┐  │
│  │ ✓ CEO Hugo Ávila Mendoza         PERMIT  09:18:18 CDT                         │  │
│  │ ✓ COO Akira Watanabe              PERMIT  09:18:42 CDT                         │  │
│  │ ◯ CTO Diego Vargas               (esperando)                                  │  │
│  │ ◯ CFO Patricia Carrillo Vega     (esperando)                                  │  │
│  │ ◯ Board-Ops-Chair Patrick Reilly (Webex from Tahoe)                          │  │
│  └────────────────────────────────────────────────────────────────────────────────┘  │
│                                                                                       │
│  Tu voto:                                                                            │
│         [ DENY ]              [ ABSTAIN ]              [ PERMIT ]                    │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

After all 5 votes:
```
QUÓRUM ALCANZADO: 5/5 PERMIT
Audit seal: EVT-J168-CAPEX-LINE-1-PERMIT-007a
Linked to incident: EVT-J168-CAPEX-LINKED-008
TrueTime uncertainty: 1.8 ms
```

## §4 — Customer-relationship repair attestation modal (Fri May 15, Komatsu meeting room)

The Komatsu CIO Watanabe-Kenji-san signs the attestation on his Surface Pro 11 device. The modal renders in Japanese on his side:

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│ 顧客関係修復証明書 · SEV-2 インシデント                                              │
│ (Customer Relationship Repair Attestation · SEV-2 Incident)                          │
│                                                                                       │
│  Tenant: komatsu-ltd-jp-tenant                                                       │
│  Counterparty: aurelia-robotics-internacional-sa-de-cv-mx                            │
│  Incident: 2026-04-15 APAC-Tokyo セルフェイルオーバーカスケード                      │
│  Affected deployment: Komatsu Indonesia Grasberg mine AHT fleet                      │
│  Affected duration: 18 min degraded read latency                                     │
│  Service credit: MXN 84,000                                                          │
│  Meeting: 2026-05-15 09:30-11:00 +09:00 at Komatsu HQ Akasaka                        │
│                                                                                       │
│  Evidence reviewed:                                                                  │
│   ✓ 5-Whys 分析 (5-Whys analysis)                                                   │
│   ✓ 87 是正措置 items (87 corrective actions)                                       │
│   ✓ MXN 12M capex commitment                                                         │
│   ✓ Merkle attestation EVT-J168-MERKLE-ATTESTED-005                                  │
│                                                                                       │
│  Customer acknowledgement:                                                           │
│   ☐ I (Watanabe Kenji-san, Komatsu CIO) acknowledge the incident debrief evidence   │
│     and accept the corrective-action plan as adequate.                              │
│                                                                                       │
│        [ Reject + escalate ]                       [ Sign + accept ]                 │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

Watanabe-Kenji-san checks the box, signs with his GMO GlobalSign EVCS QES. The dual-seal computes:

```
SIGNED · audit seal EVT-J168-CUSTOMER-REPAIR-004a
Dual-sealed in: komatsu-ltd-jp-tenant + aurelia-robotics-internacional-sa-de-cv-mx
TrueTime uncertainty: 2.8 ms
```

## §5 — Cross-time-zone display invariant

Every audit-seal record renders dual UTC + IANA-zoned local time:

```
EVT-J168-Q4-METRIC-SNAPSHOT-001
  UTC:           2026-05-12T00:00:00.000Z
  Asia/Tokyo:    2026-05-12T09:00:00.000 +09:00
  America/Mexico_City: 2026-05-11T18:00:00.000 -06:00 (sealed by Tokyo team but Mexico-tenant's view)
```

The locale toggle (top-right of every screen) lets users switch render language without altering the audit-record substance.

## §6 — Mobile view (iPhone 15 Pro Max, Akira's after-hours)

Akira's iPhone is on her hotel-room nightstand Sunday evening (May 10) when the next-day prep notifications come. The lock-screen widget renders:

```
┌──────────────────────────────────────┐
│ 渡辺 明 · aurelia-robotics            │
├──────────────────────────────────────┤
│ 明日のスケジュール (Tomorrow's)       │
│   09:00 Komatsu HQ Akasaka            │
│   13:00 Daifuku Komaki R&D            │
│   17:00 THK Tokyo HQ                  │
│                                       │
│ 来週の準備:                            │
│   Wed Q4 ops review                   │
│   Thu SEV-2 debrief                   │
│   Mon Q1-2027 capex Cedar vote        │
└──────────────────────────────────────┘
```

## §7 — Accessibility + locale + Japanese-character invariants

- Japanese kanji preservation (UTF-8 NFC): 渡辺 明 (with the half-width space between family + given name), 武井 博, 小松 (Komatsu), 三菱 (Mitsubishi), 住友重機械 (Sumitomo Heavy Industries).
- Honorifics: `-san` (さん) appended to surnames + given names per Japanese workplace conventions; the `messenger` µservice renders these correctly in chat.
- Spanish diacritics preserved: María José, Ávila, Carrillo, Vargas, Solís, Ramírez.
- en-US-side users see Latin-only transliteration (Watanabe Akira, Takei Hiroshi, etc.) with the kanji available in a hover-tooltip.
- WCAG AAA contrast on all dashboard panels.
- Keyboard navigation supports tab + enter; PERMIT/DENY buttons tab-order primary-action-last.
- Screen-reader: every modal has `aria-label` text matching user locale.
