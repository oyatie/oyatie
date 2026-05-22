---
doc_class: User-Journey-Story
journey_id: j156-carlos-reyes-ii-maintenance-emergency-after-hours
date: 2026-05-20
authority_tier: 2
status: draft
---

# j156 — Story: 2:47 AM Phoenix, the chiller-loop alarm

## §0 — Cold open: Saturday October 17, 2026, 02:47 MST

The Sonoran summer has overstayed its welcome. Outside Carlos Reyes II's stucco house on West Augusta Avenue in Glendale, the asphalt is still radiating 91°F at 02:47 in the morning. The house is dark except for the green LED on the dishwasher and the soft amber pulse of the oyatie router in the kitchen. Yesenia is asleep on her side, the bedroom ceiling fan turning slow because they killed central air at 01:00 to save on the SRP peak-summer-overrun bill.

Carlos's **Samsung XCover7 Pro** — the rugged one with the dedicated red SOS button for after-hours pages — vibrates against the nightstand. It is a long, three-pulse vibration that he and Yesenia have lived with since 2019.

He picks it up.

> **🚨 P1-HVAC · DC-PHX-3**
> aisle 7B chiller-loop overtemp
> ΔT inlet/outlet **14.2°F** (cap 6.0°F)
> 4 racks at **88°F** intake
> auto-shed in **11 min 47 sec** if uncorrected
> respond **Y / N**

He swipes the green check.

His thumb hits the latent-print scanner on the side of the phone (he keeps the right index print enrolled because his left hand is the wrench hand and is usually dirty). The phone confirms his identity locally in 180 ms, then signs the acknowledgment with his passkey. A second screen flashes briefly: a small chip in the top-right says **`Cascade FM Services`** then flips to **`Cascade · MeridianStack (scoped)`**. Carlos has seen that flip dozens of times — it means the cross-tenant grant just came in. Saturdays are when this matters most.

He swings his legs out of bed. Yesenia mumbles "DC-PHX-3?" without opening her eyes. He says "sí, aisle 7B, voy a salir en cinco". She nods. She knows the routine. He'll be back by ten unless something is really wrong.

## §1 — 02:48–02:54 MST: Page acknowledgment + permit + cross-tenant grant

Carlos walks to the kitchen, pulls his keys off the hook by the back door, grabs the truck-go-bag from the bench. His phone vibrates again with a longer, calmer rhythm — Tomás Alvarado, Cascade's on-call manager. Tomás is in Mesa, about 28 miles away. The MLS-encrypted thread `dm-tomas-carlos-cascade-2026-10-17-0247` opens with the audit-seal indicator green.

**Tomás 02:49**: "carlos buen tiempo. priya en NOC ya despertó. permit incoming"
**Carlos 02:49**: "ya en truck. ETA 3:11"
**Tomás 02:50**: "voy a co-firmar. NFPA-70E cat-2 confirmed?"
**Carlos 02:50**: "sí, cert 2026-09-14"

By 02:51:14 MST Carlos's phone shows the permit drawer:

```
PERMIT-TO-WORK
permit-id: permit-dc-phx-3-2026-10-17-0251-7b
scope: aisle 7B · chiller-loop 7B-CHL-02 · pump 7B-PUMP-04 · 480V class
PPE: NFPA-70E Cat-2 (cotton FR coverall, hard-hat, arc-rated face shield, leather gloves)
EPA: 608-Universal required (refrigerant R-454B in loop)
LOTO: required before any 480V work
co-signers required: Cascade-manager + MeridianStack-NOC
valid: 2026-10-17T02:51 → 09:00 MST
```

Carlos scrolls. Tomás's signature appears at 02:51:42 — Tomás's passkey, his face-id, his timestamp, his Cascade title. Carlos has known Tomás since 2016 when they both worked Honeywell projects. The signature is mundane, mostly. The audit-event behind it is not. `EVT-J156-WORKFLOW-PERMIT-COSIGN-003a` seals in `cascade-fm-services-llc-us` AND the redacted twin in `meridianstack-hosting-co-us`, because under ADR-0244 every cross-tenant permit emits a dual-seal even on the cooperative path.

Priya Subramanian at MeridianStack NOC signs at 02:53:08 from her workstation in the Chandler control room. Priya is the lead controller on the Saturday graveyard. She speaks Tamil, Hindi, and English, holds a BS in Mechanical Engineering from Anna University, and has been at MeridianStack since 2023. Her signature triggers `EVT-J156-WORKFLOW-PERMIT-COSIGN-003b`.

Carlos is in his Ford F-150 by 02:54:11. The truck's CarPlay screen shows the same active-tenant pill: `Cascade · MeridianStack (scoped)`. The radio is off. The desert is quiet. He merges onto Loop 101 South.

