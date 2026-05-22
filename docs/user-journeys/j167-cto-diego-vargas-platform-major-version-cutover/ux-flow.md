---
doc_class: User-Journey-UX-Flow
journey_id: j167-cto-diego-vargas-platform-major-version-cutover
date: 2026-05-20
authority_tier: 2
status: draft
---

# j167 — UX flow: cutover dashboard screens + Cedar permit modal + war-room console

## §0 — Devices in scope

| Person | Primary device | Secondary device | OS / browser |
|---|---|---|---|
| Diego Vargas (CTO) | MacBook Pro M4 14" (Space Black, 96 GB RAM) on Herman Miller Renew desk | iPad Pro M4 13" on magnetic stand | macOS 15.4 Sequoia; Safari 18.2 + Arc 1.92 |
| Yamilet Solís (VP-Eng) | ThinkPad X1 Carbon Gen 13 (Ubuntu 24.04 + Sway WM) | iPhone 15 Pro | Ubuntu 24.04 LTS; Firefox 132 ESR |
| Sofía Ramírez (NOC-QRO) | NixOS workstation (custom build, 64 GB RAM, AMD 7950X) | Dell U3223QE 32" 4K x 2 | NixOS 24.11; Firefox 132 |
| Brian Tate (SVP-CS) | MacBook Air M3 13" | iPhone 15 Pro Max | macOS 15.4; Arc 1.92 |
| Akira Watanabe (COO) | MacBook Pro M4 16" | iPad Pro M4 11" | macOS 15.4; Safari 18.2 |
| Diego's home setup | iMac M4 24" (kitchen study) for off-hours pages | iPhone 15 Pro | macOS 15.4; Safari 18.2 |

Primary UI locale for Mexican-side: `es-MX` (Spanish, Mexico). Austin-side: `en-US`. The UI auto-toggles based on the principal's `preferred_locale` attribute on identity.

## §1 — Pre-cutover readiness dashboard (Diego's MacBook, Tuesday Oct 20 07:42 CDT)

### Screen: `https://governance.aurelia-robotics-internacional-sa-de-cv-mx.oyatie.cloud/changes/CHG-V4-CUTOVER-2026-10-20/readiness`

**Layout** (1512 × 982 px in macOS Safari, dark mode):

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│ 🏢 aurelia-robotics-internacional-sa-de-cv-mx · es-MX · Diego Vargas                    │
│ governance > changes > CHG-V4-CUTOVER-2026-10-20                                        │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                          │
│  Cutover Aurelia Platform v3.x → v4.0                                                   │
│  Estado: ready_for_cohort_a_initiating                                                  │
│  Próxima compuerta: 2026-10-20T08:00:00-05:00 (en 17 min)                              │
│                                                                                          │
│  ┌─ Lista de verificación 87/87 ─────────────────────────────────────────────────────┐ │
│  │ ✓ Estado Terraform                    12/12 verde   [ver detalle]                  │ │
│  │ ✓ Despliegue Kubernetes               14/14 verde   [ver detalle]                  │ │
│  │ ✓ Observabilidad                      11/11 verde   [ver detalle]                  │ │
│  │ ✓ Configuración feature-flags          9/9  verde   [ver detalle]                  │ │
│  │ ✓ Comunicación a clientes             17/17 verde   [ver detalle]                  │ │
│  │ ✓ Preparación de incidentes           14/14 verde   [ver detalle]                  │ │
│  │ ✓ Evidencia de cumplimiento           10/10 verde   [ver detalle]                  │ │
│  └─────────────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                          │
│  ┌─ CRA firmado ─────────────────────────────────────────────────────────────────────┐ │
│  │ Yamilet Solís · 2026-10-13T16:42:18-05:00 · QES vía SAT-MX-FIEL                   │ │
│  └─────────────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                          │
│  ┌─ CAB sign-off quorum (firmado 2026-10-15) ────────────────────────────────────────┐ │
│  │ ✓ Diego Vargas (CTO)         PERMIT  14:18:00 -05:00                              │ │
│  │ ✓ Yamilet Solís (VP-Eng)     PERMIT  14:22:00 -05:00                              │ │
│  │ ✓ Akira Watanabe (COO)       PERMIT  14:28:00 -05:00                              │ │
│  │ ✓ Brian Tate (SVP-CS)        PERMIT  13:42:00 -05:00 (Austin · 12:42 CST)         │ │
│  └─────────────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                          │
│  ┌─ Vote countdown ──────────────────────────────────────────────────────────────────┐ │
│  │       VOTACIÓN COHORT A SE ABRE EN  ⏱ 16:42 ⏱ (M:SS)                              │ │
│  │       Quórum requerido: 4 de 4 PERMIT                                              │ │
│  │       TrueTime fence target: ≤ 10 ms (actual: 2.4 ms)                              │ │
│  └─────────────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                          │
│  [Sello de auditoría: EVT-J167-PRE-REVIEW-COMPLETE-001 · sellado 07:18:00 CDT]          │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

