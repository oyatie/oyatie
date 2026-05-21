---
doc_class: User-Journey-Story
journey_id: j167-cto-diego-vargas-platform-major-version-cutover
date: 2026-05-20
authority_tier: 2
status: draft
---

# j167 — Story: 07:42 CDT in Torre Manacar, the v4 dashboard turns green

## §0 — Tuesday October 20, 2026, 07:42 CDT — Torre Manacar floor 14, Colonia del Valle, CDMX

The CTO office on the 14th floor of **Torre Manacar** — the south tower of the Manacar mixed-use complex at Avenida Insurgentes Sur 1457 — has a south-facing window that catches the morning sun rising over the World Trade Center México and, on a clear day, the snow on Iztaccíhuatl 65 km southeast. Today is clear, 14 °C at 07:42 CDT, the sky a thin Mexico-City autumn blue, the haze just barely visible at the southern horizon. Diego Vargas's office is corner-east-south: a maple-and-steel desk (Herman Miller Renew), a black-leather lounge chair from his father's old PEMEX-Tampico-era house, a 1:48 scale model of an Aurelia AMR-3000 (the original 2018 robot that first won the company's Bimbo logistics contract) sitting on the windowsill, and a 32-inch Apple Studio Display on a swing-arm beside his **MacBook Pro M4 14-inch (Space Black, 96 GB RAM, 4 TB SSD)**. A second screen — an iPad Pro M4 13-inch — sits on a magnetic stand to his left, locked to the `governance` µservice's cutover-cohort dashboard.

Diego is wearing a charcoal-gray quarter-zip pullover over a navy t-shirt, dark jeans, and Chelsea boots. He has a black Café de Olla from the floor's communal Breville espresso machine in his right hand. Mexican-northern news radio plays softly in the background — Grupo Fórmula's morning broadcast about the latest BMV close (AURELIA-A closed up 1.4% yesterday, at MXN 84.20). His tenant chip at the top of the MacBook reads:

> **🏢 aurelia-robotics-internacional-sa-de-cv-mx · empresa · 1 tenant active**

He opens the `governance` µservice's **Cutover-V4 readiness dashboard**. The dashboard URL is `https://governance.aurelia-robotics-internacional-sa-de-cv-mx.oyatie.cloud/changes/CHG-V4-CUTOVER-2026-10-20`. The top of the page reads:

```
Change Record CHG-V4-CUTOVER-2026-10-20 · Aurelia Platform v3.x → v4.0
Status: ready_for_cohort_a_initiating
CRA risk assessment: APPROVED 2026-10-13 (Yamilet Solís, VP-Eng)
CAB sign-off: APPROVED 2026-10-15 (Diego Vargas + Yamilet Solís + Akira Watanabe + Brian Tate)
Pre-flight checklist: 87/87 green
Audit-chain seal: EVT-J167-PRE-REVIEW-COMPLETE-001 (sealed 2026-10-20T07:18:00-05:00)
Cedar permit required: cohort_a_initiating quorum 4-of-4
Next gate: 2026-10-20T08:00:00-05:00 (in 17 min)
```

Diego scrolls through the 87 readiness items. He sees the same 87 items that he and Yamilet walked through together yesterday evening: 12 items for the Terraform module versioning + state-machine ordering, 14 items for K8s deployment readiness (image digests pinned, CRDs published, RBAC policies precomputed), 11 items for observability (dashboards built, alert rules wired, SLO budgets reset), 9 items for feature-flag rollout config (per-cell flag-bundle published), 17 items for customer-communication (cohort-A customers notified at T-72h, T-24h, T-2h), 14 items for incident-readiness (war-room channel created, runbooks in `tools/runbooks/v4-cutover.md`, paging schedules confirmed), 10 items for compliance-evidence (ISO-27001-A.12.1.2 + SOC2-CC8.1 + EU-AI-Act-Art-17 attestation packets pre-staged).

He marks the last item — "CTO personal sign-off" — as green. The Cedar engine asks for his passkey + face-id. He authenticates. The seal computes. `EVT-J167-PRE-REVIEW-COMPLETE-001` advances to status `sealed`.

He picks up the iPad and pings Yamilet on Slack at 07:47:18 CDT.

**Diego 07:47 CDT** (`#aurelia-ops-onefloor14`, Spanish): "Yami, estás en línea? Cohort A vote en 13 minutos."

**Yamilet Solís 07:47 CDT** (CDMX 14th floor office two doors down): "Aquí. Estoy revisando el último burn de Cohort A SLO budget. Todo verde. Voy a tu oficina."

**Diego 07:47 CDT**: "Trae café."

Yamilet Solís — 39, VP Engineering, Cuban-Mexican (mother from Havana, father from Veracruz; she grew up in Coatzacoalcos), Cornell PhD in Distributed Systems 2018, joined Aurelia in 2020 from a stint at Google Mountain View — walks into Diego's office at 07:51 with two espressos and a tablet. She sits in the visitor chair to Diego's left.

**Yamilet 07:51 CDT** (Spanish): "Sofía está en el NOC en Querétaro. Brian se conecta desde Austin a las 7:58. Akira vota a las 7:59 — tiene una reunión a las 8:30 con el board, así que necesita salir rápido."

**Diego 07:52 CDT**: "Bien. Recordamos: si Cohort A no estabiliza para las 14:00, abrimos war-room en `#cutover-v4-warroom`. El árbol de decisión está en el runbook. Si la métrica de error-budget burn excede 5% en 30 minutos, rollback automático — sin votación humana."

**Yamilet 07:52 CDT**: "Sí. Los robots de Bimbo en CDMX-Vallejo están en Cohort A — me mandó Brian un mensaje que el cliente quiere status update cada 2 horas durante el primer día. Le dije OK."

They sip espresso.

## §1 — Tuesday Oct 20, 07:58:42 CDT — the Cohort A Cedar permit vote

At 07:58:00 CDT the `governance` µservice opens the Cedar-permit voting window. A modal pops on Diego's MacBook + Yamilet's tablet:

```
COHORT A INITIATING — Cedar permit vote
Quorum required: 4 of 4 PERMIT
Voters: Diego Vargas (CTO), Yamilet Solís (VP-Eng), Akira Watanabe (COO), Brian Tate (SVP-CS)
Precondition: pre_review SLO green ✓
Precondition: CRA signed ✓
Precondition: business-hours-CDT ✓ (07:58 CDT)
TrueTime fence: ≤ 10 ms (current uncertainty: 2.4 ms)
```

Diego taps PERMIT. His passkey + face-id authenticates. His vote seals at 07:58:18 CDT.

Yamilet taps PERMIT 07:58:22 CDT.

Brian Tate in Austin (06:58 CDT his clock, but Slack shows him as online with green dot in `#cutover-v4-warroom`) taps PERMIT 07:58:31 CDT.

Akira Watanabe — she's in Mexico-City at the COO office on the 12th floor — taps PERMIT 07:58:42 CDT. The quorum hits 4-of-4 PERMIT. The Cedar engine evaluates the policy:

```cedar
permit (
    principal in Group::"aurelia-cutover-quorum-members",
    action == Action::"governance.cohort_transition_vote",
    resource is CohortGate
) when {
    resource.target_cohort == "cohort_a" &&
    resource.quorum_count == 4 &&
    resource.previous_cohort_slo_green == true &&
    resource.cra_document_signed == true &&
    context.business_hours_cdt == true &&
    context.unicode_normalization == "NFC"
};
```

Permit. The decision is dual-sealed: in `aurelia-robotics-internacional-sa-de-cv-mx` AND in `oya-governance-change-management-system-tenant`. The TrueTime fence at decision-time reads `uncertainty=2.4ms`. `EVT-J167-COHORT-A-PERMIT-002` seals at 07:58:42.118 CDT.

The `governance` workflow advances state `pre_review → cohort_a_initiating`. The `cloud-iac` µservice receives the trigger and executes the Terraform module v-bump cascade across 4 canary cells:

```
cloud-iac.apply --module aurelia-platform-v4 --version 4.0.0 \
  --target aws-cdmx-cell-tier-1-primary \
  --target aws-aus-tx-cell-tier-1-secondary \
  --target aws-qro-cell-tier-1-tertiary \
  --target aws-gdl-cell-tier-1-quaternary \
  --strategy serial-with-checkpoint
```

The apply runs 02:17 minutes. Each cell's K8s namespace receives the new v4 ImageDigests; the rolling-update strategy begins. The `cloud-k8s` µservice reports pod-roll progress to the governance dashboard every 15 seconds.

At 08:00:00 CDT exactly, the `feature-flags` µservice flips the canary-traffic-split rule:

```yaml
flag_id: aurelia-fleet-coordinator-version
rule:
  cohort_a:
    percentage: 1
    cells: [aws-cdmx-cell-tier-1-primary, aws-aus-tx-cell-tier-1-secondary, aws-qro-cell-tier-1-tertiary, aws-gdl-cell-tier-1-quaternary]
    target_version: 4.0.0
    fallback: 3.x
```

The first 1% canary requests start hitting the v4 endpoints. The Bimbo CDMX-Vallejo warehouse's fleet of 47 AMRs immediately routes 1 in 100 dispatch decisions through the new v4 path-planning module. `EVT-J167-COHORT-A-LIVE-003` seals at 08:00:18 CDT.

Diego exhales. Yamilet refills the espresso machine for both of them.

## §2 — Tuesday Oct 20, 08:00–14:00 CDT — Cohort A steady-state watch

For the next 6 hours, Diego splits his attention between the cohort dashboard and his usual CTO Tuesday slots: a 09:00 architecture review with the Querétaro engineering team (over Webex; Yamilet leads, Diego listens), a 10:30 product-strategy 1:1 with CPO Renata Castro (in person, her office 14th floor), an 11:30 working lunch with the BMV-CFO compliance team (in a glass-walled room overlooking Insurgentes), and a 13:30 prep call with the Aurelia Robotics IR (Investor Relations) team about the Q3 earnings call next Thursday.

Through all of this his iPad shows the cohort-A dashboard:

```
COHORT A · 1% canary · 4 cells
Traffic to v4: 41 customer sites · 1.04% of total
p99 latency v4:  84ms (baseline 84ms — 0% delta) GREEN
p99 latency v3:  86ms (baseline 84ms — +2.3% normal noise) GREEN
Error rate v4:   0.018% (target ≤ 0.05%) GREEN
Error budget burn rate: 0.4× (target ≤ 1.0×) GREEN
Cedar policy decision latency p99: 1.8ms (target ≤ 5ms) GREEN
Cross-tenant audit-seal latency p99: 4.1ms (target ≤ 10ms) GREEN
Last update: 13:58:42 CDT
```

Sofía Ramírez — 31, lead production-engineer at the Querétaro NOC, **regiomontana** like Diego (born and raised in Monterrey, but moved to Querétaro for the Aurelia job in 2022 after her partner's PhD took him to TecNM Querétaro), graduated UANL Computer Science 2018, master at Stanford 2021 then Datadog 2 years before Aurelia — watches the same dashboard from the NOC at the Aurelia-Querétaro office (Edificio Aurelia-QRO at Avenida 5 de Febrero in Industrial Sector San Pedrito).

At 14:00:18 CDT Sofía's NOC pager fires. The Grafana alert `AlertSLO_p99Regression_dispatch-cell-qro` triggers. The metric:

```
dispatch-cell-qro · p99 latency · v4 cohort
baseline 84ms (last 7-day p99 average)
trigger threshold: > 200ms sustained 5 min
current value: 312ms sustained 12 min
sustained start: 13:48:42 CDT
```

Sofía pages Diego + Yamilet + the war-room channel within 18 seconds.

## §3 — Tuesday Oct 20, 14:00–16:42 CDT — first canary spike + war room

Diego is in the 13:30 IR prep call when his phone buzzes. He sees the page: `[SEV-2-CANDIDATE] dispatch-cell-qro p99 latency 312ms sustained 12min — cohort A`. He apologizes to the IR team, ends the call, walks fast to his office (he is two corridors away). Yamilet is already there, leaning over the MacBook.

**Yamilet 14:01 CDT** (Spanish): "Vi el alert. Sofía está en `#cutover-v4-warroom`. El spike es en QRO solamente — CDMX + AUS-TX + GDL están verdes."

**Diego 14:01 CDT**: "Abriendo war-room. ¿Cuál es el árbol de decisión?"

**Yamilet 14:02 CDT**: "Rama A: si el SLO regression se limita a 1 cell y no hay error-budget burn > 5% en 30 min, NO rollback — investigamos en vivo. Si en 30 min no hay root cause, escalamos a la rama B."

In the `#cutover-v4-warroom` Slack channel, Sofía has already posted:

```
[14:00:18 CDT] Sofía Ramírez (NOC-QRO): 🚨 SEV-2-candidate. dispatch-cell-qro p99 312ms sustained 12min.
[14:00:42 CDT] Sofía Ramírez: error-budget burn rate 2.1× — within tolerance for SEV-2 not SEV-1 (rama A).
[14:01:18 CDT] Sofía Ramírez: bisect: rolling back trace samples to find which path...
[14:02:42 CDT] Sofía Ramírez: hot path = path-planning eval; tail latency on Cedar bytecode lookup.
[14:03:18 CDT] Sofía Ramírez: Cedar bytecode hot-cache miss-rate jumped from 0.04% baseline to 18.4% on QRO cell.
[14:03:48 CDT] Sofía Ramírez: hypothesis: new principal shape (workload-identity-WebAuthn-derived) is hashing into a different bucket; the bytecode pre-warm script didn't pre-warm v4 principal shapes for QRO cell. Will verify.
```

Diego reads this. Yamilet pulls up the `policy-engine` µservice's bytecode-cache metrics.

**Diego 14:04 CDT**: "Yami, ¿el bytecode pre-warm corrió en CDMX + AUS-TX + GDL?"

**Yamilet 14:04 CDT** (looking at her tablet): "Sí. Todos los 4 cells reportan que el pre-warm corrió a las 07:42 CDT. Pero — espera — el pre-warm de QRO corrió contra la imagen v3 del bytecode. La imagen v4 cargó después del Terraform apply a las 08:00 CDT. El cache pre-warm está stale."

**Diego 14:05 CDT**: "Carajo. Bug en el order-of-operations del runbook. Pre-warm ANTES de v4 image deploy — debió ser DESPUÉS."

The fix is straightforward: re-run the bytecode pre-warm job against the v4 image on the QRO cell. The job takes 11 minutes to complete. Sofía kicks it off at 14:07 CDT. At 14:18 CDT the cache miss-rate drops back to 0.06%. p99 latency on QRO drops from 312 ms → 142 ms within 3 minutes (still above baseline because the cache is now repopulating from cold), then to 88 ms by 14:42 CDT.

**Sofía 14:42 CDT** (Slack `#cutover-v4-warroom`, Spanish): "Recuperado. p99 dispatch-cell-qro de vuelta a 88ms. Error budget burn rate de vuelta a 0.6×. Bug en runbook step 14.2 — pre-warm corre antes del Terraform apply, debió ser después."

**Diego 14:43 CDT** (Slack): "Confirmado. Sofía: edita el runbook ya, paste-link aquí. Yami: agrega Cohort B precondition — pre-warm DESPUÉS del Terraform apply."

**Sofía 14:46 CDT**: posted runbook PR `tools/runbooks/v4-cutover.md@step-14.2-fix`.

**Yamilet 14:47 CDT**: posted Cohort B precondition addendum to the CRA document.

`EVT-J167-CANARY-SPIKE-ALARM-004` and `EVT-J167-MITIGATION-APPLIED-005` both seal by 14:48 CDT.

The rest of the afternoon is calm. By 18:00 CDT Cohort A is in steady-state. Diego goes home at 18:42 CDT. He has dinner with his wife — **Lourdes Bautista**, 45, an architect at AT&T-México (her firm does workspace fit-outs; she met Diego in 2015 at a TED-X CDMX dinner) — and their daughter **Sara Vargas Bautista**, 13, who is doing her secundaria-2 homework on her iPad at the dining table. Lourdes asks how the day was. Diego says, in Spanish, "Un susto a las dos de la tarde, pero arreglado." She nods.

## §4 — Wednesday Oct 21, 08:00 CDT — Cohort B Cedar permit vote

By Wednesday morning Cohort A has run 24 hours stable. 41 customer sites × 1% traffic × 24 hours = ~24,000 v4 requests served. Zero customer complaints. Zero new alarms.

At 07:58 CDT the Cohort B vote modal opens. Same 4 quorum members. Same Cedar policy (but `target_cohort == "cohort_b"`). All 4 vote PERMIT within 90 seconds.

`EVT-J167-COHORT-B-PERMIT-006` seals at 08:00:08 CDT under TrueTime fence (uncertainty 1.8 ms). Cohort B activates: 12 cells × 10% traffic = ~3,400 customer sites × 10% of traffic.

By 14:00 CDT one minor regression surfaces: **Customer 14** (a Brazilian agricultural cooperative, Cooperativa Agrícola Cotrijal Ltda., tenant `cotrijal-coop-rs-br`) reports their custom Cedar policy bundle is failing to compile under the v4 evaluator. Sofía traces the failure to a Cedar-v3 deprecated function `principal.isInGroup()` that v4 renames to `principal in Group`. The Cotrijal team's policy bundle uses the old form.

**Sofía 14:42 CDT** (Slack): "Cotrijal policy bundle fails Cedar v4 compile. Deprecation was announced Apr 20 but their team didn't update. Mitigation: pin `cotrijal-coop-rs-br` to v3 evaluator via `feature-flags` per-tenant override; ticket them to update by Oct 27."

**Diego 14:43 CDT**: "Hazlo. NO rollback — un cliente local-pinned a v3 no justifica abortar Cohort B."

The per-tenant override ships in 22 minutes. Cotrijal's traffic continues on v3 evaluator (1 tenant of 3,400+); the rest of Cohort B is on v4. Cotrijal's account-manager Renata Castro pings their team and gets confirmation they'll update by Oct 25.

## §5 — Friday Oct 23, 08:00 CDT — Cohort C 50% gate

Cohort B ran 48 hours stable (only the Cotrijal hotfix; no SEV-level incidents). The Cohort C vote opens at 07:58 CDT Friday.

Diego is in his office, Yamilet beside him as usual. Akira Watanabe joins via Slack from her CDMX office (her quarterly-ops-review prep is ramping; cross-ref j168). Brian Tate joins from Austin (07:58 CDT his clock now — he came in early specifically for this vote).

All 4 vote PERMIT. `EVT-J167-COHORT-C-PERMIT-007` seals at 08:00:14 CDT.

Cohort C activates: 24 cells × 50% traffic. Now half of all Aurelia traffic flows through v4.

By 18:00 CDT Friday the metrics look healthier than v3.x baseline:

```
v4 p99 latency:  78ms (vs v3 baseline 84ms — 7% improvement)
v4 error rate:   0.014% (vs v3 baseline 0.022% — 36% improvement)
v4 path-planning latency p99: 142ms (vs v3 baseline 218ms — 35% improvement)
```

Diego sends a brief Slack message to `#aurelia-ops-onefloor14`:

**Diego 18:18 CDT** (Spanish): "Cohort C estable. v4 latencia 7% mejor que v3 baseline. Path-planning 35% mejor. Buen trabajo equipo. Descansen sábado."

He goes home. Lourdes has cooked **chiles en nogada** — out of season (the traditional August dish), but she had a craving — and they eat with Sara and Diego's mother **Esperanza Cantú de Vargas** (74, retired schoolteacher, visiting from Monterrey for the long weekend).

## §6 — Saturday Oct 24, 22:14 CDT — SEV-2 alarm on Austin cell

Diego is in the family room watching the Tigres UANL vs Club América futbol match (Liga MX, Tigres up 1-0 in the 68th minute) when his phone buzzes. The page reads:

```
[SEV-2] dispatch-cell-aus-tx · error-budget burn rate 4.8× sustained 30 min
Triggered: 2026-10-24T22:14:18-05:00
Active customers affected (estimated): 11 US tenants on AUS-TX cell · ~127 robot fleets
War-room: #cutover-v4-warroom reopened
On-call: Sofía Ramírez (primary) + Diego Vargas (escalation)
```

He stands. Lourdes looks at him. He says, in Spanish, "Austin. Voy a la oficina." She nods. He drives the 18 minutes from their house in Lomas de Chapultepec to Torre Manacar. He arrives at 22:42 CDT. Yamilet is already there (she lives closer, in San Ángel; she got the same page and beat him in).

In Austin, Brian Tate has joined the war room (it's 22:42 CDT his clock = 22:42 CST Austin, his daughter Madeleine is at a sleepover, his wife Hannah is reading in the living room; Brian works from his home office on East 7th Street).

**Sofía 22:48 CDT** (Slack `#cutover-v4-warroom`, English-Spanish mixed): "AUS-TX has a CRD-watch lag. The pod-rolling-update strategy for v4 deployed a CRD version mismatch on 3 pods. Those pods are serving v4 traffic with stale flag bundle — they think the flag bundle is from yesterday's snapshot."

**Brian Tate 22:49 CDT** (Slack, English): "Customer impact: I see Walmart-Sam's Bentonville-DC, Penske-Reading PA, and Amazon-Robotics-LDV2 reporting elevated dispatch-fail rates. Penske's on-call paged me direct. They want SITREP every 30 min."

**Yamilet 22:50 CDT** (Slack, English-Spanish mixed): "Fix: force re-roll the 3 pods to pick up the latest CRD. Should be 8 minutes. Sofía — can you do it now?"

**Sofía 22:51 CDT**: "On it. `kubectl rollout restart deployment/dispatch-fleet-coordinator-v4 -n aurelia-platform -context aws-aus-tx-cell-tier-1-secondary` — running now."

The restart kicks off. Sofía monitors. Pod 1 restarts at 22:53 (8 min before Service Mesh re-routes). Pod 2 at 22:56. Pod 3 at 23:01. By 23:08 CDT all 3 pods are on the correct CRD version.

**Brian 23:18 CDT** (Slack, English): "Burn rate dropping. 1.8× now from 4.8× peak. Customer dashboards stabilizing."

**Diego 23:42 CDT** (Slack, Spanish): "OK. Rollback question: Cedar permit `governance.cohort_rollback` requires 3-of-4. ¿Votamos?"

**Yamilet 23:43 CDT**: "Mi vote: NO ROLLBACK. La mitigación funcionó. Error budget burn rate ya baja. El cell se está estabilizando."

**Brian 23:43 CDT** (Slack): "No rollback. Penske and Walmart are calming down. I'll write apology emails tomorrow morning."

**Sofía 23:44 CDT**: "No rollback. Estable."

**Diego 23:44 CDT**: "Acuerdo. No rollback. Pero — quiero post-mortem el lunes. Yami, programa."

`EVT-J167-SEV2-AUS-TX-008` seals at 23:46 CDT with the no-rollback decision attached.

Diego drives home at 00:18 Sunday. Lourdes is asleep. He sleeps for 5 hours and wakes at 06:18 to a clear Sunday and a stabilizing dashboard.

## §7 — Sunday Oct 25, 09:18 CDT — full stabilization

By Sunday 09:18 CDT the AUS-TX cell's error-budget burn rate is at 0.3× — well below the 1.0× target. Diego pings the war-room:

**Diego 09:18 CDT**: "AUS-TX estable 9 horas. Cerrando incidente. Sofía — escribe post-mortem para el lunes."

**Sofía 09:42 CDT**: "Confirmado. Borrador para las 14:00 mañana."

The war-room channel goes quiet. Diego spends Sunday with Sara at the **Museo Nacional de Antropología** (her school trip is next week and she wants to scope the Mexica room ahead of time so she's not surprised by the Coatlicue statue).

## §8 — Tuesday Oct 27, 08:00 CDT — Cohort D 100% Cedar permit vote

Cohort C ran 96 hours total (the SEV-2 took ~9 hours of attention; the rest was steady-state). The Cohort D 100% vote opens at 07:58 CDT Tuesday.

Diego is in his office. Yamilet, Akira, Brian — all four quorum members vote PERMIT within 70 seconds.

`EVT-J167-COHORT-D-PERMIT-009` seals at 08:00:12 CDT under TrueTime fence.

Cohort D activates: all 47 Tier-1 cells × 100% traffic. The remaining v3.x traffic — about 0.4% on a small set of long-tail customers — automatically migrates as the feature-flag rule flips. The `cloud-iac` µservice executes the final Terraform module v-bump across the 23 cells that weren't in Cohort A/B/C.

By 10:42 CDT the observability dashboard shows solid green across all 47 cells. v3.x traffic is at 0.04% (residual from a handful of customer integrations that are still on stale SDKs and will catch up over the next 3 days).

**Yamilet 10:43 CDT** (Slack, Spanish): "Hicimos. 100% v4. Cero rollback. Total service-credits emitidos: MXN 142,000 (vs MXN 4.2M del cutover v2→v3 en 2023)."

**Diego 10:44 CDT** (Slack): "30× mejor. Beberemos con el equipo el viernes."

## §9 — Wed Oct 28 – Thu Oct 29 — stabilization period

Two days of stabilization. v3.x traffic drops to 0.01% by Wednesday 18:00 (a few customer integrations finally update). Yamilet's post-mortem from the QRO cache miss (Tuesday Oct 20) and the AUS-TX CRD lag (Saturday Oct 24) is published on Confluence at `https://aurelia-confluence.atlassian.net/wiki/spaces/ENG/pages/v4-cutover-postmortem-2026-10-29`. The two action items: (1) move bytecode pre-warm to AFTER Terraform apply in the runbook; (2) add CRD-watch lag as a hard SLO precondition before any cohort transition. Both action items shipped to `tools/runbooks/v4-cutover.md` and to the `governance` µservice's CRA-template-2027 by Thursday EOD.

## §10 — Friday Oct 30, 23:59 UTC — v3.x hard sunset

Friday Oct 30 at 16:59 CDT (= 23:59 UTC) the `feature-flags` µservice flips `v3_api_enabled` to `false` globally. The legacy systems enter shutdown lifecycle:

- **Aurelia FleetSync v3.x** — last v3 daemon shuts down at 16:59:18 CDT
- **Aurelia GatewayBridge v3.x** — last gateway pod shuts down at 16:59:42 CDT
- **Aurelia ContractAdapter v3.x** — final contract-adapter shutdown at 17:00:18 CDT (graceful drain of last in-flight session)

`EVT-J167-V3-SUNSET-010` seals at 17:00:42 CDT under TrueTime fence.

The `governance` µservice closes the change-record CHG-V4-CUTOVER-2026-10-20 at 17:01:08 CDT. The audit-chain Merkle root for the entire cutover is computed and dual-sealed in `aurelia-robotics-internacional-sa-de-cv-mx` AND in `oya-governance-change-management-system-tenant`. The Merkle root: `sha384-7e2a4b8c1d3f5e9a6b2c4d8e1f3a5c7b9d2e4f6a8c1b3d5e7f9a2c4b6d8e1f3a5c7b9d2e4f6a8c1b3d5e7f`.

Diego signs the closing attestation. ISO-27001-A.12.1.2 + SOC2-CC8.1 + EU-AI-Act-Art-17 attestation packets are auto-generated and routed to PwC México (SOC2 auditor) and to the EU-AI-Act notified body **DEKRA Certification GmbH** (which issued Aurelia's high-risk-AI module conformity assessment in 2025; for this cutover, since the AI module's safety-relevant interfaces are preserved, only a notification-of-change is required, not a full re-assessment).

`EVT-J167-V3-SUNSET-010` is the final seal of the journey.

Diego closes his MacBook at 17:42 CDT. He drives home. Lourdes has Sara at a school event. He has the house to himself for 90 minutes. He pours a Casamigos Reposado on the rocks, sits in the leather chair, and looks at the model AMR-3000 on his desk windowsill — the 2018 robot that started this company. It's quiet.

## §11 — Beats not on the wire (the human texture)

- During the Saturday Oct 24 SEV-2 Diego was at home watching Tigres UANL (his hometown team; he's been a Tigres fan since the 1986 championship season when he was 7 years old and his father took him to a match at Estadio Universitario). He drove from Lomas de Chapultepec to Torre Manacar at 22:18 CDT in his black Tesla Model S (Mexican-spec, plates from CDMX `PXC-94-12`); the city was quiet on a Saturday night and he made the 18 minutes in 14. He listened to **Caifanes' "Aviéntame"** on the drive (Mexican-rock from 1994; his go-to focus music). When he arrived at Torre Manacar the night guard — **don Filiberto**, 62, who has been at this building since 2019 and knows every executive by face — said: "Buenas noches, ingeniero. Algo serio?" Diego said: "Sí. Cuestión de horas." Don Filiberto nodded and let him through.
- Sofía Ramírez worked the entire weekend remote from her apartment in Querétaro's Centro Histórico district (Calle 5 de Mayo, near the Plaza de Armas). Her partner **Iván Mendoza Cruz**, 33, a CONACYT-funded PhD candidate in computational fluid dynamics at TecNM Querétaro, brought her quesadillas + agua de jamaica from the cocina-económica downstairs at 14:42 on Saturday + Sunday. She didn't leave the apartment from Friday 18:00 to Sunday 14:00 except to walk Iván's dog Pancho (a 6-year-old chihuahua-mix) at 06:00 and 18:00 Saturday and Sunday.
- Brian Tate's on-call burden was heavy: 11 US customer tenants on AUS-TX, several of whom are not used to multi-day cohort rollouts (they're used to monolithic SaaS deploys with maintenance windows). His SVP-CS role requires him to translate engineering-speak into commercial-speak for procurement officers + IT directors at customer companies. During the Saturday SEV-2 he sent 4 SITREP emails to Penske + 3 to Walmart-Sam's + 2 to Amazon-Robotics + 8 to smaller customers, each personalized with the cohort context + their specific impact + the mitigation status. By Sunday 09:00 CDT his Inbox was 47 emails deep but all were customer-trust-restoration replies, not complaints.
- Yamilet Solís kept a paper notebook (Moleskine cahier journal, navy cover, dot-grid, A5 size) on her desk throughout the cutover. She writes engineering decision rationale in pencil (a Caran d'Ache Fixpencil 884) in Spanish + English-mixed shorthand. The notebook entries for Oct 20–30 fill 18 pages. She'll transcribe the key entries into Confluence next week as part of the post-mortem.
- Akira Watanabe — the COO, who shows up again in j168 with her quarterly ops review — barely spoke during the cutover except at gate votes. She is by nature undertaking-quiet: she watches metrics, votes when it's her turn, doesn't intervene. She's been COO since 2022 (joined from Sony Honda Mobility); her view is that the CTO and VP-Eng run the engineering work and her job is to vote responsibly + own the cost-and-customer-impact side. On Friday Oct 30 evening she sent Diego a single Slack DM: "Buen trabajo." He replied: "Gracias Akira."
- Diego's father — **Don Reynaldo Vargas Cantú**, 76, retired PEMEX refinery process-engineer in Tampico (Tamaulipas) — called him on Friday Oct 30 at 19:42 CDT after he saw Aurelia mentioned on the Bloomberg-Línea-México evening segment. The old man speaks in slow northern-Tamaulipas Spanish. He asked: "Diego, hijo, tu compañía aparece en las noticias hoy. ¿Algo bueno?" Diego said yes, papá, something good — a big upgrade, finished today. His father asked: "¿Tomaste un día de descanso después?" Diego said no, papá, but next week. His father said: "Tómalo. Tu mamá quiere verte. Vente a Tampico." Diego said yes. They hung up.
- The night-cleaning crew at Torre Manacar floor 14 — three cleaners from **Limpieza Profesional CDMX** (a small subcontractor of the Torre Manacar facilities-management firm, with tenant `limpieza-profesional-cdmx-sa-de-cv`; the cross-journey reference to j160's Tomáš Horák kind of company, just in a CDMX context) — saw Diego in his office on Saturday night and Sunday morning. The lead cleaner **doña Refugio Pérez**, 58, who has cleaned this floor 3 nights a week since 2018, brought him a cup of café de olla from the floor's pantry at 23:18 Saturday without being asked. He thanked her in Spanish. She nodded and continued with the carpet.

## §12 — Stop condition for this story

This story documents the lived texture of the 10-day cutover. The acceptance criteria in `README.md`, the API shapes in `handshake.md`, the test cases in `integration-test-plan.md`, and the schema files together encode the machine semantics. The story exists so that the next human or agent reading the codepath can understand WHY the cohort-gate Cedar-permit quorum vote with TrueTime fence matters for IFRS-IAS-38 capitalization attestation, WHY the bytecode pre-warm ordering matters and how the runbook fix shipped without rolling back, WHY the SEV-2 rollback decision used the looser 3-of-4 quorum but ultimately voted no-rollback because the mitigation worked, WHY the v3.x hard sunset includes the legacy-system shutdown lifecycle with audit-chain Merkle root for SOC2 + EU-AI-Act-Art-17 attestation, and WHY a Mexican CTO running a publicly-listed industrial-robotics SaaS can land a major platform cutover with 30× fewer service-credits than the previous attempt because the substrate enforces the gate discipline mechanically.