## §2 — 03:11–03:19 MST: Arrival, badge-in, chiller-loop room entry

DC-PHX-3 is a low, beige tilt-up at the corner of 35th Avenue and Buckeye, surrounded by chain-link and three rows of LED parking lights. Carlos pulls into bay 7 and parks. He grabs his arc-flash bag — the bright orange one he bought himself in 2022 — and walks to the staff entrance.

The badge reader on the door reads his Cascade-tenant employee badge `cascade-emp-carlos-reyes-ii-2018-04-19`. Cedar evaluates: `permit (incident.acknowledge, badge.read_door)` against `dc-phx-3-staff-entrance`. The grant is active, the permit is co-signed, and `incident-management` shows the P1 still open with **8 min 11 sec** auto-shed countdown.

The door clicks. Carlos walks the hall to the mechanical room corridor. He passes the operations office and waves at Priya through the window; she's on a headset, three monitors lit, the chiller-loop telemetry graph showing the climbing ΔT on the leftmost panel. She gives him a thumbs-up.

At 03:16:42 he reaches **MECH-RM-07B** — the mechanical room serving aisle 7B. He scans his badge again. Cedar permit evaluates and grants. The audit `EVT-J156-PHYSICAL-ROOM-ENTRY-004a` seals in BOTH tenants.

The room is loud — chiller hum, pump whine, condensate drip. The air is heavy with the chemical sweetness of glycol and the bright sharpness of R-454B that shouldn't be there if the loop were tight. Carlos's nose flags it before any sensor. He has done this for 23 years.

He pulls his phone, opens the `tasks` µservice. 11 atomic tasks are stacked in order:

1. drive-to-site ✓ (auto-completed by GPS)
2. badge-in ✓ (auto-completed by reader)
3. ladder-setup (next)
4. lockout-tagout
5. condensate-line-inspection
6. pump-rebuild
7. refrigerant-recovery
8. post-leak-test
9. re-energize
10. log-in-CMMS
11. sign-permit-closeout

He taps task #3.

## §3 — 03:19–03:34 MST: LOTO, condensate inspection, the leak

Carlos sets up his fiberglass folding 8-ft ladder under the pump. The ladder is non-conductive — important for the 480V proximity. He photographs the ladder placement and uploads. `tasks` records `task-j156-003-ladder-setup` with photo `photo-2026-10-17-031942-7b-ladder.heic`, GPS `33.4404,-112.1359`, and Cedar context.

He moves to task #4: lockout-tagout. The `workflow-engine` shows the LOTO state machine in state `lockout_pending`. The pump's electrical disconnect is panel `DC-PHX-3-PNL-7B-04` rated 480V/3-phase/60Hz. Carlos:

- Notifies Priya via messenger ("LOTO commencing 7B-PUMP-04 panel pnl-7b-04, please confirm load isolated")
- Priya responds: "load shed to chiller 7B-CHL-01 fallback; you are isolated. proceed."
- Carlos opens the disconnect to OFF
- Applies his personal red Master Lock 410 with his name etched on it (he has had this lock since his Honeywell days)
- Tags it with a yellow danger tag pre-printed `CARLOS REYES II · CASCADE FM · 2026-10-17 · DO NOT OPERATE`
- Tests for absence of voltage with his Fluke T6-1000 across all three phases — **0.0V**, **0.0V**, **0.0V**
- Photographs each step

The LOTO state machine advances:
- `lockout_pending` → `disconnect_open` (03:21:18)
- `disconnect_open` → `personal_lock_applied` (03:21:47)
- `personal_lock_applied` → `tested_voltage_absent` (03:23:08)
- `tested_voltage_absent` → `locked_isolated_verified` (03:23:14)

Each transition seals an audit event. The dual-seal invariant requires events in BOTH tenants. They land in both. `EVT-J156-LOTO-LOCKED-004` is sealed and the merkle leaf is computed by 03:23:21.

Carlos moves on. He inspects the condensate line per task #5. The line is clear, no biofilm, no algae. The drip-pan is wet but not flooded.

He moves to task #6: pump rebuild. He pulls the pump-housing access cover. The shaft seal is wet with glycol. He runs his finger along the bottom — sticky residue, slightly oily, and a faint cool feeling as the residual refrigerant evaporates from his skin. The shaft seal has failed and the loop has been weeping R-454B for at least 4 hours.

He photographs the seal, uploads, and types into messenger: "shaft seal failed. refrigerant cross-leak. estimating 1.4 lb R-454B released to atmosphere. need EPA-608 disclosure path."

Priya's response at 03:34:42: "ack. opening EPA workflow now."