**Interaction**: Diego clicks `[ver detalle]` on "Observabilidad 11/11 verde". The detail panel slides in from the right and shows the 11 checks: dashboard URLs published, alert rules wired, SLO budgets reset, log-aggregation pipelines provisioned, etc. He spends 90 seconds scanning. Closes the panel.

## §2 — Cedar permit vote modal (07:58:00 CDT)

At 07:58:00 CDT the vote-countdown timer hits 00:00. A modal overlays the screen (with backdrop dim):

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                       │
│           VOTACIÓN COHORT A · INITIATING                              │
│                                                                       │
│  Cambio:   CHG-V4-CUTOVER-2026-10-20                                  │
│  Decisión: ¿Iniciar Cohort A (1% canary, 4 cells)?                    │
│                                                                       │
│  ┌─ Precondiciones ────────────────────────────────────────────┐    │
│  │ ✓ Pre-review SLO verde                                       │    │
│  │ ✓ CRA firmado                                                │    │
│  │ ✓ Business-hours-CDT (07:58 CDT)                             │    │
│  │ ✓ TrueTime uncertainty: 2.4 ms (≤ 10 ms target)              │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                       │
│  ┌─ Quórum ────────────────────────────────────────────────────┐    │
│  │ ◯ Diego Vargas (CTO)         (esperando)                     │    │
│  │ ◯ Yamilet Solís (VP-Eng)     (esperando)                     │    │
│  │ ◯ Akira Watanabe (COO)       (esperando)                     │    │
│  │ ◯ Brian Tate (SVP-CS)        (esperando)                     │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                       │
│  Justificación (es-MX):                                              │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │ Pre-review 87/87 verde. SLO baseline estable. CRA firmado.  │    │
│  │ Procedemos.                                                  │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                       │
│       [ DENY ]              [ ABSTAIN ]              [ PERMIT ]      │
│                                                                       │
│  Tu passkey + face-id se solicitarán al hacer clic.                  │
└─────────────────────────────────────────────────────────────────────┘
```

Diego clicks `[ PERMIT ]`. The macOS system passkey prompt slides up from the bottom edge: "Use Touch ID for Aurelia Governance?" He places his thumb on the Touch ID. Authenticates in 0.6 seconds. The face-id prompt is skipped on macOS (where Touch ID substitutes); on iPad+iOS it would prompt Face ID.

His vote stamps the modal:
```
✓ Diego Vargas (CTO)         PERMIT  07:58:18 -05:00
```

Quorum count: 1/4.

He sets the iPad on the magnetic stand so he can watch the other votes come in while he finishes his espresso.

By 07:58:42 CDT all 4 votes are in. The modal updates:

```
QUÓRUM ALCANZADO: 4/4 PERMIT
TrueTime uncertainty at decision: 2.4 ms
Audit seal: EVT-J167-COHORT-A-PERMIT-002
Dual-sealed in:
  - aurelia-robotics-internacional-sa-de-cv-mx
  - oya-governance-change-management-system-tenant

[ CERRAR ]
```

Diego closes the modal. The dashboard underneath transitions to the cohort-rollout view.

## §3 — Cohort rollout dashboard (08:00:00 CDT onward)

### Screen: `https://governance.aurelia-robotics-internacional-sa-de-cv-mx.oyatie.cloud/changes/CHG-V4-CUTOVER-2026-10-20/rollout`

**Three-pane layout**:

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│  Cohort A · 1% canary · 4 cells · ACTIVO desde 08:00:00 CDT                              │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ ┌─ Tráfico (1% v4) ────────────┐ ┌─ p99 latencia ───────┐ ┌─ Error budget burn ─────┐ │
│ │                              │ │                       │ │                          │ │
│ │ ███░░░░░░░░░░░░░░░░ 1.04%   │ │  84ms ━━━━━━━━━━     │ │ 0.4× ▁▁▁▁▁▁▁▁▁▁▁        │ │
│ │ (41 customer sites)          │ │  (baseline 84ms)      │ │ (target ≤ 1.0×)          │ │
│ │ ⬛ v4 routing  ⬜ v3 routing │ │  delta 0% verde       │ │ verde                    │ │
│ └──────────────────────────────┘ └───────────────────────┘ └──────────────────────────┘ │
│                                                                                          │
│ ┌─ Cells en Cohort A ──────────────────────────────────────────────────────────────────┐│
│ │ ✓ aws-cdmx-cell-tier-1-primary       p99: 82ms · err: 0.014% · burn: 0.3× verde     ││
│ │ ✓ aws-aus-tx-cell-tier-1-secondary   p99: 85ms · err: 0.018% · burn: 0.4× verde     ││
│ │ ✓ aws-qro-cell-tier-1-tertiary       p99: 84ms · err: 0.017% · burn: 0.4× verde     ││
│ │ ✓ aws-gdl-cell-tier-1-quaternary     p99: 86ms · err: 0.021% · burn: 0.5× verde     ││
│ └──────────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                          │
│ ┌─ Próximas compuertas ─────────────────────────────────────────────────────────────────┐│
│ │ Cohort B (10%)  Wed Oct 21 08:00 CDT  (en 23:42:14)  [ver checklist]                 ││
│ │ Cohort C (50%)  Fri Oct 23 08:00 CDT                                                  ││
│ │ Cohort D (100%) Tue Oct 27 08:00 CDT                                                  ││
│ │ v3.x sunset     Fri Oct 30 23:59 UTC                                                  ││
│ └──────────────────────────────────────────────────────────────────────────────────────┘│
│                                                                                          │
│ Última actualización: 13:58:42 CDT (auto-refresh cada 12 segundos)                       │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

The traffic-percentage gauge has a subtle pulsing animation (1% → 1.04% → 1% → 1.04% over 4 seconds; matches the 12-second auto-refresh).

## §4 — Canary-spike alarm modal (14:00:18 CDT)

At 14:00:18 CDT the SLO regression detector fires. The screen transitions:

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│ 🚨 ALARMA SEV-2 CANDIDATE                                                                │
│ dispatch-cell-qro · p99 latencia 312ms (baseline 84ms · delta +271%)                    │
│ sustained 12 min desde 13:48:42 CDT · error budget burn rate 2.1×                       │
│                                                                                          │
│ Páginas enviadas a:                                                                      │
│   - Sofía Ramírez (NOC-QRO primary)                                                      │
│   - Yamilet Solís (escalation-1)                                                         │
│   - Diego Vargas (escalation-1)                                                          │
│                                                                                          │
│ Árbol de decisión:                                                                       │
│   Rama A (acción actual): SEV-2 candidate, 1 cell afectado, burn < 5%/30min              │
│   → Investigar en vivo. No rollback automático.                                         │
│                                                                                          │
│ War-room: #cutover-v4-warroom (auto-abierto 14:00:30 CDT)                                │
│                                                                                          │
│        [ Abrir war-room ]            [ Ver root-cause-analysis live ]                   │
└─────────────────────────────────────────────────────────────────────────────────────────┘
```

Diego clicks `[ Abrir war-room ]`. The Slack channel opens in a new tab. He sees Sofía's first 4 messages already posted.

## §5 — Slack war-room (`#cutover-v4-warroom`)

The Slack channel is bridged through the `messenger` µservice with MLS end-to-end encryption. Messages render in `es-MX` for Diego/Yamilet/Sofía/Renata and `en-US` for Brian; the locale toggle is per-principal.

```
#cutover-v4-warroom · 7 members · MLS-encrypted (RFC 9420)
─────────────────────────────────────────────────────────────────────────────
[14:00:18 CDT] 🚨 sofia.ramirez:
🚨 SEV-2-candidate. dispatch-cell-qro p99 312ms sustained 12min.

[14:00:42 CDT] sofia.ramirez:
error-budget burn rate 2.1× — within tolerance for SEV-2 not SEV-1 (rama A).

[14:01:18 CDT] sofia.ramirez:
bisect: rolling back trace samples to find which path...

[14:02:42 CDT] sofia.ramirez:
hot path = path-planning eval; tail latency on Cedar bytecode lookup.

[14:03:18 CDT] sofia.ramirez:
Cedar bytecode hot-cache miss-rate jumped from 0.04% baseline to 18.4% on QRO cell.

[14:03:48 CDT] sofia.ramirez:
hypothesis: new principal shape (workload-identity-WebAuthn-derived) is hashing into
a different bucket; the bytecode pre-warm script didn't pre-warm v4 principal shapes
for QRO cell. Will verify.

[14:04:12 CDT] diego.vargas:
Yami, ¿el bytecode pre-warm corrió en CDMX + AUS-TX + GDL?

[14:04:42 CDT] yamilet.solis:
Sí. Todos los 4 cells reportan que el pre-warm corrió a las 07:42 CDT. Pero — espera —
el pre-warm de QRO corrió contra la imagen v3 del bytecode. La imagen v4 cargó después
del Terraform apply a las 08:00 CDT. El cache pre-warm está stale.

[14:05:18 CDT] diego.vargas:
Carajo. Bug en el order-of-operations del runbook. Pre-warm ANTES de v4 image deploy
— debió ser DESPUÉS.

[14:07:42 CDT] sofia.ramirez:
Pre-warm kicked off. ETA 14:18 CDT.
Job ID: bytecode-pre-warm-qro-2026-10-20-002
─────────────────────────────────────────────────────────────────────────────
Type a message... 🔒 MLS-encrypted · attachments will mark sensitive-incident
```