## §4 — 03:35–04:11 MST: EPA-608 disclosure workflow

The `workflow-engine` materializes a new workflow `wkfl-epa608-release-disclosure-dc-phx-3-2026-10-17`. It is gated by Cedar permit `workflow.epa608_disclose_release` (Carlos has the cert, the permit is active, the loop is identified). The workflow has 6 steps:

1. Identify refrigerant + cylinder of origin
2. Estimate release quantity (in lb, gross + net)
3. Document leak location + cause + first-discovery time
4. Notify EPA-608 Class IV refrigerant vendor (Trane Technologies' factory line for R-454B is `1-800-872-6377` ext factory-emergency)
5. Photograph the leak site + the labeled refrigerant cylinder + the recovery unit
6. File the disclosure form `40-CFR-82-F-disclosure-2026-10-17-dc-phx-3-7b.json` to EPA's E-GGRT submission portal

Carlos identifies the cylinder: refrigerant came from cylinder `R454B-CYL-DC-PHX-3-2026-Q3-007` which was charged to this loop on 2026-07-22 by Cascade tech Marcus Whitfield. Carlos pulls the cylinder ID via NFC tap on the label. The `plant-maintenance` µservice has the full provenance: vendor (Honeywell), cylinder weight at charge (52.0 lb), cylinder weight at last weigh (51.9 lb), in-loop estimated charge (51.4 lb).

He estimates the release at 1.4 lb based on the wet seal area and the loop pressure drop telemetry Priya reads off her panel. He photographs the cylinder label, the leak site, and his Yellow Jacket recovery unit (model 95760) plugged into the high-side service port.

At 03:42:18 he hits "submit". The `workflow-engine` packages the disclosure form, signs it with his EPA-608-Universal cert + Cascade tenant attestation + MeridianStack site attestation, and POSTs to the EPA E-GGRT endpoint via the `compliance` µservice. The receipt comes back at 03:43:11 with `egrt-receipt-2026-10-17-dc-phx-3-001`.

`EVT-J156-EPA608-DISCLOSURE-006` seals in BOTH tenants and in the `compliance` µservice's regulator-anchor ledger.

## §5 — 04:11–06:48 MST: Recovery, rebuild, re-energize

The recovery itself takes 2 hours 37 minutes:

- 04:11–04:46: Refrigerant recovery into a fresh DOT-39 cylinder. Final weight: 50.0 lb in, 51.4 lb expected — confirming the 1.4 lb release estimate.
- 04:46–05:32: Pump rebuild. Carlos replaces the failed shaft seal (Trane part `KIT-RTAF-200-SHAFT-SEAL-V3`) which he pulls from his truck's emergency stock. He photographs the new seal, the torque sequence (he uses his torque wrench at 38 ft-lb in two passes), and the gasket.
- 05:32–05:51: Post-leak test. He pressurizes the loop with dry nitrogen to 250 psi, soap-tests every joint, and watches the pressure decay for 14 minutes. Zero decay.
- 05:51–06:18: Refrigerant recharge from a fresh cylinder. New cylinder ID `R454B-CYL-CASCADE-TRUCK-2026-Q4-019`. He charges to 51.0 lb (the loop's nameplate charge minus a small post-leak adjustment).
- 06:18–06:32: Visual + thermal-camera inspection of the loop. He uses his FLIR ONE Pro to scan for any temperature anomaly. Clean.
- 06:32–06:48: Re-energize. The LOTO state machine reverses: `locked_isolated_verified` → `personal_lock_removed` → `disconnect_closed` → `energized_normal`. Each transition is photographed, signed, and sealed.

Priya watches the chiller-loop telemetry. The ΔT walks down from 14.2°F to 11.0°F to 8.0°F to 6.4°F to 5.8°F by 06:47:11 MST. She types: "loop stable. ΔT in spec. great work carlos."

## §6 — 06:48–09:05 MST: CMMS log, post-mortem, sign-off, grant expiration

Carlos opens task #10: log-in-CMMS. The `plant-maintenance` µservice generates a work order `WO-DC-PHX-3-2026-10-17-7B-PUMP-04-SHAFT-SEAL` with all 11 tasks linked, all photos attached, all timestamps preserved. Carlos adds a free-text note in English (Priya reads English; Tomás reads both) and a parallel note in Spanish for the Cascade closeout:

> "EN: 7B-PUMP-04 Trane RTAF-200 shaft seal failure caused 1.4 lb R-454B release over ~4 hrs. Recovered, rebuilt, recharged, tested. Loop ΔT recovered to 5.8°F by 06:47 MST. EPA-608 disclosure egrt-receipt-2026-10-17-dc-phx-3-001 filed."
>
> "ES: Sello del eje 7B-PUMP-04 falló. Liberación 1.4 lb R-454B. Recuperado, reconstruido, recargado, probado. Loop ΔT 5.8°F a 06:47 MST. Disclosure EPA-608 archivada."

He submits at 06:53:14. `EVT-J156-CMMS-WORKORDER-CLOSED-010` seals.

Task #11: sign-permit-closeout. The `workflow-engine` shows the permit lifecycle:

- 02:51:14 — created
- 02:53:08 — co-signed
- 03:23:14 — LOTO locked
- 06:47:11 — work complete
- 06:53:18 — closeout signed (Carlos)
- 06:55:42 — closeout co-signed (Tomás)
- 06:57:18 — closeout co-signed (Priya)

At 06:57:21 the permit transitions to `closed_post_verification`. The `audit-chain` seals the terminal event with the merkle root of every action: 87 events total over 4 hours 10 minutes of on-site work.

A small celebration message from Tomás arrives: "carlos buen trabajo. ve a desayunar. WO closeout perfecto."

Carlos packs his bag, returns the truck-bench tools, badges out of MECH-RM-07B, badges out of the building, and is at his truck by 07:14 MST.

He drives east on I-10 toward Glendale. He stops at a Filiberto's on 67th Avenue for chorizo and eggs. Yesenia texts at 08:11: "todo bien?" He replies: "todo bien. ETA 9:30."

At **09:00:00 MST exactly**, Carlos's cross-tenant grant `cross-grant-cascade-meridianstack-2026-10-17-carlos-0247` auto-expires. The Cedar policy now denies any action by Carlos against the MeridianStack tenant. The `identity` µservice records `EVT-J156-CROSS-GRANT-EXPIRED-008` in BOTH tenants. No further work is possible without a new grant — but no further work is needed.

At 09:05:18 MST Carlos's Cascade payroll posts the emergency-call hours: 6 hours 18 minutes at $1,247 base + $87.50/hr after the 2-hour minimum. Yesenia gets the SMS notification from Cascade payroll. She smiles. The bill is paid.

## §7 — Post-mortem: 10:14 MST, MeridianStack ops review

Priya hands off to the Saturday-day controller at 09:00 MST. At 10:14 MST the MeridianStack ops review meeting begins; Priya joins by video. She walks the chiller-loop telemetry from the alarm to the recovery curve and pulls the `incident-management` post-mortem template. The template auto-populates from the audit-chain: 87 events, all photos, the EPA-608 disclosure receipt, the permit lifecycle, Carlos's 11 tasks, and the cross-tenant grant timeline.

The post-mortem closes with three actions:

1. Schedule Trane factory inspection of all 12 RTAF-200 pumps in DC-PHX-3 (preventive — same shaft seal vintage)
2. Add a continuous low-side pressure-decay alarm to the chiller-loop telemetry (early-warning for slow leaks)
3. Update the Cascade emergency-response runbook for R-454B releases (Carlos's notes were excellent; turn them into a checklist)

`EVT-J156-POSTMORTEM-CLOSED-013` seals at 10:42:18 MST.

The HIPAA daily-roll-up runs at midnight (00:00:00 MST Oct 18) and produces a merkle-anchored facility-control audit summary that MeridianStack's covered-entity customer (the healthcare-integration µservice tenant) consumes the next morning. The summary shows: 1 P1 incident, 87 audit events, 1 EPA-608 disclosure filed, 0 PII data plane disruption, 100% telemetry continuity through the response.

## §8 — Beats not on the wire (the human texture)

- At 04:46 MST, Carlos asked Priya over messenger whether NOC could spare a coffee. Priya walked one down at 04:51 in a black mug with the MeridianStack logo. The mug is small but the gesture is large.
- At 05:32 MST, when the loop pressure-tested clean, Carlos exhaled a breath he had not realized he was holding. His phone's heart-rate sensor (which is correlated to the optional wellness telemetry — but he opted out years ago) would have recorded a 14-bpm drop.
- At 06:52 MST, Carlos found a small spider in the panel he was wiring back up. He gently removed it to the corner of the room before continuing. None of this is in the audit-chain. It is not nothing.
- At 09:14 MST in the Filiberto's parking lot, Carlos called his father in Yuma. His father is 71, retired, and was the one who taught Carlos to coil hoses correctly when he was 14. The call lasted 6 minutes. They talked about the weather. The chorizo was good.

## §9 — Stop condition for this story

This story documents the lived texture of the 6h18m journey. The acceptance criteria in `README.md`, the API shapes in `handshake.md`, the test cases in `integration-test-plan.md`, and the schema files together encode the precise machine semantics. The story exists to make the machine semantics legible to the next human or agent who walks through this code path.