The Slack panel auto-pins the `Job ID` line; clicking it opens the `policy-engine` µservice's job-status panel inline.

## §6 — Saturday Oct 24 22:14 CDT — SEV-2 page on Diego's iPhone

Diego is at home watching the Tigres-América match. His iPhone 15 Pro (in the leather-clad case Lourdes bought him for Christmas 2025) is on the side table. At 22:14:18 CDT the page comes:

```
┌─────────────────────────────────────────────────────────┐
│ 🚨 [SEV-2] dispatch-cell-aus-tx                          │
│ error-budget burn rate 4.8× sustained 30 min             │
│                                                           │
│ 11 US tenants affected · ~127 robot fleets               │
│ War-room: #cutover-v4-warroom (auto-reopened)            │
│                                                           │
│ On-call primary: Sofía Ramírez                           │
│ Escalation-1: Diego Vargas (you)                         │
│                                                           │
│ TrueTime uncertainty: 1.8 ms                             │
│ Issued: 2026-10-24T22:14:18-05:00                        │
│                                                           │
│ [ ACK ]              [ Snooze 5m ]              [ Open ] │
└─────────────────────────────────────────────────────────┘
```

Diego taps `[ ACK ]` then `[ Open ]`. The oyatie mobile app opens to the cohort dashboard with the war-room channel pre-selected. He reads Sofía's first message ("CRD-watch lag, 3 pods serving v4 with stale flag bundle"), grabs his car keys, and drives to Torre Manacar.

## §7 — Rollback decision modal (Saturday Oct 24, 23:42 CDT)

Once the AUS-TX cell starts stabilizing (burn-rate dropping from 4.8× peak to 1.8×), Diego opens the rollback-decision panel to formalize the no-rollback vote:

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│ ROLLBACK DECISION · SEV-2 incident-j167-sev2-aus-tx-001                              │
│                                                                                       │
│  Status: monitoring                                                                  │
│  Time since alarm: 1h 28m (≤ 4h Cedar window)                                        │
│  Mitigation: pod re-roll completed 23:08 CDT                                         │
│  Current burn rate: 1.8× (peak 4.8× at 22:42 CDT)                                    │
│                                                                                       │
│  Decisión requerida: ¿Rollback Cohort C → Cohort B?                                  │
│  Quórum requerido: 3 de 4 PERMIT (Cedar `governance.cohort_rollback`)               │
│                                                                                       │
│  Opciones de decisión:                                                               │
│    ◯ ROLLBACK_NOW          (regresa Cohort C → Cohort B; v3 retoma 50%)             │
│    ◯ ROLLBACK_AFTER_HOTFIX (espera 1 ciclo de fix antes de rollback)                │
│    ◯ NO_ROLLBACK           (mitigation funcionó; continuar)                          │
│                                                                                       │
│  Tu voto:  [ ROLLBACK_NOW ]  [ ROLLBACK_AFTER_HOTFIX ]  [ NO_ROLLBACK ]              │
│                                                                                       │
│  Justificación:                                                                      │
│  ┌─────────────────────────────────────────────────────────────────────────────┐    │
│  │                                                                              │    │
│  └─────────────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

Each of {sofia, yamilet, brian, diego} votes NO_ROLLBACK with rationale text. The modal updates to:

```
QUÓRUM ALCANZADO: 4/4 NO_ROLLBACK
Incident status: monitoring → recovering
Follow-up: post-mortem programado lunes 2026-10-26 14:00 CDT
Audit seal: EVT-J167-SEV2-AUS-TX-008
```

## §8 — V3.x hard sunset confirmation (Friday Oct 30, 23:42 UTC = 17:42 CDT)

17 minutes before the sunset, Diego is in his office reviewing the final-sunset confirmation modal:

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│ V3.X HARD SUNSET · CHG-V4-CUTOVER-2026-10-20                                         │
│                                                                                       │
│  Programado:  2026-10-30T23:59:00Z (en 17 min)                                       │
│  Acción:     feature-flags.PUT /v1/flags/v3_api_enabled = false (scope: global)     │
│  Cascada:    legacy systems entran shutdown lifecycle                                │
│                                                                                       │
│  ┌─ Sistemas legacy a desactivar ─────────────────────────────────────────────────┐ │
│  │ ⏳ aurelia-fleetsync-v3-daemon                                                  │ │
│  │ ⏳ aurelia-gatewaybridge-v3-pods                                                │ │
│  │ ⏳ aurelia-contractadapter-v3-service                                           │ │
│  └─────────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                       │
│  ┌─ Tráfico v3 residual ──────────────────────────────────────────────────────────┐ │
│  │ Actual: 0.01% (residual; long-tail customer integrations no migradas)         │ │
│  │ Customer SDKs no migradas: 7 (todos los 7 notificados; soporte vence Nov 30)  │ │
│  └─────────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                       │
│  ┌─ Atestaciones a emitir ────────────────────────────────────────────────────────┐ │
│  │ • ISO-27001-A.12.1.2 → PwC México                                              │ │
│  │ • SOC2-CC8.1 → PwC México                                                       │ │
│  │ • EU-AI-Act-Art-17 notification → DEKRA Certification GmbH                     │ │
│  │ • MX-NOM-151-SCFI-2016 → archivo interno                                       │ │
│  └─────────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                       │
│  Tu sign-off como CTO se requiere para cerrar el change-record.                      │
│                                                                                       │
│         [ CANCELAR sunset ]                          [ CONFIRMAR sunset ]            │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

Diego clicks `[ CONFIRMAR sunset ]`. His passkey + Touch ID re-authenticate. The TrueTime fence reads `uncertainty=1.8ms`. The sunset is scheduled.

At 23:59:00 UTC the sunset executes. The dashboard refreshes:

```
V3.X SUNSET COMPLETE · 2026-10-30T23:59:00.000Z
✓ aurelia-fleetsync-v3-daemon shutdown 23:59:18Z (graceful)
✓ aurelia-gatewaybridge-v3-pods shutdown 23:59:42Z (graceful)
✓ aurelia-contractadapter-v3-service shutdown 24:00:18Z (graceful)

Audit seal: EVT-J167-V3-SUNSET-010
Merkle root: sha384-7e2a4b8c1d3f5e9a6b2c4d8e1f3a5c7b9d2e4f6a8c1b3d5e7f9a2c4b6d8e1f3a5c7b9d2e4f6a8c1b3d5e7f

Change record CHG-V4-CUTOVER-2026-10-20: closed
```

## §9 — Mobile app screens (iPhone 15 Pro)

The oyatie mobile app's cutover-monitoring screen on iPhone is a single-pane summary:

```
┌──────────────────────────────────────┐
│ Cutover v4 · status                  │
├──────────────────────────────────────┤
│ Cohort actual: D (100%)              │
│ Cells live v4: 47/47                 │
│ Cells live v3: 0/47                  │
│ Active SEV-1/2: 0                    │
│ Active SEV-3: 0                      │
│                                       │
│ Servicio credits (cumulative):       │
│   MXN 142,000 / target ≤ 200,000     │
│   bar: ████████░░░░░ 71%             │
│                                       │
│ Próximas decisiones:                  │
│   v3.x sunset · Fri Oct 30 23:59 UTC │
│                                       │
│ ▶ Ver dashboard completo (web)       │
│ ▶ Abrir war-room                     │
│ ▶ Ver post-mortem                    │
└──────────────────────────────────────┘
```

## §10 — Accessibility + locale + diacritic invariants

- Spanish UI (es-MX) uses the **Mexican formal "usted"** in system-generated copy + the **informal "tú"** in human-authored chat; the locale-toggle is at the per-user attribute level.
- Diacritic preservation: principal names "Sofía Ramírez", "Yamilet Solís", "Renata Castro", "Akira Watanabe" render with full UTF-8 NFC across all screens + audit seals + Slack messages + email notifications.
- Color-blind mode: green/red rollout indicators have iconographic redundancy (✓ / ✗ / ⏳) in addition to color.
- Screen-reader: every Cedar-permit modal has explicit `aria-label` text in es-MX or en-US matching the user's locale.
- Keyboard navigation: every modal supports tab + enter without mouse; the PERMIT/DENY/ABSTAIN buttons follow tab order primary-action-last so accidental enter-key submission goes to ABSTAIN not PERMIT.
- High-contrast mode: dashboard panels meet WCAG AAA contrast (7:1 minimum) for text + critical iconography.
